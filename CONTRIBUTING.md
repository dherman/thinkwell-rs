# Contributing

Read the [Book](book/SUMMARY.md), particularly
[working on the project](book/working-on-the-project.md) and the
[current boundary](book/boundary.md), before changes. Work requires a human-approved
milestone; Phase 0 does not authorize implementing the port.

## Install the pinned tools

Run from the repository root with Git and rustup installed:

```sh
rustup toolchain install 1.100.0 --profile minimal --component rustfmt --component clippy
cargo anchor --version
cargo install mdbook --version 0.5.4 --locked --root target/tools
export PATH="$PWD/target/tools/bin:$PATH"
rustc --version
mdbook --version
```

The toolchain file selects Rust 1.100.0, including formatting and Clippy. It is a
development baseline, not an MSRV declaration. Keep `Cargo.lock` version controlled.
The expected engine identity is
`anchorbook 0.1.0 (9689b00416fd577bfb68a02f1fa310f6088404cf)`; mdBook is `0.5.4`.
`cargo anchor` verifies the repository-local engine on every run and automatically
installs/replaces it from the exact revision when needed. The pin and expected
identity live in `tools/anchor/src/main.rs`. No ambient anchorbook is invoked.
Git, rustup/Cargo, and network access are needed for first use or recovery; a
matching local engine is reused without installation. The alias uses a small,
dependency-free development binary, not a Thinkwell runtime dependency.
Tools and generated output stay under ignored `target/`. Re-export PATH in each
new shell. No global Rust default change is required.

## Local checks

After installing tools and setting PATH, run:

```sh
cargo anchor contract
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo anchor check --config anchorbook.toml
mdbook build
git diff --check
```

The library intentionally has no tests yet; the development launcher has focused
bootstrap tests. Successful Cargo commands establish
that the scaffold compiles and its test harness runs, not Thinkwell behavior.
`check` evaluates Book structure, not semantic validity or completion.
The mdBook preprocessor additionally validates authority records and roadmap
references. Inspect generated navigation, records, and links under
`target/anchorbook/`; a build is not a complete content review.

Use default rustfmt and Clippy behavior. There is no custom lint configuration.
CI runs the six Cargo/Book checks above and whitespace checking on Ubuntu 24.04.
It installs exact tool versions and uses a SHA-pinned checkout action. Runner images
can change; this is not a hermetic or bit-for-bit reproducibility guarantee.
Local results do not establish that GitHub CI has run.

## Updating tools

Inspect the candidate engine's CLI, configuration reader, compiled contract,
format documentation, and fixtures before upgrading. Features described in its
Book may still be planned. Select a full Git commit SHA and its compatible mdBook
version; install with `--locked`. Update the engine pin and expected identity in
`tools/anchor/src/main.rs`, the mdBook installation/version check in this file and
`.github/workflows/ci.yml`, and the pinned authoring links
in the Book. Update `rust-toolchain.toml` and the matching setup instructions when
changing Rust. Tool pins are setup choices, not product architecture or MSRV policy.

Read the new contract, reconcile changed formats and guidance, run all local
checks, inspect rendered records and links, and record results and limitations
in the assigned roadmap. Do not add the planned `[engine]` configuration until a
reviewed revision actually supports it. Do not install a moving branch in CI.

## Change and publication boundaries

Update durable knowledge and criterion evidence alongside the work. Record exact
commands, results, revision context, and remaining gaps; preserve stable record
and milestone identifiers. No license has been selected for this repository.
Check upstream applicability and attribution before any source adaptation.

Commits, pushes, publishing, releases, and repository settings require explicit
authorization. CI here only validates; it does not deploy or publish.
