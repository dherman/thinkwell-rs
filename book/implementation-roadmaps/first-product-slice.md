# First product slice: typed conversations

Roadmap identifier: `first-product-slice`.

Goal: make the reviewed programming model usable from crates/thinkwell through
one coherent, tested path: connect, open a conversation, bind borrowed tools,
run differently typed turns, and release resources through ordinary ownership.

## Milestone 1. Deliver the initial typed conversation API {#first-product-slice-1}

**Outcome:** An unpublished Thinkwell library provides the public programming path,
with deterministic acceptance tests and an opt-in live example. Application code
can alternate between Rust and typed prompts without explicit cleanup boilerplate
or protocol turn identifiers in its tool signatures.

**Status:** Complete

See [integrated completion](#integrated-completion) for the merge, review, and CI
evidence closing this milestone. Earlier local-stage statements remain historical.

**Identifier:** `first-product-slice-1`

**Dependencies:** The owner approved the concrete proposal and instructed
“proposal looks good, proceed.” This assigns the product slice and authorizes
local implementation after Ready and readiness reporting. The owner subsequently
authorized PR delivery and merged the reviewed result; that evidence is below.
Package publication and lifecycle transition were not assigned.

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
- [x] Incorporate the approved properties as product anchors and
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

The original six anchor identities remain in the owning chapters above;
`A-progress-observation` belongs to its separate roadmap. The current pinned
contract uses executable test criteria. The later [criterion reconciliation](executable-verification.md#executable-verification-2)
separates previously unexercised obligations: historical completion of this slice
does not establish complete coverage of those criteria. Review is delivery evidence,
not a verification method. No new proof or mutation evidence is claimed here.

### Completion criteria

#### Public programming path {#first-product-slice-1-api}

The recorded conversation example compiles and runs through the public crate API. Tests also
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

### Local implementation and review evidence

Initial local implementation was tested on port-design, based on
83ac7e6faf7b4e7dfd74c306436760e44ba46a21, before PR delivery. Its product
source-set SHA-256 was
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
running the same credential-free tests. The final-head and merge-commit runs
below subsequently supplied Linux execution evidence.

### Retained limits

The first slice did not include later one-shot tools, progress observation, or
composition conveniences. Their accepted behavior now belongs to the owning
design chapters; the six-anchor evidence here does not cover those later additions.

An old request carrying the current valid turn identifier cannot be attributed
to its original work by the envelope. Cancellation is not rollback. The bounded
HTTP body and queue do not impose a global concurrent-request limit. Process
tests observe directly owned children, not detached descendant guarantees.
Completed HTTP task records are retained for cleanup diagnostics until Agent
shutdown; no session-count memory bound is claimed. Runtime destruction may
prevent diagnostic publication; absent outcome is not cleanup confirmation.

### Documentation and API review evidence

The owner authorized draft PR delivery, CI Rustdoc generation and download, public
API documentation, a learning-focused rewrite, and `thinkwell::agent::Claude`.
The public preset namespace changed; launch/connection behavior did not. Rustdoc
is generated with warnings denied, missing public documentation is denied, and
agent-dependent examples compile without inference. CI's downloadable artifact
retains the generated assets and source pages for 14 days; this is not hosting.

The initial strict Rustdoc run caught an unclosed `Into<AgentConfig>` HTML tag;
inline code fixed it. Public-API documentation passed 23 doctests (12 compile-only,
nine runnable, two compile-fail); the learning revision passed 28 (18 compile-only,
eight runnable, two compile-fail). Comparing against e0ea812 after removing
Rustdoc/whitespace found identical executable source in the three edited files.
The learning and namespace revisions also passed 19 product tests, five bootstrap/
launcher tests, all-target compilation, strict Clippy/Rustdoc, formatting, and Book
checks. Rendered crate/type introductions and examples were inspected. An ambient
mdBook 0.5.1 mismatch was resolved by installing the pinned 0.5.4. Final CI below
covers the delivered revision; intermediate source digests remain historical.


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
checks passed before delivery. A focused implementation-author review checked
schema direction at all three use sites, preservation of validation, and request
versus response dispatch. The final CI and owner merge below include both fixes.


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

The documentation closeout was subsequently reviewed and merged in PR #3 at
`eecb8ec98424689c4deb3ce526e524421e3faef8`, satisfying the scheduler's delivery
prerequisite. Its checks below establish Book integrity, not new product behavior.
The project remains in the Porting Phase and unpublished.

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
historical PR #2 CI pass is distinct from this follow-up's final-head CI:
runs 44762298590 and 44762288442 passed before the owner reviewed and merged
PR #3 on 2026-11-20 at 15:25:01 UTC. This closes the recorded delivery prerequisite.
