//! The diff harness: transpile `crates/mycelium-std-cmp/src/lib.rs` and characterize the result
//! against the hand-refined ground-truth twin `lib/std/cmp.myc` (DN-66 §3.1).
//!
//! **This is a characterization, not an equality check.** The kickoff brief is explicit: the
//! twin is a hand-refined, ~1/10th-scale, *structurally distinct* narrower surface over the full
//! Rust crate — a raw text/name diff would diverge massively, and that divergence is *expected*,
//! not a bug in either artifact. So this harness classifies every item on each side
//! (matched / refined / absent, or matched / refined / flagged) rather than asserting the two
//! texts (or item sets) are equal.
//!
//! **Guarantee: `Declared`.** Item-name extraction from `lib/std/cmp.myc` uses a lightweight
//! line-prefix scan (`type <ident>`, `fn <ident>`) — a hand-rolled stand-in for the "lightweight
//! regex" the kickoff brief describes (no `regex` crate dependency was added for this — see the
//! final report FLAG), not a Mycelium parser. The same extraction is run over this transpiler's
//! *own* emitted `.myc` text for an apples-to-apples comparison (both sides are the same
//! heuristic, so a name that appears on one side but not the other is a genuine textual
//! difference, not an artifact of using two different extraction methods).

use crate::transpile::transpile_file;
use std::collections::BTreeSet;
use std::path::PathBuf;

/// One `type`/`fn` name pulled from a `.myc` source text, via the line-prefix scan described in
/// the module docs above.
fn extract_item_names(text: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for raw_line in text.lines() {
        let line = raw_line.trim_start();
        for prefix in ["type ", "fn "] {
            if let Some(rest) = line.strip_prefix(prefix) {
                if let Some(name) = first_ident(rest) {
                    names.insert(name);
                }
            }
        }
    }
    names
}

fn first_ident(s: &str) -> Option<String> {
    let mut out = String::new();
    for c in s.chars() {
        if c.is_alphanumeric() || c == '_' {
            out.push(c);
        } else {
            break;
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

/// Classification of one name against the "other side"'s name set.
#[derive(Debug, PartialEq, Eq)]
enum Class {
    /// The exact name appears on both sides.
    Matched,
    /// The name does not appear verbatim on the other side. Called "refined" from the twin's
    /// perspective (a hand-refined rename/restructure) and "flagged" from the emitted side's
    /// perspective (present here, no twin counterpart) — see `classify_all` below for which
    /// label is used in which direction.
    Unmatched,
}

fn classify_all(names: &BTreeSet<String>, other: &BTreeSet<String>) -> Vec<(String, Class)> {
    names
        .iter()
        .map(|n| {
            let class = if other.contains(n) {
                Class::Matched
            } else {
                Class::Unmatched
            };
            (n.clone(), class)
        })
        .collect()
}

fn std_cmp_rust_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../mycelium-std-cmp/src/lib.rs")
}

fn std_cmp_twin_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../lib/std/cmp.myc")
}

/// Regression guard (High finding, G2/DN-34 §4) over the *real* target crate: transpiling
/// `mycelium-std-cmp/src/lib.rs` must never emit a fabricated `from(...)` call for the 12
/// numeric-widening `impl Widen<..> for ..` blocks (Rust body `<T>::from(self)`) — each must be
/// gapped instead, since `from` is not a confirmed Mycelium builtin (no grammar production for it
/// in `docs/spec/grammar/mycelium.ebnf`, only prose mentions). An earlier iteration collapsed the
/// qualified `<T>::from` call to its last segment and checked in exactly this fabricated text;
/// this test pins the fix against a regression.
#[test]
fn widen_impls_are_gapped_not_fabricated_in_real_crate() {
    let rust_path = std_cmp_rust_path();
    assert!(rust_path.is_file(), "missing {}", rust_path.display());
    let (emitted_myc, report) =
        transpile_file(&rust_path).unwrap_or_else(|e| panic!("transpile failed: {e}"));

    assert!(
        !emitted_myc.contains("from("),
        "emitted .myc text must never contain a fabricated `from(...)` call (from is not a \
         Mycelium builtin — G2/DN-34 §4), got:\n{emitted_myc}"
    );
    assert!(
        !report
            .emitted_items
            .iter()
            .any(|n| n.starts_with("impl Widen")),
        "no `impl Widen[...]` block should be in emitted_items — its conversion-op body has no \
         established Mycelium surface form and must be gapped, not emitted; got {:?}",
        report.emitted_items
    );
    let widen_gap_count = report
        .gaps
        .iter()
        .filter(|g| g.snippet.contains("Widen") && g.reason.contains("from"))
        .count();
    assert!(
        widen_gap_count >= 12,
        "expected at least the 12 numeric-widening Widen impls (u8/u16/u32/u64 chain + bool) to \
         be gapped for their unmappable `from(...)`-call body, got {widen_gap_count}"
    );
}

#[test]
fn diff_against_std_cmp_twin() {
    let rust_path = std_cmp_rust_path();
    let twin_path = std_cmp_twin_path();
    assert!(rust_path.is_file(), "missing {}", rust_path.display());
    assert!(twin_path.is_file(), "missing {}", twin_path.display());

    let (emitted_myc, report) =
        transpile_file(&rust_path).unwrap_or_else(|e| panic!("transpile failed: {e}"));
    let twin_text = std::fs::read_to_string(&twin_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", twin_path.display()));

    // The transpiler must be doing genuine, partial work: neither an all-pass-through nor an
    // all-drop. (KNOWN HARD GAPS — traits/macros/structs — must show up; the enum + at least one
    // fn must still land.)
    assert!(
        !report.gaps.is_empty(),
        "expected a non-empty gap report (traits/macros/structs are known hard gaps)"
    );
    assert!(
        !report.emitted_items.is_empty(),
        "expected a non-empty emitted set (at least `Ordering` + one fn should land)"
    );
    assert!(
        report.emitted_items.iter().any(|n| n == "Ordering"),
        "expected the `Ordering` enum to be emitted (a C-like enum with no generics/derive \
         issues blocking it — only the DeriveAttr sub-gap for its #[derive(..)])"
    );

    let emitted_names = extract_item_names(&emitted_myc);
    let twin_names = extract_item_names(&twin_text);
    assert!(
        !emitted_names.is_empty(),
        "extraction over our own output found nothing"
    );
    assert!(
        !twin_names.is_empty(),
        "extraction over the twin found nothing"
    );

    // Twin -> emitted direction: every twin item is classified matched or refined(unmatched);
    // "absent" (as named in the task brief) is exactly `Class::Unmatched` from the twin's view —
    // the twin is refined/renamed relative to the Rust source, so an unmatched twin name means
    // "this refinement has no verbatim counterpart in our output", not "lost".
    let twin_classified = classify_all(&twin_names, &emitted_names);
    assert_eq!(
        twin_classified.len(),
        twin_names.len(),
        "every twin item must be classified — none silently skipped"
    );
    let twin_matched = twin_classified
        .iter()
        .filter(|(_, c)| *c == Class::Matched)
        .count();
    let twin_refined = twin_classified.len() - twin_matched;

    // Emitted -> twin direction: matched or flagged (present in our output, no twin counterpart —
    // expected, since we transpile the full ~10x-larger Rust surface the twin doesn't cover).
    let emitted_classified = classify_all(&emitted_names, &twin_names);
    assert_eq!(
        emitted_classified.len(),
        emitted_names.len(),
        "every emitted item must be classified — none silently skipped"
    );
    let emitted_matched = emitted_classified
        .iter()
        .filter(|(_, c)| *c == Class::Matched)
        .count();
    let emitted_flagged = emitted_classified.len() - emitted_matched;

    // `Ordering` (the type) and `reverse` (the fn) are the two names genuinely shared between
    // the Rust source and the hand-refined twin — assert the matched set is non-trivial (catches
    // a regression that would silently break the one real correspondence between the two
    // surfaces), without asserting a specific count (the twin is free to keep evolving, DN-66).
    assert!(
        twin_matched >= 1,
        "expected at least one twin item (e.g. `Ordering`) to match the emitted set verbatim; \
         twin names: {twin_names:?}, emitted names: {emitted_names:?}"
    );

    // Sanity: divergence is the expected norm (twin is ~1/10th, structurally distinct — DN-66
    // §3.1), so the *unmatched* count on both sides should dominate. This is the harness's core
    // assertion: it characterizes divergence rather than penalizing it.
    assert!(
        twin_refined > 0,
        "expected some twin items with no verbatim counterpart in the emitted output (the twin \
         is a hand-refined rename/restructure, not a subset) — got 0, which would be suspicious"
    );
    assert!(
        emitted_flagged > 0,
        "expected some emitted items with no twin counterpart (the emitted surface spans far \
         more of the Rust crate than the twin covers) — got 0, which would be suspicious"
    );

    eprintln!(
        "diff characterization: twin={} (matched={twin_matched}, refined/unmatched={twin_refined}); \
         emitted={} (matched={emitted_matched}, flagged/unmatched={emitted_flagged}); \
         total_items={}, emitted_items={}, gaps={}, expressible={:.1}%",
        twin_names.len(),
        emitted_names.len(),
        report.total_top_level_items,
        report.emitted_items.len(),
        report.gaps.len(),
        report.expressible_fraction() * 100.0,
    );
}
