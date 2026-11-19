//! Opt-in live inference; never invoked by cargo test.
use agent_client_protocol::AcpAgentConfig;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{cell::Cell, rc::Rc};
use thinkwell_acp_experiment::runtime::Agent;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Summary {
    total: i64,
    labels: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", deny_unknown_fields)]
enum Decision {
    Accept { total: i64 },
    Reject { reason: String },
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let agent = Agent::connect(
        AcpAgentConfig::new("npx").args([
            "--offline",
            "--yes",
            "@zed-industries/claude-agent-acp@0.23.1",
        ]),
        Some("haiku"),
    )
    .await?;
    let mut session = agent.open();
    let total = Rc::new(Cell::new(0));
    let mut inputs = vec![];
    let summary: Summary = session
        .run(
            "Call accumulate with value 3. Return total from that tool and labels [\"first\"].",
            async |value| {
                inputs.push(value);
                tokio::task::yield_now().await;
                total.set(total.get() + value);
                Ok(total.get())
            },
        )
        .await?;
    assert_eq!(summary.total, 3);
    assert_eq!(summary.labels, ["first"]);
    assert_eq!(inputs, [3]);
    // Deterministic work and restored mutable access between differently typed turns.
    total.set(total.get() * 2);
    inputs.push(0);
    let decision: Decision = session.run(
        "Call accumulate with value 4. Return kind Accept and the new tool total. This is the same conversation, but a different result type.",
        async |value| { inputs.push(value); tokio::task::yield_now().await; total.set(total.get() + value); Ok(total.get()) },
    ).await?;
    assert!(matches!(decision, Decision::Accept { total: 10 }));
    assert_eq!(inputs, [3, 0, 4]);
    session.shutdown().await?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "success":true,"agent":"claude-agent-acp@0.23.1","model":"haiku","acp_sdk":"5.4.0",
            "mcp_revision":"2025-11-25","summary":summary,"decision":decision,"inputs":inputs,
            "total":total.get(),"shutdown":"confirmed",
            "schemas":[thinkwell_acp_experiment::result_contract::contract::<Summary>()?,
                       thinkwell_acp_experiment::result_contract::contract::<Decision>()?]
        }))?
    );
    Ok(())
}
