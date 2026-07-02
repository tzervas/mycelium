# Changelog

All notable changes to the `tzervas.mycelium-language` VS Code extension are documented in this
file. Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.1.0] - 2026-07-02

### Added

- Initial release: TextMate grammar-based syntax highlighting for `.myc` files, sourced verbatim
  from the canonical generated grammar at `tools/grammar/mycelium.tmLanguage.json` (M-697;
  RFC-0026 §3.1/§3.4).
- Language configuration: line comments (`//`; Mycelium has no block comments), bracket pairs
  (`{}`/`[]`/`()`), auto-closing pairs (including double-quoted strings), surrounding pairs, and a
  word pattern for `[A-Za-z_][A-Za-z0-9_]*` identifiers.
- `npm run sync-grammar` — refreshes `syntaxes/mycelium.tmLanguage.json` from the canonical source
  so this copy can never silently diverge from the generator's output.
- Scope-assertion tests under `tests/` via `vscode-tmgrammar-test`, covering keywords, type
  keywords, strength annotations, string escapes (valid and invalid), numeric literals across all
  bases, the retired `->` arrow, and `@tier`-style attributes.
- No language server / IntelliSense wiring in this release — highlighting only.
