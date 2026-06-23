# Editor grammars (M-731; RFC-0026)

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

## Status — SCAFFOLD, not finalized (honesty, VR-5/G2)

> **RFC-0026 (the binding scope-name decision) is still `Draft`.** Per the M-731 sequencing, the
> generator and drift gate are complete, but the **TextMate / tree-sitter scope names** — the open
> question of RFC-0026 §3.2 — are emitted as explicit `TODO.rfc-0026.*` placeholders. They are
> **not** the ratified scope-name table and must not be treated as final. The scope-name mapping is
> finalized (and this note removed) when RFC-0026 is Accepted (M-693 Done); the generator is then
> re-run to replace the placeholders. The keyword *set* and *class buckets* are already
> lexer-derived and gated, so the highlighting wiring is real today — only the colour-category
> names await ratification.
