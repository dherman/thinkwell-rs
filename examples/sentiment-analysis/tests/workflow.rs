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
    let mut child = Command::new(env!("CARGO_BIN_EXE_thinkwell-sentiment-analysis"))
        .env("PATH", std::env::join_paths(paths).unwrap())
        .env("SENTIMENT_TEST_RECORD", dir.path().join("turns"))
        .env("SENTIMENT_TEST_MODE", mode)
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
        "1"
    );
    output
}

#[test]
fn scores_real_excerpts_and_returns_a_typed_report() {
    let output = run_example("success");
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(output.status.success(), "{stdout}\n{stderr}");
    assert!(stdout.contains("compound="));
    assert!(stdout.contains("Search is praised"));
    assert!(
        stdout.find("Measured passages").unwrap() < stdout.find("Agent interpretation").unwrap()
    );
    assert!(!stdout.contains("invented excerpt"));
}

#[test]
fn missing_tool_use_is_not_presented_as_tool_backed_analysis() {
    let output = run_example("no_tools");
    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(!stdout.contains("Agent interpretation"));
    assert!(stderr.contains("without successfully scoring"));
}

#[test]
fn final_answer_failure_preserves_measurements() {
    let output = run_example("missing_result");
    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stdout.contains("compound="), "{stdout}\n{stderr}");
    assert!(!stdout.contains("Agent interpretation"));
    assert!(stderr.contains("Recorded measurements above remain available"));
}
