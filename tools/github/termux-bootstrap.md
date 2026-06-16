# Mycelium GitHub bootstrap — from a phone (Termux)

`termux-setup.sh` is a single, ordered, **idempotent** script to run the **whole** GitHub
project-management bootstrap from an Android phone (Termux) with **nothing pre-configured** —
no MCP session, no desktop. It provisions the host, sets up auth + commit signing, and then
runs the bootstrap end to end.

It exists because the two existing entry points each leave a gap on mobile:

| Step | Tool | On a bare phone? |
|---|---|---|
| label colors/descriptions + **milestones** | `gh-bootstrap-local.sh` | needs `gh` + `jq` installed & authed |
| **issue creation** + milestone assignment | `mcp-bootstrap.md` (a model over MCP) | needs a Claude/Grok MCP session |
| packages, git identity, **GPG key**, **gh token** | — | nothing did this |

`termux-setup.sh` fills all three: it installs the packages, generates/uploads a GPG signing
key, authenticates `gh`, then chains `gh-bootstrap-local.sh` (labels + milestones) into
`gh-issues-sync.py` (the gh-driven local analogue of `mcp-bootstrap.md` Steps 1–2: create
absent issues, assign milestones, append `idmap.tsv`).

## Run it

No `curl | bash` — clone the repo first, then run the script from the checkout:

```sh
# in Termux
pkg install -y git
git clone https://github.com/tzervas/mycelium.git
bash mycelium/tools/github/termux-setup.sh
```

Interactive is recommended (it prompts for git identity and a GPG passphrase). Non-interactive
knobs:

```sh
GIT_USER_NAME="Tyler Zervas" GIT_USER_EMAIL=you@example.com \
GH_TOKEN=***  bash tools/github/termux-setup.sh        # token instead of browser OAuth

bash tools/github/termux-setup.sh --dry-run-issues     # show what would be created
bash tools/github/termux-setup.sh --skip-install --skip-gpg   # re-run, auth+bootstrap only
```

Flags: `--repo`, `--repo-dir`, `--no-gpg-passphrase`, `--skip-install`, `--skip-gpg`,
`--skip-issues`, `--dry-run-issues`, `--help`.

**Already provisioned (`gh` authed), just re-syncing the PM state?** Skip the full setup and run
the one idempotent gap-closer directly:

```sh
bash tools/github/gh-sync-all.sh            # preflight → labels + milestones → absent issues + idmap
bash tools/github/gh-sync-all.sh --dry-run  # preview issue creation, no repo writes
```

It reconciles the repo with `issues.yaml`/`labels.json`/`milestones.json` (a `manifest-check.py`
preflight fails fast if an issue references a label/milestone the manifests don't define — so a
missing label can't silently leave issues uncreated). Rerun any time a manifest gains entries.

## Security model (house rules: never-silent, no black boxes, KISS)

- **No secrets in the repo, ever.** Your GPG *private* key stays on the device; only the
  *public* key is uploaded (`gh gpg-key add`). The GitHub token is held by `gh` in its own
  config and the git remote uses the credential helper — never a token-in-URL.
- **Packages come from the package manager** (`pkg`, else `apt-get`). No `curl | bash`.
- **The GPG key is passphrase-protected** (pinentry) by default. `--no-gpg-passphrase` is an
  explicit, warned opt-out.
- **Every step is idempotent and announced.** Re-running is safe: existing keys/auth/issues
  are detected and reused, never duplicated. `idmap.tsv` is *appended to*, never rewritten.

## After it runs

`gh-issues-sync.py` may append new `task_id → number → db_id` rows to `tools/github/idmap.tsv`
(e.g. M-358/359/361/362/363 once Phase 8 is bootstrapped). Review and commit it — commits are
GPG-signed:

```sh
git add tools/github/idmap.tsv
git commit -m "chore(github): record bootstrapped issue ids"
```

## What it does *not* do

Dependency / sub-issue linking (`mcp-bootstrap.md` Step 4, the "Grok pass") needs the GraphQL
issue-dependencies API and is **not** automated here — same scope boundary as
`gh-bootstrap-local.sh`. Run that pass from an MCP-capable session when you want the epic
sub-issue graph wired.
