# Sentiment analysis

Roadmap identifier: `sentiment-analysis`.

Goal: teach how an agent calls a deterministic Rust package through a borrowed
application tool, then interprets the results in a typed response.

## Milestone 1. A borrowed sentiment tool {#sentiment-analysis-1}

**Identifier:** `sentiment-analysis-1`

**Status:** Complete

**Outcome:** A small runnable example and tutorial show a local analyzer and
mutable call log borrowed by a tool callback, followed by a typed agent report.

**Approval and dependencies:** The owner reviewed and merged proposal PR #5 and
assigned implementation, including the finite-f64 decision. The milestone became
Ready before implementation; final integration is recorded below. The prerequisite closeout is integrated at
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
- [x] Obtain Linux CI, owner review, and integration.

### Owning design

[Example applications](../design/examples.md#sentiment-analysis) owns the accepted
scoring, logging, presentation, and teaching rules. This first milestone used an
explicit catalog and per-turn binding. The separately approved
[one-shot tools](one-shot-tools.md) later simplified the same application to an
inline tool and `run(&agent)`; its tests and live evidence are recorded separately.


### Dependency and source evidence

Inspected the pinned TypeScript [example](https://github.com/dherman/thinkwell/blob/2393101fd66b8471f33bb3a4908aeee919fdf0ce/examples/src/sentiment.ts)
and [tutorial source](https://github.com/dherman/thinkwell/blob/2393101fd66b8471f33bb3a4908aeee919fdf0ce/website/examples/sentiment-analysis.mdx).
The public [tutorial](https://thinkwell.sh/examples/sentiment-analysis) was also inspected as a teaching reference. These are migration evidence, not Rust authority.
The original uses npm sentiment/AFINN score, comparative score, and token lists;
it lets the agent select passages and synthesize recommendations.

The selected dependency is `vader_sentiment = "=0.1.1"`, with the example's own lockfile. Its
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

### Approved floating-point adaptation

The owner approved finite f64 support after the probe identified Schemars 1.2.3's
`number`/`double` representation. [Contracts](../design/contracts.md) owns the
accepted generation, validation, serialization, and exact-comparison rules.
The numeric criterion below retains the product regression evidence; the example's
named score fields and range checks remain application responsibilities.


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
  The separate CI job passed on the final head and merge commit; the owner
  reviewed and merged PR #6. See integrated completion below.


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
Final CI and owner-reviewed delivery are recorded below. The local implementation
evidence alone establishes no publication or lifecycle transition.


### Integrated completion {#integrated-completion}

The owner reviewed and merged PR #6, integrating final head
82bfebba05af175d8603bd3b002a3f4e4b7e4760 as
0dbb432ea4372e29e1323e0bba94ce5fd286d4bf. All three jobs (Rust and Book checks,
Meeting scheduler example, Sentiment analysis example) passed on the final head
in [PR CI](https://github.com/dherman/thinkwell-rs/actions/runs/44876451294) and
[push CI](https://github.com/dherman/thinkwell-rs/actions/runs/44876447906).
[Merge-commit CI](https://github.com/dherman/thinkwell-rs/actions/runs/44879106860)
also passed. The scheduler-only rerun demonstrated cache reuse; its measurements
are recorded in the [scheduler roadmap](meeting-scheduler.md).

This completes the milestone's tool-flow, numeric-contract, and learning/delivery
criteria: the recorded local tests and live run, final Linux CI, author review,
owner review, and integration hold. The source and accuracy limitations remain
unchanged. The owner subsequently integrated this Book synchronization in PR #7
at `8e039b1215d96a23af4f6c798704f97c013fea25`.

The example validates borrowed analyzer/mutable-log callbacks and immediate JSON
context serialization. Its ergonomic finding led to the separately approved and
integrated [one-shot tools milestone](one-shot-tools.md). Tool-use verification and
faithful interpretation remain application concerns; no provenance guarantee,
publication, or lifecycle transition is added by completion.
