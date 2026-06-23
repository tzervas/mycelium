//! `spore` — packaging & publishing CLI (M-368/M-732; contract `docs/spec/Spore-Build-and-Publish-Contract.md`).
//!
//! Builds a content-addressed `spore` from a `mycelium-proj.toml` project (ADR-013) and
//! publishes/resolves it against a content-addressed registry (M-732). Identity is the code+deps
//! DAG (ADR-003); metadata is not identity. A missing/ambiguous input is an explicit error —
//! **no partial artifact** is ever written, and a registry never silently overwrites or mis-resolves (G2).
//!
//! ```text
//! spore build    [--config <manifest>] [-o <out>]              # build + write the spore descriptor
//! spore explain  [--config <manifest>]                         # the identity receipt; write nothing
//! spore publish  --registry <dir> [--config <manifest>]        # publish to a content-addressed registry
//!                [--name <n>] [--version <v>]
//! spore resolve  <name> <version|latest> --registry <dir> [-o <out>]   # fetch a hash-verified artifact
//! ```
//!
//! Exit codes: 0 ok · 2 manifest error · 3 publish-input · 4 not-found · 5 integrity · 6 conflict ·
//! 64 usage/unsupported · 66 I/O.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use mycelium_proj::parse_manifest;
use mycelium_spore::{build_spore, explain, registry, Spore};

fn usage() -> ExitCode {
    eprintln!(
        "usage:\n  \
         spore build   [--config <manifest>] [-o <out>]\n  \
         spore explain [--config <manifest>]\n  \
         spore publish --registry <dir> [--config <manifest>] [--name <n>] [--version <v>]\n  \
         spore resolve <name> <version|latest> --registry <dir> [-o <out>]"
    );
    ExitCode::from(64)
}

/// The flags shared across subcommands, parsed once.
#[derive(Default)]
struct Opts {
    config: Option<String>,
    out: Option<String>,
    registry: Option<String>,
    name: Option<String>,
    version: Option<String>,
    positionals: Vec<String>,
}

fn parse_opts(mut args: std::env::Args) -> Option<Opts> {
    let mut o = Opts::default();
    while let Some(a) = args.next() {
        match a.as_str() {
            "--config" => o.config = Some(args.next()?),
            "-o" => o.out = Some(args.next()?),
            "--registry" => o.registry = Some(args.next()?),
            "--name" => o.name = Some(args.next()?),
            "--version" => o.version = Some(args.next()?),
            s if s.starts_with('-') => return None, // an unknown flag is a usage error, never ignored
            _ => o.positionals.push(a),
        }
    }
    Some(o)
}

fn main() -> ExitCode {
    let mut args = std::env::args();
    let _argv0 = args.next();
    let Some(cmd) = args.next() else {
        return usage();
    };
    let Some(opts) = parse_opts(args) else {
        return usage();
    };

    match cmd.as_str() {
        "build" | "explain" | "publish" => run_with_spore(&cmd, &opts),
        "resolve" => run_resolve(&opts),
        _ => usage(),
    }
}

/// The subcommands that first build a spore from a manifest (`build`/`explain`/`publish`).
fn run_with_spore(cmd: &str, opts: &Opts) -> ExitCode {
    // These subcommands take no positional arguments — a stray one is a usage error, never silently
    // ignored (the same never-ignored posture parse_opts applies to unknown flags; G2).
    if !opts.positionals.is_empty() {
        eprintln!(
            "spore: unexpected argument(s): {} — `{cmd}` takes options only",
            opts.positionals.join(" ")
        );
        return usage();
    }
    let manifest_path = opts
        .config
        .as_deref()
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

    match cmd {
        "explain" => {
            print!("{}", explain(&spore));
            ExitCode::SUCCESS
        }
        "build" => emit_build(&spore, opts.out.as_deref()),
        "publish" => run_publish(&spore, opts),
        _ => usage(),
    }
}

/// `spore publish` — publish the built spore's descriptor to the registry under `name@version`.
fn run_publish(spore: &Spore, opts: &Opts) -> ExitCode {
    let Some(registry_dir) = opts.registry.as_deref() else {
        eprintln!("spore: usage: publish requires --registry <dir>");
        return ExitCode::from(64);
    };
    let name = opts.name.clone().unwrap_or_else(|| spore.name.clone());
    // The version is metadata, never guessed: take --version, else the manifest version, else error.
    let Some(version) = opts.version.clone().or_else(|| spore.version.clone()) else {
        eprintln!(
            "spore: publish-input-error: no version to publish under — pass --version or set \
             [project].version (it is never guessed; ADR-003)"
        );
        return ExitCode::from(3);
    };
    let descriptor = explain(spore).into_bytes();
    match registry::publish(Path::new(registry_dir), spore, &descriptor, &name, &version) {
        Ok(r) => {
            let state = if r.already_present {
                "already present"
            } else {
                "published"
            };
            eprintln!(
                "spore: {state} {name}@{version}\n  spore_id: {}\n  artifact: {}\n  object:   {}",
                r.spore_id.as_str(),
                r.artifact.as_str(),
                r.object_path.display()
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("spore: {e}");
            ExitCode::from(e.exit_code())
        }
    }
}

/// `spore resolve <name> <version|latest>` — fetch a hash-verified artifact from the registry.
fn run_resolve(opts: &Opts) -> ExitCode {
    let Some(registry_dir) = opts.registry.as_deref() else {
        eprintln!("spore: usage: resolve requires --registry <dir>");
        return ExitCode::from(64);
    };
    let [name, constraint] = match opts.positionals.as_slice() {
        [n, c] => [n.clone(), c.clone()],
        _ => {
            eprintln!("spore: usage: resolve <name> <version|latest> --registry <dir> [-o <out>]");
            return ExitCode::from(64);
        }
    };
    match registry::resolve(Path::new(registry_dir), &name, &constraint) {
        Ok(r) => {
            eprintln!(
                "spore: resolved {name}@{} (spore_id {}, artifact {})",
                r.version,
                r.spore_id.as_str(),
                r.artifact.as_str()
            );
            match opts.out.as_deref() {
                Some(path) => match std::fs::write(path, &r.bytes) {
                    Ok(()) => {
                        eprintln!("spore: wrote {path}");
                        ExitCode::SUCCESS
                    }
                    Err(e) => {
                        eprintln!("spore: io-error: {path}: {e}");
                        ExitCode::from(66)
                    }
                },
                None => {
                    // The descriptor is UTF-8 text (the explain receipt); stream it to stdout.
                    print!("{}", String::from_utf8_lossy(&r.bytes));
                    ExitCode::SUCCESS
                }
            }
        }
        Err(e) => {
            eprintln!("spore: {e}");
            ExitCode::from(e.exit_code())
        }
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
