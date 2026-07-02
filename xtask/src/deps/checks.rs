//! The four acyclic-deps checks (M-877/M-878/M-879) over an in-workspace [`Graph`], validated
//! against a [`StrataConfig`]. Every violation is a fully-formed, never-silent (G2) message: the
//! offending edge, its kind, the rule it broke, and that rule's basis reference — never a bare
//! `false`/exit code.

use crate::deps::graph::{DepKind, Graph};
use crate::deps::strata::StrataConfig;

/// One rule violation, ready to print. `rule` is the stable rule id (matches a
/// `deps-strata.toml` named-rule `id`, or one of the two structural rule ids below) so callers
/// (and tests) can assert on *which* rule fired without parsing prose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    pub rule: &'static str,
    pub message: String,
}

/// M-877: every NORMAL edge must go from a strictly higher frozen stratum to a strictly lower
/// one. `Exact` given the frozen `[strata]` map (a real `>` comparison); the map itself is
/// `Empirical` (see `deps-strata.toml` header) — a crate absent from the map is its own violation,
/// never silently skipped.
pub fn check_normal_downward(graph: &Graph, cfg: &StrataConfig) -> Vec<Violation> {
    let mut out = Vec::new();
    for edge in graph.all_edges(&[DepKind::Normal]) {
        let from_s = cfg.strata.get(&edge.from);
        let to_s = cfg.strata.get(&edge.to);
        match (from_s, to_s) {
            (Some(&fs), Some(&ts)) => {
                if fs <= ts {
                    out.push(Violation {
                        rule: "normal-downward-only",
                        message: format!(
                            "{} (stratum {fs}) -> {} (stratum {ts}): normal dep must target a \
                             strictly lower stratum than its source (see {})",
                            edge.from, edge.to, cfg.meta.basis_ref
                        ),
                    });
                }
            }
            _ => {
                out.push(Violation {
                    rule: "normal-downward-only",
                    message: format!(
                        "{} -> {}: one or both crates are missing from `[strata]` in \
                         deps-strata.toml — cannot verify layering (add the missing entry)",
                        edge.from, edge.to
                    ),
                });
            }
        }
    }
    out
}

/// M-878: cargo never rejects a `[dev-dependencies]` cycle, so this check does what cargo won't —
/// find every strongly-connected component over the combined NORMAL+DEV+BUILD graph. `Exact`:
/// Tarjan's SCC decomposition is deterministic and complete (see `Graph::nontrivial_sccs` for why
/// this is preferred over enumerating individual back-edge cycles, which is traversal-order
/// dependent and can under-report). Any component with more than one crate is a cycle; the crates
/// participating in the same SCC may collapse several "named" cycles (e.g. the three historical
/// cert dev-cycles) into a single reported component when they share members — that IS the
/// correct, complete answer, not a bug: report every edge, so a reader can still see each
/// contributing path.
pub fn check_acyclic_including_dev(graph: &Graph) -> Vec<Violation> {
    let kinds = [DepKind::Normal, DepKind::Dev, DepKind::Build];
    let mut out = Vec::new();
    for scc in graph.nontrivial_sccs(&kinds) {
        let members: std::collections::BTreeSet<&str> = scc.iter().map(String::as_str).collect();
        let mut edge_lines: Vec<String> = graph
            .all_edges(&kinds)
            .filter(|e| members.contains(e.from.as_str()) && members.contains(e.to.as_str()))
            .map(|e| {
                let kinds_str: Vec<&str> = e.kinds.iter().map(|k| k.as_str()).collect();
                format!("{} -[{}]-> {}", e.from, kinds_str.join("+"), e.to)
            })
            .collect();
        edge_lines.sort();
        out.push(Violation {
            rule: "acyclic-including-dev",
            message: format!(
                "cycle among {{{}}} (a dev-dependency edge closes this loop; cargo does not \
                 reject dev-dep cycles). Contributing edges: {}",
                scc.join(", "),
                edge_lines.join("; ")
            ),
        });
    }
    out
}

/// Run every `[[named_rules]]` entry from `deps-strata.toml` (M-879). Data-driven: each rule's
/// `kind` selects its evaluation, so adding a new named rule means editing the TOML, not this
/// function's control flow for the two kinds already understood. An unrecognized `kind` is itself
/// a violation (never-silent) rather than a silently-skipped rule.
pub fn check_named_rules(graph: &Graph, cfg: &StrataConfig) -> Vec<Violation> {
    let mut out = Vec::new();
    for rule in &cfg.named_rules {
        match rule.kind.as_str() {
            "forbidden-target-prefix" => {
                out.extend(check_forbidden_target_prefix(graph, rule));
            }
            "tier-order" => {
                out.extend(check_tier_order(graph, cfg, rule));
            }
            other => out.push(Violation {
                rule: "named-rule-config-error",
                message: format!(
                    "named_rule '{}' has unrecognized kind '{other}' — cannot evaluate (fix \
                     deps-strata.toml)",
                    rule.id
                ),
            }),
        }
    }
    out
}

fn check_forbidden_target_prefix(
    graph: &Graph,
    rule: &crate::deps::strata::NamedRule,
) -> Vec<Violation> {
    let (Some(source), Some(prefix)) = (&rule.source, &rule.forbidden_target_prefix) else {
        return vec![Violation {
            rule: "named-rule-config-error",
            message: format!(
                "named_rule '{}' (kind = forbidden-target-prefix) is missing `source` or \
                 `forbidden_target_prefix` in deps-strata.toml",
                rule.id
            ),
        }];
    };
    let kinds = [DepKind::Normal, DepKind::Dev, DepKind::Build];
    graph
        .edges_from(source, &kinds)
        .filter(|e| e.to.starts_with(prefix.as_str()))
        .map(|e| {
            let kinds_str: Vec<&str> = e.kinds.iter().map(|k| k.as_str()).collect();
            Violation {
                rule: "no-interp-std-dep",
                message: format!(
                    "{} -[{}]-> {}: violates named rule '{}' — {} (see {})",
                    e.from,
                    kinds_str.join("+"),
                    e.to,
                    rule.id,
                    rule.description,
                    rule.basis_ref
                ),
            }
        })
        .collect()
}

fn check_tier_order(
    graph: &Graph,
    cfg: &StrataConfig,
    rule: &crate::deps::strata::NamedRule,
) -> Vec<Violation> {
    let kinds = [DepKind::Normal, DepKind::Dev, DepKind::Build];
    let mut out = Vec::new();
    for edge in graph.all_edges(&kinds) {
        let from_tier = cfg.tiers.get(&edge.from);
        let to_tier = cfg.tiers.get(&edge.to);
        let (Some(from_tier), Some(to_tier)) = (from_tier, to_tier) else {
            out.push(Violation {
                rule: "named-rule-config-error",
                message: format!(
                    "{} -> {}: one or both crates are missing from `[tiers]` in \
                     deps-strata.toml — cannot verify '{}' (add the missing entry)",
                    edge.from, edge.to, rule.id
                ),
            });
            continue;
        };
        let (Some(from_idx), Some(to_idx)) = (cfg.tier_index(from_tier), cfg.tier_index(to_tier))
        else {
            out.push(Violation {
                rule: "named-rule-config-error",
                message: format!(
                    "{} -> {}: tier '{}' or '{}' is not listed in `tier_order` — cannot verify \
                     '{}'",
                    edge.from, edge.to, from_tier, to_tier, rule.id
                ),
            });
            continue;
        };
        if from_idx < to_idx {
            let kinds_str: Vec<&str> = edge.kinds.iter().map(|k| k.as_str()).collect();
            out.push(Violation {
                rule: "no-upward-tier-edges",
                message: format!(
                    "{} -[{}]-> {} (tier '{from_tier}' -> tier '{to_tier}'): violates named \
                     rule '{}' — {} (see {})",
                    edge.from,
                    kinds_str.join("+"),
                    edge.to,
                    rule.id,
                    rule.description,
                    rule.basis_ref
                ),
            });
        }
    }
    out
}

/// Run all four checks (M-877 base + M-878 dev-cycle + the two M-879 named rules) and return every
/// violation found, in a stable order.
pub fn run_all(graph: &Graph, cfg: &StrataConfig) -> Vec<Violation> {
    let mut out = Vec::new();
    out.extend(check_normal_downward(graph, cfg));
    out.extend(check_acyclic_including_dev(graph));
    out.extend(check_named_rules(graph, cfg));
    out
}
