#!/usr/bin/env python3
"""narrate.generator — the prose generators + the idempotent cache.

Mirrors ``coauthor.py``'s Generator posture:

  * :class:`Generator` — the protocol a real-LLM backend plugs into later.
  * :class:`MockGenerator` — the deterministic, offline / CI default.  It
    template-fills the EMIT SKELETON purely from the supplied facts, so its
    output is grounded BY CONSTRUCTION (every code token comes from a fact).
    Guarantee tag ``Declared`` (asserted mock output; VR-5).
  * :class:`CachingGenerator` — the idempotence layer.  Cache key = a blake2b
    content hash over (facts + full template text + model-id + seed).  A re-run
    with identical inputs returns byte-identical cached prose (idempotent); any
    input change ⇒ a new key ⇒ regeneration.

A real backend (e.g. llama.cpp / an API model) would tag its output ``Empirical``
and SKIP-when-absent, exactly as ``coauthor.LlmGenerator`` does — a documented
stub, :class:`LlmNarrator`, records that contract without being exercised offline.

Pure Python standard library only (Termux-portable).  Fixed ``seed=42``.
"""

from __future__ import annotations

import hashlib
from dataclasses import dataclass
from pathlib import Path
from typing import Protocol

from narrate.facts import Fact, FactSet
from narrate.prompts import PromptTemplate

DEFAULT_SEED = 42


class Generator(Protocol):
    """A narration generator.

    ``model_id`` / ``guarantee_tag`` identify the backend and its honesty tag
    (``Declared`` for a mock, ``Empirical`` for a real model — never stronger,
    VR-5).  ``generate`` returns prose grounded in ``facts``; ``feedback`` carries
    the prior round's dropped-sentence records so the backend can self-correct.
    """

    model_id: str
    guarantee_tag: str

    def generate(
        self,
        facts: FactSet,
        template: PromptTemplate,
        prior_summary: str,
        feedback: list[dict[str, object]],
    ) -> str: ...


# ---------------------------------------------------------------------------
# Fact -> paragraph renderers (deterministic, target-aware)
# ---------------------------------------------------------------------------


def _first_sentence(text: str) -> str:
    """The first sentence of a summary (deterministic; keeps code tokens)."""
    for sep in (". ", "? ", "! "):
        if sep in text:
            return text.split(sep, 1)[0].strip().rstrip(".") + "."
    return text.strip().rstrip(".") + "."


def _cite(fact: Fact) -> str:
    return f"[doc_refs: {fact.doc_ref}]"


def _member_paragraph(fact: Fact, target: str) -> str:
    """Render one member fact as a grounded, cited paragraph.

    Every code identifier is drawn verbatim from the fact (name / signature /
    summary), so the paragraph is grounded by construction.  Undocumented facts
    are narrated AS undocumented — never invented (G2).
    """
    name = fact.name
    sig = fact.signature.strip()
    lead = {
        "ref-manual-entry": f"`{name}` is a {fact.kind}.",
        "book-chapter": f"Next is `{name}`, a {fact.kind}.",
        "learning-lesson": f"Study `{name}` (a {fact.kind}).",
    }.get(target, f"`{name}` is a {fact.kind}.")

    sentences = [lead]
    if fact.documented and fact.summary:
        sentences.append(_first_sentence(fact.summary))
    else:
        sentences.append(f"This {fact.kind} is recorded without a prose summary.")
    if sig:
        sentences.append(f"Its signature is `{sig}`.")
    body = " ".join(sentences)
    return f"{body}\n{_cite(fact)}"


def _intro_paragraph(facts: FactSet, target: str) -> str | None:
    """The unit-level intro paragraph from the header fact (if any)."""
    hdr = facts.header
    if hdr is None:
        return None
    opener = {
        "ref-manual-entry": f"The unit `{facts.unit}` is documented below.",
        "book-chapter": f"This chapter introduces `{facts.unit}`.",
        "learning-lesson": f"This lesson covers `{facts.unit}`.",
    }.get(target, f"The unit `{facts.unit}`.")
    sentences = [opener]
    if hdr.documented and hdr.summary:
        sentences.append(_first_sentence(hdr.summary))
    body = " ".join(sentences)
    return f"{body}\n{_cite(hdr)}"


def render_facts_block(facts: FactSet, target: str) -> str:
    """The ``{{FACTS}}`` substitution: intro + one paragraph per member fact."""
    blocks: list[str] = []
    intro = _intro_paragraph(facts, target)
    if intro:
        blocks.append(intro)
    for fact in facts.members:
        blocks.append(_member_paragraph(fact, target))
    return "\n\n".join(blocks)


# ---------------------------------------------------------------------------
# MockGenerator — deterministic, grounded-by-construction
# ---------------------------------------------------------------------------


class MockGenerator:
    """Deterministic narration backend for offline / CI use.

    Fills the template's EMIT SKELETON from the facts.  When given ``feedback``
    (dropped-sentence records from a prior round), it self-corrects by OMITTING
    the flagged sentences on regeneration — the mock analogue of an LLM fixing an
    ungrounded claim.  Output tag: ``Declared`` (VR-5 — asserted, not empirical).
    """

    model_id = "mock-narrate-v1"
    guarantee_tag = "Declared"

    def __init__(self, inject_hallucination: str | None = None) -> None:
        # Test hook: when set, ONE ungrounded sentence citing this bogus token is
        # injected into the first paragraph (the negative control).  It is never
        # set in production paths.
        self._inject = inject_hallucination

    def generate(
        self,
        facts: FactSet,
        template: PromptTemplate,
        prior_summary: str,
        feedback: list[dict[str, object]],
    ) -> str:
        facts_block = render_facts_block(facts, template.target)
        prose = template.render_skeleton(facts.unit, prior_summary, facts_block)

        if self._inject:
            prose = self._apply_injection(prose, facts)

        # Self-correction: drop any sentence flagged in the previous round.
        flagged = {
            str(rec.get("text", "")).strip()
            for rec in (feedback or [])
            if not rec.get("validated", False)
        }
        if flagged:
            prose = _drop_sentences(prose, flagged)
        return prose

    def _apply_injection(self, prose: str, facts: FactSet) -> str:
        """Insert one ungrounded, cited sentence — the negative control.

        The injected sentence cites a real fact (so the doc_refs gate passes) but
        asserts a bogus backticked identifier absent from every fact, so the
        grounding gate MUST catch and drop it.
        """
        bogus = self._inject or "frobnicate"
        hdr = facts.header or (facts.facts[0] if facts.facts else None)
        cite = _cite(hdr) if hdr else "[doc_refs: src:unknown:0]"
        bad = (
            f"The `{bogus}` operation silently rewrites entropy across the colony."
            f"\n{cite}"
        )
        # place it as its own paragraph after the first block
        blocks = prose.split("\n\n")
        insert_at = 2 if len(blocks) > 2 else len(blocks)
        blocks.insert(insert_at, bad)
        return "\n\n".join(blocks)


class LlmNarrator:
    """Documented real-LLM backend stub — SKIP when no model is available (G2).

    A real backend would render ``template.render_instructions(...)`` as the
    prompt, shell to a model (llama.cpp / an API), and tag output ``Empirical``.
    It is never exercised in the offline demo/tests; kept to pin the contract so
    a real backend drops in without touching the loop or checker.
    """

    model_id = "llm-narrate (unconfigured)"
    guarantee_tag = "Empirical"

    def __init__(self, backend: object | None = None) -> None:
        self._backend = backend

    def available(self) -> bool:
        return self._backend is not None

    def generate(
        self,
        facts: FactSet,
        template: PromptTemplate,
        prior_summary: str,
        feedback: list[dict[str, object]],
    ) -> str:
        if not self.available():
            # never-silent: an unavailable backend returns empty; the loop treats
            # empty as SKIP, never a false PASS (G2).
            return ""
        raise NotImplementedError(
            "LlmNarrator is a documented stub; wire a real backend before use."
        )


def _drop_sentences(prose: str, flagged: set[str]) -> str:
    """Remove flagged sentences from prose (mock self-correction)."""
    if not flagged:
        return prose
    out_blocks: list[str] = []
    for block in prose.split("\n\n"):
        lines = block.splitlines()
        cite_lines = [ln for ln in lines if ln.strip().startswith("[doc_refs:")]
        prose_text = " ".join(
            ln.strip() for ln in lines if not ln.strip().startswith("[doc_refs:")
        )
        import re as _re

        kept = [
            s.strip()
            for s in _re.split(r"(?<=[.!?])\s+", prose_text)
            if s.strip() and s.strip() not in flagged
        ]
        if not kept and not prose_text.startswith("#"):
            # whole paragraph was flagged away — drop it entirely
            continue
        if prose_text.startswith("#"):
            out_blocks.append(block)
            continue
        rebuilt = " ".join(kept)
        if cite_lines:
            rebuilt = rebuilt + "\n" + "\n".join(cite_lines)
        out_blocks.append(rebuilt)
    return "\n\n".join(out_blocks)


# ---------------------------------------------------------------------------
# Idempotent caching layer
# ---------------------------------------------------------------------------


def cache_key(
    facts: FactSet,
    template: PromptTemplate,
    model_id: str,
    seed: int = DEFAULT_SEED,
) -> str:
    """Deterministic content hash over (facts, template text, model-id, seed).

    This is the idempotence contract: identical inputs ⇒ identical key ⇒ the same
    cached prose bytes; any change to the facts, the template, the model, or the
    seed yields a new key (a correct, different output).
    """
    h = hashlib.blake2b(digest_size=20)
    h.update(b"narrate-cache-v1\0")
    h.update(facts.canonical_bytes())
    h.update(b"\0")
    h.update(template.raw.encode("utf-8"))
    h.update(b"\0")
    h.update(model_id.encode("utf-8"))
    h.update(b"\0")
    h.update(str(seed).encode("utf-8"))
    return h.hexdigest()


@dataclass
class CachingGenerator:
    """Wraps a :class:`Generator` with an on-disk idempotent cache.

    A cache HIT returns byte-identical prose without re-invoking the base
    generator; a MISS computes, stores, and returns.  ``last_was_cache_hit``
    exposes which happened (for tests / reporting).  Feedback-driven regeneration
    bypasses the cache (a correction is a distinct request) unless ``use_cache``
    is set for that call.
    """

    base: Generator
    cache_dir: Path
    last_was_cache_hit: bool = False
    seed: int = DEFAULT_SEED

    def __post_init__(self) -> None:
        self.cache_dir = Path(self.cache_dir)
        self.cache_dir.mkdir(parents=True, exist_ok=True)

    @property
    def model_id(self) -> str:
        return self.base.model_id

    @property
    def guarantee_tag(self) -> str:
        return self.base.guarantee_tag

    def _path_for(self, key: str) -> Path:
        return self.cache_dir / f"{key}.txt"

    def generate(
        self,
        facts: FactSet,
        template: PromptTemplate,
        prior_summary: str,
        feedback: list[dict[str, object]],
    ) -> str:
        # A correction round (non-empty feedback) is a distinct request: do not
        # serve or store it under the base cache key (which ignores feedback).
        if feedback:
            self.last_was_cache_hit = False
            return self.base.generate(facts, template, prior_summary, feedback)

        key = cache_key(facts, template, self.model_id, self.seed)
        path = self._path_for(key)
        if path.is_file():
            self.last_was_cache_hit = True
            return path.read_text(encoding="utf-8")

        prose = self.base.generate(facts, template, prior_summary, feedback)
        path.write_text(prose, encoding="utf-8")
        self.last_was_cache_hit = False
        return prose
