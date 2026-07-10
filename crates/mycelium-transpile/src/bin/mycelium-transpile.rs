//! CLI for `mycelium-transpile` (M-873, batch mode added in the follow-on wave; `--vet` added in
//! M-1000): `mycelium-transpile [--vet] <input> <out-dir>`.
//!
//! `<input>` is either:
//! - a single `.rs` file — writes `<out-dir>/<stem>.myc` + `<out-dir>/<stem>.gap.json`, then
//!   prints a one-line summary (unchanged single-file behavior); or
//! - a directory (typically a crate's `src/`, or the whole `crates/` corpus) — recurses every
//!   `*.rs` file (skipping test infrastructure, `src/batch.rs::discover_rs_files`), transpiles each
//!   independently, and writes a `.myc`/`.gap.json` pair for every discovered file at a
//!   **path-qualified** output that mirrors the source tree under `<out-dir>` (a file's path
//!   relative to the batch root becomes its output path — `mycelium-core/src/lib.myc`, not a flat
//!   `lib.myc`), so a whole-corpus run never overwrites two crates' same-stem files (M-1006
//!   Phase-2). For a single-crate `src/` with a flat layout the mirrored path is just the stem, so
//!   the output is identical to the pre-Phase-2 flat naming. Also writes two combined artifacts:
//!   `<out-dir>/summary.json` (per-file + aggregate counts) and `<out-dir>/union.gap.json` (every
//!   gap from every file, plus aggregate category counts).
//!
//! `--vet` (M-1000) runs the **real** `myc check` oracle over every emitted `.myc`, writes
//! `<out-dir>/vet.json` (per-file + aggregate vet records), and prints the **`checked_fraction`**
//! (myc-check-clean coverage) alongside the emission-only `expressible_fraction`. The oracle is the
//! pre-built `MYC_CHECK_CMD` binary when that env var is set (the sanctioned, build-lock-safe form
//! `scripts/checks/transpile-vet.sh` uses), else the `cargo run -p mycelium-check` fallback
//! (`crate::vet::MycChecker::from_env`). See `src/vet.rs` for the metric's stated denominator.
//!
//! Every emitted artifact is `Declared`/unvalidated (see `src/lib.rs`); the vet verdict is
//! `Empirical` (measured — see `src/vet.rs`). No `clap` dependency — plain `std::env::args`
//! (kickoff-scoped minimal deps).

use mycelium_transpile::batch::{discover_rs_files, output_rel_path, summarize, transpile_batch};
use mycelium_transpile::vet::{vet_batch, MycChecker, VetInput, VetReport};
use mycelium_transpile::{transpile_file, GapReport};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    // Parse a minimal flag set: an optional `--vet` before the two positional args. Kept hand-rolled
    // (no `clap`) per the crate's minimal-deps stance.
    let mut vet = false;
    let mut positional: Vec<String> = Vec::new();
    for a in env::args().skip(1) {
        match a.as_str() {
            "--vet" => vet = true,
            _ => positional.push(a),
        }
    }
    if positional.len() != 2 {
        eprintln!("usage: mycelium-transpile [--vet] <input.rs | input-dir> <out-dir>");
        return ExitCode::FAILURE;
    }
    let input = Path::new(&positional[0]);
    let out_dir = Path::new(&positional[1]);

    if let Err(e) = fs::create_dir_all(out_dir) {
        eprintln!(
            "mycelium-transpile: failed to create {}: {e}",
            out_dir.display()
        );
        return ExitCode::FAILURE;
    }

    if input.is_dir() {
        run_batch(input, out_dir, vet)
    } else {
        run_single_file(input, out_dir, vet)
    }
}

/// Run the vet loop over the written `.myc` files and report `checked_fraction` alongside
/// `expressible_fraction`. Advisory: a vet failure/tool-unavailable is reported (never silent, G2)
/// but does **not** change the process exit code — vetting is a measurement, not a gate.
fn run_vet(inputs: &[VetInput], out_dir: &Path) {
    if inputs.is_empty() {
        eprintln!("mycelium-transpile: --vet: no emitted .myc files to vet");
        return;
    }
    // Cargo-fallback runs in the current directory (typically the workspace root); the sanctioned
    // path is a pre-built `MYC_CHECK_CMD` binary, which carries its own absolute program path.
    let checker = MycChecker::from_env(env::current_dir().ok());
    let report = vet_batch(&checker, inputs);
    let vet_path = out_dir.join("vet.json");
    match serde_json::to_string_pretty(&report) {
        Ok(j) => {
            if let Err(e) = fs::write(&vet_path, j) {
                eprintln!(
                    "mycelium-transpile: failed to write {}: {e}",
                    vet_path.display()
                );
            }
        }
        Err(e) => eprintln!("mycelium-transpile: failed to serialize vet.json: {e}"),
    }
    print_vet_summary(&report, &vet_path);
}

fn print_vet_summary(report: &VetReport, vet_path: &Path) {
    let (clean_files, files_with_emissions) = report.clean_file_fraction();
    // Per-class file breakdown, deterministically ordered (BTreeMap).
    let classes = report
        .class_counts
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join(" ");
    println!(
        "mycelium-transpile: --vet over {} file(s) — checked_fraction {:.1}% ({}/{} items \
         myc-check-clean, file-gated) vs expressible_fraction {:.1}% ({}/{} items emitted); \
         {clean_files}/{files_with_emissions} file(s) with emissions fully clean [{classes}] -> {}",
        report.records.len(),
        report.checked_fraction() * 100.0,
        report.total_checked_clean_items,
        report.total_non_test_items,
        report.expressible_fraction() * 100.0,
        report.total_emitted_items,
        report.total_non_test_items,
        vet_path.display(),
    );
}

/// Append a `.ext` suffix to a path **without** replacing any existing extension — unlike
/// `Path::with_extension`, which would eat a trailing dotted segment (`foo.bar` +`myc` →`foo.myc`).
/// So `<base>` → `<base>.myc` / `<base>.gap.json` faithfully, even for a `foo.bar` stem.
fn append_ext(base: &Path, ext: &str) -> PathBuf {
    let mut s = base.as_os_str().to_os_string();
    s.push(".");
    s.push(ext);
    PathBuf::from(s)
}

/// Write `<out_dir>/<rel_noext>.myc` + `<out_dir>/<rel_noext>.gap.json` for one already-transpiled
/// file, creating any parent directories. `rel_noext` is the output path **without** extension,
/// relative to `out_dir`: a bare stem in single-file mode (`lib`), or the source's path **mirrored
/// under the batch root** in directory mode (`mycelium-core/src/lib`) — the latter is what makes a
/// whole-corpus run non-lossy (two crates' `lib.rs` land at distinct, path-qualified outputs instead
/// of one overwriting the other; M-1006 Phase-2). Shared by both modes so they never drift. Returns
/// the written `.myc` path (for the vet loop).
fn write_pair(
    out_dir: &Path,
    rel_noext: &Path,
    myc_text: &str,
    report: &GapReport,
) -> Result<PathBuf, String> {
    let base = out_dir.join(rel_noext);
    if let Some(parent) = base.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create {}: {e}", parent.display()))?;
    }
    let myc_path = append_ext(&base, "myc");
    let gap_path = append_ext(&base, "gap.json");
    fs::write(&myc_path, myc_text)
        .map_err(|e| format!("failed to write {}: {e}", myc_path.display()))?;
    let gap_json = serde_json::to_string_pretty(report)
        .map_err(|e| format!("failed to serialize gap report for {}: {e}", base.display()))?;
    fs::write(&gap_path, gap_json)
        .map_err(|e| format!("failed to write {}: {e}", gap_path.display()))?;
    Ok(myc_path)
}

fn run_single_file(input: &Path, out_dir: &Path, vet: bool) -> ExitCode {
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

    let myc_path = match write_pair(out_dir, Path::new(stem), &myc_text, &report) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("mycelium-transpile: {e}");
            return ExitCode::FAILURE;
        }
    };

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

    if vet {
        let inputs = vec![VetInput::from_report(myc_path, &report)];
        run_vet(&inputs, out_dir);
    }
    ExitCode::SUCCESS
}

fn run_batch(input_dir: &Path, out_dir: &Path, vet: bool) -> ExitCode {
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

    // Per-file artifacts, **path-qualified** by mirroring the source tree under `out_dir` (M-1006
    // Phase-2): each file's path relative to the batch root becomes its output path, so two crates'
    // `lib.rs` land at distinct outputs (`mycelium-core/src/lib.myc` vs `mycelium-std/src/lib.myc`)
    // instead of one silently overwriting the other — the whole-corpus-completeness fix that lets an
    // automated multi-crate wave keep every emission. Distinct source files have distinct relative
    // paths, so a collision is impossible by construction; a defensive guard still flags the
    // impossible case (never silent, G2) rather than trusting the invariant blindly.
    let mut written: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();
    // One vet input per written `.myc` (only used when `--vet`), keyed by the actual output path so
    // the order is deterministic and the file vetted is exactly the file written.
    let mut vet_inputs: std::collections::BTreeMap<PathBuf, VetInput> =
        std::collections::BTreeMap::new();
    for r in &results {
        // Path relative to the batch root, `.rs` extension stripped (pure logic in `batch.rs` so it
        // is unit-tested there). Fall back to the bare stem if the path is somehow not under the
        // root (never-silent — warned, not silently mis-placed).
        let rel_noext = match output_rel_path(&r.path, input_dir) {
            Ok(rel) => rel,
            Err(fallback) => {
                eprintln!(
                    "mycelium-transpile: WARNING {} is not under the batch root {} — falling back \
                     to a bare-stem output name",
                    r.path.display(),
                    input_dir.display()
                );
                fallback
            }
        };
        let myc_path = match write_pair(out_dir, &rel_noext, &r.myc, &r.report) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("mycelium-transpile: {e}");
                return ExitCode::FAILURE;
            }
        };
        if !written.insert(myc_path.clone()) {
            eprintln!(
                "mycelium-transpile: WARNING output path collision at {} — a prior file already \
                 wrote here (should be impossible with path-qualified naming)",
                myc_path.display()
            );
        }
        if vet {
            vet_inputs.insert(myc_path.clone(), VetInput::from_report(myc_path, &r.report));
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

    if vet {
        let inputs: Vec<VetInput> = vet_inputs.into_values().collect();
        run_vet(&inputs, out_dir);
    }

    if failures.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
