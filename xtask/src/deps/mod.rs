//! `deps` — the structural acyclic-deps gate (M-877 normal-deps downward-only, M-878 dev-dep
//! cycle detection, M-879 the two named cross-boundary rules), implemented as a `cargo metadata
//! --format-version 1` analysis (not `cargo-deny`) so it stays self-contained: no runtime
//! dependency beyond `cargo` itself.
//!
//! Usage: `cargo run -p xtask -- deps`
//!
//! **Not yet wired into `just check`** (that's M-880 — the justfile is orchestrator-owned; this
//! task only exposes the subcommand). At HEAD this run is EXPECTED to report violations: the
//! three known `mycelium-cert` dev-dep cycles (fixed by sibling leaves M-881/M-882) collapse into
//! one connected strongly-connected component (see `checks::check_acyclic_including_dev` doc),
//! plus the known `mycelium-mlir -> mycelium-std-runtime` upward-tier normal edge (fixed by
//! M-883/M-884). Reporting these is CORRECT — it's the gate this task exists to build, not to make
//! green.

pub mod checks;
pub mod graph;
pub mod strata;

use cargo_metadata::MetadataCommand;

use self::checks::Violation;
use self::graph::Graph;
use self::strata::StrataConfig;

/// Run `cargo metadata --format-version 1`, build the intra-workspace graph, run every check
/// against the frozen `deps-strata.toml`, print every violation (never-silent, G2), and exit
/// non-zero iff any violation fired.
pub fn run() {
    let metadata = match MetadataCommand::new().exec() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("xtask deps: `cargo metadata` failed: {e}");
            std::process::exit(2);
        }
    };
    let graph = Graph::from_metadata(&metadata);
    let cfg = StrataConfig::embedded();
    let violations = checks::run_all(&graph, &cfg);
    report(&graph, &cfg, &violations);
    if !violations.is_empty() {
        std::process::exit(1);
    }
}

/// Never-silent (G2) console report: every violation is printed with its rule id, never merely
/// a pass/fail exit code. Leads with the config's own guarantee tags (EXPLAIN-able, G2 "no black
/// boxes" — a reader can see exactly how strong a basis each half of the map claims) before any
/// per-edge detail.
fn report(graph: &Graph, cfg: &StrataConfig, violations: &[Violation]) {
    println!(
        "== xtask deps (M-877/878/879): structural acyclic-deps gate over {} workspace crates ==",
        graph.crates.len()
    );
    println!("   basis: {}", cfg.meta.basis_ref);
    println!("   derived_from: {}", cfg.meta.derived_from);
    println!("   [strata] guarantee: {}", cfg.meta.strata_guarantee);
    println!("   [tiers] guarantee: {}\n", cfg.meta.tiers_guarantee);
    if violations.is_empty() {
        println!("OK — no violations.");
        return;
    }
    println!("{} violation(s):\n", violations.len());
    for v in violations {
        println!("  [{}] {}", v.rule, v.message);
    }
    println!(
        "\nNot wired into `just check` yet (M-880). See xtask/deps-strata.toml for the stratum \
         map, tier map, and named-rule definitions each violation cites."
    );
}
