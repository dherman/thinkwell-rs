# Scoped ACP integration experiment

This independent, unpublished Cargo package tests ownership against
`agent-client-protocol` 5.4.0. It is not a product dependency selection or API.
It uses original experiment code, not adapted upstream implementation. Dependencies
retain their own licenses; the official SDK is Apache-2.0. Source research informed
API usage and protocol fixture formats.

Run from the repository root:

```sh
CARGO_TARGET_DIR="$PWD/target/acp-integration" cargo test --manifest-path experiments/acp-integration/Cargo.toml --locked
CARGO_TARGET_DIR="$PWD/target/acp-integration" cargo clippy --manifest-path experiments/acp-integration/Cargo.toml --all-targets --locked -- -D warnings
cargo fmt --manifest-path experiments/acp-integration/Cargo.toml -- --check
python3 experiments/acp-integration/probe_agent.py > target/acp-agent-capabilities.json
```

The original seven Rust tests have five-second deadlines. A scripted in-memory ACP peer
accepts session setup and sends native MCP tool requests. The real SDK routes
requests and generates schemas/deserializes input/serializes output through its
registry. The minimal service implements only `tools/call`; it is not a complete
MCP server, and skips agent initialization and tool discovery in this fixture.
Errors use the ACP backend-error path, not a selected product tool-error policy.

`local_tool()` returns an owned Send tool handle and a local dispatcher. The
application passes an async closure directly to `dispatch.run(...)`, with no
Send, static, or boxing adapter requirement. A capacity-one Tokio channel carries
owned inputs and reply senders. This bounds queued messages, not the total number
of callers awaiting capacity. Invocations are serial. The test callback borrows
mutable state and uses an Rc across await, establishing genuinely non-Send use.
The direct SDK comparison uses `tool_fn_mut` and an explicit boxed-future adapter.

The dispatcher drops a callback when its reply receiver disappears. Tests cover
remote request cancellation followed by another call, foreground session finish,
execution abort/drop, callback failure, malformed input, and missing dispatcher.
Earlier mutations persist on cancellation. Release of a synchronous drop guard
is tested; graceful asynchronous callback cleanup, subprocess teardown, queued
cancellation races, recursive tool calls, and multithreaded behavior are not.
The service can return cancellation before the local dispatcher has observed it;
production cleanup would need an acknowledgment/join policy if it promises more.
No tasks are spawned by those original tests, and the result future must be polled.

The Python probe uses an offline cached, pinned Claude ACP package 0.23.1 in a
temporary directory. It initializes, creates a session without MCP servers,
reads configuration, and sets the model to its existing value. It sends no
prompts, grants no tool requests, prints selected capability fields only, and
terminates the process group afterward. Each request has a 25-second deadline.
A missing npm cache, auth requirement, or timeout is reported as an evidence gap.
It is a separate JSON-RPC capability probe, not a Rust-SDK-to-real-agent test.
The response does not establish the agent's negotiated MCP revision or successful
model inference. No persistent agent configuration is changed.

See the Book's ACP integration prototype roadmap for evidence and interpretation.


## Opt-in real-agent HTTP round trip

```sh
CARGO_TARGET_DIR="$PWD/target/acp-integration" cargo run --manifest-path experiments/acp-integration/Cargo.toml --locked --bin http_roundtrip > target/acp-http-roundtrip.json
```

This command sends one model prompt through the cached Claude ACP agent 0.23.1,
using its existing authentication and `haiku` model selector. Normal `cargo test`
never runs it. The Rust SDK launches the agent; a minimal loopback HTTP server
advertises a derived contract, and the existing channel dispatcher invokes a
callback borrowing local Vec/Rc state. The agent is asked for two calls (3, then
4) and the returned receipt. Success requires input order, local total 7, the
unprompted receipt appearing in the final reply, and HTTP listener shutdown.
It does not deserialize a typed final model answer or validate final prose arithmetic.

The live result negotiated MCP 2025-11-25. This is a narrow HTTP fixture, not the
official polyfill (which requires MCP 2026-07-28) or a complete MCP implementation.
It implements initialize, initialized notification, ping, tools/list, and tools/call.
The endpoint requires an ephemeral bearer value, rejects Origin headers, and
accepts only values 3 and 4, with a two-call dispatch budget. Only the exact
experiment tool receives allow-once permission. No auth values or full model
transcripts are printed. The session works in a temporary directory.

Limits: a 120-second agent-work deadline, 135-second outer SDK deadline, five
seconds per tool invocation, and seven seconds for HTTP shutdown. Normal return
drives SDK process cleanup; an outer timeout drops that transport and is reported
as a failure. HTTP task joining and listener closure are checked, but independent
process-tree exit, HTTP disconnect cancellation, and asynchronous callback cleanup
are not. Three additional offline tests cover contract negotiation, input/access
rejection, borrowing dispatch, and the call budget. Product policies remain open.


## Combined runtime and typed-result foundation

The new runtime.rs and result_contract.rs implement the bounded
runtime-results-prototype-1 assignment. The executable example is
src/bin/runtime_results.rs; tests/runtime.rs uses an original Python ACP child
that calls the same HTTP endpoint and runtime as the live agent.

```sh
CARGO_TARGET_DIR="$PWD/target/acp-integration" cargo run --manifest-path experiments/acp-integration/Cargo.toml --locked --bin runtime_results > target/runtime-results-live.json
```

This opt-in command sends two prompts in one Claude ACP 0.23.1 / haiku session.
A derived Summary result follows the first borrowed accumulator call. Deterministic
code then doubles the accumulator and changes the caller's Vec before a second
turn returns a tagged Decision result. The fixed catalog includes accumulate and
return_result. A candidate is returned only after a normal terminal response.

Default resource release is through Drop: local callback access is revoked and
the existing owned driver closes transport/process/server resources. Optional
shutdown().await confirms its outcome. Drivers contain no borrowed callbacks.
The 13 runtime integration tests cover result failures, non-Send borrows, process exit/
reaping, early returns, handle destruction, and runtime teardown. A new library
test checks route revocation; two compile-fail doctests enforce exclusive borrows.
Tests need Python 3 and Unix ps, not credentials or model inference.

The schema interpreter intentionally rejects unsupported keywords and references.
It supports the examples' object/array/primitive/tagged-enum contracts and checks
values before Serde conversion; it is not a general JSON Schema implementation.
Every failed/abandoned started turn permanently invalidates its session.
Both tools now require a turn identifier. Application arguments are wrapped as
{"turn":"opaque-id","input":{"value":3}} by the protocol contract, while Rust
callbacks remain unchanged. A delayed old identifier is rejected before queue
dispatch; a sender using the current valid identifier cannot be distinguished by
causal origin. The late-call fixture checks six rejected identifiers during a
second turn and unchanged catalogs. Process evidence is
macOS-specific, with no descendant-tree or asynchronous application cleanup claim.

See book/implementation-roadmaps/runtime-results-prototype.md for deadlines,
schema details, source/license review, exact evidence, and product-promotion limits.
