use std::{fs, io::Write};
use thinkwell::agent::Claude;
use thinkwell_javascript_unminifier::{javascript, suggest_names};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    assert_eq!(args.len(), 2, "Usage: unminifier INPUT.mjs OUTPUT.mjs");
    let source = fs::read_to_string(&args[0])?;
    let candidates = javascript::discover(&source);
    let output = if candidates.is_empty() {
        source
    } else {
        let agent = Claude::connect().await?;
        let names = suggest_names(&source, &candidates, &agent).await?;
        javascript::rename(&source, &candidates, &names)
    };
    // A new file keeps accidental reruns from overwriting the input or prior output.
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[1])?;
    file.write_all(output.as_bytes())?;
    Ok(())
}
