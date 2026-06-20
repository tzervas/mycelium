"""Hard USD spend cap for live/batch runs — the operator's ``--max-usd`` ceiling.

**Never-silent (G2).** A unit of work whose *conservative* cost estimate would push the
cumulative **actual** spend past the cap is **refused before it is sent**: the run stops
with a partial, honestly-flagged report rather than quietly over-spending. The estimate
that guards the next unit is deliberately conservative (it never under-counts tokens), so
the cap is a true upper bound on what can be billed, not a hope.

**Honesty (VR-5).** ``spent_usd`` accumulates *actual* per-request cost (the billed
tokens), priced via :meth:`models.ModelSpec.cost_usd`; the *estimate* is only used to
decide whether to start the next unit. The two are kept distinct so the reported spend is
the real one.

This module is pure stdlib and has **no network/SDK** dependency, so the cap logic is
exercised by the offline ``--self-test`` (the plumbing is Empirical; the live spend is
whatever the run actually bills).
"""

from __future__ import annotations

import logging
from dataclasses import dataclass

# A conservative per-request token figure used only to *estimate* the next unit of work
# when the real prompt size is not yet known (the first task of a model). Chosen to
# over-count a typical code-gen request so the cap errs toward stopping early, never late.
CONSERVATIVE_TOKENS_PER_REQUEST = 4096


class BudgetExceeded(Exception):
    """The next unit of work's estimated cost would breach the USD cap — work refused.

    Carries the numbers so the caller can finalize a partial, honestly-flagged report.
    """

    def __init__(
        self, *, spent_usd: float, est_usd: float, cap_usd: float, model_id: str
    ) -> None:
        self.spent_usd = spent_usd
        self.est_usd = est_usd
        self.cap_usd = cap_usd
        self.model_id = model_id
        super().__init__(
            f"USD budget cap reached: spent ${spent_usd:.4f} + next≈${est_usd:.4f} would "
            f"exceed cap ${cap_usd:.2f} (at model {model_id}). Refusing the request and "
            f"stopping — never silently over-spend (G2)."
        )


@dataclass
class BudgetGuard:
    """A shared, cross-model cumulative-spend ceiling. **One instance guards a whole run.**

    The same guard is threaded through every model so the cap is the **total** xAI spend,
    not a per-model budget.
    """

    cap_usd: float
    spent_usd: float = 0.0
    refused: bool = False

    def __post_init__(self) -> None:
        if self.cap_usd < 0:
            raise ValueError(f"budget cap must be >= 0, got {self.cap_usd}")

    @property
    def remaining_usd(self) -> float:
        """USD left before the cap (never negative)."""
        return max(0.0, self.cap_usd - self.spent_usd)

    def would_exceed(self, est_usd: float) -> bool:
        """Whether committing ``est_usd`` more would push cumulative spend past the cap."""
        return self.spent_usd + max(0.0, est_usd) > self.cap_usd

    def check_or_raise(
        self, est_usd: float, *, model_id: str, log: logging.Logger | None = None
    ) -> None:
        """Refuse (raise :class:`BudgetExceeded`) before work whose estimate breaches the cap.

        Sets :attr:`refused` so a finalizer can flag the report as stopped-at-cap.
        """
        if self.would_exceed(est_usd):
            self.refused = True
            exc = BudgetExceeded(
                spent_usd=self.spent_usd,
                est_usd=est_usd,
                cap_usd=self.cap_usd,
                model_id=model_id,
            )
            if log is not None:
                log.warning("%s", exc)
            raise exc

    def record(self, actual_usd: float) -> None:
        """Add a completed unit's **actual** billed cost to the running total."""
        self.spent_usd += max(0.0, actual_usd)

    def summary(self) -> str:
        """A one-line spend/cap status for logs and the report footer."""
        flag = " — STOPPED AT CAP" if self.refused else ""
        return (
            f"budget: spent ${self.spent_usd:.4f} of ${self.cap_usd:.2f} cap "
            f"(remaining ${self.remaining_usd:.4f}){flag}"
        )
