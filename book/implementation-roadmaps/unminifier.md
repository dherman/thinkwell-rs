# JavaScript unminifier

Roadmap identifier: `unminifier`.

The [example design](../design/examples.md#javascript-unminifier) owns the accepted
teaching scope and transformation rules; [typed conversations](../design/typed-conversations.md)
owns Plan::code. The owner selected a simple demo with happy-path assertions,
replacing the earlier production-style rejection/equivalence scope. Milestones 1–3
are Complete through owner-merged PR #11; see [delivery evidence](#integrated-completion).

## Milestone 1. Assess a small Rust unminifier example {#unminifier-1}

**Identifier:** `unminifier-1`

**Status:** Complete

**Outcome:** Assess AST libraries and the typed naming workflow; present a concrete
example direction for owner selection.

**Assignment:** Following owner-merged PR #10, the owner approved investigation,
then local testing and extension to scripted/live agent workflows. The owner
subsequently approved the standalone example and the simplicity-first direction
in the linked design. Research, scope review, and delivery are complete through PR #11.

**Design basis:** `R-code-prompt-ergonomics`, `R-easy-agent-programming`,
`R-progressive-control`, `R-source-provenance`, `D-reviewed-api-baseline`, and
[typed conversations](../design/typed-conversations.md).

**Tasks:** Source/library assessment, local experiment, concrete workflow and owner
scope review and delivery with milestone 2 are complete.

### Completion criteria

#### Source and library assessment {#unminifier-1-research}

- **Anchor coverage:** None — preparation, not a product guarantee.
- **Criterion:** Record provenance, candidate mechanisms and untested assumptions.
- **Evidence:** [Migration research](../migration.md#unminifier-refresh) records pinned
  TypeScript source/tutorial links and Oxc/SWC source inspection. Oxc was compiled
  and exercised locally; SWC and tree-sitter were not. No upstream source was adapted.

#### Concrete workflow and boundaries {#unminifier-1-workflow}

- **Anchor coverage:** None — example design preparation.
- **Criterion:** Demonstrate ordinary Rust around typed prompts and explain binding
  identity, name allocation and sequential execution.
- **Evidence:** Local scripted and live runs exercised the Plan clone/json/run loop.
  Current code and walkthrough follow the owner's simpler teaching direction.
  One active session per connection is sufficient for sequential batches; no new
  concurrency guarantees or code-block API were needed.

#### Owner review {#unminifier-1-review}

- **Anchor coverage:** None — planning delivery.
- **Criterion:** Present recommendations and obtain scope selection.
- **Evidence:** The owner approved the standalone example after local testing,
  then explicitly preferred simplicity and permitted panics outside the happy path.
  The owning design incorporates that correction; PR #11 integrated it.

## Milestone 2. Deliver the standalone unminifier example {#unminifier-2}

**Identifier:** `unminifier-2`

**Status:** Complete

**Outcome:** Teach code/prompt alternation through a small standalone example with
source-preserving renames and a beginner walkthrough.

**Approval and dependencies:** The owner instructed “continue” after the successful
experiment and recommendation to make a standalone example. The milestone became
Ready and was assigned In progress. The subsequent simplicity instruction replaces
the earlier production-style rejection, output-publication and equivalence-checking
scope. The owner subsequently requested a draft PR, authorizing commit/push and draft
delivery. No execution API or lifecycle transition is assigned; the later Plan::code
formatting convenience is separately assigned in milestone 3.

**Design basis:** The same requirements and accepted typed-conversation baseline as
milestone 1. The example is application code, not a new supported library contract.

**Tasks:** Standalone package, prompt loop, source edits, simple CLI, walkthrough,
deterministic tests, CI, and owner-reviewed integration are complete. The evidence
below distinguishes initial local checks from the final delivered revision.

### Completion criteria

#### Transformation and typed workflow {#unminifier-2-workflow}

- **Anchor coverage:** None — example-level behavior using the existing API.
- **Criterion:** Focused tests demonstrate discovery, source-preserving edits,
  shadowing/collisions, shorthand properties and sequential typed batches.
- **Evidence:** Four unit tests and two scripted-workflow tests pass. Tests compare
  exact edited output, including inline comments, Unicode text and escaped source
  identifiers. Responses arrive reversed; names are applied in candidate order.
  Agent failure propagates normally. No claim to catch all JavaScript hazards.

#### Simple CLI and teaching {#unminifier-2-output}

- **Anchor coverage:** None — application CLI and walkthrough.
- **Criterion:** Keep the code/prompt workflow visible; read input, produce a separate
  new file, and document teaching assumptions without production guarantees.
- **Evidence:** The main uses ordinary file I/O, Claude::connect and the async
  workflow. Three CLI tests demonstrate the happy path, existing-file protection,
  no connection for an empty candidate set and basic failure behavior. The rewritten
  walkthrough explains typed suggestions, plan reuse and byte-range edits. Assertions
  replace elaborate validation; no-clobber creation is retained as a small convenience,
  not transactional or durable output handling.

#### Delivery {#unminifier-2-delivery}

- **Anchor coverage:** None — review and integration.
- **Criterion:** Record local checks, author review, CI and owner integration.
- **Evidence:** Local validation and author review are below. The separate Linux CI
  job passed on the final PR head and merge commit. Owner merge and exact
  revisions are recorded in the completion record below.

## Local evidence and limitations

Initial local tests ran on macOS/Rust 1.100.0, base `ab1e821` plus the uncommitted
unminifier-design tree. With `CARGO_TARGET_DIR="$PWD/target/unminifier"` and manifest
`examples/javascript-unminifier/Cargo.toml`, locked cargo tests pass (four unit,
two scripted-workflow, three CLI). All-target check, strict Clippy and formatting
are the local validation commands documented in CONTRIBUTING.md. Book structure,
rendering, bootstrap consistency and whitespace checks cover documentation/CI edits.
Results are local, not Linux CI. Dependencies remain in the example's own lockfile.
Oxc's codegen feature was removed; tempfile is now test-only. No Thinkwell library
runtime dependency or implementation was changed.

Earlier experiments passed both deterministic and live Claude runs, returning `sum`
for the original average fixture; Node checked three expected averages. Those runs
used the former full-file printer and defensive workflow. They establish historical
integration evidence, not validation of the revised source-edit path. The previous
printer lost an inline comment; the current exact-output test requires that comment
and all surrounding formatting to remain. Tests for removed production policies were
removed as part of the owner-directed scope change, not to claim those policies pass.

The implementation and fixture are original project material. The Python scripted
peer follows this repository's existing test pattern. License/provenance research
for upstream inspiration remains migration-only; no TypeScript code, tutorial prose,
prompt or third-party JavaScript fixture was copied.

The simplified source-edit version subsequently passed a live Claude run with the
same original fixture and an outer 150-second process-group guard. It suggested
`sum`. Exact comparison confirmed that only the declaration/call names changed:
the inline comment and original whitespace survived. Node v24.12.0 checked the three
expected averages, all passing. The demo itself has no added outer timeout API;
the guard was local verification infrastructure. This single configuration is not
agent-breadth or general JavaScript-correctness evidence.

A separate author self-review of the simplified code and docs checked the owner scope
correction, span-edit order, shorthand handling, named-export selection, retained
file protection and removed guarantees. No new library policy or API was introduced.
All nine focused tests, all-target check, strict Clippy, formatting, Book structure,
rendering and whitespace checks pass. This is author review, not independent or
owner review. Subsequent owner delivery and CI are recorded below.

## PR review correction

Copilot's review of PR #11 at `e777f72` found that expanding `__proto__` shorthand
into a literal property changed a data property into JavaScript's prototype setter.
The correction uses a computed `["__proto__"]` key for that decoded binding name,
including escaped source spellings. This is a small fix to the demonstrated shorthand
behavior, not a broader production-validation policy.

The new exact-output regression failed before the fix and passes afterward for
plain and escaped spellings. Node additionally confirmed that the original shorthand
and computed-key output retain an own enumerable property and Object.prototype,
while the former expansion changes the prototype. On `e777f72` plus this correction,
all ten example tests (five unit, two workflow, three CLI), strict all-target Clippy,
formatting, Book structure/rendering and whitespace checks pass. Author review checked
the special case against decoded symbol identity and unchanged ordinary shorthand
handling. The correction is included in the final passing CI and owner merge below.


## Presentation follow-up

The owner requested two commits on PR #11: improve the teaching presentation first,
then add Plan::code and use it in the example. The first change separates JavaScript
mechanics into javascript.rs, exposes discovery → suggest_names → rename in main,
and passes the discovered candidates into the prompt workflow. The walkthrough now
explains typed answers, plan cloning and fresh history, with a two-function fixture
and a complete representative before/after result. Library APIs are unchanged in
this presentation commit. Existing deterministic checks cover the reorganized code.

## Milestone 3. Add code prompt context {#unminifier-3}

**Identifier:** `unminifier-3`

**Status:** Complete

**Outcome:** Plan::code(source, language) adds readable code context and simplifies
this example's reusable prompt.

**Assignment and basis:** The owner explicitly assigned this follow-up commit on
PR #11 after presentation improvements. It became Ready before implementation.
Basis: `R-code-prompt-ergonomics`, `R-progressive-control`, `D-reviewed-api-baseline`
and the typed-conversation design. Commit/push to the same PR is authorized.

**Tasks:** Add a documented convenience, delimiter/ownership/composition tests,
update the example, run workspace and example checks, and record review/evidence.

### Completion criteria

#### Code context and teaching {#unminifier-3-context}

- **Anchor coverage:** None — formatting convenience over existing text composition.
- **Criterion:** Preserve source text in a fenced block, handle embedded fences,
  reject invalid language labels immediately, retain Plan state and snapshot semantics;
  document source-first ordering and use in the example.
- **Evidence:** Unit and scripted-agent tests pass locally. A Result return parallels json's
  immediate context-construction errors. The fence uses at least three backticks
  and exceeds every backtick run in source. Labels containing backticks or control
  characters are InvalidValue; an empty label is allowed. This is formatting, not
  syntax validation or an instruction-trust boundary.

#### Validation and delivery {#unminifier-3-delivery}

- **Anchor coverage:** None — API documentation and delivery.
- **Criterion:** Unit/Rustdoc/workspace/example checks and author review pass;
  CI and owner integration recorded before completion.
- **Evidence:** Workspace, Rustdoc and example checks pass locally; see follow-up
  validation below. Final CI and owner integration are recorded below.


### Code-context follow-up validation

The presentation commit is `f15f9bc`. The subsequent working tree adds Plan::code,
Rustdocs with a compiled agent example, and unit coverage for embedded backtick runs,
empty input/labels, LF/CRLF, invalid labels, Unicode source, immediate snapshots,
clone composition, and preservation of bound callbacks/settings/existing errors.
The example uses code on the base plan and JSON only for each candidate batch;
the scripted peer confirms both source and candidates reach each prompt.

Passed locally: `cargo test --workspace --locked` (23 library unit tests, 32
conversation tests, 40 Rustdoc cases and launcher/bootstrap tests), workspace
all-target check and strict Clippy, warning-free workspace Rustdocs, and all ten
unminifier tests plus its strict all-target Clippy. Formatting, Book structure/rendering
and whitespace checks also pass. Unchanged scheduler/sentiment suites are left to
existing CI jobs. No runtime, transport or dependency changes were needed.

A separate author review checked source-first argument order, delimiter sizing,
label rejection, snapshot/clone semantics, state preservation and the teaching flow.
The Result return makes invalid labels immediate, consistent with json. No new
instruction-trust or JavaScript-validity claim is introduced. This is author review,
not independent/owner review. The owner's final merge and successful CI are
recorded separately below.


A bounded live Claude run of the two-function fixture with code context suggested
`sum` and `count`. Exact comparison verified that only the selected names changed,
including preserved inline comments/formatting. Node v24.12.0 checked three expected
averages, all passing. The run used a 150-second outer process-group guard and existing
authentication; it is not part of ordinary CI and establishes only this configuration.

## Integrated completion {#integrated-completion}

GitHub evidence inspected on 2026-11-24 confirms that owner `dherman` merged
[PR #11](https://github.com/dherman/thinkwell-rs/pull/11) on 2026-11-23 at
01:54:43 UTC. Final head `725f50df1f7c9db30999b91b72e14df0a78baac3` was integrated
as `5644d6fc0924b0a13aa979980582aefd9b5bf4a7`. The owner-directed teaching
simplification, presentation revision, and Plan::code follow-up are included.
Copilot's final comment recommended approval with no active findings; it is
recorded as automated review, not a human approving review. The owner selected
and revised the scope and merged the final result.

All four jobs (Rust and Book, scheduler, sentiment, and unminifier) passed on the
final head in [PR CI](https://github.com/dherman/thinkwell-rs/actions/runs/45068758821)
and [push CI](https://github.com/dherman/thinkwell-rs/actions/runs/45068754266).
[Merge-commit CI](https://github.com/dherman/thinkwell-rs/actions/runs/45097193947)
also passed. The final unminifier suite has ten tests: five unit, two scripted
workflow, and three CLI. Earlier nine-test and live-printer results remain
historical evidence, not substitutes for the corrected source-edit tests.

All three bounded milestones are Complete. General JavaScript equivalence,
permissive parsing, provider breadth, and atomic output remain outside scope;
no new live run is claimed by this status reconciliation. This Book correction
remains local work under [lifecycle-delivery-3](lifecycle-delivery.md#lifecycle-delivery-3).
