# Design

Start with the [system overview](system-overview.md) for Thinkwell's purpose,
components, runtime interactions, Rust ownership rationale, and trust boundaries.
The chapters below own the detailed design and correctness anchors. Requirements
and decisions live in [authority](../authority/index.md); roadmaps record work and
evidence rather than supplying competing system definitions.

## Components and responsibilities

| Chapter | What it defines |
| --- | --- |
| [System overview](system-overview.md) | How the application, library, external agent, and model form one system; why the architecture fits Rust |
| [Typed conversations](typed-conversations.md) | Plans, sessions, local callback ownership, prompt composition, and tool binding |
| [Execution lifecycle](execution-lifecycle.md) | State transitions, limits, failure, cancellation, and owned-resource cleanup |
| [Contracts and typed results](contracts.md) | Derived schemas, supported values, tool wire contracts, and final-answer completion |
| [Agent integration](agent-integration.md) | ACP connections, presets, model selection, HTTP MCP, routing, and permissions |
| [Progress observation](progress-observation.md) | Borrowed observers, bounded delivery, loss, and independence from control/results |
| [Example applications](examples.md) | Scheduler, sentiment analysis, and unminifier as public-library consumers |
| [User documentation](documentation.md) | Learning site, Rustdoc, source ownership, and executable tutorial validation |

## ACP client foundation

`D-acp-client` selects Thinkwell as a headless ACP client. Provider compatibility
comes through agents with explicitly supported capabilities.
[Agent integration](agent-integration.md) defines the selected transport and presets.

## Type-derived contracts

`D-derived-contracts` selects ordinary Rust derives without an extra schema build
tool. [Contracts and typed results](contracts.md) defines generation, independent
runtime validation, and the supported finite subset.

## Scoped turns and explicit conversations

`D-provisional-scoped-turns` separates reusable configuration, conversation history,
and active callback lifetimes. [Typed conversations](typed-conversations.md) defines
one-shot execution and explicit sessions with fixed catalogs and per-turn bindings.

## Per-turn result types

`R-per-turn-result-types` permits successive requests in one conversation to return
different Rust types. The [completion protocol](contracts.md#result-submission-and-completion)
requires both a validated candidate and a normally completed turn.

## Connection and session vocabulary

`D-connection-vocabulary` uses connect for agent connections and open for sessions.
[Configuration and presets](agent-integration.md#configuration-and-presets) defines
the common connection path and configuration extension boundary.

## Lifecycle direction

`D-reviewed-api-baseline` selects RAII with optional awaited shutdown confirmation;
`D-initial-cancel-invalidation` makes a cancelled started session unusable.
[Execution lifecycle](execution-lifecycle.md) defines cleanup and failure policy.
[Observation](progress-observation.md) adds visibility independently of typed success.

## Supported scope and deferrals

`D-initial-conversation-scope` bounds the supported library. The
[system overview](system-overview.md#supported-scope-and-open-choices) states the
current environment, deferrals, and remaining approval questions; owning chapters
supply the detailed limits. Passing Book checks or linked tests does not establish
that every anchor criterion is satisfied.
