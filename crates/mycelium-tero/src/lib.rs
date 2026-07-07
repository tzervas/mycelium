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
//! **M-1016 (landed on top of M-1015): the query engine + mandatory provenance.** [`QueryEngine`]
//! answers structured (`id`/`status`/`kind`), cross-reference-walk, and free-text queries over a
//! [`TeroIndexReport`] — every [`Answer`] carries ≥1 resolvable [`Citation`] by construction, and a
//! query that finds nothing citable returns a typed [`Refusal`], never an empty-but-successful
//! answer (DN-87 §6.2, the never-silent rule applied to retrieval). Every answer is EXPLAIN-able
//! ([`Answer::explain`]) — the candidate set, the ordering rule(s), and a per-hit reason. See
//! `crate::query`'s module docs for the full design (including a recorded, resolved scope gap: the
//! model has no structured `supersedes` edge yet, so the cross-reference walk covers exactly
//! `depends_on`/`doc_refs`).
//!
//! **M-1017 (the API fronts): one core, two thin fronts (DN-87 §2.3).** The engine is served
//! platform-agnostically through one framework-agnostic request→[`Query`]→JSON-envelope
//! core, wrapped by an **MCP** server ([`serve_mcp_stdio`], newline-delimited JSON-RPC over stdio)
//! and an **HTTP/JSON** API ([`serve_http`], an `axum`/`tokio` app; ADR-044). Both serialize through
//! the same core, so an answer over MCP is byte-identical to the same answer over HTTP (front
//! parity). Access is token-scoped, read-only by default ([`TokenTable`]/[`Scope`]; DN-87 §6.4) —
//! never-silent: a bad/absent token or too-narrow scope is an explicit refusal, and the servers
//! refuse to start with no tokens configured. The fronts carry the engine's provenance/refusal
//! guarantees across the wire without weakening them.
//!
//! Named in quiet homage to Atsushi Tero, for his contribution to science and engineering.

// Internal modules — the extraction + query engines. The crate's *public* surface (KC-3 small,
// auditable kernel; YAGNI) is deliberately just the re-exports below: the whole-corpus build, the
// two artifact writers, the `index.json` reader, the query engine's types, and the model types a
// downstream consumer (M-1017's API fronts) needs. The per-family extractors, query internals, and
// filesystem plumbing stay crate-internal.
mod changelog;
mod docs;
mod emit;
mod front;
mod index;
mod issues;
mod load;
mod model;
mod query;
mod skills;
mod walk;

pub use emit::{write_json, write_markdown};
pub use index::build_tero_index;
pub use load::load_report;
pub use model::{
    Family, Flagged, SiblingIndex, TeroIndexItem, TeroIndexReport, HONESTY_TAG, ITEM_TAG,
    SIBLING_INDICES,
};
pub use query::{Answer, Citation, Explain, Query, QueryEngine, RankedHit, Refusal};

// M-1017 — the API fronts (MCP + HTTP, token-scoped) over the M-1016 engine. One core, two thin
// fronts (`front::core`); the auth allow-list + the HTTP server type a binary needs to construct.
pub use front::auth::{AuthError, Scope, TokenTable, TokenTableError};
pub use front::http::{serve_http, AppState};
pub use front::mcp::serve_mcp_stdio;

/// The program's one-line summary, used by the (future) API fronts' identify endpoint.
#[must_use]
pub fn crate_summary() -> &'static str {
    "mycelium-tero: the transparent memory substrate & agent knowledge api (DN-87; Layer-1 index + \
     query engine)"
}

#[cfg(test)]
mod tests;
