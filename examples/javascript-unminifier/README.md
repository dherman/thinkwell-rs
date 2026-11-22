# Give JavaScript functions readable names

This example alternates ordinary Rust with a typed Thinkwell prompt:

1. Find functions in a JavaScript module.
2. Ask an agent for descriptive names.
3. Rename the functions and their references in the original source.

For the supplied fixture, the agent might turn `a` into `sum`:

```javascript
function a(xs) { return xs.reduce((total, x) => total + x, 0); }
export function average(xs) { return xs.length ? a(xs) / xs.length : 0; }
```

Rust changes the declaration and its call inside `average`. It leaves the public
name `average` alone. The model suggests names; it never rewrites the program.

## Run it

Use the repository's pinned Rust toolchain and an authenticated Claude installation.
Cache the ACP adapter once:

```sh
npm exec --yes --package=@zed-industries/claude-agent-acp@0.23.1 -- node --version
```

From the repository root:

```sh
export CARGO_TARGET_DIR="$PWD/target/unminifier"
cargo run --manifest-path examples/javascript-unminifier/Cargo.toml --locked -- \
  examples/javascript-unminifier/fixtures/average.mjs \
  target/unminifier/readable.mjs
cat target/unminifier/readable.mjs
```

Choose a new output path on each run. The example uses `create_new` to avoid
replacing existing files. It prints the agent's suggestions and explanations to
stderr. This uses inference and sends the source to the configured agent/provider;
names may vary between runs. Input is never executed by the example.

## The Thinkwell part

A suggestion is ordinary Rust data:

```rust,ignore
#[derive(Deserialize, JsonSchema)]
struct Suggestion {
    /// Copy the candidate ID exactly.
    id: String,
    /// A descriptive JavaScript identifier.
    name: String,
    /// Briefly explain the name.
    reason: String,
}
```

`discover` finds private top-level function declarations and gives each one an ID.
The IDs let us associate suggestions with functions even if the agent responds in
a different order. Named exports are skipped, including aliases.

The core workflow is a loop around a reusable plan:

```rust,ignore
let base = Plan::<Vec<Suggestion>>::new()
    .text("Suggest descriptive JavaScript names for these functions. \
           Return exactly one suggestion per candidate ID, with a brief reason.");

for batch in candidates.chunks(8) {
    let suggestions = base.clone()
        .json(&Context { language: "javascript", source, candidates: batch })?
        .run(agent)
        .await?;
    // Collect suggestions by ID, then apply them in source order.
}
```

`Context` is a small struct with `#[derive(Serialize)]`. `json` adds source text and
candidate information to the prompt. Each `run` returns typed Rust values using a
fresh conversation on the same agent connection. Batches run sequentially; there's
no orchestration framework or application tool to set up.

The executable reads a file, connects with `Claude::connect()`, awaits this workflow,
and writes a new file. If there are no candidates, it skips the connection.
See [main.rs](src/main.rs) and [the workflow and rename helper](src/lib.rs).

## The deterministic part

Oxc resolves which references belong to each function. The helper replaces the
function name and those reference ranges, working backwards through the source so
earlier byte offsets stay valid. It does not regenerate the file: comments,
whitespace and other text outside those edits stay unchanged. A shorthand property
`{a}` becomes `{a: sum}` so its property name stays `a`.

Existing binding names and unresolved names are reserved; repeated suggestions
receive numeric suffixes. Shadowed variables keep their names. These small pieces
make the example useful without building a complete refactoring system.

This is teaching code for ordinary lexical JavaScript modules. It assumes sensible
identifier suggestions, asserts basic parser/result expectations, and may panic on
unsupported inputs. It does not try to detect every syntax or semantic hazard,
eval, reflection, export form or unusual shorthand assignment. Renaming can change
function names and stack traces. Treat the output as something to inspect, not a
proved-equivalent program. File I/O errors use `?`; a failed write can leave a partial
new file. The example intentionally has no production input budgets or retry policy.

## Check it without a model

The example has its own workspace and lockfile. Python 3 is needed for scripted
ACP tests; checks use no credentials or inference:

```sh
export CARGO_TARGET_DIR="$PWD/target/unminifier"
cargo fmt --manifest-path examples/javascript-unminifier/Cargo.toml -- --check
cargo check --manifest-path examples/javascript-unminifier/Cargo.toml --all-targets --locked
cargo clippy --manifest-path examples/javascript-unminifier/Cargo.toml --all-targets --locked -- -D warnings
cargo test --manifest-path examples/javascript-unminifier/Cargo.toml --locked
```

Focused tests cover source preservation, shadowing, collisions, shorthand properties,
function discovery, sequential typed batches, and basic CLI behavior. CI runs these
checks separately from the library workspace. No JavaScript runtime is required.

Code, prompts and fixtures are original project material. Oxc is an MIT-licensed
dependency. The Book's migration research links the original TypeScript example
and tutorial; neither is needed to build or run this example.
