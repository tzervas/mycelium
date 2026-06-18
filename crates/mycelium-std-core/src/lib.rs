//! `std.core` — Ring-0 prelude: the honest value model, re-exported (M-515).
//!
//! The thin Ring-0 surface every other stdlib module imports to talk about values
//! honestly. It **re-exports** `mycelium-core`'s value model (RFC-0001) — `Value`,
//! `Repr`, `Meta`, the runtime sum `CoreValue`/`Datum`, the `GuaranteeStrength`
//! lattice (`Exact ⊐ Proven ⊐ Empirical ⊐ Declared`), the `Bound`/`BoundBasis`
//! companion, and the kernel's content-identity type — plus a thin §4.8 *query
//! surface* over them.
//!
//! Honesty crux (inherited, not invented): the kernel forbids a silent `Repr`
//! change and a spurious guarantee upgrade, and `std.core` is where that floor is
//! *named* for the whole library. It is Ring 0 and adds **no trusted code** (KC-3):
//! every re-export resolves to an `mycelium-core` (M-101) item, and the query
//! functions below are pure, total delegations to kernel accessors — never an
//! approximation or a selection of their own.
//!
//! Design spec: `docs/spec/stdlib/core.md`; contract: RFC-0016 §4.1 (C1–C6);
//! guarantee matrix: §4.5 (every row `Exact`/total — the honest floor for a
//! no-accuracy re-export surface).
//!
//! Scope note (boundary, spec §2): `std.core` exposes the *types* and the read-only
//! query surface; the *verbs* live elsewhere — representation change is `std.swap`
//! (M-516), ε/δ numeric helpers `std.numerics` (M-512), `Option`/`Result`
//! *combinators* `std.error` (M-527), content-addressing *as a library*
//! `std.content` (M-523).
#![forbid(unsafe_code)]

// ---- value model re-exports (RFC-0001 §4.1–§4.3) ---------------------------------
pub use mycelium_core::bound::{Bound, BoundBasis, BoundKind, NormKind};
pub use mycelium_core::datum::{CoreValue, Datum};
pub use mycelium_core::guarantee::GuaranteeStrength;
pub use mycelium_core::id::ContentHash;
pub use mycelium_core::meta::{Meta, PackScheme, PhysicalLayout, Provenance, SparsityObs};
pub use mycelium_core::repr::{Repr, ScalarKind, SparsityClass};
pub use mycelium_core::value::{Payload, Trit, Value};

/// The curated default prelude (spec §3 / FLAG Q1). `use mycelium_std_core::prelude::*;`
/// brings the value model, the lattice tags, and the query surface into scope. The
/// final membership is a ratification call (RFC-0016 §8-Q3); this is the proposed
/// minimal set, kept consistent across the module specs by the orchestrator.
pub mod prelude {
    pub use super::{
        bound_of, guarantee_of, meta_of, provenance_of, repr_of, Bound, BoundBasis, BoundKind,
        CoreValue, Datum, GuaranteeStrength, Meta, NormKind, Payload, Provenance, Repr, Trit,
        Value,
    };
}

// ---- §4.8 runtime query surface (inspectability; spec §3/§4) ---------------------
//
// These are thin, pure, *total* delegations to the kernel's own accessors. They are
// honest by construction:
//   * `repr_of` / `meta_of` return `Option` — a `CoreValue::Data` (an algebraic
//     `Datum`) has no `Repr`/`Meta`, so the absence is reported explicitly (C1
//     never-silent), never a fabricated default.
//   * `guarantee_of` is total: every `CoreValue` carries a guarantee (a `Datum`'s is
//     the meet-summary of its fields).
//   * `bound_of` / `provenance_of` follow `meta_of` and so are `Option` too.
//
// None of these *select*, *convert*, or *approximate*, so each is `Exact` (C2) and
// has nothing of its own to EXPLAIN (C3) — they are the window through which a
// *downstream* op's tag/bound/provenance is inspected (RFC-0001 §4.8).

/// The representation of `v`, or `None` if `v` is algebraic data (no `Repr`).
#[must_use]
pub fn repr_of(v: &CoreValue) -> Option<&Repr> {
    v.as_repr().map(Value::repr)
}

/// The metadata of `v`, or `None` if `v` is algebraic data (no `Meta`).
#[must_use]
pub fn meta_of(v: &CoreValue) -> Option<&Meta> {
    v.as_repr().map(Value::meta)
}

/// The guarantee tag of `v` (total — every value carries one).
#[must_use]
pub fn guarantee_of(v: &CoreValue) -> GuaranteeStrength {
    v.guarantee()
}

/// The bound attached to `v`, or `None` when there is no metadata or no bound.
#[must_use]
pub fn bound_of(v: &CoreValue) -> Option<&Bound> {
    meta_of(v).and_then(Meta::bound)
}

/// The provenance of `v`, or `None` if `v` is algebraic data (no `Meta`).
#[must_use]
pub fn provenance_of(v: &CoreValue) -> Option<&Provenance> {
    meta_of(v).map(Meta::provenance)
}

// ---- guarantee matrix, as checked data (RFC-0016 §4.5) ---------------------------

/// One row of the module guarantee matrix (RFC-0016 §4.5): an exported item, its
/// honest guarantee tag, whether it is fallible (and the explicit error shape), its
/// declared effects, and whether it exposes an EXPLAIN artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GuaranteeRow {
    /// The exported op / item name.
    pub op: &'static str,
    /// Its honest guarantee tag on the lattice `Exact ⊐ Proven ⊐ Empirical ⊐ Declared`.
    pub tag: GuaranteeStrength,
    /// The explicit fallibility: `"total"`, or the `Option`/`Result` shape returned.
    pub fallibility: &'static str,
    /// Declared effects (`"none"` for this pure re-export surface).
    pub effects: &'static str,
    /// Whether the item surfaces an inspectable EXPLAIN artifact.
    pub explainable: bool,
}

/// The `std.core` guarantee matrix (spec §4). Every row is `Exact` and effect-free:
/// `std.core` introduces no operation that selects, converts, or approximates, so the
/// honest tag for each is `Exact` (RFC-0016 §4.1 C2) — the honest *floor*, not an
/// upgrade. The query rows that surface a downstream value's own tag/bound/provenance
/// are marked EXPLAIN-able (they are the inspection window, RFC-0001 §4.8).
pub const GUARANTEE_MATRIX: &[GuaranteeRow] = &[
    GuaranteeRow {
        op: "Value/Repr/Meta (type re-exports)",
        tag: GuaranteeStrength::Exact,
        fallibility: "total",
        effects: "none",
        explainable: false,
    },
    GuaranteeRow {
        op: "CoreValue/Datum (type re-exports)",
        tag: GuaranteeStrength::Exact,
        fallibility: "total",
        effects: "none",
        explainable: false,
    },
    GuaranteeRow {
        op: "GuaranteeStrength (lattice tags)",
        tag: GuaranteeStrength::Exact,
        fallibility: "total",
        effects: "none",
        explainable: false,
    },
    GuaranteeRow {
        op: "Bound/BoundBasis (type re-exports)",
        tag: GuaranteeStrength::Exact,
        fallibility: "total",
        effects: "none",
        explainable: false,
    },
    GuaranteeRow {
        op: "repr_of",
        tag: GuaranteeStrength::Exact,
        fallibility: "Option<&Repr> (None for algebraic data)",
        effects: "none",
        explainable: false,
    },
    GuaranteeRow {
        op: "meta_of",
        tag: GuaranteeStrength::Exact,
        fallibility: "Option<&Meta> (None for algebraic data)",
        effects: "none",
        explainable: false,
    },
    GuaranteeRow {
        op: "guarantee_of",
        tag: GuaranteeStrength::Exact,
        fallibility: "total",
        effects: "none",
        explainable: true,
    },
    GuaranteeRow {
        op: "bound_of",
        tag: GuaranteeStrength::Exact,
        fallibility: "Option<&Bound> (None when no meta/bound)",
        effects: "none",
        explainable: true,
    },
    GuaranteeRow {
        op: "provenance_of",
        tag: GuaranteeStrength::Exact,
        fallibility: "Option<&Provenance> (None for algebraic data)",
        effects: "none",
        explainable: true,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use mycelium_core::data::CtorRef;

    fn exact_repr_value() -> CoreValue {
        let meta = Meta::exact(Provenance::Root);
        let v = Value::new(
            Repr::Binary { width: 2 },
            Payload::Bits(vec![true, false]),
            meta,
        )
        .expect("well-formed binary value");
        CoreValue::Repr(v)
    }

    fn nil_datum() -> CoreValue {
        let ctor = CtorRef::new(ContentHash::parse("blake3:nil").expect("hash"), 0);
        CoreValue::Data(Datum::new(ctor, vec![]))
    }

    #[test]
    fn matrix_is_all_exact_and_effect_free() {
        // Spec §4: every row of the Ring-0 re-export surface is the honest `Exact`
        // floor and declares no effects. This guards against an accidental overclaim
        // (a `Proven`/`Empirical` tag here would itself violate VR-5).
        assert_eq!(GUARANTEE_MATRIX.len(), 9, "spec §4 lists nine rows");
        for row in GUARANTEE_MATRIX {
            assert_eq!(
                row.tag,
                GuaranteeStrength::Exact,
                "{} must be Exact",
                row.op
            );
            assert_eq!(row.effects, "none", "{} must be effect-free", row.op);
        }
    }

    #[test]
    fn only_query_rows_are_explainable() {
        // The EXPLAIN window is exactly the value-tag/bound/provenance queries.
        let explainable: Vec<&str> = GUARANTEE_MATRIX
            .iter()
            .filter(|r| r.explainable)
            .map(|r| r.op)
            .collect();
        assert_eq!(explainable, ["guarantee_of", "bound_of", "provenance_of"]);
    }

    #[test]
    fn queries_on_a_repr_value_are_present() {
        let v = exact_repr_value();
        assert_eq!(repr_of(&v), Some(&Repr::Binary { width: 2 }));
        assert!(meta_of(&v).is_some());
        assert_eq!(guarantee_of(&v), GuaranteeStrength::Exact);
        assert_eq!(provenance_of(&v), Some(&Provenance::Root));
        assert_eq!(bound_of(&v), None); // an exact value carries no bound
    }

    #[test]
    fn queries_on_algebraic_data_report_absence_never_silently() {
        // C1 never-silent: a Datum has no Repr/Meta; the queries say so explicitly
        // with `None` rather than fabricating a default.
        let d = nil_datum();
        assert_eq!(repr_of(&d), None);
        assert_eq!(meta_of(&d), None);
        assert_eq!(bound_of(&d), None);
        assert_eq!(provenance_of(&d), None);
        // guarantee_of stays total even for data.
        let _g = guarantee_of(&d);
    }

    #[test]
    fn lattice_meet_never_upgrades() {
        // Sanity re-check of the re-exported floor: composition cannot strengthen.
        use GuaranteeStrength::{Declared, Empirical, Exact, Proven};
        for a in [Exact, Proven, Empirical, Declared] {
            for b in [Exact, Proven, Empirical, Declared] {
                let m = a.meet(b);
                assert!(m.rank() >= a.rank().max(b.rank()));
            }
        }
    }
}
