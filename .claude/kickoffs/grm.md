# Kickoff `grm` — Phase-I H2a: the grammar-stability gate (ratification-gated; before mass porting)

> **UID:** `grm` · **Basis:** **ADR-038** (Accepted, 2026-07-01) + the umbrella roadmap
> `docs/planning/road-to-1.0.0-and-mycelium-rewrite.md` **§5 (H2a)** · RFC-0037 (Enacted — its named
> follow-ons: D2-b short repr keywords, RFC-0025 operator wiring) · DN-54 (Accepted — extension
> checker; §10 derive-site consume design-pass M-824) · RFC-0024 §4A.8 (tuple-type decision) ·
> ADR-033 FLAG-1 (`FieldSpec::Fn` soundness; DN-56 kernel-freeze condition 2) ·
> `docs/planning/Blocked-Decisions-Ratification-Map.md` (groups G2/G4/G5).
> **Planned by:** Fable (ADR-038 §2.7); **implemented by:** Sonnet/Haiku per the PM table.
> **References the doc-maintenance contract** (`_doc-maintenance.md`) in its DoD.
> **Rule (roadmap §5): no mass `.myc` porting against a moving grammar.** This kickoff is the gate
> between "porting opportunistically" (`opp`) and "porting the corpus" (Phase II).

## Goal

Close the four grammar-stability items so the surface a mass port writes against is the *final*
one: (1) the RFC-0037 follow-ons (D2-b short repr keywords; RFC-0025 operator wiring); (2) DN-54
completion — the extension-checker verified complete + the derive-site consumption model ratified;
(3) the **tuple-type decision** (RFC-0024 §4A.8); (4) **ADR-033 FLAG-1**. Then regenerate
`mycelium.ebnf`, editor grammars, and the api-index, and open the maintainer's
no-normative-change window.

## ⚠ Maintainer-ratification prerequisites (this kickoff is decision-gated)

Three of the four items **wait on explicit maintainer decisions** — the dossier tasks below *prepare*
them; nothing here decides them (FLAG, never guess — G2/VR-5):

| Decision | Prepared by | Unblocks | Ref |
|---|---|---|---|
| Tuple-type decision | M-920 | M-921 (multi-arg lambda / partial-application delta) + DoD row 3 | RFC-0024 §4A.8 · map group G2 |
| DN-54 derive-site consumption model | M-918 | M-919 (extension-checker enactment) + DoD row 2 | DN-54 §10 (M-824 design-pass) · CURRENT-STATE open decisions |
| ADR-033 `FieldSpec::Fn` FLAG-1 | M-922 | M-923 (implementation) + DN-56 condition 2 | ADR-033 · map groups G4/G5 |

The no-normative-change **window length** is also the maintainer's call (roadmap §5 DoD,
`Declared`). Sequence the dossiers FIRST so ratification latency overlaps the two non-gated
implementation tasks (M-915/M-916).

## Scope

**In:** the four H2a items + the stability close-out (regeneration + window). **Out:** the mass
port itself (Phase II); the kernel freeze declaration (DN-56 — the maintainer's act, after its four
conditions; this kickoff advances condition 3 and prepares condition 2); any grammar *addition* not
named above (a new surface idea mid-kickoff is a FLAG, not a task).

## Swarm method + model tiering (ADR-038 §2.7)

**Small serial Sonnet swarm.** Nearly every implementation task touches the `mycelium-l1` frontend
(`lexer/parse/checkty/elab`) — the repo's one serial lane — so leaves run **one at a time**
(land + pull down before the next). The three dossier tasks are docs-only and run in parallel with
each other and with the lane. No fan-out beyond 1 implementation leaf + parallel dossier leaves.
One isolated worktree per leaf (mitigation #11); commit/push split (#12); scoped PRs to `dev` via
`/pr-land`.

## PM decomposition — bite-sized tasks

Proposed M-ids **M-915…M-924** (next-free after `enb`'s block; re-verify at minting —
mitigation #1). None minted by this doc. Verification-first discipline throughout: the 2026-06-29 serial
closeout landed adjacent work (M-822/M-826 tuple + partial application; M-824 DN-54 §10
design-pass; `lwd`'s M-812-cont checks) — every task below **verifies the landed state before
building**, so nothing is re-landed and no gap is assumed closed.

| M-id (proposed) | Task | User story | Definition of Done | Model | depends_on |
|---|---|---|---|---|---|
| M-915 | **RFC-0037 D2-b short repr keywords** — the ergonomic aliases for the paradigm type-keywords (lexer + grammar + `mycfmt` + conformance) | As a `.myc` author, I want the short repr keywords the ratified grammar promises, so that ported code reads idiomatically before the corpus is ported | Aliases lexed/parsed/formatted per RFC-0037 D2-b; `mycelium.ebnf` + editor grammars updated; conformance accept + reject; `mycelium-l1` + `mycelium-fmt` green | Sonnet | — (serial-lane slot 1) |
| M-916 | **RFC-0025 operator wiring follow-on** — close the enacted migration's named non-blocker: verify what M-745/RFC-0037 already wired (`< <= > >= << >>`, `lte`/`gte` desugaring), land ONLY the residual wiring | As a `.myc` author, I want the full ratified operator surface wired, so that operator spellings don't churn under a port | Verified inventory (wired vs residual) recorded (`Empirical`); residual wired per RFC-0025 §4.2/§4.3 as resolved by RFC-0037; desugaring conformance; no new precedence decisions (FLAG if one appears) | Sonnet | M-915 (serial lane) |
| M-917 | **DN-54 completion audit** — verify the `lwd`-landed M-812-cont checks (§4.1 RHS type-check, §4.2 cross-rule acyclicity, §6 KC-3 kernel-growth guard, §7 harness, RHS elaboration to L0) against DN-54 as written; re-run the §7 harness; ledger residuals | As the mass-port effort, I want the lowering/extension surface verified final, so that ported nodules never chase a moving `lower`/`derive` semantics | Per-section audit table (landed / residual / N-A) recorded (`Empirical`); §7 harness re-run green; residuals become explicit tasks or FLAGs — none silent | Sonnet | — (parallel; read-mostly) |
| M-918 | **DN-54 derive-site consumption-model dossier** — options + recommendation from the M-824 §10 design-pass, for maintainer ratification (the model is underdetermined; DN-54 stays Accepted until enacted) | As the maintainer, I want the consume-model choice laid out with tradeoffs, so that I ratify a grounded design, not a default | Dossier with options, evidence, recommendation, and the `enb` M-901 cross-check (same model — the two kickoffs must not fork it: FLAG if they diverge); decision requested | Sonnet | M-917 (audit input) |
| M-919 | **Enact the ratified consume model in the extension-checker** + step DN-54's status honestly (Accepted → Enacted only when fully landed; append-only) | As a `.myc` author, I want `derive`-site consumption checked per the ratified model, so that extension-lowering is sound before the corpus depends on it | Checker enforces the ratified model; reject-case conformance; DN-54 status stepped only if genuinely complete (else recorded as partial — VR-5) | Sonnet | M-918 + **maintainer ratification** |
| M-920 | **Tuple-type decision dossier** (RFC-0024 §4A.8) — reconcile the landed M-822/M-826 state (first-class tuple + `f(x)(y)` + partial application) with the open formal decision; enumerate what the decision still governs (multi-arg arrows, signature forms); options + recommendation | As the maintainer, I want the tuple question closed formally against what already landed, so that ported signatures never reshape after the fact | Dossier: landed-state inventory (`Empirical`), the residual decision surface, options + recommendation; decision requested; map group G2 updated | Sonnet | — (parallel; docs) |
| M-921 | **Post-decision tuple follow-through** — implement ONLY the delta the ratified decision directs (e.g. multi-arg lambda forms), or record "no delta" with evidence | As a `.myc` author, I want function signatures in their final shape, so that mass-ported code compiles unchanged through 1.0 | The decision's delta landed with conformance (or a grounded no-delta record); grammar regenerated; `mycelium-l1` green | Sonnet | M-920 + **maintainer decision** (serial lane) |
| M-922 | **ADR-033 FLAG-1 dossier** — `FieldSpec::Fn` (Fn-typed record-field) soundness options + recommendation; note it is DN-56 kernel-freeze condition 2 (the last open primitive-set question) | As the maintainer, I want the last primitive-set question framed with its soundness evidence, so that the kernel freeze has a decided condition 2 | Dossier with the soundness analysis, options, recommendation; decision requested; map groups G4/G5 updated | Sonnet | — (parallel; docs) |
| M-923 | **Post-decision FLAG-1 implementation** — Fn-typed record-field lowering per the ratified resolution (or the ratified refusal recorded as a reject-ledger row) | As a `.myc` author, I want record fields holding functions to either work soundly or refuse explicitly, so that object-style ports don't hit an undecided hole | The resolution landed (implementation + conformance, or ledgered refusal — a refusal is a decision, not a leftover); ADR-033 status honest | Sonnet | M-922 + **maintainer decision** (serial lane) |
| M-924 | **Stability close-out** — regenerate `mycelium.ebnf`, editor grammars, `docs/api-index/`; record the grammar-stable baseline; propose the no-normative-change window for the maintainer to set (`Declared` until set) | As the mass-port effort, I want a dated, regenerated, maintainer-windowed grammar baseline, so that "stable" is a checkable claim before the corpus port begins | All artifacts regenerated + committed; baseline recorded in the roadmap/CURRENT-STATE; window proposed to the maintainer; H2a DoD rows checked with evidence | Haiku | M-915, M-916, M-919, M-921, M-923 |

## Definition of Done (kickoff)

- All four H2a items closed **with their decisions cited** (roadmap §5): RFC-0037 follow-ons
  landed; DN-54 verified complete + consume model ratified and enacted; tuple decision ratified
  (delta landed or grounded no-delta); FLAG-1 resolved (implementation or ledgered refusal).
- `mycelium.ebnf`, editor grammars, api-index regenerated and stable; the no-normative-change
  window proposed (length = maintainer's call, `Declared` until set).
- Nothing self-ratified; every decision row shows the maintainer's act (house rule #3/#4);
  doc-maintenance per `_doc-maintenance.md`.

## Prerequisites

1. **`acy` (H0) landed** (the enforcement gate). `enb` (H1) need **not** be complete — H2a is
   grammar/decision work, parallel to the prim lane except where both touch the L1 frontend
   (serialize those slots against `enb`'s lane; one L1 surgery in flight at a time).
2. **ADR-038 ratification** for the H2a framing (the individual items are also grounded in their
   own ratified docs — RFC-0037's named follow-ons, DN-54, RFC-0024, ADR-033 — so dossier work may
   start regardless).
3. **The three maintainer decisions above** gate M-919/M-921/M-923 — schedule the dossiers first;
   expect ratification latency; never guess past a pending decision.
