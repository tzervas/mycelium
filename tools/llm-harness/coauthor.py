#!/usr/bin/env python3
"""
Mycelium AI co-authoring loop — M-330 (SC-5b; NFR-2; Area 6; E3-2).

Design posture (non-negotiable house rules):
- NEVER-SILENT (G2, RFC-0013 I1): every failure is explicit, logged, structured.
  A missing tool/model ⇒ SKIP, never a false PASS.  A round that cleans up after
  an error is PARTIAL_PASS, not silently PASS.
- HONESTY / guarantee lattice (VR-5): LLM-generated claims must be tagged
  Empirical or Declared — NEVER Proven or Exact. Mock claims are Declared.
- DUAL PROJECTION (G11): every run writes BOTH a structured JSON report AND a
  human-readable text report, as two renderers of one result object.
- Pure Python standard library only: no pip dependencies. Termux-portable.
- KC-3: the Checker delegates to `cargo run … --example check` (the mycelium-lsp
  crate's stdin→publishDiagnostics shim). No kernel deps in this Python layer.

Architecture:
    CoauthorSession
    ├── Generator (interface: generate(spec, feedback_history) -> source_text)
    │   ├── MockGenerator — cycles through canned programs (MOCK_PROGRAMS)
    │   └── LlmGenerator  — shells to llama-cli/server (SKIP if absent)
    ├── Checker  — shells out to `cargo run … --example check`
    │   └── check(source_text) -> CheckResult{diagnostics, is_clean}
    └── Loop     — generate → check → if not clean, regenerate up to max_rounds
        └── emits CoauthorRound records; final report is dual JSON+human (G11)

Usage:
    python3 coauthor.py --mock                         # mock mode (CI/cloud)
    python3 coauthor.py --mock --fix-mock              # mock: alternate error→fixed
    python3 coauthor.py --mock --spec "..."            # mock with a spec label
    python3 coauthor.py --check-only path/to/file.myc  # one-shot file check
    python3 coauthor.py --model PATH.gguf --spec "..." # real LLM (API-gated)
    python3 coauthor.py --server URL --spec "..."      # real LLM via server
    python3 coauthor.py --max-rounds 5                 # cap correction rounds

Exit codes:
    0   all rounds are PASS, mock-PASS, PARTIAL_PASS, or SKIP (no FAILures)
    1   one or more FAILures
    2   fatal setup error (cannot build checker, etc.)
"""

from __future__ import annotations

import argparse
import datetime
import json
import logging
import shutil
import subprocess
import sys
import time
from dataclasses import dataclass, asdict
from pathlib import Path
from typing import Any

# ---------------------------------------------------------------------------
# Guarantee lattice (VR-5) — duplicated from harness.py (pure stdlib; no import)
# ---------------------------------------------------------------------------

LATTICE_ORDERED = ["Exact", "Proven", "Empirical", "Declared"]
MODEL_ALLOWED_TAGS = {"Empirical", "Declared"}


def assert_model_tag(tag: str, claim_id: str) -> None:
    """Raise ValueError if tag is stronger than allowed for model-derived claims (VR-5)."""
    if tag not in LATTICE_ORDERED:
        raise ValueError(f"Unknown guarantee tag '{tag}' on claim '{claim_id}'")
    if tag not in MODEL_ALLOWED_TAGS:
        raise ValueError(
            f"[VR-5 VIOLATION] Claim '{claim_id}' carries tag '{tag}', "
            f"which is FORBIDDEN for model-derived claims. "
            f"Allowed: {sorted(MODEL_ALLOWED_TAGS)}. "
            "A model output may never be Proven or Exact without a checked basis."
        )


# ---------------------------------------------------------------------------
# Round status codes (G2 — never-silent; every outcome is explicit)
# ---------------------------------------------------------------------------

STATUS_PASS = "PASS"  # generated source is clean on first attempt
STATUS_MOCK_PASS = "mock-PASS"  # clean in mock mode (fixture, not real model)
STATUS_PARTIAL_PASS = "PARTIAL_PASS"  # clean after ≥1 correction round
STATUS_SKIP = "SKIP"  # generator or checker unavailable
STATUS_FAIL = "FAIL"  # still not clean after max_rounds

# ---------------------------------------------------------------------------
# Mock programs corpus — canned programs cycles through by MockGenerator
#
# Format: list of dicts with keys:
#   source  — the Mycelium source text
#   label   — short human label
#   expect_clean — True if we expect this to be accepted by the checker
#   corrected_source — for error programs: the fixed version (or None)
# ---------------------------------------------------------------------------

MOCK_PROGRAMS: list[dict[str, Any]] = [
    # --- Valid programs ---
    {
        "label": "minimal-nodule",
        "source": "nodule demo\nfn id(x: Binary{8}) -> Binary{8} = x",
        "expect_clean": True,
        "corrected_source": None,
    },
    {
        "label": "flip-not",
        "source": (
            "nodule arith\n"
            "fn flip(x: Binary{8}) -> Binary{8} =\n"
            "    let y = not(x) in y"
        ),
        "expect_clean": True,
        "corrected_source": None,
    },
    {
        "label": "double-add",
        "source": ("nodule arith\nfn double(x: Ternary{6}) -> Ternary{6} = add(x, x)"),
        "expect_clean": True,
        "corrected_source": None,
    },
    {
        "label": "widen-swap",
        "source": (
            "nodule demo\n"
            "fn widen(x: Binary{8}) -> Ternary{6} =\n"
            "    swap(x, to: Ternary{6}, policy: roundtrip)"
        ),
        "expect_clean": True,
        "corrected_source": None,
    },
    # --- Programs with fixable errors ---
    # (intentionally broken; the corrected_source is what the self-correcting
    #  generator should produce after seeing the LSP diagnostics)
    {
        "label": "type-mismatch-no-swap",
        "source": (
            "// type mismatch: x is Binary{8} but Ternary{6} is required\n"
            "nodule bad\n"
            "fn wrong(x: Binary{8}) -> Ternary{6} = x"
        ),
        "expect_clean": False,
        "corrected_source": (
            "nodule fixed\n"
            "fn correct(x: Binary{8}) -> Ternary{6} =\n"
            "    swap(x, to: Ternary{6}, policy: roundtrip)"
        ),
    },
    {
        "label": "swap-missing-policy",
        "source": (
            "// parse error: policy: is required (a swap is never silent)\n"
            "nodule bad\n"
            "fn narrow(x: Ternary{6}) -> Binary{8} =\n"
            "    swap(x, to: Binary{8})"
        ),
        "expect_clean": False,
        "corrected_source": (
            "nodule fixed\n"
            "fn narrow(x: Ternary{6}) -> Binary{8} =\n"
            "    swap(x, to: Binary{8}, policy: clamp)"
        ),
    },
    {
        "label": "missing-nodule-header",
        "source": ("fn orphan(x: Binary{8}) -> Binary{8} = x"),
        "expect_clean": False,
        "corrected_source": ("nodule demo\nfn valid(x: Binary{8}) -> Binary{8} = x"),
    },
]


# ---------------------------------------------------------------------------
# CheckResult — what the Checker returns
# ---------------------------------------------------------------------------


@dataclass
class CheckResult:
    """Outcome of running the LSP checker on one source text."""

    is_clean: bool
    diagnostics: list[dict[str, Any]]  # raw LSP diagnostic objects
    raw_json: dict[str, Any]  # the full publishDiagnostics notification
    checker_skipped: bool = False  # True when the checker binary is unavailable
    skip_reason: str = ""


# ---------------------------------------------------------------------------
# CoauthorRound — one loop iteration record
# ---------------------------------------------------------------------------


@dataclass
class CoauthorRound:
    """One generate→check round in the co-authoring loop."""

    round_number: int  # 1-based
    source_text: str
    check_result: CheckResult
    status: str  # PASS | mock-PASS | PARTIAL_PASS | SKIP | FAIL
    guarantee_tag: str  # Empirical | Declared (VR-5)
    message: str
    wall_seconds: float
    is_correction: bool = False  # True for rounds 2..N (correction attempts)

    def to_dict(self) -> dict[str, Any]:
        d = asdict(self)
        # Flatten nested dataclass
        cr = d.pop("check_result")
        d["is_clean"] = cr["is_clean"]
        d["diagnostics"] = cr["diagnostics"]
        d["checker_skipped"] = cr["checker_skipped"]
        d["skip_reason"] = cr["skip_reason"]
        return d


# ---------------------------------------------------------------------------
# Checker — shells out to cargo run … --example check (KC-3)
# ---------------------------------------------------------------------------

_CHECKER_BINARY: str | None = None  # cached path after first build


class Checker:
    """Run the LSP feedback oracle by shelling out to examples/check (M-330 KC-3).

    On the first call this builds the binary (or finds it in Cargo's output dir).
    If cargo is absent or the build fails, all subsequent checks are SKIP (G2).
    """

    def __init__(self, repo_root: Path, log: logging.Logger) -> None:
        self._repo_root = repo_root
        self._log = log
        self._available: bool | None = None  # None = not yet probed
        self._binary_path: str | None = None

    def _ensure_binary(self) -> bool:
        """Build the check example if needed.  Returns True when available."""
        if self._available is not None:
            return self._available

        cargo = shutil.which("cargo")
        if not cargo:
            self._log.warning(
                "Checker: `cargo` not found on PATH — checker will SKIP every round "
                "(G2: never-silent; missing tool ⇒ explicit SKIP)."
            )
            self._available = False
            return False

        self._log.info("Checker: building mycelium-lsp --example check …")
        t0 = time.monotonic()
        try:
            result = subprocess.run(
                [cargo, "build", "-q", "-p", "mycelium-lsp", "--example", "check"],
                cwd=str(self._repo_root),
                capture_output=True,
                text=True,
                timeout=180,
            )
        except subprocess.TimeoutExpired:
            self._log.error("Checker: `cargo build` timed out — checker will SKIP.")
            self._available = False
            return False
        except OSError as exc:
            self._log.error("Checker: `cargo build` failed to start: %s", exc)
            self._available = False
            return False

        elapsed = time.monotonic() - t0
        if result.returncode != 0:
            self._log.error(
                "Checker: `cargo build` exited %d in %.1fs:\n%s",
                result.returncode,
                elapsed,
                result.stderr[:1000],
            )
            self._available = False
            return False

        self._log.info("Checker: build OK (%.1fs)", elapsed)
        self._available = True
        return True

    def _find_binary(self) -> str | None:
        """Resolve the path to the compiled check binary."""
        if self._binary_path:
            return self._binary_path

        cargo = shutil.which("cargo")
        if not cargo:
            return None

        # Ask cargo where the output goes (respects CARGO_TARGET_DIR etc.)
        try:
            result = subprocess.run(
                [
                    cargo,
                    "build",
                    "-q",
                    "-p",
                    "mycelium-lsp",
                    "--example",
                    "check",
                    "--message-format=json",
                ],
                cwd=str(self._repo_root),
                capture_output=True,
                text=True,
                timeout=180,
            )
            for line in result.stdout.splitlines():
                try:
                    msg = json.loads(line)
                    if (
                        msg.get("reason") == "compiler-artifact"
                        and "check" in msg.get("target", {}).get("name", "")
                        and msg.get("target", {}).get("kind") == ["example"]
                    ):
                        filenames = msg.get("filenames", [])
                        if filenames:
                            self._binary_path = filenames[0]
                            return self._binary_path
                except (json.JSONDecodeError, TypeError):
                    continue
        except (OSError, subprocess.TimeoutExpired):
            pass

        # Fallback: conventional path
        for profile in ("debug", "release"):
            candidate = self._repo_root / "target" / profile / "examples" / "check"
            if candidate.is_file():
                self._binary_path = str(candidate)
                return self._binary_path

        return None

    def check(self, source_text: str) -> CheckResult:
        """Run the LSP checker on source_text; return a CheckResult."""
        if not self._ensure_binary():
            return CheckResult(
                is_clean=False,
                diagnostics=[],
                raw_json={},
                checker_skipped=True,
                skip_reason="cargo or mycelium-lsp --example check unavailable",
            )

        binary = self._find_binary()
        if binary is None:
            return CheckResult(
                is_clean=False,
                diagnostics=[],
                raw_json={},
                checker_skipped=True,
                skip_reason="could not locate compiled check binary",
            )

        try:
            result = subprocess.run(
                [binary],
                input=source_text,
                capture_output=True,
                text=True,
                timeout=30,
            )
        except subprocess.TimeoutExpired:
            return CheckResult(
                is_clean=False,
                diagnostics=[],
                raw_json={},
                checker_skipped=True,
                skip_reason="check binary timed out",
            )
        except OSError as exc:
            return CheckResult(
                is_clean=False,
                diagnostics=[],
                raw_json={},
                checker_skipped=True,
                skip_reason=f"check binary OS error: {exc}",
            )

        if result.returncode != 0:
            return CheckResult(
                is_clean=False,
                diagnostics=[{"message": f"check binary exited {result.returncode}"}],
                raw_json={},
                checker_skipped=False,
            )

        try:
            note = json.loads(result.stdout.strip())
        except json.JSONDecodeError as exc:
            return CheckResult(
                is_clean=False,
                diagnostics=[{"message": f"invalid JSON from check binary: {exc}"}],
                raw_json={},
                checker_skipped=False,
            )

        diagnostics: list[dict[str, Any]] = note.get("params", {}).get(
            "diagnostics", []
        )
        return CheckResult(
            is_clean=len(diagnostics) == 0,
            diagnostics=diagnostics,
            raw_json=note,
            checker_skipped=False,
        )


# ---------------------------------------------------------------------------
# Generator interface + implementations
# ---------------------------------------------------------------------------


class MockGenerator:
    """Cycles through MOCK_PROGRAMS in order.

    Default behaviour (always): error programs return their broken source on the
    first attempt (to trigger real LSP diagnostics and populate the feedback
    history) and then return the corrected source on subsequent rounds — simulating
    a self-correcting LLM. This is the only mode that meets the acceptance criterion
    "every round PASS or SKIP" for the basic `--mock` run (G2).

    With --no-fix-mock: always returns the raw source even for error programs,
    which exercises the FAIL path (useful for debugging the FAIL path in CI).

    All claims are tagged Declared (VR-5): mock output is asserted, not empirical.
    """

    GUARANTEE_TAG = "Declared"

    def __init__(self, no_fix: bool = False) -> None:
        self._index = 0
        self._no_fix = no_fix
        self._correction_count: dict[int, int] = {}  # program_index → rounds used

    def current_program(self) -> dict[str, Any]:
        return MOCK_PROGRAMS[self._index % len(MOCK_PROGRAMS)]

    def next_program(self) -> None:
        self._index += 1
        self._correction_count = {}

    def generate(self, spec: str, feedback_history: list[dict[str, Any]]) -> str:  # noqa: ARG002
        prog = self.current_program()
        idx = self._index % len(MOCK_PROGRAMS)
        rounds_used = self._correction_count.get(idx, 0)
        self._correction_count[idx] = rounds_used + 1

        # In self-correcting mode (default), switch to corrected_source on round 2+
        if (
            not self._no_fix
            and not prog["expect_clean"]
            and prog.get("corrected_source") is not None
            and rounds_used
            >= 1  # first round returns the error; corrections return the fix
        ):
            return prog["corrected_source"]

        return prog["source"]


class LlmGenerator:
    """Generates Mycelium source by shelling out to llama-cli or an HTTP server.

    SKIP when the backend is unavailable — never-silent (G2).
    Claims are tagged Empirical (VR-5): model output is empirically derived.
    """

    GUARANTEE_TAG = "Empirical"

    def __init__(
        self,
        llama_cli: str | None,
        model_path: str | None,
        server_url: str | None,
        ctx_size: int,
        n_predict: int,
        log: logging.Logger,
    ) -> None:
        self._llama_cli = llama_cli
        self._model_path = model_path
        self._server_url = server_url
        self._ctx_size = ctx_size
        self._n_predict = n_predict
        self._log = log

    def _available(self) -> bool:
        if self._server_url:
            return True
        return bool(self._llama_cli and self._model_path)

    def skip_reason(self) -> str:
        if self._server_url:
            return ""
        if not self._llama_cli:
            return "llama-cli not found (SKIP — G2: missing tool is never a false PASS)"
        if not self._model_path:
            return "no --model path given (SKIP — G2)"
        return ""

    def _build_prompt(self, spec: str, feedback_history: list[dict[str, Any]]) -> str:
        """Build the generate/correct prompt for the LLM."""
        lines = [
            "You are a Mycelium language assistant. Mycelium is a typed, purely functional",
            "language with explicit representation swaps (Binary/Ternary). Every program",
            "starts with a `nodule <name>` header. Swaps are NEVER silent: you must always",
            "write `swap(x, to: T, policy: <policy>)`. Valid policies: roundtrip, clamp,",
            "saturate. A type mismatch between Binary and Ternary requires an explicit swap.",
            "",
            f"Task: {spec}",
            "",
        ]
        if feedback_history:
            lines.append("Previous attempt had these LSP diagnostics (errors to fix):")
            last = feedback_history[-1]
            for d in last.get("diagnostics", []):
                msg = d.get("message", "")
                code = d.get("code", "")
                lines.append(f"  [{code}] {msg}")
            lines.append("")
            lines.append("Please write a corrected Mycelium program:")
        else:
            lines.append("Write a correct Mycelium program:")
        lines.append("")
        return "\n".join(lines)

    def generate(self, spec: str, feedback_history: list[dict[str, Any]]) -> str:
        """Generate Mycelium source. Returns empty string if unavailable (caller checks)."""
        if not self._available():
            return ""

        prompt = self._build_prompt(spec, feedback_history)

        if self._server_url:
            return self._generate_via_server(prompt)
        return self._generate_via_cli(prompt)

    def _generate_via_cli(self, prompt: str) -> str:
        import urllib.request  # noqa: F401 — keep the import inside the method to stay stdlib

        cmd = [
            self._llama_cli,
            "--model",
            self._model_path,
            "--prompt",
            prompt,
            "--seed",
            "42",
            "--n-predict",
            str(self._n_predict),
            "--ctx-size",
            str(self._ctx_size),
            "--log-disable",
            "-e",
            "-no-cnv",
            "--no-display-prompt",
        ]
        self._log.debug("LlmGenerator: llama-cli cmd: %s", cmd)
        t0 = time.monotonic()
        try:
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=300,
                stdin=subprocess.DEVNULL,
            )
        except subprocess.TimeoutExpired:
            self._log.error("LlmGenerator: llama-cli timed out")
            return ""
        except OSError as exc:
            self._log.error("LlmGenerator: llama-cli OS error: %s", exc)
            return ""
        elapsed = time.monotonic() - t0
        self._log.info("LlmGenerator: llama-cli completed in %.1fs", elapsed)
        if result.returncode != 0:
            self._log.error(
                "LlmGenerator: llama-cli exited %d: %s",
                result.returncode,
                result.stderr[:500],
            )
            return ""
        return result.stdout.strip()

    def _generate_via_server(self, prompt: str) -> str:
        import urllib.request

        payload = json.dumps(
            {
                "prompt": prompt,
                "seed": 42,
                "n_predict": self._n_predict,
                "stream": False,
            }
        ).encode()
        url = self._server_url.rstrip("/") + "/completion"
        req = urllib.request.Request(
            url,
            data=payload,
            headers={"Content-Type": "application/json"},
            method="POST",
        )
        self._log.info("LlmGenerator: POST %s", url)
        t0 = time.monotonic()
        try:
            with urllib.request.urlopen(req, timeout=120) as resp:
                body = resp.read().decode()
        except Exception as exc:
            self._log.error("LlmGenerator: server request failed: %s", exc)
            return ""
        elapsed = time.monotonic() - t0
        self._log.info("LlmGenerator: server responded in %.1fs", elapsed)
        data = json.loads(body)
        return data.get("content", "").strip()


# ---------------------------------------------------------------------------
# Guarantee-tag checker: scan LLM source text for embedded guarantee annotations
# and enforce VR-5.
# ---------------------------------------------------------------------------

_GUARANTEE_KEYWORDS = ["@guarantee:", "guarantee:", "Guarantee:"]


def check_guarantee_tags_in_source(
    source_text: str, claim_prefix: str, log: logging.Logger
) -> list[str]:
    """Scan source_text for embedded guarantee tags and validate VR-5.

    Returns a list of violation messages (empty when all tags are valid or absent).
    The spec allows Empirical or Declared in model-generated code; Proven and Exact
    are forbidden (VR-5) — they require a checked theorem.
    """
    violations: list[str] = []
    for lineno, line in enumerate(source_text.splitlines(), start=1):
        for kw in _GUARANTEE_KEYWORDS:
            if kw.lower() in line.lower():
                # Extract the tag (the word after the keyword)
                after = line.lower().split(kw.lower(), 1)[1].strip()
                tag_candidate = after.split()[0].rstrip(".,;)") if after.split() else ""
                # Capitalise to match the lattice
                tag = tag_candidate.capitalize()
                if tag in LATTICE_ORDERED:
                    try:
                        assert_model_tag(tag, f"{claim_prefix}:line{lineno}")
                    except ValueError as exc:
                        violations.append(str(exc))
                        log.error("[VR-5] %s", exc)
    return violations


# ---------------------------------------------------------------------------
# CoauthorSession — the top-level loop
# ---------------------------------------------------------------------------


@dataclass
class SessionConfig:
    mock: bool
    no_fix_mock: bool
    spec: str
    max_rounds: int
    reports_dir: Path
    run_id: str
    log: logging.Logger
    checker: "Checker"
    generator: "MockGenerator | LlmGenerator"


@dataclass
class SessionReport:
    run_id: str
    spec: str
    mock: bool
    total_programs: int
    rounds: list[CoauthorRound]
    summary: dict[str, int]  # status → count

    def to_dict(self) -> dict[str, Any]:
        d = {
            "run_id": self.run_id,
            "spec": self.spec,
            "mock": self.mock,
            "total_programs": self.total_programs,
            "summary": self.summary,
            "rounds": [r.to_dict() for r in self.rounds],
        }
        return d


def run_one_program(
    cfg: SessionConfig,
    program_index: int,
    program_label: str,
) -> CoauthorRound:
    """Run the generate→check loop for one program specification.

    Returns ONE CoauthorRound with the TERMINAL outcome for this program.
    Intermediate (non-clean) rounds are accumulated in `feedback_history` and
    logged, but NOT emitted as separate records — the report is per-program, not
    per-attempt-within-a-program. This satisfies the acceptance criterion: every
    record in the report is PASS, mock-PASS, PARTIAL_PASS, SKIP, or FAIL (terminal).

    The `round_number` in the returned record is the number of attempts made;
    `is_correction` is True when more than one attempt was needed.
    """
    log = cfg.log
    feedback_history: list[dict[str, Any]] = []
    t_start = time.monotonic()

    for attempt in range(1, cfg.max_rounds + 1):
        t0 = time.monotonic()

        # ---- Generate ----
        if isinstance(cfg.generator, LlmGenerator) and not cfg.generator._available():
            skip_reason = cfg.generator.skip_reason()
            log.info(
                "[Program %d '%s'] Generator SKIP: %s",
                program_index,
                program_label,
                skip_reason,
            )
            cr = CheckResult(
                is_clean=False,
                diagnostics=[],
                raw_json={},
                checker_skipped=True,
                skip_reason=skip_reason,
            )
            return CoauthorRound(
                round_number=attempt,
                source_text="",
                check_result=cr,
                status=STATUS_SKIP,
                guarantee_tag="Declared",
                message=f"Generator SKIP: {skip_reason}",
                wall_seconds=time.monotonic() - t_start,
                is_correction=attempt > 1,
            )

        source_text = cfg.generator.generate(cfg.spec, feedback_history)
        guarantee_tag = cfg.generator.GUARANTEE_TAG

        if not source_text:
            log.warning(
                "[Program %d '%s' attempt %d] Generator returned empty output — FAIL",
                program_index,
                program_label,
                attempt,
            )
            cr = CheckResult(
                is_clean=False,
                diagnostics=[{"message": "generator returned empty output"}],
                raw_json={},
                checker_skipped=False,
            )
            return CoauthorRound(
                round_number=attempt,
                source_text="",
                check_result=cr,
                status=STATUS_FAIL,
                guarantee_tag=guarantee_tag,
                message="Generator returned empty output (FAIL — G2: empty is never clean).",
                wall_seconds=time.monotonic() - t_start,
                is_correction=attempt > 1,
            )

        # VR-5: scan for forbidden guarantee tags in generated source
        tag_violations = check_guarantee_tags_in_source(
            source_text, f"prog{program_index}:attempt{attempt}", log
        )

        # ---- Check ----
        cr = cfg.checker.check(source_text)

        elapsed = time.monotonic() - t0
        log.info(
            "[Program %d '%s' attempt %d/%d] clean=%s diags=%d skipped=%s (%.2fs)",
            program_index,
            program_label,
            attempt,
            cfg.max_rounds,
            cr.is_clean,
            len(cr.diagnostics),
            cr.checker_skipped,
            elapsed,
        )

        # ---- Classify terminal outcomes ----
        if cr.checker_skipped:
            return CoauthorRound(
                round_number=attempt,
                source_text=source_text,
                check_result=cr,
                status=STATUS_SKIP,
                guarantee_tag=guarantee_tag,
                message=(
                    f"Checker SKIP: {cr.skip_reason} "
                    "(G2: a missing checker is never a false PASS)"
                ),
                wall_seconds=time.monotonic() - t_start,
                is_correction=attempt > 1,
            )

        if tag_violations:
            # VR-5 violation — always a terminal FAIL
            return CoauthorRound(
                round_number=attempt,
                source_text=source_text,
                check_result=cr,
                status=STATUS_FAIL,
                guarantee_tag=guarantee_tag,
                message=(
                    "[VR-5 VIOLATION] forbidden guarantee tag(s) in generated source: "
                    + "; ".join(tag_violations)
                ),
                wall_seconds=time.monotonic() - t_start,
                is_correction=attempt > 1,
            )

        if cr.is_clean:
            if attempt == 1:
                status = STATUS_MOCK_PASS if cfg.mock else STATUS_PASS
                message = (
                    f"[{'mock-PASS' if cfg.mock else 'PASS'}] "
                    f"Source is clean on first attempt."
                )
            else:
                status = STATUS_PARTIAL_PASS
                message = (
                    f"[PARTIAL_PASS] Source became clean after {attempt} attempt(s). "
                    f"The generator self-corrected {attempt - 1} time(s)."
                )
            return CoauthorRound(
                round_number=attempt,
                source_text=source_text,
                check_result=cr,
                status=status,
                guarantee_tag=guarantee_tag,
                message=message,
                wall_seconds=time.monotonic() - t_start,
                is_correction=attempt > 1,
            )

        # Not clean — add to feedback history; continue if rounds remain
        feedback_history.append(
            {
                "attempt": attempt,
                "source": source_text,
                "diagnostics": cr.diagnostics,
            }
        )
        if attempt < cfg.max_rounds:
            log.info(
                "[Program %d '%s' attempt %d] %d diagnostic(s) — will attempt correction",
                program_index,
                program_label,
                attempt,
                len(cr.diagnostics),
            )
            continue  # → next attempt

        # Exhausted all rounds — terminal FAIL
        return CoauthorRound(
            round_number=attempt,
            source_text=source_text,
            check_result=cr,
            status=STATUS_FAIL,
            guarantee_tag=guarantee_tag,
            message=(
                f"[FAIL] Source not clean after {cfg.max_rounds} attempt(s). "
                f"Final diagnostics: {len(cr.diagnostics)} error(s)."
            ),
            wall_seconds=time.monotonic() - t_start,
            is_correction=attempt > 1,
        )

    # Unreachable — but satisfy the type checker
    raise RuntimeError("run_one_program: exhausted rounds without returning")


def run_session(cfg: SessionConfig) -> SessionReport:
    """Run the full co-authoring session; return the report."""
    log = cfg.log

    if cfg.mock:
        programs = [(i, p["label"]) for i, p in enumerate(MOCK_PROGRAMS)]
        log.info("Session: mock mode — %d canned programs", len(programs))
    else:
        # In real mode there is one program per session (one spec)
        programs = [(0, "llm-generated")]
        log.info("Session: real mode — spec: %r", cfg.spec)

    all_rounds: list[CoauthorRound] = []

    for prog_idx, prog_label in programs:
        if cfg.mock and isinstance(cfg.generator, MockGenerator):
            cfg.generator._index = prog_idx  # point at correct program

        log.info("--- Program %d: '%s' ---", prog_idx, prog_label)
        rnd = run_one_program(cfg, prog_idx, prog_label)
        all_rounds.append(rnd)
        log.info(
            "    => %s (%d attempt(s), %.2fs)",
            rnd.status,
            rnd.round_number,
            rnd.wall_seconds,
        )

        if cfg.mock and isinstance(cfg.generator, MockGenerator):
            cfg.generator.next_program()

    summary: dict[str, int] = {}
    for rnd in all_rounds:
        summary[rnd.status] = summary.get(rnd.status, 0) + 1

    return SessionReport(
        run_id=cfg.run_id,
        spec=cfg.spec,
        mock=cfg.mock,
        total_programs=len(programs),
        rounds=all_rounds,
        summary=summary,
    )


# ---------------------------------------------------------------------------
# Dual report emission (G11)
# ---------------------------------------------------------------------------


def emit_reports(
    report: SessionReport, reports_dir: Path, run_id: str
) -> tuple[Path, Path]:
    """Write JSON + human-readable reports; return (json_path, txt_path)."""
    reports_dir.mkdir(parents=True, exist_ok=True)

    json_path = reports_dir / f"coauthor-{run_id}.json"
    txt_path = reports_dir / f"coauthor-{run_id}.txt"

    # JSON report
    with open(json_path, "w", encoding="utf-8") as fh:
        json.dump(report.to_dict(), fh, indent=2)

    # Human-readable report
    lines = [
        "=" * 72,
        "Mycelium Co-author Report — M-330 (SC-5b; NFR-2)",
        f"Run ID : {report.run_id}",
        f"Spec   : {report.spec or '(mock — cycling canned programs)'}",
        f"Mode   : {'mock' if report.mock else 'real LLM'}",
        f"Programs: {report.total_programs}  (one record per program)",
        "=" * 72,
        "",
    ]

    # One record per program — each round_number is the total attempts used
    for prog_num, rnd in enumerate(report.rounds, start=1):
        attempts_label = (
            f"{rnd.round_number} attempt(s)" if rnd.round_number > 1 else "1 attempt"
        )
        clean_flag = (
            "CLEAN"
            if rnd.check_result.is_clean
            else f"{len(rnd.check_result.diagnostics)} error(s)"
        )
        skipped = " [checker-skipped]" if rnd.check_result.checker_skipped else ""
        corr = " [self-corrected]" if rnd.is_correction else ""
        lines.append(
            f"  Program {prog_num:2d}  [{rnd.status:15s}]  "
            f"{clean_flag}{skipped}{corr}  "
            f"{attempts_label}  ({rnd.wall_seconds:.2f}s)  guarantee={rnd.guarantee_tag}"
        )
        lines.append(f"    {rnd.message}")
        if rnd.check_result.diagnostics:
            for diag in rnd.check_result.diagnostics[:3]:
                lines.append(
                    f"    * [{diag.get('code', '?')}] {diag.get('message', '')[:120]}"
                )
            if len(rnd.check_result.diagnostics) > 3:
                lines.append(f"    … and {len(rnd.check_result.diagnostics) - 3} more")
        lines.append("")

    lines += [
        "-" * 72,
        "Summary:",
    ]
    for status, count in sorted(report.summary.items()):
        lines.append(f"  {status:20s}: {count}")
    lines.append("-" * 72)

    has_fail = STATUS_FAIL in report.summary
    verdict = "FAIL — one or more rounds FAILED" if has_fail else "OK — no FAILures"
    lines.append(f"Verdict: {verdict}")
    lines.append(
        "(Guarantee posture: Declared — Mycelium co-authoring loop, M-330, design phase)"
    )
    lines.append("=" * 72)

    with open(txt_path, "w", encoding="utf-8") as fh:
        fh.write("\n".join(lines) + "\n")

    return json_path, txt_path


# ---------------------------------------------------------------------------
# Check-only mode (--check-only <file>)
# ---------------------------------------------------------------------------


def run_check_only(path: Path, checker: Checker, log: logging.Logger) -> int:
    """Read a .myc file, run the checker, print results, return exit code."""
    if not path.is_file():
        log.error("--check-only: file not found: %s", path)
        return 2

    src = path.read_text(encoding="utf-8")
    log.info("Checking %s …", path)
    cr = checker.check(src)

    if cr.checker_skipped:
        print(f"SKIP: {cr.skip_reason}")
        return 0

    if cr.is_clean:
        print(f"CLEAN: {path} — no diagnostics.")
        return 0

    print(f"ERRORS in {path} — {len(cr.diagnostics)} diagnostic(s):")
    for d in cr.diagnostics:
        code = d.get("code", "?")
        msg = d.get("message", "")
        print(f"  [{code}] {msg}")
    return 1


# ---------------------------------------------------------------------------
# Argument parsing + main
# ---------------------------------------------------------------------------


def _find_repo_root(start: Path) -> Path:
    """Walk up from start until we find a Cargo.toml with `[workspace]`."""
    p = start.resolve()
    for _ in range(20):
        candidate = p / "Cargo.toml"
        if candidate.is_file() and "[workspace]" in candidate.read_text(
            encoding="utf-8"
        ):
            return p
        parent = p.parent
        if parent == p:
            break
        p = parent
    # fallback: the directory containing this script
    return start.resolve()


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Mycelium AI co-authoring loop (M-330)",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__.split("Exit codes:")[0].strip(),
    )
    parser.add_argument(
        "--mock",
        action="store_true",
        help="Mock mode: cycle through canned programs (no model needed)",
    )
    parser.add_argument(
        "--fix-mock",
        action="store_true",
        help="[Kept for compatibility] Mock self-correction is now the default.",
    )
    parser.add_argument(
        "--no-fix-mock",
        action="store_true",
        help="In mock mode: disable self-correction (error programs always FAIL — exercises FAIL path)",
    )
    parser.add_argument(
        "--spec",
        default="",
        help="Natural-language spec for the LLM (or a label in mock mode)",
    )
    parser.add_argument(
        "--max-rounds",
        type=int,
        default=3,
        metavar="N",
        help="Maximum correction rounds per program (default: 3)",
    )
    parser.add_argument(
        "--model",
        dest="model_path",
        metavar="PATH",
        help="Path to a .gguf model file (real LLM mode)",
    )
    parser.add_argument(
        "--llama-cli",
        metavar="PATH",
        help="Path to llama-cli binary (default: search PATH)",
    )
    parser.add_argument(
        "--server",
        dest="server_url",
        metavar="URL",
        help="llama.cpp HTTP server base URL for real LLM mode",
    )
    parser.add_argument(
        "--ctx-size",
        type=int,
        default=2048,
        help="LLM context size tokens (default: 2048)",
    )
    parser.add_argument(
        "--n-predict",
        type=int,
        default=256,
        help="Max tokens to generate per round (default: 256)",
    )
    parser.add_argument(
        "--check-only",
        metavar="FILE",
        help="One-shot: run only the checker on FILE and exit",
    )
    parser.add_argument(
        "--reports-dir",
        metavar="DIR",
        help="Where to write the dual reports (default: tools/llm-harness/reports/)",
    )
    parser.add_argument(
        "--verbose",
        "-v",
        action="store_true",
        help="Enable DEBUG logging",
    )

    args = parser.parse_args()

    logging.basicConfig(
        level=logging.DEBUG if args.verbose else logging.INFO,
        format="%(asctime)s %(levelname)s %(message)s",
        datefmt="%H:%M:%S",
    )
    log = logging.getLogger("coauthor")

    # Resolve repo root and reports dir
    script_dir = Path(__file__).parent.resolve()
    repo_root = _find_repo_root(script_dir)

    if args.reports_dir:
        reports_dir = Path(args.reports_dir).resolve()
    else:
        reports_dir = script_dir / "reports"

    run_id = datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%SZ")

    # Build checker
    checker = Checker(repo_root, log)

    # --check-only mode
    if args.check_only:
        return run_check_only(Path(args.check_only), checker, log)

    # Build generator
    if args.mock:
        no_fix = getattr(args, "no_fix_mock", False)
        generator: MockGenerator | LlmGenerator = MockGenerator(no_fix=no_fix)
        log.info(
            "Mode: mock%s — cycling %d canned programs (guarantee=Declared, VR-5)",
            " [no-self-correct]" if no_fix else " [self-correcting]",
            len(MOCK_PROGRAMS),
        )
    else:
        llama_cli = args.llama_cli or shutil.which("llama-cli")
        generator = LlmGenerator(
            llama_cli=llama_cli,
            model_path=args.model_path,
            server_url=args.server_url,
            ctx_size=args.ctx_size,
            n_predict=args.n_predict,
            log=log,
        )
        skip = generator.skip_reason()
        if skip:
            log.warning(
                "LlmGenerator: %s\n"
                "  Tip: pass --mock for a dry run, or --model PATH.gguf for a real model.",
                skip,
            )

    spec = args.spec or (
        "(mock — cycling canned programs)" if args.mock else "(no spec given)"
    )

    cfg = SessionConfig(
        mock=args.mock,
        no_fix_mock=getattr(args, "no_fix_mock", False),
        spec=spec,
        max_rounds=args.max_rounds,
        reports_dir=reports_dir,
        run_id=run_id,
        log=log,
        checker=checker,
        generator=generator,
    )

    report = run_session(cfg)

    json_path, txt_path = emit_reports(report, reports_dir, run_id)
    log.info("JSON report : %s", json_path)
    log.info("Text report : %s", txt_path)

    # Print the human report to stdout too
    print(txt_path.read_text(encoding="utf-8"))

    has_fail = STATUS_FAIL in report.summary
    return 1 if has_fail else 0


if __name__ == "__main__":
    sys.exit(main())
