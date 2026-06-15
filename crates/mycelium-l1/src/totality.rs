//! The **structural totality checker** (RFC-0007 §4.5; T3.4) — *outside* the trusted kernel: its
//! verdict gates the `matured` privilege, never meaning (a wrong verdict can mis-gate a
//! promotion; semantics stay with the fuel-guarded evaluator).
//!
//! Classification (Foetus-style structural descent, v0):
//! - no (direct or mutual) recursion → **Total**;
//! - self-recursion where *every* recursive call passes, in some fixed argument position, a
//!   variable **structurally smaller** than that parameter (bound by a `Match` alternative on the
//!   parameter or on an already-smaller variable — descent is transitive) → **Total**;
//! - anything else (including mutual recursion, R7-Q3) → **Partial** — an honest classification,
//!   not an error.

use std::collections::{BTreeMap, BTreeSet};

use crate::ast::{Arm, Expr, FnDecl, Pattern};

/// The divergence bit (RFC-0007 §4.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Totality {
    /// Checked total: terminates under the reference evaluator for every sufficiently large fuel.
    Total,
    /// Not certified total (may or may not terminate) — honest, not an error.
    Partial,
}

/// Classify every function in the table.
#[must_use]
pub fn classify_all(fns: &BTreeMap<String, FnDecl>) -> BTreeMap<String, Totality> {
    // Call graph.
    let mut calls: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    for (name, fd) in fns {
        let mut out = BTreeSet::new();
        collect_calls(&fd.body, fns, &mut out);
        calls.insert(name, out);
    }
    let mut result = BTreeMap::new();
    for (name, fd) in fns {
        let self_rec = calls[name.as_str()].contains(name);
        let mutual = calls[name.as_str()]
            .iter()
            .any(|callee| callee != name && reaches(callee, name, &calls));
        // Total iff non-recursive, or self-recursive (not mutual) with structural descent.
        // Mutual recursion (R7-Q3) stays Partial — an honest deferral, not an error.
        let t = if !mutual && (!self_rec || descends(fd)) {
            Totality::Total
        } else {
            Totality::Partial
        };
        result.insert(name.clone(), t);
    }
    result
}

/// Does `from` reach `target` through the call graph (cycle detection for mutual recursion)?
fn reaches(from: &str, target: &str, calls: &BTreeMap<&str, BTreeSet<String>>) -> bool {
    let mut seen = BTreeSet::new();
    let mut stack = vec![from.to_owned()];
    while let Some(f) = stack.pop() {
        if !seen.insert(f.clone()) {
            continue;
        }
        if let Some(cs) = calls.get(f.as_str()) {
            for c in cs {
                if c == target {
                    return true;
                }
                stack.push(c.clone());
            }
        }
    }
    false
}

fn collect_calls(e: &Expr, fns: &BTreeMap<String, FnDecl>, out: &mut BTreeSet<String>) {
    walk(e, &mut |x| {
        if let Expr::App { head, .. } = x {
            if let Expr::Path(p) = head.as_ref() {
                if p.0.len() == 1 && fns.contains_key(&p.0[0]) {
                    out.insert(p.0[0].clone());
                }
            }
        }
    });
}

fn walk(e: &Expr, f: &mut impl FnMut(&Expr)) {
    f(e);
    match e {
        Expr::Let { bound, body, .. } => {
            walk(bound, f);
            walk(body, f);
        }
        Expr::If { cond, conseq, alt } => {
            walk(cond, f);
            walk(conseq, f);
            walk(alt, f);
        }
        Expr::Match { scrutinee, arms } => {
            walk(scrutinee, f);
            for a in arms {
                walk(&a.body, f);
            }
        }
        // A `for` is bounded by construction (RFC-0007 §4.8) — it adds no recursion of its own;
        // only the calls inside its sub-expressions matter.
        Expr::For { xs, init, body, .. } => {
            walk(xs, f);
            walk(init, f);
            walk(body, f);
        }
        Expr::Swap { value, .. } => walk(value, f),
        Expr::Wild(b) | Expr::Spore(b) => walk(b, f),
        Expr::App { head, args } => {
            walk(head, f);
            for a in args {
                walk(a, f);
            }
        }
        Expr::Ascribe(b, _) => walk(b, f),
        Expr::Path(_) | Expr::Lit(_) => {}
    }
}

/// Structural-descent check for a self-recursive `fd`: there must exist a parameter position `i`
/// such that **every** recursive call's `i`-th argument is a variable strictly smaller than
/// parameter `i` (smallness is seeded by `Match`-alternative binders and is transitive).
fn descends(fd: &FnDecl) -> bool {
    let params: Vec<&str> = fd
        .sig
        .value_params
        .iter()
        .map(|p| p.name.as_str())
        .collect();
    (0..params.len()).any(|i| check_position(fd, &params, i))
}

fn check_position(fd: &FnDecl, params: &[&str], i: usize) -> bool {
    // smaller[v] = v is strictly structurally smaller than params[i].
    let mut ok = true;
    descend_walk(
        &fd.body,
        &fd.sig.name,
        params[i],
        i,
        &mut BTreeSet::new(),
        &mut ok,
    );
    ok
}

/// Walk tracking the set of variables smaller-than the designated parameter; check every
/// recursive call. `smaller` grows at `Match` alternatives whose scrutinee is the parameter or an
/// already-smaller variable.
fn descend_walk(
    e: &Expr,
    fname: &str,
    param: &str,
    pos: usize,
    smaller: &mut BTreeSet<String>,
    ok: &mut bool,
) {
    match e {
        Expr::App { head, args } => {
            if let Expr::Path(p) = head.as_ref() {
                if p.0.len() == 1 && p.0[0] == fname {
                    // A recursive call: its pos-th argument must be a smaller variable.
                    let good = args.get(pos).is_some_and(|a| match a {
                        Expr::Path(v) => v.0.len() == 1 && smaller.contains(&v.0[0]),
                        _ => false,
                    });
                    if !good {
                        *ok = false;
                    }
                }
            }
            descend_walk(head, fname, param, pos, smaller, ok);
            for a in args {
                descend_walk(a, fname, param, pos, smaller, ok);
            }
        }
        Expr::Match { scrutinee, arms } => {
            descend_walk(scrutinee, fname, param, pos, smaller, ok);
            let scrut_small = match scrutinee.as_ref() {
                Expr::Path(p) if p.0.len() == 1 => p.0[0] == param || smaller.contains(&p.0[0]),
                _ => false,
            };
            for Arm { pattern, body } in arms {
                // Every binder the pattern introduces SHADOWS any outer variable of the same name,
                // so its prior smallness must not leak into the arm body (A4-01: otherwise a binder
                // reusing an outer `smaller` name lets a non-decreasing recursive call look
                // structural). Drop all introduced binders, restore afterwards — mirroring the
                // `Let`/`For` discipline. Only a constructor sub-binder of a *smaller* scrutinee is
                // itself genuinely smaller, so re-add just those.
                let mut introduced = Vec::new();
                pattern_binders(pattern, &mut introduced);
                let mut restore = Vec::new();
                for b in &introduced {
                    if smaller.remove(b) {
                        restore.push(b.clone());
                    }
                }
                let mut added = Vec::new();
                if scrut_small {
                    if let Pattern::Ctor(_, subs) = pattern {
                        for s in subs {
                            if let Pattern::Ident(b) = s {
                                if smaller.insert(b.clone()) {
                                    added.push(b.clone());
                                }
                            }
                        }
                    }
                }
                descend_walk(body, fname, param, pos, smaller, ok);
                for b in added {
                    smaller.remove(&b);
                }
                for b in restore {
                    smaller.insert(b);
                }
            }
        }
        Expr::Let {
            bound, body, name, ..
        } => {
            descend_walk(bound, fname, param, pos, smaller, ok);
            // A rebinding shadows; conservatively drop smallness for the shadowed name.
            let was = smaller.remove(name);
            descend_walk(body, fname, param, pos, smaller, ok);
            if was {
                smaller.insert(name.clone());
            }
        }
        Expr::If { cond, conseq, alt } => {
            descend_walk(cond, fname, param, pos, smaller, ok);
            descend_walk(conseq, fname, param, pos, smaller, ok);
            descend_walk(alt, fname, param, pos, smaller, ok);
        }
        Expr::For {
            x,
            xs,
            acc,
            init,
            body,
        } => {
            descend_walk(xs, fname, param, pos, smaller, ok);
            descend_walk(init, fname, param, pos, smaller, ok);
            // The binders shadow; conservatively drop smallness for the shadowed names.
            let had_x = smaller.remove(x);
            let had_acc = smaller.remove(acc);
            descend_walk(body, fname, param, pos, smaller, ok);
            if had_x {
                smaller.insert(x.clone());
            }
            if had_acc {
                smaller.insert(acc.clone());
            }
        }
        Expr::Swap { value, .. } => descend_walk(value, fname, param, pos, smaller, ok),
        Expr::Wild(b) | Expr::Spore(b) => descend_walk(b, fname, param, pos, smaller, ok),
        Expr::Ascribe(b, _) => descend_walk(b, fname, param, pos, smaller, ok),
        Expr::Path(_) | Expr::Lit(_) => {}
    }
}

/// Collect every variable a pattern binds, recursively — so a `Match` arm can shadow them all
/// (A4-01). Wildcards and literals bind nothing.
fn pattern_binders(p: &Pattern, out: &mut Vec<String>) {
    match p {
        Pattern::Ident(b) => out.push(b.clone()),
        Pattern::Ctor(_, subs) => {
            for s in subs {
                pattern_binders(s, out);
            }
        }
        Pattern::Wildcard | Pattern::Lit(_) => {}
    }
}
