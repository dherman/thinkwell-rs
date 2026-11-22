//! JavaScript mechanics, separate from the Thinkwell workflow.
use oxc::{
    allocator::Allocator,
    ast::{AstKind, ast::Statement},
    parser::Parser,
    semantic::SemanticBuilder,
    span::{GetSpan, SourceType},
};
use serde::Serialize;
use std::collections::{HashMap, HashSet};

/// A function for the model to name. IDs belong to this source snapshot.
#[derive(Debug, Serialize)]
pub struct Candidate {
    pub id: String,
    pub name: String,
    pub source: String,
}

/// Find private top-level function declarations; assume valid JavaScript modules.
pub fn discover(source: &str) -> Vec<Candidate> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::mjs()).parse();
    assert!(parsed.diagnostics.is_empty(), "expected valid JavaScript");
    let exported: HashSet<_> = parsed
        .module_record
        .local_export_entries
        .iter()
        .filter_map(|entry| entry.local_name.name())
        .collect();
    let mut candidates = Vec::new();
    for statement in &parsed.program.body {
        if let Statement::FunctionDeclaration(function) = statement {
            let id = function.id.as_ref().unwrap();
            if !exported.contains(id.name.as_str()) {
                candidates.push(Candidate {
                    id: format!("function-{}", candidates.len()),
                    name: id.name.to_string(),
                    source: source[function.span.start as usize..function.span.end as usize].into(),
                });
            }
        }
    }
    candidates
}

/// Apply suggestions in discovery order, regardless of the agent's response order.
pub fn rename(source: &str, candidates: &[Candidate], names: &HashMap<String, String>) -> String {
    assert_eq!(
        names.len(),
        candidates.len(),
        "expected one name per function"
    );
    let renames: Vec<_> = candidates
        .iter()
        .map(|c| {
            (
                c.name.as_str(),
                names.get(&c.id).expect("missing function ID").as_str(),
            )
        })
        .collect();
    apply_renames(source, &renames)
}

// Reserve existing names, then edit declarations and resolved references in place.
// This deliberately assumes ordinary lexical JavaScript, not eval or reflection.
fn apply_renames(source: &str, suggestions: &[(&str, &str)]) -> String {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::mjs()).parse();
    assert!(parsed.diagnostics.is_empty());
    let result = SemanticBuilder::new_compiler()
        .with_build_nodes(true)
        .build(&parsed.program);
    assert!(result.diagnostics.is_empty());
    let semantic = result.semantic;
    let scoping = semantic.scoping();
    let mut reserved: HashSet<String> = scoping.symbol_names().map(str::to_owned).collect();
    reserved.extend(
        scoping
            .root_unresolved_references()
            .keys()
            .map(ToString::to_string),
    );
    let mut edits = Vec::new();
    for &(old, suggested) in suggestions {
        if old == suggested {
            continue;
        }
        let symbol = scoping
            .symbol_ids()
            .find(|&id| {
                scoping.symbol_name(id) == old
                    && scoping.symbol_scope_id(id) == scoping.root_scope_id()
            })
            .unwrap();
        let mut name = suggested.to_owned();
        let mut suffix = 2;
        while reserved.contains(&name) {
            name = format!("{suggested}{suffix}");
            suffix += 1;
        }
        reserved.insert(name.clone());
        edits.push((scoping.symbol_span(symbol), name.clone()));
        for reference in semantic.symbol_references(symbol) {
            let span = semantic.nodes().get_node(reference.node_id()).span();
            let replacement = match semantic.nodes().parent_kind(reference.node_id()) {
                AstKind::ObjectProperty(property) if property.shorthand => {
                    // A literal __proto__ key is a prototype setter, unlike shorthand.
                    if old == "__proto__" {
                        format!("[\"__proto__\"]: {name}")
                    } else {
                        format!(
                            "{}: {name}",
                            &source[span.start as usize..span.end as usize]
                        )
                    }
                }
                _ => name.clone(),
            };
            edits.push((span, replacement));
        }
    }
    // Work backwards so earlier byte offsets still refer to the original source.
    edits.sort_by_key(|(span, _)| std::cmp::Reverse(span.start));
    let mut output = source.to_owned();
    for (span, replacement) in edits {
        output.replace_range(span.start as usize..span.end as usize, &replacement);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn finds_private_functions_and_preserves_named_exports() {
        let found = discover(
            "function a(){ function inner(){} } function b(){} export {b as publicName}; export function c(){} const d = () => 1;",
        );
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "a");
    }

    #[test]
    fn only_selected_ranges_change_including_comments_and_shorthand() {
        let source = "// café\nfunction a(xs) { return xs.reduce((t,x) => t + /* keep */ x, 0); }\nconst obj = {a}; obj.a([2]); a([3]);";
        let expected = "// café\nfunction sum(xs) { return xs.reduce((t,x) => t + /* keep */ x, 0); }\nconst obj = {a: sum}; obj.a([2]); sum([3]);";
        assert_eq!(apply_renames(source, &[("a", "sum")]), expected);
    }

    #[test]
    fn resolves_recursion_and_shadowing_and_avoids_collisions() {
        let source =
            "function a(n){return n ? a(n-1) : 0;} function b(a,sum){return a(sum);} a(3);";
        let expected = "function sum2(n){return n ? sum2(n-1) : 0;} function b(a,sum){return a(sum);} sum2(3);";
        assert_eq!(apply_renames(source, &[("a", "sum")]), expected);
    }

    #[test]
    fn proto_shorthand_stays_a_data_property() {
        for spelling in ["__proto__", r"\u005f_proto__"] {
            let source = format!("function {spelling}(){{}} const obj = {{{spelling}}};");
            assert_eq!(
                apply_renames(&source, &[("__proto__", "readable")]),
                "function readable(){} const obj = {[\"__proto__\"]: readable};"
            );
        }
    }

    #[test]
    fn escaped_identifiers_and_repeated_suggestions() {
        assert_eq!(
            apply_renames(r"function \u0061(){return a();}", &[("a", "one")]),
            "function one(){return one();}"
        );
        assert_eq!(
            apply_renames(
                "function a(){} function b(){}",
                &[("a", "work"), ("b", "work")]
            ),
            "function work(){} function work2(){}"
        );
    }
}
