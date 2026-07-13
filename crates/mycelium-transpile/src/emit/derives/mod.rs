//! DN-136/P1-a — the derive-rule emit-hook axis (Alt B: a static per-axis handler table
//! generalizing the already-landed `prim_map::TABLE` registry, `prim_map.rs:140`).
//!
//! **The two-level guarantee this axis must NOT collapse (DN-136 §3 item 2 / §7, DN-128):**
//! 1. **Per-derived-impl atomicity lives in the *rule* (a row's `emit`).** `derive_show_impl`/
//!    `derive_init_impl` (now [`show::compose`]/[`init::compose`]) refuse the **whole** impl the
//!    moment any field is ineligible — never a partial impl. Moved verbatim; unchanged.
//! 2. **Per-derive independence across the set lives in the *driver*
//!    ([`crate::emit::lower_struct_derives`]), which this axis does NOT touch.** The driver still
//!    owns: the attribute/derive-list walk, routing each derive's [`DeriveOutcome`] to
//!    `impls`/`sub_gaps`/`unrecognized`, and the item-still-emits compose-eligible-sub-gap-the-
//!    rest orchestration. A row can add a NEW derive rule; it can never move that orchestration
//!    out of the driver (a build-blocking review check, DN-136 §8 point 2(e)).
//!
//! A row's `recognizes` matches the derive's **trait-path text** (e.g. `"Debug"`); `emit` gets a
//! [`DeriveCtx`] carrying everything the pre-refactor inline arms closed over (`ty_name`,
//! `field_types`, `is_generic`, and the matched `name` — needed because the `Clone`/`Copy` row
//! interpolates which of the two fired into its satisfied-no-op message, exactly as the
//! pre-refactor `"Clone" | "Copy"` arm did with its shared `name` binding).

use crate::gap::GapReason;

mod clone_copy;
mod init;
mod show;

/// Everything a derive row's `emit` needs — the pre-refactor inline arms' closed-over locals,
/// reified as one struct (DN-136 §2's row shape, adapted to this axis's per-row inputs).
pub struct DeriveCtx<'a> {
    pub ty_name: &'a str,
    pub field_types: &'a [String],
    pub is_generic: bool,
    /// The matched derive-path text (e.g. `"Debug"`, `"Clone"`) — the `Clone`/`Copy` row needs
    /// this to interpolate the fired name into its message, byte-identically to the pre-refactor
    /// `"Clone" | "Copy" => { ... "derive({name}) is a satisfied no-op ..." }` arm.
    pub name: &'a str,
}

/// A derive row's outcome — the three states [`crate::emit::lower_struct_derives`] (the driver)
/// already routed inline, now reified so a row and the driver agree on the shape.
pub enum DeriveOutcome {
    /// The derive composed to this impl text — the driver appends it to `impls`.
    Composed(String),
    /// Not a failure — a satisfied no-op (e.g. `Clone`/`Copy` under value semantics) — the driver
    /// records it as a `Category::DeriveSatisfied` note, no impl emitted.
    Satisfied(GapReason),
    /// The derive could not compose — the driver records it as a sub-gap.
    Gap(GapReason),
}

/// One derive-rule handler row.
pub struct DeriveHandler {
    /// Pure recognizer — does this row own this derive-path text?
    pub recognizes: fn(&str) -> bool,
    /// The lowering — composes an impl, records a satisfied no-op, or gaps.
    pub emit: fn(&DeriveCtx) -> DeriveOutcome,
    /// For `EXPLAIN`/diagnostics (G2).
    #[allow(dead_code)] // read by future EXPLAIN tooling, not yet consumed (DN-136 §2)
    pub slug: &'static str,
    /// The DN/M-id grounding this row (VR-5).
    #[allow(dead_code)] // read by future EXPLAIN tooling, not yet consumed (DN-136 §2)
    pub citation: &'static str,
}

/// The table. `Eq`/`Ord`/`Hash`/`PartialEq`/`PartialOrd` are deliberately NOT rows here (DN-128's
/// std-derive set does not yet build them — falls through to the driver's `unrecognized`
/// tracking, byte-identical to the pre-refactor `match`'s catch-all `other =>` arm). A future
/// derive leaf adds one file here + one append-only row — never touches
/// `lower_struct_derives` (DN-136's stated objective).
pub const TABLE: &[DeriveHandler] = &[show::ROW, init::ROW, clone_copy::ROW];

/// First-match-wins linear scan over [`TABLE`] (same shape as [`crate::prim_map::lookup`]).
/// `None` for any derive name not in the DN-128 standard-derive set this leaf builds — the
/// driver's `unrecognized` bucket covers it, unchanged.
#[must_use]
pub fn lookup(name: &str) -> Option<&'static DeriveHandler> {
    TABLE.iter().find(|row| (row.recognizes)(name))
}

/// One field's DN-128 derive-eligibility — shared by [`show`] and [`init`] (both rules gate on
/// the identical "Named user type, not a primitive repr" test; see each row's own doc for the
/// empirically-verified rationale). Moved verbatim from the former `emit.rs::field_derive_eligible`.
pub(super) fn field_derive_eligible(mapped_ty: &str) -> bool {
    if matches!(mapped_ty, "Bool" | "Float" | "Bytes") {
        return false;
    }
    if mapped_ty.contains(['{', '(', '[']) {
        return false;
    }
    mapped_ty
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_uppercase())
}
