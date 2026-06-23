# Design Note DN-24 — Editor Syntax Highlighting & Grammar Distribution

| Field | Value |
|---|---|
| **Note** | DN-24 |
| **Status** | **Draft** (2026-06-23) |
| **Feeds** | RFC-0006 (lexer/grammar); DN-02/DN-03 (lexicon/syntax); `crates/mycelium-lsp`; E9-1 (editor-highlighting leg) |
| **Date** | June 23, 2026 |
| **Decides** | *Nothing normatively* — advisory. Records the design space for **editor syntax highlighting** (coloring `.myc` code) and the recommended **layered** approach, so the "make Mycelium readable in any editor" capability is tracked + grounded before it is built. A binding decision is deferred to **RFC-0026** (forthcoming). |
| **Task** | E9-1 (editor syntax highlighting & grammar distribution) |

> **Posture (honesty rule / VR-5).** Advisory (DN-17 posture). "Current state" claims cite the actual
> surface/tooling (`crates/mycelium-l1/src/{lexer,token}.rs`, `crates/mycelium-lsp/`,
> `.claude/memory/lang-lexicon-syntax.md`). No grammar is enacted by this note.

---

## 1. The goal

Maintainer ask (2026-06-23): make Mycelium code easy to **read** via syntax highlighting / coloring —
and crucially, in a form "easily plugged into pretty much any development environment." Today there is
**no highlighter**: a `.myc` file renders as plain, uncolored text in every editor and on GitHub. This
is a readability/ergonomics gap, not a correctness one — but it directly affects how pleasant the
language is to work in (and to dogfood, `dfb`).

This is a **syntax highlighter**, not a linter (the project already has `myc-lint` for lint and
`myc-check` for type/correctness diagnostics — DN-22). Highlighting *classifies tokens for color*; it
does not judge the code.

## 2. The decision space

No single artifact colorizes "any editor" — each editor family consumes a different grammar format.
The four standard mechanisms, by reach:

| Approach | Reaches | Pros | Cons |
|---|---|---|---|
| **TextMate grammar** (`.tmLanguage.json`) | VS Code, Sublime Text, TextMate, Zed (fallback), IntelliJ (plugin), **GitHub Linguist** | broadest *static* reach; one JSON file, no build step; the de-facto portable format | regex-only (no semantic awareness); hand-maintained → risk of drift from the real lexer |
| **tree-sitter grammar** (`grammar.js` → C/wasm) | Neovim, Helix, Emacs (tree-sitter), Zed, **GitHub** (Linguist is migrating its highlighting to tree-sitter), Atom-lineage | parser-accurate; incremental; the parse tree is reusable for folding / structural nav / selection | needs a build (C/wasm); a second grammar source to keep in sync with the lexer |
| **LSP semantic tokens** (`textDocument/semanticTokens`, extend `mycelium-lsp`) | any LSP client — VS Code, Neovim, Helix, Emacs, Zed, Sublime (LSP) | **type-aware / token-accurate** — the *real checker* drives the colors (e.g. a guarantee tag vs a type vs a ctor); reuses the existing crate | an *overlay* only (needs a base grammar for offline / non-LSP rendering); per-editor LSP wiring |
| **Per-editor configs** (vim `syntax/`, nano, `highlight.js`, Pygments lexer, …) | one editor / tool each | trivial per target | does not scale to "any editor"; N hand-maintained copies that drift |

## 3. Recommendation — a layered stack with one source of truth

Adopt **three complementary layers** (plus packaging), so coverage is broad *and* the grammars cannot
drift from the language:

1. **TextMate grammar as the portable baseline.** A single `.tmLanguage.json` covers the most editors
   *and* is exactly what **GitHub Linguist** consumes to colorize `.myc` in the repo. This is the
   "works almost everywhere with zero runtime" layer.
2. **tree-sitter grammar for parser-accurate highlighting** in modern editors (Neovim/Helix/Emacs/Zed)
   and GitHub's newer pipeline; the parse tree also seeds future structural tooling.
3. **LSP semantic tokens in `mycelium-lsp`** as a **type-aware overlay** — the checker's own token
   classification (the only layer that can colour, e.g., a `Proven` vs `Declared` guarantee tag or a
   generic type var accurately). `mycelium-lsp` today has `completions`/`diagnostics`/`fmt` but **no
   semantic-token provider** — this adds one.
4. **Packaging:** a reference **VS Code extension** (bundles the TextMate grammar + wires the LSP) and a
   **GitHub Linguist** registration for `.myc`, as the one-click / repo-visible reference consumers.

**Single source of truth (honesty).** The token classes (keywords, the guarantee strengths
`Exact/Proven/Empirical/Declared`, ctor vs type casing, the `@` / `!{…}` / `->` operators, ternary
literals) are **derived from the canonical lexer** — `crates/mycelium-l1/src/token.rs` (`keyword()`)
plus `.claude/memory/lang-lexicon-syntax.md` — by a generator, **not** hand-maintained. A `just`
drift-check fails CI if the generated grammar's keyword set diverges from `keyword()`, so the
highlighter is a *projection of the real lexer* and cannot silently fall out of sync (G2). This is the
same "tooling tracks the trusted base" discipline used for the api-index.

## 4. Pluggability — the "any editor" requirement, mapped

| Editor / surface | Layer that serves it |
|---|---|
| VS Code, Sublime, IntelliJ, Zed (static), **GitHub repo view** | TextMate grammar (+ Linguist) |
| Neovim, Helix, Emacs, Zed (modern), **GitHub** (new) | tree-sitter grammar |
| Any LSP client (type-aware overlay) | `mycelium-lsp` semantic tokens |

TextMate + tree-sitter + LSP **together** cover essentially every actively-used editor, and each is an
independent, standard artifact a user can drop in — satisfying "easily pluggable into pretty much any
development environment."

## 5. Scope / honesty

- A new toolchain leg (**E9-1**, Phase 8). Disjoint from the language kernel (KC-3 untouched — a
  highlighter reads tokens, it does not change semantics) and from the operator-syntax leg (E7-5/DN-23):
  if symbolic operators are later added, the grammar's generator re-derives from the updated lexer.
- Advisory only: this note **decides nothing normatively** and enacts no grammar; the binding format /
  scope-naming decision is **RFC-0026**. The "generated from the lexer" claim is grounded in
  `token.rs` `keyword()`; the "no semantic-token provider yet" claim is grounded in the
  `crates/mycelium-lsp/src/` file list. No tag is upgraded (VR-5).
- Sequencing: independent of, and runnable in parallel with, the dogfooding builds (`dfb`) — better
  highlighting makes dogfood code easier to read, so it is a natural companion rather than a blocker.
