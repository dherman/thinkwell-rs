use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{cell::Cell, path::Path, rc::Rc, time::Duration};
use thinkwell::{
    Agent, AgentConfig, Connect, Error, ErrorKind, Plan, RunOptions, Tool, agent::Claude,
};

#[derive(Deserialize, JsonSchema)]
struct Input {
    value: i64,
}
#[derive(Serialize, JsonSchema)]
struct Output {
    total: i64,
}
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
fn tool() -> Tool<Input, Output> {
    Tool::new("accumulate", "Add to accumulator")
}
fn config(mode: &str, dir: &Path) -> AgentConfig {
    AgentConfig::new("python3").args([
        format!("{}/tests/peer.py", env!("CARGO_MANIFEST_DIR")),
        mode.into(),
        dir.join("pid").display().to_string(),
    ])
}
async fn connect(mode: &str, dir: &Path) -> Agent {
    Agent::connect(config(mode, dir)).await.unwrap()
}
async fn reaped(dir: &Path) {
    let pid = std::fs::read_to_string(dir.join("pid")).unwrap();
    tokio::time::timeout(Duration::from_secs(8), async {
        loop {
            let output = std::process::Command::new("ps")
                .args(["-p", pid.trim(), "-o", "stat="])
                .output()
                .unwrap();
            if output.stdout.is_empty() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap();
}
async fn conversation(mode: &str) {
    let dir = tempfile::tempdir().unwrap();
    let agent = connect(mode, dir.path()).await;
    let tool = tool();
    let mut session = agent.session().tool(&tool).open().await.unwrap();
    let total = Rc::new(Cell::new(0));
    let mut inputs = vec![];
    let base = Plan::<Summary>::new().text("first");
    let a = base
        .clone()
        .bind(&tool, async |v: Input| {
            inputs.push(v.value);
            tokio::task::yield_now().await;
            total.set(total.get() + v.value);
            Ok(Output { total: total.get() })
        })
        .run_in(&mut session)
        .await
        .unwrap();
    assert_eq!(a.total, 3);
    total.set(total.get() * 2);
    inputs.push(0);
    let b = Plan::<Decision>::new()
        .text("second")
        .bind(&tool, async |v: Input| {
            inputs.push(v.value);
            tokio::task::yield_now().await;
            total.set(total.get() + v.value);
            Ok(Output { total: total.get() })
        })
        .run_in(&mut session)
        .await
        .unwrap();
    assert_eq!(b, Decision::Accept { total: 10 });
    assert_eq!(inputs, [3, 0, 4]);
    if mode == "late" {
        let calls = std::fs::read_to_string(dir.path().join("pid.calls")).unwrap();
        assert_eq!(
            calls
                .lines()
                .filter(
                    |line| serde_json::from_str::<serde_json::Value>(line).unwrap()["isError"]
                        == true
                )
                .count(),
            6
        );
    }
    drop(session);
    agent.shutdown().await.unwrap();
    reaped(dir.path()).await;
}
#[tokio::test]
async fn typed_turns_and_stale_calls() {
    tokio::time::timeout(Duration::from_secs(20), conversation("late"))
        .await
        .unwrap();
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn local_borrows_on_multithread_runtime() {
    tokio::time::timeout(Duration::from_secs(20), conversation("success"))
        .await
        .unwrap();
}

#[tokio::test]
async fn failures_and_corrections() {
    for (mode, expected) in [
        ("correct", None),
        ("malformed", None),
        ("duplicate", None),
        ("stale", None),
        ("invalid", Some(ErrorKind::ResultBudget)),
        ("missing", Some(ErrorKind::MissingResult)),
        ("cancelled", Some(ErrorKind::Cancelled)),
        ("transport", Some(ErrorKind::Transport)),
        ("deadline", Some(ErrorKind::Deadline)),
        ("call_budget", Some(ErrorKind::CallBudget)),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let agent = connect(mode, dir.path()).await;
        let tool = tool();
        let mut session = agent.session().tool(&tool).open().await.unwrap();
        let options = RunOptions {
            turn_timeout: Duration::from_millis(500),
            max_tool_calls: 2,
            ..Default::default()
        };
        let r = Plan::<Summary>::new()
            .options(options)
            .bind(&tool, async |v: Input| Ok(Output { total: v.value }))
            .run_in(&mut session)
            .await;
        match expected {
            None => assert_eq!(r.unwrap().total, 3, "{mode}"),
            Some(kind) => {
                assert_eq!(r.unwrap_err().kind, kind, "{mode}");
                assert!(!session.usable());
                assert_eq!(
                    Plan::<Summary>::new()
                        .run_in(&mut session)
                        .await
                        .unwrap_err()
                        .kind,
                    ErrorKind::Closed
                );
                assert_eq!(
                    std::fs::read_to_string(dir.path().join("pid.prompts")).unwrap(),
                    "1"
                );
            }
        }
        drop(session);
        let cleanup = agent.shutdown().await;
        if mode != "transport" {
            cleanup.unwrap();
        }
        reaped(dir.path()).await;
    }
}
#[tokio::test]
async fn preflight_is_local_and_session_reservation_reusable() {
    let dir = tempfile::tempdir().unwrap();
    struct ExternalPreset(AgentConfig);
    impl From<ExternalPreset> for AgentConfig {
        fn from(p: ExternalPreset) -> Self {
            p.0
        }
    }
    let agent = ExternalPreset(config("sessions", dir.path()))
        .connect()
        .await
        .unwrap();
    let tool = tool();
    assert!(
        agent
            .session()
            .tool(&tool)
            .tool(&tool)
            .open()
            .await
            .is_err()
    );
    assert!(
        agent
            .session()
            .tool(&Tool::<Input, Output>::new("return_result", "reserved"))
            .open()
            .await
            .is_err()
    );
    let mut session = agent.session().tool(&tool).open().await.unwrap();
    assert!(matches!(
        agent.session().open().await,
        Err(Error {
            kind: ErrorKind::Busy,
            ..
        })
    ));
    assert_eq!(
        Plan::<Summary>::new()
            .run_in(&mut session)
            .await
            .unwrap_err()
            .kind,
        ErrorKind::Binding
    );
    let different = Tool::<Input, Output>::new("accumulate", "different descriptor");
    assert_eq!(
        Plan::<Summary>::new()
            .bind(&different, async |v: Input| Ok(Output { total: v.value }))
            .run_in(&mut session)
            .await
            .unwrap_err()
            .kind,
        ErrorKind::Binding
    );
    assert!(!dir.path().join("pid.prompts").exists());
    // Dropping without polling has no lifecycle effect.
    drop(
        Plan::<Summary>::new()
            .bind(&tool, async |v: Input| Ok(Output { total: v.value }))
            .run_in(&mut session),
    );
    assert!(session.usable());
    drop(session);
    let next = agent.session().tool(&tool).open().await.unwrap();
    let tokens = std::fs::read_to_string(dir.path().join("pid.tokens")).unwrap();
    let tokens: Vec<_> = tokens.lines().collect();
    assert_eq!(tokens.len(), 2);
    assert_ne!(tokens[0], tokens[1]);
    drop(next);
    agent.shutdown().await.unwrap();
    reaped(dir.path()).await;
}
#[tokio::test]
async fn dropping_started_execution_releases_borrow_and_closes_connection() {
    let dir = tempfile::tempdir().unwrap();
    let agent = connect("callback", dir.path()).await;
    let tool = tool();
    let mut session = agent.session().tool(&tool).open().await.unwrap();
    let started = Rc::new(Cell::new(false));
    let mut total = 0;
    {
        let run = Plan::<Summary>::new()
            .bind(&tool, async |v: Input| {
                total += v.value;
                started.set(true);
                std::future::pending::<()>().await;
                Ok(Output { total })
            })
            .run_in(&mut session);
        tokio::pin!(run);
        tokio::time::timeout(Duration::from_secs(5), async {
            tokio::select! {
                r=&mut run=>panic!("unexpected completion {r:?}"),
                _=async {while !started.get(){tokio::task::yield_now().await;}}=>{},
            }
        })
        .await
        .unwrap();
    }
    total += 4;
    assert_eq!(total, 7);
    assert!(!session.usable());
    drop(session);
    assert!(matches!(
        agent.session().open().await,
        Err(Error {
            kind: ErrorKind::Closed,
            ..
        })
    ));
    drop(agent);
    reaped(dir.path()).await;
}
#[tokio::test]
async fn candidate_does_not_complete_before_terminal() {
    let dir = tempfile::tempdir().unwrap();
    let agent = connect("hold", dir.path()).await;
    let mut s = agent.session().open().await.unwrap();
    {
        let run = Plan::<Summary>::new().run_in(&mut s);
        tokio::pin!(run);
        tokio::select! {
            r=&mut run=>panic!("candidate returned early {r:?}"),
            observed=tokio::time::timeout(Duration::from_secs(5),async {
                while !dir.path().join("pid.candidate").exists(){tokio::time::sleep(Duration::from_millis(10)).await;}
            })=>{observed.unwrap();},
        }
    }
    assert!(!s.usable());
    drop(s);
    drop(agent);
    reaped(dir.path()).await;
}
#[tokio::test]
async fn tool_errors_are_visible_and_recoverable() {
    for mode in ["input_error", "callback_error"] {
        let dir = tempfile::tempdir().unwrap();
        let agent = connect(mode, dir.path()).await;
        let tool = tool();
        let mut s = agent.session().tool(&tool).open().await.unwrap();
        let mut called = false;
        let r = Plan::<Summary>::new()
            .bind(&tool, async |_v: Input| {
                called = true;
                Err(Error::tool("application failure"))
            })
            .run_in(&mut s)
            .await
            .unwrap();
        assert_eq!(r.total, 3);
        assert_eq!(called, mode == "callback_error");
        drop(s);
        agent.shutdown().await.unwrap();
        reaped(dir.path()).await;
    }
}
#[tokio::test]
async fn shutdown_errors_force_fallback_and_partial_setup() {
    for mode in ["success", "forced", "exit_error", "setup_failure"] {
        let dir = tempfile::tempdir().unwrap();
        let agent = Claude::configured(config(mode, dir.path()))
            .connect()
            .await
            .unwrap();
        let diagnostics = agent.diagnostics();
        let s = agent.session().open().await;
        if mode == "setup_failure" {
            assert!(s.is_err());
        } else {
            assert!(s.is_ok());
        }
        drop(s);
        let r = agent.shutdown().await;
        assert_eq!(r.is_err(), mode == "exit_error", "{mode}: {r:?}");
        assert_eq!(diagnostics.outcome(), Some(r));
        reaped(dir.path()).await;
        if mode == "forced" {
            assert!(dir.path().join("pid.term").exists());
        }
    }
}
#[tokio::test]
async fn unsupported_model_is_typed() {
    let dir = tempfile::tempdir().unwrap();
    let agent = connect("success", dir.path()).await;
    assert!(matches!(
        agent.session().model("unknown").open().await,
        Err(Error {
            kind: ErrorKind::UnsupportedModel,
            ..
        })
    ));
    drop(agent);
    reaped(dir.path()).await;
}
#[test]
fn runtime_teardown_reaps_child() {
    let dir = tempfile::tempdir().unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let agent = runtime.block_on(connect("forced", dir.path()));
    runtime.block_on(async {
        drop(agent.session().open().await.unwrap());
    });
    drop(runtime);
    drop(agent);
    let observer = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    observer.block_on(reaped(dir.path()));
}

#[tokio::test]
async fn candidate_is_discarded_when_terminal_overlaps_callback() {
    let dir = tempfile::tempdir().unwrap();
    let agent = connect("overlap", dir.path()).await;
    let tool = tool();
    let mut session = agent.session().tool(&tool).open().await.unwrap();
    let path = dir.path().join("pid.callback-started");
    let r = Plan::<Summary>::new()
        .bind(&tool, async |_v: Input| {
            std::fs::write(&path, "started").unwrap();
            std::future::pending::<()>().await;
            Ok(Output { total: 0 })
        })
        .run_in(&mut session)
        .await;
    assert_eq!(r.unwrap_err().kind, ErrorKind::CallbackOverlap);
    drop(session);
    drop(agent);
    reaped(dir.path()).await;
}

#[derive(JsonSchema)]
struct InvalidOutput {
    total: i64,
}
impl Serialize for InvalidOutput {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        // Deliberately contradict its derived schema.
        serializer.serialize_str(&self.total.to_string())
    }
}
#[tokio::test]
async fn http_limits_and_serialized_output_validation() {
    let dir = tempfile::tempdir().unwrap();
    let agent = connect("http_boundary", dir.path()).await;
    let tool = Tool::<Input, InvalidOutput>::new("accumulate", "invalid output fixture");
    let mut session = agent.session().tool(&tool).open().await.unwrap();
    let mut entries = 0;
    let r = Plan::<Summary>::new()
        .bind(&tool, async |v: Input| {
            entries += 1;
            Ok(InvalidOutput { total: v.value })
        })
        .run_in(&mut session)
        .await
        .unwrap();
    assert_eq!(r.total, 3);
    assert_eq!(entries, 1);
    drop(session);
    agent.shutdown().await.unwrap();
    reaped(dir.path()).await;
}
#[tokio::test]
async fn callback_deadline_is_a_tool_error() {
    let dir = tempfile::tempdir().unwrap();
    let agent = connect("callback_error", dir.path()).await;
    let tool = tool();
    let mut session = agent.session().tool(&tool).open().await.unwrap();
    let options = RunOptions {
        callback_timeout: Duration::from_millis(30),
        ..Default::default()
    };
    let r = Plan::<Summary>::new()
        .options(options)
        .bind(&tool, async |_v: Input| {
            std::future::pending::<()>().await;
            Ok(Output { total: 0 })
        })
        .run_in(&mut session)
        .await
        .unwrap();
    assert_eq!(r.total, 3);
    drop(session);
    agent.shutdown().await.unwrap();
    reaped(dir.path()).await;
}
#[tokio::test]
async fn drop_cleanup_reports_errors_and_closes_listener() {
    let dir = tempfile::tempdir().unwrap();
    let agent = connect("exit_error", dir.path()).await;
    let diagnostics = agent.diagnostics();
    let session = agent.session().open().await.unwrap();
    let url = std::fs::read_to_string(dir.path().join("pid.url")).unwrap();
    let address = url
        .strip_prefix("http://")
        .unwrap()
        .strip_suffix("/mcp")
        .unwrap();
    async fn application_error(agent: Agent) -> Result<(), Error> {
        let _agent = agent;
        Err(Error::tool("early return"))
    }
    drop(session);
    assert!(application_error(agent).await.is_err());
    reaped(dir.path()).await;
    tokio::time::timeout(Duration::from_secs(3), async {
        while diagnostics.outcome().is_none() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert!(diagnostics.outcome().unwrap().is_err());
    assert!(tokio::net::TcpStream::connect(address).await.is_err());
}
#[tokio::test]
async fn unsupported_contracts_and_options_fail_before_prompt() {
    #[derive(Deserialize, JsonSchema)]
    struct Referenced {
        _child: Summary,
    }
    let dir = tempfile::tempdir().unwrap();
    let agent = connect("missing", dir.path()).await;
    let mut session = agent.session().open().await.unwrap();
    let r = Plan::<Referenced>::new().run_in(&mut session).await;
    assert!(matches!(
        r,
        Err(Error {
            kind: ErrorKind::Contract,
            ..
        })
    ));
    let options = RunOptions {
        max_tool_calls: 0,
        ..Default::default()
    };
    assert_eq!(
        Plan::<Summary>::new()
            .options(options)
            .run_in(&mut session)
            .await
            .unwrap_err()
            .kind,
        ErrorKind::Configuration
    );
    assert!(session.usable());
    assert!(!dir.path().join("pid.prompts").exists());
    drop(session);
    agent.shutdown().await.unwrap();
    reaped(dir.path()).await;
}
