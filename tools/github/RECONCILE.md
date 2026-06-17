# Mycelium GitHub reconcile contract

One idempotent, **manifest-driven**, cross-platform reconciler for the **entire** GitHub project
state — labels, milestones, issues, pull requests, and the Project v2 board — plus a portable
**commit-signing** setup. Pure Python + the `gh`/`git`/`gpg` CLIs: **no new dependency**, **no
bash, no jq**, runs the same on Linux/macOS and Windows (PowerShell). Aligned to **Conventional
Commits** and standard git/GitHub conventions.

> One command:  `python tools/github/gh-issues-sync.py --all`
> (or the wrappers `gh-sync-all.sh` / `gh-sync-all.ps1`). Preview first with `--dry-run`.

## The universal contract (every level obeys it)

For **every** level — project · fields · options · items · labels · milestones · issues · PRs:

| Property | Meaning |
|---|---|
| **create-if-absent** | a declared thing that does not exist is created |
| **update-to-match** | a thing that exists but drifted is updated to the manifest |
| **in-sync ⇒ zero writes** | re-running when already converged writes nothing |
| **never duplicated** | matched by stable identity (number / title / content id / option name) |
| **`--dry-run`** | prints exactly what *would* change; writes nothing |
| **never-silent (G2)** | every create / update / skip / refusal-to-invent is printed |
| **offline `--self-test`** | the pure decision logic runs with no network |

**Honesty (never invent).** A commit type with no mapping, a scope that is not an `area:*` label,
an ambiguous milestone, a `status:*` label with no Status option — each is **FLAGGED**, never
guessed. Downgrade to a flag to stay honest; never upgrade to an invented label.

**Non-destructive by default.** Issue *bodies* change only with `--update-bodies`; OPEN/CLOSED is
never inferred from a label; **PR labels are add-only** (a human's labels are never stripped);
adding an option to an existing project field is flagged for the UI (its only API path is not
safely idempotent without live validation). Sensitive/creative state — a **GPG key** — is **never**
generated without an explicit trigger.

## Manifests (the declarative source of truth)

| Manifest | Drives | Level |
|---|---|---|
| [`labels.json`](labels.json) | label name · color · description | `--labels` |
| [`milestones.json`](milestones.json) | milestone title · state · description | `--milestones` |
| [`issues.yaml`](issues.yaml) | per-issue title · labels · milestone · body · depends_on | `--issues` |
| [`conventions.json`](conventions.json) | commit/PR-title `type(scope)` → `type:*` / `area:*` + milestone inference | `--prs` |
| [`project.json`](project.json) | Project v2 fields · options · views · automation · label→field map | `--project` |
| [`idmap.tsv`](idmap.tsv) | task-id → issue-number → db-id (append-only) | (all) |

## Levels

- **`--labels`** — reconcile every label to its exact name/color/description (`gh label --force`).
- **`--milestones`** — create each absent milestone by title (state/description from the manifest).
- **`--issues`** — create absent issues; update an existing one's labels/milestone/title to
  `issues.yaml` (matched by `idmap` number first, then title — rename-safe). Bodies only with
  `--update-bodies`; state only when an entry declares `state:`.
- **`--prs`** — enumerate PRs (`state=all`), parse the Conventional-Commit **title** (fallback: the
  PR's **commit messages**), derive `type:*` (and `area:*` only on an exact scope match, else
  FLAG), infer a milestone from referenced task-ids (`M-####` / `E#-#`, unambiguous-only, else
  FLAG), and reconcile **add-only**. Backfills every merged PR and keeps new ones in sync.
- **`--project`** — find-or-create the **Mycelium** board; reconcile fields + single-select
  options; add repo issues/PRs as items; set **Status/Phase/Area/Priority** from each item's
  labels. **Views + built-in workflows are settings-only** → recorded in `project.json` and
  **FLAGGED as manual steps**. See [`project-v2-spec.md`](project-v2-spec.md).

`--all` = the **full maintenance suite**, in order:
**preflight → validate → labels → milestones → issues → PRs → project**.

## Preflight (auto sanity check — proceed when good, remediate only when lacking)

Before any live work, `--all` (and each gh level) runs a sanity check and then **just proceeds if
everything it needs is present**:

- `gh` authenticated? If not → stop with `gh auth login`.
- The scope the *requested* operation needs (`repo` always; **`project`** only for `--project`)?
  If present → proceed **silently**. If **genuinely missing** → print the one-time
  `gh auth refresh -s project` remediation and **skip just the board** (never the whole run;
  never silent). A good token is **never** asked to refresh; a fine-grained token whose scopes
  can't be read is trusted to fail loudly at the call site.
- `--no-preflight` skips the check (the API call still fails loudly if a permission is lacking).

**Auth is secret-free.** The reconciler never reads or stores a token — it shells to `gh`, which
holds the credential (identical on Linux + Windows). Git Credential Manager covers `git` push/pull
over HTTPS and is separate from the API token (complementary, not required for the board).

## Validation (manifests accurate to the codebase)

`--validate` (offline; also the gate at the top of `--all`) checks the manifests are internally
consistent **and** accurate to the repo, never-silently:

- `manifest-check.py` — every label/milestone `issues.yaml` references is defined (blocking).
- every `conventions.json` type/alias target is a real label (blocking).
- `project.json` **Area** options == the `area:*` labels; Status-map targets are real Status
  options (blocking — they would otherwise mis-drive the board).
- `idmap.tsv` ↔ `issues.yaml` coherence and `CHANGELOG.md` hygiene (advisory warnings).

Blocking errors stop `--all` before any write (fix the manifest first); warnings are reported and
the run proceeds. Keep the changelog + per-doc footers in step with each change (the `changelog`
skill / dev-workflow) — `--validate` flags a missing `## [Unreleased]`.

## Commit signing (portable, idempotent, nondestructive)

[`git-signing-sync.py`](git-signing-sync.py) wires signed commits the same honest way (Linux +
Windows, pure Python + `git`/`gpg`/`gh`):

- **Default = sanity check (read-only).** Detects git/gpg/gh, the git identity, an existing key,
  and whether signing is wired; reports **ready / partial / absent**; writes nothing. If ready, you
  proceed with the maintenance workflow; if not, it offers `--setup` (interactive) or prints the
  remediation.
- **`--setup` / `--init`** — the explicit, opt-in trigger. Prompts for **name / email / comment**,
  **reuses** an existing key, and **generates** one only when none exists (first-time) or when
  **`--new-key`** forces a rotation. An existing key is **never replaced without `--new-key`**.
  Git config is create-if-absent / update-on-drift (an already-correct config is a no-op). An
  existing **SSH**-signing setup is left untouched unless `--new-key` is given.
- **`--dry-run`** previews; **`--upload`** publishes only the **public** key via `gh gpg-key add`.
  The private key never leaves the machine.

## House rules this honors

never-silent (G2) · no black boxes (every selection/refusal is printed/`--dry-run`-able) ·
no new dependency · small auditable tooling **above** the kernel (KC-3) · append-only docs ·
`scripts/checks/all.sh` green before each commit. The live Project-v2 GraphQL path is
`--dry-run`-validated, not yet Proven — its status is **Declared** until run on a `project`-scoped
machine (see `project-v2-spec.md`).
