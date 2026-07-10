//! **Per-constructor visibility seal** (M-1027 / ENB-4; DN-104) integration tests.
//!
//! A `priv`-marked constructor of a `pub type` exports the type **NAME** (usable cross-nodule in
//! signatures, `use`, and pattern position) but **withholds the constructor from cross-nodule
//! CONSTRUCTION** — the FR-N3 capability-gate ("only the home nodule mints one"). These tests are the
//! Rust-oracle **differential witnesses** the `/myc-dogfood` dual pairs with:
//!
//! - **home-construct OK** — a sealed ctor is constructible in its home nodule;
//! - **foreign-construct REFUSED** — constructing it from another nodule is a never-silent `CheckError`;
//! - **cross-nodule type-use OK** — the type NAME crosses (signatures) and pattern-matching is permitted.
//!
//! Every "the seal fires" test is paired with a **control** proving the check is not vacuous (the same
//! shape, minus the seal, is accepted). Honesty (VR-5): the seal is a `Declared` capability-gate whose
//! *never-silent behavior* these tests pin — not a proof.

use mycelium_l1::{check_nodule, check_phylum, parse as parse_nodule, parse_phylum, CheckError};

/// Parse + check a phylum source, returning the per-nodule envs.
fn check_phy(src: &str) -> Result<mycelium_l1::PhylumEnv, CheckError> {
    let ph = parse_phylum(src).expect("parses as a phylum");
    check_phylum(&ph)
}

/// Parse + check a phylum, expecting a never-silent `CheckError`; returns its message.
fn phy_err(src: &str) -> String {
    let ph = parse_phylum(src).expect("parses as a phylum");
    check_phylum(&ph).expect_err("must fail to check").message
}

// ---------------------------------------------------------------------------------------------
// Surface: `priv` parses + round-trips.
// ---------------------------------------------------------------------------------------------

#[test]
fn a_sealed_ctor_parses_and_the_seal_round_trips_through_expand() {
    // `priv` before a ctor name parses; the AST carries `sealed`; `expand_to_source` re-emits `priv`.
    let src = "nodule a;\npub type T = priv Mk(Binary{8});\npub fn mk(x: Binary{8}) => T = Mk(x);";
    let nod = parse_nodule(src).expect("parses");
    let td = nod
        .items
        .iter()
        .find_map(|i| match i {
            mycelium_l1::ast::Item::Type(td) => Some(td),
            _ => None,
        })
        .expect("has a type decl");
    assert!(td.ctors[0].sealed, "the `priv` marker sets Ctor.sealed");

    let rendered = mycelium_l1::expand_to_source(&nod);
    assert!(
        rendered.contains("priv Mk"),
        "the seal round-trips through expand_to_source; got:\n{rendered}"
    );
    // And the re-parsed form still carries the seal (parse → expand → parse is stable).
    let reparsed = parse_nodule(&rendered).expect("re-parses");
    let td2 = reparsed
        .items
        .iter()
        .find_map(|i| match i {
            mycelium_l1::ast::Item::Type(td) => Some(td),
            _ => None,
        })
        .expect("has a type decl");
    assert!(td2.ctors[0].sealed, "the seal survives the round-trip");
}

// ---------------------------------------------------------------------------------------------
// Home-construct OK (accept).
// ---------------------------------------------------------------------------------------------

#[test]
fn a_sealed_ctor_is_constructible_in_its_home_nodule() {
    // `Mk` is `priv`, but the home nodule `a` mints one freely — the seal only withholds *foreign*
    // construction (own decls are subtracted from the withheld set; DN-104 §4).
    check_phy("phylum p\nnodule a;\npub type T = priv Mk(Binary{8});\npub fn mk(x: Binary{8}) => T = Mk(x);")
        .expect("a home-nodule construction of a sealed ctor type-checks");
}

#[test]
fn a_sealed_ctor_is_constructible_in_its_home_phylum_of_one() {
    // A bare nodule (phylum-of-one) has no imports, so the withheld set is empty — construction OK.
    check_nodule(
        &parse_nodule(
            "nodule solo;\npub type T = priv Mk(Binary{8});\nfn mk(x: Binary{8}) => T = Mk(x);",
        )
        .expect("parses"),
    )
    .expect("a phylum-of-one home construction type-checks");
}

// ---------------------------------------------------------------------------------------------
// Foreign-construct REFUSED (reject) + the unsealed control.
// ---------------------------------------------------------------------------------------------

#[test]
fn a_foreign_nodule_constructing_a_sealed_ctor_is_refused_never_silently() {
    // `b` imports `T` and tries to forge a `Mk` — the never-silent capability-gate refusal (G2).
    let err = phy_err(
        "phylum p\nnodule a;\npub type T = priv Mk(Binary{8});\nnodule b;\nuse a.T;\nfn forge(x: Binary{8}) => T = Mk(x);",
    );
    assert!(
        err.contains("priv") && err.contains("cross-nodule construction"),
        "the seal refusal names the withheld construction; got: {err}"
    );
}

#[test]
fn the_unsealed_control_lets_the_foreign_nodule_construct() {
    // Same shape, minus the seal: an UNSEALED `Mk` IS constructible cross-nodule — proves the seal
    // refusal above is not vacuous (the only difference is the `priv` marker).
    check_phy(
        "phylum p\nnodule a;\npub type T = Mk(Binary{8});\nnodule b;\nuse a.T;\nfn forge(x: Binary{8}) => T = Mk(x);",
    )
    .expect("an unsealed ctor constructs cross-nodule (the control)");
}

// ---------------------------------------------------------------------------------------------
// Cross-nodule type-use + pattern-match OK (the NAME crosses; only construction is withheld).
// ---------------------------------------------------------------------------------------------

#[test]
fn the_sealed_types_name_is_usable_cross_nodule_in_a_signature() {
    // `b` imports `T` and uses it as a parameter + return type — no construction, so it type-checks
    // (the seal withholds construction, never the type NAME; DN-104 §4).
    check_phy("phylum p\nnodule a;\npub type T = priv Mk(Binary{8});\nnodule b;\nuse a.T;\nfn passthrough(x: T) => T = x;")
        .expect("the sealed type's name crosses in a signature");
}

#[test]
fn a_foreign_nodule_may_pattern_match_a_sealed_ctor() {
    // Pattern position is permitted (destructuring reveals the field but cannot forge a new value —
    // the capability property is unforgeability, not opacity; DN-104 §4). `b` receives a `T` and reads
    // its field via `match` without ever constructing one.
    check_phy(
        "phylum p\nnodule a;\npub type T = priv Mk(Binary{8});\nnodule b;\nuse a.T;\nfn peek(x: T) => Binary{8} = match x { Mk(v) => v };",
    )
    .expect("pattern-matching a sealed ctor cross-nodule is permitted");
}

// ---------------------------------------------------------------------------------------------
// Redundant seal on a non-`pub` type → never-silent refusal.
// ---------------------------------------------------------------------------------------------

#[test]
fn priv_on_a_non_pub_type_is_a_redundant_seal_refusal() {
    // A nodule-private type is already unimportable, so a `priv` ctor is redundant — refuse it (G2),
    // rather than accept a silent no-op marker.
    let err = check_nodule(
        &parse_nodule(
            "nodule solo;\ntype T = priv Mk(Binary{8});\nfn mk(x: Binary{8}) => T = Mk(x);",
        )
        .expect("parses"),
    )
    .expect_err("must refuse the redundant seal")
    .message;
    assert!(
        err.contains("redundant") && err.contains("priv"),
        "the redundant-seal refusal is explicit; got: {err}"
    );
}

// ---------------------------------------------------------------------------------------------
// `priv` inside an `object` body → never-silent parse refusal (seal scoped to `type`).
// ---------------------------------------------------------------------------------------------

#[test]
fn priv_in_an_object_body_is_a_parse_refusal() {
    let err = parse_nodule("nodule a;\npub object Cell { priv Cell(Binary{8}); }")
        .expect_err("must refuse `priv` in an object body")
        .message;
    assert!(
        err.contains("priv") && err.contains("object"),
        "the object-body seal refusal is explicit; got: {err}"
    );
}

// ---------------------------------------------------------------------------------------------
// A per-ctor subset seal: one sealed ctor withheld, a sibling unsealed ctor free.
// ---------------------------------------------------------------------------------------------

#[test]
fn a_multi_ctor_type_seals_only_the_marked_ctor() {
    // `Open` is free cross-nodule; `Closed` is withheld — the seal is per-constructor (DN-104 §2).
    check_phy(
        "phylum p\nnodule a;\npub type T = Open(Binary{8}) | priv Closed(Binary{8});\nnodule b;\nuse a.T;\nfn ok(x: Binary{8}) => T = Open(x);",
    )
    .expect("the unsealed sibling ctor constructs cross-nodule");

    let err = phy_err(
        "phylum p\nnodule a;\npub type T = Open(Binary{8}) | priv Closed(Binary{8});\nnodule b;\nuse a.T;\nfn forge(x: Binary{8}) => T = Closed(x);",
    );
    assert!(
        err.contains("priv") && err.contains("Closed"),
        "the sealed sibling ctor is withheld cross-nodule; got: {err}"
    );
}

// ---------------------------------------------------------------------------------------------
// KNOWN GAP (pinned, not silently absent — G2/VR-5): a same-named local shadow bypasses the seal.
// ---------------------------------------------------------------------------------------------

#[test]
fn known_gap_a_same_named_local_shadow_type_bypasses_the_seal() {
    // Mycelium resolves types by BARE NAME, re-resolved in the *calling* nodule's own scope
    // (`resolve_ty` looks the name up in the caller's `Cx.types`, not the callee's declaring
    // scope), and "local decl shadows import" is the pre-existing, documented precedence rule
    // (RFC-0006 §4.3 / M-662). So a foreign nodule can declare its OWN unsealed type of the
    // SAME NAME — never importing the real sealed `a.T` — and the checker accepts passing a
    // value of its local decoy `T` to a function that expects `a.T`, because both resolve to the
    // bare name "T" in their respective scopes. This falsifies the "unforgeable capability-gate"
    // framing this feature was built to deliver (DN-99 row #37 / FR-N3 / M-1023's `Approx::proven`
    // port) for anything but a well-behaved caller that goes through `use a.T` — it is NOT a
    // security/capability boundary against an adversarial or even accidentally-colliding same-
    // named local declaration. Pinned here so the gap is visible in the differential suite, not
    // silently absent from what the PR's "silent-hole sweep" claimed was exhaustive coverage
    // (found in review, verified by reproduction — house rule #4: no claim upgraded past a
    // checked basis). Tracked as **M-1036** (nodule-qualified type identity — the real fix; DN-104
    // §6). Until M-1036 lands, `priv` is an opt-in API-discipline nudge for well-behaved cross-
    // nodule callers, NOT an enforced capability boundary — DN-104 must not be ratified with the
    // stronger claim.
    let result = check_phy(
        "phylum p\nnodule a;\npub type T = priv Mk(Binary{8});\npub fn use_t(x: T) => Binary{8} = match x { Mk(v) => v };\nnodule b;\nuse a.use_t;\ntype T = Mk(Binary{8});\nfn forge() => T = Mk(0b00000000);\npub fn exploit() => Binary{8} = use_t(forge());",
    );
    assert!(
        result.is_ok(),
        "this PINS the current (unsound) behavior — if this now returns Err, the bypass has been \
         fixed (e.g. by M-1036 landing nodule-qualified type identity); update this test to assert \
         the refusal instead and close M-1036. Got: {result:?}"
    );
}
