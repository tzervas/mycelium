//! `myc-check` — the correctness/type-check driver CLI (M-365; contract
//! `docs/spec/Myc-Check-Driver-Contract.md`). The prototype grown up: it keeps the single-file **oracle**
//! mode (the KC-2 LLM-harness contract — exit 2 parse / 3 check / `--expect-main`) and adds a **project**
//! mode that checks a whole `phylum`/program and aggregates diagnostics routed via the M-362 baseline.
//!
//! ```text
//! myc-check [--expect-main <ret-type>] <file.myc | ->          # oracle (single file)
//! myc-check --project <dir> | --config <mycelium-proj.toml> [--explain]   # whole project (CI gate)
//! ```
//!
//! Exit codes: 0 ok · 2 parse error · 3 check error · 5 project-resolution error · 64 usage · 66 I/O.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use mycelium_check::{check_project, check_sources, FindingKind, Report};
use mycelium_cli_common::{read_source, Args};
use mycelium_l1::ast::{Item, TypeRef};
use mycelium_l1::{check_nodule, parse};

fn usage() -> ExitCode {
    eprintln!(
        "usage: myc-check [--expect-main <ret-type>] <file.myc | ->\n       \
         myc-check --project <dir> | --config <mycelium-proj.toml> [--explain]"
    );
    ExitCode::from(64)
}

fn main() -> ExitCode {
    let mut expect_main: Option<String> = None;
    let mut project: Option<String> = None;
    let mut config: Option<String> = None;
    let mut explain = false;
    let mut path: Option<String> = None;

    let mut args = Args::from_env();
    while let Some(a) = args.next() {
        match a.as_str() {
            "--expect-main" => match args.value() {
                Some(t) => expect_main = Some(t),
                None => return usage(),
            },
            "--project" => match args.value() {
                Some(p) => project = Some(p),
                None => return usage(),
            },
            "--config" => match args.value() {
                Some(p) => config = Some(p),
                None => return usage(),
            },
            "--explain" => explain = true,
            _ if path.is_none() => path = Some(a),
            _ => return usage(),
        }
    }

    // Project mode (explicit) — the whole-phylum CI gate.
    if project.is_some() || config.is_some() {
        if path.is_some() || expect_main.is_some() {
            return usage(); // project mode does not take a file/--expect-main
        }
        let dir = project
            .map(PathBuf::from)
            .or_else(|| {
                config.as_deref().map(|c| {
                    Path::new(c)
                        .parent()
                        .filter(|p| !p.as_os_str().is_empty())
                        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
                })
            })
            .unwrap_or_else(|| PathBuf::from("."));
        return run_project(&dir, explain);
    }

    // Oracle mode (single file) — back-compatible with the prototype's exact contract.
    let Some(path) = path else { return usage() };
    run_oracle(&path, expect_main.as_deref())
}

/// Project mode: check every `.myc` under `dir`, aggregate, exit non-zero on any error (CI gate).
fn run_project(dir: &Path, explain: bool) -> ExitCode {
    match check_project(dir) {
        Ok(report) => {
            print_report(&report, explain);
            ExitCode::from(report.exit_code())
        }
        Err(e) => {
            eprintln!("myc-check: {e}");
            ExitCode::from(5)
        }
    }
}

fn print_report(report: &Report, explain: bool) {
    for f in &report.findings {
        let kind = match f.kind {
            FindingKind::Parse => "parse-error",
            FindingKind::Check => "check-error",
        };
        let at = if f.site.is_empty() {
            String::new()
        } else {
            format!(" in `{}`", f.site)
        };
        if explain && f.kind == FindingKind::Check {
            println!(
                "{}: {kind}{at} [level={:?} route={}]: {}",
                f.file,
                f.level,
                f.route.as_deref().unwrap_or("-"),
                f.message
            );
        } else {
            println!("{}: {kind}{at}: {}", f.file, f.message);
        }
    }
    if report.is_ok() {
        println!("ok: {} file(s) checked, no findings", report.files_checked);
    } else {
        eprintln!(
            "myc-check: {} finding(s) across {} file(s)",
            report.findings.len(),
            report.files_checked
        );
    }
}

/// Oracle mode: the prototype's exact behavior (M-002/KC-2 harness contract). A single file (or `-`),
/// optional `--expect-main`, machine-readable first line, exit 2 (parse) / 3 (check) / 0 (ok).
fn run_oracle(path: &str, expect_main: Option<&str>) -> ExitCode {
    // `read_source` prints the same `io-error: …` line (no tool-name tag, as the prototype oracle did);
    // a refusal maps to exit 66 (EX_IOERR) here, preserving the harness contract.
    let src = match read_source("io-error", path) {
        Ok(s) => s,
        Err(_) => return ExitCode::from(66),
    };

    let nodule = match parse(&src) {
        Ok(c) => c,
        Err(e) => {
            println!("parse-error: {e}");
            return ExitCode::from(2);
        }
    };
    if let Err(e) = check_nodule(&nodule) {
        println!("check-error: {e}");
        return ExitCode::from(3);
    }
    if let Some(expected) = expect_main {
        let found = nodule.items.iter().find_map(|i| match i {
            Item::Fn(f) if f.sig.name == "main" => Some(f),
            _ => None,
        });
        let Some(f) = found else {
            println!(
                "check-error: no `fn main` declared (task requires `fn main() -> {expected}`)"
            );
            return ExitCode::from(3);
        };
        if !f.sig.value_params.is_empty() {
            println!(
                "check-error: `main` must be nullary, has {} parameter(s)",
                f.sig.value_params.len()
            );
            return ExitCode::from(3);
        }
        let got = render_type(&f.sig.ret);
        if got != expected {
            println!("check-error: `main` returns {got}, task requires {expected}");
            return ExitCode::from(3);
        }
    }
    // Keep the driver library exercised on the same input (parity), then emit the oracle's `ok`.
    let _ = check_sources(&[(path.to_owned(), src)]);
    println!("ok");
    ExitCode::SUCCESS
}

/// Render a declared return type the way the surface writes it (for `--expect-main`). Ported verbatim
/// from the prototype oracle so the harness contract is unchanged.
fn render_type(t: &TypeRef) -> String {
    use mycelium_l1::ast::{BaseType, Scalar, Sparsity, Strength};
    let base = match &t.base {
        BaseType::Binary(n) => format!("Binary{{{n}}}"),
        BaseType::Ternary(m) => format!("Ternary{{{m}}}"),
        BaseType::Dense(d, s) => format!(
            "Dense{{{d}, {}}}",
            match s {
                Scalar::F16 => "F16",
                Scalar::Bf16 => "BF16",
                Scalar::F32 => "F32",
                Scalar::F64 => "F64",
            }
        ),
        BaseType::Vsa {
            model,
            dim,
            sparsity,
        } => match sparsity {
            Sparsity::Dense => format!("VSA{{{model}, {dim}, Dense}}"),
            Sparsity::Sparse(k) => format!("VSA{{{model}, {dim}, Sparse{{{k}}}}}"),
        },
        BaseType::Substrate(tag) => format!("Substrate{{{tag}}}"),
        BaseType::Named(n, args) => {
            if args.is_empty() {
                n.clone()
            } else {
                let inner: Vec<String> = args.iter().map(render_type).collect();
                format!("{n}<{}>", inner.join(", "))
            }
        }
        BaseType::Ambient(params) => match params {
            mycelium_l1::ast::AmbientParams::Size(n) => format!("{{{n}}}"),
            mycelium_l1::ast::AmbientParams::Dense(d, s) => format!(
                "{{{d}, {}}}",
                match s {
                    Scalar::F16 => "F16",
                    Scalar::Bf16 => "BF16",
                    Scalar::F32 => "F32",
                    Scalar::F64 => "F64",
                }
            ),
            mycelium_l1::ast::AmbientParams::Vsa {
                model,
                dim,
                sparsity,
            } => match sparsity {
                Sparsity::Dense => format!("{{{model}, {dim}, Dense}}"),
                Sparsity::Sparse(k) => format!("{{{model}, {dim}, Sparse{{{k}}}}}"),
            },
        },
        // RFC-0024 §3: function type `A -> B` (right-associative). Parenthesize a function-typed
        // LHS so `(A -> B) -> C` is unambiguous, not `A -> B -> C` (Copilot #397).
        BaseType::Fn(a, b) => {
            let lhs = render_type(a);
            let lhs = if matches!(a.base, BaseType::Fn(..)) {
                format!("({lhs})")
            } else {
                lhs
            };
            format!("{lhs} -> {}", render_type(b))
        }
    };
    match t.guarantee {
        None => base,
        Some(Strength::Exact) => format!("{base} @ Exact"),
        Some(Strength::Proven) => format!("{base} @ Proven"),
        Some(Strength::Empirical) => format!("{base} @ Empirical"),
        Some(Strength::Declared) => format!("{base} @ Declared"),
    }
}
