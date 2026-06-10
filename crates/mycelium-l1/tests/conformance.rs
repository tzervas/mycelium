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

#[test]
fn reject_corpus_all_fails_explicitly() {
    for path in myc_files("reject") {
        let src = fs::read_to_string(&path).unwrap();
        // A reject fixture must fail — and fail as an explicit ParseError, not a panic (the call
        // returning at all proves no panic; `is_err` proves no silent accept).
        assert!(
            parse(&src).is_err(),
            "{} should be rejected but parsed successfully",
            path.display()
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
