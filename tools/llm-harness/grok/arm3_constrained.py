"""Arm 3 (grammar-constrained decoding) — M-381 retention ablation (M-330/M-381).

Arm 3 of the T3.6 retention ablation: a *novel-surface* arm (a retention-ratio
numerator candidate) where the model decodes under a **grammar constraint** (GBNF /
Outlines / Guidance) so its output is syntactically valid `.myc` by construction.

WHY IT IS BLOCKED ON THE xAI PATH
---------------------------------
The xAI OpenAI-compatible REST endpoint exposes **no** grammar/GBNF parameter, so
constrained decoding cannot run through it. Arm 3 therefore needs a **local**
GBNF-capable backend (llama.cpp / `llama-cpp-python`, or Outlines wrapping a local
model) — the M-331 llama.cpp harness is its natural home. With no local model present
in an environment, arm 3 must **gracefully SKIP** (never a fabricated score — G2).

CONTRACT (Leaf MS35-A implements this; the orchestrator wires it into ablation.py):
  * ``mycelium_gold_gbnf() -> str`` — a GBNF grammar covering the gold-task `.myc`
    surface subset (nodule + fn + the expression forms the gold set uses). Must be a
    well-formed GBNF string (offline-testable: balanced rules, a start symbol).
  * ``ConstrainedBackend`` — an optional local backend. Construction probes for a
    backend (``llama_cpp`` import + a model path from env ``MYC_ARM3_MODEL``); if
    absent it is ``available == False`` and ``generate`` returns a SKIP result. A
    present backend decodes under the GBNF. NEVER raises a bare ImportError mid-run.
  * ``arm3_constrained_prompt(task: Task) -> list[ChatMessage]`` — the arm-3 prompt
    (PURE; Declared).
  * ``SKIP_REASON_NO_BACKEND`` — the explicit skip reason string.

HONESTY (G2 / VR-5): a missing backend/model is an explicit SKIP, never a 0%/100%.
The grammar guarantees syntactic validity only; typecheck still runs via myc-check
(arm 3 outputs `.myc` directly, so it is scored like arms 1/2).

Guarantee tags (per operation):
  * ``mycelium_gold_gbnf``: Declared — the grammar is authored/asserted; correctness
    relative to the real parser is manual, not machine-checked here.
  * ``arm3_constrained_prompt``: Declared — prompt is asserted configuration.
  * ``ConstrainedBackend.generate`` (live): Empirical — model output under GBNF
    constraint; typecheck verdict is authoritative myc-check.
  * ``ConstrainedBackend.generate`` (skip): Declared — explicit SKIP, no model run.
"""

from __future__ import annotations

import os
import re
import time
from dataclasses import dataclass
from typing import Any

from .client import ChatMessage, ChatResult
from .coauthor_loop import SYSTEM_PROMPT
from .tasks import Task

SKIP_REASON_NO_BACKEND = "arm3-constrained-decoder-not-available"

# Environment variable that must name a local GGUF/model path for the backend.
_MODEL_ENV_VAR = "MYC_ARM3_MODEL"


# ---------------------------------------------------------------------------
# GBNF grammar (Declared — authored subset of docs/spec/grammar/mycelium.ebnf)
# ---------------------------------------------------------------------------


def mycelium_gold_gbnf() -> str:
    """Return a GBNF grammar for the gold-task `.myc` surface subset.

    Covers the expression forms used by the 8 gold tasks (g01-g08):
      - nodule <ident>
      - fn <ident>(<param>: <type>) -> <type> = <expr>
      - types: Binary{N}, Ternary{N}
      - exprs: identifiers, calls f(args) (incl. not(x), add(x,y)),
               swap(x, to: T, policy: p), let id = e in e
      - binary literals 0b..., integer literals

    This is a faithful *subset* of the real grammar, not the full language.
    Guarantee: Declared (authored, not machine-verified against the parser).

    GBNF reference: https://github.com/ggerganov/llama.cpp/blob/master/grammars/README.md
    GBNF uses PEG-style alternation (first match), quoted terminals, and
    named rules. Whitespace between tokens must be explicit.
    """
    # GBNF rule set — kept as a single string for easy offline inspection.
    # Rules are ordered so that llama.cpp resolves left-hand-side references
    # without forward-declaration issues (definitions before use where practical).
    return r"""
# Mycelium gold-task surface subset — GBNF (Declared, M-381 arm 3)
# Covers nodule header + fn definitions + Binary/Ternary types + core expressions.
# Whitespace is explicit: ws = zero-or-more spaces/newlines/tabs.

root         ::= ws nodule-header ws fn-item+ ws

# --- Nodule header ---
nodule-header ::= "nodule" ws ident ws newline

# --- Function item ---
fn-item      ::= "fn" ws ident ws "(" ws params? ws ")" ws "->" ws type-ref ws "=" ws expr ws newline?

params       ::= param (ws "," ws param)*
param        ::= ident ws ":" ws type-ref

# --- Types (gold subset: Binary{N} and Ternary{N}) ---
type-ref     ::= binary-type | ternary-type

binary-type  ::= "Binary" ws "{" ws pos-int ws "}"
ternary-type ::= "Ternary" ws "{" ws pos-int ws "}"

# --- Expressions (gold subset) ---
# Order matters in GBNF (PEG first-match): more specific forms first.
expr         ::= let-expr | swap-expr | call-expr | literal | ident

let-expr     ::= "let" ws ident ws "=" ws expr ws "in" ws expr

swap-expr    ::= "swap" ws "(" ws expr ws "," ws "to" ws ":" ws type-ref ws "," ws "policy" ws ":" ws policy-ident ws ")"

# Call: f(args) — covers id(x), not(x), add(x, y), etc.
call-expr    ::= ident ws "(" ws args? ws ")"

args         ::= expr (ws "," ws expr)*

# Swap policies (the declared set: roundtrip, clamp, saturate)
policy-ident ::= "roundtrip" | "clamp" | "saturate"

# --- Literals (gold subset: binary literals and bare integers) ---
literal      ::= bin-lit | int-lit

bin-lit      ::= "0b" [01]+
int-lit      ::= [0-9]+

# --- Identifiers ---
ident        ::= [a-zA-Z_] [a-zA-Z0-9_]*

# --- Positive integers (for type widths/trit-counts) ---
pos-int      ::= [1-9] [0-9]*

# --- Whitespace / newline helpers ---
ws           ::= [ \t\n\r]*
newline      ::= "\n"
"""


# ---------------------------------------------------------------------------
# Prompt (Declared — authored configuration)
# ---------------------------------------------------------------------------


def arm3_constrained_prompt(task: Task) -> list[ChatMessage]:
    """Build the arm-3 prompt (PURE, Declared).

    Arm 3 relies on GBNF grammar-constrained decoding for syntactic validity,
    so the prompt focuses on the semantic task spec. The SYSTEM_PROMPT from
    coauthor_loop.py supplies the Mycelium language framing.

    Guarantee: Declared (authored configuration, not empirically verified).
    """
    return [
        ChatMessage("system", SYSTEM_PROMPT),
        ChatMessage(
            "user",
            f"Task: {task.spec}\n"
            "Write a correct Mycelium program. "
            "Your output must be valid Mycelium syntax: start with `nodule <name>`, "
            "then define the function. Use `swap(x, to: T, policy: p)` for any "
            "Binary<->Ternary crossing. Reply with ONLY the Mycelium program.",
        ),
    ]


# ---------------------------------------------------------------------------
# ConstrainedBackend — optional local GBNF-capable decoding backend
# ---------------------------------------------------------------------------


@dataclass
class SkipResult:
    """A SKIP-shaped result returned when no local backend is available (G2).

    Never a fabricated score; the ablation records this as ``blocked``.
    Guarantee: Declared (explicit skip, no model run).
    """

    status: str  # always "skip"
    reason: str


def _probe_backend() -> tuple[bool, str]:
    """Lazily probe for a usable local backend.

    Returns (available, reason_if_not). Two conditions must both hold:
      1. ``llama_cpp`` is importable.
      2. ``MYC_ARM3_MODEL`` env var names an existing file path.

    Probe is lazy: only called at ConstrainedBackend construction, never at
    module import. Never raises ImportError (G2).
    """
    model_path = os.environ.get(_MODEL_ENV_VAR, "")
    if not model_path:
        return (
            False,
            f"{SKIP_REASON_NO_BACKEND}: env var {_MODEL_ENV_VAR!r} is unset or empty",
        )

    try:
        import llama_cpp  # noqa: F401  # type: ignore[import-untyped]
    except ImportError:
        return (
            False,
            f"{SKIP_REASON_NO_BACKEND}: llama_cpp is not installed "
            "(install llama-cpp-python to enable arm 3)",
        )

    if not os.path.isfile(model_path):
        return (
            False,
            f"{SKIP_REASON_NO_BACKEND}: model path from {_MODEL_ENV_VAR!r} "
            f"does not exist: {model_path!r}",
        )

    return True, ""


class ConstrainedBackend:
    """Optional local GBNF-capable decoding backend (graceful-skip if absent).

    Construction probes for a backend (``llama_cpp`` import + a model path from
    env ``MYC_ARM3_MODEL``). If either is absent, ``available`` is False and
    ``generate`` returns a SKIP-shaped result (G2 — never a fabricated score).

    When available, ``generate`` decodes the prompt under the GBNF grammar from
    ``mycelium_gold_gbnf()`` and returns a ``ChatResult`` so it scores like arms
    1/2 in the ablation.

    Guarantee: Empirical when available (model output + myc-check), Declared when
    skipping (explicit SKIP, no model run).
    """

    def __init__(self) -> None:
        available, skip_reason = _probe_backend()
        self.available: bool = available
        self._skip_reason: str = skip_reason
        self._llm: Any = None  # populated on construction if available

        if available:
            # Instantiate the model now (eager load, but only when available).
            # Wrapped so a construction failure is an explicit error, not an
            # unhandled ImportError or crash (G2).
            try:
                import llama_cpp  # type: ignore[import-untyped]

                model_path = os.environ.get(_MODEL_ENV_VAR, "")
                self._llm = llama_cpp.Llama(
                    model_path=model_path,
                    n_ctx=2048,
                    verbose=False,
                )
            except Exception as exc:
                # Construction failed — downgrade to unavailable rather than crash.
                self.available = False
                self._skip_reason = (
                    f"{SKIP_REASON_NO_BACKEND}: model load failed: {exc}"
                )

    def generate(
        self,
        task: Task,
        *,
        model: str = "arm3-constrained-local",
        seed: int | None = None,
    ) -> ChatResult | SkipResult:
        """Generate a Mycelium program for ``task`` under the GBNF grammar.

        Returns a ``SkipResult`` (with ``status="skip"``) when no backend is
        available — never a fabricated score (G2). Returns a ``ChatResult`` on
        success so it is scored identically to arms 1/2.

        Guarantee (when available): Empirical — model output under GBNF constraint;
        syntactic validity is construction-guaranteed by the grammar; type-check is
        authoritative myc-check.
        Guarantee (when unavailable): Declared — explicit SKIP, no model run.
        """
        if not self.available:
            return SkipResult(status="skip", reason=self._skip_reason)

        # Available path: decode under the GBNF grammar.
        try:
            import llama_cpp  # type: ignore[import-untyped]

            gbnf = mycelium_gold_gbnf()
            grammar = llama_cpp.LlamaGrammar.from_string(gbnf)
            messages = arm3_constrained_prompt(task)
            # Build the messages list in the OpenAI-compatible format.
            formatted = [{"role": m.role, "content": m.content} for m in messages]

            t0 = time.monotonic()
            kwargs: dict[str, Any] = {
                "messages": formatted,
                "grammar": grammar,
                "max_tokens": 512,
            }
            if seed is not None:
                kwargs["seed"] = seed

            response = self._llm.create_chat_completion(**kwargs)
            latency = time.monotonic() - t0

            choices = response.get("choices") or []
            if not choices:
                return ChatResult(
                    ok=False,
                    content="",
                    prompt_tokens=0,
                    completion_tokens=0,
                    latency_s=latency,
                    model=model,
                    error="arm3: no choices in llama_cpp response",
                    raw=response,
                )
            msg = choices[0].get("message") or {}
            content = (msg.get("content") or "").strip()
            usage = response.get("usage") or {}
            return ChatResult(
                ok=True,
                content=content,
                prompt_tokens=int(usage.get("prompt_tokens", 0) or 0),
                completion_tokens=int(usage.get("completion_tokens", 0) or 0),
                latency_s=latency,
                model=model,
                finish_reason=choices[0].get("finish_reason", ""),
                raw=response,
            )
        except Exception as exc:  # pragma: no cover — live path, no model here
            return ChatResult(
                ok=False,
                content="",
                prompt_tokens=0,
                completion_tokens=0,
                latency_s=0.0,
                model=model,
                error=f"arm3: llama_cpp generation failed: {exc}",
            )


# ---------------------------------------------------------------------------
# Offline deterministic self-test (no network, no model)
# ---------------------------------------------------------------------------


def arm3_selftest() -> list[tuple[str, bool, str]]:
    """Offline, deterministic checks — (name, ok, detail) triples.

    No network, no model calls. All checks pass in an environment without
    llama_cpp or MYC_ARM3_MODEL (the SKIP outcome IS the correct outcome here).
    Suitable for wiring into the harness self-test suite.
    """
    results: list[tuple[str, bool, str]] = []

    # T1: GBNF is non-empty
    gbnf = mycelium_gold_gbnf()
    results.append(
        (
            "arm3/gbnf-non-empty",
            bool(gbnf and gbnf.strip()),
            f"length={len(gbnf)}",
        )
    )

    # T2: GBNF has a 'root' start rule — match an actual rule definition line,
    # not just the substring "root" appearing in a comment or rule body.
    # Pattern: optional leading whitespace, then "root", then optional whitespace,
    # then "::=" — in MULTILINE mode so ^ anchors to each line start.
    has_root = bool(re.search(r"^\s*root\s*::=", gbnf, re.MULTILINE))
    results.append(
        (
            "arm3/gbnf-has-root-rule",
            has_root,
            "found 'root ::=' rule definition" if has_root else "MISSING root ::= rule",
        )
    )

    # T3: GBNF double-quotes are balanced (all strings opened are closed)
    dq_count = gbnf.count('"')
    dq_balanced = dq_count % 2 == 0
    results.append(
        (
            "arm3/gbnf-quotes-balanced",
            dq_balanced,
            f"double-quote count={dq_count} ({'balanced' if dq_balanced else 'UNBALANCED'})",
        )
    )

    # T4: GBNF brackets balanced (count '[' and ']')
    lb = gbnf.count("[")
    rb = gbnf.count("]")
    brackets_ok = lb == rb
    results.append(
        (
            "arm3/gbnf-brackets-balanced",
            brackets_ok,
            f"'['={lb} ']'={rb} ({'balanced' if brackets_ok else 'UNBALANCED'})",
        )
    )

    # T5: GBNF parens balanced
    lp = gbnf.count("(")
    rp = gbnf.count(")")
    parens_ok = lp == rp
    results.append(
        (
            "arm3/gbnf-parens-balanced",
            parens_ok,
            f"'('={lp} ')'={rp} ({'balanced' if parens_ok else 'UNBALANCED'})",
        )
    )

    # T6: GBNF covers key rules needed for the gold tasks
    required_rules = [
        "binary-type",
        "ternary-type",
        "swap-expr",
        "let-expr",
        "call-expr",
    ]
    for rule in required_rules:
        present = rule in gbnf
        results.append(
            (
                f"arm3/gbnf-has-rule-{rule}",
                present,
                f"rule '{rule}' {'found' if present else 'MISSING'} in GBNF",
            )
        )

    # T7: GBNF contains policy keywords from the spec
    for policy in ["roundtrip", "clamp", "saturate"]:
        present = policy in gbnf
        results.append(
            (
                f"arm3/gbnf-has-policy-{policy}",
                present,
                f"policy '{policy}' {'found' if present else 'MISSING'} in GBNF",
            )
        )

    # T8: prompt builds without error for a sample task
    sample_task = Task(
        "g01-identity",
        "Define a function `id` mapping a Binary{8} to itself.",
        fn_name="id",
        param_type="Binary{8}",
        return_type="Binary{8}",
    )
    try:
        msgs = arm3_constrained_prompt(sample_task)
        prompt_ok = (
            len(msgs) == 2
            and msgs[0].role == "system"
            and msgs[1].role == "user"
            and sample_task.spec in msgs[1].content
        )
        results.append(
            (
                "arm3/prompt-builds-for-sample-task",
                prompt_ok,
                f"messages={len(msgs)}, roles={[m.role for m in msgs]}",
            )
        )
    except Exception as exc:
        results.append(
            ("arm3/prompt-builds-for-sample-task", False, f"exception: {exc}")
        )

    # T9: ConstrainedBackend constructs without crashing
    try:
        backend = ConstrainedBackend()
        backend_constructed = True
        backend_detail = f"available={backend.available}"
    except Exception as exc:
        backend_constructed = False
        backend_detail = f"exception: {exc}"
    results.append(
        (
            "arm3/backend-constructs-no-crash",
            backend_constructed,
            backend_detail,
        )
    )

    # T10: generate returns a SKIP result when available=False (G2 — never fabricates)
    try:
        backend = ConstrainedBackend()
        result = backend.generate(sample_task)
        if not backend.available:
            # Must be a SkipResult with status="skip" and a non-empty reason (G2)
            is_skip = isinstance(result, SkipResult) and result.status == "skip"
            has_reason = isinstance(result, SkipResult) and bool(result.reason)
            skip_ok = is_skip and has_reason
            results.append(
                (
                    "arm3/backend-skip-when-unavailable",
                    skip_ok,
                    (
                        f"status={result.status!r}, reason={result.reason!r}"
                        if isinstance(result, SkipResult)
                        else f"unexpected result type: {type(result).__name__}"
                    ),
                )
            )
        else:
            # available=True: live path, skip test not applicable offline
            results.append(
                (
                    "arm3/backend-skip-when-unavailable",
                    True,
                    "backend available — skip test not applicable in this env",
                )
            )
    except Exception as exc:
        results.append(
            ("arm3/backend-skip-when-unavailable", False, f"exception: {exc}")
        )

    # T11: SKIP_REASON_NO_BACKEND is a non-empty string constant
    results.append(
        (
            "arm3/skip-reason-constant-non-empty",
            bool(SKIP_REASON_NO_BACKEND),
            f"value={SKIP_REASON_NO_BACKEND!r}",
        )
    )

    return results
