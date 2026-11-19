# Porting research (migration-only)

This chapter is temporary migration evidence during the [Porting Phase](lifecycle.md).
Observed legacy behavior is not an accepted Thinkwell requirement. This chapter
includes inherited scaffolding research and the fresh API review below; neither
selects a compatibility baseline. No ongoing revision tracking is required.

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

## API design review

At the owner's request on 2026-11-18, inspected the current upstream HEAD,
`2393101fd66b8471f33bb3a4908aeee919fdf0ce`, which matches the earlier snapshot.
Read both READMEs, all six top-level TypeScript examples (greeting, summarization,
classification, sentiment, scheduling, and unminification), the website quickstart
and Agent/Plan/Session API pages, and the agent-api, functional-plan,
schema-providers, and conductor RFDs. Also inspected schema generation, Plan,
and Session source, explicit-configuration design, and the opening build-command
design sections. This was document/source research: no examples or tests ran and
no implementation was copied. Historical RFD proposals and documentation are
intent evidence, not proof of implementation behavior.

### Programming model observed

- **Application code owns composition.** The
  [scheduler](https://github.com/dherman/thinkwell/blob/2393101fd66b8471f33bb3a4908aeee919fdf0ce/examples/src/schedule.ts)
  uses the model to translate availability into structured constraints, Z3 to
  solve them, then another model call to explain the result. The
  [unminifier](https://github.com/dherman/thinkwell/blob/2393101fd66b8471f33bb3a4908aeee919fdf0ce/examples/src/unminify.ts)
  interleaves model judgment with deterministic formatting and AST changes;
  application code batches and limits concurrent requests.
- **Typed results are the normal entry point.** Even the greeting and summary
  examples declare result types. Classification returns a discriminated union
  consumed by an ordinary switch. Field documentation and constraints contribute
  to the schema shown to the model.
- **Tools are local callbacks.** Greeting provides a no-input time callback;
  sentiment wraps an ordinary library with a typed input callback. Callers attach
  tools beside the prompt without constructing an MCP server themselves.
- **Control is incremental.** The
  [Agent API](https://github.com/dherman/thinkwell/blob/2393101fd66b8471f33bb3a4908aeee919fdf0ce/website/api/agent.mdx)
  starts with named agents and permits custom commands. Independent calls use
  ephemeral conversational context on a reused connection; explicit sessions
  preserve context with the same prompt-building interface.
- **Plans are composable values.** The functional-plan RFD explicitly motivates
  immutability through reusable base plans, independent variants, and helpers that
  do not modify their inputs. Prompt methods distinguish instructions, quoted
  material, and code. These formatting conventions are not a security boundary.
- **Observation is optional.** The
  [Plan API](https://github.com/dherman/thinkwell/blob/2393101fd66b8471f33bb3a4908aeee919fdf0ce/website/api/plan.mdx)
  offers `run()` for the final typed value and `stream()` for events plus a separate
  result promise. Skills add stored or programmatic instructions with on-demand
  disclosure. Neither requires a different application orchestration language.

### Schema generation and runtime integration

The [schema-provider design](https://github.com/dherman/thinkwell/blob/2393101fd66b8471f33bb3a4908aeee919fdf0ce/doc/rfd/completed/schema-providers.md)
separates schema production from the runtime API. `SchemaProvider<T>` and
`schemaOf<T>()` permit alternatives to generated schemas. The optional type-first
path discovers `@JSONSchema` annotations and injects `Type.Schema`; compiler,
loader, declaration-generation, and editor integration support that convenience.
The schema machinery serves both tool contracts and final result types.

Runtime MCP integration serves a different purpose: exposing local callbacks and
carrying the structured `return_result` invocation. The conductor design bridges
MCP-over-ACP to HTTP for agents lacking the native transport. Replacing schema
generation does not itself decide the Rust transport or conductor design.

The inspected Plan source wraps non-object result schemas for tool compatibility
and performs limited required-field checks before accepting a result; it does
not establish full JSON Schema validation. Session closure is local bookkeeping
in the inspected Session source. These details qualify documentation claims and
must not silently become Rust guarantees. This review did not audit all MCP
validation, concurrency, agent support, or failure paths.

The owner's Rust direction is to use derives for type-derived contracts without
an additional schema build tool; its durable definition is in
[design](design/index.md#type-derived-contracts). The remaining observations guide
API exploration, without accepting every legacy method, mechanism, or behavior.

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
should be selected for the port after this review? Which differences should
become explicit Rust-native decisions? No compatibility scope is accepted.
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
