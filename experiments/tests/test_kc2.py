"""Tests for the KC-2 harness (M-002): the benchmark is well-posed, the metrics are computed as
defined, the edit-to-fix loop feeds diagnostics back, and no verdict is pre-written (VR-5)."""

from __future__ import annotations

import pytest

from mycelium_experiments.kc2 import (
    TASKS,
    BaselineChecker,
    MyceliumChecker,
    StaticGenerator,
    ToolUnavailable,
    run_arm,
    run_experiment,
)


@pytest.fixture(scope="module")
def myc_checker() -> MyceliumChecker:
    """The Mycelium oracle; skip (never silently pass) when the Rust toolchain is absent."""
    try:
        return MyceliumChecker()
    except ToolUnavailable as e:  # pragma: no cover — environment-dependent
        pytest.skip(f"myc-check unavailable: {e}")


def test_every_reference_mycelium_solution_passes(myc_checker: MyceliumChecker) -> None:
    """Well-posedness: each task is solvable in the Mycelium fragment, per the real checker."""
    for task in TASKS:
        result = myc_checker.check(task.reference_mycelium, task)
        assert result.passes, f"{task.id}: {result.diagnostic}"


def test_every_reference_baseline_solution_passes() -> None:
    """Well-posedness: each task is solvable in the Python-embedded DSL."""
    checker = BaselineChecker()
    for task in TASKS:
        result = checker.check(task.reference_baseline, task)
        assert result.passes, f"{task.id}: {result.diagnostic}"


def test_the_mycelium_checker_separates_syntax_from_typecheck(
    myc_checker: MyceliumChecker,
) -> None:
    """M-002 measures syntactic validity and type-check pass separately — so must the oracle."""
    task = TASKS[0]
    unparsable = myc_checker.check("colony bench\nfn main() -> Binary{8} = (", task)
    assert not unparsable.syntactically_valid
    assert not unparsable.passes
    ill_typed = myc_checker.check("colony bench\nfn main() -> Binary{8} = nope(0b1)", task)
    assert ill_typed.syntactically_valid
    assert not ill_typed.passes
    wrong_task = myc_checker.check("colony bench\nfn main() -> Binary{4} = 0b1010", task)
    assert wrong_task.syntactically_valid
    assert not wrong_task.passes, "a well-typed program that ignores the task must not pass"


def test_edit_to_fix_loop_counts_iterations_and_feeds_back() -> None:
    """A generator that fixes its program on the second attempt scores iterations == 2."""
    task = TASKS[0]
    checker = BaselineChecker()
    generator = StaticGenerator(
        scripts={
            (task.id, "baseline"): (
                "def main(:\n    return 1\n",  # attempt 1: syntax error
                task.reference_baseline,  # attempt 2: fixed
            )
        }
    )
    report = run_arm(generator, checker, "baseline", tasks=[task], max_iters=3)
    (outcome,) = report.outcomes
    assert not outcome.first_attempt_valid
    assert outcome.passed
    assert outcome.iterations == 2
    assert len(outcome.diagnostics) == 1, "the failure's diagnostic became feedback"
    # The generator saw the feedback grow: first call 0 prior failures, second call 1.
    assert [c[2] for c in generator.calls] == [0, 1]


def test_a_never_passing_task_consumes_the_budget() -> None:
    task = TASKS[0]
    checker = BaselineChecker()
    generator = StaticGenerator(scripts={(task.id, "baseline"): ("def main(:\n",)})
    report = run_arm(generator, checker, "baseline", tasks=[task], max_iters=3)
    (outcome,) = report.outcomes
    assert not outcome.passed
    assert outcome.iterations == 3
    assert report.eventual_pass_rate == 0.0
    assert report.mean_iterations_to_pass is None


def test_metrics_are_first_attempt_based(myc_checker: MyceliumChecker) -> None:
    """SC-5b judges generation: validity/pass rates count the first attempt only."""
    tasks = TASKS[:2]
    scripts = {
        # Task 1: broken first, fixed second → counts against first-attempt rates.
        (tasks[0].id, "mycelium"): (
            "colony bench\nfn main() -> Binary{8} = (",
            tasks[0].reference_mycelium,
        ),
        # Task 2: right first time.
        (tasks[1].id, "mycelium"): (tasks[1].reference_mycelium,),
    }
    report = run_arm(StaticGenerator(scripts=scripts), myc_checker, "mycelium", tasks, 3)
    assert report.syntactic_validity_rate == 0.5
    assert report.first_attempt_pass_rate == 0.5
    assert report.eventual_pass_rate == 1.0
    assert report.mean_iterations_to_pass == 1.5


def test_the_report_never_prewrites_a_verdict(myc_checker: MyceliumChecker) -> None:
    """VR-5: even a 100%-pass run reports the KC-2 verdict as not established."""
    tasks = TASKS[:1]
    generator = StaticGenerator(
        scripts={
            (tasks[0].id, "mycelium"): (tasks[0].reference_mycelium,),
            (tasks[0].id, "baseline"): (tasks[0].reference_baseline,),
        }
    )
    report = run_experiment(generator, myc_checker, BaselineChecker(), tasks, 3)
    assert report["sc5b"] == 1.0
    assert "not established" in str(report["verdict"])
