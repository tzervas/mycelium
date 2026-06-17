# Spec (Proposed) — `mycfmt`: the canonical formatter contract

| Field | Value |
|---|---|
| **Status** | **Proposed** (2026-06-16 — the M-364 formatter contract; design-first, **present before folding**) |
| **Scope** | The contract `mycfmt` (the standalone canonical formatter — M-142 grows up) must meet: the formatting projection, its three load-bearing invariants (identity-preservation · idempotence · header-preservation), the never-silent error model, the CLI surface, config reading, `EXPLAIN`, and the conformance test plan |
| **Depends on** | M-142 (the Core-IR α-normalizing dump `mycelium_core::lower::format` + the surface printer `mycelium_l1::expand_to_source`); M-358 (DN-06 `// nodule:` marker, `mycelium_l1::parse_nodule_header`); M-359 (`// @key:` structured header + `mycelium-proj.toml`, `mycelium_proj::{parse_header, parse_manifest}`); RFC-0001 §4.6/§4.8 (canonical form; formatting is a projection); ADR-003 (content-addressed identity; metadata ≠ identity); RFC-0006 (the L1 surface); KC-3 (tooling lives **above** the kernel); G2 (never-silent); VR-5 (checked, never fabricated) |
| **Feeds** | M-361 (the full-fat toolchain — `mycfmt` is its formatter); M-366 (lint+fix shares the parse/print path); the M-363 pipeline (formatted sources are the projection input) |
| **Grounds on** | The existing M-142 wiring (`crates/mycelium-lsp/src/fmt.rs`, `crates/mycelium-lsp/src/expand.rs`, `crates/mycelium-l1/src/ambient.rs::expand_to_source`); the `[toolchain].format = "mycfmt-0"` pin already in `docs/spec/Nodule-Header-and-Project-Manifest.md` §2; the grammar conformance corpus `docs/spec/grammar/conformance/{accept,reject}/` |

## 1. Summary

`mycfmt` is the standalone canonical formatter for `.myc` sources — the prod-release tool that M-142's
in-process primitives grow up into. It is a **projection**: it rewrites a source file into one canonical
textual normal form **without ever changing what the file means**. The deepest house rule applies here
unmodified — a definition's **content-addressed identity must not change** under formatting (RFC-0001
§4.6/§4.8; ADR-003). A formatter that could silently alter identity would be the worst kind of black box,
so identity-preservation is not a hope but a **checked invariant** (C1 below), and any input where the
check would fail is an **explicit refusal**, never a lossy rewrite (G2).

This document is the **frozen contract** the implementation must satisfy. It is presented design-first;
no `mycfmt` code lands until the contract is acknowledged (the M-364 gate). The contract is deliberately
small and auditable (KC-3): `mycfmt` is a thin CLI over already-landed, already-tested primitives.

## 2. The formatting projection — what `mycfmt` touches, what it leaves

`mycfmt` normalizes **layout and the header region**; it never edits **meaning**.

| Region | What `mycfmt` does | What it must NOT do |
|---|---|---|
| **`// nodule:` marker** (DN-06 §6) | re-emit in canonical spelling (`mycelium_l1::NoduleHeader::canonical`) as the first line | drop it, invent one, or rename the nodule |
| **`// @key:` structured header** (M-359) | re-emit the present keys in the **canonical key order** (§4) with canonical `// @key: value` spacing | add keys, drop keys, reorder values within a key, or "fix"/fabricate a value (VR-5) |
| **Definition body** | re-print from the parsed AST via the surface printer — canonical indentation, spacing, operator/keyword layout | expand the ambient, insert/remove `Swap`, change literals, change names that are part of identity |
| **Comments inside the body** | *(v0)* preserved-or-refused — see §7 (comments are lexer trivia; the AST does not carry them) | silently delete a body comment |

**The load-bearing subtlety (recorded so it cannot be re-introduced as a bug):** the surface printer
`expand_to_source` is shared with the *"expand ambient"* projection (RFC-0012 §5), which is fed the
**ambient-resolved twin** and therefore *expands* `default paradigm` / `with paradigm` into longhand.
`mycfmt` is a **different** projection: it feeds the printer the **raw parse** (`parse(src)`, *not*
`resolve(parse(src))`), so `default paradigm` and `with paradigm` blocks are **preserved as written**.
Formatting ≠ expanding the ambient. Expanding the ambient changes the surface form (a legitimate, separate
tool); a formatter must not.

## 3. The three invariants (each a checked test, not a claim)

These are stated so the honesty lattice applies: each is **Proven** *only* if its test passes on the
conformance corpus; until then it is **Empirical** (passes on the cases we have) and labelled so (VR-5).

- **C1 — Identity preservation (the load-bearing one).** For every input `s` that `mycfmt` accepts and
  formats to `s'`: the L0 content hash is unchanged —
  `elaborate(parse(strip_header(s)))` and `elaborate(parse(strip_header(s')))` produce the **same**
  `ContentHash` (RFC-0001 §4.6; ADR-003). Equivalently, on the round-trip-safe fragment (§7),
  `parse(s')` is AST-equal to `parse(s)`. *Test:* property test over `grammar/conformance/accept/`
  (every accepted program formats to a same-identity program) + targeted unit cases.
- **C2 — Idempotence.** `format(format(s)) == format(s)` **byte-for-byte**. Formatting reaches a fixed
  point in one pass. *Test:* property over the corpus + targeted cases (already-canonical input is a
  no-op; double-formatting is identity).
- **C3 — Header preservation.** A valid `// nodule:` marker and a valid `// @key:` header survive
  formatting (canonicalized, never dropped); a **malformed** marker/header is an **explicit error**, not a
  silent drop or guess (G2/VR-5), and the file is left untouched. *Test:* the M-358/M-359 header fixtures
  (`crates/mycelium-proj/tests/fixtures/`), incl. `bad-header.myc` → error.

## 4. Canonical header order

When re-emitting the structured header, present keys in this fixed order (the spec §3 table order — the
single source of truth is `mycelium_proj::header::HEADER_KEYS`), each as `// @<key>: <value>`, one space
after the colon, immediately under the `// nodule:` marker, no blank line between marker and keys:

```
// nodule: <dotted.name>
// @version: …
// @license: …
// @authors: …            (comma-separated, one space after each comma)
// @since: …
// @updated: …
// @summary: …
// @repository: …
// @keywords: …           (comma-separated)
// @deprecated: …
```

A bare `// nodule` marker carries no keys (a subnodule that inherits — spec §3); `mycfmt` re-emits the
bare marker unchanged. Reordering keys to canonical order is a **reported** normalization (§6), not a
silent shuffle — and it never changes a value, only its line position (metadata ≠ identity, ADR-003).

## 5. CLI surface & exit codes

Hand-rolled arg parsing (the `myc-check` pattern — **no new dependency**; adding a CLI/TOML crate is an
ADR, not a build detail). Modes are mutually explicit; the default is to print the formatted source to
stdout (never an in-place edit without `--write`).

```
mycfmt [--check | --write] [--explain] [--config <mycelium-proj.toml>] <file.myc | ->...
```

| Flag | Meaning |
|---|---|
| *(default)* | format and write the result to **stdout** (file untouched on disk) |
| `--check` | format in memory, compare to input; **report** which files would change, **write nothing**, exit non-zero if any differ (CI gate) |
| `--write` | format and rewrite the file **in place** — only after a successful, identity-preserving format (never a partial write; §6) |
| `--explain` | also print the `EXPLAIN` of normalizations applied (§6) |
| `--config <path>` | use this manifest's `[toolchain].format` pin (a **hard pin** — §10.3: a version mismatch refuses with exit 4); default discovers `mycelium-proj.toml` upward from each file |
| `-` | read stdin, write stdout |

| Exit code | Meaning |
|---|---|
| `0` | success (formatted; or `--check` and all files already canonical) |
| `1` | `--check`: one or more files would be reformatted |
| `2` | **parse error** — input is not a valid `.myc` program (explicit diagnostic; file untouched) |
| `3` | **header error** — malformed `// nodule:` / `// @key:` (explicit diagnostic; file untouched) |
| `4` | **out-of-scope refusal** — a construct outside the round-trip-safe fragment (§7); `mycfmt` refuses rather than risk identity (C1) |
| `64` | usage error |
| `66` | I/O error |

Codes `2`/`3`/`4` are the never-silent face of the tool: **`mycfmt` never emits a partial or garbled
rewrite** (G2). On any error the on-disk file is exactly as it was.

## 6. `EXPLAIN` / no black box

With `--explain`, `mycfmt` prints a deterministic, line-oriented list of the normalizations it applied —
each one *named*, so the diff is never mysterious:

```
mycfmt: signals/demo.myc
  - re-emitted `// nodule:` marker in canonical spelling
  - reordered 2 header key(s) to canonical order (no value changed)
  - normalized body indentation (2-space)
  - normalized inter-token spacing
  (identity unchanged: blake3:… → blake3:…)        ← the C1 receipt
```

The final line is the **identity receipt**: the content hash before and after, shown equal (C1). A run
that cannot show an equal receipt does not write — it refuses (exit 4). Nothing here is learned or
fuzzy: the normalization set is a fixed, total function of the input (VR-5).

## 7. Honest scope boundary (v0)

`mycfmt` v0 formats exactly the fragment for which `parse ∘ print ∘ parse` is the identity — i.e. where
the surface printer round-trips. This is **checked** against `grammar/conformance/accept/` (the WebAssembly-
spec pattern: the corpus is ground truth). Any construct where round-trip is not yet the identity is **out
of v0 scope** and is an **explicit refusal** (exit 4), *not* a lossy reformat. Better to refuse than to
silently change meaning (G2 over convenience).

Two known printer gaps that the implementation must either close or refuse (recorded so they are not
silently shipped):

1. **Expression parenthesization.** `expand_to_source`'s `print_expr` does not parenthesize nested
   applications/ascriptions; if any `accept/` program re-parses to a different AST after printing, v0
   either (a) adds minimal identity-preserving parentheses, or (b) refuses that construct (exit 4) and
   names it. The choice is per-construct, driven by the corpus round-trip test — never a guess.
2. **Body comments.** Comments are lexer trivia (they never reach the AST), so a naive print drops them.
   v0 must **refuse** (exit 4) a file with body comments it cannot preserve, rather than delete them
   silently. (Header `//`-comments are handled by the header path; this is about *interior* comments.)
   Full comment-preserving formatting (a trivia-attaching parse) is **deferred** and named as such.

This boundary is the whole point: a formatter you cannot trust with identity is worse than none.

## 8. Architecture (KC-3 — above the kernel)

A new workspace crate `crates/mycelium-fmt` (binary `mycfmt` + a `lib` so M-366 reuses the path), depending
only on existing workspace crates — **no new external dependency** (the workspace's deps stay the pinned
`serde`/`serde_json`/`blake3`; a new one is an ADR):

- `mycelium_proj::{parse_header, parse_manifest}` — header parse/validate + the manifest (extended in v0
  to expose `[toolchain].format`; the table is already *accepted* by the reader, M-359, just not yet
  *interpreted* — `mycfmt` is its first consumer, so this is in-scope, not a new schema);
- `mycelium_l1::{parse, expand_to_source}` — body parse + surface print (fed the **raw** parse, §2);
- `mycelium_l1::{check_and_resolve, elaborate}` *(test-only)* — to compute the C1 identity receipt over
  the corpus.

The trusted kernel (`mycelium-core` and below) gains nothing and depends on nothing here.

## 9. Test plan (the acceptance gate)

1. **C1 identity** — property over `accept/`: each program formats to a same-content-hash program; plus
   the M-142 unit cases (α-rename → same canonical text → same hash) lifted to the file level.
2. **C2 idempotence** — property over `accept/`: `format(format(s)) == format(s)` byte-for-byte; canonical
   input is a no-op.
3. **C3 header** — M-358/M-359 fixtures: valid marker+keys round-trip canonically; `bad-header.myc` →
   exit 3, file untouched.
4. **Never-silent** — a `reject/` program → exit 2, **no output file written / stdout is the diagnostic**;
   an out-of-scope construct → exit 4 with a named reason.
5. **Config** — `[toolchain].format = "mycfmt-0"` honored; a pin naming a *different* formatter version is
   an explicit refusal (don't format with the wrong rules); absent config → built-in `mycfmt-0` default.
6. **`--check` / `--write`** — `--check` writes nothing and exits 1 when a file would change; `--write` is
   atomic (temp-then-rename) and never leaves a partial file.

The honesty tag on the suite: C1/C2/C3 are **Empirical** until the corpus property tests are green on the
full `accept/` set, at which point they are **Proven** for that fragment (and explicitly *Declared*-out-of-
scope for the refused remainder).

## 10. Open questions (flagged, not decided)

1. **Parenthesization vs refusal** (§7.1) — per-construct, decided by the corpus round-trip test. Default
   lean: add minimal identity-preserving parens where unambiguous; refuse otherwise.
2. **Body-comment preservation** (§7.2) — v0 refuses; full trivia-preserving formatting is a later task
   (it needs the parser to attach trivia). Confirm v0 may refuse rather than block on it.
3. **`[toolchain].format` semantics** — **Ratified (2026-06-17): hard pin.** `[toolchain].format =
   "mycfmt-0"` is a hard pin: a pin naming a *different* formatter version is an **explicit refusal**
   (exit 4), never a silent format with rules the project did not ask for (G2). `mycfmt` formats only when
   the pin matches its own version (or no pin is present, when the built-in `mycfmt-0` default applies).
4. **Multi-file / project mode** — v0 formats the files named on the CLI; a `mycfmt` over a whole
   `phylum`/surface (manifest-driven discovery) can layer on M-368's resolution. Deferred.

## Meta — changelog

- **2026-06-16 — Proposed (M-364 design).** The `mycfmt` formatter contract, design-first (present before
  folding). Pins formatting as an **identity-preserving projection** (RFC-0001 §4.6/§4.8; ADR-003) with
  three **checked** invariants — C1 identity-preservation (the load-bearing one, with an `EXPLAIN`
  identity receipt), C2 idempotence, C3 header-preservation (DN-06 marker + M-359 `// @key:` header) —
  a never-silent error model (parse/header/out-of-scope refusals; **no partial rewrite**, G2), the CLI +
  exit codes, `[toolchain].format` reading, and the honest v0 **round-trip-safe scope boundary** (refuse,
  never lossily reformat). Architecture: a new above-the-kernel `mycelium-fmt` crate over already-landed
  M-142/M-358/M-359 primitives, **no new dependency** (KC-3). No code lands until this contract is
  acknowledged. Append-only.
- **2026-06-17 — Open question §10.3 ratified.** `[toolchain].format` is a **hard pin** (refuse on
  version mismatch, exit 4 — never format with rules the project didn't ask for; G2). §10.1 (parens vs
  refusal), §10.2 (body comments — v0 refuses), §10.4 (project mode) remain deferred to the first
  implementation pass. Append-only.
