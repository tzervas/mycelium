#!/usr/bin/env python3
"""narrate.checker — the FaithfulnessChecker (the anti-hallucination core).

This is the crux.  It is the prose-vs-facts analogue of the transpiler's
``checked_fraction`` oracle (``crates/mycelium-transpile/src/vet.rs``): *only
validated prose is committed; the rest stays Declared / is dropped; we report a
``validated_fraction`` and NEVER-SILENTLY record what was dropped and why (G2).*

For generated prose it validates each sentence against ONLY its supplied facts:

  (a) **doc_refs gate** — every paragraph must carry ≥1 *resolvable* doc_refs
      token (grammar per ``tools/github/doc_refs_check.py``: ``api:`` / ``corpus:``
      / ``src:``); "resolvable" here = grammar-valid AND the token is one this
      unit's fact set licenses (``FactSet.doc_refs()``).  A paragraph with no
      resolvable citation ⇒ all its sentences are unvalidated (dropped).
  (b) **claim-grounding gate** — each declarative sentence's code tokens
      (backticked spans + bare snake_case/camelCase identifiers) must all be in
      the fact-set vocabulary.  Any token absent ⇒ the sentence is UNGROUNDED
      (an injected hallucination) and is dropped.  :class:`MockChecker` is the
      deterministic stand-in for an adversarial-LLM verifier; :class:`Checker`
      is the protocol a real verifier plugs into.
  (c) ``validated_fraction`` = validated sentences / total; only validated
      sentences are emitted into the committed output.

Guarantee (VR-5): the checker's verdict is **Empirical** (measured by the
grounding oracle over the facts), reported as such — never Proven/Exact.

Pure Python standard library only (Termux-portable).
"""

from __future__ import annotations

import re
from dataclasses import dataclass, field
from typing import Protocol

from narrate.facts import FactSet

# ---------------------------------------------------------------------------
# doc_refs grammar (mirrors tools/github/doc_refs_check.py — not imported to
# stay pure-stdlib / PyYAML-free).  We validate the SHAPE here; resolvability is
# checked against the supplied fact set.
# ---------------------------------------------------------------------------

_DOC_REF_TOKEN_RE = re.compile(
    r"(?:api:[A-Za-z0-9_\-]+::[A-Za-z0-9_:]+"
    r"|corpus:[A-Za-z0-9_\-]+(?:#[A-Za-z0-9_\-]+)?"
    r"|src:[^\s\]]+)"
)
_API_RE = re.compile(r"^api:[A-Za-z0-9_\-]+::[A-Za-z0-9_:]+$")
_CORPUS_RE = re.compile(r"^corpus:[A-Za-z0-9_\-]+(?:#[A-Za-z0-9_\-]+)?$")
_SRC_RE = re.compile(r"^src:.+?(?::\d+)?$")


def is_wellformed_doc_ref(token: str) -> bool:
    """True iff ``token`` matches the api:/corpus:/src: doc_refs grammar."""
    return bool(_API_RE.match(token) or _CORPUS_RE.match(token) or _SRC_RE.match(token))


def extract_doc_refs(text: str) -> list[str]:
    """Every doc_refs-shaped token in ``text`` (order-preserving, de-duped)."""
    seen: list[str] = []
    for m in _DOC_REF_TOKEN_RE.findall(text):
        if m not in seen:
            seen.append(m)
    return seen


# ---------------------------------------------------------------------------
# Code-token extraction from a sentence (the grounding candidates)
# ---------------------------------------------------------------------------

_BACKTICK_SPAN_RE = re.compile(r"`([^`]+)`")
_IDENT_RE = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
_SNAKE_RE = re.compile(r"[A-Za-z]+_[A-Za-z0-9_]*")  # has an underscore
_CAMEL_RE = re.compile(r"[a-z][A-Za-z0-9]*[A-Z]")  # interior lower->upper


def _is_code_like_bare(tok: str) -> bool:
    """A bare (un-backticked) token that *looks like* a code identifier."""
    return bool(_SNAKE_RE.search(tok) or _CAMEL_RE.search(tok))


def code_tokens(sentence: str) -> set[str]:
    """The code identifiers a sentence claims, that must be grounded.

    Two sources: (1) every identifier inside a `backtick` span — these are
    explicit code references and are ALWAYS checked; (2) bare tokens that look
    like code (snake_case / camelCase) — an un-backticked hallucinated symbol is
    still caught.  Ordinary English words (no underscore, no interior capital)
    are not code-like and are never flagged.
    """
    tokens: set[str] = set()
    remaining = sentence
    for span in _BACKTICK_SPAN_RE.findall(sentence):
        # every identifier run inside the span (strips generics/punctuation)
        tokens |= set(_IDENT_RE.findall(span))
    # strip backtick spans before scanning bare words (avoid double counting)
    remaining = _BACKTICK_SPAN_RE.sub(" ", remaining)
    for word in _IDENT_RE.findall(remaining):
        if _is_code_like_bare(word):
            tokens |= set(_IDENT_RE.findall(word))
    return tokens


# ---------------------------------------------------------------------------
# Paragraph / sentence segmentation (deterministic)
# ---------------------------------------------------------------------------

_SENTENCE_SPLIT_RE = re.compile(r"(?<=[.!?])\s+")
_CITATION_LINE_RE = re.compile(r"^\s*\[doc_refs:.*\]\s*$")


@dataclass
class Paragraph:
    index: int
    sentences: list[str]
    doc_refs: list[str]  # tokens found in the paragraph's citation line(s)
    raw: str


def segment(prose: str) -> list[Paragraph]:
    """Split committed prose into paragraphs, each with sentences + doc_refs.

    A paragraph is a blank-line-delimited block.  Within it, any line that is a
    ``[doc_refs: ...]`` citation contributes doc_refs tokens; the remaining lines
    are joined and split into sentences.  Markdown headings (``# ...``) are not
    prose and are skipped for grounding but preserved as structural paragraphs
    with no sentences.
    """
    paragraphs: list[Paragraph] = []
    blocks = re.split(r"\n\s*\n", prose.strip())
    for i, block in enumerate(blocks):
        lines = block.splitlines()
        prose_lines: list[str] = []
        refs: list[str] = []
        for line in lines:
            if _CITATION_LINE_RE.match(line) or line.strip().startswith("[doc_refs:"):
                refs.extend(extract_doc_refs(line))
                continue
            prose_lines.append(line)
        text = " ".join(pl.strip() for pl in prose_lines if pl.strip())
        # headings are structural, not prose sentences
        if text.startswith("#"):
            sentences: list[str] = []
        elif text:
            sentences = [s.strip() for s in _SENTENCE_SPLIT_RE.split(text) if s.strip()]
        else:
            sentences = []
        paragraphs.append(
            Paragraph(index=i, sentences=sentences, doc_refs=refs, raw=block)
        )
    return paragraphs


# ---------------------------------------------------------------------------
# Verdict records
# ---------------------------------------------------------------------------


@dataclass
class SentenceVerdict:
    paragraph_index: int
    sentence_index: int
    text: str
    grounded: bool
    has_resolvable_docref: bool
    unsupported_tokens: list[str]
    doc_ref_issues: list[str]
    reason: str

    @property
    def validated(self) -> bool:
        return self.grounded and self.has_resolvable_docref

    def to_dict(self) -> dict[str, object]:
        return {
            "paragraph_index": self.paragraph_index,
            "sentence_index": self.sentence_index,
            "text": self.text,
            "grounded": self.grounded,
            "has_resolvable_docref": self.has_resolvable_docref,
            "unsupported_tokens": self.unsupported_tokens,
            "doc_ref_issues": self.doc_ref_issues,
            "validated": self.validated,
            "reason": self.reason,
        }


@dataclass
class FaithfulnessResult:
    """The oracle verdict for one prose emission (mirrors VetReport)."""

    verdicts: list[SentenceVerdict]
    guarantee_tag: str = "Empirical"  # measured by the grounding oracle
    paragraphs: list[Paragraph] = field(default_factory=list)

    @property
    def total_sentences(self) -> int:
        return len(self.verdicts)

    @property
    def validated_sentences(self) -> int:
        return sum(1 for v in self.verdicts if v.validated)

    @property
    def validated_fraction(self) -> float:
        """Validated sentences / total.  1.0 when there are no sentences."""
        if not self.verdicts:
            return 1.0
        return self.validated_sentences / len(self.verdicts)

    @property
    def dropped(self) -> list[SentenceVerdict]:
        """The never-silent record of what was dropped and why (G2)."""
        return [v for v in self.verdicts if not v.validated]

    def validated_prose(self) -> str:
        """Reassemble ONLY validated sentences, preserving headings + citations.

        A paragraph is emitted iff it has ≥1 validated sentence (or is a
        structural heading with a citation-free heading line).  Dropped sentences
        never reach the committed output.
        """
        by_para: dict[int, list[SentenceVerdict]] = {}
        for v in self.verdicts:
            if v.validated:
                by_para.setdefault(v.paragraph_index, []).append(v)
        out: list[str] = []
        for p in self.paragraphs:
            # structural heading block (no sentences): keep verbatim
            if not p.sentences and p.raw.strip().startswith("#"):
                out.append(p.raw.strip())
                continue
            keep = by_para.get(p.index, [])
            if not keep:
                continue
            keep.sort(key=lambda v: v.sentence_index)
            body = " ".join(v.text for v in keep)
            if p.doc_refs:
                cite = "[doc_refs: " + " ".join(p.doc_refs) + "]"
                out.append(f"{body}\n{cite}")
            else:
                out.append(body)
        return "\n\n".join(out) + "\n"

    def to_dict(self) -> dict[str, object]:
        return {
            "guarantee_tag": self.guarantee_tag,
            "total_sentences": self.total_sentences,
            "validated_sentences": self.validated_sentences,
            "validated_fraction": round(self.validated_fraction, 6),
            "dropped_count": len(self.dropped),
            "verdicts": [v.to_dict() for v in self.verdicts],
        }


# ---------------------------------------------------------------------------
# Checker protocol + the deterministic MockChecker
# ---------------------------------------------------------------------------


class Checker(Protocol):
    """A prose faithfulness verifier.

    A real adversarial-LLM checker implements this same shape: given prose and
    the fact set it was meant to be grounded in, return a per-sentence verdict.
    Whatever the backend, the loop only ever COMMITS validated sentences and
    reports ``validated_fraction`` (the vet.rs discipline).
    """

    guarantee_tag: str

    def check(self, prose: str, facts: FactSet) -> FaithfulnessResult: ...


class MockChecker:
    """Deterministic faithfulness oracle — the offline / CI default.

    Stands in for an adversarial-LLM verifier with a transparent heuristic: a
    sentence is grounded iff every code token it uses is in the fact-set
    vocabulary; a paragraph is cited iff it carries a well-formed doc_refs token
    that the fact set licenses.  Deterministic ⇒ idempotent and testable.

    Guarantee tag: Empirical (a *measured* verdict over the facts), never Proven.
    """

    guarantee_tag = "Empirical"

    def check(self, prose: str, facts: FactSet) -> FaithfulnessResult:
        vocab = facts.vocabulary()
        licensed_refs = facts.doc_refs()
        paragraphs = segment(prose)
        verdicts: list[SentenceVerdict] = []

        for para in paragraphs:
            # --- (a) doc_refs gate: is the paragraph cited + resolvable? ---
            ref_issues: list[str] = []
            resolvable = False
            if not para.doc_refs:
                ref_issues.append("paragraph carries no doc_refs citation")
            for tok in para.doc_refs:
                if not is_wellformed_doc_ref(tok):
                    ref_issues.append(f"malformed doc_ref {tok!r}")
                elif tok in licensed_refs:
                    resolvable = True
                elif tok.startswith("src:"):
                    # a src: ref not among the supplied facts is unresolvable
                    ref_issues.append(
                        f"doc_ref {tok!r} not licensed by this unit's facts"
                    )
                else:
                    ref_issues.append(
                        f"doc_ref {tok!r} not resolvable against supplied facts"
                    )

            # --- (b) grounding gate: each sentence's code tokens supported? ---
            for si, sentence in enumerate(para.sentences):
                claimed = code_tokens(sentence)
                unsupported = sorted(t for t in claimed if t not in vocab)
                grounded = not unsupported
                reasons: list[str] = []
                if not grounded:
                    reasons.append(
                        "ungrounded token(s) not in facts: "
                        + ", ".join(repr(t) for t in unsupported)
                    )
                if not resolvable:
                    reasons.append("no resolvable doc_refs citation on paragraph")
                verdicts.append(
                    SentenceVerdict(
                        paragraph_index=para.index,
                        sentence_index=si,
                        text=sentence,
                        grounded=grounded,
                        has_resolvable_docref=resolvable,
                        unsupported_tokens=unsupported,
                        doc_ref_issues=ref_issues,
                        reason="; ".join(reasons) if reasons else "validated",
                    )
                )

        return FaithfulnessResult(
            verdicts=verdicts,
            guarantee_tag=self.guarantee_tag,
            paragraphs=paragraphs,
        )
