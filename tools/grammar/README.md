# Editor grammars (M-731; RFC-0026 Accepted)

Three-layer syntax-highlighting stack for `.myc`, **generated** from the canonical L1 lexer so it
can never silently diverge from the language the compiler actually accepts (G2):

| Artifact | File | Purpose |
|---|---|---|
| Keyword snapshot | `keywords.json` | The lexer-derived keyword set + class buckets (the drift baseline). |
| TextMate grammar | `mycelium.tmLanguage.json` | VS Code / Sublime / TextMate highlighting. |
| tree-sitter grammar | `tree-sitter-mycelium/grammar.js` | Structural grammar scaffold. |
| tree-sitter queries | `tree-sitter-mycelium/queries/highlights.scm` | Highlight capture queries. |
| LSP semantic tokens | `crates/mycelium-lsp/src/semantic.rs` | `textDocument/semanticTokens/full` (M-730). |

## Source of truth & drift gate

Everything here is **generated** from `crates/mycelium-l1/src/token.rs::keyword()` by
`tools/grammar/generate.py`. Do **not** hand-edit the generated files. After any change to the
lexer keyword table:

```sh
just grammar-gen     # regenerate
just drift-check     # the CI gate: committed grammars must match a fresh regeneration (G2)
```

`drift-check` is wired into `just check`; it fails CI if the committed grammars drift from the
lexer.

## Scope names — ratified (RFC-0026 §3.2, Accepted)

The TextMate / tree-sitter / LSP scope names are the **ratified RFC-0026 §3.2 table**: standard
TextMate / tree-sitter / LSP names with a `.mycelium` suffix (chosen for maximal theme
compatibility — every existing theme colors `.myc` with no theme work). The keyword/type/scalar/
strength **buckets** are mechanically derived from the lexer; the **names** they render to are fixed
by the RFC:

| Lexer bucket | TextMate scope | tree-sitter capture | LSP token type |
|---|---|---|---|
| `keyword` | `keyword.control.mycelium` | `@keyword` | `keyword` |
| `type` | `storage.type.mycelium` | `@type` | `type` |
| `scalar` | `support.type.builtin.mycelium` | `@type.builtin` | `type` |
| `strength` | `storage.modifier.guarantee.mycelium` | `@attribute` | `enumMember` |
| comment | `comment.line.double-slash.mycelium` | `@comment` | `comment` |
| numeric | `constant.numeric.mycelium` | `@number` | `number` |

A change to this table **supersedes** RFC-0026 (append-only); the generator is then re-run.

## Status & follow-ups

The TextMate grammar, the tree-sitter keyword-accurate scaffold, and the LSP semantic-token legend
ship at the E9-1 / E16-1 gate (M-731). The **full structural tree-sitter grammar** (productions
beyond the reserved-word set) and the **VS Code extension + GitHub Linguist registration** are the
**M-697** community follow-up named in RFC-0026 §3.1/§3.4 — not part of this gate.
