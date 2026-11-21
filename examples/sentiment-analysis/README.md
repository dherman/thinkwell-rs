# Let an agent use a Rust sentiment tool

This example gives an agent a tool backed by an ordinary Rust crate. The agent
chooses passages from customer feedback, calls the tool to score them, and writes
a short interpretation. Rust keeps a record of the actual measurements.

Start with [main.rs](src/main.rs): it shows the whole conversation. The data types
and scoring checks live in [model.rs](src/model.rs).

## Run it

Use the repository's Rust 1.100.0 toolchain, Node/npm, and an authenticated Claude
installation. Thinkwell's Claude preset uses the cached ACP adapter 0.23.1. If it
is not cached, install/cache it explicitly first:

```sh
npm exec --yes --package=@zed-industries/claude-agent-acp@0.23.1 -- claude-agent-acp --help
```

Authenticate Claude using its normal setup. Then, from the repository root:

```sh
CARGO_TARGET_DIR="$PWD/target/sentiment-analysis" \
  cargo run --manifest-path examples/sentiment-analysis/Cargo.toml --locked
```

This command calls a real agent and may incur provider charges. It uses the agent's
default model and the small included [feedback document](data/feedback.txt), with
no command-line options. No separate native library is needed. This unpublished
package has its own workspace and lockfile; VADER is not a Thinkwell dependency.

## 1. Connect and prepare ordinary Rust state

```rust
let agent = Claude::connect().await?;
let analyzer = SentimentIntensityAnalyzer::new();
let mut scored_passages = Vec::new();
```

The analyzer scores passages; the vector records what the tool actually measured.
Both are local variables. The callback will borrow them for one request.

## 2. Add the tool to the request

For one turn, `.tool(...)` supplies the tool's name, description, and function
in one place. `.run(&agent)` opens a fresh conversation, runs the request, and
releases that conversation before returning. The following excerpt abbreviates
only the instructions; the full prompt is in [main.rs](src/main.rs).

```rust
let report = Plan::<Analysis>::new()
    .text("Use analyze_sentiment on passages from this feedback, then interpret its tone.")
    .json(&feedback)?
    .tool(
        "analyze_sentiment",
        "Measure an exact feedback excerpt with VADER. Returns a compound score
         from -1 to 1 and positive, negative, and neutral proportions from 0 to 1.",
        async |passage: TextPassage| {
            let scores = score_passage(&analyzer, feedback, &passage.text)?;
            scored_passages.push((passage.text, scores.clone()));
            Ok(scores)
        },
    )
    .run(&agent)
    .await;
```

`TextPassage` derives `Deserialize` and `JsonSchema`; Thinkwell checks the agent's
arguments before calling Rust. The inferred output, `SentimentScores`, derives
`Serialize` and `JsonSchema`; Thinkwell checks it before sending it back.
Its fields are ordinary finite `f64` values.

The callback borrows the analyzer and mutably borrows the log. It needs no shared
ownership wrapper or lock. While the request runs, Rust prevents us from using
the mutably borrowed log elsewhere. After `.await`, that borrow ends and we can
print the measurements with a normal loop. The agent remains connected after
success, and dropping it handles cleanup without an explicit shutdown sequence.
A second `.run(&agent)` would start fresh history.

For follow-up questions in the same conversation, or to choose a model, use an
explicit session and `.run_in(&mut session)`. In that form, register a `Tool` when
opening the session and bind its function on each turn. The
[meeting scheduler](../meeting-scheduler/README.md) demonstrates retained history;
Thinkwell's `Tool` Rustdocs demonstrate registration and binding. An inline tool
does not add new tools to a session that is already open.

`score_passage` accepts only nonempty, exact excerpts of the feedback. The agent
gets a tool error if it invents a passage or changes its wording, and can retry.
The wrapper also checks that the analyzer returned every score and that each is
finite and within its documented range. These are application checks, not extra
schema keywords. Synchronous scoring is appropriate for this tiny fixed document;
a large-document application should reconsider input limits and blocking work.

## 3. Separate measurements from interpretation

The executable prints two parts:

- **Measured passages (VADER):** exact inputs and scores from the local log.
- **Agent interpretation (generated prose):** an overall tone, passage summaries,
  and a recommendation returned as `Analysis`.

The report deliberately does not ask the agent to copy numeric scores. The local
log owns those numbers. The prose is still a model's interpretation: the program
does not prove that each summary faithfully explains a particular measurement.

The agent must successfully score at least one passage. If it skips the tool,
the example reports an error instead of presenting a tool-backed analysis. If
its final answer fails, the executable still prints and flushes any measurements
already collected, then exits unsuccessfully. A failed conversation cannot undo
callback effects that have already happened.

## What the numbers mean

The [vader_sentiment](https://docs.rs/vader_sentiment/0.1.1/vader_sentiment/) crate
uses VADER, an English lexicon and rule-based heuristic designed especially for
social-media text. Its compound score runs from -1 (negative) to 1 (positive).
Positive, negative, and neutral values describe proportions of the analyzed
sentiment. These values are not probabilities that a person feels a certain way.

For example, the local dependency probe scored “This is good.” at roughly 0.440
and “This is not good.” at -0.341. That illustrates negation handling, not a
promise of reliable interpretation of every sentence. Sarcasm, context, and
languages other than English need care. Empty input gives all-zero scores in this
crate; this example rejects empty passages before scoring.

VADER is different from the AFINN-based npm `sentiment` package used by the
[original TypeScript example](https://github.com/dherman/thinkwell/blob/2393101fd66b8471f33bb3a4908aeee919fdf0ce/examples/src/sentiment.ts)
and [tutorial](https://thinkwell.sh/examples/sentiment-analysis). Do not compare
its compound score with the original's raw or comparative scores. JSON formatting
helps present the feedback but does not create an instruction-trust boundary.

## Tests and attribution

The checks below require Python 3 on macOS or Linux and never invoke a real agent:

```sh
export CARGO_TARGET_DIR="$PWD/target/sentiment-analysis"
cargo fmt --manifest-path examples/sentiment-analysis/Cargo.toml -- --check
cargo check --manifest-path examples/sentiment-analysis/Cargo.toml --all-targets --locked
cargo clippy --manifest-path examples/sentiment-analysis/Cargo.toml --all-targets --locked -- -D warnings
cargo test --manifest-path examples/sentiment-analysis/Cargo.toml --locked
```

Tests use the actual analyzer, typed contracts, tool dispatch, and executable.
They cover invalid excerpts, invalid analyzer values, a corrected final response,
missing tool use, and retaining the log when the final response fails. The scripted
agent checks behavior reproducibly; it does not establish real-model accuracy.

The Rust code, tutorial, and feedback fixture here are original. The dependency
`vader_sentiment` 0.1.1 is MIT-licensed and retains its bundled upstream notices,
including C.J. Hutto's 2016 notice. No lexicon is copied into this repository.
VADER's research reference is Hutto and Gilbert (2014), *VADER: A Parsimonious
Rule-based Model for Sentiment Analysis of Social Media Text*, ICWSM-14.

To try this example with Codex, change the `agent::Claude` import to
`agent::Codex` and the connection to `Codex::connect().await?`. The rest of the
workflow is unchanged. Follow the repository's [Codex setup instructions](../../README.md)
first; Thinkwell does not install or authenticate agents during connection.
