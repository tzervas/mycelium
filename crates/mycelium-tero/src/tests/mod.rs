//! White-box tests for the Layer-1 corpus index (M-1015) and its query engine (M-1016) — house
//! layout: one submodule per concern, `use crate::…` for white-box access to `pub(crate)`
//! internals, and a hermetic temp-dir fixture (shared by both) for the full-walk behavioural tests.

use crate::*;

mod anchors;
mod determinism;
mod families;
mod fixture;
mod flagged;
mod load;
mod query_crossref;
mod query_latency;
mod query_structured;
mod query_text;
mod units;

#[test]
fn summary_names_the_crate_and_its_dn() {
    let s = crate_summary();
    assert!(s.contains("mycelium-tero") && s.contains("DN-87"));
}
