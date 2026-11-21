# Open questions

The first product slice and preceding approved experiments were integrated in
owner-merged PR #2. The [closeout record](implementation-roadmaps/first-product-slice.md#integrated-completion)
records review and final-head/merge-commit CI. The Book closeout was reviewed and
merged in PR #3. The scheduler and its two approved API adaptations were reviewed and merged in
PR #4; the [completion record](implementation-roadmaps/meeting-scheduler.md#integrated-completion)
records final-head CI and integration. Nonrecursive nested schemas now inline
automatically, and `Plan::json(&value)?` supplies structured prompt context.
The owner reviewed and merged [sentiment analysis and finite f64 support](implementation-roadmaps/sentiment-analysis.md#integrated-completion)
in PR #6; final-head and merge-commit CI passed. The owner approved [one-shot Plan execution with inline borrowed tools](implementation-roadmaps/one-shot-tools.md)
after merging PR #7; it is complete through owner-merged PR #8.
The owner selected Codex for the next agent-compatibility investigation.
[Codex research](implementation-roadmaps/codex.md#research-evidence) verifies
model selection and a diagnostic two-turn workflow, and identifies permission
correlation as required implementation work. The owner approved the Codex milestone;
local implementation and direct live example validation now pass, with delivery
still pending.
The supported product scope remains [typed conversations](design/typed-conversations.md).

Initial choices are settled for Tokio, one session per owned ACP connection,
the finite Serde/schemars contract subset, fixed-catalog borrowed tools,
per-turn result types, turn envelopes, and invalidation after cancellation.
The earlier [integrated examples](implementation-roadmaps/integrated-api-review.md)
remain the wider provisional direction, including surfaces deliberately deferred
from this slice.

| Open area | Remaining question |
| --- | --- |
| First-use expansion | One-shot execution and inline tools are integrated; are further quoting helpers needed? |
| Runtime and embedding | What additional ownership/runtime contracts support shared host connections and concurrent sessions? |
| Schemas | Which broader validator/reference-resolution support justifies expanding the deliberately finite subset? |
| Provider coverage | Which additional agents, capabilities, model discovery, and transport paths should be tested and supported? |
| Cancellation recovery | Is recovery needed beyond the accepted initial policy of reconnecting explicitly? |
| Turn provenance | Do applications need stronger agent-bound causal-origin evidence beyond matching submitted identifiers? |
| Platform/compiler support | When should Windows or an MSRV below the development toolchain become commitments? |

Migration-only questions and provenance remain in [migration research](migration.md).
Ending the Porting Phase and publication retain separate authorization.
The following research notes preserve the exploration behind the integrated slice;
historical statements about missing evidence do not override its current roadmap.

## ACP configuration research

The official [ACP v1 session configuration documentation](https://agentclientprotocol.com/protocol/v1/session-config-options),
inspected 2026-11-18, describes optional agent-provided `configOptions`, including
model selectors, and the `session/set_config_option` request. Agents supply
current values that work without client overrides. Option IDs and available
values come from the agent; model selection is not a universal agent guarantee.
This establishes protocol availability, not a selected Thinkwell version baseline.
The later [transport experiment](implementation-roadmaps/acp-integration-prototype.md#acp-integration-prototype-2)
exercised model selection with one real agent; broader agent support remains open.

The owner selected the ACP client foundation and programming goals; see
[design](design/index.md). Type-derived tool contracts use Rust derives (`D-derived-contracts`), with
specific traits and libraries still open. The scoped
[ownership experiment](implementation-roadmaps/ownership-prototype.md) assesses
API feasibility; product implementation remains unassigned. Implementation
planning is deferred until the relevant behavior and capability scope are agreed.

## Determinishtic research

At the owner's request, inspected Cargo's published
[`determinishtic` 0.3.0](https://crates.io/crates/determinishtic/0.3.0) on
2026-11-18 using `cargo info determinishtic`. Package VCS metadata identifies
`4dc27e6fd73de58f0807856952380f0af28d25d6`. Read its README, introduction,
ThinkBuilder RFC, library source, and summarization example; inspected the
mutable-tool responders, adapter macro, and scoped session runner in its
`agent-client-protocol` 0.11.0 dependency. No source was copied or adapted,
no project dependency was added, and no compilation or live-agent tests ran.
The crate declares MIT OR Apache-2.0; the inspected ACP dependency declares
Apache-2.0. Any later adaptation still requires file-specific notice review.

Source observations that inform the open design questions:

- **Borrowed mutable callbacks:** `ThinkBuilder` carries the responder chain in
  its generic type. Tool callbacks have `AsyncFnMut` and `Send` bounds without
  a `'static` bound on the callback. The scoped `run_until` operation drives
  responders alongside session processing. Each mutable callback's responder
  awaits an invocation before receiving its next call; this is per-tool
  serialization, not serialization of the entire program. Inputs and outputs
  remain owned, with `'static` bounds. This supports investigating local mutable
  captures without requiring application locks.
- **Compiler adapter:** The caller supplies `tool_fn_mut!()`, which creates a
  higher-ranked adapter returning a boxed future tied to the callback borrow.
  Package comments attribute this to return-type notation and compiler inference
  limitations. The later [ownership experiment](implementation-roadmaps/ownership-prototype.md#ownership-prototype-1-helpers)
  tests local erased callbacks without a caller adapter and a separate Send
  adapter. It does not establish that this dependency can use the same approach;
  async closures themselves are not an unavailable language feature.
- **Awaitable builder:** `IntoFuture` consumes the builder and allows the result
  type to be inferred from the receiving binding. The published builder has no
  `Clone` implementation; this does not resolve Thinkwell's reusable-plan needs.
- **Ordinary derives:** `schemars::JsonSchema` describes contracts, Serde
  deserializes tool inputs/results and serializes tool outputs. An internal
  object wrapper around the result accommodates scalar as well as aggregate
  output types. This is a concrete precedent for `D-derived-contracts`, not a
  dependency selection or proof of full JSON Schema constraint validation.
- **Embeddability:** `from_connection` accepts an existing role-typed connection
  handle without taking responsibility for its background task. Tool callbacks
  receive connection context. The RFC describes nested reasoning in independent
  sessions; reentrancy and deadlock behavior were not tested here.
- **Prompt-local tool references:** `.tool()` inserts a reference where it appears
  in the prompt chain; `.define_tool()` registers without insertion. Display and
  Debug interpolation are available. Automatic spacing is another behavior to
  assess explicitly, rather than inherit accidentally.
- **Observation and policy:** Synchronous observer hooks expose assembled prompts,
  notifications, permission requests, and stop reasons. Permission handling
  independently chooses an available allow option. Connection ownership uses
  task abortion on drop. These choices do not settle Thinkwell's permission,
  streaming, cancellation, or graceful-shutdown contracts.

The findings guide design exploration only. The owner subsequently approved the
[ownership API prototype](implementation-roadmaps/ownership-prototype.md), which
records evidence for scoped mutable tools, reusable base plans, helper composition,
and observable execution. Its results leave Send requirements, shared reusable
tools, and ACP integration open; they do not accept a product ownership design.

## Current Rust ACP library research

On 2026-11-18, inspected Cargo-published sources for
[`agent-client-protocol` 5.4.0](https://docs.rs/crate/agent-client-protocol/5.4.0/source/),
[`agent-client-protocol-polyfill` 5.4.0](https://docs.rs/crate/agent-client-protocol-polyfill/5.4.0/source/),
and [`sacp` 11.0.0](https://docs.rs/crate/sacp/11.0.0/source/), plus manifests
and READMEs for their conductor packages and `sacp-tokio` 11.0.0. These are
different versions from Determinishtic's inspected dependency. This was source
research only: no dependency was added, source adapted, integration compiled,
or agent exercised. The official packages declare Apache-2.0 and Rust 1.88.0;
the inspected sacp packages declare MIT OR Apache-2.0, with no declared MSRV.
Later source adaptation requires file-specific license and notice review.

### Ownership and execution observations

- In the official SDK, `src/mcp_server/tool_fn.rs::tool_fn_mut` separates an
  owned, `'static` registered tool handle from its callback runner. The callback
  has `AsyncFnMut + Send` bounds but no `'static` bound. A caller-supplied
  higher-ranked adapter returns a borrowed boxed **Send** future. Inputs and
  outputs are owned and Send. The runner serializes invocations of that tool.
- `src/jsonrpc/run.rs::RunWithConnectionTo` requires Send runners and futures.
  `src/session.rs` offers scoped `block_task().run_until(...)` for borrowing
  runners; `start_session()` instead requires a `'static` runner and starts it
  in the connection's background tasks. Send and `'static` are separate
  requirements; borrowing does not itself require application locks.
- The official `src/component.rs::ConnectTo` requires Send + `'static`
  transports. However, `src/jsonrpc.rs::Builder::connect_with` and the scoped
  session operation accept foreground async callbacks without explicit Send
  bounds. This suggests a local dispatcher can be polled alongside protocol
  work without making application callbacks SDK runners. That composition
  remains uncompiled and untested.
- sacp's `src/mcp_server/builder.rs` and `src/session.rs` offer the same relevant
  distinction: borrowed mutable Send callbacks with an adapter and scoped
  execution, versus `'static` runners for background sessions. This similarity
  does not establish protocol or API interchangeability.

### Transport and capability observations

The official core is runtime-neutral, with optional process and stdio support;
its typed MCP helpers are gated by `schemars`. Stable ACP v1 is supported, with
draft v2 behind an unstable feature. `SessionBuilder::with_mcp_server` is gated
by `unstable_mcp_over_acp`: it advertises the native ACP MCP transport and does
not itself check the agent's `mcpCapabilities.acp` support. This convenience
API is not evidence that arbitrary ACP agents can invoke those tools.

The official polyfill's README specifies a loopback HTTP bridge for agents
supporting **MCP 2026-07-28** over HTTP. It explicitly does not support older MCP
revisions or agents with neither transport. The bridge uses Tokio/axum, so the
complete bridge stack is not runtime-neutral merely because the core is.
sacp directly depends on Tokio; its conductor documents its own proxy extensions
and a stdio-to-TCP MCP bridge. Transport equivalence was not established.

The official SDK exposes stable session configuration requests, consistent with
the protocol research above. Actual model selectors and supported choices still
depend on the agent. Session lifetime behavior also needs a selected baseline:
the SDK documents different handler retention for v1 and draft v2; neither
should silently become Thinkwell's cleanup contract.

### Design implications and recommended next experiment

Two integrations remain plausible. Direct use of the SDK's scoped Send tools
preserves borrowed application state but retains its callback adapter and Send
constraints. Alternatively, a registered Send tool can exchange owned requests
and responses with a local dispatcher that owns Thinkwell's borrowed callbacks.
The latter could preserve the prototype's adapter-free fluent API. It adds
responsibility for bounded queues, cancellation, late replies, dispatcher failure,
and nested calls; it is a proposal, not an established integration result.

Investigate the official 5.4.0 SDK first as a candidate, without selecting it as
a dependency yet. A separately assigned bounded experiment should compile the
local-dispatcher composition against an in-memory ACP peer, exercise typed tool
calls, cancellation and teardown, and verify that borrowed data becomes available
again after completion or drop. Include a direct scoped Send callback as a
comparison. Before claiming provider breadth, exercise a representative real
agent's advertised MCP transport and revision, plus optional model selection.
The owner subsequently assigned the
[ACP integration prototype](implementation-roadmaps/acp-integration-prototype.md).
Its seven passing tests establish local non-Send callback dispatch and direct
scoped Send callback execution through the real SDK with an in-memory peer.
The subsequent [real-agent transport experiment](implementation-roadmaps/acp-integration-prototype.md#acp-integration-prototype-2)
completed two tool calls through the Rust ACP SDK and a minimal HTTP MCP fixture
with Claude Agent ACP 0.23.1, negotiated MCP `2025-11-25`, and verified a tool-only
receipt in the final agent message. A callback borrowed mutable non-Send local
state. This supports the channel boundary as an API candidate, not a production
transport or cleanup contract. The observed revision differs from the official
polyfill's required `2026-07-28`; that polyfill was not exercised. Product API,
library selection, and broader agent compatibility remain open.


The [provisional session/execution interpretation](implementation-roadmaps/acp-integration-prototype.md#provisional-session-and-execution-api-interpretation)
separates conversation history from callback borrow lifetimes and distinguishes
awaited cancellation from execution drop. The owner selected the fixed session tool catalog and per-turn callback direction
as a provisional plan under `D-provisional-scoped-turns`, retaining re-review as
implementation develops. Catalog refresh, stale-request isolation, session
recovery, and async cleanup mechanisms remain untested and unresolved; no product
implementation is assigned by this exploration. The
[API refinement](implementation-roadmaps/acp-integration-prototype.md#session-api-refinement-for-discussion)
examines contract handles, per-turn bindings, cancellation results, and the
interaction between a fixed catalog and changing typed final-result schemas.


`R-per-turn-result-types` now requires different result types across turns of a
single conversation. The [typed-result transport proposal](implementation-roadmaps/acp-integration-prototype.md#per-turn-typed-result-transport-proposal)
recommends a stable internal submission envelope with per-turn schema guidance
and host validation. Transport selection, schema-validation coverage, correction
budgets, duplicate handling, and the relationship between candidate acceptance
and turn completion remain proposals; no implementation is assigned.

The owner requested example-driven review before selecting the result transport.
[Worked result examples](implementation-roadmaps/acp-integration-prototype.md#worked-result-examples-for-design-review)
show a Summary-to-NextAction conversation, candidate internal submissions,
validation/stale-result failures, and the upfront-declaration alternative.
These are illustrative sketches, not compiled APIs or live-agent evidence.

The [progress/cancellation walkthrough](implementation-roadmaps/acp-integration-prototype.md#progress-cancellation-and-failure-walkthrough)
explores an awaitable execution, an optional synchronous observer, a per-execution
cancellation handle, and separate failure reason versus session readiness. The
owner supports continued exploration and integrated review; these refinements and
the stable-envelope mechanism are not final selections.

[Helper and concurrency examples](implementation-roadmaps/acp-integration-prototype.md#helper-composition-and-concurrent-conversations)
explore ordinary async helpers and concurrently polled independent sessions.
They distinguish plan reuse from history/callback cloning, and wait-for-all from
fail-fast sibling drop. Real-agent concurrency, nested calls, failure isolation,
and composed cancellation remain unverified; no new orchestration API is selected.

[First-use and model-choice sketches](implementation-roadmaps/acp-integration-prototype.md#first-use-agent-configuration-and-model-choice)
explore named-agent convenience, explicit launch configuration, agent-provided
model defaults, and session-local optional model selection. Setup, authentication,
capability failures, native-tool permissions, and connection ownership remain open;
one tested agent/model selection does not establish a supported-agent catalog.


[Agent selection extensibility](implementation-roadmaps/acp-integration-prototype.md#extensible-agent-selection)
compares typed preset conversion via `Into<AgentConfig>` with a behavioral connector
trait, following the owner's concern about string selectors and closed enums.
External preset implementation and backward compatibility guide the discussion;
the exact public types/traits remain unselected.

[Preset connection conveniences](implementation-roadmaps/acp-integration-prototype.md#preset-connection-convenience-methods)
include compiler-checked language sketches for instance connect, method-level
Default shorthand, and inherent aliases. The same trait cannot overload connect
by receiver/argument count. Naming, public future bounds, and any behavioral
connector extension remain open; this is not product implementation evidence.


`D-connection-vocabulary` selects connect for agent connections, open for sessions,
and inherent default-preset shorthand alongside instance connect. The
[ownership/embedding exploration](implementation-roadmaps/acp-integration-prototype.md#connection-ownership-and-embedding-exploration)
compares borrowed session handles with retained shared ownership and makes
initialization, I/O driving, handler routing, and shutdown responsibilities explicit.
Its type/lifetime choices and cleanup APIs remain proposals, not implementation.


The owner rejected mandatory explicit cleanup and callback-scope indentation.
The approved [integrated review](implementation-roadmaps/integrated-api-review.md)
uses RAII as the default, with optional awaited shutdown confirmation.
The [runtime/result experiment](implementation-roadmaps/runtime-results-prototype.md)
now has local evidence for owned-driver cleanup, non-Send borrowed callbacks,
and different validated final result types in one live conversation.

The finite schema interpreter, permanently invalidated sessions after abandoned
turns, and macOS-only process checks establish feasibility rather than product
support guarantees. The owner selected initial cancellation invalidation
(`D-initial-cancel-invalidation`); recovery is deferred. Product choices remain
for runtime/schema coverage and late application-tool requests after normal turn
completion.
The experiment now checks turn identifiers on application calls as well as results. Detailed policies,
tests, and exclusions belong to the experiment's evidence record.

The owner explicitly approved integrated-api-review.md on 2026-11-19, recorded as
`D-reviewed-api-baseline`, and separately assigned the runtime/result milestone.
Both approvals remain effective. Local implementation and self-review are complete;
human review/integration and a future product assignment remain separate.


The owner agreed to explore library-managed turn identifiers for application tools.
The [protocol follow-up](implementation-roadmaps/runtime-results-prototype.md#application-tool-turn-association)
finds no portable prompt-to-MCP-call association in the inspected interfaces.
ACP's normal flow already ends a turn when no tool calls are pending; the implemented
experimental guard addresses stale/background calls rather than changing normal ACP semantics.
A required wire argument can reject an old identifier without changing Rust
callback arguments, but it cannot attest causal origin if a sender uses the new
identifier. Transport-origin guarantees need cooperating agent support.


The authorized turn-envelope follow-up passed deterministic late-call injection
and a live two-result session with unchanged Rust callback signatures. This
establishes stale-identifier rejection, not authenticated originating-turn identity.
The runtime/result roadmap owns the current source digest and validation results.


The [first product slice proposal](implementation-roadmaps/first-product-slice.md)
makes runtime, platform, schema subset, one-session-per-connection, budgets, error
categories, API deferrals, and acceptance criteria concrete for scope review.
It preserves the accepted cancellation policy and limits turn-envelope guarantees
to submitted identifiers. The owner approved the concrete milestone and instructed implementation.
The selected choices are now defined in the typed-conversation design; Linux CI,
human review, and integration remain pending. Broader schemas, recovery,
embedding, and other deferred surfaces remain future work.
