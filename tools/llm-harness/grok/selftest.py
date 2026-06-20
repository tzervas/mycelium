"""Offline, deterministic self-test — the green gate (no network, no API key).

The brief is explicit: no API key exists here, so the live experiment cannot run
and its metrics must NOT be fabricated. What CAN be verified offline is the
*plumbing* — and this module verifies it deterministically against a mocked client
and an injected scorer, exercising exactly the load-bearing math:

  T1  model ordering: cheapest-first sort + --models/--order overrides.
  T2  RPM pacing math: the sliding-window wait is computed correctly.
  T3  TPM pacing math: the token-window wait is computed correctly.
  T4  backoff + throttle classification: exponential/Retry-After, bounded retry.
  T5  cost accounting: live uses sync prices, batch uses batch prices — and the
      two differ when the rubric sets a discount (we inject one to prove it).
  T6  scoring classifier: myc-check exit codes -> syntactic/type/clean verdicts,
      and the aggregate KC-2 rates (skips excluded from denominators).
  T7  M-330 loop: generate->fix reaches PARTIAL_PASS via a self-correcting mock,
      and an empty/failed generation is never a PASS (G2).
  T8  M-381 ablation: pass@1 per runnable arm; retention ratio is INDETERMINATE
      while arm 4 is blocked (verdict never pre-written, VR-5).
  T9  report emission: per-model JSON + cross-model markdown are written and are
      stamped SYNTHETIC (self-test).
  T13 API discovery: from_api_discovery() maps a mock GET /v1/models response to
      ModelSpec objects with correct Declared conservative defaults; bad/missing
      pricing entries are skipped with a warning; all-bad input is a G2 error.

Each check returns a (name, ok, detail) triple; ``run_self_test`` prints them and
returns process exit 0 iff all pass. Plumbing verified this way is **Empirical**
(self-test-evidenced); the model-quality / leverage verdict stays **open**.
"""

from __future__ import annotations

import json
import math
import tempfile
from collections import deque
from dataclasses import dataclass
from pathlib import Path

from . import report as report_mod
from .budget import BudgetExceeded, BudgetGuard
from .ablation import (
    ARM_BARE,
    ARM_CANONICAL,
    AblationReport,
    compute_retention,
    default_arms,
    run_arm,
)
from .client import MockClient, MockScript
from .coauthor_loop import (
    STATUS_PARTIAL_PASS,
    run_task_loop,
)
from .models import ModelSpec, from_api_discovery, load_models, order_models
from .ratelimit import (
    RatePacer,
    backoff_seconds,
    classify_throttle,
    parse_retry_after,
    rpm_wait_seconds,
    tpm_wait_seconds,
)
from .scoring import (
    EXIT_CHECK_ERROR,
    EXIT_CLEAN,
    EXIT_PARSE_ERROR,
    VERDICT_CLEAN,
    VERDICT_SYNTAX_ERROR,
    VERDICT_TYPE_ERROR,
    MycCheckScorer,
    aggregate_metrics,
)
from .tasks import Task


@dataclass
class Check:
    name: str
    ok: bool
    detail: str


# -- helpers: deterministic fixtures -----------------------------------------


def _spec(
    id_: str,
    out_price: float,
    in_price: float,
    order: int,
    *,
    batch_in: float | None = None,
    batch_out: float | None = None,
    rpm: int = 1000,
    tpm: int = 1_000_000,
) -> ModelSpec:
    return ModelSpec(
        id=id_,
        context=128000,
        tpm=tpm,
        rpm=rpm,
        in_price=in_price,
        out_price=out_price,
        batch_in_price=in_price if batch_in is None else batch_in,
        batch_out_price=out_price if batch_out is None else batch_out,
        config_order=order,
    )


def _fake_scorer(verdict_by_call: list[int]) -> MycCheckScorer:
    """A scorer whose injected runner returns scripted exit codes per call."""
    calls = {"i": 0}

    def runner(_src: str) -> tuple[int, str, str]:
        i = calls["i"]
        calls["i"] += 1
        code = verdict_by_call[min(i, len(verdict_by_call) - 1)]
        stderr = "" if code == EXIT_CLEAN else f"diagnostic for exit {code}"
        return code, "", stderr

    return MycCheckScorer(runner=runner)


# -- checks ------------------------------------------------------------------


def check_model_ordering() -> Check:
    specs = [
        _spec("pricey", out_price=5.0, in_price=2.0, order=0),
        _spec("cheap", out_price=1.0, in_price=1.0, order=1),
        _spec("mid", out_price=2.0, in_price=9.0, order=2),
        _spec("mid-tie", out_price=2.0, in_price=1.0, order=3),
    ]
    # cheapest-first: out_price asc, then in_price asc, then config order.
    ordered = [m.id for m in order_models(specs)]
    expected = ["cheap", "mid-tie", "mid", "pricey"]
    if ordered != expected:
        return Check("T1 model-ordering", False, f"got {ordered}, want {expected}")
    # --models subset stays cheapest-first.
    sub = [m.id for m in order_models(specs, select=["pricey", "cheap"])]
    if sub != ["cheap", "pricey"]:
        return Check("T1 model-ordering", False, f"subset wrong: {sub}")
    # --order forces explicit order (not cheapest-first).
    forced = [m.id for m in order_models(specs, order=["pricey", "cheap", "mid"])]
    if forced != ["pricey", "cheap", "mid"]:
        return Check("T1 model-ordering", False, f"order override wrong: {forced}")
    return Check("T1 model-ordering", True, f"cheapest-first={ordered}; overrides ok")


def check_rpm_math() -> Check:
    # rpm=3; three requests already at t=0,10,20; window=60. now=30 -> full.
    win = deque([0.0, 10.0, 20.0])
    wait = rpm_wait_seconds(win, 3, now=30.0)
    # oldest (0.0) leaves window at t=60 -> wait 30.
    if abs(wait - 30.0) > 1e-9:
        return Check("T2 rpm-math", False, f"expected 30.0, got {wait}")
    # room when under budget.
    if rpm_wait_seconds(deque([0.0, 10.0]), 3, now=30.0) != 0.0:
        return Check("T2 rpm-math", False, "should be 0 when under budget")
    return Check("T2 rpm-math", True, "sliding-window RPM wait correct (30.0s)")


def check_tpm_math() -> Check:
    # tpm=100; window has (t=0,80) and (t=30,15) => used=95. incoming=10 -> over by 5.
    events = deque([(0.0, 80), (30.0, 15)])
    wait = tpm_wait_seconds(events, 100, 10, now=50.0)
    # Draining oldest (80) frees >=5; it leaves window at t=60 -> wait 10.
    if abs(wait - 10.0) > 1e-9:
        return Check("T3 tpm-math", False, f"expected 10.0, got {wait}")
    # Fits now when there is room.
    if tpm_wait_seconds(deque([(0.0, 10)]), 100, 10, now=5.0) != 0.0:
        return Check("T3 tpm-math", False, "should be 0 when room exists")
    # Over-budget single request never deadlocks (returns 0).
    if tpm_wait_seconds(deque(), 100, 200, now=0.0) != 0.0:
        return Check("T3 tpm-math", False, "over-budget single req must return 0")
    return Check("T3 tpm-math", True, "sliding-window TPM wait correct (10.0s)")


def check_backoff_and_throttle() -> Check:
    # exponential, capped.
    if backoff_seconds(0, base=1.0, cap=60.0) != 1.0:
        return Check("T4 backoff", False, "attempt0 != base")
    if backoff_seconds(3, base=1.0, cap=60.0) != 8.0:
        return Check("T4 backoff", False, "attempt3 != 8")
    if backoff_seconds(10, base=1.0, cap=60.0) != 60.0:
        return Check("T4 backoff", False, "cap not applied")
    # Retry-After honoured (capped).
    if parse_retry_after("Retry-After: 12") != 12.0:
        return Check("T4 backoff", False, "retry-after parse failed")
    s = classify_throttle(
        status_code=429,
        retry_after="30",
        body="rate limit",
        attempt=0,
        max_attempts=3,
        cap=20.0,
    )
    if not (s.should_retry and abs(s.seconds - 20.0) < 1e-9):
        return Check("T4 backoff", False, f"429 retry-after cap wrong: {s}")
    # retries exhausted -> no retry (bounded, never infinite).
    s2 = classify_throttle(
        status_code=429, retry_after=None, body="", attempt=2, max_attempts=3
    )
    if s2.should_retry:
        return Check("T4 backoff", False, "should stop after max attempts")
    # non-throttle error -> no retry.
    s3 = classify_throttle(
        status_code=400, retry_after=None, body="bad request", attempt=0, max_attempts=3
    )
    if s3.should_retry:
        return Check("T4 backoff", False, "400 must not be retried as throttle")
    return Check("T4 backoff", True, "exp backoff, Retry-After cap, bounded retry ok")


def check_cost_accounting() -> Check:
    # Inject a real batch discount to prove the mode->price coupling.
    m = _spec(
        "disc",
        out_price=2.0,
        in_price=1.0,
        order=0,
        batch_in=0.5,
        batch_out=1.0,
    )
    live = m.cost_usd(prompt_tokens=1_000_000, completion_tokens=1_000_000, batch=False)
    batch = m.cost_usd(prompt_tokens=1_000_000, completion_tokens=1_000_000, batch=True)
    # live: 1*1.0 + 1*2.0 = 3.0 ; batch: 1*0.5 + 1*1.0 = 1.5
    if abs(live - 3.0) > 1e-9 or abs(batch - 1.5) > 1e-9:
        return Check(
            "T5 cost", False, f"live={live} (want 3.0) batch={batch} (want 1.5)"
        )
    if not batch < live:
        return Check("T5 cost", False, "batch price should be < sync under a discount")
    # Default seed rubric (batch==sync): they must be EQUAL (no invented discount).
    m2 = _spec("nodisc", out_price=2.5, in_price=1.25, order=0)
    l2 = m2.cost_usd(prompt_tokens=500_000, completion_tokens=500_000, batch=False)
    b2 = m2.cost_usd(prompt_tokens=500_000, completion_tokens=500_000, batch=True)
    if abs(l2 - b2) > 1e-9:
        return Check("T5 cost", False, "batch==sync rubric must give equal cost")
    return Check(
        "T5 cost",
        True,
        f"live=${live} vs batch=${batch} (discount honoured); equal when rubric ties",
    )


def check_scoring_classifier() -> Check:
    scorer = _fake_scorer([EXIT_CLEAN, EXIT_PARSE_ERROR, EXIT_CHECK_ERROR])
    r_clean = scorer.score("nodule a\nfn f()=x")
    r_parse = scorer.score("garbage")
    r_check = scorer.score("nodule a\nfn f(x: Binary{8}) -> Ternary{6} = x")
    if not (r_clean.verdict == VERDICT_CLEAN and r_clean.typecheck_pass):
        return Check("T6 scoring", False, f"clean wrong: {r_clean}")
    if not (r_parse.verdict == VERDICT_SYNTAX_ERROR and not r_parse.syntactic_valid):
        return Check("T6 scoring", False, f"parse wrong: {r_parse}")
    if not (
        r_check.verdict == VERDICT_TYPE_ERROR
        and r_check.syntactic_valid
        and not r_check.typecheck_pass
    ):
        return Check("T6 scoring", False, f"check wrong: {r_check}")
    # Empty source is never clean.
    if scorer.score("   ").verdict == VERDICT_CLEAN:
        return Check("T6 scoring", False, "empty must not be clean")
    # Aggregate rates: skips excluded from denominator (real SKIP mechanism).
    skip = _skip_scorer()
    agg = aggregate_metrics(
        [r_clean, r_parse, r_check, skip.score("x")], edit_to_fix=[1]
    )
    # scored = 3 (skip excluded); syntactic_valid = clean+check = 2; typecheck = 1.
    if agg.scored != 3 or agg.syntactic_valid != 2 or agg.typecheck_pass != 1:
        return Check("T6 scoring", False, f"aggregate wrong: {agg.to_dict()}")
    if abs((agg.typecheck_pass_rate or 0) - (1 / 3)) > 1e-9:
        return Check("T6 scoring", False, "typecheck rate denominator excludes skips")
    return Check("T6 scoring", True, "exit-code classify + KC-2 rates (skips excluded)")


def _skip_scorer() -> MycCheckScorer:
    from .scoring import BackendScorerError

    def runner(_s: str) -> tuple[int, str, str]:
        raise BackendScorerError("simulated: cargo absent")

    return MycCheckScorer(runner=runner)


def check_coauthor_loop() -> Check:
    # A self-correcting mock: broken first, fixed once feedback appears.
    scripts = [
        MockScript(
            content="bad source",  # round 1: not clean (scorer returns check-error)
            prompt_tokens=50,
            completion_tokens=20,
            corrected_content="nodule fixed\nfn f(x: Binary{8}) -> Binary{8} = x",
            corrected_prompt_tokens=70,
            corrected_completion_tokens=30,
        )
    ]
    client = MockClient(scripts)
    scorer = _fake_scorer([EXIT_CHECK_ERROR, EXIT_CLEAN])  # fail then pass
    model = _spec("m", out_price=2.0, in_price=1.0, order=0)
    task = Task("t1", "identity over Binary{8}")
    outcome = run_task_loop(
        task=task,
        model=model,
        client=client,
        scorer=scorer,
        guarantee_tag="Declared",
        max_rounds=3,
    )
    if outcome.status != STATUS_PARTIAL_PASS:
        return Check("T7 loop", False, f"expected PARTIAL_PASS, got {outcome.status}")
    if outcome.iterations_to_clean != 2:
        return Check(
            "T7 loop", False, f"iterations_to_clean != 2: {outcome.iterations_to_clean}"
        )
    if outcome.total_completion_tokens != 50:  # 20 + 30
        return Check(
            "T7 loop", False, f"token total wrong: {outcome.total_completion_tokens}"
        )
    # cost: (50+20 then 70+30) tokens at 1.0/2.0 per Mtok.
    # r1: 50/1e6*1.0 + 20/1e6*2.0 ; r2: 70/1e6*1.0 + 30/1e6*2.0
    exp_cost = (50e-6 * 1.0 + 20e-6 * 2.0) + (70e-6 * 1.0 + 30e-6 * 2.0)
    if abs(outcome.total_cost_usd - exp_cost) > 1e-12:
        return Check("T7 loop", False, f"cost {outcome.total_cost_usd} != {exp_cost}")
    # Empty generation is never a PASS (G2): a mock that returns nothing scorable.
    empty_client = MockClient(
        [MockScript(content="", prompt_tokens=1, completion_tokens=0)]
    )
    empty_scorer = _fake_scorer([EXIT_CLEAN])  # even if scorer would say clean,
    out2 = run_task_loop(
        task=task,
        model=model,
        client=empty_client,
        scorer=empty_scorer,
        guarantee_tag="Declared",
        max_rounds=2,
    )
    # empty content -> scorer.score("") returns error verdict -> not clean -> FAIL.
    if out2.status == "PASS":
        return Check("T7 loop", False, "empty generation must never be PASS")
    return Check(
        "T7 loop", True, "generate->fix PARTIAL_PASS; cost+iters exact; empty!=PASS"
    )


def check_ablation() -> Check:
    client = MockClient()  # default echo; arm1/arm2 runnable
    # Scorer: make every generation clean so pass@1 is well-defined and non-trivial.
    scorer = _fake_scorer([EXIT_CLEAN])
    model = _spec("m", out_price=2.0, in_price=1.0, order=0)
    tasks = [Task("t1", "a"), Task("t2", "b")]
    seeds = [1, 2, 3]
    arm_results = [
        run_arm(
            arm=arm,
            tasks=tasks,
            seeds=seeds,
            model=model,
            client=client,
            scorer=scorer,
            guarantee_tag="Declared",
        )
        for arm in default_arms()
    ]
    by = {a.arm_id: a for a in arm_results}
    bare = by[ARM_BARE]
    if not (bare.ran and bare.n_samples == 6 and bare.pass_at_1 == 1.0):
        return Check("T8 ablation", False, f"arm1 wrong: {bare.to_dict()}")
    canonical = by[ARM_CANONICAL]
    if canonical.ran:
        return Check("T8 ablation", False, "arm4 must be BLOCKED (not fabricated)")
    verdict = compute_retention(arm_results)
    if verdict.determinate or verdict.ratio is not None:
        return Check("T8 ablation", False, "retention must be INDETERMINATE w/o arm4")
    if "INDETERMINATE" not in verdict.conclusion:
        return Check("T8 ablation", False, f"verdict not honest: {verdict.conclusion}")
    if verdict.leverage_claim_tag != "Declared":
        return Check("T8 ablation", False, "leverage claim must stay Declared/open")
    return Check(
        "T8 ablation",
        True,
        "arm1/2 pass@1 computed; arm4 blocked; retention INDETERMINATE (verdict open)",
    )


def check_live_status_guard() -> Check:
    """A non-2xx returned as a TUPLE (not raised) must yield ok=False (never-silent).

    Exercises OpenAICompatClient with a fake opener that returns a 503 + Retry-After
    as a normal response — the path a custom opener / 200-with-error-envelope could
    hit. It must be classified as a failure with status + Retry-After preserved.
    """
    from .client import ChatMessage, OpenAICompatClient

    class _FakeOpener503:
        def open(self, req, timeout=None):  # noqa: ARG002 - signature match
            return 503, b'{"error":"overloaded"}', {"Retry-After": "7"}

    client = OpenAICompatClient(api_key="dummy", opener=_FakeOpener503())
    r = client.complete(model="m", messages=[ChatMessage("user", "hi")])
    if r.ok:
        return Check("T10 live-guard", False, "non-2xx tuple must be ok=False")
    if r.status_code != 503:
        return Check("T10 live-guard", False, f"status not preserved: {r.status_code}")
    if r.retry_after != "7":
        return Check("T10 live-guard", False, f"Retry-After lost: {r.retry_after!r}")
    # And a clean 200 still parses to ok=True with usage.
    import json as _json

    class _FakeOpenerOK:
        def open(self, req, timeout=None):  # noqa: ARG002
            body = {
                "model": "m",
                "choices": [{"message": {"content": "nodule a\n"}}],
                "usage": {"prompt_tokens": 5, "completion_tokens": 3},
            }
            return 200, _json.dumps(body).encode(), {}

    ok = OpenAICompatClient(api_key="dummy", opener=_FakeOpenerOK()).complete(
        model="m", messages=[ChatMessage("user", "hi")]
    )
    if not (ok.ok and ok.prompt_tokens == 5 and ok.completion_tokens == 3):
        return Check("T10 live-guard", False, f"200 parse wrong: {ok}")
    return Check(
        "T10 live-guard",
        True,
        "non-2xx tuple -> ok=False (status+Retry-After kept); 200 parses with usage",
    )


def check_paced_retry_acquires_once() -> Check:
    """The paced live call acquires the pacer ONCE per logical request, not per retry.

    A throttled-then-OK sequence must record exactly ONE RPM slot (no double-count),
    while still issuing the retry after the backoff sleep.
    """
    from .client import ChatResult
    from .runner import _paced_live_call

    acquires = {"n": 0}
    slept: list[float] = []

    class _CountingPacer:
        def acquire(self, est: int) -> float:
            acquires["n"] += 1
            return 0.0

        def record_actual(self, n: int) -> None:
            pass

        def sleep(self, s: float) -> None:
            slept.append(s)

    seq = [
        ChatResult(
            ok=False,
            content="",
            prompt_tokens=0,
            completion_tokens=0,
            latency_s=0.0,
            model="m",
            status_code=429,
            retry_after="2",
            error="rate limit",
        ),
        ChatResult(
            ok=True,
            content="ok",
            prompt_tokens=5,
            completion_tokens=2,
            latency_s=0.0,
            model="m",
        ),
    ]
    calls = {"i": 0}

    def do_call() -> ChatResult:
        r = seq[min(calls["i"], len(seq) - 1)]
        calls["i"] += 1
        return r

    out = _paced_live_call(
        pacer=_CountingPacer(),
        do_call=do_call,
        est_tokens_holder={"est": 100},
        max_retries=3,
        log=__import__("logging").getLogger("selftest"),
    )
    if not out.ok:
        return Check("T11 paced-retry", False, "should have succeeded on retry")
    if acquires["n"] != 1:
        return Check(
            "T11 paced-retry",
            False,
            f"acquired {acquires['n']} times; must be exactly 1 per logical request",
        )
    if calls["i"] != 2:
        return Check("T11 paced-retry", False, f"expected 2 calls, got {calls['i']}")
    if slept != [2.0]:
        return Check("T11 paced-retry", False, f"backoff sleep wrong: {slept}")
    return Check(
        "T11 paced-retry",
        True,
        "1 acquire across a 429->OK retry; backoff slept Retry-After=2s",
    )


def check_report_emission() -> Check:
    with tempfile.TemporaryDirectory() as td:
        reports_dir = Path(td)
        meta = report_mod.RunMetadata(
            model="grok-test",
            mode="self-test",
            endpoint="mock",
            task_set_id="gold-compose-v1",
            seed=42,
            max_rounds=3,
            synthetic=True,
        )
        scorer = _fake_scorer([EXIT_CLEAN, EXIT_PARSE_ERROR])
        agg = aggregate_metrics([scorer.score("a"), scorer.score("b")], edit_to_fix=[1])
        perf = report_mod.build_performance(
            prompt_tokens=100,
            completion_tokens=50,
            total_cost_usd=0.00015,
            latencies=[0.0, 0.0],
            request_count=2,
        )
        mr = report_mod.ModelRunReport(
            metadata=meta, quality=agg.to_dict(), performance=perf, outcomes=[]
        )
        jp = report_mod.write_model_json(mr, reports_dir=reports_dir, run_id="TST")
        md = report_mod.write_comparison_markdown(
            [mr],
            reports_dir=reports_dir,
            run_id="TST",
            mode="self-test",
            synthetic=True,
        )
        if not jp.exists() or not md.exists():
            return Check("T9 report", False, "report files not written")
        if "SYNTHETIC" not in jp.name or "SYNTHETIC" not in md.name:
            return Check("T9 report", False, "synthetic reports must be name-stamped")
        data = json.loads(jp.read_text())
        if not data["honesty_posture"]["synthetic"]:
            return Check("T9 report", False, "synthetic flag missing in JSON")
        if "Proven" in data["honesty_posture"]["model_allowed_tags"]:
            return Check("T9 report", False, "model-allowed tags must exclude Proven")
        md_text = md.read_text()
        if "SYNTHETIC (self-test)" not in md_text:
            return Check("T9 report", False, "markdown missing synthetic banner")
    return Check(
        "T9 report", True, "per-model JSON + markdown emitted, SYNTHETIC-stamped"
    )


def check_rubric_loads() -> Check:
    """The bundled models.toml parses, orders cheapest-first, and has ≥2 models."""
    from .models import default_models_path

    path = default_models_path()
    if not path.exists():
        return Check("T0 rubric", False, f"models.toml not found at {path}")
    specs = load_models(path)
    if len(specs) < 2:
        return Check("T0 rubric", False, f"expected ≥2 seed models, got {len(specs)}")
    ordered = order_models(specs)
    prices = [m.out_price for m in ordered]
    if prices != sorted(prices):
        return Check("T0 rubric", False, f"not cheapest-first: {prices}")
    # Seed rubric uses batch==sync (no invented discount) — assert it holds.
    for m in specs:
        if m.batch_in_price != m.in_price or m.batch_out_price != m.out_price:
            return Check(
                "T0 rubric",
                False,
                f"{m.id}: seed batch price should equal sync (no invented discount)",
            )
    return Check(
        "T0 rubric",
        True,
        f"models.toml: {len(specs)} models, cheapest-first, batch==sync",
    )


def _live_pacer_smoke() -> Check:
    """RatePacer with a virtual clock: acquire never sleeps under budget, then paces."""
    t = {"now": 0.0}
    slept: list[float] = []

    pacer = RatePacer(
        rpm=2,
        tpm=1_000_000,
        now=lambda: t["now"],
        sleep=lambda s: (slept.append(s), t.__setitem__("now", t["now"] + s)),
    )
    pacer.acquire(100)  # req1 @ t=0
    pacer.acquire(100)  # req2 @ t=0
    # 3rd request must wait until the oldest (t=0) leaves the 60s window.
    waited = pacer.acquire(100)  # req3
    if abs(waited - 60.0) > 1e-9:
        return Check("T2b pacer", False, f"3rd acquire should wait 60s, got {waited}")
    return Check(
        "T2b pacer", True, "RatePacer paces 3rd request by 60s (virtual clock)"
    )


def check_budget_cap() -> Check:
    # The USD spend gate (G2): would_exceed / record / check_or_raise behave as a cumulative
    # ceiling, a breaching estimate is REFUSED (raises), and non-finite cap/estimate cannot
    # silently disable the gate.
    g = BudgetGuard(cap_usd=1.0)
    if g.would_exceed(0.5) or not g.would_exceed(1.5):
        return Check("T12 budget-cap", False, "would_exceed wrong at spent=0")
    g.record(0.6)  # actual spend
    if g.would_exceed(0.3):  # 0.6 + 0.3 = 0.9 <= 1.0
        return Check("T12 budget-cap", False, "0.9 should be within the $1.00 cap")
    if not g.would_exceed(0.5):  # 0.6 + 0.5 = 1.1 > 1.0
        return Check("T12 budget-cap", False, "1.1 should exceed the $1.00 cap")
    # The breaching unit is refused (raised), and the guard is flagged stopped-at-cap.
    raised = False
    try:
        g.check_or_raise(0.5, model_id="m")
    except BudgetExceeded as exc:
        raised = True
        if abs(exc.spent_usd - 0.6) > 1e-9 or abs(exc.cap_usd - 1.0) > 1e-9:
            return Check(
                "T12 budget-cap", False, f"exception carries wrong numbers: {exc}"
            )
    if not raised or not g.refused:
        return Check(
            "T12 budget-cap", False, "a breaching estimate must be refused (raise)"
        )
    # A negative or non-finite cap is rejected at construction (the gate cannot be disabled).
    for bad in (-1.0, math.inf, math.nan):
        try:
            BudgetGuard(cap_usd=bad)
            return Check("T12 budget-cap", False, f"cap {bad!r} must be rejected")
        except ValueError:
            pass
    # A non-finite ESTIMATE is an automatic exceed (a NaN comparison would otherwise slip past).
    fresh = BudgetGuard(cap_usd=5.0)
    if not fresh.would_exceed(math.inf) or not fresh.would_exceed(math.nan):
        return Check("T12 budget-cap", False, "a non-finite estimate must auto-exceed")
    return Check(
        "T12 budget-cap",
        True,
        "USD gate: cumulative ceiling, refusal of a breaching unit, nan/inf rejected (G2)",
    )


def check_api_discovery() -> Check:
    """T13: from_api_discovery() converts a mock GET /v1/models response correctly.

    Verifies: valid entry → ModelSpec with Declared defaults; entry missing pricing
    skipped; negative pricing skipped; duplicate id skipped; batch prices = sync prices
    (no invented discount); empty input → ModelConfigError (G2).
    """
    from .models import ModelConfigError

    good = {
        "id": "grok-test-1",
        "context_length": 65536,
        "pricing": {"input": 1.0, "output": 2.0},
    }
    no_pricing = {"id": "skip-no-price"}
    bad_pricing = {"id": "skip-neg", "pricing": {"input": -1.0, "output": 2.0}}
    duplicate = {
        "id": "grok-test-1",
        "context_length": 8192,
        "pricing": {"input": 0.5, "output": 1.0},
    }
    no_id = {"context_length": 1024, "pricing": {"input": 0.1, "output": 0.2}}

    # Case 1: valid entry produces a ModelSpec with correct values and Declared defaults.
    specs = from_api_discovery([good])
    if len(specs) != 1:
        return Check("T13 api-discovery", False, f"expected 1 spec, got {len(specs)}")
    s = specs[0]
    if s.id != "grok-test-1":
        return Check("T13 api-discovery", False, f"wrong id: {s.id!r}")
    if s.context != 65536:
        return Check("T13 api-discovery", False, f"wrong context: {s.context}")
    if s.in_price != 1.0 or s.out_price != 2.0:
        return Check(
            "T13 api-discovery", False, f"wrong sync prices: {s.in_price}/{s.out_price}"
        )
    # batch prices must equal sync (no invented discount)
    if s.batch_in_price != s.in_price or s.batch_out_price != s.out_price:
        return Check(
            "T13 api-discovery",
            False,
            f"batch prices must equal sync (no invented discount): {s.batch_in_price}/{s.batch_out_price}",
        )
    # Declared conservative defaults for rate limits
    if s.rpm != 60 or s.tpm != 2_000_000:
        return Check(
            "T13 api-discovery",
            False,
            f"expected Declared defaults rpm=60 tpm=2000000, got rpm={s.rpm} tpm={s.tpm}",
        )

    # Case 2: mixed list — bad entries are skipped, only valid one survives.
    mixed = [no_pricing, bad_pricing, no_id, good, duplicate]
    specs2 = from_api_discovery(mixed)
    if len(specs2) != 1 or specs2[0].id != "grok-test-1":
        return Check(
            "T13 api-discovery",
            False,
            f"mixed list: expected 1 valid spec, got {len(specs2)}",
        )

    # Case 3: missing context_length falls back to Declared default (131072).
    no_ctx = {"id": "grok-noctx", "pricing": {"input": 0.5, "output": 1.0}}
    specs3 = from_api_discovery([no_ctx])
    if not specs3 or specs3[0].context != 131_072:
        return Check(
            "T13 api-discovery",
            False,
            f"missing context_length should default to 131072, got {specs3[0].context if specs3 else 'empty'}",
        )

    # Case 4: all-bad input raises ModelConfigError (G2 — never-silent).
    raised = False
    try:
        from_api_discovery([no_pricing, bad_pricing, no_id])
    except ModelConfigError:
        raised = True
    if not raised:
        return Check(
            "T13 api-discovery",
            False,
            "all-bad input must raise ModelConfigError (G2), not return empty list",
        )

    return Check(
        "T13 api-discovery",
        True,
        "from_api_discovery: valid→ModelSpec, skips no-pricing/neg/dup, Declared defaults, all-bad→error (G2)",
    )


# -- driver ------------------------------------------------------------------

ALL_CHECKS = [
    check_rubric_loads,
    check_model_ordering,
    check_rpm_math,
    _live_pacer_smoke,
    check_tpm_math,
    check_backoff_and_throttle,
    check_cost_accounting,
    check_scoring_classifier,
    check_coauthor_loop,
    check_ablation,
    check_live_status_guard,
    check_paced_retry_acquires_once,
    check_report_emission,
    check_budget_cap,
    check_api_discovery,
]


def run_self_test(
    *, emit_sample: bool = False, reports_dir: Path | None = None, verbose: bool = False
) -> int:
    """Run all offline checks; print results; return 0 iff all pass.

    With ``emit_sample`` also writes a SYNTHETIC sample report pair into
    ``reports_dir`` (default: the package's ``reports/``) so the committed example
    is reproducible and clearly labelled.
    """
    results: list[Check] = []
    for fn in ALL_CHECKS:
        try:
            results.append(fn())
        except Exception as exc:  # never-silent: a crashing check is a failure
            results.append(Check(fn.__name__, False, f"raised: {exc!r}"))

    width = max(len(c.name) for c in results)
    print("Mycelium Grok harness — offline self-test")
    print("=" * 60)
    for c in results:
        mark = "PASS" if c.ok else "FAIL"
        line = f"[{mark}] {c.name.ljust(width)}  {c.detail}"
        print(line)
    n_ok = sum(1 for c in results if c.ok)
    n = len(results)
    print("=" * 60)
    print(f"{n_ok}/{n} checks passed")
    print(
        "Honesty: plumbing verified offline is EMPIRICAL (self-test-evidenced). "
        "Live model-quality / retention verdict stays OPEN (Declared, pending run)."
    )

    if emit_sample and n_ok == n:
        rd = reports_dir or (Path(__file__).resolve().parent.parent / "reports")
        _emit_sample_report(rd, verbose=verbose)

    return 0 if n_ok == n else 1


def _emit_sample_report(reports_dir: Path, *, verbose: bool) -> None:
    """Emit a deterministic SYNTHETIC sample report pair (committed reference)."""
    model = _spec("grok-4.3", out_price=2.5, in_price=1.25, order=0)
    # Per-task (client, scorer, script) so call-indexing is per-program and the
    # committed sample is fully deterministic and order-independent.
    plan = [
        (
            Task("g01", "identity"),
            [EXIT_CLEAN],
            MockScript("nodule a\nfn id(x: Binary{8}) -> Binary{8} = x", 60, 25),
        ),
        (
            Task("g02", "double"),
            [EXIT_CHECK_ERROR, EXIT_CLEAN],
            MockScript(
                "broken",
                55,
                18,
                corrected_content="nodule b\nfn dbl(x: Ternary{6}) -> Ternary{6} = add(x, x)",
                corrected_prompt_tokens=80,
                corrected_completion_tokens=30,
            ),
        ),
        (
            Task("g04", "widen"),
            [EXIT_CLEAN],
            MockScript(
                "nodule c\nfn widen(x: Binary{8}) -> Ternary{6} = "
                "swap(x, to: Ternary{6}, policy: roundtrip)",
                70,
                40,
            ),
        ),
    ]
    tasks = [t for (t, _, _) in plan]
    outcomes = []
    scores = []
    edit_to_fix = []
    tp = tc = 0
    cost = 0.0
    lat = []
    for t, exit_codes, script in plan:
        out = run_task_loop(
            task=t,
            model=model,
            client=MockClient([script]),
            scorer=_fake_scorer(exit_codes),
            guarantee_tag="Declared",
            max_rounds=3,
        )
        outcomes.append(out.to_dict())
        if out.final_score:
            scores.append(out.final_score)
        if out.iterations_to_clean:
            edit_to_fix.append(out.iterations_to_clean)
        tp += out.total_prompt_tokens
        tc += out.total_completion_tokens
        cost += out.total_cost_usd
        lat.extend(r.chat.latency_s for r in out.rounds)
    # A blocked-arm ablation snapshot (honest: arm4 blocked => indeterminate).
    arms = [
        run_arm(
            arm=a,
            tasks=tasks,
            seeds=[1, 2, 3],
            model=model,
            client=MockClient(),
            scorer=_fake_scorer([EXIT_CLEAN]),
            guarantee_tag="Declared",
        )
        for a in default_arms()
    ]
    ablation = AblationReport(
        model=model.id,
        task_set_id="gold-compose-v1",
        seeds=[1, 2, 3],
        arms=arms,
        retention=compute_retention(arms),
    ).to_dict()
    metrics = aggregate_metrics(scores, edit_to_fix)
    perf = report_mod.build_performance(
        prompt_tokens=tp,
        completion_tokens=tc,
        total_cost_usd=cost,
        latencies=lat,
        request_count=sum(len(o["rounds"]) for o in outcomes),
    )
    meta = report_mod.RunMetadata(
        model=model.id,
        mode="self-test",
        endpoint="mock (offline)",
        task_set_id="gold-compose-v1",
        seed=42,
        max_rounds=3,
        synthetic=True,
        timestamp_utc="SAMPLE-DETERMINISTIC",
    )
    mr = report_mod.ModelRunReport(
        metadata=meta,
        quality=metrics.to_dict(),
        performance=perf,
        outcomes=outcomes,
        ablation=ablation,
    )
    jp = report_mod.write_model_json(mr, reports_dir=reports_dir, run_id="SAMPLE")
    md = report_mod.write_comparison_markdown(
        [mr],
        reports_dir=reports_dir,
        run_id="SAMPLE",
        mode="self-test",
        synthetic=True,
        generated="SAMPLE-DETERMINISTIC",
    )
    print(f"wrote sample: {jp}")
    print(f"wrote sample: {md}")
