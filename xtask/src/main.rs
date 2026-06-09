//! `xtask` — repo automation entrypoint (run via `cargo xtask <task>`).
//!
//! Placeholder (M-091). Real tasks (codegen, schema sync, proof drivers) land as needed.

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        None | Some("help") => eprintln!("xtask: no tasks defined yet (M-091 skeleton)"),
        Some(other) => {
            eprintln!("xtask: unknown task {other:?}");
            std::process::exit(2);
        }
    }
}
