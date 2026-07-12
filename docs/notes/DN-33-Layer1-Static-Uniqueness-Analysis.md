# Design Note DN-33 — Layer-1 Static Uniqueness Analysis (MEM-4) & the Cross-Hypha Reconciliation

| Field | Value |
|---|---|
| **Note** | DN-33 |
| **Status** | **Accepted** (2026-06-25; ratified Draft → Accepted by the maintainer — the §8 deliberation is settled, see §8.1). Was Draft/advisory; the maintainer ruled the three load-bearing §8 questions (Q1/Q2/Q3) and the Q4–Q7 recommendations are adopted as defaults, so the **design direction is now ratified**. Still **enacts no code** — it is the agreed design the MEM-4 implementation epic builds to (a build that remains gated on the §8 Q5 measurement discipline as each increment lands). The cross-hypha commit (Q1 → Option A) feeds the eventual RFC-0027 follow-on; this note does not itself move RFC-0027's status. |
| **Feeds** | the DN-32 **Layer-2 static uniqueness analysis** leg (DN-32 §2.2 / §6b) and the **cross-hypha reconciliation sub-question** (DN-32 §7 / RFC-0027 §12); anchors **MEM-4** in the E12 memory-model build (`docs/planning/E12-Memory-Model-Build-Plan.md`) |
| **Date** | June 25, 2026 |
| **Decides** | *Nothing normatively* — advisory + design-direction capture. Records (1) that **MEM-4 is an *additive, semantics-preserving* compiler lowering pass** that statically elides provably-redundant RC operations, with the runtime `RcCell` probe (MEM-2) as the sound fallback; (2) a **recommended incremental decomposition** (non-escaping borrow elision → `rc==1` reuse annotation → full FIP static guarantee); (3) a **recommendation for the cross-hypha sub-question** — **Option A (sole-move-only / affine-channel boundary) for R1**, Option B (shared-crosses-atomic-RC) deferred to R2 — each at its supportable grounding strength. |
| **Task** | E12 / MEM-4 (the deferred Layer-1 static analysis leg of DN-32; RFC-0027 §12 reconciliation) |

> **Posture (transparency rule / VR-5 / G2).** This note **synthesises a sourced research dossier**
> into a design direction; it **enacts nothing** — no RFC/ADR status moves, no normative text
> changes, no code or property test ships. Every Mycelium-specific behavioural and performance claim
> herein is **`Declared`** (a design intent / target), not measured or proven in-repo. External
> prior-art claims are tagged at the strength their sources support (typically **`Empirical`** — the
> systems exist and are deployed — never `Proven` without a checked basis). The cross-hypha
> recommendation (§5) is an **argument on the corpus + prior art**, not a measured ergonomic result;
> its honest risk (programs may need restructuring) is surfaced, not buried (G2). MEM-4 is the
> **hardest, most kernel-growing** leg of DN-32 (DN-32 §6b) — this note's central discipline is to
> keep it *additive* so a bug downgrades to a missed optimization, **never** to unsafety (§2/§4).

---

## §1 Purpose

DN-32 settled the runtime memory architecture as a three-layer hybrid and RFC-0027 fixed the
reclamation **mechanism** (precise reference counting; LR-9 acyclicity *is* Perceus's garbage-free
precondition). MEM-1/2/3 then landed the runtime substrate — the EXPLAIN/audit record, the `RcCell`
probe (`RcZero` trigger), region batched scope-exit (`ScopeExit` trigger), and the channel-teardown
`ChannelClose` trigger — with all three live triggers wired and every MEM-1 placeholder id
canonicalized (`docs/planning/E12-Memory-Model-Build-Plan.md`, Waves 1–4).

Two design points remain **deliberately open** after that runtime substrate:

1. **The Layer-1 *static* leg (MEM-4).** DN-32 §2.2 describes "static uniqueness analysis removes RC
   ops". The runtime `RcCell::drop_ref → UniqueOwner` probe (MEM-2) is the *dynamic* fallback; MEM-4
   is the compile-time analysis that **elides** that probe wherever uniqueness is statically provable.
   DN-32 §6b flags this as the hardest leg and a real **KC-3 tension**.
2. **The cross-hypha reconciliation sub-question (DN-32 §7 / RFC-0027 §12).** Does a *shared* value
   ever cross a hypha boundary (→ atomic RC), or does only *sole* ownership cross (→ affine move, no
   cross-hypha refcount)?

This note captures the research-backed direction for both, **design-first** (per the maintainer's
"sequence through the DN backed by research" directive), so that MEM-4 implementation can proceed
against a deliberated target rather than a guess (KC-3 / G2 — flag, don't guess).

## §2 What MEM-4 is — and is not (the additive-optimization principle)

**MEM-4 is a compiler lowering pass**, between the typed Core IR and the RC-annotated lowering, that
identifies program points where a `clone_ref`/`drop_ref` pair (or a lone `drop_ref` whose
`UniqueOwner` branch is statically certain) is **redundant** and elides it. It does **not** change the
semantics of any Mycelium program; it removes RC instructions whose effect is provably a no-op.

The load-bearing design principle (from the Perceus/Koka deployment evidence — §3):

> **MEM-4 only ever *elides* provably-redundant RC ops. The runtime `RcCell` probe (MEM-2) is the
> sound fallback for everything MEM-4 cannot prove.**

Consequences, which are *why* this is the right shape for a KC-3 kernel:

- **Soundness is required; completeness is optional.** If the analysis *misses* a uniqueness
  opportunity (false negative), the runtime probe simply fires — a **performance** regression, never
  unsafety. The analysis must only be careful never to elide when uniqueness is *not* guaranteed (a
  false positive would be a use-after-free) — i.e. it must be **sound but may be incomplete**.
  *(`Declared` — this is the design intent; the soundness obligation itself is §8 Q3.)*
- **A bug in MEM-4 degrades gracefully.** Because the dynamic path is always present and correct, a
  defect in the static pass costs throughput, not memory safety — the failure mode is benign. This is
  the property that makes adding a non-trivial analysis pass *tolerable* under KC-3.
- **MEM-4 is elective.** It can be absent (the runtime model is complete without it), staged in
  increments (§6), or gated by build profile — none of which changes observable semantics.

## §3 The design space — static uniqueness analysis (prior art, tagged)

What such an analysis computes, and what Mycelium's value model buys us. (Sources in §9.)

- **Perceus precise dup/drop (`Empirical`).** Perceus (Reinking–Xie–de Moura–Leijen, PLDI'21)
  inserts `dup`/`drop` by an owned-vs-borrowed analysis per use site: *owned* params get `dup` at all
  but the last occurrence on each path; *borrowed* (non-escaping, non-consumed) params get **neither**.
  Cycle-free programs become *garbage-free*. The garbage-free theorem's **precondition is
  acyclicity** — which **LR-9 supplies for free** (no cycle detector needed; RFC-0027 §7.1).
- **Borrow inference / frame-limited reuse (`Empirical`).** Lorenzen–Leijen (ICFP'22; Lorenzen 2021
  thesis) elide RC inc/dec on parameters that *do not escape* (not stored, returned, or captured).
  The result is *frame-limited* (peak memory within a constant factor of the live set), not fully
  garbage-free — but it removes the majority of RC traffic on the hot path, and the non-escape
  precondition is a **conservative flow analysis**, achievable in a lowering pass without dependent
  types.
- **Runtime `rc==1` reuse is distinct from static proof (`Empirical`).** Koka's FBIP reuse uses a
  **runtime** `rc==1` check (`fbip`), while `fip` gives a *static* guarantee only when arguments are
  unique. **Mycelium already has the runtime half** — `RcCell::drop_ref → UniqueOwner` *is* the
  `rc==1` probe (RFC-0027 §10.1). MEM-4 adds the static half *on top*, eliding the probe where the
  compiler can prove uniqueness; the two compose (static ⊂ dynamic).
- **Clean uniqueness types / ASAP — the cost signal (`Empirical`).** Purely-static uniqueness
  (Clean, Mercury unique modes) is sound but *incomplete*; fully-automatic static deallocation (ASAP,
  Cambridge TR-908) is expensive and historically scoped out mutation/polymorphism/HOFs. The lesson:
  **complete static deallocation is costly; the Perceus "dynamic fallback + static elision" split is
  the pragmatic choice** — which is exactly the shape MEM-4 takes (§2).
- **Why Mycelium's model shrinks the problem (`Declared`, grounded on LR-8/LR-9 + RFC-0027 §7.1).**
  - **No write barriers / no mutation-alias tracking** (LR-8 immutability) — the *hardest* part of
    Rust-style borrow checking (policing aliased mutation) is simply **absent** (research/03 §T3.5).
  - **No cycle detection** (LR-9 acyclicity) — the garbage-free precondition is a type-system
    invariant, not something the analysis must prove.
  - **Content-addressing** — two values with the same hash are interchangeable, simplifying alias
    reasoning. The core query reduces to "is this value's RC exactly 1 at this program point?" — *not*
    "who may alias and mutate this?"

## §4 Soundness & the KC-3 tension (keeping the trusted base small)

DN-32 §6b names the tension plainly: a borrow/uniqueness analysis "adds compiler surface, inference
rules, and a correctness obligation." Three disciplines keep MEM-4 inside the KC-3 budget:

1. **Lowering pass, not a type-system extension.** MEM-4 should inspect the *already-typed* Core IR
   and decide elision in a pass that produces RC-annotated IR — the **type checker need not know about
   RC**. The trusted base (Core IR + type checker) is unchanged; MEM-4's correctness obligation is a
   property of the pass alone. (RFC-0027 §8 already flags that "RC emission must be added as a
   lowering pass" — the *approach* transfers, the *exact* Perceus algorithm needs adaptation to a
   typed term language: `Declared`.)
2. **Additive-only (the §2 principle).** Because the runtime probe is the sound fallback, the pass is
   permitted to be incomplete, so it can ship in small increments (§6) and be measured before each is
   extended — no flag-day, no all-or-nothing correctness cliff.
3. **Watch + measure each increment (DN-32 §6b).** Every increment's kernel-node cost is tracked
   against the KC-3 / auditability budget before the next is committed. The specific measurement gate
   is an open question (§8 Q5) — until it is stated, the discipline is `Declared` and unenforceable,
   which this note flags rather than papers over (G2).

## §5 Cross-hypha reconciliation (DN-32 §7 / RFC-0027 §12)

**The sub-question.** RFC-0027 §7.3 routes cross-hypha transfer over the **affine channel protocol**
(the `Sender`/`Receiver` pair is non-`Clone` — exactly-one-owner transfer, no cross-hypha RC). DN-32
§2.2 says RC becomes **atomic after cross-hypha transfer** for genuinely shared values. These conflict
*only* at the boundary: **what kinds of values may cross a hypha boundary?**

**Option A — sole-ownership move only (affine / iso boundary).** Only a *sole-owned* value crosses; a
shared (`rc > 1`) value cannot be sent. Cross-hypha transfer is a **move**, needing zero atomic RC —
the protocol enforces sole ownership at the transfer point.
- *Prior art (`Empirical`):* **Pony `iso` + `consume`** (single mutable alias → safe actor transfer,
  no RC); **Rust `Box<T>: Send` vs `Rc<T>: !Send`** (unique ownership moves across threads with no
  atomic ops; non-atomic RC is confined to one thread — the exact "non-atomic-within / move-across"
  boundary). RFC-0027 §7.3's channel-close synchronization gives the same guarantee as Pony's
  `consume`. LR-9 already removes the need for Pony's ORCA cycle machinery.
- *Trade-off:* a program **cannot** hold a shared `RcCell<T>` (`rc > 1`) in one hypha and send it to
  another; multi-hypha shared *read* access must restructure around the affine move model. This is a
  real ergonomic restriction whose cost is **unmeasured** (`Declared`).

**Option B — shared values may cross (atomic RC engages).** A shared, immutable value crossing a
hypha boundary is promoted to **atomic RC**.
- *Prior art (`Empirical`):* **Rust `Arc<T>`** (atomic RC = thread-safe sharing, at an atomic-op
  cost); **Pony `val`** (immutable ⇒ safely shareable across actors — maps to Mycelium's LR-8
  immutability, so the only question is reclamation ordering, not mutation safety); **Verona `cown`**
  (and Verona's trend toward *non-atomic* RC under region discipline — evidence that atomic RC should
  be *minimized*, not assumed universal).
- *Trade-off:* more expressive (shared immutable values flow freely across hyphae) at the cost of
  atomic-RC ops on the shared path and a more complex ownership/release protocol.

**Recommendation (argument, `Declared`).** **Adopt Option A for R1**; reserve Option B for R2. The
affine channel protocol *already* enforces sole ownership at the boundary (RFC-0027 §7.3), giving
Pony-iso semantics with **no** capability system and **no** atomic RC — keeping RC strictly
non-atomic intra-hypha, which is Layer-2's main performance advantage. Option B's expressiveness has
no R1 use case that justifies its complexity; introduce atomic RC when genuinely-shared values must
cross node boundaries (R2, when `xloc`/`mesh` land). *Grounding:* the prior-art existence is
`Empirical`; "Option A is simpler **and sufficient** for R1" is `Declared` — the honest risk is that
real programs may need restructuring to avoid cross-hypha shared `RcCell<T>`, a cost unknown until
example programs exist. This recommendation keeps `RcCell<T>: !Send` (MEM-2's current shape) and so
is the **lower-divergence** path from what is already built.

## §6 Proposed MEM-4 decomposition (incremental, swarm-sized)

> **Forward cross-reference (2026-07-11, append-only note — no normative text changed).** The static
> uniqueness proof this section builds toward interacts with content-addressed value identity
> (RFC-0001 §4.6 / ADR-003) whenever a proven-unique value is reused in place: **DN-35 §5** is the
> content-address-coherence answer (reuse fires only at rc==1 ⇒ no live alias of the old identity;
> the intern/hash-cons table modeled as a weak map, evict-or-copy on reuse) that Increment 2's
> `rc==1` reuse annotation composes with. **DN-120** records the verdict that this is solved-by-design
> (`Draft`, pending ratification) — not a new mechanism to design here.

Smallest sound increment first; each measured before the next (DN-32 §6b / §4.3).

1. **Increment 1 — non-escaping borrow elision (smallest, highest-value).** At a call site whose
   argument does not escape the callee (not stored / returned / captured), elide the `clone_ref`
   before and the `drop_ref` at the call. **Testable as a static invariant:** for every elided pair,
   the refcount before and after the call is identical — a structural check on the Core IR, *no
   runtime property test needed*. Scope the first cut to direct calls / first-order values; defer
   closures + higher-order cases. KC-3 cost: one escape analysis + an owned/borrowed annotation per
   binding — small, well-precedented (Lean 4, Koka).
2. **Increment 2 — `rc==1` reuse annotation.** Where the analysis proves a value is sole-owned
   throughout its scope, emit a compile-time hint that the runtime `UniqueOwner` branch will always
   fire (FBIP reuse guaranteed). A performance annotation, not a semantic change. KC-3 cost: a
   last-use dataflow over the Core IR — heavier; defer until Increment 1 is measured.
3. **Increment 3 — full FIP static guarantee (Phase 3, deferred).** A `fip`-style type-system layer
   (Koka FP², ICFP'23) giving a *static* in-place guarantee with **no** runtime check. This is the
   hardest, most kernel-growing leg (DN-32 §6b) — Phase 3, after the runtime model is stable and
   Increments 1–2 are measured.

**Swarm sizing.** MEM-4 is a **single disjoint crate** (e.g. `crates/mycelium-mir-passes/`) consuming
Core IR → producing RC-annotated IR; it owns no shared files except an orchestrator-owned
`borrow_annotation` field on the Core IR binding type. The analysis logic is a leaf-agent task; the
increments are dependency-ordered waves, mirroring MEM-1→4.

### §6.1 Prerequisite gap — Increment 1 is BLOCKED on the RC-emission pipeline (investigated 2026-06-25)

A read-only investigation of the current tree (after E12 Wave-4 landed) confirms **MEM-4 Increment 1
is not implementable yet** — it has no input to operate on. MEM-4 elides *compiler-emitted* RC ops;
today there are none. The grounded findings (file:line evidence):

- **The Core IR exists but carries no ownership mode.** `crates/mycelium-core/src/node.rs` is the
  typed `Node` grammar (`Const/Var/Let/Op/Swap/Construct/Match/Lam/App/Fix/FixGroup`) — a closed
  typed term language. But `Node::Lam { param: VarId, … }` has `VarId = String` with **no
  owned/borrowed annotation field** on any binding site. That annotation is precisely the *output*
  of Increment 1, and the *representation* of it is the open question §8 Q2 — so the field cannot be
  added without first resolving Q2.
- **No RC-annotated IR layer / no `mir-passes` crate.** There is no `crates/mycelium-mir-passes/`
  (or `mycelium-core-mir`), and no IR type representing `Node` terms with `clone_ref`/`drop_ref`
  placed-or-elided. The output IR MEM-4 produces does not exist.
- **RC ops are hand-called in tests only — never emitted by a lowering.** `clone_ref`/`drop_ref`/
  `RcCell` appear in exactly four files, all within `mycelium-std-runtime` (the definition `rc.rs`,
  a comment in `scope_region.rs`, and the two test files). The L1 lowering (`mycelium-l1/src/elab.rs`,
  the *only* IR-to-IR transform in the repo) emits `Node` terms with **zero** RC annotations. So no
  pass inserts the `clone_ref`/`drop_ref` pairs that Increment 1 would elide.

**The prerequisite chain before Increment 1 can land** (each step is itself substantial — this is
language-frontend work, broader than "finishing the memory model"):
1. **Resolve §8 Q2** — the binding-site ownership-mode *representation* (a `BorrowMode` annotation on
   `Node` binding forms vs a separate borrow IR layer vs a compiler-internal flag). Type-system- and
   KC-3-significant — a **maintainer decision**, not research-resolvable.
2. Add the chosen ownership-mode field to `mycelium-core/src/node.rs` binding forms.
3. Create `crates/mycelium-mir-passes/` with an **RC-emission lowering pass** (`Node` → RC-annotated
   IR) — the step that *inserts* `clone_ref`/`drop_ref`, wired into the `elab.rs` → interp/AOT
   pipeline so RC ops are emitted structurally instead of hand-called.
4. **Only then** does Increment 1 (non-escaping borrow elision) slot in — annotating own-vs-borrow so
   step 3 inserts fewer ops.

**Consequence (VR-5/G2 — honest sequencing).** MEM-4 is **deferred-by-prerequisite**, not merely
deferred-by-priority. The runtime substrate (MEM-1..3 + the live triggers, landed Wave-4) is the
**sound, complete fallback** and stands on its own — the `RcCell` probe needs no static analysis to
be correct (§2). Building the RC-emission pipeline is a forward epic gated on the §8 Q2 decision; it
is not undertaken speculatively here (flag, don't guess).

> **Correction (2026-06-25, append-only — §6.1 above is now a preserved historical snapshot).** The
> "prerequisite gap" §6.1 describes has since been **resolved**; its file:line findings are now
> FALSE and must not be read as current state. The `crates/mycelium-mir-passes/` crate now **exists
> and is fully built**: `rc_ir.rs` (separate RC-annotated IR mirroring `Node` + `Dup`/`Drop`/
> `MoveUnique`), `emit.rs` (RC-emission lowering `emit_owned` + Increment-1 borrow elision
> `emit_elided` + Increment-2 reuse `emit_reuse`), `eval.rs` (differential check), `balance.rs`
> (`1 + dups == uses + drops` structural invariant), and `corpus.rs` (Q5 dup-reduction gate) — i.e.
> **Increments 1–2 have landed** (see the E12 build plan Wave entries). Two specific §6.1 claims are
> now stale: the "no `mycelium-mir-passes` crate / no RC-annotated IR" finding is contradicted by the
> built crate, and the prerequisite step **"add the chosen ownership-mode field to `node.rs`"** is
> **moot** — the §8.1 **Q2 ruling above** chose a *separate RC-annotated IR* with `mycelium-core/src/
> node.rs` left **pristine**, so no field was ever added (and none is needed). §6.1's prose is kept
> verbatim as the post-Wave-4 historical snapshot it was; this note neutralizes the standing claim.
> (Append-only — original text preserved per house rule #3; VR-5/G2.)

## §7 Honest scope (VR-5 — do not omit)

- Every Mycelium-specific claim here is **`Declared`** — nothing in MEM-4 is built; the increments,
  the soundness strategy, and the cross-hypha recommendation are **design intent**, not measured or
  proven results.
- External prior art (Perceus, Lorenzen, Pony, Rust, Verona, ASAP, Koka) is **`Empirical`** — the
  systems exist and are deployed; that they *transfer cleanly* to Mycelium's typed Core IR is
  `Declared` (RFC-0027 §8: the approach transfers, the exact algorithm needs adaptation).
- The cross-hypha §5 recommendation is an **argument**, not a measurement; its ergonomic cost is the
  named open risk (§8 Q1).
- This note **moves no status** and changes no normative text. Promotion past Draft is gated on the
  §8 deliberation + maintainer ratification (house rule #3, append-only). `RcCell`/`Region`/network
  remain exactly as built; nothing here is retro-applied to them.

## §8 Open questions (the deliberation agenda)

1. **Cross-hypha boundary commit (Option A vs B).** §5 recommends A for R1, B deferred to R2. Confirm,
   and gate B explicitly — the choice fixes whether `RcCell<T>` is ever `Send` (downstream type-system +
   cross-hypha API implications).
2. **Core IR borrow-annotation representation.** Mode/annotation on binding sites (Lean 4 `@`-borrow
   style) vs a separate borrow IR layer vs a compiler-internal flag invisible to the surface IR —
   affects KC-3 impact + auditability.
3. **Soundness proof strategy for the elision pass.** Mechanized proof (Coq/Lean) vs differential
   testing (run with/without elision on a program corpus, assert identical results + identical
   reclamation records) vs argument-by-structural-invariant (the pass fires only where Core IR typing
   already guarantees linearity). The transparency rule requires this be *stated*, not assumed.
4. **Interaction with `substrate`/`consume` (DN-02/03).** A `substrate`-typed binding is already
   known unique (affine). Does MEM-4's borrow analysis *subsume* `substrate` uniqueness, or is
   `substrate` a separate static-proof path feeding the same elision mechanism?
5. **Performance-measurement gate (makes DN-32 §6b enforceable).** The specific metric Increment 1
   must pass before Increment 2 is committed — RC-op-reduction ratio on a representative benchmark? a
   kernel-node budget vs a KC-3 baseline? Without it, "watch + measure" stays `Declared`.
6. **FIP user-surface annotation?** Expose a `fip`/`@unique`-style annotation (static in-place
   guarantee, user-visible) or keep MEM-4 fully invisible (inferred, more ergonomic)? A surface-language
   decision with real KC-3 implications.
7. **Frame-limited vs garbage-free target.** Increment 1 (borrow elision) yields *frame-limited*
   memory, not full *garbage-free*. State which Mycelium targets and why (frame-limited is sufficient
   for most systems use cases; the full guarantee costs more analysis).

## §8.1 Resolutions (ratified by the maintainer, 2026-06-25)

The §8 deliberation is **settled**. The three load-bearing questions were ruled directly; the
secondary four are adopted as the recorded defaults below (overridable by a later supersession, house
rule #3). These resolutions move this note **Draft → Accepted** and are the design the MEM-4 build
implements.

- **Q1 (cross-hypha boundary) → Option A for R1.** Only **sole ownership** crosses a hypha boundary
  (affine move; the RFC-0027 §7.3 channel protocol already enforces it); **`RcCell<T>` stays
  `!Send`**, no atomic RC. Option B (shared-crosses-atomic-RC) is **gated to R2** (`xloc`/`mesh`).
  Grounding unchanged (§5): prior art `Empirical`; "A sufficient for R1" `Declared`; the
  restructuring-cost risk is carried forward, to be revisited if R1 programs hit it.
- **Q2 (ownership-mode representation) → separate RC-annotated IR.** The mode (own vs borrow) and the
  emitted `dup`/`drop` ops live on a **new RC-annotated IR** produced by the lowering pass; the
  trusted Core IR (`mycelium-core/src/node.rs` `Node`) **stays pristine** — no `BorrowMode` field on
  kernel binding forms. This honours §4 ("lowering-pass-not-type-checker; trusted base unchanged") and
  KC-3: the kernel everyone audits does not grow; MEM-4's correctness obligation lives entirely in the
  (untrusted, optimisation-only) `mir-passes` crate. Chosen over the simpler "annotate `Node`" option
  precisely because that option would tax the trusted base (the merit argument beat the convenience).
- **Q3 (soundness strategy) → differential + structural invariant; tag `Empirical`.** Soundness is
  established by (a) a **differential test** — run the RC-annotated IR through a reference RC-evaluator
  **with and without** elision and assert **identical observable results AND identical reclamation
  records** — backed by (b) a **structural-invariant argument** (the elision fires only where the
  emission's own balance invariant guarantees the removed `dup`/`drop` pair is net-zero). The pass's
  correctness tag is **`Empirical`** (differential trials), **not `Proven`** — a mechanized proof is a
  Phase-3 option (VR-5: no upgrade past the basis).

**Secondary resolutions (adopted defaults):**
- **Q4 (substrate/`consume`) → subsume, not a separate path.** A `substrate`/affine-typed binding is
  already statically unique; that uniqueness **feeds the same elision mechanism** rather than a
  parallel proof path (DRY / KC-3). The borrow analysis treats a known-affine binding as owned-unique.
- **Q5 (perf-measurement gate) → an RC-op-reduction ratio on a representative corpus.** The metric
  Increment 1 must clear before Increment 2 is committed is a **measured reduction in emitted
  `dup`/`drop` count** on a representative program corpus (threshold fixed when that corpus exists; the
  count is well-defined now via the RC-annotated IR). Until measured the perf claim stays `Declared`
  (DN-32 §6b); the **count itself is `Exact`** (it is read off the IR), so the gate is enforceable as
  soon as a corpus lands.
- **Q6 (FIP user surface) → invisible/inferred for R1.** No user-facing `fip`/`@unique` annotation in
  R1 (KISS / ergonomics — the §7 "never forced" stance). A surface annotation with a *static* in-place
  guarantee is a **Phase-3** decision tied to Increment 3 (the full FIP layer).
- **Q7 (target) → frame-limited for R1.** Increment 1 targets **frame-limited** reuse (Lorenzen — peak
  memory within a constant factor of the live set), sufficient for systems use; full *garbage-free*
  (Perceus) is the **Phase-3 aspiration**, not an R1 gate.

**Consequence — MEM-4 is unblocked at the design level.** The build proceeds per these resolutions:
(1) a new `crates/mycelium-mir-passes/` with the RC-annotated IR + a naive (fully-owned) RC-emission
lowering `Node → RcNode` + a reference RC-evaluator + the balance invariant; then (2) **Increment 1**
— non-escaping borrow elision + the Q3 differential harness. Each increment lands behind the Q5 gate;
the runtime `RcCell` probe remains the sound fallback throughout. The cross-hypha §7 reconciliation
(Q1 → Option A) will be carried into the eventual RFC-0027 follow-on; this note does not itself move
RFC-0027's status (append-only).

## §9 Relation to the corpus & grounding

- **Corpus:** DN-32 §2.2 (Layer-2 static uniqueness), §6b (KC-3 tension), §7 (cross-hypha
  sub-question); RFC-0027 §7.1 (acyclicity = Perceus precondition), §7.3 (affine channel transfer),
  §8 (Perceus-as-lowering-pass `Declared`), §10.1 (`UniqueOwner` is the `rc→0` event), §12 (the
  reconciliation sub-question); MEM-1/2/3 as built (`crates/mycelium-std-runtime/src/{rc,region,
  reclamation,network}.rs`); the `FLAG (MEM-4 / static RC elision)` in `rc.rs`. Value-model basis:
  LR-8 (immutability), LR-9 (acyclicity), RT7 (structured scopes); KC-3 (small auditable kernel);
  VR-5 / G2 (downgrade-don't-overclaim / never-silent).
- **External prior art (`Empirical` for existence; transfer is `Declared`):** Perceus (PLDI'21,
  doi:10.1145/3453483.3454032); Lorenzen–Leijen "Reference Counting with Frame-Limited Reuse"
  (ICFP'22, doi:10.1145/3547634) + Lorenzen 2021 thesis; de Moura–Ullrich "Counting Immutable Beans"
  (arXiv:1908.05647); Koka FP² "Fully in-Place Functional Programming" (ICFP'23); ASAP (Cambridge
  UCAM-CL-TR-908); Pony reference capabilities + ORCA; Rust `Arc`/`Box`/`Rc` `Send` discipline;
  Project Verona regions + `cown`; Tofte–Talpin region inference (TOPLAS'98).

---

## Meta — changelog

- **2026-06-25 — Created (Draft, advisory).** Research-backed direction capture for **MEM-4** — the
  deferred Layer-1 **static uniqueness analysis** leg of DN-32 — and the **cross-hypha reconciliation
  sub-question** (DN-32 §7 / RFC-0027 §12). Records: MEM-4 is an **additive, semantics-preserving
  lowering pass** that elides provably-redundant RC ops with the runtime `RcCell` probe as the sound
  fallback (sound-but-incomplete is benign — a bug costs throughput, never safety); the value model
  (LR-8 immutability + LR-9 acyclicity + content-addressing) shrinks the problem below Rust-style
  borrow checking; a **lowering-pass-not-type-checker** architecture + **additive-only** + **watch +
  measure** keep it inside the KC-3 budget; an incremental decomposition (non-escaping borrow elision
  → `rc==1` reuse annotation → full FIP guarantee); and a **recommendation of Option A**
  (sole-move-only / affine-channel boundary; `RcCell` stays `!Send`) **for R1**, Option B
  (shared-crosses-atomic-RC) deferred to R2 (`xloc`/`mesh`). All Mycelium-specific claims `Declared`;
  external prior art `Empirical`; the cross-hypha recommendation is an argument, its ergonomic cost
  the named open risk. **Enacts nothing; moves no status; changes no normative text.** Promotion past
  Draft requires the §8 deliberation + maintainer ratification (house rule #3, append-only). CHANGELOG
  / Doc-Index / issues.yaml / docs/api-index owned by the integrating parent. (Append-only; VR-5; G2.)
- **2026-06-25 — Ratified Draft → Accepted (§8.1 resolutions; maintainer).** The §8 deliberation is
  settled: **Q1 → Option A** (sole-move-only cross-hypha; `RcCell` stays `!Send`; Option B → R2),
  **Q2 → separate RC-annotated IR** (Core IR `node.rs` stays pristine — KC-3 / §4), **Q3 → differential +
  structural-invariant** soundness (tag `Empirical`, not `Proven`). Q4–Q7 adopted as recorded
  defaults (subsume `substrate` uniqueness; perf gate = measured `dup`/`drop`-reduction ratio, count
  `Exact` / perf `Declared`; FIP user-surface deferred to Phase 3; frame-limited R1 target). Status
  moves Draft → Accepted (legal forward move; the design is ratified, the code is the forward epic).
  **Still enacts no code**; does not move RFC-0027's status. Doc-Index DN-33 row synced to Accepted.
  (Append-only; VR-5; G2.)
- **2026-06-25 — Addendum §6.1 (prerequisite gap; append-only).** A read-only investigation of the
  post-Wave-4 tree confirms **MEM-4 Increment 1 is blocked-by-prerequisite, not merely deferred**:
  the Core IR (`mycelium-core/src/node.rs`) carries no ownership-mode field on binding sites, there is
  no RC-annotated IR / `mir-passes` crate, and `clone_ref`/`drop_ref` are hand-called only in
  `mycelium-std-runtime` tests — **no lowering emits RC ops, so MEM-4 has nothing to elide.** §6.1
  records the grounded findings + the prerequisite chain (resolve §8 Q2 ownership-mode representation →
  add the field to `node.rs` → build the `mir-passes` RC-emission lowering → wire into `elab.rs` →
  *then* Increment 1). The runtime substrate remains the sound, complete fallback; the RC-emission
  pipeline is a forward epic gated on the §8 Q2 maintainer decision — not built speculatively (G2/VR-5:
  flag, don't guess). No status moves; no normative text changes.
- **2026-06-25 — Correction to §6.1 (append-only; supersedes the §6.1 "prerequisite gap" finding).**
  An alignment audit confirmed §6.1's grounded findings are now FALSE: `crates/mycelium-mir-passes/`
  exists and is fully built (`rc_ir`/`emit`/`eval`/`balance`/`corpus` = separate RC-annotated IR +
  RC-emission lowering + Increments 1–2 + Q5 corpus; see the E12 build plan). The §6.1 step "add an
  ownership-mode field to `node.rs`" is moot — the §8.1 **Q2** ruling chose a *separate* RC-annotated
  IR, leaving `mycelium-core/src/node.rs` pristine. §6.1's prose is preserved verbatim as a historical
  snapshot; a `> Correction` callout after it neutralizes the standing claim. No status moves; no
  original text deleted (house rule #3, append-only; VR-5/G2).
- **2026-07-11 — Forward cross-reference added (§6; append-only; no status move).** Pointed
  Increment 2's `rc==1` reuse annotation to **DN-35 §5** (the content-address-coherence answer) and
  **DN-120** (Draft verdict note: solved-by-design, not a new mechanism). Status remains **Accepted**;
  no normative text changed. (Append-only; VR-5; G2.)
