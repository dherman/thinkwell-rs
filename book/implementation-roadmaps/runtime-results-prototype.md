# Runtime and typed-result foundation experiment

Roadmap identifier: `runtime-results-prototype`.

Current integration state: completed work was delivered through owner-merged
[PR #2](https://github.com/dherman/thinkwell-rs/pull/2). See the
[shared integration evidence](first-product-slice.md#integrated-completion).
Earlier local-stage evidence and pending-delivery statements below are historical;
they retain their tested scope and do not describe current integration status.

Goal: establish whether the reviewed API's two unproven foundations—per-turn typed
final results and RAII-managed connection lifetime—work together before committing
the public product API to their mechanisms.

## Milestone 1. Implement the combined foundation {#runtime-results-prototype-1}

**Outcome:** An executable, isolated Rust implementation runs two differently typed
turns in one real conversation and demonstrates bounded owned-resource cleanup
without mandatory caller cleanup or scoped callbacks. Deterministic fixtures
exercise failure and cancellation paths that live model behavior cannot reliably
cover.

**Status:** Complete

**Identifier:** `runtime-results-prototype-1`

**Dependencies:** The owner explicitly approved integrated-api-review.md on
2026-11-19 (`D-reviewed-api-baseline`); no additional design sign-off is needed.
The owner explicitly approved this milestone (“I approve that milestone; proceed”),
including the bounded experiment on the current design branch. Approval is
recorded Ready on 2026-11-19, readiness reported, then marked In progress.
Earlier research and experiments remain locally validated and unintegrated.
Commit/push/publication authority remains separate.

**Design basis:** `D-reviewed-api-baseline`, `D-acp-client`, `D-derived-contracts`,
`D-provisional-scoped-turns`, `D-connection-vocabulary`,
`R-per-turn-result-types`, `R-easy-agent-programming`, and
`R-code-prompt-ergonomics`; the owner-reviewed
[integrated proposal](integrated-api-review.md) and
[existing experiments](acp-integration-prototype.md).
This is a scoped prototyping assignment, not product implementation.

**Tasks**

- [x] Record approved scope Ready, report readiness, then mark In progress.
- [x] Implement the smallest shared runtime/session/result core needed by the
      three criterion groups below in an isolated experiment package.
- [x] Build deterministic protocol/process fixtures and ownership compile probes.
- [x] Run a bounded live two-turn agent conversation through that same core.
- [x] Review findings separately from implementation and reconcile the Book,
      including any reason to revise the provisional design.

### Scope and concrete choices for the assignment

Keep implementation under experiments rather than adding public product API or
product dependencies. Reuse the inspected official ACP SDK 5.4.0 and the earlier
HTTP integration as starting evidence. Use Tokio for this bounded runtime
experiment, without selecting it as Thinkwell's universal runtime. Reuse or factor
original experimental code; review licenses/notices before adapting third-party
implementation. Pin the experiment's dependencies in its lockfile.

Implement one owned subprocess agent path, one fixed MCP catalog containing a
borrowed application tool and the internal result-submission tool, and one explicit
session used for two consecutive result types. Use the known Claude ACP 0.23.1
HTTP integration for live compatibility, with a pinned model choice and bounded
prompts. Instrumented deterministic peers/processes supply failure and shutdown
evidence. No extra providers, installation/authentication automation, embedding
host adapter, dynamic catalog, parallel-session API, broad preset catalog, or full
progress UI is required. Public name polish is secondary to keeping the examples
recognizable; no naming redesign is authorized by the experiment.

The result mechanism to assess is the stable turn/value envelope, with a fresh
turn identifier, per-turn schema guidance, and host validation/deserialization.
Choose and record a finite supported schema subset sufficient for the examples
(required object fields, primitives, arrays, and a tagged enum), including how
schema references and unsupported constraints are handled. Validation must use
the advertised supported contract; Serde-only conversion must not be described
as full schema validation. Library choice within the isolated experiment requires
source/license inspection and recorded rationale, not a product dependency decision.
Use explicit finite correction and time budgets in fixtures; do not infer product
defaults from experimental settings.

The runtime must separate owned I/O/shutdown resources from borrowed callbacks.
The default caller path uses RAII; an optional awaited shutdown supplies bounded
confirmation and reports failure. Destruction revokes local callback access and
signals an existing driver rather than blocking on async work or spawning during
teardown. Owned-process guards supply a documented fallback when the driver is
dropped. No destructor can promise rollback, arbitrary application async cleanup,
or a completed remote acknowledgment before it returns.

### Completion criteria

#### Changed final result types {#runtime-results-prototype-1-results}

One real session, with an unchanged catalog, yields a Summary-like record followed
by a differently typed tagged-enum result. Returned values come from validated
result submissions rather than parsing an incidental prose receipt. The application
executes deterministic code between turns and can access its previously borrowed
state. A borrowed application tool is exercised, not merely registered.

- **Anchor coverage:** None — scoped feasibility experiment; product anchors and
  supported-agent guarantees are not established by it.
- **Evidence:** Passed locally on 2026-11-19 through
  experiments/acp-integration/src/bin/runtime_results.rs. Claude ACP 0.23.1,
  official Rust SDK 5.4.0, model selector haiku, HTTP MCP 2025-11-25.
  One session returned Summary { total: 3, labels: ["first"] }, then
  Decision::Accept { total: 10 }. The application doubled the accumulator and
  modified its input Vec between turns; borrowed callbacks received 3 then 4.
  Both results used return_result and host validation, with normal end_turn
  required. The catalog was unchanged. Ignored target/runtime-results-live.json
  records outputs and generated schemas; the executable retains the assertions.
  Haiku is the configured agent selector, not a pinned underlying model revision.

#### Result and turn failure semantics {#runtime-results-prototype-1-failures}

Deterministic fixtures cover malformed/contract-invalid values, corrective
submission within the configured budget, budget exhaustion, stale identifiers,
duplicate valid submissions, normal agent completion with no valid result, and
cancellation after a valid candidate but before terminal completion. No successful
T is returned merely because a candidate arrived. Distinguish local validation,
remote stop reason, and session readiness. An unsettled or abandoned turn must
not race a subsequent prompt or silently reset history.

- **Anchor coverage:** None — proposed behavior under assessment.
- **Evidence:** Passed locally: tests/runtime.rs drives runtime::Agent and
  Session with a deterministic Python ACP subprocess making actual HTTP MCP
  calls. It covers missing envelope value, invalid typed value, correction,
  two-rejection exhaustion, stale identifier, duplicate candidate, missing
  result, remote Cancelled after acceptance, and execution drop after candidate
  acceptance. Candidate arrival alone never completes the future. Failed or
  abandoned turns leave the session unusable, with a second prompt rejected
  locally and history never reset. The same runtime and validator serve live
  and deterministic peers. Policies and remaining routing limits are below.

#### RAII and owned runtime lifetime {#runtime-results-prototype-1-lifetime}

Demonstrate ordinary success, early error return, dropped in-flight execution,
dropped session/agent handles, optional awaited shutdown, and runtime/driver teardown.
Verify callback borrow release and loss of callback access after drop. Instrument
owned child processes to distinguish observed exit/reaping from merely sending a
signal; test graceful and force-termination paths under bounded deadlines.
Preserve shutdown errors in the explicit awaited path and document diagnostic
handling for destructor-initiated cleanup errors. Exercise a non-Send local capture
across await and reject overlapping mutable session use in a compiler probe.

- **Anchor coverage:** None — runtime feasibility, not a production cross-platform
  cleanup guarantee.
- **Evidence:** Passed locally on macOS: success and explicit shutdown;
  application early error and session/agent drop; drop while a non-Send callback
  is pending; driver destruction through Tokio runtime teardown. Fixtures
  observe EOF for graceful shutdown, ignored SIGTERM followed by process
  disappearance for force termination, and process-table removal after runtime
  teardown. This distinguishes exit/reaping from signal delivery alone; it does
  not establish descendant-tree cleanup. Nonzero exit is returned by shutdown
  and retained in Diagnostics for destructor-initiated cleanup. Two rustdoc
  compile-fail probes reject overlapping session use and mutation of a captured
  local while its execution exists. The direct HTTP boundary test confirms
  revocation and rejection through old queue handles. Platform limits below.

### Verification and review plan

Each asynchronous test/process fixture gets a finite timeout and owns its cleanup
resources. Use deterministic inputs to establish failure boundaries; live prompts
establish interoperability, not exhaustive correctness. Run formatting, Clippy,
ordinary tests and focused compile probes for the isolated package, plus Book
structure/rendering, changed links, and applicable bootstrap checks. No new formal
proof, mutation, or CI framework is assigned by this scoped experiment.

After implementation, conduct a separate review pass against every criterion.
Report design changes required by findings instead of weakening criteria to match
what happens to work. Record the tested uncommitted tree or revision and exact
commands; distinguish local evidence from CI and integration. Human review and
integration remain separate requirements for durable completion.

### Exit and next implementation recommendation

If the criteria hold, use the demonstrated design to propose the first product
slice in crates/thinkwell, with explicit behavioral anchors, chosen dependency and
runtime support scope, and acceptance tests. If they do not, revise the affected
provisional design with the owner before expanding the API. Do not carry every
experimental helper or fixture into the product automatically. Promotion and
public API implementation require their own scoped assignment.

### Evidence and completion record

The owner approved this milestone after approving the integrated design.
Implementation and a separate criterion-by-criterion self-review are locally
complete on 2026-11-19. Status remains In progress pending human review and
authorized integration, not Complete. No commit, push, or publication occurred.
The project remains in the Porting Phase with the engine in prototyping mode.

The experiment extends the existing isolated experiments/acp-integration package;
crates/thinkwell and the workspace dependency selection are unchanged.
The tested uncommitted source is based on port-design at
83ac7e6faf7b4e7dfd74c306436760e44ba46a21.
Its source-set SHA-256 is
f0279a62b675aa04f1f91d3078b4e19c4973436bb926529342b7af1a6d5f7a95.
Reproduce by sorting package-relative paths for Cargo.toml, Cargo.lock,
src/**/*.rs, tests/*.rs, and tests/*.py, and hashing each path, NUL, file bytes,
NUL in sequence. This identifies local evidence without implying a commit or CI.

### Implemented experimental policies

The local future holds the mutable session borrow and application callback.
The existing Tokio driver owns ACP transport, subprocess lifetime, and an HTTP
server; its channels carry only owned data. A per-turn route guard revokes tool
access when execution completes or drops. The driver observes a dropped terminal
receiver and closes the connection; session/agent destruction signals shutdown.
No experiment destructor spawns a task or waits for async work. Runtime teardown
drops the SDK's owned-process guard; SDK/async-process fallback reaping can finish
after Drop returns.

There is one serial callback dispatcher, a capacity-one request queue, four
application calls per turn, and two invalid-result submissions before exhaustion.
A valid candidate is immutable; duplicate submissions are rejected without
replacing it. Malformed/missing value and contract/deserialization failures count
toward the correction budget. Stale IDs are rejected at the HTTP boundary and
do not consume that budget. A candidate requires normal terminal completion;
remote cancellation, callback/terminal overlap, timeout, or lost driver does
not produce T. Application-tool errors are returned to the agent and may be
handled without necessarily failing the turn.

Unsupported schema generation fails before sending a prompt. Every started turn
that fails or is abandoned permanently invalidates this experimental session.
There is no automatic conversation reset or cancellation recovery. That conservative
choice establishes safe non-reuse; it does not resolve a future product policy
for settling and recovering cancelled sessions. Subsequent owner review selected
this initial invalidation policy as `D-initial-cancel-invalidation`; recovery is
deferred rather than a prerequisite for the first product slice.

Deadlines: setup 30 seconds, local turn 90 seconds, driver prompt 95 seconds,
HTTP invocation 10 seconds, HTTP shutdown 7 seconds, explicit shutdown 12 seconds,
and whole experimental driver lifetime 240 seconds. The SDK uses its existing
one-second EOF and two-second Unix termination grace periods before force
termination. These are test bounds, not proposed product defaults. A successful
explicit shutdown reports the owned driver's outcome; nonzero child exit or HTTP
shutdown failure remains an error. Diagnostics retains the outcome without
retaining the owner. Runtime teardown may prevent a diagnostic outcome from being
written; an absent outcome is not confirmation.

### Schema scope and provenance

No additional validator library was selected. result_contract.rs is an original,
deliberately finite interpreter over schemars-generated JSON, followed by Serde
deserialization. Source/license inspection confirmed schemars 1.2.3 is MIT
(Copyright Graham Esau), serde_json 1.0.150 is MIT OR Apache-2.0, and the official
ACP SDK 5.4.0 is Apache-2.0. Schemars' primitives implementation explains its
int64 annotation; SDK AcpAgent and async-process 2.5.0 cleanup source informed
the lifecycle tests. No third-party implementation was copied or adapted.
Existing dependency licenses continue to apply.

Supported contracts: object properties/required fields, boolean
additionalProperties, arrays with homogeneous items, string/boolean/null/number/
integer types, const/enum, and oneOf/anyOf sufficient for internally tagged enums.
The int64 format is enforced as signed 64-bit. Titles, descriptions, and schema
dialect metadata do not validate values. Integer representation must be an integer
JSON number; floating representations such as 3.0 are conservatively rejected.
References/$defs, arbitrary formats, numeric/string bounds, and every other
unrecognized keyword fail before prompting. This is neither full JSON Schema
support nor Serde-only validation. Broader schema compatibility needs a future
dependency/design decision; this interpreter should not be promoted automatically.

### Review findings and limits

The separate self-review found and fixed a missing-value/null ambiguity in the
result envelope; a regression test distinguishes an absent value from an explicit
null for a unit result. Initial checks also exposed schemars' int64 annotation,
which now has explicit enforcement rather than being silently ignored. The
Python fixture originally treated host transport closure on budget exhaustion as
an unrelated BrokenPipe failure; it now handles that expected closure. Genuine
nonzero child exits remain tested and preserved as errors.

In the original foundation experiment, turn IDs protected internal result
submissions only; the follow-up below extends the guard to application calls.
Revoked queues cannot reach
a later callback, and abandoned turns cannot rebind callbacks. An arbitrary late
application-tool request arriving over HTTP after a *normally completed* turn has
been rebound is not distinguishable by its tool name. This remains a product
routing/design gap; the experiment assumes a cooperating peer's terminal response
ends its turn's tool issuance. Concurrent outstanding HTTP senders are not globally
bounded by the capacity-one queue. A disconnected HTTP caller does not guarantee
immediate cancellation of a local invocation.

Cleanup tests cover directly owned Python children on macOS and one live Claude
configuration. They do not prove cross-platform behavior, detached descendant
cleanup, abrupt host process death, or application asynchronous destructor work.
Effects performed before cancellation are not rolled back. Driver teardown uses
the SDK's process guard and async-process reaper, not a new general runtime layer.
The public Plan/preset/catalog API and embedding remain outside this experiment.

The result supports retaining the reviewed RAII/per-turn-result direction.
Before product promotion, resolve supported runtime/schema scope and the late
application-tool routing boundary, then assign a product slice with anchors and
acceptance tests. No additional product milestone is assigned here.

### Verification commands and results

Run from the repository root:

```sh
CARGO_TARGET_DIR="$PWD/target/acp-integration" cargo test --manifest-path experiments/acp-integration/Cargo.toml --locked
CARGO_TARGET_DIR="$PWD/target/acp-integration" cargo clippy --manifest-path experiments/acp-integration/Cargo.toml --all-targets --locked -- -D warnings
cargo fmt --manifest-path experiments/acp-integration/Cargo.toml -- --check
CARGO_TARGET_DIR="$PWD/target/acp-integration" cargo run --manifest-path experiments/acp-integration/Cargo.toml --locked --bin runtime_results > target/runtime-results-live.json
cargo anchor check --config anchorbook.toml
/Users/dherman/Code/thinkwell-rs/target/tools/bin/mdbook build
cargo test -p anchor-runner --test bootstrap_consistency --locked
git diff --check
```

Rust 1.100.0: 23 ordinary tests and two compile-fail probes passed; Clippy and
formatting passed. The final live invocation passed both turns and confirmed
shutdown. Across development there were two successful two-prompt live runs and
one initialization-only attempt stopped by unsupported-schema detection before
any prompt. No live inference occurs in cargo test. Book structural checking and rendering, affected local links, whitespace checks
(including new files), and both bootstrap consistency tests passed after final
reconciliation. No CI run or integration is claimed.


### Application-tool turn association {#application-tool-turn-association}

The owner accepted initial cancellation invalidation and agreed to the proposed
exploration of library-managed application-tool turn identifiers. The initial follow-up
was protocol/design research with no code changes or live calls. The subsequent
authorized implementation and its evidence are recorded below.

**Protocol findings.** The ACP
[prompt-turn flow](https://agentclientprotocol.com/protocol/prompt-turn)
states: “If there are no pending tool calls, the turn ends” and requires a
response to the original prompt request. Ordinary tool execution is followed
by further model processing until completion. This supports using normal
completion as a turn boundary for cooperating agents. The late-call scenario
is defensive: background work, agent bugs, or an integration that treats
completion more loosely. It was not observed in the live experiment.

The inspected official schema 1.19.0 has session_id, prompt, and optional
metadata on v1 PromptRequest. It does not specify propagation of a client turn
identifier to MCP calls. Its unstable native MessageMcpRequest identifies the
server and inner request, with optional metadata, but no required originating
prompt association. Optional metadata is an extension opportunity, not automatic
interoperability. The live agent previously advertised HTTP rather than native MCP.

MCP [tools](https://modelcontextprotocol.io/specification/2025-11-25/server/tools)
uses tool name and arguments; the JSON-RPC request ID correlates a call with its
response, not with an ACP prompt. HTTP
[session management](https://modelcontextprotocol.io/specification/2025-11-25/basic/transports#session-management)
assigns an optional MCP-Session-Id during initialization and reuses it for subsequent
requests. Rotating that server session cannot by itself establish which ACP turn
caused a retry/reinitialization. Progress tokens and task identifiers likewise
must not be assumed to identify a Thinkwell turn.

Source inspection of the cached Claude ACP 0.23.1 confirms HTTP URLs and headers
are copied when creating the session. Its prompt loop generates an internal
prompt UUID and accounts for background task results after an earlier prompt loop
has ended. Neither observation establishes a portable prompt identifier on HTTP
tool requests, nor demonstrates a late call to our application tool. No agent
source was copied or adapted. These are observations about the inspected versions,
not an assertion about all present/future protocol extensions.

**Recommended next mechanism.** Preserve the fixed catalog and ordinary Rust
callback arguments. Give every borrowed application tool a library-owned wire
envelope, for example:

```json
{
  "turn": "opaque-turn-A",
  "input": { "comment": "Clarify this paragraph" }
}
```

The envelope's turn field is always required, while the application input schema
is derived as before. Each prompt supplies a fresh opaque identifier. Thinkwell
checks it before queueing application work, unwraps input, and invokes the ordinary
callback. Missing, old, or unknown identifiers fail without touching application
state; the runtime must never silently replace an old identifier with the current
one or return the current identifier to a stale caller as a repair hint.
This leaves the catalog schema unchanged between turns. It adds model-facing
protocol overhead and possible tool errors, not a parameter to the user's Rust
closure. The exact envelope naming is experimental.

This is a stale-identifier guard, not authenticated proof of causal origin.
A delayed request constructed with A's identifier will be rejected during B;
work that learns and submits B's valid identifier cannot be distinguished from
B's work by this mechanism. A server-side proxy that merely stamps the current
identifier on arrival has the same original ambiguity. Stronger origin isolation
requires the agent to bind an immutable turn context when it schedules the call,
or another cooperating transport mechanism.

**Approved bounded verification scope.** Extend the existing shared core and
fixed application-tool schema with this envelope, then inject an actual A-tagged
call while B's callback is installed. Check zero mutation/callback entry for
stale, missing, and malformed identifiers; correct B calls must succeed.
Also retain queued-before-revocation tests and the cancellation non-reuse checks.
Repeat the live two-result workflow to assess whether the pinned agent reliably
supplies the argument without Rust caller changes. Preserve result-correction
and application-tool error distinctions. This is the concrete implementation
follow-up to the agreed exploration; execution evidence is recorded below.

**Research validation.** Public protocol pages were read alongside the exact
cached schemas and agent implementation above. Book structure/rendering and
affected links were checked after reconciliation. At that research stage, code test results
remained those of the original source digest. The implementation below has its
own source identity and validation.


### Turn-envelope follow-up assignment

The owner explicitly said “proceed” after reviewing the concrete bounded
verification proposal above. This extends runtime-results-prototype-1 within the
isolated experimental package. Scope recorded Ready, then In progress after
readiness refresh: add the fixed application envelope, reject stale/missing/
malformed identifiers before callback dispatch, retain revocation/cancellation
checks, and repeat the live two-turn workflow. No product API or publication is
assigned. Existing local evidence remains historical until rerun for this change.


### Turn-envelope implementation evidence

The approved follow-up is implemented and locally validated. The milestone remains
In progress pending human review and authorized integration. No commit or push
was performed.

runtime.rs now advertises a fixed accumulate contract with required turn and
input fields; input uses the existing derived Input schema. Each prompt supplies
its turn identifier and envelope instructions. The HTTP boundary accepts only
known tool names with a string identifier matching the active route before
cloning its queue handle. Missing, malformed, old, or unknown identifiers return
a tool error without disclosing the current identifier. The local dispatcher
unwraps input.value; Rust application callback signatures are unchanged.
Rejected identifiers do not consume the application-call or result-correction
budget. Existing time limits still bound the turn.

The deterministic Python peer retains A's identifier, waits for B's prompt, and
then sends six rejected requests: A's identifier, missing identifier, null,
numeric, object, and unknown identifier. A valid B request then succeeds.
late_application_calls_cannot_enter_next_turn_callback asserts A received only
3, B received only 4, and all six requests returned errors. The peer also asserts
the catalog is identical across the two turns and rejection messages do not
reveal B's identifier. The prior queue-revocation and cancellation/non-reuse
tests remain passing.

The saved live output target/runtime-results-envelope-live.json confirms one
two-prompt run against Claude ACP 0.23.1, haiku, SDK 5.4.0, and HTTP MCP
2025-11-25: Summary total 3 and labels ["first"], then Decision::Accept total 10,
inputs [3, 0, 4], and confirmed shutdown. The unchanged executable's Rust closures
still receive ordinary integers; protocol wrapping is library-owned. This is
compatibility evidence for that configuration, not a measured reliability rate
or proof that all agents supply identifiers correctly.

Separate self-review checked validation before queue selection, immutable queue
association after selection, lack of identifier disclosure in rejection errors,
and unchanged public callback arguments. The guard does not attest causal origin:
old work supplying B's valid identifier remains indistinguishable from B's work.
The implementation also retains the earlier bounded-schema, macOS process,
HTTP-disconnect, and concurrency exclusions. No new dependency or third-party
source adaptation was introduced.

The final source-set SHA-256, using the same path/NUL/bytes/NUL procedure above, is
5dbb13250cefa93c1539c27bc1ad1e905cff3e907441017ebd950a5902cd4884.
Validation: 24 ordinary tests (8 library, 3 earlier HTTP, 13 runtime integration)
and two ownership compile-fail probes passed. Clippy with -D warnings and
formatting passed. The live command was the previously recorded runtime_results
command with output redirected to target/runtime-results-envelope-live.json.
Book structure/rendering, affected links and whitespace are checked after this
reconciliation. The earlier source digest and 23-test result describe the
pre-envelope foundation rather than the current tree.
