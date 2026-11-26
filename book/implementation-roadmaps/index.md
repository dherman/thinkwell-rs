# Implementation roadmaps

The [design](../design/index.md) defines the supported system. These roadmaps
record bounded assignments, approval, executable evidence, and integration.
Completed historical experiments do not expand current support or satisfy
unlinked anchor obligations.

## Current branch work

| Assignment | Current state and boundary |
| --- | --- |
| [Port working contract](lifecycle-delivery.md#lifecycle-delivery-2) | Locally validated on the fork; upstream review, CI, and integration remain outstanding. |
| [Library design reconciliation](library-design-reconciliation.md) | Component reconciliation prepared; the assigned [system overview](library-design-reconciliation.md#library-design-reconciliation-2) connects the complete Rust system. Unclear approval bases remain explicit. |
| [Executable verification](executable-verification.md) | Local runner/CI wiring and criterion reconciliation validated; the current result and all uncovered obligations are recorded there. |
| [Book contract cleanup](lifecycle-delivery.md#lifecycle-delivery-3) | Reconcile knowledge placement and stale completion accounts, preserving design, implementation, and evidence. Separate commit in the combined PR; no lifecycle transition. |

These assignments remain In progress until their delivery obligations are met.
Existing human approvals authorize their recorded scope; roadmap presence alone
does not assign further implementation or missing coverage work.

## Combined delivery

The owner assigned these changes to one PR for the Thinkwell author. The local
`book-design-reconciliation` branch keeps setup, library reconciliation,
verification, cleanup, draft anchors, system overview, and diagram styling in
separate commits. A subsequent pin commit adopts merged Anchor Book revision
`7966c9fddb3e9b9a710444c365cd698550de9b5f`, including conversational user guidance.
The local trial launcher is excluded; configured contract retrieval uses the pin.

Local validation is recorded in the owning roadmaps; the
[merged-pin validation](lifecycle-delivery.md#lifecycle-delivery-2-guidance)
checks this combined Book against the published engine. The user authorized
pushing `book-design-reconciliation` to `kwdev/thinkwell-rs` and opening the
upstream PR with both the prepared Book commits and the new pin. Required CI
must establish results for the submitted revision; upstream human review,
owner-controlled integration, and roadmap synchronization remain outstanding.

## Integrated work

| Delivered scope | Evidence record |
| --- | --- |
| Project scaffolding and lifecycle/delivery governance | [Scaffolding](phase-0.md), [lifecycle milestone 1](lifecycle-delivery.md#lifecycle-delivery-1) |
| Ownership, ACP, API review, and runtime/result experiments | [Ownership](ownership-prototype.md), [ACP](acp-integration-prototype.md), [API review](integrated-api-review.md), [runtime/results](runtime-results-prototype.md); shared integration through PR #2 |
| Initial typed-conversation library | [First product slice](first-product-slice.md#integrated-completion), PR #2 |
| Scheduler, automatic schema inlining, and JSON context | [Scheduler](meeting-scheduler.md#integrated-completion), PR #4 |
| Sentiment example and finite f64 contracts | [Sentiment](sentiment-analysis.md#integrated-completion), PR #6 |
| One-shot execution and borrowed inline tools | [One-shot tools](one-shot-tools.md#integrated-completion), PR #8 |
| Codex integration | [Codex](codex.md#integrated-completion), PR #9 |
| Borrowed progress observation | [Progress observation](progress-observation.md#integrated-completion), PR #10 |
| Unminifier and code context | [Unminifier](unminifier.md#integrated-completion), PR #11 |
| Rust learning site and its CI | [Documentation](documentation-site.md#integrated-completion), owner-authorized direct delivery at 756cd93 and 59b8bd5 |

## Recording evidence

Use the pinned [roadmap format](../working-on-the-project.md#pinned-method-references):
stable milestone and criterion fragments, outcome, status, identifier,
dependencies, design basis, tasks, criteria, evidence, and completion record.
Update current evidence in place. Preserve tested revisions, commands, assertions,
results, failures, limitations, and approval/integration basis; leave superseded
planning and session chronology in Git.

Preparation criteria may have no anchor coverage, with an explicit reason.
Local validation, an authored CI workflow, an actual GitHub run, and integration
are separate facts. Review is delivery evidence, not anchor verification.
