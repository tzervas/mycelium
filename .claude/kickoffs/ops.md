# Kickoff `ops` — Ordering/Shift Operator Wiring (RFC-0025/0030, M-745)

> Stowed kickoff, UID **`ops`**. Read `CLAUDE.md`, `.claude/kickoffs/README.md`, and
> **`.claude/kickoffs/_doc-maintenance.md`** (anti-drift) first.

## Metadata
| Field | Value |
|---|---|
| **UID** | ops |
| **Head/working branch** | `claude/head/ops-operator-wiring` (off `dev`) |
| **Status** | ready (RFC-0025 operator residue ratified 2026-06-28; wiring commissioned as M-745) |
| **Swarm mode** | serial-on-L1 (inline; parser + desugar) |
| **Depends on** | RFC-0037 (Enacted — operator-only `<`/`>`, bracket kind-split), RFC-0025 (Accepted — the ratified precedence table + desugar map) |

## Scope
Wire the **comparison and shift operators** the RFC-0025 ratification commissioned: `<` `>` (`lt`/`gt`),
`<=`/`>=`-spelled `lte`/`gte`, and `<<`/`>>` (`shl`/`shr`) — adding the **precedence tiers** (shift Tier 4,
comparison Tier 8 per RFC-0025 §4.1) and the **desugar map** entries (`lt`/`gt`/`shl`/`shr`/`lte`/`gte` →
canonical word functions, frontend-only, no L0/L1 change — KC-3) per RFC-0025 §4.2. Closes M-745 and meets
the RFC-0030 §4.3 gate; moves RFC-0025 Accepted → Enacted.

## Grounding (doc_refs)
- `corpus:RFC-0025` — the ratified §4.1 precedence table (now with shift/cmp tiers) + §4.2 desugar map
  (the second table with all six entries) + the §4.3 resolution note.
- `corpus:RFC-0030#§4.3` — the M-745 gate this meets; `corpus:RFC-0037#§6` — the `cmp_expr`/`shift_expr`
  productions the parser/EBNF must match.
- `src:crates/mycelium-l1/src/parse.rs` (operator parsing + precedence) · the desugar site (word-function
  lowering) · `src:docs/spec/grammar/mycelium.ebnf`.

## Approach (serial-on-L1, inline)
parse.rs: add the shift + comparison precedence tiers (RFC-0037 §6 `cmp_expr`/`shift_expr` shape — note
`<` is operator-only post-RFC-0037, so no ambiguity with the retired type-arg `<>`); desugar `a < b` →
`lt(a,b)` etc. (frontend-only; the canonical word fns already exist). Add accept fixtures exercising each
operator + precedence; a differential confirming `a < b` ≡ `lt(a,b)`. Regenerate `mycelium.ebnf`
(`cmp_expr`/`shift_expr`) + `just grammar-gen`.

## Definition of Done
- [ ] `<` `>` `<<` `>>` (and the `lte`/`gte` word forms) parse at the ratified precedence and desugar to
  the canonical word functions; **no L0/L1 change (KC-3)**; accept fixtures + a desugar differential green.
- [ ] `just check` green (incl. drift gate — EBNF + editor grammars regenerated); honest tags.
- [ ] **Doc maintenance (anti-drift):** `issues.yaml` **M-745 → done**; **RFC-0025 Accepted → Enacted**
  (operators wired); RFC-0030 §4.3 gate-met note; `mycelium.ebnf` `cmp_expr`/`shift_expr` (+ `just
  grammar-gen`); `.claude/memory/lang-lexicon-syntax.md` operator table; `CHANGELOG.md` entry;
  `docs/api-index/` if API changed.

## Landing
`/wave-land` → `main` after green + `/pr-review` self-review + curated squash; backprop.
