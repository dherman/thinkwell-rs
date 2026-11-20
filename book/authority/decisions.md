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

> **`D-acp-client`** (accepted): **Use ACP as a headless programmatic client**
> Choice: Thinkwell communicates with agents using the Agent Client Protocol (ACP), acting as a headless, programmatic client. This is the Rust architecture's foundation, independently of legacy implementation details.
> Requirements: `R-easy-agent-programming`; `R-progressive-control`; `R-provider-breadth`; `R-code-prompt-ergonomics`
> Rationale: The owner explicitly selected ACP as the core architecture, enabling programmatic agent interactions across model providers.
> Alternatives:
> - Build the core around direct provider APIs: would require a different integration boundary from the owner's selected ACP client architecture.
> Reconsider when: The owner explicitly changes the protocol foundation or a demonstrated protocol limitation requires an architectural review.

> **`D-derived-contracts`** (accepted): **Derive tool contracts through normal Rust compilation**
> Choice: Use Rust's derive functionality for type-derived tool contracts, without requiring an additional Thinkwell schema-generation build tool. The specific derives, traits, libraries, and supported schema features remain to be designed.
> Requirements: `R-easy-agent-programming`; `R-code-prompt-ergonomics`
> Rationale: The owner explicitly identified Rust derives as the way to provide automatic contracts without extra build tooling during port-design exploration on 2026-11-18.
> Alternatives:
> - Require a separate schema-generation command: adds a build workflow where the owner selected compiler-integrated derivation.
> Reconsider when: A concrete unsupported contract requirement demonstrates a limitation that warrants owner review.

> **`D-provisional-scoped-turns`** (accepted): **Use scoped turns as the provisional ownership design**
> Choice: Proceed provisionally with Agent connection ownership, reusable Plan configuration, explicit Session conversation state, and an Execution that exclusively borrows a session and owns turn-scoped callbacks. Explicit sessions declare fixed tool contracts up front and bind borrowed implementations per turn. Distinguish awaited cancellation from dropping execution; neither rolls back prior effects, and abandoned sessions must settle or become unusable before further prompts. Revisit this direction as implementation evidence develops. Exact API signatures, error types, recovery mechanics, and cleanup guarantees remain open.
> Requirements: `R-idiomatic-reimplementation`; `R-code-prompt-ergonomics`; `R-progressive-control`
> Rationale: After reviewing the scoped-session proposal and integration experiments, the owner explicitly chose it as the provisional plan, with re-review as the pieces come together. This selects a working design, not a verified product guarantee or implementation assignment.
> Alternatives:
> - Retain borrowed callbacks for the entire session: simpler lifetime alignment, but keeps application state borrowed between turns.
> - Require owned static callbacks everywhere: simplifies detached execution but makes local state access more cumbersome.
> Reconsider when: Implementation exposes problems with stale-call isolation, agent tool-catalog behavior, cancellation/cleanup, helper composition, or first-use ergonomics; the owner explicitly retained the option to re-review.

> **`D-connection-vocabulary`** (accepted): **Use connect for agents and open for sessions**
> Choice: Use connect to establish an agent connection and open to create a conversation session. Provide built-in default-preset shorthand such as Claude::connect() through an inherent associated function, alongside the extension-trait instance form Claude::default().connect(). Both follow the same connection path. Do not use connect_default or repurpose open for this shorthand. Exact trait names and behavioral connector contracts remain open.
> Requirements: `R-easy-agent-programming`; `R-progressive-control`; `R-idiomatic-reimplementation`
> Rationale: The owner explicitly agreed to this naming and shorthand after reviewing the Rust method-resolution alternatives. It distinguishes connections from conversations while keeping default connection syntax short.
> Alternatives:
> - connect_default: longer without a meaningful ergonomic gain.
> - Claude::open: technically possible but would reuse the session verb for a connection operation.
> - launch or spawn: implies creation of a process and does not cover attaching to an existing connection.
> Reconsider when: Integrated API review reveals a naming conflict or a different ownership abstraction changes the distinction between connecting and opening a conversation.

> **`D-reviewed-api-baseline`** (accepted): **Use the reviewed integrated API as the provisional implementation basis**
> Choice: Adopt the integrated API review's RAII-based design as the working basis for scoped implementation: typed agent construction, scoped per-turn tools, changing result types, automatic local cleanup with owned-driver teardown, and optional awaited shutdown confirmation. Assess the stable result-envelope and runtime mechanisms through the proposed bounded foundation milestone. Preserve explicitly unresolved choices, including error/repair defaults, public type details, embedding contracts, and exact lifecycle guarantees, for evidence-driven refinement.
> Requirements: `R-easy-agent-programming`; `R-progressive-control`; `R-code-prompt-ergonomics`; `R-per-turn-result-types`; `R-explicit-authority`
> Rationale: On 2026-11-19 the owner explicitly stated, “I reviewed integrated-api-review.md and approve of the design.” This completes design approval while retaining the previously agreed provisional, revisable character of the proposal. It does not authorize an unspecified implementation milestone or publication.
> Alternatives:
> - Require explicit cleanup at every call site: rejected by the owner as error-prone and confusing.
> - Require scoped callbacks for ordinary lifecycle management: the owner objected to the rightward drift and preferred exploring destructor-based cleanup.
> Reconsider when: Implementation evidence contradicts the proposed ownership, interoperability, validation, or cleanup behavior; resolve affected open choices with the owner rather than silently changing the design.

> **`D-initial-cancel-invalidation`** (accepted): **Initially make cancelled conversations unusable**
> Choice: In the initial API, cancelling or abandoning a started turn makes its session unusable for subsequent prompts. Release local callback access and initiate owned-resource cleanup; do not silently reset history or require cancellation recovery in the initial implementation. This does not promise rollback or synchronous remote acknowledgment.
> Requirements: `R-code-prompt-ergonomics`; `R-idiomatic-reimplementation`
> Rationale: After reviewing the runtime/result prototype, the owner explicitly agreed that cancelled sessions can remain unusable initially. This selects the conservative policy already exercised by the experiment.
> Alternatives:
> - Recover and reuse a cancelled session after settlement: deferred until there is an assigned recovery contract and evidence.
> Reconsider when: Applications need conversation recovery after cancellation and its settlement and callback-routing guarantees can be specified and tested.


> **`D-initial-conversation-scope`** (accepted): **Ship the bounded typed-conversation product slice first**
> Choice: Implement the approved first product slice using Tokio, owned subprocess ACP connections, one active fixed-catalog session per connection, per-turn borrowed tools and typed results, library-managed turn envelopes, and the documented finite schema subset. Target macOS/Linux with the current Rust toolchain and Claude ACP 0.23.1 interoperability; keep the explicitly deferred API and platform surfaces outside this assignment.
> Requirements: `R-easy-agent-programming`; `R-code-prompt-ergonomics`; `R-per-turn-result-types`; `R-idiomatic-reimplementation`
> Rationale: The owner reviewed the concrete first-product-slice proposal and said “proposal looks good, proceed.” The scope translates the approved integrated design and experimental findings into a first product assignment without declaring universal protocol or schema support.
> Alternatives:
> - Promote every experimental helper or implement the whole reviewed API at once: exceeds the explicitly bounded slice and carries unreviewed experimental mechanisms into the product.
> Reconsider when: Product evidence or application needs justify broader schema/runtime/platform support or one of the deferred API surfaces.
