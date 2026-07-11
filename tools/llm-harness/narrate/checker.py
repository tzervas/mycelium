#!/usr/bin/env python3
"""narrate.checker — the FaithfulnessChecker (the anti-hallucination core).

This is the crux.  It is the prose-vs-facts analogue of the transpiler's
``checked_fraction`` oracle (``crates/mycelium-transpile/src/vet.rs``): *only
validated prose is committed; the rest stays Declared / is dropped; we report a
``validated_fraction`` and NEVER-SILENTLY record what was dropped and why (G2).*

Scope of the deterministic Mock oracle — stated precisely (VR-5, no black boxes):
the Mock catches **code-token-bearing** claims and **PascalCase type-name**
claims, and buckets **unverifiable free text**.  Concretely, per sentence:

  (a) **doc_refs gate** — every paragraph must carry ≥1 *resolvable* doc_refs
      token (grammar per ``tools/github/doc_refs_check.py``: ``api:`` / ``corpus:``
      / ``src:``); "resolvable" = grammar-valid AND the token is one this unit's
      fact set licenses (``FactSet.doc_refs()``).  Uncited ⇒ its sentences are
      unvalidated (dropped).
  (b) **claim-grounding gate** — every code token a sentence uses must be in the
      fact vocabulary.  Code tokens = identifiers inside `backticks` **plus** bare
      identifiers that look like code: snake_case, camelCase, AND **bare
      PascalCase type names** (Mycelium's own convention — ``Result``, ``Binary``,
      ``Frobnicator``; leading-or-interior capital, minus a small common-word
      stoplist).  Any token absent from the facts ⇒ the sentence is UNGROUNDED (a
      hallucination) and is dropped.
  (c) **free-text gate** — a declarative sentence with ZERO code tokens is *not*
      vacuously passed: it must share ≥1 content word with the fact text its
      paragraph cites (a deterministic lexical-overlap signal).  With no overlap
      it is routed to a distinct ``unverifiable`` bucket that does **not** count
      as grounded (reported, never silently passed) — genuine free-text
      faithfulness needs the real **M-1063** adversarial-LLM verifier, which
      plugs in via the :class:`Checker` protocol.
  (d) ``validated_fraction`` = validated sentences / total; only validated
      sentences are emitted into the committed output.

Honesty about the heuristic (VR-5): the Mock is conservative — it over-flags an
unknown capitalized word as a possible type identifier (a drop-safe direction,
never optimistic), and it cannot judge the *truth* of pure free text (hence the
``unverifiable`` bucket).  Its verdict is **Empirical** (measured over the facts),
never Proven/Exact.  :class:`MockChecker` is the deterministic reference;
:class:`Checker` is the protocol a real adversarial-LLM verifier plugs into.
``FaithfulnessChecker`` is a public alias of :class:`MockChecker`.

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

# Capitalized words that are ordinary English (sentence openers / function words)
# and must NOT be mistaken for PascalCase type names.  Compared case-folded.  The
# gate is intentionally conservative: a capitalized word NOT in this set is
# treated as a possible type identifier and must be grounded (drop-safe — an
# over-flag drops a real sentence, never passes a hallucination).
_COMMON_CAPITALIZED = frozenset(
    {
        "the",
        "this",
        "that",
        "these",
        "those",
        "it",
        "its",
        "a",
        "an",
        "and",
        "or",
        "but",
        "nor",
        "for",
        "so",
        "yet",
        "as",
        "at",
        "by",
        "in",
        "on",
        "of",
        "to",
        "up",
        "off",
        "out",
        "is",
        "are",
        "was",
        "were",
        "be",
        "been",
        "being",
        "has",
        "have",
        "had",
        "do",
        "does",
        "did",
        "no",
        "not",
        "all",
        "any",
        "each",
        "every",
        "both",
        "some",
        "none",
        "one",
        "two",
        "here",
        "there",
        "when",
        "where",
        "why",
        "how",
        "what",
        "who",
        "which",
        "if",
        "then",
        "than",
        "into",
        "over",
        "under",
        "across",
        "via",
        "per",
        "we",
        "you",
        "they",
        "he",
        "she",
        "given",
        "use",
        "used",
        "see",
        "note",
        "next",
        "study",
        "first",
        "second",
        "third",
        "new",
        "only",
        "also",
        "may",
        "can",
        "will",
        "shall",
        "must",
        "consider",
        "recall",
        "suppose",
        "let",
        "assume",
        "however",
        "furthermore",
        "additionally",
        "moreover",
        "therefore",
        "thus",
        "hence",
        "because",
        "since",
        "although",
        "though",
        "while",
        "whereas",
        "unlike",
        "like",
        "together",
        "overall",
        "finally",
        "similarly",
        "conversely",
        "notably",
        "importantly",
        "specifically",
        "generally",
        "typically",
        "effectively",
        "essentially",
        "with",
        "without",
    }
)

# Content-word stoplist for the free-text lexical-overlap signal.
_STOPWORDS = _COMMON_CAPITALIZED | frozenset(
    {
        "value",
        "values",
        "input",
        "output",
        "result",
        "returns",
        "return",
        "function",
        "type",
        "its",
        "their",
        "our",
        "your",
        "his",
        "her",
        "them",
        "from",
        "about",
        "such",
        "this",
        "very",
        "just",
        "more",
        "most",
        "many",
    }
)


def _is_code_like_bare(tok: str) -> bool:
    """A bare (un-backticked) token that *looks like* a code identifier.

    True for snake_case, camelCase (interior lower->upper), tokens with an
    interior capital (``IOError``), and single-hump PascalCase type names that
    are NOT common English words (``Result``, ``Binary``, ``Frobnicator``).
    Ordinary capitalized English (``The``, ``This``, ``Given``) is excluded.
    """
    if "_" in tok:
        return True
    if _CAMEL_RE.search(tok):  # fooBar, mapErr, BinaryTree
        return True
    if tok[:1].isupper() and len(tok) > 1:
        if any(c.isupper() for c in tok[1:]):  # interior capital: IOError, HTTPd
            return True
        return tok.lower() not in _COMMON_CAPITALIZED  # single-hump type name
    return False


def code_tokens(sentence: str) -> set[str]:
    """The code identifiers a sentence claims, that must be grounded.

    Sources: (1) every identifier inside a `backtick` span — ALWAYS checked;
    (2) bare tokens that look like code — snake_case, camelCase, interior-cap,
    and bare PascalCase type names (see :func:`_is_code_like_bare`).  Ordinary
    English words are not code-like and are never flagged here (free text is
    handled by the lexical-overlap gate instead).
    """
    tokens: set[str] = set()
    for span in _BACKTICK_SPAN_RE.findall(sentence):
        tokens |= set(_IDENT_RE.findall(span))
    remaining = _BACKTICK_SPAN_RE.sub(" ", sentence)
    for word in _IDENT_RE.findall(remaining):
        if _is_code_like_bare(word):
            tokens |= set(_IDENT_RE.findall(word))
    return tokens


def content_words(text: str) -> set[str]:
    """Lower-cased content words (identifiers minus stopwords, len>1)."""
    return {
        w.lower()
        for w in _IDENT_RE.findall(text)
        if len(w) > 1 and w.lower() not in _STOPWORDS
    }


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

    A paragraph is a blank-line-delimited block.  Within it, any ``[doc_refs:
    ...]`` citation line contributes doc_refs tokens; the rest is joined and
    split into sentences.  Markdown headings (``# ...``) are structural: kept
    verbatim, no sentences.
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

# Buckets (never-silent classification of every sentence)
BUCKET_VALIDATED = "validated"
BUCKET_HALLUCINATED = "hallucinated"  # code token(s) absent from the facts
BUCKET_UNVERIFIABLE = "unverifiable"  # free text, no lexical overlap with facts
BUCKET_UNCITED = "uncited"  # grounded/overlapping but no resolvable doc_ref


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
    bucket: str = BUCKET_VALIDATED

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
            "bucket": self.bucket,
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

    def bucket_counts(self) -> dict[str, int]:
        """Count of sentences per bucket — reported, never silent."""
        counts: dict[str, int] = {}
        for v in self.verdicts:
            counts[v.bucket] = counts.get(v.bucket, 0) + 1
        return counts

    def validated_prose(self) -> str:
        """Reassemble ONLY validated sentences, preserving headings + citations.

        A paragraph is emitted iff it has ≥1 validated sentence (or is a
        structural heading).  Dropped/unverifiable sentences never reach output.
        """
        by_para: dict[int, list[SentenceVerdict]] = {}
        for v in self.verdicts:
            if v.validated:
                by_para.setdefault(v.paragraph_index, []).append(v)
        out: list[str] = []
        for p in self.paragraphs:
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
            "bucket_counts": self.bucket_counts(),
            "verdicts": [v.to_dict() for v in self.verdicts],
        }


# ---------------------------------------------------------------------------
# Checker protocol + the deterministic MockChecker
# ---------------------------------------------------------------------------


class Checker(Protocol):
    """A prose faithfulness verifier.

    A real adversarial-LLM checker (M-1063) implements this same shape: given
    prose and the fact set it was meant to be grounded in, return a per-sentence
    verdict.  Whatever the backend, the loop only ever COMMITS validated
    sentences and reports ``validated_fraction`` (the vet.rs discipline).
    """

    guarantee_tag: str

    def check(self, prose: str, facts: FactSet) -> FaithfulnessResult: ...


class MockChecker:
    """Deterministic faithfulness oracle — the offline / CI default.

    Stands in for an adversarial-LLM verifier with a transparent heuristic (see
    the module docstring for exact scope): a sentence's code tokens (backticked
    + snake/camel + PascalCase) must all be in the fact vocabulary; a free-text
    sentence must share ≥1 content word with its cited fact text or is bucketed
    ``unverifiable``; and its paragraph must carry a licensed doc_refs token.
    Deterministic ⇒ idempotent and testable.

    Guarantee tag: Empirical (a *measured* verdict over the facts), never Proven.
    """

    guarantee_tag = "Empirical"

    def check(self, prose: str, facts: FactSet) -> FaithfulnessResult:
        vocab = facts.vocabulary()
        licensed_refs = facts.doc_refs()
        # map each licensed doc_ref -> the fact text(s) it cites (overlap basis)
        ref_text: dict[str, list[str]] = {}
        for f in facts.facts:
            ref_text.setdefault(f.doc_ref, []).append(f.text)

        paragraphs = segment(prose)
        verdicts: list[SentenceVerdict] = []

        for para in paragraphs:
            # --- (a) doc_refs gate: is the paragraph cited + resolvable? ---
            ref_issues: list[str] = []
            resolvable = False
            cited_text_words: set[str] = set()
            if not para.doc_refs:
                ref_issues.append("paragraph carries no doc_refs citation")
            for tok in para.doc_refs:
                if not is_wellformed_doc_ref(tok):
                    ref_issues.append(f"malformed doc_ref {tok!r}")
                elif tok in licensed_refs:
                    resolvable = True
                    for txt in ref_text.get(tok, []):
                        cited_text_words |= content_words(txt)
                elif tok.startswith("src:"):
                    ref_issues.append(
                        f"doc_ref {tok!r} not licensed by this unit's facts"
                    )
                else:
                    ref_issues.append(
                        f"doc_ref {tok!r} not resolvable against supplied facts"
                    )

            # --- (b)/(c) per-sentence grounding ---
            for si, sentence in enumerate(para.sentences):
                claimed = code_tokens(sentence)
                unsupported = sorted(t for t in claimed if t not in vocab)
                reasons: list[str] = []

                if unsupported:
                    grounded = False
                    bucket = BUCKET_HALLUCINATED
                    reasons.append(
                        "ungrounded code token(s) not in facts: "
                        + ", ".join(repr(t) for t in unsupported)
                    )
                elif claimed:
                    # has code tokens, all supported -> grounded
                    grounded = True
                    bucket = BUCKET_VALIDATED
                else:
                    # zero code tokens: free text — require lexical overlap with
                    # the cited fact text; otherwise it is unverifiable-by-mock.
                    overlap = content_words(sentence) & cited_text_words
                    if overlap:
                        grounded = True
                        bucket = BUCKET_VALIDATED
                    else:
                        grounded = False
                        bucket = BUCKET_UNVERIFIABLE
                        reasons.append(
                            "free-text claim with no code tokens and no lexical "
                            "overlap with the cited facts — unverifiable by the "
                            "mock oracle (needs the M-1063 adversarial verifier)"
                        )

                if not resolvable:
                    reasons.append("no resolvable doc_refs citation on paragraph")
                    # a grounded-but-uncited sentence is bucketed uncited
                    if bucket == BUCKET_VALIDATED:
                        bucket = BUCKET_UNCITED

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
                        bucket=bucket,
                    )
                )

        return FaithfulnessResult(
            verdicts=verdicts,
            guarantee_tag=self.guarantee_tag,
            paragraphs=paragraphs,
        )


# Public alias: the conceptual name the SKILL.md / module docstring / DN-114 use.
# The deterministic Mock IS the reference FaithfulnessChecker; a real
# adversarial-LLM verifier (M-1063) implements the same :class:`Checker` protocol.
FaithfulnessChecker = MockChecker
