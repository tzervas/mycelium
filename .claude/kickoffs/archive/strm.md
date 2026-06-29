# Kickoff `strm` — Mandatory `;` Terminator & Human↔Stream Tooling (DN-57 follow-on)

> Stowed kickoff, UID **`strm`**. Read `CLAUDE.md`, `.claude/kickoffs/README.md`, and
> **`.claude/kickoffs/_doc-maintenance.md`** (anti-drift) first.

## Metadata
| Field | Value |
|---|---|
| **UID** | strm |
| **Head/working branch** | `claude/head/strm-mandatory-semicolon` (off `dev`) |
| **Status** | ready (grammar-completion follow-on; can run in parallel with the semantic waves — disjoint-ish, but coordinate on `parse.rs`/`mycelium-fmt`) |
| **Swarm mode** | serial-on-L1 + a fmt/CLI leaf (corpus migration can fan to leaves) |
| **Depends on** | DN-57 (Accepted; the **optional** `;` form is landed), RFC-0037 (Enacted) |

## Scope
Turn DN-57's `;` component terminator from **optional → mandatory** (the hard streaming guarantee), add
the **nodule-header terminator** (so fully whitespace-free `nodule d; …` is legal), make **`mycfmt`
emit `;` canonically**, and ship the **human↔stream round-trip tooling** — `mycfmt --flatten` (canonical
multi-line → minimal-whitespace delimiter-driven stream) and `myc --stream` / a streaming parse entry
that relies on `;` for explicit, lookahead-free component boundaries (incremental/resumable parsing).
Settle DN-57 §3 (which components terminate; the `}` interaction).

**Issue:** mint M-816+-range (grep `issues.yaml` first — mitigation #1); a DN-57 enactment epic.

## Grounding (doc_refs)
- `corpus:DN-57` — the delimiter triad (`:` ascribe / `,` separate / `;` terminate), the streaming
  rationale, and §3 open questions (mandatory-vs-optional; component scope; `}` interaction; migration).
- `corpus:RFC-0037` — layout-independence (FLAG-2) this completes; `corpus:RFC-0030` (concrete grammar).
- `src:crates/mycelium-l1/src/parse.rs` (the item/method loops already `eat(Semi)` optionally — make it
  required; add the header terminator) · `src:crates/mycelium-fmt/src/lib.rs` (emit `;`) ·
  `src:docs/spec/grammar/mycelium.ebnf`.

## Approach
parse.rs: require `;` (or newline-equivalent per §3 decision) after each component; add the header `;`;
keep a never-silent "missing terminator" error (G2). fmt: `render_*` emit `;`; add `--flatten`. CLI:
`myc --stream` streaming-parse entry. **Corpus migration** (every component gains `;`) — mechanical, fan
to parallel disjoint-dir leaves (octopus), like the RFC-0037 migration; mandatory-`;` will require
migrating the whole `.myc` corpus + every test corpus + conformance fixtures + expected formatter output.

## Definition of Done
- [ ] `;` is the **enforced** component terminator; a missing one is an explicit error (G2); fully
  whitespace-free source parses; `mycfmt` emits `;` canonically; `mycfmt --flatten` ⇄ canonical round-trips
  byte-stably (C1/C2); a streaming parse entry emits a component per `;` without keyword lookahead.
- [ ] Whole corpus migrated; `just check` green (incl. conformance/differential/fmt); honest tags.
- [ ] **Doc maintenance (anti-drift):** `issues.yaml` task → done; **DN-57 `Accepted → Enacted`** (mandatory
  form landed); §3 questions resolved in DN-57 (append-only); `mycelium.ebnf` updated to mandatory `;`
  (+ `just grammar-gen`); `.claude/memory/lang-lexicon-syntax.md`; `CHANGELOG.md` + the `mycfmt`/`myc`
  CLI docs (`docs/spec/Mycfmt-Formatter-Contract.md`); `docs/api-index/` if API changed.

## Landing
`/wave-land` → `main` after green + self-review + curated squash; backprop.
