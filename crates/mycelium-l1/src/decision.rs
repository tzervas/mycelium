//! **Maranget decision-tree compilation** for L1 `match` (M-320; RFC-0007 §3/§4.4; Maranget 2008,
//! *Compiling pattern matching to good decision trees*) — the **codegen half** of the Maranget
//! pipeline whose analysis half is [`crate::usefulness`].
//!
//! Where usefulness answers "is this match exhaustive / are any arms redundant?", this pass answers
//! "in what order do we test the scrutinee to reach the right arm?" — it lowers a (checked,
//! exhaustive) nested-pattern match into a [`Tree`] of `switch`/`leaf` nodes over **occurrences**
//! (paths into the scrutinee). This is exactly what RFC-0007 §3 means by patterns being "compiled
//! away by the elaborator": the surface keeps nested patterns; the tree is flat tests.
//!
//! **Scope / honesty (VR-5).** This builds and *verifies* the decision tree (the tests evaluate it
//! against the reference matcher), and — since RFC-0011 r3 enacted the flat L0 `Match` node
//! (RFC-0001 r3) — the elaborator **emits** it: [`crate::elab`]'s `lower_tree` walks each `Switch`
//! into a nested L0 `Match` and each `Leaf` into the surface arm body, the wiring this module's
//! `Tree` was designed for (RFC-0007 §4.6 / RFC-0011 §4.4). The tree stays the *untrusted,
//! inspectable* compilation artifact **above** the kernel: the trusted node is the flat `Match`, and
//! the three-way differential (`tests/differential.rs`) checks the emitted lowering — L1-eval ≡
//! L0-interp ≡ AOT — so a wrong column choice or specialization is caught, never rubber-stamped. The
//! tree's own [`eval_tree`] remains a *test-only* reference (it verifies the compiler; it does not
//! run programs). No accuracy guarantee is touched by the compilation — it is a meaning-preserving
//! rewrite, witnessed by the differential.
//!
//! The compiler operates on the same normalized [`Pat`] matrix usefulness uses; a *value* is just a
//! [`Pat`] with no [`Pat::Wild`] (a fully concrete constructor/literal tree), which is what the
//! verification tests feed both the tree and the reference matcher.

use std::collections::BTreeMap;

use crate::checkty::{lookup_data_info, DataInfo, GenericShell, Ty};
use crate::usefulness::{specialize_ctor, specialize_lit, Pat};

/// An **occurrence**: the path of field indices from the scrutinee root to a sub-value (`[]` is the
/// whole scrutinee, `[1]` is its second constructor field, `[1, 0]` the first field of that, …).
pub(crate) type Occurrence = Vec<usize>;

/// The head a [`Tree::Switch`] case tests for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Head {
    /// A constructor by name + arity.
    Ctor(String, usize),
    /// A `Binary`/`Ternary` literal, keyed as in [`crate::usefulness`] (`b:…` / `t:…`).
    Lit(String),
}

/// A compiled match **decision tree** (Maranget 2008). Leaves carry the **surface arm index** to run;
/// `Fail` is only reachable for a non-exhaustive match (the checker rejects those before compilation,
/// so a verified-exhaustive match never produces a reachable `Fail`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Tree {
    /// Run surface arm `usize` (the first matrix row that matched).
    Leaf(usize),
    /// No arm matches.
    Fail,
    /// Test the value at `occurrence` against each `(head, subtree)` case in turn; if none matches,
    /// fall through to `default` (present exactly when the column's signature is incomplete or its
    /// domain is open — `Binary`/`Ternary`).
    Switch {
        /// Which sub-value of the scrutinee this node tests.
        occurrence: Occurrence,
        /// The constructor/literal cases, in signature order (data) or first-seen order (literals).
        cases: Vec<(Head, Tree)>,
        /// The catch-all branch, when the cases do not cover the column's signature.
        default: Option<Box<Tree>>,
    },
}

/// One row of the working matrix: the per-column patterns and the surface arm it came from.
#[derive(Clone)]
struct Row {
    pats: Vec<Pat>,
    arm: usize,
}

/// A decision-tree row specializes exactly like a bare pattern vector, but must **carry its surface
/// arm index** through unchanged (that index is what a `Leaf` ultimately runs). Implementing
/// [`SpecializeRow`](crate::usefulness::SpecializeRow) lets the Maranget `S` specialization be the
/// single shared one in `crate::usefulness` (M-641) rather than a row-for-row duplicate here.
impl crate::usefulness::SpecializeRow for Row {
    fn columns(&self) -> &[Pat] {
        &self.pats
    }
    fn with_columns(&self, columns: Vec<Pat>) -> Self {
        Row {
            pats: columns,
            arm: self.arm,
        }
    }
}

/// Compile a checked match into a decision tree. `matrix` is the per-arm normalized pattern rows (one
/// `Pat` per column), `arms` the parallel surface arm indices, `occ` the occurrence of each column
/// (initially `[[]]` for the single scrutinee), and `tys` each column's type (drives the
/// complete-signature test + the field-type expansion). Assumes the match has already passed
/// exhaustiveness/redundancy (so the first all-wildcard row is a real catch-all).
pub(crate) fn compile(
    types: &BTreeMap<String, DataInfo>,
    generics: &BTreeMap<String, GenericShell>,
    matrix: &[Vec<Pat>],
    arms: &[usize],
    occ: &[Occurrence],
    tys: &[Ty],
) -> Tree {
    let rows: Vec<Row> = matrix
        .iter()
        .zip(arms)
        .map(|(pats, &arm)| Row {
            pats: pats.clone(),
            arm,
        })
        .collect();
    compile_rows(types, generics, &rows, occ, tys)
}

fn compile_rows(
    types: &BTreeMap<String, DataInfo>,
    generics: &BTreeMap<String, GenericShell>,
    rows: &[Row],
    occ: &[Occurrence],
    tys: &[Ty],
) -> Tree {
    // No row can match → failure (unreachable for an exhaustive match).
    let Some(first) = rows.first() else {
        return Tree::Fail;
    };
    // The first row is all wildcards (or there are no columns) → it matches everything here: run it.
    if first.pats.iter().all(|p| matches!(p, Pat::Wild)) {
        return Tree::Leaf(first.arm);
    }
    // Pick the first column with a non-wildcard head in some row (Maranget's left-to-right heuristic),
    // and rotate it to the front so the specialization helpers can work on column 0.
    let col = (0..occ.len())
        .find(|&i| rows.iter().any(|r| !matches!(r.pats[i], Pat::Wild)))
        .expect("first row is non-wildcard, so some column has a constructor/literal head");
    let (rows, occ, tys) = rotate_to_front(rows, occ, tys, col);
    let occ0 = occ[0].clone();
    let ty0 = tys[0].clone();

    // Gather the heads present in column 0 (constructors with arity, or literal keys).
    let mut ctor_heads: Vec<(String, usize)> = Vec::new();
    let mut lit_heads: Vec<String> = Vec::new();
    for r in &rows {
        match &r.pats[0] {
            Pat::Ctor(n, subs) => {
                if !ctor_heads.iter().any(|(m, _)| m == n) {
                    ctor_heads.push((n.clone(), subs.len()));
                }
            }
            Pat::Lit(k) => {
                if !lit_heads.iter().any(|j| j == k) {
                    lit_heads.push(k.clone());
                }
            }
            Pat::Wild => {}
        }
    }

    let mut cases: Vec<(Head, Tree)> = Vec::new();
    // Whether the cases cover the column's whole signature (so no default is needed).
    // Handles abstract-mangled types (e.g. "List<A>") via lookup_data_info (M-657).
    let complete = match &ty0 {
        Ty::Data(n) => {
            // Use lookup_data_info to handle both concrete and abstract-mangled types.
            if types.contains_key(n) || n.contains('<') {
                let d = lookup_data_info(types, generics, n);
                d.ctors
                    .iter()
                    .all(|ci| ctor_heads.iter().any(|(m, _)| *m == ci.name))
            } else {
                false
            }
        }
        // Binary/Ternary value domains are never enumerated — always need a default.
        _ => false,
    };

    if let Ty::Data(dn) = &ty0 {
        // Use lookup_data_info to handle both concrete and abstract-mangled types.
        if types.contains_key(dn.as_str()) || dn.contains('<') {
            let d = lookup_data_info(types, generics, dn).into_owned();
            for ci in &d.ctors {
                if ctor_heads.iter().any(|(m, _)| *m == ci.name) {
                    let a = ci.fields.len();
                    let sub = compile_rows(
                        types,
                        generics,
                        &specialize_ctor(&rows, &ci.name, a),
                        &child_occ(&occ, &occ0, a),
                        &child_tys(&tys, &ci.fields),
                    );
                    cases.push((Head::Ctor(ci.name.clone(), a), sub));
                }
            }
        }
    }
    for k in &lit_heads {
        let sub = compile_rows(
            types,
            generics,
            &specialize_lit(&rows, k),
            &occ[1..],
            &tys[1..],
        );
        cases.push((Head::Lit(k.clone()), sub));
    }

    let default = if complete {
        None
    } else {
        Some(Box::new(compile_rows(
            types,
            generics,
            &default_rows(&rows),
            &occ[1..],
            &tys[1..],
        )))
    };

    Tree::Switch {
        occurrence: occ0,
        cases,
        default,
    }
}

/// Swap column `i` to the front of the rows + the parallel occurrence/type vectors (an occurrence is
/// an intrinsic path, so reordering columns does not change leaf arms or any occurrence).
fn rotate_to_front(
    rows: &[Row],
    occ: &[Occurrence],
    tys: &[Ty],
    i: usize,
) -> (Vec<Row>, Vec<Occurrence>, Vec<Ty>) {
    let mut occ = occ.to_vec();
    let mut tys = tys.to_vec();
    occ.swap(0, i);
    tys.swap(0, i);
    let rows = rows
        .iter()
        .map(|r| {
            let mut pats = r.pats.clone();
            pats.swap(0, i);
            Row { pats, arm: r.arm }
        })
        .collect();
    (rows, occ, tys)
}

/// The occurrences of the columns after specializing column 0 on a constructor of arity `a`: the `a`
/// child occurrences `occ0.j` followed by the remaining columns.
fn child_occ(occ: &[Occurrence], occ0: &Occurrence, a: usize) -> Vec<Occurrence> {
    let mut out: Vec<Occurrence> = (0..a)
        .map(|j| {
            let mut o = occ0.clone();
            o.push(j);
            o
        })
        .collect();
    out.extend_from_slice(&occ[1..]);
    out
}

/// The column types after specializing column 0 on a constructor: its field types then the rest.
fn child_tys(tys: &[Ty], fields: &[Ty]) -> Vec<Ty> {
    let mut out = fields.to_vec();
    out.extend_from_slice(&tys[1..]);
    out
}

// Maranget's `S` specialization (constructor and literal heads) is shared with the usefulness
// analysis via `crate::usefulness::{specialize_ctor, specialize_lit}` over the `SpecializeRow`
// trait `Row` implements above (M-641) — the decision-tree-specific part is only carrying the arm
// index through, which the trait's `with_columns` does. `default_rows` (the `D(P)` matrix) stays
// local: it is a distinct operation, not part of that shared specialization.

/// The default rows `D(P)`: rows headed by a wildcard, leading column dropped.
fn default_rows(rows: &[Row]) -> Vec<Row> {
    rows.iter()
        .filter_map(|r| {
            let (first, rest) = r.pats.split_first().expect("non-empty row");
            matches!(first, Pat::Wild).then(|| Row {
                pats: rest.to_vec(),
                arm: r.arm,
            })
        })
        .collect()
}

/// Whether the tree contains a reachable [`Tree::Fail`]. Every branch a compiled tree emits is
/// reachable (each case head can occur; a `default` is present only when needed), so "contains a
/// `Fail`" is "has a reachable `Fail`". The checker uses this to confirm an **exhaustive** match
/// compiled to total coverage (defense in depth: usefulness and the Maranget compiler must agree).
pub(crate) fn has_reachable_fail(tree: &Tree) -> bool {
    match tree {
        Tree::Fail => true,
        Tree::Leaf(_) => false,
        Tree::Switch { cases, default, .. } => {
            cases.iter().any(|(_, t)| has_reachable_fail(t))
                || default.as_deref().is_some_and(has_reachable_fail)
        }
    }
}

/// Project the sub-value of `value` at `occurrence` (`value` is a concrete [`Pat`] — no `Wild`).
#[cfg(test)]
fn project<'a>(value: &'a Pat, occurrence: &[usize]) -> &'a Pat {
    let mut v = value;
    for &j in occurrence {
        match v {
            Pat::Ctor(_, subs) => v = &subs[j],
            _ => panic!("occurrence {occurrence:?} steps into a non-constructor value"),
        }
    }
    v
}

/// Evaluate a decision tree against a concrete value (a `Pat` with no `Wild`), returning the arm to
/// run — the executable semantics that lets a test check the tree against the reference matcher.
/// **Test-only by design:** production does not execute the `Tree` directly — the elaborator emits
/// it as nested L0 `Match` nodes (see the module note) which the L0 interpreter/AOT run. This
/// interpreter exists only to *verify* the compiler against the reference matcher, not to run programs.
#[cfg(test)]
pub(crate) fn eval_tree(tree: &Tree, value: &Pat) -> Option<usize> {
    match tree {
        Tree::Leaf(a) => Some(*a),
        Tree::Fail => None,
        Tree::Switch {
            occurrence,
            cases,
            default,
        } => {
            let sub = project(value, occurrence);
            let matched = cases.iter().find(|(h, _)| head_matches(h, sub));
            match matched {
                Some((_, subtree)) => eval_tree(subtree, value),
                None => default.as_deref().and_then(|d| eval_tree(d, value)),
            }
        }
    }
}

#[cfg(test)]
fn head_matches(head: &Head, value: &Pat) -> bool {
    match (head, value) {
        (Head::Ctor(n, a), Pat::Ctor(m, subs)) => n == m && *a == subs.len(),
        (Head::Lit(k), Pat::Lit(j)) => k == j,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checkty::{CtorInfo, DataInfo, GenericShell};

    fn no_generics() -> BTreeMap<String, GenericShell> {
        BTreeMap::new()
    }

    fn nat_registry() -> BTreeMap<String, DataInfo> {
        let mut m = BTreeMap::new();
        m.insert(
            "Nat".to_owned(),
            DataInfo {
                name: "Nat".to_owned(),
                ctors: vec![
                    CtorInfo {
                        name: "Z".to_owned(),
                        fields: vec![],
                    },
                    CtorInfo {
                        name: "S".to_owned(),
                        fields: vec![Ty::Data("Nat".to_owned())],
                    },
                ],
            },
        );
        m
    }

    fn ctor(n: &str, subs: Vec<Pat>) -> Pat {
        Pat::Ctor(n.to_owned(), subs)
    }

    /// The reference matcher: the first arm whose pattern matches `value` (a concrete `Pat`).
    fn reference(arms: &[Pat], value: &Pat) -> Option<usize> {
        arms.iter().position(|p| matches_value(p, value))
    }
    fn matches_value(pat: &Pat, value: &Pat) -> bool {
        match (pat, value) {
            (Pat::Wild, _) => true,
            (Pat::Lit(k), Pat::Lit(j)) => k == j,
            (Pat::Ctor(n, ps), Pat::Ctor(m, vs)) => {
                n == m
                    && ps.len() == vs.len()
                    && ps.iter().zip(vs).all(|(p, v)| matches_value(p, v))
            }
            _ => false,
        }
    }

    /// Build the Nat value `n` as a concrete `Pat` (Z, S(Z), S(S(Z)), …).
    fn nat(n: usize) -> Pat {
        let mut v = ctor("Z", vec![]);
        for _ in 0..n {
            v = ctor("S", vec![v]);
        }
        v
    }

    /// The tree agrees with the reference matcher on every Nat value up to a depth — the property
    /// that earns the compilation (a wrong column choice / specialization would diverge here).
    fn assert_agrees(arms: &[Pat], tree: &Tree, depth: usize) {
        for n in 0..=depth {
            let v = nat(n);
            assert_eq!(
                eval_tree(tree, &v),
                reference(arms, &v),
                "tree vs reference diverged on S^{n}(Z)"
            );
        }
    }

    #[test]
    fn flat_nat_match_compiles_and_agrees() {
        let t = nat_registry();
        // Z => 0 | S(_) => 1  (exhaustive, flat).
        let arms = vec![ctor("Z", vec![]), ctor("S", vec![Pat::Wild])];
        let matrix: Vec<Vec<Pat>> = arms.iter().cloned().map(|p| vec![p]).collect();
        let tree = compile(
            &t,
            &no_generics(),
            &matrix,
            &[0, 1],
            &[vec![]],
            &[Ty::Data("Nat".into())],
        );
        // Root switches on the whole scrutinee with both constructors covered → no default.
        match &tree {
            Tree::Switch {
                occurrence,
                cases,
                default,
            } => {
                assert_eq!(occurrence, &Vec::<usize>::new());
                assert_eq!(cases.len(), 2);
                assert!(
                    default.is_none(),
                    "complete data signature needs no default"
                );
            }
            other => panic!("expected a switch, got {other:?}"),
        }
        assert_agrees(&arms, &tree, 4);
    }

    #[test]
    fn nested_nat_match_compiles_and_agrees() {
        let t = nat_registry();
        // Z => 0 | S(Z) => 1 | S(S(_)) => 2  (exhaustive, nested) — the tree must reach arm 2 only
        // for depth ≥ 2, which exercises the S→S child occurrence [0].
        let arms = vec![
            ctor("Z", vec![]),
            ctor("S", vec![ctor("Z", vec![])]),
            ctor("S", vec![ctor("S", vec![Pat::Wild])]),
        ];
        let matrix: Vec<Vec<Pat>> = arms.iter().cloned().map(|p| vec![p]).collect();
        let tree = compile(
            &t,
            &no_generics(),
            &matrix,
            &[0, 1, 2],
            &[vec![]],
            &[Ty::Data("Nat".into())],
        );
        assert_agrees(&arms, &tree, 5);
        // Spot-check the arm selection directly.
        assert_eq!(eval_tree(&tree, &nat(0)), Some(0));
        assert_eq!(eval_tree(&tree, &nat(1)), Some(1));
        assert_eq!(eval_tree(&tree, &nat(2)), Some(2));
        assert_eq!(eval_tree(&tree, &nat(3)), Some(2));
    }

    #[test]
    fn first_matching_arm_wins_on_overlap() {
        let t = nat_registry();
        // S(_) => 0 | S(Z) => 1 | Z => 2 : arm 1 is shadowed by the broader arm 0 (redundant), and
        // arm 2 makes the set exhaustive. The tree must still pick arm 0 for S(Z), matching the
        // first-match semantics the reference encodes.
        let arms = vec![
            ctor("S", vec![Pat::Wild]),
            ctor("S", vec![ctor("Z", vec![])]),
            ctor("Z", vec![]),
        ];
        let matrix: Vec<Vec<Pat>> = arms.iter().cloned().map(|p| vec![p]).collect();
        let tree = compile(
            &t,
            &no_generics(),
            &matrix,
            &[0, 1, 2],
            &[vec![]],
            &[Ty::Data("Nat".into())],
        );
        assert_eq!(eval_tree(&tree, &nat(1)), Some(0)); // S(Z) → first arm S(_), never the shadowed arm 1
        assert_agrees(&arms, &tree, 4);
    }

    #[test]
    fn literal_match_with_default_compiles_and_switches_with_a_default() {
        let t = nat_registry();
        // 0b0 => 0 | _ => 1 over Binary{1}: a literal switch that always carries a default (the
        // value domain is open).
        let arms = [Pat::Lit("b:0".into()), Pat::Wild];
        let matrix: Vec<Vec<Pat>> = arms.iter().cloned().map(|p| vec![p]).collect();
        let tree = compile(
            &t,
            &no_generics(),
            &matrix,
            &[0, 1],
            &[vec![]],
            &[Ty::Binary(1)],
        );
        match &tree {
            Tree::Switch { cases, default, .. } => {
                assert_eq!(cases.len(), 1);
                assert!(
                    default.is_some(),
                    "open literal domain always needs a default"
                );
            }
            other => panic!("expected a switch, got {other:?}"),
        }
        assert_eq!(eval_tree(&tree, &Pat::Lit("b:0".into())), Some(0));
        assert_eq!(eval_tree(&tree, &Pat::Lit("b:1".into())), Some(1)); // falls to default
    }
}
