# Porting research (migration-only)

This chapter is temporary migration evidence during the [Porting Phase](lifecycle.md).
Observed legacy behavior is not an accepted Thinkwell requirement. This is inherited
scaffolding research, not a fresh upstream inspection or a selected compatibility
baseline. No ongoing revision tracking is required.

## Inspected source and concepts

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

## Source provenance

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

## Unresolved migration questions

Which capabilities and observable behavior, APIs, tests, examples, and documentation
should inform the port? Which revision should research use, and which differences
should become explicit Rust-native decisions? No compatibility scope is accepted.
For each adaptation, establish file-specific rights, notices, source revision, and
affected portions before copying; the project license alone is not clearance.

Keep TypeScript inventories, parity analysis, source links, and migration notes
here (or in migration-only chapters), with accepted Rust requirements and designs
in their durable owning chapters. Do not create runtime, build, test, packaging,
submodule, CI, release, or contributor workflow dependencies on the legacy repository.

## Removal

At the human-approved transition, remove this chapter and its SUMMARY entry and
incoming migration links. First retain any legally required notices and provenance
in durable records alongside adapted material. No substantive implementation has
been adapted so far. Migration research must not be necessary to interpret the
remaining requirements, designs, tests, or delivery records.
