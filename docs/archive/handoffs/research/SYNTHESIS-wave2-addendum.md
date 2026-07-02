# SYNTHESIS — Wave-2 Addendum (Academic + Empirical Integration)

| Field | Value |
|---|---|
| **Role** | Opus Distiller — wave-2 cross-cut; **extends** the wave-1 synthesis, does not rewrite it |
| **Extends** | `SYNTHESIS-provenance-ownership-cluster.md` (wave-1, lanes A–E) |
| **New inputs** | lane-F (ACADEMIC: Perceus/FBIP/uniqueness/δ-CRDT) · embeddenator-crossover (EMPIRICAL: maintainer's benchmarked VSA/ternary/filesystem work) |
| **Status** | Research artifact — **non-normative**. Feeds RFC-0027, the value model, DN-28, RFC-0008 RT6 (`fuse`). Touches no RFC/ADR/DN. |
| **Date** | 2026-06-24 |
| **Confidence** | Per-claim VR-5 tags throughout. Default posture: skeptical — wave-2 *refines* wave-1, it does not re-open settled rows. New IDs are prefixed `W2-` to keep the wave-1 ledger intact (append-only). |

---

## 1. What Wave-2 Changes vs. Wave-1

Wave-1's spine was the *dissolution thesis*: immutable + acyclic + content-addressed values turn
runtime/aliasing questions into hashing/structural ones, leaving **provenance** as the residue.
Wave-2 does not overturn that — it **sharpens the mechanism** and supplies the **first empirical
load-bearing evidence** the cluster had been missing. Two net changes:

- **W2-Δ1 — The reclamation residue gets a named mechanism: reference counting, not tracing GC
  (`Empirical`).** Wave-1 left RFC-0027's *how-memory-is-freed* as an open A-1/A-2 design move with
  an undetermined mechanism. Lane F closes the mechanism question: the wave-1 "no cycle detector
  needed" finding (D-2, grounded in LR-9) is *exactly* the precondition under which **Perceus-style
  precise reference counting is garbage-free and sound** (lane-F §3.2, §4.2.1; Perceus PLDI 2021,
  Lean-4 "Counting Immutable Beans" arXiv 2019). RC was implicit in wave-1's residue framing; lane F
  makes it the explicit, prior-art-backed answer and gives the reclamation EXPLAIN record's `trigger`
  field a *structural* grounding (`RcZero | ScopeExit | ChannelClose`, lane-F §4.2.1 F-2) rather than
  an editorial label. This **refines** wave-1 A-1, it does not replace it.

- **W2-Δ2 — The cluster acquires empirical confirmation at ~32K-LOC scale (`Empirical`).** Wave-1 was
  almost entirely `Declared`/`Empirical`-from-prior-art with *no Mycelium-shaped running code*. The
  embeddenator series is the maintainer's own benchmarked implementation of the cluster's core
  patterns — content-addressed immutable base + acyclic value graph + chunk-level CoW + a working
  reconstruction-on-render correction layer (embeddenator §2 Q1–Q3). It **confirms** three wave-1
  claims with running code (see §2) and **adds** one pattern wave-1 theory did not predict: a
  *minimum-cost algebraic correction delta* that makes a lossy base losslessly reconstructable cheaply
  (§3). This is the first time a cluster claim moves from "prior art elsewhere" to "running in the
  maintainer's own corpus-adjacent code."

Net: wave-2 is **refinement + first-evidence**, not redirection. The dissolution thesis,
the `Provenance`-DAG-as-shared-host finding (AG-1), and the structural-not-semantic boundary (O-9)
all survive wave-2 untouched.

---

## 2. Academic ↔ Empirical Cross-Cut (Confirm / Challenge / Add)

The sharpest test of a research cluster is whether independently-produced *academic theory* (lane F)
and *empirical implementation* (embeddenator) agree without being made to. Verdicts:

### 2.1 CONFIRM (empirical data validates an academic / wave-1 claim)

- **CF-1 — Acyclic content-addressed values ⇒ no cycle detector, RC suffices (`Empirical`).** Lane F's
  central precondition (LR-9 makes RC sound, §3.2, §4.2.1) and wave-1 D-2 are *confirmed by running
  code*: embeddenator manages its entire ~32K-LOC holographic value graph with **no cycle detection
  and no GC** — "the holographic value graph is acyclic by construction … no back-references"
  (embeddenator §2 Q2, line 80; parallels D-2 explicitly). Theory predicted acyclicity removes the
  hard RC case; the implementation never needed one. *Independent convergence ⇒ confidence up.*

- **CF-2 — Chunk-level CoW over a content-addressed immutable base is the natural copy/mut path
  (`Empirical`).** Lane F F-4/F-6 argue structural-sharing-plus-targeted-reuse is the right default;
  `VersionedEmbrFS` *is* that — "every write re-encodes only the affected 4KB chunks … unaffected
  chunks share storage unchanged" (embeddenator §2 Q1, lines 49–51), with a **~90× speedup for
  single-byte edits in 1 MB files** (`Declared`, embeddenator `versioned_embrfs.rs`:528–532). This is
  the empirical face of lane F's "reuse the unchanged path, copy only the changed path" — the
  ~90× number *quantifies* the penalty of full-re-encode vs. targeted-delta that lane F argued
  qualitatively.

- **CF-3 — The δ-CRDT "delta = the changed path" insight has an implemented analogue (`Empirical`).**
  Lane F OQ-F3 proposes that for `fuse`, the natural delta is "the new provenance chain since the last
  common ancestor" — a path-copy diff, not a full-state diff. Embeddenator's `apply_delta` dispatch
  (re-encode only the relevant chunk) is the same move at the storage layer (embeddenator §2 Q1).
  The theory's "send only the changed sub-tree" has a working precedent.

### 2.2 CHALLENGE (empirical data complicates / pressures an academic claim)

- **CH-1 — Embeddenator chose OCC, not refcount-reuse, for its mutation path (`Empirical`).** Lane F's
  unification leans on a **refcount==1 uniqueness probe** (FBIP/Swift-CoW) for in-place reuse (§4.1).
  But the maintainer's own implementation reached for **optimistic concurrency control** — a monotonic
  `global_version: AtomicU64` + `VersionMismatch` (embeddenator §2 Q1, lines 52–53), *not* an
  `isKnownUniquelyReferenced`-style probe. This is a real challenge to the "refcount probe is the
  obvious mechanism" framing: at the *filesystem/multi-writer* layer the maintainer found OCC more
  natural than uniqueness-reuse. **Reconciliation (`Declared`):** the two operate at different layers —
  FBIP-reuse is a *single-owner intra-hypha* optimization (lane F §4.1 "where the triangle is tight"),
  OCC is a *multi-writer concurrency* discipline; they are not competitors but co-residents. Still, the
  empirical record is a caution against presenting refcount-reuse as the *only* copy/mut answer — it is
  the single-owner answer; the concurrent-writer answer is OCC. (Down-tags lane F's implicit
  "refcount probe is *the* mechanism" to "the single-owner mechanism.")

- **CH-2 — `fuse`/CRDT convergence is *not* free from the algebra alone — confirmed by absence
  (`Empirical`).** Lane F's adversarial verdict was that CRDT-merge is algebraically separate from the
  refcount unification (lane-F §4.1 "where the triangle breaks"). The empirical record *strengthens
  the negative*: embeddenator's VSA `bundle` is commutative + idempotent-at-saturation (a lattice join,
  embeddenator §2 Q4, line 143) **yet the maintainer explicitly does not claim CRDT semantics — no
  tombstones, no deletion-merge, no convergence proof** (embeddenator §2 Q4, line 145). So even a
  *working, benchmarked* commutative merge does **not** yield distributed convergence for free — exactly
  lane F's "extension to `fuse` is architectural, not mechanical." Two independent sources, one verdict:
  `fuse` is **structurally** unified with the value model (shares the Provenance DAG, gets δ-efficiency
  free) but **algebraically separate** (convergence needs its own machinery). *This is the wave-2
  resolution of the `fuse` side of wave-1 TN-1's neighborhood.*

### 2.3 ADD (empirical reality the academic lane did not predict)

- **AD-1 — The minimum-cost correction delta: lossless reconstruction over a lossy base, cheaply
  (`Empirical`).** Lane F's prior art (Perceus, δ-CRDT, RRB-trees) is all about *lossless* structures;
  none of it predicts the embeddenator move of pairing a **deliberately lossy** encoding with a
  **minimum-cost algebraic correction** to recover bit-perfect output at **1.8–3.4 % overhead** on
  structured data, with a **40–62 % zero-byte "Perfect" rate** (embeddenator §2 Q3, lines 99,
  109–116; `CORRECTION.md`:266–277). This is the new pattern wave-2 contributes — and it is the crux of
  §3. The correction-type cost ladder (`Perfect 0B → BitFlips 2B → TritFlips 3B → BlockReplace 4B+ →
  Verbatim 100 %`) is a never-silent, EXPLAIN-shaped selection: it *is* a swap-with-correction, chosen
  by an inspectable cost comparator (G2-aligned).

- **AD-2 — A principled, empirically-derived representation-tier crossover (`Empirical`).** Wave-1
  never touched the binary/ternary/dense tier. Embeddenator supplies a **25 %-density sparse/dense
  crossover** with a derivable basis (`8 bytes/nonzero` vs `2 bits/trit`, embeddenator §2 Q5, §3.1.B;
  `BITSLICED_TERNARY_DESIGN.md`:235–237). This is a directly-reusable, never-silent representation-swap
  threshold — the kind of inspectable selection the transparency rule wants, with an empirical anchor.

- **AD-3 — SHA256-trunc-8 + parity-trit as a cheap tamper-evidence primitive (`Empirical`).** Wave-1
  Lane C's acceptance-hash pinning (A-5) and Lane E's `spore_id` binding both want a content-hash
  verification primitive; embeddenator runs one — first-8-bytes-of-SHA256 per chunk + a `sum(bytes) mod
  3` parity trit guarding the correction store itself (embeddenator §2 Q3, lines 94, 118). A working
  instance of wave-1 AG-1's "content-hash equality over an immutable value."

---

## 3. The Sharp Tension: VSA-Lossy vs. `Exact` — Reasoned Resolution

**The tension.** VSA encoding is inherently lossy (~94 % uncorrected fidelity, embeddenator
`versioned_embrfs.rs`:186, `Declared`). Mycelium's lattice reserves **`Exact`** for *lossless,
bound-free* representation — and the corpus pins this hard: **`Exact ⟺ bound == None`** (RFC-0001
§ M-I1, line 209; "no approximation … `bound == None`", line 61; Glossary "a lossless operation",
line 89). Embeddenator achieves **bit-perfect** output from a lossy base by pairing it with a
minimum-cost correction delta. **Question:** does this give Mycelium an `Exact`-tagged VSA
representation `= (VSA + correction-delta)`, with `Exact` holding over the *pair*?

**Reasoned resolution — `Empirical` for the structure, `Declared` for the Mycelium binding:**

The answer turns on *what carries the tag*. Tag-bearing is per-**operation**/per-**value**, not
per-storage-blob (house rule 1; RFC-0001 §4.7). Decompose:

1. **The VSA base, tagged alone, is `Empirical` at best — never `Exact`.** It has
   `bound != None` (~6 % uncorrected error), so by M-I1 (`Exact ⟺ bound == None`) it *cannot* carry
   `Exact`. Tagging the lossy encoder `Exact` would be a VR-5 violation (upgrade past basis). **The
   lossy base does force a non-`Exact` tag — on itself.** (`Empirical`, grounded in M-I1 + the ~94 %
   figure.)

2. **The *decode pipeline as one composite operation* — `reconstruct(VSA, δ) → original` — CAN be
   `Exact`, because it is bound-free over the pair.** The correction layer's `Perfect`/`BitFlips`/…/
   `Verbatim` ladder guarantees byte-identical output *by construction*: any residual the VSA base
   could not represent is carried verbatim in δ, and the result is hash-verified (SHA256-trunc-8,
   embeddenator §2 Q3). So the *composite* `reconstruct` has `bound == None` — it is a bijection from
   `(VSA, δ)` to the original bytes. Under M-I1, a bound-free composite is exactly what `Exact`
   licenses. (`Empirical` — the embeddenator correction layer is a running instance; `Declared` for the
   Mycelium-specific wiring, which is unbuilt.)

3. **Therefore `Exact` attaches to the *reconstruction*, not to the *representation*.** The honest
   tagging is: the **stored pair is `Empirical`-VSA-base + a `Verbatim`-complete δ**, and the
   **`decode`/render operation that consumes the pair is `Exact`**, *iff* δ is complete and the hash
   verifies. This is **not** "lossy thing relabeled lossless" — it is the lattice working as designed:
   the guarantee is a property of the *operation that produces the observed value*, and the operation
   that produces the rendered bytes genuinely is lossless. `meet` is not even engaged here because the
   correction is not an *approximation* the user must tolerate — it is the *exact remainder* that makes
   the pair total.

**Critically, this is exactly what DN-28 already requires — and forbids the lazy reading of.** DN-28
mandates that reconstruction be **"a lossless, content-addressed encoding: decode must reproduce
byte-identical source … never a lossy/approximate reconstruction"** (DN-28 §3.1, lines 88–89; §6
DoD line 187 "must stay lossless + content-verified"). So:

> **Verdict (`Empirical` structure / `Declared` Mycelium-binding): The pair *can* carry `Exact`, but
> only on the *reconstruction operation*, and only when the correction delta is complete-and-verified
> (the `Verbatim`-closure property). `Exact` does NOT attach to the VSA base, and a partial/heuristic
> correction (one that left residual error) would force the composite back down to `Empirical` (M-I1:
> `bound != None`). The lossy base does not poison the pair — but it does *gate* it: `Exact` over the
> pair is conditional on the δ closing the loss to zero, hash-verified. This is sound on the lattice
> precisely because DN-28 already demands lossless byte-identical reconstruction; the embeddenator
> correction layer is a reference implementation of that DN-28 requirement, not a new exception to it.**

The one trap to avoid (G2): an implementation that ships the VSA base *without* the verified δ, or
with a δ that does not close to byte-identity, must **not** be tagged `Exact` — it is `Empirical`
(lossy) and silently calling it `Exact` is the exact failure house rule 1 / VR-5 forbids. `Exact` is
earned at *decode* by the hash check, never asserted at *encode*.

---

## 4. Updated Decided / Actionable / Open Ledger (Wave-2 Deltas)

Append-only over the wave-1 ledger. New rows prefixed `W2-`; rows that *refine* a wave-1 row name it.

### 4.1 DECIDED (wave-2 settles / confirms)

| ID | Statement | Tag | Basis / Owner |
|---|---|---|---|
| W2-D1 | RC (precise reference counting), **not tracing GC**, is the reclamation mechanism for RFC-0027 — LR-9 (acyclic, D-2) is exactly Perceus's garbage-free precondition. **Refines wave-1 A-1.** | `Empirical` | lane-F §3.2,§4.2.1; Perceus PLDI'21, Lean-4 arXiv'19; embeddenator §2 Q2 (CF-1) |
| W2-D2 | `fuse` is **structurally** unified with the value model (shares the `Provenance` DAG; gets δ-CRDT efficiency free via DAG-LCA anti-entropy) but **algebraically separate** (convergence is the semilattice join law, independent of refcounting). **Resolves the `fuse` side of TN-1's neighborhood.** | `Empirical` | lane-F §4.1,§4.3; embeddenator §2 Q4 (CH-2 — two sources, one verdict) |
| W2-D3 | Acyclic content-addressed values eliminate cycle detection **in running code at ~32K-LOC scale** (confirms wave-1 D-2 empirically). | `Empirical` | embeddenator §2 Q2 line 80; CF-1 |
| W2-D4 | A lossy representation + a **complete, hash-verified** correction delta yields **byte-identical** reconstruction — and DN-28 *already requires* exactly this losslessness (§3.1). | `Empirical` (structure) / `Declared` (Myc binding) | embeddenator §2 Q3; DN-28 §3.1 lines 88–89; §3 below |

### 4.2 ACTIONABLE (wave-2 unblocks)

| ID | Move | Tag | Owner doc |
|---|---|---|---|
| W2-A1 | RFC-0027: **adopt RC** as the reclamation mechanism; set the EXPLAIN-record `trigger` field to `RcZero \| ScopeExit \| ChannelClose` (structural, not editorial). **Refines wave-1 A-1.** | `Declared` | RFC-0027/r10 (lane-F F-1,F-2) |
| W2-A2 | RFC-0027: make **sweep-order *derived from* the RT7 scope tree** (deferred RC drops accumulate per scope, flush at scope-exit child→parent) — partially resolves wave-1 O-1. | `Declared` | RFC-0027/r10 (lane-F F-3) |
| W2-A3 | Value model: add an **O(1) `rc==1` reuse check (FBIP)** to `std.collections` mutators; record `push_reuse` vs `push_copy` as distinct `Provenance::Derived` ops (never-silent, EXPLAIN-able). Single-owner only; OCC is the *concurrent* path (CH-1). | `Declared` | value model / `std.collections` (lane-F F-4,OQ-F4) |
| W2-A4 | DN-28: cite the **embeddenator three-layer correction as a reference implementation** of reconstruction-on-render; specify the correction-type **cost ladder** (`Perfect→…→Verbatim`) as the never-silent swap-with-correction selector; tag `Exact` **on the verified `decode`, never on the encode** (§3). | `Declared` | DN-28 (embeddenator §2 Q3; §3 verdict) |
| W2-A5 | Representation tier: adopt the **25 %-density sparse/dense crossover** (derivable: `8 B/nonzero` vs `2 b/trit`) as a principled, never-silent tier-selection threshold; reuse the bitsliced `PackedTritVec` design for the ternary tier. | `Declared` | VSA/representation-tier RFC (embeddenator §2 Q5,§3.1.A–B; AD-2) |
| W2-A6 | `fuse`/RT6: use the **`Provenance` DAG LCA** as the δ-CRDT anti-entropy anchor at R2; `fuse` result = `Derived{op:"fuse_join", inputs:[left_root,right_root]}` with `meet` on guarantees. Convergence machinery (tombstones/version-vectors) is *separate* (W2-D2). | `Declared` | RFC-0008 RT6 (lane-F F-7,F-8,F-9; embeddenator §2 Q4) |

### 4.3 OPEN (wave-2 surfaces / sharpens)

| ID | Question | Tag | Routed to |
|---|---|---|---|
| W2-O1 | Dynamic RC-with-reuse (Perceus) vs. static uniqueness types (Clean/FP2 `fip`, widening `substrate` to ordinary values) for copy/mut — wide design space, YAGNI-deferred. | `Declared` (open) | RFC-0027 / future type-system RFC (lane-F OQ-F1,F-5) |
| W2-O2 | Worst-case RC-cascade drop latency (drop of a 10 M-node `Seq` is O(n)) — can the fuel model bound it? **Sharpens wave-1 O-7.** | `Declared` (open) | RFC-0027 research note (lane-F OQ-F5,F-D); embeddenator §4 gap 5 |
| W2-O3 | **Correction-store merge protocol for distributed `fuse`.** Embeddenator's correction store has *no* CRDT/merge semantics (`HashMap<ChunkId,Correction>`); a convergent merge rule is unspecified. Concrete instance of wave-1 O-8. | `Declared` (open) | future R2 RFC (embeddenator §4 gap 2; §2 Q4 line 149) |
| W2-O4 | Is the in-place reuse vs. copy choice **surface-visible** or **implementation-private** (EXPLAIN-record-only)? G2 suggests the record always captures it; surface visibility is open. | `Declared` (open) | value model (lane-F OQ-F4) |
| W2-O5 | DN-28 δ-completeness as the `Exact`-gate: what *checks* that a correction delta closes loss to zero before `decode` may be tagged `Exact`? (The hash check is necessary; is it sufficient against a crafted collision in trunc-8 — 64-bit?) | `Declared` (open) | DN-28 / §5 below |

---

## 5. Adversarial-Verification Verdicts (Wave-2 Load-Bearing Claims)

For each new load-bearing claim I tried to refute it. Verdicts:

- **W2-CL-1 — "The 1.8–3.4 % correction overhead validates DN-28 reconstruction-on-render as *cheap*."**
  *Challenge 1 (provenance of the number):* the overhead percentages are labelled "Empirical
  Measurements" in `CORRECTION.md` but the embeddenator report itself flags that **no methodology
  section, no measurement conditions, and no committed criterion output** back them — it down-tags them
  to "`Declared` with a note that the values are plausible" (embeddenator §4 caveats, line 296;
  §2 Q3 caveat line 128). *Challenge 2 (scope):* the 1.8–3.4 % holds for **structured data only**;
  binary/compressed/random data run **6.2 % / 12.5 % / 18.3 %** (embeddenator §2 Q3, lines 113–116) — so
  "cheap" is *data-dependent*, not universal. *Verdict: DOWN-TAGGED.* The claim survives **only as**:
  "*for structured data (source/config), reconstruction-on-render is plausibly cheap (single-digit-%
  overhead) — `Declared`/`Empirical`-weak, pending a methodology'd benchmark; and it is NOT cheap on
  high-entropy data, where it degrades toward verbatim (≥100 % is the `Verbatim` fallback)*." The
  unqualified "1.8–3.4 % validates DN-28 as cheap" **over-reaches** on two axes (rigor + data class) and
  must not be carried at `Empirical` without running the suite (embeddenator §4 gap 1). The *pattern*
  is validated; the *number* is `Declared`.

- **W2-CL-2 — "The pair (VSA + δ) gives Mycelium an `Exact`-tagged VSA representation."**
  *Challenge:* M-I1 says `Exact ⟺ bound==None`; the VSA base has `bound!=None`. Doesn't that kill it?
  *Verdict: SURVIVES, SCOPED HARD (this is the §3 resolution).* `Exact` is sound **only on the composite
  `decode` operation, only when δ is complete + hash-verified** — never on the VSA base, never on a
  partial correction. The lossy base does not *poison* the pair (the composite is genuinely bijective)
  but it *gates* it (no verified-complete δ ⇒ `Empirical`). Carried as: "**`Exact` is earned at decode
  by the hash check, never asserted at encode**" (§3). Anything looser is a VR-5 upgrade-past-basis. The
  claim as originally phrased ("an `Exact`-tagged VSA *representation*") is **rejected**; the corrected
  claim ("an `Exact`-tagged *reconstruction* over the pair") survives.

- **W2-CL-3 — "RC, not tracing GC, is decided for RFC-0027."**
  *Challenge:* lane F's own adversarial self-check F-A down-tags "Perceus is *directly* applicable" to
  "requires adaptation" (lane-F §7 F-A: Perceus targets a RC-annotated low-level IR, Mycelium's Core IR
  is a typed term language — RC emission must be added as a lowering pass). *Verdict: SURVIVES as
  mechanism-decided, DOWN-TAGGED on readiness.* RC is the right *mechanism* (`Empirical`,
  LR-9-grounded, two-implementation precedent + embeddenator's GC-free running code). But the *specific
  algorithm* is `Declared`-needs-adaptation, and CH-1 shows the maintainer reached for OCC at the
  concurrent layer — so "RC decided" means "**RC for single-owner intra-hypha value reclamation**," not
  "RC for everything." Cross-hypha transfer still rides the channel protocol (wave-1 D-5/CL-7), not RC.

- **W2-CL-4 — "`fuse` is structurally-but-not-algebraically unified."**
  *Challenge:* is the "structural unification" real, or just shared storage? *Verdict: SURVIVES
  (`Empirical`).* The structural share is load-bearing, not cosmetic: `fuse` genuinely reuses the
  `Provenance` DAG (LCA anti-entropy = real δ-efficiency, lane-F F-7) and the guarantee lattice (`meet`
  on merge, lane-F F-9, `Proven` for the lattice laws). The algebraic separation is equally real and
  *doubly sourced* (lane F §4.1 + embeddenator's explicit non-CRDT disclaimer §2 Q4 line 145). Both
  halves hold; neither is rhetorical. No over-claim.

- **W2-CL-5 — "Embeddenator confirms the wave-1 cluster."**
  *Challenge:* embeddenator is a *filesystem/VSA* system, not a Mycelium implementation — is the
  "confirmation" actually transfer, or analogy? *Verdict: SURVIVES, NARROWED.* It confirms the
  *structural* claims that are substrate-independent (acyclicity ⇒ no cycle detector, CoW over
  content-addressed base, lossless-reconstruction-via-correction) — these transfer (embeddenator §3.1).
  It does **not** confirm Mycelium-specific wiring (the lattice tags, the `fuse` binding, the EXPLAIN
  record schema), which remain `Declared`. And the maintainer's *divergence* (OCC over refcount-reuse,
  CH-1) is itself evidence the transfer is not 1:1. Carry as: "confirms the **substrate-independent
  structural** claims; the **Mycelium bindings stay `Declared`**." (Mirrors wave-1's structural-not-
  semantic boundary, O-9.)

**Net:** of five new load-bearing claims, **two down-tagged** (W2-CL-1 the "1.8–3.4 % is cheap" number →
`Declared`/data-scoped; W2-CL-3 RC-readiness → needs-adaptation), **one rejected-as-phrased then
rescued-scoped** (W2-CL-2: `Exact` on the *reconstruction*, not the *representation*), and two survive
(W2-CL-4, W2-CL-5 narrowed). The wave-2 spine — *RC refines the residue; empirical evidence confirms the
structural claims; the correction-delta makes lossless reconstruction the lattice-honest path* — holds,
with the standing caveat that **`Exact` is earned at verified decode, never asserted at lossy encode.**

---

## Meta — Changelog

- **2026-06-24 — Created.** Opus wave-2 addendum: integrates the academic lane-F (Perceus/FBIP/δ-CRDT)
  and the empirical embeddenator-crossover into the wave-1 provenance/ownership synthesis. Extends (does
  not rewrite) `SYNTHESIS-provenance-ownership-cluster.md`. Non-normative; touches no RFC/ADR/DN.
  Per-claim VR-5 tags; new IDs `W2-*` (append-only). Feeds RFC-0027 (RC adoption), the value model
  (FBIP reuse), DN-28 (correction-delta reference impl + the `Exact`-on-decode resolution), RFC-0008
  RT6 (`fuse` structural-but-not-algebraic unification). Append-only.
