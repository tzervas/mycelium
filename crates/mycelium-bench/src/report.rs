//! Deterministic **report emission** — markdown (human) + JSON (machine), the dual projection (G11).
//! The report carries: run metadata + caveats, the per-backend/per-case numbers, the explicit
//! WIN/LOSS/REGRESSION table, an explicit **"where we're losing"** section (capability + correctness +
//! speed losses), and the ingested LLM-harness section.
//!
//! **Honesty stamped in:** the harness *plumbing* is `Empirical` (test-verified). Every measured
//! number is `Empirical` (with its trial accounting); a capability loss / skip is `Declared`. There
//! is **no pre-written performance target** (VR-5): the verdicts are whatever was measured. The
//! microbench caveats (warmup, process-spawn cost, debug-vs-release) are stated, not buried.

use std::fmt::Write as _;

use crate::backend::Backend;
use crate::llm::LlmReport;
use crate::measure::RunRecord;
use crate::verdict::{Verdict, NEUTRAL_BAND};

/// Everything the report needs: the run record, optional ingested LLM-harness report, and run
/// metadata. Serializable verbatim to the JSON projection.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Report {
    /// Schema/tool identity.
    pub tool: &'static str,
    /// The build profile the numbers were taken under (`release` is the only honest one).
    pub profile: &'static str,
    /// Whether the `mlir-dialect` feature was compiled in (affects which backends could run).
    pub mlir_dialect_feature: bool,
    /// A short note on the host (best-effort; for provenance only).
    pub host_note: String,
    /// The honesty posture: the lattice + the rule that governs the verdict tags.
    pub honesty: Honesty,
    /// The neutral band half-width used to classify speed (reified — no black box).
    pub neutral_band: f64,
    /// The execution-backend run.
    pub run: RunRecord,
    /// The ingested LLM-harness section (provenance + per-validation rows), if a report was found.
    pub llm: Option<LlmSection>,
}

/// The honesty posture block stamped into the report.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Honesty {
    /// The guarantee lattice, strongest-first.
    pub lattice: [&'static str; 4],
    /// The rule the verdict tags obey.
    pub rule: &'static str,
}

impl Default for Honesty {
    fn default() -> Self {
        Self {
            lattice: ["Exact", "Proven", "Empirical", "Declared"],
            rule: "Every measured number is Empirical (a trial mean with its trial count + spread); a \
                   capability loss / skip / runtime error is Declared. No verdict is Proven or Exact, \
                   and no performance target is pre-written (VR-5). A differential divergence from the \
                   trusted interpreter is a recorded correctness LOSS; an unlowerable node is a \
                   recorded capability LOSS (G2 — never omitted).",
        }
    }
}

/// The LLM-harness section of the unified report.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LlmSection {
    /// The source file the report was read from.
    pub source_path: String,
    /// Whether the source was the committed SYNTHETIC sample (vs a real run found in the reports dir).
    pub is_synthetic: bool,
    /// The provenance one-liner.
    pub provenance: String,
    /// Per-validation rows (id, status, tag, latency, tokens, message).
    pub validations: Vec<LlmValidationRow>,
}

/// One per-validation row in the LLM section.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LlmValidationRow {
    /// Validation id.
    pub id: String,
    /// Status (PASS / mock-PASS / SKIP / FAIL).
    pub status: String,
    /// Honest guarantee tag.
    pub guarantee_tag: Option<String>,
    /// Wall-clock latency (seconds), if recorded.
    pub wall_seconds: Option<f64>,
    /// (prompt, generated) token counts, if recorded.
    pub prompt_tokens: Option<u64>,
    /// Generated token count, if recorded.
    pub generated_tokens: Option<u64>,
    /// The one-line message.
    pub message: String,
}

impl LlmSection {
    /// Build the section from a parsed report + its source path / synthetic flag.
    #[must_use]
    pub fn from_report(report: &LlmReport, source_path: String, is_synthetic: bool) -> Self {
        let validations = report
            .results
            .iter()
            .map(|v| {
                let (p, g) = match v.token_counts() {
                    Some((p, g)) => (p, g),
                    None => (None, None),
                };
                LlmValidationRow {
                    id: v.id.clone(),
                    status: v.status.clone(),
                    guarantee_tag: v.guarantee_tag.clone(),
                    wall_seconds: v.wall_seconds(),
                    prompt_tokens: p,
                    generated_tokens: g,
                    message: v.message.clone(),
                }
            })
            .collect();
        Self {
            source_path,
            is_synthetic,
            provenance: report.provenance(),
            validations,
        }
    }

    /// Build the section from the schema-agnostic [`crate::llm::ParsedLlmSection`] produced by
    /// [`crate::llm::parse_any_llm_json`]. This is the preferred entry-point when the caller
    /// does not know in advance whether the JSON is a bench-harness or a Grok-harness report.
    #[must_use]
    pub fn from_parsed(parsed: crate::llm::ParsedLlmSection) -> Self {
        Self {
            source_path: parsed.source_path,
            is_synthetic: parsed.is_synthetic,
            provenance: parsed.provenance,
            validations: parsed.validations,
        }
    }
}

/// A roll-up of losses for the "where we're losing" section.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct LossRollup {
    /// (case, backend, reason) capability losses.
    pub capability: Vec<(String, &'static str, String)>,
    /// (case, backend, detail) correctness losses (divergences) — the most serious.
    pub correctness: Vec<(String, &'static str, String)>,
    /// (case, backend, ratio_x1000, reason) speed losses.
    pub speed: Vec<(String, &'static str, u64, String)>,
}

impl Report {
    /// Roll up every loss across the run for the "where we're losing" section.
    #[must_use]
    pub fn loss_rollup(&self) -> LossRollup {
        let mut roll = LossRollup::default();
        for case in &self.run.cases {
            for b in &case.backends {
                match &b.verdict {
                    Verdict::CapabilityLoss { reason } => {
                        roll.capability
                            .push((case.id.clone(), b.backend.label(), reason.clone()));
                    }
                    Verdict::CorrectnessLoss { detail } => {
                        roll.correctness
                            .push((case.id.clone(), b.backend.label(), detail.clone()));
                    }
                    Verdict::SpeedLoss {
                        ratio_x1000,
                        reason,
                    } => {
                        roll.speed.push((
                            case.id.clone(),
                            b.backend.label(),
                            *ratio_x1000,
                            reason.clone(),
                        ));
                    }
                    _ => {}
                }
            }
        }
        roll
    }

    /// Count (wins, speed-losses, correctness-losses, capability-losses, skips) across the run.
    #[must_use]
    pub fn tallies(&self) -> Tallies {
        let mut t = Tallies::default();
        for case in &self.run.cases {
            for b in &case.backends {
                match &b.verdict {
                    Verdict::SpeedWin { .. } => t.wins += 1,
                    Verdict::SpeedNeutral { .. } => t.neutral += 1,
                    Verdict::SpeedLoss { .. } => t.speed_losses += 1,
                    Verdict::CorrectnessLoss { .. } => t.correctness_losses += 1,
                    Verdict::CapabilityLoss { .. } => t.capability_losses += 1,
                    Verdict::RuntimeError { .. } => t.errors += 1,
                    Verdict::Skipped { .. } => t.skips += 1,
                    Verdict::BaselineFailed { .. } => t.baseline_failures += 1,
                }
            }
        }
        t
    }

    /// The machine-readable JSON projection (pretty-printed, deterministic).
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// The human-readable markdown projection (deterministic — same run ⇒ same bytes, modulo the
    /// measured numbers themselves).
    #[must_use]
    pub fn to_markdown(&self) -> String {
        let mut s = String::new();
        self.write_header(&mut s);
        self.write_winloss_table(&mut s);
        self.write_per_backend_numbers(&mut s);
        self.write_losses(&mut s);
        self.write_llm(&mut s);
        // Normalize the trailer to exactly one newline so the emitted markdown is lint-clean
        // (markdownlint MD012 — no multiple consecutive blank lines at EOF) regardless of which
        // section wrote the last line. Deterministic: idempotent on already-clean output.
        let trimmed = s.trim_end();
        s.truncate(trimmed.len());
        s.push('\n');
        s
    }

    fn write_header(&self, s: &mut String) {
        let t = self.tallies();
        let _ = writeln!(s, "# Mycelium honest benchmark report\n");
        let _ = writeln!(
            s,
            "> Tool `{}` — profile `{}` — `mlir-dialect` feature: {} — {}\n",
            self.tool,
            self.profile,
            if self.mlir_dialect_feature {
                "ON"
            } else {
                "OFF"
            },
            self.host_note
        );
        let _ = writeln!(
            s,
            "Guarantee lattice: `{}`.\n\n**Honesty:** {}\n",
            self.honesty.lattice.join(" ⊐ "),
            self.honesty.rule
        );
        let _ = writeln!(
            s,
            "Speed band: a backend within ±{:.0}% of the interpreter is *neutral*; faster is a \
             **WIN**, slower a **LOSS (speed)**. Trusted baseline: the **interpreter** (in-process; \
             NFR-7/ADR-007).\n",
            self.neutral_band * 100.0
        );
        let _ = writeln!(
            s,
            "Tally across the run: **{} win(s)**, {} neutral, **{} speed-loss(es)**, **{} \
             correctness-loss(es)**, **{} capability-loss(es)**, {} runtime-error(s), {} skip(s){}.\n",
            t.wins,
            t.neutral,
            t.speed_losses,
            t.correctness_losses,
            t.capability_losses,
            t.errors,
            t.skips,
            if t.baseline_failures > 0 {
                format!(", **{} BASELINE FAILURE(S) — investigate**", t.baseline_failures)
            } else {
                String::new()
            }
        );
        let _ = writeln!(
            s,
            "**Microbench caveats (honest):** numbers are warmup + min-mean over batches via \
             `std::time::Instant` (no `criterion`). The compiled native paths (`direct-llvm`, \
             `mlir-dialect`) are **process-spawn-bound**: each invocation execs a fresh native \
             artifact, so for a trivial kernel the per-invocation figure is spawn-dominated, **not** \
             kernel compute (the honest M-602/E1 finding — surfaced, not buried). `jit` runs \
             in-process (`dlopen`) so it is not spawn-bound. A debug build is refused for perf \
             numbers.\n"
        );
    }

    fn write_winloss_table(&self, s: &mut String) {
        let _ = writeln!(s, "## WIN / LOSS / regression table\n");
        let _ = writeln!(
            s,
            "Each non-baseline backend vs the interpreter, per case. `ratio` is `interp / backend` \
             (>1 ⇒ backend faster). Tag is per-row.\n"
        );
        let _ = writeln!(
            s,
            "| case | fragment | backend | verdict | ratio | tag | reason / detail |"
        );
        let _ = writeln!(s, "|---|---|---|---|---|---|---|");
        for case in &self.run.cases {
            for b in &case.backends {
                let (ratio, reason) = verdict_ratio_reason(&b.verdict);
                let _ = writeln!(
                    s,
                    "| `{}` | {} | `{}` | {} | {} | {} | {} |",
                    case.id,
                    case.fragment.label(),
                    b.backend.label(),
                    b.verdict.status(),
                    ratio,
                    b.verdict.guarantee_tag(),
                    md_escape(&reason),
                );
            }
        }
        let _ = writeln!(s);
    }

    fn write_per_backend_numbers(&self, s: &mut String) {
        let _ = writeln!(s, "## Per-case timings (ns/call, Empirical)\n");
        let _ = writeln!(
            s,
            "Interpreter baseline + each backend that produced a timed value. The best ns/call is \
             shown; the worst/best spread (a noise flag) is in the JSON projection \
             (`ns_per_call_worst`), omitted from this compact table. `—` = not timed (skip / \
             capability loss / error).\n"
        );
        let _ = writeln!(
            s,
            "| case | interp ns | aot-env ns | jit ns | direct-llvm ns | mlir-dialect ns |"
        );
        let _ = writeln!(s, "|---|---|---|---|---|---|");
        for case in &self.run.cases {
            let base = case.baseline_ns.map_or_else(|| "—".to_string(), fmt_ns);
            let cell = |backend: Backend| -> String {
                case.backends
                    .iter()
                    .find(|b| b.backend == backend)
                    .and_then(|b| b.timing)
                    .map_or_else(|| "—".to_string(), |t| fmt_ns(t.ns_per_call))
            };
            let _ = writeln!(
                s,
                "| `{}` | {} | {} | {} | {} | {} |",
                case.id,
                base,
                cell(Backend::AotEnv),
                cell(Backend::Jit),
                cell(Backend::DirectLlvm),
                cell(Backend::MlirDialect),
            );
        }
        let _ = writeln!(s);
        // The compiled backends' one-time compile cost is reported separately so the per-run figures
        // above stay honest (compile cost is amortized over many runs, not charged per invocation).
        let mut any_compile = false;
        for case in &self.run.cases {
            for b in &case.backends {
                if let Some(c) = b.compile_ns {
                    if !any_compile {
                        let _ = writeln!(
                            s,
                            "One-time compile cost (emit IR → toolchain → native, NOT in the per-run \
                             figures above):\n"
                        );
                        any_compile = true;
                    }
                    let _ = writeln!(
                        s,
                        "- `{}` / `{}`: {} (one-time)",
                        case.id,
                        b.backend.label(),
                        fmt_ns(c)
                    );
                }
            }
        }
        if any_compile {
            let _ = writeln!(s);
        }
    }

    fn write_losses(&self, s: &mut String) {
        let roll = self.loss_rollup();
        let _ = writeln!(s, "## Where we're losing (explicit)\n");
        if roll.capability.is_empty() && roll.correctness.is_empty() && roll.speed.is_empty() {
            let _ = writeln!(
                s,
                "No losses recorded in this run. (That is itself a measurement, not a target — \
                 VR-5.)\n"
            );
            return;
        }

        if !roll.correctness.is_empty() {
            let _ = writeln!(
                s,
                "### Correctness losses (divergence from the trusted interpreter — most serious)\n"
            );
            let _ = writeln!(s, "| case | backend | divergence |");
            let _ = writeln!(s, "|---|---|---|");
            for (case, backend, detail) in &roll.correctness {
                let _ = writeln!(s, "| `{}` | `{}` | {} |", case, backend, md_escape(detail));
            }
            let _ = writeln!(s);
        }

        if !roll.capability.is_empty() {
            let _ = writeln!(
                s,
                "### Capability losses (a backend cannot lower the program — the reason, never \
                 omitted, G2)\n"
            );
            let _ = writeln!(s, "| case | backend | reason |");
            let _ = writeln!(s, "|---|---|---|");
            for (case, backend, reason) in &roll.capability {
                let _ = writeln!(s, "| `{}` | `{}` | {} |", case, backend, md_escape(reason));
            }
            let _ = writeln!(s);
        }

        if !roll.speed.is_empty() {
            let _ = writeln!(
                s,
                "### Speed losses (slower than the in-process interpreter — measured, with the \
                 derivable reason)\n"
            );
            let _ = writeln!(s, "| case | backend | ratio (interp/backend) | reason |");
            let _ = writeln!(s, "|---|---|---|---|");
            for (case, backend, ratio_x1000, reason) in &roll.speed {
                let _ = writeln!(
                    s,
                    "| `{}` | `{}` | {} | {} |",
                    case,
                    backend,
                    fmt_ratio(*ratio_x1000),
                    md_escape(reason),
                );
            }
            let _ = writeln!(s);
        }
    }

    fn write_llm(&self, s: &mut String) {
        let _ = writeln!(s, "## LLM-harness leverage (KC-2 / SC-5b)\n");
        match &self.llm {
            None => {
                let _ = writeln!(
                    s,
                    "No LLM-harness report found to ingest (no `*-report.json` in the reports dir, \
                     and the committed synthetic sample was not reachable). This section is empty — \
                     not synthesized.\n"
                );
            }
            Some(sec) => {
                let label = if sec.is_synthetic {
                    "**SYNTHETIC sample** (a fixture run — NOT real model quality; never treated as \
                     evidence, per the harness's own VR-5/V-03 rule)"
                } else {
                    "real model run"
                };
                let _ = writeln!(s, "Source: `{}` — {}.\n", sec.source_path, label);
                let _ = writeln!(s, "> {}\n", sec.provenance);
                let _ = writeln!(
                    s,
                    "| validation | status | tag | latency (s) | prompt tok | gen tok | message |"
                );
                let _ = writeln!(s, "|---|---|---|---|---|---|---|");
                for v in &sec.validations {
                    let _ = writeln!(
                        s,
                        "| `{}` | {} | {} | {} | {} | {} | {} |",
                        v.id,
                        v.status,
                        v.guarantee_tag.as_deref().unwrap_or("—"),
                        v.wall_seconds
                            .map_or_else(|| "—".to_string(), |w| format!("{w:.4}")),
                        v.prompt_tokens
                            .map_or_else(|| "—".to_string(), |t| t.to_string()),
                        v.generated_tokens
                            .map_or_else(|| "—".to_string(), |t| t.to_string()),
                        md_escape(&v.message),
                    );
                }
                let _ = writeln!(s);
            }
        }
    }
}

/// Loss/win tallies across a run.
#[derive(Debug, Clone, Copy, Default, serde::Serialize)]
pub struct Tallies {
    /// Measured speed wins.
    pub wins: u32,
    /// Near-parity (neutral band).
    pub neutral: u32,
    /// Measured speed losses.
    pub speed_losses: u32,
    /// Differential divergences (correctness losses).
    pub correctness_losses: u32,
    /// Unlowerable-node capability losses.
    pub capability_losses: u32,
    /// Runtime errors.
    pub errors: u32,
    /// Environmental skips (toolchain absent / feature off).
    pub skips: u32,
    /// Baseline (interpreter) failures — should be zero; loud if not.
    pub baseline_failures: u32,
}

/// Extract a `(ratio_str, reason)` for a verdict's table row.
fn verdict_ratio_reason(v: &Verdict) -> (String, String) {
    match v {
        Verdict::SpeedWin { ratio_x1000 } | Verdict::SpeedNeutral { ratio_x1000 } => {
            (fmt_ratio(*ratio_x1000), String::new())
        }
        Verdict::SpeedLoss {
            ratio_x1000,
            reason,
        } => (fmt_ratio(*ratio_x1000), reason.clone()),
        Verdict::CorrectnessLoss { detail } => ("—".to_string(), detail.clone()),
        Verdict::CapabilityLoss { reason } => ("—".to_string(), reason.clone()),
        Verdict::RuntimeError { message } => ("—".to_string(), message.clone()),
        Verdict::Skipped { reason } => ("—".to_string(), reason.clone()),
        Verdict::BaselineFailed { message } => ("—".to_string(), message.clone()),
    }
}

/// Format a ns figure compactly.
fn fmt_ns(ns: f64) -> String {
    if ns >= 1_000_000.0 {
        format!("{:.2}M", ns / 1_000_000.0)
    } else if ns >= 1_000.0 {
        format!("{:.1}k", ns / 1_000.0)
    } else {
        format!("{ns:.1}")
    }
}

/// Format a parts-per-thousand ratio as `N.NNx`.
fn fmt_ratio(x1000: u64) -> String {
    #[allow(clippy::cast_precision_loss)]
    let r = x1000 as f64 / 1000.0;
    format!("{r:.2}x")
}

/// Escape `|` and newlines so a reason cannot break a markdown table row.
fn md_escape(s: &str) -> String {
    s.replace('|', "\\|").replace(['\n', '\r'], " ")
}

/// The neutral-band constant, re-exported for the binary to stamp into the report metadata.
#[must_use]
pub fn neutral_band() -> f64 {
    NEUTRAL_BAND
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::Engines;
    use crate::corpus::corpus;
    use crate::measure::run_corpus;

    /// Build a small real report by measuring two bit cases (offline, deterministic enough for the
    /// emission tests — we assert structure, never specific timings).
    fn small_report() -> Report {
        let eng = Engines::default();
        let cases: Vec<_> = corpus()
            .into_iter()
            .filter(|c| matches!(c.id, "bit-xor-not" | "rec-self"))
            .collect();
        let run = run_corpus(&cases, &eng);
        Report {
            tool: "mycelium-bench-test",
            profile: "test",
            mlir_dialect_feature: cfg!(feature = "mlir-dialect"),
            host_note: "unit-test".into(),
            honesty: Honesty::default(),
            neutral_band: NEUTRAL_BAND,
            run,
            llm: None,
        }
    }

    #[test]
    fn json_projection_is_valid_and_roundtrips_as_value() {
        let r = small_report();
        let json = r.to_json().expect("serializes");
        // It must be valid JSON.
        let v: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(v["tool"], "mycelium-bench-test");
        assert!(v["run"]["cases"].is_array());
        // Deterministic: serializing twice gives identical bytes.
        assert_eq!(json, r.to_json().unwrap());
    }

    #[test]
    fn markdown_has_all_required_sections() {
        let r = small_report();
        let md = r.to_markdown();
        assert!(md.contains("# Mycelium honest benchmark report"));
        assert!(md.contains("## WIN / LOSS / regression table"));
        assert!(md.contains("## Per-case timings"));
        assert!(md.contains("## Where we're losing"));
        assert!(md.contains("## LLM-harness leverage"));
        // The honesty posture + VR-5 must be stated.
        assert!(md.contains("VR-5"));
        assert!(md.contains("Empirical"));
        // The process-spawn caveat must be present (it is the headline honest finding).
        assert!(md.contains("process-spawn-bound"));
    }

    #[test]
    fn the_recursion_case_surfaces_a_capability_loss_in_the_losses_section() {
        // rec-self cannot be lowered by jit/direct-llvm — the "where we're losing" section MUST name
        // it as a capability loss (unless the run skipped for toolchain absence, also acceptable).
        let r = small_report();
        let roll = r.loss_rollup();
        let md = r.to_markdown();
        // Either a capability loss is recorded, or the compiled paths were skipped (no toolchain).
        let tallies = r.tallies();
        let compiled_accounted = !roll.capability.is_empty() || tallies.skips > 0;
        assert!(
            compiled_accounted,
            "the recursion case must produce a capability loss OR a skip for the compiled paths"
        );
        if !roll.capability.is_empty() {
            assert!(
                md.contains("Capability losses"),
                "a recorded capability loss must appear in the losses section"
            );
        }
    }

    #[test]
    fn md_escape_protects_table_rows() {
        assert_eq!(md_escape("a | b\nc"), "a \\| b c");
    }

    #[test]
    fn llm_section_labels_synthetic_when_present() {
        use crate::llm::LlmReport;
        let sample = r#"{
          "harness":"mycelium-llm-validation","version":"0.1.0","run_id":"X","mode":"mock",
          "honesty_posture":{"never_silent":true,"guarantee_lattice":["Exact"],
            "model_allowed_tags":["Declared"],"vr5_rule":"r"},
          "summary":{"overall":"MOCK","total":1,"pass":0,"mock_pass":1,"skip":0,"fail":0,
            "exit_code":0,"mode":"mock","model":null},
          "results":[{"id":"V-01","status":"mock-PASS","guarantee_tag":"Declared",
            "message":"m","detail":{"mode":"mock"}}]
        }"#;
        let rep = LlmReport::from_json(sample).unwrap();
        let mut r = small_report();
        r.llm = Some(LlmSection::from_report(
            &rep,
            "sample.json".into(),
            rep.is_synthetic(),
        ));
        let md = r.to_markdown();
        assert!(
            md.contains("SYNTHETIC sample"),
            "synthetic must be labeled in the LLM section"
        );
        assert!(md.contains("V-01"));
    }
}
