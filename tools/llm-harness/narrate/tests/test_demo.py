"""Demo + output-writer tests — end-to-end run, provenance, byte-idempotence."""

from __future__ import annotations

from narrate.checker import MockChecker
from narrate.demo import run_demo
from narrate.generator import CachingGenerator, MockGenerator
from narrate.report import provenance_header, write_output
from narrate.session import narrate_unit


def test_demo_runs_green():
    assert run_demo() == 0


def test_output_has_provenance_header_and_is_idempotent(
    result_facts, ref_template, tmp_path
):
    gen = CachingGenerator(base=MockGenerator(), cache_dir=tmp_path / "c")
    run = narrate_unit(result_facts, ref_template, gen, MockChecker(), max_rounds=3)

    out1 = write_output(run, tmp_path / "out")
    bytes1 = out1.read_bytes()

    # a second identical run writes byte-identical output (no wall-clock drift)
    run2 = narrate_unit(result_facts, ref_template, gen, MockChecker(), max_rounds=3)
    out2 = write_output(run2, tmp_path / "out2")
    assert out1.read_bytes() == out2.read_bytes() == bytes1

    header = provenance_header(run)
    assert "validated_fraction:" in header
    assert f"model_id:           {run.model_id}" in header
    assert "src:lib/std/result.myc:23" in header  # a real fact doc_ref
    assert run.guarantee_tag in header


def test_committed_prose_only_contains_licensed_docrefs(result_facts, ref_template):
    run = narrate_unit(
        result_facts, ref_template, MockGenerator(), MockChecker(), max_rounds=3
    )
    licensed = result_facts.doc_refs()
    from narrate.checker import extract_doc_refs

    for ref in extract_doc_refs(run.committed_prose):
        assert ref in licensed  # never cite a fact we weren't given
