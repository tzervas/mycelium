//! **M-1054 Stage 1 — production capture-avoidance verification** (DN-110-8.2-hygiene-deepdive §4
//! (A)+(B); OQ-H5). Ports `src/tests/hygiene_expr_sugar.rs`'s E1 fixture corpus onto the **real**
//! elaborator path — [`crate::elab::elaborate_lower_rule_with_args`] — instead of E1's test-only
//! `Expander`. This is the production go/no-go: E1's PASS was over a throwaway prototype; the real
//! elaborator path needs its own witness before (A)+(B) can be tagged `Empirical` *for that path*
//! (VR-5 — a checked basis, not inherited from a different code path).
//!
//! # Why the fixtures are NOT a byte-for-byte copy of E1 (an honest adaptation, not a shortcut)
//!
//! E1's `Expander` walked a **hand-built** `Node` with every binder still in its **raw surface
//! spelling** (`"t"`, never `"t%0"`), so E1's capture scenario was: does the walker's own (A)+(B)
//! correctly avoid a *surface-level* spelling collision between an RHS binder and a literal
//! use-site free variable of the same spelling?
//!
//! The **production** path is different in one respect that turns out to matter: `elaborate_value_
//! parametric_rule`'s first pass elaborates the RHS through the **real elaborator**
//! (`Elab::expr`), which — exactly like elaborating any ordinary function body — **already**
//! assigns every `let`/`lambda` binder a fresh, `%`-namespaced kernel name via `Elab::fresh`
//! (unconditionally, the same machinery every other elaboration path uses). So a *literal* surface
//! spelling collision (an RHS `let t = …` vs. a use-site free `Var("t")`) can no longer arise after
//! pass 1 — pass 1's own pre-existing scope handling already resolves it, for free, before pass 2
//! (`sugar_expand`) ever runs. Reusing E1's fixtures verbatim (with a bare `Var("t")` argument)
//! would therefore be **vacuous** on the production path: freshening on or off would produce the
//! same observable result, because there is nothing left to collide with.
//!
//! The **real** residual hazard on the production path is exactly what OQ-H5 names: **cross-
//! invocation** collision. `Elab::fresh`'s counter resets to `0` for every *independent* top-level
//! elaboration (a fresh `Elab` per call), so two unrelated elaborations can each mint the identical
//! kernel name (`"t%0"`) for their own, unrelated binders. If an argument passed to
//! `elaborate_lower_rule_with_args` is itself a **free reference into some other, already-
//! elaborated context** (the realistic shape a real caller would pass — see `/forward`'s def-site-
//! resolution note, OQ-H1) and that reference happens to be spelled exactly what *this* RHS's own
//! pass 1 is about to mint, disabling pass 2's site-qualified re-freshening captures it for real.
//! So every fixture below builds its "escaping free variable" argument by **actually eliciting**
//! the real elaborator's own first-fresh-name choice (`fresh_kernel_name_via_real_elaboration`)
//! rather than a hand-picked string — non-vacuous by construction, and robust to `Elab::fresh`'s
//! exact numbering scheme ever changing.
//!
//! # The dual non-vacuity oracle (same discipline as E1 — module doc points 1-3 there)
//!
//! For every fixture: (1) [`alpha_eq`] against an independently hand-built oracle using disjoint
//! binder spellings; (2) an independent observational check — `Interpreter::eval` on the real
//! expansion wrapped in its use-site binding, compared to `eval` on the oracle wrapped the same
//! way; (3) the **disable-freshening negative control**
//! (`elaborate_value_parametric_rule_disable_freshening_for_test`, `#[cfg(test)]`-gated in
//! `elab.rs`) — the exact same production pipeline with pass 2's freshening turned off — which
//! must (and does) evaluate to a *different*, wrong value, proving this corpus is capable of
//! catching a real capture bug, not merely failing to trigger one it never exercises.
//!
//! # Scope / guarantee tag (VR-5)
//! A PASS here moves capture-avoidance for **(A)+(B), on the real `elaborate_lower_rule_with_args`
//! path**, `Declared -> Empirical`. It says nothing about (C) def-site resolution or (D) affine-on-
//! expanded-L0 (both stay `Declared`, Stage 2/3), and nothing about the L1 check-phase
//! (`Cx::check_sugar_call`, unchanged by this leaf — still refuses every recognized call).

use crate::ast::{BaseType, Expr, Literal, LowerDecl, LowerRhs, Param, Path, TypeRef, WidthRef};
use crate::checkty::{check_nodule, Env};
use crate::elab::{
    elaborate_lower_rule, elaborate_lower_rule_with_args,
    elaborate_value_parametric_rule_disable_freshening_for_test,
};
use crate::parse;
use crate::reveal::alpha_eq;
use mycelium_core::{Meta, Node, Payload, Provenance, Repr, Value};
use mycelium_interp::Interpreter;

fn env(src: &str) -> Env {
    check_nodule(&parse(src).expect("parses")).expect("checks")
}

// -------------------------------------------------------------------------------------------
// Node-level builders (the use-site / oracle side — mirrors `hygiene_expr_sugar.rs`'s builders
// exactly, DRY-by-convention with that module rather than by shared code, per the house test-
// layout rule that each in-crate test module is self-contained).
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
// Surface-Expr builders (the RHS side — what a real `lower` rule's RHS parses/is constructed to;
// fed to `elaborate_lower_rule_with_args` through a real, checked-shape `Env`, never through
// E1's Node-level `Expander`).
// -------------------------------------------------------------------------------------------

fn bin_ty(width: u32) -> TypeRef {
    TypeRef {
        base: BaseType::Binary(WidthRef::Lit(width)),
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

/// Register a value-parametric `lower` rule (white-box — no surface grammar yet, per
/// `LowerDecl::value_params`'s doc comment) with two `Binary{8}` value parameters `a, b` and the
/// given `rhs`, into a base checked `Env`.
fn base_env_with_rule(rule_name: &str, rhs: Expr) -> Env {
    let mut e = env("nodule d;\nlower Base = 0b00000000;");
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
    e
}

/// **Non-vacuity construction (the module doc's central point).** The realistic spelling an
/// "escaping free variable from some other, unrelated elaboration" argument could carry — obtained
/// by *actually eliciting* the real elaborator's own first-fresh-name choice for a `let <base> =
/// … in <base>` rule, through the real public nullary entry point ([`elaborate_lower_rule`]), and
/// reading back the kernel variable it minted. Every independent top-level elaboration resets
/// `Elab::fresh`'s counter to `0` (a fresh `Elab` per call), so this is exactly the name a second,
/// unrelated `let <base> = …` would *also* mint — the real OQ-H5 cross-invocation collision shape,
/// not a hand-picked string that happens to look right.
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

/// One production fixture: a value-parametric rule (`a`, `b`) whose RHS shadows/reuses a binder
/// spelled the same as [`fresh_kernel_name_via_real_elaboration`]'s realistic colliding free
/// variable, an oracle built independently (disjoint binder spellings), and the hand-derived
/// hygienic/captured expected values (same numbers E1 derived — only the *colliding spelling*
/// differs, the arithmetic is unchanged).
struct Fixture {
    name: &'static str,
    rule_name: &'static str,
    rhs: Expr,
    /// `(arg_for_a, arg_for_b)` — `arg_for_b` (or `arg_for_a`, per fixture) is the colliding free
    /// variable built via [`fresh_kernel_name_via_real_elaboration`].
    args: (Node, Node),
    /// The use-site binding wrapping the expansion — binds the colliding free variable's name to
    /// its "real" outer value.
    wrap_id: String,
    wrap_value: Node,
    oracle: Node,
    expected_hygienic: i64,
    expected_captured: i64,
}

/// **Fixture 1 — binder-shadows-use-site (the swap2 classic, DN-110-8.2-hygiene-deepdive §7 E1),
/// adapted to the cross-invocation collision the production path actually needs to guard (see the
/// module doc).** `swap2(a, b) = let t = a in add(b, t)`, invoked with `a = 1`, `b` = the colliding
/// free variable — the same shape a second, unrelated `let t = …` elsewhere would also produce. The
/// use site binds that name to `7`. Hygienic: the RHS's own `t` is re-freshened under a site-
/// qualified namespace, so `b`'s `7` survives: `add(7, 1) = 8`. Captured (freshening disabled): the
/// RHS's `t` keeps pass 1's raw (unqualified) name, colliding with `b`'s own reference, so both
/// operands read the *inner* `1`: `add(1, 1) = 2`.
fn fixture_binder_shadows_use_site() -> Fixture {
    let rhs = slet("t", sv("a"), sadd(sv("b"), sv("t")));
    let colliding = fresh_kernel_name_via_real_elaboration("t");
    let oracle = letn("t_h1", c(1), add(v(&colliding), v("t_h1")));
    Fixture {
        name: "binder_shadows_use_site (swap2 classic)",
        rule_name: "Swap2",
        rhs,
        args: (c(1), v(&colliding)),
        wrap_id: colliding,
        wrap_value: c(7),
        oracle,
        expected_hygienic: 8,
        expected_captured: 2,
    }
}

/// **Fixture 2 — arg mentions the RHS's raw binder spelling, from the OTHER parameter position.**
/// `pair_add(a, b) = let t = b in add(a, t)`, `a` = the colliding free variable, `b = 9`. Hygienic:
/// `add(3, 9) = 12` (the use site binds the colliding name to `3`). Captured: the unfreshened
/// `let t = 9 in …` shadows the reference, hijacking it: `add(9, 9) = 18`.
fn fixture_arg_mentions_raw_binder_spelling() -> Fixture {
    let rhs = slet("t", sv("b"), sadd(sv("a"), sv("t")));
    let colliding = fresh_kernel_name_via_real_elaboration("t");
    let oracle = letn("t_h2", c(9), add(v(&colliding), v("t_h2")));
    Fixture {
        name: "arg_mentions_raw_binder_spelling (pair_add)",
        rule_name: "PairAdd",
        rhs,
        args: (v(&colliding), c(9)),
        wrap_id: colliding,
        wrap_value: c(3),
        oracle,
        expected_hygienic: 12,
        expected_captured: 18,
    }
}

/// **Fixture 3 — multi-param, RHS binder used twice.** `f(a, b) = let t = a in add(b, add(t, t))`,
/// `a = 5`, `b` = the colliding free variable bound to `2` at the use site. Hygienic:
/// `add(2, add(5, 5)) = 12`. Captured: `let t = 5 in …` shadows, so `b`'s reference reads the inner
/// `5` too: `add(5, add(5, 5)) = 15`.
fn fixture_multi_param_used_twice() -> Fixture {
    let rhs = slet("t", sv("a"), sadd(sv("b"), sadd(sv("t"), sv("t"))));
    let colliding = fresh_kernel_name_via_real_elaboration("t");
    let oracle = letn("t_h3", c(5), add(v(&colliding), add(v("t_h3"), v("t_h3"))));
    Fixture {
        name: "multi_param_used_twice (f)",
        rule_name: "MultiUse",
        rhs,
        args: (c(5), v(&colliding)),
        wrap_id: colliding,
        wrap_value: c(2),
        oracle,
        expected_hygienic: 12,
        expected_captured: 15,
    }
}

/// **Fixture 4 — nested binder in the RHS (TWO `let`s, both spelled `t`).**
///
/// **Honest adaptation, flagged (not a shortcut).** E1's own fixture 4 used a `let` **and** a
/// `lambda`, both spelled `t`, to exercise that (A) freshens every kind of binder independently.
/// Porting the `lambda` form literally onto the real elaborator hits a **pre-existing, orthogonal**
/// limitation — confirmed to predate this leaf and to be independent of the value-parametric (A)+
/// (B) work: `elaborate_lower_rule`'s synthetic-single-function-`Env` mechanism (unchanged since
/// before M-1054) cannot elaborate **any** `lower` rule whose RHS is a `lambda` immediately applied
/// (an IIFE) — `crate::mono`'s closure defunctionalization synthesizes a dispatcher function
/// (`apply$Fn$…`) that the ad-hoc single-function synthetic `Env` this mechanism builds does not
/// register, so elaboration refuses with `unknown function/constructor/prim
/// apply$Fn$Binary8$Binary8`. **Reproduced on the plain nullary path too**
/// (`elaborate_lower_rule` on `lower L = (lambda(x: Binary{8}) => add_s(x, 1))(2);` fails
/// identically), so this is not a Stage 1 regression — it is a standing gap in how a `lower` rule's
/// RHS is elaborated at all, independent of value parameters, out of this leaf's (A)+(B) scope
/// (FLAGGED for whoever next touches `elaborate_lower_rule`'s synthetic-entry construction).
///
/// This fixture is therefore adapted to a **second nested `let`** in place of the `lambda`
/// (`let t = b in add(t, 1)` is the beta-reduced form of `(lambda(t) => add(t, 1))(b)` — same
/// value, same binder-nesting shape, same hand-derived expected numbers as E1's original) so it
/// still exercises "(A) freshens every RHS binder independently, including two *different* binders
/// that happen to share a spelling" without depending on the orthogonal lambda-IIFE gap.
///
/// `nest(a, b) = let t = a in add((let t = b in add(t, 1)), t)`, `a = 2`, `b` = the colliding free
/// variable bound to `100` at the use site. Hygienic: inner `let t = 100 in add(t, 1) = 101`, then
/// outer `add(101, 2) = 103`. Captured (freshening disabled): the *outer* `t` keeps pass 1's raw
/// name, colliding with `b`'s own reference — so `b` (used as the *inner* let's bound expression)
/// is captured by the **outer** binder (bound to `a = 2`, not the true use-site `100`): inner
/// `let t = 2 in add(t, 1) = 3`, then outer `add(3, 2) = 5` — the *inner* `t` is a distinct pass-1
/// binder either way (its own fresh name never collides), so only the outer/use-site collision
/// fires, exactly mirroring the lambda version's captured value.
fn fixture_nested_binder_in_rhs() -> Fixture {
    let rhs = slet(
        "t",
        sv("a"),
        sadd(slet("t", sv("b"), sadd(sv("t"), sc(1))), sv("t")),
    );
    let colliding = fresh_kernel_name_via_real_elaboration("t");
    let oracle = letn(
        "oracle_let_t",
        c(2),
        add(
            letn(
                "oracle_inner_t",
                v(&colliding),
                add(v("oracle_inner_t"), c(1)),
            ),
            v("oracle_let_t"),
        ),
    );
    Fixture {
        name: "nested_binder_in_rhs (nest — two lets both spelled t; adapted from E1's let+lambda)",
        rule_name: "Nest",
        rhs,
        args: (c(2), v(&colliding)),
        wrap_id: colliding,
        wrap_value: c(100),
        oracle,
        expected_hygienic: 103,
        expected_captured: 5,
    }
}

fn core_fixtures() -> Vec<Fixture> {
    vec![
        fixture_binder_shadows_use_site(),
        fixture_arg_mentions_raw_binder_spelling(),
        fixture_multi_param_used_twice(),
        fixture_nested_binder_in_rhs(),
    ]
}

/// **The M-1054 Stage 1 production go/no-go.** For every fixture: (1) the real
/// `elaborate_lower_rule_with_args` expansion is `alpha_eq` to the independently hand-built oracle;
/// (2) `eval` of the expansion (wrapped in its use-site binding) agrees with `eval` of the oracle
/// (wrapped the same way), independent of `alpha_eq`; (3) the disable-freshening negative control —
/// the same pipeline with pass 2's freshening turned off — evaluates to the hand-derived *wrong*
/// (captured) value, proving this corpus is capable of catching a real capture bug (non-vacuity).
/// A failure anywhere here is reported honestly (house rule #2/VR-5): it is a genuine finding about
/// the production path, never patched away by adjusting a fixture to force a pass.
#[test]
fn stage1_production_capture_avoidance_corpus() {
    let interp = Interpreter::default();
    for f in core_fixtures() {
        let e = base_env_with_rule(f.rule_name, f.rhs.clone());
        let args = vec![f.args.0.clone(), f.args.1.clone()];

        // -- (1) Structural check: real expansion vs. independent oracle -----------------------
        let expanded =
            elaborate_lower_rule_with_args(&e, f.rule_name, &args).unwrap_or_else(|err| {
                panic!(
                    "[{}] the production path must expand a matched call, got {err:?}",
                    f.name
                )
            });
        assert!(
            alpha_eq(&expanded, &f.oracle),
            "[{}] elaborate_lower_rule_with_args(...) is not alpha-equivalent to the \
             independently hand-built oracle — a genuine hygiene failure on the production path, \
             not a comparator artifact (the oracle uses a disjoint binder-naming scheme).",
            f.name
        );

        // -- (2) Observational check, independent of alpha_eq -----------------------------------
        let wrap = |inner: Node| letn(&f.wrap_id, f.wrap_value.clone(), inner);
        let observed_expanded = interp
            .eval(&wrap(expanded))
            .unwrap_or_else(|err| panic!("[{}] eval(expand(...)) failed: {err}", f.name));
        let observed_oracle = interp
            .eval(&wrap(f.oracle.clone()))
            .unwrap_or_else(|err| panic!("[{}] eval(oracle) failed: {err}", f.name));
        assert_eq!(
            as_i64(&observed_expanded),
            f.expected_hygienic,
            "[{}] eval(elaborate_lower_rule_with_args(...)) did not match the hand-derived \
             expected value",
            f.name
        );
        assert_eq!(
            as_i64(&observed_oracle),
            f.expected_hygienic,
            "[{}] eval(oracle) did not match the hand-derived expected value (oracle itself is \
             miscomputed)",
            f.name
        );

        // -- (3) Disable-freshening negative control — non-vacuity, module doc + task grounding -
        let disabled = elaborate_value_parametric_rule_disable_freshening_for_test(
            &e,
            f.rule_name,
            &f.rhs,
            &[("a", &f.args.0), ("b", &f.args.1)],
        )
        .unwrap_or_else(|err| {
            panic!(
                "[{}] the negative control must still expand (only freshening is disabled), got {err:?}",
                f.name
            )
        });
        let observed_disabled = interp
            .eval(&wrap(disabled))
            .unwrap_or_else(|err| panic!("[{}] eval(disabled-freshening) failed: {err}", f.name));
        assert_eq!(
            as_i64(&observed_disabled),
            f.expected_captured,
            "[{}] disabling (A) freshening must reproduce the hand-derived captured (wrong) value \
             — if it doesn't, this fixture cannot demonstrate the corpus catches a real capture \
             bug (non-vacuity)",
            f.name
        );
        assert_ne!(
            f.expected_captured, f.expected_hygienic,
            "[{}] this fixture's captured/hygienic values coincide — it cannot demonstrate \
             discriminating power",
            f.name
        );
    }
}
