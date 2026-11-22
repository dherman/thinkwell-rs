# JavaScript unminifier

Roadmap identifier: `unminifier`.

## Current scope and owner direction

The owner approved a standalone example after local AST and live-agent experiments.
The owner then corrected its emphasis: this is a readable Thinkwell demo, not a
production refactoring tool. Happy-path code with assertions/panics is preferred
over comprehensive input rejection, resource budgets or equivalence checking.
This revises the example's scope, not Thinkwell's library guarantees.

The example reads a JavaScript module, discovers private top-level function
declarations, asks for typed naming suggestions in sequential batches, and applies
renames. Existing Plan clone/json/run APIs suffice; no library changes are needed.
The implementation and beginner walkthrough live at `examples/javascript-unminifier/`.

Oxc 0.159.0 supplies parser and semantic information. Instead of printing a new AST,
the helper edits declaration and resolved-reference byte ranges in reverse order.
Untouched source text, including comments and whitespace, stays unchanged; shorthand
object properties expand to retain their original keys. Existing names are reserved
and collisions receive numeric suffixes. Named exports and unrelated shadowed
bindings are left alone. This is not a general JavaScript refactoring guarantee.

The owner suggested tree-sitter for permissive parsing and source fidelity. Targeted
edits achieve source fidelity with the scope information already available from Oxc,
avoiding a second parser or handwritten resolver. Oxc remains a strict parser;
tree-sitter has not been selected, built or compared experimentally. Its permissive
parsing remains a possible future option, not a prerequisite for the teaching example.

The demo assumes ordinary lexical JavaScript and sensible identifier suggestions.
It does not try to detect every problematic export, eval, reflection or unusual
assignment form. It asserts basic parser and result expectations rather than building
custom errors. Renaming can change observable function names and stack traces.
Output uses a new file to avoid replacing input; failed writes may leave a partial
new file. No atomic publication, size limits, equivalence proof or retry policy is
promised. UMD conversion and parallel batches remain outside the example.

## Milestone 1. Assess a small Rust unminifier example {#unminifier-1}

**Identifier:** `unminifier-1`

**Status:** In progress

**Outcome:** Assess AST libraries and the typed naming workflow; present a concrete
example direction for owner selection.

**Assignment:** Following owner-merged PR #10, the owner approved investigation,
then local testing and extension to scripted/live agent workflows. The owner
subsequently approved the standalone example and the simplicity-first direction
above. Local research and review are satisfied; durable integration remains pending.

**Design basis:** `R-code-prompt-ergonomics`, `R-easy-agent-programming`,
`R-progressive-control`, `R-source-provenance`, `D-reviewed-api-baseline`, and
[typed conversations](../design/typed-conversations.md).

**Tasks:** Source/library assessment, local experiment, concrete workflow and owner
scope review are done; deliver the records with milestone 2.

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
  The current scope above incorporates that correction; integration remains pending.

## Milestone 2. Deliver the standalone unminifier example {#unminifier-2}

**Identifier:** `unminifier-2`

**Status:** In progress

**Outcome:** Teach code/prompt alternation through a small standalone example with
source-preserving renames and a beginner walkthrough.

**Approval and dependencies:** The owner instructed “continue” after the successful
experiment and recommendation to make a standalone example. The milestone became
Ready and was assigned In progress. The subsequent simplicity instruction replaces
the earlier production-style rejection, output-publication and equivalence-checking
scope. The owner subsequently requested a draft PR, authorizing commit/push and draft
delivery. No library API change or lifecycle transition is assigned.

**Design basis:** The same requirements and accepted typed-conversation baseline as
milestone 1. The example is application code, not a new supported library contract.

**Tasks:** Standalone package, prompt loop, source edits, simple CLI, walkthrough,
deterministic tests and CI configuration are implemented locally. Final delivery,
CI and owner integration remain outstanding.

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
  job is configured but has not run for this unpublished change. Commit/push and
  draft PR delivery are authorized; owner review and integration remain pending.

## Local evidence and limitations

Current tests run on macOS/Rust 1.100.0, base `ab1e821` plus the uncommitted
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
owner review; draft PR delivery is now authorized; CI and integration remain pending.

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
handling. CI for the correction and owner integration remain pending.


## Presentation follow-up

The owner requested two commits on PR #11: improve the teaching presentation first,
then add Plan::code and use it in the example. The first change separates JavaScript
mechanics into javascript.rs, exposes discovery → suggest_names → rename in main,
and passes the discovered candidates into the prompt workflow. The walkthrough now
explains typed answers, plan cloning and fresh history, with a two-function fixture
and a complete representative before/after result. Library APIs are unchanged in
this presentation commit. Existing deterministic checks cover the reorganized code.
