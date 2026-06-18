//! R7-Q4 (M-390) equivalence guard: the interpreter's intrinsic-guarantee assumption must agree
//! with the content-addressed prim table `Π` (`mycelium_core::PrimTable`). DN-10 §3.4:
//! `Π_new(hash(p)) = Π_old(name(p))` — here at the *intrinsic guarantee* level.
//!
//! The interpreter currently threads a hard-coded `intrinsic = Exact` for every built-in
//! (`compose_result`, `crates/mycelium-interp/src/prims.rs`). These tests pin that assumption to the
//! registry so the two can never drift: if a future prim declares a non-`Exact` intrinsic in the
//! table, the interpreter's hard-coded `Exact` would become dishonest and this guard fails — exactly
//! the signal to rewire `compose_result` to read the table (the flagged follow-up).

use mycelium_core::{GuaranteeStrength, PrimTable};
use mycelium_interp::PrimRegistry;

#[test]
fn interp_builtin_names_match_the_prim_table() {
    let interp_reg = PrimRegistry::with_builtins();
    let table_reg = PrimTable::builtins();
    let mut interp = interp_reg.names();
    let mut table = table_reg.names();
    interp.sort_unstable();
    table.sort_unstable();
    assert_eq!(
        interp, table,
        "the interpreter dispatch set and the content-addressed prim table must list the same \
         kernel prims (no drift between the executable table and the declared Π)"
    );
}

#[test]
fn every_interp_builtin_intrinsic_is_exact_in_the_table() {
    // The interpreter assumes `intrinsic = Exact` for every built-in; the table is the source of
    // truth that records it. They must agree for every dispatched prim.
    let table = PrimTable::builtins();
    for name in PrimRegistry::with_builtins().names() {
        assert_eq!(
            table.intrinsic(name),
            Some(GuaranteeStrength::Exact),
            "prim `{name}`: the table's intrinsic must match the interpreter's hard-coded Exact",
        );
    }
}
