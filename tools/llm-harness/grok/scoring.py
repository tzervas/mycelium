"""Score generated Mycelium source — syntactic validity vs type-check (KC-2/SC-5b).

The harness brief asks for, per program: **syntactic validity rate** and
**type-check pass rate**. Mycelium's own ``myc-check`` binary gives exactly this
distinction through its **exit codes** (confirmed in
``crates/mycelium-check/src/bin/myc-check.rs``):

    0  -> clean (parses AND type-checks)
    2  -> parse error  => syntactically INVALID
    3  -> check error  => syntactically valid, but FAILS type-checking
    5  -> project-resolution error
    64 -> usage error
    66 -> I/O error

So a single ``myc-check`` invocation on a temp file classifies a program into
``{syntactic_invalid, typecheck_fail, clean}``. For *richer* feedback to feed the
M-330 correction loop we additionally run the LSP diagnostics shim
(``cargo run --example check -p mycelium-lsp`` reading source on stdin, emitting an
LSP ``publishDiagnostics`` JSON), reusing the existing ``coauthor.Checker`` shape.

HONESTY / never-silent (G2): if the scorer binary cannot be built or run, the
result is an explicit ``SKIP`` (``scorer_available=False``) — never a false PASS.
The runner is injectable so the offline ``--self-test`` exercises the classifier
math with a fake ``myc-check`` and never touches cargo.
"""

from __future__ import annotations

import logging
import subprocess
import tempfile
from collections.abc import Callable
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

_LOG = logging.getLogger("grok.scoring")

# myc-check exit codes (see crates/mycelium-check/src/bin/myc-check.rs).
EXIT_CLEAN = 0
EXIT_PARSE_ERROR = 2
EXIT_CHECK_ERROR = 3
EXIT_PROJECT_ERROR = 5
EXIT_USAGE_ERROR = 64
EXIT_IO_ERROR = 66

# Verdicts (never-silent: every program lands in exactly one).
VERDICT_CLEAN = "clean"  # parses and type-checks
VERDICT_SYNTAX_ERROR = "syntax_error"  # parse error (syntactically invalid)
VERDICT_TYPE_ERROR = "type_error"  # parses but fails type-check
VERDICT_ERROR = "error"  # tool/project/usage/io error — inconclusive
VERDICT_SKIP = "skip"  # scorer unavailable (G2)


@dataclass
class ScoreResult:
    """The classification of one generated program (one source text)."""

    verdict: str  # one of VERDICT_*
    syntactic_valid: bool  # did it parse?  (False for syntax_error/skip/error)
    typecheck_pass: bool  # did it type-check? (=> also syntactically valid)
    exit_code: int | None  # raw myc-check exit code (None if not run)
    diagnostics: list[dict[str, Any]] = field(default_factory=list)
    message: str = ""
    scorer_available: bool = True

    def to_dict(self) -> dict[str, Any]:
        return {
            "verdict": self.verdict,
            "syntactic_valid": self.syntactic_valid,
            "typecheck_pass": self.typecheck_pass,
            "exit_code": self.exit_code,
            "diagnostics": self.diagnostics,
            "message": self.message,
            "scorer_available": self.scorer_available,
        }


def classify_exit_code(code: int) -> ScoreResult:
    """Map a ``myc-check`` exit code to a :class:`ScoreResult` (PURE; offline-tested).

    This is the heart of the syntactic-vs-type distinction and is exercised
    directly by the self-test without invoking cargo.
    """
    if code == EXIT_CLEAN:
        return ScoreResult(
            verdict=VERDICT_CLEAN,
            syntactic_valid=True,
            typecheck_pass=True,
            exit_code=code,
            message="clean: parses and type-checks",
        )
    if code == EXIT_PARSE_ERROR:
        return ScoreResult(
            verdict=VERDICT_SYNTAX_ERROR,
            syntactic_valid=False,
            typecheck_pass=False,
            exit_code=code,
            message="parse error: syntactically invalid",
        )
    if code == EXIT_CHECK_ERROR:
        return ScoreResult(
            verdict=VERDICT_TYPE_ERROR,
            syntactic_valid=True,  # it parsed; the type-check is what failed
            typecheck_pass=False,
            exit_code=code,
            message="check error: parses but fails type-checking",
        )
    # 5/64/66 (and any unexpected code): inconclusive — not a quality signal.
    return ScoreResult(
        verdict=VERDICT_ERROR,
        syntactic_valid=False,
        typecheck_pass=False,
        exit_code=code,
        message=f"myc-check returned inconclusive exit code {code} "
        "(project/usage/io error) — treated as non-PASS, not a quality verdict",
    )


# A runner takes (source_text) and returns (exit_code, stdout, stderr).
RunnerFn = Callable[[str], "tuple[int, str, str]"]


class MycCheckScorer:
    """Score source by shelling out to ``myc-check`` (the exit-code classifier).

    Lazily resolves a runner: the default builds + runs
    ``cargo run -q -p mycelium-check --bin myc-check -- <tmpfile>``. If cargo is
    absent or the build fails, the scorer marks itself unavailable and every score
    is a ``SKIP`` (never a false PASS). A custom ``runner`` is injected by the
    self-test to drive the classifier offline.
    """

    def __init__(
        self,
        *,
        repo_root: Path | None = None,
        runner: RunnerFn | None = None,
        timeout_s: float = 180.0,
        log: logging.Logger | None = None,
    ) -> None:
        self._repo_root = repo_root
        self._runner = runner
        self._timeout_s = timeout_s
        self._log = log or _LOG
        self._available: bool | None = None if runner is None else True
        self._skip_reason = ""

    def _default_runner(self, source_text: str) -> tuple[int, str, str]:
        """Write ``source_text`` to a temp ``.myc`` and run ``myc-check`` on it."""
        with tempfile.NamedTemporaryFile(
            "w", suffix=".myc", delete=True, encoding="utf-8"
        ) as tf:
            tf.write(source_text)
            tf.flush()
            cmd = [
                "cargo",
                "run",
                "-q",
                "-p",
                "mycelium-check",
                "--bin",
                "myc-check",
                "--",
                tf.name,
            ]
            try:
                proc = subprocess.run(
                    cmd,
                    capture_output=True,
                    text=True,
                    timeout=self._timeout_s,
                    cwd=str(self._repo_root) if self._repo_root else None,
                )
            except FileNotFoundError as exc:
                raise BackendScorerError(f"cargo not found: {exc}") from exc
            except subprocess.TimeoutExpired as exc:
                raise BackendScorerError(
                    f"myc-check timed out after {self._timeout_s}s"
                ) from exc
            return proc.returncode, proc.stdout, proc.stderr

    def score(self, source_text: str) -> ScoreResult:
        """Classify ``source_text`` into a :class:`ScoreResult` (never-silent)."""
        if not source_text.strip():
            # An empty generation is never clean (G2: matches coauthor.py policy).
            return ScoreResult(
                verdict=VERDICT_ERROR,
                syntactic_valid=False,
                typecheck_pass=False,
                exit_code=None,
                message="empty generation (no source to check) — never a PASS",
            )
        runner = self._runner or self._default_runner
        try:
            code, _out, err = runner(source_text)
        except BackendScorerError as exc:
            self._available = False
            self._skip_reason = str(exc)
            return ScoreResult(
                verdict=VERDICT_SKIP,
                syntactic_valid=False,
                typecheck_pass=False,
                exit_code=None,
                message=f"scorer unavailable (SKIP — G2): {exc}",
                scorer_available=False,
            )
        self._available = True
        result = classify_exit_code(code)
        # Carry a tail of stderr as a diagnostic hint for the correction loop.
        if err and result.verdict in (VERDICT_SYNTAX_ERROR, VERDICT_TYPE_ERROR):
            result.diagnostics = [
                {"message": line} for line in err.strip().splitlines()
            ]
        return result

    @property
    def available(self) -> bool | None:
        return self._available

    @property
    def skip_reason(self) -> str:
        return self._skip_reason


class BackendScorerError(RuntimeError):
    """The scorer backend could not be run (cargo missing, timeout, …)."""


# ---------------------------------------------------------------------------
# Aggregate metrics across a set of scored programs (KC-2/SC-5b rates)
# ---------------------------------------------------------------------------


@dataclass
class QualityMetrics:
    """Aggregate KC-2/SC-5b quality metrics over a batch of scored programs."""

    total: int
    syntactic_valid: int
    typecheck_pass: int
    skipped: int
    edit_to_fix_iterations: list[int] = field(default_factory=list)

    @property
    def scored(self) -> int:
        """Programs that actually produced a verdict (not skipped)."""
        return self.total - self.skipped

    @property
    def syntactic_validity_rate(self) -> float | None:
        """Fraction syntactically valid, over scored programs (None if none scored)."""
        return (self.syntactic_valid / self.scored) if self.scored else None

    @property
    def typecheck_pass_rate(self) -> float | None:
        """Fraction that type-check, over scored programs (None if none scored)."""
        return (self.typecheck_pass / self.scored) if self.scored else None

    @property
    def mean_edit_to_fix(self) -> float | None:
        """Mean correction iterations to reach clean (None if no clean programs)."""
        its = self.edit_to_fix_iterations
        return (sum(its) / len(its)) if its else None

    def to_dict(self) -> dict[str, Any]:
        return {
            "total": self.total,
            "scored": self.scored,
            "skipped": self.skipped,
            "syntactic_valid": self.syntactic_valid,
            "typecheck_pass": self.typecheck_pass,
            "syntactic_validity_rate": self.syntactic_validity_rate,
            "typecheck_pass_rate": self.typecheck_pass_rate,
            "edit_to_fix_iterations": self.edit_to_fix_iterations,
            "mean_edit_to_fix": self.mean_edit_to_fix,
        }


def aggregate_metrics(
    scores: list[ScoreResult], edit_to_fix: list[int] | None = None
) -> QualityMetrics:
    """Build :class:`QualityMetrics` from per-program scores (PURE; offline-tested).

    ``edit_to_fix`` is the list of iteration-counts for programs that reached
    clean (one entry per clean program). Skipped programs are excluded from rate
    denominators (a SKIP is never counted as a pass *or* a fail — G2).
    """
    total = len(scores)
    skipped = sum(1 for s in scores if s.verdict == VERDICT_SKIP)
    syn = sum(1 for s in scores if s.syntactic_valid)
    typ = sum(1 for s in scores if s.typecheck_pass)
    return QualityMetrics(
        total=total,
        syntactic_valid=syn,
        typecheck_pass=typ,
        skipped=skipped,
        edit_to_fix_iterations=list(edit_to_fix or []),
    )
