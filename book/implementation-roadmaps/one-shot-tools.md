# One-shot tools

Roadmap identifier: `one-shot-tools`.

## Milestone 1. One-turn execution with inline tools {#one-shot-tools-1}

**Identifier:** `one-shot-tools-1`

**Status:** Complete

**Outcome:** Plan::tool declares and binds a borrowed callback; Plan::run opens a
fresh temporary session for one turn and drops it before returning.

**Approval and dependencies:** The owner approved the concrete sentiment example
sketch with “merged. proceed!” after merging closeout PR #7 at
8e039b1215d96a23af4f6c798704f97c013fea25. The milestone became Ready and was
assigned in this session. PR #7's final-head CI passed all three jobs.

**Design basis:** `R-easy-agent-programming`, `R-progressive-control`,
`R-code-prompt-ergonomics`, `D-reviewed-api-baseline`, and the
[typed-conversation design](../design/typed-conversations.md).

**Tasks:** Implement both conveniences using existing Tool, session setup, typed
turn validation and cleanup; document borrowing and failure behavior; simplify
the sentiment example; test reuse, busy agents, local preflight and cancellation.

### Boundaries

Use the agent's default model for run; explicit session builders remain the path
for model selection and multi-turn history. Inline tools and named bindings both
supply the temporary session's catalog. A Plan with callbacks remains non-cloneable.
Plan::tool does not add tools to an existing session: run_in still checks that
bindings match the registered catalog exactly. No concurrent sessions, history
continuation, new provider or new shutdown mechanism is introduced.

### Completion criteria

#### Composition and validation {#one-shot-tools-1-composition}

- **Anchor coverage:** `A-supported-contract-boundary`; `A-typed-turn-completion`.
- **Criterion:** Tests cover inline and named borrowed callbacks, no-tool runs,
  unsupported answers/tools, invalid options, duplicate tools, and compatibility
  with explicit-session validation. Local preflight errors occur before setup.
- **Evidence:** Local pass: one_shot_preflight_and_busy_leave_agent_available
  checks unsupported answer/tool types, reserved/duplicate names, invalid options,
  absence of remote setup on preflight errors, Busy, and run_in catalog matching.
  Reuse tests exercise inline, named, and tool-free plans.

#### Ownership and session lifecycle {#one-shot-tools-1-lifecycle}

- **Anchor coverage:** `A-owned-resource-cleanup`.
- **Criterion:** Sequential runs use fresh sessions and release reservations; an
  occupied agent returns Busy. Unpolled futures send nothing; setup/turn failure
  and cancellation follow existing connection invalidation and bounded cleanup.
  Captures remain borrowed during execution and are released on completion/drop.
- **Evidence:** Local pass on both Tokio runtime flavors: sequential runs use three
  distinct session credentials, release listeners/reservations, and borrow Rc/local
  mutable state across await. Dedicated tests drop runs during remote setup and a
  pending callback, verify connection invalidation and child reaping, and exercise
  setup and missing-result failures. A compile-fail Rustdoc prevents capture reuse
  before completion. Existing lifetime/runtime teardown tests remain unchanged.

#### Teaching and delivery {#one-shot-tools-1-delivery}

- **Anchor coverage:** None — documentation and delivery.
- **Criterion:** Rustdocs give concrete examples for both conveniences, the sentiment
  tutorial uses the new flow, existing persistent-session and scheduler tests pass,
  and a live one-shot demonstration, CI, review and owner integration are recorded.
- **Evidence:** Local pass: the updated tutorial, both new API examples and ownership
  Rustdoc compile; all six sentiment and eight scheduler tests pass. A bounded live
  one-shot sentiment run scored all three passages and returned Mixed tone. Final-head and merge-commit CI passed; the owner reviewed and merged PR #8.
  See the integrated completion record below.


### Implementation and validation record

Tested on macOS/Rust 1.100.0 from 8e039b1215d96a23af4f6c798704f97c013fea25 plus
the uncommitted one-shot implementation. Plan::tool reuses Tool::new and bind.
Plan::run checks retained errors, options and the result contract before opening
a session with bound descriptors, then calls the existing run_in path. Descriptor
registration shares the existing duplicate-name validation. Session/turn cleanup
and transport infrastructure are unchanged. No new dependencies were added.

Passed checks:

- `cargo check --workspace --all-targets --locked`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked` (including 22 conversation tests and bootstrap)
- `cargo test --doc -p thinkwell --locked` after final API examples/ownership docs
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked`
- Sentiment example tests and all-target strict Clippy with its locked manifest
  and `CARGO_TARGET_DIR=$PWD/target/sentiment-analysis`
- Scheduler example tests with its locked manifest and previously recorded CMake
  and target directory settings
- Root and sentiment formatting; Python capability-probe regression; Book structure,
  mdBook rendering, whitespace, and tutorial relative-link checks.

The live executable used the default agent model and Claude ACP adapter 0.23.1,
with an outer 180-second timeout. It exited successfully after three real tool calls,
returning approximately 0.848, -0.637, and 0.000 compound measurements and a Mixed
report. This is single-configuration evidence, not universal agent compatibility.

A separate author self-review checked the full implementation, API docs, example,
tests and Book diff against the criteria. It confirmed run_in cannot acquire new
tools through inline syntax, preflight precedes temporary setup, and temporary
session drop uses existing ownership machinery. The tutorial explicitly explains
fresh history and the explicit-session path for model selection. No unresolved
findings; this is not independent or owner review. The retained model-choice and
one-session restrictions are intentional, not new ergonomic issues to solve here.
No commit, push, publication or phase transition is implied by local validation.

## Integrated completion

The owner reviewed and merged [PR #8](https://github.com/dherman/thinkwell-rs/pull/8),
head `da10a45948b9455352c0767d55cda0a36bf5fe52`, as
`43c147c250efd2c5fb1a1d8db57df0b03991c134`. All three final-head CI jobs passed
(runs `44915979688` and `44915974137`); merge-commit run `44917077737` also passed.
The owner reported CI and review passed without issues. Together with the local
criterion evidence above, this completes the bounded milestone. This Book
synchronization is a subsequent local change and is not yet integrated.
