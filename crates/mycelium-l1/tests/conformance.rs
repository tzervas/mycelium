//! The **parser conformance gate** (RFC-0006 §4.3; the WebAssembly-spec pattern, T3.1-B): the L1
//! parser must **accept** every program under `docs/spec/grammar/conformance/accept/` and
//! **reject** every program under `…/reject/` with an explicit [`ParseError`] — never a panic,
//! never a silent accept. The corpus is the ground truth; this test makes the grammar artifact
//! and the parser agree.

use std::fs;
use std::path::PathBuf;

use mycelium_l1::parse;

fn corpus_dir(kind: &str) -> PathBuf {
    // crate dir → repo root → the grammar conformance corpus.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/spec/grammar/conformance")
        .join(kind)
}

fn myc_files(kind: &str) -> Vec<PathBuf> {
    let dir = corpus_dir(kind);
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("reading {}: {e}", dir.display()))
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "myc"))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "no .myc fixtures in {}", dir.display());
    files
}

#[test]
fn accept_corpus_all_parses() {
    for path in myc_files("accept") {
        let src = fs::read_to_string(&path).unwrap();
        match parse(&src) {
            Ok(_) => {}
            Err(e) => panic!("{} should parse but failed: {e}", path.display()),
        }
    }
}

/// Per-file expected-error fragments (A4). Each reject fixture must fail for the *intended*
/// reason: asserting only `is_err()` would let a fixture pass on an unintended failure (e.g. a
/// lexer error masking the grammar violation the fixture is meant to exercise). Each entry maps a
/// `reject/NN-*.myc` filename to a distinctive, stable fragment of the real `ParseError` message
/// its rejection must contain — making the corpus self-policing.
///
/// Every reject fixture must have an entry here; [`reject_corpus_all_fails_explicitly`] fails if a
/// new fixture lacks one, so the table cannot silently fall behind the corpus.
const REJECT_EXPECTED: &[(&str, &str)] = &[
    (
        "01-no-nodule-header.myc",
        "expected a `nodule` header to open the program",
    ),
    ("02-swap-missing-policy.myc", "a swap is never silent"),
    ("03-unclosed-brace.myc", "expected `}` to close the match"),
    (
        "04-bad-trit.myc",
        "balanced-ternary literal has non-trit glyph",
    ),
    ("05-reserved-word-ident.myc", "expected an identifier"),
    ("06-missing-arrow.myc", "expected `->` and a result type"),
    (
        "07-empty.myc",
        "expected a `nodule` header to open the program",
    ),
    ("08-imperative-while.myc", "`while` is not a Mycelium form"),
    (
        "09-default-missing-paradigm.myc",
        "expected `paradigm` after `default`",
    ),
    (
        "10-reserved-not-active.myc",
        "expected a `nodule` header to open the program",
    ),
    ("11-matured-fn-retired.myc", "maturation is declared per"),
];

#[test]
fn reject_corpus_all_fails_explicitly() {
    for path in myc_files("reject") {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .expect("fixture has a UTF-8 name");
        let src = fs::read_to_string(&path).unwrap();
        // A reject fixture must fail — and fail as an explicit ParseError, not a panic (the call
        // returning at all proves no panic; the `Err` arm proves no silent accept).
        let err = match parse(&src) {
            Ok(_) => panic!(
                "{} should be rejected but parsed successfully",
                path.display()
            ),
            Err(e) => e,
        };
        // …and it must fail for the *intended* reason: a new fixture with no entry is a hard
        // failure (the gate can't grow blind spots), and a fixture failing for an unexpected
        // reason is caught instead of silently passing on `is_err()` alone.
        let Some((_, expected)) = REJECT_EXPECTED.iter().find(|(f, _)| *f == name) else {
            panic!(
                "{name} has no expected-error entry in REJECT_EXPECTED — every reject fixture must \
                 declare the distinctive fragment its rejection message must contain (A4)"
            );
        };
        let msg = err.to_string();
        assert!(
            msg.contains(expected),
            "{name} rejected for an unexpected reason:\n  expected message to contain: {expected:?}\n  \
             actual message: {msg:?}\n(if the fixture or diagnostic legitimately changed, update \
             REJECT_EXPECTED)"
        );
    }
}

/// The accept/reject split is meaningful: at least one fixture in each bucket, and the buckets are
/// disjoint in outcome (guards against a vacuous gate).
#[test]
fn the_gate_is_non_vacuous() {
    assert!(!myc_files("accept").is_empty());
    assert!(!myc_files("reject").is_empty());
}

/// Every entry in [`REJECT_EXPECTED`] must correspond to an actual fixture file in the reject
/// corpus — an orphaned entry (pointing at a deleted or renamed fixture) would silently pass
/// while exercising nothing, creating a blind spot the gate is supposed to close. Mutant-witness:
/// adding a `REJECT_EXPECTED` entry for a non-existent file trips this test, keeping the table
/// and the corpus in sync **in both directions** (A4 bidirectional integrity).
#[test]
fn reject_expected_table_has_no_orphaned_entries() {
    let existing: std::collections::BTreeSet<String> = myc_files("reject")
        .into_iter()
        .map(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .expect("fixture has a UTF-8 name")
                .to_owned()
        })
        .collect();
    for (name, _) in REJECT_EXPECTED {
        assert!(
            existing.contains(*name),
            "REJECT_EXPECTED entry {name:?} has no corresponding reject fixture — \
             either add the fixture or remove the orphaned entry (A4 bidirectional integrity)"
        );
    }
}
