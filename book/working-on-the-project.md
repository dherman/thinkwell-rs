# Working on the project

Start with `cargo anchor contract --config anchorbook.toml`, then the root repository guidance, this Book's
`SUMMARY.md`, and every listed chapter. Run
`cargo anchor check --config anchorbook.toml` and report readiness before implementation.
Use the compiled contract's report template, identifying assigned work, open
decisions, and gaps. `AGENTS.md` is the compact startup entry point.

## Phase and assignment

The current lifecycle state is the [Porting Phase](lifecycle.md). The single
engine working-mode setting, `[project].phase` in `anchorbook.toml`, selects
`port`. Configured contract retrieval adds the engine's port guidance to the
common contract. This permits bounded research and experiments while requiring
accepted governing design for assigned implementation. The completed
[project-scaffolding roadmap](implementation-roadmaps/phase-0.md) preserves
historical evidence, not a numbered product phase.

Work requires an accepted design basis and a human-approved Ready milestone;
record it In progress before implementation. A roadmap alone does not authorize
work. No detailed port milestone is authorized by this workflow update. Agents
must distinguish observed legacy TypeScript behavior from accepted Thinkwell
requirements: migration evidence is not permanent authority. Record Rust-native
requirements and deliberate differences explicitly. See [migration research](migration.md).

Use the existing requirements, decisions, and migration findings before asking
for more information. Raise missing intent, compatibility choices, constraints,
or completion boundaries with the engineer, with a recommendation where possible.
Incorporate agreed findings into their owning design chapters, then update linked
roadmap work. Research and roadmap history must not be the only definition of
the target system.

Ending the Porting Phase requires separate human approval of the lifecycle gate.
Future phases retain no active dependency on the TypeScript implementation.
Follow [change delivery](change-delivery.md) for deliberate review, merge authority,
durable completion, and roadmap synchronization.

## Setup and validation

The repository-root `CONTRIBUTING.md` owns exact installation, version pins, checks,
and update instructions. The workspace has one unpublished `thinkwell` library in
`crates/thinkwell`; its initial supported scope is defined in
[typed conversations](design/typed-conversations.md), with evidence and completed
integration in the [product roadmap](implementation-roadmaps/first-product-slice.md). Edition 2024 and resolver 3 are scaffold settings. Rust 1.100.0 is
the owner-selected development baseline, not the project's MSRV.

`cargo anchor` is the Anchor Engineering entry point. Its dependency-free
`anchor-runner` binary under `tools/anchor/` checks the engine identity in
`target/tools/bin/` on every invocation. Missing or mismatched engines trigger
installation of the pinned Git revision with locked dependencies, followed by
another identity check. Failure stops execution. The launcher never looks up
anchorbook on PATH. It forwards engine arguments and streams; installation output
uses stderr so mdBook's JSON protocol stays intact. Cargo/rustup and the local
filesystem remain trusted; the identity check prevents accidental version drift,
not malicious binary impersonation. First use or recovery needs network access.

The alias works from the repository root and workspace subdirectories; engine
relative arguments retain the caller's working directory. The root remains the
recommended location for Book commands. The library is the default workspace
member; `--workspace` checks also exercise the development launcher.

Edit Book sources under `book/`. Generated HTML and locally installed tools belong
under ignored `target/`. Run structural checking after Book/configuration edits,
build with mdBook, and inspect affected records and navigation. Reconcile discoveries
into owning chapters rather than appending a session journal. Keep evidence tied
to its tested state; a green structural check establishes no product behavior.

Keep current definitions in [design](design/index.md), unresolved choices in
[open questions](open-questions.md), and source observations in the migration-only
research chapters. Roadmaps own assignments, approvals, and bounded evidence.
Update status and evidence in place when new facts supersede them; retain exact
revisions, assertions, results, failures, and limitations, and leave session
chronology and obsolete sketches recoverable through Git. An integrated historical
milestone is not a substitute for current executable anchor coverage.

## Pinned method references

The installed engine is pinned to
[`fb7badc8f3bb763f9ba89d6c15c8701faefe1e46`](https://github.com/anchor-book/anchor-book/tree/fb7badc8f3bb763f9ba89d6c15c8701faefe1e46),
with mdBook 0.5.4. Use the formats from that revision:

- [Anchor definitions](https://github.com/anchor-book/anchor-book/blob/fb7badc8f3bb763f9ba89d6c15c8701faefe1e46/book/design/anchor-definitions.md)
- [Authority definitions](https://github.com/anchor-book/anchor-book/blob/fb7badc8f3bb763f9ba89d6c15c8701faefe1e46/book/design/authority-definitions.md)
- [Verification](https://github.com/anchor-book/anchor-book/blob/fb7badc8f3bb763f9ba89d6c15c8701faefe1e46/book/design/verification.md)
- [Roadmap format](https://github.com/anchor-book/anchor-book/blob/fb7badc8f3bb763f9ba89d6c15c8701faefe1e46/book/implementation-roadmaps/index.md)
- [Working guidance and implementation status](https://github.com/anchor-book/anchor-book/blob/fb7badc8f3bb763f9ba89d6c15c8701faefe1e46/book/working-on-anchor-book.md)

The CLI implements `check`, `contract`, `md`, `verify`, and `proof`, plus help
and version. `check`, `contract`, `md`, and test verification are configured here.
`check` evaluates structural/v1;
the mdBook preprocessor also validates record syntax, authority references, and
roadmap identifiers and coverage. Neither establishes semantic correctness or
milestone completion. Criteria can require tests and proofs; their declarations
are obligations, not evidence. Review is not a verification method or proof guard.

Do not assume designed features are implemented. This revision has no initialization,
materialization, RFD template command/validation, cross-Book support, or `[engine]`
configuration. Use schema 1 and manual authoring. No RFD section or proof runner
is configured here. Product anchors use test criteria; nextest runs the linked
tests, and roadmaps preserve evidence and coverage limits. Verus and mutation
tooling require their own applicable scope, not merely upstream usage.

This pin includes the Port phase, configured contract delivery, and
programmatic-only verification from the engine's `programmatic-verification`
branch. The full revision, rather than the moving branch, is the installation target. If configuration prevents contract retrieval, use
`cargo anchor contract` for the common diagnostic guidance, fix the reported
configuration problem, and retry the configured command before port work.

## Verification and review

`[verification]` in `anchorbook.toml` selects `crates/thinkwell/src` and
`crates/thinkwell/tests`, output under ignored `target/anchorbook-verify`, and
the nextest runner. Source scope includes the Python peer used by the linked
integration tests, so its changes participate in run identity. Standalone examples,
bootstrap tests, and migration experiments retain their separate checks.

Use the root `CONTRIBUTING.md` commands to install nextest 0.9.151 and run
`cargo anchor verify --config anchorbook.toml --method test`
before the Book build. Add one `/// anchorbook: <anchor-id>#<criterion-id>`
doc-comment per criterion on the existing Rust test function that asserts its
behavior. Helpers and Python fixtures are reached through those tests; names,
documentation examples, and historical passes do not establish coverage. The
[verification roadmap](implementation-roadmaps/executable-verification.md#assertion-coverage)
maps assertions and states the uncovered obligations.

Each anchor must declare all its independently verifiable obligations. The
continuing `behavior` criterion covers the existing runtime assertions; extracted
compiler, failure-path, environment, and limit obligations have separate criteria.
Keep missing or unsupported checks unlinked until executable evidence establishes
them. Roadmap caveats cannot substitute for criteria or make an anchor satisfied.
Do not attach a passing runtime test to a compiler or platform criterion it cannot
exercise. The [criterion reconciliation](implementation-roadmaps/executable-verification.md#executable-verification-2)
records the extracted scope and current gaps.

The engine selects exact linked unit/integration tests and reads their current
nextest report. `satisfied` means every linked test passed in that run; it does
not establish that the links cover the entire criterion. Exit 0 is not universal
satisfaction: it checks evaluated Complete-milestone Full claims. Inspect every
reported state, including unlinked, no-evidence, not-evaluated, and inconclusive.
Use the engine's text or `--format json` output; no computed badges are added to
Book sources or rendered pages. Runner files are temporary execution output.

Rustdoc compiler probes and `no_run` examples are not supported evidence identities
in this engine. Continue running workspace tests, which include doctests, and
retain their separate evidence. Do not wrap an old result or an approval as a
passing test. Python assertions executed by a linked subprocess test are consumed
through its Rust result. A Linux run does not re-establish historical macOS
evidence; the cleanup runtime/platform matrix and remaining cases stay explicit.

Anchor satisfaction requires executable tests or proofs. Existing compiler,
runtime, transport, and schema checks retain their scope; no historical review
counts as a passing test. Report any behavior without an executable check as a
verification gap. Review ownership and protocol boundaries against exclusions,
event disclosure, lifecycle behavior, and live examples with both supported agents
through the ordinary design and change workflow. Those judgments cannot satisfy
or gate anchor verification.

## Review boundaries

Respect the [source-provenance constraint](authority/requirements.md#R-source-provenance)
before copying or adapting source. Public research is context, not approval to
publish unrelated material. Commit, push, publication, and repository-setting
authorization remain separate from implementing an assigned milestone.


## Book diagrams {#book-diagrams}

Use Mermaid fences for architecture diagrams. mdbook-mermaid renders them after
the Anchor Book preprocessor; the pinned installer and generated browser assets
are documented in CONTRIBUTING. Keep diagram meaning available in surrounding
prose and label process boundaries, messages, and ownership explicitly.

`theme/diagrams.css` adapts Anchor Book's quiet surfaces, rounded nodes, readable
labels, and theme-aware colors. Its `guide-context`, `guide-design`, `guide-build`,
and `guide-choice` classes select the shared palettes. Prefer a few consistent
colors; color alone must not carry a distinction. Inspect light and dark themes,
narrow-screen scrolling, and print output after changing diagrams or styling.

Provenance: the owner-selected local `theme/diagrams.css` from Anchor Book,
based on revision `4a24dca491b731e69a779fd69328068ef9ff6a21` with local stylesheet
changes, was adapted on 2026-11-24 under its MIT option. The complete copyright
and permission notice is retained in the stylesheet. This is development-theme
reuse, independent of the published engine revision in `tools/anchor/src/main.rs`;
no ongoing dependency on a sibling checkout is introduced.
