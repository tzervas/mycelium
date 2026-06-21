"""Arm 5 (embedded-DSL baseline, RR-3) — SCAFFOLD / interface contract (M-381).

Arm 5 of the T3.6 retention ablation: the **embedded-DSL** baseline. RR-3 is the
"fall back to an embedded DSL in a high-resource host language" contingency
(DN-09 §3.1, Foundation §6). The arm measures: when the model writes the program in a
*familiar host language* (a small Python-embedded DSL that emits Mycelium), how does
its pass@1 compare? It is a *familiar-skin* point like arm 4, but in a mainstream host
rather than an S-expression projection.

STATUS: RR-3 is an UNSPENT contingency (KC-2 verdict = proceed), so no embedded DSL was
ever built. This arm builds the *baseline harness* so arm 5 is runnable for research;
it does not commit Mycelium to an embedded-DSL future (that needs a KC-2 reversal).

Unlike arm 3, arm 5 CAN run through the xAI REST client (it is ordinary text
generation of Python), then the emitted `.myc` is scored by the same myc-check.

CONTRACT (Leaf MS35-B implements; the orchestrator wires it into ablation.py):
  * The embedded DSL itself — a small, SAFE Python API that builds a Mycelium program
    and renders it to `.myc` text (e.g. ``nodule(...)``, ``fn(...)``, ``binary(8)``,
    ``ternary(6)``, ``swap(...)``, ``op("not", x)`` …). Pure; no I/O.
  * ``arm5_embedded_dsl_prompt(task: Task) -> list[ChatMessage]`` — the arm-5 prompt
    teaching the DSL and asking the model to write a program in it (PURE; Declared).
  * ``eval_embedded_dsl(code: str) -> str | None`` — evaluate the model's DSL code in a
    RESTRICTED sandbox (no imports, no builtins beyond the DSL surface; never exec
    arbitrary code) and return the rendered `.myc`, or ``None`` if it does not produce
    a valid program (scored not-clean — never a false PASS, G2). Must be robust to
    hostile/malformed input (no filesystem/network/`__import__` reachable).

HONESTY (G2 / VR-5): a DSL snippet that errors, times out, or escapes the sandbox
yields ``None`` (not-clean), never a fabricated PASS. The DSL only guarantees it
*emits* `.myc`; typecheck is myc-check's (arm 5 is scored like arms 1/2/4).
"""

from __future__ import annotations

from .client import ChatMessage
from .tasks import Task


def arm5_embedded_dsl_prompt(task: Task) -> list[ChatMessage]:
    """Build the arm-5 prompt (PURE, Declared). SCAFFOLD — Leaf MS35-B implements."""
    raise NotImplementedError("arm5: prompt — implemented by Leaf MS35-B")


def eval_embedded_dsl(code: str) -> str | None:
    """Evaluate DSL ``code`` in a restricted sandbox → rendered `.myc`, or None.

    SCAFFOLD: Leaf MS35-B implements with a SAFE evaluator (no arbitrary exec, no
    imports/builtins escape). Malformed/hostile input → None (G2).
    """
    raise NotImplementedError("arm5: sandboxed eval — implemented by Leaf MS35-B")
