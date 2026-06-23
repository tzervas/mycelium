# Mycelium Reference Documentation

User-facing documentation for the Mycelium language, authored for the full-language 1.0.0
documentation gate (E17-1 / M-735). These pages let you learn and look up the language **without
reading compiler source**.

| Document | What it is | Start here if… |
|---|---|---|
| **[tutorial.md](./tutorial.md)** | A hands-on walkthrough that builds a complete program from scratch | …you are new to Mycelium |
| **[language-reference.md](./language-reference.md)** | The full surface — every construct, in lookup form | …you want to look up a keyword, type, or form |

## How these docs are grounded (honesty)

> **Status: Empirical/Declared.** These are user docs, not normative specs. The **normative oracle**
> is `docs/spec/grammar/mycelium.ebnf` plus the accept/reject conformance corpus under
> `docs/spec/grammar/conformance/`; the **active keyword set** is `crates/mycelium-l1/src/token.rs`.
> Where these pages and a normative source disagree, the source wins.

Every behavioural claim is grounded in a cited spec, the grammar, the conformance corpus, or the
interpreter — nothing is invented (G2 / VR-5). The tutorial's complete program is committed as a
**parser-verified** conformance fixture (`accept/20-tutorial-classifier.myc`), parsed on every CI run
by `crates/mycelium-l1/tests/conformance.rs`. Honest VR-5 notes mark surface that *type-checks but
does not yet run* (generics/traits elaboration → M-673; effect annotations → M-677).

## Related documentation

- **Stability promise:** `docs/adr/ADR-023-Stability-and-API-Compatibility-Guarantees.md` — what is
  stable at 1.0.0, the deprecation policy, the dual-version model.
- **Stdlib specs:** `docs/spec/stdlib/` — per-module standard-library specifications.
- **Glossary:** `docs/Glossary.md` — per-term definitions with normative citations.
- **Grammar:** `docs/spec/grammar/mycelium.ebnf` + `docs/spec/grammar/README.md`.

## Changelog

- **2026-06-23 — Created (M-735).** Reference section index for the full-language 1.0.0 docs gate
  (E17-1): the language reference + tutorial. Append-only.
