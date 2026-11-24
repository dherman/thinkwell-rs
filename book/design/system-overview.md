# System overview

Thinkwell lets a Rust application combine deterministic computation with agent
reasoning. The application owns the workflow: it supplies context, chooses the
result type for each request, exposes local functions as tools when useful, and
uses the resulting value in its next computation. Thinkwell supplies the typed
conversation and its execution lifetime. An external ACP agent supplies model
access and its own agent behavior.

This overview defines how those parts form one system. The linked chapters own
the detailed contracts and anchors; implementation roadmaps retain delivery and
verification evidence. `R-easy-agent-programming`, `R-code-prompt-ergonomics`,
`R-progressive-control`, and `R-idiomatic-reimplementation` govern this division.

## System context

The supported runtime has an application process and an owned agent subprocess
connection on the same host. Within the application process, Thinkwell separates
locally polled execution from owned transport work. The agent communicates with
its model provider using its own integration; Thinkwell does not implement that
provider protocol or control the model service.

```text
Application process (Rust + Tokio)          External agent environment
+--------------------------------------+   +----------------------------+
| Application workflow and data        |   | ACP agent / adapter        |
|   |                                  |   |                            |
|   v                                  |   | Session history, model     |
| Thinkwell Plan / Session / execution |   | access, native operations  |
|   |                     |            |   |                            |
|   +--> local Rust tools |            |   +----------------------------+
|   +--> local observer   |            |       ^       ^          |
|                         v            |       |       |          v
| Owned ACP driver <-------------------------->+       |     Model provider
|   connection and process lifetime    |   ACP/stdio  |     (agent-managed)
|                                      |               |
| Session HTTP endpoint <----------------------------->+
|   owned tool requests/replies        |     MCP/HTTP on loopback
+--------------------------------------+
```

ACP carries initialization, session creation, prompts, terminal responses,
permission requests, and progress notifications. MCP over the session's local
HTTP endpoint carries application tool calls, tool responses, and structured
result submission. The driver and endpoint exchange owned messages with the
execution. Only that execution invokes callbacks that borrow application data.
A tool callback stays in the application process; describing it to the agent does
not transfer the Rust closure or its memory to the subprocess.

Agent installation, authentication, configuration, and model availability are
external prerequisites. The built-in Claude and Codex presets select launch
configurations for specific adapters; custom command configurations use the same
connection machinery. Compatibility requires the capabilities used by a request,
not merely a successful ACP handshake. [Agent integration](agent-integration.md)
owns supported versions, configuration, permissions, and adapter failure handling.

## Components and responsibilities

| Component | Responsibility and interface | Ownership boundary |
| --- | --- | --- |
| Application | Compose requests, validate domain meaning, perform deterministic work, and decide the next step | Owns workflow, data, and polling of execution |
| Plan and scoped execution | Hold prompt configuration and the current result contract; preflight, dispatch local tools, and return the typed result | Own captures; an explicit execution exclusively borrows its Session |
| Session | Associate remote conversation identity with a fixed tool catalog across successful turns | Borrows Agent; retains descriptors, never a previous turn's callbacks |
| Agent and owned driver | Initialize and drive ACP, handle control traffic, and clean up the owned connection/process | Background work contains owned state, never borrowed application callbacks |
| Session tool endpoint | Expose the MCP catalog and route tool/result calls to the active execution | Per-session credential and per-turn revocable route |
| Contract boundary | Derive schemas, validate supported JSON values, and convert between wire values and Rust types | Validates tool input, serialized output, and final answers separately |
| Optional observation | Report bounded progress to a local borrowed callback | Delivery is independent of control and authoritative completion |

The application and transport responsibilities are deliberately separate. This
allows borrowed, non-Send callbacks without requiring them to survive in a
background task. It also means a slow synchronous callback can delay application
polling; a timeout cannot preempt arbitrary synchronous Rust code.

## Rust ownership and runtime fit

The lifetime structure follows the useful application scopes. A Session borrows
its Agent; an explicit execution exclusively borrows that Session and owns the
closures supplied for the turn. Those closures can themselves borrow application
state. The compiler therefore prevents overlapping session executions and premature
access to exclusively captured data. After execution storage is consumed or
dropped, application code can use that data and supply new bindings for another
turn. Conversation identity does not require session-long callback borrows.

The transport boundary carries owned JSON values, descriptors, identifiers, and
reply channels. Deserialization and callback execution happen locally. Detached
I/O tasks consequently need no access to the caller's borrowed closures, and those
closures need neither Send nor a static lifetime. An execution with non-Send
captures must be awaited on a task that can hold them; a multithread Tokio runtime
does not make that future eligible for `tokio::spawn`. The library supports local
polling on either supported Tokio runtime flavor.

Rust Drop supplies immediate local revocation and signals infrastructure created
during connection/setup. Existing owned tasks and process guards handle cleanup;
the destructor itself does not await network replies. Applications needing a
confirmed outcome can await shutdown. This preserves ordinary scope-based use
without giving synchronous destruction an asynchronous completion guarantee.

These choices trade automatic cross-task scheduling and shared-connection
concurrency for direct access to local Rust state and explicit ownership. The
application must keep polling its execution, and independent concurrent
conversations use independent connections. Compiler rejection, runtime borrowing,
and process cleanup are separate verification obligations in the owning anchors.

## From request to answer

A request with a local tool follows this path; requests without application tools
still use the internal result-submission tool.

1. **Prepare the connection and conversation.** Connect launches and initializes
   the ACP agent. Opening a session reserves the connection, validates its
   descriptors/options, creates its loopback endpoint and credential, and supplies
   that endpoint during ACP session creation. An explicit model selection must
   succeed before prompting. One-shot execution performs this setup for its
   temporary session after local plan preflight.
2. **Start the scoped turn.** The application supplies a Plan with prompt context,
   a result type, limits, and this turn's tool bindings. Local preflight rejects
   unsupported contracts or mismatched bindings before a prompt is sent. Execution
   installs a fresh route and sends the prompt with its identifier and result
   schema through the ACP driver. The agent handles reasoning and provider access.
3. **Run application tools.** When the agent makes an MCP tool call, the HTTP
   endpoint checks session authorization and the submitted turn identifier before
   routing. The local execution validates and deserializes the input, invokes the
   callback, then serializes and validates its output. An owned reply returns over
   HTTP. Tool errors can be corrected within the configured execution budgets.
4. **Accept a candidate.** The agent invokes the internal `return_result` tool with
   the active identifier and a value. The execution validates that turn's contract
   and deserializes its Rust result type. The first valid candidate is retained;
   rejection permits bounded correction, and duplicates cannot replace it.
5. **Finish the turn.** A candidate becomes `Ok(T)` only after normal ACP terminal
   completion with settled local callbacks. Missing results, abnormal completion,
   transport loss, deadline expiry, or terminal/callback overlap fail the turn.
   Progress text and candidate-acceptance events cannot substitute for this result.
6. **Continue ordinary Rust work.** Consuming or dropping the execution releases
   its captures. A successful explicit session retains conversation history and
   can accept a new Plan with a different result type and fresh bindings. A
   successful one-shot run drops its temporary session before returning. A failed
   or abandoned started turn invalidates its session and dedicated connection;
   prior application effects remain.

The [contract protocol](contracts.md#result-submission-and-completion) owns the
candidate/terminal rules, [agent integration](agent-integration.md#turn-routing)
owns routing, and [execution lifecycle](execution-lifecycle.md) owns state changes
and cleanup. These are interacting parts of one turn, not independent success
signals.

## ACP client foundation

`D-acp-client` selects a headless ACP client. Provider breadth comes through agents;
Thinkwell does not select a separate provider SDK for each model. The supported
transport is ACP over subprocess standard I/O plus direct loopback HTTP MCP for
tools. Driver, endpoint, and local execution live in the library; the design has
no separate conductor service or proxy-chain extension contract.

The transport choice fits the owned, same-host connection scope. It does not
provide remote agent hosting, general protocol bridging, or universal MCP
conformance. [Agent integration](agent-integration.md) defines the bounded target.

## Type-derived contracts

Under `D-derived-contracts`, ordinary Rust compilation supplies Serde and Schemars
implementations. At runtime, Thinkwell uses their metadata to construct and check
contracts and advertises the supported wire schemas to the agent. There is no
additional Thinkwell schema-generation command or generated-schema loader.

Derivation describes values; runtime validation enforces the supported subset.
Neither establishes factual accuracy, faithful interpretation of a prompt, or
application-specific constraints. [Contracts and typed results](contracts.md)
owns schema direction, conversion, and finite support under
`A-supported-contract-boundary`.

## Scoped turns and explicit conversations

`D-provisional-scoped-turns` separates configuration, conversation identity, and
active callback lifetimes. Descriptors can survive between turns without retaining
a callback's borrow. One open Session reserves one Agent connection; independent
connections provide concurrency. The application chooses its own workflow and
scheduling in ordinary Rust.

[Typed conversations](typed-conversations.md) owns Plan composition, one-shot and
explicit execution, fixed-catalog binding, and `A-scoped-turn-ownership`.

## Per-turn result types

Under `R-per-turn-result-types`, the result type belongs to the Plan and its
execution. A conversation can return a record, then an enum or String. A stable
result tool carries a generic value while each prompt supplies the current
contract; changing result types does not require a changing session catalog.
[Contracts and typed results](contracts.md) owns `A-typed-turn-completion` and the
separate input, output, and final-result validation boundaries.

## Connection and session vocabulary

Under `D-connection-vocabulary`, connect establishes an agent connection and open
creates a session. Preset shorthand and configured instances use the same path.
Configuration extensibility does not imply attachment to a host-owned connection;
[agent integration](agent-integration.md#configuration-and-presets) owns this boundary.

## Lifecycle direction

`D-reviewed-api-baseline` selects ordinary RAII use with optional awaited shutdown
confirmation. `D-initial-cancel-invalidation` selects an unusable session after a
started turn is cancelled or abandoned. Revoking local access, receiving remote
completion, and finishing owned-resource cleanup are separate events.
[Execution lifecycle](execution-lifecycle.md) owns `A-cancelled-session-invalidation`
and `A-owned-resource-cleanup`, including bounded outcomes and supported-host limits.

[Progress observation](progress-observation.md) owns `A-progress-observation`.
An observer may borrow local state, receives bounded best-effort updates, and is
not required for typed completion. Agent reports are not authenticated evidence
that an application callback ran. The observation path is not a durable audit log.

## Trust and correctness boundaries

The application decides which data enters prompts and which effects tools can
perform. Agent configuration and native operations remain outside Thinkwell's
local callback authorization. Declared-tool permission grants are not an operating
system sandbox, and prompt formatting is not an instruction-trust boundary.

| Boundary | Governing design and anchor relationship |
| --- | --- |
| Caller memory and active callbacks | [Typed conversations](typed-conversations.md): `A-scoped-turn-ownership`; callback captures remain local to their execution |
| Session and turn routing | [Agent integration](agent-integration.md): accepted `A-turn-envelope-routing`; identifiers reject nonmatching calls but do not authenticate originating-turn causality |
| Local endpoint access and agent permissions | [Agent integration](agent-integration.md): draft `A-session-endpoint-isolation` and `A-declared-tool-permissions`; session credentials, loopback exposure, and scoped grants have distinct roles |
| Requested model and adapter readiness | [Agent integration](agent-integration.md): draft `A-explicit-model-selection` and `A-session-startup-failure-reporting`; explicit failures must not silently become a different request |
| Wire values and final success | [Contracts](contracts.md): `A-supported-contract-boundary` and `A-typed-turn-completion`; validation and terminal completion establish a typed answer, not its domain truth |
| Abandonment and resource ownership | [Lifecycle](execution-lifecycle.md): `A-cancelled-session-invalidation` and `A-owned-resource-cleanup`; cleanup does not undo tool effects or cover detached descendants |
| Optional user-visible activity | [Observation](progress-observation.md): `A-progress-observation`; loss of progress cannot authorize tools or complete a turn |

These references reuse the owning anchors, including their recorded acceptance
status; this overview creates no duplicate guarantees. Accepted anchors still
have uncovered criteria, and draft anchors need scope/authority reconciliation
and executable links. The [verification roadmap](../implementation-roadmaps/executable-verification.md)
owns current computed results and coverage limits. Fixed-catalog binding,
execution-budget boundaries, and progress-disclosure restrictions are also
consequential design constraints; their anchor coverage remains incomplete.

## Product and development boundaries

The runtime library is the unpublished `crates/thinkwell` Cargo package. Tokio
supplies async execution and I/O, the official ACP SDK supplies protocol types and
connection machinery, the HTTP stack serves the local endpoint, and
Serde/Schemars/serde_json supply contract metadata and value conversion. Dependency
pins and lockfiles belong to the manifests and installation guidance.

[Example applications](examples.md) are consumers of that public library:

- The scheduler uses typed extraction, deterministic Z3 solving, and another prompt
  for explanation. Solver constraints validate extracted data, not human intent.
- Sentiment analysis lets an agent call a borrowed local analyzer and retains actual
  measurements separately from its generated interpretation.
- The unminifier combines deterministic source discovery, typed naming requests,
  and bounded source edits whose guarantees stop short of general equivalence.

Their separate Cargo packages and lockfiles keep solver and analysis dependencies
out of ordinary library builds. Their application anchors belong to the example
design; they do not strengthen the library's guarantees.

[User documentation](documentation.md) separates Mintlify's task-oriented learning
site, Rustdoc's API reference, and this Book's design/authority/evidence. Cargo
builds the library and Rustdoc; the documentation checks compile tutorial programs
and compare excerpts with repository sources. Anchor Book and mdBook validate and
render project knowledge. `tools/anchor` is a development launcher, never a
Thinkwell runtime component. The [working guide](../working-on-the-project.md)
links the supported local and CI checks. Those checks use deterministic fixtures;
live-agent checks require separate, explicit execution and external credentials.

## Supported scope and open choices

`D-initial-conversation-scope` selects Tokio, owned subprocess connections,
one open fixed-catalog session per connection, finite schemas, and macOS/Linux.
The project remains unpublished and in Porting; Rust 1.100.0 is a development
baseline, not an established MSRV below that version.

Shared host-owned connections, concurrent sessions on one connection, dynamic
catalogs, explicit cancellation handles, cancellation recovery, independent
progress streams, broader JSON Schema support, and Windows support are deferred.
The owned cleanup design confers no shutdown rights over a host's connection.

The working-directory policy, compatibility with custom model-selector IDs,
precise setup/turn deadline composition, and broader MCP negotiation remain
[open design choices](../open-questions.md#approval-bases-requiring-clarification).
Existing implementation observations do not settle them. The system overview
adds no new isolation, end-to-end latency, or universal agent-support promise.
