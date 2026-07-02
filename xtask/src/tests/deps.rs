//! Tests for `xtask deps` (M-877 normal-downward, M-878 dev-cycle detection, M-879 named rules).
//!
//! Two fixture strategies, per CLAUDE.md's "fixtures + parameterization, not bespoke test-body
//! logic":
//! - `Graph::from_edges` (test-only, `xtask/src/deps/graph.rs`) builds a small synthetic graph
//!   directly, for the check-logic cases (most of this file) — no need to hand-write a full
//!   `cargo metadata` JSON document per case.
//! - `fixtures/tiny_metadata.json` is a real (trimmed) `cargo metadata --format-version 1`
//!   document, parsed once to prove `Graph::from_metadata` itself (the JSON-parsing path) is
//!   correct: workspace-only edges, external deps dropped, dep kinds preserved.
//! - `fixtures/rule_*.toml` are small `StrataConfig` documents, one per named-rule shape, so the
//!   M-879 rule-dispatch tests are data-driven rather than constructing configs inline.

use cargo_metadata::{Metadata, MetadataCommand};

use crate::deps::checks::{
    check_acyclic_including_dev, check_named_rules, check_normal_downward, run_all,
};
use crate::deps::graph::{DepKind, Graph};
use crate::deps::strata::StrataConfig;

const TINY_METADATA: &str = include_str!("fixtures/tiny_metadata.json");
const RULE_FORBIDDEN_PREFIX: &str = include_str!("fixtures/rule_forbidden_prefix.toml");
const RULE_TIER_ORDER: &str = include_str!("fixtures/rule_tier_order.toml");
const RULE_UNKNOWN_KIND: &str = include_str!("fixtures/rule_unknown_kind.toml");

// ---- Graph::from_metadata (the real JSON-parsing path) ------------------------------------

#[test]
fn from_metadata_keeps_only_workspace_edges_with_correct_kinds() {
    let meta: Metadata = MetadataCommand::parse(TINY_METADATA).expect("fixture parses");
    let graph = Graph::from_metadata(&meta);

    // Workspace members a/b/c are present; the external "ext" crate is not a node in the graph
    // (it can't participate in an intra-workspace cycle/layering violation).
    assert_eq!(
        graph.crates,
        ["a", "b", "c"].into_iter().map(str::to_owned).collect()
    );

    let a_edges: Vec<_> = graph.all_edges(&[DepKind::Normal, DepKind::Dev]).collect();
    assert_eq!(a_edges.len(), 2, "a->b normal, b->c dev; a->ext is dropped");

    let ab = a_edges
        .iter()
        .find(|e| e.from == "a" && e.to == "b")
        .expect("a->b edge present");
    assert!(ab.kinds.contains(&DepKind::Normal));
    assert!(!ab.kinds.contains(&DepKind::Dev));

    let bc = a_edges
        .iter()
        .find(|e| e.from == "b" && e.to == "c")
        .expect("b->c edge present");
    assert!(bc.kinds.contains(&DepKind::Dev));
    assert!(!bc.kinds.contains(&DepKind::Normal));

    assert!(
        !graph.crates.contains("ext"),
        "external (non-workspace) dependency must not appear as a graph node"
    );
}

// ---- M-877: normal-downward-only ------------------------------------------------------------

/// A minimal, self-consistent stratum map for the synthetic 3-crate chain used below:
/// `top`(2) -> `mid`(1) -> `bottom`(0).
fn clean_strata_fixture() -> StrataConfig {
    StrataConfig::parse(
        r#"
        tier_order = ["core"]

        [meta]
        derived_from = "test fixture"
        strata_guarantee = "n/a (test fixture)"
        tiers_guarantee = "n/a (test fixture)"
        basis_ref = "test fixture"

        [strata]
        top = 2
        mid = 1
        bottom = 0

        [tiers]
        top = "core"
        mid = "core"
        bottom = "core"
        "#,
    )
    .expect("fixture parses")
}

#[test]
fn normal_downward_check_passes_on_clean_chain() {
    let cfg = clean_strata_fixture();
    let graph = Graph::from_edges(&[
        ("top", "mid", &[DepKind::Normal]),
        ("mid", "bottom", &[DepKind::Normal]),
    ]);
    let violations = check_normal_downward(&graph, &cfg);
    assert!(
        violations.is_empty(),
        "clean downward chain: {violations:?}"
    );
}

/// **M-877 negative fixture**: prove the check FAILS on a synthetic upward edge. `bottom` (stratum
/// 0) has a normal dependency on `top` (stratum 2) — an injected upward edge the checker must
/// report, citing the specific rule it broke.
#[test]
fn normal_downward_check_flags_synthetic_upward_edge() {
    let cfg = clean_strata_fixture();
    let graph = Graph::from_edges(&[("bottom", "top", &[DepKind::Normal])]);
    let violations = check_normal_downward(&graph, &cfg);
    assert_eq!(
        violations.len(),
        1,
        "the injected upward edge must be flagged"
    );
    assert_eq!(violations[0].rule, "normal-downward-only");
    assert!(violations[0].message.contains("bottom"));
    assert!(violations[0].message.contains("top"));
}

#[test]
fn normal_downward_check_flags_equal_stratum_edge() {
    let mut cfg = clean_strata_fixture();
    // "mid" -> another crate also at stratum 1 is not *strictly* downward either.
    cfg.strata.insert("sibling".to_owned(), 1);
    let graph = Graph::from_edges(&[("mid", "sibling", &[DepKind::Normal])]);
    let violations = check_normal_downward(&graph, &cfg);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].rule, "normal-downward-only");
}

#[test]
fn normal_downward_check_flags_crate_missing_from_stratum_map() {
    let cfg = clean_strata_fixture();
    let graph = Graph::from_edges(&[("top", "unknown-crate", &[DepKind::Normal])]);
    let violations = check_normal_downward(&graph, &cfg);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].rule, "normal-downward-only");
    assert!(violations[0].message.contains("unknown-crate"));
}

#[test]
fn normal_downward_check_ignores_dev_only_edges() {
    // A dev-only edge that would violate the ordering if it were normal is out of scope for
    // M-877 (M-878 covers dev edges).
    let cfg = clean_strata_fixture();
    let graph = Graph::from_edges(&[("bottom", "top", &[DepKind::Dev])]);
    let violations = check_normal_downward(&graph, &cfg);
    assert!(violations.is_empty());
}

// ---- M-878: acyclic-including-dev (cargo never rejects dev-dep cycles) -----------------------

#[test]
fn dev_cycle_check_passes_on_acyclic_graph() {
    let graph = Graph::from_edges(&[
        ("top", "mid", &[DepKind::Normal]),
        ("mid", "bottom", &[DepKind::Dev]),
    ]);
    let violations = check_acyclic_including_dev(&graph);
    assert!(violations.is_empty());
}

/// **M-878 synthetic fixture**: a cycle that exists ONLY because of a dev edge — cargo itself
/// would accept this (dev-dep cycles are never rejected by cargo), so the checker must be the one
/// to catch it.
#[test]
fn dev_cycle_check_detects_synthetic_dev_only_cycle() {
    let graph = Graph::from_edges(&[
        ("x", "y", &[DepKind::Dev]),
        ("y", "z", &[DepKind::Normal]),
        ("z", "x", &[DepKind::Normal]),
    ]);
    let violations = check_acyclic_including_dev(&graph);
    assert_eq!(violations.len(), 1, "exactly one cyclic component");
    assert_eq!(violations[0].rule, "acyclic-including-dev");
    for c in ["x", "y", "z"] {
        assert!(
            violations[0].message.contains(c),
            "cycle report must name {c}: {}",
            violations[0].message
        );
    }
}

/// Reproduces the *shape* of the three known `mycelium-cert` dev-dep cycles (M-878) documented in
/// the task: `select ->[dev] cert -> vsa -> select`, `cert ->[dev] proj -> l1 -> cert`, and
/// `cert ->[dev] spore -> proj -> l1 -> cert`. All three share crates (`cert`, `proj`, `l1`), so a
/// correct, complete SCC decomposition reports them as ONE connected cyclic component — that's the
/// honest answer (see `checks::check_acyclic_including_dev` doc), not three separate reports.
#[test]
fn dev_cycle_check_detects_known_cert_cycle_shape() {
    let graph = Graph::from_edges(&[
        ("select", "cert", &[DepKind::Dev]),
        ("cert", "vsa", &[DepKind::Normal]),
        ("vsa", "select", &[DepKind::Normal]),
        ("cert", "proj", &[DepKind::Dev]),
        ("proj", "l1", &[DepKind::Normal]),
        ("l1", "cert", &[DepKind::Normal]),
        ("cert", "spore", &[DepKind::Dev]),
        ("spore", "proj", &[DepKind::Normal]),
    ]);
    let violations = check_acyclic_including_dev(&graph);
    assert_eq!(
        violations.len(),
        1,
        "shared members (cert/proj/l1) collapse the three named cycles into one SCC: {violations:?}"
    );
    for c in ["select", "cert", "vsa", "proj", "l1", "spore"] {
        assert!(violations[0].message.contains(c));
    }
}

// ---- M-879: named rules ------------------------------------------------------------------------

#[test]
fn forbidden_target_prefix_rule_flags_interp_std_dep() {
    let cfg = StrataConfig::parse(RULE_FORBIDDEN_PREFIX).expect("fixture parses");
    let graph = Graph::from_edges(&[("mycelium-interp", "mycelium-std-runtime", &[DepKind::Dev])]);
    let violations = check_named_rules(&graph, &cfg);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].rule, "no-interp-std-dep");
    assert!(violations[0].message.contains("mycelium-interp"));
    assert!(violations[0].message.contains("mycelium-std-runtime"));
}

#[test]
fn forbidden_target_prefix_rule_passes_when_clean() {
    let cfg = StrataConfig::parse(RULE_FORBIDDEN_PREFIX).expect("fixture parses");
    let graph = Graph::from_edges(&[("mycelium-interp", "mycelium-core", &[DepKind::Normal])]);
    let violations = check_named_rules(&graph, &cfg);
    assert!(violations.is_empty());
}

#[test]
fn tier_order_rule_flags_upward_edge() {
    let cfg = StrataConfig::parse(RULE_TIER_ORDER).expect("fixture parses");
    // mycelium-mlir (core) -> mycelium-std-runtime (std): the real, known HEAD anomaly.
    let graph = Graph::from_edges(&[("mycelium-mlir", "mycelium-std-runtime", &[DepKind::Normal])]);
    let violations = check_named_rules(&graph, &cfg);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].rule, "no-upward-tier-edges");
}

#[test]
fn tier_order_rule_allows_same_tier_and_downward_edges() {
    let cfg = StrataConfig::parse(RULE_TIER_ORDER).expect("fixture parses");
    let graph = Graph::from_edges(&[
        ("mycelium-mlir", "mycelium-core", &[DepKind::Normal]), // core -> core: same tier, OK
        ("mycelium-cli", "mycelium-std-runtime", &[DepKind::Normal]), // tools -> std: downward, OK
        ("mycelium-cli", "mycelium-mlir", &[DepKind::Normal]),  // tools -> core: downward, OK
    ]);
    let violations = check_named_rules(&graph, &cfg);
    assert!(violations.is_empty(), "{violations:?}");
}

#[test]
fn unknown_named_rule_kind_is_flagged_not_silently_skipped() {
    let cfg = StrataConfig::parse(RULE_UNKNOWN_KIND).expect("fixture parses");
    let graph = Graph::from_edges(&[("mycelium-core", "mycelium-core", &[DepKind::Normal])]);
    let violations = check_named_rules(&graph, &cfg);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].rule, "named-rule-config-error");
    assert!(violations[0].message.contains("bogus-kind"));
}

// ---- StrataConfig: the real, shipped data file must parse and be self-consistent --------------

#[test]
fn embedded_strata_config_parses_and_covers_the_whole_workspace() {
    let cfg = StrataConfig::embedded();
    assert_eq!(cfg.tier_order, vec!["core", "std", "tools"]);
    assert_eq!(
        cfg.strata.len(),
        cfg.tiers.len(),
        "every crate in `[strata]` must also appear in `[tiers]` (and vice versa)"
    );
    assert!(
        cfg.strata.len() >= 52,
        "expected the full workspace, got {}",
        cfg.strata.len()
    );
    for crate_name in cfg.strata.keys() {
        assert!(
            cfg.tiers.contains_key(crate_name),
            "{crate_name} is in [strata] but missing from [tiers]"
        );
    }
    assert_eq!(
        cfg.named_rules.len(),
        2,
        "M-879 ships exactly two named rules"
    );
    let ids: Vec<&str> = cfg.named_rules.iter().map(|r| r.id.as_str()).collect();
    assert!(ids.contains(&"no-interp-std-dep"));
    assert!(ids.contains(&"no-upward-tier-edges"));
}

/// `run_all` composes every check; a trivially clean graph (against the embedded, real config)
/// must not error out even though it reports nothing meaningful — this just proves the pipeline
/// wires together (parse config -> build graph -> run every check) without panicking.
#[test]
fn run_all_does_not_panic_on_an_empty_graph() {
    let cfg = StrataConfig::embedded();
    let graph = Graph::from_edges(&[]);
    let violations = run_all(&graph, &cfg);
    assert!(
        violations.is_empty(),
        "an empty graph has nothing to violate"
    );
}
