# Handoff — RFC-0035 Security Scanning Toolkit (working notes)

> Working notes for the RFC-0035 draft (`docs/rfcs/RFC-0035-Security-Scanning-Toolkit.md`, **Proposed**,
> 2026-06-24). Advisory scratch — not a normative artifact. Records the DN-30→RFC section mapping, the
> grounding, and what remains before RFC-0035 can move Proposed → Accepted.

## What this RFC is

The **binding** security-scanning design lifted from **DN-30** (the Draft direction capture) plus the
maintainer's now-answered DN-30 §7 five open questions. RFC-0035 **designs** the toolkit; it **implements
nothing** — every runtime-behaviour claim is a `Declared` design position for epic **E22-1** to discharge
(VR-5/G2). Status **Proposed**, awaiting ratification (stepped Proposed → Accepted, house rule #3).

## The five answers it binds (DN-30 §7 → RFC-0035 §10 Decisions)

| Q | Maintainer answer (2026-06-24) | RFC §10 |
|---|---|---|
| Q1 v0 vuln classes | FIXED base categories WITH extensibility seam | D1 |
| Q2 reporting schemas | VERSIONED PINNING — pinned = immutable; new versions allowed (additive) | D2 |
| Q3 per-class fix-strength | YES — per-class minimums; higher classes stricter; supports a "pedantic" mode | D3 |
| Q4 screening policy | CONFIGURABLE with sensible defaults; MANDATORY for high-security classes by default; per-project adjustable | D4 |
| Q5 /security-review relationship | SUPPORTING TOOL only — not a replacement/prerequisite; detection + suggested fixes + a CERTIFIED PATCH REGISTRY | D5 |

## DN-30 → RFC-0035 section mapping

| DN-30 | RFC-0035 | Notes |
|---|---|---|
| §1 the gap | §1 Problem & Goal | existing CI hygiene (`.gitleaks.toml`, `deny.sh`, `just safety-check`) + manual `/security-review` → native inherited toolkit |
| §2 automated detection over inspectable IR | §2 v0 classes (+D1) | the Q1 answer adds the fixed-base + extensibility decision; class table grounded in RFC-0028/ADR-014 surfaces + the `/security-review` recurring-defect bank |
| §3 standard reporting | §3 (+D2) | SARIF+CWE+OSV+VEX; Q2 adds versioned-pinned immutable schemas (append-only, content-addressed) |
| §4 two sinks + registry + screened/anonymized + reconstruction-on-render | §4 Registry Integration | DN-28 reconstruction-on-render reused; two content-addressed catalogs; tamper-evident |
| §5 honest semantics-preserving safe auto-fix | §5 (+D3, D5) | RFC-0002 refinement certificate reuse; Q3 per-class fix-strength + pedantic; Q5 certified patch registry |
| §7 Q4 screening governance | §6 Screening Policy (+D4) | configurable defaults, mandatory-for-high-security |
| §6 native + scoped | §7 Native + Scoped | RFC-0034 §6 `@certification` resolution reused (project/phylum/nodule/granular) |
| §8 DoD (note) | §9 DoD (RFC) | RFC DoD adds the two worked examples as pre-Accepted gates |

## Grounding / substrate reuse (all pre-existing, unchanged)

- **RFC-0001** — inspectable Core IR (the analysis surface; no black boxes), the `Exact ⊐ Proven ⊐ Empirical
  ⊐ Declared` lattice, content-addressing §4.6 (the fingerprint mechanism, the canonical-encoding rule).
- **RFC-0002** — the refinement / translation-validation certificate + checker (`crates/mycelium-cert/*`);
  the safe-fix reuses it: prove (a) vuln-eliminated AND (b) refines-original-modulo-vuln.
- **RFC-0034 §6** — `@certification` scoped resolution (project/phylum/nodule, RFC-0012 ambient scoping);
  §13/M-796 native scoped testing toolkit is the template for "native + inherited + scoped".
- **RFC-0013/0014** — structured diagnostics + declarative recovery (the reporting + fix machinery, EXPLAIN).
- **RFC-0028 / ADR-014** (sharpened by ADR-032) — the `unsafe`/FFI vuln surface.
- **DN-28** — registry content-hash DAG + reconstruction-on-render distribution; the advisory catalog is a
  second catalog of the same shape.

## What's left for Accepted (the deferred, NOT-fabricated work)

Per RFC-0035 §9 (Definition of Done), two **worked examples** are the remaining pre-Accepted work. They were
**deliberately not fabricated** in the draft (VR-5/G2 — would be over-claiming a demonstrated design):

1. **WE-1 — a safe-fix refinement-certificate worked example.** Take one concrete vuln (e.g. an
   unchecked-bounds or a `wild`-taint case), its safe fix, and discharge the **actual** RFC-0002-style
   certificate obligations end-to-end (vuln-eliminated AND refinement-modulo-the-vuln), at the honest
   strength it genuinely earns. Demonstrates §5 is realizable, not asserted.
2. **WE-2 — a screening case study.** Take one real-shaped finding from raw vulnerable logic through the §6
   screening to a published, content-addressed, anonymized pattern; show the minimization is *sufficient to
   detect + mitigate* and *insufficient to weaponize / leak source*. Demonstrates §4/§6 screening is
   realizable.

Also residual (carried as implementation work under E22-1, not RFC gates):
- the per-class detection algorithms + which honest tag ceiling each reaches in v0 (DN-30 §7 Q3 residual);
- the catalog data model + publish/screen/approve workflow + embargo/coordinated-disclosure governance;
- the `/security-review` skill's concrete consumption surface (D5).

## Files touched by this leaf

- `docs/rfcs/RFC-0035-Security-Scanning-Toolkit.md` — **new** (the RFC; Status Proposed; changelog footer).
- `docs/notes/DN-30-Security-Scanning-Toolkit.md` — **append-only** rev. 3 ("feeds RFC-0035"); existing
  prose unchanged.
- `docs/Doc-Index.md` — new RFC-0035 row (after RFC-0034).
- `CHANGELOG.md` — Unreleased/Added entry (2026-06-24).
- `docs/handoffs/security-rfc-context.md` — this file.

`python3 tools/github/doc_refs_check.py` passes (no new `doc_refs` introduced; E22-1's existing refs
already point at DN-30/RFC-0002/RFC-0034/etc.).
