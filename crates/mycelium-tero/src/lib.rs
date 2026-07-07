//! mycelium-tero — the transparent memory substrate & agent knowledge API (DN-87 / E39-1).
//!
//! **M-1015 (this crate's first landed engine): Layer 1 — the deterministic, drift-gated structured
//! index over the WHOLE corpus.** It generalizes the proven `docs/api-index/` + `docs/lib-index/`
//! pattern (a grep-friendly [`INDEX.md`](emit) beside a machine [`index.json`](emit), a never-silent
//! `flagged` section, a uniform `Empirical/Declared` honesty tag) from one source language to every
//! corpus family: docs ([`docs`]), the issue tracker ([`issues`]), the changelog ([`changelog`]),
//! research records ([`docs`]), and the agent skills ([`skills`]) — emitted to `docs/tero-index/`.
//! The `api-index`/`lib-index` outputs are *referenced* as sibling indices, never duplicated.
//!
//! DRY (house rule #5): the markdown *structure* is parsed by `mycelium_doc::corpus::ingest` (the
//! existing CommonMark-subset corpus parser), not a second heuristic; this crate adds only the
//! metadata the doc-IR never carried (a doc's status/guarantee, per-family summaries).
//!
//! Binding invariants from DN-87 §6: regeneration is **deterministic** (byte-identical at a commit —
//! [`build_tero_index`] is a pure function of the on-disc corpus, proved by the two-run test); every
//! extraction limit lands in the `flagged` section, never dropped (G2); every guarantee/accuracy
//! claim stays at its supportable strength (the index is an `Empirical/Declared` heuristic — source
//! is ground truth; VR-5). Layer 2 (the VSA semantic layer; its improved-on-RAG claim is
//! `Empirical`-gated per DN-87 §6.1) lands later (M-1018).
//!
//! Named in quiet homage to Atsushi Tero, for his contribution to science and engineering.

pub mod changelog;
pub mod docs;
pub mod emit;
pub mod index;
pub mod issues;
pub mod model;
pub mod skills;
pub mod walk;

pub use index::build_tero_index;
pub use model::{
    Family, Flagged, SiblingIndex, TeroIndexItem, TeroIndexReport, HONESTY_TAG, ITEM_TAG,
    SIBLING_INDICES,
};

/// The program's one-line summary, used by the (future) API fronts' identify endpoint.
#[must_use]
pub fn crate_summary() -> &'static str {
    "mycelium-tero: the transparent memory substrate & agent knowledge api (DN-87; Layer-1 index)"
}

#[cfg(test)]
mod tests;
