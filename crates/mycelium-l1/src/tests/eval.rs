use crate::ast::Strength;
use crate::checkty::check_nodule;
use crate::checkty::Env;
use crate::eval::*;
use crate::parse;
use mycelium_core::Payload;
use mycelium_core::{GuaranteeStrength, Value};
use mycelium_interp::EvalError as KernelError;

fn env(src: &str) -> Env {
    check_nodule(&parse(src).expect("parses")).expect("checks")
}

fn run(src: &str) -> Result<L1Value, L1Error> {
    let env = env(src);
    Evaluator::new(&env).call("main", vec![])
}

#[test]
fn literals_lets_and_prims_evaluate() {
    let v = run("nodule d;\nfn main() => Binary{8} = let a = 0b1010_1010 in not(a);")
        .expect("evaluates");
    let L1Value::Repr(v) = v else { panic!("repr") };
    assert_eq!(
        v.payload(),
        &Payload::Bits(vec![false, true, false, true, false, true, false, true])
    );
    assert_eq!(v.meta().guarantee(), GuaranteeStrength::Exact);
}

#[test]
fn data_match_and_if_evaluate() {
    let v = run(
            "nodule d;\ntype Sign = Neg | Zero | Pos;\nfn label(s: Sign) => Ternary{1} =\n  match s { Neg => 0t-, Zero => 0t0, _ => 0t+ };\nfn main() => Ternary{1} = label(Zero);",
        )
        .expect("evaluates");
    let L1Value::Repr(v) = v else { panic!("repr") };
    assert_eq!(
        v.payload(),
        &Payload::Trits(vec![mycelium_core::Trit::Zero])
    );
}

// --- nested patterns (Maranget) ----------------------------------------------------------

const NAT: &str = "nodule d;\ntype Nat = Z | S(Nat);\n";

#[test]
fn nested_pattern_match_evaluates() {
    // pred2 uses depth-2 nested patterns (S(Z), S(S(m))) and is exhaustive (Z | S(Z) | S(S(_))).
    // Mutant-witness: a flat-only matcher could not bind `m` under two constructors; pred2 of
    // S(S(S(Z))) must peel two S's to yield S(Z).
    let src = format!(
        "{NAT}fn pred2(n: Nat) => Nat = match n {{ Z => Z, S(Z) => Z, S(S(m)) => m }};\n\
             fn main() => Nat = pred2(S(S(S(Z))));"
    );
    assert_eq!(
        run(&src).expect("evaluates"),
        L1Value::Data {
            ty: "Nat".into(),
            ctor: "S".into(),
            fields: vec![L1Value::Data {
                ty: "Nat".into(),
                ctor: "Z".into(),
                fields: vec![]
            }]
        }
    );
}

#[test]
fn nested_match_falls_through_to_the_right_arm() {
    // S(Z) selects the middle arm (not S(S(m))) — the nested matcher discriminates by depth.
    let src = format!(
        "{NAT}fn pred2(n: Nat) => Nat = match n {{ Z => Z, S(Z) => S(Z), S(S(m)) => m }};\n\
             fn main() => Nat = pred2(S(Z));"
    );
    assert_eq!(
        run(&src).expect("evaluates"),
        L1Value::Data {
            ty: "Nat".into(),
            ctor: "S".into(),
            fields: vec![L1Value::Data {
                ty: "Nat".into(),
                ctor: "Z".into(),
                fields: vec![]
            }]
        }
    );
}

// --- M-320: literal-pattern match over Binary/Ternary scrutinees -------------------------

const CLASSIFY: &str = "nodule d;\nfn classify(b: Binary{4}) => Ternary{1} = match b { 0b0000 => 0t0, 0b1111 => 0t+, _ => 0t- };\nfn main() => Ternary{1} = classify(0b1111);";

#[test]
fn literal_match_over_binary_selects_the_matching_arm() {
    // Mutant-witness: if eval_literal_match compared the wrong payload (or always took the
    // first arm), classify(0b1111) would not yield 0t+.
    let v = run(CLASSIFY).expect("evaluates");
    let L1Value::Repr(v) = v else { panic!("repr") };
    assert_eq!(v.payload(), &Payload::Trits(vec![mycelium_core::Trit::Pos]));
}

#[test]
fn literal_match_falls_through_to_the_default() {
    // Mutant-witness: if a non-matching literal arm fired anyway, classify(0b0101) would not
    // reach the `_` default 0t-.
    let src = CLASSIFY.replace("classify(0b1111)", "classify(0b0101)");
    let L1Value::Repr(v) = run(&src).expect("evaluates") else {
        panic!("repr")
    };
    assert_eq!(v.payload(), &Payload::Trits(vec![mycelium_core::Trit::Neg]));
}

#[test]
fn literal_match_without_a_default_is_non_exhaustive() {
    // Mutant-witness: dropping the mandatory-default check would let a literal match silently
    // assume coverage of the 2^4 domain (W7 violation).
    let src = "nodule d;\nfn classify(b: Binary{4}) => Ternary{1} = match b { 0b0000 => 0t0, 0b1111 => 0t+ };\nfn main() => Ternary{1} = classify(0b1111);";
    let err = check_nodule(&parse(src).expect("parses")).expect_err("must reject");
    assert!(
        err.message.contains("non-exhaustive"),
        "got: {}",
        err.message
    );
}

#[test]
fn duplicate_literal_pattern_is_rejected() {
    // Mutant-witness: a duplicate literal arm is a redundant (unreachable) arm — the Maranget
    // usefulness check must reject it, never silently accept it (W7). `0b0000` and `0b00_00` are
    // the same literal (the `_` separator is canonicalized away), so the second is unreachable.
    let src = "nodule d;\nfn classify(b: Binary{4}) => Ternary{1} = match b { 0b0000 => 0t0, 0b00_00 => 0t+, _ => 0t- };\nfn main() => Ternary{1} = classify(0b0000);";
    let err = check_nodule(&parse(src).expect("parses")).expect_err("must reject");
    assert!(err.message.contains("unreachable"), "got: {}", err.message);
}

#[test]
fn literal_pattern_width_must_match_the_scrutinee() {
    // Mutant-witness: dropping the width check would let a 2-bit literal match a Binary{4}
    // scrutinee — a payload-length mismatch that could never fire (or panic downstream).
    let src = "nodule d;\nfn classify(b: Binary{4}) => Ternary{1} = match b { 0b00 => 0t0, _ => 0t- };\nfn main() => Ternary{1} = classify(0b0000);";
    let err = check_nodule(&parse(src).expect("parses")).expect_err("must reject");
    assert!(
        err.message.contains("literal pattern has type"),
        "got: {}",
        err.message
    );
}

#[test]
fn structural_recursion_terminates_within_fuel() {
    // `drop_` is classified Total (structural descent) — and indeed terminates.
    let v = run(
            "nodule d;\ntype Nat = Z | S(Nat);\nfn drop_(n: Nat) => Nat = match n { Z => Z, S(m) => drop_(m) };\nfn main() => Nat = drop_(S(S(Z)));",
        )
        .expect("terminates");
    assert_eq!(
        v,
        L1Value::Data {
            ty: "Nat".into(),
            ctor: "Z".into(),
            fields: vec![]
        }
    );
}

#[test]
fn an_unproductive_recursion_is_an_explicit_fuel_exhaustion() {
    // With the clock tighter than the depth guard, the *semantic* budget trips first.
    let env = env(
            "nodule d;\ntype Nat = Z | S(Nat);\nfn spin(n: Nat) => Nat = spin(n);\nfn main() => Nat = spin(Z);",
        );
    let err = Evaluator::new(&env)
        .with_fuel(50)
        .call("main", vec![])
        .unwrap_err();
    assert_eq!(err, L1Error::FuelExhausted);
}

#[test]
fn deep_recursion_trips_the_host_stack_guard_explicitly() {
    // With ample fuel, the depth guard refuses explicitly — never a host stack overflow.
    let env = env(
            "nodule d;\ntype Nat = Z | S(Nat);\nfn spin(n: Nat) => Nat = spin(n);\nfn main() => Nat = spin(Z);",
        );
    let err = Evaluator::new(&env).call("main", vec![]).unwrap_err();
    assert!(
        matches!(err, L1Error::DepthExceeded { .. }),
        "expected DepthExceeded, got {err:?}"
    );
}

#[test]
fn deeply_nested_expression_trips_the_depth_guard_without_any_recursive_call() {
    // A4-03 mutant-witness: depth is charged per AST node, so a *wide-but-shallow* program —
    // here a deep but call-free `not(not(… not(0b…) …))` nest, which makes no recursive
    // function call at all — still hits the host-stack guard explicitly once its nesting
    // exceeds DEFAULT_DEPTH (64). This pins the documented per-node (not per-call-frame)
    // contract: a refactor charging depth only at `invoke` would let this nest recurse on the
    // host stack unguarded, flipping this assertion (the depth guard would never trip) and so
    // turning the test red — the regression we want to catch.
    let mut expr = "0b0000_0001".to_owned();
    for _ in 0..200 {
        expr = format!("not({expr})");
    }
    let deep_env = env(&format!("nodule d;\nfn main() => Binary{{8}} = {expr};"));
    let err = Evaluator::new(&deep_env).call("main", vec![]).unwrap_err();
    assert!(
        matches!(
            err,
            L1Error::DepthExceeded {
                limit: DEFAULT_DEPTH
            }
        ),
        "expected DepthExceeded(limit=64) from a call-free 200-deep nest, got {err:?}"
    );

    // And the same nest within the depth budget evaluates fine (200 nested `not`s exceed 64;
    // a small nest does not) — confirming the guard refuses *only* genuine over-nesting.
    let shallow = env("nodule d;\nfn main() => Binary{8} = not(not(0b0000_0001));");
    Evaluator::new(&shallow)
        .call("main", vec![])
        .expect("a shallow nest is well within the depth budget");
}

#[test]
fn a_for_fold_evaluates_head_to_tail() {
    // checksum(More(0b1111_0000, More(0b0000_1111, End))) = 0b1111_1111 (xor-fold).
    let v = run(
            "nodule d;\ntype ByteList = End | More(Binary{8}, ByteList);\nfn checksum(bs: ByteList) => Binary{8} =\n    for b in bs, acc = 0b0000_0000 => xor(acc, b);\nfn main() => Binary{8} = checksum(More(0b1111_0000, More(0b0000_1111, End)));",
        )
        .expect("evaluates");
    let L1Value::Repr(v) = v else { panic!("repr") };
    assert_eq!(v.payload(), &Payload::Bits(vec![true; 8]));
}

#[test]
fn a_for_fold_over_nil_is_the_initial_accumulator() {
    let v = run(
            "nodule d;\ntype ByteList = End | More(Binary{8}, ByteList);\nfn checksum(bs: ByteList) => Binary{8} =\n    for b in bs, acc = 0b1010_1010 => xor(acc, b);\nfn main() => Binary{8} = checksum(End);",
        )
        .expect("evaluates");
    let L1Value::Repr(v) = v else { panic!("repr") };
    assert_eq!(
        v.payload(),
        &Payload::Bits(vec![true, false, true, false, true, false, true, false])
    );
}

#[test]
fn a_long_for_fold_costs_fuel_not_host_stack() {
    // 200 elements would blow the depth guard (64) as hand-written recursion; the `for`
    // spine walk is iterative and must not (RFC-0007 §4.8). The list value is built
    // programmatically — a 200-deep nested *expression* would itself be depth-guarded.
    let env = env(
            "nodule d;\ntype ByteList = End | More(Binary{8}, ByteList);\nfn checksum(bs: ByteList) => Binary{8} =\n    for b in bs, acc = 0b0000_0000 => xor(acc, b);",
        );
    let byte = || {
        L1Value::Repr(
            Value::new(
                mycelium_core::Repr::Binary { width: 8 },
                Payload::Bits(vec![false, false, false, false, false, false, false, true]),
                mycelium_core::Meta::exact(mycelium_core::Provenance::Root),
            )
            .unwrap(),
        )
    };
    let mut list = L1Value::Data {
        ty: "ByteList".into(),
        ctor: "End".into(),
        fields: vec![],
    };
    for _ in 0..200 {
        list = L1Value::Data {
            ty: "ByteList".into(),
            ctor: "More".into(),
            fields: vec![byte(), list],
        };
    }
    let v = Evaluator::new(&env)
        .call("checksum", vec![list])
        .expect("evaluates");
    let L1Value::Repr(v) = v else { panic!("repr") };
    // 200 xors of 0b0000_0001 → even count → all zeros.
    assert_eq!(v.payload(), &Payload::Bits(vec![false; 8]));
}

#[test]
fn the_certified_swap_runs_and_a_weakening_assertion_passes() {
    // The in-range binary→ternary swap is Exact; asserting `@ Proven` weakens — allowed.
    let v = run(
            "nodule d;\nfn main() => Ternary{6} @ Proven = swap(0b0000_0010, to: Ternary{6}, policy: rt);",
        )
        .expect("evaluates");
    let L1Value::Repr(v) = v else { panic!("repr") };
    assert_eq!(v.repr(), &mycelium_core::Repr::Ternary { trits: 6 });
}

#[test]
fn asserting_stronger_than_actual_is_an_explicit_error() {
    // A Declared-bound value asserted `@ Exact` must refuse (VR-5: never upgrade).
    let declared = Value::new(
        mycelium_core::Repr::Binary { width: 2 },
        Payload::Bits(vec![true, false]),
        mycelium_core::Meta::new(
            mycelium_core::Provenance::Root,
            GuaranteeStrength::Declared,
            Some(mycelium_core::Bound {
                kind: mycelium_core::BoundKind::Error {
                    eps: 0.1,
                    norm: mycelium_core::NormKind::Linf,
                },
                basis: mycelium_core::BoundBasis::UserDeclared,
            }),
            None,
            None,
            None,
        )
        .expect("well-formed meta"),
    )
    .expect("well-formed value");
    let env = env("nodule d;\nfn main() => Binary{8} = 0b0000_0000;");
    let ev = Evaluator::new(&env);
    let err = ev
        .assert_guarantee("t", &L1Value::Repr(declared), Strength::Exact)
        .unwrap_err();
    assert!(matches!(err, L1Error::GuaranteeTooWeak { .. }), "{err:?}");
}

#[test]
fn an_ungranted_wild_host_op_is_an_explicit_refusal() {
    // M-721: a `wild` host op now *dispatches* (M-720 lowering), but the default registry grants
    // no `wild:` op (RFC-0028 §4.3 — the capability handle), so running `wild { foreign(…) }`
    // without the host capability is an explicit `Kernel(UnknownPrim)` refusal — never silent
    // (G2). Drive the evaluator directly on an unchecked nodule (the checker would also gate the
    // `@std-sys` context) to confirm the refusal is the evaluator's own.
    let nodule =
        parse("nodule d;\nfn main() => Binary{8} = wild { foreign(0b0000_0001) };").unwrap();
    let env = Env {
        types: std::collections::BTreeMap::new(),
        fns: nodule
            .items
            .iter()
            .filter_map(|i| match i {
                crate::ast::Item::Fn(f) => Some((f.sig.name.clone(), f.clone())),
                _ => None,
            })
            .collect(),
        totality: std::collections::BTreeMap::new(),
        traits: std::collections::BTreeMap::new(),
        instances: std::collections::BTreeMap::new(),
        impls: std::collections::BTreeMap::new(),
        lower_rules: std::collections::BTreeMap::new(),
        derived_provenance: std::collections::BTreeMap::new(),
    };
    let err = Evaluator::new(&env).call("main", vec![]).unwrap_err();
    assert!(
        matches!(&err, L1Error::Kernel(KernelError::UnknownPrim(p)) if p == "wild:foreign"),
        "an ungranted wild host op must be an explicit UnknownPrim refusal; got: {err:?}"
    );
}

// --- M-642 additive ergonomics: EvaluatorOpts / with_opts -----------------------------------

#[test]
fn evaluator_opts_default_matches_new_budgets() {
    // `with_opts(default)` is a no-op: same observable result as plain `new` on a program that
    // runs well inside both budgets.
    let e = env("nodule d;\nfn main() => Binary{8} = not(0b0000_0000);");
    let baseline = Evaluator::new(&e).call("main", vec![]).expect("evaluates");
    let via_opts = Evaluator::new(&e)
        .with_opts(EvaluatorOpts::default())
        .call("main", vec![])
        .expect("evaluates");
    assert_eq!(baseline, via_opts);
}

#[test]
fn evaluator_opts_apply_the_fuel_budget() {
    // A starvation-level fuel budget supplied via `with_opts` must take effect — proving the
    // opts struct is actually applied (each node costs one unit; 1 unit cannot finish `not(_)`).
    let e = env("nodule d;\nfn main() => Binary{8} = not(0b0000_0000);");
    let err = Evaluator::new(&e)
        .with_opts(EvaluatorOpts::default().fuel(1))
        .call("main", vec![])
        .unwrap_err();
    assert!(matches!(err, L1Error::FuelExhausted), "{err:?}");
}

#[test]
fn evaluator_opts_builder_sets_both_fields() {
    let o = EvaluatorOpts::default().fuel(42).depth(7);
    assert_eq!(o.fuel, 42);
    assert_eq!(o.depth, 7);
    // `with_opts` is exactly the `with_fuel`+`with_depth` chain (same observable behavior under a
    // generous budget — both evaluate the program), checked here via the no-op-on-success path.
    let e = env("nodule d;\nfn main() => Binary{8} = not(0b1111_0000);");
    let chained = Evaluator::new(&e)
        .with_fuel(1_000)
        .with_depth(64)
        .call("main", vec![])
        .expect("evaluates");
    let opted = Evaluator::new(&e)
        .with_opts(EvaluatorOpts::default().fuel(1_000).depth(64))
        .call("main", vec![])
        .expect("evaluates");
    assert_eq!(chained, opted);
}

// --- M-674: deep-stack evaluation — explicit budget, never a host-stack overflow ----------

/// A genuinely deep evaluation with a raised depth budget completes on the worker stack and
/// the budget trips cleanly when exceeded — never a host-stack crash.
///
/// **Design (banked guard 4 + deep worker stack).**
/// `Evaluator::call` runs the recursive `eval` pass on a 256 MiB lazily-committed worker
/// thread (via `mycelium_stack::with_deep_stack`). The explicit `with_depth(N)` budget is
/// therefore always the bound — not the caller's stack — so raising it for deep programs is
/// safe. The two assertions here pin both sides of that contract in-process (a host-stack
/// overflow aborts the process; a clean `DepthExceeded` is a normal `Err` — so the
/// "budget trips cleanly" test *would crash the process* if the host stack overflowed first).
///
/// **Physical ceiling estimate (Empirical — varies with optimizer and IR shape).**
/// The evaluator's `eval` frame carries roughly a pointer, two integers, and a few
/// references (~2–4 KiB in a debug build). At ~4 KiB/frame the 256 MiB worker stack
/// supports ~65,000 levels; at ~2 KiB/frame ~130,000. The default budget (64) is a
/// ~1,000× safety margin. The test raises the budget to 4,096 — matching the checker's
/// `MAX_CHECK_DEPTH` — and confirms both directions; 4,096 is well inside the physical
/// ceiling at any plausible debug frame size.
#[test]
fn raised_depth_budget_completes_on_deep_worker_stack_and_trips_cleanly_past_it() {
    // Use mutual recursion (`spin`/`ping`) to avoid the totality checker's structural-descent
    // optimisation and produce genuine host-call-stack depth. The program is non-terminating
    // so we must cap fuel tightly to avoid the fuel budget being the first thing to trip.
    // `spin` calls `ping` calls `spin` calls … — each call is two `invoke` frames and several
    // `eval` frames (the function body, the argument expression, etc.). At depth budget 4,096
    // that's many hundreds of recursive calls — way past a normal 2 MiB thread stack but
    // within the 256 MiB worker stack.
    let src = "nodule d;\ntype Nat = Z | S(Nat);\nfn spin(n: Nat) => Nat = ping(n);\nfn ping(n: Nat) => Nat = spin(n);\nfn main() => Nat = spin(Z);";
    let deep_env = env(src);

    // Part A: within the raised budget — the fuel budget trips before the depth budget
    // (proving the computation ran deep on the worker stack without overflowing the host).
    // We use very large fuel here so it runs long enough to hit the depth budget first;
    // actually we expect depth to trip first because mutual recursion deeply nests `eval`.
    // Set fuel generously so the *depth* budget (4096) is what limits, not fuel.
    let result = Evaluator::new(&deep_env)
        .with_depth(4_096)
        .with_fuel(10_000_000)
        .call("main", vec![]);
    // Either DepthExceeded(4096) or FuelExhausted — both are clean explicit errors, never a
    // crash. The important property: the process does not abort (no host-stack overflow).
    match &result {
        Err(L1Error::DepthExceeded { limit: 4_096 }) | Err(L1Error::FuelExhausted) => {}
        other => panic!(
            "expected DepthExceeded(4096) or FuelExhausted on a deep mutual recursion with \
                 depth=4096, got {other:?}"
        ),
    }

    // Part B: past the budget — must always be a clean DepthExceeded, never a crash.
    // A budget of 8 on a mutual-recursive program is sure to trip the depth guard quickly.
    let err = Evaluator::new(&deep_env)
        .with_depth(8)
        .with_fuel(10_000_000)
        .call("main", vec![])
        .unwrap_err();
    assert!(
        matches!(err, L1Error::DepthExceeded { limit: 8 }),
        "expected DepthExceeded(limit=8) for a tiny depth budget on mutual recursion, \
             got {err:?}"
    );

    // Part C: the same `spin`/`ping` program with ample budgets and a small input terminates
    // via `DepthExceeded` — proving the evaluator does not hang on a valid raised budget, and
    // that completing N levels of recursion before the depth budget trips is observable.
    // (We pick depth=4 so it trips almost immediately, confirming the budget is functional
    // even when the worker stack has headroom to spare.)
    let err_small = Evaluator::new(&deep_env)
        .with_depth(4)
        .with_fuel(10_000_000)
        .call("main", vec![])
        .unwrap_err();
    assert!(
        matches!(err_small, L1Error::DepthExceeded { limit: 4 }),
        "expected DepthExceeded(limit=4), got {err_small:?}"
    );
}

// --- M-677 / RFC-0014 §4.5 I4: per-effect budget ledger wiring --------------------------------

// A recursive counter-down fn declaring `!{retry(<=3)}`. Each call to `count_down` consumes one
// unit from the shared budget ledger for the top-level `call` invocation.
//
// `count_down(S(Z))` recurses once (S(Z) → Z) = 2 total invocations → 2 consumed of 3 → ok.
// `count_down(S(S(Z)))` = 3 invocations → 3 consumed of 3 → exactly at budget → ok.
// `count_down(S(S(S(Z))))` = 4 invocations → 4th consume finds 0 remaining → EffectBudgetExhausted.
//
// `main_*` calls count_down, so it must declare `retry` too (effect coverage M-660); its own
// invoke does NOT prime a budget (no bound in `main_*`'s effect_budgets), so it only charges via
// count_down's invocations. (Guarantee: `Empirical` — v0 per-call model, RFC-0014 §9.)
const BUDGET_SRC: &str = "nodule d;\n\
    type Nat = Z | S(Nat);\n\
    fn count_down(n: Nat) => Binary{1} !{retry(<=3)} = \
      match n { Z => 0b1, S(m) => count_down(m) };\n\
    fn main_under() => Binary{1} !{retry} = count_down(S(Z));\n\
    fn main_at() => Binary{1} !{retry} = count_down(S(S(Z)));\n\
    fn main_over() => Binary{1} !{retry} = count_down(S(S(S(Z))));";

#[test]
fn budgeted_fn_under_budget_returns_value() {
    // `count_down(S(Z))` — 2 invocations, ceiling 3 → 2 consumed → success.
    // Guarantee: `Empirical` — per-call consumption, v0 approximation (RFC-0014 §9).
    let e = check_nodule(&parse(BUDGET_SRC).expect("parses")).expect("checks");
    let v = Evaluator::new(&e)
        .call("main_under", vec![])
        .expect("under budget — must succeed");
    let L1Value::Repr(r) = v else {
        panic!("expected repr, got {v:?}")
    };
    assert_eq!(r.payload(), &Payload::Bits(vec![true]));
}

#[test]
fn budgeted_fn_at_budget_returns_value() {
    // `count_down(S(S(Z)))` — 3 invocations, ceiling 3 → budget reaches 0 after last consume,
    // but the last is the base case which returns immediately before a 4th invoke → success.
    let e = check_nodule(&parse(BUDGET_SRC).expect("parses")).expect("checks");
    let v = Evaluator::new(&e)
        .call("main_at", vec![])
        .expect("at budget — must succeed");
    let L1Value::Repr(r) = v else {
        panic!("expected repr, got {v:?}")
    };
    assert_eq!(r.payload(), &Payload::Bits(vec![true]));
}

#[test]
fn budgeted_fn_over_budget_returns_effect_budget_exhausted() {
    // `count_down(S(S(S(Z))))` — 4 invocations, ceiling 3 → 4th consume finds 0 remaining →
    // explicit `L1Error::EffectBudget` (RFC-0014 §4.5 I4). Graceful, never a hang or OOM (G2).
    let e = check_nodule(&parse(BUDGET_SRC).expect("parses")).expect("checks");
    let err = Evaluator::new(&e)
        .call("main_over", vec![])
        .expect_err("over budget — must return EffectBudgetExhausted");
    assert!(
        matches!(err, L1Error::EffectBudget(_)),
        "expected L1Error::EffectBudget, got {err:?}"
    );
}
