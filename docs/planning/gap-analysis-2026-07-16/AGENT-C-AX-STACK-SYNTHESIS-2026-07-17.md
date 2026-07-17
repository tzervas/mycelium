# Design Agent C — AX-stack synthesis (swaps × tags) (2026-07-17)

| Field | Value |
|-------|--------|
| **Status** | **Draft** (council synthesis — **not** Accepted; does not ratify) |
| **Agent** | C — Deterministic machinery + cross-cutting synthesis |
| **Honesty** | Claims are **`Declared`** (design synthesis from A/B) unless tagged otherwise |
| **Scope** | Mycelium only; no product code |
| **Council** | [DESIGN-COUNCIL-SWAPS-TAGS-2026-07-17.md](./DESIGN-COUNCIL-SWAPS-TAGS-2026-07-17.md) |
| **Inputs** | [AGENT-A-SWAPS-ERGONOMICS-2026-07-17.md](./AGENT-A-SWAPS-ERGONOMICS-2026-07-17.md) · [DN-141](../../notes/DN-141-Tagging-Meta-Honesty-Lattice-UX.md) (Agent B) |

> **Posture (VR-5 / G2 / house rule #3).** This synthesizes ranked directions from A and B. It does
> **not** move RFC/DN/ADR status. Hard gates: never-silent swaps; no guarantee upgrade without
> basis; prefer deterministic machinery over ad-hoc convention.

---

## 1. What A and B independently require

| Gate | Swaps (A) | Tags / Meta / lattice (B / DN-141) |
|------|-----------|-------------------------------------|
| **Never silent** | Every Repr change stays a written `swap` (S1/WF1); policy identity always recorded | Grade raise only via meet-safe inference, Swap cert, or predicate remint — never ambient upgrade |
| **Deterministic tables first** | Legal-pair matrix + catalog policies before more sugar | Structural grade catalog + "why this grade" before presentation sugar |
| **Mode orthogonal** | Cert emission mode-gated; writing `swap` is not (DN-29) | Cert depth ≠ lattice grade ≠ typing strictness (DN-126; three axes) |
| **Fallibility honest** | Regime → total / `Option` / `Result` | Airlock = total predicate → `Option[T @ g']`, never cast-to-stronger |

Shared rejection set: ambient auto-insert of swaps; ambient grade *upgrade* defaults; silent
auto-`Proven`/`Empirical`; collapsing cert mode into grade.

---

## 2. Ranked AX-stack (cross product)

Ship as **one design story**, land as **independently reviewable slices**. Order optimizes for
deterministic machinery before sugar (A and B both rank tables first).

| Rank | Slice | Primary source | Closes |
|------|-------|----------------|--------|
| **X1** | Legal-pair matrix in checker + content-addressed catalog policies; EXPLAIN expands `policy: default` | A1 + A2 | Swap P1/P4/P5 |
| **X2** | Structural grade catalog + tag-EXPLAIN consumption tiers (grade · mode · basis) | DN-141 A + F | Tag P1/P3/P6/P7/P9 |
| **X3** | Regime-driven swap result types (total / Option / Result) | A3 | Swap fallibility lie |
| **X4** | `std.airlock` seal/recertify (companion 02 pattern B as phylum) | DN-141 E | Tag P2/P8; meet quarantine |
| **X5** | Cert ambient for swap: value-forward, cert-queryable (resolve `swap.md` §7-Q2) | A4 | Swap P2 |
| **X6** | Optional sugar: `to:` elision from unique expected type; named std ops over keyword; presentation sugar only after X2 | A5/A6; DN-141 B optional | residual ceremony |
| **X7** | Basis-carrying `@ Empirical(…)` / `@ Proven(…)` when certified APIs need it | DN-141 D | Tag P4/P5 |
| **X8** | LSP insert-swap + structured transpile *candidates* (never auto-insert) | A7 | Swap P8/P9 UX |

**Explicit non-goals (both lenses):** R12-Q2-style edge auto-swap; policy with no lexical marker;
nodule-wide grade upgrade; stage-1b grade polymorphism as a prerequisite for everyday ergonomics.

---

## 3. Joint open questions (maintainer)

1. **Swap §7-Q2:** cert-ambient (X5) as default authoring model vs forever-explicit `Swapped`?
2. **Policy elision spelling:** `policy: default` vs `policy: _` vs catalog path only?
3. **Airlock surface name / home phylum** before X4 lands as sugar.
4. **Tag-EXPLAIN generation always-on in `fast`**, or floor *consumption* only (DN-141 OQ-5)?
5. Sequencing vs ONESHOT transpile prep: design pause is intentional; implement only after
   maintainer steers this council (council brief).

---

## 4. Suggested follow-on (Declared)

- Promote X1–X2 into one or two DNs (or extend DN-141 + a future swaps DN) after maintainer steer.
- File build M-ids only after Accepted design; keep packages `Declared` until differential-witnessed.
- Re-rank ONESHOT / expressibility waves **after** council capture — do not silently interleave
  product code under this design phase.

---

## 5. Definition of Done (this synthesis artifact)

- [x] Points at council brief + full A report + DN-141 (B).
- [x] Ranked AX-stack with hard-gate alignment called out.
- [x] Joint open questions for maintainer.
- [ ] Maintainer steers / selects → capture ratifiable DN/ADR edits (out of scope for this land).
