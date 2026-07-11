"""FaithfulnessChecker tests — grammar, grounding, and the NEGATIVE CONTROL.

The negative control is the crux: a sentence asserting a fact NOT in the fact set
must be caught and dropped, driving ``validated_fraction`` below 1.0 and excluding
the bad sentence from the committed prose.
"""

from __future__ import annotations

import pytest

from narrate.checker import (
    MockChecker,
    code_tokens,
    extract_doc_refs,
    is_wellformed_doc_ref,
    segment,
)
from narrate.generator import MockGenerator

# ---------------------------------------------------------------------------
# doc_refs grammar (mirrors tools/github/doc_refs_check.py shapes)
# ---------------------------------------------------------------------------


@pytest.mark.parametrize(
    "token, ok",
    [
        ("src:lib/std/result.myc:23", True),
        ("src:lib/std/result.myc", True),
        ("api:mycelium-core::binary::bits_to_int", True),
        ("corpus:RFC-0016", True),
        ("corpus:RFC-0016#some-section", True),
        ("nope:foo", False),
        ("srclib/std", False),
    ],
)
def test_doc_ref_grammar(token, ok):
    assert is_wellformed_doc_ref(token) is ok


def test_extract_doc_refs_finds_all_tokens():
    line = "[doc_refs: src:lib/std/result.myc:23 corpus:RFC-0016]"
    refs = extract_doc_refs(line)
    assert refs == ["src:lib/std/result.myc:23", "corpus:RFC-0016"]


# ---------------------------------------------------------------------------
# code-token extraction (grounding candidates)
# ---------------------------------------------------------------------------


def test_backtick_identifiers_are_candidates():
    toks = code_tokens("The `map` function returns `Result[B, E]` here.")
    assert {"map", "Result", "B", "E"} <= toks


def test_plain_english_is_not_code_like():
    # ordinary words (no underscore, no interior capital) are never flagged
    toks = code_tokens("This returns the success value untouched.")
    assert toks == set()


def test_bare_snake_case_is_a_candidate():
    toks = code_tokens("It calls frobnicate_entropy on the colony.")
    assert "frobnicate_entropy" in toks


def test_bare_pascalcase_is_a_candidate():
    # Mycelium type-name convention: bare PascalCase must be flagged (the HIGH
    # blind spot). Ordinary capitalized English (sentence openers) must not be.
    toks = code_tokens("The colony spawns a Frobnicator to shadow Binary.")
    assert "Frobnicator" in toks
    assert "Binary" in toks
    assert "The" not in toks  # common word, not a type name
    assert "colony" not in toks  # bare lowercase is not code-like


def test_interior_capital_is_a_candidate():
    toks = code_tokens("It raises an IOError from mapErr somewhere.")
    assert {"IOError", "mapErr"} <= toks


# ---------------------------------------------------------------------------
# Grounding over a clean, grounded-by-construction emission
# ---------------------------------------------------------------------------


def test_grounded_prose_fully_validates(synthetic_facts, ref_template):
    prose = MockGenerator().generate(synthetic_facts, ref_template, "", [])
    result = MockChecker().check(prose, synthetic_facts)
    assert result.validated_fraction == 1.0
    assert result.dropped == []
    assert result.guarantee_tag == "Empirical"  # a measured verdict, not Proven


def test_real_unit_fully_validates(result_facts, ref_template):
    prose = MockGenerator().generate(result_facts, ref_template, "", [])
    result = MockChecker().check(prose, result_facts)
    assert result.validated_fraction == 1.0


# ---------------------------------------------------------------------------
# NEGATIVE CONTROL — an injected ungrounded sentence MUST be caught + dropped
# ---------------------------------------------------------------------------


def test_negative_control_backticked_hallucination(synthetic_facts, ref_template):
    gen = MockGenerator(inject_hallucination="frobnicate")
    prose = gen.generate(synthetic_facts, ref_template, "", [])
    result = MockChecker().check(prose, synthetic_facts)

    # the crux assertions
    assert result.validated_fraction < 1.0
    assert len(result.dropped) == 1
    bad = result.dropped[0]
    assert "frobnicate" in bad.unsupported_tokens
    assert not bad.validated

    # the bogus sentence is EXCLUDED from committed prose (never silently kept)
    committed = result.validated_prose()
    assert "frobnicate" not in committed
    # but the good, grounded sentences survive
    assert "twice" in committed


def test_negative_control_on_real_unit(result_facts, ref_template):
    gen = MockGenerator(inject_hallucination="Frobnicator")
    prose = gen.generate(result_facts, ref_template, "", [])
    result = MockChecker().check(prose, result_facts)
    assert result.validated_fraction < 1.0
    assert any("Frobnicator" in v.unsupported_tokens for v in result.dropped)
    assert "Frobnicator" not in result.validated_prose()


def test_negative_control_bare_pascalcase_hallucination(synthetic_facts, ref_template):
    # (a) a BARE PascalCase type-name hallucination (NO backticks) — the HIGH
    # blind spot — must be caught and dropped, not vacuously "grounded".
    gen = MockGenerator(inject_hallucination="Frobnicator", inject_style="pascal")
    prose = gen.generate(synthetic_facts, ref_template, "", [])
    assert "`Frobnicator`" not in prose  # the injection is bare, un-backticked
    assert "Frobnicator" in prose

    result = MockChecker().check(prose, synthetic_facts)
    assert result.validated_fraction < 1.0
    assert any("Frobnicator" in v.unsupported_tokens for v in result.dropped)

    committed = result.validated_prose()
    assert "Frobnicator" not in committed  # dropped, never silently kept
    assert "twice" in committed  # grounded sentences survive


def test_negative_control_free_text_is_bucketed(synthetic_facts, ref_template):
    # (b) a pure free-text unsupported claim (zero code tokens, no overlap with
    # the cited facts) must be flagged/bucketed, NOT vacuously passed.
    from narrate.checker import BUCKET_UNVERIFIABLE

    gen = MockGenerator(inject_hallucination="x", inject_style="freetext")
    prose = gen.generate(synthetic_facts, ref_template, "", [])
    result = MockChecker().check(prose, synthetic_facts)

    assert result.validated_fraction < 1.0
    unver = [v for v in result.verdicts if v.bucket == BUCKET_UNVERIFIABLE]
    assert any("whatsoever" in v.text for v in unver)  # the injected free text
    assert all(not v.validated for v in unver)  # never counted as grounded
    assert result.bucket_counts().get(BUCKET_UNVERIFIABLE, 0) >= 1  # reported
    assert "whatsoever" not in result.validated_prose()  # excluded from output


def test_free_text_with_overlap_is_grounded(synthetic_facts):
    # positive control: the overlap gate is a signal, not a blanket reject — a
    # free-text sentence sharing content words with its cited fact IS grounded.
    prose = (
        "# Reference: demo.unit\n\n"
        "The input value doubles here via double.\n"
        "[doc_refs: src:lib/demo/unit.myc:3]\n"
    )
    result = MockChecker().check(prose, synthetic_facts)
    assert result.validated_fraction == 1.0


def test_faithfulness_checker_alias_is_mockchecker():
    from narrate.checker import FaithfulnessChecker

    assert FaithfulnessChecker is MockChecker
    assert FaithfulnessChecker().guarantee_tag == "Empirical"


# ---------------------------------------------------------------------------
# doc_refs gate — an uncited paragraph is not validated
# ---------------------------------------------------------------------------


def test_uncited_paragraph_is_dropped(synthetic_facts):
    prose = "# Reference: demo.unit\n\n`twice` doubles the input value x.\n"
    result = MockChecker().check(prose, synthetic_facts)
    # the sentence is grounded but has no doc_refs citation -> not validated
    assert result.validated_fraction < 1.0
    assert all(not v.has_resolvable_docref for v in result.verdicts)


def test_unlicensed_docref_is_not_resolvable(synthetic_facts):
    # a well-formed src: ref that points nowhere in the facts is unresolvable
    prose = (
        "# Reference: demo.unit\n\n"
        "`twice` doubles the input value x.\n"
        "[doc_refs: src:lib/other/elsewhere.myc:99]\n"
    )
    result = MockChecker().check(prose, synthetic_facts)
    assert result.validated_fraction < 1.0
    assert any(v.doc_ref_issues for v in result.verdicts)


def test_segment_separates_citation_from_prose():
    block = "`twice` doubles x.\n[doc_refs: src:lib/demo/unit.myc:3]"
    paras = segment(block)
    assert len(paras) == 1
    assert paras[0].doc_refs == ["src:lib/demo/unit.myc:3"]
    assert paras[0].sentences == ["`twice` doubles x."]
