//! Opt-in live acceptance check with explicit shutdown confirmation.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thinkwell::{Plan, Tool, agent::Claude};

#[derive(Deserialize, JsonSchema)]
struct Add {
    value: i64,
}

#[derive(Serialize, JsonSchema)]
struct Total {
    total: i64,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Summary {
    total: i64,
    labels: Vec<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "kind", deny_unknown_fields)]
enum Decision {
    Accept { total: i64 },
    Reject { reason: String },
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let agent = Claude::connect().await?;
    let add = Tool::<Add, Total>::new("add", "Add to the accumulator");
    let mut session = agent.session().tool(&add).model("haiku").open().await?;
    let mut total = 0;

    let summary = Plan::<Summary>::new()
        .text("Add 3, then summarize the total with labels [first].")
        .bind(&add, async |input: Add| {
            total += input.value;
            Ok(Total { total })
        })
        .run_in(&mut session)
        .await?;

    assert_eq!(summary.total, 3);
    assert_eq!(summary.labels, ["first"]);
    total *= 2;

    let decision = Plan::<Decision>::new()
        .text("Add 4, then return Accept with the new total.")
        .bind(&add, async |input: Add| {
            total += input.value;
            Ok(Total { total })
        })
        .run_in(&mut session)
        .await?;

    match decision {
        Decision::Accept { total } => assert_eq!(total, 10),
        Decision::Reject { reason } => panic!("unexpected rejection: {reason}"),
    }
    assert_eq!(total, 10);
    drop(session);
    agent.shutdown().await?;
    println!("two typed turns, borrowed callbacks, and shutdown: passed");
    Ok(())
}
