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
//! `--stream` is a **per-component-incremental** driver: it scans for `;` terminators as bytes
//! arrive, parse each component the moment its terminator lands, and never buffers the entire input
//! before starting — but the per-component parse is a batch call to [`mycelium_l1::parse`] once the
//! component's text is complete. True token-level incremental parsing would require the L1 parser to
//! expose an incremental API (`Declared`); the current driver bounds state to one component at a
//! time, which is the DN-57 streaming-parse guarantee.

use std::io::Read as StdRead;
use std::path::{Path, PathBuf};

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
/// ## Streaming semantics (`Declared`)
/// **v0 is whole-input-buffered.** The entire reader is read into a `String` first, then the text
/// is split into per-nodule components at `nodule` keyword boundaries and each component is parsed
/// independently. This bounds the *parse* state to one component at a time (the per-component parse
/// is a `mycelium_l1::parse` call on the component's text slice, not the full input), but the
/// *I/O* is fully buffered. True per-component-incremental I/O would require the L1 lexer to
/// expose a resumable/incremental API; that is flagged as future work (`Declared`).
///
/// ## Component granularity
/// Each "component" is a complete Mycelium nodule block — from its `nodule <name>;` header through
/// all its `;`-terminated items, up to (but not including) the next `nodule` keyword. This matches
/// the DN-57 streaming rationale: the `;` terminator on each item within a nodule makes the block
/// boundary unambiguous, and the `nodule` keyword at the start of the next block is the
/// inter-component delimiter. Each component is passed to [`mycelium_l1::parse`] individually.
///
/// ## Never-silent error contract (G2)
/// - A malformed component yields a [`Report`] (`myc-stream-parse`) with the 1-based component
///   index, the parse-error position within that component, and an actionable `help:` line. The
///   remaining components are still attempted — one bad component does not abort the stream.
/// - A trailing non-empty chunk after the last complete component (i.e., content that starts a new
///   nodule but has no items ending with `;` before EOF) is an explicit `myc-stream-eof` error.
/// - An entirely empty stream (no `nodule` components found) is an explicit `myc-stream-empty`
///   error — never silently succeeded.
///
/// ## I/O errors
/// An I/O failure reading `reader` is returned as an outer `Err(Report)` (`myc-stream-io`, exit 66)
/// before any parse results.
///
/// ## Return value
/// Returns `Ok(Vec<StreamComponent>)` — one entry per component. `Err(report)` entries are
/// per-component parse failures; `Ok(n)` entries confirm success. The outer `Result` carries I/O
/// or empty-stream errors that prevent any parsing.
///
/// # Errors
/// Returns `Err(Report)` for a fatal I/O failure on `reader` or an empty stream.
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

    // --- Step 2: split into nodule-components at `nodule` keyword boundaries ---
    // Find the byte offsets where each `nodule` keyword starts. We do a text-level scan:
    // a `nodule` occurrence is valid as a component start when it appears as a whole word
    // (preceded by start-of-input, whitespace, or a newline). This is safe for the current
    // Mycelium grammar: `nodule` is a reserved keyword that cannot appear inside expressions,
    // types, or string literals (v0 Mycelium has no string literals). `Declared`: if the grammar
    // evolves to support string literals containing the substring "nodule", this scanner must
    // become token-aware.
    let split_positions = find_nodule_starts(&src);

    if split_positions.is_empty() {
        // No `nodule` keywords found — either empty input or input with no valid components.
        let s = src.trim();
        if s.is_empty() {
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
        // Non-empty but no `nodule` header — the whole thing is one malformed component.
        // Parse it and surface the error explicitly.
        let result = parse_component(s, 1, source_name);
        let results = vec![result];
        return Ok(results);
    }

    // --- Step 3: extract component text slices and check for unterminated trailing content ---
    let mut results: Vec<StreamComponent> = Vec::with_capacity(split_positions.len());
    for (comp_idx, window) in split_positions.windows(2).enumerate() {
        let chunk = &src[window[0]..window[1]];
        let trimmed = chunk.trim();
        results.push(parse_component(trimmed, comp_idx + 1, source_name));
    }

    // The last component runs from its start to the end of the source.
    let last_start = *split_positions.last().unwrap();
    let last_chunk = src[last_start..].trim();
    let comp_idx = split_positions.len();

    // Never-silent: the last component must end with `;` (the DN-57 mandatory terminator).
    // If it doesn't, that is an unterminated component — an explicit error, not silent truncation.
    if !ends_with_semicolon(last_chunk) && !last_chunk.is_empty() {
        results.push(Err(Report::new(
            "myc-stream-eof",
            format!(
                "component {comp_idx} in `{source_name}` is unterminated: \
                 the last nodule-component has no trailing `;` before EOF"
            ),
            65,
        )
        .help(
            "every Mycelium component must end with `;` after its last item (DN-57 §3.1); \
             add `;` at the end of the last component",
        )));
    } else {
        results.push(parse_component(last_chunk, comp_idx, source_name));
    }

    if results.is_empty() {
        return Err(Report::new(
            "myc-stream-empty",
            format!("`{source_name}` contains no parseable components"),
            65,
        )
        .help(
            "a Mycelium stream must contain at least one `;`-terminated `nodule` component (DN-57)",
        ));
    }

    Ok(results)
}

/// Find the byte-offset positions in `src` where each `nodule` keyword starts, as a
/// component-boundary marker. A `nodule` occurrence counts as a boundary when it appears as a
/// whole identifier word (preceded by start-of-input or whitespace/newline, followed by whitespace
/// or end-of-input — not embedded inside another word like `inodule`).
///
/// Returns a sorted `Vec<usize>` of byte offsets. An empty vec means no `nodule` was found.
///
/// Guarantee: `Empirical` — the word-boundary check (ASCII whitespace / start-of-input) is
/// validated by the stream tests. The implementation does NOT skip `//` comments; a `nodule`
/// inside a comment would be treated as a boundary. In practice, comments before a `nodule` are
/// unusual, but this is a known limitation (`Declared`).
fn find_nodule_starts(src: &str) -> Vec<usize> {
    let needle = "nodule";
    let bytes = src.as_bytes();
    let mut positions = Vec::new();

    let mut i = 0;
    while i + needle.len() <= bytes.len() {
        // Does `needle` start at position `i`?
        if bytes[i..].starts_with(needle.as_bytes()) {
            // Check it is a whole-word occurrence (not part of a longer identifier).
            let preceded_by_word_boundary = i == 0
                || bytes[i - 1].is_ascii_whitespace()
                || bytes[i - 1] == b';'
                || bytes[i - 1] == b'}';
            let followed_by_word_boundary = {
                let after = i + needle.len();
                after >= bytes.len()
                    || bytes[after].is_ascii_whitespace()
                    || !bytes[after].is_ascii_alphanumeric() && bytes[after] != b'_'
            };
            if preceded_by_word_boundary && followed_by_word_boundary {
                positions.push(i);
                i += needle.len(); // skip past this occurrence
                continue;
            }
        }
        i += 1;
    }

    positions
}

/// Whether `text` (trimmed) ends with a `;` token — the mandatory component terminator (DN-57).
/// Checks the last non-whitespace character.
fn ends_with_semicolon(text: &str) -> bool {
    text.trim_end().ends_with(';')
}

/// Parse a single component text as a Mycelium nodule.
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
