//! `load_report` — the `index.json` round-trip (M-1016): write a report, load it back, and confirm
//! it is exactly the report that was written (the read-side twin of `determinism.rs`'s write-side
//! byte-identical check).

use crate::load::load_report;
use crate::tests::fixture::{temp_dir, write_corpus};
use crate::{build_tero_index, write_json};

#[test]
fn a_written_report_loads_back_identical() {
    let root = temp_dir("load-roundtrip");
    write_corpus(&root, true); // defects included: exercises duplicate ids, missing fields, etc.
    let written = build_tero_index(&root).unwrap();

    let out = temp_dir("load-roundtrip-out");
    write_json(&written, &out).unwrap();

    let loaded = load_report(&out.join("index.json")).unwrap();
    assert_eq!(loaded.items, written.items);
    assert_eq!(loaded.flagged, written.flagged);
}

#[test]
fn loading_a_missing_file_is_an_io_error_not_a_silent_empty_report() {
    let root = temp_dir("load-missing");
    let err = load_report(&root.join("does-not-exist.json")).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
}

#[test]
fn loading_malformed_json_is_an_invalid_data_error() {
    let root = temp_dir("load-malformed");
    std::fs::write(root.join("bad.json"), "{ not json").unwrap();
    let err = load_report(&root.join("bad.json")).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
}

#[test]
fn loading_ignores_the_top_level_fields_it_does_not_round_trip() {
    // `generated`/`item_tag`/`siblings` are the crate's own constants, not read back — a payload
    // that carries only `items`/`flagged` (the minimal shape `write_json` always emits a superset
    // of) must still load cleanly.
    let root = temp_dir("load-minimal");
    std::fs::write(root.join("minimal.json"), r#"{"items": [], "flagged": []}"#).unwrap();
    let loaded = load_report(&root.join("minimal.json")).unwrap();
    assert!(loaded.items.is_empty());
    assert!(loaded.flagged.is_empty());
}
