use crate::ambient::*;
use crate::ast::{Expr, Item, Literal, Nodule, Paradigm};
use crate::parse;

fn nodule(src: &str) -> Nodule {
    parse(src).expect("parses")
}

#[test]
fn no_ambient_is_the_identity() {
    let c = nodule("nodule d\nfn main() -> Binary{8} = not(0b1011_0010)");
    assert_eq!(resolve(&c).unwrap(), c);
}

#[test]
fn a_paradigm_less_repr_is_filled_and_traced() {
    let c = nodule("nodule d\ndefault paradigm Binary\nfn main() -> {8} = 0b1011_0010");
    let r = resolve_report(&c).unwrap();
    // The `default` item is stripped; the return type is now concrete.
    assert!(!r.nodule.items.iter().any(|i| matches!(i, Item::Default(_))));
    // A provenance note records the fill (EXPLAIN: "where did this paradigm come from?").
    assert!(
        r.notes
            .iter()
            .any(|n| n.paradigm == Paradigm::Binary && n.detail.contains("Binary{8}")),
        "notes: {:?}",
        r.notes
    );
}

#[test]
fn a_with_block_is_stripped_to_its_body() {
    let c = nodule(
            "nodule d\nfn main() -> Ternary{6} = with paradigm Ternary { swap(0b1011_0010, to: {6}, policy: rt) }",
        );
    let r = resolve(&c).unwrap();
    let Some(Item::Fn(fd)) = r.items.iter().find(|i| matches!(i, Item::Fn(_))) else {
        unreachable!("main is present")
    };
    // The `with paradigm` wrapper is gone; the body is the bare swap with a concrete target.
    assert!(matches!(fd.body, Expr::Swap { .. }));
}

#[test]
fn multiple_defaults_are_refused() {
    let c = nodule(
            "nodule d\ndefault paradigm Binary\ndefault paradigm Ternary\nfn main() -> {8} = 0b1011_0010",
        );
    assert!(matches!(
        resolve(&c),
        Err(AmbientError::MultipleDefaults { .. })
    ));
}

#[test]
fn a_shape_mismatch_is_refused() {
    let c = nodule("nodule d\ndefault paradigm Ternary\nfn main() -> {4, F32} = <0+-->");
    assert!(matches!(
        resolve(&c),
        Err(AmbientError::ParadigmShapeMismatch { .. })
    ));
}

#[test]
fn a_wild_body_is_opaque_to_ambient_resolution() {
    // M-661 (Copilot, PR #360): the `wild` body is the trusted/opaque FFI escape — ambient
    // resolution must NOT descend into it. Non-vacuous control: the SAME bare decimal under a
    // `Dense` ambient, OUTSIDE a wild, is a `BareDecimalNoEncoding` refusal (Dense gives a bare
    // decimal no encoding — RFC-0012 §4.3).
    let bad = nodule("nodule d\ndefault paradigm Dense\nfn g() -> Binary{8} = 5");
    assert!(
        matches!(
            resolve(&bad),
            Err(AmbientError::BareDecimalNoEncoding { .. })
        ),
        "control: a bare decimal under a Dense ambient must refuse outside a wild, got: {:?}",
        resolve(&bad)
    );
    // Inside a `wild` body the same literal is preserved **verbatim** — resolution succeeds, no
    // interior refusal (the body is audited, not verified — VR-5; execution staged → Residual).
    // Before the fix the resolver descended and raised `BareDecimalNoEncoding` from inside the
    // opaque body — a surprising refusal for trusted FFI.
    let good = nodule(
            "nodule std.sys.x @std-sys\ndefault paradigm Dense\nfn f() -> Binary{8} !{ffi} = wild { 5 }",
        );
    let r = resolve(&good)
        .expect("the wild body is opaque to ambient resolution — no interior refusal (M-661)");
    let Some(Item::Fn(fd)) = r
        .items
        .iter()
        .find(|i| matches!(i, Item::Fn(f) if f.sig.name == "f"))
    else {
        unreachable!("f is present")
    };
    let Expr::Wild(b) = &fd.body else {
        panic!("the body is still a wild block, got: {:?}", fd.body)
    };
    assert!(
        matches!(**b, Expr::Lit(Literal::Int(5))),
        "the wild body's bare decimal must be untouched (verbatim), got: {b:?}"
    );
}

#[test]
fn the_std_sys_marker_round_trips_through_expand_to_source() {
    // M-661 (Copilot, PR #360): the canonical longhand printer must re-emit `@std-sys`. Dropping
    // it would silently relocate audited `wild` code into a non-`@std-sys` context (changing
    // meaning — G2); `mycelium-lsp`'s `expand_ambient`/EXPLAIN routes through this printer.
    let marked =
        nodule("nodule std.sys.fs @std-sys\nfn read() -> Binary{8} !{ffi} = wild { host() }");
    let resolved = resolve(&marked).expect("a @std-sys nodule resolves");
    assert!(resolved.std_sys, "resolution preserves the marker");
    let printed = expand_to_source(&resolved);
    assert!(
        printed.contains("nodule std.sys.fs @std-sys"),
        "the longhand twin must re-emit `@std-sys`, got:\n{printed}"
    );
    // An unmarked nodule must NOT sprout the marker (the marker is opt-in — never invented).
    let plain = nodule("nodule d\nfn f() -> Binary{8} = 0b0");
    assert!(
        !expand_to_source(&resolve(&plain).unwrap()).contains("@std-sys"),
        "an unmarked nodule must not gain `@std-sys`"
    );
}
