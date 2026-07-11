#!/usr/bin/env python3
"""narrate — a validated, idempotent narrative generator for the language corpus.

Generates the *read* (prose) that accompanies extracted code facts for the
language book, the reference manuals, and a learning corpus — with MANDATORY
faithfulness validation so it never hallucinates or emits trash.

The pipeline mirrors ``coauthor.py`` (Generator -> Checker -> Loop) and the
transpiler vet loop's ``checked_fraction`` honesty (only validated prose is
committed; the rest stays Declared / is dropped; a ``validated_fraction`` is
reported).  All backends default to deterministic Mock implementations so the
whole thing runs offline / in CI.

Public surface:
  * :mod:`narrate.facts`     — ``Fact`` / ``FactSet`` + index loaders
  * :mod:`narrate.prompts`   — the parameterized template family
  * :mod:`narrate.generator` — ``Generator`` protocol, ``MockGenerator``, cache
  * :mod:`narrate.checker`   — ``Checker`` protocol, ``MockChecker`` (grounding)
  * :mod:`narrate.session`   — the ``narrate_unit`` loop
  * :mod:`narrate.report`    — dual JSON + human reports; output writer
  * :mod:`narrate.demo`      — the end-to-end D3 demo on one real ``lib/std`` unit

Guarantee: Declared/Empirical only (VR-5) — model/mock prose is never Proven or
Exact.  Pure Python standard library (Termux-portable).
"""

from __future__ import annotations

__all__ = [
    "facts",
    "prompts",
    "generator",
    "checker",
    "session",
    "report",
]

__version__ = "0.1.0"
