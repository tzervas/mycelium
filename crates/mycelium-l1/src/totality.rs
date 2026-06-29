//! The **structural totality checker** (RFC-0007 §4.5; T3.4) — *outside* the trusted kernel: its
//! verdict gates the `matured` privilege, never meaning (a wrong verdict can mis-gate a
//! promotion; semantics stay with the fuel-guarded evaluator).
//!
//! Classification (Foetus-style structural descent, v0):
//! - no (direct or mutual) recursion → **Total**;
//! - self-recursion where *every* recursive call passes, in some fixed argument position, a
//!   variable **structurally smaller** than that parameter (bound by a `Match` alternative on the
//!   parameter or on an already-smaller variable — descent is transitive) → **Total**;
//! - **mutual recursion** (a `FixGroup` / strongly-connected call-graph component, RFC-0001 r5,
//!   R7-Q3) where there is a **mutual structural descent**: a designated argument position `p(f)`
//!   for each member `f` such that *every* call from a member `f` to a member `g` passes, in `g`'s
//!   position `p(g)`, a variable structurally smaller than `f`'s parameter `p(f)` → **Total**.
//!   Self-recursion is the size-1 case. Sound by one well-founded measure: the structural size of
//!   the designated argument strictly decreases at every call along any path through the group, so
//!   no infinite call path exists;
//! - anything else (a non-productive cycle, a group too large to search, or one this structural
//!   criterion cannot witness) → **Partial** — an honest, incomplete classification, not an error.
//!
//! The checker is **sound, not complete**: it never classifies a non-terminating group `Total`
//! (that would mis-grant `matured`), but it may leave a terminating group `Partial`. Widening it
//! (here, from self- to mutual-descent) only ever *adds* `Total` verdicts that the well-founded
//! measure justifies — it never relaxes the bar.

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

/// A bound on the position-assignment search for a mutual group (∏ of member arities). Beyond it
/// the group stays `Partial` — sound (we never *over*-classify), just incomplete, and well past any
/// realistic hand-written mutual cycle.
const MAX_ASSIGNMENTS: usize = 4096;

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
    for scc in strongly_connected(fns, &calls) {
        // A component is recursive iff it has > 1 member (necessarily a cycle) or its single member
        // calls itself directly. A non-recursive definition is `Total` with no descent obligation.
        let recursive = scc.len() > 1 || calls[scc[0].as_str()].contains(&scc[0]);
        let total = !recursive || group_descends(&scc, fns);
        let t = if total {
            Totality::Total
        } else {
            Totality::Partial
        };
        for name in scc {
            result.insert(name, t);
        }
    }
    result
}

/// Partition the functions into strongly-connected components of the call graph (each is a
/// `FixGroup`, RFC-0001 r5). Two functions share a component iff they are mutually reachable;
/// that relation is an equivalence, so a greedy grouping yields the full components. Deterministic
/// (iteration follows the `BTreeMap` key order).
fn strongly_connected(
    fns: &BTreeMap<String, FnDecl>,
    calls: &BTreeMap<&str, BTreeSet<String>>,
) -> Vec<Vec<String>> {
    let mut assigned: BTreeSet<&str> = BTreeSet::new();
    let mut sccs = Vec::new();
    for name in fns.keys() {
        if assigned.contains(name.as_str()) {
            continue;
        }
        let mut group = vec![name.clone()];
        assigned.insert(name);
        for other in fns.keys() {
            if assigned.contains(other.as_str()) {
                continue;
            }
            if reaches(name, other, calls) && reaches(other, name, calls) {
                group.push(other.clone());
                assigned.insert(other);
            }
        }
        sccs.push(group);
    }
    sccs
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
    walk_expr(e, &mut |x| {
        if let Expr::App { head, .. } = x {
            if let Expr::Path(p) = head.as_ref() {
                if p.0.len() == 1 && fns.contains_key(&p.0[0]) {
                    out.insert(p.0[0].clone());
                }
            }
        }
    });
}

/// The shared **pre-order `Expr` traversal** (M-641): visit `e` with `f`, then recurse into every
/// sub-expression (calling `f` on each in turn). One canonical structural walk reused by every pass
/// that needs to fold a *stateless* visitor over an expression tree — here totality's `collect_calls`
/// and the elaborator's call-set collector (`crate::elab`). It is the structure only; each caller
/// supplies its own visitor action, so factoring it changes no collected set.
///
/// Passes that thread *context* down the tree (e.g. the totality descent measure in
/// [`descend_walk`], which shadows binders per `Match` arm) are deliberately **not** expressed over
/// this — their per-node state is not a plain `FnMut(&Expr)`, and collapsing them would lose the
/// scoping that keeps the analysis sound (A4-01).
pub(crate) fn walk_expr(e: &Expr, f: &mut impl FnMut(&Expr)) {
    f(e);
    match e {
        Expr::Let { bound, body, .. } => {
            walk_expr(bound, f);
            walk_expr(body, f);
        }
        Expr::If { cond, conseq, alt } => {
            walk_expr(cond, f);
            walk_expr(conseq, f);
            walk_expr(alt, f);
        }
        Expr::Match { scrutinee, arms } => {
            walk_expr(scrutinee, f);
            for a in arms {
                walk_expr(&a.body, f);
            }
        }
        // A `for` is bounded by construction (RFC-0007 §4.8) — it adds no recursion of its own;
        // only the calls inside its sub-expressions matter.
        Expr::For { xs, init, body, .. } => {
            walk_expr(xs, f);
            walk_expr(init, f);
            walk_expr(body, f);
        }
        Expr::Swap { value, .. } => walk_expr(value, f),
        // `with paradigm` is pure surface scoping (stripped by resolution before this runs); recurse
        // transparently into the body in case totality is consulted on an unresolved tree.
        Expr::WithParadigm { body, .. } => walk_expr(body, f),
        // `wild` is the audited/opaque FFI escape (M-661): its body is trusted foreign code, **not**
        // analyzable Mycelium, so the shared traversal treats it as a LEAF. The `wild` node itself is
        // still visited by `f` above (so effect coverage credits it the `ffi` source — M-661/§8-Q6),
        // but its interior is **never descended**: effects/calls/recursion inside a `wild` body do not
        // leak into the enclosing fn's analysis — consistent with `Cx::check_wild` not recursively
        // checking the body (audited, not verified; VR-5/ADR-014). Execution is staged (`elab` →
        // `Residual`). `spore(value)`, by contrast, wraps a *real* value expression (deferred —
        // E2-5/M-260), so it recurses transparently.
        Expr::Wild(_) => {}
        Expr::Spore(b) => walk_expr(b, f),
        // M-664: `consume <expr>` wraps a real value expression — walk it transparently so any
        // calls inside the operand are still seen by the call-set/totality collectors.
        Expr::Consume(b) => walk_expr(b, f),
        // A `lambda` body is deferred (M-704; never executes in v0), but walk it transparently so
        // any calls inside are still seen by the call-set/totality collectors (conservative).
        Expr::Lambda { body, .. } => walk_expr(body, f),
        // A `colony` block's calls are exactly the calls inside its `hypha` bodies (RFC-0008 §4.7).
        // Each hypha body is walked transparently so the call-set / totality collectors see them.
        Expr::Colony(hyphae) => {
            for h in hyphae {
                walk_expr(&h.body, f);
            }
        }
        Expr::App { head, args } => {
            walk_expr(head, f);
            for a in args {
                walk_expr(a, f);
            }
        }
        Expr::Ascribe(b, _) => walk_expr(b, f),
        // DN-58 §A/§B (M-667): `fuse(a, b)` and `reclaim(policy) { body }` — walk sub-expressions
        // transparently so the call-set / totality collectors see any recursive calls inside.
        Expr::Fuse { left, right } => {
            walk_expr(left, f);
            walk_expr(right, f);
        }
        Expr::Reclaim { policy, body } => {
            walk_expr(policy, f);
            walk_expr(body, f);
        }
        Expr::Path(_) | Expr::Lit(_) => {}
    }
}

/// A mutual group (size ≥ 1) descends iff some assignment of one designated argument position to
/// each member makes *every* inter-member call structural (§4.5). Searches the bounded product of
/// member arities. The size-1 case is exactly self-descent: the one member ranges over its
/// positions, and the only group member it can call is itself.
fn group_descends(scc: &[String], fns: &BTreeMap<String, FnDecl>) -> bool {
    let members: Vec<&FnDecl> = scc.iter().map(|n| &fns[n]).collect();
    let arities: Vec<usize> = members.iter().map(|fd| fd.sig.value_params.len()).collect();
    // A nullary member has no parameter to descend on, so this structural criterion cannot witness
    // the group total — honestly `Partial`.
    if arities.contains(&0) {
        return false;
    }
    let combos: usize = arities.iter().product();
    if combos > MAX_ASSIGNMENTS {
        return false;
    }
    // Each candidate is a mixed-radix index over the member arities: digit k chooses the designated
    // position of member k.
    (0..combos).any(|mut rem| {
        let mut pos = BTreeMap::new();
        for (fd, &arity) in members.iter().zip(&arities) {
            pos.insert(fd.sig.name.as_str(), rem % arity);
            rem /= arity;
        }
        assignment_descends(&members, &pos)
    })
}

/// Check one position assignment: every member's body, walked with that member's designated
/// parameter as the descent measure, makes every call to a group member pass a strictly-smaller
/// argument in the **callee's** designated position.
fn assignment_descends(members: &[&FnDecl], pos: &BTreeMap<&str, usize>) -> bool {
    members.iter().all(|fd| {
        let param = fd.sig.value_params[pos[fd.sig.name.as_str()]].name.as_str();
        let mut ok = true;
        descend_walk(&fd.body, pos, param, &mut BTreeSet::new(), &mut ok);
        ok
    })
}

/// Walk tracking the set of variables smaller-than the designated parameter; check every call to a
/// group member. `smaller` grows at `Match` alternatives whose scrutinee is the parameter or an
/// already-smaller variable. `pos` maps each group member to the argument position that must
/// receive a smaller variable on a call to it.
fn descend_walk(
    e: &Expr,
    pos: &BTreeMap<&str, usize>,
    param: &str,
    smaller: &mut BTreeSet<String>,
    ok: &mut bool,
) {
    match e {
        Expr::App { head, args } => {
            if let Expr::Path(p) = head.as_ref() {
                if p.0.len() == 1 {
                    if let Some(&tpos) = pos.get(p.0[0].as_str()) {
                        // A call to a group member: its designated argument must be a smaller var.
                        let good = args.get(tpos).is_some_and(|a| match a {
                            Expr::Path(v) => v.0.len() == 1 && smaller.contains(&v.0[0]),
                            _ => false,
                        });
                        if !good {
                            *ok = false;
                        }
                    }
                }
            }
            descend_walk(head, pos, param, smaller, ok);
            for a in args {
                descend_walk(a, pos, param, smaller, ok);
            }
        }
        Expr::Match { scrutinee, arms } => {
            descend_walk(scrutinee, pos, param, smaller, ok);
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
                        // Every binder under a constructor of a smaller-or-equal scrutinee is itself
                        // strictly smaller — including binders nested under further constructors
                        // (e.g. `m` in `S(S(m))`), so structural descent works through nested
                        // patterns, not just one level deep.
                        let mut nested = Vec::new();
                        for s in subs {
                            pattern_binders(s, &mut nested);
                        }
                        for b in nested {
                            if smaller.insert(b.clone()) {
                                added.push(b);
                            }
                        }
                    }
                }
                descend_walk(body, pos, param, smaller, ok);
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
            descend_walk(bound, pos, param, smaller, ok);
            // A rebinding shadows; conservatively drop smallness for the shadowed name.
            let was = smaller.remove(name);
            descend_walk(body, pos, param, smaller, ok);
            if was {
                smaller.insert(name.clone());
            }
        }
        Expr::If { cond, conseq, alt } => {
            descend_walk(cond, pos, param, smaller, ok);
            descend_walk(conseq, pos, param, smaller, ok);
            descend_walk(alt, pos, param, smaller, ok);
        }
        Expr::For {
            x,
            xs,
            acc,
            init,
            body,
        } => {
            descend_walk(xs, pos, param, smaller, ok);
            descend_walk(init, pos, param, smaller, ok);
            // The binders shadow; conservatively drop smallness for the shadowed names.
            let had_x = smaller.remove(x);
            let had_acc = smaller.remove(acc);
            descend_walk(body, pos, param, smaller, ok);
            if had_x {
                smaller.insert(x.clone());
            }
            if had_acc {
                smaller.insert(acc.clone());
            }
        }
        Expr::Swap { value, .. } => descend_walk(value, pos, param, smaller, ok),
        Expr::WithParadigm { body, .. } => descend_walk(body, pos, param, smaller, ok),
        // A `wild` body is opaque trusted FFI (M-661) — a leaf here too: a call inside a `wild` block
        // is not analyzable Mycelium recursion (and execution is staged), so it is not subject to the
        // structural-descent check, mirroring `walk_expr` (the opacity invariant is uniform — VR-5).
        Expr::Wild(_) => {}
        Expr::Spore(b) => descend_walk(b, pos, param, smaller, ok),
        // M-664: `consume <expr>` introduces no binders; walk the operand transparently so a
        // recursive call inside it is still subject to the structural-descent check.
        Expr::Consume(b) => descend_walk(b, pos, param, smaller, ok),
        // A `lambda` introduces its own parameter binders and is a deferred form (M-704); walk its
        // body transparently (it adds no recursive-descent of the enclosing fn's parameter).
        Expr::Lambda { body, .. } => descend_walk(body, pos, param, smaller, ok),
        // A `colony`'s hyphae introduce no binders; walk each body transparently so a recursive call
        // inside a hypha is still subject to the structural-descent check (A4-01).
        Expr::Colony(hyphae) => {
            for h in hyphae {
                descend_walk(&h.body, pos, param, smaller, ok);
            }
        }
        Expr::Ascribe(b, _) => descend_walk(b, pos, param, smaller, ok),
        // DN-58 §A/§B (M-667): `fuse(a, b)` and `reclaim(policy) { body }` — walk sub-expressions
        // transparently so recursive calls inside fuse/reclaim are subject to structural-descent
        // analysis (A4-01; neither `fuse` nor `reclaim` introduces binders, so no shadowing).
        Expr::Fuse { left, right } => {
            descend_walk(left, pos, param, smaller, ok);
            descend_walk(right, pos, param, smaller, ok);
        }
        Expr::Reclaim { policy, body } => {
            descend_walk(policy, pos, param, smaller, ok);
            descend_walk(body, pos, param, smaller, ok);
        }
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
