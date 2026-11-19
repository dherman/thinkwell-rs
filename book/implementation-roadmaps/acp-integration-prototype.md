# ACP integration prototype

Roadmap identifier: `acp-integration-prototype`.

Goal: test whether the official ACP SDK can preserve scoped local tool ergonomics.

The [integrated API review](integrated-api-review.md) is the current entry point
for reviewing the combined proposal. This chapter preserves experiment evidence
and the detailed exploration behind it.

## Milestone 1. Assess SDK integration {#acp-integration-prototype-1}

**Outcome:** A bounded integration experiment compares local dispatch and direct
Send callbacks, assesses cancellation and teardown, and records agent capability evidence.

**Status:** In progress

**Identifier:** `acp-integration-prototype-1`

**Dependencies:** The owner approved the proposed four-part experiment with
“Proceed” in the port-design conversation. The preceding ownership experiment
supplies local API evidence; its pending integration does not block this explicitly
assigned experiment. No product dependency selection or publication is authorized.

**Design basis:** `D-acp-client`, `D-derived-contracts`, `R-code-prompt-ergonomics`,
`R-easy-agent-programming`, and `R-provider-breadth`; [SDK research](../open-questions.md#current-rust-acp-library-research).

**Tasks**

- [x] Build an isolated Cargo experiment using the official SDK 5.4.0.
- [x] Exercise a typed borrowing local callback and direct Send comparison.
- [x] Assess cancellation, failure, and teardown with an in-memory peer.
- [x] Inspect an available real agent's MCP and configuration capabilities.
- [x] Review results and reconcile the Book.

### Experimental scope

Use an independent, unpublished package under `experiments/acp-integration`, with
its own lockfile and build output under root `target/`. Experimental dependencies
are not product selections. Use stable ACP v1 and explicitly gated native MCP
support for a scripted peer. Keep application callbacks local and communicate
with SDK tool handlers using owned channel messages. Compare with scoped SDK
Send callbacks. Use original experiment code; dependency licenses remain theirs.
Real-agent probing is limited to available tooling and capability/configuration
requests; model inference is unnecessary. Record unavailable capabilities or
transport incompatibilities rather than claiming broad support. Product API,
HTTP bridge implementation, publication, and lifecycle transition are excluded.

### Completion criteria

#### Typed borrowing integration {#acp-integration-prototype-1-local}

A real SDK session dispatches serialized tool inputs to a local mutable callback,
returns its typed output, and releases application borrows after completion.

- **Anchor coverage:** None — scoped experiment, no product guarantee.
- **Evidence:** Passed `local_non_send_borrow_survives_await_and_returns_typed_output`:
  a callback exclusively borrows a local accumulator and uses Rc across await.
  Two serialized requests return totals 3 and 7; the caller then mutates the
  accumulator after execution. The SDK registry handles typed conversion.
  This is an in-memory native MCP session, not a real-agent tool round trip.

#### Direct Send comparison {#acp-integration-prototype-1-send}

A scoped SDK Send callback compiles and runs, with its caller syntax and lifetime
requirements compared explicitly to local dispatch.

- **Anchor coverage:** None — scoped experiment, no product guarantee.
- **Evidence:** Passed `direct_send_callback_borrows_without_static`: two SDK
  `tool_fn_mut` invocations borrow and mutate local state, return typed totals,
  and release the borrow afterward. This path requires a higher-ranked adapter
  `|callback, input, context| Box::pin(callback(input, context))`; the local
  dispatcher accepts the async closure directly with no Send bound.

#### Cancellation and teardown {#acp-integration-prototype-1-teardown}

Tests assess an in-flight callback's cancellation, failure and execution drop;
record effects, released borrows, and any distinction from remote cancellation
or graceful process shutdown. No unbounded waiting is permitted in the harness.

- **Anchor coverage:** None — scoped experiment, no product guarantee.
- **Evidence:** Five further tests passed: remote request cancellation drops an
  in-flight callback and allows a later call; execution abort/drop and foreground
  session finish release captures; malformed input and callback errors leave the
  session usable; a missing dispatcher reports an error without hanging. Prior
  mutations persist after cancellation. Each test has a five-second deadline.
  Drop guards establish synchronous destruction, not asynchronous cleanup joins,
  subprocess shutdown, rollback, or arbitrary concurrent cancellation behavior.

#### Agent capabilities {#acp-integration-prototype-1-agent}

Inspect an available real ACP agent for MCP transport/revision support and
optional model configuration; report exact tooling and observations, or an
explicit evidence gap if no suitable agent can be exercised.

- **Anchor coverage:** None — compatibility exploration, no supported-agent promise.
- **Evidence:** `probe_agent.py` exercised cached
  `@zed-industries/claude-agent-acp` 0.23.1 using Node 24.12.0. Initialization
  negotiated ACP v1 and advertised MCP HTTP and SSE, with no native ACP MCP
  capability. Session creation returned a model selector with `default`,
  `sonnet`, and `haiku`; setting its existing `default` value succeeded via
  `session/set_config_option`. No prompts or model inference were requested.
  Each request is bounded to 25 seconds; the temporary agent process group is
  terminated afterward. This separate JSON-RPC probe does not establish a Rust
  SDK real-agent round trip, actual model switching/inference, or negotiated MCP
  revision. HTTP support alone cannot establish compatibility with the official
  bridge's MCP 2026-07-28 requirement.

### Validation and review evidence

Executed on macOS arm64, Rust 1.100.0, on the uncommitted `port-design` tree
based on `83ac7e6faf7b4e7dfd74c306436760e44ba46a21`. The independent package
under `experiments/acp-integration` has its own Cargo.lock, pinning the official
SDK 5.4.0, schema crate 1.19.0, schemars 1.2.3, and Tokio 1.54.1. Tokio is an
experiment harness choice, not an accepted product runtime. Product workspace
manifests, dependencies, CI, and the earlier ownership experiment are unchanged.

Checks (run from the repository root):

- `CARGO_TARGET_DIR="$PWD/target/acp-integration" cargo test --manifest-path experiments/acp-integration/Cargo.toml --locked`: seven tests passed.
- `CARGO_TARGET_DIR="$PWD/target/acp-integration" cargo clippy --manifest-path experiments/acp-integration/Cargo.toml --all-targets --locked -- -D warnings`: passed.
- `cargo fmt --manifest-path experiments/acp-integration/Cargo.toml -- --check`: passed.
- `python3 experiments/acp-integration/probe_agent.py`: capability/session probe succeeded; zero prompts. Selected output is stored locally in ignored `target/acp-agent-capabilities.json`.
- `cargo anchor check --config anchorbook.toml`: structural/v1 passed, zero errors. mdBook 0.5.4 build and affected rendered local links/fragments passed.
- `cargo test -p anchor-runner --test bootstrap_consistency --locked`: two tests passed.
- `git diff --check`, new-file whitespace inspection, and Python syntax compilation: passed.

A separate self-review pass assessed source, criteria, and evidence scope. It
identified two important limits: the minimal service implements only tools/call
and uses binding errors for failures, and cancellation can return from the service
before the dispatcher observes the closed reply. Documentation now states both.
The channel has bounded queued messages but does not bound all waiting callers.
No production MCP server, asynchronous cleanup guarantee, nested-call guarantee,
or supported-agent promise is inferred. No source was copied or adapted; source
research informed API usage. No formal proof or mutation testing is assigned for
this bounded prototyping milestone. No independent or human review is claimed.

### Findings and recommendations

The local channel boundary works through the actual SDK and preserves direct
async-closure syntax with borrowed, non-Send state. SDK Send constraints need not
become universal application callback constraints. The direct Send path also
works and remains useful for callers whose execution needs to move across threads,
although this experiment only runs it on one thread.

Retain the local path as an API candidate while designing explicit cancellation
and cleanup semantics. An execution future that owns borrowed tools should have
a documented lifetime and polling requirement; cancellation is not rollback.
Before selecting a transport stack, test a complete tool round trip through an
HTTP or stdio MCP server supported by a representative agent, including its
negotiated MCP revision. The inspected agent did not advertise the native
transport used by the successful in-memory experiment. The owner subsequently assigned the bounded transport experiment below. Existing API, dependency, and runtime decisions remain open.

### Completion record

The bounded local experiment and capability probe are finished. Status remains
In progress pending owner review and authorized integration. Real-agent tool
transport/revision compatibility remains an explicit evidence gap. No commits,
pushes, publication, product implementation, or lifecycle transition occurred.


## Milestone 2. Exercise a real-agent tool round trip {#acp-integration-prototype-2}

**Outcome:** A real ACP agent calls a locally hosted tool over a supported MCP
transport, and its final reply demonstrates receipt of the tool output.

**Status:** In progress

**Identifier:** `acp-integration-prototype-2`

**Dependencies:** The owner approved the proposed complete tool round trip with
“proceed” following milestone 1's results. This explicitly assigns a bounded
experiment despite pending review/integration of earlier local experiments.

**Design basis:** `D-acp-client`, `D-derived-contracts`,
`R-code-prompt-ergonomics`, `R-provider-breadth`; milestone 1's ownership evidence.

**Tasks**

- [x] Connect the Rust ACP client to the available pinned Claude ACP agent.
- [x] Host a minimal loopback HTTP MCP tool with a scoped local callback.
- [x] Exercise tool discovery and typed calls, inspect revision negotiation,
      and verify final response and local state.
- [x] Assess bounded shutdown, review the experiment, and record limitations.

### Experimental scope

Extend only the independent experiment package. Use a temporary working directory,
one small inference request per run, a derived tool contract, and deterministic local
state. Permit only the experiment's tool; reject unrelated permission requests.
Use HTTP as advertised by the inspected agent; implement only the MCP subset
needed for this experiment, not a general server or the official polyfill.
Keep model/provider credentials in their existing agent-managed configuration.
Use bounded timeouts and shut down owned resources after success or failure.
Do not select product dependencies, add a product API, commit, or publish.

### Completion criteria

#### Real tool execution {#acp-integration-prototype-2-roundtrip}

Record exact agent and SDK versions, MCP revision negotiation, tool discovery,
serialized typed calls, callback effects, and final response containing a receipt
that was supplied only by the tool. If runtime or authentication prevents the
round trip, preserve that explicit gap rather than claiming success.

- **Anchor coverage:** None — experiment, no product compatibility promise.
- **Evidence:** Passed the live `http_roundtrip` executable with Rust ACP SDK
  5.4.0 and `@zed-industries/claude-agent-acp` 0.23.1, selecting its advertised
  `haiku` model through `session/set_config_option`. ACP initialization and
  session creation succeeded. The agent offered MCP `2025-11-25`; the HTTP
  fixture selected that revision, then observed initialized notification,
  tools/list, and two tools/call requests carrying that protocol header.
  Inputs 3 and 4 reached the Rust callback in order, leaving total 7. The agent's
  final message contained the fresh UUID receipt supplied only in tool responses,
  and ACP reported end_turn. The fixture checks receipt presence and local total;
  it does not parse the final prose's arithmetic as a typed model result.

#### Ownership and shutdown {#acp-integration-prototype-2-lifetime}

The callback borrows local mutable, non-Send state; the caller can access it after
execution. Bound agent work, shut down the HTTP listener, and record what cleanup
was observed versus inferred from library behavior.

- **Anchor coverage:** None — experiment, no product cleanup guarantee.
- **Evidence:** The live callback borrows a local Vec and uses `Rc<Cell<i64>>`
  across await through the same channel dispatcher as milestone 1. After its
  execution future is dropped, caller mutation of the Vec succeeds. The HTTP
  server's graceful-shutdown future joined successfully and a new connection to
  its former listener failed. Agent/session work has a 120-second inner deadline,
  SDK cleanup a 135-second outer bound, each local tool invocation a five-second
  bound, and HTTP shutdown a seven-second bound. No deadline fired in the passing
  run. SDK connection return was observed; child-process-group cleanup is based
  on the SDK's inspected implementation, not independent process-tree verification.
  HTTP disconnect cancellation and asynchronous callback cleanup are untested.

### Validation and review evidence

Executed on the same uncommitted `port-design` base as milestone 1, macOS arm64,
Rust 1.100.0. Experimental Cargo.lock now also pins axum 0.8.9 and tempfile 3.27.0;
the SDK's process feature is enabled. No product manifest, dependency, CI, or API
changed. Original experiment code uses the installed SDK and agent packages;
no third-party implementation was copied or adapted.

Commands and results:

- `CARGO_TARGET_DIR="$PWD/target/acp-integration" cargo run --manifest-path experiments/acp-integration/Cargo.toml --locked --bin http_roundtrip > target/acp-http-roundtrip.json`: passed, including live receipt, callback-state, and listener-shutdown assertions.
- `CARGO_TARGET_DIR="$PWD/target/acp-integration" cargo test --manifest-path experiments/acp-integration/Cargo.toml --locked`: ten tests passed (seven existing and three HTTP fixture tests). Tests do not run inference. HTTP tests cover derived contracts/revision negotiation, rejected requests before dispatch, typed borrowed-state dispatch, and the two-call budget.
- `CARGO_TARGET_DIR="$PWD/target/acp-integration" cargo clippy --manifest-path experiments/acp-integration/Cargo.toml --all-targets --locked -- -D warnings`: passed.
- `cargo fmt --manifest-path experiments/acp-integration/Cargo.toml -- --check`: passed.
- Book structural/v1 check, mdBook 0.5.4 build, and affected rendered links/fragments: passed.
- `cargo test -p anchor-runner --test bootstrap_consistency --locked`: two tests passed.
- `git diff --check` and new-file whitespace inspection: passed.

Two live prompts were sent, one per run. The initial run negotiated MCP and
listed tools but the fixture's permission argument matcher denied both requested
tool calls. Its exact argument mismatch was not captured. The correction checks
the exact experiment tool identity at permission time and validates values 3 or
4 at the HTTP boundary before dispatch. Other tools remain denied; the HTTP
fixture limits successful dispatch opportunities to two. The second run passed.
The first result remains locally in ignored
`target/acp-http-roundtrip-permission-failure.json`; the passing result is in
`target/acp-http-roundtrip.json`. Neither credentials nor full model transcripts
are included in these evidence files.

A separate implementing-agent review assessed source and criterion coverage.
The server is a narrow fixture: initialization, ping, tool listing, tool calls,
and initialized notification, with JSON HTTP responses and no persistent MCP
session. It binds loopback, checks an ephemeral bearer value, rejects browser
origins, and grants only the named experiment tool once per permission request.
These choices do not define Thinkwell's production permission or transport policy.
It does not implement general MCP conformance, SSE, HTTP cancellation, resumable
streams, or asynchronous callback cleanup. The SDK's stable process transport
and this HTTP fixture bypass the experimental native MCP binding. Tests and a
single successful agent/model combination do not establish provider breadth.
No independent or human review, formal proof, or mutation verification is claimed.

### Findings and completion record

The TS-style local tool experience and scoped Rust ownership are compatible with
a real ACP agent over HTTP MCP. Application callback futures need not inherit
Send or static bounds from the HTTP/ACP infrastructure; owned request/reply
channels can isolate those requirements. Derives supply the demonstrated input
and output contracts without an extra schema-generation build step.

The actual agent negotiated MCP `2025-11-25`. The inspected official polyfill
requires `2026-07-28` and does not fall back to older revisions. Therefore this
successful fixture is not evidence for selecting that polyfill; a production
transport must explicitly support and test the chosen revision(s).

The bounded experiment is locally finished. Milestone status remains In progress
pending owner review and authorized integration. Cleanup semantics, production
transport/library selection, typed final model results, and first-use API design
remain open. No additional experiment, product milestone, commit, push, or
publication is assigned by these findings.


### Provisional session and execution API interpretation

The owner requested continued API exploration after the successful transport
experiment. The owner subsequently endorsed this as the provisional working plan, retaining
explicit scope for re-review as implementation develops. Its governing record is
`D-provisional-scoped-turns`; the current design lives in
[scoped turns](../design/index.md#scoped-turns-and-explicit-conversations).
The interpretation below records its experimental basis; illustrative API spelling
is not implemented or accepted, and no additional implementation is assigned.

Separate four lifetimes: Agent owns or references the ACP connection; a reusable
Plan describes a request; Session retains conversation identity/history; an
Execution owns the active turn's scoped callbacks and borrows the session
exclusively. Attaching borrowed callbacks consumes a plan variation into scoped
work; it need not mutate or prevent cloning the original tool-free base plan.
Cloneability of reusable tool registrations remains open.

Candidate syntax is `plan.run(&agent).await` for a fresh conversation on a reused
connection and `plan.run_in(&mut session).await` for an explicit conversation.
These are illustrative signatures, not compiled evidence. Both paths should use
the same prompt composition and typed-result model. One active turn per mutable
Session is the proposed default; independent sessions provide concurrency.
Typed output alone would not establish that a turn or its callbacks have finished.
A successful run should settle the turn and release scoped callback borrows before
handing control back to deterministic code. This is a proposed contract still
requiring implementation and verification, including malformed/missing final output.

A session's history should not require retaining each turn's mutable captures.
The proposed split is session-lived tool contracts and transport registration,
with turn-lived callback implementations. Outside their active turn, routes must
not retain or invoke those callbacks. Subsequent turns may bind new closures to
the same declared contracts. Late calls must fail or be associated with the
correct turn; rebinding by name alone must not dispatch stale calls to new state.
MCP requests may not carry a Thinkwell turn identity, so the implementation must
establish a trustworthy boundary before rebinding. That behavior is untested.

The experiment supplied the MCP server at ACP session creation and advertised a
fixed catalog. Adding/removing tools between turns depends on agent discovery
and refresh behavior, which was not exercised. A conservative initial explicit
session could declare its available tool contracts before opening, then bind
callbacks for each turn; omitted bindings would report unavailable rather than
invoke an old callback. Alternatively, callbacks can be scoped to the entire
session, but then their captures remain borrowed between turns. The latter is
simpler but conflicts with convenient interleaving of deterministic mutations.
The fixed-catalog direction is now provisionally selected; exact catalog API
spelling and dynamic-tool capability remain open.

Cancellation has two distinct surfaces to consider. An explicit asynchronous
cancel-and-wait operation can request ACP session/cancel, stop new local dispatch,
and wait for a terminal prompt response and local completion under a deadline.
Dropping an execution can synchronously destroy local futures and release Rust
borrows but cannot promise completed remote or asynchronous cleanup. In both
cases prior effects persist. Cancellation must not masquerade as a valid typed
result; cancellation reason and whether the session can be reused are separate
facts. After an abandoned turn, the session should require recovery/settlement
or become unusable instead of silently accepting a new prompt while the old one
may still be active. Returning from a Rust borrow is not remote quiescence.

Source check: `agent-client-protocol-schema` 1.19.0 `src/v1/agent.rs` defines
session MCP registration on NewSessionRequest, session/cancel on
CancelNotification, and the terminal Cancelled stop reason on StopReason.
This supports distinguishing prompt cancellation from the earlier native MCP
request-cancellation test; no real-agent prompt-cancellation test has run.
Whether cleanup includes application async work and what happens on timeout
remain design choices. The current synchronous-drop evidence is insufficient
for a graceful asynchronous-cleanup guarantee.

The owner provisionally selected explicit sessions with upfront tool contracts.
The next discussion should refine declaration/binding syntax and whether
cancellation is represented as an ordinary error or a structured turn outcome. Then specify observable guarantees before assigning
further implementation. Defaults, installation, final-result validation, event
backpressure, multithreaded execution, and production transport selection remain
open; this proposal settles none of them.


### Session API refinement for discussion

These refinements continue the owner-endorsed provisional direction; they are
not additional accepted decisions or assigned implementation. Illustrative names
are not compiled API evidence.

A reusable `Tool<I, O>` value could carry a name, description, and derived input
and output contracts without owning a callback. A session builder registers the
contract before opening; a plan's `bind(&tool, async |input| ...)` provides a
turn-scoped implementation. An independent call can keep the existing inline
`tool(name, description, callback)` convenience and derive its catalog before
creating its ephemeral session. Ordinary callers need not manage MCP servers.
Contract values can be reused across sessions without sharing mutable callbacks.

Proposed local preflight checks reject a binding whose contract is absent from
the session, mismatched schemas for the same advertised tool, or duplicate
bindings. For missing bindings, prefer rejecting the turn before sending its
prompt by default; explicit unavailability could be a later opt-in behavior.
This is a refinement beyond the earlier unavailable-tool fallback and remains
for review. Never silently retain a previous turn's callback.

For the simple path, `run` / `run_in` should yield `Result<T, RunError>` so normal
application code can use `?`. A cancelled turn is not T. A more explicit execution
surface could expose the stop reason and cleanup disposition, with cancellation
requested through a separate control handle while the execution future continues
to be driven. Exact handle methods and outcome types remain open. A session's
ability to accept another turn must reflect settlement, not just whether the
caller received an error: a malformed result can follow a fully settled turn,
whereas a dropped execution may leave remote work running.

Suggested session states for further design are ready, running, needs settlement,
and closed/unusable. The mutable borrow prevents overlapping local runs; internal
state prevents reuse after the borrow ends prematurely. Do not force callers to
manage these states on the successful path. Explicit recovery may wait for the
old turn under a deadline; it must not silently create a new conversation while
presenting it as the old history. Recovery guarantees and stale-call isolation
remain unverified.

A fixed catalog also constrains typed final-result transport. If final values use
a schema-specific return_result MCP tool, successive plans with different output
types cannot silently replace that schema. The owner subsequently required
changing result types between turns (`R-per-turn-result-types`), excluding a
single fixed result type for a session. Declaring several result contracts up
front and a stable result envelope with per-turn validation remain alternatives.
The refinement below explores this boundary before freezing session generics
or final-result APIs.


### Per-turn typed result transport proposal

The owner accepted varying result types within one session under
`R-per-turn-result-types`. The following mechanism is recommended for discussion;
it is not an accepted transport/validation decision or an implementation assignment.

Prefer one session-lived internal result-submission tool with a stable object
input envelope containing an opaque turn identifier and a JSON value. Each turn
supplies its derived result schema and submission instructions as prompt context,
and installs a local validator/deserializer for its particular T. The result
schema describes the value being submitted *to* the tool; it is not the MCP
outputSchema of the tool's acknowledgment. Scalar, array, null, and object
results all fit inside the envelope's value field. Whether target agents accept
the generic value schema reliably requires testing; do not silently stringify
results or narrow supported shapes to obtain compatibility.

A fresh identifier binds submissions to the intended turn and rejects stale
result submissions before validating their values. It is a correlation mechanism,
not an authorization boundary or protection against an agent deliberately using
the current identifier. It does not solve stale calls to arbitrary application
tools, which lack this internal envelope. Identifiers and transport details should
stay out of ordinary caller code and final product-facing results.

Source observation: the inspected `agent-client-protocol-schema` 1.19.0
`src/v1/agent.rs::PromptRequest` has session_id, prompt content blocks, and metadata,
with no standard response-schema field. Arbitrary metadata alone does not
establish agent support for structured generation. A session-lived generic
submission tool plus per-turn prompt guidance is therefore a portable candidate;
it does not promise provider-enforced structured generation or model compliance.
An agent-specific structured-output feature or dynamic catalog refresh could
later be a capability-gated optimization with equivalent host validation.

Compared with declaring every possible result type before opening a session,
the envelope allows helper functions and ordinary control flow to introduce new
result types as needed. Upfront declarations could offer more specific
model-visible tool schemas but add setup and couple session creation to future
turns. Dynamic result-tool schemas offer specificity without upfront declarations,
but reliable agent catalog refresh has not been demonstrated. These remain
alternatives, not rejected implementations.

Proposed result acceptance pipeline:

1. Associate the submission with the active turn and apply input size/depth limits.
2. Validate the JSON value against the supported advertised contract, then
   deserialize into that turn's T. Serde deserialization by itself does not enforce
   every JSON Schema constraint; validation coverage and compatible derives must
   be explicit. Unsupported asserted constraints should be detected locally,
   rather than silently advertised as enforced. No validator or schema subset is
   selected yet.
3. On invalid input, return a tool-level validation error with actionable field
   paths so the agent can correct the submission in the same turn. Bound invalid
   submissions and the whole turn by explicit budgets. This does not automatically
   replay the prompt or promise that the model will avoid repeating side-effecting
   tools while correcting its answer.
4. Store one valid candidate and acknowledge it. Prefer rejecting further
   submissions after acceptance rather than silently replacing the result.
5. Return success only when the candidate, normal terminal prompt response, and
   local callback settlement requirements are all satisfied. A submission is not
   itself turn completion. Cancellation, timeout, and malformed/missing output
   remain distinct failures even if some partial data was received; precise
   stop-reason policy and outcome types are still proposals.

The simple API can remain `Plan<T>::run_in(&mut Session) -> Result<T, RunError>`
conceptually, with no result parameter on Session. Detailed execution observation
may expose intermediate submissions, validation feedback, and stop reasons; a
partial candidate must not be mistaken for the successful T. Borrowed output
lifetimes, custom domain validation, retry defaults, duplicate policy, and
cancellation races still require design and evidence.

Suggested next assessment, not assigned implementation: use one real session
with an unchanged catalog for an enum/record result followed by a differently
typed result; include malformed values, wrong-turn submissions, duplicate valid
submissions, a normal end with no submission, and cancellation after a valid
candidate. Use deterministic peers for exact failure coverage and a live agent
for interoperability. Prior experiments tested typed application tools and a
receipt in prose, not this typed-final-result mechanism.

### Worked result examples for design review

The owner requested concrete examples before judging the result-transport
proposal and authorized continued exploration. Future design choices should be
presented with caller examples and observable behavior. This does not accept the
stable-envelope proposal, validation/retry defaults, or new implementation work.
The following sketches are not compiled product APIs or executed agent exchanges.
Serde/schemars illustrate the derives already used in experiments; their product
selection and supported schema contract remain open.

Consider a support workflow with these per-turn result types:

```rust,ignore
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

The same conversation can return Summary and then NextAction, with deterministic
code choosing the second instruction:

```rust,ignore
let mut session = agent.session().open().await?;

let summary = Plan::<Summary>::new()
    .text("Summarize this customer report and identify missing information.")
    .quote(customer_message, "customer_report")
    .run_in(&mut session)
    .await?;

let instruction = if summary.needs_follow_up {
    "Ask one question that would help diagnose the issue."
} else {
    "Route the report to the appropriate support team."
};

let action = Plan::<NextAction>::new()
    .text(instruction)
    .run_in(&mut session)
    .await?;

match action {
    NextAction::Ask { question } => show_question(question),
    NextAction::Route { team } => enqueue_ticket(team, summary.issue),
}
```

Candidate hidden submissions to the same internal result tool:

```json
{"turn":"turn-a","value":{"issue":"App exits during startup","needs_follow_up":true}}
```

```json
{"turn":"turn-b","value":{"action":"ask","question":"Which operating system are you using?"}}
```

Thinkwell would supply each turn's schema and identifier, validate the value as
that turn's type, and return the typed result only after completion/settlement.
Callers would not construct these envelopes. The fixed MCP tool schema would
cover the envelope and generic JSON value, not either turn-specific result type.
Schema guidance in the prompt does not promise constrained generation.

For an invalid second submission such as
`{"turn":"turn-b","value":{"action":"ask","team":"desktop"}}`, the proposed
feedback identifies a missing `$.question` string. Unknown-field rejection is a
separate schema/Serde policy; the example does not depend on rejecting `team`.
The turn could correct its submission under a bounded repair policy, or expose a
validation error immediately under a fail-fast policy. Both must be demonstrated
before choosing defaults. Neither silently creates a new conversation or replays
an entire turn. Correction still permits model-initiated repeated application
calls unless separately constrained; earlier effects are not rolled back.

A valid NextAction submitted with turn-a must be rejected as stale, even if its
JSON shape matches the current type. A normal agent end with no valid submission
must fail as missing output. A valid submission followed by cancellation must
not silently become successful T under the proposed completion policy. These are
expected behaviors for design review, not test results.

For comparison, a predeclared-results alternative could require a builder such as
`agent.session().result::<Summary>().result::<NextAction>().open().await`.
It might expose a specialized result-submission tool for each declared type, but
couples opening the session to knowing future result types. A helper introducing
another result type would require changing that setup. This alternative is still
compatible with R-per-turn-result-types; rejecting it requires a separate design
choice. The stable envelope is recommended for composition, not accepted solely
because varying result types are required.

### Progress, cancellation, and failure walkthrough

The owner described the envelope preference as plausible and requested continued
exploration followed by review of how the proposal fits together. This is support
for exploring the candidate, not final selection of the envelope, retry policy,
or the API refinements below. No new implementation work is assigned.

Keep ordinary completion as `plan.run_in(&mut session).await -> Result<T, RunError>`.
An explicit execution surface could add progress and cancellation without changing
result typing, tool bindings, or conversation semantics:

```rust,ignore
let execution = plan.execute_in(&mut session);
let cancel = execution.cancellation_handle();

// Illustrative UI helper, not a Thinkwell API.
connect_cancel_button(move || cancel.request());

let outcome = execution
    .observe(|event| render_progress(event))
    .await;

// The owned execution has been consumed; scoped callback captures are available.
match outcome {
    Ok(report) => show_report(report),
    Err(error) => show_failure(error),
}
```

This sample proposes an awaitable execution and a synchronous observation callback,
not a spawned background job. Polling the execution drives protocol work and local
tool dispatch; constructing it or polling a separate event receiver does not.
`observe` would be a convenience for UI/logging callbacks; slow synchronous work
would delay progress and asynchronous observers would require a different surface.
An event-receiver alternative can follow the earlier experiment's split result and
events, with explicit concurrent polling. Neither that experiment's lossy queue
nor a production buffering policy is accepted by this example.

The cancellation handle carries no borrow of application data or mutable session
access. It requests cancellation of this specific execution; it cannot outlive
its logical target and accidentally cancel a later turn. Repeated requests are
idempotent and requests after completion have no effect. Awaiting execution still
drives settlement and releases callback captures. A request is not confirmation
that the model has stopped. The exact race policy when completion and cancellation
arrive together remains open; avoid promising that every late cancellation request
overwrites an already settled successful result.

Borrow release applies after the owned execution future has been consumed/dropped.
Awaiting a pinned mutable reference can leave completed future storage and captures
alive until that storage is dropped. Examples using select loops must demonstrate
the surrounding scope that releases those captures before inspecting tool-owned
application state. No unconditional early-borrow-release claim follows from Poll::Ready.

Progress should describe observations, such as a tool call beginning/ending or an
invalid result being corrected, without exposing mutable references into borrowed
tool state. An application cannot simultaneously read a Vec that its callback
exclusively borrows; it can instead render owned event summaries. Observation
should not be required for successful execution. Permission requests, result
validation, and cancellation acknowledgment are control operations, not optional
or lossy progress delivery. Event payload disclosure and backpressure remain open.

Proposed user-visible outcomes:

| Scenario | Result seen by the caller | Session disposition |
| --- | --- | --- |
| Invalid submission corrected within policy, then normal completion | Valid T; optional progress explains correction | Ready after settlement |
| Agent finishes normally without a valid submission | Missing/invalid-result error | May be ready if the turn and callbacks settled |
| Cancellation is requested and confirmed, local work settles | Cancelled outcome/error | May be ready after settlement; prior effects remain |
| Deadline expires without confirmed settlement | Deadline/cleanup failure, no successful T | Requires recovery or is unusable |
| Execution is dropped | No awaited outcome | Must not admit another prompt until settled/recovered, or must become unusable |

These are proposed semantics, not test results or final error names. Error kind
explains why a value was not returned; session readiness separately explains
whether another turn can start. A repair attempt does not itself end the turn or
prove readiness. An invalid final value need not destroy a settled conversation.
Cancellation and protocol failure must not implicitly reset history under the
same Session identity. Execution observation should not silently retry side-effecting
work or turn a partial candidate into the successful typed result.

Review points: whether synchronous `observe` is useful alongside an event receiver;
whether cancelled work is an ordinary RunError on the simple path or a distinct
outcome; what bounded correction defaults are appropriate; and how callers request
recovery after abandoned work. Demonstrate these through one coherent session
walkthrough before treating the surface as accepted or implementing it.

### Helper composition and concurrent conversations

This continues the authorized design exploration. The examples are proposed API
sketches, not compiled product code or additional accepted guarantees. Prefer
ordinary Rust helpers and async composition before adding a Thinkwell workflow
abstraction.

A helper can own an entire scoped turn and expose only its typed result:

```rust,ignore
async fn investigate(
    session: &mut Session,
    record: &Tool<Finding, ()>,
    findings: &mut Vec<Finding>,
    document: &str,
) -> Result<Summary, RunError> {
    Plan::<Summary>::new()
        .text("Investigate the report; record the supporting findings.")
        .quote(document, "report")
        .bind(record, async |finding| {
            findings.push(finding);
            Ok(())
        })
        .run_in(session)
        .await
}
```

The caller declares the record contract when opening the session, passes it to
the helper, and regains access to findings after the helper's owned future is
consumed. This does not require exposing closure types or adding Send/static
bounds merely to extract a helper. Helper signatures returning an unexecuted
scoped plan still need explicit borrowing/state types; that is separate from the
simple async helper pattern. A helper may introduce a new result type under the
stable-envelope candidate, but cannot silently introduce a new application-tool
contract into an already-open fixed catalog. Its required tools should be
visible in its parameters or configuration and checked before the turn starts.

A reusable tool-free base Plan can be cloned before attaching per-run callbacks.
Two variants can then borrow different application state. Cloning a base plan
copies request configuration, not conversation history or mutable tool captures.
Cloning an exclusive borrowed callback or implicitly cloning a Session is not
part of this proposal. Shared reusable callbacks remain a separate open design.

Independent conversations can be polled concurrently without spawning:

```rust,ignore
let (left_result, right_result) = futures::join!(
    investigate(&mut left_session, &record, &mut left_findings, left_document),
    investigate(&mut right_session, &record, &mut right_findings, right_document),
);

// Both operations have returned before either error is propagated.
let left = left_result?;
let right = right_result?;
combine(left, right, &left_findings, &right_findings);
```

Both sessions must have registered the required tool contract. Each run retains
its own local callback bindings and result validation. Interleaving futures on
one task need not require Send or static; it does not establish parallel CPU
execution, throughput, or agent support for multiple active sessions. The earlier
simulation interleaved runs; the live-agent test did not exercise concurrency.
Multiple sessions on one Agent have separate histories but may share a process
and connection failure domain. A connection-wide failure can affect both.

The choice of combinator has observable cancellation consequences. `join!` waits
for both results, including when one is an error. Fail-fast combinators such as
`try_join!` drop unfinished siblings when one fails; that invokes the proposed
execution-drop behavior rather than an awaited graceful cancellation. For
stop-on-first-failure with cleanup, application code would explicitly request
cancellation of the sibling and keep driving it to a bounded terminal outcome.
No special Thinkwell combinator or default orchestration policy is selected.

The mutable Session borrow rejects overlapping runs through the same session.
A nested model call from inside a tool must not reenter that already-borrowed
Session; a separate conversation would be required, with its own budgets and
lifecycle. Separate sessions alone do not prove freedom from application lock or
callback reentrancy deadlocks. Nested-agent execution remains untested and is not
promised by these examples.

Implications for the integrated design review: the simple surface composes through
ordinary async functions; reuse applies to request configuration; concurrency is
explicit and preserves scoped borrowing; cancellation behavior remains visible
at composition boundaries. A future ownership/API assessment should compile these
helpers, reject overlapping session borrows, and test sibling failure under both
wait-for-all and explicit cancel-and-settle policies before claiming those
behaviors. This assessment is not assigned by the design sketch.

### First use, agent configuration, and model choice

This is continued design exploration, not a selected product startup/configuration
API. The accepted goals favor a short first successful program and incremental
control. An explicit agent choice can coexist with agent-provided model defaults.
The names below are illustrative, with no implemented preset registry or automatic
installation behavior implied.

A typed named-agent convenience might look like (see the extensibility discussion below):

```rust,ignore
let agent = Agent::connect(agents::Claude::default()).await?;
let summary = Plan::<Summary>::new()
    .text("Summarize the report.")
    .quote(report, "report")
    .run(&agent)
    .await?;
```

The proposed convenience resolves an agent launch configuration; it does not
select a universal model/provider or hide who performs inference. Initial setup
still includes obtaining a compatible ACP agent and authenticating it. Installation
instructions, supported versions, and any named presets must be specified before
claiming this first-use experience works. A successful connection need not prove
authentication or all requested capabilities; some agents defer these checks to
session creation or inference.

A corresponding explicit path could be
`Agent::command("claude-agent-acp").connect().await`, with structured arguments
and environment overrides rather than requiring callers to construct a shell
command. Here the executable must already be available. Thinkwell could provide
setup helpers later, but silent downloads, interactive login during run, global
configuration locations, automatic agent selection, and a bundled agent catalog
are not selected. The experiment used a cached pinned npm package; that supplies
integration evidence, not a completed installation experience.

Suggested defaults are a fresh conversation for each independent run, reuse of
the established agent connection, agent-provided current model when no override
is requested, and the existing typed-result path. Optional model selection must
not become a prerequisite for the simplest call. Conversely, explicit model or
capability requests should not silently fall back to another choice. Any result
mechanism requiring an MCP tool needs a usable transport; ACP initialization alone
cannot establish typed-result compatibility.

For persistent sessions, model discovery and mutation belong to the session:

```rust,ignore
let mut session = agent.session().open().await?;
let choices = session.model_choices()?; // proposed owned snapshot
let chosen = choose_model(&choices);   // application/UI helper
session.set_model(chosen.id()).await?;

let summary = plan.run_in(&mut session).await?;
```

The model selector would reflect the agent's advertised configuration category
and opaque values, not a hard-coded global list. Display labels are not portable
IDs. Missing support, stale selections, multiple ambiguous selectors, and rejected
updates should be represented explicitly. The mutable Session operation would
permit changes between settled turns; it would not mutate a shared Agent's model
for every concurrent session. Whether all agents isolate that setting correctly
still requires tests. A fresh-session builder could offer a requested model,
implemented by creating the session, discovering/validating the choice, and
applying it before sending the first prompt.

Illustrative failure behavior for review:

- Missing executable: identify the requested command/profile and relevant setup
  instruction before a prompt is sent; do not pretend the model request failed.
- Authentication required: surface an actionable agent-specific setup result,
  without turning a headless call into an unbounded interactive login.
- Unsupported optional model control: default calls can still use the agent's
  current model; an explicit override returns an unsupported-control error.
- Advertised model choice rejected or no longer available: preserve that failure
  rather than quietly using another model.
- No compatible tool transport for the proposed typed-result mechanism: explain
  the missing capability; do not imply that speaking ACP alone is sufficient.

These categories are proposed behavior, not finalized error types or new tested
claims. Secrets and connection diagnostics need intentional handling; provider
credentials remain with the agent in the tested setup. Permission policy for
agent-native tools, working-directory defaults, profile storage, launch ownership
versus an externally supplied connection, and graceful Agent shutdown remain open.
The first-use walkthrough must eventually cover those without exposing protocol
plumbing to ordinary callers.

Evidence boundary: the live Claude ACP 0.23.1 experiment successfully selected
its advertised `haiku` value before one prompt and negotiated MCP HTTP 2025-11-25.
The earlier capability probe observed default/sonnet/haiku choices. This does not
establish portable IDs, mid-conversation switching, installation, credential setup,
settings isolation across sessions, or support for another agent. Review a complete
first-use example and explicit-control variant before choosing named presets or
configuration APIs. No implementation is assigned by this proposal.


### Extensible agent selection

The owner questioned the string-selector sketch as a Rust API and suggested a
trait-based extensibility boundary. This is an open design discussion, not an
accepted new trait or a requirement to reproduce TypeScript string constants.

Distinguish adding agent launch configurations from adding connection behavior.
For command-based agents, a standard conversion trait may suffice:

```rust,ignore
impl Agent {
    pub async fn connect(source: impl Into<AgentConfig>) -> Result<Self, ConnectError> {
        // Resolve configuration, launch, and initialize ACP.
    }
}

let agent = Agent::connect(agents::Claude::default()).await?;
let custom = Agent::connect(AgentConfig::command("company-acp")).await?;
```

These signatures and names are sketches, not implemented/compiled APIs. Built-in
preset structs would have private fields and constructors/builders, so their
internal configuration can evolve without exposing struct-literal construction.
Each converts into owned launch data (command, structured arguments, environment,
and other explicitly supported settings). AgentConfig itself satisfies Into via
Rust's identity conversion. New preset types need no new Agent::connect branch
or public closed union.

An application or extension crate can define its own source type and implement
`From<MyAgent> for thinkwell::AgentConfig`, thereby gaining `Into<AgentConfig>`.
That foreign-trait implementation is allowed because MyAgent is local to the
implementing crate; this does not allow arbitrary conversions between two
unrelated foreign types. The configuration API must expose the constructors
needed for these external implementations. A downstream custom preset need not
be registered in a central list or use an untyped name.

For example, an extension crate could supply a preset without modifying Thinkwell:

```rust,ignore
pub struct CompanyAgent;

impl From<CompanyAgent> for thinkwell::AgentConfig {
    fn from(_: CompanyAgent) -> Self {
        thinkwell::AgentConfig::command("company-acp")
    }
}

let agent = Agent::connect(CompanyAgent).await?;
```

This illustrates the extension shape, not an implemented API or a selected preset.

A custom public connector trait becomes useful if extensions need to implement
behavior beyond forming launch data: a distinct transport, asynchronous setup,
or integration with an externally managed connection. Its contract would need
lifecycle/cancellation/cleanup semantics and considered future/Send bounds, rather
than merely returning a command. Keep that boundary separate from the basic
preset conversion until its requirements are clear. Whether it should use SDK
traits internally or expose a Thinkwell-owned trait remains open.

Rust nuance: an exhaustive public enum cannot add variants without breaking some
callers, but an enum declared non_exhaustive from the outset permits future
variants while requiring external wildcard matches. It still cannot be extended
by downstream crates. A trait/conversion boundary better fits third-party presets.
Public traits also carry compatibility obligations: adding required methods breaks
external implementations, so a custom trait should remain small and avoid exposing
unstable SDK details without a deliberate compatibility policy.

Recommend exploring typed presets plus `Into<AgentConfig>` first, with a behavioral
connector interface only where needed. String names may remain useful in runtime
configuration files, resolved explicitly to typed launch data, rather than being
the sole Rust construction API. No config-file format, runtime registry, preset
catalog, automatic installation, custom connector trait, or implementation work
is selected by this recommendation.

### Preset connection convenience methods

The owner supported exploring typed preset conversion and suggested a trait
convenience for `Claude::default().connect().await`, possibly also
`Claude::connect().await` conditioned on `Self: Default`. This refinement is
provisional; no product implementation, final naming, or behavioral connector
contract is assigned.

A small extension trait can supply the instance convenience to every convertible
preset, including third-party types:

```rust,ignore
trait ConnectAgent: Into<AgentConfig> + Sized {
    async fn connect(self) -> Result<Agent, ConnectError> {
        Agent::connect(self).await
    }

    async fn connect_default() -> Result<Agent, ConnectError>
    where
        Self: Default,
    {
        Self::default().connect().await
    }
}

impl<T: Into<AgentConfig>> ConnectAgent for T {}
```

With this trait in scope, instances use `.connect()` and defaultable preset types
can use `Claude::connect_default()`. The Default requirement is on that one method,
not on the trait or blanket implementation; a configured-only type can still
connect its instance. Agent remains the connected handle in this sketch, so
ConnectAgent names the convenience trait. Renaming the handle/trait remains open.

A zero-argument associated `connect()` can instead have a method-level
`where Self: Default` bound and permit `Claude::connect()`. However, a single
trait cannot also define `connect(self)` with the same name: Rust does not overload
trait items by argument count. Distinct names avoid that conflict. An inherent
associated `connect()` on an individual preset can coexist with the extension
trait's instance `.connect()`; it requires a forwarding definition on each such
type and is an optional convenience rather than universal trait behavior.

Compiler-only language research used Rust 1.100.0 on the current uncommitted
port-design tree. Disposable sources and diagnostics are under ignored
`target/agent-convenience-probe/`. The positive source models the existing sketch
with stub Agent/AgentConfig types; it performs no I/O or connection work.

- `rustc --edition=2024 --crate-type=lib target/agent-convenience-probe/positive.rs --out-dir target/agent-convenience-probe -D warnings`: passed. Compiles instance conversion, method-specific Default shorthand, direct Agent::connect, configured-only instances, an associated-only `Claude::connect()`, and an inherent alias alongside the instance method.
- The missing_default variant calls the default shorthand on a non-Default type:
  rejected with E0277 as expected.
- The duplicate variant defines both connect forms in the same trait: rejected
  with E0428 as expected.

The negative cases were checked with the same compiler flags through a Python
subprocess harness with 30-second bounds. These results establish language/API
shape only, not public async-trait Send guarantees, real connection behavior,
package integration, or a new product milestone. Public trait visibility, future
bounds, and lint policy remain unresolved. No dependency was added for the probe.

The owner subsequently selected instance `.connect()` together with per-preset
inherent `Claude::connect()` shorthand, retaining open for sessions; see
`D-connection-vocabulary`. The connect_default and associated-only variants above
remain language research, not the selected naming direction. The blanket
implementation intentionally fixes this convenience to the standard configuration
connection path; downstream types cannot override it with an overlapping impl.
Custom connection behavior still needs a separately designed connector boundary.


### Connection ownership and embedding exploration

The owner agreed to the connection/session vocabulary and requested the next
phase of exploration. This means continued design work within the Porting Phase,
not a lifecycle transition or new product implementation assignment. The following
ownership refinements are proposals for integrated review.

The standard preset/configuration path launches an agent whose process and ACP
connection are managed by Thinkwell. A separate embedding path accepts access to
an application-managed connection. Ownership must be explicit: dropping a wrapper
around a shared connection must not accidentally terminate the application's
agent or disrupt unrelated clients. Custom connection behavior may require a
connector trait; ordinary command-based presets still need only config conversion.

For the owned path, favor sessions borrowing their Agent handle initially:

```rust,ignore
let agent = Claude::connect().await?;
{
    let mut session = agent.session().open().await?;
    let summary = plan.run_in(&mut session).await?;
    show_summary(summary);
    session.close().await?;
}
agent.shutdown().await?;
```

Here close/shutdown signatures are proposed, not implemented or accepted.
An exclusive shutdown consuming the owned handle cannot run while sessions still
borrow it. Multiple independent sessions can borrow the same Agent immutably;
each execution then borrows its individual Session mutably. This extends the
existing scoped-turn proposal one level outward without requiring session lifetime
to retain application callback borrows. Exact lifetime parameters and ability to
return sessions from helpers need a compiler/API assessment before selection.

An alternative is reference-counted connection ownership retained by sessions,
which makes detached session storage easier but raises questions about last-owner
cleanup and explicit shutdown invalidating live handles. Neither model eliminates
remote connection failure. Do not silently select Agent/Session Clone semantics
or a background runtime architecture through these examples. The borrowed-handle
model is recommended for initial assessment, not accepted as a new guarantee.

The successful-path example is not sufficient to guarantee async cleanup on error:
its question-mark exits can skip explicit close/shutdown. A deliberate cleanup
path must preserve the operation error while also reporting cleanup failure;
Drop alone cannot await. A scoped callback-style connection API could guarantee
that the library drives cleanup on ordinary callback return, but its nesting and
behavior under cancellation need comparison before adding another public surface.
The current SDK's connect_with foreground pattern provides relevant precedent.

Keep distinct lifecycle operations:

- Cancelling execution stops one turn; it does not automatically discard the
  conversation or close a shared connection.
- Closing a Session ends local use of that conversation and its tool registrations.
  Any agent-side session-close request depends on supported protocol capabilities;
  local closure does not promise deletion of history or universal remote cleanup.
- Shutting down an owned Agent settles/closes its connection and terminates any
  process it owns under the selected cleanup policy. It affects all work on that
  connection and is not a synonym for cancelling one prompt.

The embedded path needs a connection-access contract describing who performs ACP
initialization, supplies negotiated capabilities, routes inbound requests, drives
the I/O future, and owns shutdown. A raw cloneable SDK request handle alone cannot
establish all of these. Do not initialize twice or silently install competing
message handlers. Externally managed I/O remains the host's responsibility unless
ownership is explicitly transferred. Thinkwell still owns settlement of the work
and registrations it creates; detachment is not abandonment of callbacks.
No constructor or public SDK-dependent handle type is chosen yet.

A behavioral connector must therefore yield both usable connection access and an
explicit lifecycle/driver arrangement. Avoid a public trait that returns only a
request sender while leaving these responsibilities implicit. Also avoid copying
SDK Send/static bounds onto application tool callbacks: the earlier channel
boundary separates those concerns. Whether the connector abstraction is public,
sealed, or implemented through a lower-level adapter remains open.

Source observation: ACP SDK 5.4.0 src/acp_agent.rs documents normal connection
shutdown by closing stdin, waiting, and escalating process termination; dropping
an active connection kills its process group on Unix. This is library behavior,
not a selected Thinkwell policy. The live experiment exercised normal connection
return and verified HTTP-listener closure, not shared-connection ownership or
independent process-tree cleanup. Evidence for those boundaries remains absent.

Review examples should cover a standalone one-shot program, two sessions sharing
an owned Agent, and an embedding host with an existing initialized connection.
Exercise ordinary errors and early execution drop as well as the success path.
The open ownership choice affects connection lifetime and shutdown ergonomics,
not the accepted ability to change result types between turns. No new experiment
or implementation milestone is assigned by this design discussion.
