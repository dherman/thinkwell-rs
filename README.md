# Thinkwell in Rust

This repository prepares a Rust reimplementation of the
[Thinkwell TypeScript framework](https://github.com/dherman/thinkwell), which
combines deterministic code with LLM-powered reasoning through a fluent API.

Phase 0 supplies project scaffolding only. The unpublished `thinkwell` library
has no product API or dependencies. Its presence makes workspace checks usable;
it does not establish the eventual crate architecture or compatibility goals.

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
