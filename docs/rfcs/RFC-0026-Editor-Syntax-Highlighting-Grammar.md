# RFC-0026 — Editor Syntax Highlighting Grammar

| Field | Value |
|---|---|
| **RFC** | 0026 |
| **Status** | **Accepted** (2026-06-23; Draft → Proposed → Accepted same day — maintainer-ratified per the E9-1 ask) |
| **Type** | Toolchain / normative |
| **Date** | June 23, 2026 |
| **Feeds** | E9-1 (editor syntax highlighting & grammar distribution); E16-1 (toolchain, IDE & package distribution) |
| **Depends on** | DN-24 (Editor Syntax Highlighting — the advisory design space note this RFC decides → Resolved); RFC-0006 (surface language + lexer); `crates/mycelium-l1/src/token.rs` (`keyword()` — the source of truth for all keyword and token classes) |
| **Task** | E9-1 (M-693 — RFC-0026 spec/decision); implemented by M-731 (M-694/M-695/M-696) |

> **Posture (honesty rule / VR-5).** Accepted. The normative §3 tables are now **fixed**. They are
> grounded in `crates/mycelium-l1/src/{lexer,token}.rs` (the keyword set + class buckets are
> *mechanically derived* from `keyword()`) and in the standard TextMate / LSP scope-name conventions
> (checked against VS Code's grammar guide and `rust-analyzer`'s LSP legend, per §5 Q2). The
> generator + drift gate that enforce this RFC **already exist** (`tools/grammar/generate.py`, `just
> drift-check`, landed with M-731); this RFC ratifies the scope names they emit. Guarantee tags:
> the generator's correctness is **`Empirical`** (the drift-check is green in CI), never `Proven`
> (VR-5). Append-only: a later change to the scope-name table **supersedes** this RFC, it does not
> rewrite it.

---

## 1. Problem & goal

`.myc` source files currently render as plain, uncolored text in every editor and on
GitHub. This makes the language difficult to read and impedes the dogfooding effort
(`dfb`, E16-1). DN-24 surveyed the design space and recommended a **layered** approach —
TextMate baseline + tree-sitter + LSP semantic tokens — generated from the canonical
lexer so grammars cannot drift from the real language definition (G2). This RFC is the
**binding decision** that fixes:

1. Which grammar artifacts ship at the 1.0.0 toolchain gate (§3.1).
2. The scope-name conventions — the mapping from Mycelium token classes to standard
   TextMate / tree-sitter / LSP scope names (§3.2, the central normative table).
3. The **single-source-of-truth rule**: how the grammars are generated from
   `crates/mycelium-l1/src/token.rs` `keyword()` and how drift is detected (§3.3).
4. The VS Code extension packaging model and the GitHub Linguist registration path (§3.1,
   deferred to M-697).

### Grounding

- **Token classes that exist today** (grounded in `crates/mycelium-l1/src/token.rs`):
  keywords (`keyword()`), guarantee-strength annotations (`Exact/Proven/Empirical/Declared`),
  substrate/representation types (`Binary/Ternary/Dense/VSA/Substrate/Sparse`), scalar element
  types (`F16/BF16/F32/F64`), snake_case identifiers, `@` guarantee index, `!{…}` effect
  set, `->` function-type arrow, `=>` match arm, balanced-ternary literals, binary literals.
- **`mycelium-lsp` today** (`crates/mycelium-lsp/src/`): provides completions,
  diagnostics, `fmt`, and — since M-730 — `textDocument/semanticTokens/full`.
- **The generator + drift gate exist** (`tools/grammar/`, M-731): they emit the artifacts this
  RFC's §3.2 names.

## 2. User stories

- As a **language user**, I want `.myc` files to render with syntax colors in VS Code,
  Neovim, and Helix, so that I can read and write Mycelium code as comfortably as any
  mainstream language.
- As a **compiler engineer**, I want the highlighting grammar to be generated from the
  canonical lexer `keyword()` table with a CI drift-check, so that adding a new keyword
  never silently breaks the highlighter (G2 — never silent).
- As a **library/phylum author**, I want type-aware semantic-token colors (guarantee
  strength, types, ctors) via LSP, so that I can distinguish language concepts at a glance.
- As a **tool author**, I want a GitHub Linguist registration for `.myc`, so that code
  shared on GitHub is highlighted in the repo browser without any user action (M-697).
- As a **downstream app developer**, I want a one-file TextMate grammar I can drop into
  any TextMate-compatible editor, so that I am not blocked by the need for an LSP server.
- As a **maintainer**, I want a single `just` target to regenerate all grammar artifacts
  and a drift-check that fails CI on keyword divergence, so that the highlighter is a
  reliable projection of the real lexer at every commit.

## 3. Decision (normative)

### 3.1 Artifact scope (decided)

| Artifact | Decision | Gate |
|---|---|---|
| TextMate grammar (`.tmLanguage.json`) | **In** — the portable baseline | **E9-1 / M-731 (landed)** |
| tree-sitter grammar (`grammar.js` + `highlights.scm`) | **In** — parser-accurate (scaffold; full structural grammar is community follow-up) | **E9-1 / M-731 (scaffold landed)** |
| LSP semantic tokens (`textDocument/semanticTokens/full`) | **In** — the type-aware overlay | **E9-1 / M-730 (landed)** |
| Reference VS Code extension | **In** — packaging | **M-697 (follow-up)** |
| GitHub Linguist registration | **In** — repo visibility | **M-697 (follow-up)** |
| Per-editor manual configs (vim `syntax/`, Pygments, …) | **Out** — community-driven | deferred |

The three grammar layers (TextMate · tree-sitter · LSP) are the **E9-1 / E16-1 gate**; the VS Code
extension + Linguist registration are the **M-697** packaging follow-up (they wrap, but do not change,
the artifacts decided here).

### 3.2 Scope-name table (decided — the normative output)

**Convention (decided, §5 Q2):** follow **standard names per layer** — TextMate scopes carry a
`.mycelium` language suffix (TextMate convention); **tree-sitter capture names and LSP token types are
the standard *unsuffixed* names** (`@keyword`, `keyword`, …). Standard names are chosen over a custom
scheme so that **every existing editor theme colors `.myc` with zero theme work** (a custom scheme
would need bespoke theme support); they are checked against the VS Code grammar guide and
`rust-analyzer`'s LSP legend.

The **class column is the lexer-derived bucket** (mechanically extracted from `token.rs::keyword()`
by `tools/grammar/generate.py` — see §3.3); each bucket maps to one name per layer. A **`—`** marks a
cell **not emitted by the shipped artifact** (deferred — see the coverage note below); it is *not* a
name choice:

| Lexer class (bucket) | Members (examples) | TextMate scope | tree-sitter capture | LSP token type |
|---|---|---|---|---|
| `keyword` | `nodule` `use` `pub` `type` `trait` `fn` `let` `if` `then` `else` `match` `for` `swap` `spore` … | `keyword.control.mycelium` | `@keyword` | `keyword` |
| `type` | `Binary` `Ternary` `Dense` `VSA` `Substrate` `Sparse` | `storage.type.mycelium` | `@type` | `type` |
| `scalar` | `F16` `BF16` `F32` `F64` | `support.type.builtin.mycelium` | `@type.builtin` | `type` |
| `strength` | `Exact` `Proven` `Empirical` `Declared` | `storage.modifier.guarantee.mycelium` | `@attribute` | `enumMember` |
| *comment* | `// …` | `comment.line.double-slash.mycelium` | `—` (deferred) | `comment` |
| *numeric* | `0b0010_1010`, `<+0-0>` | `constant.numeric.mycelium` | `—` (deferred) | `number` |
| *operator* | `->` `=>` `@` `!` `+` `-` … | `—` (deferred) | `—` (deferred) | `operator` |
| *identifier* | snake_case names | `—` (unscoped) | `—` (deferred) | `variable` |

**Coverage (honest — what the shipped artifacts emit today):**
- **The four word buckets** (`keyword`/`type`/`scalar`/`strength`) ship in **all three layers** — they
  are **mechanically derived** from the `=> Tok::…` right-hand side of each `keyword()` arm (no
  hand-maintained per-word list; a new lexer keyword is auto-classified and fails the drift gate until
  regenerated).
- **`comment` and `numeric`** ship in **TextMate + LSP** (a TextMate regex rule and the LSP classifier).
- **`operator` and `identifier`** ship in **LSP only** (the M-730 semantic-token classifier); TextMate
  emits no operator scope and leaves identifiers unscoped today.
- **The tree-sitter artifact is a reserved-word scaffold:** its `highlights.scm` captures **only the
  four word buckets**. The *comment*/*numeric*/*operator*/*identifier* tree-sitter captures (and the
  TextMate operator scope) arrive with the **full structural grammar** — the §3.4 / M-697 follow-up.
  The names above are the *decided targets* for those captures; the `—` records that the scaffold does
  not emit them yet (never an overclaim — VR-5/G2).

Notes:
- **Guarantee strengths** map to `storage.modifier.guarantee.mycelium` / `@attribute` /
  `enumMember`: they are honesty-lattice annotations (modifier-like), not control keywords — coloring
  them distinctly is the whole point of the type-aware layer (DN-24 §3).
- **Control vs. declaration keywords are NOT split** (both are `keyword.control.mycelium`). Splitting
  them would need a hand-maintained classification the lexer does not encode — a drift risk rejected
  here (G2). A finer split is a future superseding refinement, not part of this gate.
- The **LSP `function` token type is reserved in the legend but not assigned** by the lexical provider
  (it cannot tell a function name from a binding without semantic context — M-730 honesty note, VR-5).

### 3.3 Single-source-of-truth rule (decided — the binding contract)

- **Generator:** `tools/grammar/generate.py` (Python — no extra workspace dep; runs in CI via
  `just`). It extracts the keyword set + class buckets from `crates/mycelium-l1/src/token.rs`
  `keyword()` (the `=> Tok::…` arms) and renders `keywords.json`, `mycelium.tmLanguage.json`, the
  tree-sitter `grammar.js` + `queries/highlights.scm` from the §3.2 table.
- **Drift gate:** `just drift-check` (→ `scripts/checks/drift.sh`, wired into `just check`,
  component id 25) regenerates in memory and fails CI on any divergence between the committed
  artifacts and a fresh generation (`generate.py --check`), plus a `--self-test` (extraction +
  determinism). A new lexer keyword that is not regenerated into the grammars **fails CI** — the
  highlighter can never silently fall out of sync with the lexer (G2).
- **Regeneration:** `just grammar-gen` re-emits the committed artifacts; the owning parent commits
  the delta after any change to `keyword()`.

### 3.4 Out of scope

- Language semantics changes (no kernel change — KC-3 untouched).
- Linting or type-correctness diagnostics (those are `myc-lint` / `myc-check` — DN-22).
- The full structural tree-sitter grammar (this RFC ships the keyword-accurate scaffold; the complete
  production grammar is a community follow-up). Per-operator scope refinement re-derives from the
  lexer if RFC-0025 symbolic operators change `keyword()`.

## 4. Definition of Done

- [x] A binding decision recorded for each open item in §3 with a grounded rationale.
- [x] Scope-name table fixed and committed (§3.2 — the normative output of this RFC).
- [x] Single-source-of-truth rule and drift-check contract specified (§3.3 — generator
  `tools/grammar/generate.py`, `just drift-check`, CI hook in `scripts/checks/all.sh`).
- [x] Artifact scope confirmed (§3.1 — TextMate + tree-sitter + LSP semantic tokens ship at the
  gate; VS Code extension + Linguist are the M-697 follow-up).
- [x] This RFC moves `Draft → Proposed → Accepted` (maintainer-ratified; never silently `Enacted`
  — append-only, house rule #3). It is `Enacted` when M-697 ships the packaging and the gate's
  artifacts are stable.
- [x] `Doc-Index.md` and `CHANGELOG.md` updated (orchestrator-owned).

## 5. Open questions — resolved

1. **Artifact scope gate** → resolved (§3.1): the three grammar layers are the gate; VS Code
   extension + Linguist are M-697.
2. **Scope-name convention** → resolved (§3.2): standard names per layer — TextMate scopes carry a
   `.mycelium` suffix; tree-sitter captures and LSP token types are the standard *unsuffixed* names —
   maximal theme compatibility, no bespoke theme work.
3. **Generator architecture** → resolved (§3.3): a Python generator under `tools/grammar/`, no extra
   workspace dependency, `just`-driven, CI drift-checked.
4. **tree-sitter build artifact** → *deferred to the community follow-up*: the scaffold ships
   `grammar.js` + `highlights.scm`; whether to ship compiled C or wasm is decided when the full
   structural grammar is built (not gated here).
5. **LSP semantic-token legend versioning** → resolved (§3.2): the legend uses the standard LSP token
   types (`keyword`/`type`/`enumMember`/`number`/`operator`/`comment`/`variable`), which never
   collide with Mycelium-specific names; the legend is advertised at capability-exchange (M-730).
6. **Linguist registration** → *deferred to M-697*: a PR to `github-linguist` with `.myc` sample
   files; owner + samples decided there.

## 6. Grounding & honesty

- **Source of truth for keywords/tokens:** `crates/mycelium-l1/src/token.rs` `keyword()` — the §3.2
  class buckets are mechanically derived from it and drift-checked (§3.3).
- **LSP provider:** `crates/mycelium-lsp/src/semantic.rs` (M-730) advertises the legend; it is an
  additive toolchain change (KC-3 untouched).
- **Guarantee tags:** the generator's correctness is **`Empirical`** (drift-check green in CI), not
  `Proven` (VR-5). No tag upgraded without a checked basis.
- **Append-only:** status transitions follow `Draft → Proposed → Accepted → Enacted` only; a
  retroactive change supersedes, it does not rewrite.

## 7. Changelog

- **2026-06-23 — Draft.** Stub created for the E9-1 / E16-1 binding decision on editor
  syntax highlighting grammar artifacts, scope-name conventions, and the
  single-source-of-truth/drift-check rule. Grounds in DN-24; all normative choices deferred.
- **2026-06-23 — Proposed → Accepted.** §3 filled normatively (maintainer-ratified per the E9-1
  ask): artifact scope (§3.1 — three grammar layers gate; VS Code/Linguist → M-697), the scope-name
  table (§3.2 — standard TextMate/tree-sitter/LSP names, lexer-derived buckets), and the
  single-source-of-truth/drift contract (§3.3 — already implemented in `tools/grammar/` + `just
  drift-check`, M-731). DN-24 → Resolved. Implemented by M-731 (TextMate + tree-sitter scaffold + the
  M-730 LSP legend); VS Code extension + Linguist remain M-697. Append-only.
