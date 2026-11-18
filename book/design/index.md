# Design

The intended system combines deterministic Rust code with LLM-powered reasoning
while preserving important Thinkwell concepts. The governing
[requirements](../authority/requirements.md) and [boundary](../boundary.md) frame
future design work.

Product architecture and API are intentionally unspecified. Research should
establish user-visible guarantees and compatibility scope before selecting crate
boundaries or dependencies.

No product anchors are defined. Future design chapters should state meaningful
properties, ground them in accepted authority, and give criteria with explicit
verification methods. Preparation progress belongs in the
[project-scaffolding roadmap](../implementation-roadmaps/phase-0.md).
