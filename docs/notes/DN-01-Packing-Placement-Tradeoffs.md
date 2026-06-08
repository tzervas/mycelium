# Design Note DN-01 — Representation/Layout Placement: In-Type vs Metadata vs Schedule-Staged

| Field | Value |
|---|---|
| **Note** | DN-01 |
| **Status** | **Resolved** — recommendation adopted: **schedule-staged** packing (confirmed by research T1.4); folded into RFC-0001 §4.1 + RFC-0004 §5 |
| **Feeds** | RFC-0001 §4.1 (packing placement), §4.3 (`Meta`), §4.5 (typing); ADR-009 (AOT/JIT split); Foundation FR-M3/M5, NFR-6/7, VR-1/3 |
| **Date** | June 08, 2026 |
| **Resolution** | The §6 candidate (lossless packing → schedule-staged; precision/sparsity-class → type/refinement) is **accepted**. Research T1.4 confirmed the small fixed packing set (≈5 schemes) is materially easier than Halide's general scheduling problem, removing the E1 contingency as a blocker (E1 remains a build-phase confirmation). Now normative in RFC-0001 §4.1 and RFC-0004 §5. |
| **Question** | Where should a value's *physical packing/layout* live — in its **type**, in its **runtime metadata**, or as a **schedule artifact decided at a lowering stage**? And what are the second- and third-order effects of each, given Mycelium's constraints? |
| **Closes (partial)** | survey coverage gap on representation-aware array languages (Halide/schedule separation; Futhark/Dex per T2.4) |

> RFC-0001 §4.1 tentatively chose **metadata** (packing in `Meta.physical`). This note re-examines that against the prior art the survey deferred, surfaces a **third option RFC-0001 did not consider** (schedule-staged), and isolates the distinction that actually drives the answer (lossless layout vs semantically-significant parameter). It is decision-support: it narrows and frames, and proposes experiments, but the ratification is the maintainer's.

---

## 1. Why this is not a local choice

Packing placement propagates into at least six other decisions already on the table: type compatibility/interop, content-addressed identity (RFC-0001 §4.6), the AOT-stable/JIT-dev split (ADR-009), what is statically provable (VR-1/3), how operations are dispatched, and memory soundness. The systems that have made this choice paid for it in exactly these places. This note treats those ripples as the primary subject, not the surface choice.

## 2. The distinction that must come first: lossless layout vs. semantically-significant parameter

The single most clarifying move is to stop treating "representation" as one thing:

- **Lossless physical layout.** Re-encodings that preserve the value *exactly*: ternary `Unpacked` vs `TwoBitPerTrit` vs `FiveTritPerByte` vs `I2S`/`TL1`/`TL2` (the BitNet packings from the survey). Same trits, same value, different bytes. There is *no observable semantic difference* — only performance and memory.
- **Semantically-significant parameter.** Choices that change the value or its guarantees: dense `dtype` precision (`F32` vs `BF16` changes representable values and bounds the error), and the **sparsity *class*** of a VSA value (whether `Sparse{max_active:k}` is a *promised* property), because capacity/crosstalk bounds depend on it (survey Area 2; Clarkson-Ubaru-Yang vs Frady-Sommer).

These pull in opposite directions and **should likely be placed differently** — conflating them is what makes the question feel hard. The rest of this note keeps them separate.

## 3. The three placement options

- **Option A — In the type.** Layout is part of the value's type (and thus of identity, of what every operation's signature mentions, of what the checker proves). *Exemplars: GHC `RuntimeRep`, MLIR `memref` layout, Rust `repr`.*
- **Option B — Runtime metadata.** Layout is a queryable attribute carried with the value; the type is layout-agnostic. *Exemplars: Apache Arrow logical-type-vs-physical-layout; database physical data independence.* (RFC-0001's current choice.)
- **Option C — Schedule-staged.** The type is layout-agnostic; layout is **unspecified at the high level and chosen at a lowering/"schedule" stage**, then recorded as inspectable metadata on the lowered form. *Exemplar: Halide's algorithm/schedule split (also TVM, Tiramisu).*

## 4. What each choice cost the systems that made it

### 4.1 In-type (A): GHC, MLIR, Rust

- **GHC representation polymorphism.** GHC lifts representation into the *kind* system (`TYPE :: RuntimeRep -> Type`), so `Int#` and `Bool` have different kinds. The decisive consequence, stated in the GHC User's Guide and the Eisenberg & Peyton Jones "Levity Polymorphism" paper (PLDI 2017): because the code generator must know a value's width and register class to compile a binding or a call, **no variable may have a representation-polymorphic type** — you cannot write the obvious generalization of `($)` over arbitrary representations. GHC further *defaults* `RuntimeRep` to lifted during inference and hides it when printing because exposing it is, in their words, unhelpful to most users. Lesson: layout-in-type forbids the natural polymorphism and is intrusive enough that the language deliberately hides it by default.
- **MLIR `memref`.** MLIR encodes layout in the type as a (semi-)affine map, with a sugared strided form. Three concrete lessons: (1) affine-map layouts proved *fragile and hard to read*, prompting a move to a first-class strided-layout attribute (LLVM review D132864); (2) a wrong layout in the type is a **memory-safety hole** — the `memref.transpose` stride bug (issue #54292) produced incorrect strides and an out-of-bounds access/segfault; (3) on an MLIR design thread, a maintainer argued that pushing the static/dynamic layout decision into *the middle of the compiler* leads to undecidable analyses or **heavy multi-versioning**, that JIT can help but is no panacea, and that **specializing too early is high-risk**. Lesson: layout-in-type buys analysis/transformation power but at the cost of multi-versioning and new soundness obligations, and the static/dynamic boundary placement is itself fraught.
- **Rust `repr`.** `repr(C/packed/transparent)` is part of the *type definition* (and is not parametric/polymorphic). `repr(packed)` opened a real **soundness hole** (issue #27060 → RFC 1240): taking a reference into a packed field can cause undefined behavior via unaligned access, which had to be made `unsafe`/a hard error. Separately, `repr(transparent)` exists precisely to *guarantee layout equivalence* so a newtype can be transmuted to its field. Lesson: fixing layout in the type creates soundness obligations that leak into the type rules, and you end up needing explicit *layout-equivalence/coercion* markers — which is exactly the friction Option A causes for Mycelium's no-implicit-conversion rule (§5).

### 4.2 Metadata / logical-physical split (B): Arrow, databases

- **Apache Arrow.** Arrow's columnar spec explicitly separates the **logical type** (application-facing semantics) from the **physical layout** (memory representation "without taking into account any value semantics"); a 32-bit int and a 32-bit float share a layout, and dictionary-/run-end-encoding are *layouts*, not types. This is Option B done at scale and it works for interchange. But an Arrow dev-list discussion (on Vortex-style advanced encodings and late materialization) names the precise second-order cost: once logical and physical are separated, **every compute kernel must handle all physical forms of a logical type, or a dispatcher must cast on the fly as a non-ideal fallback** — and operations like `slice`/`take` need *some* knowledge of the physical form, so a fully *opaque* layout breaks them. Lesson: Option B keeps types clean but relocates the cost to "the big switch goes somewhere" — kernel/dispatch complexity proportional to (operations × layouts) — and pure opacity is not viable.
- **Database physical data independence.** The classic ANSI/SPARC three-schema architecture (and Codd's relational model) institutionalized exactly this: the physical storage can change without changing the logical schema or applications. Decades of practice show the *principle* (separate logical from physical) is sound and durable; the cost lands in the query planner/optimizer that must bridge the gap — again, the dispatch/planning cost relocated, not removed.

### 4.3 Schedule-staged (C): Halide

- **Halide.** Ragan-Kelley et al. (PLDI 2013; CACM 2018) decouple the **algorithm** (what is computed — the output) from the **schedule** (storage layout + computation order). The program's *output depends only on the algorithm*; the schedule changes only performance. This is the cleanest articulation of "lossless layout is not semantics." The acknowledged cost, from the CACM retrospective: **modularizing scheduling choices without sacrificing performance remains an open problem**, and choosing a good schedule (auto-scheduling) is ongoing, hard work — i.e., you still must *decide* the layout somehow, and a poor schedule loses the order-of-magnitude performance the approach exists to capture. A second caveat from the compiler literature surfaced in the same searches: low-level IRs tend to *lose* layout/loop information, so the staged layout info must be deliberately preserved (which matches Mycelium's dimensional-persistence requirement, FR-M5).

## 5. Second- and third-order effects, mapped to Mycelium's constraints

| Constraint | A: in-type | B: metadata | C: schedule-staged |
|---|---|---|---|
| **No implicit conversion (FR-M3)** | **Worst.** Lossless re-pack changes the type → needs either a certified `swap` for a *bijective, zero-loss* re-encoding (heavy, semantically silly) or a special layout-coercion exception (a `repr(transparent)`-like hole — a place "implicit" creeps back in). | **Good.** Re-pack doesn't change the type; no swap, no exception. | **Best.** Re-pack isn't even expressible at the type level; it's a schedule choice, so the rule is never engaged. |
| **Content-addressed identity (§4.6)** | **Costly.** Packing is in the type → in the hash → a function specialized to a packing is a *different definition*; re-packing forks identity and defeats Unison-style sharing/caching across dev vs AOT forms. | **Clean.** Identity is packing-independent (matches §4.6 intent). | **Clean.** Core-IR identity is packing-independent; the schedule is separate metadata on the lowered artifact. |
| **AOT-stable / JIT-dev split (ADR-009)** | **Forces early specialization** (the MLIR "specializing too early is high-risk" warning); stable and dev forms become different types → the type fork I worried about. | OK, but the AOT compiler must still *choose and bake* a layout and prove it sound — and metadata alone gives the compiler weak guidance. | **Best fit.** A "schedule" is exactly per-backend layout selection: JIT picks one schedule, AOT-stable bakes another, *same type and identity*. Directly mirrors ADR-009. |
| **Kernels/ops dispatch (the "big switch")** | Avoided at the type level (each packing is a distinct monomorphic type → code-gen knows the layout, GHC-style) — but at the cost of multi-versioning/monomorphization and no layout-polymorphism. | **Cost relocated to runtime** (Arrow's lesson): VSA/arithmetic ops must handle all packings or materialize/cast on the fly; pure opacity breaks layout-dependent ops. | Cost relocated to the **lowering stage**: each packing needs ops at the Substrate-IR level, but Core-IR ops stay packing-free, and the dispatch is *explicit and inspectable* (a lowering pass, not a hidden runtime switch). |
| **Static provability of bounds (VR-1/3)** | Strong *iff* the parameter is semantic (precision, sparsity class). For lossless packing there is nothing to prove (it's lossless). | Weak: a metadata bound on a *value* isn't a static guarantee about a *definition*. | Neutral for lossless packing; semantic params should not be staged (see §6). |
| **No-black-box / soundness (NFR-3)** | Soundness obligation: the layout in the type must match the actual buffer or you get the Rust-packed / MLIR-transpose class of UB. | **Sharpest risk:** if the runtime trusts a wrong `physical` tag, it misreads memory. Metadata must be *authoritative and checked*, not advisory. | The schedule is a checked lowering decision validated against the interpreter reference semantics (NFR-7) — the soundness check has a natural home. |
| **Human + AI intelligibility (G10)** | More type information (good for some reasoning) but more types and visible noise (GHC hides it for a reason). | Simplest surface; layout is queryable when wanted. | Clean surface (layout absent from the algorithm) + an explicit, inspectable schedule artifact when wanted — arguably the best of both. |

**The sharpest single finding:** Option A collides head-on with Mycelium's no-implicit-conversion rule. A lossless re-pack is *bijective and zero-loss*, yet under Option A it changes the type, so the language must either force a certified `swap` (absurd overhead for a no-op-on-values transform, and it would pollute the swap-certificate semantics RFC-0002 is meant to keep meaningful) or carve out a layout-coercion exception — and that exception is precisely a spot where "implicit, unaudited" behavior re-enters, which is the thing the project forbids. B and C both avoid this entirely.

## 6. Synthesis — where the evidence points (candidate, not decision)

The evidence converges on a **split answer driven by §2's distinction**:

1. **Lossless physical packing → Option C (schedule-staged), not A, and an upgrade from B.** Keep the *type* packing-agnostic (so interop and identity are clean and the no-implicit-conversion rule is never engaged), decide packing at a **lowering/schedule stage** (so AOT-stable and JIT-dev can choose differently for the *same* typed component — the ADR-009 fit), and record the chosen packing as **inspectable metadata on the lowered artifact** (so it's auditable and the soundness check has a home, NFR-7). This is strictly better than RFC-0001's current "free-floating runtime metadata" framing: it gets B's clean types *and* gives the layout *choice* an explicit, staged, checkable locus rather than an advisory tag.
2. **Semantically-significant parameters → stay in the type (or as a static refinement), i.e., Option A for these.** `dtype` precision belongs in the `Dense` type (RFC-0001 already does this). The VSA **sparsity *class*** (`Sparse{max_active:k}` as a *promise*) is the interesting case: if capacity/crosstalk bounds are to be *statically* checked (VR-3), the class should be a type-level refinement, not merely observed runtime metadata — even though the *observed* sparsity remains runtime metadata. This is RFC-0001's flagged open question, and DN-01's view is that the answer differs from packing precisely because sparsity is semantic, not lossless.

Net: **packing is a schedule concern; precision and sparsity-class are type/refinement concerns.** This dissolves the original dilemma rather than resolving it by fiat.

Costs this candidate still owes (carried, not hidden): Halide's open problem — you must *choose* the schedule, and a bad one forfeits performance; and each packing still needs Substrate-IR operations (the dispatch cost relocates to lowering, it does not vanish — Arrow's lesson). Both are acceptable *because they are explicit and staged*, but they are real and should be budgeted.

## 7. Residual uncertainties & experiments to resolve them

- **E1 (does staging cost too much performance?).** Prototype one ternary kernel (e.g., balanced-ternary dot product or a `bundle`) at Core IR with packing unspecified, lower it to two schedules (`Unpacked`, `TL2`), and measure: does the staged path reach hand-packed performance? Halide says modular scheduling-without-perf-loss is unsolved in general — measure whether it holds *here*, where the packing set is tiny and fixed (5 schemes), which is a far easier regime than Halide's. *Resolves whether C is viable vs. forced back to A's early specialization.*
- **E2 (sparsity: type-refinement vs runtime).** Take the P0.1 bundling-bound probe and check whether the capacity bound can be discharged with sparsity as a *static refinement* vs only as runtime metadata. *Resolves the §6.2 open question and feeds RFC-0003.*
- **E3 (soundness of layout metadata).** Construct the Mycelium analogue of the MLIR-transpose / Rust-packed bug: can a wrong `physical`/schedule tag cause a memory misread, and does the NFR-7 reference-equivalence check catch it? *Resolves whether the metadata/schedule must be a checked artifact (expected: yes).*
- **E4 (AI/human surface).** Small slice of the G10 LLM experiment: generate code under "packing in type" vs "packing absent from type" surfaces; measure error rate and whether models spuriously over-specify packing. *Resolves the intelligibility column with data rather than assertion.*

## 8. Implications for RFC-0001 (if this note is accepted)

- **§4.1:** make §2's lossless-vs-semantic distinction explicit and normative; state that *lossless physical packing is not a type distinction*.
- **§4.1/§4.5:** move packing from "free-floating `Meta.physical`" to a **schedule artifact bound at lowering** (defined fully in RFC-0004, Execution/Lowering), with `Meta.physical` retained as the *inspectable record* of the chosen schedule on a lowered value (not the decision locus).
- **§4.3 / §4.5 (open question):** elevate VSA **sparsity class** to a candidate *static refinement* (decision deferred to RFC-0003 + E2), distinct from observed runtime sparsity.
- **No change** to `dtype`-in-type for dense (already correct under §6.2).

If the note is **not** accepted, the fallback ranking is **B (current RFC-0001 choice) > A**: keep packing as runtime metadata, never put lossless packing in the type — because A's collision with no-implicit-conversion (§5) is the worst outcome for this project specifically.

---

## Meta
- Status flow: Research → (feeds) RFC-0001 revision → Accepted/Rejected as part of that RFC's ratification. Append-only, like the ADRs.
- This note relied on web research into GHC representation polymorphism, MLIR `memref` layout, Halide's algorithm/schedule split, Apache Arrow's logical/physical separation, Rust `repr`, and the database data-independence principle; claims are attributed inline to those sources and paraphrased.
- Honest limits: the performance viability of Option C for Mycelium's specific (small, fixed) packing set is **unmeasured** — E1 is the gating experiment; everything in §6 is contingent on it. The Halide "open problem" caveat is about the general case and may not bind a 5-scheme regime, but that is a hypothesis to test, not a settled fact.