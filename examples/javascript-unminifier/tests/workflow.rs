use thinkwell::{Agent, AgentConfig};
use thinkwell_javascript_unminifier::propose_and_rename;

async fn run(mode: &str) -> Result<String, Box<dyn std::error::Error>> {
    let source: String = (0..9)
        .map(|i| format!("function f{i}(){{return {i};}}\n"))
        .collect();
    tokio::time::timeout(std::time::Duration::from_secs(20), async {
        let agent = Agent::connect(AgentConfig::new("python3").args([
            format!("{}/tests/peer.py", env!("CARGO_MANIFEST_DIR")),
            mode.into(),
        ]))
        .await?;
        Ok(propose_and_rename(&source, &agent).await?)
    })
    .await?
}

#[tokio::test(flavor = "current_thread")]
async fn sequential_batches_reuse_agent_and_restore_source_order() {
    let output = run("success").await.unwrap();
    assert!(output.contains("function work()"));
    assert!(output.contains("function work9()"));
    assert!(output.find("function work()").unwrap() < output.find("function work2()").unwrap());
}

#[tokio::test(flavor = "current_thread")]
async fn agent_failure_is_returned() {
    assert!(run("missing_result").await.is_err());
}
