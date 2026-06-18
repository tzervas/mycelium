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
import time
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
    fh_fmt = logging.Formatter("%(asctime)s  %(levelname)-7s  %(message)s", "%Y-%m-%dT%H:%M:%SZ")
    fh_fmt.converter = time.gmtime  # the `Z` claims UTC — make asctime actually UTC, not local
    fh.setFormatter(fh_fmt)
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


def _partial_arm_metrics(
    arm: str, attempts: list[dict[str, object]], max_iters: int
) -> dict[str, object]:
    """Reconstruct an arm's metrics from the attempts captured before an interruption.

    Same shape as ``_arm_metrics`` (so ``assess`` reads it) but flagged ``partial`` and
    scoped to the tasks actually attempted — honest rates over a subset, never faked over
    the full suite.
    """
    by_task: dict[str, list[dict[str, object]]] = {}
    for a in attempts:
        if a["arm"] == arm:
            by_task.setdefault(str(a["task"]), []).append(a)
    n = len(by_task)
    outcomes: list[dict[str, object]] = []
    fv = fp = ev = 0
    iters_to_pass: list[int] = []
    for task, ts in by_task.items():
        ts.sort(key=lambda x: int(x["attempt"]))  # type: ignore[arg-type]
        first = ts[0]
        won = next((x for x in ts if x["passed"]), None)
        fv += bool(first["syntactically_valid"])
        fp += bool(first["passed"])
        if won is not None:
            ev += 1
            iters_to_pass.append(int(won["attempt"]))
        outcomes.append(
            {
                "task": task,
                "first_attempt_valid": first["syntactically_valid"],
                "first_attempt_passed": first["passed"],
                "passed": won is not None,
                "iterations": int(won["attempt"]) if won else len(ts),
            }
        )
    return {
        "ran": True,
        "partial": True,
        "tasks_attempted": n,
        "syntactic_validity_rate": round(fv / n, 3) if n else None,
        "first_attempt_pass_rate": round(fp / n, 3) if n else None,
        "eventual_pass_rate": round(ev / n, 3) if n else None,
        "mean_iterations_to_pass": (
            round(sum(iters_to_pass) / len(iters_to_pass), 3) if iters_to_pass else None
        ),
        "outcomes": outcomes,
    }


def run_one(
    cfg: RunConfig,
    generator: llm.LlamaGenerator,
    *,
    model_label: str,
    backend_label: str,
    results_dir: Path,
    tasks: Sequence[object] = TASKS,
    log: logging.Logger,
) -> dict[str, object]:
    """Execute one run config; return the report document (assessment included).

    Durable: every attempt is appended to ``<stem>.attempts.jsonl`` as it happens, so an
    OOM-kill or outer timeout loses nothing. A backend error mid-arm (e.g. a generation
    that outran the per-attempt ``--timeout``) is caught, the arm is recorded as PARTIAL
    from the attempts so far, and the run is flagged ``interrupted`` — never a lost report.
    """
    run_utc = now_utc()
    stem = f"{run_utc}-{cfg.name}"
    results_dir.mkdir(parents=True, exist_ok=True)
    jsonl = results_dir / f"{stem}.attempts.jsonl"
    arms_report: dict[str, object] = {}
    attempts: list[dict[str, object]] = []
    interrupted: str | None = None

    with jsonl.open("w", encoding="utf-8") as ckpt:

        def record(
            task_id: str, a: str, attempt: int, source: str, result: CheckResult, wall: float
        ) -> None:
            rec = {
                "task": task_id,
                "arm": a,
                "attempt": attempt,
                "syntactically_valid": result.syntactically_valid,
                "passed": result.passes,
                "wall_seconds": wall,
                "diagnostic": result.diagnostic,
                "source": source,
            }
            attempts.append(rec)
            ckpt.write(json.dumps(rec) + "\n")
            ckpt.flush()  # durable per attempt — survive an OOM-kill mid-run
            log.info(
                "[%s] %-22s attempt %d: valid=%s passed=%s  %.1fs",
                a,
                task_id,
                attempt,
                result.syntactically_valid,
                result.passes,
                wall,
            )

        for arm in cfg.arms:
            checker = _checker_for(arm, cfg, log)
            if isinstance(checker, str):  # SKIP reason
                arms_report[arm] = {"ran": False, "skipped": checker}
                log.warning("SKIP %s arm: %s", arm, checker.splitlines()[0])
                continue
            try:
                report = run_arm(generator, checker, arm, tasks, cfg.max_iters, on_attempt=record)
                arms_report[arm] = _arm_metrics(report)
            except RuntimeError as exc:  # backend/server error (e.g. per-attempt timeout)
                interrupted = f"{type(exc).__name__}: {exc}"
                log.error("arm '%s' interrupted — %s", arm, interrupted)
                arms_report[arm] = _partial_arm_metrics(arm, attempts, cfg.max_iters)
                break  # the backend is likely down; stop this run (data is checkpointed)

    myc = arms_report.get("mycelium", {})
    sc5b = myc.get("first_attempt_pass_rate") if isinstance(myc, dict) else None
    total_wall = round(sum(float(a["wall_seconds"]) for a in attempts), 2)
    doc: dict[str, object] = {
        "experiment": "KC-2 LLM-leverage (M-002; Foundation §6 P0.2)",
        "run": cfg.name,
        "run_utc": run_utc,
        "backend": backend_label,
        "model": model_label,
        "task_count": len(tasks),
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
    if interrupted:
        doc["interrupted"] = interrupted
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
    tasks: Sequence[object] = TASKS,
) -> dict[str, object]:
    """Run each config back-to-back, write per-run files + an index.json. Returns the index.

    Writes the index after EVERY run (not just at the end), so an interruption still leaves
    a coherent index of what completed. Stops the sequence if a run is interrupted — a dead
    backend won't recover for the next seed.
    """
    index: dict[str, object] = {
        "suite_utc": now_utc(),
        "model": model_label,
        "backend": backend_label,
        "task_count": len(tasks),
        "runs": [],
    }
    runs_list: list[dict[str, object]] = index["runs"]  # type: ignore[assignment]

    def flush_index() -> None:
        (results_dir / "index.json").write_text(
            json.dumps(index, indent=2, sort_keys=True) + "\n", encoding="utf-8"
        )

    for cfg in configs:
        log.info(
            "=== run '%s' (seed=%d, arms=%s, max_iters=%d, tasks=%d) ===",
            cfg.name,
            cfg.seed,
            ",".join(cfg.arms),
            cfg.max_iters,
            len(tasks),
        )
        generator = llm.LlamaGenerator(backend=backend_factory(cfg.seed), primers=primers)
        doc = run_one(
            cfg,
            generator,
            model_label=model_label,
            backend_label=backend_label,
            results_dir=results_dir,
            tasks=tasks,
            log=log,
        )
        path = write_run(doc, results_dir, log)
        myc = doc["arms"].get("mycelium", {}) if isinstance(doc["arms"], dict) else {}
        runs_list.append(
            {
                "run": cfg.name,
                "seed": cfg.seed,
                "report": path.name,
                "sc5b": doc["sc5b"],
                "mycelium_ran": bool(isinstance(myc, dict) and myc.get("ran")),
                "interrupted": doc.get("interrupted"),
            }
        )
        flush_index()
        if doc.get("interrupted"):
            log.error("stopping the suite — run '%s' was interrupted (see report).", cfg.name)
            break

    log.info("suite complete — index: %s", results_dir / "index.json")
    return index
