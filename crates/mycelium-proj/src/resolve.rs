//! **Top-down inheritance resolution** (M-359; spec §4) with per-field provenance and an `EXPLAIN`.
//!
//! The *effective* header of a file is resolved most-specific-first — `in-file @key` > the
//! `mycelium-proj.toml` `[project]` table — and is **always inspectable** (no black box, G2): every
//! resolved field carries its [`Origin`], so "where did this license come from?" is answerable by
//! [`explain`]. Inherited fields (`version`/`license`/`authors`/`since`/`repository`/`keywords`) fall
//! back to the manifest; per-file fields (`updated`/`summary`/`deprecated`) never inherit. A local
//! value **overrides** the manifest (local wins) — that is an allowed override, not a conflict
//! (spec §4). Resolution produces *associated metadata* only — the content hash is unaffected
//! (ADR-003).
//!
//! Note (honest scope): the spec's middle tier — a nearest-ancestor *nodule-root* header — is a
//! multi-file concern; v0 resolves the single-file (`in-file > manifest`) case and names the
//! ancestor tier as deferred. Disallowed cross-tier conflicts (e.g. license-incompatible overrides)
//! are likewise a later compliance check (M-361), not fabricated here.

use crate::header::{Deprecated, HeaderFields, StructuredHeader};
use crate::manifest::Manifest;

/// Where a resolved field's value came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    /// Set by an in-file `// @key:` line.
    Local,
    /// Inherited from the `mycelium-proj.toml` `[project]` table.
    ProjectManifest,
}

impl Origin {
    fn label(self) -> &'static str {
        match self {
            Origin::Local => "local",
            Origin::ProjectManifest => "mycelium-proj.toml",
        }
    }
}

/// A resolved field: its effective value and where it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolved<T> {
    /// The effective value.
    pub value: T,
    /// Its provenance.
    pub origin: Origin,
}

/// The fully-resolved header — each inherited field annotated with its [`Origin`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResolvedHeader {
    /// The nodule's dotted name (from the marker; `None` for a bare marker).
    pub name: Option<Vec<String>>,
    /// Effective `version`.
    pub version: Option<Resolved<String>>,
    /// Effective `license`.
    pub license: Option<Resolved<String>>,
    /// Effective `authors`.
    pub authors: Option<Resolved<Vec<String>>>,
    /// Effective `since`.
    pub since: Option<Resolved<String>>,
    /// Effective `repository`.
    pub repository: Option<Resolved<String>>,
    /// Effective `keywords`.
    pub keywords: Option<Resolved<Vec<String>>>,
    /// `updated` — per-file (always local; never inherited).
    pub updated: Option<String>,
    /// `summary` — per-file.
    pub summary: Option<String>,
    /// `deprecated` — per-file.
    pub deprecated: Option<Deprecated>,
    /// `matured` — the nodule/phylum is a matured (AOT) scope; RFC-0017; inherited top-down.
    pub matured: Option<Resolved<bool>>,
}

/// Resolve a parsed header against an optional project manifest.
#[must_use]
pub fn resolve(header: &StructuredHeader, manifest: Option<&Manifest>) -> ResolvedHeader {
    let f: &HeaderFields = &header.fields;
    let p = manifest.map(|m| &m.project);

    // Inherited string field: local > manifest.
    let inherit_str = |local: &Option<String>, from_manifest: Option<&String>| {
        if let Some(v) = local {
            Some(Resolved {
                value: v.clone(),
                origin: Origin::Local,
            })
        } else {
            from_manifest.map(|v| Resolved {
                value: v.clone(),
                origin: Origin::ProjectManifest,
            })
        }
    };
    let inherit_list = |local: &Option<Vec<String>>, from_manifest: Option<&Vec<String>>| {
        if let Some(v) = local {
            Some(Resolved {
                value: v.clone(),
                origin: Origin::Local,
            })
        } else {
            from_manifest.map(|v| Resolved {
                value: v.clone(),
                origin: Origin::ProjectManifest,
            })
        }
    };
    let inherit_bool = |local: &Option<bool>, from_manifest: Option<bool>| {
        if let Some(v) = local {
            Some(Resolved {
                value: *v,
                origin: Origin::Local,
            })
        } else {
            from_manifest.map(|v| Resolved {
                value: v,
                origin: Origin::ProjectManifest,
            })
        }
    };

    ResolvedHeader {
        name: header.marker.name.clone(),
        version: inherit_str(&f.version, p.and_then(|p| p.version.as_ref())),
        license: inherit_str(&f.license, p.and_then(|p| p.license.as_ref())),
        authors: inherit_list(&f.authors, p.and_then(|p| p.authors.as_ref())),
        since: inherit_str(&f.since, p.and_then(|p| p.since.as_ref())),
        repository: inherit_str(&f.repository, p.and_then(|p| p.repository.as_ref())),
        keywords: inherit_list(&f.keywords, p.and_then(|p| p.keywords.as_ref())),
        // `matured` (RFC-0017) is *specified* to inherit top-down, but in this single-file resolver
        // both inheritance tiers are deferred: manifest-level `[project].matured` is not yet enacted
        // (R17-Q1) and multi-file ancestor-nodule resolution is out of scope here (see the module
        // header, §"deferred"). So `@matured` resolves **local-only** for now — the manifest source is
        // `None` and there is no ancestor tier; the inherited tiers land with those enactments.
        matured: inherit_bool(&f.matured, None),
        // Per-file: never inherited.
        updated: f.updated.clone(),
        summary: f.summary.clone(),
        deprecated: f.deprecated.clone(),
    }
}

/// The `EXPLAIN` of a resolved header — every field with its value and source, so nothing about the
/// metadata is ambient (G2). Stable, line-oriented, deterministic.
#[must_use]
pub fn explain(r: &ResolvedHeader) -> String {
    let mut out = String::new();
    let name = r
        .name
        .as_ref()
        .map_or_else(|| "(bare nodule)".to_owned(), |segs| segs.join("."));
    out.push_str(&format!("nodule: {name}\n"));

    let mut row = |field: &str, value: Option<String>, origin: Option<Origin>| match value {
        Some(v) => out.push_str(&format!(
            "  {field}: {v}  [{}]\n",
            origin.map_or("local", Origin::label)
        )),
        None => out.push_str(&format!("  {field}: —  [unset]\n")),
    };

    row(
        "version",
        r.version.as_ref().map(|x| x.value.clone()),
        r.version.as_ref().map(|x| x.origin),
    );
    row(
        "license",
        r.license.as_ref().map(|x| x.value.clone()),
        r.license.as_ref().map(|x| x.origin),
    );
    row(
        "authors",
        r.authors.as_ref().map(|x| x.value.join(", ")),
        r.authors.as_ref().map(|x| x.origin),
    );
    row(
        "since",
        r.since.as_ref().map(|x| x.value.clone()),
        r.since.as_ref().map(|x| x.origin),
    );
    row(
        "repository",
        r.repository.as_ref().map(|x| x.value.clone()),
        r.repository.as_ref().map(|x| x.origin),
    );
    row(
        "keywords",
        r.keywords.as_ref().map(|x| x.value.join(", ")),
        r.keywords.as_ref().map(|x| x.origin),
    );
    // Per-file fields are always local when present.
    row(
        "updated",
        r.updated.clone(),
        r.updated.as_ref().map(|_| Origin::Local),
    );
    row(
        "summary",
        r.summary.clone(),
        r.summary.as_ref().map(|_| Origin::Local),
    );
    let dep = r.deprecated.as_ref().map(|d| match d {
        Deprecated::Flag(b) => b.to_string(),
        Deprecated::Reason(s) => s.clone(),
    });
    row(
        "deprecated",
        dep,
        r.deprecated.as_ref().map(|_| Origin::Local),
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::header::parse_header;
    use crate::manifest::parse_manifest;

    fn manifest() -> Manifest {
        parse_manifest(
            "[project]\nname=\"geometry\"\nkind=\"phylum\"\nversion=\"1.2.0\"\n\
             license=\"Apache-2.0\"\nauthors=[\"Tyler Zervas\"]\nsince=\"2026-01-10\"\n\
             repository=\"https://github.com/example/geometry\"\nkeywords=[\"geometry\"]\n",
        )
        .unwrap()
    }

    #[test]
    fn a_subnodule_inherits_from_the_manifest() {
        let h = parse_header("// nodule: geometry.shapes.circle\n// @updated: 2026-06-16\n")
            .unwrap()
            .unwrap();
        let r = resolve(&h, Some(&manifest()));
        // Inherited from the manifest.
        assert_eq!(r.license.as_ref().unwrap().value, "Apache-2.0");
        assert_eq!(r.license.as_ref().unwrap().origin, Origin::ProjectManifest);
        assert_eq!(r.version.as_ref().unwrap().origin, Origin::ProjectManifest);
        // Per-file: local, never inherited.
        assert_eq!(r.updated.as_deref(), Some("2026-06-16"));
    }

    #[test]
    fn a_local_value_overrides_the_manifest() {
        let h = parse_header("// nodule: geometry.shapes\n// @license: MIT\n")
            .unwrap()
            .unwrap();
        let r = resolve(&h, Some(&manifest()));
        assert_eq!(r.license.as_ref().unwrap().value, "MIT");
        assert_eq!(r.license.as_ref().unwrap().origin, Origin::Local);
    }

    #[test]
    fn explain_names_every_source() {
        let h =
            parse_header("// nodule: geometry.shapes\n// @license: MIT\n// @updated: 2026-06-16\n")
                .unwrap()
                .unwrap();
        let r = resolve(&h, Some(&manifest()));
        let ex = explain(&r);
        assert!(ex.contains("license: MIT  [local]"), "{ex}");
        assert!(ex.contains("version: 1.2.0  [mycelium-proj.toml]"), "{ex}");
        assert!(ex.contains("updated: 2026-06-16  [local]"), "{ex}");
    }

    #[test]
    fn no_manifest_means_only_local_fields_resolve() {
        let h = parse_header("// nodule: solo\n// @license: MIT\n")
            .unwrap()
            .unwrap();
        let r = resolve(&h, None);
        assert_eq!(r.license.as_ref().unwrap().origin, Origin::Local);
        assert!(r.version.is_none());
    }
}
