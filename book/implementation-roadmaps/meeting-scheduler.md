# Meeting scheduler proposal

Roadmap identifier: `meeting-scheduler`.

Goal: teach a useful workflow that alternates typed agent requests with ordinary
Rust computation, and identify ergonomic problems through actual use.

## Milestone 1. Deliver a runnable meeting scheduler {#meeting-scheduler-1}

**Outcome:** A self-contained Rust example and newcomer-oriented walkthrough turn
natural-language availability into checked data, use Z3 to find a shared meeting
time, and ask the agent for a short explanation. The authoritative result comes
from Rust and the solver. Users can inspect the extracted availability before
trusting the answer.

**Status:** Proposed

Scope approved; closeout integration is the remaining readiness prerequisite.

**Identifier:** `meeting-scheduler-1`

**Dependencies:** The first product slice is integrated at 1b21c83. On
2026-11-20 the owner approved this scheduler scope and publication of the Book
closeout/proposal in a new draft PR. No repeat scope approval is needed. Integrate
that closeout before recording Ready and beginning implementation; scope approval
does not waive the agreed delivery prerequisite. New public API, package
publication, and an end-of-Porting-Phase transition remain outside this assignment.

**Design basis:** `R-easy-agent-programming`, `R-code-prompt-ergonomics`,
`R-per-turn-result-types`, `R-idiomatic-reimplementation`, `R-source-provenance`,
and the accepted [typed-conversation design](../design/typed-conversations.md).
Source comparisons and alternatives live in [migration research](../migration.md#example-selection).
The proposed application rules below are not new library guarantees.

**Tasks**

- [x] Obtain approval of this bounded scope and its delivery prerequisite.
- [ ] Integrate the Book closeout; record Ready, then In progress when implementation starts.
- [ ] Create an unpublished standalone example package with its own lockfile,
      walkthrough, and original small input fixtures.
- [ ] Implement typed extraction, deterministic input validation and solving,
      then a differently typed explanatory turn using the existing public API.
- [ ] Test domain rules and schema compatibility without inference; add explicit
      example checks to CI without adding a solver dependency to the library.
- [ ] Run one bounded opt-in live demonstration with existing agent credentials,
      review the code and learning flow, and record ergonomic findings and limits.
- [ ] Deliver through owner review, passing CI, and authorized integration.

### Proposed caller flow

1. Load a small JSON roster containing stable attendee IDs, display names, and
   natural-language availability. Reject an empty roster and duplicate IDs.
2. Connect with `agent::Claude::connect()` and open a session without application tools.
3. Run `Plan<ParsedAvailability>` to translate the roster into weekday/hour windows.
   Print the extraction and validate it in Rust before solving.
4. Solve for a one-hour weekday meeting in business hours with Z3. Return a Rust
   enum distinguishing a valid slot, no feasible slot, and an inconclusive solver run.
5. Run `Plan<Explanation>` in the same session to explain that result. Display
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

Propose `examples/meeting-scheduler/` as an unpublished standalone Cargo package,
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

The application rules proposed for this milestone are:

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
- **Evidence:** Pending. Require compiled example, exact run commands, reviewed
  walkthrough, and a bounded live demonstration with recorded agent/model setup.

#### Validated solver behavior {#meeting-scheduler-1-solver}

Deterministic tests cover a unique solution, multiple solutions, no intersection,
an attendee with no availability, roster mismatch, invalid hours, business-hour
boundaries, and the full one-hour duration. The domain conversion distinguishes
Sat, Unsat, and Unknown without relying on a flaky timeout to trigger Unknown.
Every successful solver answer passes an independent Rust membership check.

- **Anchor coverage:** None — application-specific rules, not a new Thinkwell guarantee.
- **Evidence:** Pending. Require credential-free tests using the real solver for
  satisfiable/unsatisfiable cases and a deterministic test of Unknown mapping.

#### Supported schemas and truthful presentation {#meeting-scheduler-1-contracts}

The actual example schemas pass the current library's support boundary, including
nested inline types. Extraction shape validation does not substitute for roster
and time-domain checks. Output separates canonical solver facts from model prose,
and failed explanations do not erase a known solver result.

- **Anchor coverage:** `A-supported-contract-boundary`; `A-typed-turn-completion`.
- **Evidence:** Pending. Require a deterministic product-API fixture or equivalent
  test reaching plan preflight with the actual example types, valid/invalid domain
  fixtures, and review of both success and error output. The planning probe below
  is feasibility evidence only, not acceptance of future example types.

#### Reproducible delivery and findings {#meeting-scheduler-1-delivery}

Example-specific formatting, compilation, Clippy, tests, and walkthrough links
pass locally and in Linux CI. Lock dependencies, document licenses and native
setup, and keep model calls opt-in. Record a separate review, owner approval,
merged revision, concrete ergonomic findings, and unresolved limitations.

- **Anchor coverage:** None — example delivery and evaluation.
- **Evidence:** Pending. The ordinary library build must remain independent of Z3,
  and neither the example nor CI may depend on the TypeScript repository.

### Planning evidence and approval boundary

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
