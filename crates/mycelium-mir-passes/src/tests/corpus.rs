//! Tests for `crate::corpus` — the MEM-4 Increment 1 measurement / Q5 gate (DN-33 §8.1).

use crate::corpus::{measure, measure_standard, standard_corpus};

#[test]
fn standard_corpus_is_a_mix() {
    // The corpus must contain BOTH elision-friendly and elision-neutral terms, so the measured
    // ratio is honest (not a cherry-picked best case).
    let corpus = standard_corpus();
    assert!(corpus.len() >= 8, "corpus should be reasonably sized");
    let report = measure(&corpus).expect("measurement must not fault");
    let with_win = report.rows.iter().filter(|(_, o, e, _)| o > e).count();
    let neutral = report.rows.iter().filter(|(_, o, e, _)| o == e).count();
    assert!(with_win >= 3, "corpus must include elision wins");
    assert!(
        neutral >= 2,
        "corpus must include elision-neutral terms (honest mix)"
    );
}

#[test]
fn q5_gate_elision_reduces_dups_and_preserves_semantics() {
    // THE Q5 GATE: across the representative corpus, borrow elision must (a) preserve semantics for
    // EVERY term (same reclamation multiset, no use-after-free), and (b) measurably reduce the
    // emitted Dup count. Both are required before Increment 2 may be committed (DN-33 §8.1 Q5).
    let report = measure_standard().expect("measurement must not fault");

    assert!(
        report.all_semantics_preserved,
        "every term's elision must be semantics-preserving (Q3)"
    );
    assert!(
        report.elided_dups < report.owned_dups,
        "elision must reduce the aggregate Dup count: owned={}, elided={}",
        report.owned_dups,
        report.elided_dups
    );
    assert!(
        report.reduction_ratio() > 0.0,
        "the Q5 dup-reduction ratio must be positive (got {:.3})",
        report.reduction_ratio()
    );
}

#[test]
fn elided_never_exceeds_owned_per_term() {
    // Per-term monotonicity: elision never INCREASES Dups for any term (it only removes them).
    let report = measure_standard().expect("measurement must not fault");
    for (name, owned, elided, preserved) in &report.rows {
        assert!(
            elided <= owned,
            "term {name}: elision increased Dups ({owned} -> {elided})"
        );
        assert!(
            preserved,
            "term {name}: elision was not semantics-preserving"
        );
    }
}

#[test]
fn report_ratio_is_exact_arithmetic() {
    // The ratio is an exact count ratio (Exact tag): (owned - elided) / owned.
    let report = measure_standard().expect("measurement must not fault");
    let expected = (report.owned_dups - report.elided_dups) as f64 / report.owned_dups as f64;
    assert!((report.reduction_ratio() - expected).abs() < f64::EPSILON);
    assert_eq!(
        report.dups_removed(),
        report.owned_dups - report.elided_dups
    );
}
