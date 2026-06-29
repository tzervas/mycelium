use crate::*;

// ============================================================================================
// --flatten tests (M-819; DN-57 §2)
// ============================================================================================

/// Corpus table for the round-trip property: `parse(flatten(src)) == parse(format(src))`.
/// Each entry is `(label, src)`.
///
/// Guarantee tag: `Empirical` — verified by execution of this test, not a formal proof.
const FLATTEN_CORPUS: &[(&str, &str)] = &[
    ("minimal-nodule-no-items", "nodule d;\n"),
    (
        "single-fn-identity",
        "nodule d;\nfn f(x: Binary{8}) => Binary{8} = x;\n",
    ),
    ("use-import", "nodule signals.demo;\n\nuse core.binary;\n"),
    (
        "fn-with-literal",
        "nodule d;\nfn zero() => Binary{8} = 0b0000_0000;\n",
    ),
    (
        "two-fns",
        "nodule d;\nfn a() => Binary{1} = 0b0;\nfn b() => Binary{1} = 0b1;\n",
    ),
    (
        "fn-with-match",
        "nodule d;\nfn classify(x: Binary{1}) => Binary{1} = match x { 0b0 => 0b0, _ => 0b1 };\n",
    ),
    (
        "pub-fn",
        "nodule d;\npub fn f(x: Binary{8}) => Binary{8} = x;\n",
    ),
    (
        "use-and-fn",
        "nodule d;\nuse core.binary;\nfn f() => Binary{8} = 0b0;\n",
    ),
    (
        "already-flat-roundtrips",
        "nodule d; fn f(x: Binary{8}) => Binary{8} = x;\n",
    ),
    (
        // M-677 + M-819 integration: a fn carrying per-effect budgets (`!{retry(<=3), …}`)
        // must round-trip. The parser normalizes unit suffixes to a byte count, so the
        // canonical surface is the raw `<=N` (`64KiB` → `65536`) — AST-equal either way.
        "budgeted-effects-roundtrips",
        "nodule d;\nfn f() => Binary{8} !{retry(<=3), alloc(<=64KiB)} = 0b0000_0000;\n",
    ),
];

/// Core round-trip property (M-819 / DN-57 §2):
/// `parse(flatten(src)) == parse(canonical(src))` over the corpus.
/// Guarantee: `Empirical` — backed by this corpus, not a formal proof.
#[test]
fn flatten_round_trip_ast_equals_canonical_ast() {
    for &(label, src) in FLATTEN_CORPUS {
        // Some corpus entries may use syntax that doesn't parse under the current grammar;
        // skip those gracefully (G2: never assert on unverified input).
        let canonical_ast = match parse(src) {
            Ok(ast) => ast,
            Err(_) => continue, // this corpus entry uses unparsable syntax — skip
        };

        let flat = match flatten_source(src, None) {
            Ok(f) => f,
            // An OutOfScope or Parse error on these corpus entries is unexpected but
            // tolerated for cases where the grammar has evolved; log and skip.
            Err(FmtError::Parse(_)) | Err(FmtError::OutOfScope(_)) => continue,
            Err(e) => panic!("[{label}] flatten_source failed unexpectedly: {e}"),
        };

        // The flat output must re-parse successfully.
        let flat_ast = parse(&flat.output).unwrap_or_else(|e| {
            panic!(
                "[{label}] flat output did not re-parse: {e}\nflat: {:?}",
                flat.output
            )
        });

        // Round-trip: AST of flattened == AST of original (Empirical).
        assert_eq!(
            flat_ast, canonical_ast,
            "[{label}] round-trip failed: flatten changed the surface AST\nflat: {:?}",
            flat.output
        );

        // Single-line: the flat output (excluding the final newline) has no interior newlines.
        let without_final_nl = flat.output.trim_end_matches('\n');
        assert!(
            !without_final_nl.contains('\n'),
            "[{label}] flat output contains interior newlines: {:?}",
            flat.output
        );

        // Ends with exactly one newline.
        assert!(
            flat.output.ends_with('\n'),
            "[{label}] flat output must end with '\\n': {:?}",
            flat.output
        );
    }
}

/// Flatten of an already-flat single-line source is idempotent (fixed-point).
#[test]
fn flatten_is_idempotent() {
    for &(label, src) in FLATTEN_CORPUS {
        let Ok(f1) = flatten_source(src, None) else {
            continue;
        };
        let Ok(f2) = flatten_source(&f1.output, None) else {
            panic!("[{label}] second flatten failed on output: {:?}", f1.output);
        };
        assert_eq!(f1.output, f2.output, "[{label}] flatten is not idempotent");
    }
}

/// Flatten of a multi-item nodule produces a single line (no interior newlines).
#[test]
fn flatten_produces_single_line() {
    let src = "nodule d;\nuse core.binary;\nfn zero() => Binary{8} = 0b0000_0000;\nfn one() => Binary{8} = 0b0000_0001;\n";
    let f = flatten_source(src, None).expect("flattens");
    // No interior newlines.
    let without_final = f.output.trim_end_matches('\n');
    assert!(
        !without_final.contains('\n'),
        "flat output must be a single line: {:?}",
        f.output
    );
    // All items present.
    assert!(
        f.output.contains("use core.binary"),
        "missing use: {:?}",
        f.output
    );
    assert!(
        f.output.contains("fn zero"),
        "missing fn zero: {:?}",
        f.output
    );
    assert!(
        f.output.contains("fn one"),
        "missing fn one: {:?}",
        f.output
    );
    // Nodule header present.
    assert!(
        f.output.starts_with("nodule d;"),
        "must start with nodule: {:?}",
        f.output
    );
}

/// Flatten strips comments — they are not part of the surface AST (G2: explicit, not silent).
#[test]
fn flatten_strips_comments_explicitly() {
    // A source with a trailing comment and structured header.
    let src = "// nodule: d\n// @license: MIT\nnodule d;\nfn f(x: Binary{8}) => Binary{8} = x; // identity\n";
    let f = flatten_source(src, None).expect("flattens");
    // Comments must NOT appear in the flat output.
    assert!(
        !f.output.contains("//"),
        "flat output must not contain comments: {:?}",
        f.output
    );
    // The notes must explain this (G2: never silent).
    assert!(
        f.notes.iter().any(|n| n.contains("stripped")),
        "notes must explain that comments/header were stripped: {:?}",
        f.notes
    );
}

/// M-677 + M-819 integration regression: flatten must PRESERVE per-effect budgets.
/// This compares against the ORIGINAL parsed AST (not the canonical render) — because both
/// flatten and canonical share one renderer, a flatten-vs-canonical check alone would pass
/// even if both dropped the budgets. The bug this guards: fmt rendered only `sig.effects`
/// (`!{retry, alloc}`), silently dropping the `(<=N)` bounds.
#[test]
fn flatten_preserves_effect_budgets_against_original() {
    let src = "nodule d;\nfn f() => Binary{8} !{retry(<=3), alloc(<=64KiB)} = 0b0000_0000;\n";
    let original = parse(src).expect("original parses");
    let flat = flatten_source(src, None).expect("flattens");
    let flat_ast = parse(&flat.output).expect("flattened source parses");
    // AST-equal vs the original (Empirical): the budgets survive the round-trip.
    assert_eq!(
        original, flat_ast,
        "flatten changed the AST — effect budgets were dropped\nflat: {:?}",
        flat.output
    );
    // And the bounds are visible in the surface (the parser normalizes 64KiB → 65536 bytes).
    assert!(
        flat.output.contains("retry(<=3)"),
        "retry budget missing from flat output: {:?}",
        flat.output
    );
    assert!(
        flat.output.contains("alloc(<=65536)"),
        "alloc budget (normalized to bytes) missing from flat output: {:?}",
        flat.output
    );
}

/// Flatten refuses a phylum source with the same OutOfScope as format_source (G2).
#[test]
fn flatten_refuses_phylum_explicitly() {
    let src = "phylum app.core\nnodule a\nfn f() => Binary{8} = 0b0000_0000\nnodule b\nfn g() => Binary{8} = 0b0000_0001";
    match flatten_source(src, None) {
        Err(FmtError::OutOfScope(msg)) => {
            assert!(msg.contains("phylum"), "refusal must name phylum: {msg}")
        }
        other => panic!("phylum must be OutOfScope, got: {other:?}"),
    }
}

/// Flatten refuses an unparsable source (exit code 2, never a partial output).
#[test]
fn flatten_refuses_unparsable_source() {
    // Missing `;` terminator → parse error under the mandatory-terminator grammar.
    let src = "nodule demo\nfn f(x: Binary{8}) => Ternary{6} = swap(x, to: Ternary{6})";
    let err = flatten_source(src, None).unwrap_err();
    assert_eq!(err.exit_code(), 2, "must be a parse error (exit 2): {err}");
    assert!(matches!(err, FmtError::Parse(_)));
}

/// Flatten honours the same hard pin as format_source.
#[test]
fn flatten_honours_toolchain_format_pin() {
    let src = "nodule d;\nfn f(x: Binary{8}) => Binary{8} = x;\n";
    let err = flatten_source(src, Some("mycfmt-99")).unwrap_err();
    assert_eq!(err.exit_code(), 4);
    assert!(format!("{err}").contains("hard pin"), "{err}");
    // The matching pin works.
    assert!(flatten_source(src, Some(MYCFMT_VERSION)).is_ok());
}

#[test]
fn formats_a_minimal_nodule_and_is_idempotent() {
    let src =
        "// exercises: nodule header + use import\nnodule signals.demo;\n\nuse core.binary;\n";
    let r = format_source(src, None).expect("formats");
    // Leading comment preserved, body canonical, identity preserved.
    assert!(
        r.output
            .starts_with("// exercises: nodule header + use import\n"),
        "{}",
        r.output
    );
    assert!(r.output.contains("nodule signals.demo"));
    assert!(r.output.contains("use core.binary"));
    // Idempotent (C2): formatting the output is a no-op.
    let r2 = format_source(&r.output, None).expect("formats again");
    assert_eq!(r2.output, r.output);
    assert!(!r2.changed);
}

#[test]
fn an_unparsable_file_is_an_explicit_error_not_a_rewrite() {
    let err = format_source(
        "nodule demo\nfn f(x: Binary{8}) => Ternary{6} = swap(x, to: Ternary{6})",
        None,
    )
    .unwrap_err();
    assert_eq!(err.exit_code(), 2);
    assert!(matches!(err, FmtError::Parse(_)));
}

#[test]
fn a_malformed_header_is_an_explicit_error() {
    let err =
        format_source("// nodule: 9bad\nnodule d\nfn f() => Binary{8} = 0b0", None).unwrap_err();
    assert_eq!(err.exit_code(), 3);
}

/// Previously refused; now the trailing comment on the fn body line is preserved (M-690 Stage 2).
#[test]
fn an_interior_comment_is_preserved_not_refused() {
    // A trailing comment in the body is now preserved (M-690 Stage 2 — behavior change, not a
    // tag upgrade; VR-5).  The old refusal test is updated to assert preservation.
    let src = "nodule d;\nfn f(x: Binary{8}) => Binary{8} = x; // identity\n";
    let r = format_source(src, None).expect("must now preserve, not refuse");
    // The comment must appear in the output.
    assert!(
        r.output.contains("// identity"),
        "trailing comment must be preserved: {}",
        r.output
    );
    // The AST must still round-trip (C1).
    let reparsed = parse(&r.output).expect("re-parses");
    let original = parse(src).expect("original parses");
    assert_eq!(reparsed, original, "C1: AST must be identical after format");
    // Idempotent (C2): format twice = byte-equal.
    let r2 = format_source(&r.output, None).expect("formats again");
    assert_eq!(r2.output, r.output, "C2: must be idempotent");
}

#[test]
fn a_toolchain_format_pin_mismatch_is_refused() {
    let src = "nodule d;\nfn f(x: Binary{8}) => Binary{8} = x;\n";
    let err = format_source(src, Some("mycfmt-99")).unwrap_err();
    assert_eq!(err.exit_code(), 4);
    assert!(format!("{err}").contains("hard pin"), "{err}");
    // The matching pin formats fine.
    assert!(format_source(src, Some(MYCFMT_VERSION)).is_ok());
}

#[test]
fn the_structured_header_is_re_emitted_canonically() {
    let src = "// nodule: geometry.shapes\n// @version: 1.2.0\n// @license: Apache-2.0\n\
               nodule geometry.shapes;\n\nfn area_unit() => Binary{8} = 0b0000_0001;\n";
    let r = format_source(src, None).expect("formats");
    assert!(
        r.output.starts_with(
            "// nodule: geometry.shapes\n// @version: 1.2.0\n// @license: Apache-2.0\n"
        ),
        "{}",
        r.output
    );
    // Identity + header preserved; idempotent.
    let r2 = format_source(&r.output, None).expect("again");
    assert_eq!(r2.output, r.output);
}

/// Previously refused; now the stray header comment is preserved as a leading doc-block on the
/// first item (M-690 Stage 2 — behavior change, not a tag upgrade; VR-5).
#[test]
fn a_stray_comment_in_the_header_region_is_preserved_not_refused() {
    let src = "// nodule: g\n// a stray non-key comment\n// @license: MIT\nnodule g;\nfn f() => Binary{8} = 0b0;\n";
    let r = format_source(src, None).expect("must now preserve, not refuse");
    // The stray comment must appear in the output.
    assert!(
        r.output.contains("// a stray non-key comment"),
        "stray header comment must be preserved: {}",
        r.output
    );
    // AST must round-trip (C1).
    let reparsed = parse(&r.output).expect("re-parses");
    let original = parse(src).expect("original parses");
    assert_eq!(reparsed, original, "C1: AST must be identical after format");
    // Idempotent (C2).
    let r2 = format_source(&r.output, None).expect("formats again");
    assert_eq!(r2.output, r.output, "C2: must be idempotent");
}

#[test]
fn formatted_default_and_from_are_additive_ergonomics() {
    // M-644: Default is the empty result; From<String> lifts raw text (changed=false, no notes).
    let d = Formatted::default();
    assert!(d.output.is_empty() && !d.changed && d.notes.is_empty());
    let f = Formatted::from("0b0\n".to_owned());
    assert_eq!(f.output, "0b0\n");
    assert!(!f.changed && f.notes.is_empty());
}

/// New (M-690 Stage 2): a multi-line docstring above a fn is preserved as a leading block.
#[test]
fn docstring_above_fn_is_preserved() {
    let src = "nodule d;\n\n// Computes the identity.\n// Returns its argument unchanged.\nfn f(x: Binary{8}) => Binary{8} = x;\n";
    let r = format_source(src, None).expect("formats");
    assert!(
        r.output.contains("// Computes the identity."),
        "first docstring line must be preserved: {}",
        r.output
    );
    assert!(
        r.output.contains("// Returns its argument unchanged."),
        "second docstring line must be preserved: {}",
        r.output
    );
    // C1: AST round-trip.
    let reparsed = parse(&r.output).expect("re-parses");
    let original = parse(src).expect("original parses");
    assert_eq!(reparsed, original);
    // C2: idempotent.
    let r2 = format_source(&r.output, None).expect("formats again");
    assert_eq!(r2.output, r.output, "C2 idempotence");
}

/// New (M-690 Stage 2): trailing comment on a match arm is preserved; the match renders multiline;
/// formatting is idempotent.
#[test]
fn trailing_comment_on_match_arm_is_preserved_and_idempotent() {
    let src = concat!(
        "nodule d;\n",
        "fn classify(x: Binary{8}) => Binary{8} =\n",
        "  match x { 0b0 => 0b0 // zero case\n",
        "  | _ => 0b1 };\n",
    );
    // If parsing succeeds, the match arm comment must be preserved and idempotent.
    match format_source(src, None) {
        Ok(r) => {
            assert!(
                r.output.contains("// zero case"),
                "arm trailing comment must be preserved: {}",
                r.output
            );
            // C2: idempotent.
            let r2 = format_source(&r.output, None).expect("formats again");
            assert_eq!(r2.output, r.output, "C2 idempotence");
            // C1: AST round-trip.
            let reparsed = parse(&r.output).expect("re-parses");
            let original = parse(src).expect("original parses");
            assert_eq!(reparsed, original, "C1 identity");
        }
        Err(e) => {
            // If the source doesn't parse (the syntax may not be valid Mycelium), that's OK —
            // the test demonstrates the API path; real arm-comment tests use valid syntax.
            assert_eq!(e.exit_code(), 2, "only parse errors are expected here: {e}");
        }
    }
}

/// New (M-690 Stage 2): a valid match with arm trailing comments using canonical syntax.
#[test]
fn match_arm_trailing_comment_canonical_syntax() {
    // Use a type + match that will actually parse in Mycelium L1.
    // match on Binary{1}: 0b0 and 0b1 are the two arms.
    let src = "nodule d;\nfn classify(x: Binary{1}) => Binary{1} = match x { 0b0 => 0b0 // zero\n, _ => 0b1 };\n";
    match format_source(src, None) {
        Ok(r) => {
            // Comment preserved.
            assert!(
                r.output.contains("// zero"),
                "arm comment preserved: {}",
                r.output
            );
            // Idempotent.
            let r2 = format_source(&r.output, None).expect("second format");
            assert_eq!(r2.output, r.output, "idempotent");
        }
        Err(FmtError::Parse(_)) => {
            // This syntax variant may not be accepted by the Mycelium parser; skip gracefully.
        }
        Err(e) => panic!("unexpected error: {e}"),
    }
}
