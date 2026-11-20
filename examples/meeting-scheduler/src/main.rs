use std::{io::Write, path::PathBuf};
use thinkwell::agent::Claude;
use thinkwell_meeting_scheduler::{
    Attendee, explain, extract, format_extraction, solve, validate_availability, validate_roster,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> thinkwell_meeting_scheduler::Result<()> {
    let mut args = std::env::args_os().skip(1);
    let mut path = None;
    let mut model = None;
    while let Some(arg) = args.next() {
        if arg == "--model" {
            model = Some(
                args.next()
                    .ok_or("--model needs an agent model name")?
                    .into_string()
                    .map_err(|_| "model name must be UTF-8")?,
            );
        } else if arg == "--help" || arg == "-h" {
            println!(
                "Usage: thinkwell-meeting-scheduler [roster.json] [--model NAME]\nDefaults to the included roster and the agent's default model."
            );
            return Ok(());
        } else if arg.to_string_lossy().starts_with('-') || path.is_some() {
            return Err("expected one roster path and optional --model NAME (see --help)".into());
        } else {
            path = Some(PathBuf::from(arg));
        }
    }
    let source = match path {
        Some(path) => std::fs::read_to_string(path)?,
        None => include_str!("../data/attendees.json").to_owned(),
    };
    let roster: Vec<Attendee> = serde_json::from_str(&source)?;
    validate_roster(&roster)?;

    let agent = Claude::connect().await?;
    let mut builder = agent.session();
    if let Some(model) = model {
        builder = builder.model(model);
    }
    let mut session = builder.open().await?;

    let parsed = extract(&mut session, &roster).await?;
    println!("{}", format_extraction(&roster, &parsed));
    let checked = validate_availability(&roster, parsed)?;

    // Native solving is synchronous. Leave the Tokio thread free to drive ACP I/O.
    let outcome = tokio::task::spawn_blocking(move || solve(&checked).map_err(|e| e.to_string()))
        .await?
        .map_err(std::io::Error::other)?;
    println!("\nSolver result: {outcome}");
    std::io::stdout().flush()?; // Preserve the canonical answer even if the next turn fails.

    match explain(&mut session, &outcome).await {
        Ok(answer) => println!(
            "\nAgent explanation (generated prose):\n{}",
            answer.explanation
        ),
        Err(error) => {
            eprintln!("Explanation unavailable: {error}. The solver result above still stands.");
            return Err(error.into());
        }
    }
    Ok(())
}
