//! Load a previously emitted `index.json` back into a [`TeroIndexReport`] — the read-side twin of
//! [`crate::emit::write_json`] (M-1016). The query engine (`crate::query`) is deliberately built to
//! run over the *persisted* Layer-1 artifact rather than re-walking + re-parsing the whole corpus on
//! every query: `docs/tero-index/index.json` is already the deterministic, committed snapshot
//! (M-1015 / DN-87 §6.3); "generate once, query many" is the same split `docs/api-index/` and
//! `docs/lib-index/` were built for.
//!
//! Honesty (G2): a malformed/truncated `index.json` is a hard I/O/deserialization error, never a
//! silent partial load — the caller sees exactly what failed and where.

use std::path::Path;

use serde::Deserialize;

use crate::model::{Flagged, TeroIndexItem, TeroIndexReport};

/// The subset of `index.json`'s top-level shape this reads back. Mirrors
/// [`crate::emit::write_json`]'s `Payload`; `generated`/`item_tag`/`siblings` are the crate's own
/// constants ([`crate::model::HONESTY_TAG`] / [`crate::model::ITEM_TAG`] / [`crate::model::SIBLING_INDICES`]),
/// not round-tripped — an unknown top-level field is ignored by `serde_json`, not an error.
#[derive(Deserialize)]
struct Payload {
    items: Vec<TeroIndexItem>,
    #[serde(default)]
    flagged: Vec<Flagged>,
}

/// Load a `TeroIndexReport` from a previously emitted `index.json` at `path` (typically
/// `docs/tero-index/index.json`). The loaded report preserves the file's row order, which — for
/// every `index.json` this crate itself emits — is already the canonical `(family, file, line,
/// anchor)` sort [`TeroIndexReport::sort`] establishes; [`crate::query::QueryEngine`] relies on that
/// invariant (checked in debug builds) rather than re-sorting on every load.
///
/// # Errors
/// Any filesystem error reading `path`, or a JSON shape that does not match
/// [`TeroIndexItem`]/[`Flagged`] — reported via [`std::io::ErrorKind::InvalidData`], never a silent
/// partial load.
pub fn load_report(path: &Path) -> std::io::Result<TeroIndexReport> {
    let src = std::fs::read_to_string(path)?;
    let payload: Payload = serde_json::from_str(&src)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(TeroIndexReport {
        items: payload.items,
        flagged: payload.flagged,
    })
}
