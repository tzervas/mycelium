"""Loader + Fact-model tests (data-driven over the real + synthetic fact sets)."""

from __future__ import annotations

import pytest

from narrate.facts import Fact, FactSet, list_units, load_facts

# ---------------------------------------------------------------------------
# Loader over the real committed lib-index
# ---------------------------------------------------------------------------


def test_load_real_unit_has_expected_members(result_facts):
    ids = {f.id for f in result_facts.facts}
    # a representative subset the index is known to carry
    for expected in (
        "std.result",
        "std.result::Result",
        "std.result::map",
        "std.result::and_then",
        "std.result::fold",
    ):
        assert expected in ids


def test_header_and_members_partition(result_facts):
    hdr = result_facts.header
    assert hdr is not None
    assert hdr.kind == "nodule"
    assert hdr not in result_facts.members
    assert len(result_facts.members) == len(result_facts.facts) - 1


def test_missing_unit_raises_keyerror(lib_index):
    with pytest.raises(KeyError):
        load_facts(lib_index, "std.does-not-exist")


def test_list_units_includes_known(lib_index):
    units = list_units(lib_index)
    assert "std.result" in units
    assert units == sorted(units)  # deterministic order


# ---------------------------------------------------------------------------
# Undocumented facts are explicit, never invented (G2)
# ---------------------------------------------------------------------------


def test_undocumented_facts_are_flagged(synthetic_facts):
    undoc = synthetic_facts.undocumented()
    assert any(f.id == "demo.unit::Shadow" for f in undoc)
    shadow = next(f for f in synthetic_facts.facts if f.name == "Shadow")
    assert shadow.documented is False
    # verbatim signature is still present (groundable), summary is None (honest)
    assert shadow.summary is None
    assert "Shadow" in shadow.signature


def test_fact_text_falls_back_when_undocumented():
    f = Fact(
        id="x::y",
        kind="fn",
        unit="x",
        source_path="lib/x.myc",
        line=2,
        signature="",
        summary=None,
        guarantee_tag="Declared",
        documented=False,
    )
    assert f.text == "(undocumented)"
    assert f.doc_ref == "src:lib/x.myc:2"


# ---------------------------------------------------------------------------
# Vocabulary (the grounding basis) + content hash (cache-key input)
# ---------------------------------------------------------------------------


def test_vocabulary_licenses_signature_and_summary_tokens(synthetic_facts):
    vocab = synthetic_facts.vocabulary()
    # from signatures / names
    for tok in ("twice", "Nat", "Shadow", "Dim", "Bright", "demo", "unit"):
        assert tok in vocab
    # from the summary prose
    assert "double" in vocab  # appears in "double the input value x"


def test_content_hash_is_stable_and_order_independent():
    a = Fact("u::a", "fn", "u", "lib/u.myc", 2, "fn a()", "s", "Declared", True)
    b = Fact("u::b", "fn", "u", "lib/u.myc", 3, "fn b()", "t", "Declared", True)
    fs1 = FactSet("u", [a, b], "<x>")
    fs2 = FactSet("u", [b, a], "<x>")  # different input order
    assert fs1.content_hash() == fs2.content_hash()  # sorted internally
    assert fs1.doc_refs() == {"src:lib/u.myc:2", "src:lib/u.myc:3"}


def test_content_hash_changes_with_content():
    a = Fact("u::a", "fn", "u", "lib/u.myc", 2, "fn a()", "s", "Declared", True)
    a2 = Fact(
        "u::a", "fn", "u", "lib/u.myc", 2, "fn a()", "DIFFERENT", "Declared", True
    )
    assert (
        FactSet("u", [a], "<x>").content_hash()
        != FactSet("u", [a2], "<x>").content_hash()
    )
