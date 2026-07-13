//! DN-128 (M-1086) `derive(PartialEq)`/`derive(Eq)` -> a structural field-wise equality fold —
//! DN-136 Phase-2 (DERIVE-COMPLETION) additive row over the frozen `emit/derives` axis (DN-136
//! P1-a). Composes real, `myc check`-clean equality code over the field-wise `cmp.eq` prim
//! (RFC-0032 D1) DN-128 §2 cites: "PartialEq/Eq = field-wise `cmp.eq` ∧-fold" over "landed
//! `cmp.eq`/`bytes.eq` prims".
//!
//! **Recognizes ONLY `"PartialEq"`, never `"Eq"` — a verified, disclosed, deliberate choice
//! (mitigation #14 verify-first).** `derives::lookup` is consulted independently PER derive-list
//! ENTRY by the driver (`lower_struct_derives`), so a struct written as the extremely common real
//! Rust `#[derive(PartialEq, Eq)]` would, if this row recognized BOTH names, have `emit` invoked
//! TWICE for the SAME struct — composing the identical `fn eq_<T>(...)` text twice, which the real
//! `myc-check` oracle refuses with `"duplicate function"` (empirically confirmed against a scratch
//! probe during this leaf's development; the analogous `impl Ord3[T] for T` shape in
//! [`super::ord`] would instead trip RFC-0019 §4.5's instance-uniqueness "overlapping instance"
//! refusal — same root cause, same fix). Recognizing only `"PartialEq"` sidesteps the collision by
//! construction: Rust's own `Eq: PartialEq` supertrait bound means valid Rust source never derives
//! bare `Eq` without `PartialEq` also present (in the same or a sibling `#[derive(...)]`), so
//! `PartialEq` is the reliable, always-co-occurring signal — a solo `#[derive(Eq)]` (invalid Rust,
//! syntactically representable but never emitted by rustc-accepted source) falls through to the
//! driver's honest `unrecognized` bucket rather than composing twice.
//!
//! **Composes a plain top-level `fn eq_<T>(a: T, b: T) => Binary{1} = ...;`, NOT
//! `impl Eq[T] for T` — a second disclosed, verified deviation from the naive DN-128-worklist
//! sketch.** Unlike [`super::show`]/[`super::init`] (whose `Show`/`Init` targets are landed
//! `PRELUDE_TRAIT_SEEDS` — `crates/mycelium-l1/src/checkty.rs:2282`), there is **no landed `Eq`
//! prelude trait** — only `Fuse`/`Ord3`/`Show`/`Init`/`Fault` are seeded. Composing
//! `impl Eq[T] for T` would therefore need this row to ALSO self-declare `trait Eq[T]{ fn
//! eq(a:T,b:T) => Binary{1}; }` inline in the emitted text (naming the method `eq` was tried first
//! and rejected: it SHADOWS the bare-call `eq` prim for every `eq(...)` call in the whole file,
//! including the ones between primitive-typed inner fields — confirmed empirically: `myc-check`
//! then refuses the inner field comparison with `"no instance Eq for Binary{8}"`; `equal`, the
//! `trait Eq<A> { fn equal(...) }` spelling RFC-0007 §4.4/RFC-0019 §3.1 already use as their
//! illustrative example, avoids the prim-name collision). But self-declaring the trait per-impl
//! ALSO fails the moment a SECOND struct in the same file derives `PartialEq` too (a real, common
//! shape — the landed `derive_composes_end_to_end_over_a_same_file_nested_derived_field` test
//! already exercises multi-struct-per-file derive composition for `Show`/`Init`): `myc-check`
//! refuses the second `trait Eq[T]{...}` with `"duplicate trait declaration"` (confirmed
//! empirically). `lower_struct_derives` calls a row's `emit` once per struct with no cross-call
//! state (a pure `fn(&DeriveCtx) -> DeriveOutcome`), so this row cannot deduplicate a shared
//! trait-decl preamble across multiple derive sites without driver changes — out of this leaf's
//! scope (DN-136 §7's "the driver's per-derive orchestration is NOT touched by a row" invariant).
//! A plain, deterministically-named top-level fn sidesteps all of this: no trait to (re)declare,
//! and the deterministic `eq_<FieldType>` name lets a NESTED eligible field's own derived
//! comparator resolve BY CONSTRUCTION — mirroring [`super::show`]'s `render(field)` compositional
//! call, without needing trait-based dispatch at all.
//!
//! **The ADR-040 Float/NaN refusal** fires FIRST, ahead of the general
//! [`field_derive_eligible`] gate (which also excludes `Float`, so this is currently redundant in
//! practice but kept as its own explicit, clearly-worded check per the DN-136 Phase-2 worklist's
//! L1 spec — "not just ineligible-repr fields" — so the emitted [`GapReason`] cites the REAL
//! (NaN/ADR-040) reason for a float field, not the generic no-ambient-instance one).
//!
//! Guarantee: `Empirical` (every emitted shape above is live-oracle-verified against the real
//! `myc-check` toolchain, `src/tests/emit.rs`'s `derive_forms_check_clean_against_real_toolchain`);
//! the field-eligibility heuristic itself stays `Declared` (same VR-5 boundary
//! [`super::show`]/[`super::init`] already carry — a nested field's OWN derive is not verified to
//! have actually succeeded, only that its type NAME has the right shape).

use super::{field_derive_eligible, DeriveCtx, DeriveHandler, DeriveOutcome};
use crate::gap::{Category, GapReason};

fn recognizes(name: &str) -> bool {
    name == "PartialEq"
}

/// The deterministic top-level fn name this row's compose emits/expects for a given type name —
/// shared between a struct's OWN emission and a nested eligible field's compositional call (no
/// cross-call state needed; both derive from `ty_name`/`field_type` alone).
fn eq_fn_name(ty_name: &str) -> String {
    format!("eq_{ty_name}")
}

/// Left-fold `parts` into a single `and(...)` chain — mirrors [`super::show`]'s
/// `bytes_concat_chain` shape, folding with the width-preserving `and` prim (`Binary{1} x
/// Binary{1} -> Binary{1}`, RFC-0032 D2) instead of `bytes_concat`. `parts` is never empty in the
/// caller below (the fieldless case is handled separately, without this helper).
fn and_chain(parts: &[String]) -> String {
    let mut iter = parts.iter();
    let mut acc = iter.next().cloned().unwrap_or_default();
    for p in iter {
        acc = format!("and({acc}, {p})");
    }
    acc
}

/// **Fieldless (unit) struct:** `fn eq_T(a: T, b: T) => Binary{1} = 0b1;` — always equal, always
/// succeeds (live-oracle-proven, `src/tests/emit.rs`). **Struct with fields:** an `and`-fold of
/// `eq_<FieldType>(p_i, q_i)` per field, gated per field via the ADR-040 float check (first) then
/// [`field_derive_eligible`] (same as [`super::show`]/[`super::init`]) — refuses the WHOLE derive
/// (never a partial/fabricated equality, G2) the moment any field is ineligible.
fn compose(ty_name: &str, field_types: &[String]) -> Result<String, GapReason> {
    let fname = eq_fn_name(ty_name);
    if field_types.is_empty() {
        return Ok(format!(
            "fn {fname}(a: {ty_name}, b: {ty_name}) => Binary{{1}} =\n    0b1;"
        ));
    }
    for (i, ft) in field_types.iter().enumerate() {
        if ft == "Float" {
            return Err(GapReason::new(
                Category::DeriveAttr,
                format!(
                    "struct `{ty_name}` derive(PartialEq): field {i} has type `Float` — a \
                     derived TOTAL equality over a float field is refused (ADR-040 §2.4 NaN \
                     semantics: NaN != NaN under IEEE-754, so a structural `and`-fold cannot \
                     honestly claim total equality here — matching Rust's own `derive(Eq)` \
                     refusal for `f64`) — the whole derive is left an honest gap rather than a \
                     silently-wrong equality (G2)"
                ),
            ));
        }
        if !field_derive_eligible(ft) {
            return Err(GapReason::new(
                Category::DeriveAttr,
                format!(
                    "struct `{ty_name}` derive(PartialEq): field {i} has type `{ft}`, a \
                     primitive repr with no derived (or hand-written) structural-equality \
                     function in this file — the whole derive is left an honest gap rather than \
                     a partial/fabricated equality (G2)"
                ),
            ));
        }
    }
    let vars_a: Vec<String> = (0..field_types.len()).map(|i| format!("p{i}")).collect();
    let vars_b: Vec<String> = (0..field_types.len()).map(|i| format!("q{i}")).collect();
    let parts: Vec<String> = field_types
        .iter()
        .enumerate()
        .map(|(i, ft)| format!("{}({}, {})", eq_fn_name(ft), vars_a[i], vars_b[i]))
        .collect();
    let body = and_chain(&parts);
    Ok(format!(
        "fn {fname}(a: {ty_name}, b: {ty_name}) => Binary{{1}} =\n    match a {{ {ty_name}({pa}) \
         => match b {{ {ty_name}({pb}) => {body} }} }};",
        pa = vars_a.join(", "),
        pb = vars_b.join(", ")
    ))
}

/// A **generic** struct refuses `derive(PartialEq)` — a derived fn for a generic type needs
/// DN-130's generic-instance mechanism, out of this leaf's scope. Mirrors
/// [`super::show`]/[`super::init`]'s identical `is_generic` gate.
fn emit(ctx: &DeriveCtx) -> DeriveOutcome {
    if ctx.is_generic {
        return DeriveOutcome::Gap(GapReason::new(
            Category::DeriveAttr,
            format!(
                "struct `{}` derive(PartialEq): generic struct — a derived equality fn for a \
                 generic type needs DN-130's generic-instance mechanism, out of this leaf's scope \
                 (DN-128/M-1086, DN-136 Phase-2 DERIVE-COMPLETION)",
                ctx.ty_name
            ),
        ));
    }
    match compose(ctx.ty_name, ctx.field_types) {
        Ok(myc) => DeriveOutcome::Composed(myc),
        Err(g) => DeriveOutcome::Gap(g),
    }
}

pub const ROW: DeriveHandler = DeriveHandler {
    recognizes,
    emit,
    slug: "DN-128 (Phase-2 DERIVE-COMPLETION) — PartialEq -> structural `and`-fold over cmp.eq",
    citation: "DN-128 §2 (PartialEq/Eq -> field-wise cmp.eq fold); ADR-040 §2.4 (Float/NaN \
               refusal); RFC-0007 §4.4 / RFC-0019 §3.1 (`equal` as the collision-free method-name \
               precedent); DN-136 Phase-2 bulk-gap-close worklist B1/L1 (disclosed deviation: a \
               plain fn, not `impl Eq[T] for T` — no landed Eq prelude trait; verified \
               duplicate-trait/duplicate-fn collision when Eq+PartialEq or two structs co-occur)",
};
