"""Orchestration: run the gold task set (and/or the ablation) across models.

Ties the pieces together for a real run:
  * **live mode** — for each model (cheapest-first), build an OpenAI-compatible
    client and a per-model :class:`~grok.ratelimit.RatePacer`, run the M-330
    generate->fix loop per task (paced + backed off), aggregate quality +
    performance, and emit reports.
  * **batch mode** — submit the independent first-pass generations via
    ``xai_sdk``, poll, score with batch prices, and emit reports. The iterative
    correction loop is NOT batchable and is documented as a live-only follow-up.

This module is the live/batch driver; it is NOT exercised by ``--self-test`` (that
uses :mod:`grok.selftest`, which drives the same loop/scoring/report functions with
the Mock client and an injected scorer). Keeping the orchestration here and the
deterministic verification there means the green gate never needs a key or network.
"""

from __future__ import annotations

import logging
from collections.abc import Callable
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from . import report as report_mod
from .ablation import (
    AblationReport,
    compute_retention,
    default_arms,
    run_arm,
)
from .budget import (
    CONSERVATIVE_TOKENS_PER_REQUEST,
    BudgetExceeded,
    BudgetGuard,
)
from .batch import (
    BatchPollConfig,
    build_generation_requests,
    score_batch_results,
    submit_and_collect,
)
from .client import ChatClient, ChatResult, OpenAICompatClient, XaiBatchClient
from .coauthor_loop import STATUS_PARTIAL_PASS, STATUS_PASS, run_task_loop
from .models import ModelSpec
from .ratelimit import RatePacer, classify_throttle
from .scoring import VERDICT_CLEAN, MycCheckScorer, ScoreResult, aggregate_metrics
from .tasks import Task

_LOG = logging.getLogger("grok.runner")


@dataclass
class RunConfig:
    """Resolved options for a full run."""

    mode: str  # "live" | "batch"
    models: list[ModelSpec]
    tasks: list[Task]
    task_set_id: str
    reports_dir: Path
    seed: int = 42
    max_rounds: int = 3
    max_retries: int = 5
    run_ablation: bool = False
    ablation_seeds: list[int] | None = None
    repo_root: Path | None = None
    base_url: str = "https://api.x.ai/v1"
    # Conservative, never-silent gate on TOTAL xAI spend across all models (G2): a unit whose
    # estimated cost would breach this is refused before it is sent. Best-effort, not a formal
    # bound (heuristic token estimate; unbounded live completions). Default $10.
    max_usd: float = 10.0
    # Optional: task outcomes from a prior run, keyed by model id.  Tasks whose prior status
    # is STATUS_PASS are carried forward (marked resumed=True in output); all others are retried.
    # Outcomes for task ids not in the current task set are silently dropped.
    prior_outcomes: dict[str, list[dict[str, Any]]] = field(default_factory=dict)


def _paced_live_call(
    *,
    pacer: RatePacer,
    do_call: Callable[[], ChatResult],
    est_tokens_holder: dict[str, int],
    max_retries: int,
    log: logging.Logger,
) -> ChatResult:
    """Issue one paced live call with throttle-aware bounded retry.

    ``do_call`` performs the actual client.complete(). The pacer is acquired ONCE
    for this logical request (the first acquire reserves the RPM/TPM slot); on a
    throttle the bounded backoff sleep — routed through ``pacer.sleep`` so it also
    advances the pacer's clock and ages its windows — spaces the retry. We do NOT
    re-acquire per retry (that would double-count one logical request against RPM
    and could add a spurious extra wait). The token window is corrected to the real
    usage once a response arrives.
    """
    pacer.acquire(est_tokens_holder.get("est", 256))
    attempt = 0
    while True:
        chat = do_call()
        pacer.record_actual(chat.total_tokens or est_tokens_holder.get("est", 0))
        if chat.ok:
            return chat
        signal = classify_throttle(
            status_code=chat.status_code,
            retry_after=chat.retry_after,
            body=chat.error,
            attempt=attempt,
            max_attempts=max_retries,
        )
        if not signal.should_retry:
            return chat  # non-throttle error, or retries exhausted (never-silent)
        log.warning("live throttle: %s; sleeping %.2fs", signal.reason, signal.seconds)
        pacer.sleep(signal.seconds)
        attempt += 1


def run_live_model(
    *,
    model: ModelSpec,
    cfg: RunConfig,
    log: logging.Logger,
    budget: BudgetGuard | None = None,
) -> report_mod.ModelRunReport:
    """Run the full gold set (and optional ablation) for one model in live mode.

    When ``budget`` is given, each task — and the optional ablation block as a whole — is
    gated by a *conservative* cost estimate: if the estimate would push cumulative spend past
    the cap, the unit raises :class:`~grok.budget.BudgetExceeded` *before* it runs, and actual
    cost (gold-set per task, ablation summed from its per-sample tokens) is recorded into the
    guard as the run proceeds. This is a best-effort gate, **not** a formal upper bound: the
    token estimate is a heuristic and live completions are unbounded (no ``max_tokens``), so a
    single in-flight request can overrun — the gate biases high and stops *new* work early (G2).
    """
    client = OpenAICompatClient(base_url=cfg.base_url)
    pacer = RatePacer(rpm=model.rpm, tpm=model.tpm)
    scorer = MycCheckScorer(repo_root=cfg.repo_root, log=log)
    guarantee_tag = "Empirical"  # live model output (VR-5)

    # Resume: carry forward already-PASSed tasks from a prior run; retry the rest.
    # .get("task_id") avoids KeyError on malformed entries; None key is discarded.
    # Filter to the current task set so stale PASS outcomes are never silently carried.
    prior = {o.get("task_id"): o for o in cfg.prior_outcomes.get(model.id, [])}
    prior.pop(None, None)
    current_task_ids = {t.id for t in cfg.tasks}
    carried = {
        tid: o
        for tid, o in prior.items()
        if o.get("status") == STATUS_PASS and tid in current_task_ids
    }
    tasks_to_run = [t for t in cfg.tasks if t.id not in carried]
    if carried:
        log.info(
            "resume: model %s — carrying forward %d PASS(es), retrying %d task(s): %s",
            model.id,
            len(carried),
            len(tasks_to_run),
            [t.id for t in tasks_to_run],
        )

    # Seed outcomes with carried-forward PASSes (marked so the report is self-describing).
    outcomes: list[dict] = [dict(o, resumed=True) for o in carried.values()]
    scores: list[ScoreResult] = []
    edit_to_fix: list[int] = []
    latencies: list[float] = []
    total_prompt = total_completion = 0
    total_cost = 0.0
    request_count = 0
    est_holder: dict[str, int] = {"est": 256}

    for task in tasks_to_run:
        # Budget gate (G2): refuse to START a task whose conservative cost estimate
        # (max_rounds requests at the floored token estimate) would breach the cap.
        if budget is not None:
            est_tokens = max(est_holder["est"], CONSERVATIVE_TOKENS_PER_REQUEST)
            est_task_usd = (
                model.cost_usd(
                    prompt_tokens=est_tokens, completion_tokens=est_tokens, batch=False
                )
                * cfg.max_rounds
            )
            budget.check_or_raise(est_task_usd, model_id=model.id, log=log)

        # Wrap the loop's client.complete via hooks so each call is paced.
        def before(est: int) -> None:
            est_holder["est"] = est

        # The loop calls client.complete directly; to keep pacing centralised we
        # use a thin wrapper client that routes through _paced_live_call.
        paced_client = _PacedClient(
            client=client,
            pacer=pacer,
            est_holder=est_holder,
            max_retries=cfg.max_retries,
            log=log,
        )
        outcome = run_task_loop(
            task=task,
            model=model,
            client=paced_client,
            scorer=scorer,
            guarantee_tag=guarantee_tag,
            max_rounds=cfg.max_rounds,
            batch=False,
            before_request=before,
            after_request=None,
            seed=cfg.seed,
            log=log,
        )
        outcomes.append(outcome.to_dict())
        if outcome.final_score is not None:
            scores.append(outcome.final_score)
        if outcome.status in (STATUS_PASS, STATUS_PARTIAL_PASS):
            n = outcome.iterations_to_clean
            if n is not None:
                edit_to_fix.append(n)
        total_prompt += outcome.total_prompt_tokens
        total_completion += outcome.total_completion_tokens
        total_cost += outcome.total_cost_usd
        if budget is not None:
            budget.record(outcome.total_cost_usd)  # ACTUAL billed cost (VR-5)
        latencies.extend(r.chat.latency_s for r in outcome.rounds)
        request_count += len(outcome.rounds)

    metrics = aggregate_metrics(scores, edit_to_fix)
    perf = report_mod.build_performance(
        prompt_tokens=total_prompt,
        completion_tokens=total_completion,
        total_cost_usd=total_cost,
        latencies=latencies,
        request_count=request_count,
    )
    ablation_dict = None
    if cfg.run_ablation:
        # Gate the ablation block too (G2): estimate its cost (arms × seeds × tasks requests at
        # the floored token figure) and refuse to start it if that would breach the cap, so an
        # `--ablation` run cannot exceed the cap unaccounted-for. Actual cost is recorded after.
        if budget is not None:
            seeds = cfg.ablation_seeds or [11, 23, 42]
            n_req = len(default_arms()) * len(seeds) * len(cfg.tasks)
            est_abl_usd = model.cost_usd(
                prompt_tokens=CONSERVATIVE_TOKENS_PER_REQUEST * n_req,
                completion_tokens=CONSERVATIVE_TOKENS_PER_REQUEST * n_req,
                batch=False,
            )
            budget.check_or_raise(
                est_abl_usd, model_id=f"{model.id} (ablation)", log=log
            )
        ablation_dict = _run_ablation_live(
            model=model, cfg=cfg, client=client, pacer=pacer, scorer=scorer, log=log
        )
        if budget is not None:
            budget.record(
                _ablation_cost_usd(ablation_dict, model)
            )  # ACTUAL billed (VR-5)
    meta = report_mod.RunMetadata(
        model=model.id,
        mode="live",
        endpoint=cfg.base_url,
        task_set_id=cfg.task_set_id,
        seed=cfg.seed,
        max_rounds=cfg.max_rounds,
        synthetic=False,
    )
    return report_mod.ModelRunReport(
        metadata=meta,
        quality=metrics.to_dict(),
        performance=perf,
        outcomes=outcomes,
        ablation=ablation_dict,
    )


class _PacedClient:
    """Adapter that makes a live client's ``complete`` go through the pacer+retry."""

    def __init__(
        self,
        *,
        client: ChatClient,
        pacer: RatePacer,
        est_holder: dict[str, int],
        max_retries: int,
        log: logging.Logger,
    ) -> None:
        self._client = client
        self._pacer = pacer
        self._est_holder = est_holder
        self._max_retries = max_retries
        self._log = log

    def complete(self, *, model: str, messages: Any, **params: Any) -> ChatResult:
        return _paced_live_call(
            pacer=self._pacer,
            do_call=lambda: self._client.complete(
                model=model, messages=messages, **params
            ),
            est_tokens_holder=self._est_holder,
            max_retries=self._max_retries,
            log=self._log,
        )


def _run_ablation_live(*, model, cfg, client, pacer, scorer, log) -> dict[str, Any]:
    """Run the runnable ablation arms in live mode (paced)."""
    seeds = cfg.ablation_seeds or [11, 23, 42]
    est_holder = {"est": 256}
    paced = _PacedClient(
        client=client,
        pacer=pacer,
        est_holder=est_holder,
        max_retries=cfg.max_retries,
        log=log,
    )

    def before(est: int) -> None:
        est_holder["est"] = est

    arm_results = [
        run_arm(
            arm=arm,
            tasks=cfg.tasks,
            seeds=seeds,
            model=model,
            client=paced,
            scorer=scorer,
            guarantee_tag="Empirical",
            before_request=before,
            log=log,
        )
        for arm in default_arms()
    ]
    retention = compute_retention(arm_results)
    return AblationReport(
        model=model.id,
        task_set_id=cfg.task_set_id,
        seeds=seeds,
        arms=arm_results,
        retention=retention,
    ).to_dict()


def _ablation_cost_usd(ablation_dict: dict[str, Any], model: ModelSpec) -> float:
    """Sum the ACTUAL billed cost of an ablation run from its per-sample token records.

    Lets the budget guard account for ablation spend (live prices), so an ``--ablation`` run
    does not under-report against the cap (VR-5).
    """
    total = 0.0
    for arm in ablation_dict.get("arms", []):
        for sample in arm.get("per_sample", []):
            total += model.cost_usd(
                prompt_tokens=int(sample.get("prompt_tokens", 0) or 0),
                completion_tokens=int(sample.get("completion_tokens", 0) or 0),
                batch=False,
            )
    return total


def run_batch_model(
    *,
    model: ModelSpec,
    cfg: RunConfig,
    log: logging.Logger,
    budget: BudgetGuard | None = None,
) -> report_mod.ModelRunReport:
    """Run the first-pass generations for one model in batch mode (xai_sdk).

    LIVE-ONLY (needs xai_sdk + key + network). The iterative correction loop is not
    batchable and is intentionally skipped here (documented); batch mode measures
    first-pass pass@1 quality + batch-priced cost.

    When ``budget`` is given, the whole batch's conservative cost (one request per task at
    the floored token estimate, **batch-priced**) is gated *before submission*: a batch that
    would breach the cap raises :class:`~grok.budget.BudgetExceeded` (G2 — never submit work
    that would over-spend). Actual batch cost is recorded into the guard after collection.
    """
    client = XaiBatchClient()  # raises clear error if xai_sdk missing (G2)
    scorer = MycCheckScorer(repo_root=cfg.repo_root, log=log)
    requests, items = build_generation_requests(
        tasks=cfg.tasks, model=model, seed=cfg.seed
    )
    # Budget gate (G2): a batch is all-or-nothing once submitted, so estimate the whole
    # batch conservatively (one request/task at the floored token estimate, batch-priced) and
    # refuse to submit if it would breach the cap.
    if budget is not None:
        est_batch_usd = model.cost_usd(
            prompt_tokens=CONSERVATIVE_TOKENS_PER_REQUEST * len(requests),
            completion_tokens=CONSERVATIVE_TOKENS_PER_REQUEST * len(requests),
            batch=True,
        )
        budget.check_or_raise(est_batch_usd, model_id=model.id, log=log)
    results = submit_and_collect(
        client=client,
        batch_name=f"mycelium-gold-{model.id}-{cfg.seed}",
        requests=requests,
        poll=BatchPollConfig(),
        log=log,
    )
    scored = score_batch_results(
        items=items, results=results, model=model, scorer=scorer
    )
    scores = [s.score for s in scored]
    edit_to_fix = [1 for s in scored if s.score.verdict == VERDICT_CLEAN]  # pass@1
    metrics = aggregate_metrics(scores, edit_to_fix)
    total_prompt = sum(s.chat.prompt_tokens for s in scored)
    total_completion = sum(s.chat.completion_tokens for s in scored)
    total_cost = sum(s.cost_usd for s in scored)
    if budget is not None:
        budget.record(total_cost)  # ACTUAL batch-billed cost (VR-5)
    perf = report_mod.build_performance(
        prompt_tokens=total_prompt,
        completion_tokens=total_completion,
        total_cost_usd=total_cost,
        latencies=[],
        request_count=len(scored),
        batch_count=1,
    )
    meta = report_mod.RunMetadata(
        model=model.id,
        mode="batch",
        endpoint="xai_sdk batch",
        task_set_id=cfg.task_set_id,
        seed=cfg.seed,
        max_rounds=1,
        synthetic=False,
    )
    return report_mod.ModelRunReport(
        metadata=meta,
        quality=metrics.to_dict(),
        performance=perf,
        outcomes=[s.to_dict() for s in scored],
        ablation=None,
    )


def run(
    cfg: RunConfig, *, log: logging.Logger | None = None
) -> tuple[Path, list[Path]]:
    """Run every model in ``cfg`` (cheapest-first) and emit reports.

    Returns (comparison_markdown_path, [per_model_json_paths]).
    """
    log = log or _LOG
    run_id = report_mod.now_iso()
    reports: list[report_mod.ModelRunReport] = []
    json_paths: list[Path] = []
    # One guard for the whole run: the cap is TOTAL xAI spend, not per-model (G2).
    budget = BudgetGuard(cap_usd=cfg.max_usd)
    log.info(
        "spend cap: $%.2f (total across all models; conservative gate)", cfg.max_usd
    )
    first_refusal: BudgetExceeded | None = None
    for model in cfg.models:
        log.info(
            "=== model %s (mode=%s) — %s ===", model.id, cfg.mode, budget.summary()
        )
        try:
            if cfg.mode == "batch":
                mr = run_batch_model(model=model, cfg=cfg, log=log, budget=budget)
            else:
                mr = run_live_model(model=model, cfg=cfg, log=log, budget=budget)
        except BudgetExceeded as exc:
            # Never-silent: stop the run with whatever completed, honestly flagged. The
            # remaining (pricier) models are skipped — the gate refuses further work.
            first_refusal = exc
            log.warning(
                "STOPPING the run at the spend cap before model %s: %s", model.id, exc
            )
            break
        reports.append(mr)
        json_paths.append(
            report_mod.write_model_json(mr, reports_dir=cfg.reports_dir, run_id=run_id)
        )
    log.info("run complete — %s", budget.summary())
    if not reports:
        # The very first unit breached the cap (e.g. an absurdly low --max-usd): emit nothing
        # rather than a misleading empty comparison, and re-raise the ORIGINAL refusal so its
        # estimate + model id are preserved (G2 — the message stays informative).
        raise first_refusal or BudgetExceeded(
            spent_usd=budget.spent_usd,
            est_usd=0.0,
            cap_usd=budget.cap_usd,
            model_id="(none ran)",
        )
    md_path = report_mod.write_comparison_markdown(
        reports,
        reports_dir=cfg.reports_dir,
        run_id=run_id,
        mode=cfg.mode,
        synthetic=False,
    )
    return md_path, json_paths
