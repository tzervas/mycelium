# ADR-032 — Certification is tunable, not always-on; reframe "honesty" as "transparency & auditability"; reposition the north star toward a fast, memory-safe, ergonomic language

| Field | Value |
|---|---|
| **ADR** | 032 |
| **Title** | Supersede the *unconditional* reading of the always-on certification mandates (SC-3/FR-M3 framing) in favour of RFC-0034's tunable modes; reframe the project's "honesty" vocabulary as "transparency & auditability" (mechanism unchanged); and reposition the north star toward *a fast, memory-safe, ergonomic multi-paradigm language with certification baked in as optional* |
| **Status** | **Enacted** (2026-06-24) — ratified **Proposed → Accepted → Enacted** by the maintainer (house rule #3, stepped, not skipped). The decision is a documentation/decision change and is **fully realized**: the supersession + the whole-corpus transparency reframe + the append-only footnotes were **applied to the corpus** (21 amendments across 13 files via `tools/dn29_apply.py`). The runtime mode *mechanism* it adopts (decision 1) is **RFC-0034's paired TDD implementation** — pending, never claimed done (VR-5/G2). |
| **Date** | 2026-06-24 |
| **Depends on** | RFC-0034 (the tunable-certification *mechanism* this ADR adopts); DN-29 (the settled deliberation — §9 fully resolved, §11 ripple map); **KC-4** (cost-driven downgrade already authorized); **VR-5 / G2** (downgrade-don't-overclaim; never-silent — preserved); ADR-014 (unsafe `permitted-but-warned` — sharpened, not superseded) |
| **Supersedes** | the **unconditional reading** of **SC-3** ("zero swaps without a certificate"), **FR-M3** ("always emits a certificate"), and the always-on framing of RFC-0001 §3.4 / §4.6, RFC-0002 §2, RFC-0005 §2 — they hold **at the active mode** (`certified`), with the **mode itself never silent**. The mechanisms are **not** superseded; only "always, on every value/swap" becomes "at the active mode." |
| **Amends** | **Foundation §1** (the north-star / value-proposition framing); the **CLAUDE.md** "What this repo is" + house-rule 1 ("The honesty rule") wording; **CONTRIBUTING.md** honesty-rule wording; **docs/Glossary.md** "honesty" entries — reframing "honesty / never lie" → "transparency & auditability." Append-only §-end footnotes are added to the Accepted RFCs/ADRs the modes relax (RFC-0001/0002/0005, ADR-010/011/013/016/017). The Foundation **mission sentence is already transparency-framed** ("a transparent, first-class, formally-auditable artifact") and needs no change. |

## Context

The maximalist design phase mandated the full machinery **unconditionally**: every value carries a
`GuaranteeStrength` (RFC-0001 §3.4), every swap emits + checks a certificate (SC-3, FR-M3, RFC-0002 §2),
every value/definition is content-hashed (RFC-0001 §4.6). That was the right default for *establishing* the
honesty discipline. The maintainer's assessment (2026-06-24): *"certification everywhere is messy and
expensive"* — it taxes every line for assurance most code does not need.

DN-29 deliberated the fix to a settled conclusion (its §9 has no open questions): **certification depth and
transparency are separable.** Transparency = *operations are never opaque and never overclaim*;
certification depth = *how much you bother to establish, and whether it is checked.* Splitting them makes the
expensive machinery a **tunable policy** (RFC-0034) while transparency **survives at every setting**, because
the corpus already contains the hooks (the `Declared` tier, KC-4's authorized downgrade, VR-5's
downgrade-and-disclose, G2's never-silent).

Two framing problems surfaced alongside the mechanism, and DN-29 resolved both as **charter-level** changes
that belong in a decision record rather than a silent edit:

1. **The "honesty" vocabulary overreaches.** The "honesty rule / never lie" register is moralizing for what
   is really an *engineering* property: **transparent, inspectable operations** (debug + non-certified
   auditability by default) that become a **fully auditable framework** when certification is engaged. The
   *mechanism* — never-silent (G2), the `Exact⊐Proven⊐Empirical⊐Declared` lattice, `EXPLAIN`,
   downgrade-don't-overclaim (VR-5) — is unchanged; only the words move.
2. **The north star reads as "certified-everything substrate."** The intended end-state is **a fast,
   memory-safe, ergonomic multi-paradigm language**, with certification/transparency baked in as **optional**
   capabilities engaged when needed — not a tax paid on every line.

Per DN-29 §11.5 (Q12), **RFC-0034 stays implementation-focused** and **this ADR carries the charter reframe**,
keeping the charter change in the append-only decision record.

## Decision

1. **Adopt RFC-0034's tunable-certification model.** Certification is a matrix of independent knobs grouped
   by compile/runtime phase, preset into two **first-class modes** — **`fast`** (default) and **`certified`**
   — with **`balanced`** an optional intermediate. `fast` is the project default; `certified` is opt-in per
   `global/phylum/nodule` scope (RFC-0034 §5–§6).

2. **Supersede the *unconditional* reading of the always-on mandates.** SC-3, FR-M3, RFC-0001 §3.4/§4.6,
   RFC-0002 §2, and RFC-0005 §2 hold **at the active mode** (i.e. at `certified` for swap-cert checking; the
   `EXPLAIN`-of-the-mode obligation is mode-independent). The **mode itself is never silent** (G2): every
   result is mode-tagged, tooling surfaces the mode, and cross-mode composition is explicit. The
   **mechanisms are not superseded** — only "always, on every value/swap" becomes "at the active mode."

3. **Reframe "honesty" → "transparency & auditability" across the corpus.** Whole-corpus (DN-29 §9 Q11):
   CLAUDE.md house-rule 1, CONTRIBUTING, Foundation, the Glossary, and the colloquial RFC/ADR uses reword
   from "honesty / honest / never lie" to "transparency & auditability / transparent / accurate." Default
   `fast` gives **transparent, inspectable, non-certified auditability**; `certified` upgrades the same trail
   into a **fully auditable** framework. **The mechanism, the lattice, VR-5, and G2 are unchanged** — VR-5's
   *formal* Gaussian-approximation rule and G2's never-silent rule keep their force; only their colloquial
   "honest" phrasing is reworded.

4. **Reposition the north star.** Foundation §1's value-proposition framing (and CLAUDE.md's "What this repo
   is") move from *"certified, never-silent substrate with honest per-operation guarantees"* toward *"a fast,
   memory-safe, ergonomic multi-paradigm language, with certified/auditable semantics as optional, baked-in
   capabilities."* Memory-safety, speed, and ergonomics rise to **first-class goals**. (The Foundation
   mission *sentence* already reads "transparent, first-class, formally-auditable" — unchanged.)

5. **Memory-safe by default, explicit per-use escape (sharpens ADR-014).** The surface is memory-safe by
   default; unsafe memory ops are reachable only through an **explicit per-use** escape at the call site —
   ADR-014's `permitted-but-warned` sharpened from a global toggle to a per-use, source-visible opt-in.
   ADR-014 is **sharpened, not superseded** (its dev/test warning + mandatory `// SAFETY:` still apply).

6. **Append-only footnotes, not rewrites, on the relaxed Accepted decisions.** RFC-0001/0002/0005 and
   ADR-010/011/013/016/017 each get a §-end footnote — *"mandates apply at `certified`; `fast`/`balanced`
   relaxations per RFC-0034 + ADR-032"* — preserving the originals while pointing forward (house rule #3).

The amendments of (2)–(6) are applied **after this ADR and RFC-0034 are Accepted**, via the staged,
never-silent, anchor-keyed manifest (RFC-0034 §13 / DN-29 §11.4) — never silently, never by hand-mangling.

## Consequences

- **Enables** a fast, ergonomic default (`fast`) without forfeiting assurance where it matters (`certified`),
  and **decouples deployability from certification** — spores are mintable with the runtime cert-off
  (RFC-0034 §8).
- **Costs** the absolute "every swap is certified, always" property as a *static* guarantee. **Mitigations:**
  the mode is never-silent (G2) — every result is mode-tagged and `EXPLAIN`-able; `fast` never overclaims
  (VR-5 — structural tags only); `certified` reinstates the full machinery unchanged; the relaxation is
  **recorded here**, not slipped in.
- **Honesty/transparency survives** the relaxation exactly as VR-5/G2 require: this is a *systematic, flagged
  downgrade*, the generalization of KC-4 from a kill-switch into a knob — not a new way to hide behaviour.
- **Append-only:** this ADR *amends* the framing and *supersedes* the unconditional reading; it does **not**
  rewrite the maximalist-phase rationale. If tunability proves too loose (e.g. consumers need
  certified-by-default for shared phyla), **supersede with a tighter ADR** (e.g. default-by-kind), don't
  rewrite history.
- The corpus text edits are a **single batched pass per file** (RFC-0034 §13) with a never-silent guard, so
  the high-collision files (README, Foundation, CLAUDE.md) and the multi-category lines (DN-29 §11.3) are
  rewritten whole, not sequentially mangled.

## Definition of Done

Ratified **Proposed → Accepted → Enacted** by the maintainer in lockstep with RFC-0034; the supersession
scope (which mandates become per-mode), the vocabulary reframe scope (whole-corpus, **living docs + charter
criteria; Accepted RFCs/ADRs footnoted, not rewritten; process/colloquial "honest" excluded**), the
north-star reframe, the memory-safety sharpening, and the footnote list are stated and grounded. **Enacted
(satisfied 2026-06-24):** the manifest was **applied** — the living-doc transparency reframe + the SC-3/FR-M3
charter conditionalize + the append-only footnotes landed (21 amendments / 13 files), `doc_refs` is green,
and **DN-29 is now Superseded** by this ADR + RFC-0034. *Residual (paired TDD):* the runtime mode mechanism
(decision 1) and the ADR-014 per-use lint sharpening's enforcement land as code — tracked, never claimed
done.

## Grounding

DN-29 (settled deliberation; §9/§11), RFC-0034 (the mechanism), **KC-4** (authorized cost downgrade), **VR-5
/ G2** (downgrade-don't-overclaim; never-silent — preserved), ADR-014 (unsafe policy — sharpened),
RFC-0001/0002/0005 + ADR-010/011/013/016/017 (the relaxed mandates, footnoted append-only), Foundation §1
(the reframed charter).
