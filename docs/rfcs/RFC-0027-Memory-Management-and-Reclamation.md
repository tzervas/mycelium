# RFC-0027 — Memory Management and Reclamation

| Field | Value |
|---|---|
| **RFC** | 0027 |
| **Status** | **Draft** (2026-06-23) |
| **Feeds** | E12-1 (runtime & concurrency execution maturity) |
| **Decides** | The reclamation model for Mycelium runtime values: ownership/lifetime semantics, reclaim-cascade scope, explicit-vs-implicit discipline, and the "no silent GC pause" honesty stance (G2/VR-5). |
| **Date** | June 23, 2026 |
| **Task** | E12-1 (M-712) |

> **Posture (honesty rule / VR-5).** This is a planning stub — scope, user stories, and open
> questions only. Nothing is decided normatively here. All guarantee claims remain `Declared`
> until a mechanized or empirical basis is recorded. Status is **Draft** until a binding
> decision is reached and the maintainer signs off.

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

- [ ] The reclamation model is defined normatively: ownership transfer at hypha/channel
  boundaries, scope-exit reclamation cascade, interaction with sweep order.
- [ ] The `reclaim` surface construct's typing and elaboration are specified (types, effects
  annotation, EXPLAIN record fields).
- [ ] A never-silent honesty stance is codified: every reclamation event is logged/observable;
  a "silent GC pause" violates this RFC and is a rejection criterion for implementation PRs.
- [ ] Guarantee tags are assigned per-op on the guarantee lattice (`Exact`/`Empirical`/
  `Declared`) — none `Proven` without a mechanized proof in-repo (VR-5).
- [ ] A property-test specification is given for the sweep-order reclamation cascade (the
  property; the test itself ships in M-712).
- [ ] The interaction with RFC-0008 §4.4 (`cyst` checkpointing) is addressed: what is
  serialized vs. what is reclaimed at checkpoint time.
- [ ] Status advances from `Draft` → `Proposed` → `Accepted` per the append-only discipline;
  maintainer sign-off required for `Accepted`.

---

## 5. Open questions

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

## Meta — changelog

- **2026-06-23 — Draft created.** Planning stub for the runtime memory/reclamation model. Scope,
  user stories, open questions established. Status: Draft. Task: E12-1 (M-712). No normative
  decision made. (Append-only; VR-5.)
