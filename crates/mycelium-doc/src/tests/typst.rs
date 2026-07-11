//! White-box tests for [`crate::emit::typst`] — extracted from the logic file (as-touched, CLAUDE.md
//! test layout rule) when the print code-legibility pass landed. Uses `pub(crate)` access to `escape`.

use crate::corpus::{ingest, AnchorAlloc};
use crate::emit::typst::{escape, render};
use crate::ir::{DocModel, SourceKind};

fn model() -> DocModel {
    let mut a = AnchorAlloc::new();
    let src = "# Doc\n\nLead.\n\n## Sec\n\nBody text.\n\n```myc\nfn f() = 0\n```\n";
    DocModel::new(vec![ingest("d.md", src, SourceKind::Rfc, &mut a)])
}

#[test]
fn typst_has_a_preamble_and_outline() {
    let typ = render(&model());
    assert!(typ.contains("#set document"));
    assert!(typ.contains("#outline()"));
}

#[test]
fn headings_use_typst_equals_syntax() {
    let typ = render(&model());
    assert!(typ.contains("= Doc"));
    assert!(typ.contains("== Sec"));
}

#[test]
fn every_block_carries_its_cid() {
    let m = model();
    let typ = render(&m);
    for id in m.id_set() {
        assert!(typ.contains(&id), "missing cid {id}");
    }
}

#[test]
fn body_metacharacters_are_escaped() {
    assert_eq!(escape("a #b $c*"), "a \\#b \\$c\\*");
}

#[test]
fn code_blocks_get_a_print_legible_show_rule() {
    // The print pass (§8.2): body ~10.5pt, comfortable margins, and a `raw.where(block:true)` show
    // rule that renders code smaller (~0.82x) with tighter leading in a hairline-bordered tinted box
    // (never a filled dark panel). Assert the structural markers are emitted.
    let typ = render(&model());
    assert!(typ.contains("size: 10.5pt"), "body scale");
    assert!(typ.contains("margin:"), "comfortable margins");
    assert!(
        typ.contains("raw.where(block: true)"),
        "code show rule present"
    );
    assert!(typ.contains("size: 8.6pt"), "code is set smaller than body");
    assert!(typ.contains("leading: 0.42em"), "code has tighter leading");
    // A hairline stroke and a light fill, not a dark panel.
    assert!(typ.contains("stroke: 0.5pt"), "hairline border");
    assert!(typ.contains("fill: rgb(\"#e4e9d9\")"), "light tinted box");
}
