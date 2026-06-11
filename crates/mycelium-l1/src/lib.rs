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
//! Still ahead: the Maranget match compiler and full L1-in-Core-IR (the RFC-0001 revision).
//!
//! Honesty: every malformed input is an explicit [`ParseError`] with a source position — the
//! parser never panics and never silently accepts (S5/G2). The lexer disambiguates the one tricky
//! token (`<` opening a ternary literal vs a type-argument list) by lookahead, and a malformed
//! ternary literal is an explicit error, not a silent truncation.

pub mod ast;
pub mod checkty;
pub mod elab;
pub mod error;
pub mod eval;
pub mod lexer;
pub mod parse;
pub mod token;
pub mod totality;

pub use ast::Colony;
pub use checkty::{check_colony, CheckError, Env, Ty};
pub use elab::{elaborate, ElabError};
pub use error::ParseError;
pub use eval::{Evaluator, L1Error, L1Value};
pub use parse::parse;
pub use totality::Totality;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BaseType, Expr, Item, Literal};

    #[test]
    fn parses_a_colony_with_a_swap() {
        let src = "colony demo\nfn f(x: Binary{8}) -> Ternary{6} =\n  swap(x, to: Ternary{6}, policy: rt)";
        let colony = parse(src).expect("parses");
        assert_eq!(colony.path.0, vec!["demo"]);
        assert_eq!(colony.items.len(), 1);
        let Item::Fn(f) = &colony.items[0] else {
            panic!("expected a fn item");
        };
        assert!(!f.matured);
        assert!(matches!(f.body, Expr::Swap { .. }));
        assert_eq!(f.sig.ret.base, BaseType::Ternary(6));
    }

    #[test]
    fn a_swap_without_policy_is_an_explicit_error() {
        // S1/WF2: the policy is mandatory; its absence is a diagnostic, never a silent accept.
        let src = "colony demo\nfn f(x: Binary{8}) -> Ternary{6} = swap(x, to: Ternary{6})";
        let err = parse(src).unwrap_err();
        assert!(err.message.contains("policy"), "got: {}", err.message);
    }

    #[test]
    fn a_reserved_word_is_not_a_usable_identifier() {
        let src = "colony demo\nfn colony(x: Binary{8}) -> Binary{8} = x";
        assert!(parse(src).is_err());
    }

    #[test]
    fn a_malformed_ternary_literal_is_explicit() {
        let src = "colony demo\nfn f() -> Ternary{3} = <+x->";
        let err = parse(src).unwrap_err();
        assert!(err.message.contains("non-trit"), "got: {}", err.message);
    }

    #[test]
    fn matured_and_literals_parse() {
        let src = "colony demo\nmatured fn k() -> Binary{8} = 0b1011_0010";
        let colony = parse(src).unwrap();
        let Item::Fn(f) = &colony.items[0] else {
            panic!("fn");
        };
        assert!(f.matured);
        assert!(matches!(&f.body, Expr::Lit(Literal::Bin(s)) if s == "1011_0010"));
    }
}
