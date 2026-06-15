//! L1 static checking (RFC-0007 §4.4/§4.5): the monomorphic typechecker, the structural totality
//! checker, and the `matured ⟹ total` gate. Every refusal is an explicit `CheckError`.

use mycelium_l1::{check_colony, parse, Totality};

fn check(src: &str) -> Result<mycelium_l1::Env, mycelium_l1::CheckError> {
    let colony = parse(src).expect("parses");
    check_colony(&colony)
}

#[test]
fn well_typed_swap_fn_checks() {
    let env = check(
        "colony d\nfn widen(x: Binary{8}) -> Ternary{6} = swap(x, to: Ternary{6}, policy: rt)",
    )
    .expect("checks");
    assert_eq!(env.totality["widen"], Totality::Total);
}

#[test]
fn type_mismatch_is_explicit() {
    // body is Ternary{6}, signature says Binary{8}.
    let err =
        check("colony d\nfn f(x: Binary{8}) -> Binary{8} = swap(x, to: Ternary{6}, policy: rt)")
            .unwrap_err();
    assert!(err.message.contains("type"), "got: {}", err.message);
}

#[test]
fn exhaustive_match_checks_and_nonexhaustive_is_refused() {
    let ok = "colony d\ntype Sign = Neg | Zero | Pos\nfn f(s: Sign) -> Sign = match s { Neg => s, Zero => s, Pos => s }";
    assert!(check(ok).is_ok());

    let bad = "colony d\ntype Sign = Neg | Zero | Pos\nfn f(s: Sign) -> Sign = match s { Neg => s, Pos => s }";
    let err = check(bad).unwrap_err();
    assert!(
        err.message.contains("non-exhaustive"),
        "got: {}",
        err.message
    );
}

#[test]
fn structural_recursion_is_total_and_gates_matured() {
    // A structurally-decreasing self-recursion over a Peano-like type is classified Total.
    let src = "colony d\n\
               type Nat = Z | S(Nat)\n\
               matured fn count(n: Nat) -> Nat = match n { Z => n, S(m) => count(m) }";
    let env = check(src).expect("checks");
    assert_eq!(env.totality["count"], Totality::Total);
}

#[test]
fn non_decreasing_recursion_cannot_be_matured() {
    // The recursive call passes the parameter unchanged → not structurally smaller → Partial,
    // so `matured` is refused (RFC-0007 §4.5).
    let src = "colony d\n\
               type Nat = Z | S(Nat)\n\
               matured fn spin(n: Nat) -> Nat = match n { Z => n, S(m) => spin(n) }";
    let err = check(src).unwrap_err();
    assert!(err.message.contains("matured"), "got: {}", err.message);
}

#[test]
fn non_decreasing_recursion_is_allowed_when_not_matured() {
    // Same body without `matured` checks fine — partiality is an honest classification, not an error.
    let src = "colony d\n\
               type Nat = Z | S(Nat)\n\
               fn spin(n: Nat) -> Nat = match n { Z => n, S(m) => spin(n) }";
    let env = check(src).expect("checks");
    assert_eq!(env.totality["spin"], Totality::Partial);
}

#[test]
fn shadowing_rebind_does_not_leak_smallness() {
    // A4-01 regression: the inner arm rebinds `m` (matching `p`, an unrelated parameter), shadowing
    // the outer `m` (a piece of `n`). The recursion `f(m, p)` is therefore NOT structural —
    // `f(3,2) → f(1,2) → f(1,2) → …` diverges — so it must be classified Partial. Mutant-witness:
    // without the drop-and-restore shadow handling in descend_walk's Match arm, the stale smallness
    // of the outer `m` leaks in, `f` is wrongly classified Total, and the `matured` form below is
    // wrongly accepted.
    let body = "match n { Z => Z, S(m) => match p { Z => Z, S(m) => f(m, p) } }";
    let src = format!("colony d\ntype Nat = Z | S(Nat)\nfn f(n: Nat, p: Nat) -> Nat = {body}");
    let env = check(&src).expect("checks");
    assert_eq!(env.totality["f"], Totality::Partial);

    let matured =
        format!("colony d\ntype Nat = Z | S(Nat)\nmatured fn f(n: Nat, p: Nat) -> Nat = {body}");
    let err = check(&matured).unwrap_err();
    assert!(err.message.contains("matured"), "got: {}", err.message);
}

#[test]
fn wild_is_denied_by_default() {
    let src = "colony d\nfn f(x: Binary{8}) -> Binary{8} = wild { x }";
    let err = check(src).unwrap_err();
    assert!(err.message.contains("wild"), "got: {}", err.message);
}

#[test]
fn generics_are_an_explicit_deferral_not_a_guess() {
    let src = "colony d\ntype Box<T> = Wrap(T)";
    let err = check(src).unwrap_err();
    assert!(err.message.contains("deferred"), "got: {}", err.message);
}

// --- bounded iteration (RFC-0007 §4.8, r2) ---

const BYTES: &str = "colony d\ntype Bytes = End | More(Binary{8}, Bytes)\n";

#[test]
fn a_for_fold_typechecks_and_is_total() {
    let env = check(&format!(
        "{BYTES}matured fn checksum(bs: Bytes) -> Binary{{8}} =\n    for b in bs, acc = 0b0000_0000 => xor(acc, b)"
    ))
    .expect("checks");
    // Bounded by construction: the fn is non-recursive, so `matured` is admissible.
    assert_eq!(env.totality["checksum"], Totality::Total);
}

#[test]
fn for_over_a_non_linear_type_is_an_explicit_refusal() {
    // A branching (tree) type is outside the v0 linear-recursion shape.
    let err = check(
        "colony d\ntype Tree = Leaf | Node(Tree, Tree)\nfn f(t: Tree) -> Binary{8} =\n    for x in t, acc = 0b0000_0000 => acc",
    )
    .unwrap_err();
    assert!(
        err.message.contains("linearly recursive"),
        "got: {}",
        err.message
    );
}

#[test]
fn for_body_must_yield_the_accumulator_type() {
    let err = check(&format!(
        "{BYTES}fn f(bs: Bytes) -> Binary{{8}} =\n    for b in bs, acc = 0b0000_0000 => <+0->"
    ))
    .unwrap_err();
    assert!(err.message.contains("accumulator"), "got: {}", err.message);
}

#[test]
fn for_over_a_repr_value_is_an_explicit_refusal() {
    let err = check("colony d\nfn f(x: Binary{8}) -> Binary{8} = for b in x, acc = x => acc")
        .unwrap_err();
    assert!(err.message.contains("data value"), "got: {}", err.message);
}

#[test]
fn imperative_words_get_teaching_diagnostics() {
    // Juxtaposition (`while x`) was never valid syntax — the parse error teaches (§4.8).
    let perr = parse("colony d\nfn f(x: Binary{8}) -> Binary{8} = while x").unwrap_err();
    assert!(
        perr.message.contains("for x in xs"),
        "got: {}",
        perr.message
    );
    // Call-shaped use fails name resolution — the check error teaches too.
    let cerr = check("colony d\nfn f(x: Binary{8}) -> Binary{8} = loop(x)").unwrap_err();
    assert!(
        cerr.message.contains("not a Mycelium form"),
        "got: {}",
        cerr.message
    );
}
