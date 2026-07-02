//! `myc` — the one-command Mycelium toolchain driver (M-733).
//!
//! ```text
//! myc init  <name>                 # scaffold a new phylum
//! myc build [--config <manifest>]  # build the content-addressed spore
//! myc check [--config <manifest>]  # parse + type-check every .myc source
//! myc test  [--config <manifest>]  # run the available verification (check)
//! myc run   [--config <manifest>]  # run a single-nodule project's `main` (M-908 v0)
//! myc --stream [<file>]            # parse a `;`-terminated component stream (M-820/DN-57)
//! ```
//!
//! Every failure is a DN-22 structured [`Report`](mycelium_cli::Report) — `error[<code>]: …` with a
//! source location and an actionable `help:` line; no raw panic ever reaches the user (G2).
//!
//! Exit codes: 0 ok · 2 manifest · 64 usage · 65 source/eval error · 66 I/O · 70 unsupported
//! (multi-nodule project, or a program outside the evaluation-complete fragment — M-909/RFC-0007).

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use mycelium_cli::{build, check_project, init, run, run_stream_parse, Report};

fn usage() -> ExitCode {
    eprintln!(
        "usage:\n  \
         myc init  <name>\n  \
         myc build [--config <manifest>]\n  \
         myc check [--config <manifest>]\n  \
         myc test  [--config <manifest>]\n  \
         myc run   [--config <manifest>]\n  \
         myc --stream [<file>]"
    );
    ExitCode::from(64)
}

/// Print a [`Report`] to stderr and return its exit code.
fn fail(r: &Report) -> ExitCode {
    eprintln!("{}", r.render());
    ExitCode::from(r.exit)
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(cmd) = args.next() else {
        return usage();
    };
    let rest: Vec<String> = args.collect();

    match cmd.as_str() {
        "init" => match rest.as_slice() {
            [name] => match init(Path::new("."), name) {
                Ok(files) => {
                    println!("created {} file(s):", files.len());
                    for f in files {
                        println!("  {}", f.display());
                    }
                    println!("next: cd {name} && myc check");
                    ExitCode::SUCCESS
                }
                Err(r) => fail(&r),
            },
            _ => usage(),
        },
        "build" => with_manifest(&rest, cmd_build),
        "check" => with_manifest(&rest, cmd_check),
        "test" => with_manifest(&rest, cmd_test),
        "run" => with_manifest(&rest, |m| match run(m) {
            Ok(report) => {
                println!("{}", report.rendered);
                eprintln!("myc: ran `{}` in {}", report.entry, report.source);
                ExitCode::SUCCESS
            }
            Err(r) => fail(&r),
        }),
        "--stream" => cmd_stream(&rest),
        _ => usage(),
    }
}

/// Resolve the `--config <manifest>` flag (default `mycelium-proj.toml`) and dispatch.
fn with_manifest(rest: &[String], f: impl FnOnce(&Path) -> ExitCode) -> ExitCode {
    let mut manifest = PathBuf::from("mycelium-proj.toml");
    let mut it = rest.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--config" => match it.next() {
                Some(p) => manifest = PathBuf::from(p),
                None => return usage(),
            },
            _ => return usage(),
        }
    }
    f(&manifest)
}

fn cmd_build(manifest: &Path) -> ExitCode {
    match build(manifest) {
        Ok((spore, descriptor)) => {
            print!("{descriptor}");
            eprintln!("myc: built {} ({})", spore.name, spore.id.as_str());
            ExitCode::SUCCESS
        }
        Err(r) => fail(&r),
    }
}

fn cmd_check(manifest: &Path) -> ExitCode {
    match check_project(manifest) {
        Ok(report) => {
            for r in &report.failures {
                eprintln!("{}\n", r.render());
            }
            if report.ok() {
                eprintln!("myc: {} nodule(s) checked clean", report.checked.len());
                ExitCode::SUCCESS
            } else {
                eprintln!(
                    "myc: {} checked, {} failed",
                    report.checked.len(),
                    report.failures.len()
                );
                ExitCode::from(65)
            }
        }
        Err(r) => fail(&r),
    }
}

fn cmd_test(manifest: &Path) -> ExitCode {
    // `test` runs the available verification (type-check). Honest (VR-5): a dedicated `.myc`
    // unit-test runner does not exist yet — this does not pretend to have run user-authored tests.
    let code = cmd_check(manifest);
    eprintln!(
        "myc: note — `test` ran the type-check verification; a dedicated .myc unit-test runner is \
         future work (no user tests were discovered or executed)."
    );
    code
}

/// `myc --stream [<file>]` — parse a `;`-terminated Mycelium component stream (M-820 / DN-57).
///
/// Without a file argument, reads from stdin (`<stdin>`). With a file argument, opens and reads
/// that file. The source is lexed once and the token stream is segmented at `nodule` header tokens
/// into per-nodule components, each parsed with `mycelium_l1::parse`. The split is token-driven,
/// so it is comment-/string-safe by construction (a `nodule`/`;` inside a comment is never a token;
/// DN-57 §2). v0 I/O is whole-input-buffered (`Declared` — see [`mycelium_cli::stream_parse`]).
///
/// Every malformed component surfaces an explicit error with a component:line:col location (G2).
/// An unterminated component (its last item has no `;` before the next `nodule`/EOF) is likewise an
/// explicit error, never a silent partial accept (G2 / DN-57 §3.1).
///
/// Exit 0 on all-green; exit 65 if any component failed (or on lex error); exit 66 on I/O error.
fn cmd_stream(rest: &[String]) -> ExitCode {
    // Parse the optional file argument; reject anything else (unknown flags) as usage.
    let (reader, source_name): (Box<dyn std::io::Read>, String) = match rest {
        [] => (Box::new(std::io::stdin()), "<stdin>".to_owned()),
        [path] if !path.starts_with('-') => match std::fs::File::open(path) {
            Ok(f) => (Box::new(f), path.clone()),
            Err(e) => {
                let r = Report::new("myc-stream-io", format!("{path}: {e}"), 66)
                    .help("check that the file path is correct and the file is readable");
                return fail(&r);
            }
        },
        _ => return usage(),
    };

    match run_stream_parse(reader, &source_name) {
        Err(r) => fail(&r),
        Ok(report) => {
            // Print any failures to stderr, each as a structured DN-22 report.
            for f in &report.failures {
                eprintln!("{}\n", f.render());
            }
            if report.ok() {
                eprintln!(
                    "myc: stream `{}` — {} component(s) parsed clean",
                    report.source_name, report.parsed_ok,
                );
                ExitCode::SUCCESS
            } else {
                eprintln!(
                    "myc: stream `{}` — {} ok, {} failed",
                    report.source_name, report.parsed_ok, report.parsed_err,
                );
                ExitCode::from(65)
            }
        }
    }
}
