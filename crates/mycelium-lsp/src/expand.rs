//! **Expand ambient** (M-344; RFC-0012 §5): the toolchain projection that renders a document's
//! resolved *longhand* form on demand — the answer to "what does this paradigm-less `{…}` /
//! `default paradigm` actually mean here?". Because the ambient is pure surface elaboration
//! (RFC-0012 I2), the expanded form is the program a reader would write by hand, and it elaborates
//! to the identical L0 (identical content hash). This is the "expand ambient" the editor surfaces
//! so the elided default is never *hidden*, only *elided* (§5).
//!
//! Width resolution needs the checker, so full expansion runs the parse → resolve → check pipeline
//! ([`mycelium_l1::check_and_resolve`]) and pretty-prints the checker-resolved twin
//! ([`mycelium_l1::expand_to_source`]); a parse/check failure is reported, never a partial render.

use mycelium_l1::{check_and_resolve, expand_to_source, parse};

/// Render `text`'s fully-resolved longhand twin (paradigm tags filled, `with paradigm` blocks
/// stripped, bare-decimal widths resolved from context).
///
/// # Errors
/// Returns the parse/check diagnostic message if the document does not parse or check (so the
/// expansion is never a partial or guessed artifact — G2/never-silent).
pub fn expand_ambient(text: &str) -> Result<String, String> {
    let colony = parse(text).map_err(|e| e.to_string())?;
    let (_, twin) = check_and_resolve(&colony).map_err(|e| e.to_string())?;
    Ok(expand_to_source(&twin))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_a_binary_ambient_to_longhand() {
        let src = "colony d\ndefault paradigm Binary\nfn main() -> {8} = not(0b1011_0010)";
        let out = expand_ambient(src).expect("expands");
        assert!(out.contains("Binary{8}"), "{out}");
        assert!(!out.contains("default paradigm"), "{out}");
    }

    #[test]
    fn a_check_failure_is_reported_not_partially_rendered() {
        // A paradigm-less repr with no ambient cannot be expanded — it is an explicit diagnostic.
        let err = expand_ambient("colony d\nfn main() -> {8} = 0b1011_0010").unwrap_err();
        assert!(err.contains("no enclosing ambient"), "{err}");
    }
}
