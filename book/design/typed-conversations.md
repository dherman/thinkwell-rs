# Typed conversations

An application composes a Plan, executes it against an agent, and receives a Rust
value. Explicit sessions keep history across turns; tools let a turn call Rust
functions that borrow application state. `D-provisional-scoped-turns`,
`D-reviewed-api-baseline`, and `R-per-turn-result-types` govern this model.

> **`A-scoped-turn-ownership`** (accepted): **Scoped callbacks belong to their execution**
> Property: Only an active execution accesses its bound callbacks; application borrows end when execution is consumed or dropped. Overlapping mutable session executions are rejected.
> Authority: `D-provisional-scoped-turns`
> - Criterion `behavior` [test]: Runtime tests and compiler probes establish non-Send borrowing across await, restored caller access, and rejection of overlapping mutable use.

## Values and lifetimes

| Value | Purpose | What can be reused |
| --- | --- | --- |
| Agent | An owned connection to an ACP agent program | Sequential conversations; independent connections for concurrency |
| Session | Conversation identity and an upfront tool catalog | History and descriptors across successful turns |
| `Tool<Input, Output>` | A name, description, and derived input/output contracts | Descriptors can be cloned or registered in another session; no callback is stored |
| `Plan<Result>` | Prompt text, options, and the result type for one turn | An unbound plan can be cloned before attaching captures |
| Bound plan / execution future | Per-turn callback implementations and their captures | Consumed or dropped to release captures; not cloneable |
| ObservedPlan | A plan plus one local observer | Consumed by run or run_in; not cloneable |

Session borrows Agent, and an explicit execution exclusively borrows Session.
Rust rejects overlapping mutable executions and use of exclusively captured state
while those captures are retained. Application callbacks may be non-Send and hold
local state across await. They are polled locally, not spawned into the owned driver.
Input and result types use `DeserializeOwned` and `JsonSchema`; tool outputs use
`Serialize` and `JsonSchema`. Owned wire values cross the transport boundary;
callback captures need not be static.

Consuming or dropping the plan or its owned execution future releases captures.
Awaiting a pinned mutable reference can leave completed future storage alive; that
storage must also be dropped before reusing captures. Completion of a turn does not
roll back any tool effects.

## Choosing a conversation

For one independent request, `Plan::<T>::new().text(...).run(&agent).await` creates
a temporary session, executes one turn, and drops the session before returning.
Its catalog comes from the plan's bound tools. A successful later run on the same
Agent has fresh history. Temporary sessions use the agent's default model.

For history or a model selection, build and open a Session explicitly, then call
`plan.run_in(&mut session).await`. Successive plans may return different types.
The application can inspect or mutate previously borrowed data between turns,
then bind fresh callbacks for the next turn. It need not replay prior prompts to
retain the conversation.

One Agent permits one open Session at a time. An occupied Agent returns `Busy`;
it does not silently queue a second conversation. Normal session drop releases
that reservation and revokes its endpoint. A replacement gets fresh remote history
and credentials. Failure after setup or a turn has started follows the
[execution lifecycle](execution-lifecycle.md), including connection invalidation.

Building a plan or obtaining an unpolled execution sends no agent work. Polling
run includes temporary-session setup; polling run_in uses an already open session.
Application code remains responsible for driving the future and composing any
independent concurrent work.

## Declaring and binding tools

Declare a `Tool<I, O>` with its name and description, then register its descriptor
with `agent.session().tool(&tool).open().await`. The catalog is fixed when the
session opens. `plan.bind(&tool, callback)` supplies that turn's implementation.
Every declared application tool needs exactly one matching binding on every turn,
even if the agent is unlikely to call it. A later turn never inherits the previous
callback. Matching includes the descriptor's name, description, and contracts.

For one-shot use, `plan.tool(name, description, callback)` constructs and binds the
descriptor together. Both inline tools and named bindings contribute to run's
temporary catalog. With run_in, inline syntax still has to match the existing
catalog; it cannot register a new tool in an open session.

Preflight rejects duplicate or reserved names, invalid descriptors, missing or
mismatched bindings, unsupported contracts, and invalid execution options. Tool
construction may retain an error for open/run to report. The internal name
`return_result` is reserved for [typed results](contracts.md#result-submission-and-completion).
For run, local validation precedes remote session setup. For run_in, validation
precedes prompting and leaves an otherwise usable session available for correction.
The library does not promise a precedence among multiple simultaneous invalidities.

A callback receives its ordinary Rust input, not the transport envelope. Its output
is validated after serialization. Application calls are dispatched serially within
the active execution. Callback errors become tool errors the agent may handle;
the [budget and failure rules](execution-lifecycle.md#limits-and-failures) distinguish
those errors from failures that end the turn.

## Prompt composition

Plans store owned prompt text. Composition consumes and returns a plan, preserving
its other state; cloning an unbound base copies configuration, not history or
mutable callback captures. Attaching bindings makes the plan non-cloneable.

| Method | Composition rule | Immediate failure |
| --- | --- | --- |
| text | Append the supplied text and one newline | None |
| json | Serialize the borrowed value immediately as compact JSON followed by one newline; requires only Serialize | Serialization failure returns InvalidValue |
| code(source, language) | Copy source into a Markdown code block; source argument comes first | A label containing backticks or control characters returns InvalidValue |

json retains no borrow of its input and adds no special content block. code uses
at least three backticks and a fence longer than every backtick run in source.
It preserves source bytes, adds a newline before the closing fence if missing,
and ends the block with a newline. An empty language label is allowed. Neither
method starts agent work. These are formatting conveniences, not syntax validation,
a quoting protocol, or an instruction-trust boundary.

`options` replaces the execution limits. Attach `observe` last, after composition
and bindings; the resulting wrapper supplies run and run_in. Clone an unobserved
base before adding an observer. [Progress observation](progress-observation.md)
defines the observer's independent capture lifetime and bounded delivery.

## Turn completion

A returned `Ok(T)` means supported-contract validation and conversion succeeded,
the agent ended normally, and local callbacks have settled. Intermediate text or
an accepted result candidate is insufficient. [Contracts and typed results](contracts.md)
defines correction, duplicates, and missing answers; [execution lifecycle](execution-lifecycle.md)
defines the resulting session state and cleanup on failure.
