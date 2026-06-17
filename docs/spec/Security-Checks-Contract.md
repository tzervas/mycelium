# Spec (Proposed) — security checks as tooling contract (secrets / supply-chain / `wild`-audit)

| Field | Value |
|---|---|
| **Status** | **Proposed** (2026-06-16 — the M-367 security-checks contract; design-first, present before folding) |
| **Scope** | The contract for lifting the `/security-review` posture into a full-fat-suite tool: the three check families (secrets · supply-chain · `wild`-block audit), the severity/honesty contract (every finding cites *why*; a reduced-coverage skip is named, never a silent pass), `EXPLAIN`, and the "a new scanner is an ADR" rule |
| **Depends on** | The existing check scripts `scripts/checks/{secrets,deny}.sh` (gitleaks + fallback; cargo-deny + cargo-audit); `deny.toml` + `.gitleaks.toml` (present); ADR-014 (the `unsafe`-code policy — permitted-but-warned, `// SAFETY:` mandatory); LR-9 / S6 / DN-02 §5 (the **`wild`** block — the denied-by-default unsafe escape hatch, lexically marked); the grammar (`docs/spec/grammar/mycelium.ebnf` — `wild`); G2 (a flagged item cites why; never-silent); KC-3 (above the kernel) |
| **Feeds** | M-361 (the full-fat toolchain — the security gate); CI (`scripts/checks/`); the `/security-review` skill (shared posture) |
| **Grounds on** | `scripts/checks/secrets.sh`, `scripts/checks/deny.sh`; `deny.toml`, `.gitleaks.toml`; the `wild` lexicon (DN-02 §5, Glossary, EBNF `wild`); ADR-014 |

## 1. Summary

The `/security-review` posture already exists as advisory scripts; M-367 lifts it into a **first-class tool**
of the full-fat suite, with the same honesty discipline the language demands of itself: **every finding
cites why it is a finding**, and — the load-bearing point — **a check that cannot run is reported as
reduced coverage, never as a pass** (a green that is actually "the scanner is absent" would be exactly the
silent lie G2 forbids). Three check families, all reusing existing infrastructure:

1. **Secrets** — `gitleaks` (+ the narrow high-confidence fallback) over the tree.
2. **Supply-chain** — `cargo-deny` (advisories/licenses/sources/bans via `deny.toml`) + `cargo-audit`
   (RustSec) over `Cargo.lock`; the "pinned deps, lockfile integrity, no `curl|bash`" review lines.
3. **`wild`-block audit** — inventory **every** `wild { … }` block (LR-9/S6/DN-02 §5 — the denied-by-default
   unsafe escape hatch) and require each to be justified (ADR-014's `// SAFETY:` discipline at the language
   level).

Presented design-first; no new tooling code lands until acknowledged (the M-367 gate). Adding any **new
external scanner** stays an **ADR decision**, not a build detail (the M-359 dependency line).

## 2. The honesty contract for a security tool (the crux)

A security tool's most dangerous failure mode is a **false green**. So:

- **Skip ≠ pass.** When `gitleaks`/`cargo-deny`/`cargo-audit` is absent, the tool reports
  **`reduced-coverage: <tool> not installed`** and says exactly which checks did *not* run. The existing
  scripts already model this (`skip` lines); M-367 makes the *aggregate* honest — an overall "OK" is only
  emitted when coverage is full, otherwise "OK (reduced coverage: …)" (named, never hidden). (G2 / VR-5 —
  the honest claim is *what was actually checked*, not *clean*.)
- **Every finding cites why.** A flagged item carries: the family, the rule (e.g. the gitleaks rule id, the
  RustSec advisory id, or `wild-unjustified`), a location, and a one-line *why this matters*. No bare
  "suspicious" with no basis.
- **Severity is declared, not guessed.** A fixed severity map (below); a finding's severity is looked up,
  never inferred by heuristics.

## 3. Severity model

| Severity | Meaning | Examples |
|---|---|---|
| **critical** | ships a secret or a known-exploitable advisory | a real private key / token (gitleaks); a RustSec advisory with a fix |
| **high** | a supply-chain policy violation | a denied license, an unvetted source, a banned crate (`cargo-deny`) |
| **medium** | an unjustified unsafe surface | a `wild` block with no `// SAFETY:`/justification; an unmaintained-crate advisory |
| **low / info** | hygiene | a `curl\|bash` pattern in a script; a fallback-only secret scan (reduced coverage) |

`critical`/`high` fail the gate (non-zero); `medium` fails under `--strict`; `low/info` are advisory. The
map is fixed and inspectable (no learned scoring).

## 4. The `wild`-block audit (the Mycelium-specific check)

`wild` is the language's only unsafe escape hatch — **denied by default, lexically marked** (LR-9/S6;
DN-02 §5; a `phylum`/`nodule` with no `wild` blocks is safe by construction). The audit:

- **Inventories every `wild` block** in the project's sources (a total scan, like the M-141 lints over text)
  — so the unsafe surface is *known*, never ambient.
- **Requires each to be justified** — a `wild` block must carry a justification (the ADR-014 `// SAFETY:`
  discipline lifted to the surface); an unjustified `wild` is a `wild-unjustified` finding (medium).
- **Reports the inventory under `EXPLAIN`** — "here is every place this project leaves the safe culture, and
  why" — the security analogue of the never-silent rule (the unsafe surface is reified, not hidden).

This is the check no off-the-shelf scanner gives Mycelium; it is the reason the audit is in the suite.

## 5. `EXPLAIN` / no black box

```
security: geometry
  secrets:      ok (gitleaks 8.x, 0 findings)
  supply-chain: ok (cargo-deny clean; cargo-audit 0 advisories)
  wild-audit:   1 block, 1 justified, 0 unjustified
                - crates/x/src/ffi.rs: wild { foreign_decode(ptr,len) }  [justified: // SAFETY: …]
  coverage:     FULL
```

If a scanner is absent: `coverage: REDUCED (gitleaks not installed — secret scan ran fallback only)`. The
coverage line is the receipt — an overall OK with `REDUCED` coverage is **not** a clean bill, and says so.

## 6. CLI surface & scope

The tool wraps the existing scripts (so local↔CI parity is preserved — one source of truth) and adds the
`wild`-audit:

```
myc-sec [--strict] [--explain] [--format human|json] [--config <toml>]
```

`--strict` promotes `medium` to failing. **No new external dependency** in v0: it orchestrates the
already-present `gitleaks`/`cargo-deny`/`cargo-audit` (each optional, honestly degraded when absent) and
implements the `wild`-audit in-repo (a text scan, like the M-141 header lints). Adding a **new** scanner
(e.g. a SAST engine) is an **ADR** (the dependency discipline; KC-3 keeps it above the kernel).

**v0 scope (honest):** secrets + supply-chain are *orchestration + honest aggregation* over existing tools;
the `wild`-audit is the new in-repo check, scoped to **inventory + justification-presence** (it does not
*prove* a `wild` block sound — that is the author's `// SAFETY:` argument, surfaced not adjudicated; VR-5 —
we report the claim, we don't fabricate a verdict).

## 7. Test plan (acceptance gate)

1. **Skip ≠ pass** — with a scanner absent, the aggregate is `OK (reduced coverage)`, never a bare OK; the
   coverage line names the missing tool.
2. **Findings cite why** — a planted high-confidence secret, a denied license (`deny.toml`), and an
   unjustified `wild` each produce a finding with family/rule/location/why.
3. **`wild`-audit** — a justified `wild` block passes; an unjustified one is `wild-unjustified` (medium);
   the inventory is complete and deterministic.
4. **Severity gating** — `critical`/`high` fail; `medium` fails only under `--strict`; `low/info` advisory.
5. **Parity** — the tool's secrets/supply-chain results match the underlying `scripts/checks/` scripts.
6. **JSON** — `--format json` emits the structured findings (G11).

## 8. Open questions (flagged, not decided)

1. **`wild` justification syntax** — reuse ADR-014's `// SAFETY:` comment convention at the surface, or a
   structured `wild` attribute? v0 assumes the comment convention (lowest friction); confirm.
2. **`--strict` in CI** — should the suite's release gate run `--strict` (medium fails)? Defaults to
   advisory medium; confirm the release posture.
3. **New scanners** — any addition (SAST, dependency-confusion checks) is an ADR; none proposed in v0.

## Meta — changelog

- **2026-06-16 — Proposed (M-367 design).** The security-checks-as-tooling contract, design-first. Lifts the
  `/security-review` posture into a suite tool over the existing `scripts/checks/{secrets,deny}.sh` (gitleaks,
  cargo-deny, cargo-audit) plus a new in-repo **`wild`-block audit** (LR-9/S6/DN-02 §5 — inventory every
  denied-by-default unsafe block and require an ADR-014 `// SAFETY:` justification). The crux is the
  **honesty contract**: every finding **cites why**, severity is a **fixed declared map** (not guessed), and
  a missing scanner is **reduced coverage, never a silent pass** (an OK with `REDUCED` coverage is not a
  clean bill — G2/VR-5). `EXPLAIN` prints the coverage receipt + the unsafe-surface inventory. **No new
  dependency** in v0 (a new scanner is an ADR); above the kernel (KC-3). No code lands until acknowledged.
  Append-only.
