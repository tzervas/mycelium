//! White-box tests for the Layer-1 corpus index (house layout: no inline tests in logic files —
//! one submodule per concern, `use crate::…` for white-box access to `pub(crate)` internals, and a
//! hermetic temp-dir fixture for the full-walk behavioural tests).

use crate::*;

mod anchors;
mod determinism;
mod families;
mod fixture;
mod flagged;
mod units;

#[test]
fn summary_names_the_crate_and_its_dn() {
    let s = crate_summary();
    assert!(s.contains("mycelium-tero") && s.contains("DN-87"));
}
