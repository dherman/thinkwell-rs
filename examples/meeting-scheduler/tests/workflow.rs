use std::{
    os::unix::fs::PermissionsExt,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

fn run_example(mode: &str) -> std::process::Output {
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
        .env("SCHEDULER_TEST_MODE", mode)
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
    assert_eq!(
        std::fs::read_to_string(dir.path().join("turns")).unwrap(),
        "2"
    );
    output
}

#[test]
fn executable_runs_two_typed_turns_with_validated_data_and_real_solving() {
    let output = run_example("success");
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(output.status.success(), "{stdout}\n{stderr}");
    assert!(stdout.contains("Solver result: Scheduled: Wednesday 11:00–12:00"));
    assert!(stdout.contains("All three parsed windows"));
    assert!(stdout.find("Solver result:").unwrap() < stdout.find("Agent explanation").unwrap());
}

#[test]
fn executable_prints_canonical_result_even_when_explanation_fails() {
    let output = run_example("explanation_failure");
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
