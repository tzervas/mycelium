"""M-381 retention-ratio ablation runner (research/11 §T11.7; RP-1 / T3.6).

Implements the *turnkey protocol* of ``research/11-semantic-projection-framework-
RECORD.md`` §T11.7 as a runnable mode. It runs the arms it CAN run over a
composition task set across ≥3 seeds, computes pass@1 per arm, and — when the
familiar-skin arm (arm 4) is present — the **retention ratio**

    retention = pass@1(best novel-surface arm) / pass@1(familiar-skin arm 4)

and compares it to the ``~70%`` falsification threshold (< ~70% ⇒ RFC-0021 §4.7
trigger: L3 must become an `LlmCanonical`-primary projection).

NON-NEGOTIABLE HONESTY (§T11.7 step 4; VR-5):
  * **Never pre-write the verdict.** This module refuses to assert a leverage
    conclusion. With no API key here it produces no real pass@1, and even with a
    key the verdict is reported as the *computed comparison*, tagged Empirical only
    for arms actually run, with the overall leverage claim left **Declared / open**.
  * **Report each arm at the strength actually run.** Arms that could not run
    (missing dependency) are recorded as ``blocked`` with their reason, never as a
    0% or 100% result.
  * Arms 3 (grammar-constrained decoding) and 4 (`LlmCanonical` projection
    renderer) depend on build deps that **do not exist yet** (M-380). They are
    declared and wired, but marked ``blocked`` until those deps land — we do not
    fabricate their outputs. The threshold comparison "applies only when arm 4 is
    present" (§T11.7 step 3), so it is reported as *indeterminate* until arm 4 runs.

Independent samples ((arm, task, seed)) are batchable; this runner can drive them
live (paced) or be fed batch results.
"""

from __future__ import annotations

import logging
from collections.abc import Callable
from dataclasses import dataclass, field
from typing import Any

from .client import ChatClient, ChatMessage
from .coauthor_loop import SYSTEM_PROMPT, scan_forbidden_self_tags
from .models import ModelSpec
from .scoring import VERDICT_CLEAN, MycCheckScorer
from .tasks import Task

_LOG = logging.getLogger("grok.ablation")

# The ~70% falsification threshold (research/11 §T11.7 step 3; RFC-0021 §4.7).
RETENTION_THRESHOLD = 0.70

# Arm ids, in protocol order (research/11 §T11.7 step 1).
ARM_BARE = "arm1-bare-novel-surface"
ARM_PRIMER = "arm2-grammar-primer"
ARM_CONSTRAINED = "arm3-grammar-constrained-decoding"
ARM_CANONICAL = "arm4-llm-canonical-projection"
ARM_EMBEDDED_DSL = "arm5-embedded-dsl-baseline"

# Arms whose surface is "novel" (the numerator candidates for the retention ratio).
NOVEL_SURFACE_ARMS = (ARM_BARE, ARM_PRIMER, ARM_CONSTRAINED)


@dataclass
class Arm:
    """One ablation arm: how it shapes the prompt, and whether it can run here."""

    id: str
    description: str
    runnable: bool
    blocked_reason: str = ""
    # Builds the messages for (task, seed). Only used when runnable.
    prompt_builder: Callable[[Task], list[ChatMessage]] | None = None


# A short, book-quality grammar primer for arm 2 (kept compact; the protocol asks
# for a "book-quality grammar-in-context primer" — this is a faithful seed).
_GRAMMAR_PRIMER = """\
Mycelium grammar (essentials):
  program  := nodule_header fn_def+
  nodule_header := 'nodule' IDENT
  fn_def   := 'fn' IDENT '(' params ')' '->' type '=' expr
  type     := 'Binary' '{' INT '}' | 'Ternary' '{' INT '}'
  expr     := IDENT | call | swap | let
  call     := IDENT '(' args ')'
  swap     := 'swap' '(' expr ',' 'to:' type ',' 'policy:' policy ')'
  policy   := 'roundtrip' | 'clamp' | 'saturate'
  let      := 'let' IDENT '=' expr 'in' expr
A swap is NEVER silent: every Binary<->Ternary crossing needs an explicit swap.
"""


def _bare_prompt(task: Task) -> list[ChatMessage]:
    return [
        ChatMessage("system", SYSTEM_PROMPT),
        ChatMessage("user", f"Task: {task.spec}\nWrite a correct Mycelium program."),
    ]


def _primer_prompt(task: Task) -> list[ChatMessage]:
    return [
        ChatMessage("system", SYSTEM_PROMPT + "\n\n" + _GRAMMAR_PRIMER),
        ChatMessage("user", f"Task: {task.spec}\nWrite a correct Mycelium program."),
    ]


def default_arms() -> list[Arm]:
    """The five protocol arms, with arms 3 & 4 & 5 marked blocked on missing deps.

    Honest wiring (research/11 §T11.7): arms 1 and 2 are runnable against any chat
    backend; arm 3 needs a grammar-constrained decoder, arm 4 needs the
    ``LlmCanonical`` projection renderer (both M-380, not yet built), and arm 5
    needs an embedded-DSL baseline harness. Blocked arms carry their reason and are
    never given a fabricated score.
    """
    return [
        Arm(
            id=ARM_BARE,
            description="bare novel text surface",
            runnable=True,
            prompt_builder=_bare_prompt,
        ),
        Arm(
            id=ARM_PRIMER,
            description="+ book-quality grammar-in-context primer",
            runnable=True,
            prompt_builder=_primer_prompt,
        ),
        Arm(
            id=ARM_CONSTRAINED,
            description="+ grammar-constrained decoding (GBNF/Outlines/Guidance)",
            runnable=False,
            blocked_reason=(
                "needs a grammar-constrained decoder integration (M-380); not built "
                "yet, and the OpenAI-compatible REST surface does not expose GBNF. "
                "Not fabricated (VR-5)."
            ),
        ),
        Arm(
            id=ARM_CANONICAL,
            description="LlmCanonical projection (familiar-skin, same AST)",
            runnable=False,
            blocked_reason=(
                "needs the LlmCanonical projection renderer over mycelium-core "
                "(M-380 / T11.4); not built yet. The retention-ratio DENOMINATOR — "
                "so the threshold comparison is indeterminate until this arm runs "
                "(research/11 §T11.7 step 3). Not fabricated (VR-5)."
            ),
        ),
        Arm(
            id=ARM_EMBEDDED_DSL,
            description="embedded-DSL baseline (RR-3)",
            runnable=False,
            blocked_reason=(
                "needs an embedded-DSL baseline harness (RR-3); out of scope for the "
                "Grok harness wiring. Not fabricated (VR-5)."
            ),
        ),
    ]


@dataclass
class ArmResult:
    """pass@1 outcome for one arm over (task × seed), or a blocked record."""

    arm_id: str
    description: str
    ran: bool
    blocked_reason: str = ""
    n_samples: int = 0
    n_clean: int = 0
    guarantee_tag: str = "Declared"
    per_sample: list[dict[str, Any]] = field(default_factory=list)

    @property
    def pass_at_1(self) -> float | None:
        """Fraction of independent samples that were clean on the single attempt."""
        return (self.n_clean / self.n_samples) if self.n_samples else None

    def to_dict(self) -> dict[str, Any]:
        return {
            "arm_id": self.arm_id,
            "description": self.description,
            "ran": self.ran,
            "blocked_reason": self.blocked_reason,
            "n_samples": self.n_samples,
            "n_clean": self.n_clean,
            "pass_at_1": self.pass_at_1,
            "guarantee_tag": self.guarantee_tag,
            "per_sample": self.per_sample,
        }


@dataclass
class RetentionVerdict:
    """The retention-ratio comparison — reported, NEVER pre-written (VR-5).

    ``determinate`` is False whenever arm 4 (the denominator) did not run; then
    ``ratio`` is None and ``conclusion`` is the honest "indeterminate — pending
    run" string. Even when determinate, ``leverage_claim_tag`` stays ``Declared``:
    a single harness run reports the computed comparison; it does not *ratify* the
    leverage hypothesis (that needs the full ≥3-seed ≥1-frontier campaign).
    """

    determinate: bool
    ratio: float | None
    threshold: float
    best_novel_arm: str | None
    best_novel_pass_at_1: float | None
    canonical_pass_at_1: float | None
    conclusion: str
    leverage_claim_tag: str = "Declared"

    def to_dict(self) -> dict[str, Any]:
        return {
            "determinate": self.determinate,
            "retention_ratio": self.ratio,
            "threshold": self.threshold,
            "best_novel_arm": self.best_novel_arm,
            "best_novel_pass_at_1": self.best_novel_pass_at_1,
            "canonical_pass_at_1": self.canonical_pass_at_1,
            "conclusion": self.conclusion,
            "leverage_claim_tag": self.leverage_claim_tag,
            "note": (
                "VR-5: this is the COMPUTED comparison from the arms actually run, "
                "not a ratified verdict. The leverage hypothesis stays open/Declared "
                "until the full ≥3-seed, ≥1-frontier campaign (research/11 §T11.6)."
            ),
        }


def compute_retention(arm_results: list[ArmResult]) -> RetentionVerdict:
    """Compute the retention ratio + threshold comparison (PURE; offline-tested).

    retention = pass@1(best novel-surface arm) / pass@1(arm 4 / familiar-skin).
    Indeterminate (and never a fabricated verdict) when arm 4 did not run or has a
    zero/None denominator.
    """
    by_id = {a.arm_id: a for a in arm_results}
    canonical = by_id.get(ARM_CANONICAL)

    # Best novel-surface arm among those that actually ran.
    novel_ran = [
        a
        for a in arm_results
        if a.arm_id in NOVEL_SURFACE_ARMS and a.ran and a.pass_at_1 is not None
    ]
    best_novel = max(novel_ran, key=lambda a: a.pass_at_1) if novel_ran else None

    if canonical is None or not canonical.ran or not canonical.pass_at_1:
        reason = (
            "arm 4 (LlmCanonical) did not run"
            if (canonical is None or not canonical.ran)
            else "arm 4 pass@1 is 0 (cannot form a ratio)"
        )
        return RetentionVerdict(
            determinate=False,
            ratio=None,
            threshold=RETENTION_THRESHOLD,
            best_novel_arm=best_novel.arm_id if best_novel else None,
            best_novel_pass_at_1=best_novel.pass_at_1 if best_novel else None,
            canonical_pass_at_1=canonical.pass_at_1 if canonical else None,
            conclusion=(
                f"INDETERMINATE — pending run: {reason}. The threshold comparison "
                "applies only when arm 4 is present (research/11 §T11.7 step 3)."
            ),
        )

    if best_novel is None:
        return RetentionVerdict(
            determinate=False,
            ratio=None,
            threshold=RETENTION_THRESHOLD,
            best_novel_arm=None,
            best_novel_pass_at_1=None,
            canonical_pass_at_1=canonical.pass_at_1,
            conclusion="INDETERMINATE — no novel-surface arm ran to form a numerator.",
        )

    ratio = best_novel.pass_at_1 / canonical.pass_at_1
    if ratio >= RETENTION_THRESHOLD:
        verdict = (
            f"retention {ratio:.1%} >= ~{RETENTION_THRESHOLD:.0%}: the novel surface "
            "RETAINS leverage in this run (working hypothesis not falsified here)."
        )
    else:
        verdict = (
            f"retention {ratio:.1%} < ~{RETENTION_THRESHOLD:.0%}: FALSIFICATION "
            "trigger in this run — RFC-0021 §4.7 says L3 should become an "
            "LlmCanonical-primary projection."
        )
    return RetentionVerdict(
        determinate=True,
        ratio=ratio,
        threshold=RETENTION_THRESHOLD,
        best_novel_arm=best_novel.arm_id,
        best_novel_pass_at_1=best_novel.pass_at_1,
        canonical_pass_at_1=canonical.pass_at_1,
        conclusion=verdict,
    )


def run_arm(
    *,
    arm: Arm,
    tasks: list[Task],
    seeds: list[int],
    model: ModelSpec,
    client: ChatClient,
    scorer: MycCheckScorer,
    guarantee_tag: str,
    batch: bool = False,
    before_request: Any = None,
    after_request: Any = None,
    log: logging.Logger | None = None,
) -> ArmResult:
    """Run one arm's independent (task × seed) pass@1 samples (single attempt each).

    A blocked arm returns an :class:`ArmResult` with ``ran=False`` and its reason —
    never a fabricated score (VR-5). pass@1 counts a sample clean iff the scorer's
    verdict is ``clean`` on the single generation (no correction rounds — pass@1).
    """
    log = log or _LOG
    if not arm.runnable or arm.prompt_builder is None:
        return ArmResult(
            arm_id=arm.id,
            description=arm.description,
            ran=False,
            blocked_reason=arm.blocked_reason or "arm not runnable",
        )
    per_sample: list[dict[str, Any]] = []
    n_clean = 0
    for task in tasks:
        for seed in seeds:
            messages = arm.prompt_builder(task)
            est = max(1, sum(len(m.content) for m in messages) // 4) + 256
            if before_request is not None:
                before_request(est)
            chat = client.complete(model=model.id, messages=messages, seed=seed)
            if after_request is not None:
                after_request(chat)
            clean = False
            note = ""
            if not chat.ok:
                note = f"api-error: {chat.error}"
            elif scan_forbidden_self_tags(chat.content):
                note = "VR-5: forbidden self-tag in output (counts as not-clean)"
            else:
                score = scorer.score(chat.content)
                clean = score.verdict == VERDICT_CLEAN
                note = score.verdict
            if clean:
                n_clean += 1
            per_sample.append(
                {
                    "task_id": task.id,
                    "seed": seed,
                    "clean": clean,
                    "verdict": note,
                    "prompt_tokens": chat.prompt_tokens,
                    "completion_tokens": chat.completion_tokens,
                }
            )
    return ArmResult(
        arm_id=arm.id,
        description=arm.description,
        ran=True,
        n_samples=len(per_sample),
        n_clean=n_clean,
        guarantee_tag=guarantee_tag,
        per_sample=per_sample,
    )


@dataclass
class AblationReport:
    """The full ablation outcome for one model: per-arm results + retention verdict."""

    model: str
    task_set_id: str
    seeds: list[int]
    arms: list[ArmResult]
    retention: RetentionVerdict

    def to_dict(self) -> dict[str, Any]:
        return {
            "experiment": "M-381 retention-ratio ablation (research/11 §T11.7; RP-1)",
            "model": self.model,
            "task_set_id": self.task_set_id,
            "seeds": self.seeds,
            "arms": [a.to_dict() for a in self.arms],
            "retention": self.retention.to_dict(),
        }
