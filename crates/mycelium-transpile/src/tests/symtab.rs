//! Unit tests for `src/symtab.rs` (gap-close-2, DN-34 §8.19/§8.20 Import gap-class lever) —
//! `use_candidates`' tree-flattening + head classification, and `SymbolTable`'s resolve/has_module
//! contract. End-to-end (batch-driven) coverage — cross-file resolution, `pub`-propagation, the
//! no-bare-name-collapse property — lives in `src/tests/batch.rs`, alongside the rest of the
//! batch-mode test corpus.

use crate::symtab::{use_candidates, CandidateKind, SymbolTable, UseCandidate};

fn candidates_of(src: &str) -> Option<Vec<UseCandidate>> {
    let item: syn::ItemUse = syn::parse_str(src).unwrap_or_else(|e| panic!("{src}: {e}"));
    use_candidates(&item.tree)
}

/// One `use_candidates` case: a `use` item's source text and the expected flattened leaves —
/// data-driven per CLAUDE.md "complex test logic lives in fixtures, not test bodies".
struct Case {
    name: &'static str,
    src: &'static str,
    expected: Option<&'static [(&'static [&'static str], Expect)]>,
}

#[derive(Debug, PartialEq, Eq)]
enum Expect {
    Name(&'static str),
    SelfModule,
    Rename(&'static str, &'static str),
    Glob,
}

const CASES: &[Case] = &[
    Case {
        name: "crate_headed_single",
        src: "use crate::checkty::Width;",
        expected: Some(&[(&["checkty"], Expect::Name("Width"))]),
    },
    Case {
        name: "crate_headed_grouped",
        src: "use crate::checkty::{Width, CheckError};",
        expected: Some(&[
            (&["checkty"], Expect::Name("Width")),
            (&["checkty"], Expect::Name("CheckError")),
        ]),
    },
    Case {
        name: "bare_head_crate_root_form",
        src: "use error::FsErr;",
        expected: Some(&[(&["error"], Expect::Name("FsErr"))]),
    },
    Case {
        name: "bare_head_pub_use",
        src: "pub use metadata::{FileKind, Metadata};",
        expected: Some(&[
            (&["metadata"], Expect::Name("FileKind")),
            (&["metadata"], Expect::Name("Metadata")),
        ]),
    },
    Case {
        name: "nested_module_path",
        src: "use crate::foo::bar::Baz;",
        expected: Some(&[(&["foo", "bar"], Expect::Name("Baz"))]),
    },
    Case {
        name: "self_in_group",
        src: "use crate::decision::{self, Head};",
        expected: Some(&[
            (&["decision"], Expect::SelfModule),
            (&["decision"], Expect::Name("Head")),
        ]),
    },
    Case {
        name: "rename",
        src: "use mycelium_interp::EvalError as KernelError;",
        expected: Some(&[(
            &["mycelium_interp"],
            Expect::Rename("EvalError", "KernelError"),
        )]),
    },
    Case {
        name: "glob",
        src: "use crate::checkty::*;",
        expected: Some(&[(&["checkty"], Expect::Glob)]),
    },
    Case {
        name: "self_headed_out_of_scope",
        src: "use self::foo::Bar;",
        expected: None,
    },
    Case {
        name: "super_headed_out_of_scope",
        src: "use super::foo::Bar;",
        expected: None,
    },
];

#[test]
fn use_candidates_matches_expected_for_every_case() {
    for case in CASES {
        let got = candidates_of(case.src);
        match (got, case.expected) {
            (None, None) => {}
            (Some(got), Some(expected)) => {
                assert_eq!(
                    got.len(),
                    expected.len(),
                    "{}: leaf count mismatch — got {got:?}",
                    case.name
                );
                for (leaf, (segs, exp)) in got.iter().zip(expected.iter()) {
                    assert_eq!(
                        leaf.module_segs,
                        segs.to_vec(),
                        "{}: module_segs mismatch for {leaf:?}",
                        case.name
                    );
                    let matches = match (&leaf.kind, exp) {
                        (CandidateKind::Name(n), Expect::Name(e)) => n == e,
                        (CandidateKind::SelfModule, Expect::SelfModule) => true,
                        (CandidateKind::Rename { from, to }, Expect::Rename(ef, et)) => {
                            from == ef && to == et
                        }
                        (CandidateKind::Glob, Expect::Glob) => true,
                        _ => false,
                    };
                    assert!(
                        matches,
                        "{}: kind mismatch for {leaf:?} vs {exp:?}",
                        case.name
                    );
                }
            }
            (got, expected) => panic!(
                "{}: expected {expected:?}-shaped result, got {got:?}",
                case.name
            ),
        }
    }
}

/// `SymbolTable::resolve` only ever hits when the module key AND the name are both present (an
/// item that exists in Rust source but never made it into the sibling's `emitted` set is a miss,
/// not a partial match) — and a hit is always the sibling's own derived nodule path, never the
/// bare module key (no-bare-name-collapse, the M-1060 lesson).
#[test]
fn symbol_table_resolve_requires_both_module_and_emitted_name() {
    let mut table = SymbolTable::new();
    table.insert(
        "checkty".to_string(),
        "l1.checkty".to_string(),
        ["Width".to_string(), "CheckError".to_string()]
            .into_iter()
            .collect(),
    );

    assert_eq!(table.resolve("checkty", "Width"), Some("l1.checkty"));
    assert_ne!(
        table.resolve("checkty", "Width"),
        Some("Width"),
        "a resolved hit must never be the bare module key or item name"
    );
    // In the Rust source `checkty.rs` may well declare `Env`/`Ty`, but this batch's baseline pass
    // never emitted them (they gapped) — so they are absent from `emitted` and must miss, not
    // fall back to a guessed resolution.
    assert_eq!(table.resolve("checkty", "Env"), None);
    // An unknown module entirely.
    assert_eq!(table.resolve("elab", "Width"), None);

    assert!(table.has_module("checkty"));
    assert!(!table.has_module("elab"));
}

/// `SymbolTable::module_key` is the exact `.`-join `use_candidates`' `module_segs` are matched
/// against — pinned directly so a future refactor of either side is caught by this contract test.
#[test]
fn symbol_table_module_key_is_dot_joined() {
    assert_eq!(SymbolTable::module_key(&["checkty".to_string()]), "checkty");
    assert_eq!(
        SymbolTable::module_key(&["foo".to_string(), "bar".to_string()]),
        "foo.bar"
    );
    assert_eq!(SymbolTable::module_key(&[]), "");
}
