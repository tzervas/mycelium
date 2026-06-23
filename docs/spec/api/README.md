# Public-API snapshots

Committed snapshots of each crate's **public API surface**, one `*.txt` per workspace crate,
produced by [`cargo public-api`](https://github.com/enselic/cargo-public-api).

`scripts/checks/api.sh` (run by `just api`, part of `just check`) diffs the live surface against
these snapshots and **fails on an unreviewed change** — so an accidental `pub` item, or a widened
surface, is caught in review rather than shipped. This is a guardrail for **KC-3** (keep the kernel
small and auditable) and supports the move to private kernel-type fields (review finding A2-05).

## Bootstrapping / updating

The snapshots are generated on demand (the tool is optional and drives a nightly rustdoc, used
only to introspect the surface — it does not change the MSRV-pinned build). To create or refresh
them after an **intended** API change:

```sh
just setup          # installs cargo-public-api (best-effort) if missing
just api-baseline   # writes docs/spec/api/<crate>.txt for every crate
git add docs/spec/api && git diff --cached   # review the surface delta, then commit
```

Until the snapshots exist (or when `cargo-public-api` is absent), `just api` **skips gracefully**,
consistent with the repo's other optional checks.

## Current state (2026-06-23)

The baseline was last regenerated 2026-06-23 (E11-1 / E12-1 wave). Notable additions:

- **`mycelium-l1.txt`** — gained the `op_expr` / `InfixOp` / `UnaryOp` AST nodes and the operator
  desugaring table (RFC-0025 / M-705).
- **`mycelium-std-runtime.txt`** — gained the `scheduler::Scheduler` OS-thread pool, the
  `dataflow::run_dataflow_scheduled` + `DEADLOCK_DETECTION_STRENGTH` entries, and the
  `supervision::CancelTree` / `run_supervised` / `supervise_with_restart` surface (M-709/M-711/M-713).

## Changelog

- **2026-06-23 — Baseline regenerated (E11-1/E12-1).** `mycelium-l1` and `mycelium-std-runtime`
  snapshots updated to include the operator-syntax AST (RFC-0025/M-705) and the real scheduler +
  deadlock-freedom + supervision API (M-709/M-711/M-713). Append-only.
