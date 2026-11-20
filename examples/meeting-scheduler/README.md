# Find a meeting time with Rust and an agent

People describe availability in words. A solver needs precise constraints. This
example uses Thinkwell to connect the two: ask an agent to read the availability,
find a valid meeting time in Rust with Z3, then ask the agent to explain the result.

The program has two agent turns with ordinary Rust work between them. It uses the
existing Thinkwell API; the validation and solving helpers belong to this example.
There are no application tools: Rust calls the solver directly.

## Try it

Run these commands from the repository root. You need:

- The repository's Rust 1.100.0 toolchain.
- CMake 3.26 or newer and a C++ compiler supported by the bundled Z3 source.
  Local validation used CMake 3.31.10 and Apple Clang 17; CI uses Ubuntu 24.04's
  compiler and CMake 3.31.10. A passing local build does not establish a Linux pass.
- Python 3 for the deterministic tests.
- For a live conversation: Node/npx, cached
  `@zed-industries/claude-agent-acp@0.23.1`, and existing Claude authentication.
  Thinkwell starts the adapter with offline npx; it does not install or sign in
  for you. The roster is sent to that agent.

If CMake is missing, one local installation option is:

```sh
python3 -m venv target/scheduler-tools
target/scheduler-tools/bin/pip install cmake==3.31.10
export PATH="$PWD/target/scheduler-tools/bin:$PATH"
```

Keep build products in the repository's ignored target directory:

```sh
export CARGO_TARGET_DIR="$PWD/target/meeting-scheduler"
cargo test --manifest-path examples/meeting-scheduler/Cargo.toml --locked
cargo run --manifest-path examples/meeting-scheduler/Cargo.toml --locked
```

The first command runs without an installed agent or credentials. The second
makes a real conversation using [the included roster](data/attendees.json):
Sam is free Wednesday 10–12, Lee Wednesday 11–14, and Jo Wednesday mornings.

This package builds Z3 from source through `z3`'s `vendored` feature. The first
build may take several minutes; later runs reuse the compiled solver. It is an
independent Cargo workspace, so ordinary Thinkwell builds do not compile Z3.
The example's lockfile currently resolves `z3` 0.23.0, `z3-sys` 0.14.2, and
`z3-src` 504.1.0 (native Z3 5.4.1). Cargo downloads dependencies initially;
the native source build itself needs no separate system Z3 installation.

Use the agent's default model, or select a name the agent supports:

```sh
cargo run --manifest-path examples/meeting-scheduler/Cargo.toml --locked -- --model haiku
```

A successful run with the sample roster looks like this (the explanation may vary):

```text
Extracted availability (review this interpretation):
  Sam [sam]: Wednesday 10:00–12:00
  Lee [lee]: Wednesday 11:00–14:00
  Jo [jo]: Wednesday 09:00–12:00

Solver result: Scheduled: Wednesday 11:00–12:00 (shared timezone)

Agent explanation (generated prose):
The meeting was scheduled for Wednesday 11:00–12:00, which falls within all three attendees' extracted availability windows.
```

## 1. Choose the first answer's shape

The first question asks for data, not prose. `ParsedAvailability` contains one
entry for each attendee ID, and each entry contains weekday/hour windows.
`Deserialize` lets Thinkwell convert the answer into Rust values. `JsonSchema`
tells the agent what fields to produce.

From [model.rs](src/model.rs):

```rust
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(inline)]
pub struct TimeWindow {
    /// Weekday in the shared timezone.
    pub day: Day,
    /// First available hour, from 0 through 23, inclusive.
    pub start_hour: i64,
    /// End of availability, from 1 through 24, exclusive; greater than start_hour.
    pub end_hour: i64,
}
```

`Day` is a lowercase weekday enum. The enclosing records also derive these
traits; `Serialize` is useful for inspecting the extracted data. Inline schemas
keep nested types within Thinkwell's current supported subset. They avoid schema
references; they do not add new validation features to the library.

The field descriptions guide the agent, but do not enforce hour ranges. The Rust
validation step below enforces them.

## 2. Open a conversation and ask for availability

In [main.rs](src/main.rs), the program loads the roster and opens an agent session.
The optional model setting is applied before opening the conversation. The
[extract helper](src/workflow.rs) builds a `Plan<ParsedAvailability>`, explains
weekday/hour conventions, includes the roster as JSON text, and runs the plan
in that session:

```rust
let parsed = extract(&mut session, &roster).await?;
println!("{}", format_extraction(&roster, &parsed));
let checked = validate_availability(&roster, parsed)?;
```

An agent answer can have the right JSON shape and still be wrong for the
application. The validator requires every original attendee ID exactly once,
rejects unknown IDs, and checks that `0 <= start < end <= 24`. An empty list of
windows means that person cannot attend; it does not remove the person from the
problem. Invalid data produces an error instead of a schedule.

The extraction is printed so you can compare it with the original wording.
Z3 cannot tell whether the agent interpreted the wording faithfully. All people
share one timezone and one notional week; the program does not resolve dates,
DST, or different timezones. Prompt instructions are guidance, not a security
boundary for arbitrary text in the roster.

## 3. Solve the checked data in Rust

The [solver helper](src/solver.rs) creates two symbolic integers: a weekday and a
starting hour. It constrains the meeting to business hours, then requires the
whole one-hour meeting to fit in at least one window for **every** attendee.
For a window, the core constraint is:

```rust
Bool::and(&[
    day.eq(window.day.index()),
    hour.ge(window.start_hour),
    (&hour + 1).le(window.end_hour),
])
```

Z3 searches for values that satisfy all constraints. The result is a Rust enum:

- `Scheduled(slot)`: a valid meeting time. Rust independently checks that the
  returned slot fits every attendee's windows.
- `NoSlot`: no one-hour slot satisfies these parsed constraints.
- `Inconclusive(reason)`: the solver did not establish an answer, for example
  because its ten-second query timeout expired. This does **not** mean no slot exists.

Model-reading errors are ordinary errors, not `NoSlot`. When several slots work,
the program may choose any one; it does not promise the earliest.

The main program runs the synchronous native solver with `spawn_blocking`, leaving
Tokio free to drive the agent connection while Z3 works. The solver timeout is
separate from Thinkwell's per-turn limits. Only owned, validated data moves into
that task; the session stays on the calling async task.

## 4. Ask for an explanation of the result

The program prints and flushes the solver result before asking another question:

```rust
println!("\nSolver result: {outcome}");
std::io::stdout().flush()?;
```

The [explain helper](src/workflow.rs) runs a `Plan<Explanation>` in the same session.
That session remembers the roster and previous answer. The return type changes
from `ParsedAvailability` to `Explanation`; no new agent or conversation is needed.

The explanation is labeled generated prose. It cannot replace the Rust enum or
the printed canonical slot. If this second turn fails, the program reports the
error and exits unsuccessfully, but the solver result remains visible. No extra
finish call is required. The session and agent begin cleanup when they go out
of scope.

## Try a conflicting roster

```sh
cargo run --manifest-path examples/meeting-scheduler/Cargo.toml --locked -- \
  examples/meeting-scheduler/data/no-overlap.json --model haiku
```

One attendee is free only Monday and the other only Tuesday. With a faithful
extraction the canonical output is:

```text
Solver result: No feasible one-hour meeting in weekday business hours.
```

To use your own file, supply a JSON array of objects with `id`, `name`, and
`availability` strings. Use distinct nonempty IDs and names/descriptions. Pass
`--help` for the command syntax. The program prints an interpretation to review;
it does not pause for interactive approval and never books a calendar event.

## Checks and boundaries

```sh
cargo fmt --manifest-path examples/meeting-scheduler/Cargo.toml -- --check
cargo check --manifest-path examples/meeting-scheduler/Cargo.toml --all-targets --locked
cargo clippy --manifest-path examples/meeting-scheduler/Cargo.toml --all-targets --locked -- -D warnings
cargo test --manifest-path examples/meeting-scheduler/Cargo.toml --locked
```

Tests cover roster/range errors, full-hour/business-hour boundaries, empty and
conflicting availability, and real Z3 results compared with an independent finite
search. Unknown handling is tested directly rather than through a flaky timeout.
A scripted ACP peer submits an invalid answer followed by valid nested data, then
answers the second turn with a different type. An executable test confirms that
an explanation failure leaves the canonical result visible. No test runs a real
model. Linux execution belongs to the separate **Meeting scheduler example** CI job.

The example supports the library's macOS/Linux environment. It intentionally omits
calendar integration, multiple timezones, fractional hours, variable durations,
streaming progress, concurrent agents, and new public API conveniences.

The Rust code, prompt wording, tutorial, and fixtures are original. The example
is inspired by the [TypeScript meeting-scheduler tutorial](https://thinkwell.sh/examples/meeting-scheduler);
its design research and deliberate differences are recorded in the Book's
[migration research](../../book/migration.md#example-selection). This is not a
behavioral-parity promise or a runtime dependency on the TypeScript project.
Original material uses the repository's MIT OR Apache-2.0 license. The z3 wrapper,
z3-sys bindings, and native Z3 dependency carry their own MIT notices; preserve
the applicable dependency notices when distributing a linked executable.
