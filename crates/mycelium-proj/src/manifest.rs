//! The **project manifest** `mycelium-proj.toml` (M-359; spec §2) and a **minimal, auditable
//! TOML-subset reader**.
//!
//! The workspace keeps its external dependencies few and vetted (the `/security-review` ethos —
//! only pinned `serde`/`serde_json`/`blake3` in the toolchain crates). Rather than **add** a full
//! TOML crate (a new-dependency decision that is an ADR, not a build detail), this reads the
//! *subset* the manifest needs: `# comments`, `[table]` headers, and
//! single-line `key = value` where a value is a **basic string** (`"…"`), an **array** of values, an
//! **inline table** (`{ k = v, … }`), or a **boolean**. Anything outside the subset (a bare number, a
//! multi-line array, an unknown `[project]` key, an unknown `[project].kind`) is an **explicit** error
//! — never silently dropped or guessed (G2). It is honestly a *subset*, named as one; it is not a
//! conformant TOML parser.
//!
//! Only `[project]` is **typed and validated** in v0 (the fields headers inherit from); the optional
//! `[surface]`/`[dependencies]`/`[toolchain]`/`[spore]` tables are accepted but not interpreted yet
//! (their consumers are M-361). Metadata is **not** identity (ADR-003).

use crate::header::{is_iso_date, is_semver, is_spdx, is_url};

/// The shape of a Mycelium project (spec §2 — `[project].kind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectKind {
    /// A library — a content-addressed `phylum`.
    Phylum,
    /// An executable program.
    Program,
    /// A single-file / small script.
    Script,
}

/// The typed `[project]` table (the v0 closed key set).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    /// `name` — the project name (required).
    pub name: String,
    /// `kind` — `phylum` | `program` | `script` (required).
    pub kind: ProjectKind,
    /// `version` — semver release label.
    pub version: Option<String>,
    /// `license` — SPDX identifier.
    pub license: Option<String>,
    /// `authors`.
    pub authors: Option<Vec<String>>,
    /// `since` — first publication ISO date.
    pub since: Option<String>,
    /// `summary`.
    pub summary: Option<String>,
    /// `repository` — source URL.
    pub repository: Option<String>,
    /// `keywords` — discovery tags.
    pub keywords: Option<Vec<String>>,
    /// `lang` — the surface-language edition this project targets.
    pub lang: Option<String>,
}

/// A parsed `mycelium-proj.toml` (v0: the typed `[project]` table).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    /// The required `[project]` table.
    pub project: Project,
}

/// An explicit manifest error (G2): a syntax error, an out-of-subset construct, or a bad value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestError {
    /// 1-based source line.
    pub line: u32,
    /// What is wrong.
    pub message: String,
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for ManifestError {}

/// The closed v0 `[project]` key set.
const PROJECT_KEYS: &[&str] = &[
    "name",
    "kind",
    "version",
    "license",
    "authors",
    "since",
    "summary",
    "repository",
    "keywords",
    "lang",
];

/// A parsed TOML value (the supported subset).
#[derive(Debug, Clone, PartialEq, Eq)]
enum Val {
    Str(String),
    Arr(Vec<Val>),
    Table(Vec<(String, Val)>),
    Bool(bool),
}

/// Parse a `mycelium-proj.toml` source into a [`Manifest`].
///
/// # Errors
/// Returns [`ManifestError`] on a syntax error, an out-of-subset construct, a missing/unknown
/// `[project]` key, or a malformed value.
pub fn parse_manifest(src: &str) -> Result<Manifest, ManifestError> {
    let mut current: Option<String> = None;
    // Collected `[project]` key→(value, line). Other tables are accepted but not interpreted (v0).
    let mut project_kv: Vec<(String, Val, u32)> = Vec::new();

    for (idx, raw) in src.lines().enumerate() {
        let line_no = (idx + 1) as u32;
        let line = strip_comment(raw).trim();
        if line.is_empty() {
            continue;
        }
        if let Some(table) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            current = Some(table.trim().to_owned());
            continue;
        }
        let (key, rhs) = line.split_once('=').ok_or_else(|| ManifestError {
            line: line_no,
            message: format!("expected `key = value` or `[table]`, got {line:?}"),
        })?;
        let key = key.trim().to_owned();
        let val = parse_value(rhs.trim(), line_no)?;
        if current.as_deref() == Some("project") {
            project_kv.push((key, val, line_no));
        }
        // Other tables: accepted, not interpreted in v0 (their consumers are M-361).
    }

    build_project(project_kv)
}

/// Strip a trailing `#` comment that is **outside** a quoted string (single-line values only).
fn strip_comment(line: &str) -> &str {
    let mut in_str = false;
    for (i, c) in line.char_indices() {
        match c {
            '"' => in_str = !in_str,
            '#' if !in_str => return &line[..i],
            _ => {}
        }
    }
    line
}

fn build_project(kv: Vec<(String, Val, u32)>) -> Result<Manifest, ManifestError> {
    if kv.is_empty() {
        return Err(ManifestError {
            line: 1,
            message: "no `[project]` table — a manifest needs at least `[project]` with `name` and `kind`".to_owned(),
        });
    }
    let mut name = None;
    let mut kind = None;
    let mut version = None;
    let mut license = None;
    let mut authors = None;
    let mut since = None;
    let mut summary = None;
    let mut repository = None;
    let mut keywords = None;
    let mut lang = None;

    for (key, val, line) in kv {
        if !PROJECT_KEYS.contains(&key.as_str()) {
            return Err(ManifestError {
                line,
                message: format!(
                    "unknown `[project]` key `{key}` — the v0 set is closed: {} (G2)",
                    PROJECT_KEYS.join(", ")
                ),
            });
        }
        match key.as_str() {
            "name" => name = Some(as_str(&val, "name", line)?),
            "kind" => kind = Some(as_kind(&as_str(&val, "kind", line)?, line)?),
            "version" => version = Some(checked_str(&val, "version", line, is_semver, "a semver")?),
            "license" => {
                license = Some(checked_str(
                    &val,
                    "license",
                    line,
                    is_spdx,
                    "a recognised SPDX id/expression",
                )?)
            }
            "since" => {
                since = Some(checked_str(
                    &val,
                    "since",
                    line,
                    is_iso_date,
                    "an ISO-8601 date",
                )?)
            }
            "repository" => {
                repository = Some(checked_str(&val, "repository", line, is_url, "a URL")?)
            }
            "summary" => summary = Some(as_str(&val, "summary", line)?),
            "lang" => lang = Some(as_str(&val, "lang", line)?),
            "authors" => authors = Some(as_str_list(&val, "authors", line)?),
            "keywords" => keywords = Some(as_str_list(&val, "keywords", line)?),
            _ => unreachable!("key membership checked above"),
        }
    }

    let project = Project {
        name: name.ok_or_else(|| ManifestError {
            line: 1,
            message: "`[project]` is missing the required `name`".to_owned(),
        })?,
        kind: kind.ok_or_else(|| ManifestError {
            line: 1,
            message: "`[project]` is missing the required `kind` (phylum | program | script)"
                .to_owned(),
        })?,
        version,
        license,
        authors,
        since,
        summary,
        repository,
        keywords,
        lang,
    };
    Ok(Manifest { project })
}

fn as_kind(s: &str, line: u32) -> Result<ProjectKind, ManifestError> {
    match s {
        "phylum" => Ok(ProjectKind::Phylum),
        "program" => Ok(ProjectKind::Program),
        "script" => Ok(ProjectKind::Script),
        other => Err(ManifestError {
            line,
            message: format!("`kind` must be `phylum`, `program`, or `script`, got {other:?} (G2)"),
        }),
    }
}

fn as_str(val: &Val, key: &str, line: u32) -> Result<String, ManifestError> {
    match val {
        Val::Str(s) => Ok(s.clone()),
        _ => Err(ManifestError {
            line,
            message: format!("`{key}` must be a string"),
        }),
    }
}

fn checked_str(
    val: &Val,
    key: &str,
    line: u32,
    ok: fn(&str) -> bool,
    want: &str,
) -> Result<String, ManifestError> {
    let s = as_str(val, key, line)?;
    if ok(&s) {
        Ok(s)
    } else {
        Err(ManifestError {
            line,
            message: format!(
                "`{key}` value {s:?} is not {want} (checked, never fabricated — VR-5)"
            ),
        })
    }
}

fn as_str_list(val: &Val, key: &str, line: u32) -> Result<Vec<String>, ManifestError> {
    match val {
        Val::Arr(items) => items.iter().map(|v| as_str(v, key, line)).collect(),
        _ => Err(ManifestError {
            line,
            message: format!("`{key}` must be an array of strings"),
        }),
    }
}

// --- the minimal value scanner (single-line) ---

fn parse_value(s: &str, line: u32) -> Result<Val, ManifestError> {
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    let v = scan_value(&chars, &mut i, line)?;
    skip_ws(&chars, &mut i);
    if i != chars.len() {
        return Err(ManifestError {
            line,
            message: format!(
                "trailing characters after value: {:?}",
                chars[i..].iter().collect::<String>()
            ),
        });
    }
    Ok(v)
}

fn skip_ws(chars: &[char], i: &mut usize) {
    while *i < chars.len() && chars[*i].is_whitespace() {
        *i += 1;
    }
}

fn scan_value(chars: &[char], i: &mut usize, line: u32) -> Result<Val, ManifestError> {
    skip_ws(chars, i);
    match chars.get(*i) {
        Some('"') => scan_string(chars, i, line).map(Val::Str),
        Some('[') => scan_array(chars, i, line),
        Some('{') => scan_inline_table(chars, i, line),
        Some('t') | Some('f') => scan_bool(chars, i, line),
        Some(c) => Err(ManifestError {
            line,
            message: format!(
                "unsupported value starting with {c:?} — the v0 manifest reader supports strings, \
                 arrays, inline tables, and booleans only (G2; not a full TOML parser)"
            ),
        }),
        None => Err(ManifestError {
            line,
            message: "expected a value after `=`".to_owned(),
        }),
    }
}

fn scan_string(chars: &[char], i: &mut usize, line: u32) -> Result<String, ManifestError> {
    *i += 1; // opening quote
    let mut out = String::new();
    while let Some(&c) = chars.get(*i) {
        match c {
            '"' => {
                *i += 1;
                return Ok(out);
            }
            '\\' => {
                *i += 1;
                match chars.get(*i) {
                    Some('"') => out.push('"'),
                    Some('\\') => out.push('\\'),
                    Some('n') => out.push('\n'),
                    Some('t') => out.push('\t'),
                    Some(other) => {
                        return Err(ManifestError {
                            line,
                            message: format!("unsupported escape `\\{other}` in string"),
                        })
                    }
                    None => break,
                }
                *i += 1;
            }
            _ => {
                out.push(c);
                *i += 1;
            }
        }
    }
    Err(ManifestError {
        line,
        message: "unterminated string (missing closing `\"`)".to_owned(),
    })
}

fn scan_array(chars: &[char], i: &mut usize, line: u32) -> Result<Val, ManifestError> {
    *i += 1; // '['
    let mut items = Vec::new();
    loop {
        skip_ws(chars, i);
        match chars.get(*i) {
            Some(']') => {
                *i += 1;
                return Ok(Val::Arr(items));
            }
            Some(',') => {
                *i += 1;
            }
            None => return Err(ManifestError {
                line,
                message:
                    "unterminated array (missing `]`; multi-line arrays are not in the v0 subset)"
                        .to_owned(),
            }),
            _ => items.push(scan_value(chars, i, line)?),
        }
    }
}

fn scan_inline_table(chars: &[char], i: &mut usize, line: u32) -> Result<Val, ManifestError> {
    *i += 1; // '{'
    let mut pairs = Vec::new();
    loop {
        skip_ws(chars, i);
        match chars.get(*i) {
            Some('}') => {
                *i += 1;
                return Ok(Val::Table(pairs));
            }
            Some(',') => {
                *i += 1;
            }
            None => {
                return Err(ManifestError {
                    line,
                    message: "unterminated inline table (missing `}`)".to_owned(),
                })
            }
            _ => {
                let key = scan_bare_key(chars, i, line)?;
                skip_ws(chars, i);
                if chars.get(*i) != Some(&'=') {
                    return Err(ManifestError {
                        line,
                        message: format!("expected `=` after inline-table key `{key}`"),
                    });
                }
                *i += 1;
                let v = scan_value(chars, i, line)?;
                pairs.push((key, v));
            }
        }
    }
}

fn scan_bare_key(chars: &[char], i: &mut usize, line: u32) -> Result<String, ManifestError> {
    skip_ws(chars, i);
    let mut out = String::new();
    while let Some(&c) = chars.get(*i) {
        if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
            out.push(c);
            *i += 1;
        } else {
            break;
        }
    }
    if out.is_empty() {
        return Err(ManifestError {
            line,
            message: "expected a bare key in inline table".to_owned(),
        });
    }
    Ok(out)
}

fn scan_bool(chars: &[char], i: &mut usize, line: u32) -> Result<Val, ManifestError> {
    let rest: String = chars[*i..].iter().collect();
    if rest.starts_with("true") {
        *i += 4;
        Ok(Val::Bool(true))
    } else if rest.starts_with("false") {
        *i += 5;
        Ok(Val::Bool(false))
    } else {
        Err(ManifestError {
            line,
            message: format!("unrecognised bare token {rest:?} (expected `true`/`false`)"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
# mycelium-proj.toml
[project]
name        = "geometry"          # the project name
kind        = "phylum"
version     = "1.2.0"
license     = "Apache-2.0"
authors     = ["Tyler Zervas", "A. N. Other"]
since       = "2026-01-10"
summary     = "2D/3D geometry primitives and certified swaps."
repository  = "https://github.com/example/geometry"
keywords    = ["geometry", "linear-algebra"]
lang        = "mycelium-0"

[surface]
exports     = ["geometry.shapes"]

[dependencies]
numerics    = { phylum = "numerics", version = "^2", hash = "blake3:abc" }
"#;

    #[test]
    fn the_sample_manifest_parses() {
        let m = parse_manifest(SAMPLE).unwrap();
        assert_eq!(m.project.name, "geometry");
        assert_eq!(m.project.kind, ProjectKind::Phylum);
        assert_eq!(m.project.version.as_deref(), Some("1.2.0"));
        assert_eq!(m.project.license.as_deref(), Some("Apache-2.0"));
        assert_eq!(m.project.authors.as_ref().unwrap().len(), 2);
        assert_eq!(
            m.project.keywords.as_ref().unwrap(),
            &["geometry", "linear-algebra"]
        );
    }

    #[test]
    fn missing_required_fields_are_explicit_errors() {
        assert!(parse_manifest("[project]\nname = \"x\"\n").is_err()); // no kind
        assert!(parse_manifest("[project]\nkind = \"program\"\n").is_err()); // no name
        assert!(parse_manifest("[surface]\nexports = []\n").is_err()); // no [project]
    }

    #[test]
    fn an_unknown_project_key_is_an_explicit_error() {
        let e =
            parse_manifest("[project]\nname=\"x\"\nkind=\"script\"\nfoo=\"bar\"\n").unwrap_err();
        assert!(e.message.contains("unknown `[project]` key"), "{e}");
    }

    #[test]
    fn bad_kind_and_values_are_explicit_errors() {
        assert!(parse_manifest("[project]\nname=\"x\"\nkind=\"library\"\n").is_err());
        assert!(
            parse_manifest("[project]\nname=\"x\"\nkind=\"phylum\"\nlicense=\"Nope\"\n").is_err()
        );
        assert!(
            parse_manifest("[project]\nname=\"x\"\nkind=\"phylum\"\nsince=\"2026/01/10\"\n")
                .is_err()
        );
    }

    #[test]
    fn out_of_subset_constructs_are_explicit_errors() {
        // A bare number is outside the v0 subset — flagged, never silently dropped.
        let e = parse_manifest("[project]\nname=\"x\"\nkind=\"phylum\"\nversion=12\n").unwrap_err();
        assert!(e.message.contains("v0 manifest reader supports"), "{e}");
    }
}
