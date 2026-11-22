# Thinkwell

This repository is the future of Thinkwell, implemented in Rust. The current
lifecycle state is the [Porting Phase](book/lifecycle.md).

The unpublished `thinkwell` library now provides an initial typed-conversation
API: connect an ACP agent, declare tools, bind borrowed callbacks per turn, and
return different typed results in one conversation. See the
[compiled conversation example](crates/thinkwell/examples/conversation.rs) and
[implementation evidence and limits](book/implementation-roadmaps/first-product-slice.md).
The first product slice is integrated; this is not a published release.

Use Tokio with I/O/time enabled. Agent installation and authentication are external.
The Claude preset uses the locally cached ACP adapter 0.23.1 via offline npx.
Normal tests use a Python 3 fixture and Unix ps; no inference or credentials are
required. To explicitly run the live acceptance example with model selector haiku:

```sh
cargo run -p thinkwell --locked --example live_check
```

The default path uses RAII. Cancellation of a started turn closes its session and
connection; application effects are not rolled back. Schema support is deliberately
finite, and Windows/embedding/concurrent sessions are not initial support targets.
The integrated first product slice passed local macOS checks and Linux CI.

Codex is also available through `thinkwell::agent::Codex`: use
`Codex::connect().await?` with the same plans, tools and sessions. Cache its adapter
before first use:

```sh
npm exec --yes --package=@agentclientprotocol/codex-acp@2.8.0 -- node --version
```

Authenticate Codex separately. The preset uses offline npx and the adapter's
read-only mode. Codex can still perform native reads without asking Thinkwell;
Thinkwell's declared-tool permission policy is not an operating-system sandbox.
Existing Codex configuration applies; reserve the MCP server name `thinkwell`
for this library. See the [Codex compatibility record](book/implementation-roadmaps/codex.md)
for tested versions and limitations.

For a practical code/prompt workflow, try the
[meeting scheduler example and walkthrough](examples/meeting-scheduler/README.md).
It translates availability into typed data, solves it with Z3 in Rust, then asks
for an explanation. Its separate Cargo package keeps the solver out of ordinary
library builds.

The [sentiment analysis walkthrough](examples/sentiment-analysis/README.md) shows
the complementary pattern: lend a local Rust analyzer and mutable score log to
an agent tool, then read a typed interpretation of its measurements.

Add optional progress without changing the answer type:

```rust,ignore
let answer = Plan::<String>::new()
    .text("Suggest a garden name.")
    .observe(|event| {
        if let thinkwell::ProgressEvent::Message(message) = event {
            eprint!("{}", message.text());
        }
    })
    .run(&agent)
    .await?;
```

Observers can borrow local state. Progress is bounded and best-effort; the awaited
result determines success. The opt-in `cargo run -p thinkwell --locked --example progress`
shows console messages and a borrowed tool with Claude (its source explains the
Codex substitution). It requires an installed, authenticated agent and uses inference.

Project knowledge, decisions, roadmaps, and evidence live in the
[Anchor Book](book/SUMMARY.md). Start with its [introduction](book/introduction.md)
and [boundary](book/boundary.md). See [CONTRIBUTING.md](CONTRIBUTING.md) for
tool installation and local checks, and [AGENTS.md](AGENTS.md) for agent startup.

The crate name is `thinkwell`; this is not a publication or affiliation claim.
No substantive upstream implementation has been copied or adapted.

## License

Original code and documentation in this repository are licensed under either the
[MIT License](LICENSE-MIT) or the [Apache License, Version 2.0](LICENSE-APACHE),
at your option (`MIT OR Apache-2.0`).

Separately identified third-party material retains its applicable license and
notices. Any future adaptation of upstream Thinkwell code must preserve required
copyright and license notices; choosing Apache-2.0 does not remove upstream MIT
notice obligations. See [contribution terms](CONTRIBUTING.md#licensing-and-contributions).
