# Kickoff: gap-close-run

> Instigator for a fresh session resuming the autonomous **zero-hand-port gap-closure program**
> (+ the parallel workspace dev/AI-tools fleet). Reloads clean CLAUDE.md + updated tool-use policies.
> On start: read the memory index `MEMORY.md`, then the linked memories below, then drive.

## First actions on kickoff
1. **Read the durable state** (all in `/root/.claude/projects/-root-git-isolated-mycelium/memory/`):
   - `session-program-state.md` — LIVE what's DONE / IN-FLIGHT / QUEUED + trunk tips (verify with `gh api` — real GitHub, not local).
   - `kernel-unfrozen-zero-handport.md` — the design foundation: all-layers-writable, sugar-transparency, gap-closure-via-mechanically-lowering-sugar, deliberate-exclusions-have-native-solutions, **sugars-are-largely-MACROS**, structural-remapping-as-separate-pass-iteratively-automated (DN-109 §7-d).
   - `workspace-tools-publish-contract.md` — the fleet contract (package-via-oras publish NOT containers; SDK-vs-in-house; consume-packaged; scope = dev/AI tools).
   - `sanctioned-paths-blocked-ops.md` (blocked-op paths, storage/RAM hygiene, sccache), `local-swarm-rag-docs-vision.md`, `delegate-nested-swarms-for-efficiency.md`.
2. **Verify trunk tips + any in-flight agents' landings** via `gh api repos/tzervas/mycelium/...` before acting.

## The program (drive order)
- **Design foundation is RATIFIED + on `main`** (DN-101–109 + Zero-Hand-Port Delta Ledger + DN-109). So the design is locked; this run is the IMPLEMENTATION + fleet.
- **A. mycelium zero-hand-port** (the core mission):
  1. **Phase-1 force-multipliers** (highest leverage now): **M-1041** DRY `ExprVisitor`/fold (kills the ~13-site tax on every new construct) + **M-1042/M-1044** structured transpiler output + remap-manifest (fix flat-emit). The **macro system** (DN-100/M-1032/M-875) is a PRIMARY sugar/gap-closure vehicle — prioritize it alongside.
  2. **L1 surface gap-closure** by blast-radius (Delta Ledger): cross-nodule symbol table → records → external-trait impls → bounded generics → signed/platform-width → transcendentals/format-string — mostly "add a mechanically-lowering sugar (often a macro)" or "map exclusion's problem → native solution + suggested_idiom".
  3. **Phase-0 re-measure** `checked_fraction` (last honest ~7.8%, PRE-enb-wave — likely much higher now).
  4. Open design items: **DN-102** second research pass (running / M-1049), **DN-101** B-vs-C API (M-1048), **DN-104** `pub(path)` (M-1050/M-1036).
  5. Endgame: full self-host → dogfood via transpiler → rip through nodule-by-nodule, phylum-by-phylum (differential-witnessed).
- **B. workspace dev/AI-tools fleet** (parallel, RAM-sized waves): Wave-1 rest (webpuppet-rs → its `-mcp`, context/security-mcp on the rmcp recipe), Wave-2. **Publish via GitHub RELEASES** (`gh release` + the built package as a sha256'd asset — `.whl`/`.tar.gz` from **`uv build`** for Python; `.crate`/binary for Rust — anchored to a `vX.Y.Z` tag), NOT container images, NOT crates.io/PyPI; GHCR-OCI-via-`oras` is optional-only. **DONE:** tero-rs v0.1.3, tero-mcp v0.1.1, agent-mcp v0.2.0 (the first two/three went out as *container* images under an earlier reading — re-publish as Release package artifacts as a tidy-up); GPU stack + `tools/model-router` live.
- **C. Consume-packaged pattern:** mycelium CONSUMES the published tools (no recompiling); stable-extracted tools get a **trim pass** (chore PR) removing their dev-source from mycelium + wiring to the packaged tool; future tool changes = issues on the tool repos.

## Operating pattern (updated policies)
- **You are the ORCHESTRATOR** — decompose, delegate to swarms (Haiku mechanical / Sonnet judgment / Opus deep design), reconcile, land. Agents may spawn sub-swarms.
- **Autonomous** publish/merge/land on the **NON-DESTRUCTIVE path** (fresh tags, PR merges, `oras`/`gh release`); the updated `permissions.allow` now cover the tool-repo destructive/admin ops (tag re-point/delete, branch delete, `gh repo edit`) so agents STOP hitting walls — **but mycelium `main`/`integration`/`dev`/`claude/head/*` protected-branch + no-force guards REMAIN** (branch-guard hook). Main squash-only via PR.
- **Blocked-op protocol (brief EVERY spawned agent with this):** when an agent hits a permission wall / PreToolUse hook block, it does NOT wall-bash/retry-loop/fabricate. It (1) uses the sanctioned alternative (PR not direct-push; non-destructive not force; split compound commands), or (2) **pings the orchestrator via `SendMessage(to: "main")`** requesting the exact auth/guidance, keeps doing other non-blocked work, and flags it — never stalls silently. Full protocol: [[sanctioned-paths-blocked-ops]]. (Bake this into the `.claude/agents/*.md` personas + CLAUDE.md as an early task.)
- **Verify-don't-fabricate** (VR-5/G2): confirm every publish/landing against ground truth; a verified partial beats a fabricated whole. Persist state to memory before compaction.
