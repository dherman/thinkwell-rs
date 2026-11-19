# Thinkwell

This repository is the future of Thinkwell, implemented in Rust. The current
lifecycle state is the [Porting Phase](book/lifecycle.md).

The unpublished `thinkwell` library now provides an initial typed-conversation
API: connect an ACP agent, declare tools, bind borrowed callbacks per turn, and
return different typed results in one conversation. See the
[compiled conversation example](crates/thinkwell/examples/conversation.rs) and
[implementation evidence and limits](book/implementation-roadmaps/first-product-slice.md).
This is an initial local implementation, not a published release.

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
macOS checks passed locally; Linux CI and owner integration remain pending.

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
