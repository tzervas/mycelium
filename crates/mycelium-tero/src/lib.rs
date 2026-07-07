//! mycelium-tero — the transparent memory substrate & agent knowledge API (DN-87 / E39-1).
//!
//! **M-1015 (this crate's first landed engine): Layer 1 — the deterministic, drift-gated structured
//! index over the WHOLE corpus.** It generalizes the proven `docs/api-index/` + `docs/lib-index/`
//! pattern (a grep-friendly `INDEX.md` beside a machine `index.json`, a never-silent `flagged`
//! section, a uniform `Empirical/Declared` honesty tag) from one source language to every corpus
//! family — docs (RFC/ADR/DN/spec/guide/planning), research records, the issue tracker, the
//! changelog, and the agent skills — emitted to `docs/tero-index/`. The `api-index`/`lib-index`
//! outputs are *referenced* as sibling indices ([`SIBLING_INDICES`]), never duplicated.
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

// Internal modules — the extraction engine. The crate's *public* surface (KC-3 small, auditable
// kernel; YAGNI) is deliberately just the three re-exports below: the whole-corpus build, the two
// artifact writers, and the model types a downstream consumer (M-1016 query / M-1017 API) needs.
// The per-family extractors and filesystem plumbing stay crate-internal.
mod changelog;
mod docs;
mod emit;
mod index;
mod issues;
mod model;
mod skills;
mod walk;

pub use emit::{write_json, write_markdown};
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
