---
name: tero-explain
description: >-
  Get the EXPLAIN trace for a mycelium-tero query — why these sources, in what order: the candidate
  count, the ordering rule(s) applied, a per-hit reason, and any cross-reference edges that could
  not be resolved. The auditability view of the transparent memory API (DN-87 / E39-1).
when_to_use: >-
  Use when you need to audit or debug a retrieval — understand why a query returned what it did (or
  ranked one hit above another), or which cross-reference edges were unresolved — rather than just
  consuming the answer.
allowed-tools: Bash(curl:*), Read
---

# /tero-explain — The EXPLAIN Trace for a Query

`explain` runs the same query as `/tero-query` but returns **only the EXPLAIN trace** — retrieval is
never a black box (DN-87 §4). The trace is a pure function of the query and the (already-sorted)
index: two runs produce byte-identical output (no clock, no rng).

Fields: `query` (the query as executed, including any depth clamp), `candidates_scanned`,
`candidates_matched` (pre-cap total, so a truncated result set is visible — never silently),
`order_by` (the ordering rule(s), outermost first), `hits` (per-result `anchor` + `score` + a `why`
reason, in final order), and `unresolved_edges` (for `cross_ref`: `depends_on`/`doc_refs` edges
considered but not resolvable within Layer 1 — recorded, never dropped, G2).

## HTTP

```bash
curl -s -H 'Authorization: Bearer demo' 'http://127.0.0.1:8787/v1/explain?kind=text&value=eval%20gate'
curl -s -H 'Authorization: Bearer demo' 'http://127.0.0.1:8787/v1/explain?kind=cross_ref&start=M-1017&depth=2'
```

Envelope: `{"kind":"explain","explain":{ "query":…, "candidates_scanned":…, "order_by":[…], "hits":[…], "unresolved_edges":[…] }}`.

## MCP

`tools/call` with `{ "name": "explain", "arguments": { "kind": "text", "value": "eval gate",
"token": "demo" } }`.

## Honesty

The text ranking weighting is stated verbatim in `order_by` (id match ×4 + title ×3 + summary ×1
per matched term). A `cross_ref` walk is clamped to a hard depth cap; the clamp, if applied, is
recorded in the `query` string (never silent). Unresolved `api:`/`src:` doc_refs and dangling
`depends_on` ids are listed, not hidden — the trace is the honest account of what the engine did.
