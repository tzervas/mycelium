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
  * Arm 4 (`LlmCanonical` projection) is now scored at the same parse+typecheck bar
    as the novel arms via the ``llm_canonical_to_l1`` bridge (DN-09 §9.4 option b),
    so it provides the retention-ratio denominator and the threshold comparison is
    determinate when it runs. Arm 5 (embedded-DSL baseline, RR-3) is now RUNNABLE:
    the model writes a small Python embedded DSL which is evaluated in a restricted
    sandbox to ``.myc`` and scored by the same ``myc-check``. Arm 3
    (grammar-constrained decoding) is implemented + offline-tested but stays
    runtime-``blocked`` until a local GBNF backend is present (the xAI REST surface
    exposes no grammar param — M-331 llama.cpp path); we never fabricate outputs (VR-5).

Independent samples ((arm, task, seed)) are batchable; this runner can drive them
live (paced) or be fed batch results.
"""

from __future__ import annotations

import logging
from collections.abc import Callable
from dataclasses import dataclass, field
from typing import Any

from .arm3_constrained import arm3_constrained_prompt
from .arm5_embedded_dsl import arm5_embedded_dsl_prompt, eval_embedded_dsl
from .client import ChatClient, ChatMessage
from .coauthor_loop import SYSTEM_PROMPT, scan_forbidden_self_tags
from .llm_canonical_arm4 import arm4_llm_canonical_prompt
from .llm_canonical_to_l1 import convert_to_myc
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

# A canonical bridge maps (model output, task) -> a typecheckable .myc source, or
# None if the LlmCanonical output cannot be faithfully converted (G2: never a false
# PASS). Injected so the offline self-test can exercise arm-4 plumbing without a
# scorer/binary; the default is the real DN-09 §9.4 option-(b) bridge.
CanonicalBridge = Callable[[str, Task], "str | None"]


def default_canonical_bridge(content: str, task: Task) -> str | None:
    """The real LlmCanonical->L1 bridge for arm 4 (DN-09 §9.4 option b).

    Wraps the model's converted expression in the task's *known* signature so the
    authoritative ``myc-check`` can typecheck it at the same bar as arms 1/2. A task
    with no declared signature cannot be bridged -> ``None`` (scored not-clean, G2).

    Guarantee tag: Empirical (the conversion is a heuristic rewrite; the type-check
    is ``myc-check``'s — see ``llm_canonical_to_l1``).
    """
    if task.fn_name is None or task.param_type is None or task.return_type is None:
        return None
    return convert_to_myc(
        content,
        fn_name=task.fn_name,
        param_type=task.param_type,
        return_type=task.return_type,
        param_name=task.param_name or "x",
    )


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
    """The five protocol arms: 1, 2, 4, 5 runnable; arm 3 blocked on a local backend.

    Honest wiring (research/11 §T11.7): arms 1 and 2 are runnable against any chat
    backend; arm 3 is implemented + offline-tested but runtime-blocked (needs a local
    GBNF decoder — the OpenAI-compatible REST surface exposes no grammar param);
    arm 5 (embedded-DSL, RR-3) is runnable (model writes a Python DSL → sandboxed
    eval → ``.myc`` → ``myc-check``); arm 4 is RUNNABLE — the
    LlmCanonical renderer (M-380) is enacted and the parser + harness integration
    (M-381 Arm 4, W2L3) landed.
    Blocked arms carry their reason and are never given a fabricated score.

    Note on arm 4 scoring (DN-09 §9.4 option b, M-381): the harness converts the
    model's LlmCanonical S-expression to ``.myc`` via the
    ``llm_canonical_to_l1`` bridge and scores the produced ``.myc`` with the SAME
    ``myc-check`` (parse+typecheck) used for arms 1/2 — so arm 4 sits on the same
    quality bar and yields a determinate retention-ratio denominator. The bridge is
    Empirical (heuristic rewrite; the type-check is authoritative ``myc-check``); an
    output it cannot convert is scored not-clean, never a false PASS (G2). The model
    is not asked to author the fn signature (LlmCanonical cannot express it) — the
    bridge supplies the task's known signature, which makes the retention ratio a
    CONSERVATIVE estimate (see ``RetentionVerdict.to_dict``'s ``arm4_basis``).
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
            description="+ grammar-constrained decoding (GBNF, local backend)",
            # The arm-3 module is implemented + offline-tested (GBNF grammar +
            # ConstrainedBackend in arm3_constrained.py), but it stays BLOCKED at
            # runtime until a local GBNF backend is present: the xAI REST surface
            # exposes no grammar parameter, so routing REST output here would
            # mislabel un-constrained text as "constrained" (dishonest). Activating
            # it = install llama_cpp + set MYC_ARM3_MODEL (the M-331 llama.cpp path)
            # and route run_arm's generation through ConstrainedBackend. Never
            # fabricated (VR-5).
            runnable=False,
            blocked_reason=(
                "arm-3 module implemented + offline-tested (GBNF grammar + "
                "ConstrainedBackend, arm3_constrained.py); BLOCKED at runtime — no "
                "local GBNF backend here and the xAI REST surface exposes no grammar "
                "param. To run: install llama_cpp + set MYC_ARM3_MODEL (M-331 path), "
                "then route generation through ConstrainedBackend. Not fabricated (VR-5)."
            ),
            prompt_builder=arm3_constrained_prompt,
        ),
        Arm(
            id=ARM_CANONICAL,
            description="LlmCanonical projection (familiar-skin, same AST)",
            runnable=True,
            prompt_builder=arm4_llm_canonical_prompt,
        ),
        Arm(
            id=ARM_EMBEDDED_DSL,
            description="embedded-DSL baseline (RR-3): model writes a Python DSL → .myc",
            runnable=True,
            prompt_builder=arm5_embedded_dsl_prompt,
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
            "arm4_basis": (
                "arm 4 (denominator) is scored via the LlmCanonical->L1 bridge "
                "(DN-09 §9.4 option b): the same myc-check parse+typecheck as arms "
                "1/2, but the model is NOT asked to author the fn signature (the "
                "format cannot express it) — the bridge supplies the task's known "
                "signature. Arm 4 is therefore slightly advantaged, so the retention "
                "ratio is a CONSERVATIVE (downward-biased) estimate: clearing the "
                "threshold is robust; falling below it is ambiguous (Empirical)."
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
    canonical_bridge: CanonicalBridge | None = None,
    log: logging.Logger | None = None,
) -> ArmResult:
    """Run one arm's independent (task × seed) pass@1 samples (single attempt each).

    A blocked arm returns an :class:`ArmResult` with ``ran=False`` and its reason —
    never a fabricated score (VR-5). pass@1 counts a sample clean iff the scorer's
    verdict is ``clean`` on the single generation (no correction rounds — pass@1).

    Arm 4 (``LlmCanonical``) is scored at the **same** parse+typecheck bar as the
    novel arms: its S-expression output is first converted to ``.myc`` by
    ``canonical_bridge`` (default :func:`default_canonical_bridge`, DN-09 §9.4
    option b) and the produced ``.myc`` is then scored by the same ``myc-check``.
    An output the bridge cannot convert is scored not-clean — never a false PASS (G2).
    """
    log = log or _LOG
    if not arm.runnable or arm.prompt_builder is None:
        return ArmResult(
            arm_id=arm.id,
            description=arm.description,
            ran=False,
            blocked_reason=arm.blocked_reason or "arm not runnable",
        )
    bridge = canonical_bridge or default_canonical_bridge
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
            elif arm.id == ARM_CANONICAL:
                # Bridge LlmCanonical -> .myc, then score with the SAME myc-check.
                myc = bridge(chat.content, task)
                if myc is None:
                    note = "bridge: LlmCanonical not convertible to L1 (not-clean — G2)"
                else:
                    score = scorer.score(myc)
                    clean = score.verdict == VERDICT_CLEAN
                    note = f"bridge->{score.verdict}"
            elif arm.id == ARM_EMBEDDED_DSL:
                # Eval the model's Python embedded-DSL in a restricted sandbox -> .myc,
                # then score with the SAME myc-check (same bar as arms 1/2/4). A snippet
                # that errors/escapes the sandbox -> None -> not-clean, never a false
                # PASS (G2). The sandbox is best-effort restricted eval (Declared).
                myc = eval_embedded_dsl(chat.content)
                if myc is None:
                    note = "arm5-dsl: not evaluable to L1 (not-clean — G2)"
                else:
                    score = scorer.score(myc)
                    clean = score.verdict == VERDICT_CLEAN
                    note = f"dsl->{score.verdict}"
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
