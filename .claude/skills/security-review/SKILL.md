---
name: security-review
description: >-
  Security review for the Mycelium repo, scaled to the change. Covers secrets,
  supply-chain (pinned deps, no curl|bash, lockfile integrity), shell/CI safety,
  input handling, and — for code — the usual vulnerability classes. Auto-detects
  docs-only diffs and runs a light pass; can run an exhaustive all-severities pass.
when_to_use: >-
  Use when asked for a security review/audit of a PR, branch, commit, or the working
  tree; before merging changes that touch dependencies, CI, shell scripts, auth/secrets,
  or anything network-facing.
argument-hint: "[PR#|<base>...<head>|<commit>] [--tier T0|T1|T2] [--all]"
allowed-tools: Bash(git diff:*), Bash(git show:*), Bash(git log:*), Bash(git status:*), Read, Grep, Glob
---

# security-review

Security review, repo-grounded. Shared triage/severity/report contract:
[review-rubric](../_shared/review-rubric.md) — read it first, pick a tier, emit the report.

## Scaling (important)
Security depth scales with **what** changed, not just size:
- **Docs/markdown-only diff** → light pass: secrets scan + check no dangerous shell/links/
  copy-paste commands were introduced. Most findings will be Low/Nit. Say so explicitly.
- **Touches dependencies / CI / shell / `tools/*` / network / parsing untrusted input** →
  this is a fragile path → **T2**, full pass, regardless of line count.

## Checklist
**Secrets & data**
- No credentials/tokens/keys/private hosts in the diff or history. Respect `.gitleaks.toml`;
  run/trust `just secrets`. Any real secret → **Critical** (and advise rotation).
- No PII or internal URLs leaking into committed docs/fixtures.

**Supply chain**
- New dependencies are **pinned** (version/hash), from a known source, and justified. Flag
  unpinned ranges on security-relevant deps, typosquat-prone names, and lockfile drift.
- **No `curl … | bash`** / unverified remote execution in scripts, CI, or docs. Installs should
  be reproducible (pin tool versions; prefer `uv tool`/`npx --yes <pinned>` over piping).
- Note this repo's own pattern: `gh-bootstrap-local.sh` and the package script auto-`pip install`
  pyyaml — call out unpinned/implicit installs as **High** and suggest pinning.

**Shell & CI safety**
- Shell: `set -euo pipefail`; quote expansions; no eval of untrusted input; no command
  injection via interpolated vars; safe `tmp` handling. (Trust `just shell`/shellcheck.)
- GitHub Actions: least-privilege `permissions:`; pinned action SHAs/tags; no untrusted input
  in `run:`; secrets not echoed; **per repo policy, remote CI is manual-dispatch only** — flag
  any workflow that auto-runs on `push`/`pull_request` as a policy violation (Medium+).

**Code (when present)**
- Input validation / injection (SQL/cmd/path/deserialization); authn/authz; unsafe Rust
  (`unsafe` blocks justified + bounded); integer/overflow in numeric/packing code; panics on
  attacker-controlled input; crypto misuse. Unbounded resource use / DoS.

## Report
Emit the rubric §4 format, advisory posture. For a docs-only change, a clean light pass should
say plainly: "docs-only; no security-relevant surface touched; <n> minor notes." Always include
summary + recommendations in `--all` mode.
