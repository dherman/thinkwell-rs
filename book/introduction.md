# Introduction

This repository, [dherman/thinkwell-rs](https://github.com/dherman/thinkwell-rs),
will hold an idiomatic Rust reimplementation of
[Thinkwell](https://github.com/dherman/thinkwell). Thinkwell combines deterministic
code with LLM-powered reasoning through a fluent API. The Rust library is named
`thinkwell`, as explicitly requested by the project owner; publication and wider
project identity remain separate questions. No affiliation claim is made here.

The [Anchor Book method](https://anchor-book.github.io/anchor-book/) organizes
version-controlled project knowledge. Authority records state requirements and
accepted decisions; design chapters define the intended system; roadmaps record
assigned work, results, and gaps. Acceptance, implementation, and verification
are distinct. The phase is configured in the root `anchorbook.toml`.

## Upstream context

The Phase 0 research inspected Thinkwell's README, AGENTS guidance, manifests,
public exports, examples, and completed MVP, functional-plan, and conductor RFDs.
The source snapshot was
[`2393101fd66b8471f33bb3a4908aeee919fdf0ce`](https://github.com/dherman/thinkwell/tree/2393101fd66b8471f33bb3a4908aeee919fdf0ce).

Its public surface includes agent opening and lifecycle, sessions, immutable
fluent plans, structured output schemas, tools, skills, and streaming thought
events. The examples combine ordinary deterministic code with model calls and
typed results. These concepts inform later design research; they do not settle
Rust compatibility or implementation choices.

The TypeScript monorepo separates its high-level library, ACP handling, protocol
types, and conductor, alongside editor tooling and examples. Historical designs
describe an external Rust conductor and its later TypeScript replacement.
Neither historical implementation constraints nor package boundaries determine
this Rust workspace's eventual crates.

Reference entry points:

- [Public API exports](https://github.com/dherman/thinkwell/blob/2393101fd66b8471f33bb3a4908aeee919fdf0ce/packages/thinkwell/src/index.ts)
- [MVP design](https://github.com/dherman/thinkwell/blob/2393101fd66b8471f33bb3a4908aeee919fdf0ce/doc/rfd/completed/mvp.md)
- [Immutable plans](https://github.com/dherman/thinkwell/blob/2393101fd66b8471f33bb3a4908aeee919fdf0ce/doc/rfd/completed/functional-plan.md)
- [Conductor design](https://github.com/dherman/thinkwell/blob/2393101fd66b8471f33bb3a4908aeee919fdf0ce/doc/rfd/completed/conductor.md)
- [Examples](https://github.com/dherman/thinkwell/tree/2393101fd66b8471f33bb3a4908aeee919fdf0ce/examples)

## Licensing context

The owner selected `MIT OR Apache-2.0` for original project code and documentation,
as recorded in `D-dual-licensing`. Recipients may choose either license. Complete
texts are in the repository-root `LICENSE-MIT` and `LICENSE-APACHE`; both Cargo
packages inherit the shared license declaration. The root `CONTRIBUTING.md`
defines submission under both licenses unless explicitly agreed otherwise.

The four core package manifests and the high-level package README declare MIT.
The inspected tree has no root license file; its explicit license file is under
the VS Code extension. That MIT text requires copyright and permission notices
to accompany copies or substantial portions. Its location alone does not establish
applicability to every source file or any third-party material.

Before copying or adapting implementation, independently establish the applicable
license, copyright holders, and required notices for the selected files. Preserve
those notices and document the source revision and affected portions. Choosing
Apache-2.0 for this project does not remove upstream MIT notice obligations or
establish an Apache patent grant from MIT-only upstream authors. Separately
identified third-party material retains its own terms. Phase 0 copies no
substantive upstream implementation; no upstream code attribution is claimed yet.

License references: [MIT terms](https://opensource.org/license/mit),
[Apache-2.0 terms](https://www.apache.org/licenses/LICENSE-2.0), and
[Apache's compatibility guidance](https://www.apache.org/legal/resolved.html#category-a).
