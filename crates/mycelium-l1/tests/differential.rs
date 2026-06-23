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

use mycelium_cert::{
    check, check_core, BinaryTernarySwapEngine, CheckVerdict, Evidence, RefinementRelation,
};
use mycelium_core::{GuaranteeStrength, Payload, Repr, Value};
use mycelium_interp::{Interpreter, PrimRegistry};
use mycelium_l1::elab::build_registry;
use mycelium_l1::{check_nodule, elaborate, monomorphize, parse, Evaluator, L1Error};
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
        "nodule d\nfn main() -> Binary{8} = 0b1011_0010",
        // let / var
        "nodule d\nfn main() -> Binary{8} = let a = 0b1011_0010 in a",
        // unary + binary bit ops
        "nodule d\nfn main() -> Binary{8} = not(0b1011_0010)",
        "nodule d\nfn main() -> Binary{8} = xor(0b1011_0010, 0b1111_1111)",
        // balanced-ternary arithmetic (in range — never a silent wrap)
        "nodule d\nfn main() -> Ternary{4} = add(<00+->, <0+0->)",
        "nodule d\nfn main() -> Ternary{4} = mul(<00+0>, <00-0>)",
        // the certified binary→ternary swap
        "nodule d\nfn main() -> Ternary{6} = swap(0b1011_0010, to: Ternary{6}, policy: rt)",
        // a call, inlined (acyclic call graph)
        "nodule d\nfn flip(x: Binary{8}) -> Binary{8} = not(x)\nfn main() -> Binary{8} = flip(flip(0b1010_1010))",
        // round-trip swap through a let
        "nodule d\nfn main() -> Binary{8} =\n  let b = 0b0010_1010 in swap(swap(b, to: Ternary{6}, policy: rt), to: Binary{8}, policy: rt)",
        // an op feeding a swap, through a helper
        "nodule d\nfn widen(x: Binary{8}) -> Ternary{6} = swap(not(x), to: Ternary{6}, policy: rt)\nfn main() -> Ternary{6} = widen(0b1011_0010)",
        // --- M-666: the `colony { hypha … }` structured-concurrency surface (RFC-0008 §4.7) ---
        // The reference semantics is the RT2 spawn-order sequentialization (RFC-0008 §4.2), so all
        // three execution paths (L1-eval ≡ elaborate→L0-interp ≡ AOT) must agree on it like any
        // other in-fragment program — a single-hypha colony is exactly its body.
        "nodule d\nfn main() -> Binary{8} = colony { hypha not(0b1011_0010) }",
        // A multi-hypha colony: leading hyphae are evaluated for effect (here pure), the observable
        // is the last hypha's value (no v0 product type). Determinism: the value is independent of
        // any scheduling — the sequentialization is the meaning.
        "nodule d\nfn compute(x: Binary{8}) -> Binary{8} = not(x)\n\
         fn main() -> Binary{8} =\n  colony { hypha compute(0b0000_1111), hypha compute(0b1010_1010), hypha xor(0b1111_0000, 0b0000_1111) }",
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
        let env = check_nodule(&parse(src).expect("parses")).expect("checks");

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
        "nodule d\ntype Sign = Neg | Zero | Pos\n\
         fn label(s: Sign) -> Ternary{1} = match s { Neg => <->, Zero => <0>, _ => <+> }\n\
         fn main() -> Ternary{1} = label(Zero)",
        // a data RESULT (the program evaluates to a datum)
        "nodule d\ntype Nat = Z | S(Nat)\nfn main() -> Nat = S(S(Z))",
        // nested patterns (Maranget) returning a datum
        "nodule d\ntype Nat = Z | S(Nat)\n\
         fn pred2(n: Nat) -> Nat = match n { Z => Z, S(Z) => Z, S(S(m)) => m }\n\
         fn main() -> Nat = pred2(S(S(S(Z))))",
        // a literal-pattern match over a Binary scrutinee
        "nodule d\nfn classify(b: Binary{4}) -> Ternary{1} = \
         match b { 0b0000 => <0>, 0b1111 => <+>, _ => <-> }\n\
         fn main() -> Ternary{1} = classify(0b1111)",
        // a data value with a repr field, destructured (binds a field, runs a prim on it)
        "nodule d\ntype Box = Mk(Binary{8})\n\
         fn flip(x: Box) -> Binary{8} = match x { Mk(b) => not(b) }\n\
         fn main() -> Binary{8} = flip(Mk(0b1010_1010))",
        // `if` desugaring to a Bool match
        "nodule d\nfn pick(b: Bool) -> Binary{8} = if b then 0b1111_1111 else 0b0000_0000\n\
         fn main() -> Binary{8} = pick(True)",
        // a constructed result carrying a computed repr field
        "nodule d\ntype Box = Mk(Binary{8})\nfn main() -> Box = Mk(not(0b0000_1111))",
        // a multi-field constructor matched with a NESTED wildcard at a non-root occurrence
        // (M-320 Maranget: column ordering over two fields + a `_` at occurrence [1]) — the kind of
        // decision tree the flat Nat cases don't stress; all three paths must still agree
        "nodule d\ntype Pair = Mk(Bool, Bool)\n\
         fn both(p: Pair) -> Bool = match p { Mk(True, b) => b, Mk(False, _) => False }\n\
         fn main() -> Bool = both(Mk(True, False))",
        // --- r4: functions + recursion (Lam/App/Fix), now in the fragment ---
        // self-recursion returning a datum (Fix + App + Match)
        "nodule d\ntype Nat = Z | S(Nat)\n\
         fn drop_(n: Nat) -> Nat = match n { Z => Z, S(m) => drop_(m) }\n\
         fn main() -> Nat = drop_(S(S(S(Z))))",
        // self-recursion building data on the way back (a recursive copy)
        "nodule d\ntype Nat = Z | S(Nat)\n\
         fn copy(n: Nat) -> Nat = match n { Z => Z, S(m) => S(copy(m)) }\n\
         fn main() -> Nat = copy(S(S(Z)))",
        // a `for` fold over a list spine (desugars to a synthesized Fix fold)
        "nodule d\ntype Bytes = End | More(Binary{8}, Bytes)\n\
         fn checksum(bs: Bytes) -> Binary{8} = for b in bs, acc = 0b0000_0000 => xor(acc, b)\n\
         fn main() -> Binary{8} = checksum(More(0b1111_0000, More(0b0000_1111, End)))",
        // a recursive helper called by a non-recursive one (inlining + Fix coexist)
        "nodule d\ntype Nat = Z | S(Nat)\n\
         fn drop_(n: Nat) -> Nat = match n { Z => Z, S(m) => drop_(m) }\n\
         fn twice_drop(n: Nat) -> Nat = drop_(drop_(n))\n\
         fn main() -> Nat = twice_drop(S(S(Z)))",
        // --- r5: mutual recursion (FixGroup), M-343 ---
        // a mutually-recursive pair returning a datum: ping(SS Z) ⟶ pong(S Z) ⟶ ping(Z) ⟶ Z
        "nodule d\ntype Nat = Z | S(Nat)\n\
         fn ping(n: Nat) -> Nat = match n { Z => Z, S(m) => pong(m) }\n\
         fn pong(n: Nat) -> Nat = match n { Z => Z, S(m) => ping(m) }\n\
         fn main() -> Nat = ping(S(S(Z)))",
        // mutual recursion over a Bool result (even/odd): even(SSS Z) ⟶ odd(SS Z) ⟶ … ⟶ False
        "nodule d\ntype Nat = Z | S(Nat)\n\
         fn even(n: Nat) -> Bool = match n { Z => True, S(m) => odd(m) }\n\
         fn odd(n: Nat) -> Bool = match n { Z => False, S(m) => even(m) }\n\
         fn main() -> Bool = even(S(S(S(Z))))",
        // mutual recursion that BUILDS data on the way back (constructive through the group):
        // f(SSS Z) ⟶ S(g(SS Z)) ⟶ S(f(S Z)) ⟶ S(S(g(Z))) ⟶ S(S(Z))
        "nodule d\ntype Nat = Z | S(Nat)\n\
         fn f(n: Nat) -> Nat = match n { Z => Z, S(m) => S(g(m)) }\n\
         fn g(n: Nat) -> Nat = match n { Z => Z, S(m) => f(m) }\n\
         fn main() -> Nat = f(S(S(S(Z))))",
        // a three-function mutual cycle (f → g → h → f) returning a datum
        "nodule d\ntype Nat = Z | S(Nat)\n\
         fn f3(n: Nat) -> Nat = match n { Z => Z, S(m) => g3(m) }\n\
         fn g3(n: Nat) -> Nat = match n { Z => Z, S(m) => h3(m) }\n\
         fn h3(n: Nat) -> Nat = match n { Z => Z, S(m) => f3(m) }\n\
         fn main() -> Nat = f3(S(S(S(S(Z)))))",
        // --- M-391 (R7-Q3 surface): two further surface-written mutual-recursion shapes ---
        // a mutual pair returning a REPR (not a datum): hi(SS Z) ⟶ lo(S Z) ⟶ hi(Z) ⟶ 0b1111_1111
        "nodule d\ntype Nat = Z | S(Nat)\n\
         fn hi(n: Nat) -> Binary{8} = match n { Z => 0b1111_1111, S(m) => lo(m) }\n\
         fn lo(n: Nat) -> Binary{8} = match n { Z => 0b0000_0000, S(m) => hi(m) }\n\
         fn main() -> Binary{8} = hi(S(S(Z)))",
        // a mutual pair destructuring a MULTI-FIELD constructor (Maranget over two fields, inside a
        // FixGroup): shrink(Mk(S Z, S Z)) ⟶ grow(Mk(Z, S Z)) ⟶ shrink(Mk(Z, Z)) ⟶ Z
        "nodule d\ntype Nat = Z | S(Nat)\ntype Two = Mk(Nat, Nat)\n\
         fn shrink(t: Two) -> Nat = match t { Mk(Z, b) => b, Mk(S(a), b) => grow(Mk(a, b)) }\n\
         fn grow(t: Two) -> Nat = match t { Mk(a, Z) => a, Mk(a, S(b)) => shrink(Mk(a, b)) }\n\
         fn main() -> Nat = shrink(Mk(S(Z), S(Z)))",
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
        let env = check_nodule(&parse(src).expect("parses")).expect("checks");
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

        // The single shared M-210 checker validates each pair through `check_core` — now over the
        // **whole** `CoreValue` (datum *or* repr), so the data + recursion fragment's *datum* results
        // validate through the same checker the repr fragment uses, closing M-302's "through the
        // M-210 ObservationalEquiv checker" obligation for the full kernel corpus (never a bespoke
        // structural compare; a mislabeled lowering is an explicit `NotValidated`, not a silent pass).
        for (x, y, pair) in [
            (&l1_core, &l0_core, "L1↔interp"),
            (&l0_core, &aot_core, "interp↔AOT"),
        ] {
            assert_eq!(
                check_core(x, y),
                CheckVerdict::Validated {
                    strength: GuaranteeStrength::Exact
                },
                "program #{i}: the shared checker must validate the {pair} result pair"
            );
        }
    }
}

/// The arities (member counts) of **every** `FixGroup` in `n`, in pre-order — a small structural probe
/// so the M-391 identity assertion can confirm a surface group lowered to *exactly* the FixGroup(s)
/// expected (count *and* size, not just the first one). Walks the whole term, including inside each
/// `FixGroup`'s member lambdas and body, so a nested or spurious group cannot hide.
fn fixgroup_arities(n: &mycelium_core::Node) -> Vec<usize> {
    use mycelium_core::{Alt, Node};
    match n {
        Node::FixGroup { defs, body } => {
            let mut v = vec![defs.len()];
            for (_, d) in defs {
                v.extend(fixgroup_arities(d));
            }
            v.extend(fixgroup_arities(body));
            v
        }
        Node::Let { bound, body, .. } => {
            let mut v = fixgroup_arities(bound);
            v.extend(fixgroup_arities(body));
            v
        }
        Node::Fix { body, .. } | Node::Lam { body, .. } => fixgroup_arities(body),
        Node::App { func, arg } => {
            let mut v = fixgroup_arities(func);
            v.extend(fixgroup_arities(arg));
            v
        }
        Node::Op { args, .. } | Node::Construct { args, .. } => {
            args.iter().flat_map(fixgroup_arities).collect()
        }
        Node::Swap { src, .. } => fixgroup_arities(src),
        Node::Match {
            scrutinee,
            alts,
            default,
        } => {
            let mut v = fixgroup_arities(scrutinee);
            for alt in alts {
                match alt {
                    Alt::Ctor { body, .. } | Alt::Lit { body, .. } => {
                        v.extend(fixgroup_arities(body))
                    }
                }
            }
            if let Some(d) = default {
                v.extend(fixgroup_arities(d));
            }
            v
        }
        Node::Const(_) | Node::Var(_) => Vec::new(),
    }
}

/// M-391 / ADR-003 (identity-first): a mutually-recursive group written in surface syntax lowers to
/// *the* `FixGroup` the SCC decomposition dictates — deterministically (same source ⟶ byte-equal term,
/// so the content hash is stable) and materialized as that concrete, content-addressed L0 node (the
/// grouping is reified, never a black box; walked here). There is a single `FixGroup` emission path
/// (RP-6 nodule-wide visibility feeds the existing Tarjan→`FixGroup` lowering; DN-13), so
/// "surface-written ≡ programmatic" is pinned here against that canonical path.
#[test]
fn surface_mutual_recursion_lowers_to_the_canonical_fixgroup() {
    let src = "nodule d\ntype Nat = Z | S(Nat)\n\
        fn ping(n: Nat) -> Nat = match n { Z => Z, S(m) => pong(m) }\n\
        fn pong(n: Nat) -> Nat = match n { Z => Z, S(m) => ping(m) }\n\
        fn main() -> Nat = ping(S(S(Z)))";
    let env = check_nodule(&parse(src).expect("parses")).expect("checks");

    // Determinism: the lowering (fresh-name numbering, group member order) is reproducible, so the
    // term — and therefore its content hash — is stable across elaborations of the same source.
    let a = elaborate(&env, "main").expect("elaborates");
    let b = elaborate(&env, "main").expect("elaborates");
    assert_eq!(
        a, b,
        "elaboration must be deterministic (stable content identity)"
    );

    // The surface ping/pong group is materialized as exactly one 2-member `FixGroup` — the concrete,
    // content-addressed L0 node that reifies the grouping (no black box). Walking the whole term must
    // find exactly one `FixGroup`, of arity 2 (uniqueness — not merely "the first one encountered").
    assert_eq!(
        fixgroup_arities(&a),
        vec![2],
        "the surface ping/pong group must lower to exactly one 2-member FixGroup"
    );
}

/// Never-silent (G2): RP-6 makes top-level functions mutually visible, so `ping` may forward-reference
/// `pong` — but a reference to a function that does **not** exist must stay an explicit checker error,
/// never silently absorbed into the mutual group as a phantom member. Here `pongg` is a typo for
/// `pong`; the program must be REJECTED at check time, not elaborated.
#[test]
fn an_undefined_reference_is_an_explicit_error_not_a_silent_mutual_group() {
    let src = "nodule d\ntype Nat = Z | S(Nat)\n\
        fn ping(n: Nat) -> Nat = match n { Z => Z, S(m) => pongg(m) }\n\
        fn pong(n: Nat) -> Nat = match n { Z => Z, S(m) => ping(m) }\n\
        fn main() -> Nat = ping(S(Z))";
    let nodule = parse(src).expect("parses");
    let err = check_nodule(&nodule).expect_err("an undefined reference must be rejected");
    let msg = err.to_string();
    assert!(
        msg.contains("pongg"),
        "the error must explicitly name the undefined reference; got: {msg}"
    );
}

/// A **mutant-witness** for the elaboration: a deliberately wrong elaboration must be caught by the
/// differential. We construct a divergence directly — two structurally different data programs whose
/// L0 values must *not* compare equal — confirming the data comparison discriminates (a vacuous
/// `assert_eq!` that always passed would be the bug this guards against).
#[test]
fn the_data_differential_distinguishes_divergent_elaborations() {
    let env = |src| check_nodule(&parse(src).unwrap()).unwrap();
    let reg = |e: &mycelium_l1::Env| build_registry(e).unwrap();
    let e1 = env("nodule d\ntype Nat = Z | S(Nat)\nfn main() -> Nat = S(Z)");
    let e2 = env("nodule d\ntype Nat = Z | S(Nat)\nfn main() -> Nat = S(S(Z))");
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
    // And the shared M-210 checker must *report* the divergence on the datum pair — a mislabeled
    // lowering is an explicit `NotValidated`, not merely unequal (M-302; NFR-7/VR-4).
    assert!(
        matches!(check_core(&a, &b), CheckVerdict::NotValidated { .. }),
        "the shared checker must reject the divergent datum pair, not silently pass"
    );
}

/// Sanity: the harness discriminates — the shared checker explicitly rejects a genuinely
/// divergent pair, so a passing differential is meaningful, not vacuous.
#[test]
fn the_differential_distinguishes_different_programs() {
    let env = |src| check_nodule(&parse(src).unwrap()).unwrap();
    let e1 = env("nodule d\nfn main() -> Binary{8} = 0b1011_0010");
    let e2 = env("nodule d\nfn main() -> Binary{8} = 0b1111_1111");
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
    let src = "nodule d\ntype Nat = Z | S(Nat)\n\
               fn drop_(n: Nat) -> Nat = match n { Z => Z, S(m) => drop_(m) }\n\
               fn main() -> Nat = drop_(S(S(Z)))";
    let env = check_nodule(&parse(src).unwrap()).unwrap();
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
    let src = "nodule d\ntype Nat = Z | S(Nat)\n\
               fn ping(n: Nat) -> Nat = match n { Z => Z, S(m) => pong(m) }\n\
               fn pong(n: Nat) -> Nat = match n { Z => Z, S(m) => ping(m) }\n\
               fn main() -> Nat = ping(S(S(Z)))";
    let env = check_nodule(&parse(src).unwrap()).unwrap();
    let registry = build_registry(&env).unwrap();

    // The mutually-recursive group structurally descends on position 0, so the totality checker
    // classifies it `Total` (M-343 / R7-Q3 mutual-descent classification, RFC-0007 §4.5).
    assert_eq!(env.totality["ping"], mycelium_l1::Totality::Total);
    assert_eq!(env.totality["pong"], mycelium_l1::Totality::Total);

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

    assert_eq!(
        l1, l0,
        "L1-eval vs elaborate→L0-interp diverged on mutual recursion"
    );
    assert_eq!(
        l0, aot,
        "L0-interp vs AOT env-machine diverged on mutual recursion"
    );
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

/// **M-352 (RFC-0014):** an explicit recovery handling site elaborates to an L0 `Match` over a
/// **result sum** — recovery introduces **no new kernel node** (KC-3). `handle e { Ok(v) => v,
/// Err(_) => fallback }` *is* a `Match` on `Result = Ok | Err`, the data+match fragment the three-way
/// differential already covers; this pins the named recovery case: L1-eval ≡ elaborate→L0-interp ≡
/// AOT. (The concrete `handle` spelling is KC-2-gated, RFC-0006; the semantics is this match.)
#[test]
fn recovery_match_over_a_result_sum_agrees_three_ways() {
    let prims = PrimRegistry::with_builtins();
    let engine = BinaryTernarySwapEngine;
    // Written in the existing data+match surface (no new syntax) — the lowering target of a recovery
    // handling site: match the result sum, recover the `Err` case with an explicit fallback.
    let src = "nodule d\n\
               type Result = Ok(Binary{8}) | Err(Binary{8})\n\
               fn recover(r: Result) -> Binary{8} = match r { Ok(v) => v, Err(e) => 0b0000_0000 }\n\
               fn main() -> Binary{8} = recover(Err(0b1111_1111))";
    let env = check_nodule(&parse(src).unwrap()).unwrap();
    let registry = build_registry(&env).unwrap();
    let node = elaborate(&env, "main").expect("a recovery match elaborates (no new kernel node)");

    let l1 = Evaluator::new(&env)
        .call("main", vec![])
        .unwrap()
        .to_core(&env, &registry)
        .unwrap();
    let l0 = Interpreter::default()
        .eval_core(&node)
        .expect("L0-interp runs the recovery match");
    let aot = mycelium_mlir::run_core(&node, &prims, &engine).expect("AOT runs the recovery match");

    assert_eq!(
        l1, l0,
        "L1-eval vs elaborate→L0-interp diverged on the recovery match"
    );
    assert_eq!(
        l0, aot,
        "L0-interp vs AOT env-machine diverged on the recovery match"
    );
    // The `Err(_)` arm recovers to the explicit fallback `0b0000_0000`.
    let recovered = l0.as_repr().expect("a Binary result value");
    assert_eq!(recovered.repr(), &Repr::Binary { width: 8 });
    assert_eq!(
        recovered.payload(),
        &Payload::Bits(vec![false; 8]),
        "the recovery fallback must be the zero byte"
    );
}

/// **M-353 (RFC-0014 §4.8):** wiring the recovery `Budgets` ledger into the env-machine must be
/// **meaning-preserving** (NFR-7). The same recovery match, run through the env-machine with an *ample*
/// effect ledger threaded (`run_core_with_effects`), produces the identical observable as the plain
/// `run_core` / L0-interp paths — the §4.8 budget plumbing perturbs *nothing* when budgets suffice; it
/// only adds the explicit, graceful `EffectBudget` refusal at an overrun (tested on the runtime path in
/// `mycelium-mlir`). This pins the L0 touch-point of the integration to the three-way differential.
#[test]
fn the_effect_ledger_is_meaning_preserving_on_the_recovery_match() {
    use mycelium_interp::{Budgets, EffectBudget};
    let prims = PrimRegistry::with_builtins();
    let engine = BinaryTernarySwapEngine;
    let src = "nodule d\n\
               type Result = Ok(Binary{8}) | Err(Binary{8})\n\
               fn recover(r: Result) -> Binary{8} = match r { Ok(v) => v, Err(e) => 0b0000_0000 }\n\
               fn main() -> Binary{8} = recover(Err(0b1111_1111))";
    let env = check_nodule(&parse(src).unwrap()).unwrap();
    let node = elaborate(&env, "main").unwrap();

    let plain = mycelium_mlir::run_core(&node, &prims, &engine).unwrap();
    // An ample `alloc` budget — never overruns on this shallow match — must not change the observable.
    let mut budgets = Budgets::new().with(EffectBudget::Bytes(1 << 30));
    let with_ledger = mycelium_mlir::run_core_with_effects(
        &node,
        &prims,
        &engine,
        1_000_000,
        1_000_000,
        &mut budgets,
    )
    .unwrap();
    assert_eq!(
        plain, with_ledger,
        "threading an ample effect ledger must be observable-transparent (NFR-7)"
    );
}

/// A `Partial`-classified unproductive recursion: still runnable, but the clock is the guard —
/// an explicit `FuelExhausted`, never a hang (§4.5).
#[test]
fn a_partial_program_exhausts_fuel_explicitly() {
    let src = "nodule d\ntype Nat = Z | S(Nat)\n\
               fn spin(n: Nat) -> Nat = spin(n)\nfn main() -> Nat = spin(Z)";
    let env = check_nodule(&parse(src).unwrap()).unwrap();
    assert_eq!(env.totality["spin"], mycelium_l1::Totality::Partial);
    let err = Evaluator::new(&env)
        .with_fuel(50)
        .call("main", vec![])
        .unwrap_err();
    assert_eq!(err, L1Error::FuelExhausted);
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// M-666 (redone): the `colony` **RT2 real-concurrency differential** — concurrent ≡ sequential.
//
// RFC-0008 §4.2/§4.7/RT2: the reference semantics of a deterministic concurrent program is its
// deterministic sequentialization. `mycelium_l1::elaborate` lowers a colony to that sequentialization
// (the oracle); `mycelium_l1::elaborate_colony` lowers it to per-hypha closed L0 programs which
// `mycelium_mlir::run_colony` runs as **real concurrent tasks** (`Scope`/`Colony`, structured
// fork/join, M-357), validating concurrent ≡ sequential. These tests pin that the concurrent
// observable equals the sequential reference every other path already agrees on — Empirically (a
// differential over a corpus + a property, not a `Proven` theorem; VR-5).
// ─────────────────────────────────────────────────────────────────────────────────────────────────

/// Run `entry`'s colony through `mycelium_mlir::run_colony` (real concurrent execution of the
/// per-hypha L0 programs) and return the colony's observable as a `CoreValue`.
fn run_colony_concurrent(env: &mycelium_l1::Env, entry: &str) -> mycelium_core::CoreValue {
    let prims = PrimRegistry::with_builtins();
    let engine = BinaryTernarySwapEngine;
    let hyphae = mycelium_l1::elaborate_colony(env, entry)
        .expect("the colony elaborates to its per-hypha L0 programs");
    mycelium_mlir::run_colony(&hyphae, &prims, &engine, 1_000_000, 1_000_000)
        .expect("the colony runs concurrently and the schedules agree (RT2)")
}

/// **The RT2 real-concurrency differential (M-666).** For each colony in the corpus, the **concurrent**
/// run (`run_colony`, real interleaved tasks) must produce the **identical** observable as the
/// **sequential reference** — both the L1 evaluator's spawn-order run and the `elaborate`→interp
/// sequentialization — and every agreeing pair validates through the shared M-210 checker. This is the
/// faithful RT2 obligation (RFC-0008 §4.6): "concurrent observable ≡ the deterministic reference's
/// observable", now over an executor that genuinely interleaves the hyphae (not a sequential stand-in).
#[test]
fn colony_concurrent_run_equals_the_sequential_reference_rt2() {
    let interp = Interpreter::new(
        PrimRegistry::with_builtins(),
        Box::new(BinaryTernarySwapEngine),
    );
    // Colonies of varied shapes: a single hypha (degenerate), pure multi-hypha, hyphae that call a
    // helper and a recursive function (exercising the shared recursive-binder prelude per hypha).
    let corpus = [
        "nodule d\nfn main() -> Binary{8} = colony { hypha not(0b1011_0010) }",
        "nodule d\nfn compute(x: Binary{8}) -> Binary{8} = not(x)\n\
         fn main() -> Binary{8} =\n  \
         colony { hypha compute(0b0000_1111), hypha compute(0b1010_1010), hypha xor(0b1111_0000, 0b0000_1111) }",
        // a hypha that drives a recursive (Total) function — the per-hypha prelude must carry the `Fix`
        "nodule d\ntype Nat = Z | S(Nat)\n\
         fn depth(n: Nat) -> Binary{8} = match n { Z => 0b0000_0000, S(m) => not(depth(m)) }\n\
         fn main() -> Binary{8} =\n  \
         colony { hypha depth(S(S(Z))), hypha not(0b0000_0001), hypha depth(S(Z)) }",
        // a swap reached through a helper call — the colony spans the repr-conversion fragment too.
        // (A hypha body is an `app_expr`, the prior M-666 surface — KEEP; the `swap` keyword form is
        // wrapped in `widen`, exactly the existing differential corpus's `widen` pattern.)
        "nodule d\nfn widen(x: Binary{8}) -> Ternary{6} = swap(x, to: Ternary{6}, policy: rt)\n\
         fn keep(x: Ternary{6}) -> Ternary{6} = x\n\
         fn main() -> Ternary{6} =\n  \
         colony { hypha keep(<00+0-+>), hypha widen(0b1011_0010) }",
    ];

    for (i, src) in corpus.iter().enumerate() {
        let env = check_nodule(&parse(src).expect("parses")).expect("checks");

        // The sequential reference, two independent ways (the RT2 oracle).
        let l1_seq = Evaluator::new(&env)
            .call("main", vec![])
            .unwrap_or_else(|e| panic!("colony #{i}: L1-eval (sequential reference) failed: {e}"));
        let l1_seq = l1_seq
            .as_repr()
            .unwrap_or_else(|| panic!("colony #{i}: reference result must be a repr value"))
            .clone();
        let seq_node = elaborate(&env, "main")
            .unwrap_or_else(|e| panic!("colony #{i}: sequentialization must elaborate: {e}"));
        let interp_seq = interp
            .eval(&seq_node)
            .unwrap_or_else(|e| panic!("colony #{i}: elaborate→interp (reference) failed: {e}"));

        // The CONCURRENT run (real interleaved tasks) — the heart of M-666.
        let concurrent = run_colony_concurrent(&env, "main");
        let concurrent = concurrent
            .as_repr()
            .unwrap_or_else(|| panic!("colony #{i}: concurrent result must be a repr value"))
            .clone();

        // RT2: the concurrent observable equals the sequential reference (both ways).
        assert_eq!(
            observable(&concurrent),
            observable(&l1_seq),
            "colony #{i}: concurrent run diverged from the L1 sequential reference (RT2 violated)"
        );
        assert_eq!(
            observable(&concurrent),
            observable(&interp_seq),
            "colony #{i}: concurrent run diverged from the elaborate→interp reference (RT2 violated)"
        );

        // The shared M-210 checker validates the concurrent↔reference pair like any other equivalence.
        assert_eq!(
            check(
                &concurrent,
                &interp_seq,
                RefinementRelation::ObservationalEquiv,
                Certificate::exact(),
                &Evidence::Observational,
            ),
            CheckVerdict::Validated {
                strength: GuaranteeStrength::Exact
            },
            "colony #{i}: the shared checker must validate the concurrent↔reference pair"
        );
    }
}

/// **Property: the bound on the RT2 differential.** For *any* number `k` of leading hyphae (0..=8), a
/// colony whose hyphae are pure unary ops run **concurrently** (`run_colony`) yields exactly the
/// **last** hypha's value — i.e. the concurrent observable is independent of the `k` leading hyphae and
/// equal to the deterministic sequentialization (RFC-0008 RT2). This is the empirical-confidence
/// breadth behind the `Empirical` determinism tag: many shapes, all concurrent ≡ sequential.
#[test]
fn prop_colony_concurrent_value_is_its_last_hypha_for_any_leading_count() {
    use mycelium_core::{Payload, Repr};
    for k in 0u32..=8 {
        // k leading `not(<i>)` hyphae (evaluated concurrently for effect), then a fixed last hypha
        // whose value (`not(0b0101_0101) = 0b1010_1010`) is the colony's observable for every k.
        let mut hyphae = String::new();
        for i in 0..k {
            let bits = format!("{:08b}", i & 0xFF);
            let bits = format!("{}_{}", &bits[..4], &bits[4..]);
            hyphae.push_str(&format!("hypha not(0b{bits}), "));
        }
        hyphae.push_str("hypha not(0b0101_0101)");
        let src = format!("nodule d\nfn main() -> Binary{{8}} = colony {{ {hyphae} }}");
        let env = check_nodule(&parse(&src).expect("parses")).expect("checks");

        let concurrent = run_colony_concurrent(&env, "main");
        let v = concurrent
            .as_repr()
            .unwrap_or_else(|| panic!("k={k}: concurrent colony result must be a repr value"));
        assert_eq!(v.repr(), &Repr::Binary { width: 8 });
        assert_eq!(
            v.payload(),
            &Payload::Bits(vec![true, false, true, false, true, false, true, false]),
            "k={k}: the CONCURRENT colony's value must equal its LAST hypha (RT2), \
             independent of the {k} leading hyphae"
        );

        // And it equals the sequential reference for this k (the differential, parameterised).
        let seq = Evaluator::new(&env).call("main", vec![]).unwrap();
        assert_eq!(
            observable(seq.as_repr().unwrap()),
            observable(v),
            "k={k}: concurrent ≡ sequential reference (RT2)"
        );
    }
}

/// **A hypha's explicit failure is surfaced, never silently dropped (G2/RT4/I1).** `run_colony`
/// requires every hypha to complete cleanly; a hypha whose L0 evaluation fails (here a deliberate
/// `FuelExhausted` from a starved fuel budget on a recursive hypha) is reported as an explicit
/// `ColonyError::HyphaFailed` carrying its index — never absorbed into a "successful" colony.
#[test]
fn a_failing_hypha_is_an_explicit_colony_error_not_a_silent_drop() {
    let prims = PrimRegistry::with_builtins();
    let engine = BinaryTernarySwapEngine;
    // A Total recursion that needs more than the tiny fuel we give it → an explicit FuelExhausted in
    // that hypha's L0 evaluation; the colony must surface it, not return the last hypha's value.
    let src = "nodule d\ntype Nat = Z | S(Nat)\n\
               fn depth(n: Nat) -> Binary{8} = match n { Z => 0b0000_0000, S(m) => not(depth(m)) }\n\
               fn main() -> Binary{8} =\n  \
               colony { hypha depth(S(S(S(S(S(Z)))))), hypha not(0b0000_0001) }";
    let env = check_nodule(&parse(src).expect("parses")).expect("checks");
    let hyphae = mycelium_l1::elaborate_colony(&env, "main").expect("elaborates per-hypha");
    // Starve fuel so the recursive hypha #0 cannot finish — an explicit, graceful refusal.
    let err = mycelium_mlir::run_colony(&hyphae, &prims, &engine, 2, 1_000_000).expect_err(
        "a starved recursive hypha must make the colony refuse, never silently succeed",
    );
    match err {
        mycelium_mlir::ColonyError::HyphaFailed { index, outcome } => {
            assert_eq!(index, 0, "the failing hypha is #0 (the recursive one)");
            assert!(
                outcome.contains("Fuel") || outcome.contains("Failed"),
                "the failure is the explicit evaluator refusal; got: {outcome}"
            );
        }
        other => panic!("expected an explicit HyphaFailed, got: {other}"),
    }
}

// --- M-673: monomorphization — generics + traits to closed L0, three-way differential ----------
//
// After M-673 a generic *instantiation* and a *trait-method call* both elaborate to closed L0 (the
// monomorphization pre-pass — `crate::mono`). The obligation is the SAME three-way differential as
// the data/recursion corpus (L1-eval ≡ elaborate→L0-interp ≡ AOT), but run on the **monomorphized
// env**: the L1 evaluator has no trait-method dispatch (`eval_app` resolves only `env.fns`/ctor/prim),
// so a trait program is only runnable once mono has rewritten its trait calls to direct calls. Running
// the *generic* cases on the mono'd env too keeps the harness uniform (a generic call's head name is
// already in `env.fns`, so L1-eval would also run the source env — but the mono'd env is the honest
// common ground for both kinds).

/// The generic + trait fragment corpus (mirrors `data_corpus`): each program has a nullary `main`
/// whose reachable graph uses generics and/or a trait/impl, and monomorphizes to closed L0.
fn generic_corpus() -> Vec<&'static str> {
    vec![
        // (1) `List<A>` + `first_or` → closed L0 (the M-673 acceptance fixture)
        "nodule d\ntype List<A> = Nil | Cons(A, List<A>)\n\
         fn first_or<A>(xs: List<A>, d: A) -> A = match xs { Nil => d, Cons(x, _) => x }\n\
         fn main() -> Binary{8} = first_or(Cons(0b0000_0001, Nil), 0b0000_0000)",
        // (2) a generic returning a datum (the program evaluates to a `List<Binary{8}>`)
        "nodule d\ntype List<A> = Nil | Cons(A, List<A>)\n\
         fn main() -> List<Binary{8}> = Cons(0b0000_0001, Nil)",
        // (3) a trait + impl, the method called directly (static resolution to a direct call)
        "nodule d\ntrait Cmp<A> { fn cmp(a: A, b: A) -> Binary{2} }\n\
         impl Cmp<Binary{8}> for Binary{8} { fn cmp(a: Binary{8}, b: Binary{8}) -> Binary{2} = 0b00 }\n\
         fn main() -> Binary{2} = cmp(0b0000_0001, 0b0000_0010)",
        // (4) a bounded generic `use_cmp<T: Cmp>` calling the trait method through its bound, at Binary{8}
        "nodule d\ntrait Cmp<A> { fn cmp(a: A, b: A) -> Binary{2} }\n\
         impl Cmp<Binary{8}> for Binary{8} { fn cmp(a: Binary{8}, b: Binary{8}) -> Binary{2} = 0b00 }\n\
         fn use_cmp<T: Cmp>(a: T, b: T) -> Binary{2} = cmp(a, b)\n\
         fn main() -> Binary{2} = use_cmp(0b0000_0001, 0b0000_0010)",
        // (5) fragmentation witness — `first_or` at Binary{8} AND Binary{4} reachable from one main
        "nodule d\ntype List<A> = Nil | Cons(A, List<A>)\n\
         fn first_or<A>(xs: List<A>, d: A) -> A = match xs { Nil => d, Cons(x, _) => x }\n\
         fn lo() -> Binary{4} = first_or(Cons(0b0001, Nil), 0b0000)\n\
         fn hi() -> Binary{8} = first_or(Cons(0b0000_0001, Nil), 0b0000_0000)\n\
         fn main() -> Binary{8} = let _w = lo() in hi()",
        // (6) a generic recursive fold over a generic spine (Fix over List<Binary{8}>)
        "nodule d\ntype List<A> = Nil | Cons(A, List<A>)\n\
         fn sum_(xs: List<Binary{8}>) -> Binary{8} = \
           match xs { Nil => 0b0000_0000, Cons(x, r) => xor(x, sum_(r)) }\n\
         fn main() -> Binary{8} = sum_(Cons(0b0000_1111, Cons(0b1111_0000, Nil)))",
        // (7) a generic instantiated at a USER DATA TYPE as the type arg (not just reprs) — exercises
        //     the repr/data-name mangling boundary end-to-end (the locus of the M-673 injectivity fix)
        "nodule d\ntype Bit = O | I\ntype Box<A> = Wrap(A)\n\
         fn unbox(b: Box<Bit>) -> Bit = match b { Wrap(x) => x }\n\
         fn main() -> Bit = unbox(Wrap(I))",
    ]
}

#[test]
fn l1_eval_l0_interp_and_aot_agree_on_the_monomorphized_generic_and_trait_fragment() {
    let interp = Interpreter::new(
        PrimRegistry::with_builtins(),
        Box::new(BinaryTernarySwapEngine),
    );
    let prims = PrimRegistry::with_builtins();
    let engine = BinaryTernarySwapEngine;
    for (i, src) in generic_corpus().iter().enumerate() {
        let env = check_nodule(&parse(src).expect("parses")).expect("checks");
        // Monomorphize: a closed, trait-free, monomorphic env L1-eval can run (it has no trait
        // dispatch). The entry stays `main` (nullary monomorphic ⇒ name unchanged).
        let mono = monomorphize(&env, "main")
            .unwrap_or_else(|e| panic!("program #{i}: must monomorphize: {e}"));
        // The mono'd env has no generics/traits left (the M-673 closure invariant).
        assert!(
            mono.fns.values().all(|fd| fd.sig.params.is_empty())
                && mono.types.values().all(|d| d.params.is_empty())
                && mono.traits.is_empty()
                && mono.instances.is_empty()
                && mono.impls.is_empty(),
            "program #{i}: monomorphized env must be closed (no generics/traits)"
        );
        let registry = build_registry(&mono).expect("the mono'd data registry builds");

        // Path 1: the L1 fuel-guarded evaluator, on the MONOMORPHIZED env (trait calls are now direct).
        let l1 = Evaluator::new(&mono)
            .call("main", vec![])
            .unwrap_or_else(|e| panic!("program #{i}: L1-eval failed: {e}"));
        let l1_core = l1
            .to_core(&mono, &registry)
            .unwrap_or_else(|| panic!("program #{i}: L1 result is outside the r3 data fragment"));

        // Path 2: elaborate to L0 (elaborate monomorphizes internally; on the source env it produces
        // the same closed term), run on the reference interpreter.
        let node = elaborate(&env, "main")
            .unwrap_or_else(|e| panic!("program #{i}: must elaborate after M-673: {e}"));
        let l0_core = interp
            .eval_core(&node)
            .unwrap_or_else(|e| panic!("program #{i}: L0-interp failed: {e}"));

        // Path 3: the same L0 term through the AOT env-machine.
        let aot_core = mycelium_mlir::run_core(&node, &prims, &engine)
            .unwrap_or_else(|e| panic!("program #{i}: AOT run_core failed: {e}"));

        assert_eq!(
            l1_core, l0_core,
            "program #{i} diverged: L1-eval(mono) vs elaborate→L0-interp"
        );
        assert_eq!(
            l0_core, aot_core,
            "program #{i} diverged: L0-interp vs AOT env-machine"
        );
        // The single shared M-210 checker validates each pair (a mislabeled lowering is an explicit
        // NotValidated, never a silent pass).
        for (x, y, pair) in [
            (&l1_core, &l0_core, "L1↔interp"),
            (&l0_core, &aot_core, "interp↔AOT"),
        ] {
            assert_eq!(
                check_core(x, y),
                CheckVerdict::Validated {
                    strength: GuaranteeStrength::Exact
                },
                "program #{i}: the shared checker must validate the {pair} pair"
            );
        }
    }
}

/// Determinism across the boundary (M-673): monomorphizing twice yields a byte-equal `Env`, and
/// elaborating the same source twice yields a byte-equal L0 term — the content identity the swarm's
/// hashing relies on. (Identity is *fragmented* per instantiation, but each is stable.)
#[test]
fn monomorphization_and_its_elaboration_are_deterministic() {
    for src in generic_corpus() {
        let env = check_nodule(&parse(src).expect("parses")).expect("checks");
        let a = monomorphize(&env, "main").expect("mono a");
        let b = monomorphize(&env, "main").expect("mono b");
        assert_eq!(
            format!("{a:?}"),
            format!("{b:?}"),
            "monomorphization must be deterministic"
        );
        let ea = elaborate(&env, "main").expect("elab a");
        let eb = elaborate(&env, "main").expect("elab b");
        assert_eq!(
            ea, eb,
            "elaboration of a mono'd program must be deterministic"
        );
    }
}

/// A **mutant-witness** for the monomorphized differential: two structurally different trait/generic
/// programs must NOT produce equal L0 values — confirming the comparison discriminates (a vacuous
/// `assert_eq!` would be the bug this guards). Here two impls give the method different bodies.
#[test]
fn the_monomorphized_differential_distinguishes_divergent_instances() {
    let run = |src: &str| {
        let env = check_nodule(&parse(src).unwrap()).unwrap();
        let node = elaborate(&env, "main").unwrap();
        Interpreter::new(
            PrimRegistry::with_builtins(),
            Box::new(BinaryTernarySwapEngine),
        )
        .eval_core(&node)
        .unwrap()
    };
    // Same trait + call shape, different impl method body (`0b00` vs `0b11`) ⇒ different L0 results.
    let a = run(
        "nodule d\ntrait Cmp<A> { fn cmp(a: A, b: A) -> Binary{2} }\n\
         impl Cmp<Binary{8}> for Binary{8} { fn cmp(a: Binary{8}, b: Binary{8}) -> Binary{2} = 0b00 }\n\
         fn main() -> Binary{2} = cmp(0b0000_0001, 0b0000_0010)",
    );
    let b = run(
        "nodule d\ntrait Cmp<A> { fn cmp(a: A, b: A) -> Binary{2} }\n\
         impl Cmp<Binary{8}> for Binary{8} { fn cmp(a: Binary{8}, b: Binary{8}) -> Binary{2} = 0b11 }\n\
         fn main() -> Binary{2} = cmp(0b0000_0001, 0b0000_0010)",
    );
    assert_ne!(
        a, b,
        "different impl bodies must yield different L0 values (the differential discriminates)"
    );
}

// --- M-688: HOF differential — named fns passed to map/and_then/fold over Result ----------------
//
// RFC-0024 §4 (M-685/686/687): a named top-level function is now a first-class value; the
// monomorphizer (mono.rs) specializes the HOF combinator at the call site (defunctionalization),
// yielding closed first-order L0. The obligation is the SAME three-way differential as the
// generic/trait corpus (L1-eval ≡ elaborate→L0-interp ≡ AOT) on the MONOMORPHIZED env — run on
// the mono'd env so L1-eval (which has no HOF dispatch) can run the same program. Differential
// agreement is `Empirical` (trials; VR-5). Contract is `Declared`.
//
// All programs include the std.result combinators inline (inlining from lib/std/result.myc so each
// program is a self-contained source string; the checker sees the full nodule). Named helpers:
//   `not_val(x: Binary{8}) -> Binary{8} = not(x)` — the function value passed to map/and_then
//   `mk_ok_inner(x: Binary{8}) -> Result<Binary{8},Binary{8}> = Ok(not(x))` — for and_then
//   `id_val(x: Binary{8}) -> Binary{8} = x`           — on_ok branch for fold
//   `const_zero(e: Binary{8}) -> Binary{8} = xor(e, e)` — on_err branch for fold (always 0)

/// The HOF corpus: programs using map/and_then/fold with named function arguments, inline.
/// Each must monomorphize to closed L0 — the defunctionalization obligation (RFC-0024 §4).
///
/// Empirical: differential agreement confirmed by the three-way harness below; not a proof.
fn hof_corpus() -> Vec<&'static str> {
    vec![
        // (1) map Ok: map(Ok(0b0000_0001), not_val) → Ok(not(0b0000_0001)) = Ok(0b1111_1110)
        "nodule d\n\
         type Result<A, E> = Ok(A) | Err(E)\n\
         fn map<A, B, E>(r: Result<A, E>, f: A -> B) -> Result<B, E> =\n  \
           match r { Ok(x) => Ok(f(x)), Err(e) => Err(e) }\n\
         fn not_val(x: Binary{8}) -> Binary{8} = not(x)\n\
         fn mk_ok() -> Result<Binary{8},Binary{8}> = Ok(0b0000_0001)\n\
         fn main() -> Result<Binary{8},Binary{8}> = map(mk_ok(), not_val)",
        // (2) map Err: map(Err(0b1111_1111), not_val) → Err(0b1111_1111) [Err passes through]
        "nodule d\n\
         type Result<A, E> = Ok(A) | Err(E)\n\
         fn map<A, B, E>(r: Result<A, E>, f: A -> B) -> Result<B, E> =\n  \
           match r { Ok(x) => Ok(f(x)), Err(e) => Err(e) }\n\
         fn not_val(x: Binary{8}) -> Binary{8} = not(x)\n\
         fn mk_err() -> Result<Binary{8},Binary{8}> = Err(0b1111_1111)\n\
         fn main() -> Result<Binary{8},Binary{8}> = map(mk_err(), not_val)",
        // (3) and_then Ok: and_then(Ok(0b0000_0001), mk_ok_inner) → Ok(not(0b0000_0001)) = Ok(0b1111_1110)
        "nodule d\n\
         type Result<A, E> = Ok(A) | Err(E)\n\
         fn and_then<A, B, E>(r: Result<A, E>, f: A -> Result<B, E>) -> Result<B, E> =\n  \
           match r { Ok(x) => f(x), Err(e) => Err(e) }\n\
         fn mk_ok_inner(x: Binary{8}) -> Result<Binary{8},Binary{8}> = Ok(not(x))\n\
         fn mk_ok() -> Result<Binary{8},Binary{8}> = Ok(0b0000_0001)\n\
         fn main() -> Result<Binary{8},Binary{8}> = and_then(mk_ok(), mk_ok_inner)",
        // (4) and_then Err: and_then(Err(0b1111_1111), mk_ok_inner) → Err(0b1111_1111) [short-circuits]
        "nodule d\n\
         type Result<A, E> = Ok(A) | Err(E)\n\
         fn and_then<A, B, E>(r: Result<A, E>, f: A -> Result<B, E>) -> Result<B, E> =\n  \
           match r { Ok(x) => f(x), Err(e) => Err(e) }\n\
         fn mk_ok_inner(x: Binary{8}) -> Result<Binary{8},Binary{8}> = Ok(not(x))\n\
         fn mk_err() -> Result<Binary{8},Binary{8}> = Err(0b1111_1111)\n\
         fn main() -> Result<Binary{8},Binary{8}> = and_then(mk_err(), mk_ok_inner)",
        // (5) fold Ok: fold(Ok(0b1010_1010), id_val, const_zero) → id_val(0b1010_1010) = 0b1010_1010
        "nodule d\n\
         type Result<A, E> = Ok(A) | Err(E)\n\
         fn fold<A, E, B>(r: Result<A, E>, on_ok: A -> B, on_err: E -> B) -> B =\n  \
           match r { Ok(x) => on_ok(x), Err(e) => on_err(e) }\n\
         fn id_val(x: Binary{8}) -> Binary{8} = x\n\
         fn const_zero(e: Binary{8}) -> Binary{8} = xor(e, e)\n\
         fn mk_ok() -> Result<Binary{8},Binary{8}> = Ok(0b1010_1010)\n\
         fn main() -> Binary{8} = fold(mk_ok(), id_val, const_zero)",
        // (6) fold Err: fold(Err(0b1111_0000), id_val, const_zero) → xor(0b1111_0000,0b1111_0000) = 0b0000_0000
        "nodule d\n\
         type Result<A, E> = Ok(A) | Err(E)\n\
         fn fold<A, E, B>(r: Result<A, E>, on_ok: A -> B, on_err: E -> B) -> B =\n  \
           match r { Ok(x) => on_ok(x), Err(e) => on_err(e) }\n\
         fn id_val(x: Binary{8}) -> Binary{8} = x\n\
         fn const_zero(e: Binary{8}) -> Binary{8} = xor(e, e)\n\
         fn mk_err() -> Result<Binary{8},Binary{8}> = Err(0b1111_0000)\n\
         fn main() -> Binary{8} = fold(mk_err(), id_val, const_zero)",
    ]
}

/// **M-688 (RFC-0024 §4):** HOF programs using named fn arguments to map/and_then/fold over
/// Result run through the three-way differential — L1-eval(mono) ≡ elaborate→L0-interp ≡ AOT —
/// on the MONOMORPHIZED env. This is the end-to-end proof that static defunctionalization (M-687)
/// produces closed first-order L0 that agrees on all three evaluation paths. Mirrors the
/// `l1_eval_l0_interp_and_aot_agree_on_the_monomorphized_generic_and_trait_fragment` harness.
///
/// Empirical: differential agreement is by trial (VR-5 — never Proven). Declared: type contract.
#[test]
fn l1_eval_l0_interp_and_aot_agree_on_hof_via_defunctionalization() {
    let interp = Interpreter::new(
        PrimRegistry::with_builtins(),
        Box::new(BinaryTernarySwapEngine),
    );
    let prims = PrimRegistry::with_builtins();
    let engine = BinaryTernarySwapEngine;
    for (i, src) in hof_corpus().iter().enumerate() {
        let env = check_nodule(&parse(src).expect("parses")).expect("checks");
        // Monomorphize: resolves both generic type args AND defunctionalizes the fn-valued params
        // (RFC-0024 §4, M-687). The result is a closed, first-order, trait-free env L1-eval can run.
        let mono = monomorphize(&env, "main").unwrap_or_else(|e| {
            panic!("HOF program #{i}: must monomorphize + defunctionalize: {e}")
        });
        // Closure invariant (M-673 / RFC-0024 §4): no generics, no traits, no fn-typed params.
        assert!(
            mono.fns.values().all(|fd| fd.sig.params.is_empty())
                && mono.types.values().all(|d| d.params.is_empty())
                && mono.traits.is_empty()
                && mono.instances.is_empty()
                && mono.impls.is_empty(),
            "HOF program #{i}: monomorphized+defunctionalized env must be closed (no generics/traits)"
        );
        let registry = build_registry(&mono).expect("the mono'd data registry builds");

        // Path 1: the L1 fuel-guarded evaluator, on the MONOMORPHIZED+DEFUNCTIONALIZED env.
        // (L1-eval has no HOF dispatch — it can only run the defunctionalized, first-order version.)
        let l1 = Evaluator::new(&mono)
            .call("main", vec![])
            .unwrap_or_else(|e| panic!("HOF program #{i}: L1-eval(mono) failed: {e}"));
        let l1_core = l1.to_core(&mono, &registry).unwrap_or_else(|| {
            panic!("HOF program #{i}: L1 result is outside the r3 data fragment")
        });

        // Path 2: elaborate to L0 (elaborate calls monomorphize internally — on the source env),
        // run on the reference interpreter. Empirical: Err arms must pass through, Ok arms transform.
        let node = elaborate(&env, "main").unwrap_or_else(|e| {
            panic!("HOF program #{i}: must elaborate after defunctionalization: {e}")
        });
        let l0_core = interp
            .eval_core(&node)
            .unwrap_or_else(|e| panic!("HOF program #{i}: L0-interp failed: {e}"));

        // Path 3: the same L0 term through the AOT env-machine.
        let aot_core = mycelium_mlir::run_core(&node, &prims, &engine)
            .unwrap_or_else(|e| panic!("HOF program #{i}: AOT run_core failed: {e}"));

        // All three paths must agree — Empirical (differential over HOF corpus; VR-5).
        assert_eq!(
            l1_core, l0_core,
            "HOF program #{i} diverged: L1-eval(mono+defun) vs elaborate→L0-interp"
        );
        assert_eq!(
            l0_core, aot_core,
            "HOF program #{i} diverged: L0-interp vs AOT env-machine"
        );

        // The shared M-210 checker validates each agreeing pair (a mislabeled lowering is an
        // explicit NotValidated, never a silent pass — NFR-7/VR-4/G2).
        for (x, y, pair) in [
            (&l1_core, &l0_core, "L1↔interp"),
            (&l0_core, &aot_core, "interp↔AOT"),
        ] {
            assert_eq!(
                check_core(x, y),
                CheckVerdict::Validated {
                    strength: GuaranteeStrength::Exact
                },
                "HOF program #{i}: the shared checker must validate the {pair} pair"
            );
        }
    }
}

/// **Mutant-witness (M-688):** two different named functions passed to `map` produce different L0
/// results — confirming the defunctionalization discriminates. A vacuous differential that always
/// passes regardless of the fn argument would not be caught by the harness above; this closes
/// that gap. `not_val` and `id_val` yield different images on a non-all-ones input (Empirical).
#[test]
fn the_hof_differential_distinguishes_different_named_fn_arguments() {
    let run = |src: &str| {
        let env = check_nodule(&parse(src).unwrap()).unwrap();
        let node = elaborate(&env, "main").unwrap();
        Interpreter::new(
            PrimRegistry::with_builtins(),
            Box::new(BinaryTernarySwapEngine),
        )
        .eval_core(&node)
        .unwrap()
    };
    // Same map call, different fn argument: not_val vs id_val — must give different L0 results on
    // input 0b0000_0001 (not(0b0000_0001) = 0b1111_1110 ≠ 0b0000_0001 = id_val(0b0000_0001)).
    let with_not = run("nodule d\n\
         type Result<A, E> = Ok(A) | Err(E)\n\
         fn map<A, B, E>(r: Result<A, E>, f: A -> B) -> Result<B, E> =\n  \
           match r { Ok(x) => Ok(f(x)), Err(e) => Err(e) }\n\
         fn not_val(x: Binary{8}) -> Binary{8} = not(x)\n\
         fn mk_ok() -> Result<Binary{8},Binary{8}> = Ok(0b0000_0001)\n\
         fn main() -> Result<Binary{8},Binary{8}> = map(mk_ok(), not_val)");
    let with_id = run("nodule d\n\
         type Result<A, E> = Ok(A) | Err(E)\n\
         fn map<A, B, E>(r: Result<A, E>, f: A -> B) -> Result<B, E> =\n  \
           match r { Ok(x) => Ok(f(x)), Err(e) => Err(e) }\n\
         fn id_val(x: Binary{8}) -> Binary{8} = x\n\
         fn mk_ok() -> Result<Binary{8},Binary{8}> = Ok(0b0000_0001)\n\
         fn main() -> Result<Binary{8},Binary{8}> = map(mk_ok(), id_val)");
    assert_ne!(
        with_not, with_id,
        "map with not_val vs id_val must yield different L0 values (the HOF differential discriminates)"
    );
}
