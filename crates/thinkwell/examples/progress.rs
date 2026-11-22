//! Opt-in progress display with a borrowed tool and a typed result.
use thinkwell::{Plan, ProgressEvent, agent::Claude};

#[tokio::main(flavor = "current_thread")]
async fn main() -> thinkwell::Result<()> {
    // To use Codex, change the import and this call to Codex::connect().
    let agent = Claude::connect().await?;
    let mut total = 0_i64;
    let mut updates = 0;
    let answer = Plan::<i64>::new()
        .text("Call add with 3, then return the total reported by that tool.")
        .tool("add", "Add to the running total", async |amount: i64| {
            total += amount;
            Ok(total)
        })
        .observe(|event| {
            updates += 1;
            match event {
                ProgressEvent::Message(message) => eprint!("{}", message.text()),
                ProgressEvent::AgentTool(tool) => {
                    if let Some(title) = tool.title() {
                        eprintln!("\nAgent activity: {title}");
                    }
                }
                ProgressEvent::ToolStarted(tool) => eprintln!("\nRunning {}", tool.name()),
                ProgressEvent::ToolFinished(tool) => {
                    eprintln!("{}: {:?}", tool.name(), tool.outcome());
                }
                ProgressEvent::ResultRejected(rejection) => {
                    eprintln!("Result needs correction: {}", rejection.reason());
                }
                ProgressEvent::EventsDropped(loss) => {
                    eprintln!("{} progress updates omitted", loss.count());
                }
                _ => {}
            }
        })
        .run(&agent)
        .await?;

    // Both the tool's and the observer's captures are available again.
    assert_eq!(total, 3);
    assert_eq!(answer, total);
    println!("\nTyped answer: {answer}; Rust total: {total}; progress updates: {updates}");
    Ok(())
}
