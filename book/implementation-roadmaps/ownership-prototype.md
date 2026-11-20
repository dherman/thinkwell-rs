# Ownership API prototype

Roadmap identifier: `ownership-prototype`.

Current integration state: completed work was delivered through owner-merged
[PR #2](https://github.com/dherman/thinkwell-rs/pull/2). See the
[shared integration evidence](first-product-slice.md#integrated-completion).
Earlier local-stage evidence and pending-delivery statements below are historical;
they retain their tested scope and do not describe current integration status.

Goal: assess Rust ownership and ergonomics for the proposed fluent API through
an isolated simulated driver, before choosing product API signatures or ACP libraries.

## Milestone 1. Assess scoped execution {#ownership-prototype-1}

**Outcome:** Compile-checked examples and rejection cases establish the practical
tradeoffs of borrowed callbacks, reusable plans, helper composition, and observation.

**Status:** Complete

**Identifier:** `ownership-prototype-1`

**Dependencies:** The owner explicitly approved preparation and execution of this
five-case experiment in the port-design conversation on 2026-11-18. This scoped
prototype is authorized while the lifecycle record synchronization remains local;
it does not declare that delivery milestone complete or authorize publication.

**Design basis:** `R-easy-agent-programming`, `R-progressive-control`,
`R-code-prompt-ergonomics`, `D-acp-client`, and `D-derived-contracts`;
[design](../design/index.md) and [research](../open-questions.md).
The experimental design below is approved for learning, not accepted product API.

**Tasks**

- [x] Build an isolated simulated agent with typed callback dispatch and fixture results.
- [x] Exercise five usage patterns and meaningful compile-fail boundaries.
- [x] Compare callback storage and async adapter options on the pinned compiler.
- [x] Review the experiment and record evidence, recommendations, and limitations.

### Experimental design and scope

Use standalone Rust sources under `experiments/ownership-api`, compiled with the
pinned toolchain and a local harness. Keep dependencies and product code unchanged.
Use typed in-memory requests and fixture outputs, not schemas, serialization, ACP,
MCP, real agents, or derived contract implementation. This tests ownership and
execution, not model behavior or protocol integration. Explore erased scoped
callback storage and generic alternatives, with an explicit reusable/scoped
boundary where necessary. Drive borrowed operations by polling their futures;
observation must not require spawning a borrowed future. Cancellation policy,
production event buffering, session protocol behavior, and publication are excluded.

### Completion criteria

#### Basic composition {#ownership-prototype-1-composition}

A fluent plan executes a typed async tool borrowing application state and returns
a typed fixture result; tool failures and incorrect fixture types are distinguishable.

- **Anchor coverage:** None — API experiment, no product guarantee.
- **Evidence:** Passed `borrowed_read_only_tool_and_typed_result`, `typed_errors_and_cleanup`,
  and `output_inference_and_direct_await`. Tools borrow application state;
  the simulated driver distinguishes callback errors, incorrect input/result
  types, and unknown tools. Result types can be inferred at the receiving binding.
  Results are supplied fixtures, not computed model responses; tool outputs are
  discarded by this driver. No schema or serialization behavior is established.

#### Exclusive capture {#ownership-prototype-1-capture}

A tool mutates local state across async suspension; the caller can access it after
execution. The compiler rejects overlapping exclusive access and escaping borrows.

- **Anchor coverage:** None — API experiment, no product guarantee.
- **Evidence:** Passed `exclusive_capture_survives_suspension_and_returns_to_caller`:
  two calls mutate the same local vector before and after explicit Pending
  boundaries. `overlapping_access` and `two_mutable_tools` are rejected with
  E0499; `escape` is rejected with E0515. Captures are available after the owned
  execution future is consumed/dropped. This tests single-threaded scoped work,
  not cross-thread execution or remote cancellation.

#### Reusable plans {#ownership-prototype-1-reuse}

Independent variations of a base plan retain distinct configuration and can attach
different scoped tools. Unsupported cloning of exclusive callbacks is demonstrated.

- **Anchor coverage:** None — API experiment, no product guarantee.
- **Evidence:** Passed `clone_base_and_execute_variations_concurrently`: base prompt
  configuration remains unchanged; two variations attach different local tools
  and execute as interleaved futures with independent effects. `clone_scoped`
  is rejected with E0599. The experiment conservatively prohibits cloning every
  scoped plan; it does not implement shared reusable tools or conditional Clone.

#### Helper composition {#ownership-prototype-1-helpers}

Helper functions and conditional tool registration compile with the chosen storage;
generic-storage costs and any adapter-macro need are assessed explicitly.

- **Anchor coverage:** None — API experiment, no product guarantee.
- **Evidence:** Passed `conditional_helpers_and_heterogeneous_tools` for enabled and
  disabled branches. Callback erasure preserves one scoped type across different
  tool types. `generic_helper` compiles, while `generic_conditional` is rejected
  with E0308. Local erased AsyncFnMut invocation requires no caller adapter on
  Rust 1.100.0. `send_without_adapter` is rejected because the invocation future
  is not known to be Send; `send_adapter` compiles and runs using an explicit
  higher-ranked boxed-future adapter. This assesses these signatures, not the
  impossibility of other Send APIs or compatibility with an ACP dependency.

#### Observable execution {#ownership-prototype-1-observation}

Progress can be consumed while borrowed callbacks execute. Dropping observation
does not stop the result, and ignored events do not accumulate without bound.

- **Anchor coverage:** None — API experiment, no product guarantee.
- **Evidence:** Passed `observe_events_while_borrowed_tool_is_running`,
  `dropping_observation_mid_run_does_not_cancel`, and
  `unread_events_are_bounded_and_end_after_result`. Events and results are polled
  together while callbacks borrow local state. Dropping observation after a tool
  starts permits both scripted calls to finish. With an unread observer, 21 events
  leave two queued and 19 reported dropped. The bounded, lossy queue is an
  experimental mechanism, not an accepted product event policy; execution must
  be driven by polling the result future.

### Evidence

Tested locally on macOS arm64 with `rustc 1.100.0 (0ef1bc386 2026-11-09)`.
Revision context: the uncommitted `port-design` working tree based on
`83ac7e6faf7b4e7dfd74c306436760e44ba46a21`, including the new experiment and Book
updates. Nothing is committed, pushed, or integrated by this assignment.

Reproducible checks:

- `python3 experiments/ownership-api/check.py`: nine runtime tests, two positive
  compiler probes, and six expected compiler rejections passed. Positive sources
  compile with warnings denied. The harness stores negative diagnostics under
  ignored `target/ownership-api/` and applies a 60-second subprocess timeout.
- `rustfmt --edition 2024 --check experiments/ownership-api/*.rs experiments/ownership-api/cases/*.rs`:
  passed.
- Direct `clippy-driver` checks with `--edition=2024 -D warnings` passed for
  `prototype.rs`, `scenarios.rs` (test mode), and the two positive compiler probes.
  These standalone sources are intentionally outside the Cargo workspace.
- `cargo anchor check --config anchorbook.toml`: structural/v1 passed, zero errors.
- mdBook 0.5.4 build passed; new navigation, criterion sections, and research links
  were inspected in generated HTML.
- `cargo test -p anchor-runner --test bootstrap_consistency --locked`: passed.
- `git diff --check` and separate new-source whitespace inspection: passed.

A deliberate post-implementation review by the implementing agent checked all
new sources, the five criteria, compiler diagnostics, and evidence scope. This is
self-review, not independent or human review. The initial harness expected E0373
for escaping a local capture; Rust reports E0515 for this returned-plan form.
The expectation was corrected to that specific borrow error without changing
or permitting the invalid program. Review also made explicit that callback
outputs are discarded, the Send probe covers only one invocation, and a boxed
completed future may retain captures until dropped. No broad concurrency,
protocol validation, or cleanup claim is inferred from these tests.

The experiment uses original code with no source adaptation or new dependencies.
No product code, manifests, CI workflow, or toolchain pins changed. No live-agent,
ACP/MCP, schema/derive, multithreaded execution, performance, or mutation tests
were run. Formal proofs are not required by any product anchor; none is defined.

### Findings and recommendations

The local scoped model supports the proposed fluent style: typed results, tools
borrowing mutable application data, cloneable base prompts, and optional progress
observation. Erasing callback types simplifies conditional registration compared
with a generic chain. A reusable/scoped marker still needs normalization with
`scoped()` when conditional branches differ in whether a tool is added.

Keep the local API as a design candidate, not an accepted runtime choice. The
next design decision is whether execution futures must be Send, whether local
and Send execution need distinct surfaces, and what the selected ACP library
requires. The tested Send adapter adds syntax but preserves a borrowed invocation;
Send does not itself imply a detached or 'static task. Shared reusable tool
registrations, session ownership, final validation, event overflow policy, and
async cleanup remain open. Review these before proposing product implementation.

Subsequent [ACP library research](../open-questions.md#current-rust-acp-library-research)
identifies scoped Send runners and a possible channel boundary for local
callbacks. It does not extend this experiment's implementation or evidence.

### Completion record

Complete. All five bounded assessments have the evidence recorded above; their
sources and evidence were integrated in owner-merged PR #2. See the
[shared delivery record](first-product-slice.md#integrated-completion).
Completion of this experiment does not promote its experimental observation or
Send adapters into supported product API. No further prototype expansion is assigned.
