//! Unit tests for `derive_nodule_path` (M-1042, DN-109 section 5.1 item 1 — data-driven, per
//! CLAUDE.md "Complex test logic lives in fixtures + parameterization, not in test bodies").
//!
//! The residual gap this covers: the flat-emit output-path collision (DN-109 section 2/section
//! 5.1) was already fixed by M-1006 Phase-2 (`e0085ec0`, `batch::output_rel_path` mirrors the
//! source tree under `out_dir`) — see `src/tests/batch.rs` for that coverage. What remained was
//! the `// nodule: <path>;` **header** itself: it was derived only from the crate directory name,
//! so two nested `mod.rs` files in the same crate still emitted the *same* nodule header even
//! though M-1006 now writes them to distinct output paths. This fixture table pins the fix: the
//! nodule path now also incorporates the intra-crate module path (the components between `src/`
//! and the leaf file).

use crate::transpile::derive_nodule_path;
use std::path::PathBuf;

/// One `derive_nodule_path` case: an input path (repo-root-relative, as the CLI would see it)
/// and its expected dotted nodule path.
struct Case {
    path: &'static str,
    expected: &'static str,
}

const CASES: &[Case] = &[
    // Regression: the pre-existing top-level case (a file directly under a crate's `src/`) is
    // unchanged by the intra-crate-path generalization.
    Case {
        path: "crates/mycelium-std-cmp/src/lib.rs",
        expected: "std.cmp",
    },
    // The DN-109 section 5.1 item 1 residual: `a/b/mod.rs` -> a dotted nodule name that
    // distinguishes it from the crate-root nodule and from any sibling `mod.rs`.
    Case {
        path: "crates/mycelium-std-cmp/src/foo/mod.rs",
        expected: "std.cmp.foo",
    },
    // Two-deep nesting: `mod.rs` never contributes its own segment, only the directories do.
    Case {
        path: "crates/mycelium-std-cmp/src/foo/bar/mod.rs",
        expected: "std.cmp.foo.bar",
    },
    // A non-`mod.rs` submodule file contributes its own stem as the trailing segment.
    Case {
        path: "crates/mycelium-std-cmp/src/foo/bar.rs",
        expected: "std.cmp.foo.bar",
    },
    // A crate-root file that is *not* `lib.rs`/`mod.rs` (e.g. a single-file semcore transpile
    // target): the old grandparent-only heuristic collapsed every such file in a crate onto the
    // same crate-level nodule name (a real collision this fix also closes); the file's own stem
    // now disambiguates it.
    Case {
        path: "crates/mycelium-l1/src/checkty.rs",
        expected: "l1.checkty",
    },
    Case {
        path: "crates/mycelium-l1/src/elab.rs",
        expected: "l1.elab",
    },
    // A hyphenated crate name maps every `-` to `.`, same as the crate-root case.
    Case {
        path: "crates/mycelium-std-sys-host/src/lib.rs",
        expected: "std.sys.host",
    },
    // No `src` ancestor at all: falls back to the bare file stem rather than mis-deriving a path
    // (never-silent, G2) — exercises the same fallback the pre-existing `None` arm covered.
    Case {
        path: "standalone/probe.rs",
        expected: "probe",
    },
];

#[test]
fn derive_nodule_path_matches_expected_for_every_case() {
    for case in CASES {
        let got = derive_nodule_path(&PathBuf::from(case.path));
        assert_eq!(
            got, case.expected,
            "derive_nodule_path({:?}) = {got:?}, expected {:?}",
            case.path, case.expected
        );
    }
}

/// The DN-109 collision property directly: two distinct nested `mod.rs` files in the same crate
/// must not derive the same nodule path (the exact bug this residual fix closes — same-crate
/// sibling `mod.rs` files no longer collide on the crate-level prefix alone).
#[test]
fn nested_mod_rs_siblings_do_not_collide() {
    let a = derive_nodule_path(&PathBuf::from("crates/mycelium-std-cmp/src/foo/mod.rs"));
    let b = derive_nodule_path(&PathBuf::from("crates/mycelium-std-cmp/src/bar/mod.rs"));
    assert_ne!(
        a, b,
        "two distinct mod.rs files in the same crate derived the same nodule path: {a:?}"
    );
}
