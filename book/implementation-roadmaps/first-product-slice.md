# First product slice: typed conversations

Roadmap identifier: `first-product-slice`.

Goal: make the reviewed programming model usable from crates/thinkwell through
one coherent, tested path: connect, open a conversation, bind borrowed tools,
run differently typed turns, and release resources through ordinary ownership.

## Milestone 1. Deliver the initial typed conversation API {#first-product-slice-1}

**Outcome:** An unpublished Thinkwell library provides the public API below,
with deterministic acceptance tests and an opt-in live example. Application code
can alternate between Rust and typed prompts without explicit cleanup boilerplate
or protocol turn identifiers in its tool signatures.

**Status:** In progress

**Identifier:** `first-product-slice-1`

**Dependencies:** The owner approved the concrete proposal and instructed
“proposal looks good, proceed.” This assigns the product slice and authorizes
local continuation on the current unintegrated branch; prior work remains
pending delivery, not silently Complete. No commit, push, merge, publication,
or lifecycle transition is authorized. Scope recorded Ready, readiness reported, then In progress.

**Design basis:** `D-acp-client`, `D-derived-contracts`,
`D-provisional-scoped-turns`, `D-connection-vocabulary`,
`D-reviewed-api-baseline`, `D-initial-cancel-invalidation`,
`R-easy-agent-programming`, `R-code-prompt-ergonomics`,
`R-provider-breadth`, `R-idiomatic-reimplementation`, and
`R-per-turn-result-types`; the
[reviewed examples](integrated-api-review.md) and
[foundation evidence](runtime-results-prototype.md).

**Tasks**

- [x] Obtain approval of this scope and settle the delivery prerequisite; record
      Ready, report readiness, then record In progress.
- [x] Incorporate the approved properties below as product anchors with test/review
      criteria, reconciling design, authority, supported environments, and this roadmap.
- [x] Implement the public types and example in crates/thinkwell, with normal
      documentation and actionable typed errors.
- [x] Implement owned-driver lifecycle, fixed contracts, borrowed bindings,
      validation, result completion, and turn-envelope checks.
- [x] Build deterministic protocol/process acceptance tests against the product API;
      add compiler probes and the opt-in live example.
- [x] Run local checks and the live example, perform a separate review, and record
      findings, exact tested source, unsupported scope, and integration state.

### Approved caller example

This approved target is implemented in crates/thinkwell/examples/conversation.rs.
During implementation it must become a normal compiled example with these same
user-visible steps; no protocol fixtures or explicit cleanup are required in the
ordinary caller flow. Compilation may refine inferred error annotations and
imports without introducing callback scopes or mandatory cleanup.

```rust,ignore
use thinkwell::{Claude, Plan, Tool};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, JsonSchema)]
struct Add {
    value: i64,
}

#[derive(Serialize, JsonSchema)]
struct Total {
    total: i64,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Summary {
    total: i64,
    labels: Vec<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "kind", deny_unknown_fields)]
enum Decision {
    Accept { total: i64 },
    Reject { reason: String },
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let agent = Claude::connect().await?;
    let add = Tool::<Add, Total>::new("add", "Add to the accumulator");
    let mut session = agent.session().tool(&add).open().await?;
    let mut total = 0;

    let summary = Plan::<Summary>::new()
        .text("Add 3, then summarize the total with labels [first].")
        .bind(&add, async |input: Add| {
            total += input.value;
            Ok(Total { total })
        })
        .run_in(&mut session)
        .await?;

    assert_eq!(summary.total, 3);
    assert_eq!(summary.labels, ["first"]);
    total *= 2;

    let decision = Plan::<Decision>::new()
        .text("Add 4, then return Accept with the new total.")
        .bind(&add, async |input: Add| {
            total += input.value;
            Ok(Total { total })
        })
        .run_in(&mut session)
        .await?;

    assert!(matches!(decision, Decision::Accept { total: 10 }));
    assert_eq!(total, 10);
    Ok(())
}
```

The live verification harness also exercises optional awaited shutdown so its
process-exit evidence is explicit. Ordinary examples retain the RAII default.
Dropping the owning runtime immediately afterward permits forceful fallback;
it does not promise that asynchronous graceful cleanup finished before main returns.

### Concrete scope and support choices

**Runtime and platform.** Require an existing Tokio 1.x runtime with I/O/time
enabled, supporting both current-thread and multithread runtimes. Borrowed callback
futures may be non-Send and remain on the calling task; owned I/O tasks may be
spawned. Use the official ACP SDK 5.4.0 initially. Establish macOS and Linux
support through deterministic owned-process tests; Linux is verified by the
existing Ubuntu CI job after an authorized push. Windows, runtime neutrality,
shared host-owned connections, and an MSRV below the current Rust 1.100.0
development toolchain are outside this slice. These are initial support limits,
not permanent architectural exclusions.

**Agent construction.** Provide one Claude preset, the accepted default
Claude::connect() shorthand, and configured instance connect through an exported
extension trait. Presets convert to an owned AgentConfig; Agent::connect(config)
is the custom command/arguments/environment path. This is an open conversion
boundary, not a closed provider enum or string-based provider registry.
Keep installation and authentication external. Use Claude ACP 0.23.1 as the
initial interoperability target; no installer or automatic authentication.
The normal path uses the agent's default model. Expose an optional session model
selection, tested using haiku, and report unsupported selection explicitly.
Do not promise that all custom ACP agents support the chosen HTTP MCP path.

**Ownership.** Session borrows Agent; an execution exclusively borrows Session.
Initially allow one open session per connection; a second open returns a typed
busy error rather than waiting indefinitely. After normal session drop the agent
may open a fresh session, with a distinct endpoint credential and no reused
conversation history. Drop revokes the old route. A cancelled/abandoned started
turn makes its session permanently unusable and closes its dedicated connection;
subsequent use of that Agent returns a closed error. Recovery requires an explicit
new connection. Dropping an unpolled execution sends no prompt and need not
invalidate the session. Validate these boundaries rather than transplanting the
experiment's consuming Agent::open convenience.

**Plans and tools.** Include `Tool<Input, Output>`, the fixed-catalog session builder,
`Plan<Result>::new/text/bind/run_in`, and a cloneable unbound base plan.
Bound plans carrying borrowed closures need not be Clone. Preflight rejects
duplicate names, missing bindings, mismatched descriptors, reserved internal-tool
names, and unsupported contracts before a prompt is sent. Require a binding for
every declared application tool each turn. Input implements DeserializeOwned and
JsonSchema; Output implements Serialize and JsonSchema; final Result implements
DeserializeOwned and JsonSchema. No custom procedural-macro crate is needed yet.
Standalone Plan::run, inline tool declaration, quoting helpers, observation/event
streams, explicit cancellation handles, dynamic catalogs, and parallel sessions
are deferred. A caller can cancel this slice by dropping its execution future.

**Schemas.** Use Serde 1, schemars 1.2.3, and serde_json 1, with resolved versions
in the workspace lockfile. Support a documented finite contract subset:
object properties/required fields and boolean additionalProperties; homogeneous
arrays; strings, booleans, null, numeric values; integer JSON representation and
signed int64 enforcement; const/enum; oneOf/anyOf sufficient for internally tagged
enums. Reject references/$defs, unknown formats/keywords, and unsupported constraints
before starting a turn. This deliberately excludes some otherwise ordinary Rust
types and nested named types whose generated schemas contain references.
Provide a schema-location error explaining the unsupported feature.

Use an internal validator for that subset, with contract tests independent of
Serde conversion. The experimental interpreter is source evidence, not an automatic
product component: review each accepted keyword and numeric semantics before
adoption, reject unsupported dialects, and document conservative integer handling
(for example, rejection of floating representation 3.0 for an integer contract).
Validate tool input, serialized tool output, and final result against their
advertised supported contracts, not only against Serde's conversion behavior.
A broader JSON Schema validator and reference resolution are future scope.

**Transport and result handling.** Use one loopback HTTP MCP server per active
session, an ephemeral bearer credential, Origin rejection, a bounded request body,
and a fixed catalog. Restrict permissions to that session's registered tools.
Keep HTTP MCP 2025-11-25 as the initial tested path; native ACP MCP and the newer
polyfill are not requirements. Every application tool requires the library-owned
turn/input envelope; the result tool requires turn/value. Generate fresh identifiers,
validate before choosing a callback queue, never repair stale identifiers by
substituting the current one, and never disclose the current identifier in stale-call
errors. Reject unknown tools without granting access.

A candidate becomes successful only after normal end_turn and settlement of local
callback work. A duplicate valid submission cannot replace the first candidate.
Missing result, cancellation, non-normal stop, validation exhaustion, transport
loss, and deadlines have distinguishable errors. Application-tool failures are
reported as tool errors and may be handled by the agent.

**Budgets and errors.** Expose a small RunOptions value rather than hard-coding the
experiment's lifetime limits. Defaults: 120-second turn deadline,
30-second callback deadline, 32 application calls, and at most three invalid result
submissions (two opportunities to correct). Expose bounded connection/setup and
shutdown options with proposed defaults of 30 and 12 seconds. These values are
approved initial choices, not experimentally established universal defaults. No fixed
whole-agent lifetime cap. Validate nonsensical options before starting work.
Use non-exhaustive public error enums or error structs with documented categories;
do not expose opaque SDK error strings as the whole error contract.

**Cleanup.** Ordinary Drop releases local borrows and revokes routes, then signals
the already-running owned driver. It neither blocks on asynchronous work nor
spawns a fresh shutdown task. The driver attempts bounded shutdown; process guards
provide forceful fallback on driver/runtime loss. Optional Agent::shutdown().await
reports the outcome after sessions have been dropped. Provide an owned diagnostic
handle for observing destructor-initiated cleanup failure without retaining Agent.
No rollback, arbitrary asynchronous callback destructors, detached-descendant
guarantee, or synchronous remote acknowledgment on Drop is promised.

### Product properties and verification

These approved properties are incorporated as accepted anchors in
[typed conversations](../design/typed-conversations.md), with test/review criteria. No formal
proof or mutation framework is proposed for this bounded first slice.

| Proposed property | Authority | Required verification |
| --- | --- | --- |
| scoped-turn-ownership: only the active execution can access its bound callbacks; application borrows end when it is consumed or dropped; overlapping mutable session use is rejected | D-provisional-scoped-turns; R-code-prompt-ergonomics | Tests, compiler probes, ownership review |
| typed-turn-completion: supported-schema validation and conversion, a valid candidate, normal terminal response, and settled local callbacks are all required for successful T; type may change between turns | R-per-turn-result-types; D-derived-contracts | Tests and protocol/state-machine review |
| turn-envelope-routing: a missing, malformed, or nonmatching turn identifier cannot select or invoke the current turn's callback; revoked queue handles cannot reach replacement callbacks | D-provisional-scoped-turns | Deterministic HTTP tests and dispatch-boundary review |
| cancelled-session-invalidation: a cancelled or abandoned started turn cannot be followed by another prompt on that session, nor by an implicit history reset | D-initial-cancel-invalidation | Tests and lifecycle review |
| owned-resource-cleanup: Drop revokes local access and initiates owned cleanup without awaiting or new task creation; explicit shutdown reports bounded outcome; guard fallback terminates the directly owned process on supported platforms | D-reviewed-api-baseline | Instrumented process tests and resource-ownership review |
| supported-contract-boundary: unsupported schemas fail before prompting and advertised supported tool/result constraints are enforced at their respective boundaries | D-derived-contracts; R-easy-agent-programming | Contract/error tests and schema review |

The routing property identifies submitted credentials, not semantic or authenticated
causal origin. A sender that supplies the current valid turn identifier is within
the current routing authority even if its work began earlier. Document this
explicitly rather than claiming that the envelope proves a tool call's origin.

### Completion criteria

#### Public programming path {#first-product-slice-1-api}

The example above compiles and runs through the public crate API. Tests also
exercise an external custom preset/config conversion, configured/default connect,
cloneable unbound plans, reusable Agent after normal session drop, and informative
busy/closed/preflight failures. Callback code can hold Rc across await and mutate
borrowed locals between turns.

- **Anchor coverage:** `A-scoped-turn-ownership`.
- **Evidence:** Local pass. The compiled conversation and live_check examples use
  the public API. conversations.rs covers external preset conversion, configured
  connect, base-plan cloning, normal session reopening with fresh credentials,
  busy/closed/preflight errors, and Rc state across await on current-thread and
  multithread Tokio. Two rustdoc compiler probes reject overlapping session
  borrows and mutation of captured state. Default Claude::connect is exercised
  by the live example.

#### Contracts and turn results {#first-product-slice-1-results}

Deterministic tests exercise supported and rejected schema features, invalid tool
inputs/outputs, invalid candidates and corrections, exhausted budgets, missing
result, duplicates, non-normal terminal responses, transport loss, and candidate
acceptance followed by cancellation or callback/terminal overlap. All scenarios
drive product code; no parallel test-only state machine.

- **Anchor coverage:** `A-typed-turn-completion`; `A-supported-contract-boundary`.
- **Evidence:** Local pass. schema::tests exercises supported keywords, exact
  numeric const/enum comparison, signed bounds, unsupported dialects and located
  reference/constraint rejection independently of Serde. Product subprocess tests
  exercise invalid input/output, correction/exhaustion, duplicate/missing result,
  transport failure, cancellation after candidate, and terminal/callback overlap.
  Every case uses product dispatch and validation.

#### Routing and cancellation {#first-product-slice-1-routing}

Inject A-tagged application requests while B's callbacks are installed and assert
zero callback entry or state mutation. Include missing/malformed identifiers,
unknown tool names, queue revocation, current-ID success, and unchanged catalog.
Check dropped unpolled versus started executions, permanent session invalidation,
and no subsequent prompt or implicit history reset. Exercise body/call/time bounds.

- **Anchor coverage:** `A-turn-envelope-routing`; `A-cancelled-session-invalidation`.
- **Evidence:** Local pass. typed_turns_and_stale_calls and the Python peer
  inject six rejected calls during the second turn, asserting unchanged state,
  unchanged catalog, and no current-ID disclosure. The private boundary test
  exercises revoked queue handles directly. Preflight, unpolled/started drop,
  closed reuse, body/header limits, callback/turn deadlines, and call/result
  budgets are tested. Causal origin remains outside the stated property.

#### Owned lifecycle {#first-product-slice-1-lifetime}

Test success, partial constructor failure, early caller error, dropped execution,
session and agent destruction, awaited shutdown errors, and runtime teardown.
Observe directly owned child exit/reaping and listener closure, distinguishing
them from signal requests. Cover graceful and forced paths with bounded fixtures,
both supported runtime flavors, and supported platforms. Diagnostics must not
create an ownership cycle; absent diagnostics after runtime loss are not success.

- **Anchor coverage:** `A-owned-resource-cleanup`; `A-scoped-turn-ownership`.
- **Evidence:** macOS local pass against product code. Process tests observe
  child disappearance from ps, EOF and ignored SIGTERM for forced cleanup,
  listener closure, early error return, diagnostics, runtime destruction, and
  malformed session-constructor response. Both Tokio runtime flavors run the
  public conversation path. Linux CI is pending; no cross-platform pass is claimed.

#### Interoperability and review {#first-product-slice-1-review}

An opt-in real-agent run yields two different typed results with deterministic
work between turns, actual borrowed-tool calls, a stable catalog, and confirmed
shutdown. Record actual package/model selector/protocol/library versions and
bounds. Ordinary tests and CI require no model credentials or inference.
Complete a separate review of product code, anchors, claims, tests, dependencies,
licenses, and documentation; record findings and resolutions.

- **Anchor coverage:** All six accepted product anchors, with live coverage limited to the
  chosen agent configuration; live success is not exhaustive verification.
- **Evidence:** Local live check and separate self-review pass, as detailed
  below. Human review, Linux CI, and authorized integration remain pending.

### Verification and delivery

Use the repository's fmt/check/Clippy/workspace-test/Book/bootstrap commands.
Product process fixtures have finite deadlines and owned cleanup guards.
Include the example in compilation, compiler rejection tests, and deterministic
acceptance tests in normal workspace CI; live inference remains opt-in.
Reconcile CONTRIBUTING.md's empty-library description and initial support limits
when implementation actually lands. Inspect source licensing before adaptation
or new dependencies; record dependency versions in Cargo.lock.

Local passing checks do not establish CI or durable completion. Obtain authorized
delivery, inspect checks for the actual revision, and perform owner review before
merge. Keep the milestone In progress until all criteria and integration conditions
hold. Publication and ending the Porting Phase need separate authorization.

### Planning evidence

This proposal was prepared under the owner's explicit planning assignment.
The API shape follows the approved integrated examples, with deliberate deferrals
listed above. Runtime/results experiments supply bounded feasibility evidence;
their code, wire fixtures, hard-coded budgets, hand-written validation, and
consuming connection API are not silently promoted to product authority.
Book structural checking, rendering, local links/fragments, and whitespace
validation passed for the planning artifact. Product work is recorded below.


### Local implementation and review evidence

The implementation follows owner approval on the current unintegrated port-design
branch, based on 83ac7e6faf7b4e7dfd74c306436760e44ba46a21.
The milestone remains In progress. Local implementation is not durable completion:
Linux CI, owner review, and authorized integration are outstanding. No commit,
push, publication, or repository-setting change was performed.

The current product source-set SHA-256 is
235deecba3745c22a71eb944a541b2db1c7cad89bb5060f2b3c65d66d2c62bb9.
Reproduce by lexically sorting repository-relative Cargo.lock,
crates/thinkwell/Cargo.toml, and all .rs/.py files under that crate's src, tests,
and examples, then hashing each path, NUL, bytes, NUL. This identifies the
uncommitted implementation tested here, not a commit or CI revision.

### Review record

A separate implementation-author review checked the six anchors, lifetime
boundaries, public API and example, schema semantics, HTTP authorization,
dependency provenance, and criterion coverage. It is a self-review, not an
independent-agent or owner review.

Findings resolved before the final checks:

- Tool output needed a published MCP object contract, including primitive
  application outputs. The output schema and structuredContent now use a value
  envelope, while callback signatures and textual results stay unchanged.
- HTTP cleanup must not depend on the caller dropping a session after connection
  failure. The driver and session share a shutdown signal; owned task guards also
  revoke routes and abort on runtime teardown.
- A fixed HTTP invocation timeout could override caller-selected longer turn
  limits. The endpoint now uses the active turn's configured bound.
- Numeric const/enum comparison must not round large integers through f64.
  Canonical decimal comparison and regression tests cover values above 2^53 and
  equivalent integral/floating constants. Integer type validation retains the
  explicitly conservative representation rule.
- The candidate-before-terminal test must fail if the candidate marker never
  arrives. Its observation timeout is now asserted, not silently accepted.
- Connection/run bounds reject zero and nonrepresentable deadlines before work.

Fixture development also corrected a test's retained `Result<Session>` borrow before
Agent::shutdown. The compiler rejection confirms the public owner/session
relationship rather than requiring runtime lifetime management by callers.

No third-party implementation was adapted. The product factors this repository's
original experiments; those were research-informed original code. Direct dependency
license metadata was inspected: ACP SDK 5.4.0 Apache-2.0; schemars 1.2.3, Tokio
1.54.1, and axum 0.8.9 MIT; Serde 1.0.229, serde_json 1.0.154, tempfile 3.27.0,
and uuid 1.29.3 MIT/Apache-2.0 alternatives. Cargo.lock records the resolved graph.
Their licenses remain their own; project dual licensing does not relicense them.

### Executed checks

From the repository root, on Rust 1.100.0 and macOS:

```sh
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo run -p thinkwell --locked --example live_check > target/product-live-check.txt
cargo anchor check --config anchorbook.toml
/Users/dherman/Code/thinkwell-rs/target/tools/bin/mdbook build
cargo test -p anchor-runner --test bootstrap_consistency --locked
git diff --check
```

Formatting, compilation, and Clippy passed. Workspace tests passed: 19 product
tests, two ownership compile-fail probes, and five launcher/bootstrap tests.
Examples compile during all-target checks; ordinary tests never invoke inference.

The final live run passed through the product API after review corrections:
Claude ACP 0.23.1, haiku selector, SDK 5.4.0, HTTP MCP 2025-11-25, one session,
two prompts. Summary returned total 3 and labels ["first"]; deterministic code
doubled the accumulator; Decision::Accept returned total 10 after the second
borrowed tool call. Awaited shutdown succeeded. The ignored output artifact records
the pass, and live_check.rs contains the assertions. Two live two-prompt runs
were made during product development; the second covers the final output envelope
and cleanup changes. This is not a statistical reliability claim.

Book structure/rendering, local links/fragments, changed-file whitespace, and
bootstrap consistency are checked after documentation reconciliation. The existing
Ubuntu CI workflow now explicitly checks Python 3 and ps prerequisites before
running the same credential-free tests. It has not been pushed or run for this
tree. Retain this gap before claiming Linux support verified.

### Retained limits and next delivery step

The public slice intentionally has no standalone Plan::run, inline-tool helper,
progress stream, explicit cancellation handle, dynamic catalog, concurrent
sessions per Agent, shared embedding adapter, Windows support, arbitrary schema
references, or installation/authentication automation. Unsupported contracts fail
before prompting; model selection failures are explicit. Every failed started
turn closes its connection, while preflight failure leaves the session reusable.
Callback errors may be handled as tool errors until a turn-level bound is hit.

An old request carrying the current valid turn identifier cannot be attributed
to its original work by the envelope. Cancellation is not rollback. The bounded
HTTP body and queue do not impose a global concurrent-request limit. Process
tests observe directly owned children, not detached descendant guarantees.
Completed HTTP task records are retained for cleanup diagnostics until Agent
shutdown; no session-count memory bound is claimed. Runtime destruction may
prevent diagnostic publication; absent outcome is not cleanup confirmation.

The next delivery action is owner review of this local product slice and
authorization for its branch/PR delivery together with the outstanding prior
work. After authorized publication, require current Linux CI and owner merge
approval, then synchronize durable evidence before another milestone.

### Draft PR delivery authorization

The owner requested a draft PR for review in the GitHub UI. This authorizes
committing and pushing the current design, original experiments, and product
slice to port-design and opening a draft against main. Merge, package publication,
and ending the Porting Phase remain unassigned. Prior “not pushed” statements
above describe the local validation stage. The PR will carry the tested commit
and current CI state; milestone status remains In progress pending review,
passing required CI, and authorized integration.


### Rustdoc CI follow-up

The owner requested generating Rustdocs in GitHub CI. The existing Rust and Book
checks job now runs `cargo doc --workspace --no-deps --locked` with
`RUSTDOCFLAGS="-D warnings"`. CONTRIBUTING.md documents the matching local check
and target/doc output. This checks Rustdoc generation and warnings separately from
the existing doctests. It adds no deployment or package-publication step.

The first local Rustdoc run caught an unclosed HTML tag in the plain-text
`Into<AgentConfig>` API comment. Marking it as inline code fixes the warning.
This is a documentation-only source change after the product digest above.
The corrected Rustdoc build, doctests, Book checks/rendering, bootstrap consistency,
and whitespace checks passed locally; CI verification of this follow-up is pending.

The owner also asked to view the generated output. CI uploads target/doc as the
rustdoc artifact using a SHA-pinned actions/upload-artifact v4, retained for
14 days; CONTRIBUTING.md explains downloading and opening thinkwell/index.html.
The generated files include sibling assets and source pages. No external service
or hosted-site deployment is configured. Upload execution awaits GitHub CI.


### Public API Rustdoc coverage

The owner requested thorough documentation for all public APIs and concrete
examples for major APIs. Crate-level guidance now introduces the ownership model,
a quick start, supported schemas, and initial limits. Public types, enum variants,
fields, traits, and methods document their behavior; the major workflows include
examples for presets/configuration, custom preset conversion, session reuse,
borrowed tools, changing result types, budgets, errors, diagnostics, and shutdown.

The crate now denies missing public API documentation. Agent-dependent examples
use no_run: they are compiled by doctests without starting processes or inference.
Local examples execute, and both existing ownership compile-fail examples remain.
This follow-up changes documentation and its enforcement, not runtime behavior;
the earlier product source digest is historical. Contributor guidance records
the policy. Local Rustdoc with warnings denied and all 23 doctests passed
(12 compile-only examples, nine runnable examples, two compile-fail probes).
Final workspace and Book checks are run before updating the draft PR.


### Learning-focused Rustdoc revision

Owner review found the initial API docs too dense and architectural for newcomers.
The revised crate introduction follows a first conversation, explains Agent,
Session, and Plan through that example, and links to progressively richer uses.
Public API pages lead with practical purpose and small examples: follow-up
questions, structured answers, word counting, and a borrowed shopping list.
Cleanup, errors, and supported schemas remain documented after the basic workflows.
Contributor guidance records this learning-first approach.

The revision changes documentation only: comparison against e0ea812 after removing
Rustdoc comments and whitespace confirms identical executable source in all three
edited Rust files. Local strict Rustdoc generation and all 28 doctests passed
(18 compile-only examples, eight runnable examples, two compile-fail probes).
Rendered crate and major type pages were inspected for introductory text and
examples. Formatting, compilation of all targets, and Clippy with warnings denied
passed. All 19 product tests and five launcher/bootstrap tests passed via
`cargo test --workspace --locked`. Book structure, rendering with pinned mdBook
0.5.4, and whitespace checks passed. The ambient mdBook was 0.5.1; installing
the documented 0.5.4 under ignored target/tools resolved that rendering mismatch.
GitHub CI for this revision has not yet been verified.
No product behavior, API decision, or milestone status changes in this revision.
