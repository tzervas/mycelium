"""The M-330 generate -> feedback -> fix co-authoring loop, backend-agnostic.

This is the iterative loop the brief assigns to M-330, rebuilt on the pluggable
:class:`~grok.client.ChatClient` + :class:`~grok.scoring.MycCheckScorer` instead of
being welded to llama. One *program* runs up to ``max_rounds`` generate->score
attempts; the loop stops early when the scorer reports ``clean``. Each attempt
records tokens / latency / cost (mode-appropriate price) so the report can total
them per model.

The iterative loop stays **live / multi-turn** — it is NOT batchable, because each
fix depends on the previous round's diagnostics. (Batch covers only the
independent first-pass generations and the ablation's independent samples; see
``grok/batch.py``.)

HONESTY (VR-5 / G2):
  * model output is tagged **Empirical** (live) or **Declared** (mock) — set by the
    client, never upgraded here.
  * generated source is scanned for forbidden ``Proven``/``Exact`` self-tags
    (reusing the existing ``coauthor.check_guarantee_tags_in_source`` rule,
    re-implemented locally to avoid importing the sibling module).
  * a scorer SKIP is a SKIP, never a PASS; an empty generation is never clean.
"""

from __future__ import annotations

import datetime
import logging
from collections.abc import Sequence
from dataclasses import dataclass, field
from typing import Any

from .client import ChatClient, ChatMessage, ChatResult
from .models import ModelSpec
from .scoring import VERDICT_CLEAN, VERDICT_SKIP, MycCheckScorer, ScoreResult
from .tasks import Task

_LOG = logging.getLogger("grok.coauthor")

# VR-5: a model may self-tag Empirical/Declared but never Proven/Exact.
_FORBIDDEN_SELF_TAGS = ("proven", "exact")
_GUARANTEE_KEYWORDS = ("@guarantee:", "guarantee:")

SYSTEM_PROMPT = (
    "You are a Mycelium language assistant. Mycelium is a typed, purely functional "
    "language with explicit representation swaps between Binary and Ternary types. "
    "Every program begins with a `nodule <name>` header. Swaps are NEVER silent: "
    "always write `swap(x, to: T, policy: <policy>)` with a policy from "
    "{roundtrip, clamp, saturate}. A Binary/Ternary type mismatch REQUIRES an "
    "explicit swap. Reply with ONLY the Mycelium program, no prose, no code fences."
)


def scan_forbidden_self_tags(source_text: str) -> list[str]:
    """Return VR-5 violations: source self-claiming Proven/Exact guarantees (PURE).

    Mirrors ``coauthor.check_guarantee_tags_in_source`` but is self-contained so the
    grok package does not import its sibling.
    """
    violations: list[str] = []
    for lineno, line in enumerate(source_text.splitlines(), start=1):
        low = line.lower()
        for kw in _GUARANTEE_KEYWORDS:
            if kw in low:
                after = low.split(kw, 1)[1].strip()
                tag = after.split()[0].rstrip(".,;)") if after.split() else ""
                if tag in _FORBIDDEN_SELF_TAGS:
                    violations.append(
                        f"line {lineno}: model output self-claims '{tag.capitalize()}' "
                        "guarantee — FORBIDDEN for model-derived claims (VR-5)"
                    )
    return violations


def build_messages(
    spec: str, feedback: list[dict[str, Any]] | None
) -> list[ChatMessage]:
    """Build the chat messages for a generate or correction turn (PURE)."""
    msgs = [ChatMessage("system", SYSTEM_PROMPT)]
    user = [f"Task: {spec}"]
    if feedback:
        last = feedback[-1]
        user.append("")
        user.append("Your previous program had these checker diagnostics to fix:")
        for d in last.get("diagnostics", []):
            user.append(f"  - {d.get('message', '')}")
        user.append("")
        user.append("Return a corrected Mycelium program.")
    else:
        user.append("Write a correct Mycelium program.")
    msgs.append(ChatMessage("user", "\n".join(user)))
    return msgs


@dataclass
class RoundRecord:
    """One generate->score attempt within a program's loop."""

    attempt: int  # 1-based
    source_text: str
    score: ScoreResult
    chat: ChatResult
    cost_usd: float
    is_correction: bool
    timestamp_utc: str = ""  # ISO wall-clock when the response was received

    def to_dict(self) -> dict[str, Any]:
        return {
            "attempt": self.attempt,
            "is_correction": self.is_correction,
            "timestamp_utc": self.timestamp_utc,
            "verdict": self.score.verdict,
            "syntactic_valid": self.score.syntactic_valid,
            "typecheck_pass": self.score.typecheck_pass,
            "exit_code": self.score.exit_code,
            "prompt_tokens": self.chat.prompt_tokens,
            "completion_tokens": self.chat.completion_tokens,
            "latency_s": round(self.chat.latency_s, 4),
            "cost_usd": round(self.cost_usd, 8),
            "chat_ok": self.chat.ok,
            "chat_error": self.chat.error,
            "diagnostics": self.score.diagnostics,
        }


@dataclass
class TaskOutcome:
    """The terminal result of running one task's full loop."""

    task_id: str
    spec: str
    model: str
    status: str  # PASS | PARTIAL_PASS | FAIL | SKIP
    guarantee_tag: str
    rounds: list[RoundRecord] = field(default_factory=list)
    message: str = ""

    @property
    def final_score(self) -> ScoreResult | None:
        return self.rounds[-1].score if self.rounds else None

    @property
    def iterations_to_clean(self) -> int | None:
        """1-based attempt index at which the program first became clean, else None."""
        for r in self.rounds:
            if r.score.verdict == VERDICT_CLEAN:
                return r.attempt
        return None

    @property
    def total_prompt_tokens(self) -> int:
        return sum(r.chat.prompt_tokens for r in self.rounds)

    @property
    def total_completion_tokens(self) -> int:
        return sum(r.chat.completion_tokens for r in self.rounds)

    @property
    def total_cost_usd(self) -> float:
        return sum(r.cost_usd for r in self.rounds)

    @property
    def total_latency_s(self) -> float:
        return sum(r.chat.latency_s for r in self.rounds)

    def to_dict(self) -> dict[str, Any]:
        return {
            "task_id": self.task_id,
            "spec": self.spec,
            "model": self.model,
            "status": self.status,
            "guarantee_tag": self.guarantee_tag,
            "iterations_to_clean": self.iterations_to_clean,
            "rounds": [r.to_dict() for r in self.rounds],
            "total_prompt_tokens": self.total_prompt_tokens,
            "total_completion_tokens": self.total_completion_tokens,
            "total_latency_s": round(self.total_latency_s, 4),
            "total_cost_usd": round(self.total_cost_usd, 8),
            "message": self.message,
        }


# Status codes (mirror coauthor.py vocabulary).
STATUS_PASS = "PASS"
STATUS_PARTIAL_PASS = "PARTIAL_PASS"
STATUS_FAIL = "FAIL"
STATUS_SKIP = "SKIP"


def run_task_loop(
    *,
    task: Task,
    model: ModelSpec,
    client: ChatClient,
    scorer: MycCheckScorer,
    guarantee_tag: str,
    max_rounds: int = 3,
    batch: bool = False,
    before_request: Any = None,
    after_request: Any = None,
    seed: int = 42,
    log: logging.Logger | None = None,
) -> TaskOutcome:
    """Run the generate->score->fix loop for one task against one model.

    ``before_request(estimated_tokens)`` / ``after_request(chat_result)`` are
    optional hooks the live runner uses to drive the rate pacer (estimate-then-
    correct). ``batch`` selects the price pair for cost accounting. Returns a
    :class:`TaskOutcome` whose status is one of PASS / PARTIAL_PASS / FAIL / SKIP.
    """
    log = log or _LOG
    feedback: list[dict[str, Any]] = []
    rounds: list[RoundRecord] = []
    for attempt in range(1, max_rounds + 1):
        messages = build_messages(task.spec, feedback if attempt > 1 else None)
        est_tokens = _estimate_tokens(messages)
        if before_request is not None:
            before_request(est_tokens)
        chat = client.complete(model=model.id, messages=messages, seed=seed)
        ts = datetime.datetime.now(datetime.UTC).strftime("%Y%m%dT%H%M%SZ")
        if after_request is not None:
            after_request(chat)

        if not chat.ok:
            # An API failure is never a PASS; record and stop (G2).
            score = ScoreResult(
                verdict="error",
                syntactic_valid=False,
                typecheck_pass=False,
                exit_code=None,
                message=f"API call failed: {chat.error}",
            )
            cost = model.cost_usd(
                prompt_tokens=chat.prompt_tokens,
                completion_tokens=chat.completion_tokens,
                batch=batch,
            )
            rounds.append(
                RoundRecord(
                    attempt,
                    "",
                    score,
                    chat,
                    cost,
                    is_correction=attempt > 1,
                    timestamp_utc=ts,
                )
            )
            return TaskOutcome(
                task_id=task.id,
                spec=task.spec,
                model=model.id,
                status=STATUS_FAIL,
                guarantee_tag=guarantee_tag,
                rounds=rounds,
                message=f"API failure on attempt {attempt}: {chat.error}",
            )

        source = chat.content
        cost = model.cost_usd(
            prompt_tokens=chat.prompt_tokens,
            completion_tokens=chat.completion_tokens,
            batch=batch,
        )

        # VR-5: reject forbidden self-tags before even scoring (terminal FAIL).
        viol = scan_forbidden_self_tags(source)
        if viol:
            score = ScoreResult(
                verdict="error",
                syntactic_valid=False,
                typecheck_pass=False,
                exit_code=None,
                message="; ".join(viol),
            )
            rounds.append(
                RoundRecord(
                    attempt,
                    source,
                    score,
                    chat,
                    cost,
                    is_correction=attempt > 1,
                    timestamp_utc=ts,
                )
            )
            return TaskOutcome(
                task_id=task.id,
                spec=task.spec,
                model=model.id,
                status=STATUS_FAIL,
                guarantee_tag=guarantee_tag,
                rounds=rounds,
                message=f"VR-5 violation: {'; '.join(viol)}",
            )

        score = scorer.score(source)
        rounds.append(
            RoundRecord(
                attempt,
                source,
                score,
                chat,
                cost,
                is_correction=attempt > 1,
                timestamp_utc=ts,
            )
        )

        if score.verdict == VERDICT_SKIP:
            return TaskOutcome(
                task_id=task.id,
                spec=task.spec,
                model=model.id,
                status=STATUS_SKIP,
                guarantee_tag=guarantee_tag,
                rounds=rounds,
                message=f"scorer unavailable: {score.message}",
            )

        if score.verdict == VERDICT_CLEAN:
            status = STATUS_PASS if attempt == 1 else STATUS_PARTIAL_PASS
            return TaskOutcome(
                task_id=task.id,
                spec=task.spec,
                model=model.id,
                status=status,
                guarantee_tag=guarantee_tag,
                rounds=rounds,
                message=(
                    "clean on first attempt"
                    if attempt == 1
                    else f"clean after {attempt} attempts (self-corrected)"
                ),
            )

        # Not clean and not skipped: feed diagnostics back and try again.
        feedback.append(
            {"attempt": attempt, "source": source, "diagnostics": score.diagnostics}
        )
        log.info(
            "task %s model %s attempt %d: %s — feeding back %d diagnostic(s)",
            task.id,
            model.id,
            attempt,
            score.verdict,
            len(score.diagnostics),
        )

    # Exhausted rounds without reaching clean.
    return TaskOutcome(
        task_id=task.id,
        spec=task.spec,
        model=model.id,
        status=STATUS_FAIL,
        guarantee_tag=guarantee_tag,
        rounds=rounds,
        message=f"not clean after {max_rounds} round(s)",
    )


def _estimate_tokens(messages: Sequence[ChatMessage]) -> int:
    """Cheap pre-call token estimate for pacing (~4 chars/token heuristic).

    Plus a fixed completion headroom so the TPM pacer reserves room for the reply.
    Deterministic so the self-test can assert on it.
    """
    chars = sum(len(m.content) for m in messages)
    prompt_est = max(1, chars // 4)
    completion_headroom = 256  # reserve for the model's reply
    return prompt_est + completion_headroom
