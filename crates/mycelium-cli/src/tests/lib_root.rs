// Tests for `mycelium-cli` lib root (extracted from inline `#[cfg(test)]` per M-797 / CLAUDE.md).
// White-box access via `use crate::*`.
use crate::*;
use std::path::PathBuf;

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
fn run_is_honestly_unwired_not_a_silent_noop() {
    // VR-5 / G2: `myc run` must say it is not wired, with an actionable help line.
    let err = run(std::path::Path::new("whatever/mycelium-proj.toml")).unwrap_err();
    assert_eq!(err.code, "myc-run-unwired");
    assert!(err.help.is_some());
    assert!(err.render().contains("help:"));
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
