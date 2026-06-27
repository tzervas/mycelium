use crate::checkty::{CtorInfo, DataInfo, Ty, Width};
use crate::usefulness::*;

fn nat_registry() -> std::collections::BTreeMap<String, DataInfo> {
    let mut m = std::collections::BTreeMap::new();
    m.insert(
        "Nat".to_owned(),
        DataInfo {
            name: "Nat".to_owned(),
            params: vec![],
            ctors: vec![
                CtorInfo {
                    name: "Z".to_owned(),
                    fields: vec![],
                },
                CtorInfo {
                    name: "S".to_owned(),
                    fields: vec![Ty::Data("Nat".to_owned(), vec![])],
                },
            ],
        },
    );
    m
}

fn ctor(n: &str, subs: Vec<Pat>) -> Pat {
    Pat::Ctor(n.to_owned(), subs)
}

#[test]
fn complete_flat_match_is_exhaustive() {
    let t = nat_registry();
    // rows: Z, S(_) — a wildcard `_` is then not useful ⇒ exhaustive.
    let rows = vec![vec![ctor("Z", vec![])], vec![ctor("S", vec![Pat::Wild])]];
    assert!(useful(&t, &rows, &[Pat::Wild], &[Ty::Data("Nat".into(), vec![])]).is_none());
}

#[test]
fn missing_case_yields_a_witness() {
    let t = nat_registry();
    // rows: only Z — `_` is useful, witness is the missing `S(_)`.
    let rows = vec![vec![ctor("Z", vec![])]];
    let w =
        useful(&t, &rows, &[Pat::Wild], &[Ty::Data("Nat".into(), vec![])]).expect("non-exhaustive");
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
    let w =
        useful(&t, &rows, &[Pat::Wild], &[Ty::Data("Nat".into(), vec![])]).expect("non-exhaustive");
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
    assert!(useful(&t, &rows, &[Pat::Wild], &[Ty::Data("Nat".into(), vec![])]).is_none());
}

#[test]
fn redundant_arm_is_not_useful() {
    let t = nat_registry();
    // After Z and S(_), the arm S(Z) is redundant (already covered) ⇒ not useful.
    let prior = vec![vec![ctor("Z", vec![])], vec![ctor("S", vec![Pat::Wild])]];
    let row = vec![ctor("S", vec![ctor("Z", vec![])])];
    assert!(useful(&t, &prior, &row, &[Ty::Data("Nat".into(), vec![])]).is_none());
}

#[test]
fn literal_column_needs_a_default() {
    let t = nat_registry();
    // A Binary{1} column with literal rows 0b0, 0b1 but no default is still non-exhaustive: the
    // value domain is never enumerated (M-320), so `_` stays useful.
    let rows = vec![vec![Pat::Lit("b:0".into())], vec![Pat::Lit("b:1".into())]];
    assert!(useful(&t, &rows, &[Pat::Wild], &[Ty::Binary(Width::Lit(1))]).is_some());
    // With a default, `_` is no longer useful.
    let with_default = vec![vec![Pat::Lit("b:0".into())], vec![Pat::Wild]];
    assert!(useful(&t, &with_default, &[Pat::Wild], &[Ty::Binary(Width::Lit(1))]).is_none());
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
