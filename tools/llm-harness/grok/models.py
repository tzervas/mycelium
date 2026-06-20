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

import sys
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


# Conservative Declared defaults for fields the GET /v1/models API does not provide.
# Tagged Declared — they are asserted conservative values, never independently measured.
_DISCOVER_RPM_DEFAULT = 60  # requests per minute (conservative)
_DISCOVER_TPM_DEFAULT = 2_000_000  # tokens per minute (conservative)
_DISCOVER_CTX_DEFAULT = 131_072  # context tokens when context_length absent


def from_api_discovery(raw_models: list[dict[str, Any]]) -> list["ModelSpec"]:
    """Convert raw ``GET /v1/models`` items into :class:`ModelSpec` objects.

    The API provides ``id``, ``context_length``, and ``pricing: {input, output}``
    (USD per Mtok).  Fields the API omits — ``tpm``, ``rpm``, and batch prices —
    receive conservative ``Declared`` defaults so the harness never under-estimates
    cost or over-drives rate limits.

    HONESTY (VR-5): every value produced here is ``Declared`` (API-asserted or
    conservatively defaulted).  The caller must not present resulting ModelSpec
    objects as ``Empirical``.  The conservative defaults are:
      - ``rpm`` = 60 req/min
      - ``tpm`` = 2,000,000 tok/min
      - ``batch_*_price`` = sync price (no invented discount; conservative)

    Models without a parseable pricing block are skipped with a warning to stderr
    (never-silent G2 — a model that cannot be priced cannot be safely gated).

    Raises :class:`ModelConfigError` when no usable models remain after filtering.
    """
    specs: list[ModelSpec] = []
    seen: set[str] = set()

    for i, raw in enumerate(raw_models):
        mid = raw.get("id")
        if not mid or not isinstance(mid, str):
            print(
                f"  discover: skipping entry #{i} — no usable id: {str(raw)[:80]}",
                file=sys.stderr,
            )
            continue
        if mid in seen:
            print(f"  discover: skipping duplicate model id {mid!r}", file=sys.stderr)
            continue

        pricing = raw.get("pricing")
        if not isinstance(pricing, dict):
            print(
                f"  discover: skipping {mid!r} — missing or non-object pricing block",
                file=sys.stderr,
            )
            continue

        raw_in = pricing.get("input")
        raw_out = pricing.get("output")
        if raw_in is None or raw_out is None:
            print(
                f"  discover: skipping {mid!r} — pricing.input or pricing.output absent",
                file=sys.stderr,
            )
            continue
        try:
            in_price = float(raw_in)
            out_price = float(raw_out)
        except (TypeError, ValueError):
            print(
                f"  discover: skipping {mid!r} — non-numeric pricing: {pricing}",
                file=sys.stderr,
            )
            continue
        if in_price < 0 or out_price < 0:
            print(
                f"  discover: skipping {mid!r} — negative pricing: {pricing}",
                file=sys.stderr,
            )
            continue

        raw_ctx = raw.get("context_length")
        try:
            context = int(raw_ctx) if raw_ctx is not None else _DISCOVER_CTX_DEFAULT
        except (TypeError, ValueError):
            context = _DISCOVER_CTX_DEFAULT
        context = max(1, context)

        spec = ModelSpec(
            id=mid,
            context=context,
            tpm=_DISCOVER_TPM_DEFAULT,  # Declared conservative default
            rpm=_DISCOVER_RPM_DEFAULT,  # Declared conservative default
            in_price=in_price,
            out_price=out_price,
            batch_in_price=in_price,  # = sync (no invented batch discount)
            batch_out_price=out_price,  # = sync (no invented batch discount)
            config_order=len(specs),
        )
        seen.add(mid)
        specs.append(spec)

    if not specs:
        raise ModelConfigError(
            "API discovery returned no usable models with pricing "
            "(every entry was skipped — check stderr for reasons)"
        )
    return specs
