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

_REGISTRY: list[tuple[str, str, ValidationFn]] = []  # (id, description, fn)


def validation(vid: str, description: str):
    """Decorator: register a validation function."""

    def decorator(fn: ValidationFn) -> ValidationFn:
        _REGISTRY.append((vid, description, fn))
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


def _resolve_llama_cli(explicit: str | None) -> str | None:
    """Return path to llama-cli if resolvable, else None."""
    if explicit:
        if os.path.isfile(explicit) and os.access(explicit, os.X_OK):
            return explicit
        return None
    # Try common names on PATH
    for name in ("llama-cli", "llama.cpp", "main"):
        found = shutil.which(name)
        if found:
            return found
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


def _download_with_resume(url: str, dest_part: Path, log: logging.Logger) -> int:
    """Stream url → dest_part with HTTP Range resume. Returns total bytes on disk.

    Raises on a hard transfer failure (the .part file is left in place to resume).
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
            return existing
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
    return downloaded


def ensure_model(
    model_id: str,
    model_dir: Path,
    log: logging.Logger,
    *,
    allow_download: bool = True,
    url_override: str | None = None,
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

    url = url_override or _model_url(spec)
    log.info(
        "Fetching model '%s' (~%sGB, %s)",
        model_id,
        spec.get("approx_gb", "?"),
        spec.get("license", "?"),
    )
    log.info("  from %s", url)
    log.info("  into %s", dest)

    dest_part = dest.with_suffix(dest.suffix + ".part")
    try:
        _download_with_resume(url, dest_part, log)
    except Exception as exc:  # noqa: BLE001 — never-silent: surface + keep .part
        log.error(
            "Download failed: %s. Re-run to resume (any partial bytes are kept at %s).",
            exc,
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
        llama_cli = _resolve_llama_cli(llama_cli_arg)
        if llama_cli:
            log.info("Mode: REAL (llama-cli) — binary: %s", llama_cli)
        else:
            log.warning(
                "llama-cli binary not found%s. "
                "SKIPPING all model-dependent validations (skip-gracefully, G2). "
                "To run real mode: provide --llama-cli PATH or ensure llama-cli is on PATH. "
                "To suppress this and run CI-safe: use --mock.",
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
            log.info("Ensuring model '%s' in %s (idempotent)…", model_id, model_dir)
            ensured = ensure_model(
                model_id,
                model_dir,
                log,
                allow_download=not bool(getattr(args, "no_download", False)),
                url_override=getattr(args, "model_url", None),
            )
            if ensured:
                model_path = str(ensured)
            else:
                log.warning(
                    "Model could not be ensured — model-dependent validations "
                    "will SKIP (skip-gracefully, never a false pass)."
                )

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
    for vid, description, fn in _REGISTRY:
        log.info("--- Running %s: %s ---", vid, description)
        try:
            # If we have no model and are not in explicit mock mode,
            # model-dependent validations (V-01, V-02, V-04) become SKIPs.
            # V-03 always runs (pure logic).
            if (
                not mock_mode
                and not server_url
                and not llama_cli
                and vid != "V-03-tag-honesty"
            ):
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
    overall = "FAIL" if any_fail else "PASS"

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
            "Path to the llama-cli binary. "
            "If omitted, searches PATH for 'llama-cli', 'llama.cpp', 'main'."
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
    p.add_argument(
        "--list-models",
        action="store_true",
        dest="list_models",
        help="Print the registered models and the cache dir, then exit.",
    )
    return p


def main() -> None:
    parser = _build_parser()
    args = parser.parse_args()
    if getattr(args, "list_models", False):
        print(list_models_text())
        sys.exit(0)
    sys.exit(run_harness(args))


if __name__ == "__main__":
    main()
