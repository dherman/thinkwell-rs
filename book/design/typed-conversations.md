# Typed conversations

The initial product API joins borrowed Rust tools and per-turn typed results over
ACP. The following properties govern its supported scope.

> **`A-scoped-turn-ownership`** (accepted): **Scoped callbacks belong to their execution**
> Property: Only an active execution accesses its bound callbacks; application borrows end when execution is consumed or dropped. Overlapping mutable session executions are rejected.
> Authority: `D-provisional-scoped-turns`
> - Criterion `behavior` [test, review]: Runtime tests and compiler probes establish non-Send borrowing across await, restored caller access, and rejection of overlapping mutable use. Review ownership and protocol boundaries against the stated exclusions.

> **`A-typed-turn-completion`** (accepted): **Typed results require completed turns**
> Property: Success requires supported-schema validation and conversion, a valid candidate, normal terminal response, and settled local callbacks. Result types may change between turns in one conversation.
> Authority: `R-per-turn-result-types`
> - Criterion `behavior` [test, review]: Tests cover changing types, invalid/corrected/exhausted submissions, missing and duplicate results, abnormal stops, transport loss, and cancellation or callback overlap after candidate acceptance. Review ownership and protocol boundaries against the stated exclusions.

> **`A-turn-envelope-routing`** (accepted): **Submitted turn identifiers control callback routing**
> Property: Missing, malformed, and nonmatching identifiers cannot select or invoke current callbacks. Revoked queues cannot reach replacement callbacks. This authenticates no causal origin when a sender supplies the current identifier.
> Authority: `D-provisional-scoped-turns`
> - Criterion `behavior` [test, review]: Deterministic HTTP tests inject stale and malformed calls during a replacement turn, assert zero mutation, and verify valid calls and revoked queues. Review ownership and protocol boundaries against the stated exclusions.

> **`A-cancelled-session-invalidation`** (accepted): **Cancelled sessions cannot resume**
> Property: Cancelling or abandoning a started turn permanently invalidates that session and its dedicated connection; no further prompt or implicit history reset occurs. Dropping an unpolled execution sends no work.
> Authority: `D-initial-cancel-invalidation`
> - Criterion `behavior` [test, review]: Tests distinguish unpolled and started drops, reject reuse, and inspect peer prompt counts. Review ownership and protocol boundaries against the stated exclusions.

> **`A-owned-resource-cleanup`** (accepted): **Owned resources have bounded cleanup**
> Property: Drop revokes local access and signals existing cleanup infrastructure without async waiting or spawning. Awaited shutdown reports a bounded outcome; owned-process guards force termination on driver/runtime loss on supported macOS and Linux hosts. No detached-descendant or asynchronous application-destructor guarantee is made.
> Authority: `D-reviewed-api-baseline`
> - Criterion `behavior` [test, review]: Instrumented child-process tests observe exit/reaping and listener closure for success, constructor failure, early errors, drops, awaited shutdown and runtime teardown on both supported runtime flavors. Review ownership and protocol boundaries against the stated exclusions.

> **`A-supported-contract-boundary`** (accepted): **Advertised supported contracts are enforced**
> Property: Unsupported contracts fail before prompting. Supported input, serialized tool output, and final-result constraints are validated independently of Serde conversion.
> Authority: `D-derived-contracts`
> - Criterion `behavior` [test, review]: Contract tests cover every supported keyword and unsupported dialects/references/constraints, malformed input and output, and located preflight errors. Review ownership and protocol boundaries against the stated exclusions.

## Initial support

Use Tokio with I/O and time enabled on current-thread or multithread runtimes.
Borrowed callbacks remain local and may be non-Send; owned drivers contain only
owned transport and request/reply state. One session may be open per Agent.
Normal session drop releases its reservation and revokes its endpoint; another
session uses fresh history and credentials. Failed/abandoned started turns close
the connection. Recovery is explicit reconnection.

Expose typed command configuration, the `thinkwell::agent::{Claude, Codex}` presets, fixed typed Tool
descriptors, cloneable unbound Plans, borrowed per-turn bindings and run_in.
Plan::tool constructs a typed tool descriptor and binds its callback in one step.
Plan::run validates the plan locally before opening a temporary session whose
catalog is the plan's bound tools. It executes one turn and drops the session
before returning. Sequential successful runs on an agent use fresh history;
run_in retains explicit-session catalog matching and history. Invalid local
plans send no setup request; occupied agents return Busy. Unpolled runs do
nothing, and setup/turn cancellation or failure retains existing invalidation
rules. The temporary session uses the agent's default model; explicit builders
provide model selection. Both inline tools and bind retain borrowed, non-Send
callbacks and non-cloneable bound plans.
Plan::json borrows any Serialize value only while appending its compact JSON plus
one newline to the prompt, just like text composition. Serialization is immediate
and fallible, returning InvalidValue on failure; no schema derivation is required.
It preserves existing Plan state and introduces no special content block, quoting
protocol, or instruction-trust boundary.
Use HTTP MCP 2025-11-25 with a loopback listener, bearer credential, Origin and
body-size checks, and client permission grants restricted to declared tools. Turn/input and
turn/value envelopes are library protocol details, not callback parameters.

Validate the finite object/array/primitive/const/enum/oneOf/anyOf contract subset,
including signed int64 representation and finite f64 values (`format: double`
with `type: number`). Double values must convert to a finite f64; this does not
promise exact representation of every JSON decimal. An f64 contract rejects
serialized nonfinite values, including values encoded as null. Generate
nonrecursive nested derived types
inline for both input/result and output contracts, without requiring application
inline annotations. Recursive types remain unsupported. Unsupported dialects, references,
definitions, formats and constraints are preflight errors. Local contract
validation precedes conversion and follows output serialization. Tool inputs and
final answers use deserialization schemas; tool outputs use serialization schemas
so direction-specific Serde field names describe the actual values on the wire.

Defaults are 120 seconds per turn, 30 seconds per callback, 32 application calls,
three invalid result submissions, 30 seconds for setup, and 12 for shutdown.
Expose configurable nonzero bounds. Do not cap the whole agent lifetime.
Model selection is optional; default selection belongs to the agent. Preset
installation and authentication remain external.

The interoperability targets are Claude ACP 0.23.1 and Codex ACP 2.8.0 with
official Rust ACP SDK 5.4.0. Codex defaults to the adapter's read-only mode;
installation and authentication remain external. The adapter pin does not freeze
its compatible-range Codex dependency. Agent-owned native reads can occur without
ACP permission requests, and user MCP configuration still applies. Reserve the
MCP server name thinkwell for the library. A reported Codex Thinkwell-MCP startup
failure becomes a Configuration error during session setup.

Permission requests can identify tools directly or refer to an earlier ACP tool
notification. Correlation is scoped to the active session and prompt, recognizes
the tested Claude and Codex naming conventions, and grants AllowOnce only for
declared tools (including the internal result tool). Unknown, conflicting and
terminal call IDs are denied. At 1024 tracked IDs, further new IDs disable grants
for that turn rather than evicting old identities. Correlation state is discarded
at turn completion/connection teardown. This is client permission policy, not
an operating-system sandbox or authenticated originating-tool identity. Support is initially macOS/Linux and Rust 1.100.0; Windows, embedding,
dynamic catalogs, concurrent sessions, progress streams, explicit cancellation
handles, and broader JSON Schema support are deferred.


## Wire contracts and validation

Application calls use turn/input envelopes; serialized outputs use a value envelope
in MCP structuredContent, with a matching object outputSchema. Text content retains
the serialized application value. This accommodates primitive outputs while keeping
the MCP output schema object-shaped. Both contracts derive from application types.
The internal result tool uses turn/value with a per-turn schema in the prompt.

HTTP bodies are limited to 256 KiB. Callback/turn deadlines and call budgets bound
individual executions; queued message capacity does not globally limit concurrent
HTTP senders. Header credentials are per session, so reopening cannot authorize
requests using the old credential. A session's routing is revoked on normal drop;
each later session obtains fresh remote history and endpoint credentials.

Every failed started turn closes its connection in this initial conservative
policy, including local exhaustion and abnormal remote completion. Preflight
errors and dropping an unpolled execution leave a usable session unchanged.
Callback errors and callback deadlines are tool errors the agent may handle;
turn deadline, result exhaustion, and call exhaustion terminate the execution.

Const and enum compare finite JSON numbers in their serialized decimal
representation without integer-to-float rounding. Integer contracts separately
reject floating representations. Unsupported schema errors identify the containing
property/items/alternative path. Validate schema dialect metadata before work;
only the 2020-12 dialect is supported.
