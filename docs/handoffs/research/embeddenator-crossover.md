# Embeddenator-* Crossover Research — Mycelium Open Design Questions

> ⚠️ **CORRECTION (2026-06-24) — superseded in part by `embeddenator-groundtruth.md`.** A later full-source
> ground-truth pass (against the maintainer's authoritative repo zips) **refuted several claims below**, most
> importantly the copy/mut finding: `VersionedEmbrFS` mutation is **re-encode-the-whole-chunk-and-overwrite-in-place**
> (`versioned_embrfs.rs:744–788`), **not** "chunk-level copy-on-write / structural sharing" as stated in §Q1 — the
> refcount/dedup/GC code is present but **dead/unwired** (`ref_count` never decremented, `gc()` a no-op, no `compact`).
> Embeddenator **sidesteps** efficient copy/mut by re-encoding; it does not solve it (this *strengthens* the synthesis
> thesis that provenance + reclamation is the unsolved residue). Also corrected: **zero committed *measured* benchmark
> numbers** exist (the figures here are "Expected"/theoretical projections); the `TernaryVecRepr` 25% auto-selector is
> module-header guidance, **not code**; the correction model is **5 declared / 4 active** (`TritFlips` never generated).
> The bitsliced ternary kernel finding is *strengthened* (now line-anchored to real code). **Read `embeddenator-groundtruth.md`
> for the corrected, file:line-grounded record; this file is preserved as the superseded first pass (VR-5: corrections
> are recorded, not hidden).**

| Field | Value |
|---|---|
| **Role** | Investigation agent — full-file local read of 13 repos |
| **Status** | Research artifact — **non-normative**. Feeds RFC-0027, DN-28, RFC-0034/E21-1, VSA representation tier design. |
| **Date** | 2026-06-24 |
| **Confidence** | Per-claim VR-5 tags throughout. Numbers cited to actual file:line. |
| **Repos read** | `/tmp/embeddenator-research/<repo>-main/` — full local clones |

---

## 1. What the Embeddenator Series Is

`Empirical` — grounded in `/tmp/embeddenator-research/embeddenator-main/.orchestration/STATUS_REPORT.md` (lines 8–12, 47–70).

The **embeddenator** series (13 repos, ~32,000 LOC Rust) is a **holographic computing substrate** implementing Vector Symbolic Architecture (VSA) with sparse balanced ternary vectors. As of January 2026 it is mid-way through Phase 2A decomposition from a monolithic design into modular crates:

**Core layer**
- `embeddenator-vsa` — `SparseVec` + `PackedTritVec` + bitsliced ternary; bind/bundle/cosine; codebook encoding
- `embeddenator-fs` — `EmbrFS` (immutable holographic FUSE filesystem) + `VersionedEmbrFS` (mutable, optimistic-lock layer over the immutable base)
- `embeddenator-retrieval` — content-addressed retrieval; resonator network

**Infrastructure**
- `embeddenator-io` — envelope/serialization (zstd/LZ4/bincode); `CompressionProfiler`
- `embeddenator-obs` — lock-free atomic metrics; hi-res timers; tracing spans
- `embeddenator-interop` — kernel interop

**Dev/Test**
- `embeddenator-contract-bench` — deterministic criterion benchmarks (vsa_ops, fs_operations, simd_cosine, retrieval_index, hierarchical_scale)
- `embeddenator-testkit`, `embeddenator-workspace`, `embeddenator-cli`, `embeddenator-workflows`

**Architectural invariants** (grounded in `embeddenator-fs-main/docs/ARCHITECTURE.md` lines 1–50 and `embeddenator-vsa-main/docs/ARCHITECTURE.md` lines 1–50):
- All file data lives in a content-addressed, VSA-bundled "engram" (a `SparseVec` superposition)
- A three-layer correction system guarantees bit-perfect reconstruction despite VSA's inherent approximation
- The base `EmbrFS` is **immutable by design**; `VersionedEmbrFS` adds mutation via optimistic locking, never by changing the immutable layer in place
- No memory-management facility beyond Rust's ownership; no GC, no arena allocation

---

## 2. Per-Mycelium-Question Crossover Findings

### Q1. COPY/MUT — Efficient Immutable Update (`VersionedEmbrFS`)

`Empirical` — grounded in `embeddenator-fs-main/src/fs/versioned_embrfs.rs` (lines 1–218, 519–804, 1197–1334).

**How they added mutability over immutable content-addressed storage:**

`VersionedEmbrFS` does **not** mutate the immutable engram in place. The design is:

1. **Chunk-level copy-on-write.** Every write operation (byte replace, append, truncate) re-encodes only the *affected 4KB chunks* and inserts new versioned chunks alongside corrections. Unaffected chunks share storage unchanged (`VersionedChunkStore` — a versioned `HashMap<ChunkId, VersionedChunk>` under `Arc<RwLock<...>>`). Source: `versioned_embrfs.rs` lines 724–804 (`apply_byte_replace`), 953–1090 (`apply_append`).

2. **Optimistic locking, not structural sharing.** Mutation is guarded by a monotonic `global_version: Arc<AtomicU64>` per file entry (line 797 `fetch_add`). Concurrent writers detect conflicts via `VersionMismatch { expected, actual }` (lines 107–115). The pattern is exactly OCC (optimistic concurrency control), not MVCC or CoW snapshot trees.

3. **Delta operations for efficiency.** `apply_delta` dispatches to `apply_byte_replace` / `apply_same_length_replace` / `apply_append` / `apply_truncate` — all of which re-encode only the relevant chunk(s). The documented speedup is **~90x for single-byte edits in 1MB files** (docstring lines 528–532): full re-encode ~90ms vs. delta ~1ms; 10-byte edits ~9x speedup; append 1KB to 1MB file ~90x speedup. Tag: `Declared` (documented in source, not separately benchmarked in the committed criterion suite).

4. **Soft delete, no structural GC.** `delete_file` marks a `VersionedFileEntry` with `deleted: true` (line 1502–1509); chunks remain until an explicit `compact` call.

5. **Hybrid WAL journaling** for persistence: `versioned/journal.rs` (lines 1–100) uses fsync barriers for single small-file writes (<64KB) and a Write-Ahead Log for multi-file transactions. Durability modes: Immediate (~5–10ms SSD), GroupCommit (~5ms batch latency, default), Relaxed (near-zero, OS-flush).

**Mycelium relevance (RFC-0027, `cyst`, copy-on-write design):**

The embeddenator pattern directly informs RFC-0027's open questions:
- Mycelium's immutable + content-addressed value model makes the same chunk-level CoW natural: a mutation produces a new value node (new hash, new `Provenance::Derived`) while sharing unchanged sub-values.
- The `VersionedChunkStore` / `AtomicU64` OCC pattern is the concrete implementation of what Mycelium calls "optimistic update on content-addressed state." It shows OCC is workable, but also that it requires a per-file version counter — this is the structural analog of Mycelium's reclamation EXPLAIN record's `sweep_epoch`.
- The `apply_delta` speedup figures (90x for byte replace in large files) quantify the penalty of full re-encode vs. targeted chunk update — a concrete data point for the `cyst` checkpoint-and-free vs. checkpoint-and-keep tradeoff (SYNTHESIS §4.1 TN-4).

---

### Q2. RECLAMATION / Memory Model

`Empirical` — grounded in `versioned_embrfs.rs` lines 133–175, `correction.rs` (via `embeddenator-fs-main/docs/ARCHITECTURE.md` lines 219–258), `versioned/journal.rs`.

**How embeddenator manages memory/lifetime/reclamation:**

The series uses **no explicit arena/region/refcount pattern** beyond Rust's standard ownership. Key patterns:

- **`Arc<RwLock<T>>`** for shared concurrent access to the chunk store, manifest, corrections, and the root VSA vector (lines 133–175 of `versioned_embrfs.rs`). This is Rust's standard reference-counted shared ownership — equivalent to Mycelium's `Rc`/`Arc` analog in the residue cluster (SYNTHESIS §2 Lane A rows 5–7,9).
- **Soft delete** (`deleted: bool` flag, line 1502–1509) — chunks are logically freed but physically retained until explicit `compact`. This is analogous to Mycelium's `cyst` checkpoint-and-keep default (SYNTHESIS A-4).
- **No cycle detection** — the holographic value graph is acyclic by construction (a VSA engram is a pure superposition of encoded byte sequences; no back-references). This directly parallels Mycelium D-2.
- **No cross-chunk reference counting** — chunk IDs are assigned monotonically (`next_chunk_id: AtomicU64`, line 154); reclamation happens only at compact time. There is no ORCA-style distributed refcount.

**Crossover:** This is consistent with the SYNTHESIS cluster's conclusion that acyclic, content-addressed values eliminate cycle detection and reduce reclamation to scope-exit / explicit compact. The embeddenator series provides a working Rust implementation of this pattern at ~32K LOC scale.

---

### Q3. CONTENT-ADDRESSED STORAGE + Reconstruction-on-Render (DN-28) + Tamper Evidence

`Empirical` — grounded in `embeddenator-fs-main/docs/CORRECTION.md` (full file); `embeddenator-fs-main/docs/ARCHITECTURE.md` lines 219–260, 406–424.

**The three-layer correction architecture:**

Layer 1: VSA holographic encoding (~94% uncorrected fidelity with `ReversibleVSAEncoder`, `versioned_embrfs.rs` line 186; ~0% uncorrected with legacy codebook encoding, line 71).
Layer 2: SHA256 verification hash (first 8 bytes) computed immediately after encoding; mismatch triggers correction generation (`CORRECTION.md` lines 56–68).
Layer 3: Algebraic correction store — five correction types chosen for minimum space:

| Type | Overhead | When |
|---|---|---|
| `Perfect` | 0 bytes | VSA already bit-perfect (~40–62% of chunks for structured data) |
| `BitFlips` | 2 bytes per flip | Sparse bit differences |
| `TritFlips` | 3 bytes per flip | Ternary value differences |
| `BlockReplace` | 4 bytes + region | Contiguous differing regions |
| `Verbatim` | 100% (full chunk) | Fallback for highly entropic data |

Source: `CORRECTION.md` lines 79–208.

**Observed correction overhead by data type** (`CORRECTION.md` lines 266–277, `ARCHITECTURE.md` lines 413–424):

| Data Type | Perfect % | Overhead % |
|---|---|---|
| Config Files | 62% | 1.8% |
| Source Code | 55% | 2.1% |
| Text Files | 48% | 3.4% |
| Binary Data | 30% | 6.2% |
| Compressed Data | 15% | 12.5% |
| Random Data | 8% | 18.3% |

**Parity trit** (`CORRECTION.md` lines 342–364): `sum(bytes) mod 3` stored alongside each correction, used to detect corruption of the correction store itself.

**How this compares to Mycelium DN-28 reconstruction-on-render:**

DN-28 proposes rendering a value from its content-addressed representation, with a provenance record. Embeddenator's correction layer is a concrete implementation of exactly this — the VSA engram is the "holographic form" and the correction store is the "render residual." The key finding:

- Bit-perfect reconstruction is achieved not by making VSA lossless (impossible in the general case), but by storing a **minimum-cost correction delta** alongside the lossy encoding. This is a directly transferable pattern for Mycelium's representation swap: when a swap is approximate (e.g., lossy quantization), store an algebraic correction rather than falling back to full verbatim.
- The 40–62% "Perfect" rate for structured data means holographic storage is genuinely useful for typical source/config data — the correction overhead is 1.8–3.4%, not 100%.
- SHA256 truncated to 8 bytes (64 bits) is used for quick verification. This is the tamper-evidence primitive. The embedding of a hash in the correction record is the concrete form of SYNTHESIS AG-1's "content-hash equality over an immutable value."

**Caveat** (`Declared`): The benchmark numbers in `CORRECTION.md` are documented as "empirical measurements" but the criterion benchmarks do not separately time the correction layer — only encoding/decoding throughput is benchmarked. The overhead percentages are design-phase measurements, not production profiling results.

---

### Q4. CRDT / `fuse` — "Holographic Superposition of Directory Trees"

`Empirical` (structural properties) / `Declared` (CRDT convergence claim).

**Is the superposition a semilattice-like / commutative merge?**

The VSA bundle operation `⊕` has the following documented properties (`embeddenator-vsa-main/docs/ARCHITECTURE.md` lines 115–135):
- **Commutative:** `A ⊕ B = B ⊕ A` — explicitly stated
- **Approximately associative** (with thinning/thresholding)
- Result is similar to all inputs

The `bundle` implementation (`BITSLICED_TERNARY_DESIGN.md` lines 179–218) is a saturating elementwise addition over balanced ternary trits: `P+P=P, N+N=N, P+N=Z`. This is an **idempotent, commutative saturation** — structurally a join in a lattice over `{-1, 0, +1}` per trit.

**Does it converge?** `Declared` — the bundle operation is deterministic and commutative, so bundling the same set of vectors in any order yields the same result (modulo thinning). But **retrieval (unbundling) is approximate**, and the embeddenator explicitly does **not** claim CRDT semantics. There is no tombstone logic, no merge of deletion states, no convergence proof under concurrent updates.

**Mycelium relevance (`fuse` — RFC-0008 §4.x, distributed merge):**

The bundle commutativity is directly relevant to Mycelium's `fuse` operation for merging content-addressed state (SYNTHESIS O-8). The VSA bundle is a concrete, benchmarked implementation of a commutative, saturating merge over high-dimensional ternary vectors — precisely the kind of primitive that `fuse` would need for R2/`xloc`/`mesh` distributed scenarios. However, the "holographic superposition of directory trees" does **not** automatically give CRDT convergence: the correction layer is required to recover bit-perfect content, and there is no protocol for merging correction stores across concurrent writers. This is the gap that would need to fill for a distributed `fuse`.

---

### Q5. VSA / TERNARY Value Model

`Empirical` — grounded in `embeddenator-vsa-main/docs/BITSLICED_TERNARY_DESIGN.md` (full file), `embeddenator-vsa-main/docs/ARCHITECTURE.md` (full file), `embeddenator-vsa-main/src/` file list.

**What is directly reusable for Mycelium's representation tier:**

**Balanced ternary encoding (2 bits per trit, bitsliced):**
`PackedTritVec` stores 32 trits per `u64` using a bitplane encoding: even bits = P plane, odd bits = N plane. The encoding `{00=Z, 01=P, 10=N, 11=reserved}` achieves 100% bit utilization (2 bits per trit) and enables 32-parallel operations via hardware POPCNT. Source: `BITSLICED_TERNARY_DESIGN.md` lines 48–67.

**Bind (elementwise multiplication):** Branchless, 32 trits per word, pure bitwise AND/OR/shift — directly maps to Mycelium's `swap` at the ternary tier (`BITSLICED_TERNARY_DESIGN.md` lines 141–178).

**Bundle (saturating addition):** Same bitplane pattern, commutative, idempotent at saturation bounds (`BITSLICED_TERNARY_DESIGN.md` lines 180–218).

**Sparse/dense adaptive strategy:** Crossover at ~25% density (`ARCHITECTURE.md` lines 247–260). Below 25%: `SparseVec` (sorted index lists, 8 bytes/nonzero). Above 25%: `PackedTritVec` (2 bits/trit). This is precisely Mycelium's binary/ternary/dense representation tier with a principled, empirically-derived threshold.

**Self-inverse bind:** `A ⊙ A ≈ Identity` — enables querying associations without maintaining inverse keys, which maps directly to Mycelium's "never-silent representation swap" philosophy (you always know which binding key was applied because it's the same operation).

**SIMD pathway (`SIMD_DESIGN.md` lines 1–100):** Phase 1 (current): scalar + LLVM auto-vectorization (2–4x over naive). Phase 2 (planned): explicit AVX2/NEON (2–4x over auto-vectorized, 4–8x over naive). Phase 3 (planned): GPU/CUDA (10–100x for vectors >10M dimensions). The bitsliced design was chosen specifically to enable this — no API change is needed when moving from scalar to SIMD.

**What is embeddenator-specific and does not transfer cleanly:**
- The codebook/projection system (basis vectors for differential encoding) — this is an ML-like learned representation Mycelium does not need.
- The FUSE layer — embeddenator uses this for kernel interop; Mycelium's interop model is different.
- The `ReversibleVSAEncoder` with 64-byte `REVERSIBLE_CHUNK_SIZE` (`versioned_embrfs.rs` lines 71–83) — this is a heuristic empirically tuned for filesystem chunk sizes, not a VSA fundamental.

---

### Q6. The Benchmarks (Headline Numbers)

`Empirical` — extracted from source files only; these are documented design-time / criterion benchmark targets, not production profiling results. All numbers are `Declared` accuracy unless specifically marked from criterion benchmark source code (which gives the benchmark setup but not output numbers, as criterion outputs are generated at run time and not committed).

#### Table 1: Performance envelope from documentation

| Metric | Value | Source file:line | Tag |
|---|---|---|---|
| VSA encode throughput (single-thread) | ~50–100 MB/s | `CORRECTION.md`:298–300 | `Declared` |
| VSA decode throughput (single-thread) | ~100–200 MB/s | `CORRECTION.md`:299–300 | `Declared` |
| Correction application throughput | ~500–1000 MB/s | `CORRECTION.md`:301 | `Declared` |
| FUSE sequential read | ~100–200 MB/s | `FUSE.md`:267 | `Declared` |
| FUSE random read | ~10–50 MB/s | `FUSE.md`:268 | `Declared` |
| FUSE small file read (<4KB) | ~1–5 ms/file | `FUSE.md`:269 | `Declared` |
| FUSE large file read (>1MB) | ~5–20 ms + transfer | `FUSE.md`:270 | `Declared` |
| FUSE directory listing <100 files | ~1–5 ms | `FUSE.md`:273 | `Declared` |
| FUSE directory listing >1000 files | ~10–50 ms | `FUSE.md`:274 | `Declared` |
| Exact path lookup (inverted index) | ~1–10 μs | `ARCHITECTURE.md`:407 | `Declared` |
| Beam search retrieval | ~100–1000 μs | `ARCHITECTURE.md`:408 | `Declared` |
| LRU cache hit | ~10–50 μs | `ARCHITECTURE.md`:409 | `Declared` |
| LRU cache miss (disk I/O dominant) | ~10–100 ms | `ARCHITECTURE.md`:410 | `Declared` |
| Peak memory for 10,000 files (no hierarchy) | ~500 MB | `ARCHITECTURE.md`:394 | `Declared` |
| VSA bundle (sparse, 10K dim) | ~0.1 ms | `ARCHITECTURE.md`:332 | `Declared` |
| VSA cosine (sparse, 10K dim) | ~0.05 ms | `ARCHITECTURE.md`:333 | `Declared` |
| Encode 1KB data | ~0.5 ms | `ARCHITECTURE.md`:334 | `Declared` |
| Correction overhead, config files | 1.8% | `CORRECTION.md`:273 | `Empirical` |
| Correction overhead, source code | 2.1% | `CORRECTION.md`:271 | `Empirical` |
| Correction overhead, compressed data | 12.5% | `CORRECTION.md`:275 | `Empirical` |
| Correction overhead, random data | 18.3% | `CORRECTION.md`:276 | `Empirical` |
| "Perfect" rate for structured data | 40–62% | `CORRECTION.md`:266–277 | `Empirical` |
| Delta edit speedup (1 byte in 1MB file) | ~90x | `versioned_embrfs.rs`:528–532 | `Declared` |
| Holographic uncorrected accuracy (reversible encoder) | ~94% | `versioned_embrfs.rs`:186 | `Declared` |
| Holographic uncorrected accuracy (legacy codebook) | ~0% | `versioned_embrfs.rs`:71 | `Declared` |
| SIMD auto-vectorization speedup (current) | 2–4x over naive scalar | `SIMD_DESIGN.md`:138–139 | `Declared` |
| Explicit AVX2 speedup (planned Phase 2) | 2–4x over auto-vectorized | `SIMD_DESIGN.md`:397 | `Declared` |
| GPU speedup (planned Phase 3, large vectors) | 10–100x | `SIMD_DESIGN.md`:400 | `Declared` |
| PackedTritVec: trits per u64 word | 32 | `BITSLICED_TERNARY_DESIGN.md`:59 | `Exact` |
| PackedTritVec: memory (10K dim) | ~2.5 KB | `BITSLICED_TERNARY_DESIGN.md`:229 | `Exact` |
| SparseVec @ 5% density (10K dim) | ~8 KB | `ARCHITECTURE.md`:324 | `Exact` |
| Sparse/dense crossover threshold | ~25% density | `BITSLICED_TERNARY_DESIGN.md`:235–237 | `Empirical` |
| Observability counter overhead | <20 ns target | `performance_benchmark.rs`:40 | `Declared` |
| Tracing span creation overhead | <200 ns target | `performance_benchmark.rs`:69 | `Declared` |
| TestMetrics timing overhead | <100 ns target | `performance_benchmark.rs`:91 | `Declared` |

#### Table 2: Criterion benchmark suites (setup visible, output numbers generated at runtime)

| Benchmark group | Operations covered | Source |
|---|---|---|
| `sparsevec_ops` | bundle, bind, cosine, chain-8 | `contract-bench/benches/vsa_ops.rs`:13–63 |
| `reversible_encode_decode` | encode/decode at 64, 256, 1024, 4096, 16384 bytes | `contract-bench/benches/vsa_ops.rs`:66–104 |
| `bundle_modes` | pairwise/sum_many/hybrid at sparse, dense, mid-density | `contract-bench/benches/vsa_ops.rs`:107–245 |
| `cosine_scalar_vs_simd` | scalar vs SIMD cosine, 6 data patterns | `contract-bench/benches/simd_cosine.rs`:16–75 |
| `cosine_synthetic_sparsity` | scalar cosine, sparsity 10–2000 nonzeros | `contract-bench/benches/simd_cosine.rs`:78–125 |
| `cosine_query_workload` | 1000-doc corpus, scalar | `contract-bench/benches/simd_cosine.rs`:128–167 |
| `embrfs_ingest` | flat structure, 10/50/100 files × 1KB | `contract-bench/benches/fs_operations.rs`:78–109 |
| `embrfs_nested` | depth 2–3, 3–5 files/dir, 500-byte files | `contract-bench/benches/fs_operations.rs`:112–142 |
| `embrfs_extract` | single file extract from 50-file engram | `contract-bench/benches/fs_operations.rs`:144–174 |
| `hierarchical_scale` | large-scale hierarchical engram benchmarks | `contract-bench/benches/hierarchical_scale.rs` |
| `retrieval_index` | retrieval indexing performance | `contract-bench/benches/retrieval_index.rs` |

**Caveat on benchmark numbers** (`Declared`): The criterion suites are present and well-structured. No committed benchmark output files (JSON/HTML) were found in the local clones — these are generated at runtime. The documentation numbers in Table 1 are design-time targets documented in the source/docs, not committed criterion output. Do not treat them as production-validated figures.

---

## 3. What Transfers to Mycelium vs. What is Embeddenator-Specific

### 3.1 Transfers Directly

**A. Bitsliced balanced ternary representation (`PackedTritVec`)**

The design decision to encode balanced ternary as 2 bits per trit with a bitplane split (P plane in even bits, N plane in odd bits, 32 trits per u64) is a clean, implementable, directly reusable design for Mycelium's ternary tier. The bind, bundle, and dot operations are all branchless, 32-parallel, hardware-POPCNT-accelerated, and documented with full correctness proofs in `BITSLICED_TERNARY_DESIGN.md`. The sparse/dense adaptive threshold (25%) is grounded in the mathematical crossover between `8 bytes/nonzero` and `2 bits/trit`.

**B. Sparse/dense adaptive strategy with explicit crossover**

The `TernaryVecRepr { Sparse(SparseVec), Packed(PackedTritVec) }` adaptive enum with the 25% density crossover is directly applicable to Mycelium's representation tier. It solves the "which representation for this value?" question with a principled, never-silent, always-explicit threshold — aligned with Mycelium's G2/VR-5 discipline.

**C. Three-layer algebraic correction for lossless reconstruction over an approximate base**

This is the highest-value architectural pattern for Mycelium DN-28 (reconstruction-on-render). The pattern: (1) encode in the "native" approximate form; (2) immediately verify via SHA256 hash; (3) store a minimum-cost correction delta (one of five types selected by a simple cost comparator). The 40–62% "Perfect" rate for structured data means this is not just a theoretical win — it is empirically efficient for the exact workloads Mycelium cares about (source code, configs, structured values).

**D. Content-addressed chunk deduplication**

`Codebook.hash_to_id` in `embrfs.rs` (ARCHITECTURE.md line 176) performs chunk-level deduplication via content hash before VSA encoding. This is directly analogous to Mycelium's content-addressed `spore_id` binding (SYNTHESIS Lane E) and the `Provenance::Derived` DAG's identity-is-hash principle.

**E. Optimistic locking over content-addressed chunks (OCC pattern)**

`VersionedEmbrFS`'s `AtomicU64` global version + per-file `VersionedFileEntry.version` + `VersionMismatch` error type is a complete, working implementation of OCC on content-addressed storage. It directly informs RFC-0027's reclamation EXPLAIN record design — the `sweep_epoch` field in SYNTHESIS A-1 is structurally identical to the `global_version` counter.

**F. Hybrid WAL journaling with durability modes**

The fsync-barrier/WAL hybrid from `versioned/journal.rs` (lines 1–100), with three durability modes (Immediate, GroupCommit, Relaxed), is a concrete design for Mycelium's eventual persistent storage layer. The 32-byte header format (magic, version, flags, txn_id, timestamp, payload_len, CRC32) is cache-line-aligned and immediately reusable.

**G. SHA256 truncated to 8 bytes for quick verification**

The `compute_hash([u8; 8])` pattern (CORRECTION.md lines 57–65) — use the first 8 bytes of SHA256 for per-chunk verification, store a full parity trit for additional error detection — is a practical, implementable tamper-evidence primitive that directly supports Mycelium SYNTHESIS Lane C's content-hash pinning proposal (A-5).

### 3.2 Not Directly Transferable

**Codebook/learned basis vectors:** The `Codebook::project()` / `basis_vectors` system is a machine-learning artifact for differential compression. Mycelium does not have a learning phase; its representation swaps are algebraic, not learned. The `ReversibleVSAEncoder` (which replaced the codebook and achieved ~94% accuracy vs. ~0%) is more relevant but is still filesystem-specific.

**FUSE layer:** EmbrFS's FUSE shim is Linux kernel interop; Mycelium's kernel interop path is different.

**`REVERSIBLE_CHUNK_SIZE = 64`:** This is heuristically tuned for filesystem chunks. Mycelium's value granularity is different and this number would need independent calibration.

**Resonator network / beam-limited search:** The retrieval subsystem is for semantic similarity search over holographic engrams. Mycelium's content-addressed retrieval is hash-based (O(1)), not similarity-based.

**GPU acceleration roadmap:** The CUDA/OpenCL plans in `SIMD_DESIGN.md` Phase 3 are embeddenator-specific. Mycelium's `fast` default mode does not assume GPU hardware.

---

## 4. Honest Caveats and Open Follow-Ups

### Confidence caveats

- All throughput/latency numbers in Table 1 are `Declared` accuracy: they are documented in source files by the embeddenator maintainer but are not committed criterion benchmark outputs. No JSON/HTML benchmark results were found in the local clones. Treat them as engineering estimates, not production-validated figures.
- The "~94% uncorrected accuracy" claim for `ReversibleVSAEncoder` (`versioned_embrfs.rs` line 186) is stated in the code comment but not separately verified by a committed property test. It is `Declared`.
- The "~90x speedup" for delta edits (`versioned_embrfs.rs` lines 528–532) is stated in a docstring, not a committed benchmark. It is `Declared`.
- The correction overhead percentages (Table 1, CORRECTION.md lines 266–277) are labeled "Empirical Measurements" in the source doc but the word "empirical" is used loosely — no methodology section describes the measurement conditions. Treated here as `Declared` with a note that the values are plausible given the algorithmic design.

### Gaps that would need follow-up

1. **No committed benchmark outputs.** To ground Table 1 numbers at `Empirical`, run `cargo bench` in `embeddenator-contract-bench-main/` and commit the criterion output. This is a research artifact; the Mycelium team should not treat the numbers as `Empirical` without running the suite.

2. **Correction store merge protocol absent.** The correction store (HashMap<ChunkId, Correction>) has no documented merge/CRDT semantics. For Mycelium's distributed `fuse` scenario (R2/`xloc`), the correction store would need a convergent merge rule. This is an open research question.

3. **`ReversibleVSAEncoder` accuracy claim unverified.** The claim that position-aware binding achieves ~94% uncorrected accuracy is stated without a committed property test in the local clone. For Mycelium to rely on this design, it would need a property test (proptest roundtrip at varying densities).

4. **No measurement of holographic reconstruction latency per chunk.** The decode throughput (100–200 MB/s) covers the full VSA decode + correction application pipeline. The correction selection algorithm (`select_correction`) is O(N) where N = chunk size but its constant factor is not separately benchmarked.

5. **Sweep-order / compaction timing not modeled.** `VersionedEmbrFS` defers reclamation to an explicit `compact` call with no latency bound. This is analogous to Mycelium's SYNTHESIS O-7 (worst-case reclamation latency bound). The embeddenator code does not address this.

---

## 5. Prioritized Crossover Findings for Mycelium

Ranked by actionability against Mycelium's open design questions:

1. **Bitsliced balanced ternary `PackedTritVec` is directly implementable** for Mycelium's ternary representation tier. The design (2 bits/trit, 32 trits/u64, bitplane split, branchless bind/bundle, POPCNT dot product) is complete, correct, and documented at implementation depth. Addresses VSA representation tier open question. Source: `BITSLICED_TERNARY_DESIGN.md` (full file). `Empirical`.

2. **Three-layer algebraic correction is the concrete form of DN-28 reconstruction-on-render.** The pattern (lossy encode → hash verify → minimum-cost delta) is directly applicable to Mycelium's swap provenance and the "representation change as a never-silent, auditable event" design. The 1.8–3.4% overhead for structured data quantifies the cost of guaranteed bit-perfect reconstruction over an approximate VSA base. Source: `CORRECTION.md` lines 266–277, `ARCHITECTURE.md` lines 219–260. `Empirical` (for the overhead percentages).

3. **OCC on content-addressed chunks (`VersionedEmbrFS`) informs RFC-0027 reclamation model.** The `AtomicU64` global version + per-object version + `VersionMismatch` error type is a working implementation of optimistic update on immutable content-addressed values. The `sweep_epoch` in Mycelium's reclamation EXPLAIN record (SYNTHESIS A-1) is structurally identical. Source: `versioned_embrfs.rs` lines 133–218. `Empirical`.

4. **Sparse/dense adaptive strategy (25% crossover) is empirically grounded** and directly applicable to Mycelium's representation tier selection. Source: `BITSLICED_TERNARY_DESIGN.md` lines 235–237, `ARCHITECTURE.md` lines 247–260. `Empirical`.

5. **Hybrid WAL journal design** (fsync for small writes, WAL for multi-file transactions) is a concrete starting point for Mycelium's eventual persistence layer. The durability mode taxonomy (Immediate/GroupCommit/Relaxed) maps cleanly onto Mycelium's `fast`/`certified` mode boundary. Source: `versioned/journal.rs` lines 1–100. `Declared` (design document, not production-profiled).

---

## Meta — Changelog

- **2026-06-24 — Created.** Full-file investigation of 13 embeddenator repos against 6 Mycelium open design questions. Non-normative research artifact. Feeds RFC-0027, DN-28, RFC-0034/E21-1, VSA representation tier design. Per-claim VR-5 tags. Append-only.
