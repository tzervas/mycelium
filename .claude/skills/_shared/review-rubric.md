# Review rubric — adaptive tiers, severity taxonomy, report format

Shared contract for the `pr-review`, `security-review`, and `docs-review` skills so they
triage, score, and report **consistently**. It encodes *Mycelium's* rules (see
`CONTRIBUTING.md` — the honesty rule, append-only decisions, grounding citations), not
generic ones. Read this first, then apply your skill's domain checklist.

---

## 1. Risk triage → pick a depth tier

Inspect the diff before reviewing. Gather it with `git show <ref>` / `git diff <base>...<head>`
for local refs, or the GitHub MCP `pull_request_read` (and the PR diff) for a PR. Then score:

**Signals**
- **Size/spread:** lines changed, files touched, number of distinct subsystems.
- **Change kind:** docs/comment/whitespace · config · code · CI/release · dependency manifest.
- **Path fragility (Mycelium-specific high-risk surfaces):**
  - Core IR / kernel & guarantee lattice — RFC-0001, `mycelium-core`, content-addressing.
  - Swaps & certificates — RFC-0002, the bijection (`LosslessWithinRange`), the cert checker.
  - Verified numerics — ADR-010, the ε/δ bound kernels, the checker.
  - VSA bounds & tags — RFC-0003 (esp. the per-model/op guarantee matrix).
  - Contracts — `docs/spec/schemas/*.json`, anything other code parses against.
  - Selection / EXPLAIN — RFC-0005. Execution/AOT — the MLIR dialect, packing selector.
  - Security/supply-chain — `.gitleaks.toml`, CI workflows, dependency manifests (`Cargo.toml`,
    `pyproject.toml`, `package.json`), shell scripts, `tools/github/*`.
- **Honesty-rule surface:** any change to a guarantee **tag** (`Exact/Proven/Empirical/Declared`),
  a **bound**, an `EXPLAIN`, or the **grounding citation** of a normative claim.

**Tiers** (start at **T1**, then adjust)
- **T0 — Editorial/Trivial.** Only docs/comments/whitespace in non-fragile paths, no
  normative-claim/tag/status change, and small (≈<50 lines). *Depth:* parse/lint/links/format +
  an honesty+grounding spot-check on touched lines + confirm changelog/status moved if a doc
  status changed.
- **T1 — Standard.** Normal change in a non-fragile area. *Depth:* full domain checklist at
  normal depth; verify each touched claim against its cited source; run (or reason through)
  `just check`.
- **T2 — Deep/High-risk.** Touches **any** fragile path or honesty-rule surface, OR changes a
  contract/dependency/CI, OR is large/complex (≈>400 lines or >15 files). *Depth:* maximum
  scrutiny — invariants, adversarial/edge cases, per-op tag justification (is a `Proven` tag
  backed by a theorem whose side-conditions are *checked*?), supply-chain, local↔CI parity,
  threat-model the change.

State the chosen tier **and the signals that set it**. A reviewer may override with an explicit
`--tier` argument.

## 1.5 Recurring defect patterns (grep-first)

Banked from the 2026-06 deep review — the honesty rule leaks at predictable seams. On any change
touching the relevant code, actively check for these (the `dev-workflow` skill states the
corresponding authoring rule). Each maps to a severity by the §2 taxonomy.

- **`Proven`/`Empirical` ε/δ composed in `f64` without outward rounding** — a round-to-nearest
  bound can fall below the real value, so the tag is unbacked. Grep for arithmetic on bound fields
  (`+`/`*` on `eps`/`delta`/radius) not using directed rounding; for re-validation checkers using an
  **absolute** tolerance (vacuous for tiny bounds). (A2-01/A2-02 → Critical/High)
- **Bound/guarantee constructor that silently coerces** out-of-range or non-finite input instead
  of returning `Option`/`Err`; **`pub` fields** on a kernel bound type (bypasses validation). (A2-03/
  A2-05 → High/Medium)
- **Serde wire struct without `#[serde(deny_unknown_fields)]`**, or that deserializes without
  re-checking the JSON-schema constraints it mirrors (e.g. `trials ≥ 1`). A tampered manifest then
  carries a stronger guarantee than its evidence. (A6-02/B2-03 → High)
- **Recursive-descent over untrusted input with no depth guard** (parser/checker/elaborator) — a
  crafted nesting crashes the process. (A4-02/B2-01 → High)
- **Value hashed for identity without rejecting ambiguous encodings** (non-finite `f64` →
  JSON `null` → collision). (A5-01/B2-02 → High/Medium)
- **Analysis tracking a fact across binders that ignores shadowing** (totality, scope). (A4-01 → High)
- **Test weaknesses:** reject/negative test asserting only `is_err()` (not the variant); a
  differential/bound test with no mutation that could make it fail; "≥N trials / empirical" wording
  over a single fixed-seed sample. (A4/A2-07/A3-08-09/A6-04 → Medium/Low)

## 2. Severity taxonomy

- **Critical** — merges a falsehood or breaks a safety invariant: a silent swap/conversion; a
  `Proven` tag without checked side-conditions; a leaked secret; injection/RCE; rewritten
  (non-append-only) decision history; a normative claim that contradicts its cited source.
- **High** — likely-incorrect or a real risk: a shipped approximate op missing its bound +
  tag + property test (SC-2); `curl|bash` / unpinned dependency / unaudited install; a broken
  cross-reference or ungrounded claim in a normative doc; a contract/API change with no doc.
- **Medium** — maintainability/consistency that won't bite immediately: notation drift
  (`⊐`/`⟹`/`µ²`), weak naming, missing-but-advisory `EXPLAIN`, doc/spec drift.
- **Low** — minor polish: wording, small redundancy, narrow test gap.
- **Nit** — subjective/style; non-blocking by definition.

## 3. Modes

- **adaptive (default).** Triage → tier → review at that depth. Report findings at the tier's
  depth but **always surface every Critical/High** regardless of tier. Keep it concise.
- **exhaustive (`--all`).** Ignore tier depth-limiting; review at **T2** depth and tag **every**
  finding by severity. This is the "inform me of everything" mode.

## 4. Report format (both modes)

```
### <skill> — <target>  ·  tier: T?  ·  mode: adaptive|exhaustive
Verdict: <one line>  (advisory — nothing here auto-blocks; you decide at human review)
Triage: <signals that set the tier>

Findings
  [CRITICAL] <file:line> — <what> → <why it matters> → <suggested fix>
  [HIGH]     ...
  [MEDIUM]   ...   (exhaustive mode: include Low/Nit too, grouped)

Summary & analysis
  <2–4 sentences: themes, systemic risks, what's solid>

Recommendations (prioritized)
  1. <highest-leverage fix>  2. ...  3. ...
```

In **exhaustive** mode always include the *Summary & analysis* and *Recommendations* blocks
with findings grouped by severity. In **adaptive** mode they may be one line each if clean.
Posture is **advisory**: report and recommend; do not gate. If asked for a gate, say which
Critical/High items you'd block on and why.
