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
use mycelium_core::{GuaranteeStrength, Payload, Repr, Value};
use mycelium_interp::{Interpreter, PrimRegistry};
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
