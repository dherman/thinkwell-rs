# ACP integration prototype

Roadmap identifier: `acp-integration-prototype`.

The [library design](../design/index.md) owns the accepted target. This chapter
retains the bounded SDK and real-agent experiments, including their limitations.

Current integration state: completed work was delivered through owner-merged
[PR #2](https://github.com/dherman/thinkwell-rs/pull/2). See the
[shared integration evidence](first-product-slice.md#integrated-completion).

Goal: test whether the official ACP SDK can preserve scoped local tool ergonomics.

The subsequent [integrated API review](integrated-api-review.md) records owner
approval of the provisional programming model. Neither experiment selects the
entire supported product scope.

## Milestone 1. Assess SDK integration {#acp-integration-prototype-1}

**Outcome:** A bounded integration experiment compares local dispatch and direct
Send callbacks, assesses cancellation and teardown, and records agent capability evidence.

**Status:** Complete

**Identifier:** `acp-integration-prototype-1`

**Dependencies:** The owner approved the proposed four-part experiment with
“Proceed” in the port-design conversation. The preceding ownership experiment
supplied local API evidence. This assignment did not select product dependencies
or authorize publication; both experiments were subsequently integrated in PR #2.

**Design basis:** `D-acp-client`, `D-derived-contracts`, `R-code-prompt-ergonomics`,
`R-easy-agent-programming`, and `R-provider-breadth`; [SDK research](../library-foundation-research.md#current-rust-acp-library-research).

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

Milestone 2 supplied the missing real-agent HTTP round trip. The later product
assignment selected the runtime, transport, cancellation, and cleanup contracts in
[library design](../design/index.md). This experiment alone did not select them.

### Completion record

Complete through owner-reviewed and merged PR #2; see the
[shared delivery record](first-product-slice.md#integrated-completion). The seven
SDK tests and capability probe retain their original scope. Real-agent tool
transport is evidence from milestone 2, not from this milestone's capability probe.

## Milestone 2. Exercise a real-agent tool round trip {#acp-integration-prototype-2}

**Outcome:** A real ACP agent calls a locally hosted tool over a supported MCP
transport, and its final reply demonstrates receipt of the tool output.

**Status:** Complete

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

Complete through owner-reviewed and merged PR #2; see the
[shared delivery record](first-product-slice.md#integrated-completion). The live
HTTP fixture does not establish general MCP conformance, provider breadth,
independent process-tree cleanup, or asynchronous callback cleanup.

## Incorporation of the API exploration

The original [integrated exploration](https://github.com/dherman/thinkwell-rs/blob/1b21c83b9b713690d953df72244c7f9fcf8244ce/book/implementation-roadmaps/acp-integration-prototype.md)
remains in Git. Its uncompiled caller sketches are superseded as definitions by:

| Explored subject | Owning definition or unresolved boundary |
| --- | --- |
| Session versus execution lifetimes, fixed contracts, per-turn bindings, helpers, and plan reuse | [Typed conversations](../design/typed-conversations.md) |
| Stable typed-result envelope, validation, duplicates, and terminal completion | [Contracts and typed results](../design/contracts.md) |
| RAII, optional awaited shutdown, cancellation, and failure settlement | [Execution lifecycle](../design/execution-lifecycle.md); recovery and explicit cancellation handles remain deferred |
| First use, extensible presets, connect/open vocabulary, and model selection | [Agent integration](../design/agent-integration.md); generic model discovery/mutation is not a supported surface |
| Optional local observation | [Progress observation](../design/progress-observation.md) |
| Concurrent sessions and host attachment | Outside the selected owned-connection scope; no concrete attachment or host-routing contract is accepted |

The embedding exploration compared borrowed and retained shared handles and
assigned initialization, I/O, routing, and shutdown to a host. It had no compiled
or live-host evidence. The [integrated review](integrated-api-review.md) retains
the provisional approval basis and its distinction from selected product scope.

### Preset compiler-probe evidence

Rust 1.100.0 checked disposable stub Agent/AgentConfig sources on the uncommitted
port-design tree under ignored `target/agent-convenience-probe/`; these probes
performed no I/O or connection work.

- `rustc --edition=2024 --crate-type=lib target/agent-convenience-probe/positive.rs --out-dir target/agent-convenience-probe -D warnings` passed. It compiled instance conversion, method-specific Default shorthand, direct Agent::connect, configured-only instances, an associated-only Claude::connect, and an inherent alias alongside an instance method.
- `missing_default`, calling default shorthand on a non-Default type, was rejected with E0277 as expected.
- `duplicate`, defining both connect forms in one trait, was rejected with E0428 as expected.

The negative cases used the same flags through a Python subprocess harness with
30-second bounds. These are language-shape checks, not public async-trait Send,
real-connection, or package-integration evidence. No dependency was added.
`D-connection-vocabulary` subsequently selected instance connect and per-preset
inherent shorthand. The default/associated-only alternatives remain research;
the blanket convenience uses the standard configuration path and cannot be
overridden by an overlapping downstream implementation. A custom behavioral
connector requires its own design.
