# Execution lifecycle

Local callback access, remote turn completion, and owned-resource cleanup have
different lifetimes. Ordinary use relies on RAII under `D-reviewed-api-baseline`;
`D-initial-cancel-invalidation` selects explicit reconnection after cancellation.
The bounded owned-connection scope is governed by `D-initial-conversation-scope`.

> **`A-cancelled-session-invalidation`** (accepted): **Cancelled sessions cannot resume**
> Property: Cancelling or abandoning a started turn permanently invalidates that session and its dedicated connection; no further prompt or implicit history reset occurs. Dropping an unpolled execution sends no work.
> Authority: `D-initial-cancel-invalidation`
> - Criterion `behavior` [test]: Tests distinguish unpolled and started drops, reject reuse, and inspect peer prompt counts.

> **`A-owned-resource-cleanup`** (accepted): **Owned resources have bounded cleanup**
> Property: Drop revokes local access and signals existing cleanup infrastructure without async waiting or spawning. Awaited shutdown reports a bounded outcome; owned-process guards force termination on driver/runtime loss on supported macOS and Linux hosts. No detached-descendant or asynchronous application-destructor guarantee is made.
> Authority: `D-reviewed-api-baseline`
> - Criterion `behavior` [test]: Instrumented child-process tests observe exit/reaping and listener closure for success, constructor failure, early errors, drops, awaited shutdown and runtime teardown on both supported runtime flavors.

## Ownership and execution

The Agent owns connection control and cleanup infrastructure. A Session borrows
that Agent and reserves its single conversation slot. The Session retains a fixed
catalog and endpoint; an execution owns the per-turn callbacks and exclusively
borrows the Session. Transport tasks carry only owned requests, replies, and
control state. They cannot retain application callback borrows after execution ends.

The local execution polls tool invocations serially while awaiting ACP terminal
completion. Routes are bound to that execution and revoked on completion or drop.
A sender holding a revoked queue cannot dispatch through a replacement route.
Routing and submitted-identifier limitations belong to [agent integration](agent-integration.md#turn-routing).

Use an existing Tokio runtime with I/O and time enabled, on current-thread or
multithread runtimes. Non-Send captures stay on the calling task. A synchronous
callback or observer that does not yield delays that task; asynchronous deadlines
cannot preempt synchronous application code.

## Setup and session state

The following are conceptual states, not additional public state-machine types.

| Operation or event | Resulting state and effect |
| --- | --- |
| Construct a plan, session builder, or unpolled execution | No remote work; dropping it does not invalidate a usable connection |
| Local preflight error | No prompt; an otherwise usable session remains available |
| Open a session | Reserve the connection, create its endpoint, request a remote session and any selected model |
| Cancel/fail setup after remote work starts | Release local resources and invalidate the connection; partial construction has cleanup responsibility |
| Start a turn | Install its route and callbacks; the session cannot admit another execution |
| Valid candidate, normal end, and settled callbacks | Return T, revoke the route, and leave the explicit session usable |
| Failed or abandoned started turn | Revoke callback access and permanently invalidate session and connection |
| Drop a normally idle session | Revoke its endpoint and release the reservation; another session starts fresh history |
| Drop the Agent | Signal existing owned cleanup infrastructure |

Every failed started turn closes its dedicated connection, including local budget
exhaustion, missing results, abnormal remote completion, and transport loss.
A callback error that the agent can handle is not itself a failed turn. The initial
API has no automatic recovery, history reset, or explicit cancellation handle.
Cancellation by dropping execution has no awaited outcome; a remote cancelled stop
returns an error. Neither undoes earlier application effects.

## Limits and failures

`RunOptions` configures nonzero bounds for one turn. Connection configuration
separately supplies setup and shutdown bounds. Durations must fit the host clock;
invalid settings are rejected before starting the relevant work. No fixed limit
caps the entire Agent lifetime.

| Bound | Default | Scope and consequence |
| --- | --- | --- |
| Setup | 30 seconds | Connection/session setup; failure after work starts requires cleanup |
| Turn | 120 seconds | Awaiting the typed answer and local tool work; expiry fails the turn |
| Callback | 30 seconds | One invocation including input/output validation; expiry reports a tool error |
| Application calls | 32 | Calls admitted to local dispatch, including invalid input and callback errors; the next call ends the turn |
| Invalid result submissions | 3 | Invalid values/conversions, including missing value; the third ends the turn, allowing two corrections |
| Shutdown | 12 seconds | Awaited confirmation and bounded HTTP cleanup; expiry is not success |

Stale identifiers and unknown tools rejected before routing consume neither call
nor result budgets. Result submission has its own budget; duplicates after acceptance
do not replace the candidate or consume the invalid-result budget. Time limits still
apply. HTTP body/route bounds are specified in [agent integration](agent-integration.md),
and observation has [separate budgets](progress-observation.md).

Errors have categories callers can match without parsing diagnostic messages:
configuration/contract/binding errors describe local setup or preflight problems;
Busy and Closed describe availability; UnsupportedModel describes selection failure;
transport loss, missing result, exhausted call/result budgets, deadline, remote
cancellation or abnormal stop, and terminal/callback overlap explain failed turns.
A callback failure or deadline is reported to the agent as a tool error until a
turn-level failure occurs. A terminal response arriving during a local invocation
is CallbackOverlap, not successful settlement. The public categories are
non-exhaustive, so callers retain a fallback.

## Automatic cleanup and confirmation

Drop releases local captures, revokes routes, and signals infrastructure that is
already running. Destructors do not block on an async runtime or spawn a fresh
cleanup task. While it can be polled, the owned driver performs bounded transport,
process, and listener cleanup. Owned-process guards provide forced termination
fallback when the driver or runtime disappears on supported macOS/Linux hosts.
The cleanup obligation covers directly owned resources, not detached descendants.

Per-turn and per-call bounds do not imply a global concurrent-request or lifetime
memory bound for an Agent used for arbitrarily many sequential sessions.

Drop initiates cleanup; it cannot return an asynchronous cleanup error or establish
that the remote agent has acknowledged termination before the destructor returns.
Caller code requiring confirmation drops its sessions, then awaits
`Agent::shutdown()`. Shutdown consumes Agent and reports a bounded outcome. It is
not needed to finish a successful typed request. Dropping a polled shutdown future
stops waiting while cleanup remains requested.

`Agent::diagnostics()` supplies an independently owned handle to the cleanup outcome
without keeping the Agent alive. `Diagnostics::outcome()` is an observation, not a
wait: absent outcome means no confirmation. Runtime teardown can prevent an outcome
from being recorded. Cleanup failure must not be silently represented as success.

There is no rollback of callback effects, asynchronous destructor guarantee for
application data, guarantee after abrupt host-process death, or grant of shutdown
rights over a host-owned connection. The latter ownership mode requires a separate
embedding contract and is outside this design's supported scope.
