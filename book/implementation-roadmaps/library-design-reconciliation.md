# Library design reconciliation

Roadmap identifier: `library-design-reconciliation`.

Goal: make the library's intended behavior understandable from its design chapters,
preserving accepted guarantees, implementation, and historical evidence.

## Milestone 1. Reconcile the library definition {#library-design-reconciliation-1}

**Outcome:** A coherent library design replaces scattered current definitions in
library roadmaps; unclear approval bases and evidence limits remain explicit.

**Status:** In progress

**Identifier:** `library-design-reconciliation-1`

**Dependencies:** The user explicitly assigned the proposed library reconciliation
on the `port-phase-adoption` checkout at `d8133a5`. Scope was recorded Ready on that
instruction and readiness reported before editing. The claim map is temporary.
Existing implementation and evidence must be preserved. Examples, the user
documentation site, delivery cleanup, and the lifecycle transition are excluded.

**Design basis:** `R-explicit-authority`, `D-versioned-project-knowledge`,
`R-reviewed-delivery`, `R-temporary-migration-reference`, the existing accepted
library authorities and anchors, and [change delivery](../change-delivery.md).

**Tasks**

- [x] Trace library claims to recorded approval, design, and implementation evidence.
- [x] Reconcile owning design chapters and replace duplicated library definitions
      with links, preserving record identities and historical evidence.
- [x] Run Book checks and prepare the design and complete diff for presentation,
      with unresolved choices, before any expansion of scope.

### Completion criteria

#### Coherent definition {#library-design-reconciliation-1-design}

The design explains the supported programming model, ownership and lifecycle,
contracts and results, agent integration, and progress without requiring readers
to reconstruct roadmap history. Approval uncertainties are explicitly identified.

- **Anchor coverage:** None — design reconciliation, not new product verification.
- **Evidence:** The library overview and five focused chapters define the
  programming model, execution lifecycle, contracts/results, agent integration,
  and observation. A temporary claim map outside the repository traces accepted
  behavior to the prior design and recorded approvals. A separate author review
  followed successful and failed turns across the chapters and inspected the
  corresponding API, runtime, schema, permissions, startup, and progress sources.
  The approval questions below are retained rather than resolved from code alone.
  This is author assessment, not independent or owner acceptance of new choices.

#### Preservation {#library-design-reconciliation-1-preservation}

Existing authority and anchor identities, properties, criteria, implementation,
and historical verification evidence retain their meaning. Roadmap links identify
the owning design without changing completed outcomes or claiming new verification.

- **Anchor coverage:** None — preservation assessment, not execution of anchor criteria.
- **Evidence:** At this pass's validation, comparison against the baseline at 4f13784
  confirmed all seven anchor definitions (including their authority and criteria) are byte-for-byte
  unchanged, with one definition per ID. The three authority files, all 72 existing
  fourth-level completion-criterion blocks, existing milestone statuses, and the
  historical execution/review sections of changed library roadmaps are unchanged.
  Prototype chapters retain their full original content after their identifiers,
  with only a new current-design pointer preceding it. Example, documentation-site,
  lifecycle-delivery, and scaffolding roadmaps are unchanged from that baseline.
  All reconciliation edits are in the Book; no implementation, test, dependency,
  or configuration file changed.

#### Reviewable Book {#library-design-reconciliation-1-checks}

The structural check and Book build pass; affected records, navigation, links, and
diagrams are inspected. The resulting diff and remaining limitations are presented
for review within the assigned library scope.

- **Anchor coverage:** None — Book validation and design review preparation.
- **Evidence:** On the uncommitted port-phase-adoption tree based on 4f13784,
  `cargo anchor check --config anchorbook.toml` passed structural/v1 with zero
  errors. `mdbook build` passed with mdBook 0.5.4 and the pinned Anchor preprocessor;
  a generic type in a table initially caused an HTML warning and was corrected.
  The repository-local mdBook path was absent; the installed 0.5.4 on PATH was used.
  Inspection checked all 32 navigated chapter outputs, the relocated anchor IDs,
  ownership diagram, tables, and 672 local links/fragments with no missing targets.
  Both bootstrap-consistency tests passed via
  `cargo test -p anchor-runner --test bootstrap_consistency --locked`.
  `git diff --check` and separate whitespace inspection including new files passed.
  The complete review diff includes untracked new chapters without staging them.
  These checks establish Book integrity, not runtime verification, new CI results,
  or durable completion. Product tests and live inference were not rerun.

### Completion record

The local library reconciliation and checks are prepared for user review. The
milestone remains In progress pending review and the project's integration process;
neither accepted anchors nor historical completion records have been reset. This
assignment stops at presenting the design and diff. Examples, the documentation
site, delivery cleanup, implementation changes, publication, and the lifecycle
transition remain outside this pass.

### Approval bases and open choices {#approval-bases-and-open-choices}

This reconciliation preserved the accepted authorities and seven anchors verbatim
from 4f13784, which migrated verification to executable tests and moved review
into project guidance. The separately assigned [criterion reconciliation](executable-verification.md#executable-verification-2)
subsequently split uncovered obligations into explicit criteria; that change is
outside this pass's preservation comparison. Historical review remains review evidence;
it does not satisfy or gate anchor verification.
The first product assignment and its accepted scope supply the basis for owned
Tokio/ACP connections, fixed catalogs, typed results, supported schemas, and
conservative failure handling. Later recorded approvals supply automatic inlining,
finite f64, JSON/code context, one-shot tools, Codex compatibility, and observation.
Those approvals remain in their original roadmap records; this pass does not
invent separate approvals for moving their definitions.

The wider integrated API review remains a provisional basis. Its host-attachment,
model-discovery/mutation, and explicit cancellation-handle sketches are not
silently incorporated as supported APIs. The bounded product scope defers them.
This pass changed no accepted requirement, decision, anchor property, or
verification method.

The following source observations have no clear specific approval basis in the
read Book. They remain observations and review questions, not new guarantees:

| Choice | Observation and consequence | Disposition for this pass |
| --- | --- | --- |
| Agent session working directory | `runtime.rs` creates a temporary directory in the owned driver and reuses it for that connection's sessions. Rustdoc says the working directory is managed internally. The Book does not clearly select this filesystem-context policy. | No new isolation or application-directory guarantee. Decide whether this is the intended durable policy before codifying it or adding a cwd API. |
| Model-selector compatibility | `runtime.rs` sends the requested value through config ID model, without discovering model-category IDs. Optional model selection is approved, but that assumption for custom agents is not separately settled. | Preserve default selection and explicit failure semantics; do not promise generic discovery or compatibility with other selector IDs. |
| Deadline start and composition | Configuration documents per-operation setup bounds; setup has nested waits, and the turn timer surrounds result/tool processing after command enqueue. The Book specifies default numbers but not an exact wall-clock bound from first poll across all stages. | Preserve the configured bounds without claiming a new end-to-end latency guarantee. Exact timer boundaries require explicit design reconciliation before changing behavior. |

The narrow HTTP handler also does not establish general MCP conformance: it
reports 2025-11-25 during initialization and permits omission of the protocol-version
header. The approved integration target does not settle every negotiation/header
case. This pass retains that bounded target, without elevating those observed
details to a general protocol contract.

These questions do not block consolidation of already accepted behavior. The
implementation is preserved, and no new product work is assigned to resolve them.
Delivery-status corrections, including unminifier closeout, belong to the later
[lifecycle-delivery-3 cleanup](lifecycle-delivery.md#lifecycle-delivery-3), using
recorded approvals and inspected integration evidence. They are not part of this
library reconciliation or its separate diff.
