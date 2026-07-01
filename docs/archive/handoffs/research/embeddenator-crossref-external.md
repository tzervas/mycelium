# Cross-Reference Report: tzervas/embeddenator-* vs. Mycelium Open Design Questions

## TL;DR
- The embeddenator series is a real, public Rust workspace ("This workspace contains 8 published library crates implementing holographic data encoding using sparse ternary Vector Symbolic Architecture," per the umbrella README, with CI that "Tests all 11 components in parallel (~20 min)"); its most directly reusable assets for Mycelium are the bitsliced balanced-ternary primitives (embeddenator-vsa) and the immutable-engram + algebraic-correction reconstruction model (embeddenator-fs) — but most headline performance figures in the repos are explicitly labeled "Expected"/"theoretical," not measured.
- Crucially, EmbrFS does NOT implement a true mutable copy-on-write layer over content-addressed values: engrams are immutable read-only snapshots, and mutation is handled at the encoding layer through incremental rebuild operations (add/modify/soft-delete/compact), which maps to Mycelium's reclamation/provenance residue more than to a solved copy/mut problem.
- Exact file:line citations and concrete criterion benchmark numbers could NOT be fully retrieved: GitHub raw/blob source files were gated by the fetch tooling. Verified line counts exist (correction.rs = 531 lines, embrfs.rs = 1,884 lines, fuse_shim.rs = 1,263 lines), but per-line code citations and in-file benchmark tables are flagged as inaccessible rather than fabricated.

## Key Findings

### (1) What the series is — architecture overview
embeddenator is a multi-component Rust workspace. The umbrella repo README (`github.com/tzervas/embeddenator`, "Last Updated: January 26, 2026; Maintained by: @tzervas") states in its Overview: "This workspace contains 8 published library crates implementing holographic data encoding using sparse ternary Vector Symbolic Architecture." Its CI/CD section states "Workspace CI - Tests all 11 components in parallel (~20 min)." The component crates and their crates.io versions (umbrella README legacy table):

- **embeddenator-vsa** — sparse/dense balanced-ternary VSA primitives (bind/bundle/permute/cosine). README lists "Version: 0.21.0 (published on crates.io)"; the repo's latest tagged release is "v0.23.0 - Parallel Batch Encoding," dated Jan 29, 2026 (4 releases total).
- **embeddenator-io** (0.21.0) — I/O, serialization, persistence.
- **embeddenator-obs** (0.21.0) — observability/metrics/tracing.
- **embeddenator-retrieval** (0.21.0) — retrieval/search/similarity.
- **embeddenator-fs** (0.23.0 in umbrella legacy table; repo/crates latest "EmbrFS: FUSE filesystem backed by holographic engrams · v0.25.0 bin+lib") — EmbrFS holographic filesystem + versioning.
- **embeddenator-interop** (0.22.0) — FFI/language interop.
- **embeddenator-cli** (0.21.0) — CLI.
- **embeddenator-core** (0.22.0) — umbrella re-export crate.
- Dev tools: **embeddenator-workspace**, **embeddenator-testkit**, **embeddenator-contract-bench**.

The intended end-state architecture is described in `HOLOGRAPHIC_REFACTOR_PLAN.md`: a trained universal codebook → position-aware holographic encoding (`encoded[i] = bind(pos_vec[i], byte_vec[data[i]])`, then `bundle`) → a root engram ("Holographic superposition of all data") [github](https://github.com/tzervas/embeddenator/blob/main/HOLOGRAPHIC_REFACTOR_PLAN.md) plus a ≤6% structural manifest. The private 27-trit ISA repo **tritium-program** was not accessible and is noted as such.

### (2) Per-question crossover findings

**Q1 — COPY/MUT (mutable layer over immutable engrams).**
*Measured/established:* The embeddenator-fs README is explicit that engrams are immutable: under "What EmbrFS IS NOT" it lists "A writable filesystem (holographic engrams are immutable snapshots)" and "No write/modify operations through FUSE (modifications require re-encoding)"; [github](https://github.com/tzervas/embeddenator-fs) FUSE is "Read-only … (by design - engrams are immutable)." [github](https://github.com/tzervas/embeddenator-fs) Mutation is implemented at the encoding layer via incremental operations enumerated in the README: `add_files` ("Add new files without full rebuild"), `modify_files` ("Update existing files"), `remove_files` ("Soft-delete files"), and `compact` ("Hard rebuild to reclaim space"). [github](https://github.com/tzervas/embeddenator-fs) `HOLOGRAPHIC_REFACTOR_PLAN.md` confirms a `VersionedEmbrFS` write layer exists and currently "Uses 100% verbatim corrections (passthrough storage)," [github](https://github.com/tzervas/embeddenator/blob/main/HOLOGRAPHIC_REFACTOR_PLAN.md) and lists `src/fs/versioned_embrfs.rs` and `src/fs/versioned/manifest.rs` among files to modify.
*Assessment:* Best characterized as a **soft-delete delta + hard-rebuild (compact) model** — a delta-journal/log-structured approach rather than classic page-level copy-on-write or refcount-based structural sharing. *Speculative inference:* the soft-delete + compact pattern is closest to a log-structured append-then-GC scheme.
*Not retrievable:* The struct definition of `VersionedEmbrFS`, its write/add/modify method bodies, and the benchmarked cost of a single mutation could not be fetched (source file gated). README "Expected Performance" gives "Incremental adds: ~1-5ms per file" and "Compaction: Similar to full re-ingestion" [github](https://github.com/tzervas/embeddenator-fs) — explicitly *expected*, not measured.

**Q2 — RECLAMATION / memory model.**
*Measured/established:* embeddenator-vsa `Cargo.toml` declares dependencies `rayon` (data-parallelism), `rustc-hash`, `serde`, `sha2`, and optional `cudarc` (CUDA). The fs README architecture shows a layered design (FUSE shim → core → correction → VSA primitives) with LRU caching ("LRU caching reduces repeated disk I/O"). [github](https://github.com/tzervas/embeddenator-fs)
*Assessment / speculative:* There is **no evidence of Perceus-style uniqueness tracking or region/arena allocation** in any retrieved artifact; the design appears to be standard Rust ownership plus structural sharing of immutable engrams and an LRU cache. The "soft-delete then compact" model is the reclamation mechanism — reclamation is deferred and batched, not reference-counted per value.
*Not retrievable:* Whether `Arc`/`RwLock`/`Mutex` wrap the engram store could not be confirmed from source (file gated). This is the thinnest-sourced question.

**Q3 — CONTENT-ADDRESSED STORAGE + reconstruction-on-render + tamper-evidence.**
*Measured/established:* The fs README states "Bit-Perfect Reconstruction: 100% accurate file recovery via correction layer," composed of "Primary SparseVec encoding," "Immediate verification on encode," and an "Algebraic correction store for exact differences." [github](https://github.com/tzervas/embeddenator-fs) Four correction strategies are named verbatim: `BitFlips` ("Sparse bit-level corrections"), `TritFlips` ("Ternary value corrections"), `BlockReplace` ("Contiguous region replacement"), `Verbatim` ("Full data storage (fallback)"). [github](https://github.com/tzervas/embeddenator-fs) The reconstruction logic lives in `src/correction.rs` (README "File Structure" gives it as 531 lines). embeddenator-vsa `Cargo.toml` declares `sha2` (">=0.10, <1.0"), [github](https://github.com/tzervas/embeddenator-vsa/blob/main/Cargo.toml) consistent with SHA256-based verification. `HOLOGRAPHIC_REFACTOR_PLAN.md` reports the underlying encoder accuracy: "ReversibleVSAEncoder: Achieves 94.12% accuracy with chunked encoding," [github](https://github.com/tzervas/embeddenator/blob/main/HOLOGRAPHIC_REFACTOR_PLAN.md) and a chunk-size/accuracy table (1 byte → 100%, 8 bytes → 94%, 64 bytes → 85-90%, 256 bytes → 70-80%). [github](https://github.com/tzervas/embeddenator/blob/main/HOLOGRAPHIC_REFACTOR_PLAN.md) The 100% figure is achieved by layering the algebraic correction store on top of the ~94% raw VSA reconstruction.
*Assessment:* Tamper-evidence = SHA256 over reconstructed bytes + parity trits; fidelity = 100% after correction, ~94.12% before correction at 8-byte chunks.
*Not retrievable:* The exact SHA256/parity-trit verification code with line numbers, and any criterion-measured reconstruction throughput, could not be fetched (docs/CORRECTION.md was gated).

**Q4 — CRDT/merge ("holographic superposition of directory trees").**
*Measured/established:* `HOLOGRAPHIC_REFACTOR_PLAN.md` describes the root engram as a "Holographic superposition of all data" produced by `bundle`. The bitsliced bundle operation is a saturating commutative join over balanced ternary (BITSLICED_TERNARY_DESIGN.md truth table: P+P=P, N+N=N, P+N=Z, Any+Z=Any).
*Assessment / speculative:* bundle is **commutative and associative** (elementwise saturating ternary superposition), the algebraic prerequisite for a semilattice/CRDT join. However, **bundle is lossy and not idempotent in the strong-eventual-consistency sense** (bundling already-bundled vectors degrades retrievability as density rises), and there is **no evidence in any retrieved artifact of an explicit `fuse`-style semilattice-merge operator or a convergence/SEC proof**. The "holographic superposition of directory trees" is therefore **architectural intent / an encoding property, not a demonstrated CRDT** with idempotent convergent merge.
*Not retrievable:* No merge/`fuse` operator source could be located.

**Q5 — VSA/TERNARY (reusable for Mycelium's representation tier).**
*Measured/established (best-sourced question):* `docs/BITSLICED_TERNARY_DESIGN.md` (496 lines, v0.20.0-alpha.1, "Status: Implementation Complete") [github](https://github.com/tzervas/embeddenator-vsa/blob/main/docs/BITSLICED_TERNARY_DESIGN.md) fully specifies the reusable primitives:
- Balanced ternary {-1,0,+1} = {N,Z,P}; **2 bits per trit**, **32 trits per u64 word**, encoding `00=Z, 01=P, 10=N, 11=reserved→Z`. [github](https://github.com/tzervas/embeddenator-vsa/blob/main/docs/BITSLICED_TERNARY_DESIGN.md)
- Bitplane separation using `const MASK_EVEN_BITS: u64 = 0x5555_5555_5555_5555;` — P plane = `word & MASK`, N plane = `(word >> 1) & MASK`. [github](https://github.com/tzervas/embeddenator-vsa/blob/main/docs/BITSLICED_TERNARY_DESIGN.md)
- `dot` (cosine) via popcount: `acc += (pp + nn) - (pn + np)` over `count_ones()` of plane intersections. [github](https://github.com/tzervas/embeddenator-vsa/blob/main/docs/BITSLICED_TERNARY_DESIGN.md)
- `bind` (elementwise multiply): `same = (a_pos & b_pos) | (a_neg & b_neg); opp = (a_pos & b_neg) | (a_neg & b_pos); out = same | (opp << 1)`. [github](https://github.com/tzervas/embeddenator-vsa/blob/main/docs/BITSLICED_TERNARY_DESIGN.md)
- `bundle` (saturating add) via branchless bitwise logic.
- Adaptive `enum TernaryVecRepr { Sparse(SparseVec), Packed(PackedTritVec) }` with a 25%-density crossover. [github](https://github.com/tzervas/embeddenator-vsa/blob/main/docs/BITSLICED_TERNARY_DESIGN.md) The vsa README corroborates: "SparseVec: For sparse vectors (< 25% density) — Memory: 8 bytes per non-zero trit"; "PackedTritVec: For dense vectors (≥ 25% density) — Memory: 2 bits per trit ... (32 trits per u64)."
- Complexity (README): dot = O(n/32) packed vs O(k log k) sparse; [GitHub](https://github.com/tzervas/embeddenator-vsa) bind/bundle = O(n/32) packed vs O(k) sparse. [GitHub](https://github.com/tzervas/embeddenator-vsa)
*Assessment:* The bitsliced ternary kernel (MASK_EVEN_BITS bitplane technique, branchless bind/bundle/dot, adaptive sparse/dense switch) is **directly reusable** for Mycelium's binary/ternary/dense/VSA representation tier. Source files are `src/ternary.rs`, `src/ternary_vec.rs`, `src/block_sparse.rs` (block-sparse feature confirmed in Cargo.toml: `block-sparse=[]`), [github](https://github.com/tzervas/embeddenator-vsa/blob/main/Cargo.toml) `src/simd_cosine.rs`, `src/gpu.rs`.
*Not retrievable:* exact line numbers within `ternary.rs`/`ternary_vec.rs`/`block_sparse.rs` (source files gated); the design-doc code blocks are authoritative for the algorithms but not line-anchored to the .rs files.

**Q6 — BENCHMARKS (headline).** See table below. *Critical caveat:* every published throughput/latency figure I could retrieve is labeled "Expected Performance" (fs README) or "theoretical" (vsa README, verbatim: "Performance characteristics are theoretical and depend on hardware, data patterns, and compiler optimizations. Run `cargo bench` to measure on your specific system."). The only hard, non-hedged numbers are accuracy/structural figures from HOLOGRAPHIC_REFACTOR_PLAN.md and the design docs. Actual criterion benchmark outputs (ns/op) from `benches/` (vsa: `vsa_ops`, `simd_cosine`, `cuda`; fs: `ingest_benchmark`, `query_benchmark`, `incremental_benchmark`), `examples/gpu_benchmark.rs`, embeddenator-obs `performance_benchmark.rs`, and `embeddenator-contract-bench` could NOT be retrieved and are flagged accordingly.

### (3) Benchmark-numbers table (measured value + source file)

| Metric | Value | Source file | Status |
|---|---|---|---|
| Raw VSA reconstruction accuracy (8-byte chunks) | 94.12% | HOLOGRAPHIC_REFACTOR_PLAN.md | Established (stated) |
| Reconstruction accuracy by chunk size | 1B→100%, 8B→94%, 64B→85-90%, 256B→70-80% | HOLOGRAPHIC_REFACTOR_PLAN.md | Established (design table) |
| Bit-perfect reconstruction after correction | 100% | embeddenator-fs README | Claimed |
| Correction/space overhead | 0-5% typical | embeddenator-fs README | Stated ("Observed") |
| Ingestion throughput | 20-50 MB/s debug, 50-100+ MB/s release | embeddenator-fs README | EXPECTED (projection) |
| Extraction throughput | 50-100 MB/s debug, 100-200+ MB/s release | embeddenator-fs README | EXPECTED (projection) |
| Incremental add latency | ~1-5 ms/file | embeddenator-fs README | EXPECTED (projection) |
| Queries | sub-ms small codebook, ms large | embeddenator-fs README | EXPECTED (projection) |
| Default chunk size | 4 KB | embeddenator-fs README | Established |
| Storage density (dense) | 2 bits/trit, 32 trits/u64 | BITSLICED_TERNARY_DESIGN.md | Established |
| Storage density (sparse) | 8 bytes/non-zero trit | BITSLICED_TERNARY_DESIGN.md / vsa README | Established |
| Density crossover | 25% | BITSLICED_TERNARY_DESIGN.md | Established |
| vsa test count | "53+ tests passing (unit + integration + doc tests)" | embeddenator-vsa README | Established |
| fs test count | 20 tests | embeddenator-fs README | Established |
| Source sizes | embrfs.rs 1,884; fuse_shim.rs 1,263; correction.rs 531 lines | embeddenator-fs README | Established |
| Theoretical SIMD speedup | AVX2 128 trits/instr, NEON 64 trits/instr | BITSLICED_TERNARY_DESIGN.md | Theoretical |
| GPU speedup (large vectors) | 10-100x (>10M dims) | BITSLICED_TERNARY_DESIGN.md | Theoretical/future |
| Concrete criterion ns/op numbers | NOT RETRIEVED | benches/, gpu_benchmark.rs, contract-bench | Inaccessible |

### (4) What transfers to Mycelium vs. what is embeddenator-specific

**Transfers well:**
- The bitsliced balanced-ternary kernel (MASK_EVEN_BITS bitplane separation, branchless bind/bundle/dot via popcount, adaptive sparse/dense representation at 25% density) — directly maps to Mycelium's binary/ternary/dense/VSA representation tier.
- The immutable-engram + algebraic-correction reconstruction model maps to Mycelium's "reconstruction-on-render" and content-addressed registry; the SHA256+parity tamper-evidence maps to "structural identity = hash."
- The refinement-certificate concept has an analog: the "≤6% manifest overhead" + correction store is a "this artifact reconstructs reference within a stated bound" relationship.
- The chunk-size/accuracy table is a concrete instantiation of Mycelium's tunable certification lattice (Exact ⊇ Proven ⊇ Empirical ⊇ Declared): chunk size trades exactness for speed.

**Embeddenator-specific / does NOT transfer cleanly:**
- bundle is lossy and non-idempotent → it is NOT a drop-in CRDT/semilattice join for Mycelium's reserved `fuse`. The "holographic superposition" is encoding intent, not a convergence-proven merge.
- The VersionedEmbrFS write layer is currently passthrough/verbatim (HOLOGRAPHIC_REFACTOR_PLAN.md: "Uses 100% verbatim corrections") [github](https://github.com/tzervas/embeddenator/blob/main/HOLOGRAPHIC_REFACTOR_PLAN.md) — it does not yet demonstrate efficient structural-sharing mutation, so it does not yet solve Mycelium's unified copy/mut + reclamation + merge problem.
- No uniqueness/region/Perceus machinery exists to transfer for Mycelium's affine Substrate/consume model.

## Details
The strongest grounded artifact is `docs/BITSLICED_TERNARY_DESIGN.md`, which is implementation-complete and fully specifies the ternary algebra and bit-level encoding Mycelium would reuse. `HOLOGRAPHIC_REFACTOR_PLAN.md` is the authoritative source for the system's accuracy numbers and confirms the internal `src/fs/` layout (versioned_embrfs.rs, correction.rs, versioned/manifest.rs) referenced in the task, even though the fs README presents a flatter `src/` view (embrfs.rs/fuse_shim.rs/correction.rs) — a discrepancy likely reflecting README staleness vs. the refactor branch. The fs system today is read-only over immutable snapshots; "mutation" is re-encoding. This is the central finding for Mycelium: embeddenator validates that an immutable + content-addressed + hash-verified store is buildable and reconstructs bit-perfectly, but it has NOT yet demonstrated the unified copy/mut + reclamation + CRDT-merge solution — it currently sidesteps mutation (re-encode) and merge (lossy bundle). This is itself informative: it empirically supports the research synthesis's claim that the hard residue of an immutable/content-addressed model is PROVENANCE + reclamation, because that is exactly the part embeddenator has not yet solved efficiently.

## Recommendations
1. **Adopt the bitsliced ternary kernel now.** Port `src/ternary.rs` + `BITSLICED_TERNARY_DESIGN.md`'s MASK_EVEN_BITS scheme into Mycelium's representation tier as the reference dense-ternary encoding. Threshold to revisit: if measured bind/bundle throughput on target hardware is <10x scalar, prioritize the explicit-SIMD path before committing.
2. **Treat embeddenator's reconstruction model as the template for reconstruction-on-render**, but require an actual measured fidelity/throughput run (`cargo bench` in embeddenator-fs/benches) before relying on the 100%/0-5%-overhead claims — they are currently unverified projections. Benchmark threshold: if measured release-mode reconstruction is <50 MB/s or correction overhead >5% on representative data, the model is not yet production-ready for Mycelium's registry.
3. **Do NOT model Mycelium's `fuse` on bundle.** Because bundle is lossy/non-idempotent, design `fuse` as a true semilattice join over content hashes (set-union of immutable values), reserving bundle only for approximate similarity/superposition, not authoritative merge.
4. **For copy/mut + reclamation**, treat the VersionedEmbrFS soft-delete+compact pattern as a baseline (log-structured delta + batched GC) and improve on it with structural sharing keyed on content hash — embeddenator does not yet do this. This is where Mycelium can advance past embeddenator rather than merely reuse it.
5. **Re-attempt source extraction via `git clone`** (not web fetch) to obtain exact file:line citations for versioned_embrfs.rs and correction.rs before any code-level port; the verified line counts (531/1,884/1,263) confirm the files exist and are substantial.

## Caveats
- **Source-line citations are incomplete by necessity.** GitHub raw/blob source files were gated by the fetch tooling (raw.githubusercontent.com blocked entirely; blob URLs only fetchable when surfaced by a prior search). Per-line code citations for .rs files and the in-file benchmark tables in docs/CORRECTION.md, docs/FUSE.md, docs/ARCHITECTURE.md could not be retrieved. I have cited file paths and verified line counts but have NOT fabricated line numbers. A dedicated subagent confirmed the same constraint independently.
- **Headline performance numbers are mostly projections.** The fs README explicitly labels throughput/latency figures "Expected Performance"; the vsa README labels its complexity figures "theoretical" (verbatim quote above). Only accuracy/structural figures are stated as established.
- **embeddenator-contract-bench, embeddenator-obs benchmark sources, and examples/gpu_benchmark.rs were not retrievable** — their concrete numbers are unknown, not zero.
- **tritium-program (private 27-trit ISA) was inaccessible** as anticipated.
- **Version skew:** the umbrella README legacy table lists older versions (e.g., vsa 0.21.0, fs 0.23.0) than the repos/crates themselves (vsa release v0.23.0 dated Jan 29 2026; fs v0.25.0); figures may have moved between snapshots.
