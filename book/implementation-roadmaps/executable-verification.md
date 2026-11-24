# Executable anchor verification

Roadmap identifier: `executable-verification`.

Goal: connect the library's existing assertions to the pinned Anchor Book runner,
while retaining historical evidence and exposing coverage limits.

The first milestone records the initial wiring and its original broad criteria.
Its seven satisfied results apply to that earlier criterion revision. Milestone 2
makes the previously prose-only gaps part of current computed satisfaction.

## Milestone 1. Connect existing checks {#executable-verification-1}

**Outcome:** Each library criterion has assertion-based test links and a current
computed result; local and CI validation execute verification before rendering.

**Status:** In progress

**Identifier:** `executable-verification-1`

**Dependencies:** The owner explicitly assigned this work after the local library
design reconciliation on `port-phase-adoption` at `4f13784`. That instruction made
this milestone Ready; it is assigned and In progress. The design draft and its
evidence are preserved separately. No commit, push, publication, or phase transition
is assigned.

**Design basis:** The seven accepted library anchors and their `behavior [test]`
criteria, `R-explicit-authority`, `D-versioned-project-knowledge`, and the pinned
contract's programmatic-verification rules. [Project guidance](../working-on-the-project.md)
owns the runner setup and its limits. This work changes neither properties nor criteria.

**Tasks**

- [x] Map existing assertions, including unsupported compiler and platform evidence.
- [x] Configure nextest and add criterion links to the tests that exercise them.
- [x] Run verification, investigate failures, and record computed states and gaps.
- [x] Integrate local/CI checks and validate tests, bootstrap, and the Book.

### Completion criteria

#### Assertion mapping {#executable-verification-1-mapping}

Each anchor criterion maps to actual assertions, with uncovered obligations and
checks outside the runner stated explicitly. Existing evidence remains intact.

- **Anchor coverage:** None — assertion mapping and coverage assessment.
- **Evidence:** The mapping below inspects the Rust assertions and Python peer
  branches, not just names or earlier passes. The four changed library files add
  links to 38 existing tests without changing their assertions or product bodies.
  Anchor definitions and all previous roadmap evidence are preserved.

#### Executable results {#executable-verification-1-results}

The configured nextest runner executes linked checks, and every criterion's actual
computed state is reported separately from the command's exit status and coverage.

- **Anchor coverage:** `A-scoped-turn-ownership` · `behavior` — Partial: linked runtime checks; rustdoc compiler probes remain outside the runner.
- **Anchor coverage:** `A-typed-turn-completion` · `behavior` — Partial: existing result and terminal scenarios listed below.
- **Anchor coverage:** `A-turn-envelope-routing` · `behavior` — Partial: existing stale/malformed/current identifiers and revoked-queue cases on the current platform.
- **Anchor coverage:** `A-cancelled-session-invalidation` · `behavior` — Partial: existing unpolled/started-drop and closed-reuse cases on the current platform.
- **Anchor coverage:** `A-owned-resource-cleanup` · `behavior` — Partial: current-platform process checks with the incomplete runtime matrix below.
- **Anchor coverage:** `A-supported-contract-boundary` · `behavior` — Partial: existing schema and wire-value assertions, with missing cases below.
- **Anchor coverage:** `A-progress-observation` · `behavior` — Partial: existing queue and observer assertions; compiler probe and pressure scenarios remain outside this evidence.
- **Evidence:** `cargo anchor verify --config anchorbook.toml --method test`
  on Linux, Rust 1.100.0 and nextest 0.9.151 returned exit 0 and computed all seven
  criteria satisfied. All 38 distinct linked tests passed. The engine reports
  passing linked checks, not completeness of the mapping; remaining obligations
  below still apply. A subsequent run with `--format json` confirms all seven
  states with no findings; `complete: true` describes completed evaluation, not
  complete coverage. No Complete-milestone Full claims were present in the report.
  The nextest filter excluded 17 other library tests, none of which supplies
  anchor evidence in this run. Workspace tests separately execute all 55.

  | Anchor criterion | Computed state | Linked tests passed |
  | --- | --- | --- |
  | `A-scoped-turn-ownership` · `behavior` | satisfied | 9 |
  | `A-typed-turn-completion` · `behavior` | satisfied | 8 |
  | `A-turn-envelope-routing` · `behavior` | satisfied | 3 |
  | `A-cancelled-session-invalidation` · `behavior` | satisfied | 9 |
  | `A-owned-resource-cleanup` · `behavior` | satisfied | 12 |
  | `A-supported-contract-boundary` · `behavior` | satisfied | 12 |
  | `A-progress-observation` · `behavior` | satisfied | 11 |

  Tests can link multiple criteria; these counts total 64 associations, not 64
  distinct executions. The engine's input identities are criteria
  `430a13eaa269155e`, configuration `a9835f786b88232f`, and code `c5aec1dd9dd03591`,
  at `4f13784f3b07cfee26f745901cafe1b0ca3b2cf6` with uncommitted changes.
  These are engine content identities, not cryptographic hashes. Raw text/JSON
  reports are temporary local artifacts; the runner retains its current JUnit
  report and package log under `target/anchorbook-verify/nextest/`.

#### Project validation {#executable-verification-1-validation}

Installation and local checks are documented; existing CI runs verification before
the Book build. Affected tests, bootstrap, structural and rendering checks pass,
or their failures and limitations are explicitly reported.

- **Anchor coverage:** None — verification integration and Book validation.
- **Evidence:** Local validation on the same uncommitted Linux tree passed:
  `cargo test --workspace --locked` (23 library unit tests, 32 subprocess
  integration tests, three launcher tests, two bootstrap tests, 36 ordinary
  doctests and four compile-fail probes). Compile-only doctests establish no
  runtime behavior. The explicit
  `cargo test -p anchor-runner --test bootstrap_consistency --locked` also passed;
  its regression checks now reject missing verification commands and verification
  placed after rendering. Formatting, all-target compilation, Clippy with warnings
  denied, strict Rustdoc, and the Python capability-probe regression passed using
  the contributor commands. No product logic changed, so no mutation run was added.

  `cargo anchor check --config anchorbook.toml` passed structural/v1 with zero
  errors; `mdbook build` passed with mdBook 0.5.4. Rendered inspection checked all
  33 navigated chapters and 685 local links/fragments, including the new roadmap
  tables. CI YAML and the verification-before-render order were validated.
  `git diff --check` and whitespace checks including the new chapter passed.
  Snapshot comparisons confirm unchanged design chapters, anchor definitions,
  authority records, prior roadmap evidence, product bodies and test assertions.
  A separate diff against the saved design draft contains only this assignment.
  GitHub CI has not run for these local changes.

  Initial sandbox tool installation failed DNS resolution; installation succeeded
  with network access. The existing subprocess tests require loopback sockets and
  ran with that access. Neither condition is represented as a failing product test.

### Completion record

Local implementation and validation are prepared for review, separate from the
design-reconciliation diff. The milestone remains In progress pending the project's
review and integration process. Current computed results and outstanding coverage
are recorded above; no criteria were weakened and no review judgment contributed
to verification. CI execution, publication and the Port transition are not claimed.

### Assertion coverage {#assertion-coverage}

At the initial wiring revision, all anchors had one `behavior` criterion. Names below identify
the executable checks; the assertion descriptions explain their contribution.
`conversations::` means `crates/thinkwell/tests/conversations.rs`; `schema::tests`,
`runtime::tests`, and `progress::tests` are the modules in the corresponding files
under `crates/thinkwell/src`. The source doc-comments are the canonical executable
associations, and the engine's JSON report enumerates all selected identities.

| Criterion | Existing executable assertions | Remaining coverage obligations |
| --- | --- | --- |
| `A-scoped-turn-ownership` · `behavior` | `typed_turns_and_stale_calls` and `local_borrows_on_multithread_runtime` retain Rc and mutate borrowed vectors across await, then reuse those values between turns. `dropping_started_execution_releases_borrow_and_closes_connection`, both `one_shot_*borrows*` runtime tests, `dropping_one_shot_during_setup_or_callback_cleans_up`, `observed_turn_borrows_and_preserves_typed_result`, `observer_panic_releases_borrows_and_closes_connection`, and `dropping_observed_turn_discards_pending_events_and_releases_capture` exercise access after completion, drop, or unwind. | Rejection of overlapping mutable session executions and premature use of callback captures is in three rustdoc compile-fail probes, not nextest. Passing runtime checks alone cannot establish those compiler obligations. |
| `A-typed-turn-completion` · `behavior` | `typed_turns_and_stale_calls` and `local_borrows_on_multithread_runtime` return Summary then Decision in one session. `failures_and_corrections` checks correction, malformed/missing/duplicate results, exhaustion, cancellation and transport failure. `candidate_does_not_complete_before_terminal` asserts no early result and invalidation on drop; `candidate_is_discarded_when_terminal_overlaps_callback` expects CallbackOverlap. `contracts_follow_serde_direction`, `finite_floats_round_trip_and_nonfinite_tool_outputs_fail`, and `observed_errors_and_candidates_do_not_change_completion` check typed conversion and outcomes through the public API. | Abnormal-stop cases cover cancellation, not each other stop reason. Transport loss is injected before candidate acceptance, not afterward. No direct case makes a schema-valid result fail a custom Deserialize implementation. The peer's `null_missing` branch has no invoking product test, so it supplies no executed evidence for nullable results. |
| `A-turn-envelope-routing` · `behavior` | `typed_turns_and_stale_calls` drives six stale/missing/malformed/nonmatching calls while replacement callbacks are installed; exact input history and totals exclude unwanted mutation, and the peer asserts rejections and no current-ID disclosure. Current-ID calls succeed. `runtime::tests::revoked_queue_never_reaches_replacement_callback` checks failed sends on the revoked channel, rejection of A after installing B, and an empty replacement receiver. `http_limits_and_serialized_output_validation` also exercises unknown-tool rejection through the peer. | No missing named identifier/revocation case was identified in this bounded mapping. These are fixture cases on the current platform, not authenticated originating-turn evidence or a general MCP conformance result. |
| `A-cancelled-session-invalidation` · `behavior` | `failures_and_corrections` asserts unusable sessions, Closed on reuse and exactly one peer prompt after failures. `preflight_is_local_and_session_reservation_reusable` distinguishes an unpolled drop from started work. `dropping_started_execution_releases_borrow_and_closes_connection`, `candidate_does_not_complete_before_terminal`, `one_shot_preflight_and_busy_leave_agent_available`, `one_shot_failure_closes_connection`, `dropping_one_shot_during_setup_or_callback_cleans_up`, `unpolled_and_invalid_observed_plans_send_nothing`, and `dropping_observed_turn_discards_pending_events_and_releases_capture` check no unpolled work, closed reuse, or started-drop invalidation. | Named unpolled/started-drop, reuse and prompt-count cases have executable coverage. Prompt counts are asserted in the failure loop, not after every drop variant; no exhaustive ordering claim is made. |
| `A-owned-resource-cleanup` · `behavior` | `typed_turns_and_stale_calls` and `local_borrows_on_multithread_runtime` check normal shutdown and child reaping. `shutdown_errors_force_fallback_and_partial_setup` exercises normal, ignored-SIGTERM, exit-error and malformed-session setup paths, including diagnostic equality. `runtime_teardown_reaps_child` observes child disappearance after runtime destruction. `drop_cleanup_reports_errors_and_closes_listener` checks early-return cleanup, error diagnostics and refused TCP connections. Started-drop, both one-shot runtime tests, one-shot failure/setup-drop, observer panic and observed-drop tests also await child disappearance; one-shot reuse checks listener closure. | Failure, teardown and listener-closure cases do not cover the complete matrix of both Tokio runtime flavors. Runtime teardown is current-thread only. There is no direct instrumentation asserting that Drop spawns no task or performs no asynchronous wait. Current results cover Linux; historical macOS passes remain historical. Reaping checks use bounded ps polling and do not cover detached descendants. |
| `A-supported-contract-boundary` · `behavior` | All six `schema::tests` functions assert nested directional schemas, recursive/reference/constraint rejection, finite doubles, supported keyword acceptance/rejection independently of Serde, precise numeric equality and located unsupported-schema errors. `unsupported_contracts_and_options_fail_before_prompt` and `one_shot_preflight_and_busy_leave_agent_available` assert Contract failures with no peer prompt/setup. `tool_errors_are_visible_and_recoverable` excludes callback entry for invalid input. `http_limits_and_serialized_output_validation`, `contracts_follow_serde_direction`, and `finite_floats_round_trip_and_nonfinite_tool_outputs_fail` check serialization that contradicts its schema, directional names, and invalid/nonfinite wire values. | No explicit assertion covers extra-field acceptance under `additionalProperties: true`, malformed title/description metadata, or schema-valid custom Deserialize rejection. Generated valid metadata is exercised indirectly. The supported/rejected keyword tables are examples, not exhaustive schema-combination coverage. |
| `A-progress-observation` · `behavior` | Five `progress::tests` check queue count/bytes, overflow accounting, revoked producers, retained order while work waits, internal-tool filtering, and unwind cleanup after completed work. Six observation tests in `conversations.rs` check thread identity and non-Send captures, no unpolled/preflight notifications, release after panic/drop, old-session filtering, omitted private data, result correction/cancellation, and true callback deadlines versus application errors. | The observer capture compile-fail probe remains outside nextest. Queue tests compare against implementation constants, so they do not independently fix the designed 128-event/64-KiB values. The pending-work test does not assert the 16-event polling batch or starvation under sustained event production. Permission/control independence under a full queue is not directly exercised. |

### Checks outside the engine

`cargo test --workspace --locked` retains rustdoc checks. Three compile-fail blocks
on Plan::run/run_in in `api.rs` reject premature callback-capture access and
overlapping session borrows; one on ObservedPlan in `progress.rs` rejects premature
observer-capture access. Rustdoc runs these compiler rejection probes; the engine
cannot link their generated identities or consume that run. No proxy test or
previous result is relabeled as a nextest pass. Other `no_run` examples compile
without executing the described conversations.

The linked conversation tests launch the existing `peer.py`, and their Rust
results include its protocol assertions through the public API and expected
outcomes. The engine fingerprints the whole configured test directory, including
that peer, but does not import an independent Python report. Assertions after a
terminal response are not universally checked by exit status: some Rust cases
only drop and reap. The standalone migration capability-probe test and live-agent
examples remain outside this runner; their earlier evidence is preserved.

Local verification is Linux x86_64. The current-thread and multithread success
tests both run here; historical macOS and earlier Linux CI results retain their
original revisions in the product roadmaps. CI configuration adds a new Linux
verification run, not a macOS result. No inference, external-agent authentication,
new product behavior, or rendered verification-state UI is part of this work.

## Milestone 2. Every obligation participates in anchor satisfaction {#executable-verification-2}

**Outcome:** Known uncovered obligations have explicit criteria, and missing
anchors for the assessed design constraints are visible as draft proposals.
Passing runtime checks cannot satisfy independently uncovered obligations.

**Status:** In progress

**Identifier:** `executable-verification-2`

**Dependencies:** The engineer assigned this correction and selected Thinkwell as
the acceptance case. Milestone 1's runner and evidence links are available. The
separate design-reconciliation draft remains intact.

**Design basis:** The existing seven anchor properties, their refined criteria,
and the engine contract's complete-obligation rule. No product guarantee is
removed and no product body or existing test assertion is changed.

**Tasks**

- [x] Extract independently missing obligations from the broad criteria and the existing coverage map, preserving continuing criterion IDs and all anchor properties.
- [x] Keep source links on the assertions they establish; leave missing and unsupported checks explicitly unlinked.
- [x] Run verification and assert the expected covered and uncovered criterion states, including anchor aggregation.
- [x] Validate bootstrap, Book structure and rendered records/links, and record the exact test scope and limitations.
- [x] Assess example, documentation, and agent-integration constraints; propose missing anchors while preserving existing records and unresolved choices.
- [ ] Review the proposed anchor scope and authority, then map actual assertions and configure execution where supported.

### Completion criteria

#### Visible obligations {#executable-verification-2-obligations}

Every gap identified in the initial assertion map has a current owning criterion
or an explicit explanation of its scope. No passing link stands in for an
independently missing compiler, environment, or behavioral assertion.

- **Anchor coverage:** None — criterion decomposition and evidence mapping.
- **Evidence:** The seven `behavior` IDs continue the existing runtime scope; the table below identifies extracted obligations. Routing and cancellation keep their existing criteria because the bounded mapping found no independently missing named obligation. Original broader criterion results remain historical evidence, not evidence for the new criteria.

| Anchor | Criteria added for previously uncovered scope |
| --- | --- |
| `A-scoped-turn-ownership` | `borrow-rejection`: the existing compiler probes, which the current runner cannot consume |
| `A-typed-turn-completion` | `abnormal-completion`, `candidate-transport-loss`, `custom-result-decoding`, `nullable-result-submission` |
| `A-owned-resource-cleanup` | `runtime-cleanup-matrix`, `supported-platform-cleanup`, `drop-locality` |
| `A-supported-contract-boundary` | `open-object-fields`, `schema-metadata`, `custom-input-decoding`; custom result decoding belongs to the completion anchor |
| `A-progress-observation` | `observer-borrow-rejection`, `queue-limits`, `polling-fairness`, `control-under-pressure` |

The gap criteria assert behavior, not a manual coverage judgment. Existing
runtime tests are still linked only to `behavior`. The compiler probes remain
separate passing workspace evidence until the engine can consume their results.
Current Linux evidence cannot supply the required macOS result. All existing
requirements and prior results remain in their owning records and milestones.

#### Design constraint coverage {#executable-verification-2-design-coverage}

The assessed design exposes consequential constraints, their governing basis,
missing anchor coverage, and executable verification obligations for review.
Existing agreements and records retain their scope; new records remain draft.

- **Anchor coverage:** None — design/coverage reconciliation does not verify the proposed properties.
- **Evidence:** The common contract's constraint-coverage rule was applied to
  example applications, user documentation, and agent integration. Seven draft
  anchors below represent constraints already stated in those chapters. The seven
  existing anchors and all 22 criteria remain byte-identical. No implementation,
  test assertion, authority record, verification configuration, or unresolved
  choice changed. This is a bounded assessment, not a complete audit of the Book.

| Owning design and draft anchor | Constraint and consequence for review |
| --- | --- |
| [Scheduler](../design/examples.md#A-scheduler-validated-outcomes) | Rust validates extracted constraints and solver outcomes; generated explanation cannot replace the canonical result or establish interpretation accuracy. |
| [Sentiment](../design/examples.md#A-sentiment-measurement-provenance) | Reports distinguish observed tool measurements from generated interpretation and retain measurements after answer failure. |
| [Unminifier](../design/examples.md#A-unminifier-source-preservation) | Bounded source edits preserve unaffected text and property keys; general JavaScript equivalence and atomic output remain excluded. |
| [Session endpoint](../design/agent-integration.md#A-session-endpoint-isolation) | Per-session credentials, loopback binding, Origin rejection, and body limits control endpoint access; submitted turn IDs alone do not supply these controls. |
| [Permission grants](../design/agent-integration.md#A-declared-tool-permissions) | AllowOnce grants stay within declared tools and bounded correlation state; agent metadata is not authenticated provenance or an OS sandbox. |
| [Model selection](../design/agent-integration.md#A-explicit-model-selection) | Explicit selection fails visibly instead of silently falling back; portable discovery remains unresolved. |
| [Startup reports](../design/agent-integration.md#A-session-startup-failure-reporting) | Session-scoped adapter failures interrupt setup/work without poisoning replacement sessions; setup correlation has a fail-closed bound. |

The draft properties and their authority references are the review surface. Existing
first-product, Codex, example, and documentation approvals remain valid; any proposed
change to their meaning needs a design decision. The four [open choices](../open-questions.md#approval-bases-requiring-clarification)
about filesystem context, model discovery, deadline composition, and MCP negotiation
remain unresolved. The design overview only links owning properties and needs no
duplicate anchor. The selected properties protect session trust boundaries, explicit user intent,
validated results, measurement provenance, and user source files. Tutorial
compilation, excerpt and link checks, and example dependency layout remain
ordinary design and test obligations; being testable alone does not justify an
anchor. Explanatory prose and subjective teaching goals remain design judgments.

The new criteria start unlinked. Existing permission/model/startup tests and example
workspace tests are candidates for assertion mapping, not
passing evidence for these new criteria. The configured runner currently covers
library sources/tests; example execution requires its own supported
integration. Keep missing or unsupported checks explicit.

#### Computed state {#executable-verification-2-state}

The 38 existing linked checks retain their scope and pass. Among the original
seven anchors, two remain satisfied and five partial, with seven satisfied criteria
and 15 unlinked criteria. The seven new draft anchors and their 26 criteria remain
unlinked pending assertion mapping and execution. No missing obligation is reported
satisfied.

- **Anchor coverage:** None — consumer acceptance of the engine's all-criteria rule; it does not claim that the uncovered product obligations are implemented.
- **Evidence:** On 2026-11-24, `cargo anchor verify --config anchorbook.toml
  --method test --format json` passed on the local Linux `port-phase-adoption`
  tree based on `4f13784f3b07cfee26f745901cafe1b0ca3b2cf6`, with the Book and
  verification integration uncommitted. JSON and JUnit report 38 passing tests,
  seven satisfied criteria and 41 unlinked criteria: 14 anchors, with two
  satisfied, five partial, and seven unlinked. No failure, skipped test, or
  finding is reported. Exit 0 checks Complete-milestone Full claims (none here);
  it does not mean every anchor is satisfied.

  The original seven anchors retain their 22 criteria and previous states
  (seven satisfied, 15 unlinked). All 26 criteria in the seven new draft anchors
  are unlinked. Code identity `c5aec1dd9dd03591` and configuration identity
  `a9835f786b88232f` are unchanged; criteria identity is `cc07783c28a61d03`.
  The earlier 22-criterion run used `3173bc7858e8742e`; its results do not establish
  the added properties. Snapshot comparison confirms the original design prose
  and records are unchanged after excluding the new anchor blocks.

  Structural checking and `mdbook build --dest-dir target/design-constraints-book`
  pass. The engine's rendered-output acceptance passes for 44 complete records
  and 1,410 local links. `git diff --check` passes. The five bootstrap tests passed
  during the earlier criterion reconciliation; no launcher changes were made in
  this refinement. Product code and test assertions are unchanged, and missing
  executable coverage remains work rather than an inferred pass.

### Completion record

The seven draft additions await scope/authority review and executable evidence.
Earlier criterion decomposition and validation are complete. The milestone remains In progress
pending the project's review and durable integration process; CI, publication,
and a Port transition are not claimed. The correction exposes existing gaps;
implementing their missing checks is subsequent work. CI's report-only exit
policy and rendered-state UI are unchanged by this acceptance case.
