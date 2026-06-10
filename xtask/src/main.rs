//! `xtask` — repo automation entrypoint (run via `cargo xtask <task>`).
//!
//! Tasks: `kc4` — the KC-4 per-swap certificate-check overhead measurement (M-212). Further tasks
//! (codegen, schema sync, proof drivers) land as needed.

mod kc4;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("kc4") => kc4::run(),
        None | Some("help") => {
            eprintln!(
                "xtask tasks:\n  kc4   KC-4 per-swap cert-check overhead (run with --release)"
            );
        }
        Some(other) => {
            eprintln!("xtask: unknown task {other:?}");
            std::process::exit(2);
        }
    }
}
