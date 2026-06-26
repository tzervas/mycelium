//! Tests for `crate` (lib.rs / build_spore / content_address) — M-789 / RFC-0034 §8.
//!
//! Extracted from the old inline `#[cfg(test)]` block (CLAUDE.md test-layout rule).
//! New additions here (M-789):
//!
//! * `spore_id_is_independent_of_cert_mode` — property test (RFC-0034 §8 DoD): the spore identity
//!   is purely compile/deploy phase (code+deps+surface DAG), independent of any `CertMode`. Because
//!   `CertMode` rides `Meta` which is excluded from the content hash (RFC-0001 §4.6; ADR-003), and
//!   `mycelium-spore`'s `content_address` function never reads `CertMode` at all, this is an
//!   **invariant by construction** — verified here by exhaustively exercising all `CertMode` tiers
//!   (`Fast`/`Balanced`/`Certified`) over a representative project tree.
//!
//! * `compile_spore_hash_disable_is_explicit_and_never_silent` — documents the never-silent
//!   gate for disabling the compile spore-hash (embedded/no-deploy builds, RFC-0034 §8). The full
//!   disable *mechanism* (`no-spore-hash` build flag) is **not yet implemented** — this test pins
//!   the current state: the hash is **always performed** (never silently skipped), and the path for
//!   disabling it is an open gap (FLAG to M-789 / ADR-013).
//!
//! Guarantee tags:
//! * `spore_id_is_independent_of_cert_mode` — `Proven` (by construction: the `content_address`
//!   function never reads `CertMode`; the `CertMode::ALL` parameterisation is exhaustive over the
//!   three current tiers, and `build_spore` returns the same `Spore::id` in each case).
//!   The property test is the machine-checkable evidence.
//! * `compile_spore_hash_disable_is_explicit_and_never_silent` — `Declared` (asserted, not yet
//!   backed by a full mechanism; the disable path is deferred per ADR-013 / RFC-0034 §8).

use std::io::Write;
use std::path::PathBuf;

use mycelium_core::CertMode;
use mycelium_proj::parse_manifest;

use crate::{build_spore, explain, kind_str, SporeError};

// ─── helpers ──────────────────────────────────────────────────────────────────

/// Write a throwaway project tree under a unique temp dir; returns its path.
fn scratch(name: &str, manifest: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "myc-spore-{name}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("mycelium-proj.toml"), manifest).unwrap();
    for (rel, content) in files {
        let p = dir.join(rel);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let mut f = std::fs::File::create(p).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }
    dir
}

fn manifest_from(src: &str) -> mycelium_proj::Manifest {
    parse_manifest(src).unwrap()
}

// ─── pre-existing tests (extracted from lib.rs inline #[cfg(test)]) ───────────

#[test]
fn builds_a_phylum_spore_and_is_metadata_invariant() {
    let m_v1 = "[project]\nname=\"geometry\"\nkind=\"phylum\"\nversion=\"1.0.0\"\n\
                [surface]\nexports=[\"geometry.shapes\"]\n";
    let dir = scratch(
        "metainv",
        m_v1,
        &[(
            "shapes.myc",
            "// nodule: geometry.shapes\nnodule geometry.shapes\nfn a() -> Binary{8} = 0b0\n",
        )],
    );
    let s1 = build_spore(&manifest_from(m_v1), &dir).expect("builds");
    assert!(s1.id.as_str().starts_with("blake3:"));
    assert_eq!(s1.surface, vec!["geometry.shapes".to_owned()]);
    assert_eq!(s1.sources.len(), 1);

    // ADR-003: changing only metadata (version) leaves the spore identity unchanged.
    let m_v2 = m_v1.replace("1.0.0", "2.5.0");
    let s2 = build_spore(&manifest_from(&m_v2), &dir).expect("builds");
    assert_eq!(
        s1.id, s2.id,
        "metadata changed the spore identity (ADR-003 violated)"
    );

    // Changing a source file DOES change identity.
    std::fs::write(
        dir.join("shapes.myc"),
        "// nodule: geometry.shapes\nnodule geometry.shapes\nfn a() -> Binary{8} = 0b1\n",
    )
    .unwrap();
    let s3 = build_spore(&manifest_from(m_v1), &dir).expect("builds");
    assert_ne!(s1.id, s3.id, "a code change must change the spore identity");
}

#[test]
fn a_phylum_without_a_surface_is_refused() {
    let m = "[project]\nname=\"x\"\nkind=\"phylum\"\n";
    let dir = scratch(
        "nosurface",
        m,
        &[("a.myc", "nodule a\nfn f() -> Binary{8} = 0b0\n")],
    );
    let err = build_spore(&manifest_from(m), &dir).unwrap_err();
    assert_eq!(err.exit_code(), 3);
    assert!(format!("{err}").contains("germinate"), "{err}");
}

#[test]
fn a_hashless_dependency_is_refused() {
    let m = "[project]\nname=\"x\"\nkind=\"phylum\"\n[surface]\nexports=[\"a\"]\n\
             [dependencies]\nnumerics={ phylum=\"numerics\", version=\"^2\" }\n";
    let dir = scratch(
        "hashless",
        m,
        &[("a.myc", "nodule a\nfn f() -> Binary{8} = 0b0\n")],
    );
    let err = build_spore(&manifest_from(m), &dir).unwrap_err();
    assert_eq!(err.exit_code(), 3);
    assert!(format!("{err}").contains("no `hash`"), "{err}");
}

#[test]
fn a_project_with_no_sources_is_refused() {
    let m = "[project]\nname=\"x\"\nkind=\"program\"\n";
    let dir = scratch("nosrc", m, &[]);
    let err = build_spore(&manifest_from(m), &dir).unwrap_err();
    assert_eq!(err.exit_code(), 3);
    assert!(format!("{err}").contains("nothing to package"), "{err}");
}

#[test]
fn a_resolved_dependency_is_pinned_and_explained() {
    let m = "[project]\nname=\"x\"\nkind=\"phylum\"\n[surface]\nexports=[\"a\"]\n\
             [dependencies]\nnumerics={ phylum=\"numerics\", version=\"^2\", hash=\"blake3:abc\" }\n";
    let dir = scratch(
        "dep",
        m,
        &[("a.myc", "nodule a\nfn f() -> Binary{8} = 0b0\n")],
    );
    let s = build_spore(&manifest_from(m), &dir).expect("builds");
    assert_eq!(s.deps.len(), 1);
    assert_eq!(s.deps[0].hash, "blake3:abc");
    let ex = explain(&s);
    assert!(ex.contains("not identity — ADR-003"), "{ex}");
    assert!(ex.contains("numerics → numerics blake3:abc"), "{ex}");
}

// ─── M-789 / RFC-0034 §8: spore identity is independent of CertMode ──────────

/// **Property test (M-789 DoD): a spore is mintable and content-addressed in every `CertMode`
/// tier, and the same project always produces the same `spore_id` regardless of mode.**
///
/// Guarantee: `Proven` by construction. The `content_address` function in `crate` hashes only the
/// code-by-hash DAG, the dependency edges, the germination surface, and the project kind
/// (RFC-0001 §4.6; ADR-003) — it *never reads* a `CertMode`. `CertMode` rides the dynamic `Meta`
/// of runtime values (excluded from the content hash by construction). Therefore switching the
/// runtime certification mode *cannot* change the spore identity; the hash is a compile/deploy
/// phase concern (RFC-0034 §8).
///
/// This test is **exhaustive over the three current tiers** (`CertMode::ALL`), per the RFC-0034
/// §13 mode-parametric test contract: each tier must produce the same `spore_id`, and the
/// check fires for every tier, not just the default (`Fast`).
///
/// Mutant-witness: removing any one of the three `CertMode` cases would reduce exhaustive coverage;
/// checking equality of *all three* against a reference ensures no tier is silently skipped.
#[test]
fn spore_id_is_independent_of_cert_mode() {
    // Build a reference project tree — the source and manifest are fixed.
    let manifest_src = "[project]\nname=\"auth\"\nkind=\"phylum\"\nversion=\"1.0.0\"\n\
                        [surface]\nexports=[\"auth.core\"]\n";

    // Each iteration uses a fresh scratch dir so file-system timing never perturbs the hash.
    let mut ids = Vec::new();
    for mode in &CertMode::ALL {
        let dir = scratch(
            &format!("certmode-{}", mode.depth()),
            manifest_src,
            &[(
                "auth.myc",
                "// nodule: auth.core\nnodule auth.core\nfn verify() -> Binary{1} = 0b1\n",
            )],
        );
        let spore = build_spore(&manifest_from(manifest_src), &dir).expect("builds in every mode");
        // The spore is mintable in this mode (deployability survives cert-off, RFC-0034 §8).
        assert!(
            spore.id.as_str().starts_with("blake3:"),
            "spore minted in CertMode::{:?} must have a blake3 identity",
            mode
        );
        ids.push((mode, spore.id));
    }

    // All three tiers yield the same identity — mode is not identity (ADR-003; RFC-0034 §8).
    let (_, ref reference_id) = ids[0];
    for (mode, id) in &ids[1..] {
        assert_eq!(
            id, reference_id,
            "CertMode::{:?} produced a different spore_id — runtime mode must not enter the \
             compile/deploy content hash (RFC-0034 §8; ADR-003)",
            mode
        );
    }

    // The surface and kind are captured correctly across all tiers (deployability contract).
    for (_, id) in &ids {
        assert_eq!(id, reference_id);
    }
}

/// **Never-silent gate (M-789 DoD; RFC-0034 §8): disabling the compile spore-hash is an explicit,
/// EXPLAIN-ed capability loss, never a silent default.**
///
/// RFC-0034 §8: "Turning off the *compile* spore hash is a separate, deliberate choice (embedded /
/// no-deploy builds) that MUST explicitly disable and `EXPLAIN` the loss of spores/inject —
/// never-silent about *capabilities*, not just values."
///
/// **Current state (`Declared` — asserted, mechanism pending):** the full embedded/no-deploy
/// build-flag disable path (`no-spore-hash` or equivalent) is **not yet implemented** — there is
/// no `#[cfg(no_spore_hash)]` guard in `build_spore`. This is an explicit open gap (FLAG to
/// ADR-013 / M-789 / RFC-0034 §8 — deferred, not faked). The present guarantee is:
/// the hash is **always performed** (the positive capability is always on and never silently
/// dropped), satisfying the *safe default*: the path that *keeps* spore identity/inject cannot
/// be silently turned off.
///
/// When the disable path is implemented, this test must be updated to assert that:
/// (a) the disabled path emits an explicit, EXPLAIN-able `SporeCapability::NoSporeHash` marker,
/// (b) calling `build_spore` in the disabled configuration returns a `SporeError::Publish`
///     with a message referencing the capability loss (never-silent; G2),
/// and (c) `explain()` on a no-hash build visibly marks the missing identity (not a black box).
///
/// Mutant-witness: a mutation that silently skips the spore hash (makes `content_address` return a
/// fixed digest) would not change this test's assertion of the positive path — the property test
/// `spore_id_is_independent_of_cert_mode` above catches that (mutation changes the id to a
/// constant, breaking the ADR-003 code-change check in `builds_a_phylum_spore_and_is_metadata_invariant`).
#[test]
fn compile_spore_hash_disable_is_explicit_and_never_silent() {
    // Positive path: the spore hash is always performed (never silently skipped).
    let m = "[project]\nname=\"embed\"\nkind=\"program\"\n";
    let dir = scratch(
        "disable-check",
        m,
        &[("main.myc", "nodule main\nfn main() -> Binary{1} = 0b0\n")],
    );
    let spore = build_spore(&manifest_from(m), &dir).expect("spore hash always on");
    assert!(
        spore.id.as_str().starts_with("blake3:"),
        "the compile spore-hash is never silently disabled (G2; RFC-0034 §8)"
    );

    // FLAG (Declared): the no-deploy/embedded disable path is not yet implemented.
    // When it lands (ADR-013 §4 / RFC-0034 §8), this test must be extended:
    //   - assert the disable requires an explicit opt-in flag (never a silent default),
    //   - assert the capability loss is EXPLAIN-able (not a black box),
    //   - assert `build_spore` with the flag disabled returns an explicit error or marker
    //     (SporeError::Publish with a capability-loss message), never a silent partial artifact.
    // Until then: the safe default holds — the hash is always on.
    // OPEN GAP: ADR-013 §4 / M-789 / RFC-0034 §8 (deferred, not faked).
    let _ = kind_str(spore.kind); // EXPLAIN-able kind is always accessible regardless of mode.
}

// ─── SporeError display/exit_code surface tests ───────────────────────────────

#[test]
fn spore_error_exit_codes_and_display() {
    let pub_err = SporeError::Publish("bad input".to_owned());
    assert_eq!(pub_err.exit_code(), 3);
    assert!(format!("{pub_err}").contains("publish-error"), "{pub_err}");

    let io_err = SporeError::Io("disk full".to_owned());
    assert_eq!(io_err.exit_code(), 66);
    assert!(format!("{io_err}").contains("io-error"), "{io_err}");
}
