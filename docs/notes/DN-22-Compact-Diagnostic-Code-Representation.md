# Design Note DN-22 — Compact Diagnostic Code Representation (packed byte + base-36 string)

| Field | Value |
|---|---|
| **Note** | DN-22 |
| **Status** | **Draft** (2026-06-22; planning/design capture, DN-17 posture) |
| **Feeds** | RFC-0013 (Structured Diagnostics & Reified Error-Handling Policy, **Enacted**) — this is a *projection* over its reified diagnostic, never a replacement; DN-04 (the DynEL basis + the governing "additive, never substitutive" constraint); G2 (never-silent), G11 (multiple projections of one content-addressed artifact), VR-5 (honesty), KC-3 (small auditable kernel), house rule #2 (no black boxes — every selection/representation is `EXPLAIN`-able); ADR-018 (versioning policy — a code is a versioned interface); NFR-2 / SC-5b (semantic feedback for the AI co-author loop, M-330); the M-380 projection framework |
| **Date** | June 22, 2026 |
| **Decides** | *Planning capture, advisory (DN-17 posture) — **not** a ratified decision.* Records the direction for a **compact, machine-first diagnostic code**: a packed exit/status **byte** plus a variable-length **base-36 string** that together encode *which component failed and why* at high granularity in very few tokens — as a **lossless, `EXPLAIN`-able projection** of an RFC-0013 reified diagnostic, optimized for the AI development assistant that will be paired with the language. No kernel change; the reference implementation already exists in the check tooling (`scripts/checks/all.sh`). |
| **Task** | Design capture — issue id to be allocated (not minted here; mitigation #1) |

> **Posture (honesty rule / VR-5).** This note proposes a *transport/compression layer*, not a new
> source of truth. Every claim about the existing surfaces is grounded (RFC-0013 enacted; DN-04
> Resolved; the reference implementation cited by `file`). The one rule below governs the whole idea.

---

## 1. The governing constraint (read this first)

DN-04 fixed the rule that governs all diagnostics, and it governs this one without exception:

> **Diagnostics are *additive presentation* over explicit errors — never a substitute for one.**

A compact code is a **projection** (G11) of a reified RFC-0013 diagnostic — the same "two views of
one content-addressed artifact" stance DN-04 §2 takes for human-vs-JSON output. So:

- The reified diagnostic (RFC-0013) remains the **single source of truth**. The code is a *lossy
  glance* (the byte) plus a *lossless handle* (the b36 string) **into** it — it must always expand
  back to the full structured diagnostic via the registry (§3). A code that could **not** be expanded
  would be the black box house rule #2 forbids.
- The code **never swallows or stands in for** the explicit `Option`/error/refusal the never-silent
  rule requires (G2). It is emitted *alongside* the propagating error, as a denser rendering of it.

The motivation is **NFR-2 / SC-5b + the AI co-author loop (M-330)**: a model paired with the language
consumes a *graded, machine-readable* diagnostic stream. Once it has learned the codebook, a code
like `Fn2` carries extreme granularity at a fraction of the tokens of a prose diagnostic — while
humans and untrained tooling still `EXPLAIN` it back to the full text. This is token-efficiency for
the development-assistance channel, *without* sacrificing honesty or granularity.

## 2. The layered code

A process exit status is a single byte (0–255), and many machine channels want a fixed, tiny summary;
yet a language's diagnostic space is far larger than 256 entries. So the code is **two layers**, a
lossy summary over a lossless handle:

1. **The packed byte (fast-path summary, 0–255).** A glance-level code carrying the highest-value
   fields — for the check runner: `(component << 3) | reason`, i.e. 5 bits of **component** (a stable
   id) and 3 bits of **reason** sub-code. `echo $?` decodes uniquely: `component = code >> 3`,
   `reason = code & 7`. For the language at large the byte should carry **severity + domain** (the two
   fields a consumer triages on first); the byte is deliberately *coarse* — it is the at-a-glance
   field, not the full address. **Per-domain numbering** keeps the 256-value ceiling from binding: the
   byte is unique *within a domain*, not globally.
2. **The base-36 string (lossless handle, unbounded).** A variable-length `0-9a-z`-per-position string
   that is the *real* address into the registry — `F<component><reason>` today (e.g. `Fb2` =
   format/rust-unformatted, `Fn2` = doc-index/index-stale), generalizing to
   `<severity><domain><component-path><reason>[·<payload>]`. Each added position multiplies the space
   by 36, so granularity is effectively unbounded while staying short and copy-pasteable. The string,
   not the byte, is what a model learns and emits.

The byte is what you read at a glance / branch on in a shell; the string is what you log, learn, and
expand. They encode the *same* fields where they overlap (the byte is a truncation of the string's
leading positions), so they never disagree.

## 3. The registry — codes are derived, not invented

For the code to be honest it must be **decodable and collision-free**, which requires a registry that
is the single source of truth (the same "content-addressed declaration, file is a projection" stance
DN04-Q3 / R7-Q4 took for config):

- **Codes are *derived* from the structured diagnostic** (its severity + domain + component path +
  reason class), **never hand-assigned ad hoc** (G2/VR-5) — so two emitters cannot collide or drift,
  and the mapping is mechanical, not editorial.
- **`EXPLAIN <code>` expands to the full RFC-0013 diagnostic** (house rule #2): message, provenance,
  `NotValidatedReason`, fallback, tags. The compact form degrades gracefully — *before* a model has
  learned the codebook, anyone (or any tool) recovers the full meaning from the registry. The codebook
  is itself a content-addressed, queryable artifact (a natural `diag`-module / tooling citizen — cf.
  DN04-Q5).
- **A code is a versioned interface** (ADR-018), exactly like the guarantee tags: **append-only**; a
  code's meaning is **never silently re-pointed**, or a model's learned mapping (and every logged
  code) rots. New reasons get new sub-codes; deprecated ones are tombstoned, not reused.
- **Severity is encoded in the code** so criticality survives compression (a consumer triages without
  expanding). This pairs with RFC-0013's free-form string `tags` (DN04-Q2): the code is the *compressed
  index*; the tags remain the rich, queryable side-channel.

## 4. Reference implementation (already landed)

The check runner is a working proof-of-concept of the whole scheme, at small scale:

- `scripts/checks/all.sh` assigns each gate a **stable component id** (1–24) and packs the process
  exit as `(component << 3) | reason`; it prints a per-failure **base-36 code** `F<c><r>` and a
  **lossless digest** (the failed gate, its reproduce command, and the tail of its actual output — the
  `EXPLAIN`-equivalent for this domain).
- Gates opt into specific **reason sub-codes** (e.g. `format` → 2 rust-unformatted / 3
  python-unformatted / 4 tool-error; `doc-index` → 2 stale / 3 self-test-failed); un-wired gates
  report reason `1` (generic) and rely on the digest tail for the human "why".

This validates the ergonomics (byte decodes uniquely; string is glanceable; the full error is always
one step away) before any language-level commitment.

## 5. Open questions

- **DN22-Q1 — byte field layout for the language.** Confirm the byte's two fields (proposed:
  severity + domain) and their widths, and that numbering is **per-domain** (so 256 never binds).
- **DN22-Q2 — the b36 string grammar.** Fix the positional grammar
  (`<severity><domain><component-path><reason>[·<payload>]`), the separator, and how a variable-depth
  **component path** is encoded (fixed-width segments vs delimited).
- **DN22-Q3 — registry home + format.** Is the codebook a `diag`-stdlib module (DN04-Q5), a tooling
  artifact, or both? It must be content-addressed and `EXPLAIN`-queryable (house rule #2).
- **DN22-Q4 — derivation from RFC-0013.** Pin exactly which diagnostic fields project into which code
  positions, so codes are mechanically derived (and stable) rather than authored.
- **DN22-Q5 — payload tier.** Should the optional trailing `·<payload>` carry small operands (e.g. a
  width, an offset) in-band, or always defer detail to `EXPLAIN`? In-band risks unbounded codes;
  allowlist + cap if allowed (the DN-04 §6 "allowlist, never wholesale-dump" lesson).
- **DN22-Q6 — kernel boundary (KC-3).** Confirm code *generation/expansion* lives in the tooling/diag
  layer, not the trusted kernel — the kernel keeps emitting structured reasoned errors (it already
  does); the projection is downstream.

## Meta — changelog

- **2026-06-22 — Draft (design capture).** Created at the maintainer's request to capture the
  direction for a **compact diagnostic code** — a packed status **byte** plus a variable-length
  **base-36 string** — as a **lossless, `EXPLAIN`-able projection** (G11) of an RFC-0013 reified
  diagnostic, optimized for token-efficient consumption by the paired AI development assistant
  (NFR-2/SC-5b/M-330) while preserving extreme granularity. Records the governing constraint inherited
  from DN-04 (diagnostics are additive over explicit errors, never substitutive — G2), the two-layer
  code (coarse byte over an unbounded b36 handle), the **registry-as-source-of-truth** with codes
  *derived* (not hand-assigned), `EXPLAIN`-decodable (house rule #2), and **append-only versioned**
  (ADR-018, like the guarantee tags), and points at the already-landed reference implementation in the
  check tooling (`scripts/checks/all.sh`). Six open questions (§5) for ratification. Advisory
  (DN-17 posture); not yet resolved; issue id to be allocated. Append-only.
