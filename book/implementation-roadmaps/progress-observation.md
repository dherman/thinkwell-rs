# Progress observation design

Roadmap identifier: `progress-observation`.

## Milestone 1. Review progress through application examples {#progress-observation-1}

**Identifier:** `progress-observation-1`

**Status:** In progress

**Outcome:** Review a small optional observation API through console, borrowed-tool,
error and cancellation examples, with event meanings and delivery limits explicit.

**Assignment:** After owner-merged PR #9, the owner approved synchronizing Codex
completion and preparing example-driven progress/diagnostic sketches with “Agreed,
proceed”. This design assignment became Ready and is now In progress. It does not
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
- **Evidence:** The sketches below cover these cases. They are uncompiled proposed
  APIs; existing names retain their current meanings.

#### Behavior and scope {#progress-observation-1-behavior}

- **Anchor coverage:** None — proposed semantics need review before adoption.
- **Criterion:** Distinguish agent reports from host facts, typed completion from
  progress, control from observation, and bounded delivery from a complete audit.
- **Evidence:** Event and lifecycle proposals below state these distinctions,
  buffering limits, explicit exclusions and implementation questions.

#### Owner review {#progress-observation-1-review}

- **Anchor coverage:** None — delivery of design for review.
- **Criterion:** Present alternatives and recommended choices; record owner review
  before accepting the new surface or assigning implementation.
- **Evidence:** The owner approved the proposal after agreeing to non_exhaustive
  enums and private payload fields/accessors. Local structural and rendering checks pass;
  no product code, dependency or live-agent test was added for this assignment.

## Reviewed proposal

The owner approved this direction, including a non_exhaustive enum and private
payload fields/accessors. The initial sketches below preserve the review context;
implementation names and guarantees are defined in Rustdocs and the live design.

### Recommendation

Add `Plan::observe(callback)` beside the existing tool bindings, then call run or
run_in as usual. Observation is optional; the awaited value remains `Result<T>`.
The callback is synchronous `FnMut(&ProgressEvent) -> ()`, may borrow local state,
and has no Send or static requirement. It runs on the task polling the execution,
never on the owned transport driver. Attaching an observer does no agent work.

This refines the earlier execute_in(...).observe(...).await sketch to fit today's
API. A separate execution type may be useful for explicit cancellation handles
or event receivers later, but is not necessary for console progress. No existing
run/run_in behavior or signature needs to change.

An observed plan owns its observer capture, just as a bound plan owns its tool
captures. It is not cloneable. Clone reusable unobserved plans first, then attach
an observer for each execution. Attach one observer per execution; applications
can fan out to multiple consumers inside that callback. The prototype should
assess an ObservedPlan wrapper that forwards run/run_in, leaving ordinary Plan
unchanged. Place observe last in the chain initially, rather than duplicating
all plan-building methods on that wrapper. These type choices are proposals.

## Example 1: a console answer with optional progress

These remain illustrative review sketches; the crate Rustdocs and progress
example provide compiled uses of the implemented API.

```rust,ignore
use thinkwell::{agent::Codex, Plan, ProgressEvent};

let agent = Codex::connect().await?;
let names = Plan::<Vec<String>>::new()
    .text("Suggest three names for a community garden.")
    .observe(|event| {
        if let ProgressEvent::Message(message) = event {
            eprint!("{}", message.text());
        }
    })
    .run(&agent)
    .await?;

// Progress text is not the validated answer.
for name in names {
    println!("Suggestion: {name}");
}
```

Removing observe leaves the current program. User-visible message chunks are
optional agent output: silence is not evidence of a hung execution. A spinner
can use the application's own timer. An Event enum should be non_exhaustive;
clients match only the events they need and include a fallback. This allows
future variants without requiring exhaustive matches to change.

## Example 2: a tool and an observer borrowing different state

```rust,ignore
use thinkwell::{agent::Claude, Plan, ProgressEvent};

let agent = Claude::connect().await?;
let mut total = 0_i64;
let mut activity = Vec::new();

let answer = Plan::<i64>::new()
    .text("Call add with 3 and report the total.")
    .tool("add", "Add to the total", async |amount: i64| {
        total += amount;
        Ok(total)
    })
    .observe(|event| {
        if let ProgressEvent::ToolStarted(tool) = event {
            activity.push(format!("Running {}", tool.name()));
        }
    })
    .run(&agent)
    .await?;

println!("Answer: {answer}; Rust total: {total}");
for line in activity {
    println!("{line}");
}
```

Both borrows end when the owned run future completes or is dropped. The observer
cannot read total while the tool exclusively borrows it. ToolStarted carries
metadata, not references to callback captures. Applications can explicitly copy
measurements into their own shared state if required; Thinkwell need not require
locks or static captures merely to show progress. The local activity Vec is
application-owned and can grow; a bounded library queue does not bound user logs.

## Example 3: persistent history, fallible output and cancellation

An observer's output destination can fail independently of the agent's work.
The recommended first version leaves that choice in ordinary application code:

```rust,ignore
use std::{io::Write, time::Duration};
use thinkwell::{Plan, ProgressEvent};

let mut session = agent.session().open().await?;
let mut log_error = None;
let mut output = std::io::stderr();

let summary = Plan::<String>::new()
    .text("Summarize the supplied document.")
    .json(&document)?
    .observe(|event| {
        if log_error.is_none()
            && let ProgressEvent::Message(message) = event
            && let Err(error) = write!(output, "{}", message.text())
        {
            log_error = Some(error);
        }
    })
    .run_in(&mut session)
    .await?;

if let Some(error) = log_error {
    eprintln!("Progress output failed: {error}");
}

// The same history, a different result type, and a fresh observer.
let mut updates = 0;
let outcome = tokio::time::timeout(
    Duration::from_secs(20),
    Plan::<Vec<String>>::new()
        .text("List the next actions based on that summary.")
        .observe(|_| updates += 1)
        .run_in(&mut session),
).await;

println!("Observed {updates} updates");
match outcome {
    Ok(result) => show_actions(result?), // An application helper.
    Err(_) => {
        // timeout consumed/dropped the future; captures are released.
        // If work had started, existing policy invalidates the session/connection.
        eprintln!("Timed out; reconnect before starting more work.");
    }
}
```

A synchronous callback must return promptly: blocking it stalls execution, and
an async timeout cannot preempt synchronous code. An observer panic follows normal
Rust unwinding; existing execution guards must revoke access and initiate cleanup.
Do not catch panics and silently report a successful typed result. Under panic=abort,
normal destructor guarantees do not apply. No async observer or fallible observer
trait is proposed initially. A caller wanting a logging failure to cancel work can
use application cancellation composition in a later example/assignment.

## Proposed initial events

| Event | Meaning and source |
| --- | --- |
| Message(MessageProgress) | An agent's user-visible text chunk, not a typed answer or a promise of complete transcript delivery. |
| AgentTool(AgentToolProgress) | Agent-reported tool activity, possibly native work. It does not prove that a Rust callback ran or that Thinkwell authorized it. Partial updates may omit metadata. |
| ToolStarted(ToolProgress) | Thinkwell is beginning a local application tool invocation, including validation. |
| ToolFinished(ToolCompletion) | Thinkwell observed invocation success, error or deadline; not rollback or a guarantee that an external effect finished. |
| ResultRejected(ResultRejection) | Host validation rejected a submission; another attempt may still succeed. No raw candidate value included. |
| ResultAccepted(ResultAcceptance) | A validated candidate exists; the turn may still fail before completion. |
| EventsDropped(DroppedEvents) | Some progress could not be delivered within the bounded observation budget. |

Opaque IDs correlate events within one execution, not across sessions. Agent and
host tool IDs need not match. Keep source distinctions in the public event shape;
do not fabricate host execution facts from ACP notifications. Status/outcome enums
also need non_exhaustive handling or an explicit unknown representation. Exact
field types and whether named event structs improve Rustdocs remain prototype work.

Omit raw tool arguments/results, complete prompts, HTTP headers and credentials
from the initial events. Agent text and titles may themselves contain application
data; an observer is an explicit choice to receive that content. Do not automatically
persist it. Internal return_result activity is represented by result events rather
than teaching users its wire name. No reasoning-text feed, token accounting, agent
plans, permission-policy hooks or raw ACP trace API is in this first scope.

The awaited Result remains the authority for final success/failure. Do not require
an observer to receive a Finished event to release resources or determine the
outcome. A missing-result error can be preceded by agent text and tool activity;
it remains MissingResult, without converting that text into a typed answer.

## Delivery and lifecycle proposal

- Drive the observer alongside the existing local execution/callback dispatcher.
  Execution remains lazy: dropping an unpolled run sends nothing.
- Keep transport/HTTP control independent of progress. Permission decisions,
  routing, validation, replies and failure notification never depend on an observer
  keeping up. The driver never waits on a full progress queue.
- Use bounded, best-effort progress delivery. A starting prototype budget is 128
  queued entries and 64 KiB of owned payload data per execution; oversized events
  count as dropped. Bound retained correlation metadata too. Count losses outside
  the full queue and deliver an overflow summary on the next opportunity.
- Preserve enqueue order for retained events, without claiming a total causal order
  across ACP and HTTP sources. Both agent and local tool activity may be shown for
  the same operation. Do not infer missing lifecycle events when delivery was lossy.
- Bound delivery batches so a chatty agent cannot starve callbacks or result handling.
  At normal completion, process the bounded retained tail and a pending loss summary
  before returning. Dropping/cancelling execution discards pending progress; no final
  observer call is guaranteed on Drop. Slow callback execution has no hard time bound.
- Unobserved runs allocate no progress queue or formatting work. Observers are never
  transferred into owned background tasks. Closing an observer route revokes delivery
  to that observer; stale notifications cannot reach a later execution's capture.
- Plan-local observation starts when that run is polled, including temporary session
  setup in run. Explicit session open precedes run_in and is outside that observer's
  lifetime: its errors remain ordinary open errors. Connection-level diagnostics and
  an independent event receiver are deferred.

These are proposals, not implemented guarantees. In particular, passing progress
through the owned driver into a local callback needs tests for fairness, teardown,
late notifications, overflow and failures during setup. ACP session notifications
lack a portable originating-turn ID; routing to the active execution cannot attest
that delayed agent text originated in that turn. Host-generated events can retain
their execution identity. Do not weaken existing envelope/cleanup guarantees to
claim stronger progress provenance.

## Choices for owner review

1. **Placement:** recommend plan.observe(...).run(...), compared with the earlier
   execute(...).observe(...).await. The latter offers room for more controls but
   introduces a second execution vocabulary now.
2. **Delivery:** recommend a synchronous borrowed observer with explicit loss
   reporting. A separate async stream permits asynchronous consumers but introduces
   another polling/ownership/buffering contract. It can be a later surface.
3. **Scope:** recommend user-visible messages, distinct agent/local tool activity,
   result validation feedback and overflow. Keep the final typed Result authoritative
   and defer lower-level tracing, reasoning text and cancellation handles.

After owner review, prepare a bounded implementation milestone covering the chosen
surface, compiler-checked borrowed callbacks, deterministic ordering/overflow/error
fixtures and live console examples with both supported agents. The owner subsequently approved proceeding; milestone 2 records the assigned
implementation and the live design chapter owns the accepted anchor.


## Milestone 2. Implement optional progress observation {#progress-observation-2}

**Identifier:** `progress-observation-2`

**Status:** In progress

**Outcome:** Applications observe bounded progress through borrowed callbacks while
retaining typed results and existing run/run_in ownership and failure behavior.

**Assignment and basis:** The owner approved this proposal and instructed “let's
proceed”, including non_exhaustive event kinds and private payloads with accessors.
This bounded milestone became Ready and is now assigned. Governing basis is the
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
  record below. CI and integration remain pending.

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
  self-review and exact checks are recorded below; CI and owner integration remain pending.


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
review; CI, publication authorization and owner integration remain pending. The
milestone remains In progress until its delivery criteria hold.
