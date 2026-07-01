//! CLI for `mycelium-transpile` (M-873): `mycelium-transpile <input.rs> <out-dir>`.
//!
//! Writes `<out-dir>/<stem>.myc` (best-effort, `Declared`/unvalidated) and
//! `<out-dir>/<stem>.gap.json` (the structured, never-silent gap report), then prints a one-line
//! summary. No `clap` dependency — plain `std::env::args` (kickoff-scoped minimal deps).

use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: mycelium-transpile <input.rs> <out-dir>");
        return ExitCode::FAILURE;
    }
    let input = Path::new(&args[1]);
    let out_dir = Path::new(&args[2]);

    let (myc_text, report) = match mycelium_transpile::transpile_file(input) {
        Ok(pair) => pair,
        Err(e) => {
            eprintln!("mycelium-transpile: {e}");
            return ExitCode::FAILURE;
        }
    };

    if let Err(e) = fs::create_dir_all(out_dir) {
        eprintln!(
            "mycelium-transpile: failed to create {}: {e}",
            out_dir.display()
        );
        return ExitCode::FAILURE;
    }

    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let myc_path = out_dir.join(format!("{stem}.myc"));
    let gap_path = out_dir.join(format!("{stem}.gap.json"));

    if let Err(e) = fs::write(&myc_path, &myc_text) {
        eprintln!(
            "mycelium-transpile: failed to write {}: {e}",
            myc_path.display()
        );
        return ExitCode::FAILURE;
    }
    let gap_json = match serde_json::to_string_pretty(&report) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("mycelium-transpile: failed to serialize gap report: {e}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(e) = fs::write(&gap_path, gap_json) {
        eprintln!(
            "mycelium-transpile: failed to write {}: {e}",
            gap_path.display()
        );
        return ExitCode::FAILURE;
    }

    let emitted = report.emitted_items.len();
    let gapped = report.gaps.len();
    let non_test = report.non_test_item_count();
    println!(
        "mycelium-transpile: {} top-level item(s) ({} non-test) — {} emitted, {} gap(s) \
         recorded, {:.1}% expressible -> {} / {}",
        report.total_top_level_items,
        non_test,
        emitted,
        gapped,
        report.expressible_fraction() * 100.0,
        myc_path.display(),
        gap_path.display()
    );
    ExitCode::SUCCESS
}
