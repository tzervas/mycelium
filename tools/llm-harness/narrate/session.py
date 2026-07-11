#!/usr/bin/env python3
"""narrate.session — the generate -> check -> regenerate loop (the coauthor pattern).

Wires a :class:`~narrate.generator.Generator` to a
:class:`~narrate.checker.Checker` the way ``coauthor.py`` wires its Generator to
the LSP checker: generate prose, validate it against the facts, and if any
sentence is unvalidated, regenerate with feedback up to ``max_rounds``.  The
committed output is ALWAYS only the validated sentences, tagged honestly.

Honesty (VR-5 / G2):
  * The output guarantee tag is the generator's (``Declared`` mock / ``Empirical``
    real) — NEVER ``Proven`` / ``Exact``.  We assert this on every round, exactly
    like ``coauthor.assert_model_tag``.
  * Every round records its ``validated_fraction`` and its dropped-sentence list;
    nothing is dropped silently.
  * A generator that returns empty ⇒ SKIP (never a false PASS).

Pure Python standard library only.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any

from narrate.checker import Checker, FaithfulnessResult
from narrate.facts import FactSet
from narrate.generator import Generator
from narrate.prompts import PromptTemplate

# Guarantee lattice (VR-5) — mirrors coauthor.py; a model claim may be at most
# Empirical.  Never Proven / Exact without a checked basis.
LATTICE_ORDERED = ("Exact", "Proven", "Empirical", "Declared")
MODEL_ALLOWED_TAGS = frozenset({"Empirical", "Declared"})

# Terminal status codes (never-silent — every outcome is explicit, G2)
STATUS_VALIDATED = "VALIDATED"  # validated_fraction == 1.0 on the first round
STATUS_PARTIAL = "PARTIAL_VALIDATED"  # became fully validated after correction
STATUS_PARTIAL_DROPPED = "PARTIAL_DROPPED"  # some sentences dropped after max rounds
STATUS_EMPTY = "EMPTY"  # generator produced no committable prose
STATUS_SKIP = "SKIP"  # generator unavailable


def assert_model_tag(tag: str, claim_id: str) -> None:
    """Raise ValueError if ``tag`` is stronger than a model claim may carry."""
    if tag not in LATTICE_ORDERED:
        raise ValueError(f"Unknown guarantee tag {tag!r} on claim {claim_id!r}")
    if tag not in MODEL_ALLOWED_TAGS:
        raise ValueError(
            f"[VR-5 VIOLATION] claim {claim_id!r} carries tag {tag!r}, forbidden "
            f"for model-derived narration. Allowed: {sorted(MODEL_ALLOWED_TAGS)}."
        )


@dataclass
class NarrateRound:
    """One generate -> check attempt."""

    round_number: int
    prose: str
    result: FaithfulnessResult
    is_correction: bool

    def to_dict(self) -> dict[str, Any]:
        return {
            "round_number": self.round_number,
            "is_correction": self.is_correction,
            "validated_fraction": round(self.result.validated_fraction, 6),
            "total_sentences": self.result.total_sentences,
            "validated_sentences": self.result.validated_sentences,
            "dropped": [v.to_dict() for v in self.result.dropped],
        }


@dataclass
class NarrateRun:
    """The full result of narrating one unit for one target — dual-projectable."""

    unit: str
    target: str
    model_id: str
    guarantee_tag: str
    seed: int
    status: str
    rounds: list[NarrateRound]
    committed_prose: str
    source_index: str
    fact_ids: list[str] = field(default_factory=list)
    fact_doc_refs: list[str] = field(default_factory=list)
    message: str = ""

    @property
    def final(self) -> NarrateRound | None:
        return self.rounds[-1] if self.rounds else None

    @property
    def validated_fraction(self) -> float:
        f = self.final
        return f.result.validated_fraction if f else 0.0

    def to_dict(self) -> dict[str, Any]:
        return {
            "unit": self.unit,
            "target": self.target,
            "model_id": self.model_id,
            "guarantee_tag": self.guarantee_tag,
            "seed": self.seed,
            "status": self.status,
            "validated_fraction": round(self.validated_fraction, 6),
            "message": self.message,
            "source_index": self.source_index,
            "fact_ids": self.fact_ids,
            "fact_doc_refs": self.fact_doc_refs,
            "rounds": [r.to_dict() for r in self.rounds],
        }


def narrate_unit(
    facts: FactSet,
    template: PromptTemplate,
    generator: Generator,
    checker: Checker,
    *,
    prior_summary: str = "",
    max_rounds: int = 3,
    seed: int = 42,
) -> NarrateRun:
    """Run the generate -> check -> regenerate loop for ONE unit + target.

    Returns a :class:`NarrateRun` whose ``committed_prose`` is exactly the
    validated sentences of the best round.  ``validated_fraction`` is always
    reported; dropped sentences are always recorded (G2).
    """
    tag = generator.guarantee_tag
    # VR-5: a model's narration may never be Proven/Exact.
    assert_model_tag(tag, f"{facts.unit}:{template.target}")

    rounds: list[NarrateRound] = []
    feedback: list[dict[str, Any]] = []
    fact_ids = [f.id for f in facts.facts]
    fact_doc_refs = sorted(facts.doc_refs())

    for attempt in range(1, max_rounds + 1):
        prose = generator.generate(facts, template, prior_summary, feedback)
        if not prose.strip():
            # never-silent: empty output is SKIP, never a clean PASS
            return NarrateRun(
                unit=facts.unit,
                target=template.target,
                model_id=generator.model_id,
                guarantee_tag=tag,
                seed=seed,
                status=STATUS_SKIP,
                rounds=rounds,
                committed_prose="",
                source_index=facts.source_index,
                fact_ids=fact_ids,
                fact_doc_refs=fact_doc_refs,
                message=(
                    "generator produced no prose (SKIP — G2: empty is never a "
                    "false PASS)"
                ),
            )

        result = checker.check(prose, facts)
        rounds.append(
            NarrateRound(
                round_number=attempt,
                prose=prose,
                result=result,
                is_correction=attempt > 1,
            )
        )

        if result.validated_fraction >= 1.0:
            status = STATUS_VALIDATED if attempt == 1 else STATUS_PARTIAL
            msg = (
                "all sentences validated on first round"
                if attempt == 1
                else f"fully validated after {attempt} rounds (self-corrected)"
            )
            return NarrateRun(
                unit=facts.unit,
                target=template.target,
                model_id=generator.model_id,
                guarantee_tag=tag,
                seed=seed,
                status=status,
                rounds=rounds,
                committed_prose=result.validated_prose(),
                source_index=facts.source_index,
                fact_ids=fact_ids,
                fact_doc_refs=fact_doc_refs,
                message=msg,
            )

        # some sentences dropped — feed them back and (maybe) regenerate
        feedback = [v.to_dict() for v in result.dropped]

    # Exhausted rounds with residual drops — commit what validated (never trash).
    final = rounds[-1].result
    committed = final.validated_prose()
    status = STATUS_PARTIAL_DROPPED if final.validated_sentences else STATUS_EMPTY
    dropped_n = len(final.dropped)
    msg = (
        f"{final.validated_sentences}/{final.total_sentences} sentences validated "
        f"after {max_rounds} rounds; {dropped_n} dropped (never silent — see "
        f"dropped list)."
    )
    return NarrateRun(
        unit=facts.unit,
        target=template.target,
        model_id=generator.model_id,
        guarantee_tag=tag,
        seed=seed,
        status=status,
        rounds=rounds,
        committed_prose=committed,
        source_index=facts.source_index,
        fact_ids=fact_ids,
        fact_doc_refs=fact_doc_refs,
        message=msg,
    )
