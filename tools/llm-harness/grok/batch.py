"""Batch execution: submit independent generations via xai_sdk, poll, then score.

WHAT IS BATCHABLE (and what is not):
  * BATCHABLE — independent, single-turn generations: the **first-pass** program
    for every gold task, and the M-381 ablation's **independent samples** (each
    (arm, task, seed) is its own self-contained prompt). These have no inter-
    request dependency, so the whole set can be submitted to the xAI batch API at
    once for lower cost.
  * NOT BATCHABLE — the iterative generate->feedback->fix correction loop (each fix
    depends on the previous round's diagnostics). That stays **live** (multi-turn).
    See ``grok/coauthor_loop.py``.

Cost tradeoff (documented for the operator): batch mode prices each token at the
**batch** rate (``models.toml`` ``batch_*_price``) — lower-or-equal to sync — in
exchange for latency (the batch completes asynchronously) and the loss of in-line
correction. For a one-shot quality sweep this is the cheaper path; for the
iterative co-authoring experience, live is required.

HONESTY / never-silent: a missing ``xai_sdk`` or a surface mismatch is an explicit
:class:`~grok.client.BackendUnavailableError` (raised by :class:`XaiBatchClient`),
never a silent fallback to live (which would charge sync prices the operator did
not choose). The offline ``--self-test`` does not submit a real batch; it verifies
the batch **cost-accounting** path (batch prices) through the Mock client and
:func:`score_batch_results`.
"""

from __future__ import annotations

import logging
import time
from dataclasses import dataclass
from typing import Any

from .client import BatchRequest, ChatResult, XaiBatchClient
from .coauthor_loop import build_messages
from .models import ModelSpec
from .scoring import MycCheckScorer, ScoreResult
from .tasks import Task

_LOG = logging.getLogger("grok.batch")


@dataclass
class BatchItem:
    """One submitted batch unit, paired back with its task + (optional) arm/seed."""

    custom_id: str
    task_id: str
    model_id: str
    seed: int
    arm: str = ""  # set for ablation samples; empty for plain generations


@dataclass
class BatchScored:
    """A batch item after its completion was scored."""

    item: BatchItem
    chat: ChatResult
    score: ScoreResult
    cost_usd: float

    def to_dict(self) -> dict[str, Any]:
        return {
            "custom_id": self.item.custom_id,
            "task_id": self.item.task_id,
            "model": self.item.model_id,
            "seed": self.item.seed,
            "arm": self.item.arm,
            "verdict": self.score.verdict,
            "syntactic_valid": self.score.syntactic_valid,
            "typecheck_pass": self.score.typecheck_pass,
            "prompt_tokens": self.chat.prompt_tokens,
            "completion_tokens": self.chat.completion_tokens,
            "cost_usd": round(self.cost_usd, 8),
        }


def build_generation_requests(
    *, tasks: list[Task], model: ModelSpec, seed: int = 42
) -> tuple[list[BatchRequest], list[BatchItem]]:
    """Build first-pass generation requests (one per task) for batch submission.

    Returns the SDK-facing requests and the parallel bookkeeping items (PURE).
    """
    requests: list[BatchRequest] = []
    items: list[BatchItem] = []
    for t in tasks:
        cid = f"gen::{model.id}::{t.id}::s{seed}"
        msgs = build_messages(t.spec, None)
        requests.append(
            BatchRequest(
                custom_id=cid, model=model.id, messages=msgs, params={"seed": seed}
            )
        )
        items.append(
            BatchItem(custom_id=cid, task_id=t.id, model_id=model.id, seed=seed)
        )
    return requests, items


def score_batch_results(
    *,
    items: list[BatchItem],
    results: dict[str, ChatResult],
    model: ModelSpec,
    scorer: MycCheckScorer,
) -> list[BatchScored]:
    """Score completed batch results, computing **batch-price** cost (PURE-ish).

    ``results`` maps ``custom_id -> ChatResult``. A missing id (the batch dropped
    a unit) is recorded as an explicit error result (never-silent G2). Cost uses
    the batch price pair — this is the path the self-test pins to ensure batch mode
    never silently bills sync rates.
    """
    scored: list[BatchScored] = []
    for it in items:
        chat = results.get(it.custom_id)
        if chat is None:
            chat = ChatResult(
                ok=False,
                content="",
                prompt_tokens=0,
                completion_tokens=0,
                latency_s=0.0,
                model=it.model_id,
                error=f"batch result missing for custom_id={it.custom_id}",
            )
        if chat.ok and chat.content.strip():
            score = scorer.score(chat.content)
        else:
            score = ScoreResult(
                verdict="error",
                syntactic_valid=False,
                typecheck_pass=False,
                exit_code=None,
                message=chat.error or "empty batch completion",
            )
        cost = model.cost_usd(
            prompt_tokens=chat.prompt_tokens,
            completion_tokens=chat.completion_tokens,
            batch=True,  # batch mode ALWAYS uses batch prices (honest accounting)
        )
        scored.append(BatchScored(item=it, chat=chat, score=score, cost_usd=cost))
    return scored


@dataclass
class BatchPollConfig:
    """How the live operator polls a submitted batch to completion."""

    interval_s: float = 15.0
    timeout_s: float = 3600.0


def submit_and_collect(
    *,
    client: XaiBatchClient,
    batch_name: str,
    requests: list[BatchRequest],
    poll: BatchPollConfig | None = None,
    log: logging.Logger | None = None,
) -> dict[str, ChatResult]:
    """Submit a batch and poll it to completion, returning ``custom_id -> ChatResult``.

    LIVE-ONLY: requires ``xai_sdk`` + key + network (not exercised by --self-test).
    Wrapped so SDK surface drift is explicit. Because exact SDK result shapes vary
    by version, the per-item parsing is delegated to :func:`_extract_batch_result`,
    which the operator can adjust against the installed SDK.
    """
    log = log or _LOG
    poll = poll or BatchPollConfig()
    handle = client.submit(batch_name=batch_name, requests=requests)
    log.info("submitted batch %r (%d requests); polling…", batch_name, len(requests))
    deadline = time.monotonic() + poll.timeout_s
    while True:
        status = _batch_status(handle)
        if status in ("completed", "succeeded", "done"):
            break
        if status in ("failed", "cancelled", "expired"):
            raise RuntimeError(f"batch {batch_name!r} ended in status {status!r}")
        if time.monotonic() > deadline:
            raise TimeoutError(
                f"batch {batch_name!r} did not complete within {poll.timeout_s}s "
                f"(last status {status!r})"
            )
        time.sleep(poll.interval_s)
    return _collect_results(handle)


def _batch_status(handle: Any) -> str:  # pragma: no cover - live SDK
    """Best-effort status read from an SDK batch handle (never-silent on mismatch)."""
    for attr in ("status", "state"):
        val = getattr(handle, attr, None)
        if val is not None:
            return str(val).lower()
    refresh = getattr(handle, "refresh", None) or getattr(handle, "get", None)
    if callable(refresh):
        refreshed = refresh()
        return str(getattr(refreshed, "status", "unknown")).lower()
    return "unknown"


def _collect_results(handle: Any) -> dict[str, ChatResult]:  # pragma: no cover - live
    """Extract per-item ChatResults from a completed SDK batch handle."""
    out: dict[str, ChatResult] = {}
    results = getattr(handle, "results", None)
    if callable(results):
        results = results()
    for r in results or []:
        cid, chat = _extract_batch_result(r)
        out[cid] = chat
    return out


def _extract_batch_result(raw: Any) -> tuple[str, ChatResult]:  # pragma: no cover
    """Map one SDK batch result object to (custom_id, ChatResult).

    Tolerant of dict-like or attribute-like result objects. Adjust here if the
    installed ``xai_sdk`` exposes a different result shape.
    """

    def _get(obj: Any, key: str, default: Any = None) -> Any:
        if isinstance(obj, dict):
            return obj.get(key, default)
        return getattr(obj, key, default)

    cid = str(_get(raw, "custom_id", "?"))
    resp = _get(raw, "response", raw)
    choices = _get(resp, "choices", []) or []
    content = ""
    finish = ""
    if choices:
        msg = _get(choices[0], "message", {}) or {}
        content = (_get(msg, "content", "") or "").strip()
        finish = _get(choices[0], "finish_reason", "") or ""
    usage = _get(resp, "usage", {}) or {}
    return cid, ChatResult(
        ok=bool(content),
        content=content,
        prompt_tokens=int(_get(usage, "prompt_tokens", 0) or 0),
        completion_tokens=int(_get(usage, "completion_tokens", 0) or 0),
        latency_s=0.0,
        model=str(_get(resp, "model", "")),
        finish_reason=finish,
        error="" if content else "empty/missing batch completion",
    )
