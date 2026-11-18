# Lifecycle and change delivery

Roadmap identifier: `lifecycle-delivery`.

Goal: extend the completed scaffold with owner-approved lifecycle and delivery
policy, without product functionality or a detailed port plan.

## Milestone 1. Integrate lifecycle and delivery {#lifecycle-delivery-1}

**Outcome:** The Book governs the Porting Phase, its human-approved transition,
and reviewable delivery; CI protects bootstrap relationships.

**Status:** In progress

**Identifier:** `lifecycle-delivery-1`

**Dependencies:** Completed project scaffolding. The project owner explicitly
approved this bounded assignment and its acceptance criteria in the lifecycle
and change-delivery request on 2026-11-17. The milestone was recorded Ready on that approval and is now In progress.
No port milestone is authorized.

**Design basis:** The owner's accepted lifecycle and delivery direction:
`R-temporary-migration-reference`, `R-porting-transition`, `R-reviewed-delivery`,
`D-thinkwell-identity`, and `D-future-independence`; [lifecycle](../lifecycle.md)
and [change delivery](../change-delivery.md).

**Tasks**

- [x] Record lifecycle authority and isolate migration research.
- [x] Define delivery, review, completion, and recommended GitHub protections.
- [x] Add the PR template and bootstrap consistency checks to existing CI.
- [x] Validate and review the complete change; record evidence and integration gaps.

### Completion criteria

#### Lifecycle and migration boundary {#lifecycle-delivery-1-lifecycle}

Accepted authority defines identity, the Porting Phase, human transition approval,
and future independence. Migration research is removable; historical scaffold
identifiers and evidence remain intact. No product design is introduced.

- **Anchor coverage:** None — project governance and scaffold only.
- **Evidence:** Local content review found the five accepted records, explicit
  transition checklist and future independence, migration-only source revision/API
  research and provenance, and no product functionality or detailed port plan.
  Historical `phase-zero` milestone and criterion identifiers/evidence are unchanged;
  only its display title and historical framing changed. Rendered authority records,
  lifecycle sections, and navigation were inspected. See validation context below.

#### Reviewable delivery {#lifecycle-delivery-1-delivery}

Guidance and the PR template cover approval, separate review, owner merge authority,
durable completion, tested revisions, and post-merge roadmap synchronization.
GitHub protections are recommendations, not claims of enabled settings.

- **Anchor coverage:** None — delivery workflow only.
- **Evidence:** Reviewed the delivery chapter and PR template against the owner's
  requested fields. They distinguish assignment, deliberate review, human approval,
  tested revision, integration, and post-merge synchronization. Recommended GitHub
  settings address CI, PR review, direct updates, merge permission, and stale
  approvals, including GitHub's sole-maintainer self-approval constraint. No settings
  were changed. Human review and integration remain pending.

#### Checked bootstrap and bounded validation {#lifecycle-delivery-1-validation}

Automated checks protect startup, navigation, pin documentation, and CI entry
points. Documented checks pass and rendered records and navigation are inspected.
Required CI and human review pass before authorized merge; integration and roadmap
synchronization are recorded before durable completion.

- **Anchor coverage:** None — scaffold integrity, not product behavior.
- **Evidence:** All local commands below passed. Two consistency tests check the
  checkout and deliberately broken relationships, including absent guidance,
  navigation drift, mismatched documentation/pins, and direct engine invocation.
  Initial validation left launcher tests unchanged; the CI follow-up below
  serializes fixture lifetimes without changing assertions or production behavior. Rendered navigation, affected authority
  and roadmap records, and local HTML links/fragments passed inspection. GitHub CI,
  human review, merge, and post-merge synchronization remain pending.

### Validation and review evidence

Validated 2026-11-17 on macOS arm64, against the uncommitted lifecycle/delivery
working tree based on `6bf7ada5c98b64222bcd495c9c57567a780252cb` on main.
At that validation, no new tested commit existed and changes were local, not
committed, pushed, or integrated. The owner subsequently authorized committing,
pushing, and opening a PR for review on branch `lifecycle-delivery-integration`.
The PR records the tested commit and current CI/review state; merge and durable
completion remain pending.
Rust 1.100.0, mdBook 0.5.4, and Anchor Book revision
`9689b00416fd577bfb68a02f1fa310f6088404cf` remain unchanged.

Commands passed:

- `cargo anchor contract`
- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets --locked`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked` (three existing launcher tests and two new
  consistency tests; no Thinkwell behavior tests)
- `cargo anchor check --config anchorbook.toml` (structural/v1, zero errors)
- `mdbook build` (using the documented local tool PATH)
- `cargo test -p anchor-runner --test bootstrap_consistency --locked`
- `git diff --check`; new files also inspected for whitespace

A separate post-implementation pass by the implementing agent reviewed all changed
and new sources, rendered navigation and affected records, and evidence scope.
This is self-review, not fresh-context or independent human review. Findings:
GitHub prevents PR author self-approval, so the recommendation now explains a
narrow owner exception while retaining human review and passing CI; historical
local scaffold completion must not imply durable integration, now explicitly
qualified. The initial consistency test incorrectly assumed a manifest-path alias;
it was corrected to the existing package-based alias, and rejection tests now
first require the unmodified checkout to pass. No launcher changes were needed.
A subsequent review made the negative identity fixture independent of the numeric
engine version. All affected checks were rerun successfully.

Limits: these checks establish scaffold/workflow integrity only. They do not verify
Thinkwell behavior, upstream source rights, external links, GitHub CI execution,
Linux execution of the new tests, or enabled repository protections. The engine's
structural check does not validate lifecycle semantics, prose links, or delivery
approval; the human gate remains procedural. No verification runners were added.

### CI failure investigation

The initial head `6fa55d27fd4bcf7e52078f7b991559e48946f839` had different CI outcomes:
[push run 44255081373](https://github.com/dherman/thinkwell-rs/actions/runs/44255081373)
failed in the existing `missing_and_mismatched_engines_install_the_pin` test with
Linux `ExecutableFileBusy` / `Text file busy` (OS error 26), while
[PR run 44255183349](https://github.com/dherman/thinkwell-rs/actions/runs/44255183349)
passed all checks on the same head. The failure occurred starting the fixture
installer, before the new consistency tests ran.

The likely race is concurrent script creation and process spawning in launcher
tests: a child can temporarily inherit a writable script descriptor before exec
closes it, preventing Linux from executing that file. The focused correction holds
a process-local mutex for each fixture's lifetime, including script creation,
execution, and cleanup. Assertions, test coverage, production launcher behavior,
and pinning are unchanged. Lock poisoning is recovered so one failed assertion
does not prevent remaining fixtures from exercising their checks.

Local validation of this follow-up: formatting, workspace compilation, Clippy,
workspace tests, Book structure/rendering, and whitespace checks passed; 20 repeated
launcher test runs with three test threads passed. The failure was not reproduced
on macOS. A local Linux container was unavailable because Docker's daemon was not
running. The updated PR head's Linux CI remains the required confirmation; passing
local repetitions do not establish that the intermittent Linux failure is resolved.
The PR identifies the tested follow-up commit. Human review and integration remain
pending.

### Completion record

Not complete. Local implementation and validation do not establish durable completion.
