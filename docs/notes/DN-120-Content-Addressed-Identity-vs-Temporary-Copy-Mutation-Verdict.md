# Design Note DN-120 — Content-Addressed Identity vs. Temporary-Copy Mutation: Solved-by-Design (Verdict)

| Field | Value |
|---|---|
| **Note** | DN-120 |
| **Status** | **Draft** (2026-07-11). A verdict note recording a finding, not a new mechanism design — it **enacts nothing**, **ratifies nothing**, and **moves no other doc's status** (house rule #3, append-only). It does not edit `crates/mycelium-l1/**`, `crates/mycelium-std-runtime/**`, `issues.yaml`, `CHANGELOG.md`, or `Doc-Index.md` (integration-owned; FLAGGED in §6). |
| **Decides** | *Records, for ratification:* the ADR-003 disclosed residual — "content-addressed identity ⇒ efficient temporary-copy mutation may remain to be solved" — is **SOLVED-BY-DESIGN**. **DN-35 §5** already *is* the content-address-coherence answer (reuse fires only at `rc==1`, i.e. no live alias of the old identity; the intern/hash-cons table modeled as a weak map, evicted-or-copied on reuse). This note does **not** propose a new mechanism — doing so would re-litigate an already-ratified design (mitigation #14) — it clarifies exactly which slice of that design is landed today and which slice remains open, so the residual is not mistaken for an unsolved problem. |
| **Feeds** | ADR-003 (content-addressed identity — the disclosed residual this note closes out); DN-32 §2.2 (Layer-2 RC / `rc==1` reuse, cross-referenced 2026-07-11); DN-33 §6 (MEM-4 static uniqueness, Increment 2 reuse annotation, cross-referenced 2026-07-11); DN-35 (env-machine reclamation — Accepted, the design this note defers to); DN-118 (the transpiler `&mut`-under-alias lane, a distinct problem this note explicitly does not cover). |
| **Grounds on** | ADR-003 (content-addressed value identity); DN-32 §2.2 (three-layer hybrid memory — optimized RC, `rc==1` in-place reuse bullet); DN-33 §6 (Layer-1 static uniqueness analysis — Increment 2 `rc==1` reuse annotation); DN-35 §5 (the content-address side-condition: reuse-at-rc==1, weak intern table, evict-or-copy); DN-109 §6.1/D7 (the `&mut`-aliasing trap — a *different* problem, routed to the transpiler lane, not this one); DN-118 (the `FnMut` closure-lane enabler that carries D7 forward). VR-5 (no tag upgraded past its basis); G2 (no black-box swap — a reuse-vs-copy choice stays EXPLAIN-recorded); KISS/YAGNI (defer to the ratified DN-35 design rather than invent a parallel one). |
| **Date** | July 11, 2026 |
| **Task** | Record the verdict on the content-addressed-identity vs. temporary-copy-mutation residual — a docs-only close-out of a disclosed-but-unscoped gap, not a new design. |

> **Grounding + honesty (house rule #4 / VR-5 / G2).** This note's verdict rests on **DN-35 §5**
> (Accepted 2026-06-26) and the runtime substrate it's built on (`crates/mycelium-std-runtime/src/rc.rs`,
> read 2026-07-11). One correction against the task's own initial framing, made explicitly rather than
> parroted (no sycophancy): the claim that "`eval.rs` uses `Arc::get_mut` for in-place *mutation* of a
> unique value" **overstates what the L1 evaluator does today** — `eval.rs`'s own doc comments
> (`crates/mycelium-l1/src/eval.rs:56-76`) state `Data` values are **immutable and acyclic by
> construction** and **no code path mutates a live `Data`'s fields in place**; `Arc::get_mut` there
> gates the **iterative `Drop` dismantle** (an efficient-freeing optimization, M-994/M-979), not a
> live-value mutation path. The genuinely-landed piece is the **rc==1 detection gate** itself
> (`RcCell::drop_ref` → `RcProbe::UniqueOwner`, `Exact`-tagged, `mycelium-std-runtime/src/rc.rs`); the
> **reuse-write** (turning a `UniqueOwner(T)` into a new value by writing into the same allocation) is
> `Declared`, per that module's own doc comment ("no measurement yet", DN-32 §6a) — not yet built at
> either the interpreter or the AOT tier. This sharpens, but does not change, the verdict below.

---

## §1 The residual, as disclosed

ADR-003 ratifies content-addressed value identity (RFC-0001 §4.6): a value's identity **is** its
content hash. The ADR discloses a residual it does not itself resolve: *if the runtime ever reuses a
cell's storage in place for efficiency (a "temporary copy" mutated and handed back as if new), does
any live reference still holding the **old** content-hash identity observe a silently-changed value?*
Left unscoped in the docs/register, this reads as an **open problem** — it is not. It was solved by
design in **DN-35 §5**, written as the follow-on to DN-32/DN-33's memory-model work; this note's job is
only to make that connection explicit and correct the register lag (mitigation #14: the codebase and
the ratified corpus are ground truth, not the disclosed-residual phrasing that predates DN-35).

## §2 The verdict

**SOLVED-BY-DESIGN.** DN-35 §5 ("The one novel obligation — HARDER: in-place reuse vs
content-address identity") *is* the content-address-coherence answer:

- **`rc==1 ⇒` no live alias `⇒` no observer of the old identity.** Reuse of a cell's storage fires
  **only** when the runtime probe proves `rc==1` — if the count is 1, no other live reference exists,
  so no computation can observe the old content-hash pointing at changed bytes. The mutation is
  unobservable because the old value is *consumed*, not aliased (DN-35 §5, the Perceus/Clojure-transient
  argument).
- **The intern/hash-cons table is modeled as a *weak* map.** If a uniquely-owned cell is also pinned by
  a content-address intern table, its effective count is `≥2` via that table — so the reuse path either
  evicts the stale table entry first (then reuses) or, if it cannot evict atomically, copies instead of
  reusing (DN-35 §5's "evict-or-copy" rule).
- **Guarantee tag:** `Empirical`/`Declared` per DN-35 §5 itself — there is no off-the-shelf mechanized
  proof of this side-condition (no on-the-shelf theorem covers a content-addressed language's
  reuse-vs-identity interaction); it is a **Mycelium-stated, property-tested side-condition** (DN-35's
  own S5), never claimed `Proven` (VR-5).

## §3 What is landed today vs. what remains (the corrected boundary)

Verified against the tree at dev tip `b36f0ef4` (2026-07-11):

| Piece | Status | Basis |
|---|---|---|
| **rc==1 detection gate** (the "no live alias" side-condition's precondition) | **Landed, `Exact`** | `crates/mycelium-std-runtime/src/rc.rs`: `RcCell::drop_ref` returns `RcProbe::UniqueOwner(T)` iff `Rc::strong_count == 1` before decrement; `RcProbe::Shared` otherwise. This is exactly DN-35 §5's rc==1 precondition, implemented and tagged `Exact` (reads `Rc::strong_count` directly). |
| **L1 evaluator's `Drop`-path use of the same discipline** | **Landed, consistent, but a different path** | `crates/mycelium-l1/src/eval.rs:56-140` (M-994/M-979): `Data` is documented immutable-by-construction — **no code path mutates a live `Data` in place today** — so there is no live-mutation identity hazard to guard against yet. `Arc::get_mut` there gates the iterative dismantle-on-`Drop` (efficient freeing of a uniquely-owned subtree; a still-shared subtree is left intact), which independently respects the same rc==1-only discipline, but is not the "mutate a unique value, COW a shared one" runtime path a first read of the residual might suggest. |
| **The reuse-write itself** (turning a `UniqueOwner(T)` into a new value via an in-place byte-write, the actual FBIP optimization) | **Not yet built — `Declared`** | `rc.rs`'s own doc comment: *"the caller holds the owned `T` and may reuse the value's storage… The probe is `Declared` as a perf optimization — no measurement yet (DN-32 §6a)."* Not wired at the interpreter tier (no reuse-write exists — `Data` is never mutated) nor at the AOT tier. |
| **AOT env-machine reclamation** (DN-35's own named deferred build) | **Not yet built — the forward epic** | DN-35 itself: *"Accepted ratifies the direction, not an implementation — it enacts no code; the build is the forward epic (E12 Increment 3 / task #6)."* Scoped explicitly to threading real reclamation (free / in-place reuse / region batching) into the **AOT env-machine**, which today "Rust-manages values" with the §9 audit record as an additive trail only. |

**Net:** the detection-and-safety *side-condition* (rc==1 gating) is landed and `Exact` wherever
reuse could occur; the *performance optimization* it protects (the actual reuse-write) is `Declared`
and unbuilt everywhere, with DN-35 §9 (Increments 1–3) as its own ratified, not-yet-executed build
plan, tracked as **E12 Increment 3 / task #6**. This note does not reopen or resize that build plan —
it is desktop-held, already-scoped implementation work under an already-ratified design.

## §4 Why this is not a new mechanism to design (mitigation #14)

Re-designing a resolution here would **re-litigate a solved design**: DN-32 (three-layer memory) →
DN-33 (static uniqueness / MEM-4) → DN-35 (env-machine reclamation, Accepted 2026-06-26, including the
§5 content-address side-condition) already form one coherent, ratified chain. This note's only
contribution is closing the loop from ADR-003's disclosed residual to that chain, and correcting the
landed/open boundary (§3) so the residual is not mistaken for unsolved. No alternative mechanism is
proposed, ranked, or needed.

## §5 What this note explicitly does NOT cover

**The transpiler `&mut`-under-alias case is a different problem.** Rust's `&mut T` aliasing (whether
two live references to the *same* mutable binding could observably diverge) is DN-109 §6.1/D7's
concern, routed to the **DN-118 `FnMut`-lane** closure-to-value-semantics enabler — a *source-language*
aliasing-proof problem (can `syn` prove non-aliasing without `rustc`'s borrow checker?), distinct from
this note's *runtime-storage-reuse* identity question (does reusing a uniquely-owned cell's bytes ever
let a live reference observe a stale identity?). The two are related in spirit (both are about safe
reuse under uniqueness) but are solved by different mechanisms at different layers — conflating them
would mis-scope both. DN-120 covers only the latter.

## §6 Definition of Done + FLAGGED items (integration-owned, not applied here)

**For this DN (done at authoring):**
1. The verdict recorded, with the DN-35 §5 mechanism restated and cited — **done** (§2).
2. The landed-vs-open boundary corrected against the tree, including the correction to the task's own
   initial `eval.rs` framing — **done** (§3).
3. The DN-118/D7 boundary drawn so the two problems are not conflated — **done** (§5).
4. Forward cross-references added at DN-32 §2.2 and DN-33 §6 pointing here and to DN-35 §5 — **done**
   (see those files' 2026-07-11 Meta-changelog entries; append-only, no normative text changed).

**For maintainer ratification (what "Accepted" requires — this note does not self-ratify):**
5. Confirm the verdict (§2): the ADR-003 residual is closed by DN-35 §5, no new DN is warranted.
6. Confirm the corrected boundary (§3) as the honest present state — in particular that the reuse-write
   itself stays `Declared` until the E12 Increment 3 epic lands and is property-tested (DN-35's own S5).
7. **FLAG to the integrator** (`Doc-Index.md`/`CHANGELOG.md`/`issues.yaml` are integration-owned — not
   edited here): add a Design-Notes row for `DN-120 — Content-Addressed Identity vs. Temporary-Copy
   Mutation: Solved-by-Design (Draft)`; add a dated `CHANGELOG.md` entry; if `issues.yaml` tracks the
   ADR-003 residual anywhere as an open item, close it out with `doc_refs: corpus:DN-120` rather than
   leaving it looking unscoped.
Status stays **Draft** until 5–6 are ratified.

## §7 Grounding

- **ADR-003** — content-addressed value identity; the disclosed residual this note closes. Grounds §1.
- **`docs/notes/DN-32-Three-Layer-Hybrid-Memory-Architecture.md` §2.2** (read 2026-07-11) — the `rc==1`
  in-place-reuse bullet this note's verdict attaches to; cross-referenced forward to DN-35 §5/DN-120
  (2026-07-11, append-only). Grounds §2, §3.
- **`docs/notes/DN-33-Layer1-Static-Uniqueness-Analysis.md` §6** (read 2026-07-11) — MEM-4 Increment 2
  (`rc==1` reuse annotation); cross-referenced forward to DN-35 §5/DN-120 (2026-07-11, append-only).
  Grounds §2, §3.
- **`docs/notes/DN-35-Env-Machine-Reclamation.md` §5** (read 2026-07-11, full) — the content-address
  side-condition (reuse-at-rc==1, weak intern table, evict-or-copy); **Accepted**, build deferred to
  E12 Increment 3 / task #6. Grounds §2, §3, §4.
- **`crates/mycelium-std-runtime/src/rc.rs`** (read 2026-07-11, `:1-70`) — `RcCell`/`RcProbe::UniqueOwner`,
  `Exact`-tagged rc==1 detection; the reuse-write itself documented `Declared`/unmeasured. Grounds §3.
- **`crates/mycelium-l1/src/eval.rs`** (read 2026-07-11, `:56-140`) — `Data` immutability-by-construction;
  `Arc::get_mut`-gated iterative `Drop` dismantle (M-994/M-979) — the correction basis for §3/the
  epigraph note.
- **`docs/notes/DN-109-Idiom-Optimal-Transpilation-And-Structural-Remapping.md` §6.1/D7** (read
  2026-07-11) — the `&mut`-aliasing trap, the different problem routed to DN-118. Grounds §5.
- **`docs/notes/DN-118-Closure-To-Value-Semantics-Transpiler-Enabler-And-Native-Conformance-Contract.md`**
  (title read 2026-07-11) — the `FnMut` lane carrying D7 forward. Grounds §5.
- **House rules:** #3 (append-only), #4 (grounded claims, no sycophancy), G2, VR-5, KISS/YAGNI,
  mitigation #14 (verify against the codebase before treating a disclosed gap as open).

---

## Meta — changelog

- **2026-07-11 — Created (Draft).** Records the verdict that ADR-003's disclosed content-addressed-
  identity vs. temporary-copy-mutation residual is **solved-by-design**: DN-35 §5 is the answer
  (reuse fires only at rc==1; weak-intern-table evict-or-copy). Corrects the landed/open boundary
  against the tree at dev tip `b36f0ef4` (§3): the rc==1 **detection gate** is landed and `Exact`
  (`mycelium-std-runtime/src/rc.rs`); the actual **reuse-write** optimization remains `Declared` and
  unbuilt at both the interpreter and AOT tiers, with DN-35's own E12 Increment 3 / task #6 as its
  ratified, not-yet-executed build plan — corrected from an initial framing that read `eval.rs`'s
  `Arc::get_mut`-gated `Drop` dismantle as live-value in-place mutation (house rule #4: `Data` is
  documented immutable-by-construction, never mutated in place today). Draws the boundary with the
  distinct DN-109 §6.1/D7 `&mut`-aliasing problem (routed to the DN-118 `FnMut` lane), so the two are
  not conflated (§5). No new mechanism proposed (mitigation #14 — would re-litigate a solved design).
  Added forward cross-references at DN-32 §2.2 and DN-33 §6 (append-only, no normative text changed in
  either). Authored the DN only — no edit to `issues.yaml`, `CHANGELOG.md`, or `Doc-Index.md`
  (integration-owned; FLAGGED, §6). Append-only; status advances only by maintainer ratification
  (house rule #3).
