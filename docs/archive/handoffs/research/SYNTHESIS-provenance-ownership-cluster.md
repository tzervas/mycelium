# SYNTHESIS — Provenance / Ownership Cluster (5-Lane Distillation)

| Field | Value |
|---|---|
| **Role** | Opus Distiller — cross-cut + adversarial verify, not a sixth report |
| **Inputs** | lane-A (ownership map) · lane-B (reclamation provenance) · lane-C (verifiable decision chain) · lane-D (tunable-cert boundary) · lane-E (spore provenance) |
| **Status** | Research artifact — **non-normative**. Feeds RFC-0027, the transpilation DN, RFC-0034/E21-1. Touches no RFC/ADR/DN. |
| **Date** | 2026-06-24 |
| **Confidence** | Per-claim VR-5 tags throughout. Default posture: skeptical (do not upgrade plausible → solved). |

---

## 1. Connective Thesis

**`Empirical`.** Across all five lanes one structural pattern recurs and is the cluster's spine:

> **Mycelium's immutable + acyclic + content-addressed value model *dissolves* a class of problems
> that other systems must actively *solve* — collapsing each problem from a runtime/aliasing question
> into a hashing/structural question — and the residue that remains is not "aliasing safety" but
> "provenance: binding a record to an identity and tagging the strength of that binding."**

The pattern instantiates per lane:

- **Lane A:** the borrow checker exists to police `&mut T` aliasing; immutability (LR-8) + acyclicity
  (LR-9) make that whole problem *non-arising*, leaving only *reclamation mechanics* as residue.
- **Lane B:** GC's two hardest costs — cycle detection and write barriers — are dissolved by LR-8/LR-9,
  leaving reclamation as a *scope-exit + channel-close synchronization* problem the RT7 scope tree
  already structures.
- **Lane C:** the append-only decision ledger's integrity is currently *editorial assertion*;
  content-addressing turns "did the doc change?" into a *hash comparison* — dissolving trust-the-reviewer
  into verify-the-hash.
- **Lane D:** gradual typing needs *runtime casts* at the typed/untyped boundary; Mycelium's
  meet-semilattice *is* the boundary enforcement — `meet(Proven, Declared) = Declared` dissolves the
  cast into an algebraic floor.
- **Lane E:** vulnerability advisories elsewhere need *version-range matching*; `spore_id` hash-equality
  dissolves range-matching into an O(1) set lookup.

**The unifying mechanism is identical in all five:** *replace a runtime/relational check with a
content-hash equality over an immutable value, and attach a guarantee-strength tag (`Exact ⊐ Proven ⊐
Empirical ⊐ Declared`) recording how strongly that hash binds to its claim.* The `Provenance` DAG type
(RFC-0001 §4.6, `Root | Derived{op, inputs}`) and the guarantee lattice are the *two shared primitives*
that every lane independently reaches for. That convergence — five unrelated questions landing on the
same two primitives — is the strongest evidence the thesis is real and not narration.

**Where the thesis has a hard edge (the honest boundary):** dissolution is *structural*, not *semantic*.
Hash-equality proves *identity*, never *correctness*. Every lane hits the same wall — Lane C's OQ-C4
(structural vs. semantic soundness), Lane E's `Declared` ratification cert, Lane D's "lattice floors but
cannot prove the input was right." The cluster dissolves the *aliasing/identity* problems; it does **not**
dissolve the *proof-obligation* problems, which remain genuinely open and require theorem-proving the
corpus defers (KC-4).

---

## 2. Per-Lane Distilled Takeaway

**Lane A — Ownership map** (`Empirical`; `Declared` on RFC-0027-blocked rows). Six Rust constructs that
exist to manage mutable aliasing (`&mut T`, interior mutability, raw pointers, `Pin`, `dyn Trait+'a`) are
**absent-by-design**, not gaps — the intended consequence of LR-8/LR-9/RT1. The *real* open work is the
narrow RFC-0027 reclamation cluster (rows 5–7, 9: `Box`/`Rc`/`Arc`/`Drop`), which is about *when memory is
recovered*, not *aliasing safety* (already solved by immutability).

**Lane B — Reclamation provenance** (`Declared` throughout; design-stub analysis). LR-8/LR-9 reduce
reclamation to **scope-exit + channel-close synchronization**, already provided by the RT7 scope tree and
the channel close protocol; no cycle detector (cf. Pony ORCA) is needed. The one new artifact RFC-0027
must define is the **reclamation EXPLAIN record** (R-1: `scope_id`, `sweep_epoch`, `trigger`,
`value_meta_hash`, `channel_id?`). The highest-uncertainty decision is sweep-order coupling (weak vs.
strong, OQ-1).

**Lane C — Verifiable decision ledger** (mixed; `Empirical` for current-state, `Declared` for proposals,
`Proven` for the *external* CT/Rekor/in-toto patterns). The decision ledger's integrity is today mostly
`Declared` (editorial). The single cheapest high-payoff fix is **pinning the content hash of each doc at
the moment it becomes `Accepted`** (a `decision-log.json` + small tool), lifting content-integrity to
`Exact` and state-machine legality to `Proven`. The `Provenance` DAG type is the correct native host.

**Lane D — Tunable-cert boundary** (`Empirical` for lattice/structural; `Declared` for runtime wiring not
yet built). Cross-mode (`fast → certified`) composition is sound **without a runtime cast** because the
meet-semilattice algebraically floors any `Declared` input to a `Declared` result — `gate_guarantee` +
`propagate` are the two enforcement mechanisms. `fast` still carries real *structural provenance* (which
swap, repr identity, memory-safety floor, spore identity). The one residual case — a `certified` consumer
*rejecting* a `Declared` input — must be never-silent and is left open (static vs. dynamic, OQ-2).

**Lane E — Spore provenance** (`Empirical` for prior-art; `Declared` for the unbuilt schema). Content-
addressed advisory binding (`spore_id` hash-equality vs. semver ranges) is Mycelium's **single largest
precision advantage** over npm/PyPI/crates.io/GHSA/OSV: O(1), unambiguous, tamper-evident. The load-bearing
gap is the **absence of a concrete artifact-metadata schema** (ADR-013 §2 item 4, "deliberately deferred")
— without it, "full history" is aspirational. in-toto/SLSA/Rekor are the well-precedented building blocks.

---

## 3. Cross-Cutting Findings + Tensions

### 3.1 Agreements (independent lanes converging — strengthens confidence)

- **AG-1 (`Empirical`).** Lanes B, C, E independently reach for the **same `Provenance` DAG type**
  (RFC-0001 §4.6) as the native host for their records (reclamation events / decision ledger entries /
  spore attestation chains). Three unrelated questions, one primitive — the corpus already owns the right
  abstraction. *This is the cluster's most actionable convergence.*
- **AG-2 (`Proven` external / `Declared` Mycelium-mapping).** Lanes C and E **independently** select the
  **same external prior-art stack** — in-toto + SLSA + Sigstore/Rekor + Merkle/CT — for binding records to
  content-addressed identities. The decision ledger (C) and the spore bundle (E) are *the same machine*
  pointed at different artifacts (design docs vs. deployables).
- **AG-3 (`Empirical`).** Lanes A and B agree precisely on scope: LR-8/LR-9 dissolve aliasing and cycle
  problems; the residue is *reclamation timing*, and the RT7 scope tree is the structural hook for it.
  Lane A's "rows 5–7,9 are the real work" and Lane B's "scope-exit + channel-close is the whole model"
  are the same finding from two directions.
- **AG-4 (`Empirical`).** Lanes C, D, E agree that the **guarantee lattice is the universal honesty
  vocabulary** — ledger claims (C), mode-boundary results (D), and provenance/build claims (E) all tag with
  `Exact/Proven/Empirical/Declared`, and all three insist the *un*checkable claims must be surfaced as
  `Declared`, never silently omitted (G2).

### 3.2 Tensions (real, must be adjudicated by the owning docs)

- **TN-1 — Where reclamation provenance lives: a *new record* (B) vs. the *existing Provenance DAG* (C).**
  Lane B (R-1) proposes a purpose-built reclamation EXPLAIN record emitted to the RFC-0013 §8 observability
  sink. Lane C argues every provenance-bearing event should be a `Derived` node in the *one*
  RFC-0001 §4.6 DAG. **Tension:** two sinks vs. one. *Resolution direction (`Declared`):* Lane C's
  unify-on-`Provenance` view is more DRY/KC-3-aligned, but Lane B's record is an *emit-once observability*
  artifact, not an identity-bearing value — they may be the same schema rendered to two consumers. The
  transpilation DN / RFC-0027 should decide whether the reclamation record *is* a `Provenance` node or
  merely *cites* one.
- **TN-2 — Cross-mode soundness needs no cast (D) vs. boundary enforcement is load-bearing (D's own
  prior art).** Lane D claims the meet-semilattice makes a runtime cast *unnecessary* (the lattice floors
  honestly). But Lane D §3.5 (sound gradual verification) and §4.2 concede that **rejection** of an
  insufficient input *does* need explicit, possibly *dynamic*, enforcement — and that mode is a *runtime*
  configuration for cross-phylum composition, so static analysis cannot always decide it. **Tension:** "the
  lattice is sufficient" vs. "runtime boundary enforcement is required for the rejection case." These are
  *not* contradictory once separated: *flooring* needs no cast; *rejection* does. The synthesis down-tags
  the blanket "no cast needed" claim (see §5, CL-3).
- **TN-3 — Sweep-order coupling: weak (B's default) vs. strong (B's most-auditable).** Lane B recommends
  weak coupling (partial order over siblings) for throughput but flags it as the *highest-uncertainty*
  decision and notes strong coupling gives one property test covering both scheduling and reclamation.
  This is a genuine unresolved tradeoff, not a finding — routed to RFC-0027/r10 to prototype both.
- **TN-4 — `cyst` checkpoint-and-free (B §2.4 calls it "sound") vs. checkpoint-and-keep (B R-5 recommends
  as the safe default).** Lane B's *grounding* says content-addressing makes checkpoint-and-free sound
  (identity is the hash, not the address); its *recommendation* defers to checkpoint-and-keep until an
  `Empirical` serializer test exists. Internally consistent (soundness-in-principle vs.
  insufficient-evidence-to-ship), but the two statements must not be read as a settled "free is safe."

### 3.3 One lane's finding changing another's conclusion

- **CHG-1 (`Empirical`).** **Lane C/E's content-addressing-of-provenance reframes Lane B's reclamation
  record.** Lane B treats the EXPLAIN record as an observability emission. But Lane C's insight (a
  provenance event is a `Derived` node whose hash covers its inputs) means the reclamation record can be a
  *first-class verifiable ledger entry*, not just a log line — upgrading its potential strength from
  `Declared` (asserted log) toward `Exact` (hash-pinned). This is a strict improvement Lane B did not claim
  and RFC-0027 should consider.
- **CHG-2 (`Declared`).** **Lane D's two-layer tagging (`CertMode` + `GuaranteeStrength`) refines Lane E's
  build-provenance strength column.** Lane E tags build provenance `Proven`-or-`Declared`. Lane D's model
  says the *mode context of production* must also be attributable, not just the strength — so a spore's
  verification-history predicate should carry the producing `CertMode`, else two spores with identical
  `Empirical` test tags but produced under `fast` vs. `certified` are conflated. Lane E's schema (§4.3)
  should add a mode field.
- **CHG-3 (`Empirical`).** **Lane A's "reclaim is never memory" (DN-03 §4) constrains Lane B's scope.**
  Lane A surfaces the hard corpus rule that `reclaim` is task-supervision, *never* a memory primitive; Lane
  B (R-3) independently lands the same constraint. Together they *settle* that RFC-0027 must not define
  `reclaim` as memory — removing one option from the design space (see Decided ledger D-4).

---

## 4. Decided / Actionable / Open Ledger

Each row tagged (VR-5) and routed to its owning doc. **Decided** = the corpus already settles it.
**Actionable** = a concrete design move now unblocked, with an owner. **Open** = genuine open research.

### 4.1 DECIDED (corpus already settles — carry forward, do not re-litigate)

| ID | Statement | Tag | Basis / Owner |
|---|---|---|---|
| D-1 | Aliasing safety (`&mut T`, interior mutability, raw pointers, `Pin`) is **absent-by-design**, resolved by LR-8/LR-9/RT1 — not an open problem. | `Exact` | Lane A rows 3,8,11,14; RFC-0006 LR-8/LR-9; RFC-0008 §2 |
| D-2 | No cycle detection is needed (values acyclic) — eliminates GC's hardest component. | `Exact` | Lanes A,B; LR-9; RFC-0027 §3 |
| D-3 | `wild`/`swap` are *faithful and strictly safer* analogues of `unsafe`/`transmute` (denied-by-default; certified, never-silent). | `Empirical` | Lane A rows 10,12; DN-02 §5; RFC-0002; RFC-0028 |
| D-4 | `reclaim` is task/runtime-unit supervision, **never** a memory primitive — RFC-0027 must not redefine it as memory. | `Empirical` | Lanes A,B (CHG-3); DN-03 §4; RFC-0008 §4.5; ADR-020 |
| D-5 | Channel close protocol (`TrySend::Disconnected`/`TryRecv::Closed`, affine non-`Clone` Sender/Receiver) already provides single-owner cross-hypha transfer at R1 — no distributed refcount needed. | `Empirical` | Lane B R-4; RFC-0008 §4.3 |
| D-6 | The meet-semilattice (`propagate`) + `gate_guarantee` already make it algebraically impossible for a `fast`/`Declared` input to yield a `Proven`/`Empirical` result. | `Empirical` | Lane D §2.3,§4.2; `guarantee.rs`/`cert_mode.rs` (tested exhaustively) |
| D-7 | Content-addressed advisory binding (`spore_id` equality) is strictly more precise than semver-range matching; tamper-evident by reconstruction. | `Empirical` | Lane E §4.4–4.5; RFC-0035 §3 |
| D-8 | The `Provenance` DAG type (RFC-0001 §4.6) + guarantee lattice are the shared native host for reclamation / ledger / spore records. | `Empirical` | Lanes B,C,E (AG-1); RFC-0001 §4.6 |

### 4.2 ACTIONABLE (concrete design move now unblocked)

| ID | Move | Tag | Owner doc |
|---|---|---|---|
| A-1 | Define the **reclamation EXPLAIN record** minimum field set (`scope_id`, `sweep_epoch`, `trigger`, `value_meta_hash`, `channel_id?`); emit to RFC-0013 §8 sink. | `Declared` | **RFC-0027 / r10** (Lane B R-1) |
| A-2 | Adopt **partial-order reclamation** (total within a scope's child→root path; partial across siblings) as the minimal-safe default; prototype strong-coupling for comparison. | `Declared` | **RFC-0027 / r10** (Lane B R-2, OQ-1) |
| A-3 | Specify the **memory model only**; leave `reclaim` *surface typing* to a follow-on RFC (keeps RFC-0027 scope tight, KC-3). | `Declared` | **RFC-0027 / r10** (Lane B OQ-2) |
| A-4 | Default `cyst` to **checkpoint-and-keep** at R1; gate checkpoint-and-free on an `Empirical` serializer property test. | `Declared` | **RFC-0027 / r10** (Lane B R-5) |
| A-5 | Pin **`hash(doc at Accepted)`** into an append-only `decision-log.json` (Layer 0) — small tool; lifts content-integrity → `Exact`, state-machine → `Proven`. | `Declared`→`Empirical` once built | **decision-ledger RFC** / `tools/github/` (Lane C §4.2) |
| A-6 | Add a **mode field (`CertMode`)** to the spore verification-history predicate so identical strength tags under different modes are not conflated. | `Declared` | **transpilation DN** / spore-provenance RFC (CHG-2; Lanes D+E) |
| A-7 | Define the **spore artifact-metadata schema** (ADR-013 §2 item 4) reusing in-toto Statement + DN-28 reconstruction-on-render. | `Declared` | **spore-provenance RFC** extending DN-28 (Lane E OQ-1) |
| A-8 | Add **cross-mode *negative* conformance tests** (a `certified` computation receiving `fast`/`Declared` input must *not* silently upgrade the output). | `Declared` | **RFC-0034 / E21-1** (Lane D §4.4 Impl-3, M-794) |

### 4.3 OPEN (genuine open research — do not present as solved)

| ID | Question | Tag | Routed to |
|---|---|---|---|
| O-1 | Weak vs. strong sweep-order coupling — real throughput/auditability tradeoff, unresolved by corpus or prior art. | `Declared` (open) | RFC-0027/r10 (Lane B OQ-1) — highest-uncertainty |
| O-2 | Static vs. dynamic rejection of an insufficient (`Declared`) input at a `certified` boundary. | `Declared` (open) | RFC-0034/E21-1 (Lane D OQ-2) |
| O-3 | Chain provenance of a swap certificate when inputs were `fast`/`Declared` — partial vs. full cert. | `Declared` (open) | RFC-0034/E21-1 (Lane D OQ-1) |
| O-4 | Canonicalization of decision documents for hashing (whole-file vs. normative-sections-only). | `Declared` (open) | decision-ledger RFC (Lane C OQ-C1) |
| O-5 | Key management / trust root for ledger + spore signing (single maintainer → governance scale). | `Declared` (open) | decision-ledger / spore RFC (Lane C OQ-C2, Lane E OQ-6) |
| O-6 | `substrate`/`graft` reclamation protocol (drop-vs-consume of an unused affine handle). | `Declared` (open) | `graft` RFC; RFC-0027 must *flag-and-defer* (Lanes A OQ-A1/A3, B OQ-5) |
| O-7 | Worst-case reclamation latency bound derivable from the fuel model (would lift latency `Declared`→`Empirical`). | `Declared` (open) | RFC-0027 research note (Lane B OQ-3) |
| O-8 | R2 (`xloc`/`mesh`) distributed reclamation + advisory provenance — CRDT tombstone / weighted-refcount territory. | `Declared` (open, deferred) | future R2 RFC (Lane B OQ-4; Lane C/E distributed) |
| O-9 | Semantic soundness (design↔code *correctness*, not just *linkage*) — requires theorem proving the corpus defers (KC-4). | `Declared` (open, hard boundary) | all lanes; out of scope for any ledger/provenance verifier |

---

## 5. Adversarial-Verification Results

For each load-bearing claim I tried to refute it against the cited corpus/prior-art. Verdicts:

- **CL-1 — "Immutability + acyclicity dissolves the borrow checker's whole reason for existence" (Lane A,
  thesis core).** *Challenge:* does it dissolve it, or merely *relocate* it into the Rust kernel? *Verdict:
  SURVIVES, scoped.* At the **Mycelium language surface** the claim holds (`Exact`, grounded in LR-8/RFC-0008
  §2). But the Rust kernel still runs a borrow checker for the *implementation*, and `wild`/FFI re-admit raw
  `&mut` (Lane A row 3). Down-tag any unscoped "Mycelium has no borrow checker" to **"the Mycelium *surface*
  has none; the kernel and `wild` still do."** Kept at `Exact` only with that scope.

- **CL-2 — "No new artifact is needed; the existing `Provenance` DAG hosts everything" (AG-1 / Lane C).**
  *Challenge:* Lane B *does* propose a new record (R-1). *Verdict: SURVIVES as type-reuse, DOWN-TAGGED as
  zero-work.* The *type* is reused (true, `Empirical`); but a schema, canonicalization rule (O-4), and emit
  path are genuinely new work. Corrected claim: "no new *primitive*; non-trivial new *schema/tooling*."

- **CL-3 — "Cross-mode composition is sound without a runtime cast — the lattice is sufficient" (Lane D
  §4.2).** *Challenge:* Lane D's own §3.5/§4.2 say rejection needs explicit, possibly dynamic, enforcement,
  and mode is a runtime config cross-phylum. *Verdict: DOWN-TAGGED.* True for the *flooring* case
  (`meet → Declared`, `Empirical`). **Over-claim for the *rejection* case** — there the boundary event is
  load-bearing and may require a runtime check (O-2 is open). Carry forward only as: "flooring needs no cast;
  rejection enforcement is unresolved." Reduced from an implied solved-property to `Declared`-open on the
  rejection half.

- **CL-4 — "`spore_id` advisory binding is the single largest precision advantage; no ecosystem does this"
  (Lane E §6).** *Challenge:* OSV's `GIT` range type gives commit-level precision; is the advantage real?
  *Verdict: SURVIVES, narrowed.* Lane E itself distinguishes git-SHA (source identity) from content-addressed
  *build* identity (Lane E §3.5: same commit, different build flags → same OSV entry, different exposure).
  The advantage is real but **narrower than "nobody is close"** — it is precise over *build* identity, where
  git-SHA is precise only over *source*. Kept `Empirical` for the design grounding; the "genuine advance"
  framing stays `Declared` until E22-1 implements it (Lane E's own tag — correct, not upgraded).

- **CL-5 — "Checkpoint-and-free is sound because identity is the hash, not the address" (Lane B §2.4).**
  *Challenge:* soundness-in-principle vs. shippable. *Verdict: SURVIVES in principle, but Lane B's own R-5
  correctly refuses to ship it* (no `Empirical` serializer test). No over-claim — the report already
  separates principle from evidence (TN-4). Carry both halves; do not collapse to "free is safe."

- **CL-6 — "The CT/Rekor/in-toto layer makes the ledger append-only *mechanically*, unlike git's editorial
  force-push ban" (Lane C §3.2).** *Challenge:* is the gap real or rhetorical? *Verdict: SURVIVES (`Proven`
  external, `Empirical` gap-claim).* Git gives content-integrity but a force-push presents a
  self-consistent rewritten history; CT consistency proofs detect it via a held STH. Grounded in RFC 6962.
  The Mycelium *mapping* stays `Declared` (unbuilt) — correctly tagged by Lane C, not upgraded.

- **CL-7 — "RT7 gives Tofte-Talpin LIFO safety 'for free'" (Lane B §3.1).** *Challenge:* free for
  *in-scope* values, but cross-scope channel transfers are exactly what Tofte-Talpin *forbids*. *Verdict:
  DOWN-TAGGED.* "For free" holds only for within-scope reclamation. Cross-scope transfer relies on the
  channel protocol (D-5), which is a *separate* mechanism, not the LIFO property. Carry as: "RT7 gives LIFO
  for in-scope values; cross-scope correctness rides the channel protocol, not LIFO."

- **CL-8 — "`fast` mode still carries useful structural provenance" (Lane D Q-A).** *Challenge:* is
  anything load-bearing actually retained, or is it marketing? *Verdict: SURVIVES (`Empirical` for the gate;
  `Declared` for the unbuilt trace).* Swap identity, repr identity, memory-safety floor, and spore identity
  genuinely survive (Lane D §4.3 inventory, grounded in `gate_guarantee` + RFC-0034 §7/§9). The *trace
  generation* claim is correctly `Declared` (mechanism unbuilt). No over-claim.

**Net:** no claim was dropped outright; **three were down-tagged/scoped** (CL-1 scoped to surface; CL-3
rejection-half reduced to open; CL-7 "for free" narrowed to in-scope), and CL-2/CL-4 had their *scope*
tightened. The cluster's spine (the connective thesis) survived adversarial pressure intact, with the
explicit caveat that *dissolution is structural, never semantic* (O-9).

---

## 6. Prioritized Next Moves

Ranked by value unlocked / cost, with the doc each feeds:

1. **Land the reclamation EXPLAIN record + partial-order model in RFC-0027/r10 (A-1, A-2, A-3).** This is
   the cluster's critical path: Lanes A and B converge that reclamation is the *only* genuinely-open
   ownership residue, and the record + partial-order default is tightly specified and low-risk. Specify the
   memory model only; defer `reclaim` surface typing. **Unblocks the entire RFC-0027 advance.** (`Declared`)

2. **Pin acceptance hashes into an append-only `decision-log.json` (A-5, Lane C Layer 0).** Highest
   integrity-payoff-per-effort move in the whole cluster — a small Python tool turns the decision ledger's
   biggest mechanical gap (`Declared` editorial) into `Exact` content-integrity + `Proven` state-machine
   legality, using machinery (git SHA + the lattice) that already exists. Independent of RFC-0027, so it can
   proceed in parallel. (`Declared`→`Empirical` on build)

3. **Add cross-mode negative conformance tests to RFC-0034/E21-1 (A-8).** Cheap, and CL-3 shows the
   *rejection* half of mode-boundary soundness is the unverified risk — a negative test (`certified` must not
   silently upgrade `fast`/`Declared` input) directly guards it.

4. **Resolve the one cross-cutting tension that blocks unification (TN-1): is the reclamation record a
   `Provenance` node or a citation of one?** Settling this (transpilation DN + RFC-0027) lets reclamation,
   ledger, and spore provenance share one schema (AG-1) instead of three — paying off DRY/KC-3 across the
   cluster, and enabling CHG-1's upgrade of the reclamation record toward `Exact`.

5. **Draft the spore artifact-metadata schema (A-7) + add the mode field (A-6).** Unblocks Lane E's
   load-bearing gap and DN-28's deferred provenance seam; reuses the in-toto/Rekor stack Lanes C and E
   already share (AG-2). Lower urgency than 1–4 because it feeds E22-1, which is further out.

**Top-2 (the two highest-value, lowest-regret moves):** (1) **RFC-0027/r10 reclamation EXPLAIN record +
partial-order reclamation model** — the cluster's critical path; and (2) **acceptance-hash pinning
(`decision-log.json`)** — the highest integrity-payoff-per-effort fix, parallelizable now.

---

## Meta — Changelog

- **2026-06-24 — Created.** Opus distillation of the 5-lane provenance/ownership research cluster
  (lanes A–E). Non-normative research artifact; touches no RFC/ADR/DN. Per-claim VR-5 tags. Feeds
  RFC-0027, the transpilation DN, and RFC-0034/E21-1. Append-only.
