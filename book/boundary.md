# Boundary and non-goals

The long-term goal is Thinkwell implemented in idiomatic Rust. Supported user-facing
concepts, behavior, and guarantees require explicit design and human review.

The [library design](design/index.md) defines typed conversations, borrowed
per-turn tools, finite schema validation, owned subprocess cleanup, and optional
progress observation. It includes one-shot execution and explicit persistent
sessions, with Claude and Codex presets over ACP and HTTP MCP. The owning design
chapters specify the supported scope; library roadmaps own implementation and
verification evidence.

Dynamic catalogs, concurrent sessions on one connection, shared host embedding,
arbitrary JSON Schema, independent progress streams, Windows support, and automated
installation/authentication remain outside the library scope. Commit, push, merge,
release, repository settings, and ending the Porting Phase retain their separate
authorization boundaries.

The lifecycle remains Porting Phase and the engine working mode is `port`.
Accepted product anchors state intended guarantees, not proof that
every criterion has been verified or that delivery is complete.
