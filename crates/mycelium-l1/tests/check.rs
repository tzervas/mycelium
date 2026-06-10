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
