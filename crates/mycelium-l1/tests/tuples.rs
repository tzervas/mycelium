//! **M-826** — v0 **tuple/product type** and **chained application** (`f(x)(y)`) tests.
//!
//! Tuples are `(T, U, …)` (arity ≥ 2; `(T)` is grouping only — no 1-tuple). The surface form
//! desugars via **KC-3** (no new L0 node) to a synthetic `Tuple$N` data type with a single
//! constructor `Tuple$N$0`; `$` is not a surface identifier character so the synthetic names
//! cannot collide with user-defined types.
//!
//! Shapes covered:
//! 1. Tuple literal + type annotation (type-checks, evaluates, returns a Data value).
//! 2. Tuple destructuring via `match` / tuple pattern.
//! 3. Nested tuples (tuples-of-tuples).
//! 4. Arity mismatch in tuple pattern — explicit checker error.
//! 5. Chained application `f(x)(y)` — the head of an `App` may be any `Ty::Fn` expression.
//!
//! Guarantee tag: `Empirical` (trial — L1-eval on monomorphized env; VR-5; M-826).

use mycelium_core::Payload;
use mycelium_l1::{check_nodule, monomorphize, parse, Evaluator, L1Value};

/// Evaluate `src` end-to-end via the L1 interpreter and return the `L1Value` result.
fn eval_l1(src: &str) -> L1Value {
    let ast = parse(src).expect("parse");
    let env = check_nodule(&ast).expect("check");
    let mono = monomorphize(&env, "main").expect("mono");
    Evaluator::new(&mono).call("main", vec![]).expect("eval")
}

/// Extract the `Bits` payload from an `L1Value::Repr`. Panics if the value is not a `Bits` repr.
fn bits(v: L1Value) -> Vec<bool> {
    match v.as_repr().expect("Repr value").payload() {
        Payload::Bits(bs) => bs.clone(),
        other => panic!("expected Bits payload, got {other:?}"),
    }
}

/// Assert `src` type-checks and evaluates to a Data value whose ctor contains `ctor_substr`.
fn assert_data_ctor(src: &str, ctor_substr: &str) {
    match eval_l1(src) {
        L1Value::Data { ctor, .. } => assert!(
            ctor.contains(ctor_substr),
            "expected ctor containing `{ctor_substr}`, got `{ctor}`"
        ),
        other => panic!("expected Data value, got {other:?}"),
    }
}

/// Assert `src` parses but fails at the *checker* stage with an error message containing `msg`.
fn assert_check_err(src: &str, msg: &str) {
    let ast = parse(src).expect("parse");
    let err = check_nodule(&ast).expect_err("expected check error");
    let s = format!("{err:?}");
    assert!(
        s.contains(msg),
        "check error of `{src}` = `{s}`, expected to contain `{msg}`"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// 1. Tuple literal and type annotation
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn tuple_literal_pair_roundtrip() {
    // A simple 2-tuple of `Binary{8}` values. The type annotation exercises the `BaseType::Tuple`
    // path through the checker and mono. Evaluated via L1-eval; the synthetic `Tuple$2$0`
    // constructor is built at the KC-3 desugar step.
    let src = "nodule t;\n\
               fn main() => (Binary{8}, Binary{8}) = (0b0000_0001, 0b0000_0010);";
    // The eval returns a `Data { ctor: "Tuple$2$0", fields: […] }`.
    assert_data_ctor(src, "Tuple$2$0");
}

#[test]
fn tuple_literal_triple() {
    // A 3-tuple — exercises arity 3 separately from 2.
    let src = "nodule t;\n\
               fn main() => (Binary{8}, Binary{8}, Binary{8}) =\n\
                 (0b0000_0001, 0b0000_0010, 0b0000_0100);";
    assert_data_ctor(src, "Tuple$3$0");
}

// ──────────────────────────────────────────────────────────────────────────────
// 2. Tuple destructuring (match / tuple pattern)
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn tuple_destructure_first() {
    // Build a pair and project the first component via a tuple pattern.
    let src = "nodule t;\n\
               fn fst(p: (Binary{8}, Binary{8})) => Binary{8} =\n\
                 match p { (x, _) => x };\n\
               fn main() => Binary{8} = fst((0b0000_0001, 0b0000_0010));";
    // 0b0000_0001 = MSB-first: [F,F,F,F,F,F,F,T]
    assert_eq!(
        bits(eval_l1(src)),
        [false, false, false, false, false, false, false, true]
    );
}

#[test]
fn tuple_destructure_second() {
    let src = "nodule t;\n\
               fn snd(p: (Binary{8}, Binary{8})) => Binary{8} =\n\
                 match p { (_, y) => y };\n\
               fn main() => Binary{8} = snd((0b0000_0001, 0b0000_0010));";
    // 0b0000_0010 = MSB-first: [F,F,F,F,F,F,T,F]
    assert_eq!(
        bits(eval_l1(src)),
        [false, false, false, false, false, false, true, false]
    );
}

#[test]
fn tuple_pattern_in_let_binding_via_match() {
    // Pattern in a let-like position via an inline match.
    let src = "nodule t;\n\
               fn main() => Binary{8} =\n\
                 let p = (0b1010_1010, 0b0000_1111) in\n\
                 match p { (a, b) => and(a, b) };";
    // and(0b1010_1010, 0b0000_1111) = 0b0000_1010 = [F,F,F,F,T,F,T,F]
    assert_eq!(
        bits(eval_l1(src)),
        [false, false, false, false, true, false, true, false]
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// 3. Nested tuples
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn nested_tuple_literal() {
    // A 2-tuple whose first component is itself a 2-tuple.
    let src = "nodule t;\n\
               fn main() => ((Binary{8}, Binary{8}), Binary{8}) =\n\
                 ((0b0000_0001, 0b0000_0010), 0b0000_0100);";
    assert_data_ctor(src, "Tuple$2$0");
}

// ──────────────────────────────────────────────────────────────────────────────
// 4. Arity mismatch in tuple pattern — explicit checker error
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn tuple_arity_mismatch_in_pattern_is_error() {
    // A 2-tuple matched with a 3-element tuple pattern is a type error.
    let src = "nodule t;\n\
               fn main() => Binary{8} =\n\
                 match (0b0000_0001, 0b0000_0010) { (a, b, c) => a };";
    assert_check_err(src, "tuple pattern");
}

// ──────────────────────────────────────────────────────────────────────────────
// 5. Chained application f(x)(y) — Part 2 of M-826
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn chained_application_lambda() {
    // `add_curried(0b0000_0001)(0b0000_0010)` where `add_curried` returns a lambda.
    // or(0b0000_0001, 0b0000_0010) = 0b0000_0011 = [F,F,F,F,F,F,T,T]
    let src = "nodule t;\n\
               fn add_curried(a: Binary{8}) => Binary{8} => Binary{8} =\n\
                 lambda(b: Binary{8}) => or(a, b);\n\
               fn main() => Binary{8} = add_curried(0b0000_0001)(0b0000_0010);";
    assert_eq!(
        bits(eval_l1(src)),
        [false, false, false, false, false, false, true, true]
    );
}

#[test]
fn chained_application_via_let() {
    // Build the intermediate function value via a `let`, then apply it — sanity-checks that
    // the head-lifting path in mono/checkty is consistent with the non-chained path.
    // mk_const(0b1111_0000)(anything) = 0b1111_0000 = [T,T,T,T,F,F,F,F]
    let src = "nodule t;\n\
               fn mk_const(a: Binary{8}) => Binary{8} => Binary{8} =\n\
                 lambda(_b: Binary{8}) => a;\n\
               fn main() => Binary{8} =\n\
                 let f = mk_const(0b1111_0000) in f(0b0000_1111);";
    assert_eq!(
        bits(eval_l1(src)),
        [true, true, true, true, false, false, false, false]
    );
}

#[test]
fn chained_application_triple() {
    // Three-level chained application `f(a)(b)(c)` — each prefix returns a function.
    // or(or(0b0001, 0b0010), 0b0100) = 0b0000_0111 = [F,F,F,F,F,T,T,T]
    let src = "nodule t;\n\
               fn curry3(a: Binary{8}) => Binary{8} => Binary{8} => Binary{8} =\n\
                 lambda(b: Binary{8}) => lambda(c: Binary{8}) => or(or(a, b), c);\n\
               fn main() => Binary{8} = curry3(0b0000_0001)(0b0000_0010)(0b0000_0100);";
    assert_eq!(
        bits(eval_l1(src)),
        [false, false, false, false, false, true, true, true]
    );
}
