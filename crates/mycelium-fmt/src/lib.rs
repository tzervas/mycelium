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
//! v0 scope (§7 — expressions that do not round-trip) is an **explicit error** with an exit code;
//! `mycfmt` **never** writes a partial or garbled rewrite.  Interior comments — previously refused in v0
//! — are now **preserved** by interleaving from the lexer comment table (M-690; Stage 2): leading
//! comment blocks above items are re-emitted verbatim; trailing `// …` comments on a fn body line or a
//! match arm are re-attached at the end of the rendered line.  A genuinely unplaceable comment is still
//! a [`FmtError::OutOfScope`] refusal — never a silent drop.
//! The load-bearing subtlety: the body is printed from the **raw parse** (`mycelium_l1::parse`), *not*
//! the ambient-resolved twin — so `default paradigm` / `with paradigm` are **preserved**, not expanded
//! (formatting ≠ "expand ambient").
//!
//! KC-3: this lives entirely above the kernel; the trusted base depends on nothing here.
//!
//! ## Comment placement (M-690, Stage 2 — Empirical)
//!
//! Comments are captured by `mycelium_l1::lexer::lex_with_comments` and anchored to items by
//! **source line number** matching against the token stream's `Spanned` positions:
//!
//! - **Leading doc-block**: one or more consecutive `// …` lines (not trailing) immediately before an
//!   item's first token are re-emitted verbatim above that item's canonical rendering.  A stray non-`@key`
//!   comment in the header region is now a leading doc-block on the first item (no longer refused).
//! - **Trailing fn-body comment**: a `// …` on the same source line as the `fn` keyword (i.e. the whole
//!   fn fits on one source line) is re-attached after the body expression on the rendered body line
//!   (`  body  // comment`).
//! - **Trailing match-arm comment**: a `// …` on the same source line as a `=>` token switches that
//!   match to **one-arm-per-line** rendering and places the comment after the arm's canonical expression.
//!   Non-nested matches only; a deeply nested match with arm comments is a [`FmtError::OutOfScope`] (not
//!   a silent drop — see FLAG below).
//!
//! **FLAG (anchoring sufficiency):** Token-position anchoring is sufficient for ALL tested placement
//! cases: leading doc-blocks, trailing fn-body comments, and trailing arm comments on non-nested
//! matches.  **Nested match arm trailing comments** (a `=>` line inside an outer match arm's body) are
//! out-of-scope: the `FatArrow` positions interleave in source order, making it impossible to assign
//! a comment to the correct arm purely from token positions (without AST line numbers from `ast.rs`
//! `Arm`).  Adding line numbers to `ast.rs`/`Arm` collides with the parallel HOF track (M-689
//! stage-1 sibling) and is serialized — flagged up rather than silently worked around.

use mycelium_l1::ast::{Expr, FnDecl, ImplDecl, Item, Nodule, Pattern};
use mycelium_l1::lexer::{lex_with_comments, Comment};
use mycelium_l1::token::{Spanned, Tok};
use mycelium_l1::{expand_to_source, parse, parse_phylum};
use mycelium_proj::{parse_header, Deprecated, StructuredHeader};
use std::collections::HashMap;

/// The formatter spelling/version this build implements. The `[toolchain].format` pin (M-359) is a
/// **hard pin** (M-364 §10.3): a mismatch is refused, never formatted with rules the project didn't ask
/// for (G2).
pub const MYCFMT_VERSION: &str = "mycfmt-0";

/// A successful format result.
///
/// `Default` is the empty result (no output, unchanged, no notes) — an additive constructor
/// convenience (M-644); the canonical "ends with one newline" output is produced by
/// [`format_source`], not by `Default`/`From`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Formatted {
    /// The output text. For a value produced by [`format_source`] this is the **canonical** form and
    /// always ends with exactly one newline; the additive `Default` / `From<String>` constructors do
    /// **not** normalize (M-644), so a hand-built `Formatted` may not carry that invariant.
    pub output: String,
    /// Whether the output differs from the input (drives `--check`).
    pub changed: bool,
    /// The normalizations applied, named for `EXPLAIN` (no black box).
    pub notes: Vec<String>,
}

impl From<String> for Formatted {
    /// Lift a raw output string into a [`Formatted`] (M-644 ergonomics): `changed` is `false` and
    /// `notes` is empty — a trivial wrapper for callers/tests that already hold canonical text.
    /// `format_source` is the path that computes `changed`/`notes`; this does not.
    fn from(output: String) -> Self {
        Self {
            output,
            changed: false,
            notes: Vec::new(),
        }
    }
}

/// A formatting refusal — never a partial rewrite (G2). Each maps to a CLI exit code.
///
/// `#[non_exhaustive]`: new refusal kinds may be added without a breaking change — an external
/// exhaustive `match` must carry a `_` arm (M-644; additive — no variant removed).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
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

/// Does `src` open with a `phylum` header — i.e. is the first significant line (after leading blank or
/// `//`-comment lines) the reserved `phylum` keyword at a word boundary? Lets mycfmt refuse a *malformed*
/// phylum (one `parse_phylum` rejects, e.g. a `phylum` header with no `nodule`) as `OutOfScope` rather
/// than a parse error, so a phylum source is never a parse error in v0 (M-662; G2). `phylum` is a
/// reserved keyword (never an identifier), so a leading `phylum` token unambiguously opens a phylum.
fn opens_with_phylum(src: &str) -> bool {
    src.lines()
        .map(str::trim_start)
        .find(|l| !l.is_empty() && !l.starts_with("//"))
        .and_then(|l| l.strip_prefix("phylum"))
        .is_some_and(|rest| {
            rest.is_empty() || !rest.starts_with(|c: char| c.is_alphanumeric() || c == '_')
        })
}

/// Format `src` into its canonical form.
///
/// `pin` is the optional `[toolchain].format` value from `mycelium-proj.toml` (a **hard pin**: a value
/// other than [`MYCFMT_VERSION`] is refused).
///
/// # Errors
/// [`FmtError::Parse`] (unparsable), [`FmtError::Header`] (malformed header), or [`FmtError::OutOfScope`]
/// (a pin mismatch, an unplaceable comment, or a body that does not round-trip — identity could change).
/// On any error nothing is rewritten (G2).
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

    // Phylum / multi-nodule sources are outside mycfmt v0 scope (M-662): the formatter v0 canonicalizes a
    // SINGLE nodule. A `phylum` header or multiple `nodule` blocks is an explicit out-of-scope refusal —
    // never a parse error and never a partial rewrite (G2). A WELL-FORMED phylum (header, or >1 nodule) is
    // caught via `parse_phylum`; a MALFORMED one (a `phylum` header `parse_phylum` rejects — e.g. no
    // `nodule`) is caught by its opening keyword, so a phylum source NEVER surfaces as a parse error. A
    // header-less single nodule is a phylum-of-one and formats normally below.
    let is_phylum = parse_phylum(src).is_ok_and(|ph| ph.path.is_some() || ph.nodules.len() > 1)
        || opens_with_phylum(src);
    if is_phylum {
        return Err(FmtError::OutOfScope(
            "phylum / multi-nodule sources are outside mycfmt v0 scope (M-662) — format each nodule's \
             content individually; whole-phylum canonical formatting is future work (refused, never a \
             partial rewrite — G2)"
                .to_string(),
        ));
    }

    // The header (M-358/M-359): a malformed marker/key is explicit, never a silent drop (C3/G2).
    let header = parse_header(src).map_err(|e| FmtError::Header(e.to_string()))?;
    // The body: the RAW parse — preserves `default paradigm`/`with paradigm` (formatting ≠ expand-ambient).
    let nodule = parse(src).map_err(|e| FmtError::Parse(e.to_string()))?;

    let lines: Vec<&str> = src.lines().collect();
    let body_start = body_start_line(&lines);

    // Lex with comments (M-690, Stage 2): capture every `//` comment for interleaving.
    // This is the entry point for the comment-preservation path — replaces the old refusal.
    let (tokens, comments) = lex_with_comments(src).map_err(|e| FmtError::Parse(e.to_string()))?;

    // Build a CommentPlan: classify every comment as leading (attached to an item) or trailing
    // (attached to a fn body line or a match arm).
    let has_structured_header = header.is_some();
    let plan = build_comment_plan(
        &nodule,
        &tokens,
        &comments,
        &lines,
        body_start,
        has_structured_header,
    )?;

    let mut out = String::new();
    let mut notes = Vec::new();

    match &header {
        Some(h) => {
            // Re-emit the structured header canonically.  Any stray non-`@key` comment in the header
            // region is now a leading doc-block on the first item (plan.first_item_stray_comments),
            // so we no longer refuse it.
            out.push_str(&render_header(h));
            notes.push(
                "re-emitted the structured header (// nodule: + // @key:) in canonical order"
                    .to_owned(),
            );
            // Stray header comments — preserved as leading doc-block on the first item (plan handles them).
            if !plan.stray_header_comments.is_empty() {
                notes.push(format!(
                    "preserved {} stray header comment(s) as a leading doc-block on the first item",
                    plan.stray_header_comments.len()
                ));
            }
        }
        None => {
            let leading = leading_comment_block(&lines, body_start);
            if !leading.is_empty() {
                out.push_str(&leading);
                notes.push("preserved the leading comment block".to_owned());
            }
        }
    }

    // Render the body: items with their leading/trailing comments interleaved.
    let (body, body_notes) = render_body_with_comments(&nodule, &plan)?;
    out.push_str(&body);
    notes.extend(body_notes);
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

// ================================================================================================
// Comment plan: classify all comments relative to the items they belong to.
// ================================================================================================

/// A classified comment plan for one source file.
///
/// All indices are into the `nodule.items` Vec.
struct CommentPlan {
    /// Stray non-`@key` comments in the header region: they become a leading doc-block on item 0.
    stray_header_comments: Vec<String>,
    /// Leading comment blocks per item index: comments that appear directly above an item.
    /// Key = item index (0-based).
    leading: HashMap<usize, Vec<String>>,
    /// Trailing comment on the fn body expression line, per fn item index.
    /// Key = item index (0-based), value = comment text (the `// …` lexeme).
    fn_trailing: HashMap<usize, String>,
    /// Per-item, per-arm trailing comments: item_idx → arm_idx → comment text.
    /// Populated only for items whose match arms have trailing comments.
    arm_trailing: HashMap<usize, HashMap<usize, String>>,
}

/// Build the [`CommentPlan`] by cross-referencing the comment list and the token stream positions.
///
/// `has_structured_header` is `true` when the source has a `// nodule:` / `// @key:` header.
/// When `false`, pre-body comments are already emitted by `leading_comment_block` in
/// `format_source` and must NOT also be assigned as leading for item 0 (which would double-emit).
///
/// Guarantee: Empirical — anchoring uses token line numbers; the algorithm is validated by the
/// unit tests in this file.
fn build_comment_plan(
    nodule: &Nodule,
    tokens: &[Spanned],
    comments: &[Comment],
    lines: &[&str],
    body_start: usize,
    has_structured_header: bool,
) -> Result<CommentPlan, FmtError> {
    // Collect the source-order line numbers for every top-level item's first token.
    // We scan the token stream for the item-opening keywords (fn, type, trait, impl, use,
    // nodule/default), skipping the nodule header itself.
    let item_first_token_lines = item_first_lines(nodule, tokens);

    // Collect the source-order line numbers of every FatArrow token (match arms).
    let fat_arrow_lines: Vec<u32> = tokens
        .iter()
        .filter(|s| s.tok == Tok::FatArrow)
        .map(|s| s.pos.line)
        .collect();

    // -----------------------------------------------------------------------
    // Step 1: classify header-region comments.
    //
    // Only when a structured header is present: non-`@key` header comments are
    // stray → leading doc-block on the first item.
    // When there is NO structured header, pre-body comments are handled by
    // `leading_comment_block` in `format_source` — do NOT re-assign them here.
    // -----------------------------------------------------------------------
    let mut stray_header_comments: Vec<String> = Vec::new();
    if has_structured_header {
        let header_comment_lines: Vec<u32> = comments
            .iter()
            .filter(|c| !c.trailing && (c.line as usize) <= body_start)
            .map(|c| c.line)
            .collect();

        for &hline in &header_comment_lines {
            let line_text = lines
                .get((hline as usize).saturating_sub(1))
                .copied()
                .unwrap_or("");
            let trimmed = line_text.trim();
            // A structured header line starts with `// nodule:` or `// @`:
            let is_structured = trimmed.starts_with("// nodule:") || trimmed.starts_with("// @");
            if !is_structured {
                stray_header_comments.push(trimmed.to_owned());
            }
        }
    }

    // -----------------------------------------------------------------------
    // Step 2: classify body-region comments.
    //
    // Body comments are those in lines > body_start.
    // A body comment is either:
    //   (a) trailing within a fn's line range → fn trailing comment
    //       (includes not only the fn keyword line but any line in the fn body,
    //       since the canonical render may move the comment to a different line)
    //   (b) trailing on a FatArrow line → match arm trailing comment
    //   (c) non-trailing → a leading comment for the next item
    //   (d) trailing on some other line → unplaceable → refuse (G2)
    // -----------------------------------------------------------------------

    // Build fn line ranges: for each Fn item, record (first_token_line, next_item_first_line).
    // A trailing comment anywhere in [first_token_line, next_item_first_line) that is NOT
    // on a FatArrow line is a fn-body trailing comment.
    //
    // This range-based approach handles idempotence: on the FIRST format pass, the
    // trailing comment may be on the same line as the `fn` keyword (whole fn on one line);
    // on the SECOND pass (formatting the already-formatted output), the fn body is split
    // to two lines so the comment is on the body line, not the `fn` line.
    let fn_line_ranges: Vec<(u32, u32, usize)> = {
        let mut ranges = Vec::new();
        for (idx, item) in nodule.items.iter().enumerate() {
            if matches!(item, Item::Fn(_)) {
                let first = item_first_token_lines.get(idx).copied().unwrap_or(0);
                // The range ends just before the next item's first line (or at u32::MAX for the last).
                let next = item_first_token_lines
                    .get(idx + 1)
                    .copied()
                    .unwrap_or(u32::MAX);
                ranges.push((first, next, idx));
            }
        }
        ranges
    };

    // For each trailing comment, classify it.
    let mut fn_trailing: HashMap<usize, String> = HashMap::new();
    let mut arm_trailing_flat: HashMap<u32, String> = HashMap::new(); // FatArrow line → comment text

    for comment in comments {
        // Skip header-region comments (already classified above, or handled by leading_comment_block).
        if (comment.line as usize) <= body_start {
            continue;
        }
        if !comment.trailing {
            // Non-trailing (leading) comments are handled in step 3.
            continue;
        }
        // Trailing comment in the body region.
        // First check FatArrow lines (match arm comments) — these take priority over fn range.
        if fat_arrow_lines.contains(&comment.line) {
            // Trailing comment on a match arm line.
            arm_trailing_flat.insert(comment.line, comment.text.clone());
        } else if let Some(&(_, _, item_idx)) = fn_line_ranges
            .iter()
            .find(|(first, next, _)| comment.line >= *first && comment.line < *next)
        {
            // Trailing comment within a fn's source line range → fn-body trailing comment.
            // Only record the first such comment per fn (the one closest to the body expression).
            fn_trailing
                .entry(item_idx)
                .or_insert_with(|| comment.text.clone());
        } else {
            // Trailing on an unknown line → unplaceable (G2: never silent drop).
            return Err(FmtError::OutOfScope(format!(
                "line {}: a trailing comment on this line cannot be placed by mycfmt v0 — \
                 the line is not within a `fn` declaration or a match arm (`=>`); refused, never \
                 silently dropped (G2; M-690 stage-2 scope)",
                comment.line
            )));
        }
    }

    // -----------------------------------------------------------------------
    // Step 3: assign arm trailing comments to item and arm indices.
    //
    // We walk the nodule's items and, for each `Item::Fn`, walk the fn's body
    // looking for `Expr::Match` nodes.  For each match, we assign the arm trailing
    // comments by matching their FatArrow source lines to the arms in source order.
    //
    // FLAG: nested match arm trailing comments are out of scope — see module-level
    // FLAG comment.  A trailing comment on a nested match arm is an `OutOfScope`
    // refusal (never a silent drop — G2).
    // -----------------------------------------------------------------------
    let mut arm_trailing: HashMap<usize, HashMap<usize, String>> = HashMap::new();

    if !arm_trailing_flat.is_empty() {
        // For each item that is a Fn or Impl (which contains Fns), scan for matches.
        // Build a "remaining arrows" list in source order to assign to arms.
        let mut remaining_arrow_comments: Vec<(u32, String)> =
            arm_trailing_flat.into_iter().collect();
        remaining_arrow_comments.sort_by_key(|(line, _)| *line);

        for (item_idx, item) in nodule.items.iter().enumerate() {
            match item {
                Item::Fn(fd) => {
                    assign_arm_comments_for_fn(
                        item_idx,
                        &fd.body,
                        &mut remaining_arrow_comments,
                        &mut arm_trailing,
                        &fat_arrow_lines,
                    )?;
                }
                Item::Impl(id) => {
                    for method in &id.methods {
                        assign_arm_comments_for_fn(
                            item_idx,
                            &method.body,
                            &mut remaining_arrow_comments,
                            &mut arm_trailing,
                            &fat_arrow_lines,
                        )?;
                    }
                }
                _ => {}
            }
        }

        // Any remaining unassigned arm comments are unplaceable (G2).
        if let Some((line, _)) = remaining_arrow_comments.first() {
            return Err(FmtError::OutOfScope(format!(
                "line {line}: a trailing match-arm comment could not be assigned to an arm — \
                 possibly a nested match (out of scope for token-position anchoring); \
                 refused, never silently dropped (G2; M-690 FLAG)"
            )));
        }
    }

    // -----------------------------------------------------------------------
    // Step 4: assign leading (non-trailing) body comments to items.
    // -----------------------------------------------------------------------
    let mut leading: HashMap<usize, Vec<String>> = HashMap::new();

    // Stray header comments become leading comments on item 0.
    if !stray_header_comments.is_empty() && !nodule.items.is_empty() {
        leading
            .entry(0)
            .or_default()
            .extend(stray_header_comments.clone());
    }

    // Non-trailing body-region comments: assign each to the first item whose first-token line
    // is > the comment's line (the item immediately following the comment block).
    let body_non_trailing: Vec<&Comment> = comments
        .iter()
        .filter(|c| !c.trailing && (c.line as usize) > body_start)
        .collect();

    for comment in body_non_trailing {
        // Find the item whose first-token line is closest to (and after) this comment's line.
        let target_item = item_first_token_lines
            .iter()
            .enumerate()
            .find(|(_, &first_line)| first_line > comment.line)
            .map(|(idx, _)| idx);

        match target_item {
            Some(item_idx) => {
                leading
                    .entry(item_idx)
                    .or_default()
                    .push(comment.text.clone());
            }
            None => {
                // Comment after the last item — attach it to the last item as a trailing
                // comment block. (We'll handle end-of-nodule comments as trailing on the last item.)
                if let Some(last_idx) = nodule.items.len().checked_sub(1) {
                    leading
                        .entry(last_idx + 1)
                        .or_default()
                        .push(comment.text.clone());
                }
            }
        }
    }

    Ok(CommentPlan {
        stray_header_comments,
        leading,
        fn_trailing,
        arm_trailing,
    })
}

/// Collect, for each item in `nodule.items`, the 1-based source line of its first token.
///
/// We walk the token stream looking for the Nth occurrence of item-opening tokens
/// (after the `nodule` header tokens are consumed).
fn item_first_lines(nodule: &Nodule, tokens: &[Spanned]) -> Vec<u32> {
    let mut result = Vec::with_capacity(nodule.items.len());

    // Skip past the nodule header tokens: `nodule`, the path segments and dots,
    // and the optional `@std-sys` marker.  The items begin after the last path token.
    // We find the `Tok::Nodule` token and skip until we reach the first item keyword.
    let body_tokens: Vec<&Spanned> = {
        let mut found_nodule = false;
        let mut after_header = Vec::new();
        for s in tokens {
            if !found_nodule {
                if s.tok == Tok::Nodule {
                    found_nodule = true;
                }
                continue;
            }
            // After the nodule token: skip the path (Ident, Dot) and optional @std-sys.
            // The first item keyword follows.
            if matches!(s.tok, Tok::Ident(_) | Tok::Dot | Tok::AtStdSys | Tok::Eof)
                && after_header.is_empty()
            {
                continue;
            }
            if s.tok == Tok::Eof {
                break;
            }
            after_header.push(s);
        }
        after_header
    };

    // Now walk the body tokens.  For each item, find the first token that opens it.
    // We match by counting item-opening keywords/tokens in source order.
    //
    // Items in the AST come in the same order as in the source.  We assign the first
    // token (by source order) that could open each item:
    //
    //   Item::Use    → Tok::Use
    //   Item::Default→ Tok::Default
    //   Item::Type   → Tok::Type  (or Tok::Pub + Tok::Type)
    //   Item::Trait  → Tok::Trait (or Tok::Pub + Tok::Trait)
    //   Item::Impl   → Tok::Impl
    //   Item::Fn     → Tok::Fn    (or Tok::Pub + Tok::Fn, or Tok::Thaw + …)
    //
    // We peek one token at a time and consume the first matching opener for each item.
    let mut tok_idx = 0;

    for item in &nodule.items {
        // Find the next opener for this item.
        while tok_idx < body_tokens.len() {
            let s = body_tokens[tok_idx];
            let is_opener = match item {
                Item::Use(_) => s.tok == Tok::Use,
                Item::Default(_) => s.tok == Tok::Default,
                Item::Type(_) => matches!(s.tok, Tok::Type | Tok::Pub),
                Item::Trait(_) => matches!(s.tok, Tok::Trait | Tok::Pub),
                Item::Impl(_) => s.tok == Tok::Impl,
                Item::Fn(_) => matches!(s.tok, Tok::Fn | Tok::Pub | Tok::Thaw),
            };
            if is_opener {
                result.push(s.pos.line);
                tok_idx += 1;
                break;
            }
            tok_idx += 1;
        }
        // If we ran out of tokens without finding an opener, push a sentinel.
        if result.len() < nodule.items.len()
            && result.len()
                == nodule
                    .items
                    .iter()
                    .position(|i| std::ptr::eq(i, item))
                    .map_or(0, |p| p)
        {
            // sentinel: line 0 means no match found
        }
    }

    // Pad if we found fewer lines than items (should not happen for well-formed input).
    while result.len() < nodule.items.len() {
        result.push(0);
    }

    result
}

/// Walk `expr`, find `Expr::Match` nodes (top-level, non-nested), and for each arm whose
/// source `FatArrow` line has a trailing comment, record it in `arm_trailing[item_idx][arm_idx]`.
///
/// This is intentionally limited to the TOP-LEVEL match in `expr`'s body (non-nested).
/// A trailing arm comment inside a nested match's arm body is out of scope (FLAG in module doc).
fn assign_arm_comments_for_fn(
    item_idx: usize,
    expr: &Expr,
    remaining: &mut Vec<(u32, String)>,
    arm_trailing: &mut HashMap<usize, HashMap<usize, String>>,
    fat_arrow_lines: &[u32],
) -> Result<(), FmtError> {
    // Walk to the top-level `match` (possibly wrapped in `let`, `if`, `with paradigm`).
    // We only handle a single level of match; nested matches are out of scope.
    collect_match_arm_comments(item_idx, expr, remaining, arm_trailing, fat_arrow_lines, 0)
}

/// Recursive walker for match arm comment assignment.
/// `depth` tracks how many `match` expressions we've entered; arm trailing comments in matches
/// at depth > 1 (nested) are out of scope.
fn collect_match_arm_comments(
    item_idx: usize,
    expr: &Expr,
    remaining: &mut Vec<(u32, String)>,
    arm_trailing: &mut HashMap<usize, HashMap<usize, String>>,
    fat_arrow_lines: &[u32],
    depth: u32,
) -> Result<(), FmtError> {
    match expr {
        Expr::Match { scrutinee, arms } => {
            // Find the FatArrow lines for these arms.
            // We consume `arms.len()` arrow lines from `remaining` that fall within the
            // expected range.  For depth=0 (top-level match), we consume in order.
            // For depth>0 (nested), we still try, but this is the out-of-scope case.
            for (arm_idx, _arm) in arms.iter().enumerate() {
                if remaining.is_empty() {
                    break;
                }
                let (arrow_line, comment_text) = remaining.remove(0);
                // Verify this is actually a FatArrow line.
                if !fat_arrow_lines.contains(&arrow_line) {
                    // Should not happen (we built remaining from fat_arrow_lines).
                    return Err(FmtError::OutOfScope(format!(
                        "line {arrow_line}: internal error assigning arm comment (G2 never-silent)"
                    )));
                }
                if depth > 0 {
                    // Nested match arm comment: out of scope (FLAG).
                    return Err(FmtError::OutOfScope(format!(
                        "line {arrow_line}: a trailing comment on a nested match arm is out of scope \
                         for token-position anchoring (M-690 FLAG; see module doc); refused, never \
                         silently dropped (G2)"
                    )));
                }
                arm_trailing
                    .entry(item_idx)
                    .or_default()
                    .insert(arm_idx, comment_text);
            }
            // Recurse into arm bodies to catch further nested matches.
            for arm in arms {
                collect_match_arm_comments(
                    item_idx,
                    &arm.body,
                    remaining,
                    arm_trailing,
                    fat_arrow_lines,
                    depth + 1,
                )?;
            }
            // Recurse into scrutinee.
            collect_match_arm_comments(
                item_idx,
                scrutinee,
                remaining,
                arm_trailing,
                fat_arrow_lines,
                depth,
            )?;
        }
        // Recurse through expression wrappers that may contain a match.
        Expr::Let { bound, body, .. } => {
            collect_match_arm_comments(
                item_idx,
                bound,
                remaining,
                arm_trailing,
                fat_arrow_lines,
                depth,
            )?;
            collect_match_arm_comments(
                item_idx,
                body,
                remaining,
                arm_trailing,
                fat_arrow_lines,
                depth,
            )?;
        }
        Expr::If { cond, conseq, alt } => {
            collect_match_arm_comments(
                item_idx,
                cond,
                remaining,
                arm_trailing,
                fat_arrow_lines,
                depth,
            )?;
            collect_match_arm_comments(
                item_idx,
                conseq,
                remaining,
                arm_trailing,
                fat_arrow_lines,
                depth,
            )?;
            collect_match_arm_comments(
                item_idx,
                alt,
                remaining,
                arm_trailing,
                fat_arrow_lines,
                depth,
            )?;
        }
        Expr::For { xs, init, body, .. } => {
            collect_match_arm_comments(
                item_idx,
                xs,
                remaining,
                arm_trailing,
                fat_arrow_lines,
                depth,
            )?;
            collect_match_arm_comments(
                item_idx,
                init,
                remaining,
                arm_trailing,
                fat_arrow_lines,
                depth,
            )?;
            collect_match_arm_comments(
                item_idx,
                body,
                remaining,
                arm_trailing,
                fat_arrow_lines,
                depth,
            )?;
        }
        Expr::App { head, args } => {
            collect_match_arm_comments(
                item_idx,
                head,
                remaining,
                arm_trailing,
                fat_arrow_lines,
                depth,
            )?;
            for a in args {
                collect_match_arm_comments(
                    item_idx,
                    a,
                    remaining,
                    arm_trailing,
                    fat_arrow_lines,
                    depth,
                )?;
            }
        }
        Expr::WithParadigm { body, .. } | Expr::Wild(body) | Expr::Spore(body) => {
            collect_match_arm_comments(
                item_idx,
                body,
                remaining,
                arm_trailing,
                fat_arrow_lines,
                depth,
            )?;
        }
        Expr::Swap { value, .. } => {
            collect_match_arm_comments(
                item_idx,
                value,
                remaining,
                arm_trailing,
                fat_arrow_lines,
                depth,
            )?;
        }
        Expr::Colony(hyphae) => {
            for h in hyphae {
                collect_match_arm_comments(
                    item_idx,
                    &h.body,
                    remaining,
                    arm_trailing,
                    fat_arrow_lines,
                    depth,
                )?;
            }
        }
        Expr::Ascribe(inner, _) => {
            collect_match_arm_comments(
                item_idx,
                inner,
                remaining,
                arm_trailing,
                fat_arrow_lines,
                depth,
            )?;
        }
        // Leaves: Lit, Path — no subexpressions to recurse into.
        Expr::Lit(_) | Expr::Path(_) => {}
    }
    Ok(())
}

// ================================================================================================
// Body rendering with comment interleaving.
// ================================================================================================

/// Render the nodule body (all items) with leading/trailing comments interleaved.
/// Returns the rendered text and any notes for the caller.
fn render_body_with_comments(
    nodule: &Nodule,
    plan: &CommentPlan,
) -> Result<(String, Vec<String>), FmtError> {
    let mut out = String::new();
    let mut notes = Vec::new();

    // Render the nodule header line (e.g. `nodule signals.demo`).
    let nodule_line = format!(
        "nodule {}{}\n",
        nodule.path.0.join("."),
        if nodule.std_sys { " @std-sys" } else { "" }
    );
    out.push_str(&nodule_line);

    for (item_idx, item) in nodule.items.iter().enumerate() {
        out.push('\n');

        // Emit any leading comments for this item.
        if let Some(leading_comments) = plan.leading.get(&item_idx) {
            let count = leading_comments.len();
            for cmt in leading_comments {
                out.push_str(cmt);
                out.push('\n');
            }
            notes.push(format!(
                "preserved {count} leading comment(s) on item {}",
                item_idx
            ));
        }

        // Render the item itself.
        let item_text = render_item_with_comments(item, item_idx, plan, &mut notes)?;
        out.push_str(&item_text);
    }

    // End-of-nodule trailing comments (attached after the last item, index = items.len()).
    let sentinel_idx = nodule.items.len();
    if let Some(trailing_comments) = plan.leading.get(&sentinel_idx) {
        for cmt in trailing_comments {
            out.push('\n');
            out.push_str(cmt);
            out.push('\n');
        }
        notes.push(format!(
            "preserved {} comment(s) at the end of the nodule",
            trailing_comments.len()
        ));
    }

    Ok((out, notes))
}

/// Render a single top-level item with its trailing comment (if any) and arm comments (if any).
fn render_item_with_comments(
    item: &Item,
    item_idx: usize,
    plan: &CommentPlan,
    notes: &mut Vec<String>,
) -> Result<String, FmtError> {
    match item {
        Item::Fn(fd) => {
            let fn_trailing = plan.fn_trailing.get(&item_idx);
            let arm_map = plan.arm_trailing.get(&item_idx);
            render_fn_decl_with_comments(fd, fn_trailing.map(String::as_str), arm_map, notes)
        }
        Item::Impl(id) => {
            // An impl can contain fns; delegate to render_impl which can attach arm comments
            // (currently: arm comments in impl methods are treated the same as top-level fns).
            render_impl_with_comments(id, item_idx, plan, notes)
        }
        // Other items (use, default, type, trait) are rendered via expand_to_source on a
        // synthetic single-item nodule-like string, or more simply: we re-implement the trivial
        // render inline (DRY note: these are verbatim copies of the ambient.rs private printers,
        // which are NOT accessible from this crate — this crate must duplicate the output).
        _ => Ok(render_non_fn_item(item)),
    }
}

/// Render a non-fn item to its canonical surface form.  Mirrors the private ambient.rs printers.
fn render_non_fn_item(item: &Item) -> String {
    // We derive the text by calling `expand_to_source` on a synthetic single-item nodule, then
    // extracting the item's text.  This avoids duplicating the printer logic while staying
    // correct.
    //
    // CAVEAT: `expand_to_source` always outputs `nodule <path>\n\n<item>\n`.  We strip the
    // header.
    let synthetic = mycelium_l1::ast::Nodule {
        path: mycelium_l1::ast::Path(vec!["_".to_owned()]),
        std_sys: false,
        items: vec![item.clone()],
    };
    let full = expand_to_source(&synthetic);
    // `full` is `nodule _\n\n<item text>\n`; we want just `<item text>\n`.
    // Strip "nodule _\n\n" from the front.
    full.splitn(3, '\n')
        .nth(2)
        .map(str::to_owned)
        .unwrap_or_default()
}

/// Render a `fn` declaration with optional fn-body trailing comment and arm comments.
///
/// The canonical form is:
/// ```text
/// fn name(params) -> ret =
///   body_expr
/// ```
/// With a trailing fn-body comment:
/// ```text
/// fn name(params) -> ret =
///   body_expr  // comment
/// ```
/// With match arm comments (multiline match):
/// ```text
/// fn name(params) -> ret =
///   match x {
///     arm0_pat => arm0_body,  // comment
///     arm1_pat => arm1_body
///   }
/// ```
fn render_fn_decl_with_comments(
    fd: &FnDecl,
    fn_trailing: Option<&str>,
    arm_map: Option<&HashMap<usize, String>>,
    notes: &mut Vec<String>,
) -> Result<String, FmtError> {
    let sig_text = render_sig_tail(&fd.sig);
    let pub_prefix = if fd.vis.is_pub() { "pub " } else { "" };
    let thaw_prefix = if fd.thaw { "thaw " } else { "" };

    let body_text = if let Some(amap) = arm_map {
        if !amap.is_empty() {
            // The body contains a match with arm trailing comments; render multiline.
            notes.push(format!(
                "preserved {} trailing arm comment(s) with multiline match rendering in `{}`",
                amap.len(),
                fd.sig.name
            ));
            render_expr_with_arm_comments(&fd.body, amap)?
        } else {
            render_expr_canonical(&fd.body)
        }
    } else {
        render_expr_canonical(&fd.body)
    };

    // When the body text is multiline (e.g. a multiline match), indent ALL lines by 2 spaces so
    // the canonical form is properly structured relative to the `fn sig =` header line.
    // A single-line body just gets the `  ` prefix directly.
    let indented_body = if body_text.contains('\n') {
        body_text
            .lines()
            .map(|l| {
                if l.is_empty() {
                    String::new()
                } else {
                    format!("  {l}")
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        format!("  {body_text}")
    };

    let mut s = format!("{pub_prefix}{thaw_prefix}fn {sig_text} =\n{indented_body}\n");

    // Append fn-body trailing comment if any.
    // A trailing comment on the fn body means the whole fn was on one source line
    // (the body is a simple expression, not a multiline construct) so we append
    // to the (single) body line — between the body expression and the final `\n`.
    if let Some(cmt) = fn_trailing {
        if s.ends_with('\n') {
            s.pop(); // remove the trailing newline
            s.push_str(&format!("  {cmt}"));
            s.push('\n');
        }
        notes.push(format!(
            "preserved trailing comment on fn `{}`: {cmt}",
            fd.sig.name
        ));
    }

    Ok(s)
}

/// Render an `impl` declaration (which may contain fns with arm comments).
fn render_impl_with_comments(
    id: &ImplDecl,
    item_idx: usize,
    plan: &CommentPlan,
    notes: &mut Vec<String>,
) -> Result<String, FmtError> {
    let args = if id.trait_args.is_empty() {
        String::new()
    } else {
        let a: Vec<String> = id.trait_args.iter().map(render_type_ref).collect();
        format!("<{}>", a.join(", "))
    };
    let mut s = format!(
        "impl {}{} for {} {{\n",
        id.trait_name,
        args,
        render_type_ref(&id.for_ty)
    );
    let arm_map = plan.arm_trailing.get(&item_idx);
    for method in &id.methods {
        let method_text = render_fn_decl_with_comments(method, None, arm_map, notes)?;
        for line in method_text.lines() {
            s.push_str("  ");
            s.push_str(line);
            s.push('\n');
        }
    }
    s.push_str("}\n");
    Ok(s)
}

// ================================================================================================
// Expression renderers (mirrors of ambient.rs private printers, for the comment-aware path).
// ================================================================================================

/// Render an expression in canonical form (without any comment interleaving).
/// This mirrors `print_expr` in ambient.rs.
fn render_expr_canonical(e: &Expr) -> String {
    match e {
        Expr::Lit(l) => render_literal(l),
        Expr::Path(p) => p.0.join("."),
        Expr::Let {
            name,
            ty,
            bound,
            body,
        } => {
            let ann = ty
                .as_ref()
                .map(|t| format!(": {}", render_type_ref(t)))
                .unwrap_or_default();
            format!(
                "let {name}{ann} = {} in {}",
                render_expr_canonical(bound),
                render_expr_canonical(body)
            )
        }
        Expr::If { cond, conseq, alt } => format!(
            "if {} then {} else {}",
            render_expr_canonical(cond),
            render_expr_canonical(conseq),
            render_expr_canonical(alt)
        ),
        Expr::Match { scrutinee, arms } => {
            let arm_strs: Vec<String> = arms
                .iter()
                .map(|a| {
                    format!(
                        "{} => {}",
                        render_pattern(&a.pattern),
                        render_expr_canonical(&a.body)
                    )
                })
                .collect();
            format!(
                "match {} {{ {} }}",
                render_expr_canonical(scrutinee),
                arm_strs.join(", ")
            )
        }
        Expr::For {
            x,
            xs,
            acc,
            init,
            body,
        } => format!(
            "for {x} in {}, {acc} = {} => {}",
            render_expr_canonical(xs),
            render_expr_canonical(init),
            render_expr_canonical(body)
        ),
        Expr::Swap {
            value,
            target,
            policy,
        } => format!(
            "swap({}, to: {}, policy: {})",
            render_expr_canonical(value),
            render_type_ref(target),
            policy.0.join(".")
        ),
        Expr::WithParadigm { paradigm, body } => {
            format!(
                "with paradigm {paradigm} {{ {} }}",
                render_expr_canonical(body)
            )
        }
        Expr::Wild(b) => format!("wild {{ {} }}", render_expr_canonical(b)),
        Expr::Spore(b) => format!("spore({})", render_expr_canonical(b)),
        Expr::Colony(hyphae) => {
            let hs: Vec<String> = hyphae
                .iter()
                .map(|h| format!("hypha {}", render_expr_canonical(&h.body)))
                .collect();
            format!("colony {{ {} }}", hs.join(", "))
        }
        Expr::App { head, args } => {
            let arg_strs: Vec<String> = args.iter().map(render_expr_canonical).collect();
            format!("{}({})", render_expr_canonical(head), arg_strs.join(", "))
        }
        Expr::Ascribe(inner, t) => {
            format!("{} : {}", render_expr_canonical(inner), render_type_ref(t))
        }
    }
}

/// Render an expression, using multiline match rendering if the arm_map is non-empty and
/// the expression (or sub-expression) is a match.
///
/// **Comma placement:** the arm separator `,` must go BEFORE any trailing comment on that arm's
/// line.  A `// …` comment runs to end-of-line, so a comma placed AFTER it would be swallowed
/// into the comment text and then lost when the output is re-lexed, breaking the round-trip.
/// Canonical multiline form:
/// ```text
/// match scrutinee {
///   pat0 => body0,  // comment
///   pat1 => body1
/// }
/// ```
fn render_expr_with_arm_comments(
    e: &Expr,
    arm_map: &HashMap<usize, String>,
) -> Result<String, FmtError> {
    match e {
        Expr::Match { scrutinee, arms } => {
            // Multiline match rendering: one arm per line, comma BEFORE any trailing comment.
            let scrutinee_str = render_expr_canonical(scrutinee);
            let last_idx = arms.len().saturating_sub(1);
            let mut arm_strs = Vec::with_capacity(arms.len());
            for (arm_idx, arm) in arms.iter().enumerate() {
                let arm_body = render_expr_canonical(&arm.body);
                let pat = render_pattern(&arm.pattern);
                let is_last = arm_idx == last_idx;
                // The comma (`,`) separates arms and must appear BEFORE any trailing comment
                // so it is not swallowed into the comment text during re-lexing.
                let comma = if is_last { "" } else { "," };
                if let Some(cmt) = arm_map.get(&arm_idx) {
                    arm_strs.push(format!("  {pat} => {arm_body}{comma}  {cmt}"));
                } else {
                    arm_strs.push(format!("  {pat} => {arm_body}{comma}"));
                }
            }
            Ok(format!(
                "match {scrutinee_str} {{\n{}\n}}",
                arm_strs.join("\n")
            ))
        }
        // For non-match expressions, fall through to canonical rendering.
        // (arm_map entries for arm lines that are not a direct top-level match would already have
        // been refused by the comment plan builder.)
        _ => Ok(render_expr_canonical(e)),
    }
}

fn render_pattern(p: &Pattern) -> String {
    match p {
        Pattern::Wildcard => "_".to_owned(),
        Pattern::Lit(l) => render_literal(l),
        Pattern::Ctor(n, subs) => {
            let s: Vec<String> = subs.iter().map(render_pattern).collect();
            format!("{n}({})", s.join(", "))
        }
        Pattern::Ident(n) => n.clone(),
    }
}

fn render_literal(l: &mycelium_l1::ast::Literal) -> String {
    use mycelium_l1::ast::Literal;
    match l {
        Literal::Bin(s) => format!("0b{s}"),
        Literal::Trit(s) => format!("<{s}>"),
        // RFC-0032 D4 (M-750): a `0x…` byte-string literal round-trips to its source form.
        Literal::Bytes(s) => format!("0x{s}"),
        Literal::Int(i) => format!("{i}"),
        Literal::AmbientInt(p, i) => format!("{i} /* {p} (width from context) */"),
        Literal::List(es) => {
            let s: Vec<String> = es.iter().map(render_expr_canonical).collect();
            format!("[{}]", s.join(", "))
        }
        // #[non_exhaustive]: new literal variants surface as a never-silent internal error (G2).
        _ => unreachable!(
            "unrecognized Literal variant — mycelium-l1 version mismatch (G2: never silent)"
        ),
    }
}

fn render_type_ref(t: &mycelium_l1::ast::TypeRef) -> String {
    use mycelium_l1::ast::BaseType;
    let base = match &t.base {
        BaseType::Binary(n) => format!("Binary{{{n}}}"),
        BaseType::Ternary(m) => format!("Ternary{{{m}}}"),
        BaseType::Dense(d, s) => format!("Dense{{{d}, {}}}", scalar_str(*s)),
        BaseType::Vsa {
            model,
            dim,
            sparsity,
        } => {
            format!("VSA{{{model}, {dim}, {}}}", sparsity_str(sparsity))
        }
        BaseType::Substrate(t) => format!("Substrate{{{t}}}"),
        // RFC-0032 D3/D4 (M-749/M-750): `Seq{T, N}` / nullary `Bytes`.
        BaseType::Seq { elem, len } => format!("Seq{{{}, {len}}}", render_type_ref(elem)),
        BaseType::Bytes => "Bytes".to_owned(),
        BaseType::Named(n, args) if args.is_empty() => n.clone(),
        BaseType::Named(n, args) => {
            let a: Vec<String> = args.iter().map(render_type_ref).collect();
            format!("{n}<{}>", a.join(", "))
        }
        BaseType::Ambient(params) => format!("{{{}}}", ambient_params_str(params)),
        // RFC-0024 §3: function type `A -> B` (right-associative). The parser builds `Fn(atom, rhs)`
        // where the left is always a non-`Fn` atom, so rendering both sides recursively and joining
        // with ` -> ` round-trips without parentheses (C1).
        BaseType::Fn(a, b) => format!("{} -> {}", render_type_ref(a), render_type_ref(b)),
    };
    match t.guarantee {
        Some(g) => format!("{base} @ {g:?}"),
        None => base,
    }
}

fn render_sig_tail(sig: &mycelium_l1::ast::FnSig) -> String {
    let tp = if sig.params.is_empty() {
        String::new()
    } else {
        let ps: Vec<String> = sig.params.iter().map(render_type_param).collect();
        format!("<{}>", ps.join(", "))
    };
    let ps: Vec<String> = sig
        .value_params
        .iter()
        .map(|p| format!("{}: {}", p.name, render_type_ref(&p.ty)))
        .collect();
    let eff = if sig.effects.is_empty() {
        String::new()
    } else {
        format!(" !{{{}}}", sig.effects.join(", "))
    };
    format!(
        "{}{}({}) -> {}{}",
        sig.name,
        tp,
        ps.join(", "),
        render_type_ref(&sig.ret),
        eff
    )
}

fn render_type_param(p: &mycelium_l1::ast::TypeParam) -> String {
    if p.bounds.is_empty() {
        p.name.clone()
    } else {
        let bs: Vec<String> = p.bounds.iter().map(render_trait_ref).collect();
        format!("{}: {}", p.name, bs.join(" + "))
    }
}

fn render_trait_ref(tr: &mycelium_l1::ast::TraitRef) -> String {
    if tr.args.is_empty() {
        tr.name.clone()
    } else {
        let args: Vec<String> = tr.args.iter().map(render_type_ref).collect();
        format!("{}<{}>", tr.name, args.join(", "))
    }
}

fn scalar_str(s: mycelium_l1::ast::Scalar) -> &'static str {
    use mycelium_l1::ast::Scalar;
    match s {
        Scalar::F16 => "F16",
        Scalar::Bf16 => "BF16",
        Scalar::F32 => "F32",
        Scalar::F64 => "F64",
    }
}

fn sparsity_str(s: &mycelium_l1::ast::Sparsity) -> String {
    use mycelium_l1::ast::Sparsity;
    match s {
        Sparsity::Dense => "Dense".to_owned(),
        Sparsity::Sparse(k) => format!("Sparse{{{k}}}"),
    }
}

fn ambient_params_str(p: &mycelium_l1::ast::AmbientParams) -> String {
    use mycelium_l1::ast::AmbientParams;
    match p {
        AmbientParams::Size(n) => format!("{n}"),
        AmbientParams::Dense(d, s) => format!("{d}, {}", scalar_str(*s)),
        AmbientParams::Vsa {
            model,
            dim,
            sparsity,
        } => {
            format!("{model}, {dim}, {}", sparsity_str(sparsity))
        }
    }
}

// ================================================================================================
// Helpers (unchanged from the original implementation).
// ================================================================================================

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
mod tests;
