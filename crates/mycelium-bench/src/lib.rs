//! `mycelium-bench` — the **honest benchmarking & evaluation harness** (E-BENCH).
//!
//! This is the measurement counterpart to the whole project: it tells us *where Mycelium wins and
//! where it loses* — when, where, how, and why — across the execution backends, and surfaces
//! regressions and capability losses rather than hiding them. It measures the existing backends; it
//! never changes them.
//!
//! ## What it measures
//! Over a shared corpus of v0-calculus programs ([`mod@corpus`]), each backend ([`backend::Backend`]):
//! - the **interpreter** — the trusted base (NFR-7/ADR-007), the differential baseline,
//! - the **AOT env-machine** (`mycelium_mlir::run_core`),
//! - the **JIT** (`mycelium_mlir::jit_run`),
//! - the **direct-LLVM** backend (`mycelium_mlir::compile_and_run`),
//! - the **MLIR-dialect** path (`mycelium_mlir::mlir_compile_and_run`, behind the `mlir-dialect`
//!   feature; skips gracefully when the feature/toolchain is off).
//!
//! For each (backend, case) it captures wall time ([`timing`]) and the result, then classifies it vs
//! the interpreter into a [`verdict::Verdict`]: a **speed WIN/LOSS/neutral**, a **correctness LOSS**
//! (a differential divergence from the trusted base), a **capability LOSS** (an unlowerable node,
//! with its reason), a runtime error, or an environmental skip.
//!
//! It also ingests (reads, never runs) the **LLM-harness** report ([`llm`]) so the language-leverage
//! data (KC-2 / SC-5b: quality + latency + token cost) sits alongside the execution data, and emits a
//! deterministic markdown + JSON report ([`report`]) with an explicit WIN/LOSS table and a "where
//! we're losing" section.
//!
//! ## Honesty (the whole point)
//! - The harness *plumbing* is **`Empirical`** — it is test-verified (the unit tests here).
//! - Every measured number is **`Empirical`** (a trial mean with its trial count + observed spread);
//!   a capability loss / skip / error is **`Declared`**. No verdict is `Proven`/`Exact`.
//! - **No pre-written performance target** (VR-5): the verdicts are whatever was measured.
//! - A **differential divergence** from the trusted interpreter is a recorded correctness LOSS — a
//!   wrong answer, however fast, is still a loss.
//! - An **unlowerable node** is a recorded capability LOSS *with the reason* (G2 — never omitted).
//! - Micro-timing caveats (warmup, **process-spawn cost** for the compiled paths, debug-vs-release)
//!   are stated, not buried — and a **debug build is refused** for perf numbers.
//!
//! ## Relationship to the existing perf harnesses
//! `xtask e1` times the *packing codec + BitNet dot kernels* (M-250/M-303/M-340/M-360) and `xtask
//! kc4` times *per-swap certificate overhead* (M-212). This harness is complementary and broader: it
//! times *whole v0-calculus programs across all the execution backends* and classifies wins vs losses
//! with a structured, ingestible report. It reuses the same dependency-light timing style.

pub mod backend;
pub mod corpus;
pub mod llm;
pub mod measure;
pub mod report;
pub mod timing;
pub mod verdict;

pub use backend::{Backend, Engines, Outcome};
pub use corpus::{corpus, Case, Fragment};
pub use llm::{parse_any_llm_json, GrokLlmReport, LlmReport, ParsedLlmSection};
pub use measure::{run_corpus, CaseRecord, RunRecord};
pub use report::{Honesty, LlmSection, Report};
pub use verdict::{classify, Speed, Verdict};
