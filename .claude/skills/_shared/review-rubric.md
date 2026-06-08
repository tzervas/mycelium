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
