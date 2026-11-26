# Progress observation design

Roadmap identifier: `progress-observation`.

## Milestone 1. Review progress through application examples {#progress-observation-1}

**Identifier:** `progress-observation-1`

**Status:** Complete

**Outcome:** Review a small optional observation API through console, borrowed-tool,
error and cancellation examples, with event meanings and delivery limits explicit.

**Assignment:** After owner-merged PR #9, the owner approved synchronizing Codex
completion and preparing example-driven progress/diagnostic sketches with “Agreed,
proceed”. This design assignment became Ready, was completed and owner-reviewed, and is integrated in PR #10. It does not
authorize product implementation, a public streaming contract, commit or push.

**Design basis:** `R-progressive-control`, `R-easy-agent-programming`,
`R-code-prompt-ergonomics`, `D-reviewed-api-baseline`, the earlier
[integrated examples](integrated-api-review.md), and the implemented
[typed-conversation baseline](../design/typed-conversations.md).

**Tasks:** Refresh migration evidence; compare execution surfaces; show concrete
examples; specify borrowing, failure, disclosure and buffering; obtain owner
review before proposing an implementation milestone.

### Completion criteria

#### Caller examples {#progress-observation-1-examples}

- **Anchor coverage:** None — design preparation, not a product guarantee.
- **Criterion:** Show ordinary typed answers, optional progress, a borrowed tool
  alongside separate observer state, persistent history, observer failure and
  cancellation with borrows released.
- **Evidence:** The original review sketches covered these cases; their Git revision
  and incorporated design are linked below. They supplied no compiled API evidence.

#### Behavior and scope {#progress-observation-1-behavior}

- **Anchor coverage:** None — proposed semantics need review before adoption.
- **Criterion:** Distinguish agent reports from host facts, typed completion from
  progress, control from observation, and bounded delivery from a complete audit.
- **Evidence:** The reviewed proposal distinguished these roles, buffering limits,
  exclusions, and implementation questions. The accepted definition is linked below.

#### Owner review {#progress-observation-1-review}

- **Anchor coverage:** None — delivery of design for review.
- **Criterion:** Present alternatives and recommended choices; record owner review
  before accepting the new surface or assigning implementation.
- **Evidence:** The owner approved the proposal after agreeing to non_exhaustive
  enums and private payload fields/accessors. Local structural and rendering checks pass;
  no product code, dependency or live-agent test was added for this assignment.

## Review outcome and incorporation

The owner approved Plan::observe followed by run/run_in, a local synchronous
borrowed observer, non_exhaustive event kinds, and private payloads with accessors.
The [owning design](../design/progress-observation.md) defines the selected wrapper,
events, disclosure limits, buffering, fairness, and cleanup. The awaited typed
result remains authoritative; observation neither authenticates a delayed message's
originating turn nor supplies a reasoning feed or permission-policy hook.

The three original [review sketches](https://github.com/dherman/thinkwell-rs/blob/ab1e8212455c25837f1b9805ad0f666b1ba18300/book/implementation-roadmaps/progress-observation.md)
showed a console answer, distinct tool/observer captures, and persistent history
with fallible logging and timeout cancellation. They were uncompiled review
artifacts; crate Rustdocs and the progress example provide compiled uses.
Application logging failures remain application state. Synchronous callbacks must
return promptly; an async timeout cannot preempt them. Panic follows ordinary
unwinding and guards, with no destructor guarantee under panic=abort.

The reviewed alternatives—an execution-first vocabulary, independent async stream,
async/fallible observer traits, and additional cancellation handles—were not selected
for this bounded surface. Its implementation evidence remains below.


## Milestone 2. Implement optional progress observation {#progress-observation-2}

**Identifier:** `progress-observation-2`

**Status:** Complete

**Outcome:** Applications observe bounded progress through borrowed callbacks while
retaining typed results and existing run/run_in ownership and failure behavior.

**Assignment and basis:** The owner approved this proposal and instructed “let's
proceed”, including non_exhaustive event kinds and private payloads with accessors.
This bounded milestone became Ready, was assigned, and is integrated in PR #10. Governing basis is the
same requirements and reviewed API baseline as milestone 1. Implement the proposed
ObservedPlan wrapper, initial event set, bounded queue and independent control path;
no stream receiver, cancellation handle or agent-level observer is assigned.

**Tasks:** Add public API/Rustdocs; connect host and agent events to local observation;
verify borrowing, overflow, panic/drop/failure paths; run both live agents; reconcile
accepted design and evidence. Commit/push and delivery require separate authorization.

### Completion criteria

#### Observation and ownership {#progress-observation-2-ownership}

- **Anchor coverage:** `A-scoped-turn-ownership`; `A-owned-resource-cleanup`.
- **Criterion:** Compiler/runtime tests demonstrate non-Send borrowed observers and
  tools, preflight/unpolled behavior, callback capture release, panic/drop cleanup,
  session isolation and no background observer execution.
- **Evidence:** Local compiler/runtime validation passed; see the implementation
  record below. Final-head CI passed and the owner merged PR #10; see integrated completion below.

#### Events and typed completion {#progress-observation-2-events}

- **Anchor coverage:** `A-progress-observation`; `A-typed-turn-completion`; `A-turn-envelope-routing`.
- **Criterion:** Deterministic tests cover messages, partial agent tool updates,
  actual local tool events, accepted/rejected results, bounded overflow and tail
  delivery without changing validation, control or authoritative typed outcomes.
- **Evidence:** Deterministic event, loss, fairness and typed-outcome tests passed.
  Events remain best-effort diagnostics, with no authenticated originating-turn claim.

#### Teaching and delivery {#progress-observation-2-delivery}

- **Anchor coverage:** None — documentation and delivery.
- **Criterion:** Newcomer Rustdocs compile; both supported agents demonstrate console
  observation; regression/Book checks, author review, CI and owner integration recorded.
- **Evidence:** Rustdocs and both live-agent demonstrations passed locally. Author
  self-review and exact checks are recorded below; final-head CI and owner integration are recorded in integrated completion.


## Local implementation and validation

Implementation is based on `d1becc9aabd6058bc5cb1de8e5f87fdcbe2b196e` plus the
uncommitted progress-design tree, tested on macOS/Rust 1.100.0. No new dependency
or third-party source adaptation was introduced. The owner subsequently authorized commit/push and draft PR delivery.

The shipped candidate is Plan::observe returning non-cloneable ObservedPlan with
run/run_in. ProgressEvent, AgentToolStatus and ToolOutcome are non_exhaustive;
all event payloads have private fields and accessor methods. ResultAccepted also
has a payload (including submission number), so it can evolve without changing
variant shape. The live design owns the accepted observation anchor and limits.

Implementation refinements made explicit during review:

- Local tool events describe invocation processing, including input/output validation.
  Failed input validation can therefore produce ToolFinished(Failed) without entering
  the application callback. Real callback timeouts are distinguished from a callback
  returning an Error whose kind happens to be Deadline.
- ResultRejected exposes the submission number and a safe category-level reason.
  It does not copy arbitrary validator/Serde text that could contain submitted data.
  Structured validation paths are deferred rather than extracted from error strings.
- The queue's 64 KiB budget includes fixed event sizes and owned string lengths.
  Filtering internal result-tool updates uses separately bounded metadata (128 IDs,
  up to 512 bytes each). Metadata overflow omits subsequent agent-tool updates with
  loss accounting; control and host events continue independently.
- Early setup notifications without an established session ID cannot be safely
  attributed. Once the ID is known, remaining temporary setup can emit progress;
  an explicit session open remains outside run_in observation.

Tests cover local non-Send observer/tool captures and release, current-thread and
multithread execution, no work for unpolled/preflight failures, partial ACP updates,
session filtering, omitted internal-result/raw-input data, queue count/byte bounds,
loss summary and retained tail delivery, retained order while work is pending,
result correction/duplicates/cancellation/missing results, local input/callback
errors and real callback deadlines. Panic tests cover both pending work and the
completed-work tail: connection cleanup is initiated in both cases. Drop tests
confirm invalidation, no later callback invocation and subprocess reaping.

Local checks passed:

- `cargo check --workspace --all-targets --locked`
- `cargo test --workspace --locked`: 20 library unit tests, 32 conversation tests,
  39 Rustdoc cases and launcher/bootstrap tests
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test -p thinkwell --doc --locked`: 35 normal and four compile-fail cases
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked`
- Both standalone example suites: six sentiment and eight scheduler tests, using
  their locked manifests and documented target/CMake settings.
- Python capability-probe regression, rustfmt, whitespace, Book structural check
  and mdBook rendering. These establish local evidence, not Linux CI.

The new opt-in `crates/thinkwell/examples/progress.rs` used the default Claude
adapter/model, with an outer 180-second deadline and process-group cleanup. It
printed message and tool updates, invoked the borrowed add callback and returned
3 matching Rust's total, with 28 observed updates. A disposable copy under target
changed only the agent import/connection to Codex and passed the same assertions
with 22 updates. Both used existing installation/authentication; neither is part
of ordinary tests. These live runs preceded final payload-accessor refinements;
final Rustdocs/example compilation and deterministic tests cover the final shape.
Observed counts are sample output, not a stable event-count guarantee.

A separate author self-review inspected public evolution boundaries, capture
lifetimes, control/progress separation, queue bounds, internal-event filtering,
panic/drop handling and the complete code/Book diff. It identified and corrected
the distinction between actual callback timeout and application-supplied Deadline
errors, and verified tail-delivery panic cleanup explicitly. No unresolved local
implementation findings remain. This is author review, not independent or owner
review. Subsequent final-head CI and owner integration complete the delivery criteria, as recorded below.


## Integrated completion

The owner merged [PR #10](https://github.com/dherman/thinkwell-rs/pull/10), head
`2ec41594ed680475b8e14f16c061b2cbfea6aac4`, as
`ab1e8212455c25837f1b9805ad0f666b1ba18300`. Both final-head CI runs
`44998763714` and `44998759870` passed all three jobs: Rust and Book checks,
Meeting scheduler example, and Sentiment analysis example. The owner reported
merge and authorized the subsequent closeout and unminifier investigation.
Together with the design approval and criterion evidence above, this completes
both bounded milestones. Merge-commit CI has not been assessed here. The subsequent Book synchronization is present on main at 59b8bd5.
