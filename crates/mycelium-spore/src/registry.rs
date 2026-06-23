//! `spore` **registry** — publish / resolve over a content-addressed store (M-732; ADR-003/ADR-013).
//!
//! Extends the [`crate::build_spore`] artifact into a package-manager capability: a phylum author
//! **publishes** a built spore to a registry, and a downstream developer **resolves** it by
//! `name + version` and fetches the exact, hash-verified artifact.
//!
//! ## Backend (the E16-1 open question, answered honestly for v0)
//! v0 is a **local, file-based content-addressed store** — the simplest backend that preserves
//! ADR-003 identity and is never-silent on integrity. A crates.io-style networked protocol is a
//! later RFC (deferred, not faked): nothing here pretends to do remote transport or auth. Layout
//! under the registry root:
//!
//! ```text
//! <root>/objects/<algo>-<digest>.spore   the artifact bytes, addressed by BLAKE3 of those bytes
//! <root>/index/<name>/<version>          a pointer: { spore_id, artifact } → the object above
//! ```
//!
//! ## Two content addresses, both checked (ADR-003)
//! * **`spore_id`** — the spore's *identity*: BLAKE3 over the canonical code+deps+surface DAG
//!   ([`crate::Spore::id`]). Metadata (`version`) is **not** identity, so the same code republished
//!   under a new version label keeps the same `spore_id`.
//! * **`artifact`** — the *integrity* hash: BLAKE3 of the stored descriptor bytes, so a corrupted or
//!   tampered object is caught. Verified **on publish and on resolve** (DoD; G2 — never silent).
//!
//! ## Never-silent (G2)
//! Immutability is enforced: republishing a *different* artifact under an existing `name@version` is
//! a refused [`RegistryError::Conflict`], never a silent overwrite. A resolve whose object is missing
//! or whose bytes don't hash to the recorded `artifact` is a [`RegistryError::Integrity`] error. A
//! version *range* (`^1`, `~2`) is an explicit [`RegistryError::Unsupported`] — v0 resolves an exact
//! version or `latest` only, and never silently mis-resolves a constraint it cannot honestly satisfy.

use std::path::{Path, PathBuf};

use mycelium_core::ContentHash;

use crate::Spore;

/// The outcome of a successful [`publish`]: the receipt a CLI prints (no black box).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishReceipt {
    /// The published package name.
    pub name: String,
    /// The published version label (metadata; not identity — ADR-003).
    pub version: String,
    /// The spore identity (DAG hash; ADR-003).
    pub spore_id: ContentHash,
    /// The integrity hash of the stored artifact bytes (BLAKE3 of the descriptor).
    pub artifact: ContentHash,
    /// Where the object was written.
    pub object_path: PathBuf,
    /// `true` if this exact artifact was already present (idempotent re-publish), `false` if newly written.
    pub already_present: bool,
}

/// The outcome of a successful [`resolve`]: the fetched, hash-verified artifact and its identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolved {
    /// The resolved name.
    pub name: String,
    /// The concrete version selected (after resolving `latest`).
    pub version: String,
    /// The spore identity recorded at publish time (ADR-003).
    pub spore_id: ContentHash,
    /// The integrity hash the bytes were verified against.
    pub artifact: ContentHash,
    /// The fetched artifact bytes (integrity-verified before return — G2).
    pub bytes: Vec<u8>,
}

/// A registry operation refusal — always explicit, never a partial/silent result (G2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    /// A missing/empty publish input (e.g. no version label to publish under) (exit 3).
    PublishInput(String),
    /// A `name`/`version` with no index entry (exit 4).
    NotFound(String),
    /// A content-integrity failure: the object is absent, or its bytes don't hash to the recorded
    /// `artifact`, or a recorded address is malformed (exit 5).
    Integrity(String),
    /// An immutability violation: a different artifact already occupies this `name@version` (exit 6).
    Conflict(String),
    /// A version *constraint* v0 cannot honestly satisfy (a range/caret/tilde) (exit 64).
    Unsupported(String),
    /// An I/O error touching the registry tree (exit 66).
    Io(String),
}

impl RegistryError {
    /// The CLI exit code for this refusal.
    #[must_use]
    pub fn exit_code(&self) -> u8 {
        match self {
            RegistryError::PublishInput(_) => 3,
            RegistryError::NotFound(_) => 4,
            RegistryError::Integrity(_) => 5,
            RegistryError::Conflict(_) => 6,
            RegistryError::Unsupported(_) => 64,
            RegistryError::Io(_) => 66,
        }
    }
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryError::PublishInput(m) => write!(f, "publish-input-error: {m}"),
            RegistryError::NotFound(m) => write!(f, "not-found: {m}"),
            RegistryError::Integrity(m) => write!(f, "integrity-error: {m}"),
            RegistryError::Conflict(m) => write!(f, "conflict: {m}"),
            RegistryError::Unsupported(m) => write!(f, "unsupported: {m}"),
            RegistryError::Io(m) => write!(f, "io-error: {m}"),
        }
    }
}

impl std::error::Error for RegistryError {}

/// BLAKE3 the bytes into a `blake3:<hex>` [`ContentHash`] — the integrity address of an artifact.
#[must_use]
pub fn artifact_hash(bytes: &[u8]) -> ContentHash {
    let hex = blake3::hash(bytes).to_hex();
    ContentHash::from_parts("blake3", hex.as_str()).expect("blake3 hex is a valid digest")
}

/// The on-disk object path for an artifact hash: `<root>/objects/<algo>-<digest>.spore`. The `algo`
/// prefix keeps the address self-describing and avoids a `:` in the filename.
fn object_path(root: &Path, hash: &ContentHash) -> PathBuf {
    root.join("objects")
        .join(format!("{}-{}.spore", hash.algo(), hash.digest()))
}

/// The index pointer path for a `name@version`: `<root>/index/<name>/<version>`.
fn index_path(root: &Path, name: &str, version: &str) -> PathBuf {
    root.join("index").join(name).join(version)
}

fn io<E: std::fmt::Display>(ctx: &str, e: E) -> RegistryError {
    RegistryError::Io(format!("{ctx}: {e}"))
}

/// **Publish** `spore`'s `descriptor` bytes under `name@version` into the registry at `root`.
///
/// The descriptor bytes are content-addressed (BLAKE3 → `artifact`), stored once under
/// `objects/`, and pointed at by an `index/<name>/<version>` entry recording both the `artifact`
/// integrity hash and the `spore_id` DAG identity (ADR-003). Idempotent for an identical
/// re-publish; a *different* artifact under an existing `name@version` is a refused
/// [`RegistryError::Conflict`] (immutability, G2). The integrity hash is verified by reading the
/// object back after write.
///
/// # Errors
/// [`RegistryError::PublishInput`] for an empty `version`; [`RegistryError::Conflict`] on an
/// immutability violation; [`RegistryError::Integrity`] if the written object fails read-back
/// verification; [`RegistryError::Io`] on a filesystem failure.
pub fn publish(
    root: &Path,
    spore: &Spore,
    descriptor: &[u8],
    name: &str,
    version: &str,
) -> Result<PublishReceipt, RegistryError> {
    if version.trim().is_empty() {
        return Err(RegistryError::PublishInput(
            "a publish needs a non-empty version label (it is never guessed; ADR-003 metadata)"
                .to_owned(),
        ));
    }
    let artifact = artifact_hash(descriptor);
    let obj = object_path(root, &artifact);

    // Write (or verify-idempotent) the content-addressed object.
    let mut already_present = false;
    if obj.exists() {
        let existing = std::fs::read(&obj).map_err(|e| io(&obj.display().to_string(), e))?;
        if existing != descriptor {
            // The bytes at a content address must equal the address's hash; otherwise the store is
            // corrupt — refuse, never overwrite (G2).
            return Err(RegistryError::Integrity(format!(
                "object {} exists but its bytes do not match the artifact hash — store corruption",
                obj.display()
            )));
        }
        already_present = true;
    } else {
        if let Some(parent) = obj.parent() {
            std::fs::create_dir_all(parent).map_err(|e| io(&parent.display().to_string(), e))?;
        }
        std::fs::write(&obj, descriptor).map_err(|e| io(&obj.display().to_string(), e))?;
        // Read-back integrity check: the object on disk must hash to `artifact` (never-silent, G2).
        let back = std::fs::read(&obj).map_err(|e| io(&obj.display().to_string(), e))?;
        if artifact_hash(&back) != artifact {
            return Err(RegistryError::Integrity(format!(
                "post-write verification of {} failed — artifact hash mismatch",
                obj.display()
            )));
        }
    }

    // Write (or verify-immutable) the name@version index entry.
    let idx = index_path(root, name, version);
    let entry = format_entry(&spore.id, &artifact);
    if idx.exists() {
        let existing =
            std::fs::read_to_string(&idx).map_err(|e| io(&idx.display().to_string(), e))?;
        let (prev_id, prev_art) = parse_entry(&existing).ok_or_else(|| {
            RegistryError::Integrity(format!("malformed index entry {}", idx.display()))
        })?;
        if prev_id != spore.id || prev_art != artifact {
            return Err(RegistryError::Conflict(format!(
                "{name}@{version} is already published with a different artifact \
                 (have spore_id={}, artifact={}); publishing a new artifact requires a new version \
                 — a registry is immutable (G2)",
                prev_id.as_str(),
                prev_art.as_str()
            )));
        }
    } else {
        if let Some(parent) = idx.parent() {
            std::fs::create_dir_all(parent).map_err(|e| io(&parent.display().to_string(), e))?;
        }
        std::fs::write(&idx, entry).map_err(|e| io(&idx.display().to_string(), e))?;
    }

    Ok(PublishReceipt {
        name: name.to_owned(),
        version: version.to_owned(),
        spore_id: spore.id.clone(),
        artifact,
        object_path: obj,
        already_present,
    })
}

/// **Resolve** `name` at `constraint` against the registry at `root`, returning the integrity-verified
/// artifact. `constraint` is either an **exact version** (`"1.2.0"`) or `"latest"` / `"*"` (the
/// highest published version). A *range* constraint (`^`, `~`, `>=`) is [`RegistryError::Unsupported`]
/// — v0 never silently mis-resolves a SemVer range it cannot honestly evaluate (the range backend is
/// the deferred ADR-018 work).
///
/// # Errors
/// [`RegistryError::NotFound`] if no matching `name`/version exists; [`RegistryError::Unsupported`]
/// for a range constraint; [`RegistryError::Integrity`] if the object is missing or its bytes fail
/// the `artifact` hash check; [`RegistryError::Io`] on a filesystem failure.
pub fn resolve(root: &Path, name: &str, constraint: &str) -> Result<Resolved, RegistryError> {
    let version = select_version(root, name, constraint)?;
    let idx = index_path(root, name, &version);
    let entry = std::fs::read_to_string(&idx).map_err(|_| {
        RegistryError::NotFound(format!("{name}@{version} has no registry index entry"))
    })?;
    let (spore_id, artifact) = parse_entry(&entry).ok_or_else(|| {
        RegistryError::Integrity(format!("malformed index entry {}", idx.display()))
    })?;

    let obj = object_path(root, &artifact);
    let bytes = std::fs::read(&obj).map_err(|_| {
        RegistryError::Integrity(format!(
            "{name}@{version} index points at a missing object {} (store corruption)",
            obj.display()
        ))
    })?;
    // The load-bearing check (G2): fetched bytes must hash to the recorded artifact address.
    if artifact_hash(&bytes) != artifact {
        return Err(RegistryError::Integrity(format!(
            "{name}@{version}: fetched object {} does not match its content address {} — tampered or corrupt",
            obj.display(),
            artifact.as_str()
        )));
    }

    Ok(Resolved {
        name: name.to_owned(),
        version,
        spore_id,
        artifact,
        bytes,
    })
}

/// Choose the concrete version for `constraint`: an exact match, or the highest published version for
/// `latest`/`*`. A range constraint is refused (unsupported in v0; never silently mis-resolved).
fn select_version(root: &Path, name: &str, constraint: &str) -> Result<String, RegistryError> {
    let c = constraint.trim();
    if c.is_empty() {
        return Err(RegistryError::NotFound(format!(
            "{name}: an empty version constraint resolves nothing (it is never guessed)"
        )));
    }
    if c == "latest" || c == "*" {
        let mut versions = published_versions(root, name)?;
        versions.sort_by(|a, b| version_key(a).cmp(&version_key(b)));
        return versions
            .pop()
            .ok_or_else(|| RegistryError::NotFound(format!("{name}: no versions published")));
    }
    // A range/caret/tilde/comparator is explicitly unsupported in v0 (honest, never mis-resolved).
    if c.starts_with(['^', '~', '>', '<', '=']) || c.contains(',') {
        return Err(RegistryError::Unsupported(format!(
            "version constraint {c:?} is a range — v0 resolves an exact version or `latest` only; \
             SemVer range resolution is the deferred ADR-018 work, not silently approximated (VR-5)"
        )));
    }
    // An exact version: it must exist (never invented).
    if index_path(root, name, c).exists() {
        Ok(c.to_owned())
    } else {
        Err(RegistryError::NotFound(format!(
            "{name}@{c} is not published"
        )))
    }
}

/// The published version labels for `name` (the filenames under `index/<name>/`).
fn published_versions(root: &Path, name: &str) -> Result<Vec<String>, RegistryError> {
    let dir = root.join("index").join(name);
    let rd = std::fs::read_dir(&dir)
        .map_err(|_| RegistryError::NotFound(format!("{name}: not published in this registry")))?;
    let mut out = Vec::new();
    for e in rd {
        let e = e.map_err(|err| io(&dir.display().to_string(), err))?;
        if e.path().is_file() {
            if let Some(v) = e.file_name().to_str() {
                out.push(v.to_owned());
            }
        }
    }
    Ok(out)
}

/// A coarse version sort key: dotted numeric components compared numerically, with a lexical
/// fallback for non-numeric parts. Honest scope (`Declared`): this orders simple `MAJOR.MINOR.PATCH`
/// labels for `latest`; it is **not** a full SemVer precedence implementation (pre-release/build
/// metadata are not interpreted) — that is the deferred ADR-018 work.
fn version_key(v: &str) -> Vec<(u64, String)> {
    v.split('.')
        .map(|part| {
            let num = part
                .chars()
                .take_while(char::is_ascii_digit)
                .collect::<String>();
            (num.parse::<u64>().unwrap_or(0), part.to_owned())
        })
        .collect()
}

/// The two-line index-entry encoding: `spore_id` (identity) + `artifact` (integrity).
fn format_entry(spore_id: &ContentHash, artifact: &ContentHash) -> String {
    format!(
        "spore_id = {}\nartifact = {}\n",
        spore_id.as_str(),
        artifact.as_str()
    )
}

/// Parse a [`format_entry`] index entry back into `(spore_id, artifact)`. Returns `None` (→ an
/// explicit `Integrity` error at the call site) on any malformed/missing field — never a silent default.
fn parse_entry(text: &str) -> Option<(ContentHash, ContentHash)> {
    let mut spore_id = None;
    let mut artifact = None;
    for line in text.lines() {
        let (k, v) = line.split_once('=')?;
        match k.trim() {
            "spore_id" => spore_id = ContentHash::parse(v.trim()),
            "artifact" => artifact = ContentHash::parse(v.trim()),
            _ => {}
        }
    }
    Some((spore_id?, artifact?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use mycelium_proj::parse_manifest;
    use std::io::Write;

    fn scratch_registry(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "myc-registry-{tag}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Build a tiny phylum spore under a scratch project dir.
    fn demo_spore(tag: &str, body: &str) -> (Spore, Vec<u8>) {
        let m = "[project]\nname=\"geo\"\nkind=\"phylum\"\nversion=\"1.0.0\"\n\
                 [surface]\nexports=[\"geo.shapes\"]\n";
        let dir = std::env::temp_dir().join(format!(
            "myc-spore-src-{tag}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let mut f = std::fs::File::create(dir.join("shapes.myc")).unwrap();
        f.write_all(body.as_bytes()).unwrap();
        let spore = crate::build_spore(&parse_manifest(m).unwrap(), &dir).unwrap();
        let descriptor = crate::explain(&spore).into_bytes();
        (spore, descriptor)
    }

    #[test]
    fn publish_then_resolve_round_trips_and_verifies_integrity() {
        let reg = scratch_registry("roundtrip");
        let (spore, descriptor) = demo_spore(
            "rt",
            "// nodule: geo.shapes\nnodule geo.shapes\nfn a() -> Binary{8} = 0b0\n",
        );

        let receipt = publish(&reg, &spore, &descriptor, "geo", "1.0.0").unwrap();
        assert_eq!(receipt.artifact, artifact_hash(&descriptor));
        assert_eq!(receipt.spore_id, spore.id);
        assert!(!receipt.already_present);

        let got = resolve(&reg, "geo", "1.0.0").unwrap();
        assert_eq!(got.bytes, descriptor);
        assert_eq!(got.spore_id, spore.id);
        assert_eq!(got.artifact, receipt.artifact);

        // `latest` resolves the single published version.
        assert_eq!(resolve(&reg, "geo", "latest").unwrap().version, "1.0.0");
    }

    #[test]
    fn republish_is_idempotent_but_a_different_artifact_conflicts() {
        let reg = scratch_registry("immutable");
        let (s1, d1) = demo_spore(
            "a",
            "// nodule: geo.shapes\nnodule geo.shapes\nfn a() -> Binary{8} = 0b0\n",
        );
        publish(&reg, &s1, &d1, "geo", "1.0.0").unwrap();
        // Same artifact, same version → idempotent.
        let again = publish(&reg, &s1, &d1, "geo", "1.0.0").unwrap();
        assert!(again.already_present);

        // A different artifact under the SAME name@version is refused, never silently overwritten (G2).
        let (s2, d2) = demo_spore(
            "b",
            "// nodule: geo.shapes\nnodule geo.shapes\nfn a() -> Binary{8} = 0b1\n",
        );
        let err = publish(&reg, &s2, &d2, "geo", "1.0.0").unwrap_err();
        assert_eq!(err.exit_code(), 6, "{err}");
        assert!(format!("{err}").contains("immutable"), "{err}");
    }

    #[test]
    fn a_tampered_object_is_caught_on_resolve_not_silently_served() {
        let reg = scratch_registry("tamper");
        let (spore, descriptor) = demo_spore(
            "t",
            "// nodule: geo.shapes\nnodule geo.shapes\nfn a() -> Binary{8} = 0b0\n",
        );
        let receipt = publish(&reg, &spore, &descriptor, "geo", "1.0.0").unwrap();

        // Tamper with the stored object bytes (flip the tail).
        let mut bytes = std::fs::read(&receipt.object_path).unwrap();
        *bytes.last_mut().unwrap() ^= 0xFF;
        std::fs::write(&receipt.object_path, &bytes).unwrap();

        let err = resolve(&reg, "geo", "1.0.0").unwrap_err();
        assert_eq!(err.exit_code(), 5, "{err}");
        assert!(format!("{err}").contains("content address"), "{err}");
    }

    #[test]
    fn an_unpublished_name_or_version_is_not_found_never_invented() {
        let reg = scratch_registry("missing");
        assert_eq!(resolve(&reg, "nope", "1.0.0").unwrap_err().exit_code(), 4);
        let (spore, descriptor) = demo_spore(
            "m",
            "// nodule: geo.shapes\nnodule geo.shapes\nfn a() -> Binary{8} = 0b0\n",
        );
        publish(&reg, &spore, &descriptor, "geo", "1.0.0").unwrap();
        assert_eq!(resolve(&reg, "geo", "9.9.9").unwrap_err().exit_code(), 4);
    }

    #[test]
    fn a_range_constraint_is_unsupported_not_mis_resolved() {
        // VR-5: v0 must refuse a SemVer range rather than silently pretend to satisfy it.
        let reg = scratch_registry("range");
        let (spore, descriptor) = demo_spore(
            "r",
            "// nodule: geo.shapes\nnodule geo.shapes\nfn a() -> Binary{8} = 0b0\n",
        );
        publish(&reg, &spore, &descriptor, "geo", "1.2.3").unwrap();
        let err = resolve(&reg, "geo", "^1.0").unwrap_err();
        assert_eq!(err.exit_code(), 64, "{err}");
        assert!(format!("{err}").contains("range"), "{err}");
    }

    // --- property test: the hash-verification bound (M-732 DoD) ---
    proptest::proptest! {
        /// For ANY descriptor bytes, the integrity address is the BLAKE3 of those bytes, and any
        /// single-byte mutation changes the address — so a tampered object can never hash to the
        /// recorded `artifact` (the never-silent integrity guarantee, G2). Guarantee: `Empirical`
        /// (trials) — BLAKE3 collision resistance itself is `Declared` upstream, not re-proven here.
        #[test]
        fn artifact_hash_is_stable_and_mutation_sensitive(
            bytes in proptest::collection::vec(proptest::num::u8::ANY, 1..256),
            idx in 0usize..256,
        ) {
            let h = artifact_hash(&bytes);
            // Deterministic: re-hashing the same bytes yields the same address.
            proptest::prop_assert_eq!(&h, &artifact_hash(&bytes));
            // Mutation-sensitive: flipping one byte changes the address (so resolve's check fires).
            let mut tampered = bytes.clone();
            let i = idx % tampered.len();
            tampered[i] = tampered[i].wrapping_add(1);
            proptest::prop_assert_ne!(h, artifact_hash(&tampered));
        }
    }

    #[test]
    fn latest_picks_the_highest_version() {
        let reg = scratch_registry("latest");
        let (spore, descriptor) = demo_spore(
            "l",
            "// nodule: geo.shapes\nnodule geo.shapes\nfn a() -> Binary{8} = 0b0\n",
        );
        // Same artifact, multiple version labels (metadata is not identity — ADR-003).
        for v in ["1.0.0", "1.2.0", "1.10.0", "2.0.0"] {
            publish(&reg, &spore, &descriptor, "geo", v).unwrap();
        }
        // Numeric component ordering: 1.10.0 > 1.2.0, and 2.0.0 is the max.
        assert_eq!(resolve(&reg, "geo", "latest").unwrap().version, "2.0.0");
    }
}
