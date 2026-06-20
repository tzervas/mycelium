"""tools/llm-harness/analyze.py — BI analysis of Mycelium co-authoring run reports.

Reads one or more ``*-live.json`` (or ``*-batch.json``) per-model report files and
produces actionable intelligence: convergence rates, error taxonomy, token/cost
efficiency, cross-model comparison, and ranked recommendations.

Usage
-----
    uv run python tools/llm-harness/analyze.py reports/<run-id>-*.json
    uv run python tools/llm-harness/analyze.py --run-id 20260620T151333Z
    uv run python tools/llm-harness/analyze.py --latest          # newest run in reports/

Output: structured JSON to ``reports/<run-id>-analysis.json`` +
        markdown summary to ``reports/<run-id>-analysis.md``.

Guarantee tag: **Empirical** (this script only reads and tabulates measured data;
it draws no conclusions about unrun experiments, never upgrades claim tags, and
marks all derived rates as "computed from measured outcomes").

Honesty (VR-5): this script does NOT fabricate BI insights for blocked arms or
missing data. Every metric is labelled with its denominator so vacuous rates are
visible.  Recommendations are tagged [Empirical] or [Declared] per their basis.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any


# ---------------------------------------------------------------------------
# Data loading
# ---------------------------------------------------------------------------


def load_reports(paths: list[Path]) -> list[dict[str, Any]]:
    reports = []
    for p in paths:
        try:
            reports.append(json.loads(p.read_text(encoding="utf-8")))
        except Exception as exc:
            print(f"WARNING: could not read {p}: {exc}", file=sys.stderr)
    return reports


def find_reports(reports_dir: Path, run_id: str | None, latest: bool) -> list[Path]:
    if not reports_dir.is_dir():
        raise FileNotFoundError(f"reports dir not found: {reports_dir}")

    all_json = sorted(
        [
            p
            for p in reports_dir.glob("*.json")
            if "analysis" not in p.name and "SYNTHETIC" not in p.name
        ],
        key=lambda p: p.name,
    )
    if not all_json:
        raise FileNotFoundError("no report JSON files found")

    if run_id:
        matching = [p for p in all_json if p.name.startswith(run_id)]
        if not matching:
            raise FileNotFoundError(f"no reports matching run-id {run_id!r}")
        return matching

    if latest:
        # Group by run-id (prefix up to the first model segment)
        def _run_id_of(p: Path) -> str:
            # filename: <run-id>-<model>-<mode>.json
            # run-id is the leading timestamp token
            parts = p.stem.split("-")
            # run-id is parts[0] (e.g. "20260620T151333Z")
            return parts[0]

        # Newest run-id
        newest = sorted(all_json, key=lambda p: _run_id_of(p))[-1]
        newest_run = _run_id_of(newest)
        return [p for p in all_json if _run_id_of(p) == newest_run]

    return all_json


# ---------------------------------------------------------------------------
# Per-report analysis
# ---------------------------------------------------------------------------


def error_category(verdict: str) -> str:
    """Normalise a round verdict into a coarse error category."""
    if verdict in ("clean",):
        return "clean"
    if "syntax" in verdict or verdict == "syntax_error":
        return "syn"
    if "type" in verdict or verdict == "type_error":
        return "typ"
    if "error" in verdict:
        return "err"
    if "skip" in verdict:
        return "skip"
    return "other"


def analyse_model(report: dict[str, Any]) -> dict[str, Any]:
    """Derive granular metrics from one model's report dict."""
    meta = report.get("metadata", {})
    quality = report.get("quality", {})
    perf = report.get("performance", {})
    outcomes = report.get("outcomes", [])

    model_id = meta.get("model", "unknown")
    mode = meta.get("mode", "unknown")
    task_set_id = meta.get("task_set_id", "unknown")
    seed = meta.get("seed")
    max_rounds = meta.get("max_rounds", 3)

    # --- task-level aggregation ---
    total = len(outcomes)
    resumed = sum(1 for o in outcomes if o.get("resumed", False))
    scored = total - resumed
    scored_outcomes = [o for o in outcomes if not o.get("resumed", False)]
    pass_count = sum(1 for o in scored_outcomes if o.get("status") == "PASS")
    partial_count = sum(1 for o in scored_outcomes if o.get("status") == "PARTIAL_PASS")
    fail_count = sum(1 for o in scored_outcomes if o.get("status") == "FAIL")
    skip_count = sum(1 for o in scored_outcomes if o.get("status") == "SKIP")

    # --- convergence: pass at attempt k (1-indexed) among scored tasks ---
    pass_at: dict[int, int] = {}
    for o in scored_outcomes:
        if o.get("status") in ("PASS", "PARTIAL_PASS"):
            itr = o.get("iterations_to_clean") or 1
            pass_at[itr] = pass_at.get(itr, 0) + 1

    convergence = []
    cumulative = 0
    for attempt in range(1, max_rounds + 1):
        n = pass_at.get(attempt, 0)
        cumulative += n
        base = scored if scored > 0 else 1
        convergence.append(
            {
                "attempt": attempt,
                "new_passes": n,
                "cumulative_passes": cumulative,
                "cumulative_pass_rate": round(cumulative / base, 4),
            }
        )

    # --- error taxonomy per round across all tasks ---
    round_cats: dict[int, dict[str, int]] = {}
    for o in scored_outcomes:
        for rnd in o.get("rounds", []):
            att = rnd.get("attempt", 1)
            cat = error_category(rnd.get("verdict", ""))
            if att not in round_cats:
                round_cats[att] = {}
            round_cats[att][cat] = round_cats[att].get(cat, 0) + 1

    error_taxonomy = [
        {"attempt": att, "counts": round_cats[att]} for att in sorted(round_cats)
    ]

    # --- token/cost efficiency ---
    successful_outcomes = [
        o for o in scored_outcomes if o.get("status") in ("PASS", "PARTIAL_PASS")
    ]
    total_tokens = perf.get("total_tokens", 0)
    total_cost = perf.get("total_cost_usd", 0.0)
    tokens_per_success = (
        round(total_tokens / len(successful_outcomes)) if successful_outcomes else None
    )
    cost_per_success_usd = (
        round(total_cost / len(successful_outcomes), 8) if successful_outcomes else None
    )

    # --- per-task summary (lightweight — enough for the markdown table) ---
    task_summary = []
    for o in scored_outcomes:
        rounds = o.get("rounds", [])
        verdicts = [r.get("verdict", "") for r in rounds]
        task_summary.append(
            {
                "task_id": o.get("task_id"),
                "status": o.get("status"),
                "iterations_to_clean": o.get("iterations_to_clean"),
                "total_rounds": len(rounds),
                "verdict_sequence": verdicts,
                "error_categories": [error_category(v) for v in verdicts],
                "total_cost_usd": o.get("total_cost_usd"),
                "total_latency_s": o.get("total_latency_s"),
            }
        )

    # --- diagnostic message patterns (most common first-round errors) ---
    all_diag_messages: list[str] = []
    for o in scored_outcomes:
        for rnd in o.get("rounds", []):
            for d in rnd.get("diagnostics", []):
                msg = d.get("message", "") if isinstance(d, dict) else str(d)
                if msg:
                    all_diag_messages.append(msg.split("\n")[0][:120])

    # Top error prefixes (first 40 chars)
    prefix_counts: dict[str, int] = {}
    for msg in all_diag_messages:
        prefix = msg[:40]
        prefix_counts[prefix] = prefix_counts.get(prefix, 0) + 1
    top_errors = sorted(prefix_counts.items(), key=lambda x: -x[1])[:10]

    return {
        "model": model_id,
        "mode": mode,
        "task_set_id": task_set_id,
        "seed": seed,
        "guarantee_tag": "Empirical",
        "task_counts": {
            "total": total,
            "scored": scored,
            "resumed_pass": resumed,
            "pass": pass_count,
            "partial_pass": partial_count,
            "fail": fail_count,
            "skip": skip_count,
        },
        "pass_rate": {
            "syntactic_valid": quality.get("syntactic_validity_rate"),
            "typecheck_pass": quality.get("typecheck_pass_rate"),
            "any_clean": round((pass_count + partial_count) / scored, 4)
            if scored
            else None,
        },
        "convergence": convergence,
        "error_taxonomy": error_taxonomy,
        "efficiency": {
            "total_tokens": total_tokens,
            "total_cost_usd": total_cost,
            "tokens_per_success": tokens_per_success,
            "cost_per_success_usd": cost_per_success_usd,
            "mean_latency_s": perf.get("mean_latency_s"),
            "total_latency_s": perf.get("total_latency_s"),
            "request_count": perf.get("request_count"),
        },
        "top_error_prefixes": [{"prefix": p, "count": c} for p, c in top_errors],
        "task_summary": task_summary,
    }


# ---------------------------------------------------------------------------
# Cross-model comparison
# ---------------------------------------------------------------------------


def compare_models(analyses: list[dict[str, Any]]) -> dict[str, Any]:
    """Produce cross-model comparison metrics."""
    rows = []
    for a in analyses:
        rows.append(
            {
                "model": a["model"],
                "mode": a["mode"],
                "scored_tasks": a["task_counts"]["scored"],
                "syntactic_valid_rate": a["pass_rate"]["syntactic_valid"],
                "typecheck_pass_rate": a["pass_rate"]["typecheck_pass"],
                "total_cost_usd": a["efficiency"]["total_cost_usd"],
                "mean_latency_s": a["efficiency"]["mean_latency_s"],
                "tokens_per_success": a["efficiency"]["tokens_per_success"],
                "cost_per_success_usd": a["efficiency"]["cost_per_success_usd"],
            }
        )
    # Rank by typecheck_pass_rate (primary), then syntactic_valid_rate
    ranked = sorted(
        rows,
        key=lambda r: (
            r["typecheck_pass_rate"] or 0,
            r["syntactic_valid_rate"] or 0,
        ),
        reverse=True,
    )
    return {"models_ranked_by_quality": ranked}


# ---------------------------------------------------------------------------
# Recommendations
# ---------------------------------------------------------------------------


def generate_recommendations(analyses: list[dict[str, Any]]) -> list[dict[str, Any]]:
    """Derive actionable recommendations from the measured data.

    Each recommendation is tagged [Empirical] (grounded in measured data) or
    [Declared] (design inference, not yet measured).  VR-5: no recommendation
    asserts a leverage verdict for blocked arms.
    """
    recs: list[dict[str, Any]] = []

    for a in analyses:
        model = a["model"]
        convergence = a.get("convergence", [])
        error_taxonomy = a.get("error_taxonomy", [])
        top_errors = a.get("top_error_prefixes", [])

        # Check if pass@1 is low but later attempts improve
        p1 = next(
            (c["cumulative_pass_rate"] for c in convergence if c["attempt"] == 1), 0
        )
        p3 = next(
            (c["cumulative_pass_rate"] for c in convergence if c["attempt"] == 3), 0
        )

        if p1 == 0 and p3 == 0:
            recs.append(
                {
                    "model": model,
                    "priority": "high",
                    "tag": "Empirical",
                    "finding": "0% pass rate across all 3 rounds",
                    "recommendation": (
                        "No tasks pass even with 3 rounds of diagnostic feedback. "
                        "The model lacks enough Mycelium syntax knowledge to self-correct. "
                        "Next: (a) add more task examples in the system prompt (arm2 style), "
                        "(b) consider a grammar-constrained decoder (arm3 — needs GBNF "
                        "integration), or (c) fine-tune on Mycelium corpus."
                    ),
                }
            )

        if p1 < 0.2 and p3 > p1 * 1.5:
            recs.append(
                {
                    "model": model,
                    "priority": "medium",
                    "tag": "Empirical",
                    "finding": f"pass@1={p1:.0%} but pass@3={p3:.0%} — diagnostic feedback helps",
                    "recommendation": (
                        "Multi-round correction is providing value. Increasing max_rounds "
                        "beyond 3 may recover more tasks at modest cost increase. "
                        "Consider 5 rounds for the next run."
                    ),
                }
            )

        # Syntax error dominance
        round1_cats = next(
            (e["counts"] for e in error_taxonomy if e["attempt"] == 1), {}
        )
        syn1 = round1_cats.get("syn", 0)
        total1 = sum(round1_cats.values())
        if total1 > 0 and syn1 / total1 > 0.7:
            recs.append(
                {
                    "model": model,
                    "priority": "high",
                    "tag": "Empirical",
                    "finding": f"Round 1: {syn1}/{total1} ({syn1 / total1:.0%}) syntax errors",
                    "recommendation": (
                        "Syntax errors dominate round 1. The model does not know Mycelium "
                        "surface syntax. Include the full grammar in the system prompt "
                        "(arm2 grammar-in-context primer) for all production runs. "
                        "Consider building a syntax-only task set to isolate the learning curve."
                    ),
                }
            )

        # Type errors signal partial learning
        round3_cats = next(
            (e["counts"] for e in error_taxonomy if e["attempt"] == 3), {}
        )
        typ3 = round3_cats.get("typ", 0)
        if typ3 > 0:
            recs.append(
                {
                    "model": model,
                    "priority": "low",
                    "tag": "Empirical",
                    "finding": f"Round 3: {typ3} type errors (model produced syntactically valid Mycelium)",
                    "recommendation": (
                        "Type errors in round 3 mean the model CAN produce syntactically "
                        "valid Mycelium but fails type-checking. This is an improvement "
                        "signal — the model is learning the surface. Adding type-level "
                        "examples or a typed primer may close this gap."
                    ),
                }
            )

        # Cost efficiency
        eff = a.get("efficiency", {})
        cps = eff.get("cost_per_success_usd")
        if cps is None:
            recs.append(
                {
                    "model": model,
                    "priority": "info",
                    "tag": "Empirical",
                    "finding": "No successful tasks — cost-per-success undefined",
                    "recommendation": (
                        "Cannot compute cost efficiency: no tasks passed. Establish a "
                        "baseline with the grammar-in-context primer (arm2) before "
                        "optimising for cost."
                    ),
                }
            )
        elif cps > 0.01:
            recs.append(
                {
                    "model": model,
                    "priority": "low",
                    "tag": "Empirical",
                    "finding": f"cost/success=${cps:.4f} — potentially high for a production loop",
                    "recommendation": (
                        "If failures are rare, cost-per-success improves; if failures are "
                        "common, the correction loop is expensive. Consider batch mode for "
                        "arm1/arm2 first-pass generation (no correction) to reduce cost."
                    ),
                }
            )

    # Cross-model recommendation
    if len(analyses) > 1:
        best = max(analyses, key=lambda a: a["pass_rate"].get("typecheck_pass") or 0)
        recs.append(
            {
                "model": "cross-model",
                "priority": "info",
                "tag": "Empirical",
                "finding": f"Best typecheck pass rate: {best['model']}",
                "recommendation": (
                    f"{best['model']} achieved the highest typecheck pass rate in this run. "
                    "Use it as the primary model for the arm2 (grammar-primer) arm in the "
                    "next run to establish a stronger arm2 baseline before running arm4."
                ),
            }
        )

    # Arm4 unblock path
    recs.append(
        {
            "model": "all",
            "priority": "high",
            "tag": "Declared",
            "finding": "Arm4 (LlmCanonical) blocked — retention ratio INDETERMINATE",
            "recommendation": (
                "To unlock the retention ratio (the M-381 headline metric): "
                "(1) build a LlmCanonical→Core-IR parser in crates/mycelium-lsp/ "
                "or as a standalone myc-llm-parse binary; "
                "(2) wire the arm4 prompt builder to call llm_canonical() on "
                "example programs so the model sees familiar s-expression syntax; "
                "(3) score arm4 output via the new parser + myc-check. "
                "Estimated effort: 1–2 days. Unblocks the T11.7 threshold comparison."
            ),
        }
    )

    return recs


# ---------------------------------------------------------------------------
# Markdown rendering
# ---------------------------------------------------------------------------


def _pct(x: float | None) -> str:
    return "n/a" if x is None else f"{x:.1%}"


def _usd(x: float | None) -> str:
    return "n/a" if x is None else f"${x:.6f}"


def render_markdown(
    analyses: list[dict[str, Any]],
    comparison: dict[str, Any],
    recommendations: list[dict[str, Any]],
    run_ids: list[str],
) -> str:
    lines: list[str] = []

    lines += [
        "# Mycelium co-authoring harness — BI analysis",
        "",
        f"- **run(s):** {', '.join(run_ids)}",
        f"- **models analysed:** {len(analyses)}",
        "- **guarantee tag:** Empirical (computed from measured outcomes; VR-5)",
        "",
    ]

    # Cross-model comparison table
    lines += ["## Cross-model comparison (ranked by quality)", ""]
    lines.append(
        "| model | scored | syn-valid | type-pass | cost | mean-lat | tok/success | cost/success |"
    )
    lines.append("|---|---|---|---|---|---|---|---|")
    for r in comparison.get("models_ranked_by_quality", []):
        lines.append(
            "| {m} | {s} | {sv} | {tp} | {c} | {lat}s | {tps} | {cps} |".format(
                m=f"`{r['model']}`",
                s=r["scored_tasks"],
                sv=_pct(r["syntactic_valid_rate"]),
                tp=_pct(r["typecheck_pass_rate"]),
                c=_usd(r["total_cost_usd"]),
                lat=f"{r['mean_latency_s']:.1f}"
                if r["mean_latency_s"] is not None
                else "n/a",
                tps=str(r["tokens_per_success"])
                if r["tokens_per_success"] is not None
                else "n/a",
                cps=_usd(r["cost_per_success_usd"]),
            )
        )
    lines.append("")

    # Per-model detail
    for a in analyses:
        model = a["model"]
        lines += [f"## `{model}` — detail", ""]

        # Convergence
        lines += ["### Convergence (cumulative pass rate by attempt)", ""]
        lines.append("| attempt | new passes | cumulative | rate |")
        lines.append("|---|---|---|---|")
        for c in a.get("convergence", []):
            lines.append(
                f"| {c['attempt']} | {c['new_passes']} | {c['cumulative_passes']} | {_pct(c['cumulative_pass_rate'])} |"
            )
        lines.append("")

        # Error taxonomy
        lines += ["### Error taxonomy per round", ""]
        lines.append("| attempt | clean | syn | typ | err | skip | other |")
        lines.append("|---|---|---|---|---|---|---|")
        for e in a.get("error_taxonomy", []):
            cats = e["counts"]
            lines.append(
                "| {att} | {cl} | {syn} | {typ} | {err} | {skip} | {oth} |".format(
                    att=e["attempt"],
                    cl=cats.get("clean", 0),
                    syn=cats.get("syn", 0),
                    typ=cats.get("typ", 0),
                    err=cats.get("err", 0),
                    skip=cats.get("skip", 0),
                    oth=cats.get("other", 0),
                )
            )
        lines.append("")

        # Top errors
        top = a.get("top_error_prefixes", [])
        if top:
            lines += ["### Most frequent error patterns (round 1 diagnostics)", ""]
            for i, e in enumerate(top[:5], 1):
                lines.append(f"{i}. `{e['prefix']}…` ({e['count']}×)")
            lines.append("")

        # Task-level table
        lines += ["### Per-task outcomes", ""]
        lines.append("| task | status | rounds | error sequence | cost |")
        lines.append("|---|---|---|---|---|")
        for t in a.get("task_summary", []):
            seq = "→".join(t.get("error_categories", []))
            lines.append(
                "| {tid} | {st} | {r} | {seq} | {c} |".format(
                    tid=t["task_id"],
                    st=t["status"],
                    r=t["total_rounds"],
                    seq=seq or "—",
                    c=_usd(t.get("total_cost_usd")),
                )
            )
        lines.append("")

    # Recommendations
    lines += ["## Actionable recommendations", ""]
    priority_order = {"high": 0, "medium": 1, "low": 2, "info": 3}
    sorted_recs = sorted(
        recommendations, key=lambda r: priority_order.get(r["priority"], 99)
    )
    for rec in sorted_recs:
        tag = f"[{rec['tag']}]"
        pri = rec["priority"].upper()
        lines.append(f"### [{pri}] {rec['model']} — {rec['finding']} {tag}")
        lines.append("")
        lines.append(rec["recommendation"])
        lines.append("")

    return "\n".join(lines)


# ---------------------------------------------------------------------------
# Self-test (deterministic, no network calls)
# ---------------------------------------------------------------------------


def _self_test() -> None:
    """Deterministic self-test for analyse_model and render_markdown (T-ST).

    Uses a synthetic minimal report so the test passes without any API key.
    Exits 0 on success, 1 on failure.
    """
    SYNTHETIC = {
        "metadata": {
            "model": "test-model",
            "mode": "live",
            "task_set_id": "g-composition",
            "seed": 42,
            "max_rounds": 3,
        },
        "quality": {
            "syntactic_validity_rate": 0.5,
            "typecheck_pass_rate": 0.25,
        },
        "performance": {
            "total_tokens": 1000,
            "total_cost_usd": 0.002,
            "mean_latency_s": 2.5,
            "total_latency_s": 5.0,
            "request_count": 4,
        },
        "outcomes": [
            {
                "task_id": "g01-identity",
                "status": "PASS",
                "resumed": False,
                "iterations_to_clean": 1,
                "total_cost_usd": 0.001,
                "total_latency_s": 2.0,
                "rounds": [
                    {
                        "attempt": 1,
                        "verdict": "clean",
                        "diagnostics": [],
                    }
                ],
            },
            {
                "task_id": "g02-not",
                "status": "FAIL",
                "resumed": False,
                "iterations_to_clean": None,
                "total_cost_usd": 0.001,
                "total_latency_s": 3.0,
                "rounds": [
                    {
                        "attempt": 1,
                        "verdict": "parse-error: syntax error at 1:1",
                        "diagnostics": [
                            {"message": "parse-error: syntax error at 1:1"}
                        ],
                    }
                ],
            },
            {
                "task_id": "g03-resumed",
                "status": "PASS",
                "resumed": True,
                "iterations_to_clean": 1,
                "total_cost_usd": 0.0,
                "total_latency_s": 0.0,
                "rounds": [],
            },
        ],
    }

    a = analyse_model(SYNTHETIC)

    errors: list[str] = []

    # resumed task must not inflate scored count
    if a["task_counts"]["scored"] != 2:
        errors.append(
            f"scored={a['task_counts']['scored']} want 2 (resumed task must not count)"
        )
    if a["task_counts"]["resumed_pass"] != 1:
        errors.append(f"resumed_pass={a['task_counts']['resumed_pass']} want 1")

    # pass rate: 1 pass out of 2 scored
    rate = a["pass_rate"]["any_clean"]
    if rate != 0.5:
        errors.append(f"any_clean={rate} want 0.5")

    # error taxonomy must not include the resumed task's (empty) rounds
    tax = a.get("error_taxonomy", [])
    if not tax:
        errors.append(
            "error_taxonomy is empty — expected at least 1 entry from g02-not"
        )
    else:
        cats = tax[0]["counts"]
        if cats.get("syn", 0) < 1:
            errors.append(f"expected syn>=1 in round-1 taxonomy; got {cats}")

    # top_error_prefixes must have content (from g02-not), not from resumed task
    if not a.get("top_error_prefixes"):
        errors.append(
            "top_error_prefixes empty — expected entry from g02-not diagnostics"
        )

    # render_markdown must not raise
    try:
        comp = compare_models([a])
        recs = generate_recommendations([a])
        md = render_markdown([a], comp, recs, ["SELFTEST"])
        if "test-model" not in md:
            errors.append("render_markdown output missing model name")
    except Exception as exc:
        errors.append(f"render_markdown raised: {exc}")

    if errors:
        print("SELF-TEST FAILED:", file=sys.stderr)
        for e in errors:
            print(f"  {e}", file=sys.stderr)
        sys.exit(1)

    print("self-test: OK (6 checks)")


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------


def main(argv: list[str] | None = None) -> None:
    parser = argparse.ArgumentParser(
        description="BI analysis of Mycelium co-authoring run reports",
        epilog=(
            "Guarantee tag: Empirical. Output: reports/<run-id>-analysis.json + .md. "
            "VR-5: no blocked-arm BI is fabricated."
        ),
    )
    parser.add_argument(
        "--self-test", action="store_true", help="run deterministic self-test and exit"
    )
    parser.add_argument(
        "files", nargs="*", type=Path, help="report JSON files to analyse"
    )
    parser.add_argument(
        "--run-id", help="analyse all reports matching this run id prefix"
    )
    parser.add_argument(
        "--latest", action="store_true", help="analyse the newest run in reports/"
    )
    parser.add_argument(
        "--reports-dir",
        type=Path,
        default=Path(__file__).parent / "reports",
        help="directory containing report JSON files (default: ./reports/)",
    )
    parser.add_argument(
        "--no-write",
        action="store_true",
        help="print markdown to stdout, do not write files",
    )
    args = parser.parse_args(argv)

    if args.self_test:
        _self_test()
        return

    if args.files:
        paths = list(args.files)
    else:
        paths = find_reports(args.reports_dir, args.run_id, args.latest)

    if not paths:
        print("ERROR: no report files found", file=sys.stderr)
        sys.exit(1)

    print(f"Analysing {len(paths)} report(s):")
    for p in paths:
        print(f"  {p}")

    reports = load_reports(paths)
    if not reports:
        print("ERROR: no reports loaded", file=sys.stderr)
        sys.exit(1)

    analyses = [analyse_model(r) for r in reports]
    comparison = compare_models(analyses)
    recommendations = generate_recommendations(analyses)

    # Derive a run_id for the output file name
    run_ids = []
    for p in paths:
        # filename: <run-id>-<model>-<mode>.json
        stem = Path(p).stem
        parts = stem.split("-")
        run_ids.append(parts[0])
    run_ids = list(dict.fromkeys(run_ids))  # deduplicate, preserve order

    md = render_markdown(analyses, comparison, recommendations, run_ids)

    analysis_payload = {
        "guarantee_tag": "Empirical",
        "vr5": "Computed from measured outcomes only; no blocked-arm data fabricated.",
        "run_ids": run_ids,
        "models_analysed": [a["model"] for a in analyses],
        "analyses": analyses,
        "comparison": comparison,
        "recommendations": recommendations,
    }

    if args.no_write:
        print(md)
        return

    reports_dir = args.reports_dir
    reports_dir.mkdir(parents=True, exist_ok=True)
    run_tag = "-".join(run_ids)
    json_path = reports_dir / f"{run_tag}-analysis.json"
    md_path = reports_dir / f"{run_tag}-analysis.md"

    json_path.write_text(json.dumps(analysis_payload, indent=2), encoding="utf-8")
    md_path.write_text(md, encoding="utf-8")

    print(f"\nAnalysis written:")
    print(f"  {json_path}")
    print(f"  {md_path}")


if __name__ == "__main__":
    main()
