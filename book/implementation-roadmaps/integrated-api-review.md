# Integrated API review

Roadmap identifier: `integrated-api-review`.

The library's accepted definition is in [Design](../design/index.md). This record
preserves the review outcome and approval limits. The original uncompiled
[three-example proposal](https://github.com/dherman/thinkwell-rs/blob/1b21c83b9b713690d953df72244c7f9fcf8244ce/book/implementation-roadmaps/integrated-api-review.md)
remains in Git; it is not a second definition of the supported API.

Current integration state: completed work was delivered through owner-merged
[PR #2](https://github.com/dherman/thinkwell-rs/pull/2). See the
[shared integration evidence](first-product-slice.md#integrated-completion).

Goal: review one coherent candidate API through complete application workflows
before assigning another implementation experiment.

## Milestone 1. Prepare and review integrated examples {#integrated-api-review-1}

**Outcome:** Three worked examples show standalone use, a persistent conversation,
and embedding, including ordinary error cleanup and explicit review questions.

**Status:** Complete

**Identifier:** `integrated-api-review-1`

**Dependencies:** The owner approved preparing these three examples with “Sounds
good, proceed.” This is a bounded documentation/design assignment, recorded Ready
on that approval before preparation. Subsequent experiment and product assignments
were separate approvals; this review did not authorize publication.

**Design basis:** `D-acp-client`, `D-derived-contracts`,
`D-provisional-scoped-turns`, `D-connection-vocabulary`,
`R-easy-agent-programming`, `R-code-prompt-ergonomics`,
`R-progressive-control`, and `R-per-turn-result-types`;
[working design](../design/index.md) and
[experiment evidence and exploratory alternatives](acp-integration-prototype.md).

**Tasks**

- [x] Assemble three consistent caller examples with concrete result/tool types.
- [x] Include failure and cleanup paths and identify application helpers.
- [x] Review cross-example consistency, evidence limits, and unresolved choices.
- [x] Obtain owner review of the combined proposal before scoping implementation.

### Completion criteria

#### Standalone workflow {#integrated-api-review-1-standalone}

Show connection, a typed prompt with a borrowed tool, deterministic work after the
turn, and cleanup on ordinary success/error paths.

- **Anchor coverage:** None — provisional API review, no implemented guarantee.
- **Evidence:** The reviewed standalone sketch supplied all five steps, used ordinary
  RAII cleanup, and distinguished optional awaited shutdown confirmation. It was
  not compiled or executed; later product evidence is recorded separately.

#### Persistent conversation {#integrated-api-review-1-conversation}

Show reusable plans, fixed tool contracts, changed result types, optional progress,
and a concrete invalid-output/cancellation path within one conversation.

- **Anchor coverage:** None — provisional API review, no implemented guarantee.
- **Evidence:** The conversation example binds the same declared tool separately
  for two turns, clones a tool-free base plan, changes Summary to NextAction, and
  separates candidate acceptance from execution completion. Repair and cancellation
  defaults were unresolved in that review. This sketch was not compiled or run;
  the later bounded product scope selected its own defaults.

#### Embedded workflow {#integrated-api-review-1-embedding}

Show use of a host-managed connection and clearly assign initialization, I/O,
incoming routing, and cleanup responsibilities, including host failure.

- **Anchor coverage:** None — provisional API review, no implemented guarantee.
- **Evidence:** The embedded example describes a host connection-access adapter,
  shared caller-facing Agent interface, host-owned I/O and underlying shutdown,
  and cleanup limited to Thinkwell's work. No such adapter is implemented.

#### Integrated assessment {#integrated-api-review-1-review}

Make agreed directions, new candidate choices, failure behavior, and evidence
gaps distinguishable; deliver examples for owner review and record that review.

- **Anchor coverage:** None — design review only.
- **Evidence:** Local consistency assessment and checks are recorded below. On
  2026-11-19 the owner explicitly stated, “I reviewed integrated-api-review.md and
  approve of the design.” This completes owner design approval under
  `D-reviewed-api-baseline`. The design remains a provisional implementation basis
  with its explicitly unresolved choices retained, not verified behavior or an
  unspecified implementation assignment. PR #2 subsequently integrated the review.

## Review outcome and incorporation

The owner rejected mandatory explicit cleanup bookkeeping, a misleading finish
helper, and nested callback scopes. The approved provisional baseline uses RAII
for ordinary success and error paths and optional awaited shutdown for confirmation.
The [execution lifecycle](../design/execution-lifecycle.md) now owns the selected
local-release, driver-owned cleanup, fallback, and failure contracts. Drop cannot
await remote acknowledgment or provide async destructors for application callbacks.

The standalone and persistent-conversation sketches established a common Plan,
Tool, Session, and varying-result model: fixed catalogs, per-turn bindings, reusable
tool-free plans, and helpers without universal static callback bounds. Those
selected properties now belong to [typed conversations](../design/typed-conversations.md)
and [contracts](../design/contracts.md). The later result experiment and product
assignment supplied implementation evidence; this review supplied none.

The embedding sketch assigned initialization, I/O, incoming routing, and underlying
shutdown to the host; a wrapper could clean up only its own work. A common Agent
interface and connection-access adapter were provisional, with no compiled or live
host evidence. Attachment, model discovery/mutation, explicit cancellation handles,
and automatic recovery after abandoned turns were not selected by the bounded
product assignment. They remain deferred, without an accepted concrete API.

## Evidence and completion record

The implementing agent reviewed these documentation examples for consistent method
names, per-turn capture release, fixed-catalog bindings, result-type variation,
ordinary error cleanup, and host ownership. The owner reviewed the explicit-cleanup and callback-scope shapes and raised the
concerns recorded above. The owner subsequently reviewed the RAII-based proposal as a good starting
point and then explicitly approved this chapter on 2026-11-19. This is
design-approval evidence, not implementation verification. All original Rust
blocks were marked ignore; none was compiled against a product API. The live
evidence available to this review covered typed tools and a receipt in prose,
not typed final results or host embedding. No new model calls, implementation, dependencies, or runtime tests were
introduced for this documentation assignment.

Validation on the uncommitted port-design tree based on
83ac7e6faf7b4e7dfd74c306436760e44ba46a21:

- `cargo anchor check --config anchorbook.toml`: structural/v1 passed, zero errors.
- mdBook 0.5.4 build: passed; new chapter, navigation, and affected local links and
  fragments inspected successfully. The sketches remain uncompiled.
- `cargo test -p anchor-runner --test bootstrap_consistency --locked`: two tests passed.
- `git diff --check` and separate new-file whitespace inspection: passed.

Complete through owner-reviewed and merged PR #2; the
[shared completion record](first-product-slice.md#integrated-completion) supplies
final-head and merge-commit CI. Approval of the provisional model is not executable
anchor verification and does not expand the later bounded product scope.
