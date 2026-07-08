# GROK-HANDOFF — extracting `tero-mcp-lite` into its own repo

**Audience:** Grok, running on **its own side** with its GitHub connector and its own network access.
**Why Grok, not this session:** this session is a Claude Code repo-scoped worktree inside the
`mycelium` repo. It can build, test, and zip the package (done — see below), but it **cannot** reach
`api.x.ai`, cannot create a new top-level GitHub repository outside this one via its own connector in
a self-serve way, and cannot run external toolchains/security scanners this environment has no network
path to. Grok's side has the connector + network this handoff needs; do the extraction there.

## What's already done (this repo, this session)

- The package lives at `packages/tero-mcp-lite/` on branch `claude/portable-tooling-packages`
  (PR into `dev` — see the PR description for the number/link).
- It is a self-contained `uv` project: `pyproject.toml`, `src/tero_mcp_lite/` (5 modules, ~700 LOC,
  zero runtime dependencies), `tests/` (pytest, 27 tests, all offline/fast), `README.md`,
  `GENERATING-AN-INDEX.md`.
- `uv sync` and `uv run pytest` are green in this environment.
- A zipped deploy artifact is committed alongside it: `packages/tero-mcp-lite.zip` (built with
  `cd packages && zip -r tero-mcp-lite.zip tero-mcp-lite/`, excluding `.venv`/`__pycache__`/`uv.lock`
  churn — see the zip's own file list to confirm).
- It has **not** been through any dependency/supply-chain security scan, SAST, or secret scan beyond
  this repo's own light `scripts/checks/secrets.sh` fallback (no `gitleaks` binary was available in
  this session to run the full scan) — that's this handoff's job, on infrastructure that can actually
  reach scanning services/registries this repo-scoped session cannot.

## Runbook

### 1. Create the new repo (Grok's GitHub connector)

```text
Repo name suggestion: tero-mcp-lite
Visibility: your call (public matches the MIT-license intent — see below)
Initialize: empty (no README/license/gitignore — the zip brings its own)
```

Use whatever your connector's create-repo call is; the repo should end up empty and ready to receive
the extracted tree.

### 2. Extract the zip as the repo tree

```bash
# Somewhere Grok's side can run shell + git:
unzip packages/tero-mcp-lite.zip -d /tmp/extract
cd /tmp/extract/tero-mcp-lite   # the zip's root folder IS the package root

git init
git add -A
git commit -m "Import tero-mcp-lite (extracted from mycelium/packages/tero-mcp-lite)"
git branch -M main
git remote add origin <the new repo's URL>
git push -u origin main
```

If you'd rather preserve provenance instead of a fresh `git init`, note the source coordinates in the
commit body instead: *"Extracted from `mycelium` at commit `<SHA of the PR head on
claude/portable-tooling-packages>`, path `packages/tero-mcp-lite/`."* (Fill in the actual SHA — check
the PR this handoff references.)

**Add a top-level `LICENSE` file (MIT)** in the new repo — the package `pyproject.toml` declares
`license = "MIT"` and the README says "MIT — see the repository root LICENSE," but a standalone repo
needs its own actual `LICENSE` file (Mycelium's own `LICENSE` is a fine template; just update the
copyright line to the new repo's ownership/contact — see `tz-dev@vectorweight.com` /
`github.com/tzervas` in the package README, which is explicitly flagged there as swap-able).

### 3. Run what this environment couldn't

This repo-scoped Claude Code session already ran (and this handoff should **not** re-run
redundantly): `uv sync`, `uv run pytest` (27 passed), `ruff check`/`ruff format --check` (clean).

Run, on your side, whatever this session's network couldn't reach:

- **`uv lock --upgrade`** (or a fresh `uv sync`) against the *live* PyPI index from a fully-networked
  environment, to catch anything that drifted between when this session resolved `pytest` and now.
- **Security scanning** with real repository / supply-chain scanners. **Note:** `tzervas/security-mcp`
  is a *content/text* screener (PII/secret/injection patterns over MCP request/response payloads), **not
  a repository scanner** — it can be layered on to screen *output* text, but it does not replace the
  scanners below. At minimum:
  - A **dependency/SCA scan** over `pyproject.toml`/`uv.lock` — e.g. `pip-audit` or `osv-scanner` (even
    though runtime deps are empty, the `dev` group pulls in `pytest` + its transitive deps — scan those).
  - A **SAST pass** over `src/tero_mcp_lite/*.py` (5 files, ~700 lines — fast) — e.g. `semgrep` or `bandit`.
  - A **secret scan** over the whole extracted tree — `gitleaks` (this session's local fallback only
    checked a narrow high-confidence pattern set — run the real thing here).
  - Optionally a filesystem/config scan with `trivy`, plus SBOM + license generation.
- **Patch findings.** For anything the scan surfaces:
  - A real vulnerability in a transitive `pytest`/dev-only dependency: bump the pin in
    `pyproject.toml`'s `[dependency-groups] dev`, re-run `uv lock`, re-run `uv run pytest`.
  - A finding in `src/tero_mcp_lite/` itself: fix directly — the whole package is small/auditable by
    design (see the README's "why a minimal implementation instead of the `mcp` SDK" section), so a
    fix should stay small too. Keep the never-silent refusal contract and the token-scoped auth model
    intact (don't "fix" a false positive by weakening either).
  - Anything ambiguous or that would change public behavior: leave a clear `FLAG:` comment/issue in
    the new repo rather than guessing — this mirrors the source repo's own G2/VR-5 discipline
    ("never guess, flag instead").

### 4. Push

```bash
git add -A
git commit -m "chore: security scan + patch pass (pip-audit/semgrep/gitleaks/trivy)"
git push origin main
```

Then, optionally, open an issue or PR-back-reference in **this** repo (`mycelium`) noting the new
repo's URL and what the scan found/fixed, so the maintainer has a paper trail linking the two —
`tools/github/issues.yaml` is orchestrator-owned in `mycelium`, so don't edit it directly; just leave
a comment/note pointing here.

## Reference

- Package root (source of the zip): `packages/tero-mcp-lite/` in this repo.
- Package README (install/registration/tests/design tradeoffs):
  `packages/tero-mcp-lite/README.md`.
- Index schema (`GENERATING-AN-INDEX.md`) — needed if the new repo ever wants to serve its *own*
  corpus instead of pointing back at `mycelium`'s: `packages/tero-mcp-lite/GENERATING-AN-INDEX.md`.
- Contact for the package (swap-able, flagged in the README too):
  **[tz-dev@vectorweight.com](mailto:tz-dev@vectorweight.com)** ·
  **[github.com/tzervas](https://github.com/tzervas)**.
