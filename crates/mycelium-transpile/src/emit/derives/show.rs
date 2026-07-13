//! DN-128 (M-1086) `derive(Debug)` -> an explicit `impl Show[T] for T` — DN-136/P1-a row. Moved
//! verbatim (no behavior change) from `lower_struct_derives`'s `"Debug"` arms + the former
//! free-standing `derive_show_impl` helper.

use super::{field_derive_eligible, DeriveCtx, DeriveHandler, DeriveOutcome};
use crate::gap::{Category, GapReason};

fn recognizes(name: &str) -> bool {
    name == "Debug"
}

/// Left-fold `parts` into a single `bytes_concat(...)` chain — every step stays `Bytes`-typed,
/// matching `bytes_concat`'s 2-ary `Bytes -> Bytes -> Bytes` signature (`lib/std/fmt.myc`'s
/// `to_dec` uses the identical fold shape for its recursive digit accumulation). `parts` is never
/// empty in the caller below (the fieldless case is handled separately, without this helper).
/// Moved verbatim from the former `emit.rs::bytes_concat_chain` (used only here).
fn bytes_concat_chain(parts: &[String]) -> String {
    let mut iter = parts.iter();
    let mut acc = iter.next().cloned().unwrap_or_default();
    for p in iter {
        acc = format!("bytes_concat({acc}, {p})");
    }
    acc
}

/// **Fieldless (unit) struct:** `fn render(x: T) => Bytes = "T";` — always succeeds, no field
/// dependency (live-oracle-proven, `src/tests/emit.rs`). **Struct with fields:** a left-to-right
/// `bytes_concat` fold of `"T(", render(f0), ", ", render(f1), …, ")"`, gated per field via
/// [`field_derive_eligible`] — refuses the WHOLE derive (never a partial/fabricated render, G2)
/// the moment any field is ineligible, citing that field's index + mapped type. Moved verbatim
/// from the former `emit.rs::derive_show_impl`.
fn compose(ty_name: &str, field_types: &[String]) -> Result<String, GapReason> {
    if field_types.is_empty() {
        return Ok(format!(
            "impl Show[{ty_name}] for {ty_name} {{\n  fn render(x: {ty_name}) => Bytes =\n    \"{ty_name}\";\n}};"
        ));
    }
    for (i, ft) in field_types.iter().enumerate() {
        if !field_derive_eligible(ft) {
            return Err(GapReason::new(
                Category::DeriveAttr,
                format!(
                    "struct `{ty_name}` derive(Debug): field {i} has type `{ft}`, a primitive \
                     repr with no ambient `Show` instance in this file (`lib/std/fmt.myc`'s \
                     primitive impls live in a separate, unimported nodule) — the whole derive is \
                     left an honest gap rather than a partial/fabricated render (G2)"
                ),
            ));
        }
    }
    let vars: Vec<String> = (0..field_types.len()).map(|i| format!("p{i}")).collect();
    let mut parts = vec![format!("\"{ty_name}(\"")];
    for (i, v) in vars.iter().enumerate() {
        if i > 0 {
            parts.push("\", \"".to_string());
        }
        parts.push(format!("render({v})"));
    }
    parts.push("\")\"".to_string());
    let body = bytes_concat_chain(&parts);
    Ok(format!(
        "impl Show[{ty_name}] for {ty_name} {{\n  fn render(x: {ty_name}) => Bytes =\n    match x {{ {ty_name}({pats}) => {body} }};\n}};",
        pats = vars.join(", ")
    ))
}

/// A **generic** struct refuses `derive(Debug)` — a derived impl for a generic type needs
/// DN-130's generic-trait-instance-impl mechanism, out of this leaf's scope. Moved verbatim from
/// `lower_struct_derives`'s `"Debug" if is_generic` arm.
fn emit(ctx: &DeriveCtx) -> DeriveOutcome {
    if ctx.is_generic {
        return DeriveOutcome::Gap(GapReason::new(
            Category::DeriveAttr,
            format!(
                "struct `{}` derive(Debug): generic struct — a derived impl for a \
                 generic type needs DN-130's generic-trait-instance-impl mechanism, out of \
                 this leaf's scope (DN-128/M-1086)",
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
    slug: "DN-128/M-1086 (Debug -> Show)",
    citation: "DN-128 (M-1086); DN-127/M-1090 (prelude Show trait); DN-136 P1-a migration (moved \
               verbatim from lower_struct_derives's Debug arms + derive_show_impl)",
};
