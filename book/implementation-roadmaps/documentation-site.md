# Rust documentation site

Roadmap identifier: `documentation-site`.

## Milestone 1. Audit and establish the learning path {#documentation-site-1}

**Identifier:** `documentation-site-1`

**Status:** In progress

**Outcome:** Review the TypeScript website, propose navigation for Rust learners,
and prepare a complete first typed-answer tutorial for owner review.

**Assignment:** The owner approved this bounded next step with “proceed” after
proposing porting the Mintlify site to thinkwell.rs. The preparation milestone
became Ready and is assigned In progress. Site-wide implementation, hosting,
DNS, publication, commits and pushes are not assigned by this step.

**Dependencies and basis:** Integrated API through PR #11 at 5644d6f;
`R-easy-agent-programming`, `R-progressive-control`, `R-code-prompt-ergonomics`,
`R-source-provenance`. The earlier scope review and guide experiment remain
unintegrated; this assignment does not accept their proposed deferrals.

**Tasks:** Audit source/navigation and Rust differences; propose the learning path;
write and compile the first tutorial; record publication prerequisites separately.

### Completion criteria

#### Source audit and proposed navigation {#documentation-site-1-audit}

- **Anchor coverage:** None — documentation preparation, no new product guarantee.
- **Criterion:** Trace the proposed pages to inspected source and current Rust
  behavior; identify obsolete claims, provenance concerns and delivery choices.
- **Evidence:** Audit and proposed navigation below, with source revision and
  explicit limits. Owner review remains outstanding.

#### First tutorial {#documentation-site-1-tutorial}

- **Anchor coverage:** None — teaching existing behavior.
- **Criterion:** Provide complete setup and typed-answer code, explain the core
  concepts in beginner language, and compile the documented program.
- **Evidence:** The complete Rust and TOML blocks were extracted from
  `website/get-started/first-answer.mdx` into ignored `target/website-first-answer`.
  Only the local dependency path was made absolute and an empty workspace added
  for this isolated check. `CARGO_TARGET_DIR="$PWD/target/guide-disclosure" cargo
  check --manifest-path target/website-first-answer/Cargo.toml` and `cargo fmt
  --manifest-path target/website-first-answer/Cargo.toml -- --check` pass on
  5644d6f plus the uncommitted documentation draft. No model request was run;
  sample output is explicitly illustrative. Clean-machine installation and
  authentication were not exercised.

#### Local review and delivery plan {#documentation-site-1-review}

- **Anchor coverage:** None — preparation/review.
- **Criterion:** Check the Book and documentation claims, distinguish a draft from
  a rendered/published site, and identify the next bounded implementation step.
- **Evidence:** `cargo anchor check --config anchorbook.toml`, `mdbook build`
  (with target/tools/bin on PATH), and `git diff --check` pass. Rendered navigation
  and milestone anchors were inspected. An author review checked preset version,
  dependency setup, supported types, fresh-conversation semantics, cleanup wording,
  and publication claims against the Rust sources. No Mintlify rendering, GitHub CI,
  deployment or DNS operation is claimed. Owner review and integration remain open.

## Source audit

Inspected local TypeScript website navigation and page content at revision
`2393101fd66b8471f33bb3a4908aeee919fdf0ce` in dherman/thinkwell.
Source: [website tree](https://github.com/dherman/thinkwell/tree/2393101fd66b8471f33bb3a4908aeee919fdf0ce/website),
[public tutorials](https://thinkwell.sh/examples).
This is migration evidence, not authority for Rust behavior. Current Rust code,
preset docs and integrated example walkthroughs determine the draft's API claims.

| Source area | Useful teaching idea | Rust treatment |
| --- | --- | --- |
| Introduction and quickstart | Start with a concrete typed answer and explain each part | Fresh tutorial using Cargo, JsonSchema/Deserialize derives, Plan::run and automatic cleanup. No published crate assumed. |
| CLI reference | Make installation and execution explicit | Replace with Cargo setup/run instructions; no Rust Thinkwell CLI, Homebrew release or TS schema build step. |
| Coding agents | Help users obtain accurate generated examples | Defer a Rust coding skill until docs/API guidance exists; do not advertise the TS skill as Rust-compatible. |
| API overview, agent and sessions | Introduce connection, prompt and history separately | Teach a one-shot plan first, then run_in and changing answer types; only Claude/Codex presets are evidenced. No cwd, environment-override or concurrent-session parity claims. |
| Plan reference | Compose instructions, context and tools | Teach text/json/code, inline borrowed tools and observe. No skill/quote/textln/ThoughtStream parity claims. Rustdoc owns exhaustive signatures. |
| Hello world | Tiny complete runnable application | Fold the first answer into quickstart; teach a tool on the next page rather than adding tools to the first program. |
| Sentiment analysis | Let a model call deterministic analysis | Adapt the integrated Rust walkthrough with borrowed callback/log, not the TS implementation. |
| Meeting scheduler | Alternate interpretation, exact computation and explanation | Use the Rust solver example and its validation limits. A solver validates its encoded constraints, not the model's interpretation of human intent. |
| JavaScript unminifier | Combine language understanding with mechanical rewriting | Use the integrated Rust pipeline: deterministic discovery and sequential prompt batches. Do not copy TS parallelism, UMD conversion or broad correctness claims. |
| Community | Give contributors a clear route into the project | Link Rust CONTRIBUTING and the Book. Do not import TS release processes or adopt a community policy implicitly. |

The source docs.json uses Mintlify's maple theme, fonts/colors, local logo assets,
and four tabs. Several legacy redirects target pages absent from this source tree
(for example get-started/architecture and get-started/registry); do not carry those
redirects into the Rust site automatically. Website assets and community policy
text have not been cleared for copying. No root LICENSE file or website-local
license was found in this checkout; package licensing is not sufficient evidence
for adapting all site content. This draft uses original prose/code and copies no
TS prose, code, logos, configuration or Contributor Covenant text. Any later asset
or text reuse needs its applicable license/notices verified first.

## Proposed navigation

Use Mintlify for the learning site and Rustdoc for exact API details. This keeps
one authoritative signature reference while giving newcomers a task-oriented path.
Only the first tutorial is drafted; these are proposed titles, not existing routes.

- **Learn:** What is Thinkwell?; Your first typed answer; Working with structs and
  enums; Adding context; Giving the agent a tool; Continuing a conversation;
  Showing progress.
- **Examples:** Sentiment analysis; Meeting scheduler; JavaScript unminifier.
- **Reference:** Agent setup (Claude/Codex and custom commands); Supported answer
  types; Errors and cleanup; Rust API reference.
- **Contribute:** Link the repository contribution guide and project Book.

Introduce each concept when a reader needs it. Start tools with Plan::tool and a
borrowed Rust function. Introduce explicit tool catalogs only when teaching history.
Explain ACP briefly as the connection protocol; do not begin with the transport,
MCP envelopes, worker architecture, or cancellation internals. Put actionable
limitations beside relevant operations, not in a long opening architecture tour.

## Delivery proposal

Next, after review of the sample and navigation, implement the Mintlify scaffold
and core learning pages in website/. Select a pinned CLI compatible with the
existing configuration format, validate MDX and links, and compile tutorial code
in CI without inference. Avoid maintaining unchecked duplicate code snippets.
The owner subsequently requested a docs.json for local preview. A minimal
configuration now includes the existing tutorial and Rust repository link, with
preview instructions in website/README.md. JSON parsing and navigation targets
were checked; browser rendering has not been verified. This narrow follow-up
is part of the assigned preparation work, not approval for the remaining pages.

Keep thinkwell.sh as the TypeScript documentation and label thinkwell.rs as Rust.
Do not redirect the whole TypeScript site. Add cross-links when each destination
exists and its site update is authorized. Keep the project Book separate from
user tutorials.

Before publishing, confirm the owner's Mintlify project/account, repository and
website/ source-directory configuration, plan support for the custom domain,
and deployment branch. Obtain the actual DNS records from the chosen host rather
than guessing CNAME targets; verify TLS and canonical URLs after deployment.
The owner controls thinkwell.rs, but no DNS state or hosting configuration was
inspected or changed in this assignment.

Rustdoc currently exists as a downloadable CI artifact, not a public website.
Choose a durable Rustdoc host/path before adding a public reference link; docs.rs
is a future option after crate publication, not a working link today. Neither a
crate release nor Rustdoc hosting is implied by the documentation-site proposal.

Keep this milestone In progress until review and integration. A complete website,
public availability and the end of the Porting Phase are separate outcomes.

## Milestone 2. Core learning pages {#documentation-site-2}

**Identifier:** `documentation-site-2`

**Status:** In progress

**Outcome:** Extend the approved first-tutorial style into a navigable core Rust
learning path with checked code and Mintlify validation.

**Assignment:** The owner reviewed the tutorial positively and instructed “Let's
continue with the docs.” This next preparation milestone became Ready and is now
In progress. The earlier draft remains unintegrated. No publishing, DNS changes,
commit/push, library API change or lifecycle transition is assigned.

**Dependencies and basis:** Owner-reviewed first tutorial; current typed-conversation
API; `R-easy-agent-programming`, `R-progressive-control`, `R-code-prompt-ergonomics`.

**Tasks:** Write an introduction and guides for answer types, context, tools,
history and progress; connect navigation; compile complete Rust examples; validate
site syntax/links with a pinned Mintlify CLI. Detailed example ports and reference
pages remain subsequent work.

### Completion criteria

#### Learning path {#documentation-site-2-content}

- **Anchor coverage:** None — documentation of existing behavior.
- **Criterion:** Introduce concepts progressively through concrete Rust programs,
  with setup links and actionable limits; advertise only implemented capabilities.
- **Evidence:** Seven navigable pages now cover introduction, first answer,
  answer types, context, tools, conversations and progress. Original prose and
  complete programs use current Rust APIs. The first tutorial links forward;
  later guides link prerequisites and next steps. Owner approved the initial
  tutorial's style; review of these additional pages remains outstanding.

#### Executable examples {#documentation-site-2-examples}

- **Anchor coverage:** None — documentation consistency check.
- **Criterion:** Compile every complete Rust example without inference and make
  the check reproducible without maintaining duplicate source snippets.
- **Evidence:** `CARGO_TARGET_DIR="$PWD/target/guide-disclosure" python3
  website/scripts/check_examples.py` passes: nine complete Rust programs compile
  and pass rustfmt, with navigation/local-page links checked. The script extracts
  MDX directly and uses the first tutorial's dependency list with a local path
  substitution. Generated manifests/lockfiles stay under ignored target/.
  This checks compilation, not live model behavior or all runtime schemas; no
  new inference was run. The checks are not yet installed in GitHub CI.

#### Site validation {#documentation-site-2-validation}

- **Anchor coverage:** None — documentation delivery.
- **Criterion:** Validate navigation, MDX and links; run Book checks and distinguish
  local validation from browser review, CI and publication.
- **Evidence:** From website/, `npx --yes mint@4.3.84 validate --telemetry false`
  passes validation and strict build; `npx --yes mint@4.3.84 broken-links
  --telemetry false` reports no broken links. Book structural check, mdBook build
  and whitespace check pass on base 5644d6f plus uncommitted documentation changes.
  Rendered Book milestone anchors/navigation were inspected. An author review
  corrected the code-language hint description to match actual validation
  (backticks/control characters rejected, not all whitespace). No browser visual
  review, GitHub CI, publication or DNS change is claimed. Owner review and
  integration remain outstanding.

## Milestone 3. Worked examples and practical reference {#documentation-site-3}

**Identifier:** `documentation-site-3`

**Status:** In progress

**Outcome:** Connect the core learning path to the three integrated Rust examples
and document agent setup, supported values, failure handling and API lookup.

**Assignment:** The owner reviewed the core pages positively and requested continued
work after the proposed example/reference follow-up. This milestone became Ready
and is assigned In progress. Existing documentation remains unintegrated. Commit,
push, hosting, DNS, release and product API changes are not authorized.

**Dependencies and basis:** Integrated example source and walkthroughs at 5644d6f;
current Rust API; `R-easy-agent-programming`, `R-progressive-control`,
`R-code-prompt-ergonomics`, `R-source-provenance`.

**Tasks:** Add three example walkthroughs and four reference pages; update navigation
and links; compile new complete samples; run Mintlify and Book validation.

### Completion criteria

#### Practical coverage {#documentation-site-3-content}

- **Anchor coverage:** None — teaching existing behavior.
- **Criterion:** Examples explain the code/prompt division, how to run the actual
  programs and their limits. Reference pages cover tested agents, model selection,
  value support, errors/cleanup and access to unpublished Rustdoc.
- **Evidence:** Three original site walkthroughs explain the integrated sentiment,
  scheduler and unminifier programs and link their actual Rust source. Four reference
  pages cover Claude/Codex setup, model selection/custom commands, supported values,
  request limits/errors/cleanup and access to local or downloadable Rustdoc. All
  fourteen site pages are navigable; no unavailable docs.rs or hosted-reference
  link is advertised. Source was checked against the current Rust implementations
  and repository walkthroughs; no TypeScript text or assets were copied.

#### Validation and delivery {#documentation-site-3-validation}

- **Anchor coverage:** None — documentation consistency.
- **Criterion:** Compile complete snippets; validate site and links; check and render
  Book; distinguish fresh checks from historical product evidence.
- **Evidence:** The MDX extraction check compiles thirteen complete Rust programs
  and passes rustfmt plus navigation/local-link checks. Exact command:
  `CARGO_TARGET_DIR="$PWD/target/guide-disclosure" python3 website/scripts/check_examples.py`.
  From website/, `npx --yes mint@4.3.84 validate --telemetry false` passes content
  validation and strict build; `npx --yes mint@4.3.84 broken-links --telemetry false`
  reports no broken links. `cargo anchor check --config anchorbook.toml`, mdBook
  build and `git diff --check` pass on 5644d6f plus this uncommitted docs tree.
  Rendered Book navigation and criterion anchors were inspected. An author review
  checked source/command paths, preset versions, defaults and behavioral limits.
  Example executable tests and live inference were not rerun: product code is
  unchanged and its integrated evidence remains historical. No browser visual
  review, GitHub CI, commit/push, publication or DNS change is claimed. Human
  review and integration remain outstanding.


### Self-contained tutorial revision

The owner requested that the example pages include relevant code directly rather
than require following repository files alongside the prose. This is a revision
within documentation-site-3. The TypeScript tutorial presentation was refreshed
as migration context: data definitions, code beside each step, and concrete output.
Only current repository-owned Rust code is excerpted; no TS text/code is adapted.
The three pages now show their request flow, types, computation and result handling
inline. Source links remain optional for deeper helper implementation details.
The website check now distinguishes complete programs (compiled) from titled
excerpts (matched against their indicated source, allowing indentation changes).
Validation of this revision passes: the existing MDX check compiles thirteen
complete programs and matches twenty excerpts against repository source; Mintlify
4.3.84 validation/build and broken-link checks pass. Book structural checking,
mdBook rendering and whitespace checks also pass. An author review corrected
indentation and removed an unmatched closing brace from a partial source excerpt.
No new model requests or product changes were needed. This remains local,
uncommitted work awaiting owner review and integration.


### Import convention

The owner requested `crate::` for imports from an example's or documented program's
own crate. CONTRIBUTING records that convention. The unminifier library's internal
Candidate import now uses `crate::javascript`; its binary still names the library
crate because binary and library are separate Rust crates. Dependency imports in
tutorial programs and Rustdoc examples remain explicit. No module layout or API
changed. The unminifier locked all-target cargo check and rustfmt pass, as does the website
check (thirteen complete programs and twenty source excerpts). Book structural
checking, rendering and whitespace checks pass. Changes remain uncommitted.


### Documentation delivery

The owner approved committing and pushing the documentation without a PR. The
website, documentation roadmap/navigation, and import-convention follow-up are
included. Earlier port-scope research and the guide-disclosure experiment remain
separate local work. The owner reviewed the tutorials as a good starting point;
this author review checked the delivery scope and source excerpt consistency.
Push does not imply merge, publication, or completion of the Porting Phase.
Documentation milestones remain In progress pending integration.

## Milestone 4. Documentation CI {#documentation-site-4}

**Identifier:** `documentation-site-4`

**Status:** In progress

**Outcome:** Run tutorial compilation, source-excerpt checks and Mintlify validation
on pushes and pull requests without model credentials or deployment.

**Assignment:** After documentation commit 756cd93 was pushed, the owner approved
adding documentation checks to CI with “proceed”. This milestone became Ready
and is now In progress. No hosting, DNS, release or new API work is assigned.

**Dependencies and basis:** The checked-in site and validation script at 756cd93;
`R-easy-agent-programming`, `R-reviewed-delivery` and the existing CI workflow.

**Tasks:** Add a bounded documentation job with pinned tools; document its local
commands and limits; validate the workflow and run the commands locally.

### Completion criteria

#### Automated checks {#documentation-site-4-checks}

- **Anchor coverage:** None — documentation consistency, not new product behavior.
- **Criterion:** CI compiles all complete tutorial programs, matches source excerpts,
  checks formatting/navigation, validates Mintlify content/build and checks links.
  It neither invokes agents nor publishes the site.
- **Evidence:** A separate Documentation site job now runs all three existing
  website check commands on push and pull_request. It uses Ubuntu 24.04, Rust
  1.100.0, Node 24.12.0 and Mintlify CLI 4.3.84, with a 15-minute timeout and
  SHA-pinned checkout/setup-node actions. It uses only contents: read permissions,
  no agent credentials or deployment. Contributor and website instructions match
  the job. Tutorial dependency resolution intentionally follows the documented
  version ranges, not a newly introduced release lockfile.

#### Validation and delivery {#documentation-site-4-validation}

- **Anchor coverage:** None — CI setup and delivery.
- **Criterion:** Check workflow syntax and run its validation commands locally;
  record tested state and distinguish local results from an actual GitHub run.
- **Evidence:** Local checks on 756cd93 plus this CI/documentation change pass:
  website/scripts/check_examples.py (13 complete programs, 20 excerpts, formatting
  and local links); Mintlify 4.3.84 validate (including build) and broken-links;
  `go run github.com/rhysd/actionlint/cmd/actionlint@v1.7.7 -shellcheck=
  .github/workflows/ci.yml` (workflow lint, without ShellCheck); Book structural
  check, mdBook build and whitespace check. The local Node version is 24.12.0.
  An author review checked the commands, action SHA, time limit and absence of
  deployment/model work. No GitHub execution or Linux result is claimed yet.
  The owner authorized committing and pushing this workflow follow-up without a PR.
  Integration is outstanding; a push alone does not establish a successful GitHub run.
