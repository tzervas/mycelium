//! Lexer-level tests covering:
//!
//! 1. The M-661 `@std-sys` atomic marker (pre-existing).
//! 2. Comment capture via [`lex_with_comments`] (added for mycfmt comment-preservation,
//!    Stage 1): cases (a) full-line, (b) trailing same-line, (c) multiple consecutive leading,
//!    (d) comment separated by a blank line, (e) structured header lines (`// nodule:`/`// @key:`).
//! 3. Token-stream equivalence: `lex(s) == lex_with_comments(s).0` for every input used here.
use crate::lexer::*;
use crate::token::Tok;

fn toks(src: &str) -> Vec<Tok> {
    lex(src)
        .expect("lexes")
        .into_iter()
        .map(|s| s.tok)
        .collect()
}

/// Assert that `lex` and `lex_with_comments` produce identical token streams (byte-for-byte).
fn assert_token_stream_equiv(src: &str) {
    let plain = lex(src).expect("lex succeeded");
    let (with_cmt, _) = lex_with_comments(src).expect("lex_with_comments succeeded");
    assert_eq!(
        plain, with_cmt,
        "token stream mismatch between lex and lex_with_comments for input {src:?}"
    );
}

/// Copilot #397: on a CRLF source the captured comment text must not retain the trailing `\r`
/// (the lexer's "no carriage-return" contract; LF/CRLF round-trip parity).
#[test]
fn comment_capture_strips_trailing_cr_on_crlf() {
    let (_toks, comments) = lex_with_comments("nodule d;\n\nfn f() => Binary{1} =\n  0b1;\n")
        .expect("lex_with_comments succeeded");
    let last = comments.last().expect("a comment was captured");
    assert_eq!(
        last.text, "// why",
        "comment text must be CR-free on CRLF input"
    );
    assert!(
        !last.text.contains('\r'),
        "no carriage-return in comment text"
    );
}

/// Copilot #397 (perf): `lex` is the front-end fast path — it skips `//` comments without
/// building a [`Comment`], while [`lex_with_comments`] still captures them. The capture flag
/// never changes the token stream. This pins that the fast path coexists with capture rather
/// than routing every parse/check through comment allocation.
#[test]
fn lex_fast_path_skips_comments_capture_path_keeps_them() {
    let src = "nodule demo;\n\nfn f() => Binary{1} =\n  0b1;";
    let fast = lex(src).expect("lex (fast path) succeeded");
    let (cap, comments) = lex_with_comments(src).expect("lex_with_comments succeeded");
    assert_eq!(
        fast, cap,
        "the capture flag must not change the token stream"
    );
    assert_eq!(
        comments.len(),
        3,
        "all three // comments captured: {comments:?}"
    );
    assert!(
        !fast
            .iter()
            .any(|s| matches!(&s.tok, Tok::Ident(t) if t.starts_with("//"))),
        "no comment text leaks into the fast-path token stream"
    );
}

// -------------------------------------------------------------------------
// Pre-existing @std-sys marker tests (unchanged)
// -------------------------------------------------------------------------

#[test]
fn at_std_sys_lexes_as_one_atomic_marker_token() {
    // `@std-sys` is one token, immediately after the nodule path's last segment.
    let ts = toks("nodule std.sys.fs @std-sys;");
    assert!(
        ts.contains(&Tok::AtStdSys),
        "expected an AtStdSys token, got: {ts:?}"
    );
    // And it is NOT split into `@` + ident (no bare `Tok::At` here).
    assert!(
        !ts.contains(&Tok::At),
        "must not also emit a bare `@`: {ts:?}"
    );
}

#[test]
fn a_bare_at_is_still_the_guarantee_glyph() {
    // `@ Exact` (a guarantee annotation) stays `Tok::At` + the strength keyword — the special case
    // for `@std-sys` must not perturb the existing `T @ g` form.
    let ts = toks("Binary{8} @ Exact");
    assert!(ts.contains(&Tok::At), "expected a bare `@`, got: {ts:?}");
    assert!(
        !ts.contains(&Tok::AtStdSys),
        "a guarantee `@` must not be the std-sys marker: {ts:?}"
    );
}

#[test]
fn at_std_sys_is_a_whole_word_only() {
    // `@std-system` is NOT the marker: it lexes as `@` + ident `std` + `-` (the infix `sub`
    // operator, RFC-0025 / M-705) + ident `system`, so the special case is maximally narrow.
    // We assert the lexer never produces `AtStdSys` for it (never a silent over-match, G2).
    let ts = toks("@std-system");
    assert!(
        !ts.contains(&Tok::AtStdSys),
        "`@std-system` must not lex as the `@std-sys` marker (whole-word only): {ts:?}"
    );
    assert_eq!(
        ts,
        vec![
            Tok::At,
            Tok::Ident("std".to_owned()),
            Tok::Minus,
            Tok::Ident("system".to_owned()),
            Tok::Eof,
        ],
        "`@std-system` lexes as `@ std - system`"
    );
    // `@std` (no `-sys` tail) is a bare `@` + the identifier `std`.
    let ts = toks("@std");
    assert_eq!(ts, vec![Tok::At, Tok::Ident("std".to_owned()), Tok::Eof]);
}

// -------------------------------------------------------------------------
// Token-stream equivalence (regression guard)
// -------------------------------------------------------------------------

#[test]
fn lex_with_comments_token_stream_equals_lex() {
    // For every input exercised in this module, the token stream from lex_with_comments must
    // be byte-identical to the token stream from lex (comments do not bleed into tokens).
    let inputs = [
        "nodule std.sys.fs @std-sys;",
        "Binary{8} @ Exact",
        "@std",
        // comment cases used below
        "nodule demo;",
        "nodule demo;",
        "nodule demo;",
        "nodule demo;",
        "nodule demo;",
    ];
    for src in inputs {
        assert_token_stream_equiv(src);
    }
}

// -------------------------------------------------------------------------
// Case (a): a full-line comment
// -------------------------------------------------------------------------

#[test]
fn comment_capture_case_a_full_line_comment() {
    // A `//` comment occupying the whole line is captured with trailing=false.
    let src = "nodule demo;";
    let (toks, comments) = lex_with_comments(src).expect("lex_with_comments succeeded");
    assert_eq!(comments.len(), 1, "exactly one comment: {comments:?}");
    let c = &comments[0];
    assert_eq!(c.text, "// full-line comment");
    assert_eq!(c.line, 1, "comment is on line 1");
    assert_eq!(c.col, 1, "comment starts at col 1");
    assert!(!c.trailing, "full-line comment must have trailing=false");
    // Tokens: nodule, ident("demo"), eof — no comment token.
    let tok_kinds: Vec<_> = toks.iter().map(|s| &s.tok).collect();
    assert!(
        tok_kinds.contains(&&Tok::Nodule),
        "nodule token must be present"
    );
    assert!(
        !tok_kinds
            .iter()
            .any(|t| matches!(t, Tok::Ident(s) if s == "// full-line comment")),
        "comment must not appear in token stream"
    );
}

// -------------------------------------------------------------------------
// Case (b): a trailing same-line comment
// -------------------------------------------------------------------------

#[test]
fn comment_capture_case_b_trailing_same_line_comment() {
    // A comment after a real token on the same line: trailing=true.
    let src = "nodule demo;";
    let (_toks, comments) = lex_with_comments(src).expect("lex_with_comments succeeded");
    assert_eq!(comments.len(), 1, "exactly one comment: {comments:?}");
    let c = &comments[0];
    assert_eq!(c.text, "// trailing");
    assert_eq!(c.line, 1, "comment is on line 1");
    assert!(
        c.trailing,
        "comment after tokens on same line must have trailing=true"
    );
}

// -------------------------------------------------------------------------
// Case (c): multiple consecutive leading comments
// -------------------------------------------------------------------------

#[test]
fn comment_capture_case_c_consecutive_leading_comments() {
    // Two `//` lines in a row: both captured, both trailing=false, in source order.
    let src = "nodule demo;";
    let (_toks, comments) = lex_with_comments(src).expect("lex_with_comments succeeded");
    assert_eq!(comments.len(), 2, "two comments expected: {comments:?}");
    assert_eq!(comments[0].text, "// first");
    assert_eq!(comments[0].line, 1);
    assert!(!comments[0].trailing);
    assert_eq!(comments[1].text, "// second");
    assert_eq!(comments[1].line, 2);
    assert!(!comments[1].trailing);
}

// -------------------------------------------------------------------------
// Case (d): comment separated by a blank line
// -------------------------------------------------------------------------

#[test]
fn comment_capture_case_d_comment_after_blank_line() {
    // A comment on line 1, a blank line 2, a comment on line 3, then code.
    // Both comments are captured; neither is trailing.
    let src = "nodule demo;";
    let (_toks, comments) = lex_with_comments(src).expect("lex_with_comments succeeded");
    assert_eq!(comments.len(), 2, "two comments expected: {comments:?}");
    assert_eq!(comments[0].text, "// before");
    assert_eq!(comments[0].line, 1);
    assert!(!comments[0].trailing);
    assert_eq!(comments[1].text, "// after blank");
    assert_eq!(
        comments[1].line, 3,
        "comment after blank line must be on line 3"
    );
    assert!(!comments[1].trailing);
}

// -------------------------------------------------------------------------
// Case (e): structured header lines (// nodule:, // @key:) — captured verbatim
// -------------------------------------------------------------------------

#[test]
fn comment_capture_case_e_structured_header_comments() {
    // Header-style comments (`// nodule:`, `// @matured: true`) are captured verbatim.
    // Their placement (before/after the `nodule` declaration) is for the formatter to decide;
    // the lexer just captures them all without interpretation.
    let src = "nodule demo;";
    let (_toks, comments) = lex_with_comments(src).expect("lex_with_comments succeeded");
    assert_eq!(comments.len(), 2, "two header comments: {comments:?}");
    assert_eq!(comments[0].text, "// nodule: foo");
    assert_eq!(comments[0].line, 1);
    assert!(!comments[0].trailing);
    assert_eq!(comments[1].text, "// @matured: true");
    assert_eq!(comments[1].line, 2);
    assert!(!comments[1].trailing);
}

// -------------------------------------------------------------------------
// Additional edge-cases: end-of-file comment, empty comment, no comments
// -------------------------------------------------------------------------

#[test]
fn comment_capture_eof_without_trailing_newline() {
    // A comment at end-of-file (no trailing newline) is still captured.
    let src = "nodule demo;";
    let (_toks, comments) = lex_with_comments(src).expect("lex_with_comments succeeded");
    assert_eq!(comments.len(), 1, "one comment: {comments:?}");
    assert_eq!(comments[0].text, "// last line no newline");
    assert!(!comments[0].trailing, "no tokens on that line => leading");
}

#[test]
fn comment_capture_empty_comment_body() {
    // `//` with nothing after it (just a newline) stores exactly `"//"`.
    let src = "nodule demo;";
    let (_toks, comments) = lex_with_comments(src).expect("lex_with_comments succeeded");
    assert_eq!(comments.len(), 1);
    assert_eq!(
        comments[0].text, "//",
        "empty comment body: text must be exactly \"//\""
    );
    assert!(!comments[0].trailing);
}

#[test]
fn comment_capture_no_comments_gives_empty_vec() {
    // Source with no comments yields an empty comment vec.
    let src = "nodule demo;\n\nfn f() => Binary{8} =\n  0b0;";
    let (_toks, comments) = lex_with_comments(src).expect("lex_with_comments succeeded");
    assert!(
        comments.is_empty(),
        "no comments in source => empty vec: {comments:?}"
    );
}

// -------------------------------------------------------------------------
// DN-40 §3 (low) — base-prefixed literal with no digits is a never-silent
// lex error (G2). A `0b` prefix that scans no `0`/`1` is rejected, naming the
// offending position; valid binary / trit literals are unaffected.
// -------------------------------------------------------------------------

/// `0b` with no following binary digit is a lex error, not a silently-empty `BinLit`.
#[test]
fn lex_binary_empty_literal_is_a_lex_error() {
    let err = lex("0b").expect_err("`0b` alone must be a lex error");
    assert!(
        err.to_string().contains("no digits"),
        "error must name the empty-binary-literal cause: {err}"
    );
}

/// `0b` followed only by a `_` separator (no actual `0`/`1`) carries no value and is rejected.
#[test]
fn lex_binary_only_separator_is_a_lex_error() {
    lex("0b_").expect_err("`0b_` (separator, no digit) must be a lex error");
}

/// `0b` at end of a larger source still errors (no digit before EOF), and the error is raised
/// rather than emitting an empty token (never-silent).
#[test]
fn lex_binary_empty_literal_in_context_is_a_lex_error() {
    lex("fn f() => Binary{1} = 0b").expect_err("trailing `0b` with no digit must be a lex error");
}

/// Valid binary literals still lex to a `BinLit` carrying their digits (regression guard).
#[test]
fn lex_binary_valid_literals_still_lex() {
    assert_eq!(toks("0b0"), vec![Tok::BinLit("0".to_owned()), Tok::Eof]);
    assert_eq!(toks("0b1"), vec![Tok::BinLit("1".to_owned()), Tok::Eof]);
    assert_eq!(
        toks("0b1010"),
        vec![Tok::BinLit("1010".to_owned()), Tok::Eof]
    );
    // A `_` separator is allowed once at least one real digit is present.
    assert_eq!(toks("0b1_0"), vec![Tok::BinLit("1_0".to_owned()), Tok::Eof]);
}

/// RFC-0032 D4 (M-750): `0x…` byte-string literals lex like `0b…`. A valid even-hex literal
/// (optionally with `_` separators) lexes to a `BytesLit` carrying its inner string verbatim.
#[test]
fn lex_hex_bytes_valid_literals() {
    assert_eq!(toks("0x48"), vec![Tok::BytesLit("48".to_owned()), Tok::Eof]);
    assert_eq!(
        toks("0x48_65_6c_6c_6f"),
        vec![Tok::BytesLit("48_65_6c_6c_6f".to_owned()), Tok::Eof]
    );
    // Uppercase hex digits are accepted too.
    assert_eq!(
        toks("0xDEAD_BEEF"),
        vec![Tok::BytesLit("DEAD_BEEF".to_owned()), Tok::Eof]
    );
}

/// Never-silent (G2): an **odd** hex-digit count is a lex error — a byte is two hex chars, never
/// a silent half-byte.
#[test]
fn lex_hex_bytes_odd_count_is_a_lex_error() {
    let err = lex("0x123").expect_err("`0x123` (odd hex count) must be a lex error");
    assert!(
        err.to_string().contains("odd hex-digit count"),
        "error must name the odd-hex cause: {err}"
    );
    // A `_` separator does not change parity: `0x1_2_3` is still three hex digits → error.
    lex("0x1_2_3").expect_err("`0x1_2_3` (three hex digits) must be a lex error");
}

/// Never-silent (G2): an empty `0x` (no hex digit, or only a separator) is a lex error.
#[test]
fn lex_hex_bytes_empty_is_a_lex_error() {
    let err = lex("0x").expect_err("`0x` alone must be a lex error");
    assert!(
        err.to_string().contains("no hex digits"),
        "error must name the empty cause: {err}"
    );
    lex("0x_").expect_err("`0x_` (separator, no digit) must be a lex error");
}

/// A non-hex digit terminates the hex scan rather than being consumed: `0x1g` takes only `1`
/// (odd) → lex error; `0x12g` takes `0x12` (even) then the identifier `g`. This pins that the
/// scan stops at a non-hex char, and that the parity check applies to exactly what it took.
#[test]
fn lex_hex_bytes_stops_at_non_hex() {
    // `0x12` is even → BytesLit("12"), then `g` is an identifier.
    assert_eq!(
        toks("0x12g"),
        vec![
            Tok::BytesLit("12".to_owned()),
            Tok::Ident("g".to_owned()),
            Tok::Eof
        ]
    );
    // `0x1g` took only `1` (odd) before the non-hex `g` → a lex error (never a silent half-byte).
    lex("0x1g").expect_err("`0x1g` (one hex digit then non-hex) must be a lex error");
}

/// `Seq` and `Bytes` lex as keywords (never silent identifiers — G2).
#[test]
fn lex_seq_and_bytes_keywords() {
    assert_eq!(toks("Seq"), vec![Tok::Seq, Tok::Eof]);
    assert_eq!(toks("Bytes"), vec![Tok::Bytes, Tok::Eof]);
}

/// RFC-0037 D4: trit literals use the `0t…` prefix (lexed whole like `0b…`/`0x…`); the former
/// `<…>` angle form is retired. Valid trit literals lex to a `TritLit` carrying their inner glyphs.
#[test]
fn lex_trit_valid_literals_still_lex() {
    assert_eq!(
        toks("0t+-0"),
        vec![Tok::TritLit("+-0".to_owned()), Tok::Eof]
    );
    assert_eq!(toks("0t0"), vec![Tok::TritLit("0".to_owned()), Tok::Eof]);
    assert_eq!(toks("0t-"), vec![Tok::TritLit("-".to_owned()), Tok::Eof]);
    assert_eq!(toks("0t+"), vec![Tok::TritLit("+".to_owned()), Tok::Eof]);
}

/// RFC-0037 D1: `<` is now operator-only — the type-arg and trit-disambiguation roles are gone.
/// `<>` lexes as the `<`/`>` angle pair (no trit lookahead), and a `<+` is `LAngle` then `Plus`
/// (never a trit literal — the `0t…` prefix is the only trit path now).
#[test]
fn lex_angle_is_always_operator_not_a_trit() {
    // `<>` is the `<`/`>` angle pair (operators), never an (empty) trit literal.
    assert_eq!(toks("<>"), vec![Tok::LAngle, Tok::RAngle, Tok::Eof]);
    // `<+` no longer starts a trit literal: it is `LAngle` then the `+` operator.
    assert_eq!(toks("<+"), vec![Tok::LAngle, Tok::Plus, Tok::Eof]);
}

/// RFC-0025 §4.1 / M-745: `<<`/`>>` lex as the single shift tokens `Shl`/`Shr`; the single-char
/// `<`/`>` stay the `lt`/`gt` comparison glyphs (`LAngle`/`RAngle`). There is no nested-generic
/// `>>` hazard because type arguments moved to `[…]` (RFC-0037 D1), so the greedy doubled lex is
/// unambiguous.
#[test]
fn lex_shift_operators_are_whole_tokens() {
    assert_eq!(toks("<<"), vec![Tok::Shl, Tok::Eof]);
    assert_eq!(toks(">>"), vec![Tok::Shr, Tok::Eof]);
    // Single glyphs are still the comparison operators (greedy `<<`/`>>` does not eat a lone `<`).
    assert_eq!(toks("< >"), vec![Tok::LAngle, Tok::RAngle, Tok::Eof]);
    // `<<<` is `Shl` then a lone `LAngle` (two-then-one), never three separate angles.
    assert_eq!(toks("<<<"), vec![Tok::Shl, Tok::LAngle, Tok::Eof]);
    // In context: `a << b >> c` is three idents around the two shift glyphs.
    assert_eq!(
        toks("a << b >> c"),
        vec![
            Tok::Ident("a".to_owned()),
            Tok::Shl,
            Tok::Ident("b".to_owned()),
            Tok::Shr,
            Tok::Ident("c".to_owned()),
            Tok::Eof,
        ]
    );
}

/// Never-silent (G2): a bare `0t` with no trit glyph is an explicit lex error — never a
/// silently-empty `TritLit` (mirrors the empty-`0b`/`0x` rejections above).
#[test]
fn lex_trit_empty_literal_is_a_lex_error() {
    let err = lex("0t").expect_err("`0t` alone must be a lex error");
    assert!(
        err.to_string().contains("no trits"),
        "error must name the empty-trit-literal cause: {err}"
    );
    // `0t` in a larger source still errors before EOF (no glyph), rather than emitting an empty token.
    lex("fn f() => Ternary{1} = 0t").expect_err("trailing `0t` with no glyph must be a lex error");
}
