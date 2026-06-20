//! `mycelium-bench` runnable harness — runs the execution-backend benchmark over the shared corpus,
//! ingests the LLM-harness report, and writes a deterministic markdown + JSON report into
//! `crates/mycelium-bench/reports/`.
//!
//! **Run with `--release`** (`cargo run --release -p mycelium-bench --bin bench`). A debug build is
//! refused — its timings are not representative (no optimisation, overflow checks on), so any WIN/LOSS
//! verdict from it would be dishonest (VR-5/G2).
//!
//! It prints a short human summary to stdout and writes the full report to `reports/`. Optional flag:
//! `--out <DIR>` to redirect the report directory; `--stdout` to also print the markdown.

use std::path::{Path, PathBuf};

use mycelium_bench::backend::Engines;
use mycelium_bench::corpus::corpus;
use mycelium_bench::llm::{LlmIngestError, LlmReport};
use mycelium_bench::measure::run_corpus;
use mycelium_bench::report::{neutral_band, Honesty, LlmSection, Report};
use mycelium_bench::timing::refuse_debug_build;

fn main() {
    // 1. Honest profile gate: never measure a debug build.
    refuse_debug_build();

    let args: Vec<String> = std::env::args().skip(1).collect();
    let out_dir = parse_out_dir(&args).unwrap_or_else(default_reports_dir);
    let also_stdout = args.iter().any(|a| a == "--stdout");

    eprintln!("mycelium-bench: running the execution-backend corpus (release build)...");
    let eng = Engines::default();
    let cases = corpus();
    let run = run_corpus(&cases, &eng);

    // 2. Ingest the LLM-harness report: prefer the newest real one; fall back to the committed
    //    SYNTHETIC sample (labeled synthetic). Absence is recorded, never synthesized.
    let llm = ingest_llm_section();

    let report = Report {
        tool: "mycelium-bench",
        profile: "release",
        mlir_dialect_feature: cfg!(feature = "mlir-dialect"),
        host_note: host_note(),
        honesty: Honesty::default(),
        neutral_band: neutral_band(),
        run,
        llm,
    };

    // 3. Emit both projections (G11 dual projection), deterministically.
    if let Err(e) = std::fs::create_dir_all(&out_dir) {
        eprintln!(
            "mycelium-bench: cannot create report dir {}: {e}",
            out_dir.display()
        );
        std::process::exit(1);
    }
    let md = report.to_markdown();
    let json = match report.to_json() {
        Ok(j) => j,
        Err(e) => {
            eprintln!("mycelium-bench: failed to serialize JSON report: {e}");
            std::process::exit(1);
        }
    };
    // Deterministic stable filenames (latest-run convention) so the committed report is diffable.
    let md_path = out_dir.join("latest-report.md");
    let json_path = out_dir.join("latest-report.json");
    write_or_die(&md_path, &md);
    write_or_die(&json_path, &json);

    // 4. Short human summary to stdout.
    let t = report.tallies();
    println!("mycelium-bench summary:");
    println!(
        "  cases: {}   wins: {}   neutral: {}   speed-losses: {}   correctness-losses: {}   \
         capability-losses: {}   errors: {}   skips: {}",
        report.run.cases.len(),
        t.wins,
        t.neutral,
        t.speed_losses,
        t.correctness_losses,
        t.capability_losses,
        t.errors,
        t.skips,
    );
    if t.baseline_failures > 0 {
        println!(
            "  WARNING: {} baseline (interpreter) failure(s) — the trusted base failed; investigate.",
            t.baseline_failures
        );
    }
    println!("  report (markdown): {}", md_path.display());
    println!("  report (json)    : {}", json_path.display());
    if let Some(sec) = &report.llm {
        println!(
            "  llm-harness      : {} ({})",
            sec.source_path,
            if sec.is_synthetic {
                "SYNTHETIC sample"
            } else {
                "real run"
            }
        );
    } else {
        println!("  llm-harness      : none found (section recorded empty, not synthesized)");
    }

    if also_stdout {
        println!("\n{md}");
    }
}

/// `--out <DIR>` override for the report directory.
fn parse_out_dir(args: &[String]) -> Option<PathBuf> {
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if a == "--out" {
            return it.next().map(PathBuf::from);
        }
    }
    None
}

/// The default report dir: `<crate>/reports/`, resolved relative to this source file so it works from
/// any CWD (`CARGO_MANIFEST_DIR` is the crate root at build time).
fn default_reports_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("reports")
}

/// Best-effort one-line host note for report provenance (target triple + thread count). No PII.
fn host_note() -> String {
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;
    let threads = std::thread::available_parallelism()
        .map(std::num::NonZeroUsize::get)
        .unwrap_or(0);
    format!("host: {arch}-{os}, {threads} hw threads (provenance only)")
}

/// Find the LLM-harness report to ingest. Order:
/// 1. the newest `*-report.json` in `tools/llm-harness/reports/` (a real or fixture run), else
/// 2. the committed synthetic sample if one is present there, else
/// 3. `None` (recorded as "no report", never synthesized).
fn ingest_llm_section() -> Option<LlmSection> {
    let reports_dir = harness_reports_dir()?;
    let path = match LlmReport::newest_in_dir(&reports_dir) {
        Ok(Some(p)) => p,
        Ok(None) => {
            eprintln!(
                "mycelium-bench: no LLM-harness report found under {} — LLM section recorded empty.",
                reports_dir.display()
            );
            return None;
        }
        Err(LlmIngestError::Io(m)) | Err(LlmIngestError::Parse(m)) => {
            eprintln!("mycelium-bench: could not scan LLM reports dir: {m}");
            return None;
        }
    };
    match LlmReport::from_path(&path) {
        Ok(rep) => {
            let synthetic = rep.is_synthetic();
            if synthetic {
                eprintln!(
                    "mycelium-bench: ingesting SYNTHETIC llm-harness sample {} (labeled synthetic).",
                    path.display()
                );
            }
            Some(LlmSection::from_report(
                &rep,
                path.display().to_string(),
                synthetic,
            ))
        }
        Err(e) => {
            eprintln!(
                "mycelium-bench: failed to read LLM report {}: {e}",
                path.display()
            );
            None
        }
    }
}

/// Resolve `tools/llm-harness/reports/` relative to the workspace root (the crate is at
/// `<root>/crates/mycelium-bench`, so the harness dir is two levels up). Returns `None` if it does
/// not exist.
fn harness_reports_dir() -> Option<PathBuf> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let dir = manifest
        .parent() // crates/
        .and_then(Path::parent) // <root>/
        .map(|root| root.join("tools/llm-harness/reports"))?;
    dir.is_dir().then_some(dir)
}

fn write_or_die(path: &Path, contents: &str) {
    if let Err(e) = std::fs::write(path, contents) {
        eprintln!("mycelium-bench: failed to write {}: {e}", path.display());
        std::process::exit(1);
    }
}
