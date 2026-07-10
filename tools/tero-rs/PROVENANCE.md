# tero-rs — extraction provenance + consumed-artifact pin

**Status: adopted.** `crates/mycelium-tero` (DN-87 / E39-1, M-1015…M-1018) was extracted verbatim
into its own repo, `tzervas/tero-rs` (private), renamed `mycelium-tero` → `tero`, and published as
binaries + a container image. Mycelium now **consumes** the published `tero-index` binary instead of
recompiling the crate in-tree — this file is the hash-pinned, honest record of that consumption
(the same pattern as `packages/_staging/tooling-v0.1.1/PROVENANCE.md` for `tero-mcp`).

## Verification performed (Empirical, actually run — 2026-07-10)

Before removing `crates/mycelium-tero` from the workspace, a three-way differential confirmed the
published binary is a safe drop-in for the in-tree generator, run against this repo's own corpus:

1. `cargo run -p mycelium-tero --bin tero-index -- --repo-root . --out /tmp/tero-out-intree`
   (the in-tree crate, pre-removal) — 4950 rows indexed, 6 flagged.
2. The downloaded, checksum-verified `tero-index-v0.1.3-linux-x86_64` binary (below) run over the
   same repo root — 4950 rows indexed, 6 flagged.
3. `diff -rq` between (1), (2), and the already-committed `docs/tero-index/` — **all three
   byte-identical** (`index.json` and `INDEX.md`, `diff -q` exit 0 on every pair).

This is the DN-87 "alternate front returns byte-identical answers" requirement, satisfied for the
index generator specifically (the query/MCP/HTTP fronts are unaffected by this trim — `.mcp.json`
still runs the vendored `packages/tero-mcp-lite/` Python server, unchanged).

## Pinned version

- Source: `tzervas/tero-rs` (private), tag `v0.1.3`, commit `75dd27f967f9eefa0987fa9d11a04beb0d9b0793`.
- Release: <https://github.com/tzervas/tero-rs/releases/tag/v0.1.3> — "Binary behavior is unchanged
  from v0.1.1 — this is a packaging/extraction release" (upstream release notes; the crate itself was
  not modified between the mycelium extraction and this tag, only the workspace around it was pruned
  57→17 crates and the crate renamed `mycelium-tero`→`tero`).
- Hashes: `SHA256SUMS.txt` in this directory (copied verbatim from the release asset of the same
  name; `sha256sum -c` verified against the downloaded `tero-index` binary during this adoption).
- Container (not used by this repo's tooling, recorded for completeness): `ghcr.io/tzervas/tero-rs:0.1.3`.

## Consumption mechanism

`scripts/checks/tero-index.sh` and `just tero-index-gen` fetch-and-cache the pinned `tero-index`
binary via `scripts/fetch-tero-index.sh` (checksum-verified against `SHA256SUMS.txt` above, cached
under `${MYCELIUM_TERO_CACHE:-$HOME/.cache/mycelium/tero-rs}`, never committed). The repo is
**private**, so fetching requires an authenticated `gh` (`gh release download`); this is a new
prerequisite the prior self-contained `cargo run -p mycelium-tero` did not have — **flagged**, see
the trim-pass PR description for the tradeoff and the maintainer decision it's gated on (make the
repo public, vendor the binary, or accept the `gh`-auth prerequisite as-is).

## Bump policy

Bumping the pin (a new tero-rs release) is a **source-repo change, not a mycelium change** per the
maintainer's direction that "future tool changes happen in the TOOL repos via issues." Bumping here
means: download the new release's `SHA256SUMS.txt` + binary, re-run the three-way differential above
against the *currently committed* `docs/tero-index/`, and update this file + `SHA256SUMS.txt`
together in one commit once the differential is clean (never bump the pin without re-verifying).
