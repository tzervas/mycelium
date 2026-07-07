//! White-box tests for the scaffold (house layout: no inline tests in logic files).

use crate::*;

#[test]
fn summary_names_the_crate_and_its_dn() {
    let s = crate_summary();
    assert!(s.contains("mycelium-tero") && s.contains("DN-87"));
}
