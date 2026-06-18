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
    baseline,
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
    # Trusted repo fixtures — deliberate opt-in to in-process exec (A6-10/B2-04).
    checker = BaselineChecker(allow_untrusted=True)
    for task in TASKS:
        result = checker.check(task.reference_baseline, task)
        assert result.passes, f"{task.id}: {result.diagnostic}"


def test_default_baseline_checker_refuses_untrusted_exec() -> None:
    """A6-10/B2-04 regression: a default ``BaselineChecker`` must *refuse* to ``exec`` source —
    in-process execution is a deliberate, visible opt-in, never an accident. Mutant-witness:
    dropping the ``allow_untrusted`` guard would let this benign source run and pass instead of
    returning an explicit refusal diagnostic."""
    task = TASKS[0]
    checker = BaselineChecker()  # no opt-in: must refuse, not execute
    result = checker.check(task.reference_baseline, task)
    assert not result.passes
    assert not result.syntactically_valid
    assert "refusing to exec" in result.diagnostic


def test_reference_baseline_values_match_expected() -> None:
    """A6-04: the reference baselines compute the *expected value*, not merely the right shape — so a
    value-wrong reference, or a baseline↔kernel integer-convention drift (A6-01: the baseline now
    reads `Bin` as two's-complement, matching the kernel), is caught. Scoring stays shape-only for
    SC-5b symmetry; this is a well-posedness assertion over the fixtures only."""
    ns_base: dict[str, object] = {
        "Bin": baseline.Bin,
        "Tern": baseline.Tern,
        "bnot": baseline.bnot,
        "xor": baseline.xor,
        "tadd": baseline.tadd,
        "swap": baseline.swap,
    }
    checked = 0
    for task in TASKS:
        if task.expect_value is None:
            continue
        ns = dict(ns_base)
        exec(task.reference_baseline, ns)  # noqa: S102 — fixture/reference code only (see BaselineChecker)
        result = ns["main"]()  # type: ignore[operator]
        assert result.to_int() == task.expect_value, (
            f"{task.id}: reference value {result.to_int()} != expected {task.expect_value}"
        )
        checked += 1
    assert checked >= 5, "expected several tasks to pin a determinate value"


def test_the_mycelium_checker_separates_syntax_from_typecheck(
    myc_checker: MyceliumChecker,
) -> None:
    """M-002 measures syntactic validity and type-check pass separately — so must the oracle."""
    task = TASKS[0]
    unparsable = myc_checker.check("nodule bench\nfn main() -> Binary{8} = (", task)
    assert not unparsable.syntactically_valid
    assert not unparsable.passes
    ill_typed = myc_checker.check("nodule bench\nfn main() -> Binary{8} = nope(0b1)", task)
    assert ill_typed.syntactically_valid
    assert not ill_typed.passes
    wrong_task = myc_checker.check("nodule bench\nfn main() -> Binary{4} = 0b1010", task)
    assert wrong_task.syntactically_valid
    assert not wrong_task.passes, "a well-typed program that ignores the task must not pass"


def test_edit_to_fix_loop_counts_iterations_and_feeds_back() -> None:
    """A generator that fixes its program on the second attempt scores iterations == 2."""
    task = TASKS[0]
    checker = BaselineChecker(allow_untrusted=True)
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
    checker = BaselineChecker(allow_untrusted=True)
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
            "nodule bench\nfn main() -> Binary{8} = (",
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


# --- primer integrity guards (Mycelium-only context; no leaked answers) ---

import pathlib  # noqa: E402

_PRIMER_DIR = pathlib.Path(__file__).resolve().parents[1] / "primers"


def _programs_in(text: str) -> list[str]:
    """Extract left-margin `nodule …` program blocks (runs of non-blank, non-indented lines)."""
    progs: list[str] = []
    cur: list[str] = []
    for line in text.splitlines():
        if line.startswith("nodule "):
            if cur:
                progs.append("\n".join(cur))
            cur = [line]
        elif cur and line.strip() and not line.startswith((" ", "\t")):
            cur.append(line)
        elif cur:
            progs.append("\n".join(cur))
            cur = []
    if cur:
        progs.append("\n".join(cur))
    return progs


def _all_primer_texts() -> dict[str, str]:
    from mycelium_experiments.kc2.llm import PRIMER_MYCELIUM

    texts = {"PRIMER_MYCELIUM": PRIMER_MYCELIUM}
    for f in sorted(_PRIMER_DIR.glob("mycelium-*.txt")):
        texts[f.name] = f.read_text(encoding="utf-8")
    return texts


def test_primer_worked_examples_are_valid_mycelium(myc_checker: MyceliumChecker) -> None:
    """Guard: every complete program in a primer parses as Mycelium — never a Rust crate/module
    fed by accident (Rust would fail to parse → syntactically_valid False)."""
    found = 0
    for name, text in _all_primer_texts().items():
        for prog in _programs_in(text):
            found += 1
            result = myc_checker.check(prog, TASKS[0])  # any task; we only assert it parses
            assert result.syntactically_valid, (
                f"{name}: not valid Mycelium?\n{prog}\n{result.diagnostic}"
            )
    assert found >= 2, "expected the examples primer's two worked programs"


def test_no_primer_leaks_a_task_answer() -> None:
    """Measurement integrity: no task's answer line appears verbatim in any primer."""
    blob = "\n".join(_all_primer_texts().values())
    for task in TASKS:
        for line in task.reference_mycelium.splitlines():
            line = line.strip()
            if (
                not line
                or line.startswith(("nodule", "type "))
                or len(line) < 22
                or line.endswith("=")
            ):
                continue
            assert line not in blob, f"{task.id} answer line leaked into a primer: {line!r}"
