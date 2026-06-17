"""Sequential, instrumented KC-2 run orchestration (M-002).

One *run* = the task suite under one (seed, arms, budget) config. A *suite* = several
runs executed back-to-back, **unattended**, each writing its own report + summary + log,
plus a combined `index.json`. Captures richer per-attempt metrics (generated source,
checker verdict, generation wall-time) than the bare outcome rates.

Honesty unchanged (G2/VR-5): an unavailable checker SKIPs its arm with a reason; the
KC-2 *verdict* is never computed here — only measured rates + descriptive assessment.
"""

from __future__ import annotations

import datetime
import json
import logging
import sys
from collections.abc import Callable, Sequence
from dataclasses import dataclass
from pathlib import Path

from mycelium_experiments.kc2 import llm
from mycelium_experiments.kc2.checkers import (
    BaselineChecker,
    CheckResult,
    MyceliumChecker,
    ToolUnavailable,
)
from mycelium_experiments.kc2.harness import ArmReport, run_arm
from mycelium_experiments.kc2.summary import assess, render_summary
from mycelium_experiments.kc2.tasks import TASKS

VERDICT = (
    "not established — the KC-2 verdict (proceed / reweight-to-human / "
    "fall-back-to-embedded-DSL) requires a maintainer-written analysis of this run; "
    "this harness never pre-writes it (VR-5)"
)

# A backend factory: seed -> Backend (so each run gets its own seeded backend while a
# shared server stays loaded across runs).
BackendFactory = Callable[[int], llm.Backend]


def now_utc() -> str:
    return datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%SZ")


def make_logger(log_path: Path) -> logging.Logger:
    """A file+stderr logger for one suite (full trace to the file, INFO to stderr)."""
    log = logging.getLogger(f"kc2.runner.{log_path.stem}")
    log.setLevel(logging.DEBUG)
    log.handlers.clear()
    fh = logging.FileHandler(log_path, encoding="utf-8")
    fh.setLevel(logging.DEBUG)
    fh.setFormatter(
        logging.Formatter("%(asctime)s  %(levelname)-7s  %(message)s", "%Y-%m-%dT%H:%M:%SZ")
    )
    sh = logging.StreamHandler(sys.stderr)
    sh.setLevel(logging.INFO)
    sh.setFormatter(logging.Formatter("%(message)s"))
    log.addHandler(fh)
    log.addHandler(sh)
    return log


@dataclass(frozen=True)
class RunConfig:
    """One run's knobs."""

    name: str
    arms: tuple[str, ...] = ("mycelium",)
    seed: int = 42
    max_iters: int = 3
    allow_untrusted_baseline: bool = False


def _arm_metrics(report: ArmReport) -> dict[str, object]:
    return {
        "ran": True,
        "syntactic_validity_rate": report.syntactic_validity_rate,
        "first_attempt_pass_rate": report.first_attempt_pass_rate,
        "eventual_pass_rate": report.eventual_pass_rate,
        "mean_iterations_to_pass": report.mean_iterations_to_pass,
        "outcomes": [
            {
                "task": o.task_id,
                "first_attempt_valid": o.first_attempt_valid,
                "first_attempt_passed": o.first_attempt_passed,
                "passed": o.passed,
                "iterations": o.iterations,
            }
            for o in report.outcomes
        ],
    }


def _checker_for(arm: str, cfg: RunConfig, log: logging.Logger) -> object | str:
    """Return a checker, or a SKIP-reason string (never-silent)."""
    if arm == "mycelium":
        try:
            return MyceliumChecker()
        except ToolUnavailable as exc:
            return str(exc)
    if not cfg.allow_untrusted_baseline:
        return (
            "baseline arm executes model-generated Python in-process; re-run with "
            "--allow-untrusted-baseline inside a sandbox (container/VM)"
        )
    log.warning("baseline arm executes generated code in-process — ensure a sandbox.")
    return BaselineChecker(allow_untrusted=True)


def run_one(
    cfg: RunConfig,
    generator: llm.LlamaGenerator,
    *,
    model_label: str,
    backend_label: str,
    log: logging.Logger,
) -> dict[str, object]:
    """Execute one run config; return the report document (assessment included)."""
    arms_report: dict[str, object] = {}
    attempts: list[dict[str, object]] = []

    for arm in cfg.arms:
        checker = _checker_for(arm, cfg, log)
        if isinstance(checker, str):  # SKIP reason
            arms_report[arm] = {"ran": False, "skipped": checker}
            log.warning("SKIP %s arm: %s", arm, checker.splitlines()[0])
            continue

        def record(
            task_id: str, a: str, attempt: int, source: str, result: CheckResult, wall: float
        ) -> None:
            attempts.append(
                {
                    "task": task_id,
                    "arm": a,
                    "attempt": attempt,
                    "syntactically_valid": result.syntactically_valid,
                    "passed": result.passes,
                    "wall_seconds": wall,
                    "diagnostic": result.diagnostic,
                    "source": source,
                }
            )
            log.info(
                "[%s] %-22s attempt %d: valid=%s passed=%s  %.1fs",
                a,
                task_id,
                attempt,
                result.syntactically_valid,
                result.passes,
                wall,
            )

        report = run_arm(generator, checker, arm, TASKS, cfg.max_iters, on_attempt=record)
        arms_report[arm] = _arm_metrics(report)

    myc = arms_report.get("mycelium", {})
    sc5b = myc.get("first_attempt_pass_rate") if isinstance(myc, dict) else None
    total_wall = round(sum(float(a["wall_seconds"]) for a in attempts), 2)
    doc: dict[str, object] = {
        "experiment": "KC-2 LLM-leverage (M-002; Foundation §6 P0.2)",
        "run": cfg.name,
        "run_utc": now_utc(),
        "backend": backend_label,
        "model": model_label,
        "task_count": len(TASKS),
        "edit_to_fix_budget": cfg.max_iters,
        "seed": cfg.seed,
        "arms": arms_report,
        "attempts": attempts,
        "timing": {
            "total_generation_seconds": total_wall,
            "attempts": len(attempts),
            "mean_attempt_seconds": round(total_wall / len(attempts), 2) if attempts else None,
        },
        "sc5b": sc5b,
        "verdict": VERDICT,
    }
    doc["assessment"] = assess(doc)
    return doc


def write_run(doc: dict[str, object], results_dir: Path, log: logging.Logger) -> Path:
    """Write <results_dir>/<run_utc>-<name>.{json,summary.txt}; return the JSON path."""
    results_dir.mkdir(parents=True, exist_ok=True)
    stem = f"{doc['run_utc']}-{doc['run']}"
    jpath = results_dir / f"{stem}.json"
    jpath.write_text(json.dumps(doc, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    (results_dir / f"{stem}.summary.txt").write_text(
        render_summary(doc, doc["assessment"]), encoding="utf-8"
    )
    log.info("wrote %s (+ .summary.txt)", jpath.name)
    return jpath


def run_suite(
    configs: Sequence[RunConfig],
    *,
    backend_factory: BackendFactory,
    primers: dict[str, str],
    model_label: str,
    backend_label: str,
    results_dir: Path,
    log: logging.Logger,
) -> dict[str, object]:
    """Run each config back-to-back, write per-run files + an index.json. Returns the index."""
    index: dict[str, object] = {
        "suite_utc": now_utc(),
        "model": model_label,
        "backend": backend_label,
        "runs": [],
    }
    runs_list: list[dict[str, object]] = index["runs"]  # type: ignore[assignment]
    for cfg in configs:
        log.info(
            "=== run '%s' (seed=%d, arms=%s, max_iters=%d) ===",
            cfg.name,
            cfg.seed,
            ",".join(cfg.arms),
            cfg.max_iters,
        )
        generator = llm.LlamaGenerator(backend=backend_factory(cfg.seed), primers=primers)
        doc = run_one(cfg, generator, model_label=model_label, backend_label=backend_label, log=log)
        path = write_run(doc, results_dir, log)
        myc = doc["arms"].get("mycelium", {}) if isinstance(doc["arms"], dict) else {}
        runs_list.append(
            {
                "run": cfg.name,
                "seed": cfg.seed,
                "report": path.name,
                "sc5b": doc["sc5b"],
                "mycelium_ran": bool(isinstance(myc, dict) and myc.get("ran")),
            }
        )

    (results_dir / "index.json").write_text(
        json.dumps(index, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    log.info("suite complete — index: %s", results_dir / "index.json")
    return index
