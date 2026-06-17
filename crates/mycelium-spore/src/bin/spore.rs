//! `spore` — packaging & publishing CLI (M-368; contract `docs/spec/Spore-Build-and-Publish-Contract.md`).
//!
//! Builds a content-addressed `spore` from a `mycelium-proj.toml` project (ADR-013). Identity is the
//! code+deps DAG (ADR-003); metadata is not identity. A missing/ambiguous publish input is an explicit
//! error — **no partial artifact** is ever written (G2).
//!
//! ```text
//! spore build   [--config <mycelium-proj.toml>] [-o <out>]   # build + write the spore descriptor
//! spore explain [--config <mycelium-proj.toml>]              # the identity receipt; write nothing
//! ```
//!
//! Exit codes: 0 ok · 2 manifest error · 3 publish-input error · 64 usage · 66 I/O.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use mycelium_proj::parse_manifest;
use mycelium_spore::{build_spore, explain, Spore};

fn usage() -> ExitCode {
    eprintln!("usage: spore <build|explain> [--config <mycelium-proj.toml>] [-o <out>]");
    ExitCode::from(64)
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(cmd) = args.next() else {
        return usage();
    };

    let mut config: Option<String> = None;
    let mut out: Option<String> = None;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--config" => match args.next() {
                Some(p) => config = Some(p),
                None => return usage(),
            },
            "-o" => match args.next() {
                Some(p) => out = Some(p),
                None => return usage(),
            },
            _ => return usage(),
        }
    }

    let manifest_path = config
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("mycelium-proj.toml"));
    let project_dir = manifest_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    let text = match std::fs::read_to_string(&manifest_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("spore: io-error: {}: {e}", manifest_path.display());
            return ExitCode::from(66);
        }
    };
    let manifest = match parse_manifest(&text) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("spore: manifest-error: {}: {e}", manifest_path.display());
            return ExitCode::from(2);
        }
    };

    let spore = match build_spore(&manifest, &project_dir) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("spore: {e}");
            return ExitCode::from(e.exit_code());
        }
    };

    match cmd.as_str() {
        "explain" => {
            print!("{}", explain(&spore));
            ExitCode::SUCCESS
        }
        "build" => emit_build(&spore, out.as_deref()),
        _ => usage(),
    }
}

/// Emit the spore descriptor (the named-provisional v0 encoding; M-368 §9.1) — the EXPLAIN body prefixed
/// with the identity line, written to `-o <out>` or stdout. (The R2 wire-schema supersedes this.)
fn emit_build(spore: &Spore, out: Option<&str>) -> ExitCode {
    let descriptor = explain(spore);
    match out {
        Some(path) => match std::fs::write(path, &descriptor) {
            Ok(()) => {
                eprintln!("spore: wrote {} ({})", path, spore.id.as_str());
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("spore: io-error: {path}: {e}");
                ExitCode::from(66)
            }
        },
        None => {
            print!("{descriptor}");
            ExitCode::SUCCESS
        }
    }
}
