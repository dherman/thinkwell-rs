# Implementation roadmaps

The [library design reconciliation](library-design-reconciliation.md) is the
assigned Book-only pass. It consolidates the library definition while preserving
implementation and historical evidence; examples, the documentation site, and
delivery-status cleanup remain separate.

[Project scaffolding](phase-0.md) is historical completed preparation.
[Lifecycle and change delivery](lifecycle-delivery.md),
[ownership API](ownership-prototype.md),
[ACP integration](acp-integration-prototype.md),
[integrated API review](integrated-api-review.md),
[runtime and typed-result foundation](runtime-results-prototype.md), and the
[first product slice](first-product-slice.md) are complete through owner-merged
PR #2; its [integration record](first-product-slice.md#integrated-completion)
connects the reviewed revision and CI evidence to their bounded criteria.

The [meeting scheduler](meeting-scheduler.md#integrated-completion), automatic
schema inlining, and structured prompt context milestones are complete through
owner-merged PR #4 at 558f3ba. Final-head Linux CI passed for both jobs. The
completion record tracks the separate merge-commit CI and this Book follow-up.
The owner merged the closeout/proposal in PR #5 at 343f425, then reviewed and
merged [sentiment analysis and finite f64 support](sentiment-analysis.md#integrated-completion)
in PR #6 at 0dbb432. Final-head and merge-commit CI passed. These milestones are
complete; this Book follow-up records their integration. The owner merged this closeout in PR #7 at 8e039b1 and approved the
[one-shot tools milestone](one-shot-tools.md#integrated-completion), now complete
through owner-merged PR #8 at 43c147c. Final-head and merge-commit CI passed.
The owner-selected [Codex investigation](codex.md) found a permission-message
compatibility gap, resolved in owner-merged PR #9 at d1becc9. Its
[integrated completion](codex.md#integrated-completion) records final-head CI and
review. The owner merged [progress observation](progress-observation.md#integrated-completion)
in PR #10 at ab1e821; both bounded milestones are complete with final-head CI passing.
The [unminifier investigation](unminifier.md) passed local AST and live-agent
experiments. Its standalone example and the owner-assigned Plan::code follow-up
are In progress; local validation passes and integration remains pending.

Use the pinned [roadmap format](../working-on-the-project.md#pinned-method-references):
stable milestone and criterion fragments, outcome, status, identifier,
dependencies, design basis, tasks, criteria, evidence, and completion record.
Keep evidence with its criterion, including revision context, exact checks,
results, supporting observations, limitations, and integration state.

Preparation criteria may have no anchor coverage, with an explicit reason.
Do not manufacture product anchors for setup work. Local validation and an authored
CI workflow are distinct from a GitHub workflow run or integration into main.

The owner-assigned [Rust documentation site](documentation-site.md) now includes
the core learning path, worked examples, practical reference, and local validation.
The owner approved committing and pushing this starting point without a PR.
Integration, CI automation and publication remain separate work.
