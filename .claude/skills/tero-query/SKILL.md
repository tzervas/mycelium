---
name: tero-query
description: >-
  Query the mycelium-tero transparent memory API (DN-87 / E39-1) for cited answers about this
  project's decisions, issues, docs, changelog, and skills — over MCP or HTTP, from any agent
  platform. Every answer carries resolvable citations; a query that finds nothing citable is a
  typed refusal, not an empty answer.
when_to_use: >-
  Use when you need a cross-cutting answer about the corpus (a decision by id, all items of a
  status/kind, a cross-reference walk, or a free-text search) as an alternative to re-reading the
  corpus by hand. Prefer it over grep when you want the answer WITH its provenance in one call.
allowed-tools: Bash(cargo:*), Bash(curl:*), Read
---

# /tero-query — Query the Transparent Memory API

One engine (M-1016) behind two fronts (M-1017): an **MCP** server (`tero-mcp`) and an **HTTP/JSON**
API (`tero-http`). Both serve identical, byte-for-byte answers (front parity). Every answer carries
`citations` (each a resolvable `anchor` + `file:line` + the cited claim's guarantee tag) and an
`explain` trace; an uncited query returns a typed **refusal**, never a silent empty answer
(DN-87 §6.2).

## The five query kinds

| kind | matches | example |
|---|---|---|
| `id` | exact corpus id | `RFC-0034`, `M-1015`, `DN-87`, an issue id |
| `status` | a status (case-insensitive) | `Accepted`, `todo`, `done` |
| `kind` | a family kind | `rfc`, `adr`, `note`, `issue`, `section` |
| `cross_ref` | BFS walk of `depends_on`/`doc_refs` from a start id/anchor (`depth` hops) | `M-1017` depth `2` |
| `text` | ranked search over id/title/summary | `improved-on-RAG gate` |

## Over HTTP (`tero-http`, the universal floor — curl, Grok, anything)

```bash
TERO_TOKENS='demo:read' tero-http            # serves http://127.0.0.1:8787/v1 (refuses to start with no tokens)
curl -s -H 'Authorization: Bearer demo' 'http://127.0.0.1:8787/v1/query?kind=id&value=M-1015'
curl -s -H 'Authorization: Bearer demo' 'http://127.0.0.1:8787/v1/query?kind=text&value=eval%20gate'
curl -s -H 'Authorization: Bearer demo' 'http://127.0.0.1:8787/v1/query?kind=cross_ref&start=M-1017&depth=2'
```

The answer envelope: `{"kind":"answer","items":[…],"citations":[…],"explain":{…}}`; a refusal:
`{"kind":"refusal","refusal":{"variant":…,…},"message":"…"}` (both `200`).

## Over MCP (`tero-mcp`, native tool ergonomics)

Launch `tero-mcp` as a subprocess (newline-delimited JSON-RPC over stdio) with `TERO_TOKENS` set.
`initialize` → `tools/list` → `tools/call` with `{ "name": "query_by_id", "arguments": { "value":
"M-1015", "token": "demo" } }`. Tools: `query_by_id` / `query_by_status` / `query_by_kind` /
`cross_ref` / `text_search` (+ `cite` / `explain` / `identify` / `refresh`).

## Auth

Token-scoped, read-only by default. Supply `TERO_TOKENS='<token>:<read|refresh> …'` (or
`TERO_TOKENS_FILE`) at launch — never committed. Query/cite/explain need the `read` scope; the
server refuses to start with no tokens (never an accidentally-open server).

## Honesty

Answers are projections of the **Layer-1** index (`Empirical/Declared` heuristic — source is ground
truth). Each citation carries both the row's extraction tag and, where the source declares one, the
cited claim's own `Exact/Proven/Empirical/Declared` guarantee. Layer-2 (VSA) is gated off until its
eval gate opens (DN-87 §6.1); until then the API honestly serves Layer-1.
