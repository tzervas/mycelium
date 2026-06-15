//! The RFC-0007 §4.6 **differential obligation** (NFR-7): on the evaluation-complete fragment,
//! **L1-eval**, **elaborate→L0-interp**, and the **M-150 AOT path** must agree on the observable
//! (`repr + payload + guarantee`) — and every agreeing pair validates through the **M-210 shared
//! TV checker** (`mycelium_cert::check`, the `ObservationalEquiv` instance), the same checker
//! that validates swap certificates and the M-151 interp↔AOT differential.
//!
//! All three paths dispatch through the *same* trusted prim registry and certified swap engine;
//! what this test pins down is that the L1 machinery layered on top — the big-step environment
//! evaluator on one side, inlining elaboration on the other — cannot make "two execution paths
//! mean two semantics".
//!
//! Outside the fragment the obligation is different and also tested here: elaboration must refuse
//! with an explicit `Residual` (never a partial artifact), while the L1 evaluator still runs the
//! program — and a `Partial`-classified unproductive recursion ends in an explicit
//! `FuelExhausted`, never a hang (§4.5).

use mycelium_cert::{check, BinaryTernarySwapEngine, CheckVerdict, Evidence, RefinementRelation};
use mycelium_core::{CoreValue, GuaranteeStrength, Payload, Repr, Value};
use mycelium_interp::{Interpreter, PrimRegistry};
use mycelium_l1::elab::build_registry;
use mycelium_l1::{check_colony, elaborate, parse, ElabError, Evaluator, L1Error};
use mycelium_numerics::Certificate;

type Observable<'a> = (&'a Repr, &'a Payload, GuaranteeStrength);

fn observable(v: &Value) -> Observable<'_> {
    (v.repr(), v.payload(), v.meta().guarantee())
}

/// The fragment corpus: checked colonies with a nullary `main` whose bodies inline to
/// `Const/Var/Let/Op/Swap` residue. Each runs on all three paths.
fn corpus() -> Vec<&'static str> {
    vec![
        // bare literal
        "colony d\nfn main() -> Binary{8} = 0b1011_0010",
        // let / var
        "colony d\nfn main() -> Binary{8} = let a = 0b1011_0010 in a",
        // unary + binary bit ops
        "colony d\nfn main() -> Binary{8} = not(0b1011_0010)",
        "colony d\nfn main() -> Binary{8} = xor(0b1011_0010, 0b1111_1111)",
        // balanced-ternary arithmetic (in range — never a silent wrap)
        "colony d\nfn main() -> Ternary{4} = add(<00+->, <0+0->)",
        "colony d\nfn main() -> Ternary{4} = mul(<00+0>, <00-0>)",
        // the certified binary→ternary swap
        "colony d\nfn main() -> Ternary{6} = swap(0b1011_0010, to: Ternary{6}, policy: rt)",
        // a call, inlined (acyclic call graph)
        "colony d\nfn flip(x: Binary{8}) -> Binary{8} = not(x)\nfn main() -> Binary{8} = flip(flip(0b1010_1010))",
        // round-trip swap through a let
        "colony d\nfn main() -> Binary{8} =\n  let b = 0b0010_1010 in swap(swap(b, to: Ternary{6}, policy: rt), to: Binary{8}, policy: rt)",
        // an op feeding a swap, through a helper
        "colony d\nfn widen(x: Binary{8}) -> Ternary{6} = swap(not(x), to: Ternary{6}, policy: rt)\nfn main() -> Ternary{6} = widen(0b1011_0010)",
    ]
}

#[test]
fn l1_eval_l0_interp_and_aot_agree_on_the_fragment() {
    let interp = Interpreter::new(
        PrimRegistry::with_builtins(),
        Box::new(BinaryTernarySwapEngine),
    );
    let prims = PrimRegistry::with_builtins();
    let engine = BinaryTernarySwapEngine;

    for (i, src) in corpus().iter().enumerate() {
        let env = check_colony(&parse(src).expect("parses")).expect("checks");

        // Path 1: the L1 fuel-guarded evaluator.
        let l1 = Evaluator::new(&env)
            .call("main", vec![])
            .unwrap_or_else(|e| panic!("program #{i}: L1-eval failed: {e}"));
        let l1 = l1
            .as_repr()
            .unwrap_or_else(|| panic!("program #{i}: fragment result must be a repr value"))
            .clone();

        // Path 2: elaborate to L0, run on the reference interpreter.
        let node = elaborate(&env, "main")
            .unwrap_or_else(|e| panic!("program #{i}: must be in the fragment: {e}"));
        let l0 = interp
            .eval(&node)
            .unwrap_or_else(|e| panic!("program #{i}: L0-interp failed: {e}"));

        // Path 3: the same L0 term through the AOT path (M-150).
        let aot = mycelium_mlir::run(&node, &prims, &engine)
            .unwrap_or_else(|e| panic!("program #{i}: AOT failed: {e}"));

        assert_eq!(
            observable(&l1),
            observable(&l0),
            "program #{i} diverged: L1-eval vs L0-interp"
        );
        assert_eq!(
            observable(&l0),
            observable(&aot),
            "program #{i} diverged: L0-interp vs AOT"
        );

        // M-210: each agreeing pair validates through the one shared TV checker.
        for (a, b, pair) in [(&l1, &l0, "L1↔interp"), (&l0, &aot, "interp↔AOT")] {
            assert_eq!(
                check(
                    a,
                    b,
                    RefinementRelation::ObservationalEquiv,
                    Certificate::exact(),
                    &Evidence::Observational,
                ),
                CheckVerdict::Validated {
                    strength: GuaranteeStrength::Exact
                },
                "program #{i}: the shared checker must validate the {pair} pair"
            );
        }
    }
}

/// The **data-and-matching fragment** (RFC-0011 r3): with `Construct`/`Match` now L0 nodes, a
/// non-recursive program that builds/matches data elaborates to a closed L0 term. The obligation is
/// **L1-eval ≡ elaborate→L0-interp** on the L0 [`CoreValue`] observable (the AOT path stays
/// repr-only in r3 — Q5). The L1 evaluator's name-keyed data value is bridged onto the elaborated
/// value's content-addressed `#T#i` identity through the *same* registry (`L1Value::to_core`), so a
/// divergence in either machinery — the big-step `try_match` or the Maranget→flat-`Match` lowering —
/// is caught.
fn data_corpus() -> Vec<&'static str> {
    vec![
        // a flat data match returning a repr value
        "colony d\ntype Sign = Neg | Zero | Pos\n\
         fn label(s: Sign) -> Ternary{1} = match s { Neg => <->, Zero => <0>, _ => <+> }\n\
         fn main() -> Ternary{1} = label(Zero)",
        // a data RESULT (the program evaluates to a datum)
        "colony d\ntype Nat = Z | S(Nat)\nfn main() -> Nat = S(S(Z))",
        // nested patterns (Maranget) returning a datum
        "colony d\ntype Nat = Z | S(Nat)\n\
         fn pred2(n: Nat) -> Nat = match n { Z => Z, S(Z) => Z, S(S(m)) => m }\n\
         fn main() -> Nat = pred2(S(S(S(Z))))",
        // a literal-pattern match over a Binary scrutinee
        "colony d\nfn classify(b: Binary{4}) -> Ternary{1} = \
         match b { 0b0000 => <0>, 0b1111 => <+>, _ => <-> }\n\
         fn main() -> Ternary{1} = classify(0b1111)",
        // a data value with a repr field, destructured (binds a field, runs a prim on it)
        "colony d\ntype Box = Mk(Binary{8})\n\
         fn flip(x: Box) -> Binary{8} = match x { Mk(b) => not(b) }\n\
         fn main() -> Binary{8} = flip(Mk(0b1010_1010))",
        // `if` desugaring to a Bool match
        "colony d\nfn pick(b: Bool) -> Binary{8} = if b then 0b1111_1111 else 0b0000_0000\n\
         fn main() -> Binary{8} = pick(True)",
        // a constructed result carrying a computed repr field
        "colony d\ntype Box = Mk(Binary{8})\nfn main() -> Box = Mk(not(0b0000_1111))",
    ]
}

#[test]
fn l1_eval_and_l0_interp_agree_on_the_data_fragment() {
    let interp = Interpreter::new(
        PrimRegistry::with_builtins(),
        Box::new(BinaryTernarySwapEngine),
    );
    for (i, src) in data_corpus().iter().enumerate() {
        let env = check_colony(&parse(src).expect("parses")).expect("checks");
        let registry = build_registry(&env).expect("the data registry builds");

        // Path 1: the L1 fuel-guarded evaluator, projected onto the L0 CoreValue domain.
        let l1 = Evaluator::new(&env)
            .call("main", vec![])
            .unwrap_or_else(|e| panic!("program #{i}: L1-eval failed: {e}"));
        let l1_core = l1
            .to_core(&env, &registry)
            .unwrap_or_else(|| panic!("program #{i}: L1 result is outside the r3 data fragment"));

        // Path 2: elaborate to L0, run on the reference interpreter (eval_core spans repr + data).
        let node = elaborate(&env, "main")
            .unwrap_or_else(|e| panic!("program #{i}: must be in the r3 fragment: {e}"));
        let l0_core = interp
            .eval_core(&node)
            .unwrap_or_else(|e| panic!("program #{i}: L0-interp failed: {e}"));

        // The two paths must agree on the whole L0 value — constructor identity, fields, and the
        // meet-summary guarantee (for a datum) or repr+payload+guarantee (for a repr value).
        assert_eq!(
            l1_core, l0_core,
            "program #{i} diverged: L1-eval vs elaborate→L0-interp"
        );
        assert_eq!(
            l1_core.guarantee(),
            l0_core.guarantee(),
            "program #{i}: guarantee summaries disagree"
        );

        // Where the result is a representation value, the shared M-210 TV checker validates the pair
        // too (the same checker the repr fragment uses) — defense in depth, never a bespoke compare.
        if let (CoreValue::Repr(a), CoreValue::Repr(b)) = (&l1_core, &l0_core) {
            assert_eq!(
                check(
                    a,
                    b,
                    RefinementRelation::ObservationalEquiv,
                    Certificate::exact(),
                    &Evidence::Observational,
                ),
                CheckVerdict::Validated {
                    strength: GuaranteeStrength::Exact
                },
                "program #{i}: the shared checker must validate the repr-result pair"
            );
        }
    }
}

/// A **mutant-witness** for the elaboration: a deliberately wrong elaboration must be caught by the
/// differential. We construct a divergence directly — two structurally different data programs whose
/// L0 values must *not* compare equal — confirming the data comparison discriminates (a vacuous
/// `assert_eq!` that always passed would be the bug this guards against).
#[test]
fn the_data_differential_distinguishes_divergent_elaborations() {
    let env = |src| check_colony(&parse(src).unwrap()).unwrap();
    let reg = |e: &mycelium_l1::Env| build_registry(e).unwrap();
    let e1 = env("colony d\ntype Nat = Z | S(Nat)\nfn main() -> Nat = S(Z)");
    let e2 = env("colony d\ntype Nat = Z | S(Nat)\nfn main() -> Nat = S(S(Z))");
    let a = Evaluator::new(&e1)
        .call("main", vec![])
        .unwrap()
        .to_core(&e1, &reg(&e1))
        .unwrap();
    let b = Evaluator::new(&e2)
        .call("main", vec![])
        .unwrap()
        .to_core(&e2, &reg(&e2))
        .unwrap();
    assert_ne!(a, b, "S(Z) and S(S(Z)) must be distinct L0 data values");
}

/// Sanity: the harness discriminates — the shared checker explicitly rejects a genuinely
/// divergent pair, so a passing differential is meaningful, not vacuous.
#[test]
fn the_differential_distinguishes_different_programs() {
    let env = |src| check_colony(&parse(src).unwrap()).unwrap();
    let e1 = env("colony d\nfn main() -> Binary{8} = 0b1011_0010");
    let e2 = env("colony d\nfn main() -> Binary{8} = 0b1111_1111");
    let a = Evaluator::new(&e1).call("main", vec![]).unwrap();
    let b = Evaluator::new(&e2).call("main", vec![]).unwrap();
    let verdict = check(
        a.as_repr().unwrap(),
        b.as_repr().unwrap(),
        RefinementRelation::ObservationalEquiv,
        Certificate::exact(),
        &Evidence::Observational,
    );
    assert!(
        matches!(verdict, CheckVerdict::NotValidated { .. }),
        "the checker must reject a divergent pair, got {verdict:?}"
    );
}

/// Outside the fragment: elaboration refuses with an explicit `Residual` (never a partial
/// artifact) while the L1 evaluator runs the program — the §4.6 split, both halves honest.
#[test]
fn outside_the_fragment_elaboration_refuses_and_l1_eval_runs() {
    let src = "colony d\ntype Nat = Z | S(Nat)\n\
               fn drop_(n: Nat) -> Nat = match n { Z => Z, S(m) => drop_(m) }\n\
               fn main() -> Nat = drop_(S(S(Z)))";
    let env = check_colony(&parse(src).unwrap()).unwrap();

    // Total by structural descent — so the totality classifier admits it…
    assert_eq!(
        env.totality["drop_"],
        mycelium_l1::Totality::Total,
        "structural descent classifies drop_ as Total"
    );
    // …elaboration still refuses (recursion is Fix — outside the fragment)…
    assert!(matches!(
        elaborate(&env, "main"),
        Err(ElabError::Residual { .. })
    ));
    // …and the L1 evaluator runs it to a value within fuel ("checked total" = terminates for
    // every sufficiently large fuel, §4.5).
    let v = Evaluator::new(&env).call("main", vec![]).expect("runs");
    assert_eq!(
        v,
        mycelium_l1::L1Value::Data {
            ty: "Nat".into(),
            ctor: "Z".into(),
            fields: vec![]
        },
        "drop_(S(S(Z))) reduces to Z"
    );
}

/// A `Partial`-classified unproductive recursion: still runnable, but the clock is the guard —
/// an explicit `FuelExhausted`, never a hang (§4.5).
#[test]
fn a_partial_program_exhausts_fuel_explicitly() {
    let src = "colony d\ntype Nat = Z | S(Nat)\n\
               fn spin(n: Nat) -> Nat = spin(n)\nfn main() -> Nat = spin(Z)";
    let env = check_colony(&parse(src).unwrap()).unwrap();
    assert_eq!(env.totality["spin"], mycelium_l1::Totality::Partial);
    let err = Evaluator::new(&env)
        .with_fuel(50)
        .call("main", vec![])
        .unwrap_err();
    assert_eq!(err, L1Error::FuelExhausted);
}
