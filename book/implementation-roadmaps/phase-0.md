# Phase 0: project scaffolding

Roadmap identifier: `phase-zero`.

Goal: make the repository ready for a later design session with a compilable
minimal workspace, an authoritative Book, reproducible checks, and explicit gaps.

## Milestone 1. Establish the project scaffold {#phase-zero-1}

**Outcome:** Humans and agents can load project context and validate an empty
Rust workspace and its Book without product functionality.

**Status:** Complete

**Identifier:** `phase-zero-1`

**Dependencies:** The owner approved the scaffolding proposal, named the library
thinkwell, and selected Rust 1.100.0 before implementation. This assignment was
Ready on approval; implementation is limited to that scope.

**Design basis:** [Boundary](../boundary.md), [requirements](../authority/requirements.md),
and [accepted decisions](../authority/decisions.md).

**Tasks**

- [x] Create the workspace, empty unpublished library, and pinned toolchain.
- [x] Author the Book, root guidance, and compact agent startup instructions.
- [x] Define validation-only CI with pinned tools.
- [x] Run local checks, inspect rendered output and scope, and record evidence.

### Completion criteria

#### Compilable minimal Rust workspace {#phase-zero-1-workspace}

The unpublished thinkwell library has no product API or dependencies. Formatting,
compilation, Clippy with warnings denied, and the test harness pass.

- **Anchor coverage:** None — contributor setup, with no product behavior defined.
- **Evidence:** Passed locally with Rust 1.100.0: `cargo fmt --all -- --check`,
  `cargo check --workspace --all-targets --locked`,
  `cargo clippy --workspace --all-targets --locked -- -D warnings`, and
  `cargo test --workspace --locked` each exited 0. The test harness ran zero unit
  tests and zero doc tests, as expected for the empty API. At initial validation `Cargo.lock` listed only
  thinkwell 0.0.0; milestone 2 adds the dependency-free development launcher. These results establish scaffold compilation, not product behavior.

#### Valid Book and usable startup guidance {#phase-zero-1-book}

All Book chapters are navigable, the declared structure checks successfully, and
mdBook renders records and roadmap criteria. Agent guidance requires the compiled
contract, full Book reading, readiness reporting, and approved work.

- **Anchor coverage:** None — project knowledge and contributor setup.
- **Evidence:** `anchorbook contract` exited 0 and delivered the installed
  contract. `anchorbook check --config anchorbook.toml` passed structural/v1 with
  zero errors. `mdbook build` exited 0 with the Anchor Book preprocessor, validating
  records and roadmap references and producing HTML. Inspection found rendered
  authority records, all three criterion sections, and working local links and
  fragments. A repository-root contributor link initially pointed outside the
  rendered Book; it was changed to an explicit source-file reference. The startup
  instructions cover all seven requested steps. These checks do not establish
  design semantics or product verification.

#### Reproducible validation definition and bounded scope {#phase-zero-1-validation}

CI defines the same local Cargo and Book checks with pinned tools, and review
finds no product implementation or unauthorized infrastructure. Whitespace checks
pass. CI execution on GitHub is not required for this local scaffold outcome.

- **Anchor coverage:** None — validation infrastructure preparation.
- **Evidence:** Reviewed `.github/workflows/ci.yml` against the documented local
  checks: Rust 1.100.0, full-SHA Anchor Book installation with locked dependencies,
  mdBook 0.5.4, SHA-pinned checkout, read-only permissions, and no publication steps.
  `git diff --check` exited 0; new files were separately inspected for trailing
  whitespace because unstaged new files are absent from that diff. The final source
  inventory contains one empty library, Book/guidance, manifests, tool pins, and CI;
  no product implementation or speculative dependencies. GitHub execution and
  Linux validation remain untested; this criterion covers the workflow definition.

### Evidence {#phase-zero-1-evidence}

Validation date: 2026-11-17. Tested state: the uncommitted Phase 0 working tree
based on `80040d9631c5f21069a108539733edf594557c72`, with code and Book in the same
checkout. No new commit identity exists. Integration context: local changes on
main, not committed, pushed, or integrated into the remote repository.

Tool identities observed locally on macOS arm64:

- `rustc --version`: `rustc 1.100.0 (0ef1bc386 2026-11-09)`.
- `anchorbook --version`: `anchorbook 0.1.0 (9689b00416fd577bfb68a02f1fa310f6088404cf)`.
- `mdbook --version`: `mdbook v0.5.4`.

Initial validation installed both Book tools using the then-documented exact
`cargo install` commands, compiled with Rust 1.100.0. Milestone 2 replaces direct
engine installation/invocation with the checked Cargo alias. Installation and version checks
passed. Supporting observations are the local command output and reproducible
commands above; generated HTML is under ignored `target/anchorbook/`. No raw
machine-specific logs are committed. GitHub CI has not run; no push is authorized.
The Book was rebuilt and structurally rechecked after this evidence update.

### Completion record

Complete for local Phase 0 preparation: all three criteria have the bounded
evidence above. No Thinkwell functionality is implemented or verified. The project
phase remains prototyping. The approved tooling follow-up is tracked separately
in milestone 2; no product implementation is assigned. Source licensing, product design, and all listed open decisions remain
for future human-directed work.

## Milestone 2. Enforce the pinned Anchor engine {#phase-zero-2}

**Outcome:** `cargo anchor` runs the repository's pinned engine independently of
any ambient anchorbook executable, including agent startup and Book rendering.

**Status:** Complete

**Identifier:** `phase-zero-2`

**Dependencies:** Milestone 1; the owner explicitly approved a local checking and
installing launcher and selected the alias anchor. No product design is assigned.

**Design basis:** [Project tooling guidance](../working-on-the-project.md) and
`R-reproducible-checks`.

**Tasks**

- [x] Implement a dependency-free development launcher and Cargo alias.
- [x] Route startup, contributor commands, rendering, and CI through the launcher.
- [x] Exercise tool selection, recovery, argument/stream forwarding, and failure behavior.

### Completion criteria

#### Pinned execution and recovery {#phase-zero-2-launcher}

The launcher uses only the local engine, installs the exact Git revision with
locked dependencies when missing or mismatched, rechecks after installation,
and refuses to proceed on failure. Arguments, stdin, stdout, and exit codes are
preserved for engine commands; installation diagnostics do not corrupt stdout.

- **Anchor coverage:** None — development tooling, not a product guarantee.
- **Evidence:** `cargo test --workspace --locked` passed three launcher tests:
  reuse without an installer, missing/mismatched recovery with exact install
  arguments, and rejection of failed or incorrectly identified installations.
  A local actual-process acceptance check placed a failing anchorbook first on
  PATH: `cargo anchor --version` still returned the pinned identity. From
  `crates/thinkwell`, `cargo anchor contract` matched the engine's contract bytes.
  A config argument containing spaces reached the checker correctly, and
  `cargo anchor unknown-operation` preserved exit code 2. Temporarily removing
  the local engine exercised a real `cargo anchor --version` installation;
  stdout contained only the expected version and installation output used stderr.
  These checks passed on macOS arm64; bootstrap fixture tests use Unix shell scripts.

#### Consistent entry points {#phase-zero-2-integration}

Agent guidance, contributor commands, CI, and the mdBook preprocessor use
`cargo anchor`; the Book checks and renders successfully without an ambient engine.

- **Anchor coverage:** None — contributor setup.
- **Evidence:** `cargo anchor check --config anchorbook.toml` passed with zero
  structural errors. `mdbook build` passed using `cargo anchor md` while a failing
  ambient anchorbook was first on PATH, exercising JSON stdin/stdout forwarding.
  Reviewed `AGENTS.md`, contributor guidance, and CI to confirm they use the alias.
  `cargo fmt --all -- --check`, `cargo check --workspace --all-targets --locked`,
  `cargo clippy --workspace --all-targets --locked -- -D warnings`, and
  `git diff --check` all passed. The library remains empty; only development tooling
  was added. GitHub CI and Windows execution have not been exercised.

### Evidence {#phase-zero-2-evidence}

Validated 2026-11-17 against the uncommitted working tree based on
`80040d9631c5f21069a108539733edf594557c72`, using Rust 1.100.0, mdBook 0.5.4,
and the unchanged engine revision `9689b00416fd577bfb68a02f1fa310f6088404cf`.
Code and Book were checked in the same local checkout; nothing was committed or
pushed. Commands and acceptance observations above are the supporting evidence.
The Book was rechecked and rendered after recording these results. Scope is local
preparation; GitHub CI remains unrun. Engine numeric exit codes are forwarded;
termination by a signal is reported as failure (exit 1), not signal propagation.

### Completion record

Complete for the approved local tooling follow-up. The alias enforces the expected
local engine identity and is the common entry point for startup, checking, and
rendering. Product design and implementation remain unassigned.
