//! `mycelium-l1` — the **L1 surface prototype** (RFC-0006; **NON-NORMATIVE** until RFC-0006 is
//! ratified). A hand-written lexer + recursive-descent parser for the ratified DN-02 surface
//! vocabulary, validated against the `docs/spec/grammar/` conformance corpus (the
//! WebAssembly-spec pattern, T3.1-B: the corpus is the ground truth, not any single parser).
//!
//! The L1 track so far (RFC-0006 §3; RFC-0007): the lexer + recursive-descent parser prove the
//! grammar is real by parsing every `accept/` program and explicitly rejecting every `reject/`
//! one (`tests/conformance.rs`); the v0 monomorphic typechecker + structural totality checker
//! ([`checkty`], [`totality`]; RFC-0007 §4.4/§4.5) gate `matured` on checked totality; the
//! fuel-guarded big-step evaluator ([`eval`]; §4.6) runs every checked program over the *same*
//! trusted prim/swap engines as the L0 paths; and the elaborator ([`elab`]; §4.6) lowers the
//! evaluation-complete fragment to closed L0 terms — refusing everything else with an explicit
//! `Residual`, never a partial artifact. The three-way differential (L1-eval ↔ elaborate→L0-interp
//! ↔ AOT, validated through the M-210 shared checker) lives in `tests/differential.rs` (NFR-7).
//! `match` covers data types and `Binary`/`Ternary` literal patterns *and* **nested** patterns
//! (M-320): a literal arm fires on `repr + payload` equality, and coverage is decided by the
//! **Maranget usefulness** algorithm ([`usefulness`]) — exhaustiveness (a `_` must not be useful; its
//! witness names a concrete missing case) and redundancy (an arm covered by earlier rows is
//! unreachable) are both *checked* (W7 — never assumed; a `Binary`/`Ternary` value domain is never
//! enumerated, so a literal match still needs a `_`/binder default). The Maranget *compilation* to the
//! flat kernel `Match` (RFC-0007 §3, the elaborator/L0 path) lands with full L1-in-Core-IR (the
//! RFC-0001 revision).
//!
//! Honesty: every malformed input is an explicit [`ParseError`] with a source position — the
//! parser never panics and never silently accepts (S5/G2). The lexer disambiguates the one tricky
//! token (`<` opening a ternary literal vs a type-argument list) by lookahead, and a malformed
//! ternary literal is an explicit error, not a silent truncation.

pub mod ambient;
pub mod ast;
pub mod checkty;
pub(crate) mod decision;
pub mod elab;
pub mod error;
pub mod eval;
pub mod lexer;
pub mod nodule;
pub mod parse;
pub mod token;
pub mod totality;
pub(crate) mod usefulness;

pub use ambient::{expand_to_source, resolve, resolve_report, AmbientError, Resolved};
pub use ast::Nodule;
pub use checkty::{check_and_resolve, check_nodule, check_nodule_matured, CheckError, Env, Ty};
pub use elab::{elaborate, ElabError};
pub use error::ParseError;
pub use eval::{Evaluator, L1Error, L1Value};
pub use nodule::{parse_nodule_header, NoduleHeader, NoduleHeaderError};
pub use parse::parse;
pub use totality::Totality;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BaseType, Expr, Item, Literal};

    #[test]
    fn parses_a_nodule_with_a_swap() {
        let src = "nodule demo\nfn f(x: Binary{8}) -> Ternary{6} =\n  swap(x, to: Ternary{6}, policy: rt)";
        let nodule = parse(src).expect("parses");
        assert_eq!(nodule.path.0, vec!["demo"]);
        assert_eq!(nodule.items.len(), 1);
        let Item::Fn(f) = &nodule.items[0] else {
            panic!("expected a fn item");
        };
        assert!(!f.thaw);
        assert!(matches!(f.body, Expr::Swap { .. }));
        assert_eq!(f.sig.ret.base, BaseType::Ternary(6));
    }

    #[test]
    fn a_swap_without_policy_is_an_explicit_error() {
        // S1/WF2: the policy is mandatory; its absence is a diagnostic, never a silent accept.
        let src = "nodule demo\nfn f(x: Binary{8}) -> Ternary{6} = swap(x, to: Ternary{6})";
        let err = parse(src).unwrap_err();
        assert!(err.message.contains("policy"), "got: {}", err.message);
    }

    #[test]
    fn a_reserved_word_is_not_a_usable_identifier() {
        let src = "nodule demo\nfn nodule(x: Binary{8}) -> Binary{8} = x";
        assert!(parse(src).is_err());
    }

    #[test]
    fn phylum_and_colony_are_reserved_not_active() {
        // DN-06: `phylum` (library grouping) and `colony` (dynamic runtime grouping) are reserved
        // keywords — they lex as keywords (so never silent identifiers) but no L1 construct consumes
        // them yet, so neither opens a program and neither is a usable identifier (G2).
        assert!(parse("phylum signals\n").is_err());
        assert!(parse("colony signals\n").is_err());
        assert!(parse("nodule demo\nfn phylum() -> Binary{8} = 0b0").is_err());
        assert!(parse("nodule demo\nfn f(colony: Binary{8}) -> Binary{8} = colony").is_err());
    }

    #[test]
    fn a_malformed_ternary_literal_is_explicit() {
        let src = "nodule demo\nfn f() -> Ternary{3} = <+x->";
        let err = parse(src).unwrap_err();
        assert!(err.message.contains("non-trit"), "got: {}", err.message);
    }

    #[test]
    fn thaw_fn_parses_and_sets_thaw_true() {
        // RFC-0017 §4.3: `thaw fn` is the de-maturation marker; the field must be `true`.
        let src = "nodule demo\nthaw fn k() -> Binary{8} = 0b1011_0010";
        let nodule = parse(src).unwrap();
        let Item::Fn(f) = &nodule.items[0] else {
            panic!("fn");
        };
        assert!(f.thaw);
        assert!(matches!(&f.body, Expr::Lit(Literal::Bin(s)) if s == "1011_0010"));
    }

    #[test]
    fn matured_fn_at_item_position_is_a_parse_error_with_teaching_diagnostic() {
        // RFC-0017 §4.1: `matured fn` at item position is retired — the parser must return an
        // explicit error whose message teaches the scope form (`// @matured: true` header /
        // `thaw fn`). `matured` stays a reserved keyword token, so this is never a silent accept.
        let src = "nodule demo\nmatured fn k() -> Binary{8} = 0b00000000";
        let err = parse(src).unwrap_err();
        assert!(
            err.message.contains("maturation"),
            "teaching diagnostic must mention maturation, got: {}",
            err.message
        );
        assert!(
            err.message.contains("thaw"),
            "teaching diagnostic must mention `thaw`, got: {}",
            err.message
        );
    }
}
