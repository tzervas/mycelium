//! `mycelium-fmt` — **`mycfmt`**, the canonical formatter (M-364; M-142 grows up).
//!
//! Formatting is a **projection**: it rewrites a `.myc` source into one canonical textual normal form
//! and **never changes a definition's content-addressed identity** (RFC-0001 §4.6/§4.8; ADR-003). The
//! contract is `docs/spec/Mycfmt-Formatter-Contract.md`; this crate enacts it. Three invariants hold,
//! the first by a **runtime guard** (so an identity-changing format is never emitted) and all three by
//! test (`tests/`):
//!
//! - **C1 identity-preservation.** The formatted text re-parses to the **same surface AST** as the input
//!   (`parse(out) == parse(src)`), and its header re-parses equal (`parse_header`). Equivalent to
//!   content-hash preservation on the elaborable fragment (ADR-003). Checked at runtime: a mismatch is a
//!   refusal ([`FmtError::OutOfScope`]), never an emitted rewrite.
//! - **C2 idempotence.** `format(format(s)) == format(s)` byte-for-byte (the canonical form is a fixed
//!   point). Tested.
//! - **C3 header-preservation.** The DN-06 `// nodule:` marker + the M-359 `// @key:` structured header
//!   are re-emitted canonically (§4 order); a malformed header is an explicit [`FmtError::Header`], never
//!   a silent drop (G2/VR-5).
//!
//! **Never-silent (G2).** Unparsable input, a malformed header, or a construct outside the round-trip-safe
//! v0 scope (§7 — interior comments; an expression that does not round-trip) is an **explicit error** with
//! an exit code; `mycfmt` **never** writes a partial or garbled rewrite. The load-bearing subtlety: the
//! body is printed from the **raw parse** ([`mycelium_l1::parse`]), *not* the ambient-resolved twin — so
//! `default paradigm` / `with paradigm` are **preserved**, not expanded (formatting ≠ "expand ambient").
//!
//! KC-3: this lives entirely above the kernel; the trusted base depends on nothing here.

use mycelium_l1::{expand_to_source, parse};
use mycelium_proj::{parse_header, Deprecated, HeaderFields, StructuredHeader};

/// The formatter spelling/version this build implements. The `[toolchain].format` pin (M-359) is a
/// **hard pin** (M-364 §10.3): a mismatch is refused, never formatted with rules the project didn't ask
/// for (G2).
pub const MYCFMT_VERSION: &str = "mycfmt-0";

/// A successful format result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Formatted {
    /// The canonical output (always ends with exactly one newline).
    pub output: String,
    /// Whether the output differs from the input (drives `--check`).
    pub changed: bool,
    /// The normalizations applied, named for `EXPLAIN` (no black box).
    pub notes: Vec<String>,
}

/// A formatting refusal — never a partial rewrite (G2). Each maps to a CLI exit code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FmtError {
    /// Input is not a valid `.myc` program (exit 2).
    Parse(String),
    /// A malformed `// nodule:` / `// @key:` header (exit 3).
    Header(String),
    /// A construct outside the round-trip-safe v0 scope, or a `[toolchain].format` pin mismatch (exit 4).
    OutOfScope(String),
}

impl FmtError {
    /// The CLI exit code for this refusal (contract §5).
    #[must_use]
    pub fn exit_code(&self) -> u8 {
        match self {
            FmtError::Parse(_) => 2,
            FmtError::Header(_) => 3,
            FmtError::OutOfScope(_) => 4,
        }
    }
}

impl std::fmt::Display for FmtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FmtError::Parse(m) => write!(f, "parse-error: {m}"),
            FmtError::Header(m) => write!(f, "header-error: {m}"),
            FmtError::OutOfScope(m) => write!(f, "refused: {m}"),
        }
    }
}

impl std::error::Error for FmtError {}

/// Format `src` into its canonical form.
///
/// `pin` is the optional `[toolchain].format` value from `mycelium-proj.toml` (a **hard pin**: a value
/// other than [`MYCFMT_VERSION`] is refused).
///
/// # Errors
/// [`FmtError::Parse`] (unparsable), [`FmtError::Header`] (malformed header), or [`FmtError::OutOfScope`]
/// (a pin mismatch, an interior comment v0 can't preserve, or a body that does not round-trip — identity
/// could change). On any error nothing is rewritten (G2).
pub fn format_source(src: &str, pin: Option<&str>) -> Result<Formatted, FmtError> {
    // Hard pin (M-364 §10.3): never format with rules the project did not pin.
    if let Some(p) = pin {
        if p != MYCFMT_VERSION {
            return Err(FmtError::OutOfScope(format!(
                "[toolchain].format = {p:?}, but this is {MYCFMT_VERSION} — refusing to format with rules \
                 the project did not pin (hard pin; G2). Align the pin or use the matching mycfmt."
            )));
        }
    }

    // The header (M-358/M-359): a malformed marker/key is explicit, never a silent drop (C3/G2).
    let header = parse_header(src).map_err(|e| FmtError::Header(e.to_string()))?;
    // The body: the RAW parse — preserves `default paradigm`/`with paradigm` (formatting ≠ expand-ambient).
    let nodule = parse(src).map_err(|e| FmtError::Parse(e.to_string()))?;

    let lines: Vec<&str> = src.lines().collect();
    let body_start = body_start_line(&lines);

    // Interior comments are lexer trivia the canonical printer cannot place; v0 refuses rather than drop
    // them silently (§7.2). The header/leading region (all `//` or blank) is excluded from this scan.
    if let Some(line_no) = interior_comment_line(&lines, body_start) {
        return Err(FmtError::OutOfScope(format!(
            "line {line_no}: an interior comment cannot be preserved by mycfmt v0 — full \
             comment-preserving formatting is deferred; refused, never silently dropped (G2)"
        )));
    }

    let mut out = String::new();
    let mut notes = Vec::new();

    match &header {
        Some(h) => {
            // The leading region must be exactly the header (marker + its @key lines); a stray comment
            // there would be dropped by canonical re-emit — refuse instead (G2).
            let leading_comments = count_comment_lines(&lines, body_start);
            let expected = 1 + count_present_fields(&h.fields);
            if leading_comments != expected {
                return Err(FmtError::OutOfScope(format!(
                    "the header region has {leading_comments} comment line(s) but the structured header \
                     accounts for {expected} — a stray comment in the header cannot be placed by mycfmt v0; \
                     refused, never silently dropped (G2)"
                )));
            }
            out.push_str(&render_header(h));
            notes.push(
                "re-emitted the structured header (// nodule: + // @key:) in canonical order"
                    .to_owned(),
            );
        }
        None => {
            let leading = leading_comment_block(&lines, body_start);
            if !leading.is_empty() {
                out.push_str(&leading);
                notes.push("preserved the leading comment block".to_owned());
            }
        }
    }

    let body = expand_to_source(&nodule);
    out.push_str(&body);
    notes.push("re-printed the body in canonical surface form".to_owned());

    // Exactly one trailing newline.
    while out.ends_with("\n\n") {
        out.pop();
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }

    // C1 identity guard: the output must re-parse to the SAME surface AST, and the header must survive.
    // A mismatch is a refusal — mycfmt never emits an identity-changing format (round-trip-safe scope, §7).
    let reparsed = parse(&out).map_err(|e| {
        FmtError::OutOfScope(format!(
            "the formatted output did not re-parse ({e}) — refusing (round-trip-safe scope; C1/§7)"
        ))
    })?;
    if reparsed != nodule {
        return Err(FmtError::OutOfScope(
            "formatting would change the program's surface AST — identity not preserved; refusing \
             (round-trip-safe scope; C1/§7). This construct is outside mycfmt v0."
                .to_owned(),
        ));
    }
    let reheader = parse_header(&out).map_err(|e| FmtError::Header(e.to_string()))?;
    if reheader != header {
        return Err(FmtError::OutOfScope(
            "formatting would change the structured header — refusing (C3)".to_owned(),
        ));
    }

    let changed = out != src;
    Ok(Formatted {
        output: out,
        changed,
        notes,
    })
}

/// The 1-based line index where the body (the `nodule …` code) begins: the first non-blank line that is
/// not a `//` comment. Every header/leading line is a `//` comment or blank, so this cleanly separates the
/// comment/header region from the code. Returns `lines.len()` if there is no code line.
fn body_start_line(lines: &[&str]) -> usize {
    lines
        .iter()
        .position(|l| {
            let t = l.trim();
            !t.is_empty() && !t.starts_with("//")
        })
        .unwrap_or(lines.len())
}

/// Count the `//` comment lines in `lines[..end]` (the header/leading region).
fn count_comment_lines(lines: &[&str], end: usize) -> usize {
    lines[..end.min(lines.len())]
        .iter()
        .filter(|l| l.trim().starts_with("//"))
        .count()
}

/// The first 1-based line at or after `body_start` that contains a `//` comment, if any. (`.myc` has no
/// string literals, so any `//` in the body region is a comment.)
fn interior_comment_line(lines: &[&str], body_start: usize) -> Option<u32> {
    lines
        .iter()
        .enumerate()
        .skip(body_start)
        .find(|(_, l)| l.contains("//"))
        .map(|(i, _)| (i + 1) as u32)
}

/// The leading comment block (case: no structured header) — the comment lines before the code,
/// each verbatim, one per line, blank lines dropped, terminated by a newline. Empty if there are none.
fn leading_comment_block(lines: &[&str], end: usize) -> String {
    let mut out = String::new();
    for l in &lines[..end.min(lines.len())] {
        if l.trim().starts_with("//") {
            out.push_str(l);
            out.push('\n');
        }
    }
    out
}

/// Count the present (`Some`) metadata fields — each renders to exactly one `// @key:` line.
fn count_present_fields(f: &HeaderFields) -> usize {
    [
        f.version.is_some(),
        f.license.is_some(),
        f.authors.is_some(),
        f.since.is_some(),
        f.updated.is_some(),
        f.summary.is_some(),
        f.repository.is_some(),
        f.keywords.is_some(),
        f.deprecated.is_some(),
    ]
    .iter()
    .filter(|p| **p)
    .count()
}

/// Render a structured header canonically: the `// nodule:` marker, then present `// @key:` lines in the
/// fixed §4 order (`HEADER_KEYS`), one space after each colon, comma-joined lists. Values are re-emitted
/// as parsed — never fabricated (VR-5).
fn render_header(h: &StructuredHeader) -> String {
    let mut s = h.marker.canonical();
    s.push('\n');
    let f = &h.fields;
    if let Some(v) = &f.version {
        s.push_str(&format!("// @version: {v}\n"));
    }
    if let Some(v) = &f.license {
        s.push_str(&format!("// @license: {v}\n"));
    }
    if let Some(v) = &f.authors {
        s.push_str(&format!("// @authors: {}\n", v.join(", ")));
    }
    if let Some(v) = &f.since {
        s.push_str(&format!("// @since: {v}\n"));
    }
    if let Some(v) = &f.updated {
        s.push_str(&format!("// @updated: {v}\n"));
    }
    if let Some(v) = &f.summary {
        s.push_str(&format!("// @summary: {v}\n"));
    }
    if let Some(v) = &f.repository {
        s.push_str(&format!("// @repository: {v}\n"));
    }
    if let Some(v) = &f.keywords {
        s.push_str(&format!("// @keywords: {}\n", v.join(", ")));
    }
    if let Some(d) = &f.deprecated {
        let v = match d {
            Deprecated::Flag(b) => b.to_string(),
            Deprecated::Reason(r) => r.clone(),
        };
        s.push_str(&format!("// @deprecated: {v}\n"));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_a_minimal_nodule_and_is_idempotent() {
        let src =
            "// exercises: nodule header + use import\nnodule signals.demo\n\nuse core.binary\n";
        let r = format_source(src, None).expect("formats");
        // Leading comment preserved, body canonical, identity preserved.
        assert!(
            r.output
                .starts_with("// exercises: nodule header + use import\n"),
            "{}",
            r.output
        );
        assert!(r.output.contains("nodule signals.demo"));
        assert!(r.output.contains("use core.binary"));
        // Idempotent (C2): formatting the output is a no-op.
        let r2 = format_source(&r.output, None).expect("formats again");
        assert_eq!(r2.output, r.output);
        assert!(!r2.changed);
    }

    #[test]
    fn an_unparsable_file_is_an_explicit_error_not_a_rewrite() {
        let err = format_source(
            "nodule demo\nfn f(x: Binary{8}) -> Ternary{6} = swap(x, to: Ternary{6})",
            None,
        )
        .unwrap_err();
        assert_eq!(err.exit_code(), 2);
        assert!(matches!(err, FmtError::Parse(_)));
    }

    #[test]
    fn a_malformed_header_is_an_explicit_error() {
        let err = format_source("// nodule: 9bad\nnodule d\nfn f() -> Binary{8} = 0b0", None)
            .unwrap_err();
        assert_eq!(err.exit_code(), 3);
    }

    #[test]
    fn an_interior_comment_is_refused_not_dropped() {
        // A trailing comment in the body would be silently dropped by the canonical printer — refuse (§7.2).
        let src = "nodule d\nfn f(x: Binary{8}) -> Binary{8} = x // identity\n";
        let err = format_source(src, None).unwrap_err();
        assert_eq!(err.exit_code(), 4);
        assert!(format!("{err}").contains("interior comment"), "{err}");
    }

    #[test]
    fn a_toolchain_format_pin_mismatch_is_refused() {
        let src = "nodule d\nfn f(x: Binary{8}) -> Binary{8} = x\n";
        let err = format_source(src, Some("mycfmt-99")).unwrap_err();
        assert_eq!(err.exit_code(), 4);
        assert!(format!("{err}").contains("hard pin"), "{err}");
        // The matching pin formats fine.
        assert!(format_source(src, Some(MYCFMT_VERSION)).is_ok());
    }

    #[test]
    fn the_structured_header_is_re_emitted_canonically() {
        let src = "// nodule: geometry.shapes\n// @version: 1.2.0\n// @license: Apache-2.0\n\
                   nodule geometry.shapes\n\nfn area_unit() -> Binary{8} = 0b0000_0001\n";
        let r = format_source(src, None).expect("formats");
        assert!(
            r.output.starts_with(
                "// nodule: geometry.shapes\n// @version: 1.2.0\n// @license: Apache-2.0\n"
            ),
            "{}",
            r.output
        );
        // Identity + header preserved; idempotent.
        let r2 = format_source(&r.output, None).expect("again");
        assert_eq!(r2.output, r.output);
    }

    #[test]
    fn a_stray_comment_in_the_header_region_is_refused() {
        let src = "// nodule: g\n// a stray non-key comment\n// @license: MIT\nnodule g\nfn f() -> Binary{8} = 0b0\n";
        let err = format_source(src, None).unwrap_err();
        assert_eq!(err.exit_code(), 4);
    }
}
