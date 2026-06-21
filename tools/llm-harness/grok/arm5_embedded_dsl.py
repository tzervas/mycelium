"""Arm 5 (embedded-DSL baseline, RR-3) — M-381 retention ablation.

Arm 5 of the T3.6 retention ablation: the **embedded-DSL** baseline. RR-3 is the
"fall back to an embedded DSL in a high-resource host language" contingency
(DN-09 §3.1, Foundation §6). The arm measures: when the model writes the program in a
*familiar host language* (a small Python-embedded DSL that emits Mycelium), how does
its pass@1 compare? It is a *familiar-skin* point like arm 4, but in a mainstream host
rather than an S-expression projection.

STATUS: RR-3 is an UNSPENT contingency (KC-2 verdict = proceed), so no embedded DSL was
ever built. This arm builds the *baseline harness* so arm 5 is runnable for research;
it does not commit Mycelium to an embedded-DSL future (that needs a KC-2 reversal).

Unlike arm 3, arm 5 CAN run through the xAI REST client (it is ordinary text
generation of Python), then the emitted `.myc` is scored by the same myc-check.

CONTRACT (Leaf MS35-B implements; the orchestrator wires it into ablation.py):
  * The embedded DSL itself — a small, SAFE Python API that builds a Mycelium program
    and renders it to `.myc` text (e.g. ``nodule(...)``, ``fn(...)``, ``binary(8)``,
    ``ternary(6)``, ``swap(...)``, ``op("not", x)`` …). Pure; no I/O.
  * ``arm5_embedded_dsl_prompt(task: Task) -> list[ChatMessage]`` — the arm-5 prompt
    teaching the DSL and asking the model to write a program in it (PURE; Declared).
  * ``eval_embedded_dsl(code: str) -> str | None`` — evaluate the model's DSL code in a
    RESTRICTED sandbox (no imports, no builtins beyond the DSL surface; never exec
    arbitrary code) and return the rendered `.myc`, or ``None`` if it does not produce
    a valid program (scored not-clean — never a false PASS, G2). Must be robust to
    hostile/malformed input (no filesystem/network/`__import__` reachable).

HONESTY (G2 / VR-5): a DSL snippet that errors, times out, or escapes the sandbox
yields ``None`` (not-clean), never a fabricated PASS. The DSL only guarantees it
*emits* `.myc`; typecheck is myc-check's (arm 5 is scored like arms 1/2/4).

SANDBOX HONESTY (Declared / best-effort): ``eval_embedded_dsl`` uses a restricted
``exec`` namespace that exposes ONLY the DSL surface and a minimal ``__builtins__``
subset. This is NOT a fully isolated sandbox (CPython does not provide one purely in
Python); it is "best-effort restricted eval" — Declared, not Proven. Specifically:
  - The namespace has ``__builtins__ = {}`` so builtin names (``open``, ``__import__``,
    etc.) are absent from the execution scope.
  - Explicit ``import`` / ``__import__`` / ``exec`` / ``eval`` names are blocked by
    keyword scanning BEFORE evaluation, so a literal ``import os`` or ``__import__``
    call in the model's snippet is rejected pre-emptively.
  - Dunder attribute access (``__class__``, ``__subclasses__``, etc.) in the snippet
    text is rejected by the same pre-scan.
  - The restricted exec runs in a SEPARATE PROCESS (``multiprocessing``, fork context
    on Linux/WSL) with a hard wall-clock timeout (default 5 s). If the process exceeds
    the timeout it is ``terminate()``-d and ``None`` is returned. This is the robust
    non-termination guard: ``signal.alarm`` is not used because the caller may run from
    worker threads where signal-based alarms do not fire.
  - All exceptions (SyntaxError, NameError, any runtime error) → ``None`` (never raise
    out, never a fabricated score).
  LIMITS (acknowledged): a sufficiently determined adversary *could* escape via
  CPython-internal paths (C-level subclass/MRO walks, ctypes, etc.). The sandbox is
  designed for honest harness use with model-generated snippets, not for hosting
  arbitrary untrusted code. Use it accordingly.
"""

from __future__ import annotations

import multiprocessing
import re
from typing import Any

from .client import ChatMessage
from .tasks import Task

# ---------------------------------------------------------------------------
# Embedded DSL — pure Python objects that build and render .myc programs.
# ---------------------------------------------------------------------------


class _Expr:
    """Base class for DSL expression nodes. PURE; no I/O."""

    def render(self) -> str:
        raise NotImplementedError


class _Var(_Expr):
    """A variable reference (identifier) in .myc."""

    def __init__(self, name: str) -> None:
        if not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", name):
            raise ValueError(f"invalid identifier: {name!r}")
        self._name = name

    def render(self) -> str:
        return self._name


class _Op(_Expr):
    """A primitive call: ``not(x)``, ``add(x, y)``, etc. (surface spelling)."""

    # Surface primitives matching the .myc grammar / _PRIM_KERNEL_TO_SURFACE table.
    _ALLOWED: frozenset[str] = frozenset(
        ["not", "xor", "and", "or", "add", "sub", "mul", "neg"]
    )

    def __init__(self, prim: str, *args: _Expr) -> None:
        if prim not in self._ALLOWED:
            raise ValueError(
                f"unknown primitive {prim!r}; allowed: {sorted(self._ALLOWED)}"
            )
        self._prim = prim
        self._args = list(args)

    def render(self) -> str:
        args = ", ".join(a.render() for a in self._args)
        return f"{self._prim}({args})"


class _Swap(_Expr):
    """An explicit never-silent representation swap (swap_expr in the grammar).

    ``to_repr`` must be a valid repr string (e.g. ``"Ternary{6}"``, ``"Binary{8}"``).
    ``policy`` must be one of ``roundtrip``, ``clamp``, ``saturate``.
    """

    _ALLOWED_POLICIES: frozenset[str] = frozenset(["roundtrip", "clamp", "saturate"])
    # Minimal repr pattern; the rendered form goes directly into .myc.
    _REPR_RE = re.compile(r"^(Binary\{\d+\}|Ternary\{\d+\}|Dense\{\d+,[A-Za-z0-9]+\})$")

    def __init__(self, src: _Expr, *, to_repr: str, policy: str) -> None:
        if not self._REPR_RE.fullmatch(to_repr):
            raise ValueError(f"invalid repr {to_repr!r}; expected e.g. 'Ternary{{6}}'")
        if policy not in self._ALLOWED_POLICIES:
            raise ValueError(
                f"invalid policy {policy!r}; allowed: {sorted(self._ALLOWED_POLICIES)}"
            )
        self._src = src
        self._to_repr = to_repr
        self._policy = policy

    def render(self) -> str:
        return (
            f"swap({self._src.render()}, to: {self._to_repr}, policy: {self._policy})"
        )


class _Let(_Expr):
    """A let-in binding: ``let <name> = <bound> in <body>``."""

    def __init__(self, name: str, bound: _Expr, body: _Expr) -> None:
        if not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", name):
            raise ValueError(f"invalid let binder {name!r}")
        self._name = name
        self._bound = bound
        self._body = body

    def render(self) -> str:
        return f"let {self._name} = {self._bound.render()} in {self._body.render()}"


class _BinLit(_Expr):
    """A binary literal: ``0b0000_0001``."""

    _RE = re.compile(r"^0b[01_]+$")

    def __init__(self, value: str) -> None:
        if not self._RE.fullmatch(value):
            raise ValueError(f"invalid binary literal {value!r}")
        self._value = value

    def render(self) -> str:
        return self._value


class _TritLit(_Expr):
    """A ternary (trit) literal: ``<+0-->``."""

    _RE = re.compile(r"^<[+0\-]+>$")

    def __init__(self, value: str) -> None:
        if not self._RE.fullmatch(value):
            raise ValueError(f"invalid trit literal {value!r}; expected e.g. '<+0-->'")
        self._value = value

    def render(self) -> str:
        return self._value


class _FnItem:
    """A function definition item in a nodule."""

    def __init__(
        self,
        name: str,
        *,
        param: str,
        param_type: str,
        return_type: str,
        body: _Expr,
    ) -> None:
        for ident in (name, param):
            if not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", ident):
                raise ValueError(f"invalid identifier {ident!r}")
        # Minimal type validation: non-empty, printable.
        for label, t in (("param_type", param_type), ("return_type", return_type)):
            if not t or not t.strip():
                raise ValueError(f"{label} must not be empty")
        self._name = name
        self._param = param
        self._param_type = param_type
        self._return_type = return_type
        self._body = body

    def render(self) -> str:
        return (
            f"fn {self._name}({self._param}: {self._param_type})"
            f" -> {self._return_type} =\n"
            f"  {self._body.render()}"
        )


class Nodule:
    """Top-level program builder — renders a ``.myc`` nodule.

    Usage::

        p = Nodule("arm5")
        x = var("x")
        p.fn("id", param="x", param_type="Binary{8}", return_type="Binary{8}", body=x)
        print(p.render())

    A ``Nodule`` is PURE (no I/O); ``render()`` returns the ``.myc`` source string.
    """

    def __init__(self, name: str) -> None:
        if not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_.]*", name):
            raise ValueError(f"invalid nodule name {name!r}")
        self._name = name
        self._items: list[_FnItem] = []

    def fn(
        self,
        name: str,
        *,
        param: str,
        param_type: str,
        return_type: str,
        body: _Expr,
    ) -> "Nodule":
        """Add a function definition to the nodule (fluent / returns self)."""
        self._items.append(
            _FnItem(
                name,
                param=param,
                param_type=param_type,
                return_type=return_type,
                body=body,
            )
        )
        return self

    def render(self) -> str:
        """Render to ``.myc`` source (PURE)."""
        lines = [f"nodule {self._name}", ""]
        for item in self._items:
            lines.append(item.render())
            lines.append("")
        return "\n".join(lines).rstrip() + "\n"


# ---------------------------------------------------------------------------
# DSL convenience constructors (the names exposed in the sandbox namespace).
# ---------------------------------------------------------------------------


def nodule(name: str) -> Nodule:
    """Create a new :class:`Nodule` program builder."""
    return Nodule(name)


def var(name: str) -> _Var:
    """A variable reference (identifier)."""
    return _Var(name)


def op(prim: str, *args: _Expr) -> _Op:
    """A primitive call — ``op("not", x)`` -> ``not(x)``."""
    return _Op(prim, *args)


def swap(src: _Expr, *, to: str, policy: str) -> _Swap:
    """An explicit never-silent swap — ``swap(x, to=Ternary(6), policy="roundtrip")``."""
    return _Swap(src, to_repr=to, policy=policy)


def let(name: str, bound: _Expr, body: _Expr) -> _Let:
    """A let-in binding — ``let("y", op("not", x), var("y"))``."""
    return _Let(name, bound, body)


def binlit(value: str) -> _BinLit:
    """A binary literal — ``binlit("0b0000_0001")``."""
    return _BinLit(value)


def tritlit(value: str) -> _TritLit:
    """A balanced-ternary literal — ``tritlit("<+0-->")``."""
    return _TritLit(value)


def Binary(width: int) -> str:  # noqa: N802 — follows the .myc type-name casing
    """Return the Binary repr type-string for use in ``param_type``/``return_type``."""
    if not isinstance(width, int) or width <= 0:
        raise ValueError(f"Binary width must be a positive int, got {width!r}")
    return f"Binary{{{width}}}"


def Ternary(trits: int) -> str:  # noqa: N802 — follows the .myc type-name casing
    """Return the Ternary repr type-string for use in ``param_type``/``return_type``."""
    if not isinstance(trits, int) or trits <= 0:
        raise ValueError(f"Ternary trit count must be a positive int, got {trits!r}")
    return f"Ternary{{{trits}}}"


# The complete DSL surface exported into the sandbox namespace.
_DSL_NAMESPACE: dict[str, Any] = {
    "nodule": nodule,
    "var": var,
    "op": op,
    "swap": swap,
    "let": let,
    "binlit": binlit,
    "tritlit": tritlit,
    "Binary": Binary,
    "Ternary": Ternary,
}

# ---------------------------------------------------------------------------
# Sandbox: RESTRICTED exec for model-generated DSL snippets.
# ---------------------------------------------------------------------------

# Pre-scan patterns that must never appear in the model snippet.
# Blocking these pre-emptively catches the most common escape attempts before
# any Python bytecode is compiled (defence-in-depth on top of the namespace guard).
_BLOCKED_PATTERNS: list[re.Pattern[str]] = [
    re.compile(r"\bimport\b"),  # import statement / from-import
    re.compile(r"\b__import__\b"),  # built-in import function
    re.compile(r"\bexec\b"),  # recursive exec
    re.compile(r"\beval\b"),  # eval in snippet
    re.compile(r"__[A-Za-z_]+__"),  # dunder access of any kind
    re.compile(r"\bopen\b"),  # file I/O
    re.compile(r"\bcompile\b"),  # dynamic compilation
    re.compile(r"\bglobals\b"),  # globals() escape
    re.compile(r"\blocals\b"),  # locals() escape
    re.compile(r"\bgetattr\b"),  # attribute-climbing
    re.compile(r"\bsetattr\b"),  # attribute-climbing
    re.compile(r"\bhasattr\b"),  # attribute-climbing
    re.compile(r"\bdelattr\b"),  # attribute-climbing
    re.compile(r"\bvars\b"),  # vars() escape
    re.compile(r"\bdir\b"),  # dir() escape
    re.compile(r"\bbreakpoint\b"),  # debugger hook
    re.compile(r"\bsubprocess\b"),  # subprocess module name (belt+suspenders)
    re.compile(r"\bos\b"),  # os module name (belt+suspenders)
    re.compile(r"\bsys\b"),  # sys module name (belt+suspenders)
]

# Maximum snippet length accepted (characters). Prevents OOM / CPU-spin on huge inputs.
_MAX_SNIPPET_LEN = 8_000

# Name of the variable the snippet must assign to expose the rendered .myc.
_OUTPUT_VAR = "_myc_program"


def _prescan(code: str) -> str | None:
    """Return an error description if ``code`` contains a blocked pattern; else None."""
    for pat in _BLOCKED_PATTERNS:
        if pat.search(code):
            return f"blocked pattern: {pat.pattern!r}"
    return None


def _eval_worker(code: str, q: "multiprocessing.Queue[str | None]") -> None:
    """Module-level worker executed in a child process by ``eval_embedded_dsl``.

    Performs the restricted exec and puts the rendered ``.myc`` string (or None)
    into ``q``. Must be module-level so it is picklable by ``multiprocessing``.

    Any exception inside the child → None (never raise out, G2).  The result
    (a plain ``str`` or ``None``) is picklable, so the queue transfer is safe.
    """
    try:
        namespace: dict[str, Any] = dict(_DSL_NAMESPACE)
        namespace["__builtins__"] = {}
        exec(code, namespace)  # noqa: S102 — intentional, sandboxed, in child process
        result = namespace.get(_OUTPUT_VAR)
        if result is None or not isinstance(result, str) or not result.strip():
            q.put(None)
        else:
            q.put(result)
    except Exception:  # any compile or runtime error -> not-clean
        q.put(None)


def eval_embedded_dsl(code: str, *, timeout_s: float = 5.0) -> str | None:
    """Evaluate DSL ``code`` in a restricted sandbox -> rendered ``.myc``, or None.

    CONTRACT (G2 / VR-5):
    - Malformed / hostile / raising / non-terminating snippets -> ``None``
      (never a fabricated PASS).
    - Never raises out to the caller — all exceptions caught, log-dropped, -> None.
    - The snippet must assign the result of ``Nodule.render()`` to the variable
      ``_myc_program``; that variable is read back as the ``.myc`` text.

    TIMEOUT: the restricted exec runs in a separate child process (``multiprocessing``,
    fork context on Linux/WSL). If the child does not complete within ``timeout_s``
    seconds (default 5 s), it is ``terminate()``-d and ``None`` is returned.
    ``signal.alarm`` is NOT used because this function may be called from worker
    threads where SIGALRM does not fire.

    SANDBOX (Declared / best-effort — NOT a proven isolation):
    - ``__builtins__`` is set to ``{}`` in the exec namespace, removing all standard
      builtin names from the snippet's scope.
    - A pre-scan rejects any snippet containing ``import``, ``__<dunder>__``,
      ``exec``, ``eval``, ``open``, ``compile``, ``globals``, ``locals``, and a few
      more (see ``_BLOCKED_PATTERNS``).
    - Only the DSL surface names (``nodule``, ``var``, ``op``, ``swap``, ``let``,
      ``binlit``, ``tritlit``, ``Binary``, ``Ternary``) are in scope.
    - Inputs exceeding ``_MAX_SNIPPET_LEN`` characters are rejected immediately.
    See module docstring for acknowledged limits.
    """
    if not code or not code.strip():
        return None
    if len(code) > _MAX_SNIPPET_LEN:
        return None

    # Pre-scan: block patterns before touching the compiler.
    if _prescan(code) is not None:
        return None

    # Run the restricted exec in a child process with a hard wall-clock timeout.
    # fork context: fast on Linux/WSL; avoids the overhead of a full spawn.
    ctx = multiprocessing.get_context("fork")
    q: multiprocessing.Queue[str | None] = ctx.Queue()
    proc = ctx.Process(target=_eval_worker, args=(code, q), daemon=True)
    try:
        proc.start()
        proc.join(timeout=timeout_s)
        if proc.is_alive():
            # Hard wall-clock timeout exceeded — terminate the child and return None.
            proc.terminate()
            proc.join(timeout=1.0)  # give it a moment to clean up
            return None
        # Child finished within the timeout: read back the result.
        try:
            result = q.get_nowait()
        except Exception:  # empty queue (child crashed before put) -> None
            return None
        return result
    except Exception:  # any process-management error -> None (G2)
        try:
            if proc.is_alive():
                proc.terminate()
        except Exception:
            pass
        return None


# ---------------------------------------------------------------------------
# Prompt builder.
# ---------------------------------------------------------------------------

# One small, self-contained worked example (NOT the identity task so the model
# can't just copy it): a function that applies bitwise NOT.
_WORKED_EXAMPLE = """\
# Example: flip(x) — bitwise NOT over Binary{8}
p = nodule("example")
x = var("x")
p.fn("flip",
     param="x",
     param_type=Binary(8),
     return_type=Binary(8),
     body=op("not", x))
_myc_program = p.render()
"""

_DSL_REFERENCE = """\
## Mycelium Embedded DSL — quick reference

DSL constructors (all available in scope — NO imports needed):

  nodule(name)                      → program builder (call .fn(...) then .render())
  var(name)                         → variable reference (identifier)
  op(prim, *args)                   → primitive call
                                       prims: not, xor, and, or, add, sub, mul, neg
  swap(src, to=T, policy=P)         → explicit never-silent swap
                                       to     = Binary(N) or Ternary(N)
                                               (a string like "Binary{8}")
                                       policy = "roundtrip" | "clamp" | "saturate"
  let(name, bound, body)            → let-in binding
  binlit("0b0000_0001")             → a binary literal
  tritlit("<+0-->")                 → a balanced-ternary literal
  Binary(N)                         → the type-string "Binary{N}"
  Ternary(N)                        → the type-string "Ternary{N}"

Nodule.fn(name, *, param, param_type, return_type, body)
  — adds a function; body is any DSL Expr.

To output the .myc source you MUST assign the rendered program to `_myc_program`:
  _myc_program = p.render()

Rules:
  * Every Binary<->Ternary crossing requires an explicit swap — never silent.
  * Swap policy must be one of: roundtrip, clamp, saturate.
  * No imports, no print, no open, no I/O — just DSL calls + the assignment above.
"""


def arm5_embedded_dsl_prompt(task: Task) -> list[ChatMessage]:
    """Build the arm-5 prompt (PURE, Declared).

    Teaches the embedded DSL briefly (one non-answer worked example), then asks the
    model to write the task's program using it. The model's code is evaluated by
    ``eval_embedded_dsl``.

    Guarantee tag: Declared (prompt design; model quality is Empirical when run).
    """
    system = (
        "You are a Mycelium DSL assistant. Write Mycelium programs using the "
        "Python embedded DSL described below. Output ONLY valid Python DSL code — "
        "no prose, no code fences, no imports. The code must assign the rendered "
        "program to `_myc_program` (the last line should be "
        "`_myc_program = p.render()`). "
        "Every Binary<->Ternary crossing must use an explicit swap — never silent.\n\n"
        + _DSL_REFERENCE
        + "\n## Worked example\n\n"
        + _WORKED_EXAMPLE
    )
    user = (
        f"Task: {task.spec}\n\n"
        "Write the Mycelium program using the DSL above. "
        "Assign the rendered source to `_myc_program = p.render()` as the last statement. "
        "Output ONLY the Python DSL code — no explanation, no code fences."
    )
    return [ChatMessage("system", system), ChatMessage("user", user)]


# ---------------------------------------------------------------------------
# Offline deterministic self-test (no network, no API key).
# ---------------------------------------------------------------------------


def arm5_selftest() -> list[tuple[str, bool, str]]:
    """Offline deterministic checks for the embedded DSL and sandboxed evaluator.

    Returns a list of ``(name, ok, detail)`` triples. All checks must pass with NO
    network access. Tagged **Empirical** (self-test-evidenced); live model quality
    stays open/Declared.

    Guarantee tag: Empirical (deterministic offline evidence; plumbing-level).
    """
    results: list[tuple[str, bool, str]] = []

    def _ok(name: str, detail: str) -> None:
        results.append((name, True, detail))

    def _fail(name: str, detail: str) -> None:
        results.append((name, False, detail))

    # ------------------------------------------------------------------
    # DSL-1: identity function renders correct .myc
    # ------------------------------------------------------------------
    try:
        p = nodule("arm5")
        x = var("x")
        p.fn("id", param="x", param_type=Binary(8), return_type=Binary(8), body=x)
        rendered = p.render()
        assert "nodule arm5" in rendered, f"missing header: {rendered!r}"
        assert "fn id(x: Binary{8}) -> Binary{8} =" in rendered, (
            f"wrong fn sig: {rendered!r}"
        )
        body_line = rendered.strip().split("\n")[-1].strip()
        assert body_line == "x", f"wrong body line: {body_line!r}"
        _ok("DSL-1 identity", f"renders ok: {rendered!r}")
    except Exception as exc:
        _fail("DSL-1 identity", f"raised: {exc!r}")

    # ------------------------------------------------------------------
    # DSL-2: bitwise NOT function
    # ------------------------------------------------------------------
    try:
        p = nodule("arm5")
        x = var("x")
        p.fn(
            "flip",
            param="x",
            param_type=Binary(8),
            return_type=Binary(8),
            body=op("not", x),
        )
        rendered = p.render()
        assert "not(x)" in rendered, f"missing not(x): {rendered!r}"
        _ok("DSL-2 not", "op(not) ok")
    except Exception as exc:
        _fail("DSL-2 not", f"raised: {exc!r}")

    # ------------------------------------------------------------------
    # DSL-3: swap function (Binary->Ternary roundtrip)
    # ------------------------------------------------------------------
    try:
        p = nodule("arm5")
        x = var("x")
        p.fn(
            "widen",
            param="x",
            param_type=Binary(8),
            return_type=Ternary(6),
            body=swap(x, to=Ternary(6), policy="roundtrip"),
        )
        rendered = p.render()
        assert "swap(x, to: Ternary{6}, policy: roundtrip)" in rendered, (
            f"wrong swap: {rendered!r}"
        )
        _ok("DSL-3 swap", "swap ok")
    except Exception as exc:
        _fail("DSL-3 swap", f"raised: {exc!r}")

    # ------------------------------------------------------------------
    # DSL-4: nested swap (roundtrip + clamp for g08 round-trip pattern)
    # ------------------------------------------------------------------
    try:
        p = nodule("arm5")
        x = var("x")
        wide = swap(x, to=Ternary(6), policy="roundtrip")
        back = swap(wide, to=Binary(8), policy="clamp")
        p.fn(
            "roundtrip",
            param="x",
            param_type=Binary(8),
            return_type=Binary(8),
            body=back,
        )
        rendered = p.render()
        expected = "swap(swap(x, to: Ternary{6}, policy: roundtrip), to: Binary{8}, policy: clamp)"
        assert expected in rendered, f"wrong nested swap: {rendered!r}"
        _ok("DSL-4 nested-swap", "nested swap ok")
    except Exception as exc:
        _fail("DSL-4 nested-swap", f"raised: {exc!r}")

    # ------------------------------------------------------------------
    # DSL-5: let binding (double via add)
    # ------------------------------------------------------------------
    try:
        p = nodule("arm5")
        x = var("x")
        y = var("y")
        p.fn(
            "double",
            param="x",
            param_type=Ternary(6),
            return_type=Ternary(6),
            body=let("y", op("add", x, x), y),
        )
        rendered = p.render()
        assert "let y = add(x, x) in y" in rendered, f"wrong let: {rendered!r}"
        _ok("DSL-5 let+add", "let/add ok")
    except Exception as exc:
        _fail("DSL-5 let+add", f"raised: {exc!r}")

    # ------------------------------------------------------------------
    # DSL-6: invalid primitive rejected (G2: never-silent)
    # ------------------------------------------------------------------
    try:
        op("mystery_prim", var("x"))
        _fail("DSL-6 bad-prim", "should have raised ValueError for unknown primitive")
    except ValueError:
        _ok("DSL-6 bad-prim", "ValueError raised for unknown primitive (G2)")
    except Exception as exc:
        _fail("DSL-6 bad-prim", f"unexpected exception: {exc!r}")

    # ------------------------------------------------------------------
    # DSL-7: invalid policy rejected (G2: never-silent)
    # ------------------------------------------------------------------
    try:
        swap(var("x"), to=Ternary(6), policy="silent")
        _fail("DSL-7 bad-policy", "should have raised ValueError for invalid policy")
    except ValueError:
        _ok("DSL-7 bad-policy", "ValueError raised for invalid policy (G2)")
    except Exception as exc:
        _fail("DSL-7 bad-policy", f"unexpected exception: {exc!r}")

    # ------------------------------------------------------------------
    # EVAL-1: good snippet -> rendered .myc
    # ------------------------------------------------------------------
    good_snippet = (
        "p = nodule('arm5')\n"
        "x = var('x')\n"
        "p.fn('id', param='x', param_type=Binary(8), return_type=Binary(8), body=x)\n"
        "_myc_program = p.render()\n"
    )
    result = eval_embedded_dsl(good_snippet)
    if result is None:
        _fail("EVAL-1 good-snippet", "expected .myc string, got None")
    elif "nodule arm5" not in result or "fn id" not in result:
        _fail("EVAL-1 good-snippet", f"wrong output: {result!r}")
    else:
        _ok("EVAL-1 good-snippet", f"sandboxed eval -> .myc ok: {result!r}")

    # ------------------------------------------------------------------
    # EVAL-2: good snippet with swap -> rendered .myc
    # ------------------------------------------------------------------
    swap_snippet = (
        "p = nodule('arm5')\n"
        "x = var('x')\n"
        "p.fn('widen', param='x', param_type=Binary(8), return_type=Ternary(6),\n"
        "     body=swap(x, to=Ternary(6), policy='roundtrip'))\n"
        "_myc_program = p.render()\n"
    )
    result2 = eval_embedded_dsl(swap_snippet)
    if result2 is None:
        _fail("EVAL-2 swap-snippet", "expected .myc string, got None")
    elif "swap(x, to: Ternary{6}, policy: roundtrip)" not in result2:
        _fail("EVAL-2 swap-snippet", f"wrong swap output: {result2!r}")
    else:
        _ok("EVAL-2 swap-snippet", "sandboxed eval with swap ok")

    # ------------------------------------------------------------------
    # EVAL-3: syntax error -> None (never a fabricated PASS — G2)
    # ------------------------------------------------------------------
    if eval_embedded_dsl("def f( :::") is not None:
        _fail("EVAL-3 syntax-error", "syntax error must return None (G2)")
    else:
        _ok("EVAL-3 syntax-error", "syntax error -> None (G2)")

    # ------------------------------------------------------------------
    # EVAL-4: empty string -> None
    # ------------------------------------------------------------------
    if eval_embedded_dsl("") is not None:
        _fail("EVAL-4 empty", "empty string must return None")
    else:
        _ok("EVAL-4 empty", "empty -> None")

    # ------------------------------------------------------------------
    # EVAL-5: whitespace-only -> None
    # ------------------------------------------------------------------
    if eval_embedded_dsl("   \n  \t  ") is not None:
        _fail("EVAL-5 whitespace", "whitespace-only must return None")
    else:
        _ok("EVAL-5 whitespace", "whitespace-only -> None")

    # ------------------------------------------------------------------
    # EVAL-6: import os blocked -> None (sandbox)
    # ------------------------------------------------------------------
    if eval_embedded_dsl("import os\n_myc_program = 'x'\n") is not None:
        _fail("EVAL-6 import-os", "import os must be blocked -> None (sandbox)")
    else:
        _ok("EVAL-6 import-os", "import os blocked -> None (sandbox)")

    # ------------------------------------------------------------------
    # EVAL-7: __import__('os') blocked -> None
    # ------------------------------------------------------------------
    if eval_embedded_dsl("__import__('os')\n_myc_program = 'x'\n") is not None:
        _fail("EVAL-7 dunder-import", "__import__ must be blocked -> None (sandbox)")
    else:
        _ok("EVAL-7 dunder-import", "__import__ blocked -> None (sandbox)")

    # ------------------------------------------------------------------
    # EVAL-8: dunder access blocked -> None
    # ------------------------------------------------------------------
    if eval_embedded_dsl("x = ().__class__\n_myc_program = 'x'\n") is not None:
        _fail("EVAL-8 dunder-access", "dunder access must be blocked -> None (sandbox)")
    else:
        _ok("EVAL-8 dunder-access", "dunder access blocked -> None (sandbox)")

    # ------------------------------------------------------------------
    # EVAL-9: snippet that raises NameError -> None (not a crash)
    # ------------------------------------------------------------------
    if eval_embedded_dsl("_myc_program = undefined_name_xyz\n") is not None:
        _fail("EVAL-9 name-error", "NameError must return None (G2)")
    else:
        _ok("EVAL-9 name-error", "NameError -> None (G2, never raises out)")

    # ------------------------------------------------------------------
    # EVAL-10: snippet missing _myc_program assignment -> None
    # ------------------------------------------------------------------
    if eval_embedded_dsl("p = nodule('x')\n") is not None:
        _fail("EVAL-10 no-output", "missing _myc_program must return None")
    else:
        _ok("EVAL-10 no-output", "missing _myc_program -> None")

    # ------------------------------------------------------------------
    # EVAL-11: snippet with open() (blocked by prescan) -> None
    # ------------------------------------------------------------------
    if eval_embedded_dsl("open('/etc/passwd')\n_myc_program = 'x'\n") is not None:
        _fail("EVAL-11 open-blocked", "open() must be blocked -> None (sandbox)")
    else:
        _ok("EVAL-11 open-blocked", "open() blocked -> None (sandbox)")

    # ------------------------------------------------------------------
    # EVAL-12: eval() blocked -> None
    # ------------------------------------------------------------------
    if eval_embedded_dsl("eval('1+1')\n_myc_program = 'x'\n") is not None:
        _fail("EVAL-12 eval-blocked", "eval() must be blocked -> None (sandbox)")
    else:
        _ok("EVAL-12 eval-blocked", "eval() blocked -> None (sandbox)")

    # ------------------------------------------------------------------
    # EVAL-13: exec() blocked -> None
    # ------------------------------------------------------------------
    if eval_embedded_dsl("exec('x=1')\n_myc_program = 'x'\n") is not None:
        _fail("EVAL-13 exec-blocked", "exec() in snippet must be blocked -> None")
    else:
        _ok("EVAL-13 exec-blocked", "exec() in snippet blocked -> None (sandbox)")

    # ------------------------------------------------------------------
    # EVAL-14 (REGRESSION): non-terminating snippet -> None within timeout
    #
    # A snippet containing a ``while True: pass`` infinite spin must return
    # None under the hard process timeout — NEVER hang the harness.
    # Uses timeout_s=2 so this test completes quickly (target ≤ 3 s wall-clock).
    #
    # ``while True: pass`` passes the pre-scan (no blocked keywords), so the
    # process-kill timeout is the ONLY guard — this test validates the core
    # G2 timeout guarantee: process is terminate()-d and None is returned.
    # ------------------------------------------------------------------
    _nonterminating_snippet = "while True: pass\n_myc_program = 'x'\n"
    _timeout_result = eval_embedded_dsl(_nonterminating_snippet, timeout_s=2.0)
    if _timeout_result is not None:
        _fail(
            "EVAL-14 timeout-nonterminating",
            f"non-terminating snippet must return None under timeout (got {_timeout_result!r})",
        )
    else:
        _ok(
            "EVAL-14 timeout-nonterminating",
            "non-terminating snippet (while True) -> None within 2 s timeout (hard process kill, G2)",
        )

    # ------------------------------------------------------------------
    # PROMPT-1: prompt structure is well-formed (PURE check)
    # ------------------------------------------------------------------
    try:
        from .tasks import GOLD_TASKS

        task = GOLD_TASKS[0]  # g01-identity
        msgs = arm5_embedded_dsl_prompt(task)
        assert len(msgs) == 2, f"expected 2 messages, got {len(msgs)}"
        assert msgs[0].role == "system", f"first msg not system: {msgs[0].role}"
        assert msgs[1].role == "user", f"second msg not user: {msgs[1].role}"
        assert task.spec in msgs[1].content, "task spec missing from user message"
        assert "_myc_program" in msgs[0].content, "_myc_program not in system prompt"
        assert "DSL" in msgs[0].content or "nodule" in msgs[0].content, (
            "DSL reference missing from system prompt"
        )
        _ok("PROMPT-1 structure", f"2-msg prompt well-formed for task {task.id!r}")
    except Exception as exc:
        _fail("PROMPT-1 structure", f"raised: {exc!r}")

    return results
