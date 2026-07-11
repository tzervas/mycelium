//! **M-1054 Stage 1b — end-to-end reachability** (DN-110 §5-A; DN-114). Companion to
//! `src/tests/facility_stage1_hygiene.rs` (which calls [`elaborate_lower_rule_with_args`] —
//! Stage 1's L0 elab-phase mechanism — **directly**, exercising (A)+(B) hygiene on the production
//! expansion path but never through the ordinary program-elaboration pipeline) and
//! `src/tests/checkty.rs`'s `stage1b_sugar_call_recognized_and_accepted` (which exercises
//! `Cx::check_sugar_call`'s accept path in isolation, via `infer_type` on a bare call expression,
//! never through elaboration). **This module is the first to exercise the two halves together**:
//! a value-parametric sugar call reached from an ordinary caller (`fn main`), checked by the real
//! `Cx::check`/`infer_type` (Stage 1b's ACCEPT path), then elaborated by the real, ordinary
//! [`crate::elab::elaborate`] entry point — which now dispatches to the sugar rule through
//! [`crate::elab::Elab::app`]'s new §5.2 branch, **not** a direct call to the expander.
//!
//! # Why the "colliding" argument is elicited, not hand-picked (THE NON-VACUITY LAW)
//! Exactly the hazard `facility_stage1_hygiene.rs`'s own module doc names: the real elaborator's
//! Pass 1 already assigns every `let`/`lambda` binder a fresh, `%`-namespaced kernel name via
//! [`crate::elab::Elab::fresh`] (unconditionally — the same machinery every elaboration path uses),
//! so a *literal* surface-spelling collision can no longer arise after Pass 1. The real residual
//! hazard is **cross-invocation** collision (OQ-H5): `Elab::fresh`'s counter resets to `0` for
//! every *independent* top-level elaboration, so two unrelated elaborations — here, `main`'s own
//! (via [`crate::elab::elaborate`]) and `Swap2`'s RHS expansion (via
//! [`elaborate_lower_rule_with_args`], invoked mid-way through `main`'s elaboration by
//! `Elab::app`'s new dispatch) — can each independently mint the identical kernel name (`"t%0"`)
//! for their own, unrelated binders. So the argument passed for `Swap2`'s `b` parameter is built by
//! **actually eliciting** the real elaborator's own first-fresh-name choice for a `let t = … in t`
//! shape ([`fresh_kernel_name_via_real_elaboration`], the identical technique
//! `facility_stage1_hygiene.rs` uses) — non-vacuous by construction, robust to `Elab::fresh`'s exact
//! numbering scheme ever changing, and reproduces the realistic "unrelated elaboration happens to
//! mint the same first fresh name" shape rather than a hand-picked string that happens to look
//! right.
//!
//! # The dual non-vacuity oracle (same discipline as E1 / `facility_stage1_hygiene.rs`)
//! (1) [`alpha_eq`] against an independently hand-built oracle using disjoint binder spellings; (2)
//! an independent observational check — [`mycelium_interp::Interpreter::eval`] on `main`'s real,
//! fully-elaborated body (the call-site binding is *already* part of `main`'s own tree — no
//! separate "wrap" step needed, unlike testing the expander in isolation) vs. `eval` on the oracle
//! wrapped the same way; (3) the disable-freshening negative control, which — because
//! `elaborate_value_parametric_rule_disable_freshening_for_test` is `#[cfg(test)]`-only and has
//! exactly **one**, hardcoded (`freshen_binders: true`) call site inside the *production*
//! `elaborate_value_parametric_rule` (Stage 1's own invariant — see that function's doc comment: no
//! runtime flag the ordinary `Elab::app` dispatch could ever reach) — is exercised by calling the
//! disable-freshening entry point directly on this module's own fixture (mirroring
//! `facility_stage1_hygiene.rs`'s pattern), proving the *underlying expansion mechanism* this leaf
//! wires into `Elab::app` is capable of producing a real, observable capture bug when its one
//! safety mechanism is off — not that the *production dispatch path itself* has a disable switch
//! (it structurally does not, by Stage 1's own design).
//!
//! # Scope / guarantee tag (VR-5)
//! A PASS here moves **end-to-end reachability** (check-phase accept → elab-phase §5.2 dispatch →
//! observable evaluation) for the white-box, affine-free, single-nodule, monomorphic
//! value-parametric-rule fragment from `Declared` to `Empirical`. Surface-source reachability (no
//! committed grammar for `value_params` yet — DN-110 §8.6), cross-nodule resolution (Stage 2,
//! OQ-H1), affine soundness over the expanded surface (Stage 3, OQ-H4), and generic value-sugar
//! stay `Declared`/out of scope.

use crate::ast::{
    BaseType, Expr, FnDecl, FnSig, Literal, LowerDecl, LowerRhs, Param, Path, TypeRef, WidthRef,
};
use crate::checkty::{check_nodule, infer_type, Env, Ty, Width};
use crate::elab::{
    elaborate, elaborate_lower_rule, elaborate_value_parametric_rule_disable_freshening_for_test,
};
use crate::parse;
use crate::reveal::alpha_eq;
use mycelium_core::{Meta, Node, Payload, Provenance, Repr, Value};
use mycelium_interp::Interpreter;

fn env(src: &str) -> Env {
    check_nodule(&parse(src).expect("parses")).expect("checks")
}

// -------------------------------------------------------------------------------------------
// Node-level builders (the oracle side — mirrors `facility_stage1_hygiene.rs`'s builders
// exactly, DRY-by-convention with that module rather than by shared code, per the house
// test-layout rule that each in-crate test module is self-contained).
// -------------------------------------------------------------------------------------------

const WIDTH: u32 = 8;

fn c(i: i64) -> Node {
    let bits = mycelium_core::binary::int_to_bits(i, WIDTH).expect("fits in 8 bits");
    Node::Const(
        Value::new(
            Repr::Binary { width: WIDTH },
            Payload::Bits(bits),
            Meta::exact(Provenance::Root),
        )
        .expect("well-formed Binary{8} const"),
    )
}

fn v(name: &str) -> Node {
    Node::Var(name.to_owned())
}

fn letn(id: &str, bound: Node, body: Node) -> Node {
    Node::Let {
        id: id.to_owned(),
        bound: Box::new(bound),
        body: Box::new(body),
    }
}

fn add(x: Node, y: Node) -> Node {
    Node::Op {
        prim: "bin.add".to_owned(),
        args: vec![x, y],
    }
}

fn as_i64(result: &Value) -> i64 {
    match result.payload() {
        Payload::Bits(bits) => mycelium_core::binary::bits_to_int(bits),
        other => panic!("expected a Binary payload, got {other:?}"),
    }
}

// -------------------------------------------------------------------------------------------
// Surface-Expr builders (white-box — no surface grammar for `value_params`/rule-call sites yet).
// -------------------------------------------------------------------------------------------

fn bin_ty(width: u32) -> TypeRef {
    TypeRef {
        base: BaseType::Binary(WidthRef::Lit(width)),
        guarantee: None,
    }
}

fn substrate_ty(tag: &str) -> TypeRef {
    TypeRef {
        base: BaseType::Substrate(tag.to_owned()),
        guarantee: None,
    }
}

fn sc(i: u8) -> Expr {
    Expr::Lit(Literal::Bin(format!("{i:08b}")))
}

fn sv(name: &str) -> Expr {
    Expr::Path(Path(vec![name.to_owned()]))
}

fn slet(name: &str, bound: Expr, body: Expr) -> Expr {
    Expr::Let {
        name: name.to_owned(),
        ty: None,
        bound: Box::new(bound),
        body: Box::new(body),
    }
}

fn sadd(x: Expr, y: Expr) -> Expr {
    Expr::App {
        head: Box::new(sv("add_s")),
        args: vec![x, y],
    }
}

fn scall(name: &str, args: Vec<Expr>) -> Expr {
    Expr::App {
        head: Box::new(sv(name)),
        args,
    }
}

/// The base checked `Env` every fixture starts from — one ordinary nullary `lower` rule (so the
/// env's registries are non-trivially populated, matching `facility_stage1_hygiene.rs`'s own
/// `base_env_with_rule` pattern) and the ordinary prelude (`add_s` et al. are kernel primitives,
/// not `fn`s — no extra registration needed for the swap2 fixture's RHS).
fn base_env() -> Env {
    env("nodule d;\nlower Base = 0b00000000;")
}

/// Register a value-parametric `lower` rule (white-box — no surface grammar yet, per
/// `LowerDecl::value_params`'s doc comment) with two `Binary{8}` value parameters `a, b` and the
/// given `rhs`, into a base checked `Env`.
fn with_rule(e: &mut Env, rule_name: &str, rhs: Expr) {
    e.lower_rules.insert(
        rule_name.to_owned(),
        LowerDecl {
            name: rule_name.to_owned(),
            params: vec![],
            value_params: vec![
                Param {
                    name: "a".to_owned(),
                    ty: bin_ty(8),
                },
                Param {
                    name: "b".to_owned(),
                    ty: bin_ty(8),
                },
            ],
            rhs: LowerRhs::Expr(rhs),
        },
    );
}

/// Register a nullary top-level `fn main` whose body is `body`, returning `Binary{8}` (white-box —
/// `main`'s body references a sugar-rule call site the parser cannot yet produce, so it is
/// hand-built exactly like `facility_stage1_hygiene.rs` hand-builds a rule's own RHS).
fn with_main(e: &mut Env, body: Expr) {
    e.fns.insert(
        "main".to_owned(),
        FnDecl {
            vis: crate::ast::Vis::Private,
            thaw: false,
            tier: None,
            sig: FnSig {
                name: "main".to_owned(),
                params: vec![],
                value_params: vec![],
                ret: bin_ty(8),
                effects: vec![],
                effect_budgets: std::collections::BTreeMap::new(),
            },
            body,
        },
    );
}

/// **Non-vacuity construction** (see the module doc's central point) — identical technique to
/// `facility_stage1_hygiene.rs`'s helper of the same name: elicit the real elaborator's own
/// first-fresh-name choice for a `let <base> = … in <base>` shape through the real public nullary
/// entry point ([`elaborate_lower_rule`]), reading back the kernel variable it minted.
fn fresh_kernel_name_via_real_elaboration(base: &str) -> String {
    let rule_name = format!("Probe{base}");
    let src = format!("nodule d;\nlower {rule_name} = let {base} = 0b00000000 in {base};");
    let e = env(&src);
    let node = elaborate_lower_rule(&e, &rule_name).expect("the probe rule elaborates");
    let Node::Let { ref id, .. } = node else {
        panic!("expected the probe rule to elaborate to a `Let`, got {node:?}");
    };
    id.clone()
}

/// The shared swap2-classic fixture: `Swap2(a, b) = let t = a in add_s(b, t)`, called as
/// `let <colliding> = 7 in Swap2(1, <colliding>)` — `<colliding>` is the real elaborator's own
/// first-fresh-name choice for base `"t"` (see the module doc). Hygienic: the rule's own `t` is
/// re-freshened under a site-qualified namespace, so `b`'s `7` survives: `add_s(7, 1) = 8`.
/// Captured (freshening disabled): the rule's `t` keeps its raw (unqualified, but still Pass-1-
/// fresh) name, which — being the *same* first-fresh-name-for-"t" any independent elaboration
/// mints — collides with the outer `let <colliding> = 7 in …` binding; both operands then read the
/// *inner* `1`: `add_s(1, 1) = 2`.
struct Swap2Fixture {
    colliding: String,
    rhs: Expr,
    main_body: Expr,
    oracle: Node,
    expected_hygienic: i64,
    expected_captured: i64,
}

fn swap2_fixture() -> Swap2Fixture {
    let colliding = fresh_kernel_name_via_real_elaboration("t");
    let rhs = slet("t", sv("a"), sadd(sv("b"), sv("t")));
    let main_body = slet(
        &colliding,
        sc(7),
        scall("Swap2", vec![sc(1), sv(&colliding)]),
    );
    let oracle = letn(
        "oracle_use_site",
        c(7),
        letn(
            "oracle_rule_t",
            c(1),
            add(v("oracle_use_site"), v("oracle_rule_t")),
        ),
    );
    Swap2Fixture {
        colliding,
        rhs,
        main_body,
        oracle,
        expected_hygienic: 8,
        expected_captured: 2,
    }
}

// -------------------------------------------------------------------------------------------
// The full chain: check accepts -> elaborate (ordinary entry, §5.2 dispatch) -> eval agrees.
// -------------------------------------------------------------------------------------------

/// **(1) `Cx::check` accepts the call site** — was a Stage-0 refusal; M-1054 Stage 1b's accept
/// path (`Cx::check_sugar_call`) now types it, naming the RHS's own def-time-fixed result type.
#[test]
fn full_chain_step1_check_accepts() {
    let mut e = base_env();
    let f = swap2_fixture();
    with_rule(&mut e, "Swap2", f.rhs);
    let call = scall("Swap2", vec![sc(1), sv(&f.colliding)]);
    // `<colliding>` is free at this bare-call-expression scope (not wrapped in its own `let`
    // here) — check it in a scope that already binds it, matching how it appears inside `main`.
    let mut scope = vec![(f.colliding.clone(), Ty::Binary(Width::Lit(8)))];
    let ty = infer_type(&e, &mut scope, &call)
        .expect("a recognized, gate-clearing sugar call must be accepted (M-1054 Stage 1b)");
    assert_eq!(ty, Ty::Binary(Width::Lit(8)));
}

/// **(2)+(3) ordinary `elaborate` expands (the new §5.2 `Elab::app` dispatch, not a direct
/// expander call) and the result is capture-safe via two independent oracles.**
#[test]
fn full_chain_step2_elaborate_dispatches_and_is_capture_safe() {
    let mut e = base_env();
    let f = swap2_fixture();
    with_rule(&mut e, "Swap2", f.rhs);
    with_main(&mut e, f.main_body);

    let elaborated = elaborate(&e, "main")
        .expect("elaborate(&env, \"main\") must reach Elab::app's new §5.2 dispatch and expand");

    // -- alpha_eq oracle ---------------------------------------------------------------------
    assert!(
        alpha_eq(&elaborated, &f.oracle),
        "elaborate(&env, \"main\")'s output is not alpha-equivalent to the independently \
         hand-built oracle — a genuine hygiene failure reachable end-to-end, not a comparator \
         artifact (the oracle uses disjoint binder spellings)."
    );

    // -- eval oracle, independent of alpha_eq -------------------------------------------------
    let interp = Interpreter::default();
    let observed = interp
        .eval(&elaborated)
        .unwrap_or_else(|err| panic!("eval(elaborate(main)) failed: {err}"));
    let observed_oracle = interp
        .eval(&f.oracle)
        .unwrap_or_else(|err| panic!("eval(oracle) failed: {err}"));
    assert_eq!(as_i64(&observed), f.expected_hygienic);
    assert_eq!(as_i64(&observed_oracle), f.expected_hygienic);
}

/// **Non-vacuity control (a).** Disabling (A) freshening — on this module's own fixture, via the
/// `#[cfg(test)]`-only entry point (see the module doc for why the *production* `Elab::app`
/// dispatch has no reachable disable switch, by Stage 1's own structural design) — makes *both*
/// oracles fail: the observed value becomes the hand-derived *wrong* (captured) value, not the
/// hygienic one, proving this corpus can catch a real capture bug rather than vacuously never
/// exercising one.
#[test]
fn full_chain_control_disable_freshening_breaks_both_oracles() {
    let mut e = base_env();
    let f = swap2_fixture();
    with_rule(&mut e, "Swap2", f.rhs.clone());

    let args = [c(1), v(&f.colliding)];
    let disabled = elaborate_value_parametric_rule_disable_freshening_for_test(
        &e,
        "Swap2",
        &f.rhs,
        &[("a", &args[0]), ("b", &args[1])],
    )
    .expect("the negative control must still expand (only freshening is disabled)");
    let wrapped = letn(&f.colliding, c(7), disabled);

    let interp = Interpreter::default();
    let observed = interp
        .eval(&wrapped)
        .unwrap_or_else(|err| panic!("eval(disabled-freshening) failed: {err}"));
    assert_eq!(
        as_i64(&observed),
        f.expected_captured,
        "disabling (A) freshening must reproduce the hand-derived captured (wrong) value"
    );
    assert_ne!(
        as_i64(&observed),
        f.expected_hygienic,
        "the captured and hygienic values must differ — otherwise this fixture has no \
         discriminating power (THE NON-VACUITY LAW)"
    );
    assert!(
        !alpha_eq(&wrapped, &f.oracle),
        "the unfreshened (captured) expansion must NOT be alpha-equivalent to the hygienic oracle"
    );
}

// -------------------------------------------------------------------------------------------
// Non-vacuity control (b): the checker's pre-existing negative-control diagnostics still fire —
// M-1054 Stage 1b's accept path did not accidentally swallow them.
// -------------------------------------------------------------------------------------------

#[test]
fn control_arity_mismatch_still_refuses() {
    let mut e = base_env();
    let f = swap2_fixture();
    with_rule(&mut e, "Swap2", f.rhs);
    let call = scall("Swap2", vec![sc(1)]); // only 1 of 2 declared params
    let err = infer_type(&e, &mut Vec::new(), &call).expect_err("arity mismatch must be refused");
    assert!(
        err.message.contains("arity mismatch"),
        "got: {}",
        err.message
    );
}

#[test]
fn control_argument_type_mismatch_still_refuses() {
    let mut e = base_env();
    let f = swap2_fixture();
    with_rule(&mut e, "Swap2", f.rhs);
    let bad_ternary = Expr::Lit(Literal::Trit("+0-+0-".to_owned()));
    let call = scall("Swap2", vec![bad_ternary, sc(2)]);
    let err = infer_type(&e, &mut Vec::new(), &call).expect_err("a type mismatch must be refused");
    assert!(
        err.message.contains('a'),
        "expected the mismatch to name parameter `a`, got: {}",
        err.message
    );
}

#[test]
fn control_item_shaped_rule_still_has_no_expression_form() {
    let mut e = base_env();
    e.lower_rules.insert(
        "ItemRule".to_owned(),
        LowerDecl {
            name: "ItemRule".to_owned(),
            params: vec!["T".to_owned()],
            value_params: vec![],
            rhs: LowerRhs::Impl(crate::ast::ImplDecl {
                trait_name: "Cmp".to_owned(),
                trait_args: vec![],
                for_ty: crate::ast::TypeRef {
                    base: crate::ast::BaseType::Named("T".to_owned(), vec![]),
                    guarantee: None,
                },
                methods: vec![],
            }),
        },
    );
    let call = scall("ItemRule", vec![]);
    let err = infer_type(&e, &mut Vec::new(), &call).expect_err("item-shaped rule must refuse");
    assert!(
        err.message.contains("item-shaped") && err.message.contains("derive"),
        "got: {}",
        err.message
    );
}

// -------------------------------------------------------------------------------------------
// Non-vacuity control (c): the two Stage 1b gates each fire on the shape they're named for.
// -------------------------------------------------------------------------------------------

/// **Stage 3 (OQ-H4) gate fires on an affine value parameter.** A rule declaring a
/// `Substrate`-typed value parameter is refused outright — never silently accepted (G2) — citing
/// "Stage 3" and "OQ-H4".
#[test]
fn control_affine_value_param_hits_stage3_residual() {
    let mut e = base_env();
    e.lower_rules.insert(
        "AffineParam".to_owned(),
        LowerDecl {
            name: "AffineParam".to_owned(),
            params: vec![],
            value_params: vec![Param {
                name: "s".to_owned(),
                ty: substrate_ty("gpu"),
            }],
            rhs: LowerRhs::Expr(sv("s")),
        },
    );
    let call = scall("AffineParam", vec![sv("some_substrate")]);
    let mut scope = vec![("some_substrate".to_owned(), Ty::Substrate("gpu".to_owned()))];
    let err = infer_type(&e, &mut scope, &call)
        .expect_err("an affine (Substrate) value parameter must be refused (Stage 3/OQ-H4)");
    assert!(
        err.message.contains("Stage 3") && err.message.contains("OQ-H4"),
        "expected the Stage-3 (OQ-H4) diagnostic, got: {}",
        err.message
    );
}

/// **Stage 2 (OQ-H1) gate fires on a genuinely free RHS identifier.** A rule whose RHS references
/// an identifier that is neither a value parameter, an RHS-local binder, nor a same-nodule
/// fn/ctor/prim/`lower`-rule is refused — never silently spliced (G2) — citing "Stage 2" and
/// "OQ-H1".
#[test]
fn control_free_non_param_id_hits_stage2_residual() {
    let mut e = base_env();
    with_rule(
        &mut e,
        "FreeRef",
        sadd(sv("a"), sv("totally_undeclared_free_name")),
    );
    let call = scall("FreeRef", vec![sc(1), sc(2)]);
    let err = infer_type(&e, &mut Vec::new(), &call)
        .expect_err("a genuinely free RHS identifier must be refused (Stage 2/OQ-H1)");
    assert!(
        err.message.contains("Stage 2")
            && err.message.contains("OQ-H1")
            && err.message.contains("totally_undeclared_free_name"),
        "expected the Stage-2 (OQ-H1) diagnostic naming the offending identifier, got: {}",
        err.message
    );
}

/// **Stage 3 (OQ-H4) gate, part 2, fires on a structurally affine RHS-local binding** — a
/// `let`-bound call to a `fn` whose declared return type is `Substrate` (the exact
/// "helper-fn-acquired `Substrate`" shape `infer_expr_rule_rhs_type`'s own doc comment names).
#[test]
fn control_affine_rhs_local_binding_hits_stage3_residual() {
    let mut e = base_env();
    e.fns.insert(
        "acquire_thing".to_owned(),
        FnDecl {
            vis: crate::ast::Vis::Private,
            thaw: false,
            tier: None,
            sig: FnSig {
                name: "acquire_thing".to_owned(),
                params: vec![],
                value_params: vec![],
                ret: substrate_ty("gpu"),
                effects: vec![],
                effect_budgets: std::collections::BTreeMap::new(),
            },
            body: sv("a"), // never elaborated/evaluated by this test — check-phase only
        },
    );
    with_rule(
        &mut e,
        "AffineLocal",
        slet("s", scall("acquire_thing", vec![]), sv("a")),
    );
    let call = scall("AffineLocal", vec![sc(1), sc(2)]);
    let err = infer_type(&e, &mut Vec::new(), &call)
        .expect_err("a structurally affine RHS-local binding must be refused (Stage 3/OQ-H4)");
    assert!(
        err.message.contains("Stage 3") && err.message.contains("OQ-H4"),
        "expected the Stage-3 (OQ-H4) diagnostic, got: {}",
        err.message
    );
}
