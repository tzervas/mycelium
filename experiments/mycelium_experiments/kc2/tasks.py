"""The fixed KC-2 benchmark task set (M-002: "fixed benchmark: tasks generated in the minimal
Mycelium surface fragment vs a Python-embedded DSL baseline").

Each task carries the natural-language prompt a generator receives (the language/DSL primer is
generator configuration, not task content), the machine-checkable acceptance for both arms, and
**reference solutions**. The references exist to prove the benchmark is *well-posed* (every task
is solvable in both arms — checked by the test suite); they are never used to score a generator.

The set is *fixed*: append new tasks rather than editing existing ones once a baseline run has
been recorded, or the SC-5b number stops being comparable (append-only, the changelog rule
applied to a benchmark).
"""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class Task:
    """One benchmark item, with acceptance criteria for both arms."""

    id: str
    """Stable identifier."""
    prompt: str
    """The task statement handed to the generator (both arms get the same statement)."""
    expect_main: str
    """Mycelium arm: the required `fn main() -> <this>` return type (myc-check --expect-main)."""
    expect_baseline: tuple[str, int]
    """Baseline arm: required result of `main()` — ("bin"|"tern", width)."""
    reference_mycelium: str
    """A known-good Mycelium solution (well-posedness witness, not a scoring aid)."""
    reference_baseline: str
    """A known-good baseline-DSL solution (same role)."""


TASKS: tuple[Task, ...] = (
    Task(
        id="kc2-01-literal",
        prompt="Define a nullary function `main` returning the 8-bit binary constant 1011_0010.",
        expect_main="Binary{8}",
        expect_baseline=("bin", 8),
        reference_mycelium="colony bench\nfn main() -> Binary{8} = 0b1011_0010\n",
        reference_baseline="def main():\n    return Bin('1011_0010')\n",
    ),
    Task(
        id="kc2-02-complement",
        prompt=(
            "Define a nullary function `main` returning the bitwise complement of the 8-bit "
            "binary constant 1011_0010."
        ),
        expect_main="Binary{8}",
        expect_baseline=("bin", 8),
        reference_mycelium="colony bench\nfn main() -> Binary{8} = not(0b1011_0010)\n",
        reference_baseline="def main():\n    return bnot(Bin('1011_0010'))\n",
    ),
    Task(
        id="kc2-03-xor",
        prompt=(
            "Define a nullary function `main` returning the bitwise xor of the 8-bit binary "
            "constants 1011_0010 and 1111_1111."
        ),
        expect_main="Binary{8}",
        expect_baseline=("bin", 8),
        reference_mycelium=(
            "colony bench\nfn main() -> Binary{8} = xor(0b1011_0010, 0b1111_1111)\n"
        ),
        reference_baseline=("def main():\n    return xor(Bin('1011_0010'), Bin('1111_1111'))\n"),
    ),
    Task(
        id="kc2-04-ternary-add",
        prompt=(
            "Define a nullary function `main` returning the balanced-ternary sum of the 4-trit "
            "words 00+- and 0+0- (most-significant trit first)."
        ),
        expect_main="Ternary{4}",
        expect_baseline=("tern", 4),
        reference_mycelium="colony bench\nfn main() -> Ternary{4} = add(<00+->, <0+0->)\n",
        reference_baseline="def main():\n    return tadd(Tern('00+-'), Tern('0+0-'))\n",
    ),
    Task(
        id="kc2-05-swap",
        prompt=(
            "Define a nullary function `main` converting the 8-bit binary constant 1011_0010 to "
            "a 6-trit balanced-ternary word, under an explicitly named conversion policy "
            "(call it `rt`)."
        ),
        expect_main="Ternary{6}",
        expect_baseline=("tern", 6),
        reference_mycelium=(
            "colony bench\nfn main() -> Ternary{6} = swap(0b1011_0010, to: Ternary{6}, policy: rt)\n"
        ),
        reference_baseline=(
            "def main():\n    return swap(Bin('1011_0010'), to=('tern', 6), policy='rt')\n"
        ),
    ),
    Task(
        id="kc2-06-helper",
        prompt=(
            "Define a helper function `flip` taking one 8-bit binary word and returning its "
            "bitwise complement, and a nullary function `main` applying `flip` twice to the "
            "constant 1010_1010."
        ),
        expect_main="Binary{8}",
        expect_baseline=("bin", 8),
        reference_mycelium=(
            "colony bench\n"
            "fn flip(x: Binary{8}) -> Binary{8} = not(x)\n"
            "fn main() -> Binary{8} = flip(flip(0b1010_1010))\n"
        ),
        reference_baseline=(
            "def flip(x):\n    return bnot(x)\n\ndef main():\n    return flip(flip(Bin('1010_1010')))\n"
        ),
    ),
    Task(
        id="kc2-07-data-match",
        prompt=(
            "Declare a sum type `Sign` with constructors Neg, Zero, Pos. Define a function "
            "`label` mapping a Sign to a 1-trit balanced-ternary word (Neg -> -, Zero -> 0, "
            "Pos -> +) by case analysis, and a nullary function `main` returning `label` of Zero."
        ),
        expect_main="Ternary{1}",
        expect_baseline=("tern", 1),
        reference_mycelium=(
            "colony bench\n"
            "type Sign = Neg | Zero | Pos\n"
            "fn label(s: Sign) -> Ternary{1} =\n"
            "    match s { Neg => <->, Zero => <0>, _ => <+> }\n"
            "fn main() -> Ternary{1} = label(Zero)\n"
        ),
        reference_baseline=(
            "from enum import Enum\n"
            "class Sign(Enum):\n    NEG = -1\n    ZERO = 0\n    POS = 1\n"
            "def label(s):\n"
            "    match s:\n"
            "        case Sign.NEG: return Tern('-')\n"
            "        case Sign.ZERO: return Tern('0')\n"
            "        case _: return Tern('+')\n"
            "def main():\n    return label(Sign.ZERO)\n"
        ),
    ),
    Task(
        id="kc2-08-matured",
        prompt=(
            "Define a nullary function `main`, marked as a promoted stable component "
            "(`matured` in Mycelium; in the baseline DSL there is no equivalent marker — just "
            "define `main`), returning the 8-bit binary constant 0000_1111."
        ),
        expect_main="Binary{8}",
        expect_baseline=("bin", 8),
        reference_mycelium="colony bench\nmatured fn main() -> Binary{8} = 0b0000_1111\n",
        reference_baseline="def main():\n    return Bin('0000_1111')\n",
    ),
)
