# Design Note DN-13 — RP-6: Surface Grammar for Mutual Recursion (nodule-wide, no new syntax)

| Field | Value |
|---|---|
| **Note** | DN-13 |
| **Status** | **Resolved** (2026-06-19 — surface-grammar choice recorded). Resolves the **RP-6** spike (the surface grammar for mutually-recursive function groups; RFC-0007 §8 R7-Q3, surface half). **Verdict: nodule-wide mutual visibility, no new syntax** (RP-6 candidate 2). Append-only. |
| **Decides** | The v0 surface grammar for a group of ≥2 mutually-recursive top-level functions: **every top-level `fn` in a `nodule` is mutually visible; the elaborator auto-groups each call-graph SCC of ≥2 into a `FixGroup`. No new grammar production.** |
| **Feeds** | `docs/notes/research-prompts.md` RP-6 (→ Resolved); `docs/rfcs/RFC-0007-L1-Kernel-Calculus.md` §8 R7-Q3; `docs/notes/DN-10-Remaining-L1-Gaps.md` §2.6; `docs/spec/grammar/mycelium.ebnf` (scoping note); RFC-0001 r5 `FixGroup`; DN-09 §3.1 (surface commitments are append-only). |
| **Date** | June 19, 2026 |
| **Task** | M-391 (R7-Q3 surface elaboration). |

> **Posture (honesty rule / VR-5; DN-09 §3.1).** A surface-grammar commitment, recorded append-only. It
> adds **no** new grammar production — it commits a *scoping* rule (top-level mutual visibility) — so it
> is the *least* irreversible of the RP-6 candidates: an explicit grouping form (candidate 1/3) can be
> added later as an additive refinement if a diagnostic need arises, without reversing this choice.

---

## 1. The question (RP-6)

RP-6 (`research-prompts.md` §RP-6; DN-10 §2.6) asked for the v0 surface grammar for a group of ≥2
mutually-recursive top-level functions. The elaboration *mechanics* were already settled — the
`FixGroup` node (RFC-0001 r5), the Tarjan SCC path, and the §4.5 mutual-structural-descent totality
classifier, all enacted by M-343. The residual was purely *how a mutual group is written*. Three
candidates:

1. **Explicit grouping** — ML-style `let rec f = … and g = …`: the programmer marks the group.
2. **Nodule-boundary fixpoint** — Unison/ML-module semantics: every top-level function in a `nodule`
   is mutually visible; the elaborator runs Tarjan over the whole nodule call graph. **No new syntax.**
3. **Explicit `mutually_recursive { … }` block** — Idris-style explicit grouping form.

## 2. The decision: candidate 2 — nodule-wide mutual visibility, no new syntax

Every top-level `fn` in a `nodule` is mutually visible: a function may reference any sibling regardless
of declaration order. The elaborator builds the call graph over the nodule and auto-groups each
strongly-connected component of ≥2 functions into a `FixGroup` (a self-looping singleton → `Fix`; a
non-recursive function inlines). No mutual-recursion keyword is introduced.

**Rationale.**
- **Least surface commitment (KISS · YAGNI · DN-09 §3.1 append-only).** It adds *no* grammar
  production — the flat `program ::= nodule_header item*` is unchanged (`mycelium.ebnf` gains only a
  scoping note). Of the three candidates it is the most conservative append-only choice: explicit
  grouping (1/3) stays available as a *future additive refinement* on top of nodule-wide visibility if
  a diagnostic need arises; committing 1/3 now would instead lock in grammar that must then be kept.
- **Consistency / least surprise.** It is the natural generalization of the existing self-recursion
  handling (a top-level `fn` already refers to itself), and it matches the module semantics of Rust
  (the implementation language), Unison, and ML.
- **No black box (G2 / SC-3).** Although the grouping is *inferred* (no surface marker), it is not
  hidden: it is **materialized as a `FixGroup` L0 node** — concrete, content-addressed, listing its
  members — inspectable by walking the elaborated term. The inference is reified in the IR.

**Trade-off (honest).** Mutual recursion carries no explicit marker, so an *accidental* cycle is
accepted as a mutual group rather than flagged. This is mitigated by the reified `FixGroup` (the
grouping is inspectable) and may be revisited with an optional explicit form (candidate 1/3) as an
additive refinement; it is **not** adopted for v0 (YAGNI).

## 3. Confirmation (M-391; identity-first, ADR-003)

The choice is confirmed against the existing path rather than by new front-end machinery: nodule-wide
mutual visibility (`crates/mycelium-l1/src/checkty.rs` — Pass 2 collects every top-level `fn` before
Pass 3 type-checks any body) and the Tarjan→`FixGroup` lowering (`crates/mycelium-l1/src/elab.rs`)
already exist, so a surface-written mutual group already elaborates to the same `FixGroup` the SCC
decomposition dictates. M-391 pins this in `crates/mycelium-l1/tests/differential.rs`:

- the M-210 three-way differential gains **two further surface-written mutual-recursion shapes** — a
  repr-returning pair and a multi-field-constructor pair — each agreeing across
  L1-eval ≡ elaborate→L0-interp ≡ AOT through the shared checker;
- an **identity** assertion pins that a surface group lowers deterministically to the expected
  `FixGroup` (stable content identity);
- a **never-silent** regression pins that a reference to an *undefined* function stays an explicit
  checker error — never silently swept into a mutual group as a phantom member (G2).

## 4. Scope

This note records a **surface-grammar choice** only. It changes no calculus content, no L0 node, and no
kernel (KC-3), and adds **no public API / no `cargo-public-api` baseline change**. RFC-0007 §8 R7-Q3
(surface half) is resolved by this choice; the grammar spec records the scoping rule as a comment (no
production change). Append-only: to change this choice, **supersede** this note.

## Meta — changelog
- **2026-06-19 — Resolved.** RP-6 resolved: nodule-wide mutual visibility, no new syntax (candidate 2).
  Confirmed by M-391 (differential + identity + never-silent). Append-only.
