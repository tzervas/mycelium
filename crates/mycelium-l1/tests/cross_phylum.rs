//! **Cross-phylum import/resolution subsystem** (DN-113 Rank 1 / M-1060) integration tests — the
//! v1 checked-and-linked (whole-graph, content-pinned, CHECK-TIME) mechanism: the `::`
//! phylum-boundary `use dep::a.b.Item` reference, the additive `Phyla`/`ResolvedPhylum` dependency
//! set, layering over the existing `Exports`/`resolve_imports`/`PhylumEnv::link` machinery (DRY, no
//! second linker), and the acyclicity-enforcing multi-phylum graph builder (`phyla::PhylumNode` /
//! `phyla::build_phyla_graph`).
//!
//! Every "the check fires" test is paired with a **control** proving the check is not vacuous (the
//! same shape, minus the violation, is accepted) — the M-662/DN-104/DN-112 precedent this subsystem
//! extends one level up, across the phylum boundary. Honesty (VR-5): every guarantee here is
//! `Empirical` (checked by these witnesses, never `Proven` — no discharged theorem backs it).

use mycelium_core::ContentHash;
use mycelium_l1::phyla::{build_phyla_graph, PhylumNode};
use mycelium_l1::{
    check_phylum, check_phylum_with_deps, parse_phylum, CheckError, Phyla, Phylum, ResolvedPhylum,
};
use std::collections::BTreeMap;

/// A deterministic, well-formed (but not a real content digest — `Declared`, not `Exact`) hash for
/// fixture use — distinct per fixture via the discriminator byte, so two different fixture phyla
/// never accidentally share a "pin".
fn fixture_hash(discriminator: u8) -> ContentHash {
    let digest = format!("{discriminator:02x}", discriminator = discriminator).repeat(32);
    ContentHash::from_parts("blake3", &digest).expect("well-formed fixture digest")
}

/// Parse `src` as a phylum (panics on a parse error — every fixture here is deliberately
/// well-formed at the surface-syntax level; only the *check* is under test).
fn phy(src: &str) -> Phylum {
    parse_phylum(src).expect("fixture parses as a phylum")
}

/// Resolve `src` into a [`ResolvedPhylum`] (checked + linked), the dependency-fixture helper every
/// cross-phylum test builds its `Phyla` from.
fn resolved(src: &str, discriminator: u8) -> ResolvedPhylum {
    ResolvedPhylum::resolve(fixture_hash(discriminator), &phy(src), &Phyla::default())
        .expect("dependency fixture checks")
}

/// Check `src` against `deps`, returning the per-nodule envs.
fn check_with(src: &str, deps: &Phyla) -> Result<mycelium_l1::PhylumEnv, CheckError> {
    check_phylum_with_deps(&phy(src), deps)
}

/// Check `src` against `deps`, expecting a never-silent `CheckError`; returns its message.
fn check_with_err(src: &str, deps: &Phyla) -> String {
    check_with(src, deps)
        .expect_err("must fail to check")
        .message
}

// ---------------------------------------------------------------------------------------------
// The headline: a cross-phylum `use dep::nod.sym` resolves the correct foreign symbol.
// ---------------------------------------------------------------------------------------------

#[test]
fn cross_phylum_use_of_a_pub_fn_resolves_and_type_checks() {
    let dep = resolved(
        "phylum d\nnodule math;\npub fn add1(x: Binary{8}) => Binary{8} = x;",
        1,
    );
    let mut deps = BTreeMap::new();
    deps.insert("collections".to_owned(), dep);
    let phyla = Phyla::from_deps(deps);

    let penv = check_with(
        "phylum p\nnodule use_it;\nuse collections::math.add1;\n\
         pub fn go(y: Binary{8}) => Binary{8} = add1(y);",
        &phyla,
    )
    .expect("a cross-phylum `use` of a pub fn type-checks");
    let env = penv
        .nodule(&mycelium_l1::ast::Path(vec!["use_it".to_owned()]))
        .expect("nodule present");
    assert!(env.fn_decl("go").is_some(), "the consumer's own fn checked");
    assert!(
        env.fn_decl("add1").is_some(),
        "the imported cross-phylum fn is visible in the consumer's checked env \
         (M-662's cross-nodule pattern, extended one level up — DN-113 §7)"
    );
}

/// Non-vacuity control: the SAME source, minus the `use`, does not spuriously resolve `add1` (the
/// name genuinely comes from the cross-phylum import, not some ambient fallback).
#[test]
fn control_without_the_use_the_foreign_fn_is_genuinely_unresolved() {
    let dep = resolved(
        "phylum d\nnodule math;\npub fn add1(x: Binary{8}) => Binary{8} = x;",
        2,
    );
    let mut deps = BTreeMap::new();
    deps.insert("collections".to_owned(), dep);
    let phyla = Phyla::from_deps(deps);

    let err = check_with_err(
        "phylum p\nnodule use_it;\npub fn go(y: Binary{8}) => Binary{8} = add1(y);",
        &phyla,
    );
    assert!(
        !err.is_empty(),
        "without the `use`, `add1` is not in scope — a real unknown-name refusal"
    );
}

/// Non-vacuity control: an empty `Phyla` (no `[dependencies]` at all) checks a dependency-free
/// phylum identically to the pre-M-1060 [`check_phylum`] — the additive-identity claim (DN-113 §5.1).
#[test]
fn empty_phyla_checks_identically_to_check_phylum() {
    let src = "phylum p\nnodule solo;\npub fn id(x: Binary{8}) => Binary{8} = x;";
    let via_plain = check_phylum(&phy(src)).expect("plain check_phylum succeeds");
    let via_empty_deps =
        check_phylum_with_deps(&phy(src), &Phyla::default()).expect("empty-Phyla check succeeds");
    let plain_env = via_plain
        .nodule(&mycelium_l1::ast::Path(vec!["solo".to_owned()]))
        .unwrap();
    let deps_env = via_empty_deps
        .nodule(&mycelium_l1::ast::Path(vec!["solo".to_owned()]))
        .unwrap();
    assert_eq!(
        plain_env.fns.keys().collect::<Vec<_>>(),
        deps_env.fns.keys().collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------------------------
// Never-silent refusals (DN-113 §7/§8/§9): unknown dependency, unknown/private symbol, v1 glob.
// ---------------------------------------------------------------------------------------------

#[test]
fn use_of_an_undeclared_dependency_is_a_never_silent_refusal() {
    // No dependency named `nosuch` in `deps` at all.
    let err = check_with_err(
        "phylum p\nnodule use_it;\nuse nosuch::math.add1;\n\
         pub fn go() => Binary{8} = add1(0b0000_0000);",
        &Phyla::default(),
    );
    assert!(err.contains("no such dependency"), "got: {err}");
}

#[test]
fn use_of_an_unknown_symbol_in_a_known_dependency_is_a_never_silent_refusal() {
    let dep = resolved(
        "phylum d\nnodule math;\npub fn add1(x: Binary{8}) => Binary{8} = x;",
        3,
    );
    let mut deps = BTreeMap::new();
    deps.insert("collections".to_owned(), dep);
    let phyla = Phyla::from_deps(deps);

    let err = check_with_err(
        "phylum p\nnodule use_it;\nuse collections::math.no_such_fn;",
        &phyla,
    );
    assert!(err.contains("no such name"), "got: {err}");
}

#[test]
fn use_of_a_private_symbol_in_a_dependency_is_a_never_silent_refusal_distinguishing_private() {
    let dep = resolved(
        "phylum d\nnodule math;\nfn helper(x: Binary{8}) => Binary{8} = x;",
        4,
    );
    let mut deps = BTreeMap::new();
    deps.insert("collections".to_owned(), dep);
    let phyla = Phyla::from_deps(deps);

    let err = check_with_err(
        "phylum p\nnodule use_it;\nuse collections::math.helper;",
        &phyla,
    );
    assert!(
        err.contains("not `pub`") || err.contains("private"),
        "got: {err}"
    );
}

#[test]
fn a_cross_phylum_glob_is_refused_in_v1() {
    let dep = resolved(
        "phylum d\nnodule math;\npub fn add1(x: Binary{8}) => Binary{8} = x;",
        5,
    );
    let mut deps = BTreeMap::new();
    deps.insert("collections".to_owned(), dep);
    let phyla = Phyla::from_deps(deps);

    let err = check_with_err("phylum p\nnodule use_it;\nuse collections::math.*;", &phyla);
    assert!(
        err.contains("glob") && err.contains("not supported"),
        "got: {err}"
    );
}

// ---------------------------------------------------------------------------------------------
// DN-112 Rank 1 / M-1036 extension: foreign type identity stays distinct from a same-named local.
// ---------------------------------------------------------------------------------------------

#[test]
fn a_foreign_type_from_a_dependency_is_distinct_from_a_same_named_local_type() {
    // `dep` declares `T`; the consumer ALSO declares its own, differently-shaped, same-named `T`,
    // and imports the dependency's `use_t` (which takes the DEPENDENCY's `T`). Constructing a LOCAL
    // `T` and passing it to the foreign `use_t` must be a genuine type mismatch — the exact
    // cross-phylum extension of the DN-112/M-1036 ctor-seal/identity fix (no bare-name collapse
    // across the phylum boundary).
    let dep = resolved(
        "phylum d\nnodule math;\n\
         pub type T = Mk(Binary{8});\n\
         pub fn use_t(x: T) => Binary{8} = match x { Mk(v) => v };",
        6,
    );
    let mut deps = BTreeMap::new();
    deps.insert("collections".to_owned(), dep);
    let phyla = Phyla::from_deps(deps);

    let err = check_with_err(
        "phylum p\nnodule use_it;\n\
         use collections::math.use_t;\n\
         type T = Mk(Binary{8});\n\
         fn forge() => T = Mk(0b0000_0000);\n\
         pub fn exploit() => Binary{8} = use_t(forge());",
        &phyla,
    );
    // Confirms a genuine, home-qualified type mismatch — not a spurious unrelated failure: the
    // consumer's local `T` and the dependency's `T` carry DISTINCT phylum-qualified identities
    // (`use_it::T` vs `collections::math::T`), exactly the DN-112 Rank 1 mechanism extended across
    // the phylum boundary (never a bare-name collapse — G2).
    assert!(
        err.contains("use_it::T") && err.contains("collections::math::T"),
        "a same-named local `T` must NOT satisfy the foreign dependency's `T` — identity does not \
         collapse across the phylum boundary (DN-112 Rank 1 extended by DN-113/M-1060); got: {err}"
    );
}

/// Non-vacuity control: the SAME shape, but the consumer passes a value obtained from the
/// dependency's OWN factory (never constructing a local shadow) — a legitimate cross-phylum flow
/// that must NOT be over-restricted by the identity fix.
#[test]
fn a_legitimate_cross_phylum_flow_using_the_dependencys_own_factory_still_works() {
    let dep = resolved(
        "phylum d\nnodule math;\n\
         pub type T = Mk(Binary{8});\n\
         pub fn make() => T = Mk(0b0000_0000);\n\
         pub fn use_t(x: T) => Binary{8} = match x { Mk(v) => v };",
        7,
    );
    let mut deps = BTreeMap::new();
    deps.insert("collections".to_owned(), dep);
    let phyla = Phyla::from_deps(deps);

    check_with(
        "phylum p\nnodule use_it;\n\
         use collections::math.make;\nuse collections::math.use_t;\n\
         pub fn go() => Binary{8} = use_t(make());",
        &phyla,
    )
    .expect(
        "a value obtained from the dependency's own factory and passed straight back through \
         still type-checks (the identity fix must not over-restrict a legitimate flow)",
    );
}

/// A same-named type in the dependency and the consumer, used **independently** (never mixed),
/// both still check — the identity fix is about cross-phylum MIXING, not about forbidding a common
/// name (mirrors `unrelated_same_named_types_in_different_nodules_used_independently_still_check`
/// intra-phylum in `tests/ctor_seal.rs`).
#[test]
fn same_named_types_used_independently_across_the_phylum_boundary_both_still_check() {
    let dep = resolved(
        "phylum d\nnodule math;\npub type T = Mk(Binary{8});\npub fn dep_use(x: T) => Binary{8} = match x { Mk(v) => v };",
        8,
    );
    let mut deps = BTreeMap::new();
    deps.insert("collections".to_owned(), dep);
    let phyla = Phyla::from_deps(deps);

    check_with(
        "phylum p\nnodule use_it;\n\
         type T = Mk(Binary{4});\n\
         fn local_use(x: T) => Binary{4} = match x { Mk(v) => v };\n\
         pub fn go() => Binary{4} = local_use(Mk(0b0000));",
        &phyla,
    )
    .expect("the consumer's own unrelated same-named local type checks independently");
}

// ---------------------------------------------------------------------------------------------
// DN-113 §9.3: the acyclic-phyla precondition, enforced by `phyla::build_phyla_graph`.
// ---------------------------------------------------------------------------------------------

#[test]
fn a_cyclic_phyla_graph_is_refused_never_silently() {
    let mut graph = BTreeMap::new();
    graph.insert(
        "a".to_owned(),
        PhylumNode {
            phylum_hash: fixture_hash(0xA),
            phylum: phy("phylum a\nnodule n;\npub fn f() => Binary{8} = 0b0000_0000;"),
            deps: BTreeMap::from([("b".to_owned(), "b".to_owned())]),
        },
    );
    graph.insert(
        "b".to_owned(),
        PhylumNode {
            phylum_hash: fixture_hash(0xB),
            phylum: phy("phylum b\nnodule n;\npub fn g() => Binary{8} = 0b0000_0000;"),
            deps: BTreeMap::from([("a".to_owned(), "a".to_owned())]),
        },
    );

    let err = build_phyla_graph(&graph).expect_err("a two-cycle must be refused");
    assert!(err.message.contains("cyclic"), "got: {}", err.message);
}

/// Non-vacuity control: an ACYCLIC two-level graph (root depends on a leaf) resolves cleanly, and
/// the root's own `use` of the leaf's symbol checks — proves the cycle detector is not vacuously
/// refusing every multi-node graph.
#[test]
fn an_acyclic_two_level_graph_resolves_and_the_roots_cross_phylum_use_checks() {
    let mut graph = BTreeMap::new();
    graph.insert(
        "leaf".to_owned(),
        PhylumNode {
            phylum_hash: fixture_hash(0xC),
            phylum: phy("phylum leaf\nnodule math;\npub fn add1(x: Binary{8}) => Binary{8} = x;"),
            deps: BTreeMap::new(),
        },
    );
    graph.insert(
        "root".to_owned(),
        PhylumNode {
            phylum_hash: fixture_hash(0xD),
            phylum: phy("phylum root\nnodule use_it;\nuse leafdep::math.add1;\n\
                 pub fn go(y: Binary{8}) => Binary{8} = add1(y);"),
            deps: BTreeMap::from([("leafdep".to_owned(), "leaf".to_owned())]),
        },
    );

    let resolved = build_phyla_graph(&graph).expect("an acyclic graph resolves");
    assert_eq!(resolved.len(), 2, "both nodes resolved");
    let (root_env, root_phyla) = &resolved["root"];
    assert!(
        root_phyla.deps().contains_key("leafdep"),
        "the root's `Phyla` retains its resolved dependency"
    );
    let env = root_env
        .nodule(&mycelium_l1::ast::Path(vec!["use_it".to_owned()]))
        .unwrap();
    assert!(env.fn_decl("go").is_some());
}

#[test]
fn build_phyla_graph_refuses_an_edge_to_an_absent_node() {
    let mut graph = BTreeMap::new();
    graph.insert(
        "root".to_owned(),
        PhylumNode {
            phylum_hash: fixture_hash(0xE),
            phylum: phy("phylum root\nnodule n;\npub fn f() => Binary{8} = 0b0000_0000;"),
            deps: BTreeMap::from([("missing".to_owned(), "does-not-exist".to_owned())]),
        },
    );
    let err = build_phyla_graph(&graph).expect_err("an edge to an absent node must be refused");
    assert!(
        err.message.contains("unknown dependency") || err.message.contains("not present"),
        "got: {}",
        err.message
    );
}

// ---------------------------------------------------------------------------------------------
// DN-113 §7 (US-4): layering over the ONE canonical linker — a lightweight differential.
// ---------------------------------------------------------------------------------------------

/// `ResolvedPhylum::resolve`'s linked `Env` for a dependency carries the SAME fn/type names as
/// independently checking + linking that same phylum via the plain (pre-M-1060) `check_phylum` +
/// `PhylumEnv::link` path — i.e. resolving a phylum as a *dependency* does not go through some
/// alternate/parallel linker (DN-113 §7 US-4 / §9.6's own "no second resolver" self-test, as a
/// differential rather than a source-level argument).
#[test]
fn a_resolved_dependencys_linked_env_matches_the_plain_check_and_link_path() {
    let src = "phylum d\nnodule math;\npub fn add1(x: Binary{8}) => Binary{8} = x;\nfn helper() => Binary{8} = 0b0000_0001;";
    let via_resolved_phylum = resolved(src, 9);

    let via_plain = check_phylum(&phy(src)).expect("plain check_phylum succeeds");
    let via_plain_linked = via_plain.link().expect("plain link succeeds");

    let mut resolved_fns: Vec<&String> = via_resolved_phylum.env.fns.keys().collect();
    let mut plain_fns: Vec<&String> = via_plain_linked.fns.keys().collect();
    resolved_fns.sort();
    plain_fns.sort();
    assert_eq!(
        resolved_fns, plain_fns,
        "the SAME linker (`PhylumEnv::link`) produced both — no parallel resolver"
    );
}
