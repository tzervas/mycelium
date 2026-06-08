# scripts/ — local, no-AI check tooling

The deterministic quality gate for Mycelium. **One implementation, three entrypoints** — so
what you run locally is exactly what CI runs:

```
just check            ──┐
pre-commit (just hooks) ─┼──►  scripts/checks/*.sh  ──►  same result everywhere
.github/workflows (CI) ──┘
```

## Use it
```sh
just setup     # install the check tools (uv tool / npx / pip); safe to re-run
just check     # run the full suite
just fmt       # auto-format (rust + python)
just hooks     # install pre-commit so checks run on every commit
just <name>    # run one check: md, links, schema, spell, shell, structured, secrets, lint, fmt-check
```
No `just`? The scripts are plain bash: `bash scripts/checks/all.sh`.

## What runs
| Check | Tool | Notes |
|---|---|---|
| `structured` | python | every tracked `.json/.yaml/.toml` parses |
| `shell` | shellcheck | `*.sh` (via `shellcheck-py` if no system binary) |
| `markdown` | markdownlint-cli2 | config `.markdownlint.jsonc`; run via `npx` |
| `links` | `lint_links.py` | **offline** relative-link / cross-ref / `@import` checker |
| `schema` | check-jsonschema | draft 2020-12 metaschema + example instances (per M-010) |
| `spell` | codespell | config `.codespellrc` |
| `secrets` | gitleaks | respects `.gitleaks.toml`; narrow fallback if gitleaks absent |
| `format` | cargo fmt / ruff | check-only; `--fix` to write |
| `lint` | clippy / ruff | `clippy -D warnings` per CONTRIBUTING |

## Design rules
- **Graceful skip:** a check whose tool or language isn't present prints `skip` and exits 0 —
  it never fails the suite. (Most code doesn't exist yet; checks light up as it lands.)
- **Tracked files only:** checks operate on `git ls-files` output (no `node_modules`, `target`).
- **Parity:** add new logic to `scripts/checks/*`, then expose it as a `just` recipe and a
  pre-commit hook — never reimplement a check in the workflow or the justfile.
- **Exit convention:** `0` = pass or skip; non-zero = real failure.

## Remote CI
`.github/workflows/checks.yml` is **manual-dispatch only** (`workflow_dispatch`) and
**advisory** (non-blocking) — it runs `just ci`, i.e. this same suite. See the repo CI policy
in `CLAUDE.md`.
