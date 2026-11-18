# Decisions

These choices were explicitly supplied by the project owner. Alternatives below
describe tradeoffs, not an assertion that an earlier evaluation occurred.

> **`D-rust-language`** (accepted): **Use Rust for the implementation**
> Choice: Implement the reimplementation in Rust.
> Requirements: `R-idiomatic-reimplementation`
> Rationale: Rust is an explicit project constraint; architecture and API will be designed for it.
> Alternatives:
> - Continue only the TypeScript implementation: avoids a port but does not meet the Rust project goal.
> Reconsider when: The owner explicitly changes the project's language goal.

> **`D-github-monorepo`** (accepted): **Use this GitHub monorepo**
> Choice: Keep the project in the already-created dherman/thinkwell-rs GitHub repository as a monorepo.
> Requirements: none
> Rationale: The owner explicitly selected this repository; it allows coordinated changes without fixing crate boundaries.
> Alternatives:
> - Separate repositories: allow independent ownership but add coordination and do not match the selected home.
> Reconsider when: Ownership or independently managed components require a reviewed repository change.

> **`D-anchor-book-method`** (accepted): **Use the public Anchor Book method and tools**
> Choice: Use Anchor Book for design, authority, roadmaps, and evidence, manually scaffolding with supported commands.
> Requirements: `R-explicit-authority`; `R-scaffolding-boundary`
> Rationale: The owner explicitly selected this development method and public tooling.
> Alternatives:
> - Unstructured project notes: require less format discipline but omit the selected method's explicit relationships.
> Reconsider when: A concrete tooling or method limitation warrants an owner-approved change.

> **`D-versioned-project-knowledge`** (accepted): **Version project knowledge with the work**
> Choice: Keep project knowledge, decisions, roadmaps, and evidence in version-controlled sources.
> Requirements: `R-explicit-authority`
> Rationale: The owner requires durable context that can be reviewed with implementation changes.
> Alternatives:
> - Keep knowledge only in conversations: reduces repository content but loses durable reviewable context.
> Reconsider when: The owner changes the knowledge-management policy while preserving necessary provenance.

> **`D-deferred-product-design`** (accepted): **Design the port after scaffolding**
> Choice: Limit this session to scaffolding; design the port's architecture and API in later sessions.
> Requirements: `R-scaffolding-boundary`
> Rationale: The owner explicitly separated preparation from product design and implementation.
> Alternatives:
> - Design and implement now: might produce early behavior but violates the assigned boundary and risks premature architecture.
> Reconsider when: Phase 0 is complete and the owner assigns the next design work.

> **`D-library-name`** (accepted): **Name the Rust library thinkwell**
> Choice: Name the initial unpublished Rust library thinkwell, without a -rs suffix.
> Requirements: none
> Rationale: The owner explicitly selected this name and reported checking crates.io availability; no name is reserved or published by this decision.
> Alternatives:
> - Use a temporary scaffold name: avoids early naming but conflicts with the owner's explicit choice.
> Reconsider when: Publication constraints or an explicit owner decision require a different crate name.
