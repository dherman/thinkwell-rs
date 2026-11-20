use std::{
    os::unix::fs::PermissionsExt,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
use thinkwell::{Agent, AgentConfig, ErrorKind};
use thinkwell_meeting_scheduler::*;

#[tokio::test]
async fn actual_types_pass_product_validation_across_two_turns() {
    let dir = tempfile::tempdir().unwrap();
    let record = dir.path().join("turns");
    let agent = Agent::connect(
        AgentConfig::new("python3")
            .args([format!("{}/tests/peer.py", env!("CARGO_MANIFEST_DIR"))])
            .env("SCHEDULER_TEST_RECORD", record.to_str().unwrap())
            .env("SCHEDULER_TEST_MODE", "success"),
    )
    .await
    .unwrap();
    let mut session = agent.session().open().await.unwrap();
    let roster: Vec<Attendee> =
        serde_json::from_str(include_str!("../data/attendees.json")).unwrap();
    let parsed = extract(&mut session, &roster).await.unwrap();
    let checked = validate_availability(&roster, parsed).unwrap();
    let outcome = solve(&checked).unwrap();
    let explanation = explain(&mut session, &outcome).await.unwrap();
    assert_eq!(
        outcome,
        Schedule::Scheduled(Slot {
            day: Day::Wednesday,
            start_hour: 11
        })
    );
    assert!(explanation.explanation.contains("All three"));
    assert_eq!(std::fs::read_to_string(record).unwrap(), "2");
    drop(session);
    agent.shutdown().await.unwrap();
}

#[tokio::test]
async fn explanatory_failure_leaves_the_computed_answer_available() {
    let dir = tempfile::tempdir().unwrap();
    let agent = Agent::connect(
        AgentConfig::new("python3")
            .args([format!("{}/tests/peer.py", env!("CARGO_MANIFEST_DIR"))])
            .env(
                "SCHEDULER_TEST_RECORD",
                dir.path().join("turns").to_str().unwrap(),
            )
            .env("SCHEDULER_TEST_MODE", "explanation_failure"),
    )
    .await
    .unwrap();
    let mut session = agent.session().open().await.unwrap();
    let roster =
        serde_json::from_str::<Vec<Attendee>>(include_str!("../data/attendees.json")).unwrap();
    let checked =
        validate_availability(&roster, extract(&mut session, &roster).await.unwrap()).unwrap();
    let outcome = solve(&checked).unwrap();
    assert_eq!(
        explain(&mut session, &outcome).await.unwrap_err().kind,
        ErrorKind::MissingResult
    );
    assert_eq!(
        outcome.to_string(),
        "Scheduled: Wednesday 11:00–12:00 (shared timezone)"
    );
    drop(session);
    agent.shutdown().await.unwrap();
}

#[test]
fn executable_prints_canonical_result_even_when_explanation_fails() {
    let dir = tempfile::tempdir().unwrap();
    let npx = dir.path().join("npx");
    std::fs::copy(
        format!("{}/tests/peer.py", env!("CARGO_MANIFEST_DIR")),
        &npx,
    )
    .unwrap();
    std::fs::set_permissions(&npx, std::fs::Permissions::from_mode(0o755)).unwrap();
    let mut paths = vec![dir.path().to_path_buf()];
    paths.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    let mut child = Command::new(env!("CARGO_BIN_EXE_thinkwell-meeting-scheduler"))
        .env("PATH", std::env::join_paths(paths).unwrap())
        .env("SCHEDULER_TEST_RECORD", dir.path().join("turns"))
        .env("SCHEDULER_TEST_MODE", "explanation_failure")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if child.try_wait().unwrap().is_some() {
            break;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("example executable exceeded test deadline");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let output = child.wait_with_output().unwrap();
    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stdout.contains("Solver result: Scheduled: Wednesday 11:00–12:00"),
        "{stdout}\n{stderr}"
    );
    assert!(!stdout.contains("Agent explanation"));
    assert!(stderr.contains("The solver result above still stands"));
}
