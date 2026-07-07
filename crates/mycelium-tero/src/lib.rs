//! mycelium-tero — the transparent memory substrate & agent knowledge API (DN-87 / E39-1).
//!
//! Scaffold only (the `mem` kickoff's pre-flight): the M-1015…M-1018 lanes land the engine —
//! Layer 1 (deterministic, drift-gated structured index over the whole corpus) and Layer 2
//! (the VSA semantic layer; its improved-on-RAG claim is `Empirical`-gated per DN-87 §6.1).
//! Binding invariants from DN-87 §6: an answer without a resolvable citation is a refusal,
//! never an answer (G2); regeneration is deterministic; per-layer guarantee tags are honest.
//!
//! Named in quiet homage to Atsushi Tero, for his contribution to science and engineering.

/// The program's one-line summary, used by the (future) API fronts' identify endpoint.
pub fn crate_summary() -> &'static str {
    "mycelium-tero: the transparent memory substrate & agent knowledge api (DN-87; scaffold)"
}

#[cfg(test)]
mod tests;
