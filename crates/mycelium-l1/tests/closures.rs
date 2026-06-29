//! **M-704 / RFC-0024 §4A** — the three-way differential for **closures** (environment-capturing
//! lambdas, partial-flow closures, dynamic fn-flow) lowered by **Reynolds defunctionalization**.
//!
//! A closure lowers (in `mono.rs`) to a **tag-sum data value** (its captured environment) + a
//! generated **`apply` dispatcher** (an ordinary fn whose body is a `match`) — all over existing L0
//! constructs, so **no `mycelium-core` node is added** (KC-3). The acceptance bar is the same as the
//! §4 landed named-fn case (NFR-7), now **per closure shape**: each fixture must evaluate
//! **identically across the three paths** — L1-eval ≡ elaborate→L0-interp ≡ AOT — on the
//! **monomorphized + defunctionalized** program. This is `Empirical` (trials), never `Proven` (VR-5).
//!
//! Shapes covered (RFC-0024 §4A.9): captureless lambda, single-capture, multi-capture,
//! closure-capturing-closure, dynamic-fn-out-of-match, dynamic-fn-as-field, and a **capturing stdlib
//! combinator** (`map` with a closure) as the consuming proof.

use mycelium_cert::{check_core, BinaryTernarySwapEngine, CheckVerdict};
use mycelium_core::GuaranteeStrength;
use mycelium_interp::{Interpreter, PrimRegistry};
use mycelium_l1::elab::build_registry;
use mycelium_l1::{check_nodule, elaborate, monomorphize, parse, Evaluator};

/// One closure fixture: a name + a self-contained nodule source with a nullary `main`.
struct Shape {
    name: &'static str,
    src: &'static str,
}

/// The closure corpus — one entry per RFC-0024 §4A.9 closure shape. Each `main` is closed and
/// nullary; the expected value is asserted only relative to the three agreeing paths (the
/// differential), with a separate mutant-witness test pinning that the dispatch is not vacuous.
fn closure_corpus() -> Vec<Shape> {
    vec![
        // (1) captureless lambda, applied through a `let` binder: not(0b0000_0001) = 0b1111_1110.
        Shape {
            name: "captureless",
            src: "nodule d;\nfn main() => Binary{8} =\n  let f = lambda(x: Binary{8}) => not(x) in f(0b0000_0001);",
        },
        // (2) single-capture: captures `c`; and(0b1010_1010, 0b0000_1111) = 0b0000_1010.
        Shape {
            name: "single-capture",
            src: "nodule d;\nfn main() => Binary{8} =\n  let c = 0b0000_1111 in\n  let f = lambda(x: Binary{8}) => and(x, c) in f(0b1010_1010);",
        },
        // (3) multi-capture: captures `a` and `b`; and(and(0xFF, a), b).
        Shape {
            name: "multi-capture",
            src: "nodule d;\nfn main() => Binary{8} =\n  let a = 0b0000_1111 in let b = 0b1100_1100 in\n  let f = lambda(x: Binary{8}) => and(and(x, a), b) in f(0b1111_1111);",
        },
        // (4) closure-capturing-closure: `h` captures the closure `inc` and applies it via `apply2`.
        Shape {
            name: "closure-capturing-closure",
            src: "nodule d;\nfn apply2(g: Binary{8} => Binary{8}, y: Binary{8}) => Binary{8} = g(y);\nfn main() => Binary{8} =\n  let inc = lambda(x: Binary{8}) => not(x) in\n  let h = lambda(z: Binary{8}) => apply2(inc, z) in h(0b0000_0001);",
        },
        // (5) dynamic-fn-out-of-match: the closure is chosen in a `match`, then applied.
        Shape {
            name: "dyn-fn-out-of-match",
            src: "nodule d;\ntype Bit = Hi | Lo;\nfn main() => Binary{8} =\n  let sel = Hi in\n  let f = match sel {\n    Hi => lambda(x: Binary{8}) => not(x),\n    Lo => lambda(x: Binary{8}) => x\n  } in f(0b0000_0001);",
        },
        // (6) dynamic-fn-as-field: a closure stored in a data field, applied after destructuring.
        Shape {
            name: "dyn-fn-as-field",
            src: "nodule d;\ntype Box = Mk(Binary{8} => Binary{8});\nfn run(b: Box, v: Binary{8}) => Binary{8} = match b { Mk(f) => f(v) };\nfn main() => Binary{8} =\n  let c = 0b0000_1111 in\n  run(Mk(lambda(x: Binary{8}) => and(x, c)), 0b1010_1010);",
        },
        // (7) capturing stdlib combinator: `map` over Result with a CLOSURE (the consuming proof).
        Shape {
            name: "map-with-closure",
            src: "nodule d;\ntype Result[A, E] = Ok(A) | Err(E);\nfn map[A, B, E](r: Result[A, E], f: A => B) => Result[B, E] =\n  match r { Ok(x) => Ok(f(x)), Err(e) => Err(e) };\nfn mk_ok() => Result[Binary{8},Binary{8}] = Ok(0b0000_0001);\nfn main() => Result[Binary{8},Binary{8}] =\n  let c = 0b0000_1111 in map(mk_ok(), lambda(x: Binary{8}) => and(x, c));",
        },
        // (8) named fn as an escaping value (RFC-0024 §4A.4 — a bare named fn becomes a NULLARY
        // closure constructor): `let f = negate in f(x)`. not(0b0000_0011) = 0b1111_1100.
        Shape {
            name: "named-fn-as-value",
            src: "nodule d;\nfn negate(x: Binary{8}) => Binary{8} = not(x);\nfn main() => Binary{8} = let f = negate in f(0b0000_0011);",
        },
    ]
}

/// **M-704 (RFC-0024 §4A.9):** every closure shape evaluates identically across the three paths —
/// L1-eval(mono) ≡ elaborate→L0-interp ≡ AOT — on the **monomorphized + defunctionalized** env.
/// This is the end-to-end proof that closure lowering (Reynolds defunctionalization) produces closed
/// first-order L0 that agrees on all three evaluation paths. `Empirical` (trials; VR-5).
#[test]
fn l1_eval_l0_interp_and_aot_agree_on_closures_via_defunctionalization() {
    let interp = Interpreter::new(
        PrimRegistry::with_builtins(),
        Box::new(BinaryTernarySwapEngine),
    );
    let prims = PrimRegistry::with_builtins();
    let engine = BinaryTernarySwapEngine;
    for shape in closure_corpus() {
        let name = shape.name;
        let env =
            check_nodule(&parse(shape.src).unwrap_or_else(|e| panic!("[{name}] parses: {e}")))
                .unwrap_or_else(|e| panic!("[{name}] checks: {e}"));
        // Monomorphize: resolves generics AND lowers closures (tag-sum + apply dispatcher).
        let mono = monomorphize(&env, "main").unwrap_or_else(|e| {
            panic!("[{name}] must monomorphize + defunctionalize closures: {e}")
        });
        // Closed invariant (M-673 / RFC-0024 §4A): no generics, no traits, no fn-typed params remain
        // (every arrow lowered to a `Fn$A$B` data type).
        assert!(
            mono.fns.values().all(|fd| fd.sig.params.is_empty())
                && mono.types.values().all(|d| d.params.is_empty())
                && mono.traits.is_empty()
                && mono.instances.is_empty()
                && mono.impls.is_empty(),
            "[{name}]: monomorphized+defunctionalized env must be closed"
        );
        let registry = build_registry(&mono).expect("the mono'd data registry builds");

        // Path 1: the L1 fuel-guarded evaluator on the MONOMORPHIZED+DEFUNCTIONALIZED env.
        let l1 = Evaluator::new(&mono)
            .call("main", vec![])
            .unwrap_or_else(|e| panic!("[{name}] L1-eval(mono) failed: {e}"));
        let l1_core = l1
            .to_core(&mono, &registry)
            .unwrap_or_else(|| panic!("[{name}] L1 result is outside the r3 data fragment"));

        // Path 2: elaborate to L0 (elaborate monomorphizes internally on the source env), run on the
        // reference interpreter.
        let node = elaborate(&env, "main")
            .unwrap_or_else(|e| panic!("[{name}] must elaborate closures: {e}"));
        let l0_core = interp
            .eval_core(&node)
            .unwrap_or_else(|e| panic!("[{name}] L0-interp failed: {e}"));

        // Path 3: the same L0 term through the AOT env-machine.
        let aot_core = mycelium_mlir::run_core(&node, &prims, &engine)
            .unwrap_or_else(|e| panic!("[{name}] AOT run_core failed: {e}"));

        // All three paths must agree — Empirical (differential per closure shape; VR-5).
        assert_eq!(
            l1_core, l0_core,
            "[{name}] diverged: L1-eval(mono+defun) vs elaborate→L0-interp"
        );
        assert_eq!(
            l0_core, aot_core,
            "[{name}] diverged: L0-interp vs AOT env-machine"
        );

        // The shared M-210 checker validates each agreeing pair (a mislabeled lowering is an explicit
        // NotValidated, never a silent pass — NFR-7/VR-4/G2).
        for (x, y, pair) in [
            (&l1_core, &l0_core, "L1↔interp"),
            (&l0_core, &aot_core, "interp↔AOT"),
        ] {
            assert_eq!(
                check_core(x, y),
                CheckVerdict::Validated {
                    strength: GuaranteeStrength::Exact
                },
                "[{name}]: the shared checker must validate the {pair} pair"
            );
        }
    }
}

/// **Mutant-witness (M-704):** two **different captured environments** in the *same* lambda shape
/// produce different L0 results — confirming the closure dispatch reads the capture, not a constant.
/// A vacuous lowering that ignored the capture would pass the corpus above; this closes that gap.
/// `and(0b1111_1111, 0b0000_1111) = 0b0000_1111` ≠ `and(0b1111_1111, 0b1111_0000) = 0b1111_0000`.
#[test]
fn the_closure_differential_distinguishes_different_captured_environments() {
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
    let with_low = run("nodule d;\nfn main() => Binary{8} =\n  let c = 0b0000_1111 in\n  let f = lambda(x: Binary{8}) => and(x, c) in f(0b1111_1111);");
    let with_high = run("nodule d;\nfn main() => Binary{8} =\n  let c = 0b1111_0000 in\n  let f = lambda(x: Binary{8}) => and(x, c) in f(0b1111_1111);");
    assert_ne!(
        with_low, with_high,
        "different captured environments must yield different results — the dispatch reads the capture"
    );
}

/// **Multi-argument lambda is a never-silent refusal (RFC-0024 §4A.8 — tuple-gated).** A two-parameter
/// `lambda` needs the tuple-type prerequisite the v0 surface lacks; the checker refuses it explicitly
/// (G2/VR-5), never a silent accept. This pins the honest scope boundary (FLAG: multi-arg/partial).
#[test]
fn multi_argument_lambda_is_an_explicit_refusal() {
    let src = "nodule d;\nfn main() => Binary{8} =\n  let f = lambda(x: Binary{8}, y: Binary{8}) => and(x, y) in f(0b1111_1111);";
    let r = check_nodule(&parse(src).expect("parses — the grammar admits a 2-param lambda"));
    assert!(
        r.is_err(),
        "a multi-argument lambda must be refused (tuple-gated — RFC-0024 §4A.8), not silently accepted"
    );
}

/// **Regression (M-704 / mono.rs `rewrite_lambda` capture filter).** A statically-specialized HOF
/// value-parameter (baked into `fn_param_subst` and *dropped* from the emitted signature, yet still
/// present in `scope` for inference) must **not** be added to an inner lambda's capture list: it is a
/// compile-time-baked constant, not a runtime capture. Trigger: a static named fn (`negate`) passed
/// to a HOF (`apply_wrap`) whose body contains an inner lambda that captures the HOF's fn-param (`f`).
/// Before the fix, `f` was spuriously captured → the closure ctor `Clo$...(f)` referenced a param with
/// no runtime value (elaboration error, or a silent wrong-entity if a ctor/fn named `f` existed — G2).
/// We pin the concrete three-way value: `apply_wrap(negate, 0b0000_0001) == not(0b0000_0001) =
/// 0b1111_1110`. All three paths must agree (Empirical — VR-5).
#[test]
fn a_static_fn_param_baked_by_specialization_is_not_captured_by_an_inner_lambda() {
    let interp = Interpreter::new(
        PrimRegistry::with_builtins(),
        Box::new(BinaryTernarySwapEngine),
    );
    let prims = PrimRegistry::with_builtins();
    let engine = BinaryTernarySwapEngine;
    let src = "nodule d;\n\
        fn apply_wrap(f: Binary{8} => Binary{8}, x: Binary{8}) => Binary{8} =\n\
          let g = lambda(y: Binary{8}) => f(y) in g(x);\n\
        fn negate(x: Binary{8}) => Binary{8} = not(x);\n\
        fn main() => Binary{8} = apply_wrap(negate, 0b0000_0001);";
    let env = check_nodule(&parse(src).expect("parses")).expect("checks");
    let mono = monomorphize(&env, "main").expect("monomorphizes + defunctionalizes");
    let registry = build_registry(&mono).expect("the mono'd data registry builds");

    // Path 1: L1 evaluator on the monomorphized + defunctionalized env.
    let l1 = Evaluator::new(&mono)
        .call("main", vec![])
        .expect("L1-eval(mono) — a baked static fn-param must not become a spurious capture");
    let l1_core = l1
        .to_core(&mono, &registry)
        .expect("L1 result is in the r3 data fragment");

    // Path 2: elaborate → L0 reference interpreter.
    let node = elaborate(&env, "main").expect("elaborates");
    let l0_core = interp.eval_core(&node).expect("L0-interp");

    // Path 3: the same L0 term through the AOT env-machine.
    let aot_core = mycelium_mlir::run_core(&node, &prims, &engine).expect("AOT run_core");

    // Pin the concrete value and the three-way agreement: not(0b0000_0001) = 0b1111_1110.
    assert_eq!(
        l1_core, l0_core,
        "L1-eval(mono) vs elaborate→L0-interp diverged"
    );
    assert_eq!(l0_core, aot_core, "L0-interp vs AOT env-machine diverged");
    assert_eq!(
        check_core(&l1_core, &l0_core),
        CheckVerdict::Validated {
            strength: GuaranteeStrength::Exact
        },
        "the shared checker validates the L1↔interp pair"
    );
}
