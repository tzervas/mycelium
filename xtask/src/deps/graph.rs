//! In-workspace dependency graph, built from `cargo_metadata::Metadata`.
//!
//! Only edges between **workspace members** matter for the acyclic-deps invariant (M-877/878/879)
//! — external crates.io dependencies can't participate in an intra-workspace layering violation or
//! cycle, so they're dropped at construction. Guarantee: **Exact** — this is a direct, lossless
//! read of `cargo metadata`'s resolver graph (`resolve.nodes[].deps[].dep_kinds[]`), not a
//! heuristic.

use std::collections::{BTreeMap, BTreeSet};

use cargo_metadata::{DependencyKind, Metadata, PackageId};

/// The kind of an intra-workspace dependency edge. `Build` is modeled for completeness (cargo
/// distinguishes it) even though the current workspace has none — never-silent (G2): if one is
/// ever added, it is classified, not dropped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DepKind {
    Normal,
    Dev,
    Build,
}

impl DepKind {
    pub fn as_str(self) -> &'static str {
        match self {
            DepKind::Normal => "normal",
            DepKind::Dev => "dev",
            DepKind::Build => "build",
        }
    }

    fn from_cargo(k: DependencyKind) -> Self {
        match k {
            DependencyKind::Development => DepKind::Dev,
            DependencyKind::Build => DepKind::Build,
            // `Normal` and the `#[doc(hidden)] Unknown` catch-all both resolve to `Normal`: an
            // unrecognized-but-present kind is safer treated as the most restrictive (normal,
            // always-compiled) category than silently dropped.
            _ => DepKind::Normal,
        }
    }
}

/// One directed edge `from -> to` with every kind cargo reported for it (a single dependency can
/// be both `normal` and `dev` if declared in both sections, though that's rare).
#[derive(Debug, Clone)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub kinds: BTreeSet<DepKind>,
}

/// The intra-workspace dependency graph: crate name -> its outgoing edges.
#[derive(Debug, Default)]
pub struct Graph {
    pub crates: BTreeSet<String>,
    pub edges: Vec<Edge>,
    adjacency: BTreeMap<String, Vec<usize>>, // crate -> indices into `edges`
}

impl Graph {
    /// Build the graph from a parsed `cargo metadata` document, keeping only edges whose source
    /// AND target are workspace members. `Exact`: this is a faithful reduction of the resolver's
    /// own graph, not an approximation.
    pub fn from_metadata(meta: &Metadata) -> Self {
        let members: BTreeSet<&PackageId> = meta.workspace_members.iter().collect();
        let id_to_name: BTreeMap<&PackageId, &str> = meta
            .packages
            .iter()
            .filter(|p| members.contains(&p.id))
            .map(|p| (&p.id, p.name.as_str()))
            .collect();

        let mut graph = Graph::default();
        for name in id_to_name.values() {
            graph.crates.insert((*name).to_owned());
        }

        let Some(resolve) = &meta.resolve else {
            // `cargo metadata` always includes `resolve` unless invoked with `--no-deps`; an
            // absent resolve graph means we cannot check anything — never-silent: an empty graph
            // (0 crates) makes every downstream check's "crate not in map" path fire loudly rather
            // than pretending success.
            return graph;
        };

        for node in &resolve.nodes {
            let Some(&from) = id_to_name.get(&node.id) else {
                continue; // not a workspace member; skip (this node is an external dep of one)
            };
            // Merge by (from, to) so multiple `dep_kinds` entries for the same target collapse
            // into one edge with a kind-set, rather than duplicate edges.
            let mut per_target: BTreeMap<&str, BTreeSet<DepKind>> = BTreeMap::new();
            for dep in &node.deps {
                let Some(&to) = id_to_name.get(&dep.pkg) else {
                    continue; // external dependency; not part of the intra-workspace graph
                };
                if to == from {
                    continue; // no self-edges (shouldn't occur, but never-silent to skip not panic)
                }
                let kinds = per_target.entry(to).or_default();
                for dk in &dep.dep_kinds {
                    kinds.insert(DepKind::from_cargo(dk.kind));
                }
                // A `dep_kinds` list can legitimately be empty pre-1.41-format metadata; treat an
                // empty list as `Normal` (cargo's own default) rather than silently dropping the edge.
                if dep.dep_kinds.is_empty() {
                    kinds.insert(DepKind::Normal);
                }
            }
            for (to, kinds) in per_target {
                let idx = graph.edges.len();
                graph.edges.push(Edge {
                    from: from.to_owned(),
                    to: to.to_owned(),
                    kinds,
                });
                graph
                    .adjacency
                    .entry(from.to_owned())
                    .or_default()
                    .push(idx);
            }
        }
        graph
    }

    /// Outgoing edges of `crate_name`, restricted to edges carrying at least one of `kinds`.
    pub fn edges_from<'a>(
        &'a self,
        crate_name: &str,
        kinds: &'a [DepKind],
    ) -> impl Iterator<Item = &'a Edge> + 'a {
        self.adjacency
            .get(crate_name)
            .into_iter()
            .flatten()
            .map(move |&i| &self.edges[i])
            .filter(move |e| kinds.iter().any(|k| e.kinds.contains(k)))
    }

    /// All edges carrying at least one of `kinds`.
    pub fn all_edges<'a>(&'a self, kinds: &'a [DepKind]) -> impl Iterator<Item = &'a Edge> + 'a {
        self.edges
            .iter()
            .filter(move |e| kinds.iter().any(|k| e.kinds.contains(k)))
    }

    /// Strongly-connected components of the subgraph restricted to edges carrying at least one of
    /// `kinds`, via Tarjan's algorithm. `Exact` — deterministic, complete: unlike a single
    /// DFS-back-edge pass (which reports only the cycles a particular traversal order happens to
    /// hit), an SCC decomposition finds every crate that participates in *some* cycle, so no cyclic
    /// participant is missed regardless of iteration order. Returns only components with >1 member
    /// (a lone crate is never "a cycle" even if the algorithm technically also emits it as a
    /// trivial 1-node component internally).
    pub fn nontrivial_sccs(&self, kinds: &[DepKind]) -> Vec<Vec<String>> {
        struct Tarjan<'g> {
            graph: &'g Graph,
            kinds: Vec<DepKind>,
            index_counter: usize,
            index: BTreeMap<String, usize>,
            lowlink: BTreeMap<String, usize>,
            on_stack: BTreeSet<String>,
            stack: Vec<String>,
            result: Vec<Vec<String>>,
        }
        impl Tarjan<'_> {
            fn visit(&mut self, v: &str) {
                self.index.insert(v.to_owned(), self.index_counter);
                self.lowlink.insert(v.to_owned(), self.index_counter);
                self.index_counter += 1;
                self.stack.push(v.to_owned());
                self.on_stack.insert(v.to_owned());

                let targets: Vec<String> = self
                    .graph
                    .edges_from(v, &self.kinds)
                    .map(|e| e.to.clone())
                    .collect();
                for w in targets {
                    if !self.index.contains_key(&w) {
                        self.visit(&w);
                        let w_low = self.lowlink[&w];
                        let v_low = self.lowlink[v];
                        self.lowlink.insert(v.to_owned(), v_low.min(w_low));
                    } else if self.on_stack.contains(&w) {
                        let w_idx = self.index[&w];
                        let v_low = self.lowlink[v];
                        self.lowlink.insert(v.to_owned(), v_low.min(w_idx));
                    }
                }

                if self.lowlink[v] == self.index[v] {
                    let mut comp = Vec::new();
                    loop {
                        let w = self.stack.pop().expect("stack non-empty: v is on it");
                        self.on_stack.remove(&w);
                        let is_v = w == v;
                        comp.push(w);
                        if is_v {
                            break;
                        }
                    }
                    self.result.push(comp);
                }
            }
        }

        let mut t = Tarjan {
            graph: self,
            kinds: kinds.to_vec(),
            index_counter: 0,
            index: BTreeMap::new(),
            lowlink: BTreeMap::new(),
            on_stack: BTreeSet::new(),
            stack: Vec::new(),
            result: Vec::new(),
        };
        for c in &self.crates {
            if !t.index.contains_key(c) {
                t.visit(c);
            }
        }
        t.result
            .into_iter()
            .filter(|c| c.len() > 1)
            .map(|mut c| {
                c.sort();
                c
            })
            .collect()
    }
}

#[cfg(test)]
impl Graph {
    /// Test-only constructor: build a graph directly from `(from, to, kinds)` triples, bypassing
    /// `cargo_metadata` JSON parsing entirely. This lets the check-logic tests
    /// (`xtask/src/tests/deps.rs`) exercise `checks.rs` against small, purpose-built synthetic
    /// graphs (the negative fixtures for M-877/M-878) without hand-writing a full metadata
    /// document for every case; `from_metadata`'s own parsing is covered separately by a real
    /// (trimmed) metadata JSON fixture.
    pub fn from_edges(edges: &[(&str, &str, &[DepKind])]) -> Self {
        let mut graph = Graph::default();
        for &(from, to, kinds) in edges {
            graph.crates.insert(from.to_owned());
            graph.crates.insert(to.to_owned());
            let idx = graph.edges.len();
            graph.edges.push(Edge {
                from: from.to_owned(),
                to: to.to_owned(),
                kinds: kinds.iter().copied().collect(),
            });
            graph
                .adjacency
                .entry(from.to_owned())
                .or_default()
                .push(idx);
        }
        graph
    }
}
