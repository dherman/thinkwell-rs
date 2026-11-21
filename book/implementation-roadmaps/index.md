# Implementation roadmaps

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
No next implementation milestone is assigned. Sentiment analysis is the
complementary candidate in [migration research](../migration.md#example-selection),
subject to a bounded proposal and owner approval.

Use the pinned [roadmap format](../working-on-the-project.md#pinned-method-references):
stable milestone and criterion fragments, outcome, status, identifier,
dependencies, design basis, tasks, criteria, evidence, and completion record.
Keep evidence with its criterion, including revision context, exact checks,
results, supporting observations, limitations, and integration state.

Preparation criteria may have no anchor coverage, with an explicit reason.
Do not manufacture product anchors for setup work. Local validation and an authored
CI workflow are distinct from a GitHub workflow run or integration into main.
