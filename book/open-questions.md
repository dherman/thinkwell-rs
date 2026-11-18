# Open questions

These are unresolved decisions, not proposed implementations or assigned work.

| Decision | What the next design work must establish |
| --- | --- |
| Crate boundaries | What responsibilities need independent crates after examining idiomatic Rust ownership and usage? |
| Async runtime | What execution, cancellation, concurrency, and embedding constraints govern runtime selection? |
| ACP/MCP libraries | Which existing Rust libraries can meet the selected protocol scope and lifecycle guarantees? |
| Schema representation and generation | How are typed values, validation, and schema production represented in Rust? |
| Supported agents | Which agents and integration behaviors will be supported and tested? |
| MSRV and platforms | What minimum compiler and platform commitments should the project make beyond its pinned development toolchain? |
| Verification policy | What criteria, test methods, review obligations, and any proof needs follow from the agreed product guarantees? |

Migration-specific compatibility and source-provenance questions live in
[migration research](migration.md). Project identity is settled by `D-thinkwell-identity`;
publication remains separately authorized.

The next session should first establish compatibility scope and source provenance,
then identify enough Rust design constraints to propose meaningful anchors and a
bounded milestone. This ordering is a recommendation, not a detailed port roadmap.
