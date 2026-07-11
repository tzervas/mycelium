"""Loop tests — validation status, self-correction, VR-5 tag enforcement, SKIP."""

from __future__ import annotations

import pytest

from narrate.checker import MockChecker
from narrate.generator import MockGenerator
from narrate.session import (
    STATUS_PARTIAL,
    STATUS_PARTIAL_DROPPED,
    STATUS_SKIP,
    STATUS_VALIDATED,
    assert_model_tag,
    narrate_unit,
)


def test_clean_run_is_validated_first_round(synthetic_facts, ref_template):
    run = narrate_unit(
        synthetic_facts, ref_template, MockGenerator(), MockChecker(), max_rounds=3
    )
    assert run.status == STATUS_VALIDATED
    assert run.validated_fraction == 1.0
    assert run.guarantee_tag == "Declared"  # mock output, VR-5
    assert run.committed_prose.strip()
    assert len(run.rounds) == 1


def test_self_correction_reaches_full_validation(synthetic_facts, ref_template):
    # inject a hallucination; the mock self-corrects by dropping it on round 2
    gen = MockGenerator(inject_hallucination="frobnicate")
    run = narrate_unit(synthetic_facts, ref_template, gen, MockChecker(), max_rounds=3)
    assert run.status == STATUS_PARTIAL  # became fully validated after correction
    assert run.validated_fraction == 1.0
    assert len(run.rounds) >= 2
    # round 1 caught the injection; the final committed prose omits it
    assert run.rounds[0].result.validated_fraction < 1.0
    assert "frobnicate" not in run.committed_prose


def test_dropped_sentences_recorded_when_uncorrectable(synthetic_facts, ref_template):
    # a generator that keeps re-injecting -> never fully validates, but the
    # residual drop is recorded and the good prose is still committed (never trash)
    class Persistent(MockGenerator):
        def generate(self, facts, template, prior, feedback):  # noqa: ANN001
            # ignore feedback: always re-inject (adversarial)
            return MockGenerator(inject_hallucination="frobnicate").generate(
                facts, template, prior, []
            )

    run = narrate_unit(
        synthetic_facts, ref_template, Persistent(), MockChecker(), max_rounds=2
    )
    assert run.status == STATUS_PARTIAL_DROPPED
    assert run.validated_fraction < 1.0
    assert run.final is not None
    assert len(run.final.result.dropped) >= 1
    assert "frobnicate" not in run.committed_prose  # dropped, never committed


def test_vr5_rejects_forbidden_tag(synthetic_facts, ref_template):
    class ProvenGen(MockGenerator):
        guarantee_tag = "Proven"  # forbidden for model output

    with pytest.raises(ValueError, match="VR-5"):
        narrate_unit(
            synthetic_facts, ref_template, ProvenGen(), MockChecker(), max_rounds=1
        )


@pytest.mark.parametrize("bad", ["Proven", "Exact", "Bogus"])
def test_assert_model_tag_rejects(bad):
    with pytest.raises(ValueError):
        assert_model_tag(bad, "claim")


@pytest.mark.parametrize("ok", ["Empirical", "Declared"])
def test_assert_model_tag_accepts(ok):
    assert_model_tag(ok, "claim")  # no raise


def test_empty_generator_is_skip(synthetic_facts, ref_template):
    class Empty(MockGenerator):
        def generate(self, facts, template, prior, feedback):  # noqa: ANN001
            return ""

    run = narrate_unit(
        synthetic_facts, ref_template, Empty(), MockChecker(), max_rounds=2
    )
    assert run.status == STATUS_SKIP  # never a false PASS (G2)
    assert run.committed_prose == ""
