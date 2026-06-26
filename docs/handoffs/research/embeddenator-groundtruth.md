# Embeddenator Ground-Truth Correction Pass (Authoritative Full-Source)

| Field | Value |
|---|---|
| **Role** | Opus ground-truth + reconciliation agent — corrects the prior crossover with real file:line |
| **Status** | Research artifact — **non-normative**. Corrects `embeddenator-crossover.md` (prior, partial) and refines `SYNTHESIS-wave2-addendum.md`. Feeds RFC-0027, DN-28, RFC-0034/E21-1, VSA representation tier. |
| **Date** | 2026-06-24 |
| **Sources reconciled** | (1) AUTHORITATIVE CODE `/tmp/embr-auth/<repo>-main/` (full files, real file:line — read this pass); (2) maintainer's EXTERNAL web-session report (caveated, no file:line — copied verbatim to `embeddenator-crossref-external.md`); (3) PRIOR crossover `embeddenator-crossover.md` (sonnet, partial — **over-read** several items). |
| **Confidence** | Per-claim VR-5. Every number cites the file I read it in. **Posture: skeptical — the prior pass over-claimed; this pass down-tags rather than repeats its optimism.** |

> **Headline correction (the one that matters most).** The prior crossover called
> `VersionedEmbrFS` mutation **"chunk-level copy-on-write"** with **"unaffected chunks share storage
> unchanged"** and **"structural sharing"** (`embeddenator-crossover.md` §Q1, lines 49–51, §3.1.E).
> The authoritative code shows this is **false as stated**: a mutation **decodes the affected chunk,
> edits the plaintext, RE-ENCODES the whole chunk, and OVERWRITES the same `chunk_id` slot in place**
> (`versioned_embrfs.rs:744–788`, `apply_byte_replace`; the store is a single
> `HashMap<ChunkId, Arc<VersionedChunk>>` — `chunk_store.rs:38` — with `insert` overwriting, not
> appending a version — `chunk_store.rs:114`). There is **no MVCC, no snapshot retention, no
> structural-sharing tree**. "Copy-on-write" and "structural sharing" are the prior pass's optimistic
> gloss. The corrected verdict matches the external report: embeddenator **sidesteps efficient
> copy/mut by re-encoding the touched chunk**, it does not solve it. **This STRENGTHENS the synthesis
> thesis** ("the residue of an immutable/content-addressed model is provenance + reclamation"):
> copy/mut is exactly the part embeddenator has *not* solved efficiently.

---

## A. COPY/MUT — does embeddenator SOLVE or SIDESTEP efficient copy/mut?

### (i) What the prior crossover/synthesis claimed
- Prior crossover §Q1 (lines 49–51): *"`VersionedEmbrFS` does not mutate the immutable engram in
  place … **Chunk-level copy-on-write.** Every write operation … re-encodes only the affected 4KB
  chunks and inserts new versioned chunks alongside corrections. **Unaffected chunks share storage
  unchanged** (`VersionedChunkStore` …)."*
- Prior crossover §3.1.E: *"Optimistic locking over content-addressed chunks … a **complete, working
  implementation of OCC on content-addressed storage**."*
- Wave-2 addendum CF-2 (`SYNTHESIS-wave2-addendum.md`:63–68): carried this forward as *"chunk-level
  CoW over a content-addressed immutable base is the natural copy/mut path"* with the *"~90× speedup"*
  number.

### (ii) What the authoritative code actually shows (file:line)
- **Mutation = decode → edit plaintext → re-encode whole chunk → overwrite same slot.** In
  `apply_byte_replace` (`versioned_embrfs.rs:725–804`): get+decode the chunk (`:744–752`), apply the
  byte to the decoded plaintext (`:760–765`), **`encode_chunk(&modified, …)` re-encodes the entire
  chunk** (`:768`), recompute correction (`:771–773`), then **`chunk_store.insert(chunk_id, new_chunk,
  store_version)`** (`:787–788`). The same shape repeats in `apply_multi_byte_replace`
  (`:807–892`), `apply_append` (`:953–1091`), and `apply_truncate` (`:1094–1195`).
- **The store is single-slot-per-id, overwrite-on-write — not versioned history.** `VersionedChunkStore`
  is `chunks: Arc<RwLock<HashMap<ChunkId, Arc<VersionedChunk>>>>` (`chunk_store.rs:38`); `insert`
  does `chunks.insert(chunk_id, Arc::new(chunk))` (`chunk_store.rs:114`) — it **replaces** the prior
  `Arc<VersionedChunk>` at that id. The word "Versioned" refers to an **OCC version *counter*** for
  conflict detection (`global_version: AtomicU64`, `chunk_store.rs:42`; `VersionMismatch` check
  `:102–108`), **not** to retained version snapshots.
- **"Unaffected chunks share storage" is technically true but trivial and misframed.** Untouched
  chunks keep their `chunk_id` in the file entry's `chunks: Vec<ChunkId>` (`manifest.rs:257`) and are
  simply not revisited — that is "we only re-encoded the touched chunk," **not** CoW structural
  sharing of a persistent tree. The manifest itself overwrites in place: `update_file` does
  `files[idx] = new_entry` (`manifest.rs:1000`), retaining no prior entry.
- **The refcount/dedup machinery exists but is DEAD in the mutation path.** `VersionedChunk` carries a
  `ref_count: Arc<AtomicU32>` with `inc_ref`/`dec_ref`/`is_unreferenced` (`chunk.rs:31–90`), the store
  has a `hash_index: HashMap<[u8;8], ChunkId>` for dedup with `find_by_hash` (`chunk_store.rs:46,
  77–86`), and there is a `gc()` that removes `is_unreferenced()` chunks (`chunk_store.rs:240–275`).
  **None of it is wired in**: a repo-wide grep shows `inc_ref`/`dec_ref`/`find_by_hash` are referenced
  **only inside their own definition files** (`chunk.rs`, `chunk_store.rs`) — no write/append/truncate/
  delete path calls them. `ref_count` is initialised to `1` (`chunk.rs:51`) and **never decremented**,
  so `gc()` can collect **nothing** in the versioned path — it is structurally a no-op. There is **no
  `VersionedEmbrFS::compact`** at all (only `VersionedManifest::compact` — `manifest.rs:1075` — which
  drops *deleted manifest entries*, leaving their chunks; and a separate *immutable*-path
  `EmbrFS::compact` at `embrfs.rs:1333`).
- **Delete = soft-delete flag, no reclamation.** `delete_file` calls `manifest.remove_file` which sets
  `deleted = true` (`manifest.rs:1032`); chunks are never freed.
- **The maintainer's own plan confirms passthrough.** `HOLOGRAPHIC_REFACTOR_PLAN.md:14`:
  *"VersionedEmbrFS: Uses 100% verbatim corrections (passthrough storage)"* with TODOs to
  *"Replace VersionedEmbrFS encoding with ReversibleVSAEncoder"* and *"Remove verbatim correction
  layer"* (`:293–294`). (Code nuance: the read path *does* call the real VSA encoder when
  `holographic_mode` is on — `encode_chunk` → `ReversibleVSAEncoder` — `versioned_embrfs.rs:658–664`;
  but the default constructor is **legacy/non-holographic** — `with_config_and_profiler` sets
  `holographic_mode: false`, `:215` — so by default the correction is the byte-diff `compute_correction`
  which for fully-mismatched legacy output falls to `Verbatim` = the original bytes, i.e. passthrough.)

### (iii) Corrected conclusion + impact on the thesis
**Embeddenator SIDESTEPS efficient copy/mut; it does not solve it.** Mutation is **re-encode-the-
touched-chunk + overwrite-in-place**, guarded by an **OCC version counter** — not copy-on-write, not
MVCC, not structural sharing, and not refcount-driven reuse (the refcount/dedup/GC code is present but
**unwired and effectively inert**). This **confirms the external report** (its §Q1 "soft-delete delta +
hard-rebuild … not classic page-level CoW or refcount-based structural sharing") and **refutes the prior
crossover's "chunk-level copy-on-write / structural sharing" framing**.

**Impact: STRENGTHENS the synthesis thesis.** Wave-2's CF-2 should be re-scoped: the running evidence
is that embeddenator **re-encodes** on edit and **defers/forgoes** reclamation — precisely the
"unsolved residue" the cluster identifies (provenance + reclamation). The maintainer reached for OCC at
the multi-writer layer and left in-place reuse / GC unwired; this is the *empirical* form of "copy/mut +
reclamation is the hard part," not evidence that CoW is already solved. The wave-2 down-tag of the
"~90× speedup" to `Declared` (CL-1 / addendum:229–242) was correct and stands — that number is a
docstring table (`versioned_embrfs.rs:526–530`), not a measured result.

---

## B. BENCHMARKS — measured numbers, or only "Expected"/harness?

### (i) What the prior crossover/synthesis claimed
- Prior crossover §2 "Q6" Table 1 presented ~30 throughput/latency rows tagged `Declared`/`Empirical`,
  including three rows citing **`performance_benchmark.rs:40/69/91`** as asserted `<20 ns`/`<200 ns`/
  `<100 ns` "targets," plus correction-overhead rows tagged `Empirical`.
- The wave-2 addendum already down-tagged the "1.8–3.4 % overhead is cheap" claim to
  `Declared`/data-scoped (CL-1). Good — but the broader table still implied a grounded perf envelope.

### (ii) What the authoritative code actually shows (file:line)
- **No committed measured numbers anywhere.** No criterion output (`estimates.json`/`benchmark.json`/
  `criterion/`) is committed in any of the nine authoritative repos (searched; none found).
- **The benches are harnesses, not assertions.** vsa `benches/{vsa_ops,simd_cosine,cuda}.rs` +
  `examples/gpu_benchmark.rs`; fs `benches/{filesystem_benchmarks,incremental_benchmark,ingest_benchmark,
  query_benchmark}.rs`; contract-bench `benches/{vsa_ops,simd_cosine,fs_operations,hierarchical_scale,
  io_operations,query_hierarchical,retrieval_index}.rs` — all are criterion `bench_function`/`black_box`
  timing harnesses with **no asserted ns/op thresholds**.
- **The testkit "validation" benches assert NOTHING about performance.** `performance_validation.rs` and
  `optimization_validation.rs` are pure criterion harnesses — grep for `assert|threshold|target|< Nns`
  returns **zero** perf assertions in either. (The closest-to-measured-data hope in the task brief does
  not pan out: they time, they do not assert a budget.)
- **The only `assert_eq!`s in contract-bench are determinism/correctness checks**, not perf budgets
  (`contract-bench/src/dataset.rs:487–591` — round-trip `pos/neg` equality, dataset counts/seeds).
- **The vsa README is explicit the figures are theoretical**: *"Performance characteristics are
  theoretical and depend on hardware … Run `cargo bench` to measure on your specific system"*
  (`embeddenator-vsa-main/README.md:106`).
- **The prior table cited a file that is not in the ground-truth corpus.** `embeddenator-obs` is **not**
  among the nine authoritative repos, and **no `performance_benchmark.rs` exists anywhere** in
  `/tmp/embr-auth/`. The prior crossover's three "asserted threshold" rows
  (`performance_benchmark.rs:40/69/91`) are therefore **unverifiable from ground truth** and must not be
  carried.

### (iii) Benchmark-status verdict
**There are NO committed measured benchmark numbers in the authoritative source.** Every
throughput/latency/overhead figure the prior crossover tabulated is either (a) a **doc/README
projection** explicitly labelled "Expected"/"theoretical," (b) a **docstring table**
(`versioned_embrfs.rs:526–530`), or (c) a **design-doc estimate** (`CORRECTION.md`, `ARCHITECTURE.md`,
`BITSLICED_TERNARY_DESIGN.md`). The benches exist and are well-structured, but produce numbers **only at
runtime** and **none are committed**. The testkit "validation" benches **do not** assert thresholds.
**Corrected tag for the entire perf envelope: `Declared` — and for the three `performance_benchmark.rs`
rows, `Unverifiable` (source file absent from the corpus).** This **confirms the external report** ("most
headline performance figures are explicitly labeled 'Expected'/'theoretical', not measured") and tightens
the prior crossover's softer "Table 1 are design-time targets" caveat into a flat verdict: **no measured
data exists to cite.**

The **structural/encoding** numbers remain solid (`Exact`), because they are properties of the encoding,
not benchmarks: 2 bits/trit, 32 trits/u64, `MASK_EVEN_BITS = 0x5555_5555_5555_5555`, balanced-ternary
ranges — all confirmed in code below (§C).

---

## C. TERNARY KERNEL — the directly-reusable asset (anchored to real .rs file:line)

### (i) What the prior crossover/external report claimed
Both attributed the bitsliced primitives (2-bit packing, `MASK_EVEN_BITS` bitplane split, branchless
bind/bundle, popcount dot, 25 % sparse/dense crossover, `enum TernaryVecRepr`) to
`BITSLICED_TERNARY_DESIGN.md` — a **design doc** — and the external report explicitly flagged it
*could not* line-anchor them to the `.rs` files ("source files gated"). Prior crossover §3.1.A–B
presented `enum TernaryVecRepr { Sparse(SparseVec), Packed(PackedTritVec) }` with a 25 % auto-crossover
as if implemented.

### (ii) What the authoritative code actually shows (file:line) — now line-anchored
- **The bitsliced kernel is REAL in `ternary_vec.rs`** (not doc-only), with scalar + AVX2 + AVX512 paths:
  - **Struct + mask:** `pub struct PackedTritVec { len, data: Vec<u64> }` (`ternary_vec.rs:66–70`);
    `const MASK_EVEN_BITS: u64 = 0x5555_5555_5555_5555;` (`:73`).
  - **2-bit lane packing, 32 trits/u64:** word count = `(len*2).div_ceil(64)` (`:95–98`); lane encoding
    `01 ⇒ P` (even bit), `10 ⇒ N` (odd bit), `00 ⇒ Z` (`get` `:129–141`, `set` `:143–157`,
    `fill_from_sparsevec` `:168–189`).
  - **Bitplane separation:** `pos = word & MASK_EVEN_BITS; neg = (word >> 1) & MASK_EVEN_BITS`
    (`:210–211`, and again at `:298–301`, `:388–391`, `:517–520`, `:587–590`, `:662–665`, `:736–739`).
  - **Branchless `bind` (elementwise multiply):** `same = (a_pos&b_pos)|(a_neg&b_neg);
    opp = (a_pos&b_neg)|(a_neg&b_pos); out = same | (opp << 1)` (`bind_scalar` `:572–595`; dispatcher
    `bind` `:539`; AVX512 `:613`, AVX2 `:687`, `bind_into` `:754`).
  - **`dot` via popcount:** `pp=(a_pos&b_pos).count_ones(); nn=(a_neg&b_neg).count_ones();
    pn=(a_pos&b_neg).count_ones(); np=(a_neg&b_pos).count_ones(); acc += (pp+nn) − (pn+np)`
    (`dot_scalar` `:298–306`; dispatcher `dot` `:254`; AVX512 `:322`, AVX2 `:413`).
  - **`bundle` (saturating add):** dispatcher `:798`, scalar `:831`, AVX512 `:877`, AVX2 `:964`,
    `bundle_into` `:1042`.
- **Scalar trit algebra is in `ternary.rs`** (the proven single-trit layer, distinct from the bitsliced
  vector layer): `enum Trit { N=-1, Z=0, P=1 }` (`:47–55`); `mul` truth table with self-inverse
  `a×a=P` (`:159–165`); `Tryte3` (3 trits, 27 states, `:257–456`); `Word6` (`:478–570`); `ParityTrit`
  = `sum(trits) mod 3` balanced (`:611–622`). These carry exhaustive correctness tests
  (`:636–827` — full mul truth table, commutativity/associativity, 27-case add-with-carry, roundtrips).

### (iii) Corrected conclusion
The bitsliced ternary kernel **is implemented in code** and is the cleanest directly-portable asset —
**STRENGTHENED** vs the prior pass because it is now line-anchored to `ternary_vec.rs`/`ternary.rs`
rather than to a design doc. **One correction (down-tag):** the **`enum TernaryVecRepr` adaptive
selector with an automatic 25 % crossover does NOT exist as code.** The 25 % rule is **guidance only**
— a module-header recommendation (`ternary_vec.rs:33–39`: "Use PackedTritVec when density ≥ 25 % … Use
SparseVec when density < 25 %") and a design-doc threshold. Both concrete types (`SparseVec` in `vsa.rs`,
`PackedTritVec` in `ternary_vec.rs`) exist and `from_sparsevec` bridges them (`ternary_vec.rs:159`), but
**representation selection is left to the caller** — there is no runtime auto-switching enum. So Mycelium
would *design* the never-silent selector itself; embeddenator supplies the two representations and the
empirical threshold, not the switch. (Refines wave-2 AD-2 / W2-A5: adopt the **bitsliced design + the
25 % threshold as a derivable guideline**, but build the auto-selector — it is not inherited.)

---

## D. CORRECTION / RECONSTRUCTION — strategies + tamper-evidence (file:line)

### (i) What the prior crossover/external report claimed
Prior crossover §Q3 / external §Q3: a "three-layer" / "five correction types" model
(`Perfect/BitFlips/TritFlips/BlockReplace/Verbatim`) with SHA256-trunc-8 + parity-trit verification,
40–62 % "Perfect" rate, 1.8–3.4 % overhead on structured data. External report correctly flagged it
could not retrieve `correction.rs` line-anchored.

### (ii) What the authoritative code actually shows (`correction.rs`, 539 lines)
- **Reconstruction invariant (the real model):** `D = decode(E) + R` by construction — module docstring
  `correction.rs:31–40`: *"If decode(E)=D then R=0 … else R=D−decode(E) (exact correction stored).
  Either way D is perfectly recoverable."* This is the concrete reconstruction-on-render mechanism.
- **Correction enum — 5 variants DECLARED:** `enum CorrectionType { None, BitFlips(Vec<(u64,u8)>),
  TritFlips(Vec<(u64,Trit,Trit)>), BlockReplace{offset,original}, Verbatim(Vec<u8>) }`
  (`correction.rs:48–60`).
- **CORRECTION (VR-5): only 4 of the 5 are ever PRODUCED.** The selector `compute_correction`
  (`:186–241`) emits **`None`** (identical, `:188–189`), **`Verbatim`** (if `diff_count >
  original.len()/2`, `:213–214`), **`BlockReplace`** (if `diff_count > 10` and the diff span is tight,
  `:218–231`), else **`BitFlips`** (XOR mask per differing byte, `:234–240`). **`TritFlips` is declared
  and has an `apply` arm (`:111–131`) but is NEVER generated by `compute_correction`** — it is dead on
  the encode side. The prior/external "5 strategies" should be stated as **"5 declared, 4 active."**
- **Per-chunk record:** `ChunkCorrection { chunk_id, correction, hash: [u8;8], parity: Trit }`
  (`:63–73`); built by `ChunkCorrection::new` which computes `hash = compute_hash(original)` and
  `parity = compute_data_parity(original)` (`:77–89`).
- **Tamper-evidence is concrete:** `compute_hash` = **first 8 bytes of SHA256** (`:164–172`);
  `verify(result)` = `compute_hash(result) == self.hash` (`:148–150`); `compute_data_parity` =
  `sum(bytes) mod 3` mapped to a balanced `Trit` (`:174–183`). `CorrectionStore::apply` **re-verifies
  after applying** and returns `None` on hash mismatch (`:290–300`) — a never-silent failure, not a
  silent fallthrough. A standalone `ReconstructionVerifier` checks chunk hashes en masse (`:362–410`).
- **Reconstruction-on-render in the read path:** `read_file` decodes each chunk then
  `corrections.get(id).map(|(c,_)| c.apply(&decoded)).unwrap_or(decoded)` (`versioned_embrfs.rs:304–311`)
  — the rendered bytes are decode(VSA) + correction, exactly the DN-28 shape.

### (iii) Corrected conclusion
The reconstruction-on-render + tamper-evidence model is **real and confirmed** (SHA256-trunc-8 +
parity-trit + post-apply re-verify; `D = decode(E)+R` invariant) — **STRENGTHENS** wave-2 AD-1/AD-3 and
the DN-28 `Exact`-on-verified-decode resolution (addendum §3). **One correction:** present it as **5
declared / 4 active** correction types — `TritFlips` is dead on the encode side (`compute_correction`
never emits it), so any Mycelium port should not assume a ternary-flip correction path is exercised. The
overhead percentages remain `Declared` (no committed measurement — §B), and the `Verbatim` fallback is
the default-legacy reality (passthrough — §A), so "1.8–3.4 % cheap" stays data-scoped and unmeasured per
wave-2 CL-1.

---

## Corrections to the Prior Crossover (consolidated)

| # | Prior crossover claim (file:line in prior doc) | Authoritative code (file:line) | Corrected verdict | Tag |
|---|---|---|---|---|
| C1 | "**Chunk-level copy-on-write** … unaffected chunks **share storage** … **structural sharing**" (§Q1 :49–51; §3.1.E) | `apply_byte_replace` re-decodes→re-encodes→**overwrites same slot** (`versioned_embrfs.rs:744–788`); store is single-slot `HashMap<ChunkId,Arc<VersionedChunk>>`, `insert` overwrites (`chunk_store.rs:38,114`) | **No CoW / no MVCC / no structural sharing.** Mutation = **re-encode-the-chunk + overwrite-in-place**, guarded by an OCC version **counter**. Embeddenator **sidesteps** copy/mut. | `Empirical` |
| C2 | OCC is "a complete working impl of OCC **on content-addressed storage**" with refcount reclamation analog (§3.1.E; §Q2) | `ref_count`/`inc_ref`/`dec_ref`/`find_by_hash`/`gc()` exist (`chunk.rs:31–90`, `chunk_store.rs:46,240`) but are **unwired** — referenced only in their own files; `ref_count` never decremented; no `VersionedEmbrFS::compact` | OCC version-**counter** is real; **refcount/dedup/GC are dead code**. Reclamation in the versioned path is **soft-delete only, no chunk GC**. | `Empirical` |
| C3 | "**five correction types**" / "three-layer correction" with `TritFlips` active (§Q3) | `compute_correction` emits only `None`/`Verbatim`/`BlockReplace`/`BitFlips` (`correction.rs:186–241`); `TritFlips` declared+`apply`-able but **never generated** | **5 declared / 4 active.** No ternary-flip correction is produced. | `Exact` (read the selector) |
| C4 | `enum TernaryVecRepr { Sparse, Packed }` adaptive **25 % auto-crossover** implemented (§3.1.B; §3.2) | No such enum in `src/`; 25 % is a **module-header guideline** (`ternary_vec.rs:33–39`); both types exist, selection is caller's | **Auto-selector NOT implemented** — it is guidance + two concrete types. Mycelium must build the never-silent switch. | `Exact` (grepped absence) |
| C5 | Table-1 perf rows incl. `performance_benchmark.rs:40/69/91` `<20/200/100 ns` "targets"; overhead rows `Empirical` | No committed criterion output anywhere; benches are harnesses; testkit "validation" benches assert **no** thresholds; **`obs` crate + `performance_benchmark.rs` absent from the corpus** | **No measured numbers exist.** Whole envelope → `Declared`; the three `performance_benchmark.rs` rows → **Unverifiable** (source absent). | `Declared` / `Unverifiable` |
| C6 | "~94 % uncorrected accuracy" + default holographic encoding implied | Default ctor is **non-holographic** (`with_config_and_profiler` sets `holographic_mode:false`, `versioned_embrfs.rs:215`); refactor plan: "VersionedEmbrFS uses 100 % verbatim corrections (passthrough)" (`HOLOGRAPHIC_REFACTOR_PLAN.md:14`) | The ~94 % path (`ReversibleVSAEncoder`) exists and is selectable, but **default writes are legacy → Verbatim passthrough**. Confirms external report. | `Empirical` |

---

## Impact on the Synthesis Thesis ("residue is provenance + reclamation")

**Net: the corrections STRENGTHEN the thesis, and tighten (not overturn) the wave-2 ledger.**

- **CF-2 (chunk-level CoW "is the natural copy/mut path") — RE-SCOPE.** The running code does
  **not** demonstrate CoW; it demonstrates **re-encode-on-edit + overwrite**, with reclamation
  **deferred/forgone** (refcount+GC unwired). This is the *empirical confirmation that copy/mut +
  reclamation is the unsolved residue* — exactly the thesis. CF-2 should read: *"embeddenator's mutation
  path re-encodes the touched chunk and overwrites in place (OCC-guarded); it does **not** implement
  structural-sharing CoW or refcount reuse — confirming copy/mut + reclamation as the hard residue, not
  as a solved primitive."* (External report reached the same verdict independently — two sources, one
  conclusion.)
- **CH-1 (OCC chosen over refcount-reuse) — STRENGTHENED.** Not only did the maintainer choose OCC at the
  multi-writer layer, the refcount-reuse machinery they *did* write is **inert**. The single-owner
  refcount/FBIP path (wave-2 A3) remains a Mycelium *design* move with **no working precedent** in
  embeddenator — down-tag any implication that embeddenator validates refcount reuse.
- **AD-1/AD-3 + DN-28 `Exact`-on-decode (§3 of addendum) — CONFIRMED, line-anchored.** The
  `D = decode(E)+R`, SHA256-trunc-8, parity-trit, post-apply re-verify model is real
  (`correction.rs:31–40,148–183,290–300`). The `Exact`-earned-at-verified-decode resolution stands; add
  the caveat that `TritFlips` is dead and the default path is `Verbatim` passthrough.
- **AD-2 / W2-A5 (representation tier) — REFINE.** Adopt the **bitsliced `PackedTritVec` design** (now
  line-anchored) and the **25 % threshold as a derivable guideline**, but **build the never-silent
  auto-selector** — embeddenator has the two representations and the number, **not** the switch.
- **The "1.8–3.4 % is cheap" number — stays DOWN-TAGGED (`Declared`/unmeasured), reaffirmed.** §B shows
  there is *no* committed measurement at all; wave-2 CL-1 was right and is reinforced.

---

## Meta — Changelog

- **2026-06-24 — Created.** Opus ground-truth correction pass over the authoritative full-source
  embeddenator repos (`/tmp/embr-auth/`). Corrects six over-reads in the prior partial crossover
  (`embeddenator-crossover.md`) — chiefly the copy/mut "CoW/structural-sharing" claim (C1) — with real
  file:line, and refines `SYNTHESIS-wave2-addendum.md` (CF-2 re-scope, CH-1 strengthen). Reconciles with
  the maintainer's external web-session report (copied to `embeddenator-crossref-external.md`).
  Non-normative; touches no RFC/ADR/DN. Per-claim VR-5; append-only.
