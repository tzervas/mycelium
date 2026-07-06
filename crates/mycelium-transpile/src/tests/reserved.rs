//! Tests for the reserved-word collision guard (`src/reserved.rs`, M-1001), incl. the **drift
//! guard** against `mycelium-l1`'s real lexer.
//!
//! **Guarantee: `Empirical`** for the drift test (it runs the real `mycelium_l1::token::keyword`
//! over every snapshot word); pure/`Declared` for the guard behaviour tests.

use crate::gap::Category;
use crate::reserved::{guard_ident, is_reserved, RESERVED};

/// **Drift guard.** Every word in the [`RESERVED`] snapshot must still be rejected as an identifier
/// by the *real* `mycelium-l1` lexer (`mycelium_l1::token::keyword` returns `Some` for a keyword). A
/// snapshot word that drifts to a *non*-reserved word — the direction that would make the guard
/// over-gap and regress a valid emission — fails here. (The under-gap direction — a new keyword `l1`
/// adds that this snapshot misses — is a residual the vet loop catches as a parse error, not a
/// silent bad emission; it is not asserted here because this crate should not have to track every
/// future keyword addition to stay correct — over-gap is the only regressing direction.)
#[test]
fn snapshot_words_are_all_still_reserved_in_the_lexer() {
    for word in RESERVED {
        assert!(
            mycelium_l1::token::keyword(word).is_some(),
            "reserved-word snapshot drift: `{word}` is in crate::reserved::RESERVED but the real \
             mycelium-l1 lexer no longer treats it as a keyword — the snapshot must be re-synced \
             with crates/mycelium-l1/src/token.rs `fn keyword` (this would otherwise over-gap a \
             now-valid identifier)"
        );
    }
}

/// The snapshot is non-empty and free of accidental duplicates (a duplicate is harmless for
/// `contains` but signals a copy error).
#[test]
fn snapshot_is_nonempty_and_deduplicated() {
    assert!(
        !RESERVED.is_empty(),
        "reserved-word snapshot must not be empty"
    );
    let mut seen = std::collections::BTreeSet::new();
    for w in RESERVED {
        assert!(
            seen.insert(*w),
            "duplicate reserved word in snapshot: `{w}`"
        );
    }
}

/// `is_reserved` / `guard_ident` accept ordinary identifiers and reject reserved words, tagging the
/// rejection [`Category::ReservedWord`] with a diagnostic that names the colliding word (G2).
#[test]
fn guard_rejects_reserved_accepts_ordinary() {
    // Ordinary Rust identifiers that are NOT Mycelium reserved words → accepted.
    for ok in [
        "Ordering",
        "ForageError",
        "reverse",
        "is_lt",
        "NoCandidates",
        "Foo",
        "my_fn",
    ] {
        assert!(!is_reserved(ok), "`{ok}` should not be reserved");
        assert!(
            guard_ident(ok, "test position").is_ok(),
            "`{ok}` should pass the guard"
        );
    }
    // Reserved words that a Rust enum/variant/type could easily be named → rejected as ReservedWord.
    for bad in [
        "Exact",
        "Proven",
        "Empirical",
        "Declared",
        "F16",
        "Binary",
        "swap",
        "fuse",
    ] {
        assert!(is_reserved(bad), "`{bad}` should be reserved");
        let err = guard_ident(bad, "match pattern").expect_err("reserved word must be gapped");
        assert_eq!(err.category, Category::ReservedWord);
        assert!(
            err.reason.contains(bad),
            "the gap reason names the colliding word `{bad}` (never silent): {}",
            err.reason
        );
    }
}
