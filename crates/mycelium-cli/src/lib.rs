//! `mycelium-cli` — the **`myc`** one-command toolchain driver (M-733; E16-1).
//!
//! A single front door over the Mycelium toolchain: `myc init` scaffolds a phylum, `myc build`
//! packages it (the content-addressed spore — M-368), `myc check` type-checks it (parse + check via
//! the L1 front-end), `myc test` runs the available verification, `myc run` is the (honestly
//! not-yet-wired) execution entry point, and `myc --stream` parses a `;`-delimited component stream
//! from stdin or a file (M-820 / DN-57).
//!
//! ## Error-message quality bar (DN-22 / RFC-0013)
//! Every user-visible failure is a structured [`Report`]: a stable `code`, a human-readable
//! `message`, an optional source `location`, and an actionable `help`. No raw Rust panic ever
//! reaches the user (G2 — never opaque); a failure the driver cannot honestly act on is reported as
//! such, never swallowed and never faked (VR-5).
//!
//! ## Honesty about scope (`Declared`)
//! `init` / `build` / `check` do real end-to-end work. `test` runs `check` and is explicit that a
//! dedicated `.myc` unit-test *runner* does not exist yet (it does not pretend to have run tests
//! that were never written). `run` is **not yet wired** — the project→interpreter pipeline is later
//! work — and says so with an actionable [`Report`] instead of a stub that silently does nothing.
//! `--stream` is a **token-driven** component splitter: it lexes the source once
//! ([`mycelium_l1::lexer::lex`]), segments the token stream at `nodule` header tokens (`;` as
//! `Tok::Semi` is the per-item terminator — DN-57), and parse each component slice with
//! [`mycelium_l1::parse`]. Splitting on *tokens* (not raw text) makes it comment-/string-safe by
//! construction: a `nodule`/`;` inside a `//` comment is never a token, so it can never mis-split
//! (DN-57 §2). The per-component parse bounds parse state to one component at a time. **v0 I/O is
//! whole-input-buffered** (`Declared`); true per-`;`-component incremental I/O would require a
//! resumable L1 token-stream API that does not exist yet (flagged future work).

use std::io::Read as StdRead;
use std::path::{Path, PathBuf};

use mycelium_l1::lexer::lex;
use mycelium_l1::token::{Pos, Spanned, Tok};
use mycelium_l1::{check_nodule, parse, ParseError};
use mycelium_proj::parse_manifest;
use mycelium_spore::{build_spore, explain, Spore};

/// A structured, actionable diagnostic (the DN-22 quality bar; a projection of an RFC-0013
/// diagnostic). It renders as `error[<code>]: <message>` with optional `--> <location>` and
/// `help:` lines — never an opaque internal error (G2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    /// A stable, machine-readable diagnostic code (e.g. `myc-parse`, `myc-build`, `myc-run-unwired`).
    pub code: &'static str,
    /// The human-readable, specific message.
    pub message: String,
    /// An optional `path:line:col` (or `path`) the user can jump to.
    pub location: Option<String>,
    /// An optional actionable next step.
    pub help: Option<String>,
    /// The process exit code this report maps to (sysexits-flavoured; never 0).
    pub exit: u8,
}

impl Report {
    /// A report with a code, message and exit code (no location/help).
    #[must_use]
    pub fn new(code: &'static str, message: impl Into<String>, exit: u8) -> Self {
        Report {
            code,
            message: message.into(),
            location: None,
            help: None,
            exit,
        }
    }

    /// Attach a `path:line:col` (or `path`) location.
    #[must_use]
    pub fn at(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    /// Attach an actionable `help:` line.
    #[must_use]
    pub fn help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Render the multi-line, structured form (no trailing newline).
    #[must_use]
    pub fn render(&self) -> String {
        let mut s = format!("error[{}]: {}", self.code, self.message);
        if let Some(loc) = &self.location {
            s.push_str(&format!("\n  --> {loc}"));
        }
        if let Some(help) = &self.help {
            s.push_str(&format!("\n  help: {help}"));
        }
        s
    }
}

impl std::fmt::Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.render())
    }
}

impl std::error::Error for Report {}

/// `myc init <name>` — scaffold a new phylum named `name` under `parent`, returning the created
/// files. The name must be a simple lowercase identifier (`[a-z][a-z0-9_]*`); a dotted/empty/
/// mixed-case name is refused, never silently normalized (G2). An existing project at the target is
/// refused — `init` never overwrites (G2).
///
/// # Errors
/// [`Report`] (`myc-init-name` / `myc-init-exists` / `myc-io`) on a bad name, a pre-existing project,
/// or a filesystem failure.
pub fn init(parent: &Path, name: &str) -> Result<Vec<PathBuf>, Report> {
    validate_name(name)?;
    let dir = parent.join(name);
    let manifest_path = dir.join("mycelium-proj.toml");
    if manifest_path.exists() {
        return Err(Report::new(
            "myc-init-exists",
            format!("a project already exists at {}", manifest_path.display()),
            66,
        )
        .help(
            "choose a new name or remove the existing project — `myc init` never overwrites (G2)",
        ));
    }
    std::fs::create_dir_all(&dir)
        .map_err(|e| Report::new("myc-io", format!("{}: {e}", dir.display()), 66))?;

    let manifest = scaffold_manifest(name);
    let nodule = scaffold_nodule(name);
    let source_path = dir.join(format!("{name}.myc"));

    write_new(&manifest_path, &manifest)?;
    write_new(&source_path, &nodule)?;
    Ok(vec![manifest_path, source_path])
}

/// `myc build` — build the content-addressed spore for the project at `manifest_path`, returning the
/// built [`Spore`] and its descriptor text (M-368). A missing/ambiguous publish input is surfaced as
/// a structured [`Report`], never a partial artifact (G2).
///
/// # Errors
/// [`Report`] (`myc-io` / `myc-manifest` / `myc-build`) on a read failure, a malformed manifest, or a
/// refused build input.
pub fn build(manifest_path: &Path) -> Result<(Spore, String), Report> {
    let (manifest, project_dir) = load_manifest(manifest_path)?;
    let spore = build_spore(&manifest, &project_dir).map_err(|e| {
        Report::new("myc-build", e.to_string(), e.exit_code())
            .at(project_dir.display().to_string())
            .help("declare the [surface].exports, add a `.myc` source, or pin a dependency `hash` (ADR-003)")
    })?;
    // Compute the descriptor from a borrow, then move `spore` out by value (no clone).
    let descriptor = explain(&spore);
    Ok((spore, descriptor))
}

/// The outcome of [`check_project`]: which nodules type-checked, and the structured failures.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CheckReport {
    /// Source files that parsed and type-checked cleanly.
    pub checked: Vec<String>,
    /// Per-file structured failures (parse or type errors), each with a location (DN-22).
    pub failures: Vec<Report>,
}

impl CheckReport {
    /// Whether every checked file passed.
    #[must_use]
    pub fn ok(&self) -> bool {
        self.failures.is_empty()
    }
}

/// `myc check` — parse and type-check every `.myc` source under the project directory containing
/// `manifest_path`. Each nodule is checked independently (per-nodule scope — honest `Declared`:
/// cross-nodule resolution is the elaborator's job, not re-implemented here). Returns a structured
/// [`CheckReport`]; a parse/type error becomes a located [`Report`] in `failures`, never a panic (G2).
///
/// # Errors
/// [`Report`] (`myc-io`) only when the source tree cannot be walked; per-file check failures are
/// carried in the returned [`CheckReport`], not as an `Err`.
pub fn check_project(manifest_path: &Path) -> Result<CheckReport, Report> {
    let (_, project_dir) = load_manifest(manifest_path)?;
    let sources =
        mycelium_cli_common::walk_myc(&project_dir).map_err(|e| Report::new("myc-io", e, 66))?;
    let mut report = CheckReport::default();
    for path in sources {
        let rel = path
            .strip_prefix(&project_dir)
            .unwrap_or(&path)
            .display()
            .to_string();
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                report
                    .failures
                    .push(Report::new("myc-io", format!("{rel}: {e}"), 66).at(rel.clone()));
                continue;
            }
        };
        match parse(&text) {
            Err(pe) => report.failures.push(
                Report::new("myc-parse", pe.message.clone(), 65)
                    .at(format!("{rel}:{}:{}", pe.pos.line, pe.pos.col))
                    .help("fix the syntax error at the indicated position"),
            ),
            Ok(nodule) => match check_nodule(&nodule) {
                Err(ce) => report.failures.push(
                    Report::new("myc-check", ce.to_string(), 65)
                        .at(rel.clone())
                        .help("resolve the type error reported above"),
                ),
                Ok(_env) => report.checked.push(rel),
            },
        }
    }
    Ok(report)
}

/// `myc run` — **not yet wired** (honest, never-silent). The project→interpreter execution pipeline
/// is later work; this returns an actionable [`Report`] rather than a stub that silently does nothing
/// (VR-5 / G2). The interpreter ([`mycelium_interp`](mycelium-interp)) evaluates Core IR, but the
/// surface-project→run path is not assembled in v0.
///
/// # Errors
/// Always returns [`Report`] (`myc-run-unwired`, exit 70) — `run` has no honest success path yet.
pub fn run(_manifest_path: &Path) -> Result<(), Report> {
    Err(Report::new(
        "myc-run-unwired",
        "running a phylum is not yet wired into `myc`",
        70,
    )
    .help(
        "the project→interpreter execution pipeline is later work; today use `myc check` to \
         type-check and `myc build` to package the spore",
    ))
}

/// The outcome of a single nodule-component parse in [`stream_parse`].
///
/// Each entry corresponds to one nodule-component extracted from the stream.
/// `Ok(n)` records its 1-based component number on success; `Err(report)` carries the structured
/// diagnostic for a malformed component — never silent, never skipped (G2 / M-820).
pub type StreamComponent = Result<usize, Report>;

/// `myc --stream` — parse a `;`-delimited Mycelium component stream from `reader` (M-820 / DN-57).
///
/// ## Streaming semantics (`Declared` for the I/O strategy; `Empirical` for the split)
/// **v0 is whole-input-buffered for I/O.** The entire reader is read into a `String` first, then
/// the source is **lexer-split** into per-nodule components and each component is parsed
/// independently. This bounds the *parse* state to one component at a time (the per-component parse
/// is a [`mycelium_l1::parse`] call on the component's source slice, not the whole input), but the
/// *I/O* is fully buffered. True per-`;`-component **incremental** I/O would require the L1 lexer to
/// expose a resumable/incremental token-stream API (one does not exist yet); that is flagged as
/// future work (`Declared`). The *split* itself is `Empirical` — it is token-accurate (see below)
/// and tested, including comment-/string-safety.
///
/// ## Component granularity — token-driven, comment-safe (DN-57 §2)
/// The source is tokenized once via [`mycelium_l1::lexer::lex`]; the token stream is then segmented
/// at [`mycelium_l1::token::Tok::Nodule`] keyword tokens. Each "component" is a complete Mycelium
/// nodule block — from its `nodule` header token through all its `;`-terminated
/// ([`Tok::Semi`](mycelium_l1::token::Tok::Semi)) items, up to (but not including) the next `nodule`
/// header token. Crucially this is **not** a raw-text keyword scan: a `nodule` or `;` appearing
/// inside a `//` comment (or a future string literal) is **not** a `Tok::Nodule`/`Tok::Semi` token,
/// so it can never cause a mis-split (DN-57 §2: "the end-of-component is a *token*, not the *absence*
/// of more tokens" — a streaming parser must not scan ahead for the next item-opening *keyword text*).
///
/// ## Never-silent error contract (G2)
/// - A **lex** failure surfaces as an outer `Err(Report)` (`myc-stream-lex`) with the source
///   position — a lexically invalid stream is never silently truncated.
/// - A malformed component yields a [`Report`] (`myc-stream-parse`) with the 1-based component
///   index, the parse-error position within that component, and an actionable `help:` line. The
///   remaining components are still attempted — one bad component does not abort the stream.
/// - A component whose last token before the next `nodule`/EOF is **not** `Tok::Semi` is an
///   unterminated component: an explicit `myc-stream-eof` error (DN-57 §3.1 — mandatory `;`), never
///   a silent partial accept.
/// - An entirely empty stream (no tokens) or one with no `nodule` header is an explicit
///   `myc-stream-empty` / per-component error — never silently succeeded.
///
/// ## I/O errors
/// An I/O failure reading `reader` is returned as an outer `Err(Report)` (`myc-stream-io`, exit 66)
/// before any parse results.
///
/// ## Return value
/// Returns `Ok(Vec<StreamComponent>)` — one entry per component. `Err(report)` entries are
/// per-component parse / unterminated failures; `Ok(n)` entries confirm success. The outer `Result`
/// carries I/O, lex, or empty-stream errors that prevent any per-component parsing.
///
/// # Errors
/// Returns `Err(Report)` for a fatal I/O failure on `reader`, a lex failure, or an empty stream.
pub fn stream_parse(
    mut reader: impl StdRead,
    source_name: &str,
) -> Result<Vec<StreamComponent>, Report> {
    // --- Step 1: read the entire input (v0: full-input buffering; `Declared` limitation) ---
    let mut src = String::new();
    reader.read_to_string(&mut src).map_err(|e| {
        Report::new("myc-stream-io", format!("{source_name}: {e}"), 66)
            .help("check that the input source is readable and produces valid UTF-8")
    })?;

    // --- Step 2: lex once (never-silent: a lex error surfaces explicitly, G2) ---
    let toks = lex(&src).map_err(|ParseError { pos, message }| {
        Report::new(
            "myc-stream-lex",
            format!("`{source_name}` failed to lex: {message}"),
            65,
        )
        .at(format!("{source_name}:{}:{}", pos.line, pos.col))
        .help("fix the lexically invalid token at the indicated position")
    })?;

    // --- Step 3: segment the token stream at `nodule` header tokens (comment-safe by construction) ---
    // A `nodule`/`;` inside a `//` comment is never a `Tok::Nodule`/`Tok::Semi`, so this split is
    // immune to comment/string-literal mis-splits (DN-57 §2).
    let segments = segment_nodule_components(&toks);

    if segments.is_empty() {
        // No `nodule` header token — either an empty stream (only `Eof`) or content with no header.
        // Distinguish: a stream that is only `Eof` is empty; otherwise it is one malformed component.
        let non_eof = toks.iter().any(|s| s.tok != Tok::Eof);
        if !non_eof {
            return Err(Report::new(
                "myc-stream-empty",
                format!("`{source_name}` is empty — no components to parse"),
                65,
            )
            .help(
                "a Mycelium stream must contain at least one `nodule`-headed component (DN-57); \
                 check that the input is non-empty",
            ));
        }
        // Tokens present but no `nodule` header — surface as one explicit malformed component.
        return Ok(vec![parse_component(src.trim(), 1, source_name)]);
    }

    // --- Step 4: per-segment, slice the source and parse (or report unterminated) ---
    // Build a line-start byte index so a token `Pos` (1-based line/col) maps to a byte offset.
    let line_starts = line_start_offsets(&src);
    let mut results: Vec<StreamComponent> = Vec::with_capacity(segments.len());

    for (comp_idx, seg) in segments.iter().enumerate() {
        let one_based = comp_idx + 1;
        // The segment's source slice runs from its first token's byte offset to its end byte offset.
        let start_byte = pos_to_byte(&line_starts, &src, seg.start_pos);
        let end_byte = seg
            .end_pos
            .map_or(src.len(), |p| pos_to_byte(&line_starts, &src, p));
        let slice = src.get(start_byte..end_byte).unwrap_or("").trim();

        if !seg.terminated {
            // Never-silent: the last token before the boundary is not `Tok::Semi` (DN-57 §3.1).
            results.push(Err(Report::new(
                "myc-stream-eof",
                format!(
                    "component {one_based} in `{source_name}` is unterminated: \
                     its last item has no `;` terminator before the next component / EOF"
                ),
                65,
            )
            .at(format!(
                "{source_name}:{one_based}:{}:{}",
                seg.start_pos.line, seg.start_pos.col
            ))
            .help(
                "every Mycelium component must end with `;` after its last item (DN-57 §3.1); \
                 add `;` at the end of the component",
            )));
        } else {
            results.push(parse_component(slice, one_based, source_name));
        }
    }

    Ok(results)
}

/// One lexer-segmented nodule-component: where its `nodule` header token starts, where the next
/// component (or EOF) starts, and whether its final token is the mandatory `;` terminator.
struct NoduleSegment {
    /// Source position of the segment's opening `nodule` token (1-based line/col).
    start_pos: Pos,
    /// Source position of the *next* segment's opening `nodule` token, or `None` for the last
    /// segment (which runs to end-of-source).
    end_pos: Option<Pos>,
    /// Whether the last non-`Eof` token of this segment is `Tok::Semi` (DN-57 mandatory terminator).
    terminated: bool,
}

/// Segment a token stream into per-nodule components at `Tok::Nodule` header boundaries.
///
/// Each segment runs from one `Tok::Nodule` token up to (but not including) the next `Tok::Nodule`
/// token (or `Tok::Eof`). A segment is `terminated` iff its last non-`Eof` token is `Tok::Semi` —
/// the DN-57 mandatory component terminator. Comment-safe by construction: comments are never in the
/// token stream, so a `nodule`/`;` inside a comment cannot start or terminate a segment.
///
/// Guarantee: `Empirical` — validated by the stream tests (including comment-/string-safety).
fn segment_nodule_components(toks: &[Spanned]) -> Vec<NoduleSegment> {
    // Collect the indices of every `nodule` header token.
    let nodule_idxs: Vec<usize> = toks
        .iter()
        .enumerate()
        .filter(|(_, s)| s.tok == Tok::Nodule)
        .map(|(i, _)| i)
        .collect();

    let mut segments = Vec::with_capacity(nodule_idxs.len());
    for (n, &start_idx) in nodule_idxs.iter().enumerate() {
        // The token range of this segment: [start_idx, next_nodule_idx) — or to the end otherwise.
        let next_nodule_idx = nodule_idxs.get(n + 1).copied();
        let end_idx = next_nodule_idx.unwrap_or(toks.len());

        // The boundary position (start of the next component) — `None` for the last segment.
        let end_pos = next_nodule_idx.map(|i| toks[i].pos);

        // Terminated iff the last non-`Eof` token in [start_idx, end_idx) is `Tok::Semi`.
        let terminated = toks[start_idx..end_idx]
            .iter()
            .rev()
            .find(|s| s.tok != Tok::Eof)
            .is_some_and(|s| s.tok == Tok::Semi);

        segments.push(NoduleSegment {
            start_pos: toks[start_idx].pos,
            end_pos,
            terminated,
        });
    }
    segments
}

/// Byte offsets of the start of each 1-based source line (`line_starts[0]` = 0 = start of line 1).
/// Used to map a token [`Pos`](mycelium_l1::token::Pos) (1-based line/col) to a byte offset.
fn line_start_offsets(src: &str) -> Vec<usize> {
    let mut starts = vec![0usize];
    for (i, b) in src.bytes().enumerate() {
        if b == b'\n' {
            starts.push(i + 1);
        }
    }
    starts
}

/// Map a 1-based `Pos` (line/col) to a byte offset in `src`, using a precomputed `line_starts`.
///
/// The lexer counts `col` in characters (1-based), so we walk `col - 1` chars from the line start to
/// land on the correct byte offset (handles multi-byte UTF-8). A position past end-of-line clamps to
/// the source length — never panics (G2).
fn pos_to_byte(line_starts: &[usize], src: &str, pos: Pos) -> usize {
    let line_idx = (pos.line as usize).saturating_sub(1);
    let Some(&line_byte) = line_starts.get(line_idx) else {
        return src.len();
    };
    // Walk `col - 1` characters from the line start.
    let col_offset = (pos.col as usize).saturating_sub(1);
    let rest = &src[line_byte..];
    match rest.char_indices().nth(col_offset) {
        Some((byte_in_line, _)) => line_byte + byte_in_line,
        None => {
            // `col` is past the last char of the line — clamp to the line end (next line start - 1)
            // or the source length for the final line.
            line_starts
                .get(line_idx + 1)
                .map_or(src.len(), |&next| next.saturating_sub(1))
        }
    }
}

/// Parse a single component's source slice as a Mycelium nodule.
///
/// Returns `Ok(component_idx)` on success; `Err(Report)` with a fully-located diagnostic on any
/// parse failure (G2: never silent, never panics — backed by [`mycelium_l1::parse`]'s own contract).
fn parse_component(text: &str, component_idx: usize, source_name: &str) -> StreamComponent {
    match parse(text) {
        Ok(_nodule) => Ok(component_idx),
        Err(ParseError { pos, message }) => Err(Report::new(
            "myc-stream-parse",
            format!("component {component_idx} in `{source_name}` failed to parse: {message}"),
            65,
        )
        .at(format!(
            "{source_name}:{component_idx}:{}:{}",
            pos.line, pos.col
        ))
        .help(
            "fix the syntax error at the indicated component:line:col position; \
             each component must be a valid Mycelium nodule terminated with `;`",
        )),
    }
}

/// The result of [`stream_parse`] summarised for the CLI.
///
/// Parallel to [`CheckReport`] but for streaming input rather than project files.
/// Carries the per-component results and the source name for display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamReport {
    /// How many components parsed cleanly.
    pub parsed_ok: usize,
    /// How many components failed to parse (or were unterminated).
    pub parsed_err: usize,
    /// The structured failures, each located to a component.
    pub failures: Vec<Report>,
    /// Human-readable source label (e.g. `"<stdin>"` or a file path).
    pub source_name: String,
}

impl StreamReport {
    /// Whether every component parsed successfully.
    #[must_use]
    pub fn ok(&self) -> bool {
        self.failures.is_empty()
    }
}

/// Drive [`stream_parse`] and collect results into a [`StreamReport`].
///
/// Converts the per-component `Vec<StreamComponent>` from [`stream_parse`] into a summary
/// suitable for CLI display and test assertions.
///
/// # Errors
/// Returns `Err(Report)` for an I/O failure or an empty stream (no components found).
pub fn run_stream_parse(reader: impl StdRead, source_name: &str) -> Result<StreamReport, Report> {
    let components = stream_parse(reader, source_name)?;
    let mut report = StreamReport {
        parsed_ok: 0,
        parsed_err: 0,
        failures: Vec::new(),
        source_name: source_name.to_owned(),
    };
    for result in components {
        match result {
            Ok(_) => report.parsed_ok += 1,
            Err(r) => {
                report.parsed_err += 1;
                report.failures.push(r);
            }
        }
    }
    Ok(report)
}

// --- internals ---------------------------------------------------------------------------------

/// Load + parse the manifest at `manifest_path`, returning it with the project directory.
fn load_manifest(manifest_path: &Path) -> Result<(mycelium_proj::Manifest, PathBuf), Report> {
    let text = std::fs::read_to_string(manifest_path).map_err(|e| {
        Report::new("myc-io", format!("{}: {e}", manifest_path.display()), 66)
            .help("run `myc` from a project directory, or pass the manifest path")
    })?;
    let manifest = parse_manifest(&text).map_err(|e| {
        Report::new("myc-manifest", e.to_string(), 2).at(manifest_path.display().to_string())
    })?;
    let project_dir = manifest_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    Ok((manifest, project_dir))
}

/// Validate an `init` name: `[a-z][a-z0-9_]*`. A bad name is refused, never normalized (G2).
fn validate_name(name: &str) -> Result<(), Report> {
    let bad = || {
        Report::new(
            "myc-init-name",
            format!("{name:?} is not a valid phylum name"),
            64,
        )
        .help("use a lowercase identifier: a letter then letters/digits/underscores, e.g. `acme_geometry`")
    };
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() => {}
        _ => return Err(bad()),
    }
    if !chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') {
        return Err(bad());
    }
    Ok(())
}

/// Write `content` to `path`, refusing to clobber an existing file (G2).
fn write_new(path: &Path, content: &str) -> Result<(), Report> {
    if path.exists() {
        return Err(Report::new(
            "myc-init-exists",
            format!("{} already exists", path.display()),
            66,
        ));
    }
    std::fs::write(path, content)
        .map_err(|e| Report::new("myc-io", format!("{}: {e}", path.display()), 66))
}

/// The scaffolded `mycelium-proj.toml` for `name`.
fn scaffold_manifest(name: &str) -> String {
    format!(
        "# Scaffolded by `myc init`. A minimal, gate-clean phylum.\n\
         [project]\n\
         name    = \"{name}\"\n\
         kind    = \"phylum\"\n\
         version = \"0.1.0\"\n\
         license = \"MIT\"\n\
         summary = \"{name} — a new Mycelium phylum.\"\n\
         \n\
         [surface]\n\
         exports = [\"{name}\"]\n"
    )
}

/// The scaffolded starter nodule for `name`.
fn scaffold_nodule(name: &str) -> String {
    format!(
        "// nodule: {name}\n\
         // @summary: {name} — scaffolded by `myc init`; replace with your own definitions.\n\
         nodule {name};\n\
         \n\
         fn answer() => Binary{{8}} =\n  \
         0b0010_1010;\n"
    )
}

#[cfg(test)]
mod tests;
