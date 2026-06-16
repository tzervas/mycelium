//! The **nodule header marker** (DN-06 §6) — the minimal, in-file declaration that a source file
//! *is* a nodule (Mycelium's static organizational unit, replacing the generic "module").
//!
//! A file declares its nodule status with a comment on its **first non-blank line**:
//!
//! ```text
//! // nodule: geometry.shapes      // a named nodule (its logical path within a phylum)
//! // nodule                       // a bare marker (e.g. a subnodule that inherits its name)
//! ```
//!
//! This marker is a **source-text** concern, not a grammar one: comments are lexer trivia (they
//! never reach the AST), and the marker is **never** part of a definition's content-addressed
//! identity — metadata is not identity (ADR-003). The recogniser is deliberately small and total:
//! it returns the parsed marker, `None` when the first non-blank line is an ordinary comment (or
//! code), or an **explicit** [`NoduleHeaderError`] when the author clearly intended a *named* marker
//! (`// nodule:`) but wrote an ill-formed name — a near-miss is flagged, never silently dropped (G2).
//!
//! The richer **structured** header (`// @key: value` — license/authors/version/…) and the
//! `mycelium-proj.toml` manifest layer on top of this marker; they are designed in
//! `docs/spec/Nodule-Header-and-Project-Manifest.md` and tracked separately (M-359). This module is
//! only the DN-06 §6 floor that M-358 wires into the linter (M-141) and formatter (M-142).

/// A recognised nodule header marker (DN-06 §6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoduleHeader {
    /// The declared logical path (dotted name) within the enclosing phylum, or `None` for the bare
    /// `// nodule` marker (the name is supplied by inheritance / the file's conventional location).
    pub name: Option<Vec<String>>,
}

impl NoduleHeader {
    /// The dotted name as written (`"geometry.shapes"`), or `None` for the bare marker.
    #[must_use]
    pub fn dotted(&self) -> Option<String> {
        self.name.as_ref().map(|segs| segs.join("."))
    }

    /// The canonical one-line spelling of this marker — what the formatter (M-142) emits.
    #[must_use]
    pub fn canonical(&self) -> String {
        match self.dotted() {
            Some(name) => format!("// nodule: {name}"),
            None => "// nodule".to_owned(),
        }
    }
}

/// An ill-formed nodule header marker — never-silent (G2): the author wrote `// nodule:` but the
/// name after it is empty or not a dotted identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoduleHeaderError {
    /// 1-based line of the offending marker (the first non-blank line).
    pub line: u32,
    /// What is wrong, in author-facing terms.
    pub message: String,
}

impl std::fmt::Display for NoduleHeaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for NoduleHeaderError {}

/// Recognise the optional nodule header marker on the first non-blank line of `src`.
///
/// - `Ok(Some(h))` — a well-formed `// nodule` / `// nodule: <dotted.name>` marker.
/// - `Ok(None)` — the first non-blank line is not a nodule marker (ordinary comment, or code).
/// - `Err(_)` — a *named* marker (`// nodule:`) with an empty or ill-formed name (G2).
///
/// # Errors
/// Returns [`NoduleHeaderError`] when a `// nodule:` marker names nothing, or names a non-dotted
/// identifier (e.g. `// nodule: 9bad`, `// nodule: a..b`).
pub fn parse_nodule_header(src: &str) -> Result<Option<NoduleHeader>, NoduleHeaderError> {
    // Find the first non-blank line (1-based line number tracked for diagnostics).
    let mut line_no = 0u32;
    for raw in src.lines() {
        line_no += 1;
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        return recognise(line, line_no);
    }
    Ok(None)
}

fn recognise(line: &str, line_no: u32) -> Result<Option<NoduleHeader>, NoduleHeaderError> {
    // Must be a line comment to be a marker at all.
    let Some(rest) = line.strip_prefix("//") else {
        return Ok(None);
    };
    let body = rest.trim();
    // Bare marker: exactly `nodule`.
    if body == "nodule" {
        return Ok(Some(NoduleHeader { name: None }));
    }
    // Named marker: `nodule:` followed by a dotted name. Anything else is an ordinary comment.
    let Some(after) = body.strip_prefix("nodule:") else {
        return Ok(None);
    };
    let name = after.trim();
    if name.is_empty() {
        return Err(NoduleHeaderError {
            line: line_no,
            message: "a `// nodule:` marker must name the nodule (its dotted path), e.g. \
                      `// nodule: geometry.shapes`; for an unnamed nodule use the bare `// nodule`"
                .to_owned(),
        });
    }
    let segments = parse_dotted(name, line_no)?;
    Ok(Some(NoduleHeader {
        name: Some(segments),
    }))
}

/// Validate a dotted name (`a.b.c`): non-empty segments, each a valid identifier.
fn parse_dotted(name: &str, line_no: u32) -> Result<Vec<String>, NoduleHeaderError> {
    let mut segments = Vec::new();
    for seg in name.split('.') {
        if seg.is_empty() {
            return Err(NoduleHeaderError {
                line: line_no,
                message: format!(
                    "nodule name {name:?} has an empty path segment (no leading/trailing/`..` dots)"
                ),
            });
        }
        if !is_ident(seg) {
            return Err(NoduleHeaderError {
                line: line_no,
                message: format!(
                    "nodule name segment {seg:?} is not a valid identifier \
                     (letters, digits, `_`; not starting with a digit)"
                ),
            });
        }
        segments.push(seg.to_owned());
    }
    Ok(segments)
}

fn is_ident(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_marker_is_recognised() {
        let h = parse_nodule_header("// nodule: geometry.shapes\nnodule geometry.shapes\n")
            .unwrap()
            .unwrap();
        assert_eq!(h.dotted().as_deref(), Some("geometry.shapes"));
        assert_eq!(h.canonical(), "// nodule: geometry.shapes");
    }

    #[test]
    fn bare_marker_is_recognised() {
        let h = parse_nodule_header("// nodule\nnodule g.s\n")
            .unwrap()
            .unwrap();
        assert_eq!(h.name, None);
        assert_eq!(h.canonical(), "// nodule");
    }

    #[test]
    fn leading_blank_lines_are_skipped() {
        let h = parse_nodule_header("\n\n   \n// nodule: a.b\n")
            .unwrap()
            .unwrap();
        assert_eq!(h.dotted().as_deref(), Some("a.b"));
    }

    #[test]
    fn an_ordinary_first_comment_is_not_a_marker() {
        assert_eq!(
            parse_nodule_header("// just a comment\nnodule d\n").unwrap(),
            None
        );
        // `nodule` mentioned in prose (no colon, not bare) is not a marker — no false positive.
        assert_eq!(
            parse_nodule_header("// nodule is Mycelium's word for module\nnodule d\n").unwrap(),
            None
        );
    }

    #[test]
    fn code_first_means_no_marker() {
        assert_eq!(
            parse_nodule_header("nodule d\nfn f() -> Binary{8} = 0b0").unwrap(),
            None
        );
    }

    #[test]
    fn empty_named_marker_is_an_explicit_error() {
        let e = parse_nodule_header("// nodule:\n").unwrap_err();
        assert_eq!(e.line, 1);
        assert!(e.message.contains("must name the nodule"), "{}", e.message);
    }

    #[test]
    fn ill_formed_name_is_an_explicit_error() {
        assert!(parse_nodule_header("// nodule: 9bad\n").is_err());
        assert!(parse_nodule_header("// nodule: a..b\n").is_err());
        assert!(parse_nodule_header("// nodule: a.b.\n").is_err());
        assert!(parse_nodule_header("// nodule: has space\n").is_err());
    }

    #[test]
    fn empty_source_has_no_marker() {
        assert_eq!(parse_nodule_header("").unwrap(), None);
        assert_eq!(parse_nodule_header("\n  \n").unwrap(), None);
    }
}
