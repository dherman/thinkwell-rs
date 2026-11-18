# Open questions

These are unresolved decisions, not proposed implementations or assigned work.

| Decision | What the next design work must establish |
| --- | --- |
| Compatibility goals | Which user-facing concepts, behavior, failure modes, and API expectations must be preserved; which upstream revision is the baseline? |
| Crate boundaries | What responsibilities need independent crates after examining idiomatic Rust ownership and usage? |
| Async runtime | What execution, cancellation, concurrency, and embedding constraints govern runtime selection? |
| ACP/MCP libraries | Which existing Rust libraries can meet the selected protocol scope and lifecycle guarantees? |
| Schema representation and generation | How are typed values, validation, and schema production represented in Rust? |
| Supported agents | Which agents and integration behaviors will be supported and tested? |
| MSRV and platforms | What minimum compiler and platform commitments should the project make beyond its pinned development toolchain? |
| Project identity | What wider project naming and affiliation claims are appropriate? The library name thinkwell is decided; publication is not authorized. |
| Licensing and attribution | What license should this project use, and what exact upstream or third-party notices apply to any future adaptation? |
| Verification policy | What criteria, test methods, review obligations, and any proof needs follow from the agreed product guarantees? |

The next session should first establish compatibility scope and source provenance,
then identify enough Rust design constraints to propose meaningful anchors and a
bounded milestone. This ordering is a recommendation, not a detailed port roadmap.
