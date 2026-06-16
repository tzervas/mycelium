//! **Document sync** (M-310; FR-S5; SC-5): the text → `Node` pipeline the LSP server runs on
//! `textDocument/didOpen` / `didChange` / `didClose`, now that RFC-0011 r3 + RFC-0001 r4 give the
//! surface a path into L0 (parse → check → elaborate).
//!
//! On each edit the server re-analyzes the *whole* document (full sync — `TextDocumentSyncKind.Full`)
//! and pushes `textDocument/publishDiagnostics`. The diagnostics are **honest about their spans**:
//! - a **parse/lex** failure carries a **real range** (the L1 lexer tracks `line:col`, [`Pos`]);
//! - a **type-check** failure carries a best-effort range at the offending function's `fn <name>`
//!   declaration (the checker tracks the failing *function* but not yet the failing *sub-expression*
//!   span — flagged, not fabricated: the precise sub-span awaits the checker carrying spans), with
//!   the function name in `data.breadcrumb`;
//! - a **clean** colony yields **no** diagnostics.
//!
//! The parser is fail-fast (one error at a time); a recovering parser that reports many diagnostics
//! at once is a later refinement — recorded, not silently implied.

use std::collections::BTreeMap;

use serde_json::{json, Value};

use mycelium_l1::{check_colony, parse, CheckError, ParseError};

use crate::wire::SERVER_NAME;

/// An in-memory store of open documents (`uri → source text`), the minimal state full-sync requires.
#[derive(Debug, Clone, Default)]
pub struct DocumentStore {
    docs: BTreeMap<String, String>,
}

impl DocumentStore {
    /// An empty store.
    #[must_use]
    pub fn new() -> Self {
        DocumentStore::default()
    }

    /// Record (or replace) a document's full text (`didOpen` / `didChange` full sync).
    pub fn set(&mut self, uri: impl Into<String>, text: impl Into<String>) {
        self.docs.insert(uri.into(), text.into());
    }

    /// Drop a document (`didClose`).
    pub fn remove(&mut self, uri: &str) {
        self.docs.remove(uri);
    }

    /// The stored text for `uri`, if open.
    #[must_use]
    pub fn text(&self, uri: &str) -> Option<&str> {
        self.docs.get(uri).map(String::as_str)
    }

    /// Number of open documents.
    #[must_use]
    pub fn len(&self) -> usize {
        self.docs.len()
    }

    /// Whether the store is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.docs.is_empty()
    }
}

/// Analyze a document's source through the text → `Node` pipeline and return its LSP diagnostics
/// (JSON). `parse` failure → one ranged syntax diagnostic; `check_colony` failure → one
/// function-located type diagnostic; a clean colony → no diagnostics.
#[must_use]
pub fn source_diagnostics(text: &str) -> Vec<Value> {
    match parse(text) {
        Err(pe) => vec![parse_error_diagnostic(&pe)],
        Ok(colony) => match check_colony(&colony) {
            Err(ce) => vec![check_error_diagnostic(text, &ce)],
            Ok(_env) => Vec::new(),
        },
    }
}

/// The full `textDocument/publishDiagnostics` notification for `uri`'s `text` (parse → check).
#[must_use]
pub fn publish_for_source(uri: &str, text: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": { "uri": uri, "diagnostics": source_diagnostics(text) },
    })
}

/// A zero-based, one-character LSP range at a 1-based lexer [`Pos`].
fn point_range(line0: u32, col0: u32, width: u32) -> Value {
    json!({
        "start": { "line": line0, "character": col0 },
        "end":   { "line": line0, "character": col0 + width.max(1) },
    })
}

/// A ranged syntax diagnostic from a [`ParseError`] (the lexer's `line:col` is 1-based; LSP is
/// 0-based — never a fabricated location, the position is real).
fn parse_error_diagnostic(pe: &ParseError) -> Value {
    let line0 = pe.pos.line.saturating_sub(1);
    let col0 = pe.pos.col.saturating_sub(1);
    json!({
        "range": point_range(line0, col0, 1),
        "severity": 1, // Error
        "code": "parse",
        "source": SERVER_NAME,
        "message": pe.message,
    })
}

/// A best-effort-located type diagnostic from a [`CheckError`]. The checker tracks the failing
/// *function* (`site`), not yet the failing sub-expression span, so we locate `fn <site>` in the
/// source for a usable range and carry the function name as the navigable breadcrumb; if the name is
/// not found textually, we fall back to a zero range (honest, never a fabricated line).
fn check_error_diagnostic(text: &str, ce: &CheckError) -> Value {
    let range = locate_fn(text, &ce.site).map_or_else(
        || point_range(0, 0, 1),
        |(line0, col0)| point_range(line0, col0, ce.site.len() as u32),
    );
    json!({
        "range": range,
        "severity": 1, // Error
        "code": "check",
        "source": SERVER_NAME,
        "message": ce.message,
        "data": { "breadcrumb": [ce.site] },
    })
}

/// Locate a `fn <name>` declaration's name position (0-based `line, col`) in `text`, if present.
fn locate_fn(text: &str, name: &str) -> Option<(u32, u32)> {
    let needle = format!("fn {name}");
    for (li, line) in text.lines().enumerate() {
        if let Some(idx) = line.find(&needle) {
            // Column of the name itself (after `fn `).
            let col = idx + "fn ".len();
            return Some((li as u32, col as u32));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_clean_colony_has_no_diagnostics() {
        let src = "colony d\nfn main() -> Binary{8} = not(0b1011_0010)";
        assert!(source_diagnostics(src).is_empty());
    }

    #[test]
    fn a_parse_error_is_ranged_at_the_real_position() {
        // A missing policy on a swap is a parse error with a real position.
        let src = "colony demo\nfn f(x: Binary{8}) -> Ternary{6} = swap(x, to: Ternary{6})";
        let diags = source_diagnostics(src);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0]["code"], "parse");
        assert_eq!(diags[0]["severity"], 1);
        // The range is real (not the zero placeholder the pre-M-310 wire used): line > 0.
        assert!(diags[0]["range"]["start"]["line"].as_u64().unwrap() >= 1);
    }

    #[test]
    fn a_type_error_is_located_at_its_function_with_a_breadcrumb() {
        // `add` over Binary is a type error (it expects Ternary) — a check diagnostic at `fn bad`.
        let src = "colony d\nfn bad() -> Binary{8} = add(0b0000_0001, 0b0000_0010)";
        let diags = source_diagnostics(src);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0]["code"], "check");
        assert_eq!(diags[0]["data"]["breadcrumb"], json!(["bad"]));
        // Located at the `fn bad` declaration line (line 1, 0-based), not a fabricated (0,0).
        assert_eq!(diags[0]["range"]["start"]["line"], 1);
    }

    #[test]
    fn the_store_tracks_open_and_closed_documents() {
        let mut store = DocumentStore::new();
        assert!(store.is_empty());
        store.set("mem://a", "colony d");
        assert_eq!(store.text("mem://a"), Some("colony d"));
        store.set("mem://a", "colony d2"); // didChange replaces (full sync)
        assert_eq!(store.text("mem://a"), Some("colony d2"));
        store.remove("mem://a");
        assert!(store.is_empty());
    }

    #[test]
    fn publish_for_source_has_the_lsp_notification_shape() {
        let note = publish_for_source(
            "mem://demo",
            "colony d\nfn main() -> Binary{8} = 0b0000_0000",
        );
        assert_eq!(note["method"], "textDocument/publishDiagnostics");
        assert_eq!(note["params"]["uri"], "mem://demo");
        assert_eq!(note["params"]["diagnostics"], json!([])); // clean
    }
}
