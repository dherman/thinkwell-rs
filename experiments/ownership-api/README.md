# Ownership API experiment

Original, isolated Rust code for the approved `ownership-prototype-1` experiment.
This is not Thinkwell's product implementation or a selected public API. No code
was copied from Thinkwell TS, determinishtic, or their dependencies.

Run from the repository root:

```sh
python3 experiments/ownership-api/check.py
rustfmt --edition 2024 --check experiments/ownership-api/*.rs experiments/ownership-api/cases/*.rs
```

The harness uses the repository's pinned Rust toolchain, no Cargo dependencies,
and ignored output under `target/ownership-api`. It runs positive scenarios and
requires specific diagnostic codes/messages from negative cases. Each subprocess
has a 60-second timeout. Negative cases are intentional compiler errors, not
workspace members. The experiment is not added to CI by this assignment.

Read `scenarios.rs` for the five user-facing patterns; `prototype.rs` contains the
simulated driver. `cases/` isolates rejected ownership patterns and the generic
storage / Send-adapter probes. Roadmap evidence and interpretation live in
`book/implementation-roadmaps/ownership-prototype.md`.

## What the simulation does

`Plan<T>` stores prompt fragments, accepts async tool callbacks, and returns T
from an in-memory scripted fixture. Typed tool inputs are transported as `Any`;
this is an ownership-testing convenience, not a serialization proposal. The
driver invokes named tools sequentially and discards their outputs. Assertions
check actual callback side effects, error paths, configuration, and final types.
It never calls a model or implements ACP/MCP, schemas, or derives.

Callbacks are boxed behind a lifetime-bound object-safe interface. The internal
adapter boxes each borrowed invocation without requiring user syntax. The local
future is deliberately not Send. Shared-reference and mutable-reference captures
both work, including captures across a Pending boundary. Inputs and outputs are
owned (`'static`); borrowed tool arguments are not assessed.

The reusable/scoped marker changes once when the first callback is attached.
Reusable plans clone prompt configuration. Scoped plans cannot clone, even when
a particular callback could safely do so: this conservative restriction keeps the
experiment small. Shared reusable tools and conditional Clone are unresolved.
`scoped()` normalizes the marker before conditional registration. A helper can
then return the same type regardless of callback type or number.

`execute(script)` produces a result future and an event receiver. Direct await
and `run(script).await` drop unused observation. With `into_parts()`, the caller
polls the result and events together; polling events alone does not drive work.
A completed boxed result future can retain captures until dropped. There is no
spawned execution task. Concurrent examples interleave futures on one thread.

The observation queue holds two events and drops the oldest on overflow, exposing
a drop count. This is only a way to test bounded memory without blocking progress,
not a proposed product event-delivery policy. Dropping observation clears the
queue and lets execution continue. Remote cancellation, shutdown, fairness,
parallel tool scheduling, and protocol failure handling remain untested.

## What the probes establish

- The pinned compiler accepts local erased AsyncFnMut callbacks without a macro.
- Requiring a Send erased future fails with only `F: AsyncFnMut + Send`; a
  higher-ranked adapter explicitly returning a Send future succeeds. This
  assesses those signatures, not every possible approach to Send callbacks.
- Generic callback-chain storage supports opaque helper return types, but
  branches returning different chain shapes do not unify without another
  abstraction. The erased prototype unifies callbacks after the Scoped boundary.
- Exclusive local captures prevent overlapping access, two independently
  registered mutable captures of the same value, and a plan escaping its referent.

No throughput, allocation, schema, dependency compatibility, or real-agent claims
follow from these results. Future API choices remain subject to design review.
