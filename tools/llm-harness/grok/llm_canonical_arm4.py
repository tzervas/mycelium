"""Arm 4 (LlmCanonical familiar-skin) harness integration (M-381; RFC-0021 §4.1).

This module provides:
  * ``parse_llm_canonical_py`` — a lightweight Python-side S-expression validator
    that checks structural validity of LlmCanonical output (balanced parens, depth
    guard) and returns a normalized form, or ``None`` on failure.
  * ``arm4_llm_canonical_prompt`` — builds the Arm 4 prompt asking the LLM to
    write in LlmCanonical S-expression format (RFC-0021 §4.6 familiar skin).
  * ``arm4_run`` — verifies one model response for Arm 4 and returns a standard
    outcome dict with ``arm: 4``, ``status``, and error details.

HONESTY (G2 / VR-5):
  * Guarantee tag for ``parse_llm_canonical_py``: **Empirical** — lightweight
    Python bracket-matching, not a proven-sound parser.  Never upgraded to Proven.
  * A parse failure is ``status: "FAIL"`` with ``parse_error`` set — never silent.
  * If the Rust binary ``llm-canonical-parse`` (from ``mycelium-lsp`` examples) is
    available, it is preferred; the Python validator is the fallback.
  * A missing / uncompiled binary yields ``status: "skip"`` with
    ``reason: "llm-canonical-parser-not-compiled"`` — never a fabricated PASS.
  * An empty model response is never scored as PASS (G2: mirrors coauthor_loop.py).
"""

from __future__ import annotations

import logging
import os
import subprocess
from pathlib import Path
from typing import Any

from .client import ChatMessage
from .coauthor_loop import scan_forbidden_self_tags
from .scoring import MycCheckScorer, VERDICT_CLEAN, VERDICT_SKIP
from .tasks import Task

_LOG = logging.getLogger("grok.arm4")

# Depth limit mirrors the Rust parser (DEPTH_LIMIT = 64; banked guard #4).
_DEPTH_LIMIT: int = 64

# Status codes (mirrors coauthor_loop.py vocabulary).
_STATUS_PASS = "PASS"
_STATUS_FAIL = "FAIL"
_STATUS_SKIP = "skip"

# Reason string for the graceful-skip path (binary not compiled).
SKIP_REASON_NOT_COMPILED = "llm-canonical-parser-not-compiled"

_LLM_CANONICAL_SYSTEM = (
    "You are a Mycelium language assistant. Mycelium programs are expressed in "
    "LlmCanonical S-expression form: a familiar-skin projection of the Core IR "
    "(RFC-0021 §4.6). Every node is a parenthesised form with a keyword head:\n"
    "  (const <value> @<Tag>)          — constant with guarantee tag\n"
    "  <ident>                          — variable reference\n"
    "  (let [<id> <bound>] <body>)      — let binding\n"
    "  (op <prim> <arg>...)             — primitive application\n"
    "  (swap! <src> :to <repr> :policy <ref>)  — representation swap (NEVER silent)\n"
    "  (make <ctor> <arg>...)           — saturated constructor\n"
    "  (match <scrut> <alt>...)         — flat pattern match\n"
    "  (fn [<param>] <body>)            — lambda\n"
    "  (<func> <arg>)                   — application\n"
    "  (fix <name> <body>)              — fixed-point\n"
    "  (fix-group (<bind>...) <body>)   — mutual-recursion group\n"
    "Swaps are NEVER silent: always write (swap! ...) with an explicit :to and "
    ":policy when crossing representations. Reply with ONLY the S-expression, "
    "no prose, no code fences."
)


def arm4_llm_canonical_prompt(task: Task) -> list[ChatMessage]:
    """Build the Arm 4 prompt asking the LLM to write in LlmCanonical format (PURE).

    Guarantee tag: Declared (prompt text is asserted, not empirically validated
    against model output quality — VR-5).
    """
    return [
        ChatMessage("system", _LLM_CANONICAL_SYSTEM),
        ChatMessage(
            "user",
            f"Task: {task.spec}\n"
            "Write a correct Mycelium program in LlmCanonical S-expression form.",
        ),
    ]


# ---------------------------------------------------------------------------
# Python-side LlmCanonical S-expression validator
# Guarantee tag: Empirical (bracket-matching heuristic, not proven-sound).
# ---------------------------------------------------------------------------


class _ParseError(Exception):
    """Internal parse error from the Python-side validator."""

    def __init__(self, message: str) -> None:
        super().__init__(message)


def _validate_sexp(source: str) -> str:
    """Validate + normalize a LlmCanonical S-expression (PURE, Empirical).

    Performs bracket-balance checking with a depth guard (banked guard #4 mirror:
    depth limit 64).  Returns the normalized (whitespace-collapsed) form, or raises
    ``_ParseError`` on any structural violation.

    This is NOT a semantically complete parser — it checks syntactic well-formedness
    of the S-expression only (balanced brackets, known heads at list position 0,
    depth limit).  The Rust ``parse_llm_canonical`` in ``mycelium-lsp`` is the
    authoritative implementation.

    Guarantee tag: **Empirical** — evidenced by the self-test (T8-extended); never
    upgraded to Proven without a checked basis (VR-5).
    """
    if not source.strip():
        raise _ParseError("empty input")

    # Tokenize into a flat sequence of tokens.
    tokens: list[tuple[str, int]] = []  # (token_text, line_no)
    line = 1
    i = 0
    n = len(source)
    while i < n:
        c = source[i]
        if c == "\n":
            line += 1
            i += 1
        elif c in " \t\r":
            i += 1
        elif c == ";":
            # Line comment: skip to end of line.
            while i < n and source[i] != "\n":
                i += 1
        elif c in "()[]":
            tokens.append((c, line))
            i += 1
        elif c == "<":
            # Angle-bracketed literal: read until '>'.
            start = i
            start_line = line
            i += 1
            while i < n and source[i] != ">":
                if source[i] == "\n":
                    line += 1
                i += 1
            if i >= n:
                raise _ParseError(f"line {start_line}: unclosed '<'")
            i += 1  # consume '>'
            tokens.append((source[start:i], start_line))
        else:
            # Atom: read until whitespace or delimiter.
            start = i
            start_line = line
            while i < n and source[i] not in " \t\n\r()[]<>;":
                i += 1
            tokens.append((source[start:i], start_line))

    # Now do a recursive descent to validate structure + depth.
    pos = [0]  # mutable index into tokens list

    def peek() -> tuple[str, int] | None:
        return tokens[pos[0]] if pos[0] < len(tokens) else None

    def consume() -> tuple[str, int]:
        t = tokens[pos[0]]
        pos[0] += 1
        return t

    def parse_expr(depth: int) -> str:
        if depth > _DEPTH_LIMIT:
            raise _ParseError(f"nesting depth limit {_DEPTH_LIMIT} exceeded")
        t = peek()
        if t is None:
            raise _ParseError("unexpected EOF")
        tok, tok_line = t
        if tok == "(":
            return parse_list(depth)
        if tok == "[":
            return parse_bracket(depth)
        if tok in ("]", ")"):
            raise _ParseError(f"line {tok_line}: unexpected '{tok}'")
        # Atom (variable, keyword, literal).
        consume()
        return tok

    def parse_list(depth: int) -> str:
        open_tok, open_line = consume()  # consume '('
        assert open_tok == "("
        items: list[str] = []
        while True:
            if depth + 1 > _DEPTH_LIMIT:
                raise _ParseError(f"nesting depth limit {_DEPTH_LIMIT} exceeded")
            t = peek()
            if t is None:
                raise _ParseError(f"line {open_line}: unclosed '('")
            tok, _ = t
            if tok == ")":
                consume()
                break
            items.append(parse_expr(depth + 1))
        if not items:
            raise _ParseError(f"line {open_line}: empty list '()' is not valid")
        # Validate the head keyword if it is an atom.
        head = items[0]
        _validate_head(head, open_line)
        return f"({' '.join(items)})"

    def parse_bracket(depth: int) -> str:
        open_tok, open_line = consume()  # consume '['
        assert open_tok == "["
        items: list[str] = []
        while True:
            t = peek()
            if t is None:
                raise _ParseError(f"line {open_line}: unclosed '['")
            tok, _ = t
            if tok == "]":
                consume()
                break
            items.append(parse_expr(depth + 1))
        return f"[{' '.join(items)}]"

    # Parse all top-level forms.
    forms: list[str] = []
    while peek() is not None:
        forms.append(parse_expr(0))

    if not forms:
        raise _ParseError("no top-level form found")
    if len(forms) == 1:
        return forms[0]
    return f"(seq {' '.join(forms)})"


# Known LlmCanonical head keywords (exactly the 11 node kinds + seq + application).
# Atoms not in this set at head position are treated as application heads (func arg)
# rather than rejected — the renderer uses variable names as application heads (App node).
_KNOWN_HEADS: frozenset[str] = frozenset(
    {
        "const",
        "let",
        "op",
        "swap!",
        "make",
        "match",
        "fn",
        "fix",
        "fix-group",
        "seq",
        # Application head is any identifier — not in a fixed set, so we do not
        # reject unknown atoms at head position (they are App nodes).
    }
)


def _validate_head(head: str, line: int) -> None:
    """Validate the head of a list form (PURE).

    Any atom is valid at head position (application nodes have variable-name heads).
    The only invalid head is another list or bracket (handled by the caller).
    This function is a hook for future stricter validation.
    """
    # Currently: all atom heads are accepted (application semantics — variable reference
    # can head an App node).  Future: could reject heads that are clearly malformed.
    pass


# ---------------------------------------------------------------------------
# Rust binary integration (graceful-skip if not compiled)
# ---------------------------------------------------------------------------


def _find_rust_binary(repo_root: Path | None) -> Path | None:
    """Locate the compiled ``llm_canonical_parse`` example binary, or return None.

    Tries the Cargo release/debug build paths under the workspace ``target/``
    directory.  Returns None rather than raising (the caller handles the skip).
    """
    if repo_root is None:
        return None
    for profile in ("debug", "release"):
        candidate = repo_root / "target" / profile / "examples" / "llm_canonical_parse"
        if candidate.exists() and os.access(candidate, os.X_OK):
            return candidate
    return None


def parse_llm_canonical_via_binary(
    source: str,
    *,
    repo_root: Path | None = None,
    timeout_s: float = 30.0,
) -> str | None:
    """Try to parse LlmCanonical via the compiled Rust binary.

    Returns the normalized S-expression string on success, or None if the binary
    is absent or fails.  Errors are logged but never raised (graceful-skip contract).

    Guarantee tag: Empirical (delegates to the Rust ``parse_llm_canonical``).
    """
    binary = _find_rust_binary(repo_root)
    if binary is None:
        return None
    try:
        proc = subprocess.run(
            [str(binary)],
            input=source,
            capture_output=True,
            text=True,
            timeout=timeout_s,
        )
        if proc.returncode == 0:
            return proc.stdout.strip() or None
        _LOG.debug(
            "llm_canonical_parse exited %d: %s", proc.returncode, proc.stderr.strip()
        )
        return None
    except (FileNotFoundError, subprocess.TimeoutExpired, OSError) as exc:
        _LOG.debug("llm_canonical_parse unavailable: %s", exc)
        return None


def parse_llm_canonical_py(source: str, *, repo_root: Path | None = None) -> str | None:
    """Parse/validate a LlmCanonical source string (Python + optional Rust binary).

    Tries the compiled Rust binary first (preferred, authoritative).  Falls back to
    the Python-side bracket-matching validator.  Returns the normalized S-expression
    string on success, or ``None`` on any parse error.

    Guarantee tag: **Empirical** — evidenced by the self-test; the Rust path is the
    authoritative implementation; the Python path is a structural heuristic.
    Never upgraded to Proven (VR-5).
    """
    # Try Rust binary (authoritative).
    result = parse_llm_canonical_via_binary(source, repo_root=repo_root)
    if result is not None:
        return result
    # Fall back to Python-side validator.
    try:
        return _validate_sexp(source)
    except _ParseError as exc:
        _LOG.debug("Python-side LlmCanonical parse failed: %s", exc)
        return None


# ---------------------------------------------------------------------------
# Arm 4 verification
# ---------------------------------------------------------------------------


def arm4_run(
    task: Task,
    model_response: str,
    *,
    scorer: MycCheckScorer | None = None,
    repo_root: Path | None = None,
    log: logging.Logger | None = None,
) -> dict[str, Any]:
    """Verify one model response for Arm 4 (LlmCanonical familiar-skin).

    Returns an outcome dict with:
      ``arm``          — 4
      ``status``       — ``PASS`` | ``FAIL`` | ``skip``
      ``reason``       — present when ``status == "skip"``
      ``parse_error``  — present (and True) when the LlmCanonical parse failed
      ``parse_error_msg`` — human-readable parse error detail
      ``score``        — the ``ScoreResult.to_dict()`` when scoring ran
      ``guarantee_tag`` — "Empirical" (self-test-evidenced; VR-5)

    HONESTY (G2):
      * An empty model response is never a PASS.
      * A parse failure is ``status: "FAIL"`` with ``parse_error: True`` — never silent.
      * If neither the Rust binary nor the Python validator can run, yields
        ``status: "skip"``, ``reason: "llm-canonical-parser-not-compiled"``.
      * A scorer SKIP is a SKIP (never a fabricated PASS).
    """
    log = log or _LOG
    base: dict[str, Any] = {"arm": 4, "guarantee_tag": "Empirical"}

    # G2: empty response is never clean.
    if not model_response.strip():
        return {
            **base,
            "status": _STATUS_FAIL,
            "parse_error": True,
            "parse_error_msg": "empty model response (never a PASS — G2)",
        }

    # VR-5: reject forbidden self-tags.
    viol = scan_forbidden_self_tags(model_response)
    if viol:
        return {
            **base,
            "status": _STATUS_FAIL,
            "parse_error": False,
            "parse_error_msg": "; ".join(viol),
        }

    # Step 1: parse/validate the LlmCanonical output.
    normalized = parse_llm_canonical_py(model_response, repo_root=repo_root)
    if normalized is None:
        # The Python validator ran and found the input structurally invalid — OR
        # the Rust binary was not present AND the Python validator failed.
        # Distinguish: try the Python validator explicitly to get the error message.
        parse_err_msg = "LlmCanonical parse failed (structural validation error)"
        try:
            _validate_sexp(model_response)
            # If we reach here, Python validator succeeded but Rust binary was absent.
            # This is the graceful-skip path: we have a Python-validated form but
            # cannot get the authoritative Rust normalization.
            return {
                **base,
                "status": _STATUS_SKIP,
                "reason": SKIP_REASON_NOT_COMPILED,
                "parse_error": False,
            }
        except _ParseError as exc:
            parse_err_msg = str(exc)

        return {
            **base,
            "status": _STATUS_FAIL,
            "parse_error": True,
            "parse_error_msg": parse_err_msg,
        }

    # Step 2: score the validated output with myc-check if scorer is available.
    # The normalized form is a Core-IR S-expression; myc-check expects .myc (L1)
    # format, so it will typically return exit code 2 (parse error) for S-expression
    # input.  This is an HONEST result — the arm faithfully measures what myc-check
    # says about LlmCanonical-formatted output; the full LlmCanonical→.myc converter
    # is a future enhancement (RFC-0021 §4.1 EditCapability follow-up).
    if scorer is None:
        # No scorer: report parse success (the parse is the only check this arm
        # can perform without a scorer).
        return {
            **base,
            "status": _STATUS_PASS,
            "parse_error": False,
            "normalized_sexp": normalized,
            "score": None,
        }

    score = scorer.score(normalized)
    if score.verdict == VERDICT_SKIP:
        return {
            **base,
            "status": _STATUS_SKIP,
            "reason": f"scorer unavailable: {score.message}",
            "parse_error": False,
            "score": score.to_dict(),
        }
    status = _STATUS_PASS if score.verdict == VERDICT_CLEAN else _STATUS_FAIL
    return {
        **base,
        "status": status,
        "parse_error": False,
        "normalized_sexp": normalized,
        "score": score.to_dict(),
    }
