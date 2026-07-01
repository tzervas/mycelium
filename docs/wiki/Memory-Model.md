# Memory Model

Mycelium values are **immutable** (LR-8), **acyclic** (LR-9), and **content-addressed** (identity =
structural hash). On that substrate the runtime uses a **three-layer hybrid memory architecture**
(DN-32, Accepted), with the reclamation mechanism fixed by **RFC-0027** (Accepted) and the static
optimization leg by **MEM-4 / DN-33** (Accepted).

## The three layers (DN-32)

- **L1 — affine / linear ownership (primary).** Unique data is owned and moved, not shared — the
  default, zero/near-zero-cost path.
- **L2 — optimized reference counting (explicit sharing only).** Non-atomic intra-hypha reference
  counting; `rc == 1` enables in-place reuse (Perceus / FBIP). Cross-hypha transfer is **sole-move
  only** (the affine channel protocol; `RcCell` stays `!Send`) for R1 — shared-crosses-atomic-RC is
  deferred to R2 (DN-33 §8.1 Q1, ratified Option A).
- **L3 — region-based batched scope reclamation.** A scope is a region; values dying within it are
  reclaimed in bulk at scope-exit. Parent–child reclamation is **total** (child before parent);
  siblings are **concurrent** (weak coupling, safe by acyclicity — RFC-0027 OQ-1 resolved).

## Never-silent reclamation (G2 / RFC-0027 §9)

Every reclamation event yields a structured `ReclamationRecord` (`{scope_id, sweep_epoch, trigger,
value_meta_hash, channel_id?}`) routed through a `ReclamationSink` — a reclamation that produces no
record is a transparency violation. There are **three live triggers**, all wired and firing from the
running runtime (`mycelium-std-runtime`):

- **`RcZero`** — the last reference to a shared value is dropped (`rc.rs`).
- **`ScopeExit`** — a region closes, batch-reclaiming its deferred values (`region.rs`,
  `scope_region.rs`).
- **`ChannelClose`** — a channel is torn down with values still in transit, reclaiming them
  (`network.rs`).

An end-to-end composition test exercises all three through one audit trail.

## Static RC elision (MEM-4 / DN-33)

`mycelium-mir-passes` is an **optimisation-only** crate **outside the trusted Core IR** (KC-3): a bug
there is a missed optimisation, never unsafety — the runtime reference-count probe is always the
sound fallback. It lowers the Core IR `Node` grammar to a **separate** RC-annotated IR (`RcNode` with
`Dup`/`Drop` ops), then:

- **emits** reference-counting operations (the naive fully-owned baseline), and
- **elides** them where uniqueness/borrowing is statically provable — **Increment 1** borrow-elides
  *fully-borrowable* `let` bindings (every use a reader-primitive read), replacing `Dup`s with
  non-consuming `Borrow`s and a single `DropAfter`.

Soundness follows the ratified **differential + structural-invariant** strategy (DN-33 §8.1 Q3): a
balance invariant on the emitted IR, plus a reference RC-evaluator that confirms the elided and
non-elided emissions reclaim the **same** values with no use-after-free. Semantics-preservation is
tagged **`Empirical`**; the `Dup`-count reduction is **`Exact`** (read off the IR), and the
*performance* benefit stays **`Declared`** until measured on a corpus (Q5).

## Status & references

- Landed: the three-layer runtime substrate (all triggers) + MEM-4 B0 (RC-emission) + Increment 1
  (borrow elision).
- Deferred (per DN-33): Increment 2 (`rc == 1` reuse annotation), Increment 3 (full FIP static
  guarantee, Phase 3), interprocedural borrowing, and recursion RC.
- Docs:
  [DN-32](https://github.com/tzervas/mycelium/blob/main/docs/notes/DN-32-Three-Layer-Hybrid-Memory-Architecture.md) ·
  [RFC-0027](https://github.com/tzervas/mycelium/blob/main/docs/rfcs/RFC-0027-Memory-Management-and-Reclamation.md) ·
  [DN-33](https://github.com/tzervas/mycelium/blob/main/docs/notes/DN-33-Layer1-Static-Uniqueness-Analysis.md) ·
  [build plan](https://github.com/tzervas/mycelium/blob/main/docs/planning/E12-Memory-Model-Build-Plan.md).

---

**Up:** [Home](Home.md) · [Architecture](Architecture.md) ·
[Doc Index](https://github.com/tzervas/mycelium/blob/main/docs/Doc-Index.md) ·
[repo root README](https://github.com/tzervas/mycelium/blob/main/README.md)
