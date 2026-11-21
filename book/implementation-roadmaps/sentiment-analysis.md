# Sentiment analysis

Roadmap identifier: `sentiment-analysis`.

Goal: teach how an agent calls a deterministic Rust package through a borrowed
application tool, then interprets the results in a typed response.

## Milestone 1. A borrowed sentiment tool {#sentiment-analysis-1}

**Identifier:** `sentiment-analysis-1`

**Status:** Proposed

**Outcome:** A small runnable example and tutorial show a local analyzer and
mutable call log borrowed by a tool callback, followed by a typed agent report.

**Dependencies:** Integrate the scheduler closeout in PR #5. Obtain owner approval
of this scope and the floating-point contract decision below before implementation.
The owner assigned proposal preparation, not this implementation milestone.

**Design basis:** `R-code-prompt-ergonomics`, `R-easy-agent-programming`, and
[typed conversations](../design/typed-conversations.md). No new agent lifecycle,
callback concurrency, or tool registration abstraction is proposed.

**Tasks**

- [ ] Agree on finite floating-point contracts and example scope.
- [ ] Build an unpublished standalone example with original short English feedback.
- [ ] Wrap the analyzer in a typed tool borrowing local state; show Plan composition
      and binding directly in main, without CLI parsing or helper frameworks.
- [ ] Add meaningful deterministic/tool integration tests and a learning-focused tutorial.
- [ ] Run an opt-in live demonstration and Linux CI, review, and owner integration.

### Proposed flow

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

This is an illustrative sketch, not compiled example evidence. Use a short bounded
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

### Floating-point decision for owner review

The probe confirmed Schemars 1.2.3 emits `type: number, format: double` for f64.
Thinkwell currently rejects that format. Proposed adaptation: accept that format
only with number, and validate finite representability in f64. Apply it to result,
tool input and tool output contracts. Do not treat a nonfinite serialized value
as a valid number (Serde JSON may encode nonfinite floats as null). Preserve exact
const/enum comparisons and existing integer rules. Test invalid format/type
combinations, finite boundaries, and nonfinite tool outputs before accepting this
change. No general floating-point range keywords or arbitrary formats are added.

This needs an accepted design update and scoped library tests if approved. Avoid
example-specific schema overrides, string scores, or scaled integers merely to
conceal the ergonomic gap. Typed fields should name compound/positive/negative/
neutral explicitly instead of exposing the dependency's string-keyed map.

### Completion criteria

#### Tool flow and ownership {#sentiment-analysis-1-flow}

- **Anchor coverage:** `A-typed-turn-completion`.
- **Criterion:** The example compiles with a callback borrowing local analyzer and
  mutable log; tests exercise actual tool dispatch and final typed response,
  rejection of invalid excerpts, missing tool use, and preserved log on failure.
- **Evidence:** Pending.

#### Numeric contracts {#sentiment-analysis-1-numbers}

- **Anchor coverage:** `A-supported-contract-boundary`.
- **Criterion:** If approved, f64 tests establish finite valid input/output/results,
  unsupported format rejection, nonfinite-output failure, and unchanged exact
  numeric comparisons. Wrapper tests cover missing keys and finite score ranges.
- **Evidence:** Probe confirms the current gap only; product behavior remains unchanged.

#### Learning and delivery {#sentiment-analysis-1-delivery}

- **Anchor coverage:** None — example teaching and delivery.
- **Criterion:** Tutorial explains VADER semantics, the borrowed callback, and
  measured scores versus generated interpretation. Original-source/license review,
  deterministic checks, bounded live run, Linux CI, owner review and integration
  are recorded. Flag further API friction rather than silently expanding scope.
- **Evidence:** Draft proposal; no implementation or delivery completion claimed.
