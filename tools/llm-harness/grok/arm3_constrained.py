"""Arm 3 (grammar-constrained decoding) — SCAFFOLD / interface contract (M-381).

Arm 3 of the T3.6 retention ablation: a *novel-surface* arm (a retention-ratio
numerator candidate) where the model decodes under a **grammar constraint** (GBNF /
Outlines / Guidance) so its output is syntactically valid `.myc` by construction.

WHY IT IS BLOCKED ON THE xAI PATH
---------------------------------
The xAI OpenAI-compatible REST endpoint exposes **no** grammar/GBNF parameter, so
constrained decoding cannot run through it. Arm 3 therefore needs a **local**
GBNF-capable backend (llama.cpp / `llama-cpp-python`, or Outlines wrapping a local
model) — the M-331 llama.cpp harness is its natural home. With no local model present
in an environment, arm 3 must **gracefully SKIP** (never a fabricated score — G2).

CONTRACT (Leaf MS35-A implements this; the orchestrator wires it into ablation.py):
  * ``mycelium_gold_gbnf() -> str`` — a GBNF grammar covering the gold-task `.myc`
    surface subset (nodule + fn + the expression forms the gold set uses). Must be a
    well-formed GBNF string (offline-testable: balanced rules, a start symbol).
  * ``ConstrainedBackend`` — an optional local backend. Construction probes for a
    backend (``llama_cpp`` import + a model path from env ``MYC_ARM3_MODEL``); if
    absent it is ``available == False`` and ``generate`` returns a SKIP result. A
    present backend decodes under the GBNF. NEVER raises a bare ImportError mid-run.
  * ``arm3_constrained_prompt(task: Task) -> list[ChatMessage]`` — the arm-3 prompt
    (PURE; Declared).
  * ``SKIP_REASON_NO_BACKEND`` — the explicit skip reason string.

HONESTY (G2 / VR-5): a missing backend/model is an explicit SKIP, never a 0%/100%.
The grammar guarantees syntactic validity only; typecheck still runs via myc-check
(arm 3 outputs `.myc` directly, so it is scored like arms 1/2).
"""

from __future__ import annotations

from .client import ChatMessage
from .tasks import Task

SKIP_REASON_NO_BACKEND = "arm3-constrained-decoder-not-available"


def mycelium_gold_gbnf() -> str:
    """Return a GBNF grammar for the gold-task `.myc` surface subset.

    SCAFFOLD: Leaf MS35-A implements the real grammar. Must be well-formed GBNF.
    """
    raise NotImplementedError("arm3: GBNF grammar — implemented by Leaf MS35-A")


def arm3_constrained_prompt(task: Task) -> list[ChatMessage]:
    """Build the arm-3 prompt (PURE, Declared). SCAFFOLD — Leaf MS35-A implements."""
    raise NotImplementedError("arm3: prompt — implemented by Leaf MS35-A")


class ConstrainedBackend:
    """Optional local GBNF-capable decoding backend (graceful-skip if absent).

    SCAFFOLD: Leaf MS35-A implements. ``available`` is False when no local backend
    + model is configured; ``generate`` then returns a SKIP-shaped result (G2).
    """

    def __init__(self) -> None:  # pragma: no cover - scaffold
        self.available = False
        raise NotImplementedError("arm3: backend — implemented by Leaf MS35-A")
