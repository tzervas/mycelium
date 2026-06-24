# RFC-0027 — Memory Management and Reclamation

| Field | Value |
|---|---|
| **RFC** | 0027 |
| **Status** | **Proposed** (2026-06-24) — *proposed move from Draft; awaiting maintainer ratification (see banner below)* |
| **Feeds** | E12-1 (runtime & concurrency execution maturity) |
| **Decides** | The reclamation model for Mycelium runtime values: the reclamation **mechanism** (reference counting), ownership/lifetime semantics, reclaim-cascade scope, explicit-vs-implicit discipline, the reclamation **EXPLAIN/audit record**, and the "no silent GC pause" honesty stance (G2/VR-5). |
| **Date** | June 24, 2026 (Draft: June 23, 2026) |
| **Task** | E12-1 (M-712) |

> **⚠️ STATUS-MOVE FLAG — maintainer ratification required (house rule #3, append-only).**
> This revision **proposes** advancing RFC-0027 **Draft → Proposed**. The *reclamation mechanism*
> is now resolved by the provenance/ownership research cluster (wave-1 + wave-2 + the embeddenator
> ground-truth pass — see §7): **reference counting** (RC), justified by LR-9 acyclicity (which *is*
> Perceus's garbage-free precondition), is the decided mechanism. The move stops at **Proposed**,
> **not** `Accepted`: the highest-uncertainty design point — sweep-order vs. reclamation-order
> coupling (§5/§11, OQ-1) — is *deliberately left open* for prototyping before commitment, and no
> in-repo mechanized proof or property test yet exists (the soundness is **`Proven`-modulo the LR-9
> side-condition**, not `Proven` outright — §8). **A maintainer must ratify this Draft → Proposed
> move; it must not skip to `Accepted` (never skip a state — house rule #3).** Until ratified,
> treat the status as Draft-with-a-proposed-advance.

> **Posture (transparency/honesty rule / VR-5).** The original (2026-06-23) revision was a planning
> stub — scope, user stories, and open questions only, all `Declared`. This (2026-06-24) revision
> **advances the content** by incorporating the research-resolved design (§§7–11), tagging each
> claim at its supportable strength: the RC-soundness is **`Proven`-modulo the LR-9 side-condition**
> (cited explicitly — §8), the ~32K-LOC embeddenator confirmation is **`Empirical`**, and all
> Mycelium-specific wiring (EXPLAIN-record schema, `fuse` binding, the `trigger` field) remains
> **`Declared`** (unbuilt). The advance is **append-only**: prior prose (§§1–6) is preserved
> verbatim; the resolved design is additive (§§7–11). Nothing is upgraded past its checked basis.

---

## 1. Problem / Goal

RFC-0008 RT7 establishes that runtime lifetimes are structured — a scope does not exit until all
its children complete, are cancelled, or are detached into another owning scope, and "in safe
Mycelium a leaked task is not expressible." This gives the concurrency model a lifetime discipline,
but it does not yet define *how memory backing immutable values is reclaimed*, what the runtime
sweep-order obligation (RFC-0008 §4.3) means for reclamation timing, or how the EXPLAIN contract
extends to the reclamation decision.

The gap: `mycelium-std-runtime` (ADR-020 v0 R1 slice) relies on Rust's ownership + drop system
for reclamation of values within a `Scope<T,E>`, which is a correct but implicit mechanism. As
the runtime grows toward the full RFC-0008 vocabulary (`reclaim` surface construct, `cyst`
checkpointing, `xloc` cross-node transfer), the reclamation model must be:

1. **Explicit and EXPLAIN-able** — G2/ADR-006: reclamation is not a silent ambient GC pause; the
   decision to reclaim a runtime unit is a reified, inspectable event.
2. **Grounded in the sweep-order property** — RFC-0008 §4.3: the Kahn-determinism differential
   depends on a defined sweep order; the reclamation cascade must respect this order or violate
   RT2.
3. **Honest about the guarantee strength** — a reclamation timing claim is `Empirical` at best
   until a verified property test or proof exists; the model must state what it is, not
   over-claim.
4. **Compatible with value semantics (LR-8/LR-9)** — values are immutable and acyclic; there is
   no aliased mutation to police, so cycle-breaking GC is not needed. The reclamation model should
   exploit this advantage rather than ignoring it.

This RFC will decide the ownership/lifetime model for values crossing hypha/scope/channel
boundaries, the `reclaim` surface construct's typing and elaboration (the R1 activation gated in
ADR-020), the never-silent honesty stance for reclamation events, and how `cyst` checkpointing
interacts with the reclamation lifecycle.

---

## 2. User stories

- As a **language user**, I want to write concurrent Mycelium programs without thinking about
  memory reclamation, so that I can focus on the algorithm rather than the allocator — with the
  guarantee that values I send over channels will not silently persist in memory after the
  receiving scope completes.
- As a **compiler engineer**, I want the reclamation model to be formally specifiable in terms of
  the sweep order (RFC-0008 §4.3) and the structured-concurrency scope tree (RT7), so that I can
  write a property test that catches any implementation that violates the order.
- As a **library/phylum author** building on `std.runtime`, I want `reclaim` to be a first-class,
  EXPLAIN-able surface construct so that a supervision policy (RFC-0008 §4.7) can introspect and
  audit every reclamation event — not receive a silent drop.
- As a **downstream app developer** targeting resource-constrained environments, I want a honest
  worst-case bound on reclamation latency — tagged `Empirical` or `Declared` as appropriate —
  rather than a prose assertion that "GC never pauses."
- As an **AI co-author agent** synthesizing runtime code, I want the reclamation model to be
  normatively described so I can check a proposed implementation for sweep-order violations
  without reverse-engineering the runtime source.
- As a **maintainer**, I want the "no silent GC pause" stance codified here so that any future
  Phase-7 construct that introduces a pause must supersede this RFC rather than silently
  contradicting it (append-only, G2).

---

## 3. Scope and decision space

**In scope:**

- Ownership and lifetime semantics for values crossing hypha/channel/scope boundaries
- `reclaim` surface construct: typing, elaboration, and interaction with the supervision tree
  (RT7; ADR-020 reserved vocabulary)
- Sweep-order reclamation cascade: the RFC-0008 §4.3 property-checked ordering and how
  reclamation events respect it
- EXPLAIN record for reclamation events (what was reclaimed, from which scope, at which sweep
  epoch)
- Worst-case reclamation latency model and its honest guarantee tag (`Declared`/`Empirical`)
- Interaction with `cyst` checkpointing: checkpoint serialization vs. reclamation lifecycle
- Honesty stance: the "no silent GC pause" policy codified as an invariant with an explicit
  fallback (the only acceptable pause is a bounded, logged, never-silent reclamation event)

**Out of scope:**

- Cycle detection (values are acyclic by LR-9; no cycle-breaking GC is defined here)
- Allocator design or Rust allocator selection (an implementation detail, not a model decision)
- `xloc` cross-node transfer memory semantics (a separate RFC for the R2 distribution
  constructs; this RFC covers single-node reclamation only)
- `mesh` gossip-layer memory (R2; deferred)
- Any heap-layout or packing decisions (those are `Swap` territory, RFC-0002)

---

## 4. Definition of Done

> **Progress note (2026-06-24).** The 2026-06-24 advance (§§7–11) addresses several DoD items
> at **Proposed** strength; the per-item status below marks which are now met-as-proposed, which
> are partially met, and which remain open. None are claimed `Accepted`-complete (the status move
> is Draft → Proposed only — see the banner).

- [x] *(met-as-proposed, §§7–8, §10)* The reclamation model is defined: the **mechanism is reference
  counting** (LR-9 = the garbage-free precondition), ownership transfer at hypha/channel boundaries
  rides the affine channel protocol (D-5), the scope-exit reclamation cascade follows the RT7 scope
  tree, and the interaction with sweep order is specified as *RC-drops-derive-from-the-scope-tree*
  (with the strong-vs-weak coupling tail left open — §11 OQ-1).
- [ ] *(partial / deferred, §10.5)* The `reclaim` surface construct's typing and elaboration are
  specified. **Decision (§10.5): this RFC specifies the memory model only and defers `reclaim`
  *surface typing* to a follow-on RFC** (KC-3 scope-tightening; `reclaim` is task supervision, not
  memory — DN-03 §4). The EXPLAIN-record *fields* are specified here (§9).
- [x] *(met-as-proposed, §9)* A never-silent honesty stance is codified: every reclamation event is
  observable via the reclamation EXPLAIN record (§9); a "silent GC pause" / a silently-dropped value
  violates this RFC and is a rejection criterion for implementation PRs.
- [x] *(met-as-proposed, §8)* Guarantee tags are assigned per-op on the guarantee lattice — and held
  at their supportable strength: RC-soundness `Proven`-**modulo** LR-9 (not bare `Proven`), the
  empirical embeddenator confirmation `Empirical`, the Mycelium wiring `Declared`. No bare `Proven`
  without an in-repo mechanized proof (VR-5).
- [ ] *(open, §11 OQ-1)* A property-test specification is given for the sweep-order reclamation
  cascade. The property *shape* is sketched (§10.3), but the **strong-vs-weak coupling decision is
  deliberately left open** to prototype both before committing the property (lane-B's recommendation);
  the test ships in M-712 after that decision.
- [x] *(met-as-proposed, §10.4)* The interaction with RFC-0008 §4.4 (`cyst` checkpointing) is
  addressed: **checkpoint-and-keep** is the R1 default; checkpoint-and-free is gated on an
  `Empirical` serializer property test (RC makes the free mechanically clean — §10.4).
- [ ] *(in progress)* Status advances `Draft` → `Proposed` → `Accepted` per the append-only
  discipline. **This revision proposes Draft → Proposed (flagged for ratification); maintainer
  sign-off is still required for both Proposed and, later, `Accepted`.**

---

## 5. Open questions

> **Resolution pointer (2026-06-24).** The five Draft open questions below are preserved verbatim
> (append-only). The 2026-06-24 advance (§§7–11) **resolves three** (model choice → RC, §7/§10.5;
> `reclaim` scope → task-only/defer surface, §10.5; checkpoint → checkpoint-and-keep at R1, §10.4),
> **partially resolves one** (sweep-order coupling — RC makes reclamation *derive from* the scope
> tree, but strong-vs-weak across siblings stays open, §11 OQ-1), and **re-frames one** (pause budget
> → "no silent GC pause" is an honesty stance + a `Declared` per-epoch bound, latency-SLO derivation
> open, §11 OQ-3). The carried-forward + newly-surfaced open questions are consolidated in **§11**.

- **Model choice:** Should reclamation be purely Rust-drop-order (implicit, no surface) with
  `reclaim` as a supervision primitive only, or should Mycelium expose explicit "reclaim regions"
  analogous to arena allocators? The former is simpler (KC-3); the latter is more EXPLAIN-able.
- **Sweep-order coupling:** RFC-0008 §4.3 defines sweep order for Kahn determinism. Is the
  reclamation cascade *required* to follow the same order (strong coupling), or merely *permitted*
  to? The strong coupling is property-testable; the weak coupling is more flexible.
- **`reclaim` construct scope:** ADR-020 lists `reclaim` as a supervision primitive for runtime
  units (tasks), not a memory primitive. Should this RFC also define memory-level reclamation, or
  only the task-reclamation surface?
- **Pause budget:** Is "no silent GC pause" a hard real-time bound (must specify a worst-case
  budget) or a best-effort stance (the guarantee is `Declared`/honesty, not a latency SLO)?
- **Checkpoint interaction:** A `cyst` checkpoint writes live values to content-addressed storage.
  Does the checkpoint operation reclaim the original allocation (checkpoint-and-free) or copy
  (checkpoint-and-keep)? The former is more memory-efficient; the latter is safer.

---

## 6. Grounding / honesty

Claims in this stub are `Declared` (stated intent) or open questions. No guarantee is `Proven`
without a mechanized proof; none is `Empirical` without a property test in-repo. The "no silent
GC pause" stance is an honesty policy (G2), not a performance claim — it means the runtime must
surface every reclamation event, not that reclamation is latency-free.

Grounding basis: RFC-0008 RT7 (structured lifetimes + scope tree), RFC-0008 §4.3 (sweep order +
Kahn determinism), RFC-0008 §4.4 (cyst checkpointing + dormable fragment), ADR-020 (reserved
vocabulary list including `reclaim`), LR-8/LR-9 (immutable acyclic values — no aliased mutation
to police), G2 (no black boxes / never silent), VR-5 (honesty rule / no upgrade without checked
basis), KC-3 (small auditable kernel — reclamation model must not grow the kernel node budget).

---

## 7. Resolved design — the reclamation mechanism is reference counting (2026-06-24)

> **Provenance of this section.** Everything in §§7–11 is *incorporated from* the just-landed
> provenance/ownership research cluster — non-normative research artifacts, cited per-claim:
> `SYNTHESIS-provenance-ownership-cluster.md` (wave-1, lanes A–E),
> `SYNTHESIS-wave2-addendum.md` (wave-2: academic + empirical integration),
> `lane-B-reclamation-provenance.md` (the reclamation-provenance lane),
> `lane-F-efficient-immutable-value-mgmt.md` (Perceus/FBIP/δ-CRDT lane),
> `lane-A-ownership-map.md` (the Rust→Mycelium ownership map), and
> `embeddenator-groundtruth.md` (the maintainer's ~32K-LOC implementation, ground-truth-corrected).
> The research is the *basis*; the RFC is the *decision-in-proposal*. Per house rule #3 the status
> stops at **Proposed** until ratified.

### 7.1 The decision

**Mycelium's runtime reclamation mechanism is precise reference counting (RC), not tracing GC.**
This is the wave-2 mechanism-resolution (`SYNTHESIS-wave2-addendum.md` W2-D1 / W2-A1; `lane-F` F-1).

The choice is *forced by the value model rather than imposed on it*. The usual Achilles' heel of
reference counting — reclaiming cycles, which requires either a cycle detector (Pony ORCA's most
complex component, `lane-B` §3.4) or a tracing fallback — **cannot arise** for Mycelium values:

- **LR-9 makes values acyclic** (RFC-0006; RFC-0008 §3; restated §3 "Out of scope: cycle detection").
- **LR-8 makes values immutable**, so there is no write barrier and no aliased-mutation hazard
  (`lane-A` §2.1; `lane-B` §2.6).

Acyclicity is *exactly* the precondition under which **Perceus** (`Perceus: Garbage-Free Reference
Counting with Reuse`, PLDI 2021) proves a cycle-free program is **garbage-free** — only live
references are retained — with no cycle collector (`lane-F` §3.2). The same structural argument
underpins Lean 4's `Counting Immutable Beans` (arXiv 2019), whose RC scheme is built for an
immutable-acyclic value model identical in shape to Mycelium's. So the corpus's own LR-9 *is* the
side-condition the academic soundness result needs — the two meet exactly (see §8 for the guarantee
tag this licenses, and its honest limit).

### 7.2 Why RC fits the never-silent stance better than tracing GC

RC is the mechanism most aligned with the "no silent GC pause" honesty stance (§1, G2):

- There is **no stop-the-world phase**. Each `rc_dec` is a bounded O(1) event; a chain-drop is
  O(*n*) in the nodes freed but **deterministic and incremental** — triggered by scope exit, not by
  an ambient collector firing at an unpredictable time (`lane-F` §4.2.1 F-1, §5.1).
- Each increment/decrement is a **reifiable event**, so the reclamation EXPLAIN record (§9) has a
  *structural* anchor (the refcount transition) rather than an editorial label (`lane-F` §4.2.1 F-2).

The honest caveat (`lane-F` §5.1, adversarial check F-D): "bounded" means *deterministic, observable,
and EXPLAIN-able*, **not** constant-time. Dropping a deep value tree is O(*n*) in its node count — a
bounded-but-large pause, not a sub-millisecond one. Whether the fuel model can bound that latency is
open (§11 OQ-3).

### 7.3 Scope of the RC decision — single-owner intra-hypha; cross-hypha rides the channel protocol

RC is decided **for single-owner, intra-hypha value reclamation** (`SYNTHESIS-wave2-addendum.md`
W2-CL-3). It is **not** "RC for everything":

- **Cross-hypha transfer rides the affine channel protocol, not a distributed refcount.** The
  channel `Sender`/`Receiver` pair is affine (non-`Clone`), giving exactly-one-owner cross-hypha
  transfer at R1 — no distributed reference counting is needed (wave-1 D-5; `lane-B` R-4). This is
  the same role Pony ORCA's deferred weighted ref-count plays, but Mycelium gets it from the channel
  close protocol for free, and **without ORCA's cycle detector** (eliminated by LR-9).
- **`reclaim` is not memory.** Per DN-03 §4 / RFC-0008 §4.5, the `reclaim` surface construct is
  task/runtime-unit supervision — **never** a memory primitive (wave-1 D-4; `lane-A`/`lane-B`
  converge here). Memory reclamation is the RC mechanism, exposed to supervision *through* the
  EXPLAIN record (§9), not through `reclaim`.

---

## 8. Guarantee tagging of the RC decision (VR-5 — the honest strength)

Per the transparency rule, each claim is tagged at its supportable strength and **not upgraded**:

| Claim | Tag | Basis / honest limit |
|---|---|---|
| RC over Mycelium values is **garbage-free / sound without a cycle collector** | **`Proven`-modulo the LR-9 side-condition** | Perceus (PLDI'21) + Lean-4 (arXiv'19) prove garbage-freedom *given* cycle-freedom; LR-9 supplies that side-condition. **Honest limit:** the theorem is external and its side-condition (LR-9) is a corpus *invariant*, not yet an in-repo *mechanized check* — so this is **not** bare `Proven`. It does not become unqualified `Proven` until LR-9 is mechanically enforced and a Mycelium-side adaptation proof exists in-repo. |
| RC suffices at scale **in running code** — acyclic content-addressed values need no cycle detector / no GC | **`Empirical`** | The maintainer's embeddenator (~32K-LOC) manages its entire acyclic holographic value graph with **no cycle detection and no GC** (`embeddenator-groundtruth.md`; `SYNTHESIS-wave2-addendum.md` CF-1, W2-D3). Independent convergence of theory (lane-F) and implementation. |
| The Perceus *algorithm* is **directly** portable to Mycelium's Core IR | **`Declared` (needs adaptation)** | Perceus targets an RC-annotated low-level IR; Mycelium's Core IR is a typed term language, so RC emission must be added as a lowering pass (`lane-F` §7 F-A; W2-CL-3). The *approach* transfers; the *exact algorithm* needs adaptation. |
| The reclamation EXPLAIN-record schema, `trigger` field, and the whole runtime wiring | **`Declared`** | Unbuilt. Derived from G2 + the sweep-order model + the RFC-0005 EXPLAIN contract; no property test, no proof in-repo (`lane-B` R-1). |

**Crucially:** the embeddenator confirmation is *structural* (substrate-independent: acyclicity ⇒
no cycle detector; CoW-shaped re-encode; lossless-reconstruction-via-correction). The
**Mycelium-specific bindings stay `Declared`** — and the maintainer's *divergence* (reaching for
**optimistic concurrency control** at the multi-writer filesystem layer, with the refcount/dedup/GC
machinery present but **unwired and inert** in the versioned path) is itself evidence the transfer is
not 1:1 (`embeddenator-groundtruth.md` C1–C2, CH-1; `SYNTHESIS-wave2-addendum.md` W2-CL-5). The
single-owner FBIP/refcount-reuse path (§10.2) is therefore a Mycelium *design* move with **no working
precedent** in embeddenator — its only precedent is the external Perceus/Lean/Swift-CoW line.

---

## 9. The reclamation EXPLAIN / audit record (never-silent, G2)

Every reclamation event MUST be observable as a structured EXPLAIN record (`lane-B` R-1; refined to
RC by `lane-F` F-2 / `SYNTHESIS-wave2-addendum.md` W2-A1). The minimum field set:

| Field | Type | Rationale |
|---|---|---|
| `scope_id` | stable scope identifier | RT7 — which scope's exit triggered reclamation. |
| `sweep_epoch` | monotonic counter from `SweepOrder` (RFC-0008 §4.3) | epoch-based safety anchor; ties reclamation to the scheduling model. |
| `trigger` | enum `RcZero \| ScopeExit \| ChannelClose` | G2 — never silent; the record knows *why*. The RC framing makes this **structural, not editorial**: `RcZero` = the refcount hit 0 (deferred to scope exit), `ScopeExit` = a scope-tree node closed, `ChannelClose` = a channel disconnected and released ownership. |
| `value_meta_hash` | content hash of the value's `Meta` | content identity — ties the event to the value's provenance/guarantee history (the `Provenance` DAG, RFC-0001 §4.6). |
| `channel_id` | optional; present for `ChannelClose` | which channel boundary the value crossed. |

Discipline (`lane-B` R-1, R-6, R-8):

- The record is an **emit-once** observability artifact at reclamation time, routed to the
  supervision policy's observability sink (RFC-0013 §8) — **not** a per-value runtime invariant.
- It **extends the one-mechanism RFC-0005 EXPLAIN contract** (`inputs considered` → which
  scope/channel triggered; `chosen option` → drop / checkpoint-and-free / return-to-sender;
  `sweep_epoch` → the deterministic audit anchor), reusing the same sink (KC-3/DRY, no second
  mechanism).
- **Failing to emit the record is a G2 violation — exactly as silently dropping a value is.**

**Open relationship (carried, not decided here — TN-1 / `SYNTHESIS` §3.2):** whether this record
*is* a `Provenance::Derived` node (RFC-0001 §4.6) or merely *cites* one is left to the transpilation
DN + this RFC's follow-on. Wave-1's CHG-1 notes that hosting it as a `Derived` node could upgrade its
strength from `Declared` (asserted log) toward content-hash-pinned identity — a strict improvement to
consider, not yet adopted.

---

## 10. Resolved design — the reclamation–copy/mut–`fuse` relationship (2026-06-24)

### 10.1 RC unifies reclamation and copy/mut via one `rc`-probe

Reclamation and copy/mutation are **two faces of one sharing-state question**, unified by the RC
count (`lane-F` §4.1; Perceus reuse / Swift CoW / Lean-4 borrow):

```
rc(v) → 0  ⟹  v is unreachable        → free its allocation             [RECLAMATION]
rc(v) == 1 ⟹  v has a single owner     → mutate in place (copy elided)   [COPY/MUT — FBIP reuse]
rc(v) >  1 ⟹  v is shared              → structural-share on update      [DEFAULT TODAY]
```

The `rc == 1` reuse check is the **Perceus/FBIP** ("Functional But In-Place") lever: a value about
to be dropped whose refcount is 1 may have its allocation **reused** for the next value of compatible
shape — in-place mutation written in a purely functional style, surface-semantically a new value.
The affine `Substrate`/`consume` (DN-02/03; Glossary) is the corpus's existing **uniqueness lever**;
generalizing uniqueness-tracking from external resources to *ordinary* values is the path that brings
Mycelium to the Clean/Koka FBIP design point (`lane-F` F-5). Today `substrate` is scoped to external
resources only — widening it is named as a future direction, **not** proposed here (KC-3/YAGNI).

### 10.2 The copy/mut path — `rc==1` reuse, never-silent, single-owner only

A future O(1) `rc==1` reuse check may be added to `std.collections` mutators
(`SYNTHESIS-wave2-addendum.md` W2-A3; `lane-F` F-4), with two hard constraints:

- **Never-silent (G2):** the reuse-vs-copy choice is recorded in the `Provenance` DAG as distinct
  ops — a `Derived{op: "push_reuse", …}` is distinguishable from `Derived{op: "push_copy", …}`. The
  guarantee tag is `Exact` either way (the operation carries no approximation); only the *path*
  differs. Whether the choice is *surface-visible* or *EXPLAIN-record-only* is open (§11 OQ-4).
- **Single-owner only.** The `rc==1` probe is an *intra-hypha single-owner* optimization. The
  *concurrent multi-writer* path is **optimistic concurrency control (OCC)**, not a uniqueness probe
  — this is the empirical lesson from embeddenator, which reached for an `AtomicU64` version counter +
  `VersionMismatch`, not `isKnownUniquelyReferenced` (`SYNTHESIS-wave2-addendum.md` CH-1;
  `embeddenator-groundtruth.md` C1–C2). FBIP-reuse and OCC are co-residents at different layers, not
  competitors; presenting refcount-reuse as *the* copy/mut answer would over-claim — it is the
  single-owner answer.

This whole copy/mut subsection is `Declared` and **sequenced after** the RC mechanism settles; at v0
interpreter scale, structural sharing remains the correct default and the reuse check is an
optimization, not a prerequisite (`lane-F` F-6).

### 10.3 Sweep-order derives from the scope tree (the partial resolution)

Under RC, reclamation of a value follows deterministically from its last owner's drop, which is
itself ordered by the RT7 scope tree. So **sweep-order is not an *additional* constraint on
reclamation — it is *derived from* the scope-tree order** (`lane-F` F-3;
`SYNTHESIS-wave2-addendum.md` W2-A2). Deferred RC drops accumulate per scope and flush at scope-exit
in child→parent order. The minimal-safe model (`lane-B` R-2):

- **Within a scope:** Rust drop order (reverse construction) — `Exact` by Rust's specification.
- **Parent–child:** children fully reclaim before the parent exits — a total order along the
  child→root path (the RT7 LIFO for in-scope values; cross-scope transfer rides the channel protocol,
  not the LIFO — wave-1 CL-7).
- **Across siblings:** the open question — see §11 OQ-1. Sibling scopes are concurrent by design;
  the *property-test shape* would assert that for two same-level scopes, neither's reclamation record
  appears in the other's sweep epoch (sibling epochs non-overlapping/unordered).

### 10.4 `cyst` checkpoint — checkpoint-and-keep at R1 (RC makes free clean)

**Decision (`lane-B` R-5; `lane-F` OQ-F2):** the R1 default is **checkpoint-and-keep** — the
original allocation survives alongside the content-addressed `cyst` artifact. Checkpoint-and-*free*
is sound *in principle* (the value's identity is its content hash, not its address — wave-1 TN-4 /
CL-5), and RC makes it mechanically clean (the serializer holds a temporary `rc+1` during
serialization, `rc−1` on completion; if last reference, the allocation frees — no separate
free-after-checkpoint decision). But checkpoint-and-free is **gated on an `Empirical` serializer
property test** that does not yet exist in-repo; until then the safe default holds. EXPLAIN record:
`trigger` distinguishes the checkpoint event; `value_meta_hash` + the `cyst` artifact hash are
recorded.

### 10.5 `reclaim` surface typing is deferred (KC-3 scope-tightening)

This RFC specifies the **memory model only**. The `reclaim` *surface construct* (L1 typing +
elaboration) is **left to a follow-on RFC** (`lane-B` OQ-2; ADR-020 §consequences defers it). This
keeps RFC-0027 tight and lets the memory model ratify before the surface shape is locked. `reclaim`
remains reserved + task-supervision-only (DN-03 §4); it is **not** redefined as memory here.

### 10.6 `fuse` is structurally unified with the value model but algebraically separate

`fuse` (the RT6 CRDT-merge / semilattice join — RFC-0008) has a **precise, deliberately-bounded**
relationship to RC (`SYNTHESIS-wave2-addendum.md` W2-D2; `lane-F` §4.1, §4.3). State it without
over-claiming:

- **Structurally unified (real, `Empirical`/`Declared`):** `fuse` shares the *same* `Provenance` DAG
  (RFC-0001 §4.6) and the *same* guarantee lattice. From the DAG it gets **δ-CRDT anti-entropy
  efficiency for free**: two replicas exchange their DAG **root hashes**, walk to the **least common
  ancestor (LCA)**, and ship **only the divergent sub-DAG** — the Merkle-CRDT anti-entropy protocol,
  O(change) not O(state) (`lane-F` F-7, F-8). A `fuse` result is
  `Derived{op:"fuse_join", inputs:[left_root, right_root]}`, and guarantees compose by `meet`
  (`meet(Proven, Empirical) = Empirical` — honesty degrades, never spuriously upgrades; `lane-F`
  F-9, `meet`-laws `Proven` over the finite lattice). δ-completeness for *non-monotone* merges
  (removals) needs tombstones/version-vectors — `Declared`, grow-only is `Empirical` (`lane-F` F-C).
- **Algebraically separate (real, doubly-sourced):** `fuse` correctness is the **semilattice-join
  law** (commutative/associative/idempotent — convergence is the CRDT strong-eventual-consistency
  theorem, mechanized in Isabelle/HOL per RFC-0008 RT6). This is **independent of refcounting** — the
  refcount tells you nothing about the merge function. Two independent sources reach this verdict:
  lane-F's adversarial check ("the extension to `fuse` is architectural, not mechanical") and the
  empirical record (embeddenator's VSA `bundle` is commutative + idempotent-at-saturation **yet the
  maintainer explicitly claims no CRDT semantics — no tombstones, no deletion-merge, no convergence
  proof**) — `SYNTHESIS-wave2-addendum.md` CH-2, W2-CL-4.

**So `fuse` is unified with the value model in *sharing/structure* (the Provenance DAG + the lattice),
and separate from it in *convergence law* (the semilattice algebra).** The convergence machinery
(distributed reclamation, tombstones, weighted ref-counts) is an **R2** concern (`xloc`/`mesh`),
explicitly out of scope here (§3) and flagged in §11 OQ-2.

---

## 11. Open questions — consolidated (carried-forward + newly surfaced, 2026-06-24)

These remain **open** — the advance does not decide them. They are the named tradeoffs a future
`Accepted` revision must close.

- **OQ-1 — Sweep-order vs. reclamation-order coupling: partial vs. total across siblings
  (`Declared`, the highest-uncertainty decision).** RC makes reclamation *derive from* the scope
  tree (§10.3), which collapses parent–child ordering — but the **sibling** question stands:
  - **Weak / partial coupling** (lane-B's `Declared` default): sibling scopes reclaim *concurrently*
    → better throughput; the reclamation-order property test is *separate from* (and weaker than) the
    Kahn-determinism differential.
  - **Strong / total coupling:** reclamation order is *identical* to `SweepOrder` → **one** property
    test covers both scheduling and reclamation (maximal auditability); cost: sibling cleanup is
    serialized, a throughput cost with no safety benefit (LR-9 rules out cross-sibling aliases).
  **lane-B explicitly recommends prototyping BOTH and measuring the property-test surface before
  committing** (`lane-B` OQ-1, R-2; wave-1 O-1; `SYNTHESIS-wave2-addendum.md` W2-A2). This is a
  genuine unresolved tradeoff, not a finding — it is the reason the status stops at **Proposed**.
- **OQ-2 — R2 distributed reclamation provenance (`Declared`, deferred).** When `xloc`/`mesh` land
  (R2), a value may cross node boundaries and the reclaiming scope may not be the creating scope —
  CRDT tombstone GC + weighted reference counting territory (`lane-B` §3.5–3.6, OQ-4; wave-1 O-8;
  W2-O3: embeddenator's correction store has *no* merge semantics, a concrete instance). Explicitly
  out of scope for R1; flagged so the R2 RFC has a clean handoff.
- **OQ-3 — Worst-case RC-cascade drop latency / the pause budget (`Declared`).** Re-frames the Draft
  "pause budget" question. Each `rc_dec` is O(1) but a deep-tree drop is O(*n*); "no silent GC pause"
  is satisfied as an *honesty stance* (every step observable) but is **not** a sub-millisecond SLO.
  Can the fuel model (RFC-0014 §4.8) bound the cascade, lifting latency `Declared` → `Empirical` /
  `Proven`? The sweep-epoch model could spread the cost across epochs at the price of deferred
  reclamation (`lane-B` OQ-3; `lane-F` OQ-F5, F-D; wave-1 O-7; W2-O2). A dedicated research note is
  warranted.
- **OQ-4 — Is the `rc==1` reuse-vs-copy choice surface-visible or EXPLAIN-record-only? (`Declared`).**
  G2 requires the `Provenance` record always capture it (§10.2); whether the *caller* can observe
  which path was taken is open — Perceus makes reuse fully transparent, FP2 `fip` makes it static and
  visible (`lane-F` OQ-F4; W2-O4).
- **OQ-5 — `substrate`/`graft` reclamation (`Declared`, flag-and-defer).** What is the protocol when
  an affine `substrate` handle is *dropped* rather than *consumed* — runtime error, silent no-op, or
  explicit EXPLAIN event? Out of scope here (depends on the `graft` implementation RFC), but flagged
  so a future `graft` RFC cannot silently contradict this model (`lane-A` OQ-A1/A3; `lane-B` OQ-5;
  wave-1 O-6).
- **OQ-6 — Reclamation record: a `Provenance::Derived` node or a citation of one? (`Declared`,
  TN-1).** Settling this lets reclamation / decision-ledger / spore provenance share one schema
  (DRY/KC-3) and could upgrade the record's strength toward content-hash-pinned identity (§9; wave-1
  TN-1, CHG-1). Routed to the transpilation DN + this RFC's follow-on.

---

## Meta — changelog

- **2026-06-23 — Draft created.** Planning stub for the runtime memory/reclamation model. Scope,
  user stories, open questions established. Status: Draft. Task: E12-1 (M-712). No normative
  decision made. (Append-only; VR-5.)
- **2026-06-24 — Content advanced; status move proposed (Draft → Proposed, flagged for
  ratification).** Incorporated the just-landed provenance/ownership research cluster
  (`SYNTHESIS-provenance-ownership-cluster.md`, `SYNTHESIS-wave2-addendum.md`,
  `lane-B-reclamation-provenance.md`, `lane-F-efficient-immutable-value-mgmt.md`,
  `lane-A-ownership-map.md`, `embeddenator-groundtruth.md`) as the basis for the resolved design.
  **Added §§7–11** (append-only; §§1–6 preserved verbatim): (§7) the reclamation **mechanism is
  reference counting** — LR-9 acyclicity *is* Perceus's garbage-free precondition, so no cycle
  detector is needed; scoped to single-owner intra-hypha reclamation, with cross-hypha transfer
  riding the affine channel protocol and `reclaim` staying task-supervision-only. (§8) Guarantee
  tags held at strength: RC-soundness **`Proven`-modulo the LR-9 side-condition** (not bare
  `Proven` — external theorem, corpus-invariant side-condition, no in-repo mechanized check yet),
  the ~32K-LOC embeddenator confirmation **`Empirical`**, all Mycelium wiring **`Declared`**. (§9)
  The reclamation **EXPLAIN/audit record** minimum field set: `scope_id`, `sweep_epoch`,
  `trigger ∈ {RcZero, ScopeExit, ChannelClose}`, `value_meta_hash`, optional `channel_id`. (§10)
  RC **unifies reclamation + copy/mut** via the `rc==1` FBIP reuse check (single-owner only; OCC is
  the concurrent path); sweep-order **derives from** the RT7 scope tree; `cyst` = **checkpoint-and-
  keep** at R1; `reclaim` surface typing **deferred** to a follow-on RFC; **`fuse` is structurally
  unified** (shares the Provenance DAG → free δ-CRDT anti-entropy via DAG-LCA; `meet` on guarantees)
  **but algebraically separate** (convergence is the semilattice-join law, independent of
  refcounting). (§11) Consolidated open questions — **OQ-1 sweep-order sibling coupling (partial vs.
  total) is left OPEN, with lane-B's recommendation to prototype both before committing** — the
  reason the status stops at **Proposed**, not `Accepted`. **⚠️ Status move requires maintainer
  ratification; must not skip to `Accepted` (house rule #3).** Touches no other doc; CHANGELOG.md /
  issues.yaml / docs/api-index are owned by the integrating parent. (Append-only; VR-5; G2.)
