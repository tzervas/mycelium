"""The gold task set the co-authoring loop and ablation run against (M-330/M-381).

Each task is a natural-language spec the model must realise as a Mycelium program,
plus a short id. These are *composition*-flavoured tasks (the M-381 protocol asks
for "a composition-task subset wider than kc2-01…10"; research/11 §T11.7). They
are intentionally small and self-contained so a single generation is plausible and
the syntactic/type-check signal is crisp.

HONESTY: this is a seed set, tagged ``Declared`` (asserted, not yet validated as a
representative benchmark). It is configuration, not a result. The ``task_set_id``
is recorded in every report so a run's provenance is unambiguous.
"""

from __future__ import annotations

from dataclasses import dataclass

TASK_SET_ID = "gold-compose-v1"


@dataclass(frozen=True)
class Task:
    """One co-authoring task: an id and the natural-language spec."""

    id: str
    spec: str


# A compact composition-oriented gold set. Specs describe behaviour; the model
# must produce a `nodule`-headed program with explicit, never-silent swaps.
GOLD_TASKS: list[Task] = [
    Task("g01-identity", "Define a function `id` mapping a Binary{8} to itself."),
    Task(
        "g02-not",
        "Define a function `flip` over Binary{8} that returns the bitwise NOT.",
    ),
    Task(
        "g03-double",
        "Define a function `double` over Ternary{6} that adds its argument to itself.",
    ),
    Task(
        "g04-widen-swap",
        "Define a function `widen` from Binary{8} to Ternary{6} using an explicit roundtrip swap.",
    ),
    Task(
        "g05-narrow-swap",
        "Define a function `narrow` from Ternary{6} to Binary{8} using an explicit clamp swap.",
    ),
    Task(
        "g06-compose-not-double",
        "Define a function over Ternary{6} that doubles its input and returns the "
        "result; name it `dbl`.",
    ),
    Task(
        "g07-and-then-widen",
        "Define a function from Binary{8} to Ternary{6} that first applies bitwise "
        "NOT, then widens via an explicit roundtrip swap.",
    ),
    Task(
        "g08-roundtrip",
        "Define a function from Binary{8} to Binary{8} that widens to Ternary{6} "
        "with a roundtrip swap and narrows back with a clamp swap.",
    ),
]


def task_set(name: str = TASK_SET_ID) -> list[Task]:
    """Return the gold task list for ``name`` (only ``gold-compose-v1`` for now)."""
    if name in (TASK_SET_ID, "gold", "default"):
        return list(GOLD_TASKS)
    raise ValueError(f"unknown task set {name!r}; known: {TASK_SET_ID}")
