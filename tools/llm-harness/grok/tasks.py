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
    """One co-authoring task: an id, a natural-language spec, and an optional signature.

    The signature fields (``fn_name`` / ``param_name`` / ``param_type`` /
    ``return_type``) are the task's *intended* function type. They are **Declared**
    config (asserted, not a result) and are used ONLY by the arm-4 LlmCanonical->L1
    bridge (``llm_canonical_to_l1.convert_to_myc``): ``LlmCanonical`` is an
    expression IR with no function signature, so the bridge wraps the model's
    converted expression in this known signature before ``myc-check`` typechecks it
    — putting arm 4 on the same parse+typecheck bar as arms 1/2. See that module for
    the honest residual-asymmetry caveat (arm 4 is not asked to author the signature,
    making the retention ratio a conservative estimate). A task with no signature
    cannot be bridged (the arm-4 sample is scored not-clean — G2, never a false PASS).
    """

    id: str
    spec: str
    fn_name: str | None = None
    param_name: str | None = None
    param_type: str | None = None
    return_type: str | None = None


# A compact composition-oriented gold set. Specs describe behaviour; the model
# must produce a `nodule`-headed program with explicit, never-silent swaps.
# The signature fields are Declared config for the arm-4 bridge (see Task docstring).
GOLD_TASKS: list[Task] = [
    Task(
        "g01-identity",
        "Define a function `id` mapping a Binary{8} to itself.",
        fn_name="id",
        param_type="Binary{8}",
        return_type="Binary{8}",
    ),
    Task(
        "g02-not",
        "Define a function `flip` over Binary{8} that returns the bitwise NOT.",
        fn_name="flip",
        param_type="Binary{8}",
        return_type="Binary{8}",
    ),
    Task(
        "g03-double",
        "Define a function `double` over Ternary{6} that adds its argument to itself.",
        fn_name="double",
        param_type="Ternary{6}",
        return_type="Ternary{6}",
    ),
    Task(
        "g04-widen-swap",
        "Define a function `widen` from Binary{8} to Ternary{6} using an explicit roundtrip swap.",
        fn_name="widen",
        param_type="Binary{8}",
        return_type="Ternary{6}",
    ),
    Task(
        "g05-narrow-swap",
        "Define a function `narrow` from Ternary{6} to Binary{8} using an explicit clamp swap.",
        fn_name="narrow",
        param_type="Ternary{6}",
        return_type="Binary{8}",
    ),
    Task(
        "g06-compose-not-double",
        "Define a function over Ternary{6} that doubles its input and returns the "
        "result; name it `dbl`.",
        fn_name="dbl",
        param_type="Ternary{6}",
        return_type="Ternary{6}",
    ),
    Task(
        "g07-and-then-widen",
        "Define a function from Binary{8} to Ternary{6} that first applies bitwise "
        "NOT, then widens via an explicit roundtrip swap.",
        fn_name="and_then_widen",
        param_type="Binary{8}",
        return_type="Ternary{6}",
    ),
    Task(
        "g08-roundtrip",
        "Define a function from Binary{8} to Binary{8} that widens to Ternary{6} "
        "with a roundtrip swap and narrows back with a clamp swap.",
        fn_name="roundtrip",
        param_type="Binary{8}",
        return_type="Binary{8}",
    ),
]


def task_set(name: str = TASK_SET_ID) -> list[Task]:
    """Return the gold task list for ``name`` (only ``gold-compose-v1`` for now)."""
    if name in (TASK_SET_ID, "gold", "default"):
        return list(GOLD_TASKS)
    raise ValueError(f"unknown task set {name!r}; known: {TASK_SET_ID}")
