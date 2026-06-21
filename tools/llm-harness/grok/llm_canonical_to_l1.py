"""Arm 4 rigorous bridge: LlmCanonical S-expression -> L1 ``.myc`` surface (M-381).

This is the **option (b)** path of DN-09 §9.4 — the rigorous denominator for the
T3.6 retention ratio (RFC-0021 §4.1 / §4.7; research/11 §T11.7 step 3).

WHY IT EXISTS
-------------
Arms 1/2 (novel ``.myc`` text) are scored by ``myc-check`` (parse **and**
type-check). Arm 4 (the ``LlmCanonical`` familiar-skin projection) emits Core-IR
S-expressions, which ``myc-check`` cannot read — so an earlier ablation scored
arm 4 at 0 %, a *scoring artifact*, leaving the retention ratio INDETERMINATE
(DN-09 §9). This module converts an ``LlmCanonical`` expression to ``.myc`` text
so the **same** authoritative ``myc-check`` typechecks it — putting arm 4 on the
**same quality bar** as arms 1/2.

THE INTRINSIC ASYMMETRY (honest, documented — VR-5)
---------------------------------------------------
``LlmCanonical`` is an *expression* IR: a function is a bare lambda
``(fn [x] body)`` with an **untyped** parameter and **no** function signature.
The ``.myc`` ``fn`` item, by contrast, *requires* explicit parameter and return
types. There is no way to recover a full typed signature from the S-expression
alone. So the bridge **wraps** the converted expression in the task's *known*
signature (``Task.fn_name`` / ``param_type`` / ``return_type`` — ``Declared``
config, the task's intended type).

Consequence: the arm-4 model is **not** required to author the signature (it
cannot — the format can't express it), whereas the arm-1/2 models must author the
*whole* typed program. Arm 4 is therefore **slightly advantaged** (it only has to
get the body right). Because arm 4 is the retention-ratio *denominator*, this
advantage makes the computed retention ratio a **conservative** estimate:
``retention = pass@1(best novel arm) / pass@1(arm 4)`` is biased *downward*, so a
result that still clears the ~70 % threshold is *robust* evidence the novel
surface retains leverage; a result *below* threshold is ambiguous (could be the
asymmetry rather than a real deficit). This caveat travels with every reported
ratio.

GUARANTEE TAGS (G2 / VR-5)
--------------------------
* ``convert_to_myc`` is **Empirical** — a heuristic source-to-source rewrite for
  the LlmCanonical node kinds the gold set exercises, evidenced by the offline
  self-test. It is *not* a proven-sound converter and is never upgraded to Proven.
* The *type-check* itself stays authoritative: the produced ``.myc`` is fed to the
  real Rust ``myc-check`` (the same scorer arms 1/2 use). The bridge only changes
  the surface; the verdict is ``myc-check``'s.
* Any node kind the bridge cannot faithfully convert yields ``None`` (never a
  silently-fabricated program): the caller scores that arm-4 sample as not-clean,
  exactly like a parse failure — never a false PASS (G2).
"""

from __future__ import annotations

import logging

_LOG = logging.getLogger("grok.arm4.bridge")

# Depth guard mirrors the Rust parser (banked guard #4: DEPTH_LIMIT = 64).
_DEPTH_LIMIT = 64

# L0 (kernel) primitive name -> L1 (surface) spelling. The LlmCanonical `op`
# form uses kernel names (e.g. `bit.not`); `.myc` calls use the surface spelling
# (e.g. `not(x)`). Surface spellings are also accepted verbatim (passthrough).
_PRIM_KERNEL_TO_SURFACE: dict[str, str] = {
    "bit.not": "not",
    "bit.xor": "xor",
    "bit.and": "and",
    "bit.or": "or",
    "trit.add": "add",
    "trit.sub": "sub",
    "trit.mul": "mul",
    "trit.neg": "neg",
}
_SURFACE_PRIMS: frozenset[str] = frozenset(_PRIM_KERNEL_TO_SURFACE.values())


class _ConvertError(Exception):
    """Internal: the S-expression could not be faithfully converted to L1."""


# ---------------------------------------------------------------------------
# S-expression reader: parens -> list, brackets -> tuple, atoms -> str.
# (Brackets are tupled so a stray binder used as an expression is rejected.)
# ---------------------------------------------------------------------------


def _tokenize(source: str) -> list[str]:
    tokens: list[str] = []
    i, n = 0, len(source)
    while i < n:
        c = source[i]
        if c in " \t\r\n":
            i += 1
        elif c == ";":  # line comment
            while i < n and source[i] != "\n":
                i += 1
        elif c in "()[]":
            tokens.append(c)
            i += 1
        elif c == "<":  # angle-bracketed literal (e.g. ternary <+-0>); read to '>'
            start = i
            i += 1
            while i < n and source[i] != ">":
                i += 1
            if i >= n:
                raise _ConvertError("unclosed '<'")
            i += 1  # consume '>'
            tokens.append(source[start:i])
        elif c == ">":
            # A stray '>' (the closer of '<…>' is consumed in the '<' branch above).
            # Reject it explicitly — it is in the atom stop-set, so the atom branch
            # below would loop forever on it (G2: fail-fast, never hang).
            raise _ConvertError("unexpected '>'")
        else:  # atom: read to whitespace or a delimiter
            start = i
            while i < n and source[i] not in " \t\r\n()[]<>;":
                i += 1
            if i == start:  # defensive: no atom char consumed -> would loop (G2)
                raise _ConvertError(f"unexpected delimiter {c!r}")
            tokens.append(source[start:i])
    return tokens


def _read(source: str):
    """Parse ``source`` into a single node (parens->list, brackets->tuple, atom->str)."""
    tokens = _tokenize(source)
    if not tokens:
        raise _ConvertError("empty input")
    pos = [0]

    def parse(depth: int):
        if depth > _DEPTH_LIMIT:
            raise _ConvertError(f"nesting depth limit {_DEPTH_LIMIT} exceeded")
        if pos[0] >= len(tokens):
            raise _ConvertError("unexpected EOF")
        tok = tokens[pos[0]]
        pos[0] += 1
        if tok == "(":
            items = []
            while True:
                if pos[0] >= len(tokens):
                    raise _ConvertError("unclosed '('")
                if tokens[pos[0]] == ")":
                    pos[0] += 1
                    break
                items.append(parse(depth + 1))
            return items
        if tok == "[":
            items = []
            while True:
                if pos[0] >= len(tokens):
                    raise _ConvertError("unclosed '['")
                if tokens[pos[0]] == "]":
                    pos[0] += 1
                    break
                items.append(parse(depth + 1))
            return tuple(items)
        if tok in (")", "]"):
            raise _ConvertError(f"unexpected '{tok}'")
        return tok

    forms = []
    while pos[0] < len(tokens):
        forms.append(parse(0))
    if len(forms) == 1:
        return forms[0]
    # Multiple top-level forms -> a seq (the renderer's top-level convention).
    return ["seq", *forms]


# ---------------------------------------------------------------------------
# Expression emitter: LlmCanonical node -> .myc expression text.
# ---------------------------------------------------------------------------


def _emit(node, depth: int = 0) -> str:
    if depth > _DEPTH_LIMIT:
        raise _ConvertError("nesting depth limit exceeded")
    if isinstance(node, str):
        return node  # variable, literal (0b…, <…>, int), or bare identifier
    if isinstance(node, tuple):
        raise _ConvertError("a binder [...] cannot appear as an expression")
    if not node:
        raise _ConvertError("empty list '()' is not an expression")

    head = node[0]
    if not isinstance(head, str):
        # ((f a) b) — application with a compound callee.
        return _emit_app(node, depth)

    if head == "const":
        # (const <value-tokens…> @Tag [:bound]) -> the value tokens only.
        value_tokens = [
            t
            for t in node[1:]
            if not (isinstance(t, str) and (t.startswith("@") or t == ":bound"))
        ]
        if not value_tokens or any(not isinstance(t, str) for t in value_tokens):
            raise _ConvertError(f"unsupported const payload: {node!r}")
        return " ".join(value_tokens)

    if head == "op":
        if len(node) < 2 or not isinstance(node[1], str):
            raise _ConvertError(f"malformed op: {node!r}")
        prim = node[1]
        surface = _PRIM_KERNEL_TO_SURFACE.get(prim)
        if surface is None:
            if prim in _SURFACE_PRIMS:
                surface = prim  # already a surface spelling
            else:
                raise _ConvertError(f"unknown primitive {prim!r}")
        args = ", ".join(_emit(a, depth + 1) for a in node[2:])
        return f"{surface}({args})"

    if head == "swap!":
        return _emit_swap(node, depth)

    if head == "let":
        # (let [id bound] body)
        if len(node) != 3 or not isinstance(node[1], tuple) or len(node[1]) != 2:
            raise _ConvertError(f"malformed let: {node!r}")
        ident, bound = node[1]
        if not isinstance(ident, str):
            raise _ConvertError("let binder name must be an identifier")
        return f"let {ident} = {_emit(bound, depth + 1)} in {_emit(node[2], depth + 1)}"

    if head == "match":
        return _emit_match(node, depth)

    if head in ("fn", "fix", "fix-group", "make", "seq"):
        # Not expressible in the .myc *expression* grammar at this position
        # (e.g. a nested lambda). Honest refusal — never a fabricated program.
        raise _ConvertError(
            f"node kind {head!r} is not convertible in expression position"
        )

    # Otherwise: application with a bare-identifier callee — (f a b) -> f(a, b).
    return _emit_app(node, depth)


def _emit_app(node, depth: int) -> str:
    callee = _emit(node[0], depth + 1)
    args = ", ".join(_emit(a, depth + 1) for a in node[1:])
    return f"{callee}({args})"


def _emit_swap(node, depth: int) -> str:
    # (swap! src :to <repr> :policy <ref>)  — keyword order tolerant.
    if len(node) < 2:
        raise _ConvertError(f"malformed swap!: {node!r}")
    src = node[1]
    to_repr: str | None = None
    policy: str | None = None
    i = 2
    rest = node[2:]
    j = 0
    while j < len(rest):
        kw = rest[j]
        if kw == ":to" and j + 1 < len(rest) and isinstance(rest[j + 1], str):
            to_repr = rest[j + 1]
            j += 2
        elif kw == ":policy" and j + 1 < len(rest) and isinstance(rest[j + 1], str):
            policy = rest[j + 1]
            j += 2
        else:
            raise _ConvertError(f"unexpected swap! keyword/arg: {kw!r}")
    if to_repr is None or policy is None:
        raise _ConvertError("swap! requires both :to and :policy (never silent — G2)")
    return f"swap({_emit(src, depth + 1)}, to: {to_repr}, policy: {policy})"


def _emit_match(node, depth: int) -> str:
    # (match scrut (pat body) (pat body) …)
    if len(node) < 3:
        raise _ConvertError(f"match needs a scrutinee and >=1 arm: {node!r}")
    scrut = _emit(node[1], depth + 1)
    arms: list[str] = []
    for alt in node[2:]:
        if not isinstance(alt, list) or len(alt) != 2:
            raise _ConvertError(f"match arm must be (pattern body): {alt!r}")
        pat, body = alt
        if not isinstance(pat, str):
            raise _ConvertError(f"unsupported match pattern: {pat!r}")
        arms.append(f"{pat} => {_emit(body, depth + 1)}")
    return "match " + scrut + " { " + ", ".join(arms) + " }"


# ---------------------------------------------------------------------------
# Top-level: LlmCanonical program -> a typecheckable .myc nodule.
# ---------------------------------------------------------------------------


def convert_to_myc(
    source: str,
    *,
    fn_name: str,
    param_type: str,
    return_type: str,
    param_name: str = "x",
    nodule: str = "arm4",
) -> str | None:
    """Convert an ``LlmCanonical`` program to a typecheckable ``.myc`` nodule.

    The model output is expected to be a single LlmCanonical expression. If it is a
    top-level lambda ``(fn [p] body)`` the lambda's parameter name is used and its
    body becomes the function body; otherwise the whole expression is the body and
    ``param_name`` names the (supplied) parameter.

    Returns the ``.myc`` source string, or ``None`` if the output cannot be
    faithfully converted (caller scores that as not-clean — G2, never a false PASS).

    Guarantee tag: **Empirical** (heuristic rewrite; the authoritative type-check is
    ``myc-check`` over the produced ``.myc``).
    """
    if not source or not source.strip():
        return None
    # Strip accidental code fences (models sometimes wrap output despite the prompt).
    cleaned = source.strip()
    if cleaned.startswith("```"):
        lines = [ln for ln in cleaned.splitlines() if not ln.strip().startswith("```")]
        cleaned = "\n".join(lines).strip()
    try:
        ast = _read(cleaned)
        param = param_name
        body = ast
        if isinstance(ast, list) and ast and ast[0] == "fn":
            # (fn [p] body) — adopt the model's parameter name so body vars resolve.
            if len(ast) != 3 or not isinstance(ast[1], tuple) or len(ast[1]) != 1:
                raise _ConvertError(f"malformed top-level fn: {ast!r}")
            lam_param = ast[1][0]
            if not isinstance(lam_param, str):
                raise _ConvertError("lambda parameter must be an identifier")
            param = lam_param
            body = ast[2]
        body_myc = _emit(body)
    except _ConvertError as exc:
        _LOG.debug("LlmCanonical->L1 conversion failed: %s", exc)
        return None

    return (
        f"nodule {nodule}\n\n"
        f"fn {fn_name}({param}: {param_type}) -> {return_type} =\n"
        f"  {body_myc}\n"
    )
