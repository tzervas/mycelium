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
        "nodule d\ntrait Tr[A] { fn m(x: A) => A }\n\
         impl Tr[Binary{8}] for Binary{8} { fn m(x: Binary{8}) => Binary{8} = x }",
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
        "nodule d\ntrait Tr[A] { fn m(x: A) => A }\ntype Foreign = Mk(Binary{8})\n\
         impl Tr[Foreign] for Foreign { fn m(x: Foreign) => Foreign = x }",
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
        "nodule d\nfn compute(x: Binary{8}) => Binary{8} = not(x)\n\
         fn run() => Binary{8} = colony { hypha compute(0b0000_0001), hypha compute(0b0000_0010) }",
    );
    assert!(e.fn_decl("run").is_some());
}

#[test]
fn a_colony_whose_last_hypha_mistypes_is_an_explicit_error() {
    // The last hypha carries the colony's type, so a `Ternary` last hypha under a `Binary{8}`
    // return is a never-silent body mismatch (the bidirectional check catches it).
    let err = check_err(
        "nodule d\nfn run() => Binary{8} = colony { hypha not(0b0000_0001), hypha 0t00+0 }",
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
        "nodule d\nfn run() => Binary{8} = colony { hypha nope(0b0), hypha not(0b0000_0001) }",
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
    let e = env("nodule d\ntype Nat = Z | S(Nat)\n\
         fn count(n: Nat) => Nat = match n { Z => Z, S(m) => S(count(m)) }\n\
         fn main() => Nat = count(S(Z))");
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
