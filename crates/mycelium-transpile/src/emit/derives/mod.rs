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
mod eq;
mod hash;
mod init;
mod ord;
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

/// The table. **DN-136 Phase-2 (DERIVE-COMPLETION) update (append-only, additive):** `PartialEq`/
/// `PartialOrd`/`Hash` now have rows ([`eq`]/[`ord`]/[`hash`]); bare `Eq`/`Ord` are DELIBERATELY
/// still NOT recognized (each new row's own module doc explains why: recognizing both names in a
/// derive-list would invoke that row's `emit` twice for one struct, composing a duplicate
/// fn/impl the real toolchain refuses — `PartialEq`/`PartialOrd` are the reliable co-occurring
/// signal Rust's own `Eq: PartialEq`/`Ord: Eq + PartialOrd` supertrait bounds guarantee). A solo
/// `#[derive(Eq)]`/`#[derive(Ord)]` (invalid Rust on its own, syntactically representable but
/// never rustc-accepted) still falls through to the driver's `unrecognized` tracking, exactly as
/// every other never-built name does — this is unchanged from the pre-Phase-2 catch-all behavior.
/// A future derive leaf adds one file here + one append-only row — never touches
/// `lower_struct_derives` (DN-136's stated objective).
pub const TABLE: &[DeriveHandler] = &[
    show::ROW,
    init::ROW,
    clone_copy::ROW,
    eq::ROW,
    ord::ROW,
    hash::ROW,
];

/// First-match-wins linear scan over [`TABLE`] (same shape as [`crate::prim_map::lookup`]).
/// `None` for any derive name not in the DN-128 standard-derive set this leaf builds — the
/// driver's `unrecognized` bucket covers it, unchanged.
#[must_use]
pub fn lookup(name: &str) -> Option<&'static DeriveHandler> {
    TABLE.iter().find(|row| (row.recognizes)(name))
}

/// One struct field's DN-138 §4.5 derive-composition classification — shared by all five
/// field-gating rows ([`show`]/[`init`]/[`ord`]/[`eq`]/[`hash`]; [`clone_copy`] does not gate).
/// **Replaces the former boolean `field_derive_eligible`** (DN-136 P1-a): a classification, not a
/// `bool`, because DN-138 §3's heterogeneity finding means the SAME primitive kind composes
/// differently depending on the row — `Show`/`Init`/`Ord3` dispatch through a resolvable TRAIT
/// INSTANCE (`crates/mycelium-l1/src/checkty.rs`'s `PRELUDE_INSTANCE_SEEDS`), while `PartialEq`/
/// `Hash` route directly to an already-landed PRIM (`eq`/`bytes_eq`/`hash.blake3`) — a bare `bool`
/// cannot express that distinction; each row's own `compose` routes per kind (see each row's doc).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FieldDeriveKind {
    /// A leading-uppercase, non-bracketed, non-primitive-repr name (a user-declared type; the
    /// pre-DN-138 boolean gate's sole `true` case). Composes exactly as before this DN — every row
    /// routes it through its own pre-existing user-type call shape (`render`/`init`/`cmp` trait
    /// dispatch; `eq_<Type>`/`hash_<Type>` deterministic nested-derive fn names).
    UserNamed,
    /// `Binary{N}` for some concrete width `N` (any width matches this KIND). DN-138 increment 1
    /// seeds a trait instance (`Show`/`Init`/`Ord3`) at exactly ONE concrete width, `Binary{64}`
    /// (§2 fact 1 — the width-erased coherence key admits at most one instance per head): a row
    /// that dispatches through that SEEDED INSTANCE must additionally gate on
    /// [`is_seeded_scalar_width`] before composing (a narrower/wider width is an honest, disclosed
    /// gap — increment 2, DN-138 §6). A row that routes to a PRIM instead (`eq` — `PartialEq`) has
    /// no such restriction: `eq`/`lt` are width-generic over any concrete `Binary{N}` (RFC-0032
    /// D1), so `PartialEq` composes over EVERY width, not just 64 (`Hash` still defers every width
    /// — no `Binary{N} -> Bytes` raw-byte prim exists yet, DN-138 §6).
    ScalarBinary,
    /// `Bytes` (mapped from a Rust `String`/`str`/`[u8]` field).
    BytesLike,
    /// `Bool`.
    BoolLike,
    /// `Float` — ineligible for every row (ADR-040 §2.3/§2.4): no `Show`/`Init`/`Ord3` instance is
    /// ever seeded for it, and a derived TOTAL `Eq`/`Ord` over a float field is refused (NaN has no
    /// order position, NaN != NaN) — `eq.rs`/`ord.rs` special-case this ahead of the classifier so
    /// their gap message cites the real (NaN/ADR-040) reason, not the generic no-route one.
    Float,
    /// `Seq`/`Vec[T]`, tuples, or any other bracketed shape this leaf does not resolve — deferred
    /// to increment 2 (WU-4, DN-138 §6). Also the fallback for a non-uppercase-leading,
    /// non-primitive name the pre-DN-138 boolean gate's implicit "else ineligible" branch covered
    /// (never silently reclassified as `UserNamed`).
    Deferred,
}

/// Classify one struct field's mapped Mycelium type for derive composition (DN-138 §4.5) — shared
/// by all five field-gating rows. See [`FieldDeriveKind`]'s own doc for why this replaces the
/// former `field_derive_eligible(&str) -> bool` (DN-136 P1-a).
#[must_use]
pub(super) fn field_derive_kind(mapped_ty: &str) -> FieldDeriveKind {
    if mapped_ty == "Float" {
        return FieldDeriveKind::Float;
    }
    if mapped_ty == "Bool" {
        return FieldDeriveKind::BoolLike;
    }
    if mapped_ty == "Bytes" {
        return FieldDeriveKind::BytesLike;
    }
    if mapped_ty.starts_with("Binary{") && mapped_ty.ends_with('}') {
        return FieldDeriveKind::ScalarBinary;
    }
    if mapped_ty.contains(['{', '(', '[']) {
        return FieldDeriveKind::Deferred;
    }
    if mapped_ty
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_uppercase())
    {
        return FieldDeriveKind::UserNamed;
    }
    FieldDeriveKind::Deferred
}

/// `true` iff `ft` is the ONE concrete `Binary{N}` width DN-138 increment 1 seeds a trait instance
/// at (`Binary{64}` — DN-138 §2 fact 1's width-erased coherence key: at most one `Show`/`Init`/
/// `Ord3` instance may exist per head `"Binary"`, and the real corpus's `u64` fields hit it
/// exactly). Used by the three TRAIT-DISPATCHED rows ([`show`], [`init`], [`ord`]) to gate a
/// `ScalarBinary` field beyond the classifier alone — a narrower/wider width is an honest,
/// disclosed gap (deferred to increment 2, DN-138 §6), never a silent width-mismatch `myc check`
/// failure at the emitted call site (`crate::checkty::Checker::require_instance`'s own
/// `info.for_ty == concrete` guard would refuse it anyway — this gate keeps that refusal at EMIT
/// time, an honest gap, rather than composing text a downstream `myc check` then rejects).
#[must_use]
pub(super) fn is_seeded_scalar_width(ft: &str) -> bool {
    ft == "Binary{64}"
}
