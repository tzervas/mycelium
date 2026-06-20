//! Ingestion of the **LLM-harness** report (`tools/llm-harness/`) — read, never run. The language-
//! leverage data (KC-2 / SC-5b: per-validation quality + latency + token cost) sits in the unified
//! report alongside the execution-backend data, so both halves of "what are we getting out of this
//! lang" are in one place.
//!
//! **Honesty:** the harness marks a fixture run `mode: "mock"` (and each result `status: "mock-PASS"`);
//! a real model run is `mode: "real"`/`"server"`. This module *preserves* that label — a synthetic
//! sample is surfaced as SYNTHETIC and never presented as evidence of real model quality (VR-5, and
//! the harness's own V-03 rule). We bind to the harness's documented schema
//! (`tools/llm-harness/harness.py::_write_json_report`); unknown `detail` fields are kept opaque.

use std::path::Path;

use serde::Deserialize;

/// A parsed LLM-harness report (the subset of the schema the unified report needs; extra fields are
/// ignored, and `detail` is kept as opaque JSON so the binding does not over-fit the harness).
#[derive(Debug, Clone, Deserialize)]
pub struct LlmReport {
    /// Harness identifier (expected `"mycelium-llm-validation"`).
    pub harness: String,
    /// Harness schema version (e.g. `"0.1.0"`).
    pub version: String,
    /// The run id (an ISO-8601-ish `YYYYMMDDTHHMMSSZ` stamp).
    pub run_id: String,
    /// The run mode: `"mock"` (synthetic fixtures), `"real"` (llama-cli), `"server"` (llama.cpp
    /// HTTP), or `"skip"` (model unavailable). This is the primary synthetic/real discriminator.
    pub mode: String,
    /// The honesty posture block (lattice, allowed model tags, VR-5 rule text).
    pub honesty_posture: HonestyPosture,
    /// The roll-up summary.
    pub summary: Summary,
    /// Per-validation results (V-01..V-04 and any further checks).
    pub results: Vec<ValidationResult>,
}

/// The honesty posture the harness stamps into every report.
#[derive(Debug, Clone, Deserialize)]
pub struct HonestyPosture {
    /// Always true for the harness (it is never-silent).
    pub never_silent: bool,
    /// The guarantee lattice, strongest-first (`["Exact","Proven","Empirical","Declared"]`).
    pub guarantee_lattice: Vec<String>,
    /// The tags a model-derived claim is allowed to carry (`["Declared","Empirical"]`).
    pub model_allowed_tags: Vec<String>,
    /// The VR-5 rule text.
    pub vr5_rule: String,
}

/// The report roll-up.
#[derive(Debug, Clone, Deserialize)]
pub struct Summary {
    /// `"PASS"` | `"FAIL"` | `"MOCK"` | `"INCONCLUSIVE"`.
    pub overall: String,
    /// Total validations.
    pub total: u32,
    /// Count of real PASS.
    pub pass: u32,
    /// Count of fixture (mock) PASS — never counted as real-quality evidence.
    pub mock_pass: u32,
    /// Count of SKIP.
    pub skip: u32,
    /// Count of FAIL.
    pub fail: u32,
    /// Process exit code (0 if no FAILs).
    pub exit_code: i32,
    /// The run mode (duplicated here by the harness).
    pub mode: String,
    /// The model path / server URL, or null.
    pub model: Option<String>,
}

/// One validation result. `detail` is kept opaque (its shape varies per validation); the latency /
/// token fields the unified report shows are pulled from it defensively in [`ValidationResult::wall_seconds`]
/// and [`ValidationResult::token_counts`].
#[derive(Debug, Clone, Deserialize)]
pub struct ValidationResult {
    /// The validation id (e.g. `"V-04-latency-tokens"`).
    pub id: String,
    /// `"PASS"` | `"FAIL"` | `"SKIP"` | `"mock-PASS"`.
    pub status: String,
    /// The honest guarantee tag (`"Empirical"` | `"Declared"` | null).
    pub guarantee_tag: Option<String>,
    /// A human-readable one-line summary.
    pub message: String,
    /// The validation-specific detail object (opaque).
    #[serde(default)]
    pub detail: serde_json::Value,
}

impl ValidationResult {
    /// The wall-clock latency this validation recorded, in seconds, if present in `detail`
    /// (`detail.wall_seconds`, or the V-01 `run_a_wall_seconds`). `None` when the validation carries
    /// no latency (or it is the mock sentinel `0.0`, which we surface as `Some(0.0)` so the report can
    /// label it synthetic rather than silently dropping it).
    #[must_use]
    pub fn wall_seconds(&self) -> Option<f64> {
        self.detail
            .get("wall_seconds")
            .and_then(serde_json::Value::as_f64)
            .or_else(|| {
                self.detail
                    .get("run_a_wall_seconds")
                    .and_then(serde_json::Value::as_f64)
            })
    }

    /// The (prompt, generated) token counts this validation recorded, if present
    /// (`detail.token_counts.{prompt,generated}`). Either side may be `None`.
    #[must_use]
    pub fn token_counts(&self) -> Option<(Option<u64>, Option<u64>)> {
        let tc = self.detail.get("token_counts")?;
        let prompt = tc.get("prompt").and_then(serde_json::Value::as_u64);
        let generated = tc.get("generated").and_then(serde_json::Value::as_u64);
        Some((prompt, generated))
    }

    /// Whether this result is a fixture (mock) result, not real-model evidence.
    #[must_use]
    pub fn is_mock(&self) -> bool {
        self.status == "mock-PASS"
            || self.detail.get("mode").and_then(serde_json::Value::as_str) == Some("mock")
    }
}

impl LlmReport {
    /// `true` when this report is a SYNTHETIC fixture run (no real model) — the primary honesty gate.
    /// Driven by `mode == "mock"` (and corroborated by `summary.overall == "MOCK"`). The unified
    /// report MUST label such a report synthetic.
    #[must_use]
    pub fn is_synthetic(&self) -> bool {
        self.mode == "mock" || self.summary.overall == "MOCK"
    }

    /// A one-line provenance string for the unified report header.
    #[must_use]
    pub fn provenance(&self) -> String {
        let kind = if self.is_synthetic() {
            "SYNTHETIC (fixture; not real model quality)"
        } else {
            "real model run"
        };
        format!(
            "{} v{} — run {} — mode={} — {} ({} validations: {} pass / {} mock-pass / {} skip / {} fail)",
            self.harness,
            self.version,
            self.run_id,
            self.mode,
            kind,
            self.summary.total,
            self.summary.pass,
            self.summary.mock_pass,
            self.summary.skip,
            self.summary.fail,
        )
    }

    /// Parse a report from JSON text. Errors are explicit (a malformed report is loud, not skipped).
    pub fn from_json(text: &str) -> Result<Self, LlmIngestError> {
        serde_json::from_str(text).map_err(|e| LlmIngestError::Parse(e.to_string()))
    }

    /// Read + parse a report from a file path.
    pub fn from_path(path: &Path) -> Result<Self, LlmIngestError> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| LlmIngestError::Io(format!("{}: {e}", path.display())))?;
        Self::from_json(&text)
    }

    /// Find the **newest** report under a harness reports directory (`*-report.json`, lexicographic
    /// max — the timestamped names sort chronologically), if any. Returns `Ok(None)` when the
    /// directory has no report (the caller then falls back to the committed synthetic sample).
    pub fn newest_in_dir(dir: &Path) -> Result<Option<std::path::PathBuf>, LlmIngestError> {
        if !dir.is_dir() {
            return Ok(None);
        }
        let mut newest: Option<std::path::PathBuf> = None;
        for entry in std::fs::read_dir(dir).map_err(|e| LlmIngestError::Io(e.to_string()))? {
            let entry = entry.map_err(|e| LlmIngestError::Io(e.to_string()))?;
            let path = entry.path();
            let is_report = path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with("-report.json"));
            if is_report {
                match &newest {
                    Some(cur) if path <= *cur => {}
                    _ => newest = Some(path),
                }
            }
        }
        Ok(newest)
    }
}

/// A never-silent ingestion error.
#[derive(Debug)]
pub enum LlmIngestError {
    /// The report file could not be read.
    Io(String),
    /// The report JSON could not be parsed against the schema.
    Parse(String),
}

impl std::fmt::Display for LlmIngestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmIngestError::Io(m) => write!(f, "llm-report I/O error: {m}"),
            LlmIngestError::Parse(m) => write!(f, "llm-report parse error: {m}"),
        }
    }
}

impl std::error::Error for LlmIngestError {}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal but faithful synthetic sample matching the harness schema (the committed
    /// `tools/llm-harness/reports/*-report.json` shape). Used so the unit test is fully offline +
    /// deterministic and does not depend on a path outside the crate.
    const SAMPLE: &str = r#"{
      "harness": "mycelium-llm-validation",
      "version": "0.1.0",
      "run_id": "20260617T182214Z",
      "mode": "mock",
      "timestamp_utc": "20260617T182214Z",
      "honesty_posture": {
        "never_silent": true,
        "guarantee_lattice": ["Exact", "Proven", "Empirical", "Declared"],
        "model_allowed_tags": ["Declared", "Empirical"],
        "vr5_rule": "Model-derived claims are Empirical or Declared — NEVER Proven or Exact."
      },
      "summary": {
        "overall": "MOCK", "total": 4, "pass": 1, "mock_pass": 3,
        "skip": 0, "fail": 0, "exit_code": 0, "mode": "mock", "model": null
      },
      "results": [
        {
          "id": "V-01-determinism", "status": "mock-PASS", "guarantee_tag": "Declared",
          "message": "[MOCK] Determinism simulated with fixture.",
          "detail": {"mode": "mock", "matched": true}
        },
        {
          "id": "V-04-latency-tokens", "status": "mock-PASS", "guarantee_tag": "Declared",
          "message": "[MOCK] latency simulated.",
          "detail": {"mode": "mock", "wall_seconds": 0.0,
                     "token_counts": {"prompt": 12, "generated": 7, "note": "Declared"}}
        }
      ]
    }"#;

    #[test]
    fn parses_the_synthetic_sample_and_marks_it_synthetic() {
        let r = LlmReport::from_json(SAMPLE).expect("parses");
        assert_eq!(r.harness, "mycelium-llm-validation");
        assert_eq!(r.version, "0.1.0");
        assert_eq!(r.mode, "mock");
        assert!(
            r.is_synthetic(),
            "a mock-mode report must be flagged synthetic"
        );
        assert!(
            r.provenance().contains("SYNTHETIC"),
            "provenance must surface the synthetic label: {}",
            r.provenance()
        );
        assert_eq!(r.results.len(), 2);
        assert!(r.honesty_posture.never_silent);
    }

    #[test]
    fn pulls_latency_and_token_counts_from_detail() {
        let r = LlmReport::from_json(SAMPLE).unwrap();
        let v4 = r
            .results
            .iter()
            .find(|v| v.id == "V-04-latency-tokens")
            .unwrap();
        assert_eq!(
            v4.wall_seconds(),
            Some(0.0),
            "the mock sentinel 0.0 is surfaced, not dropped"
        );
        assert_eq!(v4.token_counts(), Some((Some(12), Some(7))));
        assert!(v4.is_mock(), "a mock-PASS result must be flagged mock");
    }

    #[test]
    fn a_real_report_is_not_flagged_synthetic() {
        let real = SAMPLE
            .replace("\"mode\": \"mock\"", "\"mode\": \"real\"")
            .replace("\"overall\": \"MOCK\"", "\"overall\": \"PASS\"");
        let r = LlmReport::from_json(&real).unwrap();
        assert!(
            !r.is_synthetic(),
            "a real-mode report must NOT be flagged synthetic"
        );
        assert!(r.provenance().contains("real model run"));
    }

    #[test]
    fn malformed_json_is_an_explicit_error_not_a_silent_skip() {
        let err = LlmReport::from_json("{ not json").unwrap_err();
        assert!(matches!(err, LlmIngestError::Parse(_)));
    }

    #[test]
    fn the_committed_harness_sample_if_present_parses() {
        // If the real committed sample is reachable from the workspace, it must parse against the
        // schema we bind to (a guard that our binding stays faithful to the harness). Skips silently
        // (Ok) when the path is not reachable from the test's CWD — it is not crate-local.
        let candidates = [
            "../../tools/llm-harness/reports/20260617T182214Z-report.json",
            "tools/llm-harness/reports/20260617T182214Z-report.json",
        ];
        for c in candidates {
            let p = Path::new(c);
            if p.is_file() {
                let r = LlmReport::from_path(p)
                    .unwrap_or_else(|e| panic!("the committed harness sample must parse: {e}"));
                assert_eq!(r.harness, "mycelium-llm-validation");
                assert!(r.is_synthetic(), "the committed sample is the mock fixture");
                return;
            }
        }
        // Not reachable from CWD — fine; the SAMPLE-based tests above cover the binding.
    }
}
