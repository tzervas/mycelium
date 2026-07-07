---
name: tero-refresh
description: >-
  Reload the mycelium-tero memory API's served index from disk after the committed
  docs/tero-index/index.json has been regenerated — the freshness operation of the transparent
  memory API (DN-87 / E39-1). Requires the broader `refresh` token scope (not read-only).
when_to_use: >-
  Use when the corpus index has been regenerated (just tero-index) and you want a running tero-http
  / tero-mcp server to pick up the new rows without a restart. Requires a token with the `refresh`
  scope.
allowed-tools: Bash(curl:*), Bash(just:*), Read
---

# /tero-refresh — Reload the Served Index

The fronts serve a snapshot of `docs/tero-index/index.json` loaded at startup. `refresh` reloads it
from disk in place — for use after the drift-gated index is regenerated:

```bash
just tero-index          # regenerate docs/tero-index/{INDEX.md,index.json} (deterministic)
```

`refresh` is the one operation that is **not** read-only, so it requires a token with the `refresh`
scope (`read`-only tokens get a `403` / JSON-RPC `-32002`). Read-only stays the default (DN-87 §6.4).

## HTTP

```bash
curl -s -X POST -H 'Authorization: Bearer admin' http://127.0.0.1:8787/v1/refresh
```

Success: `{"kind":"refreshed","ok":true,"items":<count>}`. If the on-disk index is missing/unreadable
the server returns a `500` with a JSON error envelope — never a silent stale-serve (G2).

## MCP

`tools/call` with `{ "name": "refresh", "arguments": { "token": "admin" } }` (a `refresh`-scoped
token).

## Honesty / freshness

Freshness is drift-gated like the indices tero generalizes: the encoding regenerates
deterministically from the corpus (`scripts/checks/tero-index.sh` fails on drift). `refresh` never
fabricates or partially-applies — it swaps in a fully-loaded new snapshot or fails loudly, leaving
the previous snapshot intact.
