# Editor grammars (M-731; RFC-0026 Accepted — v2 full-surface, M-697)

Three-layer syntax-highlighting stack for `.myc`, **generated** from the canonical L1 lexer so it
can never silently diverge from the language the compiler actually accepts (G2):

| Artifact | File | Purpose |
|---|---|---|
| Keyword snapshot | `keywords.json` | The lexer-derived keyword set + class buckets (the drift baseline). |
| TextMate grammar | `mycelium.tmLanguage.json` | VS Code / Sublime / TextMate / Linguist highlighting. |
| tree-sitter grammar | `tree-sitter-mycelium/grammar.js` | Structural grammar (v2) — Neovim / Zed / Helix / Emacs. |
| tree-sitter queries | `tree-sitter-mycelium/queries/highlights.scm` | Highlight capture queries. |
| LSP semantic tokens | `crates/mycelium-lsp/src/semantic.rs` | `textDocument/semanticTokens/full` (M-730). |

Distribution (VS Code extension packaging, Open VSX, GitHub Linguist, Rouge/GitLab, and the
per-editor tree-sitter wiring) is documented in **`DISTRIBUTION.md`**.

## Source of truth & drift gate

Everything here is **generated** from `crates/mycelium-l1/src/token.rs::keyword()` by
`tools/grammar/generate.py`. Do **not** hand-edit the generated files. After any change to the
lexer keyword table:

```sh
just grammar-gen     # regenerate
just drift-check     # the CI gate: committed grammars must match a fresh regeneration (G2)
```

`drift-check` is wired into `just check`; it fails CI if the committed grammars drift from the
lexer. The v2 `--self-test` additionally asserts every lexer keyword appears in **every** rendered
artifact, so a keyword the structural templates miss fails loudly (exit 3), never silently drops a
word from highlighting.

## v2 — full-surface grammars (M-697; RFC-0026 §3.4 follow-up)

The v1 artifacts were a reserved-word scaffold. v2 covers the full landed surface, derived from
the normative oracle `docs/spec/grammar/mycelium.ebnf` + the L1 lexer/parser:

- **Literals:** strings + the minimal escape set (M-910/M-911; unknown escapes render
  `invalid.illegal`), floats (ADR-040/M-897), `0b…`/`0t…`/`0x…` (the retired `<…>` compact-ternary
  pattern is REMOVED — RFC-0037 D4 moved ternary literals to `0t…`).
- **Operators:** the RFC-0025/M-705/M-745 set (`== != ! && || & | + - * / % ^ < > << >> = =>`).
  The `<=`/`>=` glyphs are **retired** (RFC-0037 D1 — `lte`/`gte` are word-form calls) and the
  retired `->` renders `invalid.deprecated`.
- **Types:** repr types + the RFC-0037 D2-b short aliases (`bin`/`tern`/`emb`/`hvec`, M-915),
  ambient `{…}` reprs, tuples, generics `[…]`, guarantee annotations `T @ Strength`, and
  right-associative function types `A => B` (implemented in parse.rs; flagged as a pending
  `mycelium.ebnf` addition).
- **Declarations & expressions** (tree-sitter): nodule/phylum headers (+`@std-sys`), use/type/
  trait/impl/object/`via`/lower/derive/fn (+`@tier`), effects `!{…}`, let/if/match (or-patterns)/
  for/swap/with/wild/spore/consume/colony+hypha (+`@forage`)/fuse/reclaim/lambda, and the full
  operator-precedence tiers.

**Guarantee tags (VR-5):** the keyword set/buckets are lexer-derived (mechanical). The structural
productions are **Empirical** — verified by parsing the full conformance accept corpus +
`lib/std/*.myc` with **zero ERROR nodes** — not proven equivalent to the EBNF, which stays the
accept/reject oracle. Two **Declared, documented permissive deviations**: (1) reserved-not-active
keywords parse as an explicit `reserved_keyword` atom (the L1 parser rejects them; a highlighting
grammar renders them in snippets/docs); (2) repr-type brace arities are checker territory.

## Scope names — ratified (RFC-0026 §3.2, Accepted) + v2 structural scopes

The four bucket names are the **ratified RFC-0026 §3.2 table**; the v2 comment/string/literal/
operator/declaration scopes are the standard TextMate / tree-sitter names the RFC's §3.4 follow-up
anticipated. A change to the bucket table **supersedes** RFC-0026 (append-only); the generator is
then re-run.

| Lexer bucket | TextMate scope | tree-sitter capture | LSP token type |
|---|---|---|---|
| `keyword` | `keyword.control.mycelium` | `@keyword` | `keyword` |
| `type` | `storage.type.mycelium` | `@type` | `type` |
| `scalar` | `support.type.builtin.mycelium` | `@type.builtin` | `type` |
| `strength` | `storage.modifier.guarantee.mycelium` | `@attribute` | `enumMember` |
| comment | `comment.line.double-slash.mycelium` | `@comment` | `comment` |
| string | `string.quoted.double.mycelium` | `@string` | `—` (M-910 postdates the legend) |
| numeric | `constant.numeric.<kind>.mycelium` | `@number` | `number` |
| operator | `keyword.operator.mycelium` | `@operator` | `operator` |
| declarations | `entity.name.*.mycelium` | `@function` / `@type` / `@module` … | `function`/`type` |

**Bucket correction (v2):** `Float` and the M-915 short aliases bucket as `type` (token.rs reserves
the shorts as "the same class as `Binary` itself"; `Float` is the ADR-040 repr-type keyword — v1
misfiled it under `keyword` because `TYPE_VARIANTS` predated ADR-040).

## Verifying locally

```sh
python3 tools/grammar/generate.py --self-test     # extraction + coverage + determinism
cd tools/grammar/tree-sitter-mycelium
npx tree-sitter-cli@0.25 generate                 # grammar.js -> src/parser.c
npx tree-sitter-cli@0.25 parse -q ../../../lib/std/*.myc   # zero ERROR nodes expected
```
