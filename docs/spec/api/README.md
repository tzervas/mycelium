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
