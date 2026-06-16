//! `recursion-probe` — empirical grounding for DN-05 / M-347 (AOT recursion stack-robustness).
//!
//! Measures, rather than presumes, the difference between the two execution paths on **unbounded
//! object-level recursion**, using a *tiny-AST* non-productive program so the only deep recursion is
//! at **evaluation** time (no deep lowering/`subst`/hashing to confound the result):
//!
//! ```text
//! spin = (fix f => λx. f x) c        -- f x ⟶ f x ⟶ …  (never productive)
//! ```
//!
//! - The **reference interpreter** iterates `step`, so it uses *O(1)* host stack: at any fuel it
//!   returns an explicit `FuelExhausted` — graceful, never a crash.
//! - The **AOT env-machine** recurses on the *host call stack* (each `Fix` unfold nests Rust frames):
//!   below a depth threshold it returns `FuelExhausted`; **above it the process aborts** (stack
//!   overflow). This task binary-searches the fuel **in subprocesses** to find that threshold — the
//!   empirical max host-stack recursion depth — so DN-05's claim is `Empirical`, not `Declared`.
//!
//! Usage:
//!   cargo run -p xtask -- recursion-probe            # the full sweep + report
//!   cargo run -p xtask -- recursion-probe run aot N  # one trial at fuel N (a subprocess worker)

use std::process::Command;
use std::time::Instant;

use mycelium_core::{Meta, Node, Payload, Provenance, Repr, Value};
use mycelium_interp::{IdentitySwapEngine, Interpreter, PrimRegistry};

/// `spin = (fix f => λx. f x) c` — a tiny AST whose *evaluation* recurses without bound.
fn spin_program() -> Node {
    let c = Value::new(
        Repr::Binary { width: 1 },
        Payload::Bits(vec![false]),
        Meta::exact(Provenance::Root),
    )
    .expect("well-formed const");
    Node::App {
        func: Box::new(Node::Fix {
            name: "f".into(),
            body: Box::new(Node::Lam {
                param: "x".into(),
                body: Box::new(Node::App {
                    func: Box::new(Node::Var("f".into())),
                    arg: Box::new(Node::Var("x".into())),
                }),
            }),
        }),
        arg: Box::new(Node::Const(c)),
    }
}

/// A single trial at `fuel`, in *this* process. Prints the outcome and exits 0 for any **graceful**
/// result (a `FuelExhausted` is the expected, never-silent outcome); a host-stack overflow aborts the
/// process before we get here (observed by the parent as a non-success exit status).
fn run_one(mode: &str, fuel: u64) {
    let prog = spin_program();
    match mode {
        "interp" => {
            let r = Interpreter::default().with_fuel(fuel).eval_core(&prog);
            println!("interp fuel={fuel}: {r:?}");
        }
        "aot" => {
            let r = mycelium_mlir::run_core_with_fuel(
                &prog,
                &PrimRegistry::with_builtins(),
                &IdentitySwapEngine,
                fuel,
            );
            println!("aot fuel={fuel}: {r:?}");
        }
        other => {
            eprintln!("recursion-probe: unknown mode {other:?}");
            std::process::exit(2);
        }
    }
}

/// Run one AOT trial as a subprocess. Returns `(graceful, outcome)`: `graceful` is `true` iff the
/// process exited cleanly (no host-stack abort); `outcome` is the worker's printed result line (the
/// *actual* error — `DepthLimit` or `FuelExhausted` — or the abort signal), so the report is honest.
fn aot_trial(fuel: u64) -> (bool, String) {
    let exe = std::env::current_exe().expect("current_exe");
    match Command::new(exe)
        .args(["recursion-probe", "run", "aot", &fuel.to_string()])
        .output()
    {
        Ok(o) if o.status.success() => {
            let line = String::from_utf8_lossy(&o.stdout)
                .lines()
                .last()
                .unwrap_or("(no output)")
                .trim()
                .to_owned();
            (true, line)
        }
        Ok(o) => (false, format!("ABORT (exit {:?})", o.status)),
        Err(e) => (false, format!("spawn failed: {e}")),
    }
}

pub fn run() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    // Subprocess worker form: `recursion-probe run <mode> <fuel>`.
    if args.get(1).map(String::as_str) == Some("run") {
        let mode = args.get(2).map(String::as_str).unwrap_or("aot");
        let fuel: u64 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(1000);
        run_one(mode, fuel);
        return;
    }

    println!("== recursion-probe (DN-05 / M-347): empirical AOT recursion stack-robustness ==\n");

    // 1) The interpreter is O(1)-stack: graceful at large fuel (in-process — it cannot crash).
    let big = 5_000_000u64;
    let t = Instant::now();
    let r = Interpreter::default()
        .with_fuel(big)
        .eval_core(&spin_program());
    let dt = t.elapsed();
    println!(
        "interpreter  @ fuel {big}: {r:?}  ({:.1} ms) — O(1) host stack, graceful",
        dt.as_secs_f64() * 1e3
    );

    // 2) The AOT env-machine: probe upward for a crashing fuel, then binary-search the threshold.
    println!(
        "\nAOT env-machine: searching the host-stack recursion-depth threshold (subprocesses)…"
    );
    let mut lo = 0u64; // last known graceful
    let mut hi = 0u64; // first known crash (0 ⇒ not yet found)
    for &probe in &[
        1_000u64, 10_000, 50_000, 100_000, 250_000, 500_000, 1_000_000, 5_000_000,
    ] {
        let (ok, outcome) = aot_trial(probe);
        println!("  probe fuel {probe:>9}: {outcome}");
        if ok {
            lo = probe;
        } else {
            hi = probe;
            break;
        }
    }

    if hi == 0 {
        println!(
            "\nEMPIRICAL RESULT (post-trampoline, M-347): no host-stack abort up to fuel {lo} — the \
             AOT env-machine returns an explicit, graceful budget error (DepthLimit / FuelExhausted) \
             at every depth, like the O(1)-stack interpreter. The ~600-unfold host-stack abort \
             (DN-05 §1.1, pre-fix) is gone: object recursion now lives on the heap control stack."
        );
        return;
    }

    // Binary-search between the last graceful (lo) and first crash (hi).
    while hi - lo > lo / 50 + 1 {
        let mid = lo + (hi - lo) / 2;
        if aot_trial(mid).0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }

    println!(
        "\nEMPIRICAL RESULT: the AOT env-machine recurses gracefully to ~{lo} unfolds and ABORTS \
         (host-stack overflow) by ~{hi} — the pre-trampoline behaviour (DN-05 §1.1). The fix \
         (DN-05 #2, M-347) replaces this abort with an explicit DepthLimit/FuelExhausted."
    );
}
