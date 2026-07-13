//! DN-128 (M-1086) `derive(Hash)` -> a structural fold over the landed `hash.blake3` kernel prim
//! (M-912) — DN-136 Phase-2 (DERIVE-COMPLETION) additive row over the frozen `emit/derives` axis
//! (DN-136 P1-a). DN-128 §2: "Hash = field-wise `hash.blake3` fold" over the "landed
//! `hash.blake3` prim (M-912)".
//!
//! **Composes a plain top-level `fn hash_<T>(a: T) => Bytes = hash_blake3(...);`, NOT
//! `impl Hash[T] for T` — the identical disclosed deviation [`super::eq`] documents in full, for
//! the identical reason.** No `Hash` prelude trait is landed — only `Fuse`/`Ord3`/`Show`/`Init`/
//! `Fault` are seeded (`PRELUDE_TRAIT_SEEDS`, `crates/mycelium-l1/src/checkty.rs:2282`) — so
//! composing an `impl` would need this row to self-declare the trait inline, which fails with a
//! "duplicate trait declaration" `myc-check` refusal the moment a second struct in the same file
//! also derives `Hash` (see [`super::eq`]'s module doc for the full empirical trail; the root
//! cause and the fix are identical here). `recognizes` has no `Eq`/`PartialEq`-style co-occurring-
//! name collision to avoid (Rust has no `PartialHash`), so this row simply recognizes `"Hash"`.
//!
//! **Domain-separated by folding the TYPE NAME itself into the hash input** ahead of the fields
//! (`hash_blake3(bytes_concat("T", bytes_concat(hash_<F0>(p0), hash_<F1>(p1)) ...))`) — mirrors
//! [`super::show`]'s `"T("`-prefixed render discriminator — so two differently-named,
//! identically-shaped types never hash identically (a live-oracle-verified shape, confirmed
//! `myc check`-clean for both the fieldless and the nested-field case during this leaf's
//! development). The field-hash fold reuses [`hash_fn_name`]'s deterministic naming so a nested
//! eligible field's own derived hash fn resolves by construction, exactly like [`super::eq`]'s
//! `eq_<FieldType>` composition.
//!
//! Guarantee: `Empirical` (live-oracle-verified, `src/tests/emit.rs`); the field-eligibility
//! heuristic stays `Declared` (same VR-5 boundary as every other row in this axis).

use super::{field_derive_eligible, DeriveCtx, DeriveHandler, DeriveOutcome};
use crate::gap::{Category, GapReason};

fn recognizes(name: &str) -> bool {
    name == "Hash"
}

/// The deterministic top-level fn name this row's compose emits/expects for a given type name —
/// mirrors `eq.rs`'s identical `eq_fn_name` role (no cross-call state needed; both derive from
/// `ty_name`/`field_type` alone).
fn hash_fn_name(ty_name: &str) -> String {
    format!("hash_{ty_name}")
}

/// Left-fold `parts` into a single `bytes_concat(...)` chain — a local copy of
/// [`super::show`]'s private `bytes_concat_chain` helper (not shared across files: each row stays
/// a self-contained, independently-reviewable unit per this axis's row shape — see `mod.rs`'s
/// doc; the ~10-line duplication is a deliberate, disclosed KISS trade-off over refactoring the
/// already-landed, frozen `show.rs`). `parts` is never empty in the caller below.
fn bytes_concat_chain(parts: &[String]) -> String {
    let mut iter = parts.iter();
    let mut acc = iter.next().cloned().unwrap_or_default();
    for p in iter {
        acc = format!("bytes_concat({acc}, {p})");
    }
    acc
}

/// **Fieldless (unit) struct:** `fn hash_T(a: T) => Bytes = hash_blake3("T");` — the type-name
/// string literal alone is the hash input, always succeeds (live-oracle-proven,
/// `src/tests/emit.rs`). **Struct with fields:** `hash_blake3(bytes_concat("T",
/// bytes_concat(hash_<F0>(p0), hash_<F1>(p1), ...)))`, gated per field via
/// [`field_derive_eligible`] (same as [`super::show`]/[`super::init`]/[`super::eq`]) — refuses the
/// WHOLE derive (never a partial/fabricated hash, G2) the moment any field is ineligible.
fn compose(ty_name: &str, field_types: &[String]) -> Result<String, GapReason> {
    let fname = hash_fn_name(ty_name);
    if field_types.is_empty() {
        return Ok(format!(
            "fn {fname}(a: {ty_name}) => Bytes =\n    hash_blake3(\"{ty_name}\");"
        ));
    }
    for (i, ft) in field_types.iter().enumerate() {
        if !field_derive_eligible(ft) {
            return Err(GapReason::new(
                Category::DeriveAttr,
                format!(
                    "struct `{ty_name}` derive(Hash): field {i} has type `{ft}`, a primitive \
                     repr with no derived (or hand-written) structural-hash function in this \
                     file — the whole derive is left an honest gap rather than a \
                     partial/fabricated hash (G2)"
                ),
            ));
        }
    }
    let vars: Vec<String> = (0..field_types.len()).map(|i| format!("p{i}")).collect();
    let mut parts = vec![format!("\"{ty_name}\"")];
    for (i, ft) in field_types.iter().enumerate() {
        parts.push(format!("{}({})", hash_fn_name(ft), vars[i]));
    }
    let inner = bytes_concat_chain(&parts);
    Ok(format!(
        "fn {fname}(a: {ty_name}) => Bytes =\n    match a {{ {ty_name}({pats}) => \
         hash_blake3({inner}) }};",
        pats = vars.join(", ")
    ))
}

/// A **generic** struct refuses `derive(Hash)` — a derived fn for a generic type needs DN-130's
/// generic-instance mechanism, out of this leaf's scope. Mirrors every other row's identical
/// `is_generic` gate.
fn emit(ctx: &DeriveCtx) -> DeriveOutcome {
    if ctx.is_generic {
        return DeriveOutcome::Gap(GapReason::new(
            Category::DeriveAttr,
            format!(
                "struct `{}` derive(Hash): generic struct — a derived hash fn for a generic \
                 type needs DN-130's generic-instance mechanism, out of this leaf's scope \
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
    slug: "DN-128 (Phase-2 DERIVE-COMPLETION) — Hash -> structural hash.blake3 fold",
    citation: "DN-128 §2 (Hash -> field-wise hash.blake3 fold); M-912 (the landed hash.blake3 \
               kernel prim); DN-136 Phase-2 bulk-gap-close worklist B3/L3 (disclosed deviation: a \
               plain fn, not `impl Hash[T] for T` — no landed Hash prelude trait; same root cause \
               eq.rs documents)",
};
