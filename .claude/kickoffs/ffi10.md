# Kickoff `ffi10` — FFI & System Interface (E14-1)

> Stowed kickoff, UID **`ffi10`**. Read `.claude/agent-context.md` + `CLAUDE.md` first.

## Metadata

| Field | Value |
|---|---|
| **UID** | ffi10 |
| **Head branch** | `claude/head/ffi10-ffi-system-interface` |
| **Status** | ready |
| **Swarm mode** | Sonnet |
| **Depends on** | *(none — can start after RFC-0028 reaches Accepted)* |

---

## Scope

Make the `wild`/`@std-sys` FFI floor — currently type-checked and capability-gated but with
execution staged as an explicit `Residual` (DN-14 row 9, M-661) — **execute for real**. This
unblocks every stdlib module that bottoms out in a syscall: `std.io`, `std.fs`, `std.sys`,
`std.rand`, `std.time`. The full-language 1.0.0 north star requires programs to do real I/O;
this epic is the necessary path.

**Epic issues:** E14-1 (parent epic), M-720, M-721, M-722, M-723, M-724.

---

## Grounding

- RFC-0028 (Draft) — FFI and system interface model; the capability model + `wild` execution
  host + ABI honesty stance. M-720 and M-721 depend on RFC-0028 reaching `Accepted`.
- ADR-014 (Accepted) — `unsafe` policy (permitted-but-warned, `// SAFETY:` required); the FFI
  surface is the primary site this ADR targets.
- RFC-0016 §8-Q6 — `std-sys` phylum split (the mechanism for confining OS-level surface).
- DN-14 row 9 — `wild`/FFI gate: `conditionally present (audited, std-sys context;
  type-checks + gates; execution staged)`. This epic closes the staged-execution gap.
- `crates/mycelium-mlir/src/jit.rs` — the only current `unsafe` site in the workspace (confined
  per DN-21/M-682); the FFI surface follows the same confinement pattern.
- `crates/mycelium-std-{io,fs,sys,rand,time}/src/` — existing partial/stub implementations
  that this epic makes real.

---

## Swarm / parallelization pattern

**Sonnet orchestrator + parallel leaf agents (one per issue), octopus-merge.**

Ownership split:

- **Orchestrator** owns: workspace `Cargo.toml`, `CHANGELOG.md`, `docs/Doc-Index.md`,
  `tools/github/issues.yaml`, `docs/adr/`, `docs/rfcs/RFC-0028-*.md` (once it advances),
  `scripts/checks/` (if `just safety-check` recipe is added per M-724).
- **Leaf M-720** owns: `crates/mycelium-l1/src/elab.rs` (the `wild` `Residual` to concrete
  dispatch change) + `crates/mycelium-std-sys/src/` (the FFI capability surface). Co-ownership
  note: `elab.rs` may be touched by other epics; co-ordinate via FLAG-up.
- **Leaf M-721** owns: `crates/mycelium-std-sys/src/` `wild` host execution shim — the Rust
  trampoline or capability handle that resolves `wild` blocks at runtime.
- **Leaf M-722** owns: `crates/mycelium-std-{io,fs,sys}/src/` syscall bindings —
  real `io`/`fs`/`sys` ops replacing stubs; each with a guarantee matrix row (`Declared`).
- **Leaf M-723** owns: `crates/mycelium-std-{rand,time}/src/` — time/clock + entropy real
  bindings hardened for 1.0; `Declared` guarantee tags; never-silent error paths.
- **Leaf M-724** owns: `scripts/checks/` and/or `justfile` (safety-check recipe) + the FFI
  audit sweep over all `@std-sys` nodules — the ADR-014 unsafe-floor confinement check.

Pre-flight: align head with `main` (`git fetch origin; git merge --ff-only origin/main`);
prune stale worktrees; push the head branch before spawning any leaf.

Swarm failure mitigations (per CLAUDE.md):

- ID-namespace check before assigning new M-ids (grep issues.yaml for M-7xx candidates).
- After octopus merge: validate `issues.yaml` with
  `python3 -c "import yaml; yaml.safe_load(open('tools/github/issues.yaml')); print('OK')"`.
- Merge the branch the leaf **reports**, not an assumed name; verify
  `git ls-tree --name-only <ref> crates/mycelium-std-sys/` is non-empty before merging.
- `just fmt` and `just check` via the justfile; never `python3 -m ruff` directly (PATH issue
  mitigation #3).

---

## Sequencing and dependencies

```
RFC-0028 Accepted ──┬─ M-720 (FFI surface / elab.rs wild dispatch) ──┐
                    └─ M-721 (wild host execution shim)  ──────────────┤
                                                                        ├─ M-722 (syscall: io/fs/sys)
                                                                        └─ M-723 (syscall: rand/time)
                                                                               └─ M-724 (safety-check audit)
                                                                                      └─ octopus merge
```

M-720 and M-721 must sequence M-720 before M-721 (the elab dispatch must exist before the host
shim resolves it). M-722 and M-723 can run in parallel once M-721 is done. M-724 (safety-check
audit) runs after M-720 through M-723 are merged, as a final gating audit before landing.

---

## Definition of Done

- [ ] `wild` blocks in `@std-sys` nodules execute for real: `elab.rs`'s `Residual` for `wild`
  is resolved to a concrete host dispatch; the three-way differential (L1-eval ≡ L0-interp ≡
  AOT) extends to cover at least one `wild`-backed operation.
- [ ] The capability model from RFC-0028 is implemented: a `wild` block outside `@std-sys`
  remains a hard `CheckError` (G2, unchanged from M-661); inside `@std-sys`, the host capability
  is granted and auditable.
- [ ] Real syscall bindings: `std.io` (read/write), `std.fs` (open/close/read/write/stat),
  `std.rand` (entropy source), `std.time` (clock read) — each with a guarantee matrix row
  (`Declared` for all syscall-backed ops in v0; VR-5).
- [ ] ADR-014 unsafe-floor confinement: every `wild` site in a `@std-sys` nodule carries a
  `// SAFETY:` comment; `just safety-check` verifies this mechanically and is green.
- [ ] Never-silent error paths: every syscall failure is an explicit `Result::Err` or
  `Option::None` — no silent discard (G2).
- [ ] `just check` green across all modified crates; guarantee tags honest per VR-5.
- [ ] Orchestrator-owned files reconciled: `CHANGELOG.md` entry, `issues.yaml` status updated
  (M-720 through M-724 done), RFC-0028 status note added, `docs/api-index/` regenerated.

---

## Landing

`/wave-land` from `claude/head/ffi10-ffi-system-interface` to `main` after:

1. `just check` green on the head.
2. Orchestrator self-review (`/pr-review` lens) — honesty/grounding/append-only pass.
3. `just safety-check` green (FFI audit, M-724).
4. Curated squash commit message (net change description, not WIP trail).
5. Propagate squashed `main` back to other heads via `scripts/sync-heads.sh`.
