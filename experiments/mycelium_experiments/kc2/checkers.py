"""KC-2 pass/fail oracles (M-002).

The Mycelium arm is judged by the same deterministic toolchain a human uses — the `myc-check`
binary (parse + typecheck + task-signature conformance; no AI in the loop, S6). The baseline arm
is judged by Python's own parser plus execution of `main()` against the task's expected result
shape.

Failure taxonomy mirrors M-002's metrics: *syntactic validity* (does it parse at all?) is
reported separately from the *check pass* (does it typecheck / run and answer the task?), and
every failure carries the diagnostic the generator would receive as edit-to-fix feedback.
"""

from __future__ import annotations

import ast
import os
import subprocess
from dataclasses import dataclass
from pathlib import Path

from mycelium_experiments.kc2 import baseline
from mycelium_experiments.kc2.tasks import Task


class ToolUnavailable(RuntimeError):
    """The deterministic toolchain needed by a checker is absent (checks skip gracefully)."""


@dataclass(frozen=True)
class CheckResult:
    """One attempt's verdict, with the diagnostic that becomes edit-to-fix feedback."""

    syntactically_valid: bool
    passes: bool
    diagnostic: str = ""


def _repo_root() -> Path:
    """The workspace root (the directory holding `Cargo.toml` and `justfile`)."""
    for p in Path(__file__).resolve().parents:
        if (p / "Cargo.toml").is_file() and (p / "justfile").is_file():
            return p
    msg = "could not locate the Mycelium workspace root above this file"
    raise ToolUnavailable(msg)


class MyceliumChecker:
    """Judge Mycelium sources with `myc-check` (exit 0 ok / 2 parse error / 3 check error).

    Binary discovery, in order: the ``MYC_CHECK`` environment variable; an existing
    ``target/debug/myc-check``; building it once via ``cargo build``. A missing toolchain raises
    :class:`ToolUnavailable` — callers (tests) skip, never silently pass.
    """

    def __init__(self, binary: Path | None = None) -> None:
        self._binary = binary or self._discover()

    @staticmethod
    def _discover() -> Path:
        env = os.environ.get("MYC_CHECK")
        if env:
            p = Path(env)
            if p.is_file():
                return p
            msg = f"MYC_CHECK points at a non-existent file: {env}"
            raise ToolUnavailable(msg)
        root = _repo_root()
        exe = "myc-check.exe" if os.name == "nt" else "myc-check"
        built = root / "target" / "debug" / exe
        if built.is_file():
            return built
        # The `myc-check` binary lives in the `mycelium-check` crate (NOT mycelium-l1).
        cmd = ["cargo", "build", "-p", "mycelium-check", "--bin", "myc-check"]
        try:
            proc = subprocess.run(
                cmd,
                cwd=root,
                capture_output=True,
                text=True,
                timeout=900,
            )
        except OSError as e:
            msg = f"could not run cargo ({' '.join(cmd)}): {e}"
            raise ToolUnavailable(msg) from e
        if proc.returncode != 0:
            # Never-silent: surface cargo's actual diagnostic, not just the exit code,
            # so a real compile failure is actionable instead of a bare "exit 101".
            # Keep WHOLE trailing lines (the last N) — a raw byte-tail cut mid-line and
            # rendered as garbage like "y: cc help". The first line stays the concise
            # reason a summary can show; the rest is the actionable detail.
            raw_lines = (proc.stderr or proc.stdout or "").strip().splitlines()
            tail = raw_lines[-25:]
            prefix = "…(truncated; full output in the cargo log)\n" if len(raw_lines) > 25 else ""
            detail = prefix + "\n".join(tail)
            msg = f"`{' '.join(cmd)}` failed (exit {proc.returncode}). {detail}"
            raise ToolUnavailable(msg)
        if not built.is_file():
            msg = f"cargo reported success but {built} does not exist"
            raise ToolUnavailable(msg)
        return built

    def check(self, source: str, task: Task) -> CheckResult:
        """Parse + typecheck `source` and require `fn main() -> {task.expect_main}`."""
        proc = subprocess.run(
            [str(self._binary), "--expect-main", task.expect_main, "-"],
            input=source,
            text=True,
            capture_output=True,
            timeout=60,
            check=False,
        )
        diagnostic = (proc.stdout + proc.stderr).strip()
        if proc.returncode == 0:
            return CheckResult(syntactically_valid=True, passes=True)
        if proc.returncode == 2:
            return CheckResult(syntactically_valid=False, passes=False, diagnostic=diagnostic)
        if proc.returncode == 3:
            return CheckResult(syntactically_valid=True, passes=False, diagnostic=diagnostic)
        msg = f"myc-check failed unexpectedly (exit {proc.returncode}): {diagnostic}"
        raise ToolUnavailable(msg)


class BaselineChecker:
    """Judge baseline-DSL sources: `ast.parse` for syntax, then execute and call `main()`.

    .. warning::
       Execution happens **in-process** (`exec`). That is fine for this repo's fixtures and
       reference solutions; scoring *untrusted model output* must run the harness inside a
       disposable sandbox (container/VM) — running it directly would execute arbitrary generated
       code. This is a documented operational requirement of the eventual M-002 run, not a
       property this class can provide.

    To keep that requirement **structural** (A6-10/B2-04) rather than a docstring footnote, the
    in-process `exec` is gated behind ``allow_untrusted``, which defaults to ``False``. A default
    :class:`BaselineChecker` *refuses* to run any source — :meth:`check` returns an explicit
    error :class:`CheckResult` instead of silently executing it. Running real (trusted) fixtures,
    or scoring generated output inside a sandbox, is then a deliberate, visible opt-in
    (``BaselineChecker(allow_untrusted=True)``) at the call site — never an accident.
    """

    def __init__(self, *, allow_untrusted: bool = False) -> None:
        self._allow_untrusted = allow_untrusted

    def check(self, source: str, task: Task) -> CheckResult:
        """Check `source` defines `main()` returning the task's expected result shape.

        Refuses (returns an explicit non-passing :class:`CheckResult`, never silently) unless this
        checker was constructed with ``allow_untrusted=True`` — running arbitrary source in-process
        must be a deliberate choice made where the sandbox guarantees live.
        """
        if not self._allow_untrusted:
            return CheckResult(
                syntactically_valid=False,
                passes=False,
                diagnostic=(
                    "refusing to exec baseline source: in-process execution is disabled by "
                    "default — construct BaselineChecker(allow_untrusted=True) at a call site "
                    "that owns the sandbox guarantee (see class docstring)"
                ),
            )
        try:
            ast.parse(source)
        except SyntaxError as e:
            return CheckResult(
                syntactically_valid=False, passes=False, diagnostic=f"syntax error: {e}"
            )
        namespace: dict[str, object] = {
            "Bin": baseline.Bin,
            "Tern": baseline.Tern,
            "bnot": baseline.bnot,
            "xor": baseline.xor,
            "tadd": baseline.tadd,
            "swap": baseline.swap,
        }
        try:
            exec(source, namespace)  # noqa: S102 — see class docstring: sandboxing is operational
            main = namespace.get("main")
            if not callable(main):
                return CheckResult(
                    syntactically_valid=True,
                    passes=False,
                    diagnostic="no callable `main` defined",
                )
            result = main()
        except Exception as e:  # noqa: BLE001 — any generated-code failure is one diagnostic
            return CheckResult(
                syntactically_valid=True, passes=False, diagnostic=f"{type(e).__name__}: {e}"
            )
        kind, width = task.expect_baseline
        expected_type = baseline.Bin if kind == "bin" else baseline.Tern
        if not isinstance(result, expected_type) or result.width != width:
            got = f"{type(result).__name__}(width={getattr(result, 'width', '?')})"
            return CheckResult(
                syntactically_valid=True,
                passes=False,
                diagnostic=f"main() returned {got}, task requires {kind} of width {width}",
            )
        return CheckResult(syntactically_valid=True, passes=True)
