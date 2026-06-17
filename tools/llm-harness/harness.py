#!/usr/bin/env python3
"""
Mycelium LLM-validation harness — portable, Termux-friendly (ARM/aarch64).

Design posture (non-negotiable house rules):
- NEVER-SILENT (G2, RFC-0013 I1): every failure is explicit, logged, structured.
  A missing tool/model ⇒ SKIP, never a false PASS.
- HONESTY / guarantee lattice (VR-5): model-derived claims are tagged
  Empirical or Declared — NEVER Proven or Exact. The harness enforces this.
- DUAL PROJECTION (G11): every run writes BOTH a structured JSON report AND a
  human-readable text report, as two renderers of one result object.
- Pure Python standard library only: no pip dependencies. Termux-portable.

Usage:
    python3 harness.py --mock              # dry-run: no model needed (CI/cloud)
    python3 harness.py --list-models       # show the model registry + cache dir
    python3 harness.py --ensure-model      # fetch the default model if absent, then run
    python3 harness.py --ensure-model --model-id qwen2.5-coder-14b   # desktop tier
    python3 harness.py --model PATH.gguf   # real mode via a local model file
    python3 harness.py --llama-cli PATH --model PATH.gguf
    python3 harness.py --server URL        # real mode via llama.cpp HTTP server

Exit codes:
    0   all results are PASS or SKIP (no FAILures)
    1   one or more FAILures
"""

from __future__ import annotations

import argparse
import datetime
import json
import logging
import os
import shutil
import subprocess
import sys
import time
from dataclasses import dataclass, field, asdict
from pathlib import Path
from typing import Any

# ---------------------------------------------------------------------------
# Guarantee lattice (VR-5)
# ---------------------------------------------------------------------------

LATTICE_ORDERED = ["Exact", "Proven", "Empirical", "Declared"]
# Model-derived claims may only carry these two tags.
MODEL_ALLOWED_TAGS = {"Empirical", "Declared"}


def assert_model_tag(tag: str, claim_id: str) -> None:
    """Raise ValueError if tag is stronger than allowed for model-derived claims."""
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
# Validation result
# ---------------------------------------------------------------------------

STATUS_PASS = "PASS"
STATUS_SKIP = "SKIP"
STATUS_FAIL = "FAIL"
# Special variant: mock-PASS means the validation ran against a fixture, not a
# real model. It is NEVER treated as a real PASS for model quality purposes.
STATUS_MOCK_PASS = "mock-PASS"


@dataclass
class ValidationResult:
    """One validation's outcome — the single canonical result type."""

    id: str
    status: str  # PASS | SKIP | FAIL | mock-PASS
    guarantee_tag: str | None  # Empirical | Declared | None (for pure-logic checks)
    message: str
    detail: dict[str, Any] = field(default_factory=dict)

    def is_failure(self) -> bool:
        return self.status == STATUS_FAIL

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


# ---------------------------------------------------------------------------
# Validation registry
# ---------------------------------------------------------------------------

ValidationFn = Any  # Callable[[RunContext], ValidationResult]

_REGISTRY: list[
    tuple[str, str, bool, ValidationFn]
] = []  # (id, desc, requires_model, fn)


def validation(vid: str, description: str, *, requires_model: bool = True):
    """Decorator: register a validation function.

    requires_model=True (default) means the check needs a real model/server; with
    none available it becomes an explicit SKIP (never a false PASS). Set False for
    pure-logic checks (e.g. the VR-5 tag gate) that run in every mode. Carrying this
    as metadata — instead of matching on a specific id — keeps NEVER-SILENT correct
    as validations are added.
    """

    def decorator(fn: ValidationFn) -> ValidationFn:
        _REGISTRY.append((vid, description, requires_model, fn))
        return fn

    return decorator


# ---------------------------------------------------------------------------
# Run context — carries all resolved config for the run
# ---------------------------------------------------------------------------


@dataclass
class RunContext:
    mock: bool
    llama_cli: str | None  # resolved path to llama-cli binary
    model_path: str | None  # resolved path to .gguf model
    server_url: str | None  # llama.cpp HTTP server base URL
    reports_dir: Path
    run_id: str  # ISO timestamp
    log: logging.Logger


# ---------------------------------------------------------------------------
# llama.cpp invocation helpers
# ---------------------------------------------------------------------------


def _call_llama_cli(
    ctx: RunContext,
    prompt: str,
    seed: int = 42,
    n_predict: int = 128,
    extra_args: list[str] | None = None,
) -> dict[str, Any]:
    """
    Shell out to llama-cli. Returns dict with keys:
        stdout, stderr, returncode, wall_seconds, token_counts (best-effort)
    Raises RuntimeError on non-zero exit.
    """
    cmd = [
        ctx.llama_cli,
        "--model",
        ctx.model_path,
        "--prompt",
        prompt,
        "--seed",
        str(seed),
        "--n-predict",
        str(n_predict),
        "--log-disable",  # suppress llama.cpp's internal log spam to stderr
        "-e",  # escape newlines in prompt
    ]
    if extra_args:
        cmd.extend(extra_args)

    ctx.log.debug("llama-cli cmd: %s", cmd)
    t0 = time.monotonic()
    result = subprocess.run(
        cmd,
        capture_output=True,
        text=True,
        timeout=120,
    )
    wall = time.monotonic() - t0

    # Best-effort token count from llama.cpp stderr summary lines
    prompt_tokens = None
    gen_tokens = None
    for line in result.stderr.splitlines():
        if "prompt eval time" in line.lower() and "tokens" in line.lower():
            parts = line.split()
            for i, p in enumerate(parts):
                if p in ("tokens", "token") and i > 0:
                    try:
                        prompt_tokens = int(parts[i - 1])
                    except (ValueError, IndexError):
                        pass
        if "eval time" in line.lower() and "runs" in line.lower():
            parts = line.split()
            for i, p in enumerate(parts):
                if p in ("runs",) and i > 0:
                    try:
                        gen_tokens = int(parts[i - 1])
                    except (ValueError, IndexError):
                        pass

    if result.returncode != 0:
        raise RuntimeError(
            f"llama-cli exited {result.returncode}.\nstderr: {result.stderr[:2000]}"
        )

    return {
        "stdout": result.stdout,
        "stderr": result.stderr,
        "returncode": result.returncode,
        "wall_seconds": round(wall, 3),
        "token_counts": {
            "prompt": prompt_tokens,
            "generated": gen_tokens,
            "note": "Declared — parsed from llama.cpp stderr summary; may be None if format changed",
        },
    }


def _call_server(
    ctx: RunContext,
    prompt: str,
    seed: int = 42,
    n_predict: int = 128,
) -> dict[str, Any]:
    """
    POST to llama.cpp /completion endpoint. Pure stdlib (urllib).
    Returns same shape as _call_llama_cli.
    """
    import urllib.request

    payload = json.dumps(
        {
            "prompt": prompt,
            "seed": seed,
            "n_predict": n_predict,
            "stream": False,
        }
    ).encode()

    url = ctx.server_url.rstrip("/") + "/completion"
    req = urllib.request.Request(
        url,
        data=payload,
        headers={"Content-Type": "application/json"},
        method="POST",
    )

    ctx.log.debug("POST %s", url)
    t0 = time.monotonic()
    with urllib.request.urlopen(req, timeout=120) as resp:
        body = resp.read().decode()
    wall = time.monotonic() - t0

    data = json.loads(body)
    text = data.get("content", "")
    tokens_predicted = data.get("tokens_predicted")
    tokens_evaluated = data.get("tokens_evaluated")

    return {
        "stdout": text,
        "stderr": "",
        "returncode": 0,
        "wall_seconds": round(wall, 3),
        "token_counts": {
            "prompt": tokens_evaluated,
            "generated": tokens_predicted,
            "note": "Empirical — reported by llama.cpp server /completion response",
        },
    }


def _generate(
    ctx: RunContext,
    prompt: str,
    seed: int = 42,
    n_predict: int = 128,
    extra_args: list[str] | None = None,
) -> dict[str, Any]:
    """Dispatch to the right backend (llama-cli or server)."""
    if ctx.server_url:
        return _call_server(ctx, prompt, seed=seed, n_predict=n_predict)
    return _call_llama_cli(
        ctx, prompt, seed=seed, n_predict=n_predict, extra_args=extra_args
    )


# ---------------------------------------------------------------------------
# Validation 1: model-load + deterministic-seed round-trip
# ---------------------------------------------------------------------------

_DETERMINISM_PROMPT = "Complete this sequence with exactly the next three integers and nothing else: 1, 1, 2, 3, 5,"
_DETERMINISM_FIXTURE = "8, 13, 21"


@validation(
    "V-01-determinism",
    "Fixed-seed round-trip: same prompt ⇒ same output across two runs",
)
def v01_determinism(ctx: RunContext) -> ValidationResult:
    vid = "V-01-determinism"

    if ctx.mock:
        # Mock mode: return a synthetic fixture that demonstrates the check logic.
        out_a = _DETERMINISM_FIXTURE
        out_b = _DETERMINISM_FIXTURE
        matched = out_a.strip() == out_b.strip()
        return ValidationResult(
            id=vid,
            status=STATUS_MOCK_PASS,
            guarantee_tag="Declared",
            message=(
                "[MOCK] Determinism check simulated with fixture — "
                "not a real model run. "
                f"Fixture outputs matched: {matched}"
            ),
            detail={
                "mode": "mock",
                "fixture": _DETERMINISM_FIXTURE,
                "run_a": out_a,
                "run_b": out_b,
                "matched": matched,
                "guarantee_note": (
                    "Declared — synthetic fixture, not model-derived. "
                    "Real runs would be tagged Empirical."
                ),
            },
        )

    # Real mode
    try:
        ctx.log.info("[V-01] Run A: generating with seed=42")
        res_a = _generate(ctx, _DETERMINISM_PROMPT, seed=42, n_predict=32)
        ctx.log.info("[V-01] Run B: generating with seed=42 (repeat)")
        res_b = _generate(ctx, _DETERMINISM_PROMPT, seed=42, n_predict=32)
    except Exception as exc:
        ctx.log.exception("[V-01] Generation failed")
        return ValidationResult(
            id=vid,
            status=STATUS_FAIL,
            guarantee_tag=None,
            message=f"Generation error: {exc}",
            detail={"error": str(exc)},
        )

    out_a = res_a["stdout"].strip()
    out_b = res_b["stdout"].strip()
    matched = out_a == out_b

    return ValidationResult(
        id=vid,
        status=STATUS_PASS if matched else STATUS_FAIL,
        guarantee_tag="Empirical",
        message=(
            f"Determinism check {'PASSED' if matched else 'FAILED'} "
            f"(seed=42, two runs, outputs {'matched' if matched else 'DIFFERED'})."
        ),
        detail={
            "mode": "real",
            "seed": 42,
            "run_a_output": out_a,
            "run_b_output": out_b,
            "matched": matched,
            "run_a_wall_seconds": res_a["wall_seconds"],
            "run_b_wall_seconds": res_b["wall_seconds"],
            "guarantee_note": (
                "Empirical — verified by two independent generations. "
                "Determinism is hardware/quantisation-dependent; "
                "a mismatch here is a FAIL but does not imply a broken model, "
                "only that fixed-seed reproducibility is not guaranteed on this platform."
            ),
        },
    )


# ---------------------------------------------------------------------------
# Validation 2: structured-output / JSON-projection conformance (G11)
# ---------------------------------------------------------------------------

_JSON_SCHEMA_PROMPT = (
    "Respond with ONLY a JSON object, no other text. "
    "The object must have exactly these keys: "
    '"value" (an integer), "label" (a short string), "confidence" (a float between 0 and 1). '
    'Example: {"value": 42, "label": "example", "confidence": 0.95}'
)

_JSON_FIXTURE = '{"value": 42, "label": "fixture", "confidence": 0.95}'

_EXPECTED_KEYS = {"value", "label", "confidence"}
_EXPECTED_TYPES: dict[str, type | tuple[type, ...]] = {
    "value": int,
    "label": str,
    "confidence": float,
}


def _validate_json_shape(raw: str) -> tuple[bool, str, dict[str, Any]]:
    """
    Parse raw string as JSON and validate it matches the expected schema.
    Returns (ok, error_message, parsed_object).
    """
    # Strip markdown code fences if present
    stripped = raw.strip()
    if stripped.startswith("```"):
        lines = stripped.splitlines()
        stripped = "\n".join(
            line for line in lines if not line.startswith("```")
        ).strip()

    try:
        obj = json.loads(stripped)
    except json.JSONDecodeError as exc:
        return False, f"JSON parse error: {exc}", {}

    if not isinstance(obj, dict):
        return False, f"Expected a JSON object, got {type(obj).__name__}", {}

    missing = _EXPECTED_KEYS - obj.keys()
    if missing:
        return False, f"Missing keys: {sorted(missing)}", obj

    for key, expected_type in _EXPECTED_TYPES.items():
        val = obj[key]
        # JSON numbers: Python parses integers as int, floats as float.
        # Accept int where float is expected (e.g., 1 instead of 1.0).
        if key == "confidence" and isinstance(val, int):
            val = float(val)
            obj[key] = val
        if not isinstance(val, expected_type):
            return (
                False,
                f"Key '{key}': expected {expected_type.__name__}, got {type(val).__name__}",
                obj,
            )

    if not (0.0 <= obj["confidence"] <= 1.0):
        return (
            False,
            f"'confidence' must be in [0, 1], got {obj['confidence']}",
            obj,
        )

    return True, "", obj


@validation(
    "V-02-json-projection",
    "Structured-output / JSON-projection conformance (G11)",
)
def v02_json_projection(ctx: RunContext) -> ValidationResult:
    vid = "V-02-json-projection"

    if ctx.mock:
        ok, err, parsed = _validate_json_shape(_JSON_FIXTURE)
        return ValidationResult(
            id=vid,
            status=STATUS_MOCK_PASS if ok else STATUS_FAIL,
            guarantee_tag="Declared",
            message=(
                f"[MOCK] JSON-projection check against fixture — not a real model run. "
                f"Fixture {'parsed and validated OK' if ok else 'FAILED: ' + err}"
            ),
            detail={
                "mode": "mock",
                "fixture_input": _JSON_FIXTURE,
                "parse_ok": ok,
                "parse_error": err or None,
                "parsed": parsed,
                "expected_keys": sorted(_EXPECTED_KEYS),
                "guarantee_note": (
                    "Declared — fixture response, not model-derived. "
                    "Real runs would be tagged Empirical."
                ),
            },
        )

    # Real mode
    try:
        ctx.log.info("[V-02] Prompting for structured JSON output")
        res = _generate(ctx, _JSON_SCHEMA_PROMPT, seed=42, n_predict=128)
    except Exception as exc:
        ctx.log.exception("[V-02] Generation failed")
        return ValidationResult(
            id=vid,
            status=STATUS_FAIL,
            guarantee_tag=None,
            message=f"Generation error: {exc}",
            detail={"error": str(exc)},
        )

    raw = res["stdout"].strip()
    ok, err, parsed = _validate_json_shape(raw)

    return ValidationResult(
        id=vid,
        status=STATUS_PASS if ok else STATUS_FAIL,
        guarantee_tag="Empirical",
        message=(
            f"JSON-projection {'PASSED' if ok else 'FAILED'}: "
            f"{'schema matched' if ok else err}"
        ),
        detail={
            "mode": "real",
            "raw_output": raw,
            "parse_ok": ok,
            "parse_error": err or None,
            "parsed": parsed,
            "expected_keys": sorted(_EXPECTED_KEYS),
            "wall_seconds": res["wall_seconds"],
            "guarantee_note": (
                "Empirical — validated by parsing actual model output against schema. "
                "JSON conformance is prompt/model-dependent; "
                "failures here indicate the model did not follow the structured-output instruction."
            ),
        },
    )


# ---------------------------------------------------------------------------
# Validation 3: guarantee-tag honesty check (VR-5) — runs in BOTH modes
# ---------------------------------------------------------------------------

# Test fixtures: one compliant claim, one violating claim
_TAG_FIXTURES = [
    {
        "claim_id": "test-claim-empirical",
        "tag": "Empirical",
        "content": "The model produced output X in 7 out of 10 trials.",
        "expected": "ok",
    },
    {
        "claim_id": "test-claim-declared",
        "tag": "Declared",
        "content": "User asserts this is correct (unverified).",
        "expected": "ok",
    },
    {
        "claim_id": "test-claim-proven-FORBIDDEN",
        "tag": "Proven",
        "content": "The model has been formally proven correct.",
        "expected": "FAIL",
    },
    {
        "claim_id": "test-claim-exact-FORBIDDEN",
        "tag": "Exact",
        "content": "The model output is exactly bit-perfect.",
        "expected": "FAIL",
    },
]


@validation(
    "V-03-tag-honesty",
    "Guarantee-tag honesty gate (VR-5): model-derived claims must be Empirical or Declared",
    requires_model=False,  # pure logic — runs in every mode, never SKIPs
)
def v03_tag_honesty(ctx: RunContext) -> ValidationResult:
    vid = "V-03-tag-honesty"
    # This validation runs identically in both modes — it is pure logic.

    detected_violations: list[dict[str, str]] = []
    detected_compliant: list[dict[str, str]] = []

    for fixture in _TAG_FIXTURES:
        claim_id = fixture["claim_id"]
        tag = fixture["tag"]
        expected = fixture["expected"]
        try:
            assert_model_tag(tag, claim_id)
            # No exception ⇒ tag is allowed
        except ValueError as exc:
            if expected == "FAIL":
                # This is the correct outcome — we detected the violation
                detected_violations.append(
                    {
                        "claim_id": claim_id,
                        "tag": tag,
                        "violation_message": str(exc),
                        "outcome": "correctly-rejected",
                    }
                )
                ctx.log.info(
                    "[V-03] Correctly rejected forbidden tag '%s' on claim '%s'",
                    tag,
                    claim_id,
                )
            else:
                # Unexpected — the check is broken
                detected_violations.append(
                    {
                        "claim_id": claim_id,
                        "tag": tag,
                        "violation_message": str(exc),
                        "outcome": "unexpected-rejection",
                    }
                )
            continue

        if expected == "ok":
            detected_compliant.append(
                {
                    "claim_id": claim_id,
                    "tag": tag,
                    "outcome": "correctly-allowed",
                }
            )
            ctx.log.info(
                "[V-03] Correctly allowed tag '%s' on claim '%s'", tag, claim_id
            )
        else:
            # We expected a violation but didn't get one — honesty gate failed
            detected_violations.append(
                {
                    "claim_id": claim_id,
                    "tag": tag,
                    "violation_message": f"Tag '{tag}' should have been rejected but was allowed",
                    "outcome": "missed-violation",
                }
            )
            ctx.log.error(
                "[V-03] MISSED VIOLATION: tag '%s' on claim '%s' should have been rejected",
                tag,
                claim_id,
            )

    # The gate passes only if all expected violations were correctly rejected
    # and all compliant claims were correctly allowed.
    missed = [v for v in detected_violations if v["outcome"] == "missed-violation"]
    unexpected_rejections = [
        v for v in detected_violations if v["outcome"] == "unexpected-rejection"
    ]
    correctly_rejected = [
        v for v in detected_violations if v["outcome"] == "correctly-rejected"
    ]

    all_ok = len(missed) == 0 and len(unexpected_rejections) == 0

    return ValidationResult(
        id=vid,
        status=STATUS_PASS if all_ok else STATUS_FAIL,
        guarantee_tag=None,  # pure logic, no model claim involved
        message=(
            f"Tag-honesty gate {'PASSED' if all_ok else 'FAILED'}. "
            f"Correctly rejected {len(correctly_rejected)} forbidden tag(s), "
            f"correctly allowed {len(detected_compliant)} compliant tag(s). "
            + (f"MISSED {len(missed)} violation(s)! " if missed else "")
            + (
                f"UNEXPECTED rejections: {len(unexpected_rejections)}. "
                if unexpected_rejections
                else ""
            )
        ),
        detail={
            "mode": "mock" if ctx.mock else "real",
            "note": (
                "This validation runs identically in mock and real mode — "
                "it is pure logic (no model involved)."
            ),
            "fixtures_tested": len(_TAG_FIXTURES),
            "correctly_rejected": correctly_rejected,
            "correctly_allowed": detected_compliant,
            "missed_violations": missed,
            "unexpected_rejections": unexpected_rejections,
            "vr5_rule": (
                "Model-derived claims may only carry tags in {Empirical, Declared}. "
                "Proven or Exact requires a checked theorem — "
                "a model output can never be Proven or Exact by definition."
            ),
        },
    )


# ---------------------------------------------------------------------------
# Validation 4: latency/token report
# ---------------------------------------------------------------------------

_LATENCY_PROMPT = "What is 2 + 2? Answer with just the number."
_LATENCY_FIXTURE_WALL = 0.0  # clearly 0 in mock mode as a sentinel


@validation(
    "V-04-latency-tokens",
    "Latency and token-count report per generation",
)
def v04_latency_tokens(ctx: RunContext) -> ValidationResult:
    vid = "V-04-latency-tokens"

    if ctx.mock:
        return ValidationResult(
            id=vid,
            status=STATUS_MOCK_PASS,
            guarantee_tag="Declared",
            message=(
                "[MOCK] Latency/token report — synthetic numbers. "
                "wall_seconds=0.0 is a sentinel meaning 'not measured (mock mode)'. "
                "Prompt tokens and generated tokens are fixture values."
            ),
            detail={
                "mode": "mock",
                "wall_seconds": _LATENCY_FIXTURE_WALL,
                "token_counts": {
                    "prompt": 12,
                    "generated": 3,
                    "note": "Declared — synthetic fixture values, not measured",
                },
                "output": "4",
                "guarantee_note": (
                    "Declared — all numbers are synthetic fixtures. "
                    "Real runs would be tagged Empirical."
                ),
            },
        )

    # Real mode
    try:
        ctx.log.info("[V-04] Generating latency probe")
        res = _generate(ctx, _LATENCY_PROMPT, seed=1, n_predict=16)
    except Exception as exc:
        ctx.log.exception("[V-04] Generation failed")
        return ValidationResult(
            id=vid,
            status=STATUS_FAIL,
            guarantee_tag=None,
            message=f"Generation error: {exc}",
            detail={"error": str(exc)},
        )

    return ValidationResult(
        id=vid,
        status=STATUS_PASS,
        guarantee_tag="Empirical",
        message=(
            f"Latency probe completed in {res['wall_seconds']:.3f}s. "
            f"Prompt tokens: {res['token_counts']['prompt']}, "
            f"Generated tokens: {res['token_counts']['generated']}."
        ),
        detail={
            "mode": "real",
            "wall_seconds": res["wall_seconds"],
            "token_counts": res["token_counts"],
            "output": res["stdout"].strip(),
            "guarantee_note": (
                "Empirical — wall-clock time measured by harness (monotonic clock). "
                "Token counts parsed from llama.cpp output; "
                "may be None if llama.cpp changed its log format."
            ),
        },
    )


# ---------------------------------------------------------------------------
# Report writing
# ---------------------------------------------------------------------------


def _now_iso() -> str:
    return datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%SZ")


def _write_json_report(
    path: Path,
    run_id: str,
    mode: str,
    results: list[ValidationResult],
    summary: dict[str, Any],
) -> None:
    doc = {
        "harness": "mycelium-llm-validation",
        "version": "0.1.0",
        "run_id": run_id,
        "mode": mode,
        "timestamp_utc": run_id,
        "honesty_posture": {
            "never_silent": True,
            "guarantee_lattice": LATTICE_ORDERED,
            "model_allowed_tags": sorted(MODEL_ALLOWED_TAGS),
            "vr5_rule": (
                "Model-derived claims are Empirical or Declared — NEVER Proven or Exact. "
                "A SKIP is never a PASS. mock-PASS is explicitly labelled and "
                "never treated as evidence of real model quality."
            ),
        },
        "summary": summary,
        "results": [r.to_dict() for r in results],
    }
    path.write_text(
        json.dumps(doc, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )


def _write_text_report(
    path: Path,
    run_id: str,
    mode: str,
    results: list[ValidationResult],
    summary: dict[str, Any],
) -> None:
    lines: list[str] = []
    lines.append("=" * 72)
    lines.append("  Mycelium LLM-validation harness  — human-readable projection (G11)")
    lines.append("=" * 72)
    lines.append(f"  Run ID  : {run_id}")
    lines.append(f"  Mode    : {mode}")
    lines.append("  Posture : NEVER-SILENT (G2) · VR-5 honesty · DUAL PROJECTION (G11)")
    lines.append("")
    lines.append("  Honesty posture:")
    lines.append("    · A missing tool/model ⇒ SKIP, never a false PASS")
    lines.append(
        "    · Model-derived claims: Empirical or Declared — NEVER Proven/Exact"
    )
    lines.append("    · mock-PASS = fixture run, not a real model quality signal")
    lines.append("")
    lines.append("-" * 72)
    lines.append("  Validation results")
    lines.append("-" * 72)

    for r in results:
        tag_str = f" [{r.guarantee_tag}]" if r.guarantee_tag else ""
        lines.append(f"  [{r.status:10s}]{tag_str}  {r.id}")
        lines.append(f"             {r.message}")
        lines.append("")

    lines.append("-" * 72)
    lines.append("  Summary")
    lines.append("-" * 72)
    for k, v in summary.items():
        lines.append(f"  {k:20s}: {v}")
    lines.append("")
    lines.append(
        "  NOTE: This is the human-readable projection. "
        "See companion .json for the machine projection."
    )
    lines.append("=" * 72)

    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


# ---------------------------------------------------------------------------
# Main run loop
# ---------------------------------------------------------------------------


# ---------------------------------------------------------------------------
# Binary discovery + PATH self-healing (shared by llama.cpp and the hf CLI)
# ---------------------------------------------------------------------------
#
# The #1 real-world failure on a phone (Termux) is NOT "tool missing" but
# "tool installed in a dir that isn't on PATH" — `pip --user` → ~/.local/bin,
# a hand-built llama.cpp → ~/llama.cpp/build/bin. `shutil.which` only sees
# PATH, so it reports a false "missing". We therefore (a) search the dirs
# installers/builds actually use, (b) auto-prepend a found dir to THIS
# process's PATH so the rest of the run works, and (c) optionally persist the
# fix to the user's shell rc (--fix-path). Every step is logged (G2).


def _is_termux() -> bool:
    """True on Termux/Android, where $PREFIX points into com.termux."""
    return "com.termux" in (os.environ.get("PREFIX", "") or "")


def _path_entries() -> list[str]:
    return [p for p in os.environ.get("PATH", "").split(os.pathsep) if p]


def _on_path(directory: Path) -> bool:
    rd = os.path.realpath(str(directory))
    return any(os.path.realpath(p) == rd for p in _path_entries())


def _prepend_process_path(directory: Path, log: logging.Logger | None = None) -> None:
    """Add `directory` to THIS process's PATH (in-memory) so child processes see it."""
    if not _on_path(directory):
        os.environ["PATH"] = str(directory) + os.pathsep + os.environ.get("PATH", "")
        if log is not None:
            log.debug("Prepended %s to PATH for this run.", directory)


def _shell_rc_files() -> list[Path]:
    """Likely shell rc files to persist a PATH fix into (shell-implied + common)."""
    home = Path.home()
    out: list[Path] = []
    if os.environ.get("SHELL", "").endswith("zsh"):
        out.append(home / ".zshrc")
    out.append(home / ".bashrc")  # Termux's default login shell
    out.append(home / ".profile")
    seen: set[str] = set()
    uniq: list[Path] = []
    for p in out:
        if str(p) not in seen:
            seen.add(str(p))
            uniq.append(p)
    return uniq


def persist_path_fix(directory: Path, log: logging.Logger, *, assume_yes: bool) -> bool:
    """Append `export PATH="dir:$PATH"` to a shell rc, idempotently. Returns True if applied.

    Opt-in (caller gates on --fix-path) + consent unless --yes. We only APPEND a
    clearly-marked line and skip if the dir is already referenced — never rewrite
    an existing rc.
    """
    line = f'export PATH="{directory}:$PATH"'
    marker = "# added by mycelium-llm-harness (--fix-path)"
    for rc in _shell_rc_files():
        try:
            if rc.is_file() and str(directory) in rc.read_text(encoding="utf-8"):
                log.info("PATH fix already present in %s — nothing to do.", rc)
                return True
        except OSError:
            pass
    target = _shell_rc_files()[0]
    if not _confirm(
        f"Persist PATH fix for {directory} by appending to {target}?",
        assume_yes=assume_yes,
        log=log,
    ):
        log.warning("Not persisting PATH. Add it yourself:\n    %s", line)
        return False
    try:
        with open(target, "a", encoding="utf-8") as f:
            f.write(f"\n{marker}\n{line}\n")
    except OSError as exc:
        log.error("Could not write %s: %s. Add it yourself:\n    %s", target, exc, line)
        return False
    log.info(
        "Wrote PATH fix to %s. Run `source %s` or restart your shell to pick it up.",
        target,
        target,
    )
    return True


def _note_off_path(
    found: Path,
    log: logging.Logger | None,
    *,
    fix_path: bool = False,
    assume_yes: bool = False,
) -> None:
    """A binary was found off-PATH: heal this run, advise, optionally persist."""
    d = found.parent
    if _on_path(d):
        return
    _prepend_process_path(d, log)
    if log is not None:
        log.warning(
            "%s is not on PATH. Using it for this run (PATH healed in-process). "
            'Make it permanent:\n    export PATH="%s:$PATH"   '
            "(or re-run with --fix-path to append it to your shell rc).",
            found,
            d,
        )
    if fix_path and log is not None:
        persist_path_fix(d, log, assume_yes=assume_yes)


def _search_dirs_for(names: tuple[str, ...], dirs: list[Path]) -> Path | None:
    for d in dirs:
        for name in names:
            cand = d / name
            if cand.is_file() and os.access(cand, os.X_OK):
                return cand
    return None


def _dedup_existing(dirs: list[Path]) -> list[Path]:
    seen: set[str] = set()
    out: list[Path] = []
    for d in dirs:
        k = str(d)
        if k not in seen:
            seen.add(k)
            out.append(d)
    return out


def _llama_search_dirs() -> list[Path]:
    """Dirs a hand-built or packaged llama.cpp commonly lands in (besides PATH)."""
    home = Path.home()
    dirs: list[Path] = []
    roots = [home, Path.cwd()]
    prefix = os.environ.get("PREFIX")
    if prefix:
        roots.append(Path(prefix))
        dirs.append(Path(prefix) / "bin")  # Termux `pkg install` target
    opt = os.environ.get("MYCELIUM_LLAMA_DIR")
    if opt:
        p = Path(opt).expanduser()
        dirs += [p, p / "bin", p / "build" / "bin"]
    for root in roots:
        for sub in ("llama.cpp", "llama-cpp", "llamacpp", "llama"):
            base = root / sub
            dirs += [base / "build" / "bin", base / "build", base / "bin", base]
    return _dedup_existing(dirs)


_LLAMA_BIN_NAMES = ("llama-cli", "llama", "main", "llama.cpp")


def _resolve_llama_cli(
    explicit: str | None,
    log: logging.Logger | None = None,
    *,
    fix_path: bool = False,
    assume_yes: bool = False,
) -> str | None:
    """Resolve llama-cli: explicit → PATH → known build/install dirs → shallow globs."""
    if explicit:
        if os.path.isfile(explicit) and os.access(explicit, os.X_OK):
            return explicit
        if log is not None:
            log.warning("--llama-cli is not an executable file: %s", explicit)
        return None
    for name in _LLAMA_BIN_NAMES:
        found = shutil.which(name)
        if found:
            return found
    # Off PATH: search common build/install dirs, then shallow globs.
    hit = _search_dirs_for(_LLAMA_BIN_NAMES, _llama_search_dirs())
    if hit is None:
        for root in (Path.home(), Path.cwd()):
            for pat in (
                "*/build/bin/llama-cli",
                "llama*/build/bin/llama-cli",
                "*/llama-cli",
            ):
                for cand in sorted(root.glob(pat)):
                    if cand.is_file() and os.access(cand, os.X_OK):
                        hit = cand
                        break
                if hit:
                    break
            if hit:
                break
    if hit is not None:
        _note_off_path(hit, log, fix_path=fix_path, assume_yes=assume_yes)
        return str(hit)
    return None


# ---------------------------------------------------------------------------
# Claude Code CLI discovery (same off-PATH problem: npm global bin / nvm)
# ---------------------------------------------------------------------------
#
# Not used by the validations — but the user hit the identical Termux trap
# (installed, not linked onto PATH), so --doctor / --fix-path cover it too.

_CLAUDE_BIN_NAMES = ("claude",)


def _npm_query(arg: str) -> str | None:
    """Best-effort `npm <arg>` one-liner (e.g. `config get prefix`, `root -g`)."""
    npm = shutil.which("npm")
    if not npm:
        return None
    try:
        res = subprocess.run(
            [npm, *arg.split()], capture_output=True, text=True, timeout=20
        )
    except Exception:  # noqa: BLE001 — best-effort
        return None
    val = res.stdout.strip()
    return val if (res.returncode == 0 and val) else None


def _npm_global_bin() -> Path | None:
    """Best-effort npm global bin dir via `npm config get prefix`."""
    prefix = _npm_query("config get prefix")
    if not prefix:
        return None
    cand = Path(prefix) / "bin"  # unix layout; Termux $PREFIX/bin (on PATH)
    return cand if cand.is_dir() else Path(prefix)


def _claude_package_cli() -> Path | None:
    """The Claude Code package entry (cli.js) if installed but possibly UNLINKED.

    The exact Termux failure: `npm i -g @anthropic-ai/claude-code` unpacks into
    <root>/@anthropic-ai/claude-code but the `claude` bin symlink never lands on
    PATH. We locate cli.js so --doctor can say "installed, not linked" + the fix.
    """
    roots: list[Path] = []
    r = _npm_query("root -g")
    if r:
        roots.append(Path(r))
    prefix = os.environ.get("PREFIX")
    if prefix:
        roots.append(Path(prefix) / "lib" / "node_modules")
    roots.append(Path.home() / ".npm-global" / "lib" / "node_modules")
    for root in _dedup_existing(roots):
        cli = root / "@anthropic-ai" / "claude-code" / "cli.js"
        if cli.is_file():
            return cli
    return None


def _claude_search_dirs() -> list[Path]:
    """Dirs the Claude Code CLI (npm/bun/pnpm/volta/nvm) commonly lands in."""
    home = Path.home()
    dirs: list[Path] = []
    g = _npm_global_bin()
    if g:
        dirs.append(g)
    dirs += [
        home / ".local" / "bin",
        home / ".npm-global" / "bin",
        home / ".bun" / "bin",
        home / ".volta" / "bin",
        home / ".claude" / "local",  # native/local installer
    ]
    pnpm = os.environ.get("PNPM_HOME")
    if pnpm:
        dirs.append(Path(pnpm))
    prefix = os.environ.get("PREFIX")
    if prefix:
        dirs.append(Path(prefix) / "bin")
    nvm = home / ".nvm" / "versions" / "node"
    if nvm.is_dir():
        dirs += sorted(nvm.glob("*/bin"))
    return _dedup_existing(dirs)


def _resolve_claude_cli(
    explicit: str | None = None,
    log: logging.Logger | None = None,
    *,
    fix_path: bool = False,
    assume_yes: bool = False,
) -> str | None:
    """Resolve the `claude` binary: explicit → PATH → npm/nvm/Termux dirs → globs."""
    if explicit:
        if os.path.isfile(explicit) and os.access(explicit, os.X_OK):
            return explicit
        if log is not None:
            log.warning("--claude-cli is not an executable file: %s", explicit)
        return None
    found = shutil.which("claude")
    if found:
        return found
    hit = _search_dirs_for(_CLAUDE_BIN_NAMES, _claude_search_dirs())
    if hit is None:
        for pat in (".nvm/versions/node/*/bin/claude", ".npm-global/bin/claude"):
            for cand in sorted(Path.home().glob(pat)):
                if cand.is_file() and os.access(cand, os.X_OK):
                    hit = cand
                    break
            if hit:
                break
    if hit is not None:
        _note_off_path(hit, log, fix_path=fix_path, assume_yes=assume_yes)
        return str(hit)
    return None


def _writable_path_bin_dir(
    log: logging.Logger, *, assume_yes: bool = False, fix_path: bool = False
) -> Path | None:
    """A bin dir we can symlink into that is (or will be made) on PATH.

    Prefers a dir already on PATH and writable (on Termux that's `$PREFIX/bin`),
    so a new link is immediately usable. Otherwise falls back to `~/.local/bin`,
    creating it and healing PATH (in-process now; persisted with --fix-path).
    """
    candidates: list[Path] = []
    prefix = os.environ.get("PREFIX")
    if prefix:
        candidates.append(Path(prefix) / "bin")  # Termux: already on PATH
    candidates.append(Path.home() / ".local" / "bin")
    for d in candidates:
        if d.is_dir() and _on_path(d) and os.access(d, os.W_OK):
            return d
    fallback = Path.home() / ".local" / "bin"
    try:
        fallback.mkdir(parents=True, exist_ok=True)
    except OSError as exc:
        log.error("Could not create %s for a launcher: %s", fallback, exc)
        return None
    if not _on_path(fallback):
        _prepend_process_path(fallback, log)
        if fix_path:
            persist_path_fix(fallback, log, assume_yes=assume_yes)
    return fallback


def link_claude_cli(
    pkg: Path, log: logging.Logger, *, assume_yes: bool = False, fix_path: bool = False
) -> str | None:
    """Link `claude` → the installed npm package's cli.js into a PATH bin dir.

    The Termux trap: the package unpacked but the `claude` bin symlink never
    landed on PATH. cli.js carries a `#!/usr/bin/env node` shebang, so a symlink
    to it is directly runnable. No curl|bash, no global npm mutation. Prompts
    unless --yes. Returns the new launcher path or None.
    """
    if not shutil.which("node"):
        log.warning(
            "Claude Code package is at %s but `node` is not on PATH — install "
            "Node first (Termux: pkg install nodejs), then re-run --doctor.",
            pkg,
        )
        return None
    bin_dir = _writable_path_bin_dir(log, assume_yes=assume_yes, fix_path=fix_path)
    if bin_dir is None:
        return None
    target = bin_dir / "claude"
    if not _confirm(
        f"Link `claude` → {pkg} in {bin_dir}?", assume_yes=assume_yes, log=log
    ):
        log.warning('Not linking. Do it yourself:\n    ln -s "%s" "%s"', pkg, target)
        return None
    try:
        # cli.js must be executable for the shebang launcher to work.
        mode = pkg.stat().st_mode
        pkg.chmod(mode | 0o111)
        if target.exists() or target.is_symlink():
            target.unlink()
        target.symlink_to(pkg)
    except OSError as exc:
        log.error(
            'Could not link claude: %s. Do it yourself:\n    ln -s "%s" "%s"',
            exc,
            pkg,
            target,
        )
        return None
    log.info("Linked claude: %s → %s", target, pkg)
    return str(target)


CLAUDE_NPM_PACKAGE = "@anthropic-ai/claude-code"


def install_claude_cli(
    log: logging.Logger, *, assume_yes: bool = False, fix_path: bool = False
) -> str | None:
    """Best-effort `npm install -g @anthropic-ai/claude-code`, then resolve/link.

    Honesty / supply-chain (CONTRIBUTING.md): installs the published npm package
    — never curl|bash. On Termux we first point npm's global prefix at `$PREFIX`
    so the `claude` link lands on the existing PATH. Prompts unless --yes.
    """
    npm = shutil.which("npm")
    if not npm:
        log.warning(
            "Cannot auto-install the Claude Code CLI: `npm` not found. Install "
            "Node/npm first (Termux: pkg install nodejs), then re-run --doctor."
        )
        return None
    if not _confirm(
        f"Claude Code CLI not found. Install '{CLAUDE_NPM_PACKAGE}' globally via npm now?",
        assume_yes=assume_yes,
        log=log,
    ):
        log.warning(
            "Skipping Claude Code install. Do it yourself:\n    npm install -g %s",
            CLAUDE_NPM_PACKAGE,
        )
        return None
    prefix = os.environ.get("PREFIX")
    if _is_termux() and prefix:
        # Make global bin links land on the existing PATH ($PREFIX/bin).
        try:
            subprocess.run(
                [npm, "config", "set", "prefix", prefix],
                capture_output=True,
                text=True,
                timeout=60,
            )
        except Exception as exc:  # noqa: BLE001 — best-effort, surfaced below
            log.warning("Could not set npm prefix to $PREFIX: %s", exc)
    cmd = [npm, "install", "-g", CLAUDE_NPM_PACKAGE]
    log.info("Installing Claude Code CLI: %s", " ".join(cmd))
    try:
        res = subprocess.run(cmd, capture_output=True, text=True, timeout=900)
    except Exception as exc:  # noqa: BLE001 — never-silent
        log.error("npm failed to start: %s", exc)
        return None
    if res.returncode != 0:
        log.error(
            "npm install exited %d: %s",
            res.returncode,
            (res.stderr or res.stdout or "").strip()[:400],
        )
        return None
    found = _resolve_claude_cli(None, log, fix_path=fix_path, assume_yes=assume_yes)
    if found:
        log.info("Claude Code CLI installed: %s", found)
        return found
    # Installed but the bin link missed PATH again — link it ourselves.
    pkg = _claude_package_cli()
    if pkg:
        return link_claude_cli(pkg, log, assume_yes=assume_yes, fix_path=fix_path)
    log.warning(
        "npm reported success but `claude` was not found on PATH or in the usual "
        "dirs. Restart your shell, or add npm's global bin to PATH, then re-run."
    )
    return None


# ---------------------------------------------------------------------------
# Hugging Face CLI: detect → (opt-in) install → auth check/prompt
# ---------------------------------------------------------------------------
#
# The hf CLI gives a robust, resumable, auth-aware download path (gated repos,
# CDN retries) that the stdlib urllib fallback can't match. The harness:
#   1. detects `hf` (new) or `huggingface-cli` (legacy) on PATH;
#   2. on --install-hf-cli, installs the published `huggingface_hub[cli]`
#      package — NOT via `curl … | bash` (CONTRIBUTING.md supply-chain rule:
#      no piping a remote script into a shell). The upstream one-liner is
#      printed as a reviewed manual fallback only;
#   3. checks auth (`hf auth whoami`) and, if unauthenticated, prompts to log
#      in. This is NON-FATAL: the default registry is public/ungated, so a
#      token-less run still downloads. Auth only matters for gated repos.
# Every step is explicit and logged (G2 never-silent); a missing/declined CLI
# simply falls back to the built-in stdlib downloader.

HF_INSTALL_SCRIPT_URL = "https://hf.co/cli/install.sh"
HF_PIP_PACKAGE = "huggingface_hub[cli]"


def _confirm(prompt: str, *, assume_yes: bool, log: logging.Logger) -> bool:
    """Yes/no gate. --yes auto-confirms; a non-TTY without --yes declines (safe default)."""
    if assume_yes:
        log.info("%s  [--yes: auto-confirmed]", prompt)
        return True
    if not sys.stdin.isatty():
        log.warning(
            "%s  — no TTY and --yes not set; declining (non-interactive).", prompt
        )
        return False
    try:
        ans = input(f"{prompt} [y/N] ").strip().lower()
    except EOFError:
        return False
    return ans in ("y", "yes")


def _candidate_bin_dirs() -> list[Path]:
    """Bin dirs where a pip/uv/pipx-installed `hf` lands but may NOT be on PATH.

    Termux (and `pip install --user` generally) drop console scripts in
    ~/.local/bin / $PREFIX/bin / pipx-and-uv venvs without adding them to PATH,
    so `which hf` misses a perfectly-installed CLI. We search them explicitly.
    """
    dirs: list[Path] = []
    try:
        import site
        import sysconfig

        scripts = sysconfig.get_path("scripts")
        if scripts:
            dirs.append(Path(scripts))  # this interpreter's scripts dir
        try:
            dirs.append(Path(sysconfig.get_path("scripts", scheme="posix_user")))
        except Exception:  # noqa: BLE001 — scheme may not exist
            pass
        dirs.append(Path(site.getuserbase()) / "bin")  # pip --user → ~/.local/bin
    except Exception:  # noqa: BLE001 — best-effort discovery
        pass
    home = Path.home()
    dirs.append(home / ".local" / "bin")  # pipx / uv tool symlinks
    xdg = Path(os.environ.get("XDG_DATA_HOME") or (home / ".local" / "share"))
    dirs.append(xdg / "pipx" / "venvs" / "huggingface_hub" / "bin")
    for tool in ("huggingface-hub", "huggingface_hub"):
        dirs.append(xdg / "uv" / "tools" / tool / "bin")
    pipx_home = os.environ.get("PIPX_HOME")
    if pipx_home:
        dirs.append(Path(pipx_home) / "venvs" / "huggingface_hub" / "bin")
    prefix = os.environ.get("PREFIX")  # Termux: /data/data/com.termux/files/usr
    if prefix:
        dirs.append(Path(prefix) / "bin")
    return _dedup_existing(dirs)


def _hf_module_cmd(log: logging.Logger | None = None) -> list[str] | None:
    """If huggingface_hub's CLI is importable by some Python, return a `-m` invocation.

    Last-resort fallback when no console script exists/links anywhere: the
    package itself still drives downloads via the legacy CLI module.
    """
    module = "huggingface_hub.commands.huggingface_cli"
    pythons = [sys.executable]
    for name in ("python3", "python"):
        w = shutil.which(name)
        if w and w not in pythons:
            pythons.append(w)
    for py in pythons:
        try:
            res = subprocess.run(
                [py, "-c", f"import {module}"], capture_output=True, timeout=20
            )
        except Exception:  # noqa: BLE001 — try the next interpreter
            continue
        if res.returncode == 0:
            return [py, "-m", module]
    return None


def _find_hf_off_path(
    log: logging.Logger | None = None,
    *,
    fix_path: bool = False,
    assume_yes: bool = False,
) -> tuple[list[str] | None, str | None]:
    """Search known install bin dirs for an `hf`/`huggingface-cli` not on PATH."""
    for d in _candidate_bin_dirs():
        for name, style in (("hf", "hf"), ("huggingface-cli", "legacy")):
            cand = d / name
            if cand.is_file() and os.access(cand, os.X_OK):
                _note_off_path(cand, log, fix_path=fix_path, assume_yes=assume_yes)
                return [str(cand)], style
    return None, None


def _resolve_hf_cli(
    explicit: str | None = None,
    log: logging.Logger | None = None,
    *,
    fix_path: bool = False,
    assume_yes: bool = False,
) -> tuple[list[str] | None, str | None]:
    """Return (cmd, style) for the Hugging Face CLI, or (None, None).

    cmd is an argv PREFIX — usually [path], or [python, -m, module] for the
    importable-package fallback. style is "hf" (new `hf auth …`) or "legacy"
    (`huggingface-cli …`). Searches PATH → known install dirs → `-m` fallback.
    """
    if explicit:
        if os.path.isfile(explicit) and os.access(explicit, os.X_OK):
            style = (
                "legacy" if "huggingface-cli" in os.path.basename(explicit) else "hf"
            )
            return [explicit], style
        if log is not None:
            log.warning("--hf-cli is not an executable file: %s", explicit)
        return None, None
    found = shutil.which("hf")
    if found:
        return [found], "hf"
    found = shutil.which("huggingface-cli")
    if found:
        return [found], "legacy"
    cmd, style = _find_hf_off_path(log, fix_path=fix_path, assume_yes=assume_yes)
    if cmd:
        return cmd, style
    mod = _hf_module_cmd(log)
    if mod:
        if log is not None:
            log.warning(
                "No `hf`/`huggingface-cli` executable found, but huggingface_hub is "
                "importable — driving it via `%s -m huggingface_hub…`. Install the "
                "[cli] extra (or fix PATH) to get the real `hf` command.",
                Path(mod[0]).name,
            )
        return mod, "legacy"
    return None, None


def install_hf_cli(
    log: logging.Logger, *, assume_yes: bool = False, fix_path: bool = False
) -> tuple[list[str] | None, str | None]:
    """Best-effort install of the hf CLI. Returns (cmd, style) or (None, None).

    Honesty / supply-chain (CONTRIBUTING.md): we do NOT `curl … | bash`. We
    install the published `huggingface_hub[cli]` package — pinnable, auditable,
    and the source of the `hf` command — via uv / pipx / pip, in that order.
    """
    methods: list[tuple[list[str], str]] = []
    if shutil.which("uv"):
        methods.append((["uv", "tool", "install", HF_PIP_PACKAGE], "uv tool"))
    if shutil.which("pipx"):
        methods.append((["pipx", "install", HF_PIP_PACKAGE], "pipx"))
    # On Termux, plain `pip install` lands in $PREFIX/bin (already on PATH);
    # `--user` (~/.local/bin) is the off-PATH trap. Off-Termux, prefer --user.
    pip_cmd = [sys.executable, "-m", "pip", "install"]
    if not _is_termux():
        pip_cmd.append("--user")
    pip_cmd.append(HF_PIP_PACKAGE)
    methods.append((pip_cmd, "pip" if _is_termux() else "pip --user"))

    # On Termux, pip/pipx/uv come from `pkg` (≡ `apt`); surface that so a phone
    # with no installer yet has a clear first step.
    termux_hint = (
        "\n  On Termux (pkg ≡ apt): pkg install python  # provides pip\n"
        "    pkg install pipx   # or:  pkg install uv   (then the lines above)"
        if _is_termux()
        else ""
    )
    manual = (
        "Install it yourself, then re-run:\n"
        f"    uv tool install '{HF_PIP_PACKAGE}'\n"
        f"    pipx install '{HF_PIP_PACKAGE}'\n"
        f"    pip install --user '{HF_PIP_PACKAGE}'"
        f"{termux_hint}\n"
        f"  (or the upstream script, after reviewing it: "
        f"curl -LsSf {HF_INSTALL_SCRIPT_URL} | bash)"
    )

    if not _confirm(
        f"Hugging Face CLI not found. Install '{HF_PIP_PACKAGE}' now?",
        assume_yes=assume_yes,
        log=log,
    ):
        log.warning("Skipping hf-CLI install. %s", manual)
        return None, None

    for cmd, label in methods:
        log.info("Installing hf CLI via %s: %s", label, " ".join(cmd))
        try:
            res = subprocess.run(cmd, capture_output=True, text=True, timeout=600)
        except Exception as exc:  # noqa: BLE001 — never-silent: surface + try next
            log.warning("  %s failed to start: %s", label, exc)
            continue
        if res.returncode != 0:
            log.warning(
                "  %s exited %d: %s",
                label,
                res.returncode,
                (res.stderr or res.stdout or "").strip()[:400],
            )
            continue
        cli, style = _resolve_hf_cli(log=log, fix_path=fix_path, assume_yes=assume_yes)
        if cli:
            log.info("hf CLI installed via %s: %s", label, " ".join(cli))
            return cli, style
        log.warning(
            "  %s reported success but 'hf'/'huggingface-cli' was not found, even in "
            "the usual install dirs (%s). Restart your shell, or add the install bin "
            "dir to PATH, then re-run.",
            label,
            ", ".join(str(d) for d in _candidate_bin_dirs()),
        )
    log.error("Could not install the hf CLI automatically. %s", manual)
    return None, None


def hf_auth_status(
    cmd: list[str], style: str, log: logging.Logger
) -> tuple[bool, str | None]:
    """Return (authenticated, who). Best-effort; a query failure ⇒ (False, None)."""
    sub = ["auth", "whoami"] if style == "hf" else ["whoami"]
    try:
        res = subprocess.run([*cmd, *sub], capture_output=True, text=True, timeout=30)
    except Exception as exc:  # noqa: BLE001 — never-silent
        log.warning("Could not query Hugging Face auth status: %s", exc)
        return False, None
    out = (res.stdout + "\n" + res.stderr).strip()
    if res.returncode == 0 and out and "not logged in" not in out.lower():
        return True, out.splitlines()[0].strip()
    return False, None


def hf_login(
    cmd: list[str],
    style: str,
    log: logging.Logger,
    *,
    assume_yes: bool = False,
    token: str | None = None,
) -> bool:
    """Authenticate to Hugging Face. Token (flag/env) is non-interactive; else prompt.

    Returns True on a successful login. NON-FATAL on failure/decline — public
    models still download; only gated repos need auth.
    """
    token = (
        token or os.environ.get("HF_TOKEN") or os.environ.get("HUGGING_FACE_HUB_TOKEN")
    )
    base = [*cmd, "auth", "login"] if style == "hf" else [*cmd, "login"]

    if token:
        log.info("Logging in to Hugging Face with a token from --hf-token/$HF_TOKEN…")
        try:
            res = subprocess.run(
                [*base, "--token", token], capture_output=True, text=True, timeout=60
            )
        except Exception as exc:  # noqa: BLE001 — never-silent
            log.error("Hugging Face login failed to run: %s", exc)
            return False
        if res.returncode == 0:
            log.info("Hugging Face login OK (token).")
            return True
        log.error(
            "Hugging Face login failed: %s",
            (res.stderr or res.stdout or "").strip()[:400],
        )
        return False

    instructions = (
        "Not logged in to Hugging Face. Public models (the default registry) "
        "still download without auth — this only blocks GATED repos. To "
        "authenticate:\n"
        f"    {' '.join(base)}            # interactive\n"
        "    export HF_TOKEN=hf_xxxxxxxx        # or set a token, then re-run "
        "(or pass --hf-token)"
    )

    if sys.stdin.isatty() and _confirm(
        "Log in to Hugging Face now (interactive prompt)?",
        assume_yes=assume_yes,
        log=log,
    ):
        log.info("Launching `%s` (interactive)…", " ".join(base))
        try:
            res = subprocess.run(base)  # inherit stdio so the prompt works
        except Exception as exc:  # noqa: BLE001 — never-silent
            log.error("Hugging Face login failed: %s", exc)
            return False
        return res.returncode == 0

    log.warning(instructions)
    return False


def ensure_hf_ready(
    explicit_cli: str | None,
    log: logging.Logger,
    *,
    allow_install: bool = False,
    assume_yes: bool = False,
    want_auth: bool = True,
    token: str | None = None,
    fix_path: bool = False,
) -> tuple[list[str] | None, str | None]:
    """Detect → (opt-in) install → auth-check the hf CLI. Returns (cmd, style).

    (None, None) means no usable CLI — the caller falls back to the stdlib
    downloader. Auth is checked/prompted but never fatal (public registry).
    """
    cli, style = _resolve_hf_cli(
        explicit_cli, log, fix_path=fix_path, assume_yes=assume_yes
    )
    if not cli:
        log.info(
            "Hugging Face CLI not found%s.",
            f" (looked for: {explicit_cli})"
            if explicit_cli
            else " on PATH, known install dirs, or as an importable module",
        )
        if allow_install:
            cli, style = install_hf_cli(log, assume_yes=assume_yes, fix_path=fix_path)
        else:
            log.info(
                "Pass --install-hf-cli to install '%s' automatically, or install it "
                "yourself (uv tool / pipx / pip). Falling back to the built-in "
                "stdlib downloader for now.",
                HF_PIP_PACKAGE,
            )
        if not cli:
            return None, None
    else:
        log.info(
            "Hugging Face CLI: %s (%s)",
            " ".join(cli),
            "hf" if style == "hf" else "legacy huggingface-cli",
        )

    if want_auth and style is not None:
        authed, who = hf_auth_status(cli, style, log)
        if authed:
            log.info("Hugging Face: authenticated as %s.", who)
        else:
            log.info("Hugging Face: not authenticated.")
            hf_login(cli, style, log, assume_yes=assume_yes, token=token)

    return cli, style


def _download_via_hf_cli(
    cli: list[str],
    repo: str,
    filename: str,
    model_dir: Path,
    log: logging.Logger,
) -> Path | None:
    """Fetch a single GGUF file via `hf download`. Returns the verified path or None.

    Downloads only the named file (not the whole repo), into model_dir/filename,
    then verifies the GGUF magic before promoting — same honesty bar as the
    stdlib path: a missing/truncated/non-GGUF result is an explicit error, never
    a false "present".
    """
    dest = model_dir / filename
    cmd = [*cli, "download", repo, filename, "--local-dir", str(model_dir)]
    log.info("Downloading via hf CLI: %s", " ".join(cmd))
    try:
        res = subprocess.run(
            cmd, text=True, timeout=3600
        )  # inherit stdio: live progress
    except Exception as exc:  # noqa: BLE001 — never-silent
        log.error("hf download failed to run: %s", exc)
        return None
    if res.returncode != 0:
        log.error(
            "hf download exited %d (auth/network/repo error). "
            "Falling back to the built-in downloader.",
            res.returncode,
        )
        return None
    if is_valid_gguf(dest):
        log.info(
            "Model ready (hf CLI): %s (%s)", dest, _human_bytes(dest.stat().st_size)
        )
        return dest
    log.error(
        "hf download completed but %s failed GGUF validation "
        "(missing/truncated/not a GGUF). Not promoting.",
        dest,
    )
    return None


# ---------------------------------------------------------------------------
# Model acquisition (idempotent, stdlib-only, Termux-friendly)
# ---------------------------------------------------------------------------
#
# A small registry of GGUF models sized for THIS harness's use case — seeded
# generation, structured/JSON output (G11), and instruction-following for the
# guarantee-tag honesty gate. The defaults are mobile-sized and UNGATED
# (Apache-2.0 Qwen2.5 family), so a phone can `--ensure-model` and walk away;
# bigger desktop tiers are registered for later runs on a real GPU.
#
# Honesty (G2/VR-5): the URLs/filenames below are BEST-EFFORT and may change
# upstream — a download is verified by the GGUF magic header + a clean,
# complete transfer, never assumed. A failed/partial/HTML-error fetch is an
# explicit error (the .part file is kept for resume), never a false "present".
# Override any entry with --model-url, or bypass the registry with --model PATH.

HF_RESOLVE = "https://huggingface.co/{repo}/resolve/main/{filename}"

# Each entry: repo + filename (→ HF resolve URL), tier, rough size, license,
# and the use-case note that justifies it for this harness.
# tier: "mobile" (phone / Termux / CPU) | "desktop" (discrete GPU)
MODELS: dict[str, dict[str, Any]] = {
    "qwen2.5-0.5b-instruct": {
        "repo": "Qwen/Qwen2.5-0.5B-Instruct-GGUF",
        "filename": "qwen2.5-0.5b-instruct-q4_k_m.gguf",
        "tier": "mobile",
        "approx_gb": 0.4,
        "license": "Apache-2.0",
        "use_case": "smallest/fastest smoke-test tier; weakest reasoning",
    },
    "qwen2.5-1.5b-instruct": {
        "repo": "Qwen/Qwen2.5-1.5B-Instruct-GGUF",
        "filename": "qwen2.5-1.5b-instruct-q4_k_m.gguf",
        "tier": "mobile",
        "approx_gb": 1.0,
        "license": "Apache-2.0",
        "use_case": "general instruct + JSON/structured output on a phone",
    },
    "qwen2.5-coder-1.5b": {
        "repo": "Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF",
        "filename": "qwen2.5-coder-1.5b-instruct-q4_k_m.gguf",
        "tier": "mobile",
        "approx_gb": 1.0,
        "license": "Apache-2.0",
        "use_case": (
            "DEFAULT — code + structured output; best fit for this app "
            "(Mycelium surface generation + JSON projection conformance)"
        ),
    },
    "qwen2.5-3b-instruct": {
        "repo": "Qwen/Qwen2.5-3B-Instruct-GGUF",
        "filename": "qwen2.5-3b-instruct-q4_k_m.gguf",
        "tier": "mobile",
        "approx_gb": 2.0,
        "license": "Qwen-Research (non-commercial — check before use)",
        "use_case": "stronger reasoning, still phone-feasible (slow)",
    },
    "qwen2.5-coder-3b": {
        "repo": "Qwen/Qwen2.5-Coder-3B-Instruct-GGUF",
        "filename": "qwen2.5-coder-3b-instruct-q4_k_m.gguf",
        "tier": "mobile",
        "approx_gb": 2.0,
        "license": "Qwen-Research (non-commercial — check before use)",
        "use_case": "stronger code/structured output; phone-feasible (slow)",
    },
    "qwen2.5-coder-7b": {
        "repo": "Qwen/Qwen2.5-Coder-7B-Instruct-GGUF",
        "filename": "qwen2.5-coder-7b-instruct-q4_k_m.gguf",
        "tier": "desktop",
        "approx_gb": 4.7,
        "license": "Apache-2.0",
        "use_case": "strong code model; fits 8GB+ VRAM (RTX 3090Ti / 5080)",
    },
    "qwen2.5-coder-14b": {
        "repo": "Qwen/Qwen2.5-Coder-14B-Instruct-GGUF",
        "filename": "qwen2.5-coder-14b-instruct-q4_k_m.gguf",
        "tier": "desktop",
        "approx_gb": 9.0,
        "license": "Apache-2.0",
        "use_case": "high-quality code/structured; fits 16GB (5080) / 24GB (3090Ti)",
    },
    "qwen2.5-coder-32b": {
        "repo": "Qwen/Qwen2.5-Coder-32B-Instruct-GGUF",
        "filename": "qwen2.5-coder-32b-instruct-q4_k_m.gguf",
        "tier": "desktop",
        "approx_gb": 19.8,
        "license": "Apache-2.0",
        "use_case": (
            "best open code model; ~24GB at q4_k_m (3090Ti tight / 5080 needs "
            "offload). FLAG: some repos split this into parts — verify on HF or "
            "fetch with huggingface-cli, then pass --model PATH."
        ),
    },
}

DEFAULT_MODEL_ID = "qwen2.5-coder-1.5b"

GGUF_MAGIC = b"GGUF"
# A real q4_k_m model is hundreds of MB; this floor catches an HTML 404/gated
# page saved under a .gguf name (never let that masquerade as a model).
_MIN_GGUF_BYTES = 1_000_000


def _human_bytes(n: int | None) -> str:
    if n is None:
        return "?"
    f = float(n)
    for unit in ("B", "KB", "MB", "GB", "TB"):
        if f < 1024 or unit == "TB":
            return f"{f:.1f}{unit}" if unit != "B" else f"{int(f)}B"
        f /= 1024
    return f"{f:.1f}TB"


def _model_url(spec: dict[str, Any]) -> str:
    if spec.get("url"):
        return str(spec["url"])
    return HF_RESOLVE.format(repo=spec["repo"], filename=spec["filename"])


def default_model_dir() -> Path:
    """Resolve the model cache dir (outside the repo, so models are never committed)."""
    env = os.environ.get("MYCELIUM_LLM_MODEL_DIR")
    if env:
        return Path(env).expanduser()
    xdg = os.environ.get("XDG_CACHE_HOME")
    base = Path(xdg).expanduser() if xdg else Path.home() / ".cache"
    return base / "mycelium-llm-harness" / "models"


def is_valid_gguf(path: Path) -> bool:
    """True iff path looks like a real, complete-enough GGUF (magic + size floor)."""
    try:
        if not path.is_file() or path.stat().st_size < _MIN_GGUF_BYTES:
            return False
        with open(path, "rb") as f:
            return f.read(4) == GGUF_MAGIC
    except OSError:
        return False


def expected_model_path(model_id: str, model_dir: Path) -> Path:
    """Where a given model id would live on disk (registry filename, else <id>.gguf)."""
    spec = MODELS.get(model_id)
    filename = str(spec["filename"]) if spec else f"{model_id}.gguf"
    return model_dir / filename


def find_cached_model(model_dir: Path) -> str | None:
    """Return a valid GGUF already in model_dir — the default first, else any.

    Lets real mode reuse an already-downloaded model with no --ensure-model and
    no hf round-trip (the walk-away property, post-download).
    """
    default = expected_model_path(DEFAULT_MODEL_ID, model_dir)
    if is_valid_gguf(default):
        return str(default)
    try:
        for p in sorted(model_dir.glob("*.gguf")):
            if is_valid_gguf(p):
                return str(p)
    except OSError:
        pass
    return None


def list_models_text() -> str:
    lines = [
        "Registered GGUF models (--model-id ID). Default: " + DEFAULT_MODEL_ID,
        "Cache dir: "
        + str(default_model_dir())
        + "  (override: --model-dir / $MYCELIUM_LLM_MODEL_DIR)",
        "",
    ]
    for tier in ("mobile", "desktop"):
        lines.append(f"  [{tier}]")
        for mid, spec in MODELS.items():
            if spec["tier"] != tier:
                continue
            star = " *" if mid == DEFAULT_MODEL_ID else "  "
            lines.append(f"  {star}{mid:<24} ~{spec['approx_gb']}GB  {spec['license']}")
            lines.append(f"        {spec['use_case']}")
        lines.append("")
    lines.append(
        "Honesty: URLs are best-effort; a download is verified by the GGUF magic "
        "header. Override with --model-url, or use a self-verified file via --model PATH."
    )
    return "\n".join(lines)


def _download_with_resume(
    url: str, dest_part: Path, log: logging.Logger
) -> tuple[int, int | None]:
    """Stream url → dest_part with HTTP Range resume.

    Returns (bytes_on_disk, expected_total). expected_total is the full file size
    when the server advertises it (Content-Length / Content-Range), else None.
    The caller verifies bytes_on_disk == expected_total before accepting the file,
    so a silently-truncated transfer is never promoted. Raises on a hard transfer
    failure (the .part file is left in place to resume).
    """
    import urllib.error
    import urllib.request

    headers = {"User-Agent": "mycelium-llm-harness/0 (+python-stdlib-urllib)"}
    existing = dest_part.stat().st_size if dest_part.exists() else 0
    if existing:
        headers["Range"] = f"bytes={existing}-"
        log.info("Resuming at %s", _human_bytes(existing))

    req = urllib.request.Request(url, headers=headers)
    try:
        resp = urllib.request.urlopen(req, timeout=60)  # follows redirects (HF CDN)
    except urllib.error.HTTPError as exc:
        if exc.code == 416 and existing:
            # Range not satisfiable ⇒ the .part is already the full file.
            return existing, existing
        raise

    status = getattr(resp, "status", resp.getcode())
    # Determine total expected size for progress (best-effort).
    total: int | None = None
    crange = resp.headers.get("Content-Range")
    clen = resp.headers.get("Content-Length")
    if crange and "/" in crange:
        try:
            total = int(crange.rsplit("/", 1)[1])
        except ValueError:
            total = None
    elif clen is not None:
        try:
            total = (existing + int(clen)) if status == 206 else int(clen)
        except ValueError:
            total = None

    # If the server ignored our Range (status 200), restart from scratch.
    append = existing > 0 and status == 206
    mode = "ab" if append else "wb"
    downloaded = existing if append else 0

    last_log = time.monotonic()
    with resp, open(dest_part, mode) as f:
        while True:
            chunk = resp.read(256 * 1024)
            if not chunk:
                break
            f.write(chunk)
            downloaded += len(chunk)
            now = time.monotonic()
            if now - last_log >= 5.0:
                pct = f" ({downloaded * 100 // total}%)" if total else ""
                log.info(
                    "  …%s%s%s",
                    _human_bytes(downloaded),
                    pct,
                    f" of {_human_bytes(total)}" if total else "",
                )
                last_log = now
    return downloaded, total


def ensure_model(
    model_id: str,
    model_dir: Path,
    log: logging.Logger,
    *,
    allow_download: bool = True,
    url_override: str | None = None,
    hf_cmd: list[str] | None = None,
    prefer_hf: bool = True,
) -> Path | None:
    """Idempotent: return a local GGUF path, downloading ONLY if absent/invalid.

    Returns the Path on success, or None on failure (explicit + logged — never a
    false path). Re-running is a cheap presence check (the walk-away property).
    """
    spec = MODELS.get(model_id)
    if spec is None and url_override is None:
        log.error(
            "Unknown model id '%s'. Run --list-models, or pass --model-url "
            "with any --model-id name to fetch an arbitrary GGUF.",
            model_id,
        )
        return None
    if spec is None:
        spec = {"filename": f"{model_id}.gguf", "url": url_override}

    model_dir.mkdir(parents=True, exist_ok=True)
    dest = model_dir / spec["filename"]

    if is_valid_gguf(dest):
        log.info(
            "Model already present (idempotent skip): %s (%s)",
            dest,
            _human_bytes(dest.stat().st_size),
        )
        return dest
    if dest.exists():
        log.warning("Existing file failed GGUF validation; will re-fetch: %s", dest)
    if not allow_download:
        log.warning("Model absent and --no-download set: %s. SKIP.", dest)
        return None

    log.info(
        "Fetching model '%s' (~%sGB, %s)",
        model_id,
        spec.get("approx_gb", "?"),
        spec.get("license", "?"),
    )
    log.info("  into %s", dest)

    # Preferred path: the hf CLI (robust, resumable, auth-aware). Only when we
    # have a registry repo and no explicit URL override (the CLI is repo-based).
    if prefer_hf and hf_cmd and spec.get("repo") and url_override is None:
        ensured = _download_via_hf_cli(
            hf_cmd, str(spec["repo"]), str(spec["filename"]), model_dir, log
        )
        if ensured:
            return ensured
        log.warning(
            "hf CLI download did not succeed — falling back to the built-in "
            "stdlib downloader."
        )

    url = url_override or _model_url(spec)
    log.info("  from %s", url)

    dest_part = dest.with_suffix(dest.suffix + ".part")
    try:
        downloaded, total = _download_with_resume(url, dest_part, log)
    except Exception as exc:  # noqa: BLE001 — never-silent: surface + keep .part
        log.error(
            "Download failed: %s. Re-run to resume (any partial bytes are kept at %s).",
            exc,
            dest_part,
        )
        return None

    # Completeness: if the server advertised a size, the transfer must match it.
    # A truncated body that still starts with the GGUF magic must NOT be promoted.
    if total is not None and downloaded != total:
        log.error(
            "Incomplete download: got %s of %s. Not promoting (re-run to resume): %s",
            _human_bytes(downloaded),
            _human_bytes(total),
            dest_part,
        )
        return None

    if not is_valid_gguf(dest_part):
        log.error(
            "Downloaded file failed GGUF validation — not a GGUF (truncated, or an "
            "HTML 404/gated page): %s. NOT promoting; removing it. Check the "
            "URL/filename or pass --model-url.",
            dest_part,
        )
        try:
            dest_part.unlink()
        except OSError:
            pass
        return None

    os.replace(dest_part, dest)
    log.info("Model ready: %s (%s)", dest, _human_bytes(dest.stat().st_size))
    return dest


def run_harness(args: argparse.Namespace) -> int:
    """Execute all validations. Returns exit code (0 = no FAIL)."""
    run_id = _now_iso()
    reports_dir = Path(__file__).parent / "reports"
    reports_dir.mkdir(parents=True, exist_ok=True)

    # Set up logging: both file and stderr
    log_path = reports_dir / f"{run_id}.log"
    log = logging.getLogger("mycelium.llm-harness")
    log.setLevel(logging.DEBUG)
    fmt = logging.Formatter(
        fmt="%(asctime)s  %(levelname)-8s  %(message)s",
        datefmt="%Y-%m-%dT%H:%M:%SZ",
    )
    # File handler (full trace)
    fh = logging.FileHandler(log_path, encoding="utf-8")
    fh.setLevel(logging.DEBUG)
    fh.setFormatter(fmt)
    log.addHandler(fh)
    # Stderr handler (INFO and above)
    sh = logging.StreamHandler(sys.stderr)
    sh.setLevel(logging.INFO)
    sh.setFormatter(fmt)
    log.addHandler(sh)

    log.info("=== Mycelium LLM-validation harness ===")
    log.info("Run ID: %s", run_id)

    # Determine mode
    mock_mode = bool(getattr(args, "mock", False))
    server_url: str | None = getattr(args, "server", None)
    model_path: str | None = getattr(args, "model", None)
    llama_cli_arg: str | None = getattr(args, "llama_cli", None)

    if mock_mode:
        log.info("Mode: MOCK (dry-run) — no model, fixture responses, CI-safe")
        llama_cli = None
    elif server_url:
        log.info("Mode: REAL (server) — %s", server_url)
        llama_cli = None
        if model_path:
            log.warning(
                "--model is ignored in server mode (model is loaded server-side)"
            )
    else:
        fix_path = bool(getattr(args, "fix_path", False))
        assume_yes = bool(getattr(args, "assume_yes", False))
        llama_cli = _resolve_llama_cli(
            llama_cli_arg, log, fix_path=fix_path, assume_yes=assume_yes
        )
        if llama_cli:
            log.info("Mode: REAL (llama-cli) — binary: %s", llama_cli)
        else:
            log.warning(
                "llama-cli binary not found%s (searched PATH, ~/llama.cpp/build/bin, "
                "$PREFIX/bin, $MYCELIUM_LLAMA_DIR, and shallow globs). "
                "SKIPPING all model-dependent validations (skip-gracefully, G2). "
                "To run real mode: provide --llama-cli PATH or build llama.cpp. "
                "To suppress this and run CI-safe: use --mock. Run --doctor to diagnose.",
                f" (looked for: {llama_cli_arg})" if llama_cli_arg else "",
            )

        # Idempotent model acquisition. Triggered by --ensure-model or by
        # naming a --model-id; downloads ONLY if absent/invalid. Runs even when
        # llama-cli is missing, so a phone can prefetch the model first and
        # build llama.cpp later (the cached model is reused — walk-away property).
        want_ensure = bool(getattr(args, "ensure_model", False)) or bool(
            getattr(args, "model_id", None)
        )
        if model_path is None and want_ensure:
            model_id = getattr(args, "model_id", None) or DEFAULT_MODEL_ID
            md_arg = getattr(args, "model_dir", None)
            model_dir = Path(md_arg).expanduser() if md_arg else default_model_dir()
            url_override = getattr(args, "model_url", None)

            # If the model is already on disk, there is nothing to download — so
            # do NOT spin up / install the hf CLI just to no-op. (The user's phone
            # has the model already; this avoids nagging for a tool it won't use.)
            already_present = url_override is None and is_valid_gguf(
                expected_model_path(model_id, model_dir)
            )

            prefer_hf = not bool(getattr(args, "no_hf_cli", False))
            hf_cmd = None
            if (
                prefer_hf
                and not bool(getattr(args, "no_download", False))
                and not already_present
            ):
                # Hugging Face CLI: detect → (opt-in) install → auth check/prompt.
                # Falls back to the built-in stdlib downloader when unavailable.
                hf_cmd, _ = ensure_hf_ready(
                    getattr(args, "hf_cli", None),
                    log,
                    allow_install=bool(getattr(args, "install_hf_cli", False)),
                    assume_yes=assume_yes,
                    want_auth=True,
                    token=getattr(args, "hf_token", None),
                    fix_path=fix_path,
                )
            elif already_present:
                log.info(
                    "Model already present — skipping hf-CLI setup (no download needed)."
                )

            log.info("Ensuring model '%s' in %s (idempotent)…", model_id, model_dir)
            ensured = ensure_model(
                model_id,
                model_dir,
                log,
                allow_download=not bool(getattr(args, "no_download", False)),
                url_override=url_override,
                hf_cmd=hf_cmd,
                prefer_hf=prefer_hf,
            )
            if ensured:
                model_path = str(ensured)
            else:
                log.warning(
                    "Model could not be ensured — model-dependent validations "
                    "will SKIP (skip-gracefully, never a false pass)."
                )
        elif model_path is None and not want_ensure:
            # No --model / --ensure-model, but a model may already be cached from a
            # previous run — reuse it so real mode "just works" once llama.cpp exists.
            md_arg = getattr(args, "model_dir", None)
            model_dir = Path(md_arg).expanduser() if md_arg else default_model_dir()
            cached = find_cached_model(model_dir)
            if cached:
                model_path = cached
                log.info("Using cached model (no --ensure-model needed): %s", cached)

        if not llama_cli and model_path and not server_url:
            log.warning(
                "Model path provided (%s) but no llama-cli found — "
                "model-dependent validations will SKIP.",
                model_path,
            )

    # Validate model path if in real (llama-cli) mode
    if not mock_mode and llama_cli and not server_url:
        if not model_path:
            log.warning(
                "No --model provided. "
                "Model-dependent validations will SKIP (skip-gracefully)."
            )
            llama_cli = None  # treat as unavailable
        elif not os.path.isfile(model_path):
            log.warning(
                "Model file not found: %s. Model-dependent validations will SKIP.",
                model_path,
            )
            llama_cli = None

    # Build context — if we have neither server nor cli, run as effective-skip-mode
    effective_mock = mock_mode or (not server_url and not llama_cli)
    if effective_mock and not mock_mode:
        log.info(
            "Falling back to SKIP mode for model-dependent validations "
            "(no model/binary available — use --mock for fixture mode)."
        )

    ctx = RunContext(
        mock=mock_mode,
        llama_cli=llama_cli,
        model_path=model_path,
        server_url=server_url,
        reports_dir=reports_dir,
        run_id=run_id,
        log=log,
    )

    results: list[ValidationResult] = []

    # Run all registered validations
    for vid, description, requires_model, fn in _REGISTRY:
        log.info("--- Running %s: %s ---", vid, description)
        try:
            # If we have no model and are not in explicit mock mode,
            # model-dependent validations become SKIPs. Pure-logic checks
            # (requires_model=False, e.g. V-03) always run.
            if not mock_mode and not server_url and not llama_cli and requires_model:
                result = ValidationResult(
                    id=vid,
                    status=STATUS_SKIP,
                    guarantee_tag=None,
                    message=(
                        "SKIP — no model or llama-cli available. "
                        "This is not a PASS. "
                        "Provide --model + (--llama-cli or --server) for real mode, "
                        "or use --mock for fixture mode."
                    ),
                    detail={"skip_reason": "no_model_available"},
                )
            else:
                result = fn(ctx)
        except Exception as exc:
            log.exception(
                "Unhandled exception in %s — converting to FAIL (RFC-0013 I1)", vid
            )
            result = ValidationResult(
                id=vid,
                status=STATUS_FAIL,
                guarantee_tag=None,
                message=f"Unhandled exception (never-silent rule): {exc}",
                detail={"exception": str(exc), "type": type(exc).__name__},
            )

        results.append(result)
        status_str = result.status
        log.info(
            "[%s] %s — %s",
            status_str,
            vid,
            result.message[:120] + ("..." if len(result.message) > 120 else ""),
        )

    # Compute summary
    counts: dict[str, int] = {
        STATUS_PASS: 0,
        STATUS_SKIP: 0,
        STATUS_FAIL: 0,
        STATUS_MOCK_PASS: 0,
    }
    for r in results:
        counts[r.status] = counts.get(r.status, 0) + 1

    any_fail = counts[STATUS_FAIL] > 0
    exit_code = 1 if any_fail else 0
    # Honest overall (SKIP/mock is never a real PASS). Exit code stays 0 unless a
    # real FAIL: MOCK and INCONCLUSIVE are "no failures, but nothing was actually
    # validated against a real model" — not a green light, not an error.
    if any_fail:
        overall = "FAIL"
    elif mock_mode:
        overall = "MOCK"  # fixtures only — never conflated with real PASS
    elif counts[STATUS_SKIP] > 0 or counts[STATUS_PASS] == 0:
        overall = "INCONCLUSIVE"  # something skipped, or nothing passed
    else:
        overall = "PASS"

    summary = {
        "overall": overall,
        "total": len(results),
        "pass": counts[STATUS_PASS],
        "mock_pass": counts[STATUS_MOCK_PASS],
        "skip": counts[STATUS_SKIP],
        "fail": counts[STATUS_FAIL],
        "exit_code": exit_code,
        "mode": "mock"
        if mock_mode
        else ("server" if server_url else ("real" if llama_cli else "skip")),
        "model": model_path or server_url or None,
    }

    mode_label = summary["mode"]
    log.info(
        "=== DONE: overall=%s | pass=%d mock-pass=%d skip=%d fail=%d ===",
        overall,
        counts[STATUS_PASS],
        counts[STATUS_MOCK_PASS],
        counts[STATUS_SKIP],
        counts[STATUS_FAIL],
    )

    # Write dual projection reports (G11)
    json_path = reports_dir / f"{run_id}-report.json"
    txt_path = reports_dir / f"{run_id}-report.txt"

    _write_json_report(json_path, run_id, mode_label, results, summary)
    _write_text_report(txt_path, run_id, mode_label, results, summary)

    log.info("JSON report  : %s", json_path)
    log.info("Text report  : %s", txt_path)
    log.info("Run log      : %s", log_path)

    # Also print the text report to stdout for visibility
    print(txt_path.read_text(encoding="utf-8"))

    return exit_code


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------


def _build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        description=(
            "Mycelium LLM-validation harness. "
            "Validates llama.cpp model behaviour against honesty/correctness properties. "
            "NEVER-SILENT: missing tool/model ⇒ SKIP, never false PASS."
        )
    )
    mode = p.add_mutually_exclusive_group()
    mode.add_argument(
        "--mock",
        action="store_true",
        help=(
            "Dry-run: no model needed. Exercises full plumbing with fixture responses. "
            "All model-dependent validations are labelled mock-PASS. CI/cloud-safe."
        ),
    )
    mode.add_argument(
        "--server",
        metavar="URL",
        help=(
            "llama.cpp HTTP server base URL (e.g. http://localhost:8080). "
            "Uses /completion endpoint."
        ),
    )
    p.add_argument(
        "--model",
        metavar="PATH",
        help="Path to a local .gguf model file (bypasses the model registry).",
    )
    p.add_argument(
        "--llama-cli",
        metavar="PATH",
        dest="llama_cli",
        help=(
            "Path to the llama-cli binary. If omitted, searches PATH, then common "
            "build/install dirs (~/llama.cpp/build/bin, $PREFIX/bin, $MYCELIUM_LLAMA_DIR) "
            "and shallow globs."
        ),
    )
    # --- idempotent model acquisition ---
    p.add_argument(
        "--ensure-model",
        action="store_true",
        dest="ensure_model",
        help=(
            "Check for the selected --model-id and download it ONLY if absent "
            "(idempotent; resumable). Safe to re-run; great for a phone to "
            "prefetch in the background."
        ),
    )
    p.add_argument(
        "--model-id",
        metavar="ID",
        dest="model_id",
        help=(
            f"Registered model to use/fetch (default: {DEFAULT_MODEL_ID}). "
            "Naming one implies --ensure-model. See --list-models."
        ),
    )
    p.add_argument(
        "--model-dir",
        metavar="DIR",
        dest="model_dir",
        help=(
            "Where to cache models (default: $MYCELIUM_LLM_MODEL_DIR or "
            "~/.cache/mycelium-llm-harness/models — outside the repo)."
        ),
    )
    p.add_argument(
        "--model-url",
        metavar="URL",
        dest="model_url",
        help="Override the download URL for --model-id (fetch an arbitrary GGUF).",
    )
    p.add_argument(
        "--no-download",
        action="store_true",
        dest="no_download",
        help="With --ensure-model: only check presence; never download.",
    )
    # --- Hugging Face CLI integration ---
    p.add_argument(
        "--hf-cli",
        metavar="PATH",
        dest="hf_cli",
        help=(
            "Path to the `hf` (or legacy `huggingface-cli`) binary. If omitted, "
            "searches PATH then known install dirs (~/.local/bin, $PREFIX/bin)."
        ),
    )
    p.add_argument(
        "--install-hf-cli",
        action="store_true",
        dest="install_hf_cli",
        help=(
            "If the hf CLI is missing, install 'huggingface_hub[cli]' (via "
            "uv/pipx/pip — never curl|bash). Prompts unless --yes."
        ),
    )
    p.add_argument(
        "--no-hf-cli",
        action="store_true",
        dest="no_hf_cli",
        help="Disable the hf-CLI path; use the built-in stdlib downloader only.",
    )
    p.add_argument(
        "--hf-token",
        metavar="TOKEN",
        dest="hf_token",
        help=(
            "Hugging Face token for non-interactive login (or set $HF_TOKEN). "
            "Only needed for gated repos; the default registry is public."
        ),
    )
    p.add_argument(
        "--setup-hf",
        action="store_true",
        dest="setup_hf",
        help=(
            "Set up the hf CLI (detect → install if missing → check/prompt auth), "
            "then exit. Implies --install-hf-cli."
        ),
    )
    p.add_argument(
        "-y",
        "--yes",
        action="store_true",
        dest="assume_yes",
        help="Assume yes to prompts (install/login) — for non-interactive runs.",
    )
    p.add_argument(
        "--list-models",
        action="store_true",
        dest="list_models",
        help="Print the registered models and the cache dir, then exit.",
    )
    # --- diagnostics + PATH self-healing ---
    p.add_argument(
        "--doctor",
        action="store_true",
        dest="doctor",
        help=(
            "Diagnose AND heal the environment (llama.cpp, hf CLI, Claude Code "
            "CLI, model cache): auto-install missing packages (uv/pipx/pip, npm — "
            "never curl|bash), link an unlinked CLI, and fix PATH, then exit. "
            "Mutations prompt unless --yes. Add --check-only for a read-only report."
        ),
    )
    p.add_argument(
        "--check-only",
        action="store_true",
        dest="check_only",
        help=(
            "With --doctor: read-only report — diagnose and print the fix for each "
            "miss, but install nothing and write no PATH changes. Safe to run on a "
            "phone and paste back."
        ),
    )
    p.add_argument(
        "--fix-path",
        action="store_true",
        dest="fix_path",
        help=(
            "When a binary is found off-PATH, append an `export PATH=…` line to "
            "your shell rc (idempotent; prompts unless --yes). PATH is always "
            "healed in-process for the current run regardless of this flag. "
            "(--doctor implies this unless --check-only.)"
        ),
    )
    p.add_argument(
        "--claude-cli",
        metavar="PATH",
        dest="claude_cli",
        help="Explicit path to the `claude` binary (for --doctor/--fix-path).",
    )
    return p


def _console_logger(name: str = "mycelium.llm-harness.setup") -> logging.Logger:
    """A minimal stderr logger for commands that run before run_harness()."""
    log = logging.getLogger(name)
    if not log.handlers:
        log.setLevel(logging.DEBUG)
        sh = logging.StreamHandler(sys.stderr)
        sh.setLevel(logging.INFO)
        sh.setFormatter(
            logging.Formatter(
                "%(asctime)s  %(levelname)-8s  %(message)s", "%Y-%m-%dT%H:%M:%SZ"
            )
        )
        log.addHandler(sh)
    return log


def run_doctor(args: argparse.Namespace) -> int:
    """Diagnose AND heal tool/PATH/auth state. Always exits 0.

    By default doctor is self-healing: if a required package is missing it
    installs it (hf CLI via uv/pipx/pip; Claude Code via npm — never curl|bash),
    links an installed-but-unlinked CLI, and fixes PATH (in-process now, and —
    since healing implies --fix-path — persisted to your shell rc). Every
    mutation prompts for consent unless --yes; non-interactive runs without --yes
    decline safely (never-silent, G2). Pass --check-only for a pure, read-only
    report (the classic "run it on a phone and paste it back" mode).
    """
    log = _console_logger()
    check_only = bool(getattr(args, "check_only", False))
    heal = not check_only
    # Healing implies persisting PATH fixes too ("fix the paths automatically").
    fix_path = heal or bool(getattr(args, "fix_path", False))
    assume_yes = bool(getattr(args, "assume_yes", False))
    no_hf = bool(getattr(args, "no_hf_cli", False))
    actions: list[str] = []  # what healing actually did, summarised at the end
    lines: list[str] = []

    def out(s: str = "") -> None:
        lines.append(s)

    out("=" * 72)
    out("  Mycelium LLM-harness — environment doctor")
    out("=" * 72)
    out(
        "  Mode     : "
        + (
            "CHECK-ONLY (read-only report; no installs, no PATH writes)"
            if check_only
            else "HEAL (auto-install missing packages + fix PATH; --yes to skip prompts)"
        )
    )
    out(f"  Platform : {sys.platform}{'  (Termux/Android)' if _is_termux() else ''}")
    out(f"  Python   : {sys.version.split()[0]}  ({sys.executable})")
    out(f"  PREFIX   : {os.environ.get('PREFIX', '(unset)')}")
    out(f"  HOME     : {Path.home()}")
    out("  PATH     :")
    for p in _path_entries():
        out(f"      {p}")
    out("")

    out("-" * 72)
    out("  Installers / build tools")
    out("-" * 72)
    for tool in ("uv", "pipx", "npm", "node", "git", "cmake", "make", "clang"):
        w = shutil.which(tool)
        out(f"      {tool:8s}: {'OK  ' + w if w else '–   (not found)'}")
    out(f"      {'pip':8s}: OK  {sys.executable} -m pip")
    out("")

    out("-" * 72)
    out("  llama.cpp (llama-cli)")
    out("-" * 72)
    llama = _resolve_llama_cli(
        getattr(args, "llama_cli", None), log, fix_path=fix_path, assume_yes=assume_yes
    )
    if llama:
        out(f"      FOUND: {llama}")
        try:
            v = subprocess.run(
                [llama, "--version"], capture_output=True, text=True, timeout=20
            )
            blob = (v.stderr or v.stdout).strip()
            out(f"      version: {blob.splitlines()[0] if blob else '(no output)'}")
        except Exception as exc:  # noqa: BLE001
            out(f"      version: (could not run --version: {exc})")
    else:
        out("      NOT FOUND. Searched PATH +:")
        for d in _llama_search_dirs():
            out(f"          {d}")
        out("      Fix: build llama.cpp (README Termux steps), pass --llama-cli PATH,")
        out("           or set $MYCELIUM_LLAMA_DIR to its directory.")
    out("")

    out("-" * 72)
    out("  Hugging Face CLI")
    out("-" * 72)
    cmd, style = _resolve_hf_cli(
        getattr(args, "hf_cli", None), log, fix_path=fix_path, assume_yes=assume_yes
    )
    if not cmd and heal and not no_hf:
        out("      NOT FOUND — auto-installing '%s'…" % HF_PIP_PACKAGE)
        cmd, style = install_hf_cli(log, assume_yes=assume_yes, fix_path=fix_path)
        if cmd:
            actions.append(f"Installed Hugging Face CLI ('{HF_PIP_PACKAGE}').")
    if cmd:
        out(f"      FOUND: {' '.join(cmd)}  (style: {style})")
        authed, who = hf_auth_status(cmd, style, log)
        out(
            f"      auth : {'authenticated as ' + str(who) if authed else 'NOT authenticated (fine for public models)'}"
        )
    else:
        out("      NOT FOUND. Searched PATH + these bin dirs:")
        for d in _candidate_bin_dirs():
            out(f"          {d}")
        if no_hf:
            out("      (--no-hf-cli set: skipping install; stdlib downloader is used.)")
        elif check_only:
            out(
                f"      Fix: --setup-hf  (installs '{HF_PIP_PACKAGE}' via uv/pipx/pip)."
            )
        else:
            out(
                f"      Could not auto-install '{HF_PIP_PACKAGE}' (declined or no installer)."
            )
            out("      Install an installer first (uv/pipx/pip), then re-run --doctor.")
    out("")

    out("-" * 72)
    out("  Claude Code CLI (claude)")
    out("-" * 72)
    claude = _resolve_claude_cli(
        getattr(args, "claude_cli", None),
        log,
        fix_path=fix_path,
        assume_yes=assume_yes,
    )
    if not claude and heal:
        pkg = _claude_package_cli()
        if pkg:
            out(f"      INSTALLED BUT NOT LINKED ({pkg}) — auto-linking…")
            linked = link_claude_cli(pkg, log, assume_yes=assume_yes, fix_path=fix_path)
            if linked:
                claude = linked
                actions.append("Linked the installed Claude Code CLI onto PATH.")
        else:
            out("      NOT FOUND — auto-installing via npm…")
            installed = install_claude_cli(
                log, assume_yes=assume_yes, fix_path=fix_path
            )
            if installed:
                claude = installed
                actions.append("Installed the Claude Code CLI via npm.")
    if claude:
        out(f"      FOUND: {claude}")
        try:
            v = subprocess.run(
                [claude, "--version"], capture_output=True, text=True, timeout=20
            )
            blob = (v.stdout or v.stderr).strip()
            out(f"      version: {blob.splitlines()[0] if blob else '(no output)'}")
        except OSError as exc:
            # e.g. "Exec format error" — a wrong-arch/corrupt binary, NOT a PATH
            # miss. Auto-install can't fix arch; surface the precise reinstall.
            out(f"      version: (could not run --version: {exc})")
            out(
                "      This looks like a wrong-arch or corrupt binary, not a PATH issue."
            )
            out("      Fix: reinstall for this CPU —")
            out(
                f"          npm uninstall -g {CLAUDE_NPM_PACKAGE}; npm install -g {CLAUDE_NPM_PACKAGE}"
            )
            out(
                "          (On Termux: pkg reinstall nodejs first if node itself errors.)"
            )
        except Exception as exc:  # noqa: BLE001 — never-silent
            out(f"      version: (could not run --version: {exc})")
    else:
        pkg = _claude_package_cli()
        if pkg:
            node = shutil.which("node") or "node"
            out(f"      INSTALLED BUT NOT LINKED: package at {pkg}")
            out(
                "      The npm package is there; the `claude` symlink never landed on PATH."
            )
            out("      Fix (one of):")
            out(f'          ln -s "{pkg}" "$PREFIX/bin/claude" && chmod +x "{pkg}"')
            out(
                '          npm config set prefix "$PREFIX" && npm install -g @anthropic-ai/claude-code'
            )
            out(f"          # or just run it directly:  {node} {pkg}")
        else:
            out("      NOT FOUND. Searched PATH +:")
            for d in _claude_search_dirs():
                out(f"          {d}")
            out(
                "      Fix: npm install -g @anthropic-ai/claude-code, and put npm's global"
            )
            out(
                '           bin on PATH. On Termux: npm config set prefix "$PREFIX" first,'
            )
            out("           so links land on the existing PATH.")
    out("")

    out("-" * 72)
    out("  Model cache")
    out("-" * 72)
    md_arg = getattr(args, "model_dir", None)
    md = Path(md_arg).expanduser() if md_arg else default_model_dir()
    spec = MODELS[DEFAULT_MODEL_ID]
    dest = md / str(spec["filename"])
    out(f"      dir: {md}")
    if is_valid_gguf(dest):
        out(
            f"      default model present: {dest.name} ({_human_bytes(dest.stat().st_size)})"
        )
    elif heal and _confirm(
        f"Default model absent. Download {dest.name} (~GB) now?",
        assume_yes=assume_yes,
        log=log,
    ):
        got = ensure_model(
            DEFAULT_MODEL_ID,
            md,
            log,
            allow_download=True,
            hf_cmd=(cmd if not no_hf else None),
            prefer_hf=not no_hf,
        )
        if got and is_valid_gguf(got):
            out(
                f"      default model fetched: {got.name} ({_human_bytes(got.stat().st_size)})"
            )
            actions.append(f"Downloaded the default model ({got.name}).")
        else:
            out(
                f"      default model still absent: {dest.name}  (download failed — see log)"
            )
    else:
        out(f"      default model absent : {dest.name}  (fetch with --ensure-model)")
    out("")

    out("-" * 72)
    if check_only:
        out("  Result: check-only — no changes made.")
        out(
            "  Re-run without --check-only to auto-install missing packages + fix PATH,"
        )
        out("  or with --fix-path to only persist an off-PATH binary to your shell rc.")
    elif actions:
        out("  Healed:")
        for a in actions:
            out(f"      ✓ {a}")
        out("  Open a new shell (or `source` your rc) so persisted PATH fixes apply.")
    else:
        out("  Nothing to heal — everything required was already present, or fixes")
        out("  were declined/unavailable (see the per-tool notes above).")
    out("=" * 72)
    print("\n".join(lines))
    return 0


def _dispatch(args: argparse.Namespace) -> int:
    if getattr(args, "list_models", False):
        print(list_models_text())
        return 0
    if getattr(args, "doctor", False):
        return run_doctor(args)
    if getattr(args, "setup_hf", False):
        # Standalone: detect → install (opt-in implied) → auth check/prompt, then exit.
        log = _console_logger()
        cli, _ = ensure_hf_ready(
            getattr(args, "hf_cli", None),
            log,
            allow_install=True,  # --setup-hf is an explicit request to install
            assume_yes=bool(getattr(args, "assume_yes", False)),
            want_auth=True,
            token=getattr(args, "hf_token", None),
            fix_path=bool(getattr(args, "fix_path", False)),
        )
        return 0 if cli else 1
    return run_harness(args)


def main() -> None:
    args = _build_parser().parse_args()
    try:
        code = _dispatch(args)
    except BrokenPipeError:
        code = 0  # downstream pager/pipe closed — not our error
    # Flush explicitly: on some platforms (e.g. Termux/Android) an unflushed
    # buffer at interpreter teardown can surface as a spurious "Aborted".
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.flush()
        except (BrokenPipeError, ValueError, OSError):
            pass
    sys.exit(code)


if __name__ == "__main__":
    main()
