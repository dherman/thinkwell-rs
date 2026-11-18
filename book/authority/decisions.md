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

> **`D-dual-licensing`** (accepted): **Offer MIT or Apache-2.0 at the recipient's option**
> Choice: License original project code and documentation under MIT OR Apache-2.0. Contributions are submitted under both licenses unless explicitly agreed otherwise; separately identified third-party material retains its own terms and required notices.
> Requirements: `R-source-provenance`
> Rationale: The owner explicitly selected and approved dual licensing after reviewing compatibility with MIT upstream material. Recipient choice does not remove upstream notice obligations or imply patent grants from MIT-only upstream authors.
> Alternatives:
> - MIT only: provides permissive reuse but omits an explicit Apache-2.0 option.
> - Apache-2.0 only: provides an explicit patent-license framework but omits the MIT option requested by the owner.
> Reconsider when: The owner proposes a licensing change or a source-specific incompatibility requires review; preserve rights already granted and obtain necessary permissions.

> **`D-thinkwell-identity`** (accepted): **Make this repository the future of Thinkwell**
> Choice: The project is Thinkwell; this repository and its Rust implementation are its future. The current lifecycle state is Porting Phase, with legacy TypeScript evidence temporary and distinct from accepted Thinkwell requirements. After transition, unqualified Thinkwell means this project and its Rust implementation.
> Requirements: `R-temporary-migration-reference`; `R-porting-transition`
> Rationale: The owner explicitly establishes project identity and a bounded migration lifecycle, without accepting product compatibility or architecture.
> Alternatives:
> - Treat this as a permanently subordinate implementation: conflicts with the owner's chosen project identity.
> Reconsider when: The owner explicitly revises project identity or lifecycle direction.

> **`D-future-independence`** (accepted): **Govern future phases independently**
> Choice: After the human-approved transition, deprecate the TypeScript implementation. Thinkwell is governed by this repository's Book, decisions, code, tests, and delivery records, with no continuing compatibility, parity, synchronization, or tracking obligation. Remove migration-only material except required notices and provenance; active instructions, requirements, designs, roadmaps, CI, builds, tests, and releases must not depend on or refer developers back to that implementation. Introduce future lifecycle states as needed without a numbered sequence.
> Requirements: `R-temporary-migration-reference`; `R-porting-transition`
> Rationale: The owner explicitly requires independence after migration and clean removal of temporary research.
> Alternatives:
> - Maintain indefinite cross-implementation parity: creates the permanent obligation the owner rejected.
> Reconsider when: The owner explicitly approves a changed lifecycle policy; no automatic transition or inherited compatibility duty is implied.
