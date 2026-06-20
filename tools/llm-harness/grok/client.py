"""Pluggable chat clients for the Grok/xAI harness (M-330, M-331; never-silent G2).

Three backends behind one tiny protocol (:class:`ChatClient`):

  * :class:`OpenAICompatClient` — the **live** path. Synchronous, one request at a
    time, against the OpenAI-compatible REST surface (``$base/chat/completions``,
    ``Bearer`` key from ``$XAI_API_KEY`` or ``$GROK_API_KEY``). Pure stdlib
    ``urllib`` — no ``requests``/``openai`` dependency. Pacing + backoff live in
    the caller (it owns the per-model :class:`~grok.ratelimit.RatePacer`); the
    client surfaces enough (status, ``Retry-After``, body) to classify a throttle.
  * :class:`XaiBatchClient` — the **batch** path, via the optional native
    ``xai_sdk``. Import is lazy + guarded: if the SDK is absent the constructor
    raises a clear "run ``uv add xai_sdk``" error (never a hard crash on import).
  * :class:`MockClient` — deterministic, offline. The self-test's client: no
    network, reproducible content + token counts seeded by ``(model, prompt)``.

Never-silent (G2): a missing API key, a missing SDK, or an HTTP error is an
explicit exception or a structured error result — never a silent empty string.
Model claims are tagged **Empirical** (live) / **Declared** (mock) by the caller,
per VR-5; the client itself asserts no guarantee strength.
"""

from __future__ import annotations

import json
import os
import time
import urllib.error
import urllib.request
from collections.abc import Sequence
from dataclasses import dataclass, field
from typing import Any, Protocol

XAI_BASE_URL = "https://api.x.ai/v1"
_API_KEY_ENV_VARS = ("XAI_API_KEY", "GROK_API_KEY")


class ApiKeyMissingError(RuntimeError):
    """No API key in the environment (never-silent G2: we refuse to call out blind)."""


class BackendUnavailableError(RuntimeError):
    """A requested backend (e.g. ``xai_sdk``) is not importable/usable here."""


@dataclass
class ChatMessage:
    """One OpenAI-style chat message."""

    role: str  # "system" | "user" | "assistant"
    content: str

    def to_dict(self) -> dict[str, str]:
        return {"role": self.role, "content": self.content}


@dataclass
class ChatResult:
    """The outcome of one chat completion — content + usage + timing (or an error).

    ``ok=False`` carries a structured failure (status/body) for throttle
    classification and never-silent reporting; ``content`` is empty then.
    """

    ok: bool
    content: str
    prompt_tokens: int
    completion_tokens: int
    latency_s: float
    model: str
    finish_reason: str = ""
    status_code: int | None = None
    retry_after: str | None = None
    error: str = ""
    raw: dict[str, Any] = field(default_factory=dict)

    @property
    def total_tokens(self) -> int:
        return self.prompt_tokens + self.completion_tokens


def resolve_api_key(explicit: str | None = None) -> str:
    """Return the API key from ``explicit`` or the environment, else raise.

    Never logs the key. Never-silent (G2): a missing key stops the live run with a
    clear message naming both accepted env vars.
    """
    if explicit:
        return explicit
    for var in _API_KEY_ENV_VARS:
        val = os.environ.get(var)
        if val:
            return val
    raise ApiKeyMissingError(
        "no xAI API key found. Set one of "
        f"{' / '.join(_API_KEY_ENV_VARS)} in the environment "
        "(e.g. `export XAI_API_KEY=...`). Refusing to call the API without a key."
    )


class ChatClient(Protocol):
    """The minimal surface the co-authoring loop and ablation need from a backend."""

    def complete(
        self, *, model: str, messages: Sequence[ChatMessage], **params: Any
    ) -> ChatResult:
        """Run one chat completion synchronously and return a :class:`ChatResult`."""
        ...


# ---------------------------------------------------------------------------
# Live: OpenAI-compatible REST (stdlib only)
# ---------------------------------------------------------------------------


class OpenAICompatClient:
    """Synchronous OpenAI-compatible chat client (the live backend).

    One request per :meth:`complete` call — the caller sequences and paces them.
    Token usage is read from the ``usage`` block the API returns; if a response
    omits it (some compatible servers do), counts fall back to ``0`` and the
    result is flagged (never-silent — a downstream cost of ``$0`` is visibly an
    estimate gap, not a real free call).
    """

    def __init__(
        self,
        *,
        api_key: str | None = None,
        base_url: str = XAI_BASE_URL,
        timeout_s: float = 120.0,
        opener: Any = None,
    ) -> None:
        # resolve_api_key raises (never-silent) if no key is available.
        self._api_key = resolve_api_key(api_key)
        self._base_url = base_url.rstrip("/")
        self._timeout_s = timeout_s
        # ``opener`` is an injection seam for tests (a urllib OpenerDirector-like
        # object exposing ``.open(req, timeout=...)``). Defaults to urllib.
        self._opener = opener

    def _open(self, req: urllib.request.Request) -> tuple[int, bytes, dict[str, str]]:
        if self._opener is not None:
            return self._opener.open(req, timeout=self._timeout_s)
        with urllib.request.urlopen(req, timeout=self._timeout_s) as resp:
            return resp.status, resp.read(), dict(resp.headers)

    def complete(
        self, *, model: str, messages: Sequence[ChatMessage], **params: Any
    ) -> ChatResult:
        url = f"{self._base_url}/chat/completions"
        body: dict[str, Any] = {
            "model": model,
            "messages": [m.to_dict() for m in messages],
        }
        # Pass through known generation params if provided (seed, temperature, ...).
        for k in ("seed", "temperature", "max_tokens", "top_p", "stop"):
            if k in params and params[k] is not None:
                body[k] = params[k]
        data = json.dumps(body).encode("utf-8")
        req = urllib.request.Request(
            url,
            data=data,
            method="POST",
            headers={
                "Content-Type": "application/json",
                "Authorization": f"Bearer {self._api_key}",
            },
        )
        t0 = time.monotonic()
        try:
            status, raw_bytes, headers = self._open(req)
        except urllib.error.HTTPError as exc:
            # 4xx/5xx — capture status + Retry-After + body for throttle handling.
            latency = time.monotonic() - t0
            try:
                err_body = exc.read().decode("utf-8", "replace")
            except Exception:  # pragma: no cover - defensive
                err_body = ""
            return ChatResult(
                ok=False,
                content="",
                prompt_tokens=0,
                completion_tokens=0,
                latency_s=latency,
                model=model,
                status_code=exc.code,
                retry_after=exc.headers.get("Retry-After") if exc.headers else None,
                error=f"HTTP {exc.code}: {err_body[:500]}",
            )
        except (urllib.error.URLError, TimeoutError, OSError) as exc:
            latency = time.monotonic() - t0
            return ChatResult(
                ok=False,
                content="",
                prompt_tokens=0,
                completion_tokens=0,
                latency_s=latency,
                model=model,
                status_code=None,
                error=f"transport error: {exc}",
            )
        latency = time.monotonic() - t0
        # urllib's urlopen raises HTTPError for >=400, so the success path here is
        # 2xx — but a custom opener (or a server that returns 200 with an error
        # envelope) could deliver a non-2xx status as a normal tuple. Guard it so a
        # non-2xx is never silently parsed as success (G2); keep status + Retry-After
        # so the caller can still classify a throttle.
        if status is not None and status >= 400:
            retry_after = None
            if isinstance(headers, dict):
                retry_after = headers.get("Retry-After") or headers.get("retry-after")
            return ChatResult(
                ok=False,
                content="",
                prompt_tokens=0,
                completion_tokens=0,
                latency_s=latency,
                model=model,
                status_code=status,
                retry_after=retry_after,
                error=f"HTTP {status}: {raw_bytes[:500].decode('utf-8', 'replace')}",
            )
        return parse_openai_response(raw_bytes, model=model, latency_s=latency)


def parse_openai_response(
    raw_bytes: bytes, *, model: str, latency_s: float
) -> ChatResult:
    """Parse an OpenAI-compatible chat-completions JSON body into a ChatResult.

    PURE (offline-testable). A malformed body yields ``ok=False`` with the error
    text (never a silent empty success). Missing ``usage`` -> zero token counts
    flagged in ``error`` so the cost gap is visible, but ``ok`` stays True if
    content was present (the call did happen).
    """
    try:
        data = json.loads(raw_bytes.decode("utf-8"))
    except (json.JSONDecodeError, UnicodeDecodeError) as exc:
        return ChatResult(
            ok=False,
            content="",
            prompt_tokens=0,
            completion_tokens=0,
            latency_s=latency_s,
            model=model,
            error=f"could not parse response JSON: {exc}",
        )
    choices = data.get("choices") or []
    if not choices:
        return ChatResult(
            ok=False,
            content="",
            prompt_tokens=0,
            completion_tokens=0,
            latency_s=latency_s,
            model=data.get("model", model),
            error=f"no choices in response: {str(data)[:300]}",
            raw=data,
        )
    msg = choices[0].get("message") or {}
    content = (msg.get("content") or "").strip()
    finish = choices[0].get("finish_reason", "")
    usage = data.get("usage") or {}
    prompt_tokens = int(usage.get("prompt_tokens", 0) or 0)
    completion_tokens = int(usage.get("completion_tokens", 0) or 0)
    note = ""
    if not usage:
        note = "response omitted `usage`; token counts/cost are an estimate gap (0)"
    return ChatResult(
        ok=True,
        content=content,
        prompt_tokens=prompt_tokens,
        completion_tokens=completion_tokens,
        latency_s=latency_s,
        model=data.get("model", model),
        finish_reason=finish,
        error=note,
        raw=data,
    )


# ---------------------------------------------------------------------------
# Batch: native xai_sdk (optional dependency)
# ---------------------------------------------------------------------------


@dataclass
class BatchRequest:
    """One unit of batchable work: a custom id + the messages to complete."""

    custom_id: str
    model: str
    messages: list[ChatMessage]
    params: dict[str, Any] = field(default_factory=dict)


class XaiBatchClient:
    """Native ``xai_sdk`` batch client (the batch backend).

    Construction probes for ``xai_sdk`` and raises a clear, actionable
    :class:`BackendUnavailableError` if it is absent — never a bare ImportError
    mid-run (G2). The SDK surface used follows the brief
    (``Client().batch.create(batch_name=...)``); because the exact method shapes
    can drift across SDK versions, every SDK call is wrapped so a surface mismatch
    surfaces as an explicit, debuggable error rather than a silent partial run.

    NOTE: this client is *not* exercised by the offline ``--self-test`` (it needs
    the SDK + a key + network). It is structured so the live operator can run it;
    its cost accounting flows through the same batch-price path the self-test
    verifies with the Mock client.
    """

    def __init__(self, *, api_key: str | None = None, client: Any = None) -> None:
        self._api_key = resolve_api_key(api_key)
        if client is not None:
            self._client = client
            return
        try:
            from xai_sdk import Client  # type: ignore
        except ImportError as exc:
            raise BackendUnavailableError(
                "the native `xai_sdk` is required for --mode batch but is not "
                "installed. Install it with `uv add xai_sdk` (or "
                "`uv pip install xai-sdk`), then re-run. "
                "Live mode (--mode live) needs only the standard library."
            ) from exc
        # The SDK reads the key from the environment; we resolved it above to fail
        # fast with our own message if it is missing.
        self._client = Client(api_key=self._api_key)

    def submit(self, *, batch_name: str, requests: Sequence[BatchRequest]) -> Any:
        """Create a batch from ``requests``. Returns the SDK batch handle.

        Wrapped so an SDK surface mismatch is explicit (never-silent).
        """
        try:
            batch = self._client.batch.create(batch_name=batch_name)
            for r in requests:
                batch.add(
                    custom_id=r.custom_id,
                    model=r.model,
                    messages=[m.to_dict() for m in r.messages],
                    **r.params,
                )
            return batch.submit()
        except AttributeError as exc:  # pragma: no cover - depends on live SDK
            raise BackendUnavailableError(
                "the installed `xai_sdk` does not expose the expected batch API "
                f"(batch.create/add/submit): {exc}. Check the SDK version against "
                "https://github.com/xai-org/xai-sdk-python and adjust grok/client.py."
            ) from exc


# ---------------------------------------------------------------------------
# Mock: deterministic, offline (the self-test backend)
# ---------------------------------------------------------------------------


@dataclass
class MockScript:
    """A scripted mock reply: the content to return and a fixed token-count pair.

    ``self_correct`` lets a program respond with a *broken* body first, then a
    *fixed* body once the prompt contains feedback (used to exercise the M-330
    correction loop deterministically).
    """

    content: str
    prompt_tokens: int
    completion_tokens: int
    corrected_content: str | None = None
    corrected_prompt_tokens: int = 0
    corrected_completion_tokens: int = 0


class MockClient:
    """Deterministic, network-free client for the offline self-test.

    Replies are driven by an ordered list of :class:`MockScript`, or by a default
    echo when no script covers a call. Script selection rule (predictable, no
    magic): call ``i`` uses ``scripts[i]`` while ``i`` is in range, and **reuses
    the last script** once calls exceed the list. That reuse is what lets a
    single self-correcting script serve a multi-round correction loop: round 1
    (no feedback) returns the broken body; round 2 reuses the same script and,
    because the prompt now carries feedback, returns the corrected body. For
    independent-sample callers (the ablation), pass no scripts and every call hits
    the deterministic echo. Records every call so the self-test can assert on
    ordering, token math and cost.
    """

    GUARANTEE_TAG = "Declared"  # mock output is asserted, not measured (VR-5)

    def __init__(self, scripts: Sequence[MockScript] | None = None) -> None:
        self._scripts = list(scripts or [])
        self.calls: list[dict[str, Any]] = []

    def complete(
        self, *, model: str, messages: Sequence[ChatMessage], **params: Any
    ) -> ChatResult:
        idx = len(self.calls)
        prompt_text = "\n".join(m.content for m in messages)
        has_feedback = (
            "diagnostic" in prompt_text.lower() or "fix" in prompt_text.lower()
        )
        self.calls.append(
            {
                "index": idx,
                "model": model,
                "prompt": prompt_text,
                "params": dict(params),
            }
        )
        if self._scripts:
            # Clamp to the last script once calls exceed the list (reuse rule).
            s = self._scripts[min(idx, len(self._scripts) - 1)]
            if has_feedback and s.corrected_content is not None:
                return ChatResult(
                    ok=True,
                    content=s.corrected_content,
                    prompt_tokens=s.corrected_prompt_tokens or s.prompt_tokens,
                    completion_tokens=s.corrected_completion_tokens
                    or s.completion_tokens,
                    latency_s=0.0,
                    model=model,
                    finish_reason="stop",
                )
            return ChatResult(
                ok=True,
                content=s.content,
                prompt_tokens=s.prompt_tokens,
                completion_tokens=s.completion_tokens,
                latency_s=0.0,
                model=model,
                finish_reason="stop",
            )
        # Default deterministic echo with synthetic but stable token counts.
        content = f"// mock[{model}] reply to {len(prompt_text)} chars\nnodule m\n"
        return ChatResult(
            ok=True,
            content=content,
            prompt_tokens=max(1, len(prompt_text) // 4),
            completion_tokens=max(1, len(content) // 4),
            latency_s=0.0,
            model=model,
            finish_reason="stop",
        )
