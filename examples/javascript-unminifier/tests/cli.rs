use std::{
    os::unix::fs::PermissionsExt,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

fn run(input: &std::path::Path, output: &std::path::Path, mode: &str) -> std::process::Output {
    let tools = tempfile::tempdir().unwrap();
    let npx = tools.path().join("npx");
    std::fs::copy(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/peer.py"), &npx).unwrap();
    std::fs::set_permissions(&npx, std::fs::Permissions::from_mode(0o755)).unwrap();
    let mut paths = vec![tools.path().to_path_buf()];
    paths.extend(std::env::split_paths(&std::env::var_os("PATH").unwrap()));
    let mut child = Command::new(env!("CARGO_BIN_EXE_thinkwell-javascript-unminifier"))
        .args([input, output])
        .env("PATH", std::env::join_paths(paths).unwrap())
        .env("UNMINIFIER_TEST_MODE", mode)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(25);
    while child.try_wait().unwrap().is_none() {
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("CLI timed out");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    child.wait_with_output().unwrap()
}

#[test]
fn cli_transforms_without_touching_input_and_rejects_existing_output() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.mjs");
    let output = dir.path().join("output.mjs");
    let source = include_str!("../fixtures/average.mjs");
    std::fs::write(&input, source).unwrap();
    let result = run(&input, &output, "success");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(
        std::fs::read_to_string(&output)
            .unwrap()
            .contains("function work(xs)")
    );
    assert_eq!(std::fs::read_to_string(&input).unwrap(), source);
    let saved = std::fs::read(&output).unwrap();
    assert!(!run(&input, &output, "missing_result").status.success());
    assert_eq!(std::fs::read(&output).unwrap(), saved);
    assert!(!run(&input, &input, "success").status.success());
    assert_eq!(std::fs::read_to_string(&input).unwrap(), source);
}

#[test]
fn invalid_response_and_invalid_source_leave_no_output() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.mjs");
    let output = dir.path().join("output.mjs");
    std::fs::write(&input, "function a(){}").unwrap();
    for mode in ["unknown", "missing_result"] {
        assert!(!run(&input, &output, mode).status.success());
        assert!(!output.exists());
    }
    std::fs::write(&input, "function a(").unwrap();
    assert!(!run(&input, &output, "success").status.success());
    assert!(!output.exists());
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn empty_candidate_set_needs_no_agent_and_symlinks_are_not_replaced() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.mjs");
    let output = dir.path().join("output.mjs");
    let source = "export function publicName(){}";
    std::fs::write(&input, source).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_thinkwell-javascript-unminifier"))
        .args([&input, &output])
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(result.status.success());
    assert_eq!(std::fs::read_to_string(&output).unwrap(), source);
    std::fs::remove_file(&output).unwrap();
    std::os::unix::fs::symlink(dir.path().join("absent"), &output).unwrap();
    assert!(!run(&input, &output, "success").status.success());
    assert!(output.symlink_metadata().unwrap().file_type().is_symlink());
}
