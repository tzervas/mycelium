"""Descriptive assessment of a KC-2 run (M-002) — analysis, never a verdict.

This turns the measured rates into a readable **executive summary** + a structured
assessment object (one assessment, two projections — G11). It *characterises* the
findings (ratings, gaps, which tasks struggled, whether edit-to-fix feedback helped)
so a maintainer can read and decide quickly.

It does **not** decide (VR-5): the KC-2 verdict — proceed / reweight-to-human /
fall-back-to-embedded-DSL — is the maintainer's, made from these numbers. The
``decision`` field says exactly that, and every characterisation is hedged to the
evidence (notably the coarse, small-n task set).
"""

from __future__ import annotations

from collections.abc import Mapping
from typing import Any

# First-attempt pass-rate buckets — a coarse, descriptive label, NOT a pass/fail gate.
_STRONG, _MODERATE = 0.8, 0.5


def _rate_label(rate: float | None) -> str:
    if rate is None:
        return "n/a"
    if rate >= _STRONG:
        return "strong"
    if rate >= _MODERATE:
        return "moderate"
    return "weak"


def _arm_assessment(metrics: Mapping[str, Any]) -> dict[str, Any]:
    if not metrics.get("ran"):
        return {"ran": False, "skipped": metrics.get("skipped", "skipped")}
    outcomes = metrics.get("outcomes", [])
    first = metrics.get("first_attempt_pass_rate")
    eventual = metrics.get("eventual_pass_rate")
    leverage = (
        round(eventual - first, 4)
        if isinstance(first, (int, float)) and isinstance(eventual, (int, float))
        else None
    )
    return {
        "ran": True,
        "syntactic_validity_rate": metrics.get("syntactic_validity_rate"),
        "first_attempt_pass_rate": first,
        "eventual_pass_rate": eventual,
        "mean_iterations_to_pass": metrics.get("mean_iterations_to_pass"),
        "first_attempt_rating": _rate_label(first),
        "edit_to_fix_leverage": leverage,
        "never_passed_tasks": [o["task"] for o in outcomes if not o.get("passed")],
        "parsed_but_failed_first": [
            o["task"]
            for o in outcomes
            if o.get("first_attempt_valid") and not o.get("first_attempt_passed")
        ],
    }


def assess(document: Mapping[str, Any]) -> dict[str, Any]:
    """Build the structured assessment object from a run document."""
    arms_in = document.get("arms", {})
    arms = {name: _arm_assessment(m) for name, m in arms_in.items()}

    comparison: dict[str, Any] | None = None
    myc, base = arms.get("mycelium"), arms.get("baseline")
    if myc and base and myc.get("ran") and base.get("ran"):
        fm, fb = myc["first_attempt_pass_rate"], base["first_attempt_pass_rate"]
        em, eb = myc["eventual_pass_rate"], base["eventual_pass_rate"]
        gap = round((fm - fb) * 100, 1)
        comparison = {
            "first_attempt_gap_pp": gap,  # +ve favours Mycelium
            "eventual_gap_pp": round((em - eb) * 100, 1),
            "leans": "mycelium" if gap > 0 else ("baseline" if gap < 0 else "even"),
        }

    n = document.get("task_count", 0)
    caveats = [
        f"coarse signal: {n} tasks — each task is ~{round(100 / n, 1) if n else '?'}pp; "
        "do not over-read small gaps",
        "first-attempt pass rate is the SC-5b number; eventual-minus-first is the "
        "edit-to-fix (G10) leverage signal",
        "rates depend on the primer (generator configuration) and the model/seed — "
        "record them with the verdict",
    ]
    skipped = [name for name, a in arms.items() if not a.get("ran")]
    if skipped:
        caveats.append(f"arm(s) skipped, so the comparison is incomplete: {', '.join(skipped)}")

    return {
        "arms": arms,
        "comparison": comparison,
        "caveats": caveats,
        "decision": (
            "deferred to the maintainer (VR-5) — this is descriptive analysis of the "
            "measured rates, NOT the KC-2 verdict"
        ),
    }


def _fmt_rate(x: float | None) -> str:
    return f"{x * 100:.0f}%" if isinstance(x, (int, float)) else "n/a"


def _fmt_pp(x: float | None) -> str:
    return f"{x:+.1f}pp" if isinstance(x, (int, float)) else "n/a"


def render_summary(document: Mapping[str, Any], assessment: Mapping[str, Any]) -> str:
    """Render the human-readable executive summary (the thing a maintainer reads)."""
    lines: list[str] = []
    lines.append("=" * 72)
    lines.append("  KC-2 LLM-leverage — executive summary (analysis, NOT a verdict)")
    lines.append("=" * 72)
    lines.append(f"  Model   : {document.get('model')}")
    lines.append(f"  Backend : {document.get('backend')}   Seed: {document.get('seed')}")
    lines.append(
        f"  Tasks   : {document.get('task_count')}   "
        f"Edit-to-fix budget: {document.get('edit_to_fix_budget')}"
    )
    lines.append("")

    for name, a in assessment.get("arms", {}).items():
        lines.append(f"  [{name}]")
        if not a.get("ran"):
            # Show only the first line of the (possibly multi-line) skip reason here;
            # the full detail lives in the JSON report's `skipped` field.
            reason = str(a.get("skipped") or "skipped")
            rlines = reason.splitlines() or ["skipped"]
            lines.append(f"      SKIPPED — {rlines[0]}")
            if len(rlines) > 1:
                lines.append("               (full detail in the JSON report's `skipped` field)")
            lines.append("")
            continue
        lines.append(
            f"      first-attempt pass : {_fmt_rate(a['first_attempt_pass_rate'])} "
            f"({a['first_attempt_rating']})   "
            f"syntactic valid: {_fmt_rate(a['syntactic_validity_rate'])}"
        )
        lev = a["edit_to_fix_leverage"]
        lines.append(
            f"      eventual pass      : {_fmt_rate(a['eventual_pass_rate'])}   "
            f"edit-to-fix gain: {_fmt_pp(lev * 100 if lev is not None else None)}   "
            f"mean iters: {a['mean_iterations_to_pass']}"
        )
        if a["never_passed_tasks"]:
            lines.append(f"      never passed       : {', '.join(a['never_passed_tasks'])}")
        if a["parsed_but_failed_first"]:
            lines.append(f"      parsed but wrong#1 : {', '.join(a['parsed_but_failed_first'])}")
        lines.append("")

    comp = assessment.get("comparison")
    if comp:
        lines.append("  [comparison]")
        lines.append(
            f"      first-attempt gap  : {_fmt_pp(comp['first_attempt_gap_pp'])} "
            f"(positive favours Mycelium) — leans {comp['leans']}"
        )
        lines.append(f"      eventual-pass gap  : {_fmt_pp(comp['eventual_gap_pp'])}")
        lines.append("")

    lines.append("  Read-this-way (cues, not conclusions):")
    for c in assessment.get("caveats", []):
        lines.append(f"      · {c}")
    lines.append("")
    lines.append(f"  Decision: {assessment.get('decision')}")
    lines.append("=" * 72)
    return "\n".join(lines) + "\n"
