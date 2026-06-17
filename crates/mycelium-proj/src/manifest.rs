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

/// The typed `[toolchain]` table (M-364): the optional pins the toolchain reads. v0 closed key set:
/// `format` (the formatter spelling/version — a **hard pin**, M-364 §10.3) and `lints` (the lint
/// profile). Unknown keys are explicit errors (G2). Metadata, not identity (ADR-003).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Toolchain {
    /// `format` — the formatter spelling/version pin (e.g. `"mycfmt-0"`). A hard pin: `mycfmt` refuses a
    /// mismatch rather than format with rules the project did not ask for (M-364 §10.3 / G2).
    pub format: Option<String>,
    /// `lints` — the lint profile (e.g. `"strict"`).
    pub lints: Option<String>,
}

/// The typed `[surface]` table (M-368): a phylum's **public exports** — the germination boundary. v0
/// closed key set: `exports` (a list of dotted nodule names). Metadata layer (ADR-003).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Surface {
    /// `exports` — the public nodule names a phylum germinates from.
    pub exports: Vec<String>,
}

/// One `[dependencies]` entry (M-368): another phylum, **content-addressed** (ADR-003) — pinned by
/// `hash` (authoritative) with a human `version` requirement. A `hash`-less dep is an explicit error at
/// publish (the spore build refuses an unpinned, non-reproducible input; G2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dependency {
    /// The dependency's local name (the `[dependencies]` key).
    pub name: String,
    /// `phylum` — the depended-on phylum's name.
    pub phylum: String,
    /// `version` — a human version requirement (e.g. `"^2"`), checked against the pinned hash's version.
    pub version: Option<String>,
    /// `hash` — the content-addressed pin (`blake3:…`); authoritative (ADR-003).
    pub hash: Option<String>,
}

/// The typed `[spore]` table (M-368): how the project publishes as a deployable (ADR-013). v0 closed key
/// set: `include` (what germinates; defaults to the public `[surface]`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SporeConfig {
    /// `include` — what germinates (e.g. `["surface"]`, or explicit nodule names).
    pub include: Vec<String>,
}

/// A parsed `mycelium-proj.toml` (v0: the typed `[project]` table + the optional `[toolchain]`,
/// `[surface]`, `[dependencies]`, and `[spore]` tables).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    /// The required `[project]` table.
    pub project: Project,
    /// The optional `[toolchain]` pins (M-364; M-361). `None` when the table is absent.
    pub toolchain: Option<Toolchain>,
    /// The optional `[surface]` exports (M-368). `None` when the table is absent.
    pub surface: Option<Surface>,
    /// The `[dependencies]` (M-368); empty when the table is absent.
    pub dependencies: Vec<Dependency>,
    /// The optional `[spore]` packaging config (M-368). `None` when the table is absent.
    pub spore: Option<SporeConfig>,
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
    // Collected `[project]` key→(value, line). Other tables stay accepted-but-uninterpreted (v0),
    // except `[toolchain]` — its first consumer is M-364 (`mycfmt` reads `[toolchain].format`).
    let mut project_kv: Vec<(String, Val, u32)> = Vec::new();
    let mut toolchain_kv: Vec<(String, Val, u32)> = Vec::new();
    let mut surface_kv: Vec<(String, Val, u32)> = Vec::new();
    let mut deps_kv: Vec<(String, Val, u32)> = Vec::new();
    let mut spore_kv: Vec<(String, Val, u32)> = Vec::new();
    let (mut saw_toolchain, mut saw_surface, mut saw_spore) = (false, false, false);

    for (idx, raw) in src.lines().enumerate() {
        let line_no = (idx + 1) as u32;
        let line = strip_comment(raw).trim();
        if line.is_empty() {
            continue;
        }
        if let Some(table) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            current = Some(table.trim().to_owned());
            match current.as_deref() {
                Some("toolchain") => saw_toolchain = true,
                Some("surface") => saw_surface = true,
                Some("spore") => saw_spore = true,
                _ => {}
            }
            continue;
        }
        let (key, rhs) = line.split_once('=').ok_or_else(|| ManifestError {
            line: line_no,
            message: format!("expected `key = value` or `[table]`, got {line:?}"),
        })?;
        let key = key.trim().to_owned();
        let val = parse_value(rhs.trim(), line_no)?;
        match current.as_deref() {
            Some("project") => project_kv.push((key, val, line_no)),
            Some("toolchain") => toolchain_kv.push((key, val, line_no)),
            Some("surface") => surface_kv.push((key, val, line_no)),
            Some("dependencies") => deps_kv.push((key, val, line_no)),
            Some("spore") => spore_kv.push((key, val, line_no)),
            // Other tables: accepted, not interpreted in v0.
            _ => {}
        }
    }

    let project = build_project(project_kv)?;
    let toolchain = saw_toolchain
        .then(|| build_toolchain(toolchain_kv))
        .transpose()?;
    let surface = saw_surface.then(|| build_surface(surface_kv)).transpose()?;
    let spore = saw_spore.then(|| build_spore(spore_kv)).transpose()?;
    let dependencies = build_dependencies(deps_kv)?;
    Ok(Manifest {
        project,
        toolchain,
        surface,
        dependencies,
        spore,
    })
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

/// The closed v0 `[toolchain]` key set.
const TOOLCHAIN_KEYS: &[&str] = &["format", "lints"];

fn build_toolchain(kv: Vec<(String, Val, u32)>) -> Result<Toolchain, ManifestError> {
    let mut tc = Toolchain::default();
    for (key, val, line) in kv {
        if !TOOLCHAIN_KEYS.contains(&key.as_str()) {
            return Err(ManifestError {
                line,
                message: format!(
                    "unknown `[toolchain]` key `{key}` — the v0 set is closed: {} (G2)",
                    TOOLCHAIN_KEYS.join(", ")
                ),
            });
        }
        match key.as_str() {
            "format" => tc.format = Some(as_str(&val, "format", line)?),
            "lints" => tc.lints = Some(as_str(&val, "lints", line)?),
            _ => unreachable!("key membership checked above"),
        }
    }
    Ok(tc)
}

fn build_surface(kv: Vec<(String, Val, u32)>) -> Result<Surface, ManifestError> {
    let mut exports = None;
    for (key, val, line) in kv {
        match key.as_str() {
            "exports" => exports = Some(as_str_list(&val, "exports", line)?),
            other => {
                return Err(ManifestError {
                    line,
                    message: format!(
                        "unknown `[surface]` key `{other}` — the v0 set is closed: exports (G2)"
                    ),
                })
            }
        }
    }
    Ok(Surface {
        exports: exports.unwrap_or_default(),
    })
}

fn build_spore(kv: Vec<(String, Val, u32)>) -> Result<SporeConfig, ManifestError> {
    let mut include = None;
    for (key, val, line) in kv {
        match key.as_str() {
            "include" => include = Some(as_str_list(&val, "include", line)?),
            other => {
                return Err(ManifestError {
                    line,
                    message: format!(
                        "unknown `[spore]` key `{other}` — the v0 set is closed: include (G2)"
                    ),
                })
            }
        }
    }
    Ok(SporeConfig {
        include: include.unwrap_or_default(),
    })
}

/// The closed v0 `[dependencies]` inline-table key set.
const DEP_KEYS: &[&str] = &["phylum", "version", "hash"];

fn build_dependencies(kv: Vec<(String, Val, u32)>) -> Result<Vec<Dependency>, ManifestError> {
    let mut deps = Vec::with_capacity(kv.len());
    for (name, val, line) in kv {
        let Val::Table(pairs) = val else {
            return Err(ManifestError {
                line,
                message: format!(
                    "dependency `{name}` must be an inline table \
                     `{{ phylum = \"…\", version = \"…\", hash = \"blake3:…\" }}` (G2)"
                ),
            });
        };
        let (mut phylum, mut version, mut hash) = (None, None, None);
        for (k, v) in &pairs {
            if !DEP_KEYS.contains(&k.as_str()) {
                return Err(ManifestError {
                    line,
                    message: format!(
                        "unknown dependency key `{k}` in `{name}` — the v0 set is closed: {} (G2)",
                        DEP_KEYS.join(", ")
                    ),
                });
            }
            match k.as_str() {
                "phylum" => phylum = Some(as_str(v, "phylum", line)?),
                "version" => version = Some(as_str(v, "version", line)?),
                "hash" => hash = Some(as_str(v, "hash", line)?),
                _ => unreachable!("key membership checked above"),
            }
        }
        deps.push(Dependency {
            phylum: phylum.unwrap_or_else(|| name.clone()),
            name,
            version,
            hash,
        });
    }
    Ok(deps)
}

fn build_project(kv: Vec<(String, Val, u32)>) -> Result<Project, ManifestError> {
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
    Ok(project)
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
    fn the_toolchain_table_is_interpreted() {
        // M-364: `[toolchain].format` is now read (the manifest's first toolchain consumer).
        let m = parse_manifest(SAMPLE).unwrap();
        assert!(m.toolchain.is_none(), "SAMPLE declares no [toolchain]");
        let m = parse_manifest(
            "[project]\nname=\"x\"\nkind=\"phylum\"\n[toolchain]\nformat=\"mycfmt-0\"\nlints=\"strict\"\n",
        )
        .unwrap();
        let tc = m.toolchain.unwrap();
        assert_eq!(tc.format.as_deref(), Some("mycfmt-0"));
        assert_eq!(tc.lints.as_deref(), Some("strict"));
    }

    #[test]
    fn the_surface_dependencies_and_spore_tables_are_interpreted() {
        // M-368: the packaging tables are now typed (first consumer).
        let m = parse_manifest(SAMPLE).unwrap();
        assert_eq!(
            m.surface.as_ref().unwrap().exports,
            vec!["geometry.shapes".to_owned()]
        );
        assert_eq!(m.dependencies.len(), 1);
        let d = &m.dependencies[0];
        assert_eq!(d.name, "numerics");
        assert_eq!(d.phylum, "numerics");
        assert_eq!(d.hash.as_deref(), Some("blake3:abc"));
        assert_eq!(d.version.as_deref(), Some("^2"));

        let m = parse_manifest(
            "[project]\nname=\"x\"\nkind=\"phylum\"\n[spore]\ninclude=[\"surface\"]\n",
        )
        .unwrap();
        assert_eq!(m.spore.unwrap().include, vec!["surface".to_owned()]);
    }

    #[test]
    fn a_malformed_dependency_or_unknown_key_is_explicit() {
        // A non-inline-table dependency is an error.
        assert!(parse_manifest(
            "[project]\nname=\"x\"\nkind=\"phylum\"\n[dependencies]\nfoo=\"bar\"\n"
        )
        .is_err());
        // An unknown dependency key is an error (closed set).
        let e = parse_manifest(
            "[project]\nname=\"x\"\nkind=\"phylum\"\n[dependencies]\nfoo={ phylum=\"f\", oops=\"x\" }\n",
        )
        .unwrap_err();
        assert!(e.message.contains("unknown dependency key"), "{e}");
        // An unknown [surface] key is an error.
        assert!(parse_manifest(
            "[project]\nname=\"x\"\nkind=\"phylum\"\n[surface]\nexprts=[\"a\"]\n"
        )
        .is_err());
    }

    #[test]
    fn an_unknown_toolchain_key_is_an_explicit_error() {
        let e = parse_manifest(
            "[project]\nname=\"x\"\nkind=\"phylum\"\n[toolchain]\nformatt=\"mycfmt-0\"\n",
        )
        .unwrap_err();
        assert!(e.message.contains("unknown `[toolchain]` key"), "{e}");
    }

    #[test]
    fn out_of_subset_constructs_are_explicit_errors() {
        // A bare number is outside the v0 subset — flagged, never silently dropped.
        let e = parse_manifest("[project]\nname=\"x\"\nkind=\"phylum\"\nversion=12\n").unwrap_err();
        assert!(e.message.contains("v0 manifest reader supports"), "{e}");
    }
}
