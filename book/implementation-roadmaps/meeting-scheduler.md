# Meeting scheduler

Roadmap identifier: `meeting-scheduler`.

Goal: teach a useful workflow that alternates typed agent requests with ordinary
Rust computation, and identify ergonomic problems through actual use.

## Milestone 1. Deliver a runnable meeting scheduler {#meeting-scheduler-1}

**Outcome:** A self-contained Rust example and newcomer-oriented walkthrough turn
natural-language availability into checked data, use Z3 to find a shared meeting
time, and ask the agent for a short explanation. The authoritative result comes
from Rust and the solver. Users can inspect the extracted availability before
trusting the answer.

**Status:** Complete

The owner reviewed and merged PR #3, then instructed implementation to continue.
The closeout prerequisite is satisfied at eecb8ec98424689c4deb3ce526e524421e3faef8.
Ready and readiness were recorded before implementation on 2026-11-20.
All three milestones are Complete through the integration record below.

**Identifier:** `meeting-scheduler-1`

**Dependencies:** The first product slice is integrated at 1b21c83. On
2026-11-20 the owner approved this scheduler scope and publication of the Book
closeout/proposal in a new draft PR. No repeat scope approval is needed. The owner merged
that closeout in PR #3 at 2026-11-20 15:25:01 UTC after both final-head CI runs
passed (44762298590 and 44762288442), and assigned implementation with
“reviewed and merged. let’s continue.” No readiness prerequisite remains. New public API, package
publication, and an end-of-Porting-Phase transition remain outside this assignment.

**Design basis:** `R-easy-agent-programming`, `R-code-prompt-ergonomics`,
`R-per-turn-result-types`, `R-idiomatic-reimplementation`, `R-source-provenance`,
and the accepted [typed-conversation design](../design/typed-conversations.md).
Source comparisons and alternatives live in [migration research](../migration.md#example-selection).
The approved application rules are not new library guarantees.

**Tasks**

- [x] Obtain approval of this bounded scope and its delivery prerequisite.
- [x] Integrate the Book closeout; record Ready, then In progress when implementation starts.
- [x] Create an unpublished standalone example package with its own lockfile,
      walkthrough, and original small input fixtures.
- [x] Implement typed extraction, deterministic input validation and solving,
      then a differently typed explanatory turn using the existing public API.
- [x] Test domain rules and schema compatibility without inference; add explicit
      example checks to CI without adding a solver dependency to the library.
- [x] Run one bounded opt-in live demonstration with existing agent credentials,
      review the code and learning flow, and record ergonomic findings and limits.
- [x] Deliver through owner review, passing CI, and authorized integration.

### Owning design and source basis

[Example applications](../design/examples.md#meeting-scheduler) owns the approved
caller flow, domain rules, solver boundary, teaching shape, and exclusions. The
selected vendored Z3 setup is documented with the actual resolved dependencies
and platform evidence below. [Migration research](../migration.md#example-selection)
retains the TypeScript source/tutorial comparison and licensing limits.

[Contracts](../design/contracts.md) owns automatic nonrecursive inlining;
[typed conversations](../design/typed-conversations.md) owns JSON context.
These were separately approved in milestones 2 and 3; they do not retroactively
change the initial annotated-schema tests.

### Completion criteria

#### Learning flow and typed turns {#meeting-scheduler-1-flow}

The example compiles against the public crate and demonstrates connection, one
session, typed extraction, deterministic computation, and a differently typed
follow-up. Its walkthrough explains each step and prerequisites to a newcomer.

- **Anchor coverage:** `A-typed-turn-completion` (usage coverage; existing runtime
  tests retain responsibility for failure and lifetime guarantees).
- **Evidence:** Local pass. The standalone package compiles against the unchanged
  product API; its README walks through actual code and shows success/no-slot
  output. A bounded live run using Claude ACP 0.23.1 and model selector haiku
  returned ParsedAvailability, solved Wednesday 11–12, then returned Explanation
  in the same session. Details and source identity are recorded below.

#### Validated solver behavior {#meeting-scheduler-1-solver}

Deterministic tests cover a unique solution, multiple solutions, no intersection,
an attendee with no availability, roster mismatch, invalid hours, business-hour
boundaries, and the full one-hour duration. The domain conversion distinguishes
Sat, Unsat, and Unknown without relying on a flaky timeout to trigger Unknown.
Every successful solver answer passes an independent Rust membership check.

- **Anchor coverage:** None — application-specific rules, not a new Thinkwell guarantee.
- **Evidence:** Local pass. Four domain tests cover roster/range errors, display,
  exact one-hour/business-hour boundaries, and 64 pairs of availability patterns
  compared with independent finite search. The real solver covers empty,
  conflicting, unique, and multiple-slot cases. Two unit tests check Unknown,
  Unsat, and model-extraction-error mapping without a forced timing race.

#### Supported schemas and truthful presentation {#meeting-scheduler-1-contracts}

The actual example schemas pass the current library's support boundary, including
nested inline types. Extraction shape validation does not substitute for roster
and time-domain checks. Output separates canonical solver facts from model prose,
and failed explanations do not erase a known solver result.

- **Anchor coverage:** `A-supported-contract-boundary`; `A-typed-turn-completion`.
- **Evidence:** Local pass. Two executable workflow tests exercise the actual
  Plan calls and types through a Python ACP/HTTP peer. An invalid nested answer
  is rejected before a corrected one succeeds; the same session returns a String
  on its second turn. The failure case verifies canonical stdout and nonzero
  exit on explanation failure. Six domain/solver tests retain the validation and
  outcome coverage. See the learning-focused revision below.

#### Reproducible delivery and findings {#meeting-scheduler-1-delivery}

Example-specific formatting, compilation, Clippy, tests, and walkthrough links
pass locally and in Linux CI. Lock dependencies, document licenses and native
setup, and keep model calls opt-in. Record a separate review, owner approval,
merged revision, concrete ergonomic findings, and unresolved limitations.

- **Anchor coverage:** None — example delivery and evaluation.
- **Evidence:** The initial implementation at 07e745b passed both Ubuntu CI jobs,
  including native Z3 and example tests, on push and PR runs. The subsequent
  learning-focused revision passed eight example tests locally and in final-head
  Linux CI. The root workspace retains only thinkwell and anchor-runner, with no
  Z3 or TypeScript dependency. Owner review and PR #4 integration are recorded below.

### Historical planning evidence and approval boundary

Reviewed the current library and the relevant upstream source/tutorial evidence
recorded in migration research. Inspected published z3 0.23.0 metadata, README,
and APIs; z3-sys 0.14.0 documents native/system, vendored, and GitHub-release setup.
No usable Z3 executable or pkg-config entry was found locally. Native solver
compilation and execution remain unverified, so this proposal does not claim
that setup is already solved on either supported platform.

An ignored scratch program under target/example-research derived nested weekday,
window, attendee, and result schemas with Schemars 1.2.3. It compiled and ran
against the unchanged product schema support check and decoder: no $ref/$defs,
a valid nested sample decoded, and the support check passed. This establishes
an example-local route through the current finite schema subset. It is not
scheduler implementation, solver testing, or a real-agent run.

No upstream implementation, prompts, tutorial prose, or data fixtures are copied.
Write original Rust code, prompts, fixtures, and explanation for this proposed
workflow, with an inspiration link. Any later substantive source adaptation needs
file-specific licensing/attribution verification under `R-source-provenance`.

The owner approved this single scheduler milestone with the stated scope,
dependency approach, application rules, and criteria on 2026-11-20. Sentiment
analysis and the unminifier remain candidates, not assigned follow-on milestones.
Implementation starts after the Book closeout is integrated; no solver or example
implementation is included in the documentation delivery.


### Implementation and validation evidence

Implemented locally on branch `meeting-scheduler` from PR #3's merge commit
`eecb8ec98424689c4deb3ce526e524421e3faef8`. The owner explicitly confirmed review
and merge before assigning continuation. Scope was recorded Ready, readiness
reported, then In progress. PR #3 final-head CI runs 44762298590 and 44762288442
passed; those runs do not verify the new example.

The example is an independent, unpublished Cargo workspace under
examples/meeting-scheduler with its own lockfile and original fixtures/tutorial.
The core library source and root Cargo manifests/lockfile are unchanged.
The separate CI job explicitly checks this package; root workspace checks alone
would not exercise it. Solver execution uses spawn_blocking so native work does
not block the Tokio task driving the agent. Canonical output is printed/flushed
before the second prompt and survives an explanation error.

The initial implementation source-set SHA-256 (before the owner-review revision below) is
`fbf20adc935be14487f2dbf67f439379e2b3534d612005e07138116779fe3c54`.
Reproduce by sorting the repository-relative example Cargo.toml, Cargo.lock,
src/*.rs, tests/*.rs, tests/*.py, and data/*.json paths, then hashing each UTF-8
path, a NUL, its file bytes, and a NUL in that order. This identifies the local
implementation, not an integrated revision.

### Native build observations

Local host: macOS arm64, Rust 1.100.0, Apple Clang 17, CMake 3.31.10 installed
under ignored target/cmake-tools. Z3's source requires CMake 3.26 or newer.
The example lockfile resolves z3 0.23.0, z3-sys 0.14.2, and z3-src 504.1.0;
the linked solver reports 5.4.1.0. The first source build plus Rust dependencies
took approximately 92 seconds locally; subsequent checks reuse it. No system Z3
or system-package changes were needed. The wrapper/bindings and bundled native
Z3 license files were inspected and are MIT; native Z3 identifies Microsoft as
copyright holder. No dependency implementation was copied into this repository.

The new CI job uses Ubuntu 24.04, two build jobs, a 30-minute job limit, and
CMake 3.31.10 installed in a temporary Python virtual environment. The job runs
formatting, all-target compilation, Clippy, and deterministic tests without model
credentials. Linux execution is still unverified: the local Docker client could
not connect to a running daemon. No Linux pass or native-build portability beyond
the tested host is claimed before that CI job runs.

### Local commands and results

From the root, with CARGO_TARGET_DIR set to target/meeting-scheduler and CMAKE
set to the local 3.31.10 executable:

- `cargo check --manifest-path examples/meeting-scheduler/Cargo.toml --all-targets --locked`: passed.
- `cargo clippy --manifest-path examples/meeting-scheduler/Cargo.toml --all-targets --locked -- -D warnings`: passed.
- `cargo test --manifest-path examples/meeting-scheduler/Cargo.toml --locked`: nine tests passed; no inference.
- `cargo fmt --manifest-path examples/meeting-scheduler/Cargo.toml -- --check`: passed.
- Root `cargo fmt --all -- --check`, workspace all-target check, Clippy with warnings
  denied, workspace tests, and strict Rustdoc: passed. The tests include 20 product
  tests, five launcher/bootstrap tests, and 28 doctests.
- `python3 -m unittest discover -s experiments/acp-integration -p 'test_probe_agent.py'`: passed.
- Root `cargo metadata --no-deps --format-version 1 --locked`: inspected, with no
  example or solver dependency added to the ordinary workspace.

The live executable was run with `--model haiku` under a 300-second outer
subprocess deadline. Its two prompts use Thinkwell's default 120-second turn
bounds; the solver has its own ten-second query timeout. The pinned Claude
adapter used existing authentication. The first answer gave Sam Wednesday 10–12,
Lee Wednesday 11–14, and Jo Wednesday 09–12. Z3 found Wednesday 11–12; the second
answer correctly explained that overlap. Exit status was zero. The model selector
is not a pinned underlying model revision, and one live pass is not exhaustive
interpretation evidence. The no-overlap case is verified by deterministic solver
tests; no additional live no-overlap run is claimed. The final changes after the
live run only reformatted identical prompt text and expressed the same one-hour
membership check in Clippy-compliant form; deterministic checks were rerun.

### Review, ergonomic findings, and remaining delivery

A separate implementation-author review inspected source, fixtures, the learning
flow, dependency isolation, and every criterion. This is self-review, not a fresh
agent or owner review. It confirmed: empty windows constrain the solver to false;
all roster IDs must be present exactly once; a full hour must fit; Unknown and
model-reading errors cannot become NoSlot; the independent membership check guards
returned slots; and the printed solver answer precedes potentially failing prose.
Review also confirmed that the fixture executable substitutes npx only in its own
child environment and does not invoke a real model during tests.

Concrete ergonomic findings:

- Different typed turns in one session worked without an API extension or manual
  cleanup. The standalone package keeps native dependency cost out of the library.
- Nested example types require inline schema annotations under the current finite
  validator. This is workable but should be explained early in a tutorial.
- Shape validation does not establish application meaning. Explicit roster/range
  checks remain natural Rust work between the two prompts.
- Native synchronous libraries need a blocking task when used with the current-thread
  async example. No new Thinkwell runtime abstraction was necessary.
- Generated prose can fail or disagree; application-owned facts should remain
  independently visible. This example adds no claim of verified model interpretation.

The owner authorized draft delivery after the local completion report, then
reviewed and merged PR #4. The completion record below supplies native Linux CI
and owner integration. No publication or lifecycle transition followed from it.

Final documentation checks passed: `cargo anchor check --config anchorbook.toml`
(structural/v1, zero errors), mdBook 0.5.4 rendering, and whitespace checks,
including previously untracked example files. All relative tutorial/source links
resolved, and 71 generated local Book links/fragments across the affected
chapters were inspected. Workflow YAML parses and contains the separate scheduler
job; parsing is not evidence that GitHub has executed it. The source digest above
was rechecked after final formatting and documentation reconciliation.


### Learning-focused owner review

The owner requested internal `crate::` imports, readable source formatting, and
learnability over command-line utility in PR #4's review. The example is now a
single binary crate with ordinary internal modules. CLI path/model options and
the separate helper library were removed. Both Plan calls are visible in main:
ParsedAvailability for extraction and String for the explanation, using an API
already supported by Thinkwell. The wrapper Explanation struct was unnecessary.
The fixed sample and agent-default model keep the demonstration focused; readers
can edit the included roster/path when experimenting.

The scheduler CI job already ran `cargo fmt --check`; rustfmt does not wrap long
string literals. Prompts are now laid out as readable multiline strings, and the
removed usage/help parser eliminates the reported long-line example. No new lint
or formatter policy was needed. Tests now exercise both success and explanatory
failure through the executable itself, with six domain/solver tests and two
executable tests. This replaces duplicated tests of extracted workflow helpers
while retaining actual product schema validation and both typed turns.

Before this revision, both **Rust and Book checks** and **Meeting scheduler example**
passed on 07e745b250c4e9cd485073713219b6d41c2ffb6e in
[PR run 44804899310](https://github.com/dherman/thinkwell-rs/actions/runs/44804899310)
and [push run 44804888464](https://github.com/dherman/thinkwell-rs/actions/runs/44804888464).
That establishes the initial native Linux build; it does not establish CI results
for the learning-focused changes. Earlier statements about unverified Linux refer
to the local implementation stage before these CI runs.

The review identified inline-schema annotations and manual JSON serialization as
API friction. The owner separately approved the inlining and context milestones
below. `Plan<String>`, blocking Z3 execution, and application validation needed
no new abstraction. The owning library design incorporates those accepted changes.

Validation of the learning-focused revision (07e745b plus the local changes):
example formatting, all-target compilation, Clippy with warnings denied, and all
eight tests passed. A fresh live run of the simplified executable, with the
agent's default model and a 300-second outer deadline, passed both turns and
returned Wednesday 11–12 plus a plain String explanation. No underlying model
revision is inferred from the default selection. Book structure/rendering and
whitespace checks passed after correcting an unquoted generic type in the prose.
The previous nine-test and haiku live records describe the initial implementation;
they are retained as historical evidence rather than overwritten.


### Schema inlining research basis

Source inspection of Schemars 1.2.3 found `SchemaSettings::inline_subschemas` and
its retention of references for recursive types. Milestone 2 exercised the setting
through product input, output, and result paths. This source observation alone did
not verify runtime behavior or deeply nested resource bounds; inlining can duplicate
schemas and increase prompt size. The accepted finite subset still rejects recursion
and manually supplied references, as defined in [contracts](../design/contracts.md).


## Milestone 2. Automatic schema inlining {#meeting-scheduler-inlining}

**Identifier:** `meeting-scheduler-inlining`
**Status:** Complete
**Approval:** The owner approved the proposed bounded implementation milestone
  with “proceed”; it became Ready and was assigned in this session.
**Dependencies:** Existing contract generation and scheduler review revision
  45323db; no dependency on scheduler merge. This is an explicitly approved
  adaptation alongside the scheduler, not implied by the original example scope.
**Design basis:** `A-supported-contract-boundary` and the nested-type behavior
  in [typed conversations](../design/typed-conversations.md).
**Outcome:** Ordinary nonrecursive nested derived types need no inline annotation.
**Tasks:** Enable generation inlining in both directions; cover nested validation,
  recursion and unsupported contracts; simplify scheduler and documentation.

### Contract behavior {#meeting-scheduler-inlining-contracts}

**Anchor coverage:** `A-supported-contract-boundary`.
**Criterion:** Tests cover nested/reused types in input/result and output contracts,
  direction-specific names, self/mutual recursion rejection, and unsupported nested
  constraints/manual references. Existing preflight and validation tests still pass.
**Evidence:** Local pass at 45323db plus this uncommitted adaptation. New schema
tests cover both directions, reused nested records, invalid nested values, recursive
types and unsupported schemas. The existing no-prompt test now uses recursion,
because ordinary nesting is intentionally accepted. All workspace tests pass.

### Example and delivery {#meeting-scheduler-inlining-delivery}

**Anchor coverage:** `A-typed-turn-completion` (application usage only).
**Criterion:** Scheduler workflow tests pass without inline annotations; API docs
  explain nesting and limits; review, CI, and owner integration are recorded.
**Evidence:** All eight scheduler tests pass without inline annotations. Workspace
Clippy and strict Rustdoc pass. A separate author self-review checked the diff,
shared generation path, unchanged validator, and scope of recursion rejection;
this is not independent or owner review. Final CI and owner integration are below.
No new live inference run was needed for this schema-only change.

Commands: `cargo test --workspace --locked`,
`cargo clippy --workspace --all-targets --locked -- -D warnings`,
`RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked`, and
`cargo test --manifest-path examples/meeting-scheduler/Cargo.toml --locked`
with the previously recorded CMAKE and CARGO_TARGET_DIR settings.


## Milestone 3. Structured prompt context {#meeting-scheduler-context}

**Identifier:** `meeting-scheduler-context`

**Status:** Complete

**Outcome:** Callers append serializable context with `Plan::json(&value)?`;
the scheduler demonstrates it instead of manually converting the roster to text.

**Approval:** The owner explicitly assigned implementation after schema inlining;
this bounded milestone became Ready and was assigned in the same session.

**Dependencies:** Existing Plan text composition. Schema inlining is pushed at
8fb2cba; neither feature depends on the other's integration.

**Design basis:** `R-code-prompt-ergonomics` and the Plan context behavior in
[typed conversations](../design/typed-conversations.md).

**Tasks:** Add an immediately fallible JSON helper for all Plan binding states,
explain it with a concrete Rustdoc example, update the scheduler, and validate.
No quoting protocol, trust boundary, new content block, or schema requirement is added.

### Composition and failures {#meeting-scheduler-context-behavior}

- **Anchor coverage:** None — prompt composition convenience, not a new protocol guarantee.
- **Criterion:** Tests verify text/JSON ordering, JSON escaping, no retained borrow,
  and immediate serialization failure. The helper requires only Serialize,
  preserves Plan state, and reports InvalidValue without invoking an agent.
- **Evidence:** Local pass on 8fb2cba plus this adaptation: three API tests cover
  composition/escaping and owned snapshots, bound-state preservation, and immediate
  serialization failure. `cargo test --workspace --locked` passes, including
  conversation preflight tests and the new runnable Rustdoc example.

### Example and documentation {#meeting-scheduler-context-delivery}

- **Anchor coverage:** `A-typed-turn-completion` (application usage only).
- **Criterion:** Scheduler tests and the illustrative Rustdoc pass; documentation
  explains fallibility and the absence of a trust boundary. Review and integration
  are tracked independently of local evidence.
- **Evidence:** All eight scheduler tests pass with `.json(&roster)?`. Workspace
  Clippy with warnings denied, strict Rustdoc, formatting, Book structural checking,
  mdBook rendering, and whitespace checks pass. Commands and native environment are
  the same as the inlining milestone above. Separate author self-review examined
  the full diff, error behavior, ownership, docs, and unchanged prompt bytes; no
  unresolved findings. This is not independent or owner review. No additional live
  agent run was performed. Final CI and owner-reviewed integration are recorded below.


## Integrated completion {#integrated-completion}

The owner reviewed and merged PR #4 on 2026-11-21 at 01:19:13 UTC, integrating
final head `0efc8e45f29d81a2223d104f25e2a3b24d4e1493` as
`558f3ba6e429a84f1e9630ef6b0fb6ee12b66485`. This completes the scheduler,
automatic schema inlining, and structured prompt context milestones. Earlier
local-stage evidence above retains its original scope; this record supplies
final review and integration evidence.

Both **Rust and Book checks** and **Meeting scheduler example** passed on the final
head in [PR CI](https://github.com/dherman/thinkwell-rs/actions/runs/44811985839)
and [push CI](https://github.com/dherman/thinkwell-rs/actions/runs/44811975060).
This includes the final native Linux example build and tests. The separate
[merge-commit run](https://github.com/dherman/thinkwell-rs/actions/runs/44849466252)
was still running when this closeout was prepared; no pass is claimed for it.

Criterion closeout:

- Learning flow and typed turns: the reviewed tutorial and executable show both
  Plan calls, typed extraction, ordinary Rust solving, and a String explanation.
- Validated solver behavior: deterministic domain/solver tests and the recorded
  live demonstration provide the bounded evidence; model interpretation remains
  fallible and no calendar booking is performed.
- Supported contracts and trustworthy presentation: executable tests cover nested
  answer validation and preservation of the canonical result after prose failure.
- Reproducible delivery and findings: native macOS evidence, final Linux CI,
  original source/provenance review, owner review, and merge are recorded.
- Inlining contracts and delivery: both schema directions, recursion/reference
  rejection, nested constraints, and the annotation-free scheduler pass tests.
- Context composition/failures and delivery: JSON ordering/escaping, owned data,
  bound Plan state, immediate serialization errors, Rustdoc and scheduler tests pass.

The owner's review and merge follow the recorded author self-reviews; no independent
agent review is claimed. The owner integrated this closeout in PR #5 at
`343f4253f9686a0ff56e5a1dc65d87aea00b2437`. No further implementation milestone,
package publication, or end of the Porting Phase is authorized by completion.


### CI build caching follow-up

The owner approved caching the vendored scheduler build after observing slow CI
in PR #6. The workflow now uses Swatinem/rust-cache pinned to
`d1527c8f67ef9af357290475d4af4b0fab8f12e0`, selecting the standalone scheduler
workspace and its root-level target/meeting-scheduler directory. The action's
Rust/dependency cache keys are supplemented with Ubuntu 24.04 and a native-tool
version fingerprint (C, C++, CMake, linker, libc). Cargo binary caching is disabled.
All existing compilation, Clippy and test commands still run; no solver version,
source-build policy, or product behavior changes. Cache misses rebuild normally.

This is an approved delivery optimization, not a new product milestone. Local
workflow inspection, Book structure/rendering, bootstrap consistency and whitespace
checks validated configuration only. The subsequent cache-reuse measurements below
supply execution evidence; eviction and branch visibility mean warm-cache performance
is not guaranteed.

Cache reuse was verified on 82bfebba05af175d8603bd3b002a3f4e4b7e4760 in
[run 44876447906, attempt 2](https://github.com/dherman/thinkwell-rs/actions/runs/44876447906/attempts/2).
The original push job took 25m59s overall and reported 24m36s Cargo compilation,
saving 1,766,027,313 bytes. The scheduler-only rerun on the same commit restored
an exact matching cache in approximately 48 seconds. Cargo compilation took
2.08 seconds, Clippy 0.28 seconds, and test compilation 4.81 seconds; all eight
scheduler tests passed. Total job time was 1m14s, about 21 times faster overall.
The warm log contains no Z3 compilation, confirming native artifact reuse for
this run. These results establish one warm-cache success, not guaranteed timing
under toolchain changes, cache eviction, or different runners.
