use crate::ast::{Item, Nodule};
use crate::checkty::*;
use crate::parse;
use std::collections::BTreeMap;

fn env(src: &str) -> Env {
    check_nodule(&parse(src).expect("parses")).expect("checks")
}

/// Copilot #397: a function-typed LHS is parenthesized in `Ty::Fn`'s Display, so `(A => B) => C`
/// is unambiguous (not `A => B => C`); a simple `A => B` and the right-associative RHS stay bare.
/// The `Ty::Fn` Display arrow is `=>` (RFC-0037 D4 — the internal pretty-printer matches the surface).
#[test]
fn ty_fn_display_parenthesizes_a_function_typed_lhs() {
    let var = |n: &str| Ty::Var(n.to_owned());
    let simple = Ty::Fn(Box::new(var("A")), Box::new(var("B")));
    assert_eq!(format!("{simple}"), "A => B");
    let higher_order = Ty::Fn(
        Box::new(Ty::Fn(Box::new(var("A")), Box::new(var("B")))),
        Box::new(var("C")),
    );
    assert_eq!(format!("{higher_order}"), "(A => B) => C");
    let right = Ty::Fn(
        Box::new(var("A")),
        Box::new(Ty::Fn(Box::new(var("B")), Box::new(var("C")))),
    );
    assert_eq!(format!("{right}"), "A => B => C");
}

fn check_err(src: &str) -> CheckError {
    check_nodule(&parse(src).expect("parses")).expect_err("must fail to check")
}

// ---- M-662: the orphan-rule **arm** itself fires (non-vacuous), independent of resolution ----
//
// In the phylum-wide model a *resolvable* impl is never an orphan (resolving a name implies an
// in-phylum declaration ⇒ it is in the pub-blind coherence view). To prove the orphan ARM is not
// dead code, drive `register_instances` directly with a coherence view that does/does not contain
// the impl's heads — the mutant witness that the generalized check still fires + still accepts.

/// A one-`impl` nodule `impl Tr<Binary{8}> for Binary{8} { fn m(x: Binary{8}) -> Binary{8} = x }`
/// plus the registered `types`/`traits` for `Tr`, for driving `register_instances` directly.
fn impl_fixture() -> (
    BTreeMap<String, DataInfo>,
    BTreeMap<String, TraitInfo>,
    Nodule,
) {
    // Parse a phylum-of-one so the surface `impl` + `trait` are real AST (then strip the trait so
    // it is NOT in this nodule — the orphan scenario is "trait declared elsewhere / nowhere").
    let n = parse(
        "nodule d;\n\ntrait Tr[A] {\n  fn m(x: A) => A;\n};\n\nimpl Tr[Binary{8}] for Binary{8} {\n  fn m(x: Binary{8}) => Binary{8} =\n  x;\n};",
    )
    .expect("parses");
    let mut types = BTreeMap::new();
    let p = prelude();
    types.insert(p.name.clone(), p);
    register_types(&mut types, &n).expect("types register");
    let traits = register_traits(&types, &n).expect("traits register");
    // The nodule passed to `register_instances` carries only the `impl` (its locality is decided
    // by the supplied coherence view, not by this nodule's own items — M-662).
    let impl_only = Nodule {
        path: n.path.clone(),
        std_sys: false,
        items: n
            .items
            .iter()
            .filter(|i| matches!(i, Item::Impl(_)))
            .cloned()
            .collect(),
    };
    (types, traits, impl_only)
}

#[test]
fn orphan_arm_rejects_when_neither_head_is_in_the_coherence_view() {
    // Empty coherence view ⇒ `Tr` is not phylum-local and `Binary{8}` is a primitive (always
    // phylum-owned) … so to force the orphan arm we must also deny the primitive. The primitive
    // arm is unconditional, so the genuine orphan case is a `for`-type that is a non-local DATA
    // type. Build that: `for Foreign` where `Foreign` is a registered data type NOT in coherence.
    let n = parse(
        "nodule d;\n\ntrait Tr[A] {\n  fn m(x: A) => A;\n};\n\ntype Foreign = Mk(Binary{8});\n\nimpl Tr[Foreign] for Foreign {\n  fn m(x: Foreign) => Foreign =\n  x;\n};",
    )
    .expect("parses");
    let mut types = BTreeMap::new();
    let p = prelude();
    types.insert(p.name.clone(), p);
    register_types(&mut types, &n).expect("types");
    let traits = register_traits(&types, &n).expect("traits");
    let impl_only = Nodule {
        path: n.path.clone(),
        std_sys: false,
        items: n
            .items
            .iter()
            .filter(|i| matches!(i, Item::Impl(_)))
            .cloned()
            .collect(),
    };
    // Empty coherence view: neither `Tr` nor `Foreign` is phylum-local ⇒ orphan refusal (G2).
    let empty = CoherenceView::default();
    let err = register_instances(&types, &traits, &empty, &impl_only)
        .expect_err("an impl with neither head in the phylum must orphan-reject");
    assert!(
        err.message.contains("orphan"),
        "the orphan arm must fire, got: {}",
        err.message
    );
}

#[test]
fn orphan_arm_accepts_once_the_trait_is_in_the_coherence_view() {
    // The non-vacuous control: add `Tr` to the (pub-blind) coherence view ⇒ the SAME impl is now
    // in-phylum and registers. Proves the orphan generalization accepts a cross-nodule impl whose
    // trait is declared elsewhere in the phylum.
    let (types, traits, impl_only) = impl_fixture();
    let mut coh = CoherenceView::default();
    coh.traits.insert("Tr".to_owned());
    let instances = register_instances(&types, &traits, &coh, &impl_only)
        .expect("the impl registers once its trait is phylum-local");
    assert!(
        instances.contains_key(&("Tr".to_owned(), "Binary".to_owned())),
        "the instance is keyed by (trait, type-head)"
    );
}

// ---- M-666: `colony { hypha … }` type rule (RFC-0008 §4.7) ----

#[test]
fn a_colony_types_as_its_last_hypha() {
    // The colony's result type is the LAST hypha's (the RT2 sequentialization's observable). Here
    // the body must match the fn's `Binary{8}` return — the leading hyphae may be any type.
    let e = env(
        "nodule d;\n\nfn compute(x: Binary{8}) => Binary{8} =\n  not(x);\n\nfn run() => Binary{8} =\n  colony { hypha compute(0b0000_0001), hypha compute(0b0000_0010) };",
    );
    assert!(e.fn_decl("run").is_some());
}

#[test]
fn a_colony_whose_last_hypha_mistypes_is_an_explicit_error() {
    // The last hypha carries the colony's type, so a `Ternary` last hypha under a `Binary{8}`
    // return is a never-silent body mismatch (the bidirectional check catches it).
    let err = check_err(
        "nodule d;\n\nfn run() => Binary{8} =\n  colony { hypha not(0b0000_0001), hypha <00+0> };",
    );
    assert!(
        err.message.contains("body") || err.message.contains("expected"),
        "a mistyped last hypha must be an explicit edge mismatch, got: {}",
        err.message
    );
}

#[test]
fn a_leading_hypha_that_does_not_type_check_is_still_an_error() {
    // RT4/I1: a leading hypha's refusal is never silently dropped — an ill-typed leading hypha
    // (an unknown name) fails the whole colony check.
    let err = check_err(
        "nodule d;\n\nfn run() => Binary{8} =\n  colony { hypha nope(0b0), hypha not(0b0000_0001) };",
    );
    assert!(
        err.message.contains("nope") || err.message.contains("unknown"),
        "an ill-typed leading hypha must surface its error, got: {}",
        err.message
    );
}

#[test]
fn check_error_at_is_a_public_alias() {
    // `::at` builds the same value the private `new` does (the canonical site+message struct).
    assert_eq!(
        CheckError::at("main", "boom"),
        CheckError::new("main", "boom"),
    );
}

#[test]
fn env_getters_mirror_the_public_maps() {
    // A program with a data type and two functions, one recursive (so totality is filled).
    let e = env("nodule d;\n\ntype Nat = Z | S(Nat);\n\nfn count(n: Nat) => Nat =\n  match n { Z => Z, S(m) => S(count(m)) };\n\nfn main() => Nat =\n  count(S(Z));");
    // type_info ⇔ types.get
    assert_eq!(e.type_info("Nat"), e.types.get("Nat"));
    assert!(e.type_info("Nat").is_some());
    assert!(e.type_info("Nope").is_none());
    // fn_decl ⇔ fns.get
    assert_eq!(e.fn_decl("count"), e.fns.get("count"));
    assert!(e.fn_decl("count").is_some());
    assert!(e.fn_decl("absent").is_none());
    // fn_totality ⇔ totality.get (copied)
    assert_eq!(e.fn_totality("count"), e.totality.get("count").copied());
    assert!(e.fn_totality("count").is_some());
    assert!(e.fn_totality("absent").is_none());
}

mod depth_budget_tests {
    use crate::ast::{
        BaseType, Expr, FnDecl, FnSig, Item, Literal, Nodule, Path, TypeRef, WidthRef,
    };
    use crate::checkty::*;

    /// A `not(not(… not(0b0) …))` nest `depth` deep — built directly (the parser caps surface nesting
    /// at `MAX_EXPR_DEPTH`, so a direct AST is the way to exercise the *checker's* own budget).
    pub(crate) fn deep_not(depth: usize) -> Expr {
        let mut e = Expr::Lit(Literal::Bin("0".to_string()));
        for _ in 0..depth {
            e = Expr::App {
                head: Box::new(Expr::Path(Path(vec!["not".to_string()]))),
                args: vec![e],
            };
        }
        e
    }

    pub(crate) fn nodule_with_body(body: Expr) -> Nodule {
        Nodule {
            path: Path(vec!["d".to_string()]),
            std_sys: false,
            items: vec![Item::Fn(FnDecl {
                vis: crate::ast::Vis::Private,
                thaw: false,
                tier: None,
                sig: FnSig {
                    name: "main".to_string(),
                    params: vec![],
                    value_params: vec![],
                    ret: TypeRef {
                        base: BaseType::Binary(WidthRef::Lit(1)),
                        guarantee: None,
                    },
                    effects: vec![],
                },
                body,
            })],
        }
    }

    #[test]
    fn the_depth_budget_trips_cleanly_and_just_under_it_succeeds() {
        // Just under the budget: the checker completes — the deep worker stack ([`mycelium_stack`])
        // absorbs `MAX_CHECK_DEPTH` levels with large margin (measured physical ceiling ≫ budget).
        let ok = check_nodule(&nodule_with_body(deep_not((MAX_CHECK_DEPTH - 5) as usize)));
        assert!(ok.is_ok(), "just under the budget should check ok: {ok:?}");
        // Past the budget: a clean, explicit refusal — never a host-stack overflow (banked guard 4).
        let err = check_nodule(&nodule_with_body(deep_not((MAX_CHECK_DEPTH + 50) as usize)))
            .expect_err("past the budget must refuse");
        assert!(
            err.message.contains("depth budget"),
            "expected the explicit depth-budget refusal, got: {}",
            err.message
        );
    }
}

// ---- DN-54 / M-812-cont: lower / derive validation (check-time) ------------------------------
//
// Note on RHS spelling: a `lower` rule's RHS is a real L1 expression, now **type-checked** (DN-54
// §4.1). The boolean constant is the prelude `Bool` constructor `True`/`False` (capitalised) — the
// lowercase `true`/`false` are *not* L1 names (M-812-cont discovery: the prior structural-only check
// accepted `lower X = true`, but that RHS is ill-typed — it now refuses, as it must).

/// A `lower` rule is registered in `Env::lower_rules` after a successful check.
#[test]
fn lower_rule_is_registered_in_env() {
    let e = env("nodule d;\n\nlower Trivial = True;");
    assert!(
        e.lower_rules.contains_key("Trivial"),
        "`lower Trivial = True` must register the rule name in Env::lower_rules"
    );
}

/// A parametric `lower` rule with one type param is registered. The RHS (`True`) does not mention
/// the type param, so it type-checks under the param scope (DN-54 §4.1).
#[test]
fn lower_rule_with_param_is_registered() {
    let e = env("nodule d;\n\nlower Wrap[T] = True;");
    assert!(
        e.lower_rules.contains_key("Wrap"),
        "`lower Wrap[T] = True` must register the rule name in Env::lower_rules"
    );
    assert_eq!(
        e.lower_rules["Wrap"].params,
        vec!["T".to_owned()],
        "params must be `[T]`"
    );
}

/// A `derive` application referencing a declared rule must check successfully.
#[test]
fn derive_referencing_known_rule_checks() {
    // `derive Trivial for Binary{8}` must check when `lower Trivial = True` is declared first.
    let _ = env("nodule d;\n\nlower Trivial = True;\n\nderive Trivial for Binary{8};");
}

/// A duplicate `lower` rule name in the same nodule is a never-silent check error (G2).
#[test]
fn lower_duplicate_rule_name_is_refused() {
    let err = check_err("nodule d;\n\nlower Trivial = True;\n\nlower Trivial = False;");
    assert!(
        err.message.contains("duplicate"),
        "expected duplicate-rule error, got: {}",
        err.message
    );
    assert!(
        err.message.contains("Trivial"),
        "expected rule name in error, got: {}",
        err.message
    );
}

/// Duplicate parameter names in `lower Name[T, T, …]` is a never-silent check error (G2).
#[test]
fn lower_duplicate_param_is_refused() {
    let err = check_err("nodule d;\n\nlower Bad[T, T] = True;");
    assert!(
        err.message.contains("duplicate"),
        "expected duplicate-param error, got: {}",
        err.message
    );
}

/// A `derive` referencing an unknown rule name is a never-silent check error (G2).
#[test]
fn derive_unknown_rule_name_is_refused() {
    let err = check_err("nodule d;\n\nderive UnknownRule for Binary{8};");
    assert!(
        err.message.contains("unknown"),
        "expected unknown-rule error, got: {}",
        err.message
    );
    assert!(
        err.message.contains("UnknownRule"),
        "expected rule name in error, got: {}",
        err.message
    );
}

// ---- DN-54 §4.1 IL-grammar RHS type-check (M-812-cont) ---------------------------------------

/// §4.1: an **ill-typed** `lower` RHS is refused at definition time (G2). `nope` is not a name in
/// scope, so the RHS fails the IL-grammar / type check — no `derive` site can invoke a broken rule.
#[test]
fn lower_rule_with_ill_typed_rhs_is_refused() {
    let err = check_err("nodule d;\n\nlower Bad = nope;");
    assert!(
        err.message.contains("IL-grammar") || err.message.contains("type check"),
        "expected an IL-grammar/type-check refusal, got: {}",
        err.message
    );
}

/// §4.1: a RHS that uses an in-scope name typed correctly is accepted — here a real L1 literal.
#[test]
fn lower_rule_with_well_typed_literal_rhs_is_accepted() {
    let e = env("nodule d;\n\nlower Eight = 0b0000_0001;");
    assert!(e.lower_rules.contains_key("Eight"));
}

// ---- DN-54 §4.6 purity: no `wild` in a lowering rule's RHS (M-812-cont) ----------------------

/// §4.6: a `lower` rule's RHS may not contain a `wild { … }` block — a generative-lowering rule is
/// a pure compile-time mechanism (the FFI gate is level-independent — DN-38 §3). The refusal is
/// **structural** and names DN-54 §4.6, so it holds even in an `@std-sys` nodule (G2). We assert
/// the refusal fires; the diagnostic cites §4.6 (it may surface as the explicit `wild`-refusal or,
/// for a non-`@std-sys` nodule, as the §4.1 type-check refusal of the `wild` gate — both are
/// never-silent rejections of the rule, which is the load-bearing property).
#[test]
fn lower_rule_with_wild_rhs_is_refused() {
    let err = check_err("nodule d;\n\nlower Impure = wild { host_call() };");
    assert!(
        err.message.contains("wild")
            || err.message.contains("§4.6")
            || err.message.contains("IL-grammar"),
        "expected a never-silent refusal of a `wild`-bearing lower rule, got: {}",
        err.message
    );
}

// ---- DN-54 §4.2 cross-rule acyclicity (M-812-cont) ------------------------------------------

/// §4.2: a `lower` rule whose RHS references **itself** is refused (the trivial cycle) — the
/// lowering-rule graph must be acyclic so `derive` terminates (G2). `Loop`'s RHS is a bare path to
/// `Loop`, which is a registered rule name ⇒ a self-edge.
#[test]
fn lower_rule_self_reference_is_refused() {
    let err = check_err("nodule d;\n\nlower Loop = Loop;");
    assert!(
        err.message.contains("cycle") || err.message.contains("itself"),
        "expected an acyclicity (self-reference) refusal, got: {}",
        err.message
    );
}

/// §4.2: two `lower` rules that reference each other form a cycle and are refused (G2). `A`'s RHS
/// names `B` and `B`'s RHS names `A` — a 2-cycle in the rule graph.
#[test]
fn lower_rules_mutual_cycle_is_refused() {
    let err = check_err("nodule d;\n\nlower A = B;\n\nlower B = A;");
    assert!(
        err.message.contains("cycle"),
        "expected a mutual-cycle refusal, got: {}",
        err.message
    );
}

// ---- DN-54 §6 KC-3 + RHS elaboration to L0 (M-812-cont) -------------------------------------
//
// `low` (M-812) landed `lower`/`derive` as a structural-check-only **residual** (`crate::elab`
// never read `Env::lower_rules`, so a `derive` emitted no L0). M-812-cont lands the load-bearing
// safety + the elaboration: `elaborate_lower_rule` reads `Env::lower_rules` and lowers a rule's RHS
// to a closed L0 `Node` via the **same** path a hand-written nullary fn takes (so the §7
// differential holds by construction; honest tag `Empirical`). KC-3 is `Proven`-by-construction in
// the narrow checked sense: the elaborator's codomain is the *closed* enum `mycelium_core::Node`, so
// a rule cannot add a kernel node — see the assertion below.

/// **RHS elaboration**: a nullary, monomorphic `lower` rule now elaborates to a closed L0 `Node`
/// (no longer a residual). `elaborate_lower_rule` reads `Env::lower_rules` — the M-812-cont
/// completion. The rule's RHS lowers through the same path a hand-written fn would (DRY).
#[test]
fn lower_rule_elaborates_its_rhs_to_l0() {
    let e = env("nodule d;\n\nlower Eight = 0b0000_0001;");
    let node = crate::elab::elaborate_lower_rule(&e, "Eight").expect("rule RHS elaborates to L0");
    // The hand-lowered equivalent: a fn whose body is the same RHS.
    let hand = env("nodule d;\n\nfn eight() => Binary{8} =\n  0b0000_0001;");
    let hand_node = crate::elab::elaborate(&hand, "eight").expect("hand-lowered fn elaborates");
    assert_eq!(
        format!("{node:?}"),
        format!("{hand_node:?}"),
        "DN-54 §7 differential (structural): `elaborate_lower_rule(Eight)` must equal the \
         hand-lowered `fn eight() = 0b0000_0001` — they go through one code path"
    );
}

/// **KC-3 by construction (DN-54 §6)**: the elaborated L0 of a `lower` rule contains **only** the
/// frozen `mycelium_core::Node` variants — a rule adds no new kernel node. The codomain of the
/// elaborator is the closed `Node` enum (the type system is the checked side-condition), so this is
/// `Proven`-by-construction. We confirm the produced node is one of the frozen variants and that its
/// whole tree is in the AOT-lowerable v0 fragment (a total predicate over the frozen node set) — a
/// non-vacuous, never-silent assertion that no out-of-kernel form was synthesised.
#[test]
fn lower_rule_elaboration_adds_no_kernel_node_kc3() {
    let e = env("nodule d;\n\nlower Eight = 0b0000_0001;");
    let node = crate::elab::elaborate_lower_rule(&e, "Eight").expect("rule RHS elaborates");
    // The node is one of the frozen L0 variants (closed enum) — KC-3 by construction.
    assert!(
        node.is_aot_lowerable(),
        "the elaborated rule must lie entirely within the frozen v0 L0 node set (DN-54 §6 / KC-3)"
    );
}

/// An **unknown** rule name passed to `elaborate_lower_rule` is a never-silent `UnknownFn`, never a
/// fabricated artifact (G2).
#[test]
fn elaborate_lower_rule_unknown_is_refused() {
    let e = env("nodule d;\n\nlower Eight = 0b0000_0001;");
    let err = crate::elab::elaborate_lower_rule(&e, "Nope").expect_err("unknown rule must refuse");
    assert!(
        matches!(err, crate::elab::ElabError::UnknownFn(ref n) if n == "Nope"),
        "expected UnknownFn(\"Nope\"), got: {err:?}"
    );
}

/// **KC-3 by absence still holds for an unrelated entry**: a `lower`/`derive` pair adds no L0 to an
/// entry that does not reference it (the rule's L0 is produced *on demand* by
/// `elaborate_lower_rule`, never spliced into an unrelated `main`). This is the descendant of the
/// `low`-era residual guard test — the elaboration is now real, but it stays *out* of any entry
/// that does not derive it.
#[test]
fn lower_derive_items_add_no_l0_to_an_unrelated_entry() {
    let plain = env("nodule d;\n\nfn main() => Binary{8} =\n  0b00000001;");
    let with_rules = env(
        "nodule d;\n\nlower Trivial = True;\n\nderive Trivial for Binary{8};\n\nfn main() => Binary{8} =\n  0b00000001;",
    );
    let node_plain = crate::elab::elaborate(&plain, "main").expect("plain entry elaborates");
    let node_rules =
        crate::elab::elaborate(&with_rules, "main").expect("entry elaborates with rules present");
    assert_eq!(
        format!("{node_plain:?}"),
        format!("{node_rules:?}"),
        "a `lower`/`derive` pair must add NO L0 to an unrelated entry (DN-54 §6, KC-3 by absence; \
         a rule's L0 is produced on demand, not spliced into an unrelated `main`)"
    );
}
