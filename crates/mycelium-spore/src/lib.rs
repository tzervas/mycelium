//! `mycelium-spore` — **`spore`**, packaging & publishing (M-368; ADR-013).
//!
//! Builds a **content-addressed `spore`** from a `mycelium-proj.toml` project — the deployable unit that
//! germinates into a colony (DN-06/Glossary). The load-bearing rule is ADR-003: **identity is the
//! content-addressed DAG** (the source code by hash + the resolved dependency edges + the germination
//! surface); **metadata is not identity** (`version`/`authors`/`summary`/… travel with the spore but never
//! define it). Two builds of the same code+deps produce the **same spore hash** regardless of the version
//! label. A missing or ambiguous publish input is an **explicit error**, never a guess (G2): a phylum with
//! no surface, a project with no sources, or a dependency with no `hash` is refused — no partial artifact.
//!
//! v0 scope (honest; contract §7): a **single project** with **hash-pinned** dependencies; the on-disk
//! encoding is a **named-provisional** reproducible form (M-368 §9.1), superseded append-only when the
//! RFC-0008 R2 wire-schema lands (the signing + germination contract are deferred there per ADR-013 §4).
//! Source code is content-addressed by **raw-byte BLAKE3**; canonicalized (mycfmt) hashing is a later
//! refinement. KC-3: above the kernel.

use std::path::{Path, PathBuf};

use mycelium_core::ContentHash;
use mycelium_proj::{Manifest, ProjectKind};

/// A project source file, content-addressed (raw-byte BLAKE3; ADR-003).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    /// Path relative to the project root (forward-slashed, deterministic).
    pub path: String,
    /// `blake3:<hex>` of the file bytes.
    pub hash: ContentHash,
}

/// A resolved dependency edge — pinned by content hash (authoritative, ADR-003).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedDep {
    /// The dependency's local name.
    pub name: String,
    /// The depended-on phylum.
    pub phylum: String,
    /// The content-address pin (`blake3:…`).
    pub hash: String,
    /// The human version requirement (metadata; not identity).
    pub version: Option<String>,
}

/// A built spore: its content-addressed identity plus the components that define it and the metadata that
/// travels with (but does not define) it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spore {
    /// The spore identity — `blake3` over the canonical DAG (code + deps + surface), **excluding**
    /// metadata (ADR-003).
    pub id: ContentHash,
    /// The project shape.
    pub kind: ProjectKind,
    /// The germination surface (sorted public export names).
    pub surface: Vec<String>,
    /// The content-addressed source files (sorted by path).
    pub sources: Vec<SourceFile>,
    /// The resolved dependency edges (sorted by name).
    pub deps: Vec<ResolvedDep>,
    /// The project name (metadata — carried, not identity).
    pub name: String,
    /// The project version, if any (metadata — carried, not identity).
    pub version: Option<String>,
}

/// A spore-build refusal — never a partial artifact (G2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SporeError {
    /// A missing/ambiguous publish input (no surface, no sources, a hashless dep, a bad include) (exit 3).
    Publish(String),
    /// An I/O error reading the project (exit 66).
    Io(String),
}

impl SporeError {
    /// The CLI exit code for this refusal.
    #[must_use]
    pub fn exit_code(&self) -> u8 {
        match self {
            SporeError::Publish(_) => 3,
            SporeError::Io(_) => 66,
        }
    }
}

impl std::fmt::Display for SporeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SporeError::Publish(m) => write!(f, "publish-error: {m}"),
            SporeError::Io(m) => write!(f, "io-error: {m}"),
        }
    }
}

impl std::error::Error for SporeError {}

/// Build a [`Spore`] from a parsed manifest and the project root directory.
///
/// # Errors
/// [`SporeError::Publish`] when a publish input is missing/ambiguous (no germination surface for a phylum,
/// no `.myc` sources, a dependency without a `hash`, or an `[spore].include` naming a non-exported nodule),
/// or [`SporeError::Io`] on a read failure. No partial artifact is produced (G2).
pub fn build_spore(manifest: &Manifest, project_dir: &Path) -> Result<Spore, SporeError> {
    let kind = manifest.project.kind;

    // 1. The germination surface: `[spore].include` (default `["surface"]`) resolved against
    //    `[surface].exports`. A `phylum` must expose a non-empty surface — nothing to germinate is an error.
    let exports = manifest
        .surface
        .as_ref()
        .map(|s| s.exports.clone())
        .unwrap_or_default();
    let include = manifest
        .spore
        .as_ref()
        .map(|s| s.include.clone())
        .filter(|i| !i.is_empty())
        .unwrap_or_else(|| vec!["surface".to_owned()]);

    let mut surface: Vec<String> = Vec::new();
    for entry in &include {
        if entry == "surface" {
            surface.extend(exports.iter().cloned());
        } else {
            // An explicit include must name a declared export (when a surface is declared) — a typo'd
            // surface would ship the wrong thing (G2).
            if manifest.surface.is_some() && !exports.contains(entry) {
                return Err(SporeError::Publish(format!(
                    "[spore].include names `{entry}`, which is not in [surface].exports — refusing to \
                     guess the germination surface (G2)"
                )));
            }
            surface.push(entry.clone());
        }
    }
    surface.sort();
    surface.dedup();
    if kind == ProjectKind::Phylum && surface.is_empty() {
        return Err(SporeError::Publish(
            "a phylum must declare its public [surface].exports (or [spore].include) — there is nothing \
             to germinate; the surface is never guessed (G2)"
                .to_owned(),
        ));
    }

    // 2. The code: every `.myc` source, content-addressed by raw-byte BLAKE3 (sorted, deterministic).
    let mut sources = collect_sources(project_dir)?;
    sources.sort_by(|a, b| a.path.cmp(&b.path));
    if sources.is_empty() {
        return Err(SporeError::Publish(format!(
            "no `.myc` sources under {} — nothing to package",
            project_dir.display()
        )));
    }

    // 3. The dependency edges: each pinned by `hash` (authoritative, ADR-003); a hashless dep is refused.
    let mut deps = Vec::with_capacity(manifest.dependencies.len());
    for d in &manifest.dependencies {
        let hash = d.hash.clone().ok_or_else(|| {
            SporeError::Publish(format!(
                "dependency `{}` has no `hash` — an unpinned dependency is not reproducible; pin it \
                 (`hash = \"blake3:…\"`, ADR-003/G2)",
                d.name
            ))
        })?;
        deps.push(ResolvedDep {
            name: d.name.clone(),
            phylum: d.phylum.clone(),
            hash,
            version: d.version.clone(),
        });
    }
    deps.sort_by(|a, b| a.name.cmp(&b.name));

    // 4. Content-address the DAG (code + deps + surface + kind) — metadata excluded (ADR-003).
    let id = content_address(kind, &surface, &sources, &deps);

    Ok(Spore {
        id,
        kind,
        surface,
        sources,
        deps,
        name: manifest.project.name.clone(),
        version: manifest.project.version.clone(),
    })
}

/// The canonical, deterministic identity encoding (ADR-003). Metadata (`name`/`version`/`authors`/…) is
/// **excluded** — only the code-by-hash DAG, the dependency hash edges, the germination surface, and the
/// project kind bear identity. Two builds of the same code+deps yield the same spore hash.
fn content_address(
    kind: ProjectKind,
    surface: &[String],
    sources: &[SourceFile],
    deps: &[ResolvedDep],
) -> ContentHash {
    let mut s = String::from("mycelium-spore-v0\n");
    s.push_str(&format!("kind:{}\n", kind_str(kind)));
    s.push_str("surface:\n");
    for name in surface {
        s.push_str(&format!("  {name}\n"));
    }
    s.push_str("code:\n");
    for f in sources {
        s.push_str(&format!("  {} {}\n", f.path, f.hash.as_str()));
    }
    s.push_str("deps:\n");
    for d in deps {
        // The hash is identity; the version requirement is metadata and is excluded here.
        s.push_str(&format!("  {} {} {}\n", d.name, d.phylum, d.hash));
    }
    let hex = blake3::hash(s.as_bytes()).to_hex();
    ContentHash::from_parts("blake3", hex.as_str()).expect("blake3 hex is a valid digest")
}

/// The canonical `[project].kind` spelling.
#[must_use]
pub fn kind_str(kind: ProjectKind) -> &'static str {
    match kind {
        ProjectKind::Phylum => "phylum",
        ProjectKind::Program => "program",
        ProjectKind::Script => "script",
    }
}

/// Collect every `.myc` source under `dir` (recursively), content-addressed by raw-byte BLAKE3. Skips
/// hidden entries, `target/`, and the temp files a formatter might leave — deterministic and reproducible.
fn collect_sources(dir: &Path) -> Result<Vec<SourceFile>, SporeError> {
    let mut out = Vec::new();
    walk(dir, dir, &mut out)?;
    Ok(out)
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<SourceFile>) -> Result<(), SporeError> {
    let entries =
        std::fs::read_dir(dir).map_err(|e| SporeError::Io(format!("{}: {e}", dir.display())))?;
    let mut paths: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    paths.sort();
    for path in paths {
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        if name.starts_with('.') || name == "target" {
            continue;
        }
        if path.is_dir() {
            walk(root, &path, out)?;
        } else if path.extension().is_some_and(|x| x == "myc") {
            let bytes = std::fs::read(&path)
                .map_err(|e| SporeError::Io(format!("{}: {e}", path.display())))?;
            let hex = blake3::hash(&bytes).to_hex();
            let hash = ContentHash::from_parts("blake3", hex.as_str())
                .expect("blake3 hex is a valid digest");
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            out.push(SourceFile { path: rel, hash });
        }
    }
    Ok(())
}

/// The `EXPLAIN` of a built spore (no black box): the identity receipt, the surface, the code by hash, the
/// dependency edges, and the metadata — the metadata explicitly marked *not* identity (ADR-003).
#[must_use]
pub fn explain(spore: &Spore) -> String {
    let mut out = format!("spore: {}  →  {}\n", spore.name, spore.id.as_str());
    out.push_str(&format!("  kind:    {}\n", kind_str(spore.kind)));
    out.push_str(&format!("  surface: {}\n", spore.surface.join(", ")));
    out.push_str(&format!(
        "  code:    {} source file(s)\n",
        spore.sources.len()
    ));
    for f in &spore.sources {
        out.push_str(&format!("    {} {}\n", f.path, f.hash.as_str()));
    }
    out.push_str(&format!("  deps:    {}\n", spore.deps.len()));
    for d in &spore.deps {
        let v = d.version.as_deref().unwrap_or("*");
        out.push_str(&format!(
            "    {} → {} {} (version {v})\n",
            d.name, d.phylum, d.hash
        ));
    }
    let ver = spore.version.as_deref().unwrap_or("—");
    out.push_str(&format!(
        "  metadata: name={}, version={ver}  [not identity — ADR-003]\n",
        spore.name
    ));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use mycelium_proj::parse_manifest;
    use std::io::Write;

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

    fn manifest(src: &str) -> Manifest {
        parse_manifest(src).unwrap()
    }

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
        let s1 = build_spore(&manifest(m_v1), &dir).expect("builds");
        assert!(s1.id.as_str().starts_with("blake3:"));
        assert_eq!(s1.surface, vec!["geometry.shapes".to_owned()]);
        assert_eq!(s1.sources.len(), 1);

        // ADR-003: changing only metadata (version) leaves the spore identity unchanged.
        let m_v2 = m_v1.replace("1.0.0", "2.5.0");
        let s2 = build_spore(&manifest(&m_v2), &dir).expect("builds");
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
        let s3 = build_spore(&manifest(m_v1), &dir).expect("builds");
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
        let err = build_spore(&manifest(m), &dir).unwrap_err();
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
        let err = build_spore(&manifest(m), &dir).unwrap_err();
        assert_eq!(err.exit_code(), 3);
        assert!(format!("{err}").contains("no `hash`"), "{err}");
    }

    #[test]
    fn a_project_with_no_sources_is_refused() {
        let m = "[project]\nname=\"x\"\nkind=\"program\"\n";
        let dir = scratch("nosrc", m, &[]);
        let err = build_spore(&manifest(m), &dir).unwrap_err();
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
        let s = build_spore(&manifest(m), &dir).expect("builds");
        assert_eq!(s.deps.len(), 1);
        assert_eq!(s.deps[0].hash, "blake3:abc");
        let ex = explain(&s);
        assert!(ex.contains("not identity — ADR-003"), "{ex}");
        assert!(ex.contains("numerics → numerics blake3:abc"), "{ex}");
    }
}
