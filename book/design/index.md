# Design

Thinkwell lets an application alternate deterministic Rust computation with
LLM-powered reasoning. Application code owns the workflow: it supplies context,
chooses a result type for each request, exposes Rust functions as tools when useful,
and consumes the answer as a Rust value. Thinkwell supplies the connection,
conversation, typed-value boundary, and execution lifetime around those requests.

`R-easy-agent-programming`, `R-progressive-control`, and `R-code-prompt-ergonomics`
govern the short path from a prompt to a typed answer and the ability to introduce
explicit sessions, tools, model choices, limits, and observation as needed.
`R-idiomatic-reimplementation` governs the Rust ownership model. These chapters
define the library independently of any migration source.

[Example applications](examples.md) define the scheduler, sentiment tool, and
unminifier as consumers of that library. [User documentation](documentation.md)
defines the learning site, its relationship to Rustdoc, and executable snippet
validation. Their roadmaps retain approval and delivery evidence, rather than
serving as separate system definitions.

## Components and responsibilities

| Component | Responsibility | Ownership boundary |
| --- | --- | --- |
| Application | Compose prompts and deterministic work; interpret answers and tool effects | Owns application data and polls execution |
| Plan and scoped execution | Hold prompt configuration, validate a turn, dispatch local callbacks, and return its typed answer | Own captures; an explicit execution exclusively borrows its Session |
| Session | Keep conversation identity and the fixed tool catalog across turns | Borrows Agent; retains descriptors, never a previous turn's callbacks |
| Agent and owned driver | Initialize and drive ACP, route control messages, and clean up the owned connection/process | Background work contains owned data, never borrowed application callbacks |
| Session tool endpoint | Advertise the fixed MCP catalog and route matching calls through owned request/reply messages | Per-session credential and per-turn revocable route |
| Contract boundary | Derive supported schemas, validate JSON values, and convert Rust values | Shared rules for tool input/output and per-turn results |
| Optional observation | Deliver bounded progress to a local borrowed callback | Control and typed completion remain independent of progress delivery |

The application polls its execution on its own task. The ACP driver and HTTP
endpoint exchange owned messages with that execution; the execution alone invokes
borrowed Rust callbacks. This separation permits non-Send captures without moving
those captures into transport tasks.

```text
Application task                         Owned I/O                 Agent program
  Plan / active execution
    +-- borrowed Rust callbacks
    +-- optional borrowed observer
    |
    +<-- owned control/result messages --> ACP driver <--- ACP ---> agent
    |                                       +-- process cleanup
    +<-- owned tool requests/replies ----> HTTP endpoint <-- MCP --> same agent
```

A connection may host one open session at a time. Successful turns preserve that
session's history while releasing their callback captures. Independent connections
provide concurrency. The library does not supply an application workflow language
or decide how to schedule independent model requests.

## ACP client foundation

Under `D-acp-client`, Thinkwell acts as a headless ACP client, rather than choosing
model-provider APIs directly. [Agent integration](agent-integration.md) defines
connection setup, presets, optional model selection, HTTP tools, permissions, and
adapter failure handling. Speaking ACP alone does not establish that an agent
supports every required capability.

## Type-derived contracts

Under `D-derived-contracts`, ordinary Rust derives supply contracts without an
extra schema-generation build tool. [Contracts and typed results](contracts.md)
defines supported schemas, serialization direction, runtime validation, and the
stable submission mechanism for changing result types.

## Scoped turns and explicit conversations

Under `D-provisional-scoped-turns`, configuration, conversation history, and active
callback lifetimes are separate. [Typed conversations](typed-conversations.md)
defines the caller's programming model and binding rules. A descriptor can survive
between turns without retaining a callback's borrow.

## Per-turn result types

Under `R-per-turn-result-types`, result typing belongs to the Plan and its execution,
not to the Session. A conversation can return a record, then an enum or String.
The [completion protocol](contracts.md#result-submission-and-completion) requires
both a validated candidate and a normally completed turn.

## Connection and session vocabulary

Under `D-connection-vocabulary`, connect establishes an agent connection and open
creates a session. Default preset shorthand and configured instances use the same
connection machinery. [Agent integration](agent-integration.md#configuration-and-presets)
defines this extension boundary.

## Lifecycle direction

Under `D-reviewed-api-baseline`, ordinary use relies on RAII; awaited shutdown is
optional confirmation. Under `D-initial-cancel-invalidation`, cancelled or abandoned
started turns invalidate their session. [Execution lifecycle](execution-lifecycle.md)
defines the conservative connection failure policy, callback release, and bounded
owned-resource cleanup. [Progress observation](progress-observation.md) adds local
visibility without changing the authority of the awaited result.

## Supported scope and deferrals

`D-initial-conversation-scope` selects Tokio, owned subprocess connections, fixed
catalogs, a finite schema subset, and macOS/Linux. The configuration and protocol
versions for this scope belong to [agent integration](agent-integration.md); the
schema boundary belongs to [contracts](contracts.md). The library is unpublished;
this design does not establish a release or an MSRV below Rust 1.100.0.

Shared host-owned connections, concurrent sessions on one connection, dynamic
catalogs, explicit cancellation handles, cancellation recovery, independent progress
streams, broader JSON Schema support, and Windows support are outside this scope.
The wider provisional embedding direction does not grant a wrapper shutdown rights
over a host's connection. No concrete attachment/host-routing contract is specified
here. These deferrals do not imply permanent exclusions or automatic authorization
to implement the earlier exploratory API sketches.
