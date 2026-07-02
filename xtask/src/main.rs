//! `xtask` — repo automation entrypoint (run via `cargo xtask <task>`).
//!
//! Tasks: `kc4` — the KC-4 per-swap certificate-check overhead measurement (M-212); `e1` — the E1
//! perf harness (M-250 staged-packing codec + M-303 native AOT path vs interpreter); `deps` — the
//! structural acyclic-deps gate over `cargo metadata` (M-877/M-878/M-879). Further tasks
//! (codegen, schema sync, proof drivers) land as needed.

mod deps;
mod e1;
mod kc4;
mod recursion_probe;

#[cfg(test)]
mod tests;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("kc4") => kc4::run(),
        Some("e1") => e1::run(),
        Some("recursion-probe") => recursion_probe::run(),
        Some("deps") => deps::run(),
        None | Some("help") => {
            eprintln!(
                "xtask tasks:\n  kc4   KC-4 per-swap cert-check overhead (run with --release)\n  \
                 e1    E1 perf harness: packing codec + native AOT vs interp (run with --release)\n  \
                 recursion-probe  empirical AOT recursion stack-robustness (DN-05 / M-347)\n  \
                 deps  structural acyclic-deps gate over `cargo metadata` (M-877/878/879)"
            );
        }
        Some(other) => {
            eprintln!("xtask: unknown task {other:?}");
            std::process::exit(2);
        }
    }
}
