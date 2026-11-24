# Progress observation

A caller can observe activity while awaiting its typed result. `R-progressive-control`
governs optional observation; the observer remains local to the scoped execution.

> **`A-progress-observation`** (accepted): **Optional observation preserves typed execution**
> Property: Observers execute locally within the polled execution and can borrow non-Send state. Progress is optional, bounded and best-effort, with explicit loss reporting. Permission decisions, validation, tool replies and authoritative typed results do not depend on progress delivery. Dropping observation revokes its queue; observer unwinding initiates owned-connection cleanup. Agent reports are not authenticated originating-turn or callback-execution evidence.
> Authority: `R-progressive-control`
> - Criterion `behavior` [test]: Compiler and runtime tests cover borrowed observers/tools, unchanged typed success/failure, queue limits and loss, polling fairness, callback panic/drop, capture release, and route isolation.

## Observer and event contract

Plan::observe attaches one synchronous FnMut(&ProgressEvent) observer after plan
construction. The non-cloneable ObservedPlan wrapper supplies run and run_in;
callers clone unobserved base plans before adding captures. Execution remains
lazy and observers are never stored in transport/background tasks. They may
borrow local non-Send data separately from tool captures. Returned/dropped owned
futures release both sets of captures; pinned storage still needs to be dropped.

ProgressEvent and tool status/outcome enums are non_exhaustive. Payloads have
private fields and public accessors, permitting later additions. Events include
user-visible Message chunks, agent-reported AgentTool announcements/partial updates,
local ToolStarted/ToolFinished invocation processing, ResultRejected/ResultAccepted,
and EventsDropped. Local tool processing includes input validation, so a failed
invocation need not have entered the application callback. Agent tools are not
host execution facts; their IDs need not match local invocation IDs.

Progress omits reasoning text, prompts, raw arguments/results, HTTP credentials
and arbitrary error messages. Result rejections use a category-level reason and
submission number; structured schema-path diagnostics are not exposed in this
first version. Agent text/titles can themselves contain application data. Internal
result-tool announcements are hidden; explicit result events describe validation.
A validated candidate still needs normal terminal completion and settled callbacks.

The queue holds at most 128 entries and 64 KiB of event payload (including fixed
entry sizes and owned string lengths); overflow/oversized updates increment a
saturating loss counter. Loss is reported when the retained queue drains. Internal
tool-filter metadata is separately bounded to 128 IDs of at most 512 bytes; exhaustion
omits subsequent agent-tool reports with loss accounting. This does not bound the
agent transport's own buffers or application-owned logs. Unobserved runs create
no event queue or event formatting work.

The local observer driver polls execution before progress and yields after batches
of 16 events. Control traffic never waits for progress space. On normal return,
it closes the queue and delivers the bounded retained tail/loss report. Drop
revokes the queue without observer calls. Slow synchronous callbacks delay polling
and cannot be preempted by an async deadline. Observer panics unwind normally;
an observation guard also closes the connection if unwinding occurs while draining
an already-completed execution. Callback effects are not rolled back.

Temporary-session run includes setup in the observer's lifetime; known-session
notifications during remaining setup may be observed. Notifications before the
session ID is known are not attributed to this observer. Explicit open precedes
run_in and is outside observation. Session IDs filter notifications and each run
gets a distinct queue, but delayed same-session ACP messages cannot be assigned
an authenticated originating-turn identity. Connection-wide observation, streams,
async/fallible observer protocols and cancellation handles remain deferred.

## Relationship to execution

[Typed conversations](typed-conversations.md) defines plan construction and capture
ownership. [Execution lifecycle](execution-lifecycle.md) owns cancellation and cleanup;
[contracts](contracts.md#result-submission-and-completion) defines when a candidate
becomes a successful answer. An observer neither authorizes tools nor completes a
turn, and agent progress cannot replace those controls.
