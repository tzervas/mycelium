---
name: tero-cite
description: >-
  Get the resolvable citations for a mycelium-tero query — provenance only (anchor + file:line +
  the cited claim's guarantee tag), without the full item bodies. The provenance-first view of the
  transparent memory API (DN-87 / E39-1).
when_to_use: >-
  Use when you want to cite this project's decisions/issues/docs in an answer and need just the
  resolvable sources (to deep-link or to verify a claim in one hop), not the whole matched rows.
allowed-tools: Bash(curl:*), Read
---

# /tero-cite — Citations for a Query

`cite` runs the same query as `/tero-query` but returns **citations only** — the atomic unit of
provenance. Each citation is `{ anchor, id, family, kind, file, line, item_tag, guarantee_tag }`:
a stable `anchor` (deep-link/citation key), the `file:line` to Read, the row's extraction-honesty
`item_tag`, and — where the source declares one — the cited claim's own
`Exact/Proven/Empirical/Declared` guarantee. Provenance is **mandatory**: an uncited query is a
refusal, so `cite` never returns an empty-but-successful list (DN-87 §6.2).

## HTTP

```bash
curl -s -H 'Authorization: Bearer demo' 'http://127.0.0.1:8787/v1/cite?kind=id&value=ADR-042'
```

Envelope: `{"kind":"citations","citations":[{ "anchor":…, "file":…, "line":…, "guarantee_tag":… }, …]}`.

## MCP

`tools/call` with `{ "name": "cite", "arguments": { "kind": "id", "value": "ADR-042", "token":
"demo" } }`. The `kind` argument selects the query (`id`/`status`/`kind`/`cross_ref`/`text`); the
remaining args (`value`/`start`/`depth`) are the same as the query tools.

## Honesty

A citation resolves to exactly one Layer-1 row; the two tags are kept distinct (VR-5): `item_tag`
is how much to trust the row was captured correctly (uniform `Empirical/Declared` heuristic), while
`guarantee_tag` is the cited claim's *own* declared strength (`None` where the source declares none
— never invented). Source is always ground truth; use the citation to find where to Read.
