//! `myc` — the one-command Mycelium toolchain driver (M-733).
//!
//! ```text
//! myc init  <name>                 # scaffold a new phylum
//! myc build [--config <manifest>]  # build the content-addressed spore
//! myc check [--config <manifest>]  # parse + type-check every .myc source
//! myc test  [--config <manifest>]  # run the available verification (check)
//! myc run   [--config <manifest>]  # (not yet wired — reports so, never silent)
//! ```
//!
//! Every failure is a DN-22 structured [`Report`](mycelium_cli::Report) — `error[<code>]: …` with a
//! source location and an actionable `help:` line; no raw panic ever reaches the user (G2).
//!
//! Exit codes: 0 ok · 2 manifest · 64 usage · 65 source error · 66 I/O · 70 unwired.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use mycelium_cli::{build, check_project, init, run, Report};

fn usage() -> ExitCode {
    eprintln!(
        "usage:\n  \
         myc init  <name>\n  \
         myc build [--config <manifest>]\n  \
         myc check [--config <manifest>]\n  \
         myc test  [--config <manifest>]\n  \
         myc run   [--config <manifest>]"
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
            Ok(()) => ExitCode::SUCCESS,
            Err(r) => fail(&r),
        }),
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
