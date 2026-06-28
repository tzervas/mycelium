# Kickoff `r4v` — Runtime Vocabulary Surface & Execution (R4 / M-667 + M-710)

> Stowed kickoff, UID **`r4v`**. Read `CLAUDE.md`, `.claude/kickoffs/README.md`, and
> **`.claude/kickoffs/_doc-maintenance.md`** (the anti-drift contract) first.

## Metadata
| Field | Value |
|---|---|
| **UID** | r4v |
| **Head/working branch** | `claude/head/r4v-runtime-vocab` (off `dev`) |
| **Status** | ready (first of the post-grammar semantic waves: **R4 → R3 → DN-53 → DN-54**) |
| **Swarm mode** | serial-on-L1 (do L1 surgery **inline**, never fan to leaves — rsm lesson) |
| **Depends on** | RFC-0008 (Accepted), RFC-0027 (Accepted), the landed runtime machinery (M-709/711/713 done, M-712 mechanism) |

## Scope
Activate the RFC-0008 R1 runtime vocabulary **`fuse` / `reclaim` / `tier`** as real L1 constructs and
**elaborate them to the already-landed runtime machinery** (`crates/mycelium-std-runtime`). The
keywords are already reserved (`token.rs`); today they're a teaching-reject. This wave adds: the L1
**surface** (parser), **type-checking** (`checkty.rs`), and **elaboration** (`elab.rs`) to runtime
dispatch — closing the `Residual` for each (M-710). `reclaim`'s model is settled by RFC-0027 (Accepted);
the reclamation mechanism + EXPLAIN record are landed in `mycelium-std-runtime`.

**Issues:** M-667 (L1 surface for fuse/reclaim/tier), M-710 (vocab execution end-to-end). Unblocks
E12-1/T3.

## Grounding (doc_refs — read before coding)
- `corpus:RFC-0008` — RT6 (semilattice `fuse`), RT7 (`reclaim` supervision), tier mode-switch; §4.5
  status rule, §4.6 R1 staging, §4.7 composition.
- `corpus:RFC-0027` — reclamation model (Accepted) — the `reclaim` semantics + EXPLAIN schema.
- `corpus:DN-25` — road to full-language 1.0.0 (T3 gate).
- `src:crates/mycelium-l1/src/parse.rs` (the reserved-keyword teaching arms to replace) ·
  `src:crates/mycelium-l1/src/checkty.rs` · `src:crates/mycelium-l1/src/elab.rs` ·
  `src:crates/mycelium-std-runtime/src/lib.rs` (the dispatch seam).

## Approach (serial-on-L1, inline)
parse.rs (fuse/reclaim/tier productions, replacing the reserved teaching-rejects) → ast.rs (the new
item/expr forms) → checkty.rs (typing: `fuse` op as RT6 semilattice join; `reclaim` policy vs RFC-0027;
`tier` mode-enum) → elab.rs (dispatch to `mycelium-std-runtime`; no `Residual` for R1) → three-way
differential (L1-eval ≡ L0-interp ≡ AOT) per construct. ~400 LoC. **Serializes on `checkty.rs`/`elab.rs`
with the other semantic waves — land this fully before R3.**

## Definition of Done
- [ ] `fuse`/`reclaim`/`tier` parse, type-check, and elaborate to runtime dispatch — **no `Residual`**
  for any R1 construct (G2); each has a three-way differential test (`Empirical`).
- [ ] `reclaim` elaboration produces the RFC-0027 EXPLAIN record; reclamation respects sweep order;
  no silent drop/pause (G2).
- [ ] `just check` green across modified crates; guarantee tags honest (VR-5 — no `Proven` without a
  mechanized proof).
- [ ] **Doc maintenance (anti-drift; `_doc-maintenance.md`):** `issues.yaml` M-667/M-710 → done; **RFC-0008
  R1 vocab section** updated to "executes"; **RFC-0027** → consider `Enacted` (reclaim elaboration
  landed); `.claude/memory/language-execution.md` + lexicon note the now-**active** runtime keywords
  (DN-03 status: reserved → active); `mycelium.ebnf` gains the constructs (+ `just grammar-gen` if any
  keyword surfaces); `CHANGELOG.md` entry; ADR-022 §5 T3 "gate-met" **FLAGged to `rel10`**.

## Landing
`/wave-land` from the head → `main` after green `just check` + `/pr-review` self-review + curated squash;
then backprop `main` down (mitigation #6). Sequence next: **`hof`** (R3 closures).
