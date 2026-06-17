//! `mycfmt` conformance over the grammar corpus (M-364; contract §9). The corpus is ground truth (the
//! WebAssembly-spec pattern): every `accept/` program must format to a **same-identity** program (C1) and
//! be a **fixed point** under a second format (C2); every `reject/` program must be an explicit error,
//! never a rewrite (G2). Header preservation (C3) is checked against the M-358/M-359 fixtures.

use std::path::{Path, PathBuf};

use mycelium_fmt::{format_source, FmtError};
use mycelium_l1::parse;

fn corpus_dir(which: &str) -> PathBuf {
    // crates/mycelium-fmt/tests -> repo root -> docs/spec/grammar/conformance/<which>
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/spec/grammar/conformance")
        .join(which)
}

fn myc_files(dir: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("read {dir:?}: {e}"))
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "myc"))
        .collect();
    v.sort();
    v
}

#[test]
fn every_accept_program_formats_with_preserved_identity_and_is_idempotent() {
    let dir = corpus_dir("accept");
    let files = myc_files(&dir);
    assert!(!files.is_empty(), "no accept/ corpus found at {dir:?}");
    for f in files {
        let src = std::fs::read_to_string(&f).unwrap();
        let name = f.file_name().unwrap().to_string_lossy().into_owned();

        let formatted = match format_source(&src, None) {
            Ok(r) => r,
            // An out-of-scope refusal is allowed (honest §7 boundary) — but never a parse/header error on
            // an accept/ program (those parse by construction).
            Err(FmtError::OutOfScope(msg)) => {
                eprintln!(
                    "note: {name} is outside mycfmt v0 scope (refused, not rewritten): {msg}"
                );
                continue;
            }
            Err(other) => panic!("{name}: accept/ program errored unexpectedly: {other}"),
        };

        // C1 identity: the formatted output parses to the SAME surface AST as the input.
        let before = parse(&src).expect("accept/ parses");
        let after = parse(&formatted.output).expect("formatted parses");
        assert_eq!(
            before, after,
            "{name}: formatting changed the surface AST (C1)"
        );

        // C2 idempotence: a second format is a byte-for-byte no-op.
        let again = format_source(&formatted.output, None).expect("re-format");
        assert_eq!(
            again.output, formatted.output,
            "{name}: not idempotent (C2)"
        );
        assert!(!again.changed, "{name}: re-format reported a change (C2)");
    }
}

#[test]
fn every_reject_program_is_an_explicit_error_never_a_rewrite() {
    let dir = corpus_dir("reject");
    let files = myc_files(&dir);
    assert!(!files.is_empty(), "no reject/ corpus found at {dir:?}");
    for f in files {
        let src = std::fs::read_to_string(&f).unwrap();
        let name = f.file_name().unwrap().to_string_lossy().into_owned();
        let err = format_source(&src, None)
            .err()
            .unwrap_or_else(|| panic!("{name}: a reject/ program was formatted, not refused (G2)"));
        // A refusal carries a non-zero exit code and produced no output.
        assert!(err.exit_code() >= 2, "{name}: unexpected exit code");
    }
}

#[test]
fn the_header_fixtures_round_trip_or_refuse_explicitly() {
    // M-358/M-359 fixtures: a valid structured header is re-emitted canonically and survives a round-trip;
    // the bad-header fixture is an explicit header error, never a silent drop (C3/G2).
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../mycelium-proj/tests/fixtures");

    let root = std::fs::read_to_string(fixtures.join("root.myc")).unwrap();
    let r = format_source(&root, None).expect("root.myc formats");
    assert!(
        r.output.starts_with("// nodule: geometry.shapes\n"),
        "{}",
        r.output
    );
    assert!(r.output.contains("// @version: 1.2.0"));
    assert!(r.output.contains("// @license: Apache-2.0"));
    // Idempotent.
    assert_eq!(format_source(&r.output, None).unwrap().output, r.output);

    let bad = std::fs::read_to_string(fixtures.join("bad-header.myc")).unwrap();
    let err = format_source(&bad, None).unwrap_err();
    assert_eq!(
        err.exit_code(),
        3,
        "bad-header.myc must be a header error (C3/G2)"
    );
}
