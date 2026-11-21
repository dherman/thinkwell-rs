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
Ready was recorded and readiness reported on 2026-11-20; implementation is now In progress.

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
The proposed application rules below are not new library guarantees.

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
- [ ] Deliver through owner review, passing CI, and authorized integration.

### Migration references

Use these references for the original example's intent and teaching flow during
implementation:

- [TypeScript scheduler source at the inspected revision](https://github.com/dherman/thinkwell/blob/2393101fd66b8471f33bb3a4908aeee919fdf0ce/examples/src/schedule.ts).
- [Published Meeting Scheduler tutorial](https://thinkwell.sh/examples/meeting-scheduler).
- [Tutorial source at the inspected revision](https://github.com/dherman/thinkwell/blob/2393101fd66b8471f33bb3a4908aeee919fdf0ce/website/examples/meeting-scheduler.mdx).

The pinned source links preserve the research snapshot even if the published
tutorial changes. These are temporary migration references, not permanent
Thinkwell authority or a requirement for behavioral parity. The approved Rust
scope and application rules in this roadmap govern where behavior differs.
Follow the source-provenance requirements below before adapting substantive
material; remove these migration references at the end of the Porting Phase.

### Caller flow

1. Load a small JSON roster containing stable attendee IDs, display names, and
   natural-language availability. Reject an empty roster and duplicate IDs.
2. Connect with `agent::Claude::connect()` and open a session without application tools.
3. Run `Plan<ParsedAvailability>` to translate the roster into weekday/hour windows.
   Print the extraction and validate it in Rust before solving.
4. Solve for a one-hour weekday meeting in business hours with Z3. Return a Rust
   enum distinguishing a valid slot, no feasible slot, and an inconclusive solver run.
5. Run `Plan<String>` in the same session to explain that result. Display
   Rust's canonical status/time separately from the generated prose. Let ordinary
   ownership clean up the session and agent.

The tutorial should show the actual compiled code in small steps: first a typed
answer, then ordinary validation and solving, then a follow-up of a different
type. Explain what each derive contributes and show sample success and no-slot
output. Application helper functions should be visibly separate from Thinkwell
API. Explain that Z3 checks the extracted constraints, not whether the model
understood someone's natural-language intent correctly.

This example deliberately needs no application tools: Rust calls the solver
between prompts. A later sentiment-analysis example can exercise wrapping a Rust
library in a borrowed tool. Adding a tool to this example merely to demonstrate
one would obscure its central lesson.

### Scope and dependency choice

Use `examples/meeting-scheduler/` as an unpublished standalone Cargo package,
with a path dependency on Thinkwell and its own lockfile. Keep it outside ordinary
workspace builds so solver installation and compilation are paid for only by
users of this example. CI must explicitly build/test this package; an exclusion
must not silently remove its checks.

Use the MIT-licensed `z3` crate (0.23.0 inspected), which wraps the Z3 solver.
Prefer its `vendored` feature for a locked source build of the native solver;
record the resolved z3-sys/z3-src versions and native compiler/CMake prerequisites
when implementing. Verify that build on macOS and Linux before accepting this
choice. Do not change system packages during planning. If native build cost or
compatibility is unacceptable, bring that finding back before changing the
example to another solving method. A system installation or pinned GitHub binary
release is an alternative, not a verified setup path in this proposal.

Use the existing Tokio, Serde, and Schemars approach. Nested records and the weekday
enum can opt into `#[schemars(inline)]`; use `i64` hours and explicit Rust range
checks. Do not broaden the core schema validator just to run the example. Do not
add unsupported numeric schema keywords and imply that they are enforced.

The approved application rules for this milestone are:

- One shared timezone, one notional Monday–Friday week, whole-hour starts, and
  one-hour meetings wholly inside 09:00–17:00 and each attendee's chosen window.
- Every input attendee must appear exactly once in the extracted result; unknown,
  duplicate, or missing IDs are errors. The original roster owns display names.
- Validate `0 <= start < end <= 24`; intersect with business hours when solving.
  An attendee with no available windows makes the schedule unsatisfiable.
- Bound the solver query to ten seconds. `Unknown` (including timeout) is an
  inconclusive result, never a claim that no slot exists. Model-extraction errors
  are errors. Recheck any returned slot against the validated windows in Rust.
- Any valid slot is acceptable; the example does not promise the earliest one.
  The explanation cannot replace the solver's status or canonical slot.

Exclude calendars, booking, dates/DST, multiple timezones, fractional-hour or
variable-duration meetings, concurrent agents, streaming UI, new providers, and
new Plan conveniences. API friction becomes evidence for a subsequent proposal,
not automatic permission to expand this milestone.

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
  learning-focused revision passes eight example tests locally; its CI remains
  pending until pushed. The root workspace still has only thinkwell and
  anchor-runner and no Z3 dependency. No source or CI dependency on TypeScript
  was added. Final revision review and authorized integration remain outstanding;
  this criterion is not complete.

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

The owner explicitly authorized committing, pushing, and opening a draft PR for
this implementation after reviewing the local completion report. The milestone
remains In progress pending native Linux/CI evidence, owner review, and authorized
integration. This delivery authorization does not permit merging, package
publication, or ending the Porting Phase. No new public API or follow-on example
is selected by these findings.

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

Potential library adaptations for discussion, not implementation in this change:

- Ordinary nested answer types currently need `#[schemars(inline)]` annotations.
  Automatically inlining nonrecursive derived schemas could remove this incidental
  burden without promising arbitrary JSON Schema reference support. Recursive
  types and unsupported constraints would still need explicit, tested behavior.
- Supplying structured context currently requires
  `.text(serde_json::to_string(&roster)?)`. A JSON/context convenience could improve
  discoverability and keep serialization errors composable. Its naming, fallibility,
  and relationship to quoting/delimiting should be discussed first; formatting is
  not an instruction-trust or prompt-injection security boundary.
- The second result needs no new API: `Plan<String>` already provides the simpler
  spelling. Synchronous Z3 work and application roster/range validation remain
  ordinary Rust responsibilities, not reasons to introduce Thinkwell abstractions.

No core API changes are made or authorized by these observations. The owner asked
for discussion of adaptations after seeing the ergonomic evidence.


Validation of the learning-focused revision (07e745b plus the local changes):
example formatting, all-target compilation, Clippy with warnings denied, and all
eight tests passed. A fresh live run of the simplified executable, with the
agent's default model and a 300-second outer deadline, passed both turns and
returned Wednesday 11–12 plus a plain String explanation. No underlying model
revision is inferred from the default selection. Book structure/rendering and
whitespace checks passed after correcting an unquoted generic type in the prose.
The previous nine-test and haiku live records describe the initial implementation;
they are retained as historical evidence rather than overwritten.


### Automatic schema inlining exploration

After the owner agreed to explore schema inlining, source inspection found that
pinned Schemars 1.2.3 already exposes `SchemaSettings::inline_subschemas`. Its
implementation retains references when an ordinary derived type recurs during
generation. Thinkwell currently uses default settings for both serialization and
deserialization contracts, then rejects unsupported schema keywords.

The proposed adaptation is to enable this generation setting in both directions,
while retaining the existing supported-contract check. Ordinary nonrecursive
nested derived types would no longer require `#[schemars(inline)]`; recursive
schemas and manually supplied references would still fail preflight. This is a
proposal, not an implemented or validated behavior change. It requires no new
public builder method or Thinkwell-specific derive. The scheduler would lose the
three inline annotations on Day, TimeWindow, and AttendeeAvailability; its Plan
calls and application validation would remain the same.

Before implementation, assign a Ready milestone covering nested result types,
tool inputs and outputs, direction-specific Serde names, repeated nested types,
self/mutual recursion rejection, and continued rejection of unsupported nested
constraints and manually supplied references. Verify these through actual product
contract paths, then remove the scheduler annotations and run its workflow tests.
Inlining duplicates reused schemas and can increase prompt size; this proposal
adds no reference resolver or recursive-contract support. Source inspection alone
does not establish runtime behavior or resource bounds for deeply nested types.


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
this is not independent or owner review. CI and owner integration remain pending.
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
  agent run was performed. CI and owner-reviewed integration remain outstanding.


## Integrated completion {#integrated-completion}

The owner reviewed and merged PR #4 on 2026-11-21 at 01:19:13 UTC, integrating
final head `0efc8e45f29d81a2223d104f25e2a3b24d4e1493` as
`558f3ba6e429a84f1e9630ef6b0fb6ee12b66485`. This completes the scheduler,
automatic schema inlining, and structured prompt context milestones. Earlier
pending statements above describe evidence at their implementation stages;
this record supplies their final review and integration evidence.

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
agent review is claimed. This closeout is a follow-up under the same delivery
assignment and awaits its own integration. No further implementation milestone,
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
checks validate configuration only. A successful cache save and subsequent restore
with measured native-build reuse remain CI evidence gaps. The first run may still
perform the full native build. GitHub cache eviction and branch visibility mean
warm-cache performance is not guaranteed.
