// Tests for `stream_parse` / `run_stream_parse` (M-820 / DN-57).
//
// Coverage:
//  - A well-formed multi-component stream parses to N clean components (data-driven).
//  - A malformed component within a stream yields an explicit, located error; stream continues.
//  - EOF mid-component (unterminated last component) yields an explicit error.
//  - An empty stream (no `;`-terminated components) yields an explicit error.
//  - A single-component stream (trivial case) parses cleanly.
//
// Guarantee tags:
//  - parse correctness: `Empirical` (backed by the same `mycelium-l1::parse` oracle the tests exercise).
//  - never-silent errors: `Empirical` (every error path asserted below).
//  - I/O error: `Declared` (the `myc-stream-io` path is not exercised here — I/O fault injection
//    would require a mock reader; left for integration-level tests).
use crate::{run_stream_parse, stream_parse};

// --- data-driven corpus -----------------------------------------------------------------------

/// Each entry: (label, input, expected_ok_count, expected_err_count).
/// All inputs are valid-terminated or explicitly broken.
const WELL_FORMED_CORPUS: &[(&str, &str, usize, usize)] = &[
    (
        "single nodule",
        "nodule a; fn f() => Binary{8} = 0b0000_0000;",
        1,
        0,
    ),
    (
        "two nodules",
        "nodule a; fn f() => Binary{8} = 0b0000_0000;\nnodule b; fn g() => Binary{8} = 0b0000_0001;",
        2,
        0,
    ),
    (
        "three nodules — whitespace-free",
        "nodule a;nodule b;nodule c;",
        3,
        0,
    ),
];

/// Each entry: (label, input, expected_ok_count, expected_err_count, error_code).
const MALFORMED_CORPUS: &[(&str, &str, usize, usize, &str)] = &[
    (
        "one malformed component in a two-component stream",
        // First component is broken (bad token §); second is fine.
        "nodule bad; fn f() = §;\nnodule good; fn g() => Binary{8} = 0b1111_1111;",
        1,
        1,
        "myc-stream-parse",
    ),
    (
        "first component malformed, rest clean",
        "nodule x; fn f() = §;\nnodule y;",
        1,
        1,
        "myc-stream-parse",
    ),
];

// --- well-formed tests ------------------------------------------------------------------------

#[test]
fn well_formed_multi_component_stream_parses_all() {
    for (label, input, expected_ok, expected_err) in WELL_FORMED_CORPUS {
        let report = run_stream_parse(input.as_bytes(), "<test>")
            .unwrap_or_else(|e| panic!("[{label}] run_stream_parse failed fatally: {e}"));
        assert_eq!(
            report.parsed_ok, *expected_ok,
            "[{label}] expected {expected_ok} ok components, got {}",
            report.parsed_ok
        );
        assert_eq!(
            report.parsed_err, *expected_err,
            "[{label}] expected {expected_err} failed components, got {}; failures: {:?}",
            report.parsed_err, report.failures
        );
        assert!(
            report.ok(),
            "[{label}] report should be all-ok but has failures: {:?}",
            report.failures
        );
    }
}

// --- malformed-component tests ----------------------------------------------------------------

#[test]
fn malformed_component_yields_explicit_error_stream_continues() {
    for (label, input, expected_ok, expected_err, expected_code) in MALFORMED_CORPUS {
        let report = run_stream_parse(input.as_bytes(), "<test>")
            .unwrap_or_else(|e| panic!("[{label}] run_stream_parse failed fatally: {e}"));
        assert_eq!(
            report.parsed_ok, *expected_ok,
            "[{label}] expected {expected_ok} ok, got {}",
            report.parsed_ok
        );
        assert_eq!(
            report.parsed_err, *expected_err,
            "[{label}] expected {expected_err} errors, got {}",
            report.parsed_err
        );
        assert!(
            !report.failures.is_empty(),
            "[{label}] expected at least one failure"
        );
        let first = &report.failures[0];
        assert_eq!(
            first.code, *expected_code,
            "[{label}] expected error code {expected_code}, got {}",
            first.code
        );
        // Must carry a location (component:line:col) — never opaque (G2 / DN-22).
        assert!(
            first.location.is_some(),
            "[{label}] error must carry a location"
        );
        // Must carry a help line (DN-22 actionable).
        assert!(
            first.help.is_some(),
            "[{label}] error must carry a help line"
        );
    }
}

// --- EOF mid-component test -------------------------------------------------------------------

#[test]
fn eof_mid_component_is_an_explicit_error_not_silent() {
    // A stream that has content after its last `;` (unterminated component).
    let input = "nodule a;\nnodule b"; // `nodule b` has no `;` terminator — EOF arrives mid-component.
    let report = run_stream_parse(input.as_bytes(), "<test-eof>")
        .expect("run_stream_parse must not fatally fail (the first component is fine)");

    // The first component (nodule a;) parsed clean; the second (nodule b) is unterminated.
    assert_eq!(report.parsed_ok, 1, "first component should parse ok");
    assert_eq!(
        report.parsed_err, 1,
        "unterminated component should be one error"
    );
    let err = &report.failures[0];
    assert_eq!(err.code, "myc-stream-eof");
    assert!(
        err.help.is_some(),
        "eof error must carry a help line (DN-22)"
    );
    assert!(
        err.message.contains("unterminated"),
        "eof message must mention 'unterminated': {}",
        err.message
    );
}

// --- empty stream test ------------------------------------------------------------------------

#[test]
fn empty_stream_is_an_explicit_error_not_silent() {
    // An entirely empty stream has no components at all — must not silently succeed.
    let result = run_stream_parse("".as_bytes(), "<test-empty>");
    let err = result.expect_err("empty stream must return Err");
    assert_eq!(err.code, "myc-stream-empty");
    assert!(err.help.is_some(), "empty-stream error must carry help");
}

// --- single-component stream ------------------------------------------------------------------

#[test]
fn single_component_stream_parses_cleanly() {
    let input = "nodule solo; fn answer() => Binary{8} = 0b0010_1010;";
    let report = run_stream_parse(input.as_bytes(), "<single>")
        .expect("single-component stream must not fail fatally");
    assert_eq!(report.parsed_ok, 1);
    assert_eq!(report.parsed_err, 0);
    assert!(report.ok());
}

// --- source_name propagation ------------------------------------------------------------------

#[test]
fn source_name_is_propagated_in_stream_report() {
    let input = "nodule x;";
    let report = run_stream_parse(input.as_bytes(), "myfile.myc").expect("stream parse ok");
    assert_eq!(report.source_name, "myfile.myc");
}

// --- stream_parse raw API lower-level check ---------------------------------------------------

#[test]
fn stream_parse_returns_per_component_results() {
    // Raw `stream_parse` returns a Vec<StreamComponent> — one entry per component.
    let input = "nodule a;nodule b;nodule c;";
    let components =
        stream_parse(input.as_bytes(), "<raw>").expect("stream_parse must not fail fatally");
    assert_eq!(components.len(), 3, "expected 3 components");
    for (i, comp) in components.iter().enumerate() {
        assert!(
            comp.is_ok(),
            "component {} should parse ok, got: {:?}",
            i + 1,
            comp
        );
    }
}
