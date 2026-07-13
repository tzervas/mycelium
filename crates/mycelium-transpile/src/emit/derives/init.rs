//! DN-128 (M-1086) `derive(Default)` -> an explicit `impl Init[T] for T` — DN-136/P1-a row. Moved
//! verbatim (no behavior change) from `lower_struct_derives`'s `"Default"` arms + the former
//! free-standing `derive_init_impl` helper.

use super::{field_derive_eligible, DeriveCtx, DeriveHandler, DeriveOutcome};
use crate::gap::{Category, GapReason};

fn recognizes(name: &str) -> bool {
    name == "Default"
}

/// **Fieldless (unit) struct:** `fn init() => T = T;` — the bare nullary constructor, always
/// succeeds (live-oracle-proven, `src/tests/emit.rs`). **Struct with fields:**
/// `T(init(), init(), …)`, one bare `init()` per field IN DECLARATION ORDER — no qualified
/// `Type::init()` call is needed (RFC-0019 §4.4's "seed from expected" path). Gated per field via
/// [`field_derive_eligible`] for the identical reason as [`super::show`]'s `compose`. Moved
/// verbatim from the former `emit.rs::derive_init_impl`.
fn compose(ty_name: &str, field_types: &[String]) -> Result<String, GapReason> {
    if field_types.is_empty() {
        return Ok(format!(
            "impl Init[{ty_name}] for {ty_name} {{\n  fn init() => {ty_name} =\n    {ty_name};\n}};"
        ));
    }
    for (i, ft) in field_types.iter().enumerate() {
        if !field_derive_eligible(ft) {
            return Err(GapReason::new(
                Category::DeriveAttr,
                format!(
                    "struct `{ty_name}` derive(Default): field {i} has type `{ft}`, a primitive \
                     repr with no landed `Init` instance anywhere in the corpus yet — the whole \
                     derive is left an honest gap rather than a partial/fabricated init (G2)"
                ),
            ));
        }
    }
    let calls = vec!["init()".to_string(); field_types.len()];
    Ok(format!(
        "impl Init[{ty_name}] for {ty_name} {{\n  fn init() => {ty_name} =\n    {ty_name}({args});\n}};",
        args = calls.join(", ")
    ))
}

/// A **generic** struct refuses `derive(Default)` — a derived impl for a generic type needs
/// DN-130's generic-trait-instance-impl mechanism, out of this leaf's scope. Moved verbatim from
/// `lower_struct_derives`'s `"Default" if is_generic` arm.
fn emit(ctx: &DeriveCtx) -> DeriveOutcome {
    if ctx.is_generic {
        return DeriveOutcome::Gap(GapReason::new(
            Category::DeriveAttr,
            format!(
                "struct `{}` derive(Default): generic struct — a derived impl for a \
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
    slug: "DN-128/M-1086 (Default -> Init)",
    citation: "DN-128 (M-1086); DN-129/M-1091 (prelude Init trait); DN-136 P1-a migration (moved \
               verbatim from lower_struct_derives's Default arms + derive_init_impl)",
};
