//! `myc-check` — parse + typecheck a Mycelium surface program, with explicit diagnostics.
//!
//! The CLI oracle for the KC-2 LLM-leverage harness (M-002; Foundation §6 P0.2; SC-5b): the
//! harness measures *syntactic validity* (exit 2 = parse error) and *type-check pass rate*
//! (exit 3 = check error) over generated programs, so each failure class gets its own exit code
//! and a machine-readable first line (`ok` / `parse-error: …` / `check-error: …`). Also a handy
//! standalone checker (FR-S5 direction).
//!
//! Usage: `myc-check [--expect-main <ret-type>] <file.myc | ->`
//!
//! `--expect-main T` additionally requires a nullary `fn main() -> T` (textual match on the
//! declared return type, e.g. `Ternary{6}` or `Binary{8} @ Proven`) — "the program type-checks"
//! and "the program answers the task" are different claims, and the harness needs the second.

use std::io::Read;
use std::process::ExitCode;

use mycelium_l1::ast::{Item, TypeRef};
use mycelium_l1::{check_colony, parse};

/// Render a declared return type the way the surface writes it (for `--expect-main`).
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
    };
    match t.guarantee {
        None => base,
        Some(Strength::Exact) => format!("{base} @ Exact"),
        Some(Strength::Proven) => format!("{base} @ Proven"),
        Some(Strength::Empirical) => format!("{base} @ Empirical"),
        Some(Strength::Declared) => format!("{base} @ Declared"),
    }
}

fn usage() -> ExitCode {
    eprintln!("usage: myc-check [--expect-main <ret-type>] <file.myc | ->");
    ExitCode::from(64) // EX_USAGE
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let mut expect_main: Option<String> = None;
    let mut path: Option<String> = None;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--expect-main" => match args.next() {
                Some(t) => expect_main = Some(t),
                None => return usage(),
            },
            _ if path.is_none() => path = Some(a),
            _ => return usage(),
        }
    }
    let Some(path) = path else { return usage() };

    let src = if path == "-" {
        let mut s = String::new();
        if std::io::stdin().read_to_string(&mut s).is_err() {
            eprintln!("io-error: could not read stdin");
            return ExitCode::from(66); // EX_NOINPUT
        }
        s
    } else {
        match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("io-error: {path}: {e}");
                return ExitCode::from(66);
            }
        }
    };

    // Syntactic validity: an explicit ParseError, never a panic (S5/G2).
    let colony = match parse(&src) {
        Ok(c) => c,
        Err(e) => {
            println!("parse-error: {e}");
            return ExitCode::from(2);
        }
    };

    // Type-check pass: every refusal is an explicit CheckError (RFC-0007 §4.4/§4.5).
    if let Err(e) = check_colony(&colony) {
        println!("check-error: {e}");
        return ExitCode::from(3);
    }

    // Task conformance: the declared entry signature must match, when asked for.
    if let Some(expected) = expect_main {
        let found = colony.items.iter().find_map(|i| match i {
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

    println!("ok");
    ExitCode::SUCCESS
}
