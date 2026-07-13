//! DN-128 (M-1086) `derive(PartialOrd)`/`derive(Ord)` -> a lexicographic fold over the landed
//! `Ord3` prelude trait (DN-122 §13 / M-1080) — DN-136 Phase-2 (DERIVE-COMPLETION) additive row
//! over the frozen `emit/derives` axis (DN-136 P1-a). DN-128 §2: "PartialOrd/Ord = lexicographic
//! `Ord3` fold" over the "landed `Ord3` (M-1080)" prelude trait.
//!
//! **Recognizes ONLY `"PartialOrd"`, never `"Ord"` — the exact same verified, disclosed choice
//! [`super::eq`] makes for `PartialEq`/`Eq`, for the analogous root cause.** `impl Ord3[T] for T`
//! is keyed **globally** per `(trait, type-head)` (RFC-0019 §4.5 coherence); if this row recognized
//! BOTH `"Ord"` and `"PartialOrd"`, the driver's per-derive-list-entry dispatch would invoke `emit`
//! TWICE for one struct under the common real Rust `#[derive(PartialOrd, Ord)]`, composing the
//! IDENTICAL `impl Ord3[T] for T {...}` text twice — refused by the real `myc-check` oracle as an
//! "overlapping instance" coherence violation (empirically confirmed against a scratch probe
//! during this leaf's development; mirrors [`super::eq`]'s duplicate-function finding for the same
//! underlying reason: no per-row dedup state across driver calls). `PartialOrd` is the reliable
//! signal: Rust's own `Ord: Eq + PartialOrd` supertrait bound means valid Rust source never
//! derives bare `Ord` without `PartialOrd` also present, so a solo `#[derive(Ord)]` (invalid Rust,
//! syntactically representable but never rustc-accepted) falls through to the driver's honest
//! `unrecognized` bucket rather than composing twice.
//!
//! **Unlike [`super::eq`]/[`super::hash`], this row composes the literal `impl Ord3[T] for T`
//! shape** the DN-136 Phase-2 worklist sketches — `Ord3` genuinely IS a landed `PRELUDE_TRAIT_SEEDS`
//! entry (`crates/mycelium-l1/src/checkty.rs:2282`; `crates/mycelium-l1/src/ord3.rs`), so no
//! self-declared trait (and thus no duplicate-declaration risk) is needed — only the coherence-key
//! (not-name) uniqueness above applies, and recognizing a single derive name already avoids it.
//!
//! **The three-way sentinel convention** (`Lt = 0b00000000`, `Eq = 0b00000001`, `Gt =
//! 0b00000010`) mirrors the landed `Ord3` regression fixture's own worked example
//! (`crates/mycelium-l1/src/tests/ord3.rs`'s
//! `ord3_prelude_trait_is_builtin_and_an_instance_checks_with_no_local_declaration`) — `Ord3`
//! itself carries **no fixed law** (its own module doc: "an instance's `cmp` may encode whatever
//! three-way order... the implementer intends", DN-122 §13's explicit YAGNI on a law checker), so
//! this is THIS derive's own, disclosed, self-consistent convention. Composability across a nested
//! derived field's own `Ord3` instance holds **by construction** as long as every participating
//! type's instance was ALSO produced by this same row (a heuristic boundary [`field_derive_eligible`]
//! already shares with [`super::show`]/[`super::init`], VR-5).
//!
//! **The ADR-040 Float/NaN refusal** fires FIRST, ahead of the general [`field_derive_eligible`]
//! gate, for the identical documented reason [`super::eq`] gives.
//!
//! Guarantee: `Empirical` (live-oracle-verified, `src/tests/emit.rs`); the field-eligibility
//! heuristic stays `Declared` (same VR-5 boundary as every other row in this axis).

use super::{field_derive_eligible, DeriveCtx, DeriveHandler, DeriveOutcome};
use crate::gap::{Category, GapReason};

/// The `Ord3.cmp` "equal" sentinel this derive's own convention uses (see the module doc's
/// three-way-sentinel paragraph). Only `EQ` needs to be a named constant — `Lt`/`Gt` are never
/// tested against by generated code (any *non*-`EQ` result short-circuits the fold as-is,
/// regardless of whether it happens to be the `Lt` or `Gt` value), so they stay documentation-only
/// (no unused-constant lint).
const ORD3_EQ: &str = "0b00000001";

fn recognizes(name: &str) -> bool {
    name == "PartialOrd"
}

/// **Fieldless (unit) struct:** `fn cmp(a: T, b: T) => Binary{8} = <EQ sentinel>;` — trivially
/// always equal, always succeeds. **Struct with fields:** a right-to-left short-circuit fold —
/// the LAST field's `cmp` is the base case; each earlier field wraps it in `match cmp(p_i, q_i) {
/// EQ => <inner>, other => other }`, so the first non-equal field decides the whole comparison
/// (lexicographic order, field 0 dominates). Gated per field via the ADR-040 float check (first)
/// then [`field_derive_eligible`] — refuses the WHOLE derive (never a partial/fabricated order,
/// G2) the moment any field is ineligible.
fn compose(ty_name: &str, field_types: &[String]) -> Result<String, GapReason> {
    if field_types.is_empty() {
        return Ok(format!(
            "impl Ord3[{ty_name}] for {ty_name} {{\n  fn cmp(a: {ty_name}, b: {ty_name}) => \
             Binary{{8}} =\n    {ORD3_EQ};\n}};"
        ));
    }
    if let Some((i, _)) = field_types
        .iter()
        .enumerate()
        .find(|(_, ft)| ft.as_str() == "Float")
    {
        return Err(GapReason::new(
            Category::DeriveAttr,
            format!(
                "struct `{ty_name}` derive(PartialOrd): field {i} has type `Float` — a derived \
                 TOTAL order over a float field is refused (ADR-040 §2.4 NaN semantics: NaN has \
                 no order position under IEEE-754 §5.11's partial order, so a structural \
                 three-way `Ord3.cmp` fold cannot honestly claim a total order here) — the whole \
                 derive is left an honest gap rather than a silently-wrong order (G2)"
            ),
        ));
    }
    for (i, ft) in field_types.iter().enumerate() {
        if !field_derive_eligible(ft) {
            return Err(GapReason::new(
                Category::DeriveAttr,
                format!(
                    "struct `{ty_name}` derive(PartialOrd): field {i} has type `{ft}`, a \
                     primitive repr with no `Ord3` instance in this file (a primitive repr type \
                     has no ambient `Ord3[..]` impl here) — the whole derive is left an honest \
                     gap rather than a partial/fabricated order (G2)"
                ),
            ));
        }
    }
    let vars_a: Vec<String> = (0..field_types.len()).map(|i| format!("p{i}")).collect();
    let vars_b: Vec<String> = (0..field_types.len()).map(|i| format!("q{i}")).collect();
    let last = field_types.len() - 1;
    let mut body = format!("cmp({}, {})", vars_a[last], vars_b[last]);
    for i in (0..last).rev() {
        body = format!(
            "match cmp({a}, {b}) {{ {ORD3_EQ} => {inner}, other => other }}",
            a = vars_a[i],
            b = vars_b[i],
            inner = body
        );
    }
    Ok(format!(
        "impl Ord3[{ty_name}] for {ty_name} {{\n  fn cmp(a: {ty_name}, b: {ty_name}) => \
         Binary{{8}} =\n    match a {{ {ty_name}({pa}) => match b {{ {ty_name}({pb}) => {body} \
         }} }};\n}};",
        pa = vars_a.join(", "),
        pb = vars_b.join(", ")
    ))
}

/// A **generic** struct refuses `derive(PartialOrd)` — a derived instance for a generic type
/// needs DN-130's generic-trait-instance-impl mechanism, out of this leaf's scope. Mirrors
/// [`super::show`]/[`super::init`]/[`super::eq`]'s identical `is_generic` gate.
fn emit(ctx: &DeriveCtx) -> DeriveOutcome {
    if ctx.is_generic {
        return DeriveOutcome::Gap(GapReason::new(
            Category::DeriveAttr,
            format!(
                "struct `{}` derive(PartialOrd): generic struct — a derived `Ord3` instance for \
                 a generic type needs DN-130's generic-trait-instance-impl mechanism, out of \
                 this leaf's scope (DN-128/M-1086, DN-136 Phase-2 DERIVE-COMPLETION)",
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
    slug: "DN-128 (Phase-2 DERIVE-COMPLETION) — PartialOrd -> lexicographic Ord3 fold",
    citation: "DN-128 §2 (PartialOrd/Ord -> lexicographic Ord3 fold); DN-122 §13/M-1080 (the \
               landed Ord3 prelude trait); ADR-040 §2.4 (Float/NaN refusal); DN-136 Phase-2 \
               bulk-gap-close worklist B2/L2 (recognizes only PartialOrd — verified overlapping-\
               instance collision when Ord+PartialOrd co-occur, same root cause as eq.rs)",
};
