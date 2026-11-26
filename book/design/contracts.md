# Contracts and typed results

Derives describe the wire values that tools and answers accept; validation and
conversion enforce that boundary. `D-derived-contracts` governs compiler-integrated
contract generation, and `R-per-turn-result-types` requires different answer types
within one conversation. `D-initial-conversation-scope` bounds schema support.

> **`A-typed-turn-completion`** (accepted): **Typed results require completed turns**
> Property: Success requires supported-schema validation and conversion, a valid candidate, normal terminal response, and settled local callbacks. Result types may change between turns in one conversation.
> Authority: `R-per-turn-result-types`
> - Criterion `behavior` [test]: Tests cover changing result types within a session; invalid, malformed, corrected, exhausted, missing and duplicate submissions; cancellation after candidate acceptance; terminal/callback overlap; and transport failure before a candidate.
> - Criterion `abnormal-completion` [test]: Each supported abnormal terminal stop reason rejects an accepted candidate, returns its defined error, and leaves the session unusable; only normal terminal completion with settled callbacks returns the result.
> - Criterion `candidate-transport-loss` [test]: Transport loss after accepting a candidate fails the turn, discards the candidate, and prevents session reuse.
> - Criterion `custom-result-decoding` [test]: A schema-valid result rejected by a custom Deserialize implementation consumes the invalid-result budget and never becomes a candidate; correction succeeds and exhaustion fails the turn.
> - Criterion `nullable-result-submission` [test]: An explicit null value is accepted for a supported nullable result type, while an absent value is rejected and consumes the invalid-result budget.

> **`A-supported-contract-boundary`** (accepted): **Advertised supported contracts are enforced**
> Property: Unsupported contracts fail before prompting. Supported input, serialized tool output, and final-result constraints are validated independently of Serde conversion.
> Authority: `D-derived-contracts`
> - Criterion `behavior` [test]: Contract tests assert supported primitive, array, closed-object, required-field, const/enum and union validation; directional nested schemas; numeric precision and finite doubles; rejection of unsupported dialects, references and constraints; malformed wire input/output; and located preflight errors.
> - Criterion `open-object-fields` [test]: With additionalProperties true, undeclared fields are accepted while declared field types and required fields remain enforced; false rejects undeclared fields.
> - Criterion `schema-metadata` [test]: String title and description metadata are accepted without changing value validation; malformed metadata types fail preflight with their schema locations.
> - Criterion `custom-input-decoding` [test]: Schema-valid tool input rejected by a custom Deserialize implementation returns the defined tool error without invoking the callback; a later valid invocation remains usable.

## Contract generation and value boundaries

Use Serde, Schemars, and serde_json through ordinary Rust compilation. A derived
schema is metadata, not proof of a value's validity or of the model's interpretation
of a prompt. Runtime validation checks the supported contract independently of
Serde conversion.

| Boundary | Schema direction | Processing order |
| --- | --- | --- |
| Application tool input | Deserialization | Validate JSON, deserialize I, then call the Rust callback |
| Application tool output | Serialization | Obtain O, serialize it, then validate the serialized JSON |
| Final result for a turn | Deserialization | Validate JSON, deserialize T, retain the candidate until terminal completion |

Direction-specific Serde names must describe the actual wire value. Nonrecursive
nested derived records and enums are generated inline in both directions; callers
need no inline annotations. Repeated types can duplicate schema text. Recursive
types remain unsupported; inlining does not introduce a reference resolver.

Unsupported contracts fail before prompting. For a one-shot run, answer/tool
preflight precedes remote session setup. Errors identify the containing property,
array-item, or alternative path rather than silently ignoring unsupported constraints.

## Supported schema subset

Only the JSON Schema 2020-12 dialect is supported. The finite subset is:

| Feature | Meaning and limit |
| --- | --- |
| Objects | Named properties, required fields, and boolean additionalProperties |
| Arrays | Homogeneous items |
| Primitive types | String, boolean, null, number, and integer |
| const / enum | Restrict values by JSON equality, preserving finite numeric precision as described below |
| oneOf / anyOf | Exactly one matching alternative / at least one matching alternative, including tagged-enum shapes |
| format: int64 | Only with integer; enforce signed 64-bit representation |
| format: double | Only with number; require conversion to a finite f64 |
| Metadata | Titles and descriptions guide the agent; dialect metadata selects the supported dialect, not an additional value constraint |

References, definitions, recursive schemas, other formats, and unsupported keywords
or constraints are preflight errors. This excludes otherwise valid Rust types or
JSON Schema documents; Serde deserialization alone cannot extend the supported set.

Integer contracts reject floating JSON representations such as 3.0. Numeric const
and enum comparisons use the serialized finite decimal value without rounding large
integers through f64; numerically equivalent integer/floating constants can compare
equal while the independent integer-type rule still rejects floating representation.
The int64 format additionally requires the signed bound.

A double contract must convert to a finite f64. It does not promise exact decimal
representation. Serialized NaN and infinities fail, including serialization as null.
Finite f64 validation does not impose arbitrary numeric ranges on application data.
Application rules beyond the supported schema, such as whether an answer is useful
or faithful to input, remain ordinary Rust/application responsibilities.

## Stable wire contracts

A Session advertises a fixed application-tool catalog plus an internal result tool.
Each application input is carried as `{ "turn": ID, "input": VALUE }`.
The endpoint checks the identifier before the local dispatcher validates VALUE;
the Rust callback receives only I.

A successful application response carries `{ "value": VALUE }` in MCP
structuredContent, with a matching object outputSchema. Text content retains the
serialized application value. This permits primitive outputs while keeping the MCP
output schema object-shaped. The output contract describes serialization of O;
it is independent of the contract for the final answer T.

The internal `return_result` tool takes `{ "turn": ID, "value": VALUE }`, with a
stable generic value schema. Each prompt supplies its fresh identifier and the
specific schema for that turn's T. The fixed catalog therefore supports changing
result types without upfront declarations of every future answer type or dynamic
catalog refresh. Prompt guidance is not provider-enforced constrained generation.
[Agent integration](agent-integration.md#turn-routing) owns identifier checks and
revocation; the identifier does not authenticate originating-turn causality.

## Result submission and completion

1. Reject missing, malformed, nonmatching identifiers or unknown tools before
   routing to callbacks or result handling. Do not repair an identifier or reveal
   the current one in a rejection message.
2. For a matching result submission, distinguish an absent value from explicit
   null. Validate the supported contract, then deserialize that turn's T.
3. On invalid value or conversion, return a tool error so the agent can correct
   its submission within the [configured budget](execution-lifecycle.md#limits-and-failures).
   This does not replay the whole prompt or undo repeated application calls.
4. Retain the first valid candidate and acknowledge it. Reject duplicates without
   replacing it. Candidate acceptance is an intermediate event, not successful T.
5. Return success only after a normal ACP end_turn with settled local callbacks.
   A normal end without a candidate is MissingResult. Cancellation, abnormal stop,
   transport loss, deadline, or terminal/callback overlap cannot turn the candidate
   into success.

Successful explicit sessions retain history for the next differently typed turn.
Failed started turns follow the conservative invalidation and cleanup policy in
[execution lifecycle](execution-lifecycle.md). Result validation events are optional
[progress](progress-observation.md); observing them is never required for completion.

## Verification boundary

The anchor criteria require executable tests. Contract tests establish supported
and rejected schema cases independently of Serde; protocol tests exercise the
production input/output/result paths and terminal races. Design and change review
assesses the stated schema, ownership, and protocol limits separately; its judgments
cannot satisfy or gate anchor verification. Tests cover deterministic library
behavior; a bounded live-agent interaction establishes interoperability for its
configuration, not model accuracy or universal agent compliance. No criterion here
requires formal proof, and a successful schema conversion alone is not completion
of either anchor's obligations.
