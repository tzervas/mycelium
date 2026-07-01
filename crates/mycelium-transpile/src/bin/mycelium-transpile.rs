//! CLI for `mycelium-transpile` (M-873, batch mode added in the follow-on wave):
//! `mycelium-transpile <input> <out-dir>`.
//!
//! `<input>` is either:
//! - a single `.rs` file — writes `<out-dir>/<stem>.myc` + `<out-dir>/<stem>.gap.json`, then
//!   prints a one-line summary (unchanged single-file behavior); or
//! - a directory (typically a crate's `src/`) — recurses every `*.rs` file (skipping test
//!   infrastructure, `src/batch.rs::discover_rs_files`), transpiles each independently, writes
//!   the same per-file `<stem>.myc`/`<stem>.gap.json` pair for every discovered file **plus** two
//!   combined artifacts: `<out-dir>/summary.json` (per-file + aggregate counts) and
//!   `<out-dir>/union.gap.json` (every gap from every file, plus aggregate category counts).
//!
//! Every artifact is `Declared`/unvalidated (see `src/lib.rs`). No `clap` dependency — plain
//! `std::env::args` (kickoff-scoped minimal deps).

use mycelium_transpile::batch::{discover_rs_files, summarize, transpile_batch};
use mycelium_transpile::{transpile_file, GapReport};
use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: mycelium-transpile <input.rs | input-dir> <out-dir>");
        return ExitCode::FAILURE;
    }
    let input = Path::new(&args[1]);
    let out_dir = Path::new(&args[2]);

    if let Err(e) = fs::create_dir_all(out_dir) {
        eprintln!(
            "mycelium-transpile: failed to create {}: {e}",
            out_dir.display()
        );
        return ExitCode::FAILURE;
    }

    if input.is_dir() {
        run_batch(input, out_dir)
    } else {
        run_single_file(input, out_dir)
    }
}

/// Write `<out_dir>/<stem>.myc` + `<out_dir>/<stem>.gap.json` for one already-transpiled file.
/// Shared by both single-file and batch mode so the two never drift.
fn write_pair(
    stem: &str,
    myc_text: &str,
    report: &GapReport,
    out_dir: &Path,
) -> Result<(), String> {
    let myc_path = out_dir.join(format!("{stem}.myc"));
    let gap_path = out_dir.join(format!("{stem}.gap.json"));
    fs::write(&myc_path, myc_text)
        .map_err(|e| format!("failed to write {}: {e}", myc_path.display()))?;
    let gap_json = serde_json::to_string_pretty(report)
        .map_err(|e| format!("failed to serialize gap report for {stem}: {e}"))?;
    fs::write(&gap_path, gap_json)
        .map_err(|e| format!("failed to write {}: {e}", gap_path.display()))?;
    Ok(())
}

fn run_single_file(input: &Path, out_dir: &Path) -> ExitCode {
    let (myc_text, report) = match transpile_file(input) {
        Ok(pair) => pair,
        Err(e) => {
            eprintln!("mycelium-transpile: {e}");
            return ExitCode::FAILURE;
        }
    };

    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    if let Err(e) = write_pair(stem, &myc_text, &report, out_dir) {
        eprintln!("mycelium-transpile: {e}");
        return ExitCode::FAILURE;
    }

    let emitted = report.emitted_items.len();
    let gapped = report.gaps.len();
    let non_test = report.non_test_item_count();
    println!(
        "mycelium-transpile: {} top-level item(s) ({} non-test) — {} emitted, {} gap(s) \
         recorded, {:.1}% expressible -> {}/{stem}.myc, {}/{stem}.gap.json",
        report.total_top_level_items,
        non_test,
        emitted,
        gapped,
        report.expressible_fraction() * 100.0,
        out_dir.display(),
        out_dir.display(),
    );
    ExitCode::SUCCESS
}

fn run_batch(input_dir: &Path, out_dir: &Path) -> ExitCode {
    let files = match discover_rs_files(input_dir) {
        Ok(f) => f,
        Err(e) => {
            eprintln!(
                "mycelium-transpile: failed to walk {}: {e}",
                input_dir.display()
            );
            return ExitCode::FAILURE;
        }
    };
    if files.is_empty() {
        eprintln!(
            "mycelium-transpile: no .rs files found under {} (after skipping test \
             infrastructure)",
            input_dir.display()
        );
        return ExitCode::FAILURE;
    }

    let (results, failures) = transpile_batch(&files);
    // A hard parse/read failure is never silently dropped from the run (G2) — it is reported and
    // fails the process, distinct from a per-item gap (which the summary/union artifacts do
    // capture).
    for (path, err) in &failures {
        eprintln!("mycelium-transpile: {}: {err}", path.display());
    }

    // Per-file artifacts, named by stem — collisions (two files sharing a stem, e.g. two
    // `mod.rs`) are resolved by keeping the *last* write and flagging it loudly (never silent),
    // since this PoC's per-file naming scheme has no path-qualification mechanism.
    let mut seen_stems: std::collections::HashMap<String, std::path::PathBuf> =
        std::collections::HashMap::new();
    for r in &results {
        let stem = r
            .path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output")
            .to_string();
        if let Some(prev) = seen_stems.insert(stem.clone(), r.path.clone()) {
            eprintln!(
                "mycelium-transpile: WARNING stem collision `{stem}.myc`/`{stem}.gap.json` — \
                 {} overwrites {} (no path-qualification in this PoC's per-file naming)",
                r.path.display(),
                prev.display()
            );
        }
        if let Err(e) = write_pair(&stem, &r.myc, &r.report, out_dir) {
            eprintln!("mycelium-transpile: {e}");
            return ExitCode::FAILURE;
        }
    }

    let (batch_summary, union) = summarize(&results);

    let summary_path = out_dir.join("summary.json");
    match serde_json::to_string_pretty(&batch_summary) {
        Ok(j) => {
            if let Err(e) = fs::write(&summary_path, j) {
                eprintln!(
                    "mycelium-transpile: failed to write {}: {e}",
                    summary_path.display()
                );
                return ExitCode::FAILURE;
            }
        }
        Err(e) => {
            eprintln!("mycelium-transpile: failed to serialize summary.json: {e}");
            return ExitCode::FAILURE;
        }
    }

    let union_path = out_dir.join("union.gap.json");
    match serde_json::to_string_pretty(&union) {
        Ok(j) => {
            if let Err(e) = fs::write(&union_path, j) {
                eprintln!(
                    "mycelium-transpile: failed to write {}: {e}",
                    union_path.display()
                );
                return ExitCode::FAILURE;
            }
        }
        Err(e) => {
            eprintln!("mycelium-transpile: failed to serialize union.gap.json: {e}");
            return ExitCode::FAILURE;
        }
    }

    println!(
        "mycelium-transpile: batch over {} file(s) ({} failed to parse) — {} top-level item(s) \
         ({} non-test), {} emitted, {} gap(s), {:.1}% expressible -> {}, {}",
        results.len(),
        failures.len(),
        batch_summary.totals.total_items,
        batch_summary.totals.non_test_items,
        batch_summary.totals.emitted,
        batch_summary.totals.gaps,
        batch_summary.totals.expressible_pct,
        summary_path.display(),
        union_path.display(),
    );

    if failures.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
