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
**project-field options are reconciled additively** — a missing option on an existing single-select
field is *added* via `updateProjectV2Field`, name-matched (existing options keep their id + item
assignments; new options are appended; **no option is ever deleted**, because the reconcile always
sends the *union* of the field's live options ∪ the manifest's options). A live option that is not
in `project.json` is a would-be deletion — it is **kept in place and FLAGGED** (remove it in the UI
if intended), never silently dropped (G2). Sensitive/creative state — a **GPG key** — is **never**
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
- **`--project`** — find-or-create the **Mycelium** board; create absent fields (with all their
  options) and **additively add any missing options to existing fields** (`updateProjectV2Field`,
  name-matched, union-of-live-∪-manifest, never-deleting — see *Non-destructive by default* above),
  so a maintainer adds a new option (e.g. a new `area:*` value) to `project.json` and it appears on
  the board on the next reconcile; add repo issues/PRs as items; set **Status/Phase/Area/Priority**
  from each item's labels. **Views + built-in workflows are settings-only** → recorded in
  `project.json` and **FLAGGED as manual steps**. See [`project-v2-spec.md`](project-v2-spec.md).

> **Everything is create/add-if-absent from the manifests.** Labels (`labels.json`,
> create-or-update via `gh label --force`), milestones (`milestones.json`, create-absent by title),
> project **fields** (`project.json`, create-absent) and their **options** (`project.json`,
> add-absent additively) all flow from the declarative manifests — edit the manifest, re-run, and
> the missing thing is created/added (never deleted, never duplicated, in-sync ⇒ zero writes).

`--all` = the **full maintenance suite**, in order:
**preflight → validate → labels → milestones → issues → PRs → project**.

## Bounded-concurrency, rate-negotiated, batched execution (M-397)

The live reconcile issues many small, **independent, idempotent** `gh` mutations per run (one per
label, one per drifted issue, …). They are subprocess/IO-bound, so the engine overlaps their latency
with a **bounded thread pool over the existing synchronous `gh()`** (threads, **not** an asyncio
rewrite) — `gh()`/`_run_gh` keep their M-382 retry/backoff/120s-timeout verbatim. This maximizes sync
speed while staying inside GitHub's **secondary** rate limits, and it is never-silent (G2) and
fault-tolerant. The per-batch summary and the collected failure FLAGs are reported in **submission
order** (deterministic); only the *live* per-item progress lines print in completion order, guarded by
a process-wide lock so concurrent / `--verbose` output never interleaves.

| Knob | Default | Meaning |
|---|---|---|
| `--concurrency N` | **6** | max in-flight `gh` calls per batch (conservative for secondary limits). |
| `N=1` | — | reproduces the **exact sequential behaviour** — each task runs inline, in submission order, no executor. The clean `--verbose`/debug fallback. |
| `--no-rate-probe` | off | skip the start-of-run `gh api rate_limit` budget probe (which can reduce `N` when the remaining core budget is low). |
| `--dry-run` | — | stays **sequential** (N=1) for a stable, ordered preview; mutates nothing. |

**Batches (cross-batch dependency order preserved).** Each independent per-item loop dispatches as a
batch with `as_completed` aggregation: label create-or-update · milestone create · **issue create
(pass 1)** · **per-issue field/label/milestone updates (pass 2)** · PR backfill · noncompliant-label
**migration relabels**. Ordering invariants hold: **labels + milestones before issue creation**, and
**create-pass-1 fully aggregates before the update pass** (so a future `depends_on`/sub-issue linking
pass would see every new number). Only items that are **idempotent + target disjoint resources** run
in parallel within a batch.

**Rate negotiation (three layers).** (a) bounded concurrency; (b) a shared `RateGate` — on a
`403`/`429`/`secondary rate limit`/`abuse`/`Retry-After` stderr the **whole pool PAUSES** for the
advised window (parsed by the pure, `--self-test`'d `should_pause_for_rate_limit`), then resumes with
one post-pause retry — never a continued burst; (c) an optional `gh api rate_limit` probe that reduces
`N` when the remaining core budget is low (`negotiate_concurrency`, never-silent). A **primary**
rate-limit (`API rate limit exceeded`) is *not* absorbed into a short pause — it resets on a fixed
hourly window, so `_gh_fail` surfaces it honestly with the `gh api -i rate_limit` remediation.

**Never-silent output.** Each task returns `(item, ok, err)`; a batch prints `>> <batch>: N ok, M
failed` and every failure as a re-runnable FLAG (one failure never aborts the batch). All printing is
guarded by a process-wide lock (`safe_print`/`_safe_stderr`) so concurrent / `--verbose` lines never
interleave.

> **Honesty (Declared).** GitHub's exact secondary-rate-limit / abuse thresholds and concurrency
> tolerances are **not publicly specified** and were **not** exercised against the live API here, so
> the tuning (default `N=6`, pause-the-pool + single retry, `low_water=100`, `default_backoff=60s`,
> `max_backoff=300s`) is **Declared** — conservative defaults, never claimed Proven. The pure decision
> logic is `--self-test`-covered offline; the live concurrency behaviour should be confirmed with a
> `--dry-run` and a small live run before trusting a high `--concurrency`.

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

## Issue↔PR relationship + date manifest (`--relationships`) — extraction & enrichment

The reconciler can derive, **offline + idempotently**, the per-issue **relationship/date manifest**
— which PR landed (or referenced) each `M-xxx`/`E#-#`, on what date, plus the epic edge — from the
two **in-repo dated sources of truth**, and enrich `issues.yaml` **additively** with it:

> Preview:  `python tools/github/gh-issues-sync.py --relationships --dry-run`
> Apply:    `python tools/github/gh-issues-sync.py --relationships`
> Live PR cross-check (needs a token):  add `--use-api` (reads `GITHUB_TOKEN`/`GH_TOKEN`).

### The two evidence sources (the source of truth)

| Source | Yields | Parser |
|---|---|---|
| `CHANGELOG.md` dated headers `### Kind (YYYY-MM-DD: M-xxx — …)` | the **date** (preferred) + a basis | `parse_changelog_landings` |
| `git log <ref> --format='%H\|%ad\|%s'` squash subjects | the landing **SHA** + `(#NNN)` **PR** | `parse_git_log_landings` |
| `pr-index.json` (`{task_id: pr}`, derived from the **live merged-PR list**) | a **PR** cross-check when the curated subject omits `(#NNN)` | `load_pr_index` / `api_merged_pr_index` |

A slash-run in either source (`M-656/657/658`) is expanded to its members (`expand_task_id_run`);
a `(#344)` is a **PR number**, never conflated with a task-id. Newest mention wins (both sources are
newest-first). The merged-PR list was enumerated via the GitHub REST API (the agent run used the
GitHub MCP `list_pull_requests`; `--use-api` reproduces it over a token).

### Status-aware honesty (the crux — VR-5/G2)

A strong **`landed_pr` / `landed_date`** claim ("this issue's work landed in this PR on this date")
is asserted **only for a `status:done` issue**. For an issue that is **not** done (in-progress /
blocked / needs-design) the *same* evidence is real but a **weaker** claim — the id was merely
**referenced** by that PR/CHANGELOG entry (a partial tranche, a filing commit, an aspirational
title) — so it is recorded under **`evidence_pr` / `evidence_date`** with a `landed_basis` note that
**says so**. This refuses to overclaim completion. Every emitted field carries a **`landed_basis`**
string citing the exact CHANGELOG header line and/or the PR cross-check, so the date/PR is auditable
and stays `Empirical`/`Declared` (never `Proven`). An issue with **no** grounded evidence gets **no**
field (never a null-filled guess); existing fields are **never** overwritten (append-only).

The fields added per issue (all additive, all optional): `landed_pr`/`landed_date` **or**
`evidence_pr`/`evidence_date` (mutually exclusive by status), `landed_basis`, and `epic` (for an
`E#-#` id). The write is a **surgical text insertion** after each entry's `depends_on:` line — it
does **not** reformat the file (comments/order/round-trip preserved); the result is re-parsed before
it is written (never leave `issues.yaml` broken). Re-running is **zero-write** (idempotent).

### Live-sync targets this manifest FEEDS — token/Projects-API-gated **FLAGs**

The offline run **produces + validates** the manifest; it does **not** push these live GitHub
relationships (they need a token this environment lacks, and — for the board — a Projects-v2
mutation surface the MCP tool-set does not expose):

- **Issue dependencies** (`depends_on` → GitHub "blocked by"): REST, **needs a token** (`--use-api`,
  `GITHUB_TOKEN`).
- **Sub-issue links** (epic → children): REST, **needs a token** (or the GitHub MCP
  `sub_issue_write`, which takes the issue **db-id** from `idmap.tsv`, not the number).
- **Projects-v2 field values** (Status / Start date / Target/landed date): **GraphQL**, needs a
  **`project`-scoped** token; the request construction (`GitHubApi.graphql`) is unit-tested offline,
  but the live round-trip is **Declared until run** (see `project-v2-spec.md`). The MCP tool-set in
  the agent sandbox has **no Projects-v2 mutation tool**, so the board population is the
  **maintainer's token-scoped step** — never faked here (G2).

### What is verified vs. gated (capability honesty)

- **Verified offline** (this run): the extraction + status-aware manifest + additive enrichment are
  covered by `--self-test` (pure-logic) and applied to `issues.yaml`; `manifest-check.py` +
  `doc_refs_check.py` + the YAML load stay green.
- **Script-ready but gated**: the live REST/GraphQL push of dependencies, sub-issue links, and
  Projects-v2 fields — wired over `urllib`+token (`GitHubApi`, `--use-api`), **Declared** until run
  with a token (none in this environment). `pr-index.json` lets the offline cross-check reproduce the
  live merged-PR numbers without a network call.

### gh-CLI-independent token path (`--use-api`)

`GitHubApi` (stdlib `urllib`, no new dependency) gives the same REST + GraphQL capability **without
the `gh` CLI** (some runners/sandboxes have only a token), reading `GITHUB_TOKEN`/`GH_TOKEN`. It is
**opt-in** (the default path is still `gh`-driven), **never-silent** (a non-2xx raises with the
status + body), and **`--dry-run`-aware** (mutating verbs preview, reads always run). A missing token
is the **honest stop** — it FLAGs the gated step and falls back to `pr-index.json`, **never** a
fabricated sync.

## Reconciling erroneous duplicate issues (`gh-issues-dedup.py`)

The reconciler matches by idmap-number-then-title and is designed to **never duplicate**. When that
contract slips anyway — a sync re-run before `idmap.tsv` was committed once created twins (#126-129
duplicated M-302/M-330/M-342/M-348) — `gh-issues-dedup.py` is the safety net:

> Preview:  `python tools/github/gh-issues-dedup.py`
> Act:      `python tools/github/gh-issues-dedup.py --apply [--fix-idmap]`
> Offline:  `python tools/github/gh-issues-dedup.py --self-test`

- Groups all issues by the **task-id in the title** (`M-xxx` / `Exx-x`) and by **normalized title**;
  the **lowest-numbered** (earliest) issue in a set is the **canonical**.
- **DRY-RUN by default** — prints every duplicate set (canonical + twins) and what `--apply` would do.
- `--apply` closes only the **non-canonical OPEN task-id** twins (commented to point at the canonical),
  **never the canonical**; a CLOSED canonical or a **title-only** match is reported for **manual**
  review, never auto-closed (G2 — no guessing).
- `--fix-idmap` re-anchors any `idmap.tsv` row that points at a non-canonical number.
- Same contract as the reconciler: no new dependency (the `gh` CLI), never-silent, offline `--self-test`.
