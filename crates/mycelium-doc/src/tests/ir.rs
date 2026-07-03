//! White-box tests for [`crate::ir`] — extracted from the logic file (as-touched, CLAUDE.md test
//! layout rule) when [`Node::walk`] was guarded against a host-stack overflow (RFC-0041 W1, §4.7).

use crate::ir::*;

fn prov() -> Provenance {
    Provenance {
        source: "docs/x.md".to_owned(),
        line: 1,
    }
}

#[test]
fn identity_is_content_addressed_and_excludes_provenance() {
    let a = Node::new(
        "p",
        None,
        None,
        Provenance {
            source: "a.md".to_owned(),
            line: 1,
        },
        Payload::Prose {
            text: "hi".to_owned(),
        },
        vec![],
    );
    let b = Node::new(
        "p",
        None,
        None,
        Provenance {
            source: "b.md".to_owned(),
            line: 999,
        },
        Payload::Prose {
            text: "hi".to_owned(),
        },
        vec![],
    );
    // Same projected content, different provenance → same address (provenance is not identity).
    assert_eq!(a.id.as_str(), b.id.as_str());
}

#[test]
fn different_content_gives_a_different_address() {
    let a = Node::new(
        "p",
        None,
        None,
        prov(),
        Payload::Prose {
            text: "one".to_owned(),
        },
        vec![],
    );
    let b = Node::new(
        "p",
        None,
        None,
        prov(),
        Payload::Prose {
            text: "two".to_owned(),
        },
        vec![],
    );
    assert_ne!(a.id.as_str(), b.id.as_str());
}

#[test]
fn a_parents_address_depends_on_its_children() {
    let child1 = Node::new(
        "c1",
        None,
        None,
        prov(),
        Payload::Prose {
            text: "a".to_owned(),
        },
        vec![],
    );
    let child2 = Node::new(
        "c2",
        None,
        None,
        prov(),
        Payload::Prose {
            text: "b".to_owned(),
        },
        vec![],
    );
    let p1 = Node::new(
        "s",
        Some("S".to_owned()),
        None,
        prov(),
        Payload::Section,
        vec![child1.clone()],
    );
    let p2 = Node::new(
        "s",
        Some("S".to_owned()),
        None,
        prov(),
        Payload::Section,
        vec![child1, child2],
    );
    assert_ne!(p1.id.as_str(), p2.id.as_str());
}

#[test]
fn the_model_indexes_every_anchor() {
    let doc = Node::new(
        "doc",
        Some("Doc".to_owned()),
        None,
        prov(),
        Payload::Document {
            source_kind: SourceKind::Spec,
        },
        vec![Node::new(
            "doc-s1",
            Some("S1".to_owned()),
            None,
            prov(),
            Payload::Section,
            vec![],
        )],
    );
    let m = DocModel::new(vec![doc]);
    assert!(m.anchors.contains_key("doc"));
    assert!(m.anchors.contains_key("doc-s1"));
    assert_eq!(m.all_nodes().len(), 2);
    assert_eq!(m.id_set().len(), 2);
}

/// RFC-0041 W1/§4.7 regression: [`Node::walk`] now runs on the
/// [`mycelium_workstack::ensure_sufficient_stack`]-grown worker stack, so a genuinely deep chain (far
/// past a default 2 MiB thread stack) walks to completion instead of overflowing — the companion to
/// the `mycelium-doc/tests/guard_hole_census.rs::node_walk_deep_chain` black-box repro, at a size that
/// stays fast for the unit-test tier.
///
/// Uses `mem::forget` on the built chain rather than letting it drop: `Node`'s derived recursive
/// `Drop` overflows the stack on a deep chain independently of `walk` (confirmed empirically down to
/// n=50,000 — a distinct, untracked hole flagged in the census test's doc comment, out of this leaf's
/// scope). Forgetting keeps this test isolated to the `walk` fix it regresses.
#[test]
fn walk_does_not_overflow_on_a_deep_chain() {
    let mut acc = Node::new(
        "leaf",
        None,
        Some(Level::Minimal),
        prov(),
        Payload::Section,
        vec![],
    );
    for i in 0..50_000 {
        acc = Node::new(
            format!("n{i}"),
            None,
            Some(Level::Minimal),
            prov(),
            Payload::Section,
            vec![acc],
        );
    }
    let mut count = 0usize;
    acc.walk(&mut |_n| count += 1);
    assert_eq!(count, 50_001);
    std::mem::forget(acc);
}
