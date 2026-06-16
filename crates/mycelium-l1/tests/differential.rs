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
use mycelium_l1::{check_colony, elaborate, parse, Evaluator, L1Error};
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

/// The **data + recursion fragment** (RFC-0011 r3/r4): with `Construct`/`Match`/`Lam`/`App`/`Fix` now
/// L0 nodes, a program that builds/matches data and recurses elaborates to a closed L0 term. As of
/// **M-342 (Q5 closed)** the obligation is the full three-way differential **L1-eval ≡
/// elaborate→L0-interp ≡ AOT** on the L0 [`CoreValue`] observable — the AOT `aot::run_core`
/// env-machine now covers the data + recursion fragment (it was repr-only in r3). The L1 evaluator's
/// name-keyed data value is bridged onto the elaborated value's content-addressed `#T#i` identity
/// through the *same* registry (`L1Value::to_core`), so a divergence in any of the three machineries —
/// the big-step `try_match`, the Maranget→flat-`Match` lowering, or the ANF env-machine — is caught.
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
        // --- r4: functions + recursion (Lam/App/Fix), now in the fragment ---
        // self-recursion returning a datum (Fix + App + Match)
        "colony d\ntype Nat = Z | S(Nat)\n\
         fn drop_(n: Nat) -> Nat = match n { Z => Z, S(m) => drop_(m) }\n\
         fn main() -> Nat = drop_(S(S(S(Z))))",
        // self-recursion building data on the way back (a recursive copy)
        "colony d\ntype Nat = Z | S(Nat)\n\
         fn copy(n: Nat) -> Nat = match n { Z => Z, S(m) => S(copy(m)) }\n\
         fn main() -> Nat = copy(S(S(Z)))",
        // a `for` fold over a list spine (desugars to a synthesized Fix fold)
        "colony d\ntype Bytes = End | More(Binary{8}, Bytes)\n\
         fn checksum(bs: Bytes) -> Binary{8} = for b in bs, acc = 0b0000_0000 => xor(acc, b)\n\
         fn main() -> Binary{8} = checksum(More(0b1111_0000, More(0b0000_1111, End)))",
        // a recursive helper called by a non-recursive one (inlining + Fix coexist)
        "colony d\ntype Nat = Z | S(Nat)\n\
         fn drop_(n: Nat) -> Nat = match n { Z => Z, S(m) => drop_(m) }\n\
         fn twice_drop(n: Nat) -> Nat = drop_(drop_(n))\n\
         fn main() -> Nat = twice_drop(S(S(Z)))",
        // --- r5: mutual recursion (FixGroup), M-343 ---
        // a mutually-recursive pair returning a datum: ping(SS Z) ⟶ pong(S Z) ⟶ ping(Z) ⟶ Z
        "colony d\ntype Nat = Z | S(Nat)\n\
         fn ping(n: Nat) -> Nat = match n { Z => Z, S(m) => pong(m) }\n\
         fn pong(n: Nat) -> Nat = match n { Z => Z, S(m) => ping(m) }\n\
         fn main() -> Nat = ping(S(S(Z)))",
        // mutual recursion over a Bool result (even/odd): even(SSS Z) ⟶ odd(SS Z) ⟶ … ⟶ False
        "colony d\ntype Nat = Z | S(Nat)\n\
         fn even(n: Nat) -> Bool = match n { Z => True, S(m) => odd(m) }\n\
         fn odd(n: Nat) -> Bool = match n { Z => False, S(m) => even(m) }\n\
         fn main() -> Bool = even(S(S(S(Z))))",
        // mutual recursion that BUILDS data on the way back (constructive through the group):
        // f(SSS Z) ⟶ S(g(SS Z)) ⟶ S(f(S Z)) ⟶ S(S(g(Z))) ⟶ S(S(Z))
        "colony d\ntype Nat = Z | S(Nat)\n\
         fn f(n: Nat) -> Nat = match n { Z => Z, S(m) => S(g(m)) }\n\
         fn g(n: Nat) -> Nat = match n { Z => Z, S(m) => f(m) }\n\
         fn main() -> Nat = f(S(S(S(Z))))",
        // a three-function mutual cycle (f → g → h → f) returning a datum
        "colony d\ntype Nat = Z | S(Nat)\n\
         fn f3(n: Nat) -> Nat = match n { Z => Z, S(m) => g3(m) }\n\
         fn g3(n: Nat) -> Nat = match n { Z => Z, S(m) => h3(m) }\n\
         fn h3(n: Nat) -> Nat = match n { Z => Z, S(m) => f3(m) }\n\
         fn main() -> Nat = f3(S(S(S(S(Z)))))",
    ]
}

#[test]
fn l1_eval_l0_interp_and_aot_agree_on_the_data_and_recursion_fragment() {
    let interp = Interpreter::new(
        PrimRegistry::with_builtins(),
        Box::new(BinaryTernarySwapEngine),
    );
    let prims = PrimRegistry::with_builtins();
    let engine = BinaryTernarySwapEngine;
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
            .unwrap_or_else(|e| panic!("program #{i}: must be in the r3/r4 fragment: {e}"));
        let l0_core = interp
            .eval_core(&node)
            .unwrap_or_else(|e| panic!("program #{i}: L0-interp failed: {e}"));

        // Path 3: the same L0 term through the AOT env-machine (M-342) — now spans data + recursion.
        let aot_core = mycelium_mlir::run_core(&node, &prims, &engine)
            .unwrap_or_else(|e| panic!("program #{i}: AOT run_core failed: {e}"));

        // All three paths must agree on the whole L0 value — constructor identity, fields, and the
        // meet-summary guarantee (for a datum) or repr+payload+guarantee (for a repr value).
        assert_eq!(
            l1_core, l0_core,
            "program #{i} diverged: L1-eval vs elaborate→L0-interp"
        );
        assert_eq!(
            l0_core, aot_core,
            "program #{i} diverged: L0-interp vs AOT env-machine"
        );
        assert_eq!(
            l1_core.guarantee(),
            aot_core.guarantee(),
            "program #{i}: guarantee summaries disagree (L1 vs AOT)"
        );

        // Where the result is a representation value, the shared M-210 TV checker validates each pair
        // too (the same checker the repr fragment uses) — defense in depth, never a bespoke compare.
        if let (CoreValue::Repr(a), CoreValue::Repr(b), CoreValue::Repr(c)) =
            (&l1_core, &l0_core, &aot_core)
        {
            for (x, y, pair) in [(a, b, "L1↔interp"), (b, c, "interp↔AOT")] {
                assert_eq!(
                    check(
                        x,
                        y,
                        RefinementRelation::ObservationalEquiv,
                        Certificate::exact(),
                        &Evidence::Observational,
                    ),
                    CheckVerdict::Validated {
                        strength: GuaranteeStrength::Exact
                    },
                    "program #{i}: the shared checker must validate the {pair} repr-result pair"
                );
            }
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

/// r4: self-recursion is now **in** the fragment — it elaborates to a `Fix` and agrees with the L1
/// evaluator (the differential corpus exercises this). A `Total`-classified recursion still runs on
/// the L1 evaluator too; the two paths agree on the L0 value.
#[test]
fn self_recursion_elaborates_and_agrees() {
    let src = "colony d\ntype Nat = Z | S(Nat)\n\
               fn drop_(n: Nat) -> Nat = match n { Z => Z, S(m) => drop_(m) }\n\
               fn main() -> Nat = drop_(S(S(Z)))";
    let env = check_colony(&parse(src).unwrap()).unwrap();
    let registry = build_registry(&env).unwrap();
    assert_eq!(env.totality["drop_"], mycelium_l1::Totality::Total);

    // Recursion now elaborates (no Residual) — r4 retired the §4.6 refusal for self-recursion.
    let node = elaborate(&env, "main").expect("self-recursion elaborates in r4");
    let l0 = Interpreter::default()
        .eval_core(&node)
        .expect("L0-interp runs");
    let l1 = Evaluator::new(&env)
        .call("main", vec![])
        .unwrap()
        .to_core(&env, &registry)
        .unwrap();
    assert_eq!(
        l1, l0,
        "L1-eval and elaborate→L0-interp agree on the recursive result (Z)"
    );
}

/// **M-343 (R7-Q3):** mutual recursion now elaborates to a `FixGroup`, and all three paths agree —
/// L1-eval ≡ elaborate→L0-interp ≡ AOT — on a mutually-recursive program. (Was the r4 boundary where
/// elaboration refused with a `Residual`; M-343 enacts it.) The broader corpus coverage is in
/// `l1_eval_l0_interp_and_aot_agree_on_the_data_and_recursion_fragment`; this pins the named case.
#[test]
fn mutual_recursion_elaborates_and_all_three_paths_agree() {
    let prims = PrimRegistry::with_builtins();
    let engine = BinaryTernarySwapEngine;
    let src = "colony d\ntype Nat = Z | S(Nat)\n\
               fn ping(n: Nat) -> Nat = match n { Z => Z, S(m) => pong(m) }\n\
               fn pong(n: Nat) -> Nat = match n { Z => Z, S(m) => ping(m) }\n\
               fn main() -> Nat = ping(S(S(Z)))";
    let env = check_colony(&parse(src).unwrap()).unwrap();
    let registry = build_registry(&env).unwrap();

    // Mutual recursion now elaborates (no Residual) — it lowers to a FixGroup.
    let node = elaborate(&env, "main").expect("mutual recursion elaborates to a FixGroup (M-343)");

    let l1 = Evaluator::new(&env)
        .call("main", vec![])
        .unwrap()
        .to_core(&env, &registry)
        .unwrap();
    let l0 = Interpreter::default()
        .eval_core(&node)
        .expect("L0-interp runs the FixGroup");
    let aot = mycelium_mlir::run_core(&node, &prims, &engine).expect("AOT runs the FixGroup");

    assert_eq!(l1, l0, "L1-eval vs elaborate→L0-interp diverged on mutual recursion");
    assert_eq!(l0, aot, "L0-interp vs AOT env-machine diverged on mutual recursion");
    // ping(S(S(Z))) ⟶ pong(S(Z)) ⟶ ping(Z) ⟶ Z (a nullary datum).
    assert_eq!(
        l0,
        mycelium_core::CoreValue::Data(mycelium_core::Datum::new(
            registry.ctor_ref("Nat", 0).unwrap(),
            vec![]
        )),
        "the result must be Nat::Z"
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
