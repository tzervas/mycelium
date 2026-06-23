//! The `gen-apiref` projection (output (d), fully automated — spec §4). API reference is **pure
//! projection from code + schemas + M-359 metadata**, no interpretive layer: a `.myc` nodule's header
//! ([`mycelium_proj::parse_header`]) and `fn` signatures, and the JSON schemas, become api-item IR
//! nodes. A missing `@summary` / schema `description` is an explicit [`Payload::ApiItem`] with
//! `summary: None` (rendered "undocumented") — **never invented** (the prose form of G2). The whole
//! `.myc` source is also captured as a *checked* example, so the §4.1 checked-examples lint type-checks
//! the real, dogfooded code (T7.1/T7.5).

use mycelium_proj::parse_header;
use serde_json::Value;

use crate::corpus::AnchorAlloc;
use crate::ir::{Level, Node, Payload, Provenance, SourceKind};

/// Project a `.myc` source into a [`Payload::Document`] (`source_kind: api`) of api-item nodes.
///
/// Children: the nodule itself (signature + `@summary` or undocumented), one api-item per `fn`
/// signature (currently undocumented — the doc-comment surface is later, spec §4 note), and the whole
/// source as a *checked* example.
#[must_use]
pub fn project_nodule(path: &str, src: &str, alloc: &mut AnchorAlloc) -> Node {
    let nodule_name = nodule_name(src).unwrap_or_else(|| path_stem(path));
    let doc_anchor = alloc.alloc(None, &format!("api {nodule_name}"));

    // The nodule header's @summary, if the header parses and carries one. A malformed header is not
    // *our* error to raise (myc-check/myc-lint own it) — here it simply yields no summary (honest:
    // the item renders as undocumented rather than crashing the build).
    let summary = parse_header(src)
        .ok()
        .flatten()
        .and_then(|h| h.fields.summary);

    let mut children = Vec::new();
    children.push(Node::new(
        alloc.alloc(Some(&doc_anchor), "nodule"),
        Some(format!("nodule {nodule_name}")),
        Some(Level::Medium),
        Provenance {
            source: path.to_owned(),
            line: 1,
        },
        Payload::ApiItem {
            signature: Some(format!("nodule {nodule_name}")),
            summary,
        },
        vec![],
    ));

    // One api-item per `fn` signature. The summary is the contiguous `//` doc-comment block
    // immediately preceding the `fn` (M-736) — extracted from source, never invented; a `fn` with
    // no preceding comment stays `None` (rendered "undocumented", an explicit honest gap — G2).
    // The source is split into lines once here (not per `fn`) so projection stays O(#lines).
    let lines: Vec<&str> = src.lines().collect();
    for (sig, line) in fn_signatures(src) {
        let name = fn_name(&sig).unwrap_or_else(|| "fn".to_owned());
        children.push(Node::new(
            alloc.alloc(Some(&doc_anchor), &format!("fn {name}")),
            Some(sig.clone()),
            Some(Level::Detailed),
            Provenance {
                source: path.to_owned(),
                line,
            },
            Payload::ApiItem {
                signature: Some(sig),
                summary: preceding_doc(&lines, line),
            },
            vec![],
        ));
    }

    // The whole source as a checked example (it is real, type-checked code — §4.1 #4 / T7.1).
    children.push(Node::new(
        alloc.alloc(Some(&doc_anchor), "source"),
        Some("Source".to_owned()),
        Some(Level::Detailed),
        Provenance {
            source: path.to_owned(),
            line: 1,
        },
        Payload::Example {
            lang: "myc".to_owned(),
            source: src.to_owned(),
            checked: true,
        },
        vec![],
    ));

    Node::new(
        doc_anchor,
        Some(format!("nodule {nodule_name}")),
        None,
        Provenance {
            source: path.to_owned(),
            line: 1,
        },
        Payload::Document {
            source_kind: SourceKind::Api,
        },
        children,
    )
}

/// Project a JSON-schema file into a [`Payload::Document`] of api-item nodes (one per top-level
/// property). A property with no `description` is an explicit undocumented api-item.
#[must_use]
pub fn project_schema(path: &str, json: &str, alloc: &mut AnchorAlloc) -> Option<Node> {
    let v: Value = serde_json::from_str(json).ok()?;
    let title = v
        .get("title")
        .and_then(Value::as_str)
        .map_or_else(|| path_stem(path), str::to_owned);
    let doc_anchor = alloc.alloc(None, &format!("schema {title}"));

    let mut children = Vec::new();
    if let Some(desc) = v.get("description").and_then(Value::as_str) {
        children.push(Node::new(
            alloc.alloc(Some(&doc_anchor), "overview"),
            None,
            Some(Level::Minimal),
            Provenance {
                source: path.to_owned(),
                line: 1,
            },
            Payload::Prose {
                text: desc.to_owned(),
            },
            vec![],
        ));
    }
    if let Some(props) = v.get("properties").and_then(Value::as_object) {
        for (name, spec) in props {
            let ty = spec
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or("object")
                .to_owned();
            let summary = spec
                .get("description")
                .and_then(Value::as_str)
                .map(str::to_owned);
            children.push(Node::new(
                alloc.alloc(Some(&doc_anchor), &format!("field {name}")),
                Some(format!("{name}: {ty}")),
                Some(Level::Detailed),
                Provenance {
                    source: path.to_owned(),
                    line: 1,
                },
                Payload::ApiItem {
                    signature: Some(format!("{name}: {ty}")),
                    summary,
                },
                vec![],
            ));
        }
    }

    Some(Node::new(
        doc_anchor,
        Some(format!("schema {title}")),
        None,
        Provenance {
            source: path.to_owned(),
            line: 1,
        },
        Payload::Document {
            source_kind: SourceKind::Api,
        },
        children,
    ))
}

/// The dotted nodule name from a `nodule X.Y` declaration (or the `// nodule:` marker).
fn nodule_name(src: &str) -> Option<String> {
    for line in src.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("nodule ") {
            return Some(rest.trim().trim_end_matches(['{', ' ']).to_owned());
        }
    }
    // Fall back to the marker comment.
    for line in src.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("// nodule:") {
            return Some(rest.trim().to_owned());
        }
    }
    None
}

/// Extract `fn NAME(...) -> Ty` / `fn NAME(...) =` signatures with their 1-based line numbers.
fn fn_signatures(src: &str) -> Vec<(String, u32)> {
    let mut out = Vec::new();
    for (i, line) in src.lines().enumerate() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("fn ") {
            // The signature is everything up to the body-introducing `=` (or end of line).
            let sig = rest.split_once('=').map_or(rest, |(s, _)| s).trim();
            out.push((format!("fn {sig}"), (i + 1) as u32));
        }
    }
    out
}

/// The contiguous `//` doc-comment block immediately above the `fn` at `fn_line` (1-based), over
/// the already-split source `lines`. The scan walks backward, joining `//` comment lines into one
/// summary, and stops at the first blank line, non-comment line, or header line (`// nodule…` /
/// `// @key:` are metadata, not doc prose). Returns `None` when the `fn` has no preceding comment —
/// an honest, explicit gap (never invented filler, G2). The text is taken verbatim from source, so
/// it always traces to its provenance. Takes a `&[&str]` so the caller splits the source once.
fn preceding_doc(lines: &[&str], fn_line: u32) -> Option<String> {
    if fn_line == 0 || (fn_line as usize) > lines.len() {
        return None;
    }
    let mut idx = (fn_line as usize) - 1; // 0-based index of the `fn` line itself
    let mut collected: Vec<String> = Vec::new();
    while idx > 0 {
        idx -= 1;
        let t = lines[idx].trim();
        let Some(rest) = t.strip_prefix("//") else {
            break; // a non-comment line ends the doc block
        };
        let rest = rest.trim();
        // Header lines and blank comments are not doc prose — they bound the block.
        if rest.is_empty() || rest.starts_with('@') || rest.starts_with("nodule") {
            break;
        }
        collected.push(rest.to_owned());
    }
    if collected.is_empty() {
        return None;
    }
    collected.reverse();
    Some(collected.join(" "))
}

/// The function name from a `fn NAME(...)` signature.
fn fn_name(sig: &str) -> Option<String> {
    let rest = sig.strip_prefix("fn ")?;
    let name: String = rest
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '.')
        .collect();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

fn path_stem(path: &str) -> String {
    let file = path.rsplit('/').next().unwrap_or(path);
    file.rsplit_once('.').map_or(file, |(s, _)| s).to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::Payload;

    const SRC: &str = "// nodule: hello.greeting\n\
                       // @summary: A greeting nodule.\n\
                       nodule hello.greeting\n\
                       \n\
                       fn wave() -> Ternary{4} =\n\
                         <+0-0>\n";

    #[test]
    fn a_documented_nodule_carries_its_summary() {
        let mut a = AnchorAlloc::new();
        let doc = project_nodule("examples/hello/greeting.myc", SRC, &mut a);
        let nodule_item = doc
            .children
            .iter()
            .find_map(|n| match &n.payload {
                Payload::ApiItem { summary, .. }
                    if n.title.as_deref() == Some("nodule hello.greeting") =>
                {
                    Some(summary.clone())
                }
                _ => None,
            })
            .unwrap();
        assert_eq!(nodule_item.as_deref(), Some("A greeting nodule."));
    }

    #[test]
    fn an_undocumented_fn_is_flagged_never_invented() {
        let mut a = AnchorAlloc::new();
        let doc = project_nodule("x.myc", SRC, &mut a);
        let fn_item = doc
            .children
            .iter()
            .find(|n| n.title.as_deref() == Some("fn wave() -> Ternary{4}"))
            .unwrap();
        match &fn_item.payload {
            Payload::ApiItem { summary, signature } => {
                assert!(summary.is_none(), "undocumented, never invented");
                assert_eq!(signature.as_deref(), Some("fn wave() -> Ternary{4}"));
            }
            _ => panic!("expected an api-item"),
        }
    }

    #[test]
    fn a_fn_with_a_preceding_comment_is_documented_from_source() {
        // The contiguous `//` block above a `fn` becomes its summary (M-736); a `@`/`nodule`
        // header or a blank line bounds the block, so the nodule header never leaks into a fn doc.
        const DOC_SRC: &str = "// nodule: m\n\
                               nodule m\n\
                               \n\
                               // add: combine two bytes. Why: the running total step.\n\
                               // It is total and never-silent.\n\
                               fn add(a: Binary{8}, b: Binary{8}) -> Binary{8} = a\n";
        let mut a = AnchorAlloc::new();
        let doc = project_nodule("x.myc", DOC_SRC, &mut a);
        let fn_item = doc
            .children
            .iter()
            .find(|n| n.title.as_deref() == Some("fn add(a: Binary{8}, b: Binary{8}) -> Binary{8}"))
            .unwrap();
        match &fn_item.payload {
            Payload::ApiItem { summary, .. } => {
                assert_eq!(
                    summary.as_deref(),
                    Some(
                        "add: combine two bytes. Why: the running total step. It is total and never-silent."
                    ),
                    "the two-line source comment is joined verbatim — traces to source, never invented"
                );
            }
            _ => panic!("expected an api-item"),
        }
    }

    #[test]
    fn the_whole_source_is_a_checked_example() {
        let mut a = AnchorAlloc::new();
        let doc = project_nodule("x.myc", SRC, &mut a);
        let ex = doc
            .children
            .iter()
            .find_map(|n| match &n.payload {
                Payload::Example {
                    checked, source, ..
                } => Some((*checked, source.clone())),
                _ => None,
            })
            .unwrap();
        assert!(ex.0);
        assert!(ex.1.contains("fn wave"));
    }

    #[test]
    fn a_schema_projects_its_fields_with_undocumented_gaps() {
        let mut a = AnchorAlloc::new();
        let schema = r#"{
            "title": "Bound",
            "description": "A numeric bound.",
            "properties": {
                "kind": {"type": "string", "description": "The bound kind."},
                "value": {"type": "number"}
            }
        }"#;
        let doc = project_schema("docs/spec/schemas/bound.schema.json", schema, &mut a).unwrap();
        let documented = doc
            .children
            .iter()
            .filter(|n| {
                matches!(
                    &n.payload,
                    Payload::ApiItem {
                        summary: Some(_),
                        ..
                    }
                )
            })
            .count();
        let undocumented = doc
            .children
            .iter()
            .filter(|n| matches!(&n.payload, Payload::ApiItem { summary: None, .. }))
            .count();
        assert_eq!(documented, 1, "kind is documented");
        assert_eq!(undocumented, 1, "value is an explicit undocumented gap");
    }
}
