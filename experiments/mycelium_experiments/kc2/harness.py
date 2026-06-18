"""The KC-2 measurement loop (M-002; SC-5b; G10).

Metric definitions (fixed here so a future run is comparable):

- **syntactic validity** — the *first* attempt parses (SC-5b judges generation, so first-shot);
- **type-check pass rate** — the *first* attempt fully passes its arm's checker (for the
  Mycelium arm: parse + typecheck + task signature). This is the SC-5b "X%" number;
- **edit-to-fix iterations** — attempts consumed until the checker passes, the generator
  receiving each failure's diagnostic as feedback (G10's "projections + semantic feedback"
  leverage); tasks that never pass within the budget are reported separately, not averaged in.

No KC-2 verdict is computed here: the verdict (proceed / reweight-to-human /
fall-back-to-embedded-DSL) is a maintainer-written analysis of a real model run, and a guarantee
or kill-criterion verdict is never pre-written (VR-5). The report says exactly that.
"""

from __future__ import annotations

import json
import time
from collections.abc import Callable, Mapping, Sequence
from dataclasses import dataclass, field
from typing import Protocol

from mycelium_experiments.kc2.checkers import CheckResult
from mycelium_experiments.kc2.tasks import TASKS, Task

# A per-attempt observer: (task_id, arm, attempt_1based, source, result, gen_wall_seconds).
# Optional hook so a runner can capture richer metrics/logs without the loop owning them.
AttemptObserver = Callable[[str, str, int, str, CheckResult, float], None]


class Generator(Protocol):
    """A program generator: the subject under measurement (an LLM adapter, in the real run)."""

    def __call__(self, task: Task, arm: str, feedback: Sequence[str]) -> str:
        """Produce source for `task` in `arm` ("mycelium" | "baseline"); `feedback` holds the
        diagnostics of every prior failed attempt (empty on the first attempt)."""
        ...


class Checker(Protocol):
    """An arm's pass/fail oracle."""

    def check(self, source: str, task: Task) -> CheckResult:
        """Judge one attempt."""
        ...


@dataclass(frozen=True)
class TaskOutcome:
    """The measured outcome of one task in one arm."""

    task_id: str
    first_attempt_valid: bool
    first_attempt_passed: bool
    passed: bool
    iterations: int
    """Attempts consumed: the passing attempt's index (1-based), or the budget if never passed."""
    diagnostics: tuple[str, ...]


@dataclass(frozen=True)
class ArmReport:
    """Aggregated metrics for one arm."""

    arm: str
    outcomes: tuple[TaskOutcome, ...]

    @property
    def syntactic_validity_rate(self) -> float:
        """Share of tasks whose first attempt parsed."""
        return _rate(self.outcomes, lambda o: o.first_attempt_valid)

    @property
    def first_attempt_pass_rate(self) -> float:
        """Share of tasks whose first attempt fully passed — the SC-5b number."""
        return _rate(self.outcomes, lambda o: o.first_attempt_passed)

    @property
    def eventual_pass_rate(self) -> float:
        """Share of tasks that passed within the edit-to-fix budget."""
        return _rate(self.outcomes, lambda o: o.passed)

    @property
    def mean_iterations_to_pass(self) -> float | None:
        """Mean attempts over the tasks that eventually passed; None if none did."""
        passed = [o.iterations for o in self.outcomes if o.passed]
        return sum(passed) / len(passed) if passed else None


def _rate(outcomes: tuple[TaskOutcome, ...], pred: Callable[[TaskOutcome], bool]) -> float:
    return sum(1 for o in outcomes if pred(o)) / len(outcomes) if outcomes else 0.0


def run_arm(
    generator: Generator,
    checker: Checker,
    arm: str,
    tasks: Sequence[Task] = TASKS,
    max_iters: int = 3,
    on_attempt: AttemptObserver | None = None,
) -> ArmReport:
    """Run every task through the generate → check → feedback loop for one arm.

    ``on_attempt`` (optional) is called once per attempt with the generated source, the
    checker's verdict, and the generation wall-time — so a runner can record per-attempt
    metrics/logs without this loop owning them. It never affects the measured outcome.
    """
    outcomes: list[TaskOutcome] = []
    for task in tasks:
        feedback: list[str] = []
        first_valid = False
        first_passed = False
        passed = False
        used = max_iters
        for attempt in range(1, max_iters + 1):
            t0 = time.monotonic()
            source = generator(task, arm, tuple(feedback))
            gen_wall = time.monotonic() - t0
            result = checker.check(source, task)
            if on_attempt is not None:
                on_attempt(task.id, arm, attempt, source, result, round(gen_wall, 3))
            if attempt == 1:
                first_valid = result.syntactically_valid
                first_passed = result.passes
            if result.passes:
                passed = True
                used = attempt
                break
            feedback.append(result.diagnostic)
        outcomes.append(
            TaskOutcome(
                task_id=task.id,
                first_attempt_valid=first_valid,
                first_attempt_passed=first_passed,
                passed=passed,
                iterations=used,
                diagnostics=tuple(feedback),
            )
        )
    return ArmReport(arm=arm, outcomes=tuple(outcomes))


def run_experiment(
    generator: Generator,
    mycelium_checker: Checker,
    baseline_checker: Checker,
    tasks: Sequence[Task] = TASKS,
    max_iters: int = 3,
) -> dict[str, object]:
    """Run both arms and assemble the comparison report (JSON-serializable).

    The report's ``verdict`` field is fixed text: KC-2's verdict is not established by this
    harness — it requires a maintainer-written analysis of a real generator run (VR-5).
    """
    arms = {
        "mycelium": run_arm(generator, mycelium_checker, "mycelium", tasks, max_iters),
        "baseline": run_arm(generator, baseline_checker, "baseline", tasks, max_iters),
    }
    return {
        "experiment": "KC-2 LLM-leverage (M-002; Foundation §6 P0.2)",
        "task_count": len(tasks),
        "edit_to_fix_budget": max_iters,
        "arms": {
            name: {
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
            for name, report in arms.items()
        },
        "sc5b": arms["mycelium"].first_attempt_pass_rate,
        "verdict": (
            "not established — the KC-2 verdict (proceed / reweight-to-human / "
            "fall-back-to-embedded-DSL) requires a maintainer-written analysis of a real "
            "generator run; this harness never pre-writes it (VR-5)"
        ),
    }


def report_json(report: Mapping[str, object]) -> str:
    """Pretty JSON for filing with the experiment record."""
    return json.dumps(report, indent=2, sort_keys=True)


@dataclass
class StaticGenerator:
    """A fixture-backed generator for testing the harness itself (not a measurement subject).

    ``scripts`` maps ``(task_id, arm)`` to the sequence of sources to emit attempt-by-attempt;
    the last entry repeats once exhausted. Tasks without a script raise — a missing fixture is
    a test bug, not a measurable outcome.
    """

    scripts: Mapping[tuple[str, str], Sequence[str]]
    calls: list[tuple[str, str, int]] = field(default_factory=list)

    def __call__(self, task: Task, arm: str, feedback: Sequence[str]) -> str:
        attempts = self.scripts[(task.id, arm)]
        index = min(len(feedback), len(attempts) - 1)
        self.calls.append((task.id, arm, len(feedback)))
        return attempts[index]
