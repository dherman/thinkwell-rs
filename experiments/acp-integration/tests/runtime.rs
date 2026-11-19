use agent_client_protocol::AcpAgentConfig;
use schemars::JsonSchema;
use serde::Deserialize;
use std::{cell::Cell, path::Path, rc::Rc, time::Duration};
use thinkwell_acp_experiment::{
    result_contract,
    runtime::{Agent, Session},
};

#[derive(Debug, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
struct Summary {
    total: i64,
}
#[derive(Debug, Deserialize, JsonSchema, PartialEq)]
#[serde(tag = "kind", deny_unknown_fields)]
enum Decision {
    Accept { total: i64 },
    Reject { reason: String },
}

async fn connect(mode: &str, dir: &Path) -> Session {
    Agent::connect(
        AcpAgentConfig::new("python3").args([
            format!("{}/tests/peer.py", env!("CARGO_MANIFEST_DIR")),
            mode.to_string(),
            dir.join("pid").display().to_string(),
        ]),
        None,
    )
    .await
    .unwrap()
    .open()
}

async fn wait_file(path: &Path) {
    tokio::time::timeout(Duration::from_secs(5), async {
        while !path.exists() {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
}

// ps absence observes exit and removal from the process table, rather than only
// recording a signal request. The SDK's async-process reaper may finish after Drop.
async fn reaped(dir: &Path) {
    let pid = std::fs::read_to_string(dir.join("pid")).unwrap();
    tokio::time::timeout(Duration::from_secs(8), async {
        loop {
            let status = std::process::Command::new("ps")
                .args(["-p", pid.trim(), "-o", "stat="])
                .output()
                .unwrap();
            if status.stdout.is_empty() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn changed_types_and_local_non_send_borrow() {
    tokio::time::timeout(Duration::from_secs(15), async {
        let dir = tempfile::tempdir().unwrap();
        let mut session = connect("success", dir.path()).await;
        let total = Rc::new(Cell::new(0));
        let mut inputs = vec![];
        let result: Summary = session
            .run("first", async |v| {
                inputs.push(v);
                tokio::task::yield_now().await;
                total.set(total.get() + v);
                Ok(total.get())
            })
            .await
            .unwrap();
        assert_eq!(result.total, 3);
        inputs.push(0);
        total.set(total.get() * 2);
        let result: Decision = session
            .run("second", async |v| {
                inputs.push(v);
                tokio::task::yield_now().await;
                total.set(total.get() + v);
                Ok(total.get())
            })
            .await
            .unwrap();
        assert_eq!(result, Decision::Accept { total: 10 });
        assert_eq!(inputs, [3, 0, 4]);
        session.shutdown().await.unwrap();
        reaped(dir.path()).await;
        assert!(dir.path().join("pid.eof").exists());
        assert_eq!(
            std::fs::read_to_string(dir.path().join("pid.prompts")).unwrap(),
            "2"
        );
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn result_failure_matrix_uses_real_protocol_driver() {
    for (mode, success) in [
        ("malformed", true),
        ("correct", true),
        ("stale", true),
        ("duplicate", true),
        ("invalid", false),
        ("missing", false),
        ("cancelled", false),
    ] {
        tokio::time::timeout(Duration::from_secs(15), async {
            let dir = tempfile::tempdir().unwrap();
            let mut session = connect(mode, dir.path()).await;
            let result = session
                .run::<Summary>("fixture", async |_| panic!("no application call expected"))
                .await;
            assert_eq!(result.is_ok(), success, "{mode}: {result:?}");
            if success {
                assert_eq!(result.unwrap().total, 3);
            } else {
                let expected = match mode {
                    "invalid" => "correction budget exhausted",
                    "missing" => "normal completion without result",
                    "cancelled" => "remote stop",
                    _ => unreachable!(),
                };
                assert!(result.unwrap_err().contains(expected));
                assert!(!session.usable());
                assert!(
                    session
                        .run::<Summary>("must not send", async |_| Ok(0))
                        .await
                        .is_err()
                );
                assert_eq!(
                    std::fs::read_to_string(dir.path().join("pid.prompts")).unwrap(),
                    "1"
                );
            }
            session.shutdown().await.unwrap();
            reaped(dir.path()).await;
        })
        .await
        .unwrap();
    }
}

#[tokio::test]
async fn candidate_is_not_completion_and_drop_releases_borrow() {
    let dir = tempfile::tempdir().unwrap();
    let mut session = connect("hold", dir.path()).await;
    let mut local = 0;
    {
        let execution = session.run::<Summary>("fixture", async |v| {
            local += v;
            Ok(local)
        });
        tokio::pin!(execution);
        let candidate_path = dir.path().join("pid.candidate");
        tokio::select! {
            r = &mut execution => panic!("candidate returned early: {r:?}"),
            _ = wait_file(&candidate_path) => {},
        }
    }
    local += 1;
    assert_eq!(local, 1);
    assert!(!session.usable());
    assert!(
        session
            .run::<Summary>("must not send", async |_| Ok(0))
            .await
            .is_err()
    );
    drop(session);
    reaped(dir.path()).await;
}

#[tokio::test]
async fn drop_inside_callback_revokes_local_future() {
    let dir = tempfile::tempdir().unwrap();
    let mut session = connect("callback", dir.path()).await;
    let mut local = 0;
    let started = Rc::new(Cell::new(false));
    {
        let execution = session.run::<Summary>("fixture", async |v| {
            local += v;
            started.set(true);
            std::future::pending::<()>().await;
            Ok(local)
        });
        tokio::pin!(execution);
        tokio::time::timeout(Duration::from_secs(5), async {
            tokio::select! {
                r = &mut execution => panic!("unexpected {r:?}"),
                _ = async { while !started.get() { tokio::task::yield_now().await; } } => {}
            }
        })
        .await
        .unwrap();
    }
    local += 4;
    assert_eq!(local, 7); // Cancellation did not roll back already executed effects.
    drop(session);
    reaped(dir.path()).await;
}

#[tokio::test]
async fn graceful_forced_and_error_shutdown() {
    for mode in ["success", "forced", "exit_error"] {
        let dir = tempfile::tempdir().unwrap();
        let session = connect(mode, dir.path()).await;
        let diagnostics = session.diagnostics();
        let result = session.shutdown().await;
        assert_eq!(result.is_err(), mode == "exit_error", "{mode}: {result:?}");
        assert_eq!(diagnostics.outcome(), Some(result));
        reaped(dir.path()).await;
        assert!(dir.path().join("pid.eof").exists());
        if mode == "forced" {
            assert!(dir.path().join("pid.term").exists());
        }
    }
}

#[tokio::test]
async fn early_return_and_dropped_session_report_cleanup() {
    let dir = tempfile::tempdir().unwrap();
    let session = connect("exit_error", dir.path()).await;
    let diagnostics = session.diagnostics();
    async fn early(session: Session) -> Result<(), &'static str> {
        let _session = session;
        Err("application error")
    }
    assert!(early(session).await.is_err());
    reaped(dir.path()).await;
    tokio::time::timeout(Duration::from_secs(3), async {
        while diagnostics.outcome().is_none() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert!(diagnostics.outcome().unwrap().is_err());
}

#[test]
fn runtime_teardown_kills_and_reaps_owned_child() {
    let dir = tempfile::tempdir().unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let session = runtime.block_on(connect("forced", dir.path()));
    drop(runtime); // Cancels driver without its async graceful path.
    drop(session);
    let observer = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    observer.block_on(reaped(dir.path()));
}

#[test]
fn finite_schema_contract_rejects_unknown_constraints_and_checks_values() {
    use serde_json::json;
    for schema in [
        json!({"$ref":"#/$defs/X"}),
        json!({"type":"string","minLength":2}),
        json!({"type":"string","format":"email"}),
    ] {
        assert!(result_contract::supported(&schema).is_err());
    }
    let schema = result_contract::contract::<Summary>().unwrap();
    for value in [
        json!({}),
        json!({"total":"3"}),
        json!({"total":3,"extra":true}),
        json!({"total":3.5}),
    ] {
        assert!(result_contract::decode::<Summary>(&schema, value).is_err());
    }
    assert!(result_contract::contract::<Decision>().is_ok());
}

#[tokio::test]
async fn missing_envelope_value_is_not_a_null_result() {
    let dir = tempfile::tempdir().unwrap();
    let mut session = connect("null_missing", dir.path()).await;
    session.run::<()>("fixture", async |_| Ok(0)).await.unwrap();
    session.shutdown().await.unwrap();
    reaped(dir.path()).await;
}

#[tokio::test]
async fn callback_error_is_visible_to_peer_and_can_be_handled() {
    let dir = tempfile::tempdir().unwrap();
    let mut session = connect("callback_error", dir.path()).await;
    let result = session
        .run::<Summary>("fixture", async |_| Err("application failure".into()))
        .await
        .unwrap();
    assert_eq!(result.total, 3);
    session.shutdown().await.unwrap();
    reaped(dir.path()).await;
}

#[tokio::test]
async fn dropped_agent_closes_process_and_listener() {
    let dir = tempfile::tempdir().unwrap();
    let agent = Agent::connect(
        AcpAgentConfig::new("python3").args([
            format!("{}/tests/peer.py", env!("CARGO_MANIFEST_DIR")),
            "success".into(),
            dir.path().join("pid").display().to_string(),
        ]),
        None,
    )
    .await
    .unwrap();
    let url = std::fs::read_to_string(dir.path().join("pid.url")).unwrap();
    let address = url
        .strip_prefix("http://")
        .unwrap()
        .strip_suffix("/mcp")
        .unwrap();
    assert!(tokio::net::TcpStream::connect(address).await.is_ok());
    drop(agent);
    reaped(dir.path()).await;
    tokio::time::timeout(Duration::from_secs(3), async {
        while tokio::net::TcpStream::connect(address).await.is_ok() {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
}

#[test]
fn nested_and_tagged_schema_validation() {
    use serde_json::json;
    let schema = result_contract::contract::<Decision>().unwrap();
    for invalid in [
        json!({"kind":"Other","total":3}),
        json!({"kind":"Accept","total":"3"}),
        json!({"kind":"Reject"}),
    ] {
        assert!(result_contract::decode::<Decision>(&schema, invalid).is_err());
    }
    assert_eq!(
        result_contract::decode::<Decision>(&schema, json!({"kind":"Reject","reason":"no"}))
            .unwrap(),
        Decision::Reject {
            reason: "no".into()
        }
    );
    let schema = json!({"type":"array","items":{"type":"boolean"}});
    result_contract::supported(&schema).unwrap();
    assert!(result_contract::validate(&schema, &json!([true, false])).is_ok());
    assert!(result_contract::validate(&schema, &json!([true, 0])).is_err());
}

#[tokio::test]
async fn late_application_calls_cannot_enter_next_turn_callback() {
    tokio::time::timeout(Duration::from_secs(15), async {
        let dir = tempfile::tempdir().unwrap();
        let mut session = connect("late", dir.path()).await;
        let mut first = vec![];
        let a: Summary = session
            .run("A", async |v| {
                first.push(v);
                Ok(v)
            })
            .await
            .unwrap();
        assert_eq!(a.total, 3);
        assert_eq!(first, [3]);
        let mut second = vec![];
        let b: Decision = session
            .run("B", async |v| {
                second.push(v);
                Ok(v)
            })
            .await
            .unwrap();
        assert_eq!(b, Decision::Accept { total: 4 });
        assert_eq!(first, [3]);
        assert_eq!(second, [4]);
        let calls = std::fs::read_to_string(dir.path().join("pid.calls")).unwrap();
        let rejected = calls
            .lines()
            .filter(|line| {
                serde_json::from_str::<serde_json::Value>(line).unwrap()["isError"] == true
            })
            .count();
        assert_eq!(rejected, 6);
        session.shutdown().await.unwrap();
        reaped(dir.path()).await;
    })
    .await
    .unwrap();
}
