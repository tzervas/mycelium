"""Model rubric loading, ordering and USD cost accounting (M-330; VR-5).

The rubric lives in ``models.toml`` (sibling of this package). Each entry carries
a context window, RPM/TPM budgets and *two* price pairs — sync (live) and batch —
so cost can be computed with the mode-appropriate rate (house-rule 2: never a
silent or invented discount; batch prices default to sync prices in the seed file
until the published batch rates are filled in).

HONESTY: the numbers are ``Declared`` configuration, not measurements. This module
neither verifies nor upgrades that tag; it only loads, orders and arithmetics.

Pure stdlib: ``tomllib`` ships in CPython 3.11+. No third-party TOML dependency.
"""

from __future__ import annotations

import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Any

# The required price fields. Kept explicit so a malformed rubric fails LOUD (G2)
# rather than silently costing more than the user thinks.
_REQUIRED_FIELDS = (
    "id",
    "context",
    "tpm",
    "rpm",
    "in_price",
    "out_price",
    "batch_in_price",
    "batch_out_price",
)

# 1 million tokens — the unit prices are quoted per-megatoken.
TOKENS_PER_MTOK = 1_000_000


class ModelConfigError(ValueError):
    """Raised when ``models.toml`` is missing, malformed, or internally invalid.

    Never-silent (G2): a bad rubric must stop the run, not degrade quietly to a
    wrong cost estimate or a wrong model.
    """


@dataclass(frozen=True)
class ModelSpec:
    """One model's budget + pricing. Immutable once loaded (it is configuration)."""

    id: str
    context: int
    tpm: int
    rpm: int
    in_price: float  # USD / Mtok, sync (live) input
    out_price: float  # USD / Mtok, sync (live) output
    batch_in_price: float  # USD / Mtok, batch input
    batch_out_price: float  # USD / Mtok, batch output
    config_order: int = 0  # position in models.toml; the tie-breaker for sorting

    # -- cost helpers --------------------------------------------------------

    def input_price(self, *, batch: bool) -> float:
        """USD per Mtok for input tokens in the given mode."""
        return self.batch_in_price if batch else self.in_price

    def output_price(self, *, batch: bool) -> float:
        """USD per Mtok for output tokens in the given mode."""
        return self.batch_out_price if batch else self.out_price

    def cost_usd(
        self, *, prompt_tokens: int, completion_tokens: int, batch: bool
    ) -> float:
        """Computed USD for one request/sample.

        ``batch=True`` uses the batch price pair; ``batch=False`` the sync pair.
        This is the *only* place token-counts become dollars, so the mode↔price
        coupling is enforced in exactly one spot (DRY; honest cost accounting).
        """
        if prompt_tokens < 0 or completion_tokens < 0:
            raise ValueError(
                f"token counts must be non-negative, got "
                f"prompt={prompt_tokens}, completion={completion_tokens}"
            )
        in_usd = (prompt_tokens / TOKENS_PER_MTOK) * self.input_price(batch=batch)
        out_usd = (completion_tokens / TOKENS_PER_MTOK) * self.output_price(batch=batch)
        return in_usd + out_usd


def _sort_key(spec: ModelSpec) -> tuple[float, float, int]:
    """Cheapest-first: output price, then input price, then config order.

    Output price dominates because completion tokens dominate generation cost in
    a co-authoring loop (we read short specs and write longer programs).
    """
    return (spec.out_price, spec.in_price, spec.config_order)


def _coerce_spec(raw: dict[str, Any], order: int) -> ModelSpec:
    """Validate one ``[[model]]`` table and build a :class:`ModelSpec`."""
    missing = [f for f in _REQUIRED_FIELDS if f not in raw]
    if missing:
        raise ModelConfigError(
            f"model entry #{order} (id={raw.get('id', '?')!r}) is missing required "
            f"field(s): {', '.join(missing)}"
        )
    try:
        spec = ModelSpec(
            id=str(raw["id"]),
            context=int(raw["context"]),
            tpm=int(raw["tpm"]),
            rpm=int(raw["rpm"]),
            in_price=float(raw["in_price"]),
            out_price=float(raw["out_price"]),
            batch_in_price=float(raw["batch_in_price"]),
            batch_out_price=float(raw["batch_out_price"]),
            config_order=order,
        )
    except (TypeError, ValueError) as exc:
        raise ModelConfigError(
            f"model entry #{order} (id={raw.get('id', '?')!r}) has a malformed field: {exc}"
        ) from exc
    # Non-negativity / sanity. A zero or negative budget would wedge the pacer.
    if spec.rpm <= 0 or spec.tpm <= 0:
        raise ModelConfigError(
            f"model {spec.id!r}: rpm and tpm must be positive (got rpm={spec.rpm}, tpm={spec.tpm})"
        )
    if spec.context <= 0:
        raise ModelConfigError(f"model {spec.id!r}: context must be positive")
    for fld in ("in_price", "out_price", "batch_in_price", "batch_out_price"):
        if getattr(spec, fld) < 0:
            raise ModelConfigError(f"model {spec.id!r}: {fld} must be non-negative")
    return spec


def load_models(path: str | Path) -> list[ModelSpec]:
    """Load every ``[[model]]`` from ``path`` in file order (no sorting yet).

    Raises :class:`ModelConfigError` on a missing file, a TOML parse error, an
    empty rubric, a duplicate id, or any malformed/invalid field (all never-silent).
    """
    p = Path(path)
    if not p.is_file():
        raise ModelConfigError(f"model rubric not found: {p}")
    try:
        with p.open("rb") as fh:
            data = tomllib.load(fh)
    except tomllib.TOMLDecodeError as exc:
        raise ModelConfigError(f"could not parse {p}: {exc}") from exc

    entries = data.get("model")
    if not entries:
        raise ModelConfigError(
            f"{p} defines no [[model]] entries — nothing to run (G2: refusing to "
            "proceed with an empty rubric)"
        )
    if not isinstance(entries, list):
        raise ModelConfigError(f"{p}: [[model]] must be an array of tables")

    specs: list[ModelSpec] = []
    seen: set[str] = set()
    for i, raw in enumerate(entries):
        if not isinstance(raw, dict):
            raise ModelConfigError(f"{p}: model entry #{i} is not a table")
        spec = _coerce_spec(raw, i)
        if spec.id in seen:
            raise ModelConfigError(f"{p}: duplicate model id {spec.id!r}")
        seen.add(spec.id)
        specs.append(spec)
    return specs


def order_models(
    specs: list[ModelSpec],
    *,
    select: list[str] | None = None,
    order: list[str] | None = None,
) -> list[ModelSpec]:
    """Resolve the run order from loaded specs.

    Precedence:
      * ``order`` (if given) — an EXPLICIT id sequence; the result is exactly those
        ids in exactly that order. Unknown ids raise (never-silent).
      * else ``select`` (if given) — the named subset, emitted **cheapest-first**.
      * else — every model, cheapest-first.

    "Cheapest-first" = sorted by (out_price, in_price, config_order).
    """
    by_id = {s.id: s for s in specs}

    def _resolve(ids: list[str]) -> list[ModelSpec]:
        unknown = [i for i in ids if i not in by_id]
        if unknown:
            raise ModelConfigError(
                f"unknown model id(s): {', '.join(unknown)}. Known: {', '.join(sorted(by_id))}"
            )
        return [by_id[i] for i in ids]

    if order:
        return _resolve(order)
    pool = _resolve(select) if select else list(specs)
    return sorted(pool, key=_sort_key)


def default_models_path() -> Path:
    """Path to the bundled ``models.toml`` (sibling of the ``grok`` package)."""
    return Path(__file__).resolve().parent.parent / "models.toml"
