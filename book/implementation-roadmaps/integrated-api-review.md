# Integrated API review

Roadmap identifier: `integrated-api-review`.

The library's current definition is in [Design](../design/index.md). The sketches
below retain the reviewed provisional basis and its open alternatives. In particular,
attachment, model discovery/mutation, and explicit cancellation handles are not
introduced into the supported library by this reconciliation. The initial bounded
scope and later approvals determine which parts are incorporated.

Current integration state: completed work was delivered through owner-merged
[PR #2](https://github.com/dherman/thinkwell-rs/pull/2). See the
[shared integration evidence](first-product-slice.md#integrated-completion).
Earlier local-stage evidence and pending-delivery statements below are historical;
they retain their tested scope and do not describe current integration status.

Goal: review one coherent candidate API through complete application workflows
before assigning another implementation experiment.

## Milestone 1. Prepare and review integrated examples {#integrated-api-review-1}

**Outcome:** Three worked examples show standalone use, a persistent conversation,
and embedding, including ordinary error cleanup and explicit review questions.

**Status:** Complete

**Identifier:** `integrated-api-review-1`

**Dependencies:** The owner approved preparing these three examples with “Sounds
good, proceed.” This is a bounded documentation/design assignment, recorded Ready
on that approval and In progress for preparation. It does not assign the suggested
typed-final-result prototype, product implementation, or publication.

**Design basis:** `D-acp-client`, `D-derived-contracts`,
`D-provisional-scoped-turns`, `D-connection-vocabulary`,
`R-easy-agent-programming`, `R-code-prompt-ergonomics`,
`R-progressive-control`, and `R-per-turn-result-types`;
[working design](../design/index.md) and
[experiment evidence and exploratory alternatives](acp-integration-prototype.md).

**Tasks**

- [x] Assemble three consistent caller examples with concrete result/tool types.
- [x] Include failure and cleanup paths and identify application helpers.
- [x] Review cross-example consistency, evidence limits, and unresolved choices.
- [x] Obtain owner review of the combined proposal before scoping implementation.

### Completion criteria

#### Standalone workflow {#integrated-api-review-1-standalone}

Show connection, a typed prompt with a borrowed tool, deterministic work after the
turn, and cleanup on ordinary success/error paths.

- **Anchor coverage:** None — provisional API review, no implemented guarantee.
- **Evidence:** The standalone example below supplies all five steps and preserves
  ordinary RAII cleanup and distinguishes optional awaited shutdown confirmation.
  Syntax and lifecycle behavior remain unimplemented.

#### Persistent conversation {#integrated-api-review-1-conversation}

Show reusable plans, fixed tool contracts, changed result types, optional progress,
and a concrete invalid-output/cancellation path within one conversation.

- **Anchor coverage:** None — provisional API review, no implemented guarantee.
- **Evidence:** The conversation example binds the same declared tool separately
  for two turns, clones a tool-free base plan, changes Summary to NextAction, and
  separates candidate acceptance from execution completion. Repair and cancellation
  defaults remain open. This example has not been compiled or run.

#### Embedded workflow {#integrated-api-review-1-embedding}

Show use of a host-managed connection and clearly assign initialization, I/O,
incoming routing, and cleanup responsibilities, including host failure.

- **Anchor coverage:** None — provisional API review, no implemented guarantee.
- **Evidence:** The embedded example describes a host connection-access adapter,
  shared caller-facing Agent interface, host-owned I/O and underlying shutdown,
  and cleanup limited to Thinkwell's work. No such adapter is implemented.

#### Integrated assessment {#integrated-api-review-1-review}

Make agreed directions, new candidate choices, failure behavior, and evidence
gaps distinguishable; deliver examples for owner review and record that review.

- **Anchor coverage:** None — design review only.
- **Evidence:** Local consistency assessment and checks are recorded below. On
  2026-11-19 the owner explicitly stated, “I reviewed integrated-api-review.md and
  approve of the design.” This completes owner design approval under
  `D-reviewed-api-baseline`. The design remains a provisional implementation basis
  with its explicitly unresolved choices retained, not verified behavior or an
  unspecified implementation assignment. Authorized integration remains pending.

## How to read these examples

These are review sketches, not a compiling crate or promised product API. The
Rust types use Serde/schemars as concrete illustrations of the derives used in
experiments, without selecting the final dependency/validation contract. Types
such as RunError and CleanupError describe proposed roles, not existing exports.

Agreed directions are ACP clients, derives, varying per-turn result types, the
provisional scoped-turn/fixed-tool-catalog design, and connect/open vocabulary.
Typed preset conversion, a stable result envelope, observe/cancellation methods,
borrowing sessions from Agent, and the attachment/cleanup APIs below remain
the reviewed provisional implementation basis under `D-reviewed-api-baseline`.
The owner has explicitly approved this chapter's design. Its marked open choices
and evidence gaps remain; no additional design sign-off is required to propose
and approve the bounded foundation milestone.

The owner found explicit cleanup difficult to get right, the finish helper
misleading, and callback scopes unnecessarily nested. The revised candidate uses
ordinary RAII for the default API, with optional awaited shutdown when confirmation
or cleanup errors matter. The owner approved the RAII-based design as the
provisional basis; the earlier scoped-callback alternative is not the chosen
ordinary path. The cleanup guarantees still require implementation and evidence.

Distinguish local resource release from confirmed graceful remote shutdown.
Ordinary Drop is synchronous: it can revoke local routes, release borrowed
callbacks, close channels, and request cancellation through an already-running
owned driver. It cannot await a remote acknowledgment or report a cleanup error
through the return value of code that has already completed. Drop must not block
on the async runtime or rely on spawning a fresh task during runtime teardown.

The candidate driver owns transport/process cleanup state, never application
callback borrows. It performs bounded graceful shutdown while it can be polled;
an owned-process guard provides a force-termination fallback when that driver is
dropped. Automatic cleanup therefore does not mean graceful cleanup has completed
when a handle's destructor returns. A supported runtime/driver arrangement, fallback
error reporting, ownership-cycle avoidance, and shutdown races require design and
tests. Do not claim guaranteed remote termination merely because a signal was sent.

Successful run still requires a valid result and settled turn/local callbacks.
Destruction of idle sessions and connections is a separate resource-lifetime
operation; ordinary code need not explicitly await it. An optional
`agent.shutdown().await` can request bounded confirmed shutdown and expose errors.
Dropping a turn future releases its borrowed callbacks and marks its session as
not ready for immediate reuse. A later run could internally await bounded settlement
before sending another prompt, or return a recovery error; it must not silently
reset history or race the old turn. That automatic-settlement option is untested.

`ui.on_cancel` remains an application helper that registers a cancellation handle
and unregisters on subscription drop. The illustrative AppError converts connection,
session, and run failures. Failed constructors must handle partial acquisition.
Callback bodies requiring their own asynchronous cleanup cannot be completed by
ordinary Drop; this proposal does not supply async destructors for application code.

Shared domain types:

```rust,ignore
#[derive(serde::Deserialize, schemars::JsonSchema)]
struct Finding {
    note: String,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
struct Summary {
    issue: String,
    needs_follow_up: bool,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case")]
enum NextAction {
    Ask { question: String },
    Route { team: String },
}
```

The agent must have been installed and authenticated. A typed preset is launch
configuration, not a claim of bundled installation or universal model support.
No model override is needed for these examples.

## Example 1: standalone investigation

```rust,ignore
async fn investigate_once(document: &str) -> Result<Summary, AppError> {
    let agent = Claude::connect().await?;
    let mut findings = Vec::new();

    let summary = Plan::<Summary>::new()
        .text("Summarize the report and record supporting findings.")
        .quote(document, "report")
        .tool("record", "Record a finding", async |finding: Finding| {
            findings.push(finding);
            Ok(())
        })
        .run(&agent)
        .await?;

    save_findings(&findings);
    Ok(summary)
} // Agent is dropped; its owned resources enter automatic cleanup.
```

No explicit shutdown, callback scope, or result-combining helper is required.
The run future owns the local callback; connection infrastructure owns only
request/reply plumbing. The implicit session ends when its managed run completes.
The Agent may be reused for further calls before its normal lexical lifetime ends.

**Failure example:** the agent finishes without a valid Summary. The question mark
returns the run error and ordinary destruction releases local resources and
initiates owned-agent cleanup. Earlier effects are not rolled back. Graceful
remote shutdown may continue after the function returns; if the runtime is torn
down, the driver must fall back to owned-resource guards. Confirmed graceful
shutdown is not implied by function return. A caller needing confirmation can
opt into an awaited shutdown operation and its explicit error handling.

**Review question:** does this separation meet expected automatic cleanup behavior?
The next lifecycle assessment must exercise early return, dropped execution,
driver/runtime teardown, and cleanup failure; normal-path examples alone cannot
establish these guarantees.

## Example 2: persistent conversation with progress

The helper below borrows the connected Agent. RAII handles its local Session lifetime,
so it works in both standalone and embedded contexts. Tool contracts are declared
once; mutable callback implementations are supplied separately for every turn.

```rust,ignore
async fn conversation(
    agent: &Agent,
    document: &str,
    findings: &mut Vec<Finding>,
    ui: &Ui,
) -> Result<NextAction, AppError> {
    let record = Tool::<Finding, ()>::new("record", "Record a finding");
    let mut session = agent.session().tool(&record).open().await?;
    let base = Plan::<Summary>::new()
        .text("Summarize the report and record supporting findings.");

    let summary = base.clone()
        .quote(document, "report")
        .bind(&record, async |finding| {
            findings.push(finding);
            Ok(())
        })
        .run_in(&mut session)
        .await?;

    // The first turn's mutable capture has ended.
    deduplicate_findings(findings);
    show_summary(&summary);
    let instruction = if summary.needs_follow_up {
        "Ask one question that would help diagnose the issue."
    } else {
        "Choose the appropriate support team."
    };

    let execution = Plan::<NextAction>::new()
        .text(instruction)
        .bind(&record, async |finding| {
            findings.push(finding);
            Ok(())
        })
        .execute_in(&mut session);

    let _subscription = ui.on_cancel(execution.cancellation_handle());
    let action = execution.observe(|event| ui.render_progress(event)).await?;
    Ok(action)
}
```

The unchanged session catalog contains record and the candidate internal result
submission tool. Under the envelope proposal, each turn receives a fresh identifier
and schema: Summary on the first turn, NextAction on the second. The second turn
uses retained conversation context; it does not require replaying document input.
The base plan has no captures and can be cloned independently of the active session.

The second record binding is explicit because the fixed catalog still advertises
that tool. This makes the conservative missing-binding/preflight policy visible;
it is an ergonomic cost, not proof that the policy is the right default. Shared
owned bindings, deliberate unavailable tools, and dynamic catalogs could improve
particular cases but remain outside this candidate surface.

**Invalid-output example:** a second-turn candidate of
`{"action":"ask","team":"desktop"}` lacks question. Under bounded correction,
Thinkwell returns a validation error and the observer can report the rejected
candidate; the agent may submit a corrected value. Under fail-fast behavior,
the run instead returns an error and the helper drops its session on return. This example deliberately
does not choose correction defaults or invent a configuration method for them.
Serde conversion and full advertised-schema validation are not interchangeable.

**Cancellation example:** the user presses Cancel while the agent is correcting
its result. The control handle requests cancellation of this execution. The await
continues driving the turn and callbacks toward a bounded terminal outcome. Only
then does execution return its cancellation/settlement outcome. Session Drop
revokes local registrations and requests any remaining protocol teardown.
The findings accumulated so far remain accessible to the caller after the helper
returns. The observer does not borrow findings; it receives event summaries.

A valid result candidate followed by cancellation is not automatically successful
NextAction. Completion/cancellation races, deadlines, and outcome names need an
explicit policy. If settlement cannot be confirmed, the run reports that limitation;
Session Drop revokes local access without claiming remote cleanup is complete.
Dropping the whole helper invokes automatic cleanup without an awaited outcome.
A retained session cannot start another turn before settlement/recovery. This helper
ends its local session on return, with remote teardown handled as described above.

**Review questions:** does explicit rebinding read naturally? Is ordinary `Result<T,
RunError>` sufficient, with detailed events optional? Is bounded correction the
right default? These need a joint review of the successful and interrupted workflows.

## Example 3: embedding in an existing application

```rust,ignore
async fn investigate_in_host(
    host: &Host,
    document: &str,
    findings: &mut Vec<Finding>,
    ui: &Ui,
) -> Result<NextAction, AppError> {
    // Host-specific adapter; supplies more than a raw request sender.
    let access = host.share_initialized_acp().await?;
    let agent = Agent::attach(access).await?;
    conversation(&agent, document, findings, ui).await
    // Drop releases this wrapper's registrations, not the host's connection.
}
```

This candidate uses a common caller-facing Agent handle so conversation helpers
can be reused unchanged. The explicit attach constructor grants access without
ownership of the host's transport/process. Drop revokes Thinkwell-owned access and
requests cleanup of its registrations/work without shutting down the host's
connection. Owned-agent Drop additionally initiates shutdown of its own connection
and process. Ownership-mode types remain an alternative if the shared interface
cannot make these rights clear and enforceable. Attachment and driver cleanup
contracts remain unimplemented proposals.

`Host` and share_initialized_acp are application integration helpers. Their adapter
must establish the following division of responsibility:

| Responsibility | Owner in this example |
| --- | --- |
| ACP transport and agent process | Host |
| Initial ACP handshake and negotiated capability state | Host; Thinkwell consumes validated access to that state |
| Continually polling connection I/O | Host |
| Multiplexing inbound requests/notifications | Host adapter, with scoped routes for Thinkwell's sessions/tools |
| Driving this plan and borrowed callback dispatch | Awaited Thinkwell execution |
| Removing Thinkwell routes and settling its calls | Thinkwell, through the adapter |
| Terminating the underlying shared connection | Host |

Already initialized access avoids a second handshake. A cloneable SDK connection
handle by itself is insufficient: it may not provide incoming routing, capability
state, or cleanup rights. The adapter must not install competing global handlers.
The exact public transport abstraction and SDK dependency boundary remain open.

**Failure example:** the host's connection stops while a turn is active. Thinkwell
reports the failure, releases its local callbacks, and attempts to remove its
registrations. The wrapper does not restart the host's agent, silently reset
conversation history, or acquire shutdown rights. If the host only stops polling
I/O without reporting closure, deadlines must bound the wait; borrowed callbacks
do not excuse an indefinitely pending result. Awaited run/settlement failures remain
visible to the caller; asynchronous destructor-initiated cleanup errors need a
separate diagnostic path or an optional awaited detach operation. They cannot be
returned retroactively through the completed run.

**Review questions:** can a shared Agent interface express ownership clearly, or
should attached handles have a distinct type/cleanup method? Does the adapter have
a sufficient contract to isolate routing and cleanup? The embedding example is
conceptual; it has no compile or live-host evidence.

## Cross-example assessment

The examples preserve the same Plan, Tool, Session, and per-turn result model.
They introduce control only when needed: inline callbacks for independent runs,
explicit contracts for persistent sessions, execution handles for progress/cancel,
and a connection-access adapter for embedding. Sessions are borrowed from their
Agent in this candidate, so shutdown follows the end of their use. Helpers never
require callbacks to be static merely to cross a function boundary.

The combined examples expose four substantive review points:

1. The owner objected to explicit cleanup bookkeeping and callback nesting.
   RAII now forms the proposed ordinary path. Local release, driver-owned graceful
   shutdown, force-termination fallback, and optional awaited confirmation need
   assessment together before claiming a complete lifecycle contract.
2. Fixed catalogs make tool requirements predictable but require per-turn bindings
   even when a particular turn is unlikely to call the tool.
3. The stable envelope supports changed result types without catalog refresh, but
   generation guidance, validation, repair, and terminal completion remain untested.
4. A common Agent handle simplifies helper reuse across owned/attached connections,
   but cleanup authority must be clear and enforceable; raw transport access is
   insufficient for embedding.

No all-or-nothing approval is assumed. Review these choices using the examples,
then refine the candidate and scope the next experiment. The recommended next
prototype remains two different typed final results in one real session with an
unchanged catalog, plus deterministic malformed/stale/duplicate/cancelled-result
cases. Preparing this review does not authorize that implementation.

## Evidence and completion record

The implementing agent reviewed these documentation examples for consistent method
names, per-turn capture release, fixed-catalog bindings, result-type variation,
ordinary error cleanup, and host ownership. The owner reviewed the explicit-cleanup and callback-scope shapes and raised the
concerns recorded above. The owner subsequently reviewed the RAII-based proposal as a good starting
point and then explicitly approved this chapter on 2026-11-19. This is
design-approval evidence, not implementation verification. All Rust blocks are marked ignore for review; none has compiled
against a product API. Live integration evidence remains confined to the earlier
roadmap: typed tool calls and a receipt in prose, not typed final results or host
embedding. No new model calls, implementation, dependencies, or runtime tests were
introduced for this documentation assignment.

Validation on the uncommitted port-design tree based on
83ac7e6faf7b4e7dfd74c306436760e44ba46a21:

- `cargo anchor check --config anchorbook.toml`: structural/v1 passed, zero errors.
- mdBook 0.5.4 build: passed; new chapter, navigation, and affected local links and
  fragments inspected successfully. The sketches remain uncompiled.
- `cargo test -p anchor-runner --test bootstrap_consistency --locked`: two tests passed.
- `git diff --check` and separate new-file whitespace inspection: passed.

Preparation and owner design review are locally complete; status remains In
progress pending authorized integration. The approved
[runtime and result foundation experiment](runtime-results-prototype.md) is the
current scoped implementation assignment. No commit, push, publication, or lifecycle transition
is authorized or performed by this work.
