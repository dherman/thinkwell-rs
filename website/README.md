# Rust website draft

The learning path starts with [What is Thinkwell?](get-started/introduction.mdx)
and [Your first typed answer](get-started/first-answer.mdx), then covers answer
types, context, tools, conversations and progress. Three worked-example pages
connect those concepts to the integrated programs. Reference pages cover agent
setup, supported values, errors/cleanup and local or downloadable Rustdoc.

The [documentation-site roadmap](../book/implementation-roadmaps/documentation-site.md)
records scope, evidence and delivery plans. Public hosting is paused; the
tutorials, local preview below, and CI validation remain available without
completing domain registration or configuring a hosted project.

## Local preview

With Node.js and npm installed, run from the repository root:

```sh
cd website
npx --yes mint@4.3.84 dev
```

Open the local URL printed by Mintlify (normally http://localhost:3000).

## Validate

From the repository root, with Python 3 and the repository Rust toolchain:

```sh
python3 website/scripts/check_examples.py
```

The script extracts every fenced Rust block directly from the MDX into ignored
`target/website-examples`. Plain Rust fences must contain complete programs. Titled Rust fences are source
excerpts: an adjacent MDX source comment identifies the repository file, and the
checker requires the passage to match that file (allowing display indentation). It uses the
first tutorial's dependency list, compiles all programs, checks Rust formatting, verifies source excerpts,
and checks navigation and local page links. It makes no model requests. Cargo
resolves dependencies into a local generated lockfile; this isn't a frozen
release or clean-machine installation test.

From website/:

```sh
npx --yes mint@4.3.84 validate --telemetry false
npx --yes mint@4.3.84 broken-links --telemetry false
```

These commands validate the site without publishing it. Browser review and live
agent behavior are separate from compilation and Mintlify validation. The checks
run in the **Documentation site** GitHub Actions job on pushes and pull requests.
The job uses Rust 1.100.0, Node 24.12.0 and Mintlify CLI 4.3.84, with a
15-minute timeout. It does not install agent adapters, use model credentials, or
deploy the site. Cargo resolves tutorial dependencies using the documented version
ranges, so this checks their current compatibility rather than a frozen release.
