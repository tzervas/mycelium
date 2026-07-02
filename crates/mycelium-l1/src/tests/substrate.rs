//! White-box tests for the `Substrate` v0 value form (M-902; DN-71 Model S §4.1) and its M-903
//! runtime use-once backstop (DN-71 §4.2).
//!
//! Covers the M-902 Definition of Done: creation (acquisition + provenance), passage (round-trip
//! through the evaluator's value-binding machinery), inspection (tag/id/provenance/EXPLAIN); the
//! M-903 `try_consume` runtime backstop (first-consume succeeds, a second consume — even through a
//! separate clone of the same identity — traps explicitly, never silently); and the surface
//! `consume` refusal, honestly naming the M-904 lowering seam that is still staged.

use std::collections::BTreeMap;

use mycelium_core::DataRegistry;

use crate::checkty::{check_nodule, Env};
use crate::eval::{Evaluator, L1Error, L1Value};
use crate::parse;
use crate::substrate::{SubstrateError, SubstrateHandle, SubstrateProvenance};

fn env(src: &str) -> Env {
    check_nodule(&parse(src).expect("parses")).expect("checks")
}

fn a_handle(tag: &str) -> SubstrateHandle {
    SubstrateHandle::acquire(tag, SubstrateProvenance::new("wild:open", "test-fixture"))
}

// --- creation + inspection ---------------------------------------------------------------

#[test]
fn acquire_records_tag_and_provenance() {
    let h = a_handle("Socket");
    assert_eq!(h.tag(), "Socket");
    assert_eq!(h.provenance().acquired_via, "wild:open");
    assert_eq!(h.provenance().site, "test-fixture");
}

#[test]
fn explain_is_inspectable_and_names_the_provenance() {
    // House rule 2 (no black boxes): the EXPLAIN string surfaces tag + identity + provenance.
    let h = a_handle("Socket");
    let e = h.explain();
    assert!(e.contains("Substrate{Socket}"), "got: {e}");
    assert!(e.contains(&format!("#{}", h.id())), "got: {e}");
    assert!(e.contains("wild:open"), "got: {e}");
    assert!(e.contains("test-fixture"), "got: {e}");
}

#[test]
fn each_acquisition_has_a_distinct_identity() {
    // Identity is the external resource, not its content: two acquisitions of the "same" tag are
    // two distinct handles (DN-71 §4.1 — ADR-003 content-addressing does not apply).
    let a = a_handle("Socket");
    let b = a_handle("Socket");
    assert_ne!(
        a.id(),
        b.id(),
        "distinct acquisitions must have distinct ids"
    );
    assert_ne!(a, b, "distinct-identity handles must not be equal");
}

#[test]
fn clone_preserves_identity_the_passage_semantics() {
    // Cloning is *passage* (same resource), not a second acquisition: the id is preserved, so a
    // cloned handle is equal to its source. This is the mechanism the evaluator uses to pass a
    // Substrate through a binding; affinity is a checker property, not a non-Clone Rust bound.
    let a = a_handle("Socket");
    let b = a.clone();
    assert_eq!(a.id(), b.id());
    assert_eq!(a, b);
}

// --- L1Value discrimination --------------------------------------------------------------

#[test]
fn as_substrate_and_as_repr_are_discriminated() {
    let h = a_handle("Socket");
    let sv = L1Value::Substrate(h.clone());
    // A Substrate value inspects as a handle and has no repr (never-silent None).
    assert_eq!(sv.as_substrate(), Some(&h));
    assert!(sv.as_repr().is_none());
    // A non-Substrate value has no handle (never-silent None). A data value stands in for "other".
    let dv = L1Value::Data {
        ty: "Unit".into(),
        ctor: "Unit".into(),
        fields: vec![],
    };
    assert!(dv.as_substrate().is_none());
}

#[test]
fn substrate_has_no_l0_core_projection() {
    // A Substrate handle is not a kernel value (no Repr, no L0 node), so it has no CoreValue
    // projection — `to_core` is honestly `None`, never a fabricated lowering (DN-71 §4.1; G2).
    let env = env("nodule d;");
    let registry = DataRegistry::build(&BTreeMap::new()).expect("empty registry builds");
    let sv = L1Value::Substrate(a_handle("Socket"));
    assert!(sv.to_core(&env, &registry).is_none());
}

// --- passage through the evaluator (create -> pass -> inspect round-trip) -----------------

#[test]
fn substrate_passes_through_the_evaluator_unchanged() {
    // A passthrough fn binds a Substrate param and returns it (no `consume`): the handle rides the
    // ordinary value-binding machinery and comes back out identical (same tag + id + provenance).
    let env = env("nodule d;\nfn passthrough(s: Substrate{Res}) => Substrate{Res} = s;");
    let h = a_handle("Res");
    let out = Evaluator::new(&env)
        .call("passthrough", vec![L1Value::Substrate(h.clone())])
        .expect("passthrough evaluates");
    let got = out.as_substrate().expect("result is a Substrate handle");
    assert_eq!(got, &h, "the handle round-trips unchanged (same identity)");
}

// --- M-903: the runtime use-once backstop -------------------------------------------------

#[test]
fn try_consume_succeeds_once_and_marks_the_handle_consumed() {
    // The first move (DN-71 §4.2) succeeds and flips the shared `consumed` flag.
    let h = a_handle("Socket");
    assert!(!h.is_consumed(), "a freshly acquired handle starts live");
    let moved = h.try_consume().expect("first consume succeeds");
    assert_eq!(moved.tag(), "Socket");
    assert_eq!(moved.id(), h.id(), "the move preserves identity");
    assert!(h.is_consumed(), "the original observes the shared flag too");
    assert!(moved.is_consumed());
}

#[test]
fn try_consume_traps_a_double_consume_naming_the_tag_and_id() {
    // A second `try_consume` on the same identity is the never-silent runtime backstop tripping
    // (M-903; DN-71 §4.2) — it names both the tag and the id of the violated handle, never a silent
    // no-op or a fabricated second move (G2/VR-5).
    let h = a_handle("Socket");
    h.try_consume().expect("first consume succeeds");
    let err = h
        .try_consume()
        .expect_err("a second consume of the same identity must trap");
    assert_eq!(
        err,
        SubstrateError::AlreadyConsumed {
            tag: "Socket".into(),
            id: h.id(),
        }
    );
    let msg = err.to_string();
    assert!(msg.contains("double-consume"), "got: {msg}");
    assert!(msg.contains("Socket"), "must name the tag: {msg}");
    assert!(msg.contains(&h.id().to_string()), "must name the id: {msg}");
    assert!(msg.contains("M-903"), "must cite the backstop: {msg}");
}

#[test]
fn the_consumed_flag_is_shared_across_clones_the_backstop_cannot_be_dodged() {
    // Cloning is passage, not re-acquisition (the type doc's contract) — consuming through ONE
    // clone must be visible through every other clone of the same identity, so the backstop cannot
    // be evaded by cloning before the second (illegitimate) consume.
    let a = a_handle("Socket");
    let b = a.clone();
    b.try_consume().expect("consume via the clone succeeds");
    assert!(a.is_consumed(), "the flag is shared, not per-clone");
    let err = a
        .try_consume()
        .expect_err("consuming the other clone must also trap");
    assert!(matches!(err, SubstrateError::AlreadyConsumed { .. }));
}

#[test]
fn distinct_identities_are_consumed_independently() {
    // Two separate acquisitions (distinct ids) have independent `consumed` flags — consuming one
    // never affects the other (identity, not tag, is what the backstop keys on).
    let a = a_handle("Socket");
    let b = a_handle("Socket");
    a.try_consume().expect("consuming `a` succeeds");
    assert!(a.is_consumed());
    assert!(!b.is_consumed(), "a distinct identity stays live");
    b.try_consume()
        .expect("consuming `b` independently succeeds");
}

// --- the still-staged M-904 lowering seam --------------------------------------------------

#[test]
fn surface_consume_is_an_explicit_refusal_naming_the_m904_lowering_seam() {
    // `consume s` type-checks (M-664 surface) and its affine discipline is now statically checked
    // (M-903), but the evaluator still has no execution path for the actual move — that lowering is
    // M-904 (DN-71 §4.3). The refusal is explicit, never silent (G2), and honestly describes M-903 as
    // landed rather than staged.
    let env = env("nodule d;\nfn take(s: Substrate{Sock}) => Substrate{Sock} = consume s;");
    let err = Evaluator::new(&env)
        .call("take", vec![L1Value::Substrate(a_handle("Sock"))])
        .expect_err("surface consume execution must still refuse in v0");
    let L1Error::Unsupported { what, .. } = err else {
        panic!("expected Unsupported, got {err:?}");
    };
    assert!(what.contains("M-903"), "refusal must cite M-903: {what}");
    assert!(what.contains("M-904"), "refusal must name M-904: {what}");
}

#[test]
fn guarantee_index_on_a_substrate_is_refused() {
    // A Substrate carries no Meta/guarantee tag, so a guarantee-index assertion on it is an explicit
    // refusal (never a silently-passed assertion — VR-5/G2).
    let env = env("nodule d;");
    let err = Evaluator::new(&env)
        .assert_guarantee(
            "site",
            &L1Value::Substrate(a_handle("Socket")),
            crate::ast::Strength::Exact,
        )
        .expect_err("a guarantee index on a Substrate must be refused");
    assert!(matches!(err, L1Error::Unsupported { .. }), "got {err:?}");
}
