# Agent integration

Thinkwell is a headless ACP client under `D-acp-client`. It owns a subprocess
connection and exposes application tools over HTTP MCP. `D-initial-conversation-scope`
bounds this integration; `R-provider-breadth` motivates extensible agent configuration
without implying every ACP agent supports the same capabilities.

> **`A-turn-envelope-routing`** (accepted): **Submitted turn identifiers control callback routing**
> Property: Missing, malformed, and nonmatching identifiers cannot select or invoke current callbacks. Revoked queues cannot reach replacement callbacks. This authenticates no causal origin when a sender supplies the current identifier.
> Authority: `D-provisional-scoped-turns`
> - Criterion `behavior` [test]: Deterministic HTTP tests inject stale and malformed calls during a replacement turn, assert zero mutation, and verify valid calls and revoked queues.

> **`A-session-endpoint-isolation`** (draft): **Each session controls access to its tool endpoint**
> Property: The tool endpoint is loopback-only, requires the current session's bearer credential, rejects browser Origin headers, and limits request bodies to 256 KiB. Replacement sessions receive fresh credentials; dropping a session revokes its endpoint. These controls do not authenticate a call's originating prompt or bound all concurrent senders.
> Authority: `D-initial-conversation-scope`
> - Criterion `loopback-binding` [test]: Endpoint addresses bind only loopback interfaces; session setup exposes the reachable local endpoint.
> - Criterion `credential-isolation` [test]: Missing, wrong, and prior-session credentials cannot invoke tools in the active session; current credentials permit a valid call.
> - Criterion `origin-denial` [test]: Requests carrying browser Origin headers are rejected before callback invocation, including requests with valid credentials.
> - Criterion `body-limit` [test]: Independently specified requests at the 256-KiB boundary and above it demonstrate the accepted limit and rejection before tool invocation.
> - Criterion `endpoint-revocation` [test]: Dropping a session closes its listener and prevents its credentials or queued endpoint work from authorizing a replacement session.

> **`A-declared-tool-permissions`** (draft): **Permission grants stay within the active declared-tool scope**
> Property: Thinkwell grants only AllowOnce for declared tools in the active session and turn, correlating supported direct names or earlier tool-call IDs. Unknown, conflicting, unrelated, and terminal identities cannot obtain a grant. Correlation holds at most 1024 IDs; overflow disables grants for that turn. Completion and teardown discard correlation state. Agent-provided metadata supplies neither authenticated tool provenance nor an OS sandbox.
> Authority: `D-initial-conversation-scope`; `R-provider-breadth`
> - Criterion `allowed-identities` [test]: Direct names and notification-correlated IDs in the supported Claude and Codex naming forms obtain only AllowOnce for active declared tools, including result submission.
> - Criterion `identity-rejection` [test]: Unknown, unrelated, conflicting, terminal, wrong-session, and wrong-turn identities are denied; partial updates cannot restore an invalid identity.
> - Criterion `correlation-limit` [test]: Independently constructed sequences exercise 1024 distinct IDs and overflow, showing that overflow disables grants without evicting an old identity to permit reuse.
> - Criterion `correlation-reset` [test]: Turn completion and connection teardown discard correlation state; stale IDs cannot authorize a later turn or replacement session.

> **`A-explicit-model-selection`** (draft): **Requested model selection succeeds or fails explicitly**
> Property: Without an override, a session uses the agent's default. An explicit model choice is applied before prompting; rejection returns UnsupportedModel instead of silently using another model. This does not promise portable model identifiers, discovery, or mid-conversation switching.
> Authority: `R-progressive-control`; `R-provider-breadth`
> - Criterion `default-selection` [test]: Opening without a model override leaves selection to the agent and permits the subsequent prompt.
> - Criterion `requested-selection` [test]: An accepted explicit choice is applied before prompting; a rejected choice produces UnsupportedModel and sends no prompt with an unintended fallback.

> **`A-session-startup-failure-reporting`** (draft): **Reported adapter startup failures affect only their owning session**
> Property: A reported Codex failure of the reserved Thinkwell MCP server produces Configuration during setup or a later turn, including while an ACP request is pending. Old-session reports cannot fail a replacement session. Before session identity is known, at most 64 distinct reported IDs are retained; overflow fails setup closed. Selecting or replacing a session and teardown clear the applicable correlation state.
> Authority: `D-initial-conversation-scope`; `R-provider-breadth`
> - Criterion `session-correlation` [test]: Current-session failures prevent further prompting; reports for prior or unrelated sessions cannot poison the selected session.
> - Criterion `pending-failure` [test]: Reports arriving during setup, model selection, an idle session, and a pending turn produce Configuration with the owning cleanup behavior instead of waiting for an unrelated timeout.
> - Criterion `report-capacity` [test]: Independent 64-ID and overflow sequences exercise bounded setup correlation, fail-closed overflow, and clearing on session selection, replacement, or teardown.

## Configuration and presets

`AgentConfig` holds owned command, arguments, environment overrides, and setup/shutdown
bounds. `Agent::connect(config)` launches and initializes the ACP connection; it does
not open a conversation or send a prompt. Arguments are passed directly rather than
through a shell, and overrides supplement the child environment.

Command presets convert into AgentConfig. The blanket Connect extension trait
provides instance `.connect()`; built-in presets also have the inherent default
shorthand `Claude::connect()` or `Codex::connect()`, as selected by
`D-connection-vocabulary`. A downstream type can supply its own conversion without
a registry or closed provider enum. This is configuration extensibility, not a
behavioral connector or host-attachment interface.

The built-in `thinkwell::agent` presets use offline npx with cached adapters:

| Preset | Adapter | Default native-tool mode |
| --- | --- | --- |
| Claude | @zed-industries/claude-agent-acp 0.23.1 | Adapter behavior; client grants remain restricted below |
| Codex | @agentclientprotocol/codex-acp 2.8.0 | Adapter read-only mode |

A configured preset replaces its entire launch configuration, including Codex's
mode setting. Installation and authentication remain external. An adapter pin does
not freeze its compatible-range dependencies or underlying model revision.

Use official Rust ACP SDK 5.4.0, ACP v1 over the subprocess's standard I/O, and HTTP
MCP 2025-11-25. Tokio with I/O/time enabled, macOS/Linux, and Rust 1.100.0 define the
supported environment. Connection initialization alone does not establish tool
transport, authentication, or optional model capabilities needed for a later request.

## Session setup and model selection

Opening a session validates its descriptors and options, reserves the connection,
starts its loopback endpoint with fresh credentials, and supplies the fixed MCP
catalog endpoint in ACP session creation. If requested, model selection occurs
before prompting. Setup failures after remote work starts require owned cleanup;
[execution lifecycle](execution-lifecycle.md) defines partial-construction behavior.

Without a model override, the agent chooses its default. Explicit builders provide
optional model selection; one-shot run uses the default. A rejected explicit
selection returns UnsupportedModel rather than silently selecting another model.
Model identifiers belong to the agent. This scope promises no portable global
model list, discovery API, or mid-conversation model switching.

## HTTP endpoint and turn routing

The endpoint binds loopback and checks a per-session bearer credential, rejects
browser Origin headers, and limits request bodies to 256 KiB. A normally dropped
session revokes its endpoint. Replacement sessions get fresh credentials, so an
old session's credential cannot authorize its replacement. This is a bounded tool
transport, not a claim of every MCP transport or protocol feature.

### Turn routing

Every turn installs a route with a fresh identifier and its own callback queue.
The endpoint checks a submitted string identifier and declared tool name before
selecting that queue. Missing, malformed, nonmatching identifiers and unknown names
fail before application mutation. Rejection does not substitute or disclose the
current identifier. The internal result tool uses the same check.

Once selected, a queue remains associated with that execution. Revoking the route
and ending execution closes access to its callbacks; an old queue cannot reach
replacement callbacks. Requests using the active identifier can invoke the current
bindings, with schema validation handled by [contracts](contracts.md).

Identifiers are correlation, not authentication of causal origin. A delayed operation
that learns and submits the new identifier is indistinguishable from new work.
Likewise, a bounded queue does not globally bound concurrent HTTP senders. Endpoint
waits are bounded using the active turn's configured timeout, but an HTTP disconnect
does not promise immediate cancellation of an application invocation.

## Permission requests

ACP permission handling and HTTP authorization are separate checks. Grant AllowOnce
only for the active session's declared tools, including the internal result tool.
An agent can name the tool directly in a permission request or refer to an earlier
notification by tool-call ID. Correlation is scoped to the active session and prompt,
recognizing the Claude `mcp__thinkwell__<tool>` and Codex `mcp.thinkwell.<tool>` forms.

Unknown, conflicting, unrelated, and terminal identities are denied. Partial
updates cannot invent or restore an invalid identity. Track at most 1024 call IDs;
further new IDs disable grants for that turn rather than evicting old identities
and permitting reuse. Completion or connection teardown discards correlation state.

Permission metadata is supplied by the agent. It is not authenticated evidence that
a particular originating tool ran. Agent-owned native reads can happen without ACP
permission requests; user configuration and other MCP servers may still apply.
Read-only mode and Thinkwell's grants are not an operating-system sandbox.

## Adapter startup failures

Reserve the MCP server name thinkwell for this library. A reported Codex Thinkwell-MCP
startup failure becomes Configuration during session setup or a later turn,
including while an ACP request is pending. Reports are correlated by session ID;
old-session reports cannot fail a replacement conversation.

Before the new session ID is known, retain at most 64 distinct reported IDs; excess
reports fail setup closed. Selecting the session clears the temporary set and
retains only its failure state until replacement or connection teardown. A known
failure is checked before publishing another prompt. A later report can interrupt
an already pending request under the same cleanup rules. This diagnoses the adapter's
reported failure; it does not detect every possible external configuration problem.
