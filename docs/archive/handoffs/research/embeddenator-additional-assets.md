# Embeddenator Additional Assets Scan

| Field | Value |
|---|---|
| **Role** | Supplementary scan — assets NOT captured in `embeddenator-groundtruth.md` |
| **Date** | 2026-06-24 |
| **Scope** | Six candidate targets (SIMD path, bench methodology, IO/envelope, block-sparse, retrieval/resonator, interop/FFI) |
| **Confidence** | Per-claim VR-5. Every verdict grounded in a real file:line I read. |

> **What is already captured and therefore NOT re-covered here:**
> The bitsliced `PackedTritVec` kernel, the correction model, and the copy/mut sidestep are in
> `embeddenator-groundtruth.md`. The "no committed measured benchmark numbers" finding is also there
> (groundtruth.md:108–137). This file is additive only.

---

## Ranked Table

| Asset | File:line (authoritative) | What it is | Verdict | Mycelium component |
|---|---|---|---|---|
| **Block-sparse `Block` / `BlockSparseTritVec`** | `embeddenator-vsa-main/src/block_sparse.rs:1–200` | 64-trit blocks (two `u64` bitmasks `pos`/`neg` per block), with `bind`/`bundle` ops directly on bitmasks; sorted `block_id` list; `try_new` never-silent invariant | **PORT-WORTHY** | Dense/sparse representation tier; VSA swap perf-path |
| **Deterministic dataset binary format** | `embeddenator-contract-bench-main/src/dataset.rs:1–120` | Magic `EMBR_DST`, per-vector seed via `master_seed.wrapping_add(index).wrapping_mul(0x517cc1b727220a95)`, `ChaCha8Rng`, header carries seed for full reproducibility | **PORT-WORTHY** | M-796 testing toolkit; conformance corpus generation |
| **`ContractBenchReport` schema** | `embeddenator-contract-bench-main/src/schema.rs:1–34` | Machine-readable JSON schema: `RunMeta` (git SHA, UTC timestamp, seed, profile) + `Vec<Measurement>` (ns/iter, throughput bytes/s, extra) | **PORT-WORTHY** | M-796 bench harness; regression-gate tooling |
| **`BenchConfig` + `measure_fn` harness** | `embeddenator-contract-bench-main/src/harness.rs:1–75` | Fixed-seed `ChaCha8Rng`, explicit warmup iterations (32/200), `black_box` wrapper, ns/iter output — self-contained, no criterion dependency | **PORT-WORTHY** | M-796 deterministic bench loop |
| **`TernarySignatureIndex`** | `embeddenator-retrieval-main/src/retrieval/signature.rs:1–150` | Deterministic 24-probe-dim signature (2 bits/probe packed into `u64`), bucket map, multi-probe radius-1 candidate expansion, sorted iteration for determinism | **REFERENCE-ONLY** | Registry similarity queries (informative, not directly portable) |
| **`VsaBackend` trait** | `embeddenator-interop-main/src/kernel_interop.rs:43–90` | Zero-allocation abstract seam: `zero/bundle/bind/cosine/encode_data/decode_data` typed over `Clone+Send+Sync` Vector; `SparseVecBackend` default impl | **REFERENCE-ONLY** | RFC-0028 `wild`/FFI seam design (informative) |
| **Binary envelope (`EDN1` format)** | `embeddenator-io-main/src/io/envelope.rs:1–50` | 16-byte header: magic `EDN1`, 1-byte `PayloadKind`, 1-byte `CompressionCodec` (None/Zstd/Lz4), 2 reserved bytes, 8-byte uncompressed length; `wrap_or_legacy`/`unwrap_auto` | **REFERENCE-ONLY** | ADR-013 spore serialization (envelope pattern is sound but not content-addressed) |
| **`Resonator` (pattern completion)** | `embeddenator-retrieval-main/src/core/resonator.rs:1–120` | Codebook-project loop: cosine argmax then iterative refinement to `convergence_threshold`; `FactorizeResult` with `factors`/`iterations`/`final_delta` | **NOT-USEFUL** | Already better-specified in Mycelium's own VSA RFC; implementation is thin |
| **SIMD explicit intrinsics** | `embeddenator-vsa-main/docs/SIMD_DESIGN.md:1–end` | Design-only; AVX2/NEON intrinsic snippets in doc are **planned, not implemented** (`Status: PLANNED`); scalar + auto-vectorization is the actual code | **NOT-USEFUL** | Mycelium's own perf-path (MLIR→LLVM) covers this; no implemented novel approach |
| **C FFI opaque-handle pattern** | `embeddenator-interop-main/src/ffi.rs:1–150` | `Box::into_raw`/`Box::from_raw` for opaque `repr(C)` structs, null-checked `borrow_handle`, `ByteBuffer` for returning slices | **NOT-USEFUL** | Standard Rust FFI idiom; nothing novel relative to RFC-0028 scope |

---

## PORT-WORTHY Detail

### 1. Block-sparse `Block` / `BlockSparseTritVec`
**File:** `embeddenator-vsa-main/src/block_sparse.rs:40–200`

The representation groups dimensions into 64-trit blocks, each storing two `u64` bitmasks (`pos`,
`neg`). Invariants are enforced never-silently: `Block::new` panics on overlap (`pos & neg != 0`)
or zero block; `Block::try_new` returns `Option`. The `bind` op is three bitwise-ANDs and two ORs
per block; `bundle` is slightly more (cancellation of opposite-sign overlaps via bitmask logic).
The `BlockSparseTritVec` keeps a sorted `Vec<Block>` plus a `dimension: usize`.

**Why port-worthy for Mycelium:** This is the structural middle tier between `SparseVec` (sorted
index lists) and `PackedTritVec` (fully dense bitslice). It is directly relevant to the
dense/sparse representation tier in Mycelium's VSA swap perf-path, fills the "clustered non-zeros"
regime (~50% memory reduction per the doc comment), and is SIMD-friendly by construction — the
block layout maps cleanly to MLIR vector types. The never-silent `try_new` exactly matches
Mycelium's transparency rule (G2). Tag: `Empirical` for the memory savings claim (doc comment
only, no committed benchmark); `Declared` for SIMD-friendliness claim.

### 2. Deterministic Dataset Binary Format
**File:** `embeddenator-contract-bench-main/src/dataset.rs:1–120`

The format embeds the generation seed in the file header (alongside magic `EMBR_DST`, version,
count, dimension) so any corpus can be fully reproduced. Per-vector seeding uses a fast
multiplicative hash: `master_seed.wrapping_add(index as u64).wrapping_mul(0x517cc1b727220a95)`
plus `ChaCha8Rng`. The binary layout is compact (u32 lengths + u32 indices, no padding), with an
explicit 32-byte reserved field for future extension.

**Why port-worthy for Mycelium:** M-796 (testing toolkit) needs deterministic corpus generation
for VSA conformance and regression gating. The seed-in-header pattern is the right shape: it makes
test corpora self-describing and allows exact reproduction from a single file, matching the "no
black boxes" rule. The multiplicative-hash per-vector seed is a clean technique for avoiding
correlation between adjacent vectors. Tag: `Declared` (design pattern; no measured collision
analysis read).

### 3. `ContractBenchReport` Schema
**File:** `embeddenator-contract-bench-main/src/schema.rs:1–34`

A `RunMeta` (schema version, bench version, profile, seed, UTC timestamp, optional git SHA) paired
with a `Vec<Measurement>` (name, unit, iters, warmup iters, total ns, ns/iter, optional bytes/
throughput, freeform `extra: serde_json::Value`). The struct is `Serialize/Deserialize` via serde.

**Why port-worthy for Mycelium:** This is the output schema for deterministic bench runs — it
complements `BenchConfig`/`measure_fn` below. Small and complete enough to adopt directly in
M-796's regression tooling. The `git_sha` field enables exact reproducibility tracing. The `extra`
escape hatch lets us carry guarantee-tag metadata without breaking the schema. Tag: `Declared`
(design; no tests exercising round-trip read).

### 4. `BenchConfig` + `measure_fn` Harness
**File:** `embeddenator-contract-bench-main/src/harness.rs:1–75`

A minimal, self-contained benchmark primitive: `BenchConfig` holds a `Profile` (Quick/Full) and
`u64 seed` for `ChaCha8Rng`; `measure_fn` runs warmup iterations (`black_box`), times the
measurement loop with `Instant::now`, and returns a `Measured` (iters, warmup iters, total ns,
ns/iter). No criterion dependency. Quick profile: 32 warmup + 300 iters; Full: 200 warmup + 3000.

**Why port-worthy for Mycelium:** criterion is heavy and introduces its own timing variance. This
harness is a single-file primitive usable in the `just test-fast` loop (Tier 0) or in conformance
tests where repeatable ns/iter matters more than statistical CI. It pairs naturally with the
`ContractBenchReport` schema above. Gives M-796 a thin, auditable timing layer with no transitive
framework dependency. Tag: `Declared` (methodology; warmup counts not validated against a measured
warm-up curve).

---

## REFERENCE-ONLY Notes

**`TernarySignatureIndex`** (`signature.rs`): the 24-probe-dim, 2-bits-per-probe packed-`u64`
signature scheme with multi-probe radius-1 is a clean design worth reading when specifying
Mycelium's registry similarity queries. The determinism guarantee (sorted iteration during build)
directly parallels Mycelium's transparency rule. Not directly portable: probe dimensions are
substrate-specific (`DIM = 10000`), and the registry's query semantics haven't been fixed.

**`VsaBackend` trait** (`kernel_interop.rs:43–90`): the five-method abstract seam
(`zero/bundle/bind/cosine/encode_data/decode_data`) is worth reading when designing RFC-0028's
`wild`/FFI layer. The `SparseVecBackend` default impl shows the pattern cleanly. Not directly
portable — Mycelium's swap machinery is richer and RFC-0028's surface hasn't been finalized.

**Binary envelope `EDN1`** (`envelope.rs:1–50`): the 16-byte `[magic(4), kind(1), codec(1),
reserved(2), uncompressed_len(8)]` layout is a sound, compact envelope for optional compression.
Informative for ADR-013 spore serialization. Not directly portable: no content-addressing (no hash
in the header) and the `PayloadKind` enum is application-specific.

---

*Confidence summary: PORT-WORTHY items 1–4 are grounded in code read at the cited file:line.
Memory/SIMD/perf claims within them are tagged `Declared` unless noted otherwise. REFERENCE-ONLY
and NOT-USEFUL verdicts are honest: the SIMD design doc explicitly marks intrinsic snippets
`Status: PLANNED` — there is no novel implemented approach to extract there.*
