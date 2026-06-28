//! `mycelium-cli` — the **`myc`** one-command toolchain driver (M-733; E16-1).
//!
//! A single front door over the Mycelium toolchain: `myc init` scaffolds a phylum, `myc build`
//! packages it (the content-addressed spore — M-368), `myc check` type-checks it (parse + check via
//! the L1 front-end), `myc test` runs the available verification, and `myc run` is the (honestly
//! not-yet-wired) execution entry point.
//!
//! ## Error-message quality bar (DN-22 / RFC-0013)
//! Every user-visible failure is a structured [`Report`]: a stable `code`, a human-readable
//! `message`, an optional source `location`, and an actionable `help`. No raw Rust panic ever
//! reaches the user (G2 — never opaque); a failure the driver cannot honestly act on is reported as
//! such, never swallowed and never faked (VR-5).
//!
//! ## Honesty about scope (`Declared`)
//! `init` / `build` / `check` do real end-to-end work. `test` runs `check` and is explicit that a
//! dedicated `.myc` unit-test *runner* does not exist yet (it does not pretend to have run tests
//! that were never written). `run` is **not yet wired** — the project→interpreter pipeline is later
//! work — and says so with an actionable [`Report`] instead of a stub that silently does nothing.

use std::path::{Path, PathBuf};

use mycelium_l1::{check_nodule, parse};
use mycelium_proj::parse_manifest;
use mycelium_spore::{build_spore, explain, Spore};

/// A structured, actionable diagnostic (the DN-22 quality bar; a projection of an RFC-0013
/// diagnostic). It renders as `error[<code>]: <message>` with optional `--> <location>` and
/// `help:` lines — never an opaque internal error (G2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    /// A stable, machine-readable diagnostic code (e.g. `myc-parse`, `myc-build`, `myc-run-unwired`).
    pub code: &'static str,
    /// The human-readable, specific message.
    pub message: String,
    /// An optional `path:line:col` (or `path`) the user can jump to.
    pub location: Option<String>,
    /// An optional actionable next step.
    pub help: Option<String>,
    /// The process exit code this report maps to (sysexits-flavoured; never 0).
    pub exit: u8,
}

impl Report {
    /// A report with a code, message and exit code (no location/help).
    #[must_use]
    pub fn new(code: &'static str, message: impl Into<String>, exit: u8) -> Self {
        Report {
            code,
            message: message.into(),
            location: None,
            help: None,
            exit,
        }
    }

    /// Attach a `path:line:col` (or `path`) location.
    #[must_use]
    pub fn at(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    /// Attach an actionable `help:` line.
    #[must_use]
    pub fn help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Render the multi-line, structured form (no trailing newline).
    #[must_use]
    pub fn render(&self) -> String {
        let mut s = format!("error[{}]: {}", self.code, self.message);
        if let Some(loc) = &self.location {
            s.push_str(&format!("\n  --> {loc}"));
        }
        if let Some(help) = &self.help {
            s.push_str(&format!("\n  help: {help}"));
        }
        s
    }
}

impl std::fmt::Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.render())
    }
}

impl std::error::Error for Report {}

/// `myc init <name>` — scaffold a new phylum named `name` under `parent`, returning the created
/// files. The name must be a simple lowercase identifier (`[a-z][a-z0-9_]*`); a dotted/empty/
/// mixed-case name is refused, never silently normalized (G2). An existing project at the target is
/// refused — `init` never overwrites (G2).
///
/// # Errors
/// [`Report`] (`myc-init-name` / `myc-init-exists` / `myc-io`) on a bad name, a pre-existing project,
/// or a filesystem failure.
pub fn init(parent: &Path, name: &str) -> Result<Vec<PathBuf>, Report> {
    validate_name(name)?;
    let dir = parent.join(name);
    let manifest_path = dir.join("mycelium-proj.toml");
    if manifest_path.exists() {
        return Err(Report::new(
            "myc-init-exists",
            format!("a project already exists at {}", manifest_path.display()),
            66,
        )
        .help(
            "choose a new name or remove the existing project — `myc init` never overwrites (G2)",
        ));
    }
    std::fs::create_dir_all(&dir)
        .map_err(|e| Report::new("myc-io", format!("{}: {e}", dir.display()), 66))?;

    let manifest = scaffold_manifest(name);
    let nodule = scaffold_nodule(name);
    let source_path = dir.join(format!("{name}.myc"));

    write_new(&manifest_path, &manifest)?;
    write_new(&source_path, &nodule)?;
    Ok(vec![manifest_path, source_path])
}

/// `myc build` — build the content-addressed spore for the project at `manifest_path`, returning the
/// built [`Spore`] and its descriptor text (M-368). A missing/ambiguous publish input is surfaced as
/// a structured [`Report`], never a partial artifact (G2).
///
/// # Errors
/// [`Report`] (`myc-io` / `myc-manifest` / `myc-build`) on a read failure, a malformed manifest, or a
/// refused build input.
pub fn build(manifest_path: &Path) -> Result<(Spore, String), Report> {
    let (manifest, project_dir) = load_manifest(manifest_path)?;
    let spore = build_spore(&manifest, &project_dir).map_err(|e| {
        Report::new("myc-build", e.to_string(), e.exit_code())
            .at(project_dir.display().to_string())
            .help("declare the [surface].exports, add a `.myc` source, or pin a dependency `hash` (ADR-003)")
    })?;
    // Compute the descriptor from a borrow, then move `spore` out by value (no clone).
    let descriptor = explain(&spore);
    Ok((spore, descriptor))
}

/// The outcome of [`check_project`]: which nodules type-checked, and the structured failures.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CheckReport {
    /// Source files that parsed and type-checked cleanly.
    pub checked: Vec<String>,
    /// Per-file structured failures (parse or type errors), each with a location (DN-22).
    pub failures: Vec<Report>,
}

impl CheckReport {
    /// Whether every checked file passed.
    #[must_use]
    pub fn ok(&self) -> bool {
        self.failures.is_empty()
    }
}

/// `myc check` — parse and type-check every `.myc` source under the project directory containing
/// `manifest_path`. Each nodule is checked independently (per-nodule scope — honest `Declared`:
/// cross-nodule resolution is the elaborator's job, not re-implemented here). Returns a structured
/// [`CheckReport`]; a parse/type error becomes a located [`Report`] in `failures`, never a panic (G2).
///
/// # Errors
/// [`Report`] (`myc-io`) only when the source tree cannot be walked; per-file check failures are
/// carried in the returned [`CheckReport`], not as an `Err`.
pub fn check_project(manifest_path: &Path) -> Result<CheckReport, Report> {
    let (_, project_dir) = load_manifest(manifest_path)?;
    let sources =
        mycelium_cli_common::walk_myc(&project_dir).map_err(|e| Report::new("myc-io", e, 66))?;
    let mut report = CheckReport::default();
    for path in sources {
        let rel = path
            .strip_prefix(&project_dir)
            .unwrap_or(&path)
            .display()
            .to_string();
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                report
                    .failures
                    .push(Report::new("myc-io", format!("{rel}: {e}"), 66).at(rel.clone()));
                continue;
            }
        };
        match parse(&text) {
            Err(pe) => report.failures.push(
                Report::new("myc-parse", pe.message.clone(), 65)
                    .at(format!("{rel}:{}:{}", pe.pos.line, pe.pos.col))
                    .help("fix the syntax error at the indicated position"),
            ),
            Ok(nodule) => match check_nodule(&nodule) {
                Err(ce) => report.failures.push(
                    Report::new("myc-check", ce.to_string(), 65)
                        .at(rel.clone())
                        .help("resolve the type error reported above"),
                ),
                Ok(_env) => report.checked.push(rel),
            },
        }
    }
    Ok(report)
}

/// `myc run` — **not yet wired** (honest, never-silent). The project→interpreter execution pipeline
/// is later work; this returns an actionable [`Report`] rather than a stub that silently does nothing
/// (VR-5 / G2). The interpreter ([`mycelium_interp`](mycelium-interp)) evaluates Core IR, but the
/// surface-project→run path is not assembled in v0.
///
/// # Errors
/// Always returns [`Report`] (`myc-run-unwired`, exit 70) — `run` has no honest success path yet.
pub fn run(_manifest_path: &Path) -> Result<(), Report> {
    Err(Report::new(
        "myc-run-unwired",
        "running a phylum is not yet wired into `myc`",
        70,
    )
    .help(
        "the project→interpreter execution pipeline is later work; today use `myc check` to \
         type-check and `myc build` to package the spore",
    ))
}

// --- internals ---------------------------------------------------------------------------------

/// Load + parse the manifest at `manifest_path`, returning it with the project directory.
fn load_manifest(manifest_path: &Path) -> Result<(mycelium_proj::Manifest, PathBuf), Report> {
    let text = std::fs::read_to_string(manifest_path).map_err(|e| {
        Report::new("myc-io", format!("{}: {e}", manifest_path.display()), 66)
            .help("run `myc` from a project directory, or pass the manifest path")
    })?;
    let manifest = parse_manifest(&text).map_err(|e| {
        Report::new("myc-manifest", e.to_string(), 2).at(manifest_path.display().to_string())
    })?;
    let project_dir = manifest_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    Ok((manifest, project_dir))
}

/// Validate an `init` name: `[a-z][a-z0-9_]*`. A bad name is refused, never normalized (G2).
fn validate_name(name: &str) -> Result<(), Report> {
    let bad = || {
        Report::new(
            "myc-init-name",
            format!("{name:?} is not a valid phylum name"),
            64,
        )
        .help("use a lowercase identifier: a letter then letters/digits/underscores, e.g. `acme_geometry`")
    };
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() => {}
        _ => return Err(bad()),
    }
    if !chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') {
        return Err(bad());
    }
    Ok(())
}

/// Write `content` to `path`, refusing to clobber an existing file (G2).
fn write_new(path: &Path, content: &str) -> Result<(), Report> {
    if path.exists() {
        return Err(Report::new(
            "myc-init-exists",
            format!("{} already exists", path.display()),
            66,
        ));
    }
    std::fs::write(path, content)
        .map_err(|e| Report::new("myc-io", format!("{}: {e}", path.display()), 66))
}

/// The scaffolded `mycelium-proj.toml` for `name`.
fn scaffold_manifest(name: &str) -> String {
    format!(
        "# Scaffolded by `myc init`. A minimal, gate-clean phylum.\n\
         [project]\n\
         name    = \"{name}\"\n\
         kind    = \"phylum\"\n\
         version = \"0.1.0\"\n\
         license = \"MIT\"\n\
         summary = \"{name} — a new Mycelium phylum.\"\n\
         \n\
         [surface]\n\
         exports = [\"{name}\"]\n"
    )
}

/// The scaffolded starter nodule for `name`.
fn scaffold_nodule(name: &str) -> String {
    format!(
        "// nodule: {name}\n\
         // @summary: {name} — scaffolded by `myc init`; replace with your own definitions.\n\
         nodule {name}\n\
         \n\
         fn answer() => Binary{{8}} =\n  \
         0b0010_1010\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let err = run(Path::new("whatever/mycelium-proj.toml")).unwrap_err();
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
}
