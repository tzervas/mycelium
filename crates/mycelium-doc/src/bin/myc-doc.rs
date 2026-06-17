//! `myc-doc` — the M-363 documentation BUILD CLI: project the corpus into the doc-IR, **emit** the
//! HTML/Typst/JSON views, and run the **§4.1 quality-bar lint** over the model.
//!
//! Usage:
//! ```text
//!   myc-doc build  [--repo-root .] [--out target/doc]   # project + emit every view
//!   myc-doc lint   [--repo-root .]                       # run the 8 §4.1 checks (gate)
//!   myc-doc status                                       # print the lint's active status
//! ```
//!
//! Exit codes (mirroring the Wave-A toolchain): `0` ok · `1` an error-severity finding · `64` usage ·
//! `66` I/O. Never-silent (G2): every failure is an explicit message with remediation, never a panic.

use std::path::PathBuf;
use std::process::ExitCode;

use mycelium_doc::build::{emit_all, EPUB_DEFERRAL};
use mycelium_doc::doc_lint::{CheckStatus, Severity};
use mycelium_doc::{build, doc_lint, BuildInput, CHECK_NAMES};

const EX_OK: u8 = 0;
const EX_FINDING: u8 = 1;
const EX_USAGE: u8 = 64;
const EX_IO: u8 = 66;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(code) => ExitCode::from(code),
        Err((code, msg)) => {
            eprintln!("myc-doc: {msg}");
            ExitCode::from(code)
        }
    }
}

fn run(args: &[String]) -> Result<u8, (u8, String)> {
    let Some(cmd) = args.first() else {
        return Err((EX_USAGE, usage()));
    };
    let mut repo_root = PathBuf::from(".");
    let mut out = PathBuf::from("target/doc");
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--repo-root" => {
                i += 1;
                repo_root = PathBuf::from(args.get(i).ok_or((EX_USAGE, usage()))?);
            }
            "--out" => {
                i += 1;
                out = PathBuf::from(args.get(i).ok_or((EX_USAGE, usage()))?);
            }
            other => return Err((EX_USAGE, format!("unknown argument: {other}\n{}", usage()))),
        }
        i += 1;
    }

    match cmd.as_str() {
        "status" => {
            println!(
                "doc quality-bar lint (§4.1): ACTIVE — {} checks run over the M-363 doc-IR",
                CHECK_NAMES.len()
            );
            for name in CHECK_NAMES {
                println!("  · {name}");
            }
            Ok(EX_OK)
        }
        "build" => {
            let input = BuildInput::conventional(&repo_root);
            let model = build(&input).map_err(|e| (EX_IO, format!("build: {e}")))?;
            let arts = emit_all(&model);
            let n = arts
                .write_to(&out)
                .map_err(|e| (EX_IO, format!("emit: {e}")))?;
            println!(
                ">> myc-doc build: projected {} documents ({} nodes) → {} artifacts under {}",
                model.documents.len(),
                model.all_nodes().len(),
                n,
                out.display()
            );
            println!("   {EPUB_DEFERRAL}");
            Ok(EX_OK)
        }
        "lint" => {
            let input = BuildInput::conventional(&repo_root);
            let model = build(&input).map_err(|e| (EX_IO, format!("build: {e}")))?;
            let report = doc_lint::lint(&model);
            print_report(&model, &report);
            if report.has_errors() {
                Err((
                    EX_FINDING,
                    format!(
                        "{} error-severity §4.1 finding(s) — see above (G2: never a silent pass)",
                        report.errors().len()
                    ),
                ))
            } else {
                Ok(EX_OK)
            }
        }
        other => Err((EX_USAGE, format!("unknown command: {other}\n{}", usage()))),
    }
}

fn print_report(model: &mycelium_doc::DocModel, report: &doc_lint::DocLintReport) {
    println!(
        ">> myc-doc §4.1 quality-bar lint over {} documents ({} content-addressed nodes)",
        model.documents.len(),
        model.all_nodes().len()
    );
    for outcome in &report.outcomes {
        let status = match &outcome.status {
            CheckStatus::Active => "active".to_owned(),
            CheckStatus::PartiallyDormant(why) => format!("partially-dormant ({why})"),
            CheckStatus::Dormant(why) => format!("dormant ({why})"),
        };
        let errs = outcome
            .findings
            .iter()
            .filter(|f| f.severity == Severity::Error)
            .count();
        let mark = if errs == 0 { "ok  " } else { "FAIL" };
        println!(
            "  [{mark}] {} — {} [{status}]",
            outcome.name, outcome.summary
        );
        for f in &outcome.findings {
            if f.severity == Severity::Error {
                println!("        error: {} @ {}", f.message, f.anchor);
            }
        }
    }
}

fn usage() -> String {
    "usage: myc-doc <build|lint|status> [--repo-root <dir>] [--out <dir>]".to_owned()
}
