# packages/ — Future proving-out / dogfooding backlog

Forward-looking ideas for the MCP-tooling family and Mycelium dogfooding. These are **backlog /
plan-around** items (maintainer, 2026-07-08), not committed work — captured here so they survive
across sessions. Each is `Declared` (an intent, not a decision); none is scheduled.

> **Posture (VR-5 / G2).** Nothing here is promised or dated. The dependencies (language maturity,
> the first target) are stated honestly; an item does not start until its gate clears.

---

## B1 — Mycelium-native MCP protocol SDK (a new repo; language-maturity-gated)

A repository implementing the **Anthropic MCP protocol standard natively in the Mycelium language** —
the Mycelium analogue of the Python / Rust / TypeScript MCP SDKs — so that MCP servers can be authored
**directly in Mycelium** rather than in a host language. It is a **dogfooding proof-out**: Mycelium
implementing a real, widely-used wire protocol end to end (JSON-RPC framing, the tools/resources/prompts
surface, stdio + HTTP transports, the typed-refusal + capability model).

- **Gate:** Mycelium self-hosting / expressiveness maturity — the language surface across all layers
  (parse → check → elab → eval → codegen, plus the stdlib IO/JSON/async surface a server needs) being
  nailed down enough to express an MCP server. Tracks the self-hosting effort (boot10 / DN-26) and the
  "once the full surface is finalized" bar the maintainer set for Mycelium-side codegen work.
- **Shape (future):** its own repo (like `tero-mcp`), created when the language can support it; the
  existing `tero-mcp` / `context-mcp` servers are the reference *behaviour* to reproduce natively.
- **Value:** the strongest possible dogfooding signal — if Mycelium can host MCP servers, it can host
  real protocol work; and it lets the whole MCP-tooling family eventually go native.

## B2 — Parameterized code-completion template library (MCP-served)

Capture **Python + Rust** code completions (and **Mycelium later**, once its full surface is finalized)
as **parameterized templates** for construct kinds — functions, methods, types, objects, generators, etc.
A model supplies its intent + the parameters (name, inputs, outputs, types, returns) and the template
emits a **well-formed, completed implementation** matching the intent, from the schema + inputs. A typed
scaffold / codegen system, served as an MCP (a "codegen-template MCP") so any model can plug in a schema
and get well-formed code back.

- **Gate:** none for Python/Rust (startable now); the **Mycelium** template set waits on the same
  language-surface-finalized bar as B1.
- **Connects to:** Mycelium's transpiler / codegen (schema → well-formed code is the transpiler's job in
  reverse). Could be **Grok-built** — feed the template spec + construct schemas via the xAI/Grok harness
  (`grok-build` for bulk, heavier models for the tricky construct kinds) and integrate the returned archive.
- **Priority:** after the MCP extraction (the first target). Not this wave.

---

## Notes

- Both are **MCP-family / Mycelium dogfooding** proving-out items. The MCP extraction (Tero MCP + the
  existing `*-mcp` / `dev-*` repos) stays the first target; these ride behind it.
- B1 is **language-maturity-gated**; B2 can begin on Python/Rust independently, with its Mycelium tier
  gated the same way as B1.

## Meta — changelog

- **2026-07-08 — Created.** Captured the two maintainer backlog ideas (B1 Mycelium-native MCP SDK,
  B2 parameterized code-template MCP) durably so they survive session/context resets. `Declared`;
  nothing scheduled. Append-only.
