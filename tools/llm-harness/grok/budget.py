"""Conservative USD spend gate for live/batch runs — the operator's ``--max-usd`` ceiling.

**Never-silent (G2).** Before each unit of work the harness *estimates* its cost and, if that
estimate would push the cumulative **actual** spend past the cap, **refuses the unit before it
is sent**: the run stops with a partial, honestly-flagged report rather than quietly launching
more work. Actual billed cost is then accumulated as the run proceeds.

**Honest scope (VR-5) — this is a *best-effort* gate, NOT a formal upper bound.** The per-call
token figure the harness uses is a heuristic (``coauthor_loop._estimate_tokens`` ≈ chars//4 +
fixed headroom), and live requests do not bound the completion (no ``max_tokens``), so a single
in-flight request *can* bill more than estimated. The gate therefore *reduces* over-spend risk
and stops *early* on the estimate (it is deliberately biased high), but it cannot guarantee the
final billed total stays under the cap to the cent. What it does guarantee: no *new* unit is
started once the estimate says the budget is reached, and the spend that is reported is the
real one — never a fabricated or silently-ignored cost.

Pure stdlib, no network/SDK, so the gate logic is exercised by the offline ``--self-test``.
"""

from __future__ import annotations

import logging
import math
from dataclasses import dataclass

# A conservative per-request token figure used only to *estimate* the next unit of work when the
# real prompt size is not yet known (the first task of a model). Chosen to over-count a typical
# code-gen request so the gate biases toward stopping early — but it is a heuristic, not a bound
# (live completions are not capped by ``max_tokens``; see the module docstring).
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
            f"USD spend cap reached: spent ${spent_usd:.4f} + next≈${est_usd:.4f} would "
            f"exceed cap ${cap_usd:.2f} (at model {model_id}). Refusing this unit and "
            f"stopping before it is sent (G2; the gate is conservative, not a formal bound)."
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
        if not math.isfinite(self.cap_usd):
            raise ValueError(
                f"budget cap must be a finite number, got {self.cap_usd!r} "
                "(reject nan/inf so the cap can never be silently disabled)"
            )
        if self.cap_usd < 0:
            raise ValueError(f"budget cap must be >= 0, got {self.cap_usd}")

    @property
    def remaining_usd(self) -> float:
        """USD left before the cap (never negative)."""
        return max(0.0, self.cap_usd - self.spent_usd)

    def would_exceed(self, est_usd: float) -> bool:
        """Whether committing ``est_usd`` more would push cumulative spend past the cap.

        A non-finite estimate (nan/inf) is treated as an **automatic exceed** — a unit whose
        cost cannot be bounded must never be started (a NaN comparison would otherwise be
        silently False and slip past the gate).
        """
        if not math.isfinite(est_usd):
            return True
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
