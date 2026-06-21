//! **Maranget usefulness** for L1 `match` — the checked exhaustiveness/redundancy core that makes
//! W7 hold for **nested** patterns (RFC-0007 §4.4/§4.7; Maranget 2007, *Warnings for pattern
//! matching*). It is the analysis half of the Maranget pipeline; the *decision-tree compilation* to
//! the flat kernel `Match` (Maranget 2008) is the elaborator's job and lands with full L1-in-Core-IR.
//!
//! The algorithm is the standard `U(P, q)`: given a pattern **matrix** `P` (one row per prior arm,
//! one column per scrutinee position) and a pattern **vector** `q`, decide whether some value matches
//! `q` but no row of `P` — and, when so, return a **witness** value (as a pattern) demonstrating it.
//! Two derived checks drive the typechecker:
//! - **Exhaustiveness:** the match covers everything iff `U(P, [_])` is *not* useful — a witness is a
//!   concrete missing pattern (e.g. `Cons(_, Nil)`), reported verbatim (coverage is *checked*, never
//!   assumed — W7).
//! - **Redundancy:** arm `i` is reachable iff `U(P₀..ᵢ, rowᵢ)` *is* useful; an unreachable arm is a
//!   redundancy error (subsumes the M-320 duplicate-literal check).
//!
//! The column **type** decides the constructor signature: a data type's signature is its finite
//! constructor set (so a column is *complete* once every constructor appears), while `Binary{n}` /
//! `Ternary{m}` have a value domain that is **never** enumerated — their signature is open, so a
//! literal column is complete only via a `_`/binder default (matching the M-320 rule). Recursion in
//! the data registry is handled lazily: a constructor's field column types are looked up only when
//! that constructor is actually expanded, so a recursive type (`Nat = Z | S(Nat)`) terminates.

use std::collections::{BTreeMap, BTreeSet};

use crate::checkty::{lookup_data_info, DataInfo, GenericShell, Ty};

/// A normalized pattern for the usefulness matrix. The typechecker lowers `ast::Pattern` to this:
/// binders and `_` both become [`Pat::Wild`] (they do not refine coverage), a nullary constructor or
/// constructor application becomes [`Pat::Ctor`], and a `Binary`/`Ternary` literal becomes
/// [`Pat::Lit`] keyed by its canonical form (arity 0).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Pat {
    /// `_` or a binder — matches anything.
    Wild,
    /// A constructor pattern `Name(sub, …)` (nullary ⇒ empty subs).
    Ctor(String, Vec<Pat>),
    /// A `Binary`/`Ternary` literal, keyed canonically (see `checkty::literal_key`).
    Lit(String),
}

/// The finite constructor signature of `ty`, or `None` if its value domain is open (`Binary`/
/// `Ternary` — never a complete signature, so a literal column always needs a default).
/// Handles abstract mangled types (e.g. `"List<A>"`) by falling back to `generics` (M-657).
fn signature<'a>(
    ty: &Ty,
    types: &'a BTreeMap<String, DataInfo>,
    generics: &'a BTreeMap<String, GenericShell>,
) -> Option<std::borrow::Cow<'a, DataInfo>> {
    match ty {
        Ty::Data(n) => Some(lookup_data_info(types, generics, n)),
        _ => None,
    }
}

/// The field types of constructor `c` in data type `ty` (empty if not found — the caller has already
/// type-checked the pattern, so a miss cannot happen on a well-typed matrix).
fn ctor_fields(
    ty: &Ty,
    c: &str,
    types: &BTreeMap<String, DataInfo>,
    generics: &BTreeMap<String, GenericShell>,
) -> Vec<Ty> {
    signature(ty, types, generics)
        .and_then(|d| {
            d.ctors
                .iter()
                .find(|ci| ci.name == c)
                .map(|ci| ci.fields.clone())
        })
        .unwrap_or_default()
}

/// A **matrix row** that can be specialized (Maranget `S`/default): it exposes its pattern columns
/// and can rebuild itself with a new column vector, carrying any *non-pattern* payload through
/// unchanged. Implemented by the bare `Vec<Pat>` row the usefulness analysis uses and by the
/// arm-tagged `Row` the decision-tree compiler uses (`crate::decision`), so the specialization is
/// written **once** over both (M-641). `with_columns` is the only place a row's payload is
/// preserved, keeping every implementor's identity (e.g. the surface arm index) intact.
pub(crate) trait SpecializeRow {
    /// This row's pattern columns (always non-empty when specialized).
    fn columns(&self) -> &[Pat];
    /// Rebuild a row of the same kind with `columns` as its new column vector, preserving payload.
    fn with_columns(&self, columns: Vec<Pat>) -> Self;
}

impl SpecializeRow for Vec<Pat> {
    fn columns(&self) -> &[Pat] {
        self
    }
    fn with_columns(&self, columns: Vec<Pat>) -> Self {
        columns
    }
}

/// Specialize the matrix on a constructor head `c` of arity `a`: keep rows whose first pattern is `c`
/// (expanding its sub-patterns into the new leading columns) or a wildcard (expanding to `a`
/// wildcards), dropping rows headed by a different constructor. Generic over the row type so the
/// usefulness matrix (`Vec<Pat>`) and the decision-tree matrix (`Row`) share one implementation.
pub(crate) fn specialize_ctor<R: SpecializeRow>(matrix: &[R], c: &str, a: usize) -> Vec<R> {
    let mut out = Vec::new();
    for row in matrix {
        let (first, rest) = row.columns().split_first().expect("non-empty row");
        match first {
            Pat::Ctor(n, subs) if n == c => {
                let mut r = subs.clone();
                r.extend_from_slice(rest);
                out.push(row.with_columns(r));
            }
            Pat::Wild => {
                let mut r = vec![Pat::Wild; a];
                r.extend_from_slice(rest);
                out.push(row.with_columns(r));
            }
            _ => {} // different constructor / a literal head: drop
        }
    }
    out
}

/// Specialize the matrix on a literal head `k` (arity 0): keep rows headed by that exact literal or a
/// wildcard, dropping the leading column. Generic over the row type (see [`specialize_ctor`]).
pub(crate) fn specialize_lit<R: SpecializeRow>(matrix: &[R], k: &str) -> Vec<R> {
    let mut out = Vec::new();
    for row in matrix {
        let (first, rest) = row.columns().split_first().expect("non-empty row");
        match first {
            Pat::Lit(j) if j == k => out.push(row.with_columns(rest.to_vec())),
            Pat::Wild => out.push(row.with_columns(rest.to_vec())),
            _ => {}
        }
    }
    out
}

/// The default matrix `D(P)`: rows headed by a wildcard, with the leading column dropped.
fn default_matrix(matrix: &[Vec<Pat>]) -> Vec<Vec<Pat>> {
    matrix
        .iter()
        .filter_map(|row| {
            let (first, rest) = row.split_first().expect("non-empty row");
            matches!(first, Pat::Wild).then(|| rest.to_vec())
        })
        .collect()
}

/// The set of constructor names appearing in the matrix's first column.
fn head_ctors(matrix: &[Vec<Pat>]) -> BTreeSet<String> {
    matrix
        .iter()
        .filter_map(|row| match &row[0] {
            Pat::Ctor(n, _) => Some(n.clone()),
            _ => None,
        })
        .collect()
}

/// `U(P, q)` — is `q` useful w.r.t. matrix `P` (some value matches `q` but no row of `P`)? Returns a
/// witness value (as a pattern vector of the same width) when useful, else `None`. `col_types` gives
/// the type of each column (parallel to `q`); it drives the complete-signature test and the lazy
/// field-type expansion. `generics` enables coverage of abstract-mangled types in generic fn bodies
/// (M-657): a `Ty::Data("List<A>")` scrutinee resolves via the generic shell registry.
pub(crate) fn useful(
    types: &BTreeMap<String, DataInfo>,
    generics: &BTreeMap<String, GenericShell>,
    matrix: &[Vec<Pat>],
    q: &[Pat],
    col_types: &[Ty],
) -> Option<Vec<Pat>> {
    // Base case (no columns): useful iff no row remains (every prior row already "matched"); the
    // witness is the empty value vector.
    if q.is_empty() {
        return matrix.is_empty().then(Vec::new);
    }
    let head_ty = &col_types[0];
    match &q[0] {
        Pat::Ctor(c, subs) => {
            let a = subs.len();
            let m2 = specialize_ctor(matrix, c, a);
            let mut q2 = subs.clone();
            q2.extend_from_slice(&q[1..]);
            let mut ct2 = ctor_fields(head_ty, c, types, generics);
            ct2.extend_from_slice(&col_types[1..]);
            useful(types, generics, &m2, &q2, &ct2).map(|w| rebuild_ctor(c, a, w))
        }
        Pat::Lit(k) => {
            let m2 = specialize_lit(matrix, k);
            let q2 = q[1..].to_vec();
            let ct2 = col_types[1..].to_vec();
            useful(types, generics, &m2, &q2, &ct2).map(|w| prepend(Pat::Lit(k.clone()), w))
        }
        Pat::Wild => match signature(head_ty, types, generics) {
            // Finite (data) signature: complete once every constructor appears in column 0.
            Some(d) => {
                let d = d.into_owned();
                let present = head_ctors(matrix);
                if d.ctors.iter().all(|ci| present.contains(&ci.name)) {
                    // Complete: useful iff useful under *some* constructor specialization.
                    for ci in &d.ctors {
                        let a = ci.fields.len();
                        let m2 = specialize_ctor(matrix, &ci.name, a);
                        let mut q2 = vec![Pat::Wild; a];
                        q2.extend_from_slice(&q[1..]);
                        let mut ct2 = ci.fields.clone();
                        ct2.extend_from_slice(&col_types[1..]);
                        if let Some(w) = useful(types, generics, &m2, &q2, &ct2) {
                            return Some(rebuild_ctor(&ci.name, a, w));
                        }
                    }
                    None
                } else {
                    // Incomplete: recurse on the default; the witness head is a *missing* constructor.
                    let m2 = default_matrix(matrix);
                    useful(types, generics, &m2, &q[1..], &col_types[1..]).map(|w| {
                        let missing = d.ctors.iter().find(|ci| !present.contains(&ci.name));
                        let head = missing.map_or(Pat::Wild, |ci| {
                            Pat::Ctor(ci.name.clone(), vec![Pat::Wild; ci.fields.len()])
                        });
                        prepend(head, w)
                    })
                }
            }
            // Open (Binary/Ternary) domain: never complete — recurse on the default, witness `_`.
            None => {
                let m2 = default_matrix(matrix);
                useful(types, generics, &m2, &q[1..], &col_types[1..])
                    .map(|w| prepend(Pat::Wild, w))
            }
        },
    }
}

/// Re-fold a witness whose first `a` elements are constructor `c`'s sub-witnesses.
fn rebuild_ctor(c: &str, a: usize, mut w: Vec<Pat>) -> Vec<Pat> {
    let rest = w.split_off(a);
    let head = Pat::Ctor(c.to_owned(), w);
    prepend(head, rest)
}

fn prepend(head: Pat, rest: Vec<Pat>) -> Vec<Pat> {
    let mut v = Vec::with_capacity(rest.len() + 1);
    v.push(head);
    v.extend(rest);
    v
}

/// Render a witness pattern for a diagnostic (`Cons(_, Nil)`, `0b1010`, `<+0->`, `_`). Literal keys
/// carry a `b:`/`t:` tag (from `checkty::literal_key`) that is rewritten back to surface syntax.
pub(crate) fn render(p: &Pat) -> String {
    match p {
        Pat::Wild => "_".to_owned(),
        Pat::Lit(k) => match k.split_once(':') {
            Some(("b", bits)) => format!("0b{bits}"),
            Some(("t", trits)) => format!("<{trits}>"),
            _ => k.clone(),
        },
        Pat::Ctor(n, subs) if subs.is_empty() => n.clone(),
        Pat::Ctor(n, subs) => {
            let inner: Vec<String> = subs.iter().map(render).collect();
            format!("{n}({})", inner.join(", "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checkty::{CtorInfo, DataInfo, GenericShell};

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

    fn no_generics() -> BTreeMap<String, GenericShell> {
        BTreeMap::new()
    }

    fn ctor(n: &str, subs: Vec<Pat>) -> Pat {
        Pat::Ctor(n.to_owned(), subs)
    }

    #[test]
    fn complete_flat_match_is_exhaustive() {
        let t = nat_registry();
        // rows: Z, S(_) — a wildcard `_` is then not useful ⇒ exhaustive.
        let rows = vec![vec![ctor("Z", vec![])], vec![ctor("S", vec![Pat::Wild])]];
        assert!(useful(
            &t,
            &no_generics(),
            &rows,
            &[Pat::Wild],
            &[Ty::Data("Nat".into())]
        )
        .is_none());
    }

    #[test]
    fn missing_case_yields_a_witness() {
        let t = nat_registry();
        // rows: only Z — `_` is useful, witness is the missing `S(_)`.
        let rows = vec![vec![ctor("Z", vec![])]];
        let w = useful(
            &t,
            &no_generics(),
            &rows,
            &[Pat::Wild],
            &[Ty::Data("Nat".into())],
        )
        .expect("non-exhaustive");
        assert_eq!(render(&w[0]), "S(_)");
    }

    #[test]
    fn nested_missing_case_is_found_with_a_deep_witness() {
        let t = nat_registry();
        // rows: Z, S(Z) — missing S(S(_)). The deep witness drives a precise diagnostic.
        let rows = vec![
            vec![ctor("Z", vec![])],
            vec![ctor("S", vec![ctor("Z", vec![])])],
        ];
        let w = useful(
            &t,
            &no_generics(),
            &rows,
            &[Pat::Wild],
            &[Ty::Data("Nat".into())],
        )
        .expect("non-exhaustive");
        assert_eq!(render(&w[0]), "S(S(_))");
    }

    #[test]
    fn nested_cover_is_exhaustive() {
        let t = nat_registry();
        // Z | S(Z) | S(S(_)) covers Nat exhaustively (nested).
        let rows = vec![
            vec![ctor("Z", vec![])],
            vec![ctor("S", vec![ctor("Z", vec![])])],
            vec![ctor("S", vec![ctor("S", vec![Pat::Wild])])],
        ];
        assert!(useful(
            &t,
            &no_generics(),
            &rows,
            &[Pat::Wild],
            &[Ty::Data("Nat".into())]
        )
        .is_none());
    }

    #[test]
    fn redundant_arm_is_not_useful() {
        let t = nat_registry();
        // After Z and S(_), the arm S(Z) is redundant (already covered) ⇒ not useful.
        let prior = vec![vec![ctor("Z", vec![])], vec![ctor("S", vec![Pat::Wild])]];
        let row = vec![ctor("S", vec![ctor("Z", vec![])])];
        assert!(useful(&t, &no_generics(), &prior, &row, &[Ty::Data("Nat".into())]).is_none());
    }

    #[test]
    fn literal_column_needs_a_default() {
        let t = nat_registry();
        // A Binary{1} column with literal rows 0b0, 0b1 but no default is still non-exhaustive: the
        // value domain is never enumerated (M-320), so `_` stays useful.
        let rows = vec![vec![Pat::Lit("b:0".into())], vec![Pat::Lit("b:1".into())]];
        assert!(useful(&t, &no_generics(), &rows, &[Pat::Wild], &[Ty::Binary(1)]).is_some());
        // With a default, `_` is no longer useful.
        let with_default = vec![vec![Pat::Lit("b:0".into())], vec![Pat::Wild]];
        assert!(useful(
            &t,
            &no_generics(),
            &with_default,
            &[Pat::Wild],
            &[Ty::Binary(1)]
        )
        .is_none());
    }

    // --- M-641: the shared `SpecializeRow` specialization over two row types ---------------------

    /// A stand-in payload-carrying row (like `decision::Row`) to prove the generic specializer
    /// preserves non-pattern payload and produces the same columns as the bare `Vec<Pat>` form.
    #[derive(Clone, PartialEq, Debug)]
    struct TaggedRow {
        pats: Vec<Pat>,
        tag: usize,
    }
    impl SpecializeRow for TaggedRow {
        fn columns(&self) -> &[Pat] {
            &self.pats
        }
        fn with_columns(&self, columns: Vec<Pat>) -> Self {
            TaggedRow {
                pats: columns,
                tag: self.tag,
            }
        }
    }

    fn ctorp(n: &str, subs: Vec<Pat>) -> Pat {
        Pat::Ctor(n.to_owned(), subs)
    }

    #[test]
    fn specialize_ctor_is_identical_across_row_types_and_keeps_payload() {
        // matrix: [ S(Z) | tag 7 ], [ _ | tag 9 ], [ Z | tag 5 ] — specialize on S/arity 1.
        let bare: Vec<Vec<Pat>> = vec![
            vec![ctorp("S", vec![ctorp("Z", vec![])])],
            vec![Pat::Wild],
            vec![ctorp("Z", vec![])],
        ];
        let tagged: Vec<TaggedRow> = vec![
            TaggedRow {
                pats: vec![ctorp("S", vec![ctorp("Z", vec![])])],
                tag: 7,
            },
            TaggedRow {
                pats: vec![Pat::Wild],
                tag: 9,
            },
            TaggedRow {
                pats: vec![ctorp("Z", vec![])],
                tag: 5,
            },
        ];
        let s_bare = specialize_ctor(&bare, "S", 1);
        let s_tagged = specialize_ctor(&tagged, "S", 1);
        // Same surviving columns on both: S(Z)→[Z], _→[_]; the Z-headed row is dropped.
        assert_eq!(s_bare, vec![vec![ctorp("Z", vec![])], vec![Pat::Wild]]);
        let cols: Vec<Vec<Pat>> = s_tagged.iter().map(|r| r.pats.clone()).collect();
        assert_eq!(cols, s_bare);
        // Payload preserved in row order (S row kept tag 7, wildcard row kept tag 9).
        assert_eq!(
            s_tagged.iter().map(|r| r.tag).collect::<Vec<_>>(),
            vec![7, 9]
        );
    }

    #[test]
    fn specialize_lit_is_identical_across_row_types_and_keeps_payload() {
        let bare: Vec<Vec<Pat>> = vec![
            vec![Pat::Lit("b:0".into())],
            vec![Pat::Wild],
            vec![Pat::Lit("b:1".into())],
        ];
        let tagged: Vec<TaggedRow> = vec![
            TaggedRow {
                pats: vec![Pat::Lit("b:0".into())],
                tag: 1,
            },
            TaggedRow {
                pats: vec![Pat::Wild],
                tag: 2,
            },
            TaggedRow {
                pats: vec![Pat::Lit("b:1".into())],
                tag: 3,
            },
        ];
        let l_bare = specialize_lit(&bare, "b:0");
        let l_tagged = specialize_lit(&tagged, "b:0");
        // b:0 and the wildcard survive (each drops its leading column → empty), b:1 is dropped.
        assert_eq!(l_bare, vec![Vec::<Pat>::new(), Vec::<Pat>::new()]);
        let cols: Vec<Vec<Pat>> = l_tagged.iter().map(|r| r.pats.clone()).collect();
        assert_eq!(cols, l_bare);
        assert_eq!(
            l_tagged.iter().map(|r| r.tag).collect::<Vec<_>>(),
            vec![1, 2]
        );
    }
}
