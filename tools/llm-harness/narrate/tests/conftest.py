"""Shared fixtures for the narrate test suite.

The fixtures are data-driven (CLAUDE.md test-layout rule: complex logic lives in
fixtures, test bodies assert over a case).  They load ONE real committed unit
(``std.result``) so the suite exercises the real index, and provide a synthetic
FactSet so the checker/grounding tests don't depend on corpus contents.
"""

from __future__ import annotations

import pytest

from narrate.facts import Fact, FactSet, find_repo_root, lib_index_path, load_facts
from narrate.prompts import load_template

DEMO_UNIT = "std.result"


@pytest.fixture(scope="session")
def repo_root():
    return find_repo_root()


@pytest.fixture(scope="session")
def lib_index(repo_root):
    path = lib_index_path(repo_root)
    if not path.is_file():
        pytest.skip(f"committed lib-index not present at {path}")
    return path


@pytest.fixture()
def result_facts(lib_index) -> FactSet:
    """The real ``std.result`` fact set from the committed lib-index."""
    return load_facts(lib_index, DEMO_UNIT)


@pytest.fixture()
def synthetic_facts() -> FactSet:
    """A small, self-contained fact set (no corpus dependency)."""
    facts = [
        Fact(
            id="demo.unit",
            kind="nodule",
            unit="demo.unit",
            source_path="lib/demo/unit.myc",
            line=1,
            signature="nodule demo.unit",
            summary="A tiny demo nodule with one function.",
            guarantee_tag="Empirical/Declared",
            documented=True,
        ),
        Fact(
            id="demo.unit::twice",
            kind="fn",
            unit="demo.unit",
            source_path="lib/demo/unit.myc",
            line=3,
            signature="fn twice(x: Nat) => Nat",
            summary="twice: double the input value x.",
            guarantee_tag="Empirical/Declared",
            documented=True,
        ),
        Fact(
            id="demo.unit::Shadow",
            kind="type",
            unit="demo.unit",
            source_path="lib/demo/unit.myc",
            line=5,
            signature="type Shadow = Dim | Bright",
            summary=None,
            guarantee_tag="Empirical/Declared",
            documented=False,
        ),
    ]
    return FactSet(unit="demo.unit", facts=facts, source_index="<synthetic>")


@pytest.fixture()
def ref_template():
    return load_template("ref-manual-entry")
