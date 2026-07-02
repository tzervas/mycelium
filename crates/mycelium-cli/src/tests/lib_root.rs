// Tests for `mycelium-cli` lib root (extracted from inline `#[cfg(test)]` per M-797 / CLAUDE.md).
// White-box access via `use crate::*`.
use crate::*;
use std::path::PathBuf;

/// The committed single-nodule fixture for `myc run` v0 (M-908):
/// `tests/fixtures/run-single-nodule/{mycelium-proj.toml,run_hello.myc}`.
fn run_fixture_manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/run-single-nodule/mycelium-proj.toml")
}

fn scratch(tag: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!(
        "myc-cli-{tag}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&p).unwrap();
    p
}

#[test]
fn init_scaffolds_a_buildable_checkable_phylum() {
    let parent = scratch("init");
    let files = init(&parent, "acme").expect("init succeeds");
    assert_eq!(files.len(), 2);
    let manifest = parent.join("acme").join("mycelium-proj.toml");
    assert!(manifest.exists());

    // The scaffold must BUILD (spore) ...
    let (spore, _descriptor) = build(&manifest).expect("scaffold builds");
    assert_eq!(spore.name, "acme");
    assert!(spore.id.as_str().starts_with("blake3:"));

    // ... and type-CHECK cleanly (parse + check).
    let report = check_project(&manifest).expect("walk succeeds");
    assert!(
        report.ok(),
        "scaffold should check clean: {:?}",
        report.failures
    );
    assert_eq!(report.checked.len(), 1);
}

#[test]
fn init_refuses_a_bad_name_without_normalizing() {
    let parent = scratch("badname");
    for bad in ["Acme", "1geo", "geo-metry", "", "geo.core"] {
        let err = init(&parent, bad).unwrap_err();
        assert_eq!(err.code, "myc-init-name", "{bad:?} should be rejected");
        assert_eq!(err.exit, 64);
    }
}

#[test]
fn init_never_overwrites_an_existing_project() {
    let parent = scratch("noclobber");
    init(&parent, "acme").unwrap();
    let err = init(&parent, "acme").unwrap_err();
    assert_eq!(err.code, "myc-init-exists");
}

#[test]
fn check_reports_a_located_parse_error_not_a_panic() {
    let parent = scratch("badsrc");
    init(&parent, "acme").unwrap();
    let dir = parent.join("acme");
    // Introduce a syntax error in a second nodule.
    std::fs::write(dir.join("broken.myc"), "nodule broken\nfn f() = §\n").unwrap();
    let report = check_project(&dir.join("mycelium-proj.toml")).expect("walk ok");
    assert!(!report.ok());
    let parse_fail = report
        .failures
        .iter()
        .find(|r| r.code == "myc-parse")
        .expect("a parse failure is reported");
    // DN-22: the failure carries a location and a help line, never an opaque panic.
    assert!(parse_fail.location.as_ref().unwrap().contains("broken.myc"));
    assert!(parse_fail.help.is_some());
}

#[test]
fn run_executes_a_committed_single_nodule_fixture_end_to_end() {
    // M-908: `myc run` on the committed single-nodule fixture actually runs `main` through the
    // reference interpreter (`not(0b1010_1010)` == `0b0101_0101`), never a stub / silent no-op.
    let report = run(&run_fixture_manifest()).expect("the fixture runs end-to-end");
    assert_eq!(report.entry, "main");
    assert_eq!(report.source, "run_hello.myc");
    // v0 rendering is a `Debug` dump of the interpreter's `Value` — assert on its substance (the
    // bitwise-negated payload), not a brittle exact string match on internal `Debug` formatting.
    assert!(
        report
            .rendered
            .contains("false, true, false, true, false, true, false, true"),
        "rendered result should show 0b0101_0101's bits: {}",
        report.rendered
    );
}

#[test]
fn run_refuses_zero_myc_sources_explicitly() {
    let parent = scratch("run-nosource");
    std::fs::write(
        parent.join("mycelium-proj.toml"),
        "[project]\nname=\"empty\"\nkind=\"phylum\"\nversion=\"0.1.0\"\nlicense=\"MIT\"\nsummary=\"s\"\n\n[surface]\nexports=[\"empty\"]\n",
    )
    .unwrap();
    let err = run(&parent.join("mycelium-proj.toml")).unwrap_err();
    assert_eq!(err.code, "myc-run-no-source");
    assert!(err.help.is_some());
}

#[test]
fn run_refuses_multi_nodule_projects_explicitly_never_picks_one() {
    // M-908/M-909 boundary: a project with more than one `.myc` source is an explicit, honest
    // refusal — never a silent narrowing to "the first file found" (G2).
    let parent = scratch("run-multinodule");
    init(&parent, "acme").unwrap();
    let dir = parent.join("acme");
    std::fs::write(
        dir.join("second.myc"),
        "// nodule: second\nnodule second;\n\nfn other() => Binary{8} = 0b0000_0000;\n",
    )
    .unwrap();
    let err = run(&dir.join("mycelium-proj.toml")).unwrap_err();
    assert_eq!(err.code, "myc-run-multi-nodule");
    assert!(err.help.unwrap().contains("M-909"));
}

#[test]
fn run_refuses_a_missing_main_entry_never_guessing_another_fn() {
    // `myc init`'s scaffold defines `answer()`, not `main()` — v0 `myc run` must refuse rather
    // than silently running whatever function it finds (G2/VR-5).
    let parent = scratch("run-noentry");
    init(&parent, "acme").unwrap();
    let err = run(&parent.join("acme").join("mycelium-proj.toml")).unwrap_err();
    assert_eq!(err.code, "myc-run-no-entry");
    assert!(err.help.unwrap().contains("answer"));
}

#[test]
fn run_reports_a_located_parse_error_not_a_panic() {
    let parent = scratch("run-badsrc");
    std::fs::write(
        parent.join("mycelium-proj.toml"),
        "[project]\nname=\"badsrc\"\nkind=\"phylum\"\nversion=\"0.1.0\"\nlicense=\"MIT\"\nsummary=\"s\"\n\n[surface]\nexports=[\"badsrc\"]\n",
    )
    .unwrap();
    std::fs::write(parent.join("badsrc.myc"), "nodule badsrc\nfn f() = §\n").unwrap();
    let err = run(&parent.join("mycelium-proj.toml")).unwrap_err();
    assert_eq!(err.code, "myc-parse");
    assert!(err.location.as_ref().unwrap().contains("badsrc.myc"));
}

#[test]
fn report_renders_the_dn22_structured_form() {
    let r = Report::new("myc-parse", "unexpected token", 65)
        .at("a.myc:3:7")
        .help("remove the stray character");
    let s = r.render();
    assert!(s.starts_with("error[myc-parse]: unexpected token"));
    assert!(s.contains("--> a.myc:3:7"));
    assert!(s.contains("help: remove the stray character"));
}
