# Sentiment analysis

Roadmap identifier: `sentiment-analysis`.

Goal: teach how an agent calls a deterministic Rust package through a borrowed
application tool, then interprets the results in a typed response.

## Milestone 1. A borrowed sentiment tool {#sentiment-analysis-1}

**Identifier:** `sentiment-analysis-1`

**Status:** In progress

**Outcome:** A small runnable example and tutorial show a local analyzer and
mutable call log borrowed by a tool callback, followed by a typed agent report.

**Approval and dependencies:** The owner reviewed and merged proposal PR #5 and
assigned implementation, including the finite-f64 decision. The milestone became
Ready and is now In progress. The prerequisite closeout is integrated at
343f4253f9686a0ff56e5a1dc65d87aea00b2437; both final-head CI jobs passed in
runs 44851884833 and 44851879894. No blocking design decision remains.

**Design basis:** `R-code-prompt-ergonomics`, `R-easy-agent-programming`, and
[typed conversations](../design/typed-conversations.md). No new agent lifecycle,
callback concurrency, or tool registration abstraction is proposed.

**Tasks**

- [x] Agree on finite floating-point contracts and example scope.
- [x] Build an unpublished standalone example with original short English feedback.
- [x] Wrap the analyzer in a typed tool borrowing local state; show Plan composition
      and binding directly in main, without CLI parsing or helper frameworks.
- [x] Add meaningful deterministic/tool integration tests and a learning-focused tutorial.
- [x] Run a bounded live demonstration and separate author review.
- [ ] Obtain Linux CI, owner review, and integration.

### Example flow

1. Read a fixed included feedback document. Construct one VADER analyzer and a
   local vector of scored passages.
2. Define `Tool<TextPassage, SentimentScores>` and open a session with that tool.
3. Build a `Plan<Analysis>` with instructions and `.json(&feedback)?`. Bind an async
   closure that borrows the analyzer and mutably borrows the vector. The closure
   scores each requested passage, records the actual input/output, and returns it.
4. Ask the agent to choose meaningful passages, call the tool, and return an
   overall tone, summaries of the passages, and a recommendation.
5. After awaiting the plan, print the locally recorded scores separately from the
   generated report. The vector becomes accessible again without Arc or Mutex.
   Preserve the tool log if the agent fails to provide its final answer.

Keep one tool and one agent turn. Validate that requested text is nonempty and
is an exact excerpt of the supplied document; reject invented/paraphrased passages
with a tool error. Require at least one successful scoring call before presenting
an analysis as tool-backed. Do not copy scores into the model's final answer type:
the local log owns the numbers, while the report owns generated interpretation.
This avoids making faithful numerical transcription part of the demonstration.

The intended call shape is:

```rust,ignore
let report = Plan::<Analysis>::new()
    .text("Use analyze_sentiment on passages from this feedback, then summarize its tone.")
    .json(&feedback)?
    .bind(&sentiment, async |passage: TextPassage| {
        let scores = score_passage(&analyzer, &feedback, &passage.text)?;
        scored_passages.push((passage.text, scores.clone()));
        Ok(scores)
    })
    .run_in(&mut session)
    .await;
```

The implemented main follows this shape; the tutorial abbreviates the prompt only. Use a short bounded
fixture and synchronous scoring inside the callback; no claim is made about
running large documents without blocking the async runtime. No sentiment accuracy,
sarcasm detection, multilingual support, streaming, arbitrary file inputs, or
production document-analysis service is promised.

### Dependency and source evidence

Inspected the pinned TypeScript [example](https://github.com/dherman/thinkwell/blob/2393101fd66b8471f33bb3a4908aeee919fdf0ce/examples/src/sentiment.ts)
and [tutorial source](https://github.com/dherman/thinkwell/blob/2393101fd66b8471f33bb3a4908aeee919fdf0ce/website/examples/sentiment-analysis.mdx).
The public [tutorial](https://thinkwell.sh/examples/sentiment-analysis) is also a
future implementation reference. These are migration evidence, not Rust authority.
The original uses npm sentiment/AFINN score, comparative score, and token lists;
it lets the agent select passages and synthesize recommendations.

Recommend `vader_sentiment = "=0.1.1"`, with the example's own lockfile. Its
`SentimentIntensityAnalyzer::polarity_scores` returns a map of f64 values: positive,
negative, neutral proportions and a compound score in [-1, 1]. Compound is not a
probability or an AFINN score; do not label it comparative or promise numerical
parity. Empty input returns all zeros, not neutral proportion one.

A disposable probe under ignored target/sentiment-probe built on Rust 1.100.0/macOS
with no native solver dependency. `cargo run --manifest-path
target/sentiment-probe/Cargo.toml` checked six original inputs for finite scores and
ranges. Positive/negative examples produced compound approximately 0.848/-0.735;
neutral and empty inputs produced zero. “This is good.” produced 0.440 and
“This is not good.” produced -0.341. These are smoke checks, not accuracy research,
Linux evidence, or a live tool interaction. No new product dependency was added.

Inspected the package README, source, LICENSE-MIT, and LICENSE.txt. The package
contains MIT terms including the upstream C.J. Hutto 2016 notice. Use it as a Cargo
dependency; retain its notices and credit VADER in the tutorial. Write original
Rust code, prose, and feedback fixtures; do not copy a lexicon into this repository.
Its research citation is Hutto and Gilbert (2014), VADER: A Parsimonious Rule-based
Model for Sentiment Analysis of Social Media Text. The old release date and limited
smoke evidence remain limitations, not evidence of maintained upstream support.

### Approved floating-point scope

The probe confirmed Schemars 1.2.3 emits `type: number, format: double` for f64.
Before this milestone, Thinkwell rejected that format. Approved adaptation: accept that format
only with number, and validate finite representability in f64. Apply it to result,
tool input and tool output contracts. Do not treat a nonfinite serialized value
as a valid number (Serde JSON may encode nonfinite floats as null). Preserve exact
const/enum comparisons and existing integer rules. Test invalid format/type
combinations, finite boundaries, and nonfinite tool outputs before accepting this
change. No general floating-point range keywords or arbitrary formats are added.

Implementation includes a design update and scoped library tests. Avoid
example-specific schema overrides, string scores, or scaled integers merely to
conceal the ergonomic gap. Typed fields should name compound/positive/negative/
neutral explicitly instead of exposing the dependency's string-keyed map.

### Completion criteria

#### Tool flow and ownership {#sentiment-analysis-1-flow}

- **Anchor coverage:** `A-typed-turn-completion`.
- **Criterion:** The example compiles with a callback borrowing local analyzer and
  mutable log; tests exercise actual tool dispatch and final typed response,
  rejection of invalid excerpts, missing tool use, and preserved log on failure.
- **Evidence:** Local pass: three executable tests exercise real typed tool dispatch
  and analyzer scoring through a scripted ACP peer, reject bad input/excerpts,
  recover from an invalid typed final result, reject missing tool use, and preserve
  measurements after final-answer failure. The live run below also passed.

#### Numeric contracts {#sentiment-analysis-1-numbers}

- **Anchor coverage:** `A-supported-contract-boundary`.
- **Criterion:** f64 tests establish finite valid input/output/results,
  unsupported format rejection, nonfinite-output failure, and unchanged exact
  numeric comparisons. Wrapper tests cover missing keys and finite score ranges.
- **Evidence:** Local pass: contract tests cover signed zero, maximum/minimum finite
  values, minimum normal/subnormal values, invalid types/formats, NaN and infinities.
  An actual ACP/HTTP test exercises f64 tool inputs/outputs and result decoding,
  rejects nonfinite outputs, and confirms invalid input never reaches the callback.
  Existing exact const/enum and integer checks still pass. Three example unit tests
  cover analyzer behavior, excerpt checks, missing keys and invalid score ranges.

#### Learning and delivery {#sentiment-analysis-1-delivery}

- **Anchor coverage:** None — example teaching and delivery.
- **Criterion:** Tutorial explains VADER semantics, the borrowed callback, and
  measured scores versus generated interpretation. Original-source/license review,
  deterministic checks, bounded live run, Linux CI, owner review and integration
  are recorded. Flag further API friction rather than silently expanding scope.
- **Evidence:** Local tutorial/code review, deterministic tests and live run pass.
  A separate CI job is authored, not yet executed. Owner review and integration
  remain pending; the milestone is In progress.


### Local implementation evidence

Tested on macOS with Rust 1.100.0, starting at merge commit
343f4253f9686a0ff56e5a1dc65d87aea00b2437 plus the uncommitted sentiment implementation.
The new standalone package locks vader_sentiment 0.1.1 and keeps it out of the
root workspace. Its typed output names the four measurements; local range checks
remain application logic. The new library support accepts only double/number,
without changing exact const/enum comparison or integer rules.

Commands passed:

- `cargo check --workspace --all-targets --locked`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked` (including bootstrap consistency and Rustdoc)
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked`
- `python3 -m unittest discover -s experiments/acp-integration -p 'test_probe_agent.py'`
- Example all-target check, strict Clippy, and `cargo test` with
  `--manifest-path examples/sentiment-analysis/Cargo.toml --locked` and
  `CARGO_TARGET_DIR=$PWD/target/sentiment-analysis` (six tests).
- All eight existing meeting-scheduler tests passed against the changed library,
  using its previously recorded CMAKE/CARGO_TARGET_DIR settings and locked manifest.
- Root/example formatting, `cargo anchor check --config anchorbook.toml`,
  `target/tools/bin/mdbook build`, and `git diff --check`.

The example binary ran successfully with a 180-second outer deadline, existing
Claude authentication, adapter 0.23.1 and the agent-default model. It scored the
three supplied paragraphs at approximately 0.848, -0.637, and 0.000 compound,
then returned Mixed tone and an export-reliability recommendation. This used the
same implementation before adding two descriptive final-answer field comments;
final deterministic tests passed after that documentation refinement. This is one
successful configuration, not a claim of sentiment accuracy or all-agent support.

A separate implementation-author self-review inspected code, tutorial, fixture,
licenses, tests, CI and Book diff. It confirmed borrowed state remains local,
callbacks log only validated measurements, and final failures preserve the log.
It clarified that finite validation applies to f64 contracts, not arbitrary
serialized JSON. No independent review is claimed. The agent may repeat numbers
in its prose; only the separately printed log is canonical. No further API
adaptation is proposed from this example.

All Rust example code, prompts, feedback, and tutorial prose are original. Existing
repository test-harness patterns were reused under this repository's license;
VADER source and lexicons remain in the dependency with their original notices.
CI and owner-reviewed delivery remain outstanding. No commit, push, publication,
or end-of-Porting-Phase transition is implied by local implementation evidence.
