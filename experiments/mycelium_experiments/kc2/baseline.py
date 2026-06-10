"""The Python-embedded DSL — the KC-2 baseline arm (M-002; R6; G10).

A deliberately small, *honest* embedded DSL mirroring the minimal Mycelium surface fragment:
width-checked binary words, balanced-ternary words, the elementwise logical ops, balanced-ternary
addition, and an explicit ``swap`` that always requires a policy and refuses out-of-range
conversions with an exception — never a silent wrap or coercion (the same never-silent posture as
the kernel, so the two arms are compared on expressiveness, not on safety culture).

This module is the *comparison subject* of the experiment, not part of the Mycelium kernel.
"""

from __future__ import annotations

from dataclasses import dataclass

_TRIT_VALUE = {"+": 1, "0": 0, "-": -1}
_TRIT_GLYPH = {1: "+", 0: "0", -1: "-"}


@dataclass(frozen=True)
class Bin:
    """A width-checked binary word, most-significant bit first (e.g. ``Bin("1011_0010")``)."""

    bits: str

    def __post_init__(self) -> None:
        cleaned = self.bits.replace("_", "")
        if not cleaned or any(c not in "01" for c in cleaned):
            msg = f"Bin needs a non-empty 0/1 string, got {self.bits!r}"
            raise TypeError(msg)
        object.__setattr__(self, "bits", cleaned)

    @property
    def width(self) -> int:
        """Bit width (digit count — a literal *is* its representation)."""
        return len(self.bits)

    def to_int(self) -> int:
        """Unsigned integer value."""
        return int(self.bits, 2)


@dataclass(frozen=True)
class Tern:
    """A width-checked balanced-ternary word over ``+0-``, most-significant trit first."""

    trits: str

    def __post_init__(self) -> None:
        if not self.trits or any(c not in "+0-" for c in self.trits):
            msg = f"Tern needs a non-empty +0- string, got {self.trits!r}"
            raise TypeError(msg)

    @property
    def width(self) -> int:
        """Trit count."""
        return len(self.trits)

    def to_int(self) -> int:
        """Balanced-ternary integer value."""
        v = 0
        for c in self.trits:
            v = v * 3 + _TRIT_VALUE[c]
        return v


def _int_to_tern(value: int, width: int) -> Tern:
    """Balanced-ternary encoding of ``value`` in ``width`` trits; explicit on overflow."""
    bound = (3**width - 1) // 2
    if not -bound <= value <= bound:
        msg = f"{value} is outside the balanced range of {width} trits (±{bound})"
        raise OverflowError(msg)
    digits: list[int] = []
    v = value
    for _ in range(width):
        r = v % 3
        v //= 3
        if r == 2:
            r = -1
            v += 1
        digits.append(r)
    return Tern("".join(_TRIT_GLYPH[d] for d in reversed(digits)))


def bnot(x: Bin) -> Bin:
    """Elementwise complement."""
    return Bin("".join("1" if c == "0" else "0" for c in x.bits))


def xor(a: Bin, b: Bin) -> Bin:
    """Elementwise xor; widths must agree (explicit, never broadcast)."""
    if a.width != b.width:
        msg = f"xor width mismatch: {a.width} vs {b.width}"
        raise TypeError(msg)
    return Bin("".join("1" if x != y else "0" for x, y in zip(a.bits, b.bits, strict=True)))


def tadd(a: Tern, b: Tern) -> Tern:
    """Fixed-width balanced-ternary addition; out of range is an explicit ``OverflowError``."""
    if a.width != b.width:
        msg = f"tadd width mismatch: {a.width} vs {b.width}"
        raise TypeError(msg)
    return _int_to_tern(a.to_int() + b.to_int(), a.width)


def swap(value: Bin | Tern, *, to: tuple[str, int], policy: str) -> Bin | Tern:
    """Convert between representations — never silent: the policy is mandatory, an unsupported
    pair or an out-of-range value is an explicit error (mirrors the kernel's S1/SC-3 posture).

    ``to`` is ``("bin", width)`` or ``("tern", trits)``.
    """
    if not policy:
        msg = "swap requires a policy"
        raise TypeError(msg)
    kind, width = to
    if kind == "tern" and isinstance(value, Bin):
        return _int_to_tern(value.to_int(), width)
    if kind == "bin" and isinstance(value, Tern):
        v = value.to_int()
        if v < 0 or v >= 2**width:
            msg = f"{v} is outside the unsigned range of {width} bits"
            raise OverflowError(msg)
        return Bin(format(v, f"0{width}b"))
    msg = f"unsupported swap: {type(value).__name__} -> {to!r}"
    raise TypeError(msg)
