# User documentation

The learning site teaches the supported Rust library through complete tasks.
Mintlify owns the learning path; Rustdoc owns exact public signatures. The
project Book owns project design, authority, work, and evidence. This division
supports `R-easy-agent-programming`, `R-progressive-control`, and
`R-code-prompt-ergonomics` without maintaining a second API reference in tutorials.

## Learning path

The site starts with a complete typed answer, Cargo setup, and ordinary
JsonSchema/Deserialize derives. Subsequent pages introduce structs and enums,
text/JSON/code context, borrowed tools, persistent conversations with changing
answer types, and observation. Tools first use the inline Plan API; explicit
catalogs appear when history makes them useful. ACP is introduced as the
connection protocol; transport implementation does not lead the tutorial.

Worked examples explain sentiment analysis, meeting scheduling, and JavaScript
unminification. Relevant repository-owned code appears beside each step, with
types, deterministic computation, request flow, and result handling. Source links
provide optional deeper detail. The [example design](examples.md) supplies their
behavioral boundaries; sample output remains illustrative.

Practical reference covers Claude/Codex and custom command setup, optional model
selection, supported values, errors/cleanup, and access to Rustdoc. Actionable
limitations appear beside the affected operations. Contribution links lead to
the repository guide and Book. Tutorials do not assume a published crate,
unavailable hosted Rustdoc, a Thinkwell Rust CLI, or legacy tooling parity.

## Source and validation ownership

The website contains original Rust-oriented prose and code. Complete Rust programs
are extracted directly from MDX, compiled, and checked with rustfmt. Titled partial
excerpts are matched to their indicated repository source, permitting indentation
changes. Navigation and local links are checked. Mintlify validation/build and
broken-link checks run with the pinned tooling documented in CONTRIBUTING.
These checks run in the documentation CI job without agent credentials or model
inference; they do not verify live model behavior or visual presentation.

The site is distinct from the TypeScript documentation. No legacy redirects,
assets, community policy, or release process are implicitly adopted. Public
hosting, cross-site links, DNS, and a durable public Rustdoc destination require
an assigned delivery scope; the site design itself does not specify that deployment.
