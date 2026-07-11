#!/usr/bin/env python3
"""narrate.prompts — the parameterized, idempotent prompt family.

Three narration targets, each a template FILE under ``narrate/prompts/``:

  * ``book-chapter``      — a language-book chapter section
  * ``ref-manual-entry``  — a reference-manual entry
  * ``learning-lesson``   — a learning-corpus lesson

Each template has two parts separated by ``=== EMIT SKELETON ===``:

  * the INSTRUCTIONS block — the prose a *real* LLM narrator reads (it ignores the
    skeleton);
  * the EMIT SKELETON — a deterministic fill template a real LLM ignores and the
    :class:`~narrate.generator.MockGenerator` fills, substituting ``{{UNIT}}`` /
    ``{{PRIOR}}`` / ``{{FACTS}}``.

Idempotence: a template is a pure function of its file bytes; the generator's
cache key includes the *full template text*, so a template edit invalidates the
cache (a different, correct output) while a re-run with the same template returns
byte-identical prose.

Pure Python standard library only.
"""

from __future__ import annotations

import re
from dataclasses import dataclass
from pathlib import Path

TARGETS = ("book-chapter", "ref-manual-entry", "learning-lesson")

_SKELETON_MARKER = "=== EMIT SKELETON ==="
_FRONT_RE = re.compile(r"^---\n(.*?)\n---\n", re.DOTALL)


def prompts_dir() -> Path:
    """The directory holding the committed ``*.md.tmpl`` templates."""
    return Path(__file__).resolve().parent / "prompts"


@dataclass(frozen=True)
class PromptTemplate:
    """A loaded, parameterized narration template."""

    target: str
    title: str
    instructions: str  # the real-LLM-facing prompt body
    skeleton: str  # the deterministic Mock fill template
    raw: str  # the full verbatim file text (cache-key input)

    def render_instructions(self, unit: str, prior_summary: str) -> str:
        """The real-LLM prompt with ``{{UNIT}}`` / ``{{PRIOR}}`` substituted."""
        return self.instructions.replace("{{UNIT}}", unit).replace(
            "{{PRIOR}}", prior_summary or "(none)"
        )

    def render_skeleton(self, unit: str, prior_summary: str, facts_block: str) -> str:
        """Fill the EMIT SKELETON — the deterministic Mock output substrate."""
        return (
            self.skeleton.replace("{{UNIT}}", unit)
            .replace("{{PRIOR}}", prior_summary or "")
            .replace("{{FACTS}}", facts_block)
        )


def _parse_front_matter(text: str) -> tuple[dict[str, str], str]:
    m = _FRONT_RE.match(text)
    if not m:
        return {}, text
    front: dict[str, str] = {}
    for line in m.group(1).splitlines():
        if ":" in line:
            k, _, v = line.partition(":")
            front[k.strip()] = v.strip()
    return front, text[m.end() :]


def load_template(target: str) -> PromptTemplate:
    """Load the template FILE for ``target`` (raises on unknown target/file)."""
    if target not in TARGETS:
        raise ValueError(
            f"unknown narration target {target!r}; expected one of {TARGETS}"
        )
    path = prompts_dir() / f"{target}.md.tmpl"
    raw = path.read_text(encoding="utf-8")
    front, body = _parse_front_matter(raw)
    if _SKELETON_MARKER not in body:
        raise ValueError(f"{path}: missing {_SKELETON_MARKER!r} marker")
    instructions, _, skeleton = body.partition(_SKELETON_MARKER)
    return PromptTemplate(
        target=front.get("target", target),
        title=front.get("title", target),
        instructions=instructions.strip(),
        skeleton=skeleton.strip(),
        raw=raw,
    )
