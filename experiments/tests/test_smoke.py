"""Smoke test: the experiments package imports and the toolchain runs (M-092)."""

from mycelium_experiments import answer


def test_answer() -> None:
    assert answer() == 4
