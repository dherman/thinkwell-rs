# Design

The intended system combines deterministic Rust code with LLM-powered reasoning
while preserving important Thinkwell concepts. The governing
[requirements](../authority/requirements.md) and [boundary](../boundary.md) frame
future design work.

## ACP client foundation

Thinkwell acts as a headless, programmatic Agent Client Protocol (ACP) client,
as established by `D-acp-client`. Agent communication uses ACP; the Rust API
makes it easy to interleave deterministic code with LLM prompts.

The initial programming experience minimizes setup and uses reasonable defaults.
Callers take on additional complexity as they choose more control. Compatibility
across model providers comes through ACP agents; supported agents and their
capabilities need explicit scope. These goals are governed by
`R-easy-agent-programming`, `R-progressive-control`, `R-provider-breadth`, and
`R-code-prompt-ergonomics`.

Detailed API, defaults, protocol version, crate boundaries, and dependencies
remain design questions. The provisional ownership direction is described below. Protocol support for a control does not
by itself establish that every agent exposes it.

The [typed-conversation design](typed-conversations.md) defines initial product
properties and supported scope. Preparation progress belongs in the
[project-scaffolding roadmap](../implementation-roadmaps/phase-0.md).

## Type-derived contracts

Under `D-derived-contracts`, developers derive tool contracts from Rust types as
part of ordinary compilation, without an extra Thinkwell schema-generation build
tool. The chosen schema representation must account for the serialized types
used at the tool boundary. Derivation supplies contract metadata; runtime
serialization, input handling, and tool invocation are separate responsibilities.

## Scoped turns and explicit conversations

Under `D-provisional-scoped-turns`, the working ownership design separates an
Agent's connection, reusable Plan configuration, Session conversation state, and
an Execution's active turn. An execution exclusively borrows its session and owns
its scoped tool callbacks. Conversation history can persist after those callbacks
and their application borrows have ended. Independent connections provide concurrency in the initial supported slice.

Explicit sessions declare a fixed tool catalog before opening and bind callback
implementations per turn. A new turn rejects stale submitted turn identifiers before callback routing;
identifiers do not authenticate causal origin when the sender uses the current one.
Simple independent calls use fresh conversational context on a reusable connection;
explicit sessions preserve history using the same prompt-composition model.

Awaited cancellation and execution drop are distinct operations. Dropping local
futures cannot promise completed remote or asynchronous cleanup, and neither
operation rolls back effects. Under `D-initial-cancel-invalidation`, a cancelled or abandoned started turn
makes its session unusable for further prompts in the initial API. Recovery is
deferred; starting another conversation must be explicit. Method spelling, error
representation, and detailed cleanup contracts remain to be designed.

## Per-turn result types

Under `R-per-turn-result-types`, result typing belongs to a plan or execution,
not to the lifetime of a conversation. Successive turns in one session can return
different Rust types while retaining the same conversation context. The fixed
session tool catalog must accommodate this without requiring a fixed session-wide
result type. The [typed-conversation design](typed-conversations.md) specifies initial
transport, schema validation, and result-completion semantics.

## Connection and session vocabulary

Under `D-connection-vocabulary`, connect establishes an agent connection and
open creates a conversation session. Built-in presets provide an inherent default
shorthand such as `Claude::connect()`, while configured instances use the
extension-trait method, such as `Claude::default().connect()`. Both delegate to
the same connection machinery. Trait names, ownership/embedding types, and
behavioral connector contracts remain to be designed.


## Lifecycle direction

Under `D-reviewed-api-baseline`, ordinary use relies on RAII rather than mandatory
explicit cleanup or scoped callbacks. Destruction releases local callback access
and initiates teardown through owned connection infrastructure. Confirmed graceful
shutdown remains an optional awaited operation. Owned transport/process cleanup
must not retain borrowed application callbacks or acquire shutdown rights over a
host-owned connection. The lifecycle mechanism and exact guarantees remain
provisional and require implementation evidence; synchronous destruction does not
imply remote acknowledgment or rollback of earlier effects.
