"""Report emission: per-model JSON + a cross-model markdown comparison (G11).

DUAL PROJECTION (G11): every run renders the same result object two ways — a
machine-readable per-model JSON and a human-readable markdown comparison table —
into ``tools/llm-harness/reports/``. Every report carries provenance metadata
(ISO timestamp, RNG seed, task-set id, model, endpoint, mode) and the honesty
posture (the guarantee lattice + the VR-5 rule), so a report is self-describing.

A report produced by the offline self-test is stamped ``SYNTHETIC (self-test)`` in
both renderings so it can never be mistaken for a real model measurement (VR-5).
"""

from __future__ import annotations

import datetime
import json
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

HARNESS_NAME = "mycelium-grok-coauthor"
HARNESS_VERSION = "0.1.0"

LATTICE = ["Exact", "Proven", "Empirical", "Declared"]
MODEL_ALLOWED_TAGS = ["Empirical", "Declared"]
VR5_RULE = (
    "Model-derived claims are Empirical or Declared — NEVER Proven or Exact. "
    "A SKIP is never a PASS. The retention/leverage verdict is reported as a "
    "computed comparison and stays open/Declared until the full live campaign runs."
)


def now_iso() -> str:
    """UTC timestamp, compact ISO basic form (sortable, filesystem-safe)."""
    return datetime.datetime.now(datetime.UTC).strftime("%Y%m%dT%H%M%SZ")


def honesty_posture(*, synthetic: bool) -> dict[str, Any]:
    """The honesty block embedded in every report."""
    return {
        "never_silent": True,
        "guarantee_lattice": list(LATTICE),
        "model_allowed_tags": list(MODEL_ALLOWED_TAGS),
        "vr5_rule": VR5_RULE,
        "synthetic": synthetic,
        "synthetic_note": (
            "SYNTHETIC (self-test): produced by the deterministic offline mock "
            "client. NOT a real model measurement; carries no quality evidence."
            if synthetic
            else "live/operator run"
        ),
    }


@dataclass
class RunMetadata:
    """Provenance for one model's run (recorded verbatim in the report)."""

    model: str
    mode: str  # "live" | "batch" | "self-test"
    endpoint: str
    task_set_id: str
    seed: int
    max_rounds: int
    synthetic: bool = False
    timestamp_utc: str = field(default_factory=now_iso)

    def to_dict(self) -> dict[str, Any]:
        return {
            "harness": HARNESS_NAME,
            "version": HARNESS_VERSION,
            "model": self.model,
            "mode": self.mode,
            "endpoint": self.endpoint,
            "task_set_id": self.task_set_id,
            "seed": self.seed,
            "max_rounds": self.max_rounds,
            "timestamp_utc": self.timestamp_utc,
        }


@dataclass
class ModelRunReport:
    """Everything one model produced in a run — the JSON-serialisable payload."""

    metadata: RunMetadata
    quality: dict[str, Any]  # QualityMetrics.to_dict()
    performance: dict[str, Any]  # tokens/latency/cost/request counts
    outcomes: list[dict[str, Any]]  # per-task TaskOutcome.to_dict()
    ablation: dict[str, Any] | None = None  # AblationReport.to_dict() if --ablation

    def to_dict(self) -> dict[str, Any]:
        return {
            "metadata": self.metadata.to_dict(),
            "honesty_posture": honesty_posture(synthetic=self.metadata.synthetic),
            "quality": self.quality,
            "performance": self.performance,
            "outcomes": self.outcomes,
            "ablation": self.ablation,
        }


def write_model_json(report: ModelRunReport, *, reports_dir: Path, run_id: str) -> Path:
    """Write one model's JSON report; returns the path."""
    reports_dir.mkdir(parents=True, exist_ok=True)
    safe_model = report.metadata.model.replace("/", "_").replace(" ", "_")
    syn = "SYNTHETIC-" if report.metadata.synthetic else ""
    path = reports_dir / f"{syn}{run_id}-{safe_model}-{report.metadata.mode}.json"
    path.write_text(json.dumps(report.to_dict(), indent=2), encoding="utf-8")
    return path


def _fmt_pct(x: float | None) -> str:
    return "n/a" if x is None else f"{x:.1%}"


def _fmt_usd(x: float | None) -> str:
    return "n/a" if x is None else f"${x:.6f}"


def _fmt_float(x: float | None, prec: int = 2) -> str:
    return "n/a" if x is None else f"{x:.{prec}f}"


def render_comparison_markdown(
    reports: list[ModelRunReport],
    *,
    run_id: str,
    mode: str,
    synthetic: bool,
    generated: str | None = None,
) -> str:
    """Render the cross-model comparison table + per-model detail (markdown).

    ``generated`` overrides the timestamp line (used to keep a synthetic sample
    deterministic); defaults to wall-clock for live runs.
    """
    lines: list[str] = []
    title = "Mycelium Grok co-authoring — cross-model comparison"
    lines.append(f"# {title}")
    lines.append("")
    if synthetic:
        lines.append(
            "> **SYNTHETIC (self-test).** Produced by the deterministic offline "
            "mock client — NOT a real model measurement. Carries no quality "
            "evidence (VR-5). Plumbing only."
        )
        lines.append("")
    lines.append(f"- run id: `{run_id}`")
    lines.append(f"- mode: `{mode}`")
    lines.append(f"- generated: `{generated or now_iso()}`")
    lines.append(f"- guarantee lattice: {' ⊐ '.join(LATTICE)}")
    lines.append(
        f"- model-allowed tags: {', '.join(MODEL_ALLOWED_TAGS)} (never Proven/Exact — VR-5)"
    )
    lines.append("")
    # Comparison table (models in the order they were run = cheapest-first).
    header = (
        "| model | mode | syntactic-valid | type-check pass | mean edit-to-fix "
        "| tokens (in/out) | mean latency s | total cost |"
    )
    sep = "|---|---|---|---|---|---|---|---|"
    lines.append(header)
    lines.append(sep)
    for r in reports:
        q = r.quality
        p = r.performance
        lines.append(
            "| {model} | {mode} | {syn} | {typ} | {fix} | {tin}/{tout} | {lat} | {cost} |".format(
                model=f"`{r.metadata.model}`",
                mode=r.metadata.mode,
                syn=_fmt_pct(q.get("syntactic_validity_rate")),
                typ=_fmt_pct(q.get("typecheck_pass_rate")),
                fix=_fmt_float(q.get("mean_edit_to_fix")),
                tin=p.get("prompt_tokens", 0),
                tout=p.get("completion_tokens", 0),
                lat=_fmt_float(p.get("mean_latency_s")),
                cost=_fmt_usd(p.get("total_cost_usd")),
            )
        )
    lines.append("")
    # Per-model honest tag + any ablation verdict.
    for r in reports:
        lines.append(f"## `{r.metadata.model}`")
        lines.append("")
        tag = "Declared (synthetic)" if synthetic else "Empirical (measured)"
        lines.append(f"- quality tag: **{tag}**")
        lines.append(f"- endpoint: `{r.metadata.endpoint}`")
        lines.append(
            f"- task set: `{r.metadata.task_set_id}`  seed: `{r.metadata.seed}`"
        )
        if r.ablation is not None:
            ret = r.ablation.get("retention", {})
            lines.append("")
            lines.append("### Retention-ratio ablation (M-381)")
            lines.append(
                f"- retention ratio: **{_fmt_float(ret.get('retention_ratio'), 3)}** "
                f"(threshold ~{ret.get('threshold')})"
            )
            lines.append(f"- {ret.get('conclusion', '')}")
            lines.append(
                f"- leverage claim tag: **{ret.get('leverage_claim_tag', 'Declared')} "
                "(open — pending full campaign)**"
            )
            lines.append("")
            lines.append("| arm | ran | pass@1 | note |")
            lines.append("|---|---|---|---|")
            for a in r.ablation.get("arms", []):
                note = (
                    a.get("description")
                    if a.get("ran")
                    else a.get("blocked_reason", "")
                )
                lines.append(
                    "| {aid} | {ran} | {p} | {note} |".format(
                        aid=f"`{a.get('arm_id')}`",
                        ran="yes" if a.get("ran") else "**blocked**",
                        p=_fmt_pct(a.get("pass_at_1")),
                        note=note,
                    )
                )
        lines.append("")
    return "\n".join(lines)


def write_comparison_markdown(
    reports: list[ModelRunReport],
    *,
    reports_dir: Path,
    run_id: str,
    mode: str,
    synthetic: bool,
    generated: str | None = None,
) -> Path:
    """Write the cross-model markdown comparison; returns the path."""
    reports_dir.mkdir(parents=True, exist_ok=True)
    syn = "SYNTHETIC-" if synthetic else ""
    path = reports_dir / f"{syn}{run_id}-comparison.md"
    text = render_comparison_markdown(
        reports, run_id=run_id, mode=mode, synthetic=synthetic, generated=generated
    )
    path.write_text(text, encoding="utf-8")
    return path


def build_performance(
    *,
    prompt_tokens: int,
    completion_tokens: int,
    total_cost_usd: float,
    latencies: list[float],
    request_count: int,
    batch_count: int = 0,
) -> dict[str, Any]:
    """Assemble the performance/cost block for a model report."""
    mean_lat = (sum(latencies) / len(latencies)) if latencies else None
    return {
        "prompt_tokens": prompt_tokens,
        "completion_tokens": completion_tokens,
        "total_tokens": prompt_tokens + completion_tokens,
        "total_cost_usd": round(total_cost_usd, 8),
        "request_count": request_count,
        "batch_count": batch_count,
        "mean_latency_s": (round(mean_lat, 4) if mean_lat is not None else None),
        "total_latency_s": round(sum(latencies), 4),
    }
