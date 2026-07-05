//! Shared depth-derivation + nested-source-synthesis helpers for the RFC-0041 (RR-29 §4)
//! depth-structured fuzz targets (`fuzz_depth_parse`, `fuzz_depth_check`, `fuzz_depth_interp`).
//!
//! `fuzz/` is a standalone, non-workspace cargo-fuzz crate with no `[lib]` target (see the "do NOT
//! touch the root workspace" note in `fuzz/Cargo.toml`) — each `[[bin]]` is its own translation
//! unit, so this file is pulled in via `#[path = "depth_common.rs"] mod depth_common;` in each
//! target rather than a shared library crate. Keeps the three targets DRY without restructuring the
//! manifest.
#![allow(dead_code)] // not every target uses every helper here.

/// Derive a bounded nesting depth from the first 4 bytes of the fuzz input (big-endian `u32`,
/// reduced modulo `max + 1`). Returns the depth and the remaining, unconsumed bytes (kept for
/// callers that want to derive a second value, e.g. a construct-vs-group mode bit).
#[must_use]
pub fn derive_depth(data: &[u8], max: usize) -> (usize, &[u8]) {
    let take = data.len().min(4);
    let mut buf = [0u8; 4];
    buf[..take].copy_from_slice(&data[..take]);
    let n = u32::from_be_bytes(buf) as usize;
    (n % (max + 1), &data[take..])
}

/// Synthesize a single-`nodule` source string whose body expression nests `depth` levels deep —
/// either grouping parens `(((0b0)))` or list literals `[[[0b0]]]` (M-977's Vec list-literal ->
/// Cons-chain desugaring, RFC-0040) depending on `use_brackets`. Both constructs recurse through
/// the parser's shared `MAX_EXPR_DEPTH` budget (`crates/mycelium-l1/src/parse.rs`,
/// `Parser::enter_depth`/`leave_depth`) — parens via `parse_primary`'s `LParen` grouping arm, list
/// literals via `parse_literal`'s `LBracket` arm calling back into `parse_expr` per element. This
/// harness's job is to confirm that guard holds under adversarial depths (an explicit `ParseError`,
/// never a panic/abort) — not to make the guard "pass" some other way.
#[must_use]
pub fn nested_source(depth: usize, use_brackets: bool) -> String {
    let (open, close) = if use_brackets { ("[", "]") } else { ("(", ")") };
    let mut src = String::with_capacity(depth * 2 + 64);
    src.push_str("nodule demo\nfn f() -> Binary{8} = ");
    for _ in 0..depth {
        src.push_str(open);
    }
    src.push_str("0b0");
    for _ in 0..depth {
        src.push_str(close);
    }
    src.push('\n');
    src
}
