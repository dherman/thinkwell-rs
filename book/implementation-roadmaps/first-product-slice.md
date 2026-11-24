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

**Status:** Complete

See [integrated completion](#integrated-completion) for the merge, review, and CI
evidence closing this milestone. Earlier local-stage statements remain historical.

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

The approved connect/open/bind/two-result workflow is embodied in
crates/thinkwell/examples/conversation.rs. The current
[programming model](../design/typed-conversations.md) defines it alongside one-shot
execution and later composition conveniences. The original first-slice example
and scope remain identifiable in this chapter's Git history; the approval and
criterion evidence below retain their original first-slice meaning.

### Concrete scope and support choices

The owning definitions are now:

- [Typed conversations](../design/typed-conversations.md): descriptors, bindings,
  reuse, preflight, and changing result types.
- [Execution lifecycle](../design/execution-lifecycle.md): runtime ownership,
  setup/turn failures, budgets, cancellation, diagnostics, and shutdown.
- [Contracts and typed results](../design/contracts.md): finite schemas, validation,
  wire values, corrections, duplicates, and terminal completion.
- [Agent integration](../design/agent-integration.md): connection configuration,
  protocol scope, credentials, permissions, and turn routing.

Later approved library changes extend that target. They do not expand the scope
of this milestone's historical tests or make its original six-anchor evidence
cover progress observation. The original first-slice dependency selection and
platform/live-agent observations remain in the evidence below.

### Product properties and verification

The original six anchors and their unchanged test/review criteria now reside in
the owning chapters above. Their identities and the coverage recorded below are
preserved. `A-progress-observation` belongs to its separate roadmap. No new proof
or mutation evidence is claimed by consolidating the definitions.

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
  public conversation path. Final-head and merge-commit Ubuntu CI subsequently
  passed the product fixtures; see [integration evidence](#integrated-completion).
  This establishes the named macOS/Linux checks, not other platforms or process descendants.

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
  below. Owner review, final-head and merge-commit Linux CI, and integration
  are recorded under [integrated completion](#integrated-completion).

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

### Built-in agent namespace

The owner approved moving Claude to `thinkwell::agent::Claude` and updating the
draft PR. The public `agent` module now contains the preset; it is no longer
exported at the crate root. `Agent`, `AgentConfig`, and `Connect` keep their root
paths. Launch configuration and connection behavior are unchanged. Rustdoc links,
examples, integration-test imports, and the Book example use the new path.

Local all-target compilation, Clippy with warnings denied, strict Rustdoc, and
workspace tests passed, including all 28 doctests, 19 product tests, and five
launcher/bootstrap tests. Formatting, Book structure/rendering, and whitespace
checks are required before delivery. GitHub CI for this revision remains unverified.


### Copilot review corrections

The owner requested evaluation and then correction of both findings in
[Copilot's review of 244d27f](https://github.com/dherman/thinkwell-rs/pull/2#pullrequestreview-5988211212).
Implementation-author evaluation confirmed both against the current code and the
pinned Schemars 1.2.3 settings. Tool output contracts now use serialization mode;
inputs and final answers retain explicit deserialization mode. Both paths still
apply the same finite-schema support checks. The experimental Python capability
probe accepts a matching response ID only when the message has no method, so an
incoming request with the same ID is rejected normally instead of mistaken for
a response. These fixes change no public signatures or accepted support scope.

The product regression exercises different serialize/deserialize names for tool
input, output, and final answer, verifies a single state update and successful
result, and checks the advertised output schema. It failed before the correction
and passes afterward. Existing rejection of deliberately invalid serialized output
still passes. The Python regression drives the full probe against a local scripted
peer with overlapping IDs; it demonstrably fails against the old matcher and
passes against the corrected matcher. It is now included in CI and documented
local checks, without credentials or model calls.

On base 244d27f plus this correction, all-target compilation, formatting, Clippy,
strict Rustdoc, 20 product tests, five launcher/bootstrap tests, 28 doctests, and
the Python probe regression passed locally. Book structure/rendering and whitespace
checks are required before delivery. A focused implementation-author review checked
schema direction at all three use sites, preservation of validation, and request
versus response dispatch. GitHub CI for these corrections remains unverified;
owner review and authorized integration are still outstanding.


### Integrated completion {#integrated-completion}

The owner merged [PR #2](https://github.com/dherman/thinkwell-rs/pull/2) on
2026-11-20 at 06:26:27 UTC, then requested milestone closeout and preparation of
the next example proposal. GitHub identifies `dherman` as the merger. The final
head is `e5a5c9da5f52e81cb8d4238481c272027b4efa26`; the integrated revision is
`1b21c83b9b713690d953df72244c7f9fcf8244ce`.

Both **Rust and Book checks** executions passed before merge:
[push run 44687043391](https://github.com/dherman/thinkwell-rs/actions/runs/44687043391)
and [PR run 44687046908](https://github.com/dherman/thinkwell-rs/actions/runs/44687046908).
The [main push run 44687815722](https://github.com/dherman/thinkwell-rs/actions/runs/44687815722)
also passed on the integrated revision. These supply Ubuntu CI evidence for
workspace compilation, Clippy, strict Rustdoc, 20 product tests, 28 doctests,
five launcher/bootstrap tests, the Python probe regression, Book checks/rendering,
and whitespace checks. CI does not run real-model inference or the separate Rust
experiment suites; their bounded local evidence remains as recorded in their roadmaps.

Owner review in this conversation drove the newcomer-focused documentation and
`agent::Claude` namespace changes. The owner then requested evaluation and fixes
for both Copilot findings and merged the resulting revision. This records owner
review and integration without inventing a GitHub approving review. The separate
implementation-author reviews remain identified as self-review. Copilot's two
findings—serialization schema direction and request/response ID collisions in
the capability probe—were corrected in e5a5c9d, with regressions described above.
No remaining finding from that review is deferred by this closeout.

All five product criterion groups now have their recorded local, live, review,
and applicable CI evidence and an integrated implementation. The first product
slice is Complete. PR #2 also integrated the prior lifecycle synchronization,
ownership experiment, both ACP experiment milestones, integrated API review,
and runtime/result experiment including the turn-envelope follow-up. Their
outcomes and bounded criterion evidence are complete; their milestone statuses
are reconciled accordingly. Historical source digests and limitations are retained.

This documentation follow-up records the integrated state on a fresh branch
based on 1b21c83; its own delivery remains separate from PR #2. It introduces no
product code or new supported capability. The scheduler milestone has
[approved scope](meeting-scheduler.md); implementation awaits integration
of this closeout. The project remains in the Porting Phase and unpublished.

### Closeout documentation validation

On the `example-milestone` documentation tree based on 1b21c83,
`cargo anchor check --config anchorbook.toml` passed structural/v1 with zero errors,
`target/tools/bin/mdbook build` passed with mdBook 0.5.4,
`cargo test -p anchor-runner --test bootstrap_consistency --locked` passed both tests,
`cargo fmt --all -- --check` and `git diff --check` passed. Inspection of generated
HTML checked 224 local links/fragments across changed chapters, including the new
proposal and completion records. A separate author review checked scope/status
consistency, criterion evidence, CI revision identities, and the distinction between
candidate dependencies and tested product support. This is not independent review.

No product sources or dependencies changed, so product/runtime tests and live
inference were not rerun for the documentation-only closeout. The scheduler
planning probe is described in its own roadmap and does not establish solver or
agent interoperability. On 2026-11-20 the owner approved the scheduler scope and
explicitly authorized publishing this closeout/proposal as a new draft PR. The
historical PR #2 CI pass is not a CI pass for this follow-up. Its CI and owner
merge remain outstanding; approval does not itself integrate the closeout.
