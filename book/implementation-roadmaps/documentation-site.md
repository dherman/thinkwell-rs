# Rust documentation site

Roadmap identifier: `documentation-site`.

Milestones 1–3 are integrated on main at `756cd93e674f81e7bb099dc156caf90427b61a68`;
milestone 4 at `59b8bd56e64cfd5a0318673421dd176316987c07`. The owner approved the
tutorial direction and direct commit/push delivery without a PR. The
[completion record](#integrated-completion) distinguishes that authorization,
local validation and successful CI. The separately authorized hosting assignment
is recorded in milestone 5 below.

## Milestone 1. Audit and establish the learning path {#documentation-site-1}

**Identifier:** `documentation-site-1`

**Status:** Complete

**Outcome:** Review the TypeScript website, propose navigation for Rust learners,
and prepare a complete first typed-answer tutorial for owner review.

**Assignment:** The owner approved this bounded next step with “proceed” after
proposing porting the Mintlify site to thinkwell.rs. The preparation milestone
became Ready before work. The later site expansion and direct delivery were
separately approved; hosting, DNS, and publication were not assigned.

**Dependencies and basis:** Integrated API through PR #11 at 5644d6f;
`R-easy-agent-programming`, `R-progressive-control`, `R-code-prompt-ergonomics`,
`R-source-provenance`. This assignment did not accept the separate scope review or guide experiment's
proposed deferrals.

**Tasks:** Audit source/navigation and Rust differences; propose the learning path;
write and compile the first tutorial; record publication prerequisites separately.

### Completion criteria

#### Source audit and proposed navigation {#documentation-site-1-audit}

- **Anchor coverage:** None — documentation preparation, no new product guarantee.
- **Criterion:** Trace the proposed pages to inspected source and current Rust
  behavior; identify obsolete claims, provenance concerns and delivery choices.
- **Evidence:** The linked source audit records the revision, Rust differences, and
  provenance limits. The owner reviewed the tutorial positively and authorized
  continuing with the learning path; the completed site is integrated as recorded below.

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
  deployment or DNS operation was established by those initial checks. Owner
  review and integration are recorded in the completion record below.

## Design and source basis

[User documentation](../design/documentation.md) owns the accepted learning path,
source-excerpt policy, and validation design. [Website source research](../migration.md#website-source-audit)
retains the pinned audit, source links, and licensing limits. No TypeScript prose,
code, logos, configuration, or community policy was copied.

Publication was outside milestones 1–4. The owner subsequently assigned the
Mintlify migration below; public Rustdoc hosting remains a separate choice.
Integrated documentation source does not itself establish publication.

## Milestone 2. Core learning pages {#documentation-site-2}

**Identifier:** `documentation-site-2`

**Status:** Complete

**Outcome:** Extend the approved first-tutorial style into a navigable core Rust
learning path with checked code and Mintlify validation.

**Assignment:** The owner reviewed the tutorial positively and instructed “Let's
continue with the docs.” This preparation milestone became Ready before work;
its approved direct delivery is recorded below. Publishing, DNS changes, library
API changes, and lifecycle transition remained outside its scope.

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
  tutorial's style and then reviewed the core pages positively before assigning
  milestone 3. Final documentation delivery is recorded below.

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
  new inference was run. Milestone 4 subsequently installed these checks in CI.

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
  review, GitHub CI, publication or DNS change was established by these local
  checks. Subsequent delivery and CI are recorded below.

## Milestone 3. Worked examples and practical reference {#documentation-site-3}

**Identifier:** `documentation-site-3`

**Status:** Complete

**Outcome:** Connect the core learning path to the three integrated Rust examples
and document agent setup, supported values, failure handling and API lookup.

**Assignment:** The owner reviewed the core pages positively and requested continued
work after the proposed example/reference follow-up. This milestone became Ready
before implementation. Direct commit/push delivery was subsequently approved;
hosting, DNS, release, and product API changes remained outside scope.

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
  review, GitHub CI, publication or DNS change was established by these local
  checks. The completion record supplies later review, integration, and CI evidence.


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
No new model requests or product changes were needed. This revision is included
in the owner-authorized documentation commit recorded below.


### Import convention

The owner requested `crate::` for imports from an example's or documented program's
own crate. CONTRIBUTING records that convention. The unminifier library's internal
Candidate import now uses `crate::javascript`; its binary still names the library
crate because binary and library are separate Rust crates. Dependency imports in
tutorial programs and Rustdoc examples remain explicit. No module layout or API
changed. The unminifier locked all-target cargo check and rustfmt pass, as does the website
check (thirteen complete programs and twenty source excerpts). Book structural
checking, rendering and whitespace checks passed. This convention and import
change are included in the integrated documentation commit.


### Documentation delivery

The owner approved committing and pushing the documentation without a PR. The
website, documentation roadmap/navigation, and import-convention follow-up are
included. Earlier port-scope research and the guide-disclosure experiment remain
separate local work. The owner reviewed the tutorials as a good starting point;
this author review checked the delivery scope and source excerpt consistency.
The documentation commit is on main; no PR merge is claimed for this explicitly
authorized direct delivery. Publication and the Porting Phase remain separate.

## Milestone 4. Documentation CI {#documentation-site-4}

**Identifier:** `documentation-site-4`

**Status:** Complete

**Outcome:** Run tutorial compilation, source-excerpt checks and Mintlify validation
on pushes and pull requests without model credentials or deployment.

**Assignment:** After documentation commit 756cd93 was pushed, the owner approved
adding documentation checks to CI with “proceed”. This milestone became Ready
before implementation. Hosting, DNS, release, and new API work remained outside
this assignment.

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
  deployment/model work. The owner authorized committing and pushing this workflow
  follow-up without a PR. Its integrated revision and actual Linux execution are
  recorded below; the local validation alone did not establish them.

## Integrated completion {#integrated-completion}

The existing approval records above establish the owner-reviewed tutorial style,
core learning path, self-contained example presentation, and authorization to
commit/push the documentation and CI follow-up without PRs. Local Git places
`756cd93e674f81e7bb099dc156caf90427b61a68` (site, walkthroughs, validation, import
convention) and `59b8bd56e64cfd5a0318673421dd176316987c07` (documentation CI) on main.
This reconciles the obsolete pending-integration statements; it invents no PR
approval or publication event.

GitHub evidence inspected on 2026-11-24 confirms successful
[documentation-commit CI](https://github.com/dherman/thinkwell-rs/actions/runs/45134008131)
and successful [CI at 59b8bd5](https://github.com/dherman/thinkwell-rs/actions/runs/45135752602).
The latter ran all five jobs: Rust and Book checks, scheduler, sentiment, unminifier,
and Documentation site. The documentation job compiled thirteen complete programs,
matched twenty source excerpts, and ran the pinned Mintlify validation and link
commands. The other successful run at that same final revision is
[45134438390](https://github.com/dherman/thinkwell-rs/actions/runs/45134438390).

All four bounded milestones are Complete. Their historical local checks retain
their original scope; CI does not establish clean-machine authentication, model
behavior, browser visual review, public hosting, or a lifecycle transition.
PR #12 integrated this status correction at 9bc0a5c.


## Integrated documentation and paused hosting assignment

The owner approved integrating the documentation into main and repurposing the
existing Mintlify project for Rust because the free personal account permits one
project. This supersedes the earlier separate-project proposal. Keeping the
TypeScript site online is no longer a requirement; ending the Porting Phase is
still a separate decision.

Main was fast-forwarded and pushed from 5644d6f to 59b8bd5, preserving documentation
commit 756cd93 and CI commit 59b8bd5. The reviewed revision passed its branch CI
and then [main CI run 45135752602](https://github.com/dherman/thinkwell-rs/actions/runs/45135752602),
including the Documentation site job. Milestones 1–4 are Complete through that
owner-authorized integration; earlier local-delivery statements above are historical.
PR #12 subsequently integrated the milestones 1–4 closeout. Hosting work below
remains a local follow-up.

The following configuration is retained for a future owner-approved resumption;
it is not an active assignment while publication is paused:

- Repository: dherman/thinkwell-rs, deployment branch main.
- Documentation subdirectory: `/website`, containing docs.json. It does not add
  `/website` to public URLs.
- Replace the custom domain with thinkwell.rs at the root. Obtain the project's
  actual verification/routing records before changing DNS and preserve unrelated
  records. Check apex support with the DNS provider and verify TLS and deep links.
- Reuse the current project; do not create a second project or upgrade the plan.
  No redirect from thinkwell.sh is required for launch.

Mintlify 4.3.84 `publish repo` and `publish domain` guidance and
[global settings](https://www.mintlify.com/docs/settings) informed this setup.
CLI authentication returned AWS / aws-8f18fdf7, initially believed to be the
wrong organization. Subsequent dashboard sign-in with the owner's personal account
showed thinkwell.sh connected under that label, along with the TypeScript Thinkwell
navigation. This makes the label alone insufficient to identify a wrong project.
The owner supplied project ID `6981adca63153ddab05ccfed`; compare it with the
Site settings before mutations. Dashboard authentication succeeded; project ID
verification remains pending. Repository integration is complete, but Mintlify
source/domain/GitHub App and DNS settings have not been changed.



## Milestone 5. Repurpose the existing hosted project {#documentation-site-5}

**Identifier:** `documentation-site-5`

**Status:** Blocked

**Pause:** The owner explicitly paused publication on 2026-11-25. Loopia
registration has not been completed, and local review is the current priority.
Do not resume Mintlify, registrar, or DNS setup without an owner request.
This milestone is incomplete, not cancelled or delivered.

**Outcome:** Serve the Rust documentation from the existing free Mintlify project
at thinkwell.rs, replacing the TypeScript deployment.

**Assignment:** The owner approved integrating main and repurposing the project
with “proceed”, after explicitly accepting loss of the old TypeScript website.
This milestone became Ready and work began, then the owner paused it as recorded
above. No paid upgrade or second project is authorized.

**Dependencies and basis:** Integrated documentation milestones 1–4;
`R-easy-agent-programming`, `R-reviewed-delivery`; owner-selected existing project.

**Tasks on resumption:** Confirm project identity; change source to Rust main and
/website; retrieve actual DNS records, configure domain and verify public serving.
Previous authentication succeeded but must not be assumed to remain valid.

### Completion criteria

#### Project source {#documentation-site-5-source}

- **Anchor coverage:** None — hosting configuration.
- **Criterion:** The existing project deploys dherman/thinkwell-rs main from /website.
- **Evidence:** Main integration and CI passed. The authenticated dashboard shows
  thinkwell.sh connected under AWS / aws-8f18fdf7. Match the owner's supplied
  project ID before changing settings; no source configuration has changed.

#### Domain and delivery {#documentation-site-5-domain}

- **Anchor coverage:** None — public documentation availability.
- **Criterion:** The Rust site serves over HTTPS at thinkwell.rs with working
  navigation/deep links and correct domain settings.
- **Evidence:** Pending. Local resolver and Cloudflare 1.1.1.1 returned NXDOMAIN
  for thinkwell.rs NS queries on 2026-11-23. This establishes missing public DNS
  resolution at inspection, not a conclusion about domain ownership. The owner identified Loopia as the registrar/DNS provider. Nameserver
  delegation and the Loopia zone remain to be inspected. No DNS mutations occurred. The introduction's
  TS link now points to the source repository, which remains useful independently
  of publication. The owner subsequently reported incomplete Loopia registration;
  do not assume the domain is registered or ready for DNS configuration.

## Publication pause and handoff

The owner requested completion of the hosting pause while keeping the documentation.
Retain all Rust tutorials, website/docs.json, local Mintlify preview instructions,
and existing documentation CI. Public hosting is not required to review these
locally. No Mintlify source/domain settings, GitHub App permissions, or DNS records
were changed, so no external rollback is required. Authentication was performed;
no credentials or one-time codes are recorded in the Book.

If publication is resumed, first confirm completion of Loopia registration and
match project ID `6981adca63153ddab05ccfed` in the Mintlify dashboard. The unexpected
AWS / aws-8f18fdf7 label alone does not identify the wrong project: the dashboard
showed thinkwell.sh connected. Recheck live state, then use the retained source
configuration and actual host-provided DNS records. Previous DNS observations
are historical, not current checks. No upgrade, new project, or domain purchase
is authorized by this record.

The owner's next planning priority is convenient local Book and Rustdoc previews,
with CI artifacts considered separately. Preview automation has not been
implemented or assigned by this pause. Public Rustdoc hosting remains separate.
The scope assessment and guide-disclosure experiment remain unrelated local work;
their proposed API deferrals have not been accepted.

The owner authorized a draft PR for this pause record after reviewing the local
update. Delivery is based on PR #12's merge at 9bc0a5c; owner review, CI, and
integration remain pending. The hosting milestone remains Blocked after delivery
of this record; no publication or lifecycle transition is implied.

Validation of the pause update: Book structural checking, mdBook 0.5.4 rendering,
and whitespace checks passed. The rendered pause section was checked for the
owner hold, incomplete registration, and retained project identity. No external
service checks, product tests, or deployment were needed for this prose change.
