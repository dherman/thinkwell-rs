# Open questions

The [design](design/index.md) defines accepted behavior. This chapter tracks
unresolved choices; [roadmaps](implementation-roadmaps/index.md) own assignments
and delivery evidence, and [porting research](migration.md) owns source observations.
A question does not authorize implementation or turn a source observation into a
requirement.

## Approval bases requiring clarification

The [library reconciliation](implementation-roadmaps/library-design-reconciliation.md#approval-bases-and-open-choices)
identified four implementation choices without a clear specific approval basis:

| Area | Decision still needed |
| --- | --- |
| Working directory | Is a temporary directory reused across a connection's sessions the intended filesystem-context policy? |
| Model selectors | Should custom agents be supported through discovered model-category IDs rather than assuming the ID `model`? |
| Deadline composition | What exact timer boundaries and end-to-end latency commitments should setup and turn limits provide? |
| MCP negotiation | Which initialization revisions and protocol-header omission cases belong to the supported HTTP contract? |

The existing implementation and bounded guarantees remain intact. None of these
observations is silently promoted into a new guarantee by Book cleanup.

## Future scope decisions

| Area | Question and owning boundary |
| --- | --- |
| Runtime and embedding | Is there a use case for shared host connections or concurrent sessions on one connection? [Current ownership](design/execution-lifecycle.md) covers owned connections. |
| Schemas | Which additional validation/reference-resolution capabilities justify extending the [finite contract subset](design/contracts.md)? |
| Provider coverage | Which additional agents, capabilities, model discovery, or transports should be supported beyond [the selected integration](design/agent-integration.md)? |
| Cancellation recovery | Is recovery needed beyond explicit reconnection under the [invalidation policy](design/execution-lifecycle.md)? |
| Turn provenance | Do applications need agent-bound causal-origin evidence beyond [matching submitted identifiers](design/agent-integration.md)? |
| Platform/compiler support | When should Windows or a lower MSRV become commitments? |
| Documentation publication | Publication is paused at the owner’s request; Loopia registration is incomplete. Preserve local docs and [hosting restart context](implementation-roadmaps/documentation-site.md#publication-pause-and-handoff). Local review is the priority; public Rustdoc hosting is separate. |

The [executable-verification record](implementation-roadmaps/executable-verification.md#executable-verification-2)
tracks uncovered obligations separately. Those obligations are already required
behavior, not optional design questions. Migration-source gaps remain in
[research](migration.md) and [library foundation observations](library-foundation-research.md).
Ending the Porting Phase requires separate human approval.
