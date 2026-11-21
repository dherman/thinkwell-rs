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

The example uses the agent's default model and a fixed input file so the
conversation is easy to follow. There is no command-line parsing.

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
traits; `Serialize` is useful for inspecting the extracted data. Thinkwell handles
these nested types automatically; no extra schema annotations are needed.

The field descriptions guide the agent, but do not enforce hour ranges. The Rust
validation step below enforces them.

## 2. Open a conversation and ask for availability

In [main.rs](src/main.rs), the program reads the included JSON roster, connects
with `Claude::connect()`, and opens one session. The first `Plan` call is directly
visible in `main`:

```rust
let parsed = Plan::<ParsedAvailability>::new()
    .text("Translate this roster's availability into weekday time windows.")
    .json(&roster)?
    .run_in(&mut session)
    .await?;
```

`json(&roster)?` adds the roster as JSON and reports serialization errors
immediately. The plan owns the resulting text, so it does not borrow the roster.
JSON formatting does not prevent the agent from interpreting data as instructions.

This abbreviated snippet omits only the prompt text; the full instructions are
readable in the source. The answer is then printed and checked with ordinary Rust
functions from [model.rs](src/model.rs):

```rust
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
that task; the session stays on the calling async task. The two `?` operators
in `.await??` handle the task error and the solver error, respectively. This is
Tokio error handling, not extra Thinkwell cleanup.

## 4. Ask for an explanation of the result

The program prints and flushes the solver result before asking another question:

```rust
println!("\nSolver result: {outcome}");
std::io::stdout().flush()?;
```

The second `Plan` in [main.rs](src/main.rs) asks for a `String` in the same session.
That session remembers the roster and previous answer. The return type changes
from `ParsedAvailability` to `String`; no new agent or conversation is needed.
A plain text answer needs no wrapper struct.

The explanation is labeled generated prose. It cannot replace the Rust enum or
the printed canonical slot. If this second turn fails, the program reports the
error and exits unsuccessfully, but the solver result remains visible. No extra
finish call is required. The session and agent begin cleanup when they go out
of scope.

## Try a conflicting roster

To explore another input, change the single `include_str!` path in `main.rs` from
`../data/attendees.json` to `../data/no-overlap.json`, then run the same command.
One attendee is free only Monday and the other only Tuesday. With a faithful
extraction the canonical output is:

```text
Solver result: No feasible one-hour meeting in weekday business hours.
```

To experiment further, edit the included roster. Each object has `id`, `name`,
and `availability` strings. Use distinct nonempty IDs and names/descriptions.
The program displays an interpretation to review; it does not pause for approval
and never books a calendar event. These fixtures are small enough to read alongside
the output. Restore the original include path before running the scripted workflow
tests, which intentionally check the original sample.

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
