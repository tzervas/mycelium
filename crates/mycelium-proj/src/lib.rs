//! `mycelium-proj` — the **project-metadata layer** (M-359; DN-06 §6; the
//! `docs/spec/Nodule-Header-and-Project-Manifest.md` schema, Accepted 2026-06-16).
//!
//! Three pieces, all *above* the kernel (KC-3 — nothing in the trusted base depends on this):
//!
//! - [`header`] — the **structured nodule header**: the `// @key: value` metadata lines (the closed
//!   v0 key set) that may follow the `// nodule:` marker (M-358). Unknown/duplicate keys and
//!   malformed values are explicit errors (G2/VR-5).
//! - [`manifest`] — the **`mycelium-proj.toml` manifest**, read by a deliberately **minimal,
//!   no-new-dependency TOML-subset** reader (the workspace keeps its deps few/vetted; **adding** a
//!   full TOML crate would be an ADR, not a build detail). It is honestly a subset, named as one.
//! - [`mod@resolve`] — **top-down inheritance** (`in-file > manifest`) with per-field provenance and an
//!   `EXPLAIN`, so a field's effective value and *source* are never ambient (G2).
//!
//! **Metadata is not identity (ADR-003).** Nothing here perturbs a definition's content hash — these
//! are associated, queryable fields, the human/release layer on top of content-addressing.

pub mod header;
pub mod manifest;
pub mod resolve;

pub use header::{
    parse_header, Deprecated, HeaderError, HeaderFields, StructuredHeader, HEADER_KEYS,
};
pub use manifest::{
    parse_manifest, Dependency, Manifest, ManifestError, Project, ProjectKind, SporeConfig,
    Surface, Toolchain,
};
pub use resolve::{explain, resolve, Origin, Resolved, ResolvedHeader};
