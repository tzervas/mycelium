# tero trim-pass — handoff state (2026-07-10)

Branch: `claude/chore-extract-tero-trim` (based off `origin/dev` @ `6e9869b6`). Not yet PR'd — this
file is the durable handoff for a fresh session to resume from exactly here.

## (a) Footprint map — where tero touches mycelium

| Location | What it is | Classification |
|---|---|---|
| `crates/mycelium-tero/` | Rust crate (DN-87/E39-1, M-1015…M-1018) — `tero-index`/`tero-http`/`tero-mcp`/`tero-eval` bins, ~8.2k LOC. Workspace member; recompiled by `just check` and `scripts/checks/tero-index.sh` on every run. | **Genuinely-redundant dev-source** — extracted verbatim into `tzervas/tero-rs` (private repo, crate renamed `mycelium-tero`→`tero`). Confirmed by upstream's own v0.1.3 release notes ("binary behavior unchanged from v0.1.1 — packaging/extraction release"). **REMOVED** (see below). |
| `scripts/checks/tero-index.sh` + `justfile`'s `tero-index`/`tero-index-gen` | The drift gate + regen recipe; the ONLY real build-wiring consumer of the crate (only `cargo run -p mycelium-tero --bin tero-index` reference in any script/justfile). | **Must keep working — rewired**, not removed. Now runs a checksum-pinned, cached, published `tero-index` binary instead of recompiling. |
| `xtask/deps-strata.toml` | Dependency-stratum/tier map (`deps-acyclic.sh` gate input) — had `mycelium-tero = 7` / `= "tools"` entries. | Housekeeping — entries removed alongside the crate (else the acyclic-deps checker would report a dangling/absent-crate mismatch). |
| Workspace `tokio`/`axum` deps in root `Cargo.toml` | Added at M-1017 *solely* for `mycelium-tero`'s HTTP front — grepped, no other consumer in the workspace. | Removed alongside the crate (dead weight otherwise; re-add per ADR-044 if a future in-tree crate needs async). |
| `packages/tero-mcp-lite/` | Vendored, zero-dependency Python MCP server (no compile step) — the `.mcp.json` `tero` server runs this directly (`uv run --project packages/tero-mcp-lite …`). | **NOT dev-source** — an intentional *consumed* snapshot (Python, no recompile cost). Per maintainer: fine to keep vendored as-is. **NOT TOUCHED.** (Note: `ghcr.io/tzervas/tero-mcp:0.1.1` — the published "tero-mcp" package — is literally the `tero_mcp_lite` wheel; a sibling, already-gated, NOT-yet-applied in-place source upgrade to 0.1.1 sits at `packages/_staging/tooling-v0.1.1/` on a *different*, not-yet-merged branch (commit `6ebb8dbf`, not on `dev`) — out of scope here, flagged as a related follow-up.) |
| `.mcp.json` | Registers the `tero` MCP server, pointing at `packages/tero-mcp-lite` (NOT the Rust crate — never depended on `crates/mycelium-tero` at all). | **Already correct / no rewire needed.** |
| `docs/tero-index/{INDEX.md,index.json}` | Committed corpus index — generated content, not tero *source*. | Unaffected by the crate removal (verified: regenerating post-removal via the published binary produces byte-identical output to what was committed pre-removal — the index doesn't embed rust source, only doc/corpus content). |
| `packages/tero-mcp-lite.zip` | A committed zip of the *entire* `packages/tero-mcp-lite/` source tree, referenced intentionally by `packages/GROK-HANDOFF.md` as a cross-platform handoff artifact for a different agent (Grok). | **FLAGGED, not touched.** Looks like stale cruft given the standing "no Grok API" directive ([[sanctioned-paths-blocked-ops]] memory), but it has documented intentional purpose in-repo — removing it isn't unambiguously safe without maintainer confirmation the Grok-handoff path is dead. Out of this task's core scope (tero's *build* footprint, not a Grok-handoff artifact). |

## (b) Safe-to-trim vs must-keep-and-rewire — the classification rule applied

- **Safe to trim:** `crates/mycelium-tero/` (dev-source, verbatim-duplicated upstream, verified via
  differential before removal — see below).
- **Must keep working, rewired:** `scripts/checks/tero-index.sh` + `just tero-index-gen` (the gate
  mycelium's own docs depend on) — now consumes a pinned, checksum-verified **GitHub Release asset**
  (not a container image, not GHCR-OCI — matches the maintainer's publish-target correction) via `gh
  release download`.
- **Kept as intentional vendored snapshot, untouched:** `packages/tero-mcp-lite/` + `.mcp.json`.
- **Flagged, not executed:** the `packages/tero-mcp-lite.zip` / Grok-handoff question (separate
  concern); the private-repo `gh`-auth prerequisite the rewire introduces (see "FLAG" below).

## (c) What's already done (green, verified)

All committed to this branch already:

1. **Verification differential (Empirical, actually run) BEFORE removing anything:**
   `cargo run -p mycelium-tero --bin tero-index` (in-tree, pre-removal) vs. the downloaded +
   checksum-verified `tero-index-v0.1.3-linux-x86_64` (from `gh release download v0.1.3 -R
   tzervas/tero-rs`, sha256 `8591905f…9e8`, matches the release's own `SHA256SUMS.txt`) vs. the
   already-committed `docs/tero-index/` — **all three byte-identical** (`diff -rq` exit 0 on every
   pair, both `index.json` and `INDEX.md`). Recorded in `tools/tero-rs/PROVENANCE.md`.
2. **`crates/mycelium-tero/` removed** (`git rm -r`) — workspace member entry, the tokio/axum
   workspace deps (M-1017/ADR-044, no other consumer), and the `xtask/deps-strata.toml`
   stratum/tier entries all removed alongside it.
3. **`scripts/fetch-tero-index.sh` added** — resolves a checksum-verified, locally-cached
   `tero-index` binary; on a cache miss, fetches the pinned **GitHub Release asset** via `gh release
   download` (the repo `tzervas/tero-rs` is private, so this needs an authenticated `gh`). Prints the
   resolved binary's absolute path on stdout; all diagnostics to stderr; non-zero exit = unresolved
   (caller's choice: skip vs fail).
4. **`scripts/checks/tero-index.sh` + `justfile`'s `tero-index-gen` rewired** to call
   `fetch-tero-index.sh` instead of `cargo run -p mycelium-tero`. Gate stays **skip-graceful** (no
   cached-or-fetchable binary ⇒ skip, never a false-red — matches the "skip if cargo absent"
   convention every other optional-tool gate in this repo already uses).
5. **`tools/tero-rs/PROVENANCE.md` + `SHA256SUMS.txt`** committed — the hash-pinned, honest record
   of the consumed artifact (source repo/commit/tag, the differential, the bump policy: re-verify
   the differential before ever bumping the pin).
6. **Gates run green post-removal:**
   - `cargo build --workspace` — clean.
   - `just tero-index` (the rewired gate, cache-hit path) — `ok docs/tero-index/ is current`.
   - `just tero-index-gen` (regenerated fresh) — same row/flag counts (4950 rows, 6 flagged),
     `docs/tero-index/` unchanged (confirms the index doesn't embed rust source).
   - `scripts/checks/deps-acyclic.sh` — `ok no dependency-structure violations`.
   - `just docs-index` (api-index regen, since `mycelium-tero`'s public API disappears) — clean,
     2614 items indexed.
   - `python3 tools/github/doc_refs_check.py` — clean (one dangling `M-1015` `src:` ref to the
     removed crate repointed to `tools/tero-rs/PROVENANCE.md`; a `TRIM (2026-07-10, …)` append-only
     note added to M-1015's body, per house rule #3 — no existing DONE text rewritten).
   - `python3 -c "import yaml; yaml.safe_load(...)"` on `issues.yaml` — OK.
   - `bash scripts/checks/markdown.sh` — 477 docs clean.
   - `bash scripts/checks/secrets.sh` (gitleaks) — no leaks.
   - `bash scripts/checks/branch-guard.sh` — ok, on the working branch.
   - `bash scripts/checks/structured.sh` — ok.
   - `CHANGELOG.md` — a new `[Unreleased]` entry added (append-only; existing entries untouched).
   - `docs/Doc-Index.md` — the tero-index regen instructions updated to describe the new mechanism
     (this is *operational* doc content, not a historical record, so it was edited in place rather
     than appended-to).
7. **`just check-canary` was KICKED OFF but not confirmed complete** before this handoff — it was
   still running (backgrounded by the harness; output file was still empty after several minutes) at
   stop time. **This is the first thing the fresh session should check/re-run and confirm green**
   before opening the PR.

## (d) What's left

1. **Confirm `just check-canary` (or `just check`) is green** — re-run if the backgrounded run from
   this session didn't finish/wasn't captured.
2. **Open the PR into `dev`** — title `chore(extract): trim vendored tero dev-source; wire to
   published tero`, body summarizing the footprint map / classification / verification / FLAGs (this
   file's content is the source for that body). Use `gh pr create` (repo-scoped auth already
   confirmed working — `gh auth status` shows a logged-in, `repo`-scoped token; downloading the
   private `tero-rs` release asset already succeeded with it).
3. **Self-review the diff** with the `/pr-review` lens before/alongside opening the PR (append-only,
   grounding, no black boxes) — not yet done as a dedicated pass, though the gates above cover most
   of the substance.
4. **Land via `/pr-land`** once green + reviewed (leaf/chore branch → `dev`, PR-gated, `--no-ff`,
   per CLAUDE.md's tiered-branch rule — `dev` is protected, no direct push).

## (e) FLAGs for the maintainer (not resolved here — judgment calls)

1. **The private-repo prerequisite.** Fetching a *fresh* (uncached) `tero-index` binary now requires
   an authenticated `gh` CLI with read access to `tzervas/tero-rs` (private). The prior
   self-contained `cargo run -p mycelium-tero` needed neither network nor auth. Options: (i) make
   `tero-rs` public, (ii) vendor/commit the binary (a release asset checked in, or `git-lfs`), (iii)
   accept the `gh`-auth prerequisite as the standing cost — current default, and it degrades
   gracefully (skip, not fail) when unmet, so it never blocks `just check` in an environment without
   that auth, it just stops verifying the index is current.
2. **`packages/tero-mcp-lite.zip`** — see the footprint table above; possibly stale Grok-handoff
   cruft, not removed pending confirmation.
3. **The sibling `packages/_staging/tooling-v0.1.1/` adoption-prep package** (commit `6ebb8dbf`, on a
   *different*, not-yet-merged-to-`dev` branch) proposes an in-place upgrade of the vendored
   `packages/tero-mcp-lite/` to a differential-verified 0.1.1 (9/9 byte-identical query answers). Not
   touched by this trim pass (different branch, different concern) — noted here so the fresh session
   doesn't rediscover it from scratch, and because the published `ghcr.io/tzervas/tero-mcp:0.1.1` OCI
   artifact was confirmed (via `oras manifest fetch`) to literally BE the `tero_mcp_lite` wheel+sdist,
   i.e. that staged adoption is consuming the same published tool this trim pass is about.

## Verification commands (for the fresh session to re-run/spot-check)

```bash
git log --oneline -3                          # confirm branch state
just tero-index                                # rewired gate, cache-hit path
just tero-index-gen                            # full regen via published binary
scripts/checks/deps-acyclic.sh                 # xtask deps-strata sanity
python3 tools/github/doc_refs_check.py         # no dangling refs
cargo build --workspace                        # crate removal didn't break anything
```
