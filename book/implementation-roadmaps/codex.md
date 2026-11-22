# Codex agent compatibility

Roadmap identifier: `codex`.

The owner selected Codex as the next agent investigation after merging PR #8.
The initial research below used the custom AgentConfig API. The local
implementation now includes the typed Codex preset; integration remains pending. The owner approved the concrete implementation milestone after reviewing these
findings.

## Milestone 1. Codex preset and permission interoperability {#codex-1}

**Identifier:** `codex-1`

**Status:** In progress

**Outcome:** A newcomer can connect with thinkwell::agent::Codex and run the
existing typed conversation and borrowed-tool workflows without a diagnostic
bridge or changes to application callback signatures.

**Dependencies and assignment:** One-shot tools are integrated through PR #8.
The owner assigned the investigation, then approved this concrete milestone with
“yes”. It became Ready and is now In progress.

**Design basis:** `R-easy-agent-programming`, `R-provider-breadth`,
`R-progressive-control`, `D-reviewed-api-baseline`, and the existing
[typed-conversation design](../design/typed-conversations.md). The approved API
continues to use extensible AgentConfig conversions and per-session model choice.
No new public capability or permissions API is proposed.

**Tasks:** Correlate ACP permission requests with preceding tool notifications;
add a typed Codex preset and introductory Rustdocs; exercise model choice and
both teaching examples; document installation, authentication, native-tool
limitations and exact tested adapter versions; run regression checks and review.

### Approved boundaries

Use the maintained `@agentclientprotocol/codex-acp@2.8.0` adapter. Mirror the
Claude preset's offline, pinned npx launch policy and configured-command escape
hatch. Installation and authentication remain external. Prefer read-only mode
for this programmatic preset; verify the effective native-tool behavior before
claiming support. Read-only mode is not a promise that native read operations
request client approval, nor that Thinkwell is an operating-system sandbox.

Recognize declared Thinkwell tools from ACP tool notifications, then resolve a
name-less permission request by its session and tool-call IDs. Keep correlation
state bounded and scoped to the active session/turn. Reject unknown, conflicting,
stale, terminal, and unrelated calls; never approve an arbitrary MCP call merely
because it has a tool-call ID. Preserve direct named requests for Claude and
select AllowOnce only. Application HTTP credentials, turn envelopes and schema
validation remain independent checks. ACP peer-provided metadata is not an
attestation of tool provenance.

### Completion criteria

#### Permission interoperability {#codex-1-permissions}

- **Anchor coverage:** `A-turn-envelope-routing`; `A-owned-resource-cleanup`
  (regression coverage, not stronger origin or sandbox guarantees).
- **Criterion:** Deterministic ACP tests cover name-less requests following tool
  notifications, both tested naming conventions, direct named requests, session
  and turn isolation, conflicts, unknown IDs, bounded state and cleanup. Unrelated
  native tools are not granted permission by Thinkwell.
- **Evidence:** Local unit tests cover both names, partial updates, conflicting
  identities, terminal tombstones, unknown IDs, session/turn isolation and capacity
  exhaustion/reset. The scripted ACP conversation tests notification/request
  ordering, AllowOnce-only selection, wrong sessions, terminal calls and native
  denial while completing two typed turns. Existing envelope and cleanup tests
  pass unchanged. A separate startup-failure fixture verifies Configuration
  before prompting and subprocess cleanup.

#### Public workflow {#codex-1-workflow}

- **Anchor coverage:** `A-typed-turn-completion`; `A-scoped-turn-ownership`.
- **Criterion:** Preset configuration tests and newcomer-oriented Rustdoc examples
  pass. Bounded live runs demonstrate model selection, typed results changing
  between turns, borrowed application tools, sentiment analysis and scheduling
  with the actual preset and no bridge. Document observed native-tool behavior
  and any user configuration interaction.
- **Evidence:** Preset configuration/replacement tests and Rustdocs pass. Direct
  preset execution passed the two-turn acceptance test and both full examples.
  No-prompt model selection passed. Native-read and MCP-name collision probes
  established the documented limits; see the implementation record below.

#### Review and delivery {#codex-1-delivery}

- **Anchor coverage:** None — documentation and delivery.
- **Criterion:** Relevant local regression, Rustdoc and Book checks pass; separate
  review, CI, owner review and integration are recorded for the final revision.
- **Evidence:** Local regression, strict Clippy, Rustdocs, formatting and Book
  checks pass. Author self-review completed separately after implementation.
  The owner authorized draft PR delivery; CI, owner review and integration
  remain pending. The milestone
  remains In progress; local validation is not durable completion.

## Research evidence

Inspected on 2026-11-21 from `43c147c250efd2c5fb1a1d8db57df0b03991c134`, on
branch `codex-acp`, with disposable setup under ignored `target/`.

The old [Zed adapter](https://github.com/zed-industries/codex-acp) redirects new
work to [agentclientprotocol/codex-acp](https://github.com/agentclientprotocol/codex-acp),
which wraps Codex App Server. Installed its npm release with:

```sh
npm install --prefix target/codex-adapter --no-audit --no-fund \
  @agentclientprotocol/codex-acp@2.8.0
```

The local npm lock resolved adapter 2.8.0 and `@openai/codex` 0.170.0. The adapter
uses a compatible-range dependency on Codex, so pinning the adapter alone does
not freeze the underlying Codex version. The adapter declares Apache-2.0; its
README and packaged JavaScript were inspected, with no source copied or adapted.
Global installation/configuration and authentication files were not modified.

### Capabilities and model selection

Reused `experiments/acp-integration/probe_agent.py` by importing it and overriding
COMMAND with the absolute installed `codex-acp` executable. The existing probe
uses a temporary cwd, no prompts, per-request deadlines and process-group cleanup.
Observed ACP protocol v1, adapter identity 2.8.0, HTTP and ACP MCP capabilities,
and a model-category selector with ID `model`. Setting its existing current
value succeeded. The session also advertised reasoning, speed, mode and
collaboration selectors; these are observations, not new Thinkwell API commitments.
The existing Thinkwell model-selection request shape is compatible with this probe.

Packaged `toCodexMcpServerConfig` forwards ACP HTTP headers into Codex
`http_headers`. Session configuration can inherit user MCP servers; an existing
server named thinkwell may cause the client-supplied server to be skipped. This
needs a deliberate failure/diagnostic check before claiming a reliable preset.

### Live typed-tool experiment

Created an independent disposable Cargo package under `target/codex-acceptance`
using this checkout's Thinkwell library and the library's existing `live_check`
example. Replaced only the connection with custom AgentConfig pointing to the
installed adapter, set INITIAL_AGENT_MODE=read-only, and removed Claude's haiku
model override. Used existing authentication and the agent's default model.
Compiled offline; executed with an outer 180-second subprocess deadline and
process-group cleanup. Its separate generated lockfile is not root-lockfile CI
evidence.

Two direct attempts ended with MissingResult. Capturing selected ACP events on
the second attempt identified the cause:

1. Codex announced a tool call titled `mcp.thinkwell.add`, with rawInput identifying
   server thinkwell, tool add, and the expected turn/input arguments.
2. Its permission request contained toolCallId, kind and pending status, but no
   name or title. The name was supplied by the preceding notification.
3. Thinkwell's current permission handler checks only names/titles in that request
   using Claude's `mcp__thinkwell__...` convention. It therefore cancelled the call.
4. Codex reported cancellation and completed without a result; Thinkwell correctly
   returned MissingResult rather than accepting ordinary text as a typed answer.

A disposable stdio proxy correlated (sessionId, toolCallId) with the earlier
notification, limited to server thinkwell and the two tools add/return_result,
and supplied the corresponding name to the existing permission handler. With
this diagnostic translation, the test passed: the first typed Summary contained
3 and label first, Rust doubled its borrowed accumulator, and the second typed
Decision accepted 10 after adding 4. The original assertions and awaited shutdown
passed. HTTP tool discovery, bearer-header forwarding, application dispatch and
typed result submission therefore worked in this configuration.

The proxy was a diagnostic experiment, not a shipped solution: it had no production
bounds, stale-state tests or complete conflict handling. The implementation record
below supersedes the research gaps for the bounded milestone; broader provider
coverage and Linux CI remain separate.

## Local implementation and validation

Tested on macOS/Rust 1.100.0 at base
`43c147c250efd2c5fb1a1d8db57df0b03991c134` plus the uncommitted Codex changes.
No third-party source was copied or adapted, and no Rust dependency was added.
The installed and npx-cached adapter both resolved to 2.8.0 with Codex 0.170.0.

`agent::Codex` mirrors Claude's static connect, configured instance and AgentConfig
conversion. It uses offline npx and INITIAL_AGENT_MODE=read-only. Rustdocs teach
one-shot typed answers, explicit conversations, configuration replacement and
setup. Both tutorial READMEs explain the two-line agent substitution.

The private permissions module keeps declared tool identities and terminal/conflict
tombstones during a prompt. Notifications and permission requests share that
state; completing a prompt clears it. Unknown updates cannot establish identity.
Capacity overflow disables grants for the rest of that turn instead of evicting
tombstones. This does not strengthen submitted-turn provenance or sandbox claims.
The existing HTTP/session cleanup and public callback signatures are unchanged.

Live evidence, each with an outer 180-second timeout and process-group cleanup:

- The independent acceptance runner used Codex::connect directly, no proxy, and
  the existing live_check assertions with Claude's model override removed. It
  passed two typed turns (Summary total 3, Decision total 10), borrowed mutable
  callbacks and awaited shutdown in about 17 seconds.
- Copies of both example packages under target changed only the agent import and
  connect call (plus the copied manifest's library path). Their original lockfiles
  were retained. Built with `cargo build --locked --offline`, sharing the original
  example target directories. Sentiment returned Mixed after all three exact
  passages were scored (approximately 0.848, -0.637, 0.000); scheduler extracted
  the three windows, solved Wednesday 11–12 and produced its second-turn String.
  Runs took about 14 and 17 seconds respectively. The teaching sources still use
  Claude by default.
- The unchanged Claude live_check also passed in about 19 seconds, with haiku.
- A separate read-only ACP probe read a synthetic marker file in its temporary
  working directory using a native read tool, with no permission request. The
  requested new file was not created, but there was no observed write attempt:
  this is not proof of universal write blocking or isolation from user config.
- A no-prompt probe supplied an explicitly disabled user MCP server named
  thinkwell through per-process CODEX_CONFIG. The adapter reported its client
  server was skipped. The final library implementation returned Configuration
  during setup in about 1.4 seconds, before any prompt, with an actionable message.
  Global Codex configuration was not modified. Detection uses this adapter's
  reported startup failure; it cannot detect every arbitrary agent misconfiguration.

The setup recipe is
`npm exec --yes --package=@agentclientprotocol/codex-acp@2.8.0 -- node --version`;
its offline form was checked after caching, and live preset runs used offline npx.
The adapter's --help argument does not provide a terminating installation probe;
that exploratory process was stopped and is not the documented setup command.

Local checks:

- `cargo check --workspace --all-targets --locked`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked` (14 library unit tests, 24 conversation tests,
  34 Rustdoc cases including compile-fail cases, and launcher/bootstrap tests)
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked`
- Both standalone example test suites with locked manifests and their documented
  target/CMake settings: six sentiment and eight scheduler tests passed.
- Python capability-probe regression, formatting, whitespace, Book structural
  check and mdBook rendering.

A separate author self-review examined code, tests, documentation and milestone
criteria. It checked session/turn scoping, fail-closed correlation, lock ordering,
cleanup, configuration replacement and the distinction between client grants and
agent-owned native operations. The MCP collision finding was addressed with an
explicit setup error and deterministic/live coverage. No unresolved implementation
findings were identified. This is author review, not independent or owner review;
Linux CI and durable integration remain pending. No phase transition or publication
is implied.


## PR review follow-up

Copilot's review of [PR #9](https://github.com/dherman/thinkwell-rs/pull/9) at
`b9c176418f0ea1ae6421a946393904acce1d0cda` identified two valid timing gaps:
the connection-wide startup flag could attribute a dropped session's report to
its replacement, and the one-time setup check missed asynchronous reports.
The adapter's v2.8.0 startup documentation confirms reports may follow session/new.

Replaced the flag with session-ID correlation and a watch notification path.
Pre-response reports are limited to 64 distinct IDs and cleared once the returned
session ID is selected; overflow fails setup closed. Subsequently only that
session's failures are retained. Setup/model requests and prompt requests race
against failure notification under the existing deadlines and cleanup rules.
An idle failure is checked before polling the next prompt. No new public API,
startup wait extension, retries or recovery guarantee was introduced.

Regression fixtures exercise an old report before the replacement response,
during model configuration and during its successful turn; current-session
reports during pending model configuration and prompting; and an idle report
confirmed by a protocol round trip before the next turn. The latter sends no
prompt. Error tests require Configuration within three seconds, session
invalidation when applicable and child reaping. The existing immediate failure
case remains covered. A unit test checks bounded pending state and reset.

Validation on the reviewed base plus these local fixes: focused startup tests,
workspace tests/check/strict Clippy, Rustdocs, formatting, whitespace, Book
structure and rendering. Live model runs were not repeated: these deterministic
fixtures force the timing paths the earlier live runs did not cover. Author
follow-up review checked correlation, notification races, bounds and cleanup;
CI and owner review of the revised head remain pending.
