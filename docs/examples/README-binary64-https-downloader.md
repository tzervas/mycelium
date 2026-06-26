# Example — Binary[64] basics + a secure HTTPS downloader

**Status: ILLUSTRATIVE design-phase source — NOT runnable.** Mycelium is in the design phase; there
is no executor for surface `.myc` yet. This example exists to show *idiomatic, lexicon-correct*
Mycelium, not to be compiled or run. Treat it as documentation.

File: [`binary64-https-downloader.myc`](./binary64-https-downloader.myc)

## What it demonstrates

**Part 1 — conventional programming over `Binary[64]`.** A 64-bit binary value used the way a
general-purpose program uses a machine integer: arithmetic (`checked_add`, `checked_div`),
comparison/selection (`min64`, `clamp_upper`), a sum type + pattern match, and a bounded `for`-style
fold over a linked list (`sum_readings`). Every partial operation is **never-silent** (G2):
overflow and divide-by-zero are explicit `Result` `Err`s the caller must handle — no wrap, no trap,
no sentinel. Staying inside `Binary[64]` is **not** a `swap` (no representation change — S1).

**Part 2 — a small HTTPS artifact downloader** with decent, *named* security practice. Several
properties are made **structural** (unrepresentable-when-wrong) rather than merely checked:
`TlsPolicy` has no "disabled" variant, `Url` can only be built through the HTTPS-only constructor,
and `Budget` has mandatory finite timeout/size fields (no "unlimited").

| # | Security practice | How it shows up |
|---|---|---|
| S-1 | TLS certificate verification ON, never disabled | `TlsPolicy = Verify(roots) \| SystemRoots` — no off-switch variant; `tls_connect` propagates `TlsHandshakeFailed` |
| S-2 | HTTPS-only, reject http/downgrade | `parse_https_url` is the sole `Url` constructor; non-`https://` → `Err(NotHttps)`, never a silent rewrite |
| S-3 | Bounded / streamed reads (no unbounded buffer) | `read_body_capped` folds chunks under a hard `max_bytes`; exceeding it aborts with `TooLarge` mid-stream |
| S-4 | No secrets in source | `read_bearer_token` reads the secret from the env via the `io` effect; only the *variable name* is in source |
| S-5 | Never-silent error handling on every fallible step | every step returns `Result` and is propagated via `and_then`; no path proceeds past an `Err` |
| S-6 | Integrity + size check on the artifact | `verify_artifact` checks pre-stated expected length (Exact) and BLAKE3 digest (Declared) before returning |
| S-7 | Timeout / budget | `Budget(timeout_ms, max_bytes)` bounds the handshake and the streamed read |

## Honesty / guarantee posture (VR-5)

Guarantee tags are **per-operation** and never upgraded past their basis:

- **Exact** — total operations over finite domains (`min64`, the `Binary[64]` length equality in
  the size check).
- **Declared** — type-level / effectful contracts, and anything delegated `via` an audited host
  primitive (TLS handshake, BLAKE3 digest, env read). We *assert* these, we do not *prove* them
  here — so they sit at the transparent floor, not `Proven`.

No operation claims `Proven` (that would need a theorem with checked side-conditions).

## Caveat — surface syntax is the DN-31 *accepted-but-not-yet-landed* direction

The example is written in the **DN-31 delimiter direction** (Accepted, recorded; epic still open):
`[]` for type args and sized/repr types (`Binary[64]`, `Result[A, E]`, `Option[A]`), `=>` as the
return arrow, and `<>` freed for operators. **This is not yet the landed grammar.** The
parser-verified conformance corpus (`docs/spec/grammar/conformance/`) and the self-hosted std
(`lib/std/*.myc`) still use the prior forms — `Binary{N}`, `<A, E>`, and `->`. So this file will
*not* parse against today's `mycelium.ebnf`; it illustrates the agreed target surface. When the
DN-31 grammar-supersession wave lands, the rest of the corpus moves to match it.

A few referenced helpers (`add`, `lt`, `div`, `is_zero`, `byte_len`, `eq64`, `bytes_eq`,
`starts_with`, `env_get`, `platform_tls.handshake`, `stream_fold`, `blake3`) are **assumed std/host
surface**, shown to make the example concrete; they are illustrative, not a claim that each exists
in `lib/std` today.
