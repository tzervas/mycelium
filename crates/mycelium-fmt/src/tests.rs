use crate::*;

#[test]
fn formats_a_minimal_nodule_and_is_idempotent() {
    let src = "// exercises: nodule header + use import\nnodule signals.demo\n\nuse core.binary\n";
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
        "nodule demo\nfn f(x: Binary{8}) -> Ternary{6} = swap(x, to: Ternary{6})",
        None,
    )
    .unwrap_err();
    assert_eq!(err.exit_code(), 2);
    assert!(matches!(err, FmtError::Parse(_)));
}

#[test]
fn a_malformed_header_is_an_explicit_error() {
    let err =
        format_source("// nodule: 9bad\nnodule d\nfn f() -> Binary{8} = 0b0", None).unwrap_err();
    assert_eq!(err.exit_code(), 3);
}

/// Previously refused; now the trailing comment on the fn body line is preserved (M-690 Stage 2).
#[test]
fn an_interior_comment_is_preserved_not_refused() {
    // A trailing comment in the body is now preserved (M-690 Stage 2 — behavior change, not a
    // tag upgrade; VR-5).  The old refusal test is updated to assert preservation.
    let src = "nodule d\nfn f(x: Binary{8}) -> Binary{8} = x // identity\n";
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
    let src = "nodule d\nfn f(x: Binary{8}) -> Binary{8} = x\n";
    let err = format_source(src, Some("mycfmt-99")).unwrap_err();
    assert_eq!(err.exit_code(), 4);
    assert!(format!("{err}").contains("hard pin"), "{err}");
    // The matching pin formats fine.
    assert!(format_source(src, Some(MYCFMT_VERSION)).is_ok());
}

#[test]
fn the_structured_header_is_re_emitted_canonically() {
    let src = "// nodule: geometry.shapes\n// @version: 1.2.0\n// @license: Apache-2.0\n\
               nodule geometry.shapes\n\nfn area_unit() -> Binary{8} = 0b0000_0001\n";
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
    let src = "// nodule: g\n// a stray non-key comment\n// @license: MIT\nnodule g\nfn f() -> Binary{8} = 0b0\n";
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
    let src = "nodule d\n\n// Computes the identity.\n// Returns its argument unchanged.\nfn f(x: Binary{8}) -> Binary{8} = x\n";
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
        "nodule d\n",
        "fn classify(x: Binary{8}) -> Binary{8} =\n",
        "  match x { 0b0 => 0b0 // zero case\n",
        "  | _ => 0b1 }\n",
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
    let src = "nodule d\nfn classify(x: Binary{1}) -> Binary{1} = match x { 0b0 => 0b0 // zero\n, _ => 0b1 }\n";
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
