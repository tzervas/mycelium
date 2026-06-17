# Spec (Proposed) — lint + auto-fix contract (actionable diagnostics, reified fixes, the §4.1 doc lint)

| Field | Value |
|---|---|
| **Status** | **Proposed** (2026-06-16 — the M-366 lint+auto-fix contract; design-first, present before folding) |
| **Scope** | The contract for the suite's lint+fix tool: surfacing the M-141 invariant lints + RFC-0013 diagnostics + the RFC-0015 §9 "class is only logged — add a handler?" lint as **actionable, opt-in, reified** fixes; offering RFC-0014 **recovery scaffolds** (explicit handler skeletons, never an implicit control-flow change); and hosting the M-363 **§4.1 doc quality-bar lint** (the 8 checks over the doc IR — now unblocked by the §8 ratification) |
| **Depends on** | M-141 (`mycelium_lsp::lint` — `implicit-swap`/`unverified-bound`/`placeholder-policy`/`free-variable`/`nodule-header`); RFC-0013 (structured diagnostics); RFC-0015 §9 (the "only logged; add a handler?" lint — direction set, Q4) + M-362 (`mycelium_lsp::baseline`); RFC-0014 I1–I5 (declarative recovery — a handler is explicit, bounded, opt-in; never makes an error vanish); the M-363 §4.1 quality bar + §8-ratified build stack (custom doc-IR + Typst); G2 (no silent rewrite); KC-3 (above the kernel) |
| **Feeds** | M-361 (the full-fat toolchain — the lint+fix tool); the editor/LSP (M-140); the M-363 pipeline (the §4.1 lint gates generated docs) |
| **Grounds on** | `crates/mycelium-lsp/src/lint.rs` (the M-141 `Diagnostic`/`Severity` surface); `crates/mycelium-lsp/src/baseline.rs` (the recovery profiles + the class table); RFC-0014 §I1–I5; the M-363 §4.1/§6 eight checks |

## 1. Summary

M-366 is the suite's **lint + auto-fix** tool. It does two things, under one honesty rule — **no silent
rewrite; every fix is a reified, inspectable, opt-in edit** (G2):

- **(A) Code lint+fix** — surface the existing diagnostics as *actionable* fixes: the M-141 invariant lints,
  the RFC-0013 structured diagnostics, and the RFC-0015 §9 "this class is only logged — add a handler?"
  advisory; plus **RFC-0014 recovery scaffolds** (an explicit handler skeleton the author fills — never an
  implicit control-flow change).
- **(B) Doc lint** — host the M-363 **§4.1 quality-bar lint** (8 checks over the doc IR). This half was
  gated on the §8 build stack; with §8 **ratified** (custom doc-IR + Typst), it is now specifiable.

Presented design-first; no lint+fix code lands until acknowledged (the M-366 gate).

## 2. The fix model — suggest vs apply, and the opt-in boundary (the crux)

Every finding may carry zero or more **fixes**. A fix is a **reified edit description** (what it changes,
where, and why), not a magic mutation. The boundary is bright:

| Fix tier | What it is | How it is applied |
|---|---|---|
| **suggest** (default) | a printed, reviewable edit | **never** auto-applied; shown for the author to accept |
| **apply** (opt-in, `--fix`) | a mechanical, **identity/behaviour-preserving** edit | applied only on explicit `--fix`, and only for fixes marked `safe` |
| **scaffold** (always suggest-only) | a code **skeleton** the author must complete (e.g. a recovery handler) | **never** auto-applied — it is incomplete by design |

The load-bearing rule (RFC-0014 I1/I5 lifted to tooling): **a fix that could change control flow is never
applied automatically.** It is offered as a *scaffold* — an explicit handler skeleton with a `todo` body —
so the author makes the control-flow decision. The tool never silently inserts recovery, swallows an error,
or alters behaviour (A2 of RFC-0015 §4.1: recovery is opt-in, declared, bounded). `--fix` touches only
`safe` (behaviour-preserving) edits; everything else is suggest/scaffold.

## 3. (A) The code lint+fix set

| Lint (code) | Severity | Fix offered | Tier |
|---|---|---|---|
| `placeholder-policy` (M-141) | error | replace the stub digest with a named `PolicyRef` slot | suggest |
| `nodule-header` (M-141/M-359) | error | canonicalize a near-miss marker / fix a malformed `@key` value | suggest (a value is never *fabricated* — VR-5; only obvious spelling/spacing) |
| `unverified-bound` (M-141) | warning | annotate the `Declared` value with a `// why declared` note, or scaffold a verification | suggest / scaffold |
| `implicit-swap` (M-141) | error | scaffold an explicit `swap(…, to:, policy:)` at the mixed-paradigm op | **scaffold** (inserting a swap is a semantic change — author chooses target+policy) |
| `free-variable` (M-141) | error | (no auto-fix — names a real bug) | suggest only (the diagnostic) |
| RFC-0015 §9 **"only logged"** | advisory | "this class is only logged, never handled — add a handler?" → offer an RFC-0014 recovery scaffold | **scaffold** |

The RFC-0015 §9 lint is the new actionable one: using the M-362 baseline + a definition's declared effect
classes, it finds a class that the baseline only *logs* (no handler anywhere) and offers an **RFC-0014
recovery scaffold** — a named profile (`strict`/`resilient`, the closed v0 set) or an explicit
`match`-over-its-cases handler skeleton. It **never** applies one (A2): adding recovery is the author's
declared, bounded, opt-in choice (I4/I5). The scaffold is bounded by construction (e.g. `retry(<=3)` for
`resilient`).

## 4. (B) The §4.1 doc quality-bar lint (now unblocked)

Hosts the M-363 §6 eight checks as explicit pass/fail over the doc IR (the §8-ratified custom doc-IR; the
G2 analogue for docs — a defect is a build failure, not silent rot):

(1) single-template conformance · (2) navigability (no orphan pages, search index) · (3) progressive
disclosure (RFC-0013 `minimal/medium/detailed` levels present where required) · (4) **examples are checked**
(a stale inline example fails the build) · (5) no dead xref (extends `scripts/checks/links.sh` to the site) ·
(6) **dual-projection parity** (the human and JSON forms share one IR node / content hash — G11) · (7) **no
hallucinated prose / undocumented-is-flagged** (every statement traces to the code/spec it projects; a gap
is an explicit "undocumented" marker, never invented) · (8) legibility/accessibility (semantic HTML, alt
text, heading order, contrast).

This half **consumes** the M-363 doc IR; it does not build the pipeline (M-363 build remains separate). It
can be specified and its check logic written against the ratified IR shape; it runs once the IR generator
exists. Until then it is dormant-but-defined (named honestly, not pretended-present).

## 5. `EXPLAIN` / no black box

```
lint: signals/demo.myc
  error  implicit-swap   at let a/op f   mixes [binary, ternary]
         fix (scaffold):  wrap the op in an explicit swap — you choose target repr + policy
  advisory only-logged   class SwapOutOfRange is only logged (baseline: route=audit), never handled
         fix (scaffold):  add a recovery handler — profile `resilient` (retry<=3) or an explicit match
  warning unverified-bound at const   value carries a Declared bound
         fix (suggest):   annotate `// why declared: …`
  applied 0 / suggested 2 / scaffolded 2     (run with --fix to apply the 0 safe edits)
```

Every fix names its tier, so "what would `--fix` change?" is answerable before running it. The tool never
reports an applied edit it did not make, and never makes one it did not report (G2).

## 6. CLI surface & scope

```
myc-lint [--fix] [--explain] [--docs] [--format human|json] [--config <toml>] <path|file.myc|->...
```

`--fix` applies only `safe` (behaviour-preserving) edits; scaffolds/suggests are never auto-applied.
`--docs` runs the §4.1 doc lint (B) over the doc IR (active once M-363's IR generator lands). Hand-rolled
CLI — **no new dependency** (reuses `mycelium-lsp`'s lint/baseline + `mycelium-l1` + `mycelium-proj`).
**v0 scope (honest):** half (A) is fully specifiable and buildable now (the lints + baseline exist); half
(B) is **specified now, built when the M-363 doc IR exists** — named as dormant, not faked. No kernel
change (KC-3).

## 7. Test plan (acceptance gate)

1. **No silent rewrite** — a default run writes nothing; `--fix` applies only `safe` edits and reports each;
   a scaffold is never auto-applied.
2. **Each M-141 lint** — fires on its positive case, clean on its negative, offers the table-3 fix at the
   correct tier; `implicit-swap`/`only-logged` are **scaffold** (never applied).
3. **RFC-0014 boundary** — a recovery scaffold is an explicit, bounded handler skeleton (`retry<=3` /
   match-over-cases); the tool never inserts recovery automatically (A2/I5) and never makes an error vanish
   (I1).
4. **Honest tags (VR-5)** — a `nodule-header`/value fix never *fabricates* a value; it only canonicalizes
   obvious spelling/spacing.
5. **(B) doc lint** — each of the 8 §4.1 checks passes/fails explicitly over a fixture doc IR; check 4
   (stale example) and check 7 (undocumented-is-flagged) demonstrated.
6. **EXPLAIN / JSON** — the fix tiers are reported deterministically; `--format json` emits the structured
   findings+fixes (G11).

## 8. Open questions (flagged, not decided)

1. **`safe`-edit set** — exactly which fixes are behaviour-preserving enough for `--fix` (header
   canonicalization yes; anything touching expressions → scaffold). Confirm the conservative default.
2. **§9 scope** — the "only logged" lint needs a definition's declared effect classes (RFC-0014 I3). v0
   uses the explicitly-declared set; whole-program effect inference is deferred.
3. **(B) activation** — the doc lint is specified now, dormant until M-363's IR generator exists. Confirm
   it may ship dormant-but-defined rather than block on the pipeline build.

## Meta — changelog

- **2026-06-16 — Proposed (M-366 design).** The lint+auto-fix contract, design-first. Two halves under one
  rule — **no silent rewrite; every fix is reified + opt-in** (G2): **(A)** the M-141 invariant lints +
  RFC-0013 diagnostics + the RFC-0015 §9 "class only logged — add a handler?" advisory as **actionable**
  fixes, with a bright **suggest / apply / scaffold** boundary — a control-flow-changing fix (an explicit
  `swap`, an RFC-0014 recovery handler) is offered **only as a scaffold**, never auto-applied (A2/I1/I5 —
  recovery stays declared, bounded, opt-in); **(B)** the M-363 **§4.1 doc quality-bar lint** (the 8 checks
  incl. *checked examples* and *undocumented-is-flagged*), now specifiable after the **§8 ratification**
  (custom doc-IR + Typst) — built when the M-363 IR generator lands, dormant-but-defined until then.
  `--fix` applies only `safe` behaviour-preserving edits; `EXPLAIN` names every fix's tier. **No new
  dependency**; above the kernel (KC-3). No code lands until acknowledged. Append-only.
