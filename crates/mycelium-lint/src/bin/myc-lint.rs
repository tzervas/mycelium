//! `myc-lint` — lint + auto-fix CLI (M-366; contract `docs/spec/Lint-and-Autofix-Contract.md`).
//!
//! Surfaces the M-141 lints + header lints as **actionable, reified, opt-in** fixes (suggest / apply /
//! scaffold). **No silent rewrite** (G2): `--fix` applies only behaviour-preserving `apply` edits — and in
//! v0 there are none (every lint fix is suggest or scaffold; header canonicalization is `mycfmt`'s job), so
//! `--fix` rewrites nothing. A control-flow change (an explicit `swap`, a recovery handler) is always a
//! **scaffold**, never auto-applied (RFC-0014 I1/I5).
//!
//! ```text
//! myc-lint [--project <dir>] [--fix] [--explain] [<file.myc | ->...]
//! ```
//!
//! Exit codes: 0 clean (or warnings only) · 1 an error-severity finding · 64 usage · 66 I/O.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use mycelium_lint::{doc_lint_status, lint_sources, LintReport};
use mycelium_lsp::Severity;

fn usage() -> ExitCode {
    eprintln!("usage: myc-lint [--project <dir>] [--fix] [--explain] [<file.myc | ->...]");
    ExitCode::from(64)
}

fn main() -> ExitCode {
    let mut project: Option<String> = None;
    let mut fix = false;
    let mut explain = false;
    let mut paths: Vec<String> = Vec::new();

    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--project" => match args.next() {
                Some(p) => project = Some(p),
                None => return usage(),
            },
            "--fix" => fix = true,
            "--explain" => explain = true,
            "-" => paths.push("-".to_owned()),
            s if s.starts_with("--") => return usage(),
            s => paths.push(s.to_owned()),
        }
    }

    let sources = match collect_sources(project.as_deref(), &paths) {
        Ok(s) => s,
        Err(code) => return code,
    };
    if sources.is_empty() {
        eprintln!("myc-lint: no .myc sources to lint");
        return usage();
    }

    let report = lint_sources(&sources);
    print_report(&report, explain, fix);

    if report.has_errors() {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn print_report(report: &LintReport, explain: bool, fix: bool) {
    for f in &report.findings {
        let sev = match f.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        let tier = f.fix.as_ref().map_or("", |x| x.tier.as_str());
        let tier_note = if tier.is_empty() {
            String::new()
        } else {
            format!(" (fix: {tier})")
        };
        println!(
            "{}: {sev} {} at {}{tier_note}: {}",
            f.file, f.code, f.at, f.message
        );
        if explain {
            if let Some(fx) = &f.fix {
                println!("    → {}", fx.description);
                if let Some(scaffold) = &fx.scaffold {
                    for line in scaffold.lines() {
                        println!("      {line}");
                    }
                }
            }
        }
    }

    let (apply, suggest, scaffold) = report.tier_counts();
    eprintln!(
        "myc-lint: {} finding(s) across {} file(s) — {apply} apply / {suggest} suggest / {scaffold} scaffold",
        report.findings.len(),
        report.files
    );
    if fix {
        // The never-silent boundary, made explicit: v0 has no behaviour-preserving auto-fix to apply.
        eprintln!(
            "myc-lint: --fix applied 0 edit(s) — v0 has no safe auto-fix (suggest/scaffold only; \
             header canonicalization is `mycfmt`'s job). Nothing was rewritten (G2)."
        );
    }
    if explain {
        eprintln!("myc-lint: {}", doc_lint_status());
    }
}

/// Resolve the sources: `--project <dir>` walks for `.myc`; explicit paths are read (`-` = stdin);
/// neither → the current directory.
fn collect_sources(
    project: Option<&str>,
    paths: &[String],
) -> Result<Vec<(String, String)>, ExitCode> {
    use std::io::Read;
    let mut out = Vec::new();

    if let Some(dir) = project {
        for f in walk(Path::new(dir)).map_err(|e| {
            eprintln!("myc-lint: {e}");
            ExitCode::from(66)
        })? {
            let src = std::fs::read_to_string(&f).map_err(|e| {
                eprintln!("myc-lint: io-error: {}: {e}", f.display());
                ExitCode::from(66)
            })?;
            let rel = f
                .strip_prefix(dir)
                .unwrap_or(&f)
                .to_string_lossy()
                .replace('\\', "/");
            out.push((rel, src));
        }
        return Ok(out);
    }

    if paths.is_empty() {
        // Default: walk the current directory.
        for f in walk(Path::new(".")).map_err(|e| {
            eprintln!("myc-lint: {e}");
            ExitCode::from(66)
        })? {
            if let Ok(src) = std::fs::read_to_string(&f) {
                out.push((f.to_string_lossy().into_owned(), src));
            }
        }
        return Ok(out);
    }

    for p in paths {
        if p == "-" {
            let mut s = String::new();
            if std::io::stdin().read_to_string(&mut s).is_err() {
                eprintln!("myc-lint: io-error: could not read stdin");
                return Err(ExitCode::from(66));
            }
            out.push(("<stdin>".to_owned(), s));
        } else {
            let src = std::fs::read_to_string(p).map_err(|e| {
                eprintln!("myc-lint: io-error: {p}: {e}");
                ExitCode::from(66)
            })?;
            out.push((p.clone(), src));
        }
    }
    Ok(out)
}

/// Collect every `.myc` under `dir` (recursively, sorted); skipping hidden entries and `target/`.
fn walk(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    walk_into(dir, &mut out)?;
    out.sort();
    Ok(out)
}

fn walk_into(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut paths: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    paths.sort();
    for path in paths {
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        if name.starts_with('.') || name == "target" {
            continue;
        }
        if path.is_dir() {
            walk_into(&path, out)?;
        } else if path.extension().is_some_and(|x| x == "myc") {
            out.push(path);
        }
    }
    Ok(())
}
