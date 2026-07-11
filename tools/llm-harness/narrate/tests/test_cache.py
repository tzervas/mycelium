"""Cache idempotence tests — same input => same output bytes + a cache hit."""

from __future__ import annotations

import hashlib

from narrate.generator import CachingGenerator, MockGenerator, cache_key
from narrate.prompts import load_template


def _hash(text: str) -> str:
    return hashlib.blake2b(text.encode(), digest_size=16).hexdigest()


def test_mock_generator_is_deterministic(synthetic_facts, ref_template):
    gen = MockGenerator()
    a = gen.generate(synthetic_facts, ref_template, "", [])
    b = gen.generate(synthetic_facts, ref_template, "", [])
    assert a == b  # pure function of inputs
    assert _hash(a) == _hash(b)


def test_cache_key_is_stable(synthetic_facts, ref_template):
    k1 = cache_key(synthetic_facts, ref_template, "mock-narrate-v1", 42)
    k2 = cache_key(synthetic_facts, ref_template, "mock-narrate-v1", 42)
    assert k1 == k2


def test_cache_key_varies_with_template_and_seed(synthetic_facts):
    ref = load_template("ref-manual-entry")
    book = load_template("book-chapter")
    base = cache_key(synthetic_facts, ref, "mock-narrate-v1", 42)
    assert cache_key(synthetic_facts, book, "mock-narrate-v1", 42) != base
    assert cache_key(synthetic_facts, ref, "mock-narrate-v1", 7) != base
    assert cache_key(synthetic_facts, ref, "other-model", 42) != base


def test_caching_generator_hit_returns_identical_bytes(
    synthetic_facts, ref_template, tmp_path
):
    gen = CachingGenerator(base=MockGenerator(), cache_dir=tmp_path / "cache")

    first = gen.generate(synthetic_facts, ref_template, "", [])
    assert gen.last_was_cache_hit is False  # miss: computed + stored

    second = gen.generate(synthetic_facts, ref_template, "", [])
    assert gen.last_was_cache_hit is True  # hit: served from disk
    assert first == second
    assert _hash(first) == _hash(second)


def test_cache_miss_on_changed_template(synthetic_facts, tmp_path):
    gen = CachingGenerator(base=MockGenerator(), cache_dir=tmp_path / "cache")
    gen.generate(synthetic_facts, load_template("ref-manual-entry"), "", [])
    # a different template is a different key -> a fresh miss, not a stale hit
    gen.generate(synthetic_facts, load_template("book-chapter"), "", [])
    assert gen.last_was_cache_hit is False


def test_feedback_round_bypasses_cache(synthetic_facts, ref_template, tmp_path):
    gen = CachingGenerator(base=MockGenerator(), cache_dir=tmp_path / "cache")
    gen.generate(synthetic_facts, ref_template, "", [])  # prime cache
    # a correction round carries feedback -> must not be served from the cache
    gen.generate(
        synthetic_facts,
        ref_template,
        "",
        [{"text": "whatever.", "validated": False}],
    )
    assert gen.last_was_cache_hit is False
