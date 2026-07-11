//! The Typst projection (spec §8.1/§8.2 — Typst is the ratified PDF engine). Renders the doc-IR to a
//! single `.typ` source; the actual PDF compile (`typst compile`) is an *optional* downstream step
//! that **skips gracefully when the `typst` binary is absent** (the env may lack it) — never a
//! half-build. Each block is preceded by a `// cid:` comment so the Typst view shares identity with
//! the HTML/JSON views (one content-addressed truth).
//!
//! **Print code legibility (§8.2).** Captured code blocks are tuned for *paper*, with a different
//! scale than the web theme: the body prose is ~10.5pt, but fenced/raw code is set ~0.82× body with
//! tighter leading so lines fit without wrapping, inside a **light tinted box with a hairline border**
//! (never a filled dark panel — print-ink-friendly), comfortable page margins, light-only (print has
//! no dark mode). Honest scope: the PDF path renders code as monospace **without** the web
//! highlighter's per-token colours (that would need per-token `#text(fill:)` emission or a bundled
//! theme — future work), so there is nothing to mis-colour.

use crate::ir::{DocModel, Node, Payload};

/// Render the whole model to one Typst document source.
#[must_use]
pub fn render(model: &DocModel) -> String {
    let mut out = String::new();
    out.push_str(
        "// Generated from the Mycelium corpus — a projection, never a parallel truth (ADR-003/G11).\n\
         // Compile with: typst compile doc.typ doc.pdf  (skipped gracefully when typst is absent).\n\
         #set document(title: \"Mycelium Documentation\")\n\
         #set page(numbering: \"1\", margin: (x: 2.2cm, y: 2.4cm))\n\
         #set text(font: \"New Computer Modern\", size: 10.5pt)\n\
         #set heading(numbering: \"1.1\")\n\
         // Print-legible code (§8.2): a light tinted box with a HAIRLINE border (never a filled dark\n\
         // panel), code ~0.82x body with tighter leading so lines fit without wrapping. Light-only.\n\
         #show raw.where(block: true): it => {\n\
         set text(size: 8.6pt)\n\
         set par(leading: 0.42em)\n\
         block(fill: rgb(\"#e4e9d9\"), stroke: 0.5pt + rgb(\"#cfd5c1\"), radius: 3pt, inset: (x: 8pt, y: 7pt), width: 100%, breakable: true, it)\n\
         }\n\n\
         #align(center)[#text(18pt)[*Mycelium Documentation*]]\n\
         #align(center)[_A projection of the cited corpus._]\n\n\
         #outline()\n\n",
    );
    for doc in &model.documents {
        render_doc(doc, &mut out);
    }
    out
}

fn render_doc(doc: &Node, out: &mut String) {
    out.push_str(&format!("// cid: {}\n", doc.id.as_str()));
    out.push_str(&format!(
        "= {}\n\n",
        escape(doc.title.as_deref().unwrap_or(&doc.anchor))
    ));
    for c in &doc.children {
        render_node(c, 2, out);
    }
    out.push('\n');
}

fn render_node(node: &Node, depth: usize, out: &mut String) {
    out.push_str(&format!("// cid: {}\n", node.id.as_str()));
    match &node.payload {
        Payload::Section => {
            let eq = "=".repeat(depth.clamp(2, 6));
            out.push_str(&format!(
                "{eq} {}\n\n",
                escape(node.title.as_deref().unwrap_or(""))
            ));
            for c in &node.children {
                render_node(c, depth + 1, out);
            }
        }
        Payload::Prose { text } => {
            out.push_str(&escape(text));
            out.push_str("\n\n");
        }
        Payload::Example { lang, source, .. } => {
            // Typst raw block; fence with backticks and the language tag. The closing fence must
            // start on its own line, so normalize to exactly one trailing newline in the body
            // (a source without a trailing newline would otherwise produce an invalid `…code````).
            let body = source.strip_suffix('\n').unwrap_or(source);
            out.push_str(&format!("```{lang}\n{body}\n```\n\n"));
        }
        Payload::ApiItem { signature, summary } => {
            let eq = "=".repeat(depth.clamp(2, 6));
            out.push_str(&format!(
                "{eq} `{}`\n\n",
                escape(signature.as_deref().unwrap_or(""))
            ));
            match summary {
                Some(s) => {
                    out.push_str(&escape(s));
                    out.push_str("\n\n");
                }
                None => out.push_str("_undocumented — no summary projected from source._\n\n"),
            }
            for c in &node.children {
                render_node(c, depth + 1, out);
            }
        }
        Payload::Undocumented { what } => {
            out.push_str(&format!("_undocumented: {}_\n\n", escape(what)));
        }
        Payload::Xref { target } => {
            out.push_str(&format!(
                "#link(\"{}\")[{}]\n\n",
                escape_str(&target.raw),
                escape(&target.raw)
            ));
        }
        Payload::Document { .. } | Payload::Index => {
            for c in &node.children {
                render_node(c, depth, out);
            }
        }
    }
}

/// Escape Typst markup metacharacters in body text. `pub(crate)` for white-box unit testing in
/// `src/tests/typst.rs`, not a downstream API.
pub(crate) fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '#' | '$' | '*' | '_' | '`' | '<' | '>' | '@' | '\\' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    out
}

/// Escape for a Typst string literal (used inside `#link("...")`).
fn escape_str(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
