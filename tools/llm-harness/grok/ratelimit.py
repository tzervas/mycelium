"""Per-model rate pacing for the live (sync) backend (M-330; never-silent G2).

The live backend issues one chat request at a time, sequentially, and must stay
under each model's **RPM** (requests/min) and **TPM** (tokens/min) budgets, and
back off on a ``429`` / secondary-limit / ``Retry-After``.

Approach (adapted, NOT imported, from the repo's tested
``tools/github/gh-issues-sync.py`` — its ``RateGate`` coalesces a pause and its
``should_pause_for_rate_limit`` parses ``Retry-After`` with a bounded cap):

  * **RPM** — a sliding 60 s window of request timestamps. Before a request, if
    the window already holds ``rpm`` entries, wait until the oldest falls out.
  * **TPM** — a sliding 60 s window of (timestamp, token) entries. Before a
    request, if admitting its *estimated* tokens would exceed ``tpm``, wait until
    enough token-mass ages out.
  * **Backoff** — on a throttle response, sleep ``Retry-After`` when present, else
    exponential ``base * 2**attempt`` with jitter, capped at ``max_backoff``.

Everything is **never-silent**: each wait/backoff is reported through a logger.
The clock and sleep are injectable so the self-test drives the pacing math with a
virtual clock and zero real waiting (the pure decision functions
:func:`rpm_wait_seconds`, :func:`tpm_wait_seconds`, :func:`backoff_seconds` and
:func:`parse_retry_after` are exercised directly, offline).
"""

from __future__ import annotations

import logging
import re
import time
from collections import deque
from collections.abc import Callable
from dataclasses import dataclass, field

_LOG = logging.getLogger("grok.ratelimit")

WINDOW_SECONDS = 60.0  # RPM/TPM are per-minute budgets.

# ``Retry-After`` may be an integer number of seconds (RFC 7231). We accept the
# integer form; HTTP-date form is uncommon from JSON APIs and is treated as absent
# (we fall back to exponential backoff, which is safe — never-silent, bounded).
_RETRY_AFTER_RE = re.compile(r"(\d+)")


def parse_retry_after(value: str | int | float | None) -> float | None:
    """Parse a ``Retry-After`` value into seconds, or ``None`` if unparseable.

    PURE (offline-testable). Accepts an int/float directly, or a string holding a
    leading integer count of seconds. Negative/zero -> ``None`` (treat as absent).
    """
    if value is None:
        return None
    if isinstance(value, (int, float)):
        secs = float(value)
    else:
        m = _RETRY_AFTER_RE.search(str(value))
        if not m:
            return None
        secs = float(m.group(1))
    return secs if secs > 0 else None


def backoff_seconds(
    attempt: int,
    *,
    base: float = 1.0,
    cap: float = 60.0,
    jitter: float = 0.0,
) -> float:
    """Exponential backoff for retry ``attempt`` (0-based), bounded by ``cap``.

    PURE. ``attempt=0`` -> ``base``; doubles each step; ``+ jitter`` (caller passes
    a precomputed jitter so this stays deterministic for the self-test). Clamped
    to ``[0, cap]`` AFTER adding jitter so the cap is a true ceiling.
    """
    if attempt < 0:
        raise ValueError(f"attempt must be >= 0, got {attempt}")
    raw = base * (2.0**attempt) + jitter
    return max(0.0, min(raw, cap))


def rpm_wait_seconds(timestamps: deque[float], rpm: int, now: float) -> float:
    """Seconds to wait before a new request to respect ``rpm`` (PURE).

    ``timestamps`` is the recent-request window (ascending). Returns 0.0 if there
    is room; otherwise the time until the oldest in-window entry ages past 60 s.
    Does not mutate the deque (the caller prunes + appends after waiting).
    """
    if rpm <= 0:
        raise ValueError(f"rpm must be positive, got {rpm}")
    cutoff = now - WINDOW_SECONDS
    in_window = [t for t in timestamps if t > cutoff]
    if len(in_window) < rpm:
        return 0.0
    # Need the oldest entry to leave the window. Wait until it is older than 60 s.
    oldest = in_window[0]
    return max(0.0, (oldest + WINDOW_SECONDS) - now)


def tpm_wait_seconds(
    token_events: deque[tuple[float, int]],
    tpm: int,
    incoming_tokens: int,
    now: float,
) -> float:
    """Seconds to wait so admitting ``incoming_tokens`` keeps the 60 s sum <= ``tpm``.

    PURE. ``token_events`` is (timestamp, tokens) ascending. Returns 0.0 if the
    incoming request fits now. Otherwise returns the time until enough of the
    oldest token-mass ages out to make room. If a single request's estimate
    exceeds the whole TPM budget, we cannot ever fit it — return 0.0 and let the
    caller proceed (the server, not the pacer, is the final authority; the pacer
    is best-effort and never deadlocks on an over-budget single request).
    """
    if tpm <= 0:
        raise ValueError(f"tpm must be positive, got {tpm}")
    if incoming_tokens >= tpm:
        # Can't ever fit under budget alongside anything; don't spin forever.
        return 0.0
    cutoff = now - WINDOW_SECONDS
    in_window = [(t, n) for (t, n) in token_events if t > cutoff]
    used = sum(n for _, n in in_window)
    if used + incoming_tokens <= tpm:
        return 0.0
    # Drain oldest entries (in time order) until the incoming request fits.
    need_to_free = used + incoming_tokens - tpm
    freed = 0
    for ts, n in in_window:  # ascending: oldest first
        freed += n
        if freed >= need_to_free:
            # Once this entry ages out of the window, enough mass is freed.
            return max(0.0, (ts + WINDOW_SECONDS) - now)
    return 0.0


@dataclass
class RatePacer:
    """Stateful per-model sliding-window pacer for RPM + TPM (live mode).

    Construct one per model. Call :meth:`acquire` BEFORE each request with an
    estimate of the request's total tokens; it blocks (via the injected ``sleep``)
    until both budgets have room, then records the admission. After the real token
    usage is known, call :meth:`record_actual` to correct the last estimate so the
    TPM window reflects truth.

    ``now``/``sleep`` are injectable for deterministic offline testing.
    """

    rpm: int
    tpm: int
    now: Callable[[], float] = time.monotonic
    sleep: Callable[[float], None] = time.sleep
    _req_times: deque[float] = field(default_factory=deque)
    _tok_events: deque[tuple[float, int]] = field(default_factory=deque)
    _last_admit_est: int = 0

    def _prune(self, now: float) -> None:
        cutoff = now - WINDOW_SECONDS
        while self._req_times and self._req_times[0] <= cutoff:
            self._req_times.popleft()
        while self._tok_events and self._tok_events[0][0] <= cutoff:
            self._tok_events.popleft()

    def acquire(self, estimated_tokens: int) -> float:
        """Block until RPM and TPM both have room; record the admission.

        Returns the total seconds waited (>= 0). Never-silent: a non-trivial wait
        is logged with the cause.
        """
        if estimated_tokens < 0:
            raise ValueError(f"estimated_tokens must be >= 0, got {estimated_tokens}")
        waited = 0.0
        # Loop because after sleeping for one budget the *other* may now bind.
        while True:
            now = self.now()
            self._prune(now)
            r_wait = rpm_wait_seconds(self._req_times, self.rpm, now)
            t_wait = tpm_wait_seconds(self._tok_events, self.tpm, estimated_tokens, now)
            wait = max(r_wait, t_wait)
            if wait <= 0.0:
                break
            cause = "RPM" if r_wait >= t_wait else "TPM"
            _LOG.info(
                "rate pacer: waiting %.2fs for %s budget (rpm=%d tpm=%d est_tok=%d)",
                wait,
                cause,
                self.rpm,
                self.tpm,
                estimated_tokens,
            )
            self.sleep(wait)
            waited += wait
        now = self.now()
        self._req_times.append(now)
        self._tok_events.append((now, estimated_tokens))
        self._last_admit_est = estimated_tokens
        return waited

    def record_actual(self, actual_tokens: int) -> None:
        """Correct the most-recent admission's token estimate to the real count.

        Keeps the TPM window honest once the response is in. No-op if no admission
        is pending.
        """
        if actual_tokens < 0:
            raise ValueError(f"actual_tokens must be >= 0, got {actual_tokens}")
        if not self._tok_events:
            return
        ts, _ = self._tok_events[-1]
        self._tok_events[-1] = (ts, actual_tokens)


# -- throttle-response classification (mirrors gh-issues-sync, adapted) --------

_SECONDARY_NEEDLES = (
    "rate limit",
    "rate_limit",
    "too many requests",
    "secondary rate limit",
    "overloaded",
    "try again later",
)


@dataclass(frozen=True)
class ThrottleSignal:
    """Decision about whether/how long to back off after a failed request (PURE)."""

    should_retry: bool
    seconds: float
    reason: str


def classify_throttle(
    *,
    status_code: int | None,
    retry_after: str | int | float | None,
    body: str | None,
    attempt: int,
    max_attempts: int,
    base: float = 1.0,
    cap: float = 60.0,
    jitter: float = 0.0,
) -> ThrottleSignal:
    """Decide retry + backoff for a failed live request (PURE; offline-tested).

    A throttle is a ``429``, a ``5xx``, or a body containing a rate-limit/overload
    needle. The wait is ``Retry-After`` (clamped to ``cap``) when present, else
    exponential :func:`backoff_seconds`. Returns ``should_retry=False`` once
    ``attempt`` reaches ``max_attempts`` (bounded; never an infinite loop).
    """
    body_low = (body or "").lower()
    is_429 = status_code == 429
    is_5xx = status_code is not None and 500 <= status_code < 600
    needle = any(n in body_low for n in _SECONDARY_NEEDLES)
    throttled = is_429 or is_5xx or needle
    if not throttled:
        return ThrottleSignal(False, 0.0, "not-throttled")
    if attempt + 1 >= max_attempts:
        return ThrottleSignal(
            False,
            0.0,
            f"throttled (status={status_code}) but retries exhausted "
            f"({attempt + 1}/{max_attempts})",
        )
    ra = parse_retry_after(retry_after)
    if ra is not None:
        secs = min(ra, cap)
        why = f"Retry-After={ra:g}s (capped {secs:g}s)"
    else:
        secs = backoff_seconds(attempt, base=base, cap=cap, jitter=jitter)
        why = f"exponential backoff attempt={attempt} -> {secs:g}s"
    label = "429" if is_429 else ("5xx" if is_5xx else "rate-limit-needle")
    return ThrottleSignal(True, secs, f"{label}: {why}")
