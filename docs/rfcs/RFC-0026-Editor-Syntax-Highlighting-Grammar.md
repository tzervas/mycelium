# RFC-0026 — Editor Syntax Highlighting Grammar

| Field | Value |
|---|---|
| **RFC** | 0026 |
| **Status** | **Draft** (2026-06-23) |
| **Type** | Toolchain / normative (once Accepted) |
| **Date** | June 23, 2026 |
| **Feeds** | E9-1 (editor syntax highlighting & grammar distribution); E16-1 (toolchain, IDE & package distribution) |
| **Depends on** | DN-24 (Editor Syntax Highlighting — the advisory design space note that this RFC decides); RFC-0006 (surface language + lexer); `crates/mycelium-l1/src/token.rs` (`keyword()` — the source of truth for all keyword and token classes) |
| **Task** | E9-1 (M-693 — RFC-0026 spec/decision) |

> **Posture (honesty rule / VR-5).** Draft. "Current state" claims are grounded in
> `crates/mycelium-l1/src/{lexer,token}.rs` and `crates/mycelium-lsp/src/`. No grammar
> artifact is enacted by this stub; all choices remain open for the binding decision
> that this RFC will record. Normative sections (§3) are placeholders only — decide
> nothing without a checked basis.

---

## 1. Problem & goal

`.myc` source files currently render as plain, uncolored text in every editor and on
GitHub. This makes the language difficult to read and impedes the dogfooding effort
(`dfb`, E16-1). DN-24 surveyed the design space and recommended a **layered** approach —
TextMate baseline + tree-sitter + LSP semantic tokens — generated from the canonical
lexer so grammars cannot drift from the real language definition (G2). This RFC is the
**binding decision** that fixes:

1. Which grammar artifacts ship at the 1.0.0 toolchain gate.
2. The scope-name conventions (the mapping from Mycelium token/AST concepts to standard
   TextMate / LSP scope names).
3. The **single-source-of-truth rule**: how the grammars are generated from
   `crates/mycelium-l1/src/token.rs` `keyword()` and how drift is detected (`just`
   drift-check contract).
4. The VS Code extension packaging model and the GitHub Linguist registration path.

### Grounding

- **Token classes that exist today** (grounded in `crates/mycelium-l1/src/token.rs`):
  keywords (`keyword()`), guarantee-strength annotations (`Exact/Proven/Empirical/Declared`),
  PascalCase constructors, snake_case identifiers, `@` guarantee index, `!{…}` effect
  set, `->` function-type arrow, `=>` match arm, ternary literals, numeric literals.
- **`mycelium-lsp` today** (`crates/mycelium-lsp/src/`): provides completions,
  diagnostics, `fmt`; has **no** `textDocument/semanticTokens` provider.
- **No grammar artifact exists** in the repo today — the decision in this RFC is the
  first artifact.

## 2. User stories

- As a **language user**, I want `.myc` files to render with syntax colors in VS Code,
  Neovim, and Helix, so that I can read and write Mycelium code as comfortably as any
  mainstream language.
- As a **compiler engineer**, I want the highlighting grammar to be generated from the
  canonical lexer `keyword()` table with a CI drift-check, so that adding a new keyword
  never silently breaks the highlighter (G2 — never silent).
- As a **library/phylum author**, I want type-aware semantic-token colors (guarantee
  strength, generic type vars, trait names) via LSP, so that I can distinguish language
  concepts at a glance while writing phylum code.
- As a **tool author**, I want a GitHub Linguist registration for `.myc`, so that code
  shared on GitHub is highlighted in the repo browser without any user action.
- As a **downstream app developer**, I want a one-file TextMate grammar I can drop into
  any TextMate-compatible editor, so that I am not blocked by the need for an LSP
  server.
- As a **maintainer**, I want a single `just` target to regenerate all grammar artifacts
  and a drift-check that fails CI on keyword divergence, so that the highlighter is a
  reliable projection of the real lexer at every commit.

## 3. Scope & decision space

*(Placeholder — fill at authoring time. The options below are the DN-24 candidates; the
binding choice is deferred to the actual RFC authoring.)*

### 3.1 Artifact scope (in/out)

| Candidate artifact | DN-24 recommendation | Status in this RFC |
|---|---|---|
| TextMate grammar (`.tmLanguage.json`) | **In** — portable baseline | open |
| tree-sitter grammar (`grammar.js` → C/wasm) | **In** — parser-accurate | open |
| LSP semantic tokens (extend `mycelium-lsp`) | **In** — type-aware overlay | open |
| Reference VS Code extension (M-697) | **In** — packaging | open |
| GitHub Linguist registration | **In** — visibility | open |
| Per-editor manual configs (vim `syntax/`, Pygments, …) | **Out** (community-driven) | deferred |

### 3.2 Scope-name conventions (open — the binding question)

The mapping from Mycelium concepts to scope names (e.g. `keyword.control.mycelium`,
`storage.type.guarantee.mycelium`, `entity.name.function.mycelium`) is the central
normative question. Options include following VS Code / TextMate conventions verbatim,
Rust's `rust-analyzer` convention as a comparable, or a custom Mycelium-specific scheme.
**No scope names are fixed by this stub.**

### 3.3 Single-source-of-truth rule (open — the binding contract)

DN-24 recommends: a generator reads `token.rs` `keyword()` and emits the grammar
artifacts; a `just` drift-check re-runs the generator and `diff`s the output, failing
CI on any divergence. The exact generator architecture (Python script, Rust `xtask`,
templated JSON) is undecided.

### 3.4 Out of scope

- Language semantics changes (no kernel change — KC-3 untouched).
- Linting or type-correctness diagnostics (those are `myc-lint` / `myc-check` — DN-22).
- Per-operator syntax until RFC-0025 (operator sugar) is decided; if symbolic operators
  land, the generator re-derives from the updated `keyword()`/lexer.

## 4. Definition of Done

*(To be refined at authoring time — criteria below are the honest minimum.)*

- [ ] A binding decision recorded for each open item in §3 with a grounded rationale.
- [ ] Scope-name table fixed and committed (the normative output of this RFC).
- [ ] Single-source-of-truth rule and drift-check contract specified (generator
  architecture, `just` target name, CI hook).
- [ ] Artifact scope confirmed (which of TextMate / tree-sitter / LSP semantic tokens /
  VS Code extension / Linguist ship at the E9-1 gate).
- [ ] This RFC moves `Draft → Proposed` for community review, then `Proposed → Accepted`
  at maintainer ratification (never silently `Enacted` — append-only, house rule #3).
- [ ] `Doc-Index.md` and `CHANGELOG.md` updated (orchestrator-owned, not here).

## 5. Open questions

1. **Artifact scope gate:** which artifacts are required at the E9-1 Definition of Done
   vs. which are community follow-ups? (TextMate is the clear minimum; tree-sitter and
   LSP semantic tokens may be sequenced later.)
2. **Scope-name convention:** follow TextMate/VS Code standard names, Rust-analyzer
   convention, or a Mycelium-specific scheme? Tradeoff: custom names are expressive but
   require custom theme support; standard names work with any existing theme.
3. **Generator architecture:** Python `xtask`, Rust `xtask`, or a template-only approach?
   Must integrate cleanly with `just` and run in CI without additional deps.
4. **tree-sitter build artifact:** ship compiled C or wasm? Wasm has broader reach
   (browser, Zed) but a heavier build; C is simpler but tree-sitter-cli is required.
5. **LSP semantic-token legend versioning:** semantic token types / modifiers are
   negotiated at LSP capability-exchange time — how are Mycelium-specific types
   registered without colliding with future LSP standard types?
6. **Linguist registration:** requires a PR to the github-linguist repo — who owns that
   process and what `.myc` sample files demonstrate the grammar?

## 6. Grounding & honesty

- **Source of truth for keywords/tokens:** `crates/mycelium-l1/src/token.rs` `keyword()`
  — any scope-name table in this RFC must be checked against it at authoring time.
- **Current LSP gap:** `crates/mycelium-lsp/src/` has no semantic-token provider today;
  adding one is an additive change (does not touch the kernel, KC-3).
- **Guarantee tags:** all claims about generator correctness are `Declared` (a contract
  asserted, not proven) until a drift-check is built and green on CI (`Empirical`).
  No tag is upgraded to `Proven` without a checked basis (VR-5).
- **Append-only:** status transitions follow `Draft → Proposed → Accepted → Enacted`
  only. No step is skipped; a retroactive change supersedes, it does not rewrite.

## 7. Changelog

- **2026-06-23 — Draft.** Stub created for the E9-1 / E16-1 binding decision on editor
  syntax highlighting grammar artifacts, scope-name conventions, and the
  single-source-of-truth/drift-check rule. Grounds in DN-24; all normative choices
  deferred to authoring. Append-only.
