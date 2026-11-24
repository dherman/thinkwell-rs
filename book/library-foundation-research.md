# Library foundation research (migration-only)

This source research informed the initial port experiments. Package versions,
licenses, inspected mechanisms, and evidence limits are historical observations;
the [library design](design/index.md) owns accepted behavior. Source inspection
alone does not authorize adapting code or establish compatibility.

## ACP configuration research

The official [ACP v1 session configuration documentation](https://agentclientprotocol.com/protocol/v1/session-config-options),
inspected 2026-11-18, describes optional agent-provided `configOptions`, including
model selectors, and the `session/set_config_option` request. Agents supply
current values that work without client overrides. Option IDs and available
values come from the agent; model selection is not a universal agent guarantee.
This establishes protocol availability, not a selected Thinkwell version baseline.
The later [transport experiment](implementation-roadmaps/acp-integration-prototype.md#acp-integration-prototype-2)
exercised model selection with one real agent; broader agent support remains open.

These observations preceded the approved experiments and product scope. Current
library selection, derive requirements, model selection, and supported transport
belong to [library design](design/index.md).

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
and observable execution. Its results established local API feasibility, not a
product ownership contract.
The later product assignment selected the bounded [library design](design/index.md).

## Rust ACP library research {#current-rust-acp-library-research}

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
  work without making application callbacks SDK runners. That source inspection
  did not compile or test the composition; the subsequent
  [ACP experiment](implementation-roadmaps/acp-integration-prototype.md) did.
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

## Disposition

The [ACP integration experiment](implementation-roadmaps/acp-integration-prototype.md)
compared scoped Send callbacks with a local dispatcher, then exercised HTTP tools
with Claude Agent ACP 0.23.1 and negotiated MCP 2025-11-25. The official polyfill's
2026-07-28 requirement was therefore not established as compatible. The
[runtime/result experiment](implementation-roadmaps/runtime-results-prototype.md)
then assessed typed completion, cleanup, and submitted-turn identifiers. Those
roadmaps retain their assertions, live-run evidence, and limits.

The accepted [agent integration](design/agent-integration.md),
[execution lifecycle](design/execution-lifecycle.md), and
[contract boundary](design/contracts.md) replace the earlier recommendations to
select a runtime, dispatcher, schema subset, and transport. Shared host connections,
recovery, and wider provider coverage remain outside that bounded scope; they are
not assigned follow-up work by this research.
