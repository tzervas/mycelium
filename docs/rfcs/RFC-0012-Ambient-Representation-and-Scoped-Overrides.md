# RFC-0012 — Ambient Representation: Declared Paradigm Defaults & Scoped Overrides

| Field | Value |
|---|---|
| **RFC** | 0012 |
| **Status** | **Draft** (2026-06-16) |
| **Type** | Foundational / normative (once Accepted) — surface/term-layer feature; no kernel change |
| **Date** | 2026-06-16 |
| **Depends on** | RFC-0006 §3/§4 (surface language & term-layering — this is a surface-layer feature); RFC-0005 (selection-policy language — the ambient is a reified selection); RFC-0001 §4.5/§4.6 (Core IR; content-addressing / names-as-metadata; WF1/WF2 swap-only repr change); RFC-0007 §4.6 (the elaboration this rides); ADR-006 (selections are reified, inspectable artifacts); ADR-016 (the cross-module ABI is concrete hashes/reprs); G2 (never-silent); tension **A** (the verbosity cost of honesty); KC-3 (small kernel); NFR-7 (the differential) |
| **Proposes** | a **surface elaboration** feature only. It does **not** revise RFC-0001's frozen node set — L0 is unchanged. |

---

## 1. Summary

Mycelium buys its guarantees with surface verbosity: every value annotates its representation, and
every representation change is an explicit [`Swap`] carrying a policy (RFC-0001 WF1/WF2). For a program
that is overwhelmingly *one* paradigm — all binary, or all balanced-ternary, or all VSA — the repeated
paradigm tags are pure noise that offsets the honesty payload (tension **A**).

This RFC proposes a **declared, scoped, paradigm-only default** ("the ambient") plus **scoped override
blocks** with explicit conversions. The whole feature rests on **one principle that keeps it from
becoming a black box**:

> **The ambient is pure *surface* elaboration. It fills in an *omitted paradigm*; it never inserts a
> conversion; and it elaborates to the same fully-explicit L0 a reader would get by writing the program
> longhand.** Identical L0 ⟹ identical content hash (RFC-0001 §4.6) ⟹ a free, checkable proof that the
> sugar changed *what you typed*, not *what the program means* (NFR-7).

Because the ambient touches only the **surface/term layer** (RFC-0006), the **trusted kernel (KC-3) is
untouched**: the reference interpreter, the totality/type checkers, and every certificate still see
fully-explicit L0. Per the maintainer (2026-06-16), v0 covers **the paradigm only** (widths, dims,
dtypes, VSA models stay explicit) and includes **both** the default *and* the override/conversion
blocks.

## 2. Motivation

- **Tension A — verbosity vs honesty.** The honesty rule and never-silent (G2) make annotations
  pervasive. A *declared* default removes the repetition without removing the information: the paradigm
  is still stated once, explicitly, at a named site, and is recoverable everywhere by the formatter
  (M-142) / LSP. The signal-to-noise of the *deviations* rises — the cross-paradigm sections become the
  only thing you write, which is exactly where a reader's attention belongs.
- **A better dev loop without a black box.** The maintainer's framing: QoL for developers "while
  remaining transparent and refusing black boxes." The design's job is to deliver the ergonomics
  *without* the two things that would make it a black box — **inference of repr from usage**, and
  **silent insertion of conversions**. This RFC forbids both (§4.3, §4.4).
- **Easier multi-format transition.** Set a majority paradigm globally; introduce a second paradigm in
  an explicit override block whose boundary conversions are written and **whose missing conversions are
  statically refused** (§4.4). Mixing formats becomes a localized, audited act, cross-module-safe
  because exported signatures are always concrete L0 reprs (§4.5).

## 3. Guide-level explanation

```text
colony image
default paradigm Binary            -- the ambient: omitted paradigms are Binary (paradigm only)

fn mask(x: {8}, m: {8}) -> {8} =   -- {8} ≡ Binary{8} from the ambient; widths stay explicit
  and(x, m)

fn parity(x: {8}) -> {1} =
  -- a localized excursion into ternary, with the conversion written at the boundary:
  with paradigm Ternary {
    let t = swap(x, to: Ternary{6}, policy: rt)   -- explicit Swap (WF1) — the ambient never inserts one
    in count_nonzero(t)                            -- {1} here ≡ Ternary{1} (inner ambient)
  }                                                -- result crosses back: an explicit swap is required,
                                                   -- and its absence is a static MissingConversion error
```

Reading rules a developer internalizes:

- **`{N}` is a paradigm-less repr.** The ambient supplies the paradigm; you still write the size/params.
  `{8}` under `default paradigm Binary` is `Binary{8}`; under `Ternary` it is `Ternary{8}` (8 *trits*).
  A param shape that does not fit the ambient paradigm (e.g. a dtype `{n:F32}` under a `Binary` ambient)
  is an **explicit error**, never a coerced guess.
- **A bare decimal literal adopts the ambient paradigm.** `5` under `Binary` is unsigned bits; under
  `Ternary` it is the balanced-ternary encoding. Its **width comes from the expected type** (the
  checker), or must be annotated — there is **no default width** (paradigm-only; §8 Q1). Paradigm-tagged
  literals (`0b1011`, `<+0->`, dense `[…]`) are unaffected — they already name their paradigm.
- **`with paradigm P { … }` overrides the ambient** for a block (innermost-wins, like a binder).
- **Crossing paradigms is always an explicit `swap`.** Inside or outside an override, a value of
  paradigm A used where B is expected needs a written `swap` (WF1/WF2). The compiler **detects a missing
  one and refuses** (`MissingConversion`) — it never fabricates a conversion.

What the developer is promised: **the program means exactly what it would mean written longhand.** The
formatter can always show the resolved, fully-tagged form (`just`/LSP "expand ambient"), so the
non-local default is never *hidden*, only *elided*.

## 4. Reference-level design (normative once Accepted)

### 4.1 The ambient is a reified, scoped selection (RFC-0005; ADR-006)

A `default paradigm P` declaration is a **reified, inspectable, content-addressed artifact** — the same
posture ADR-006 mandates for every selection. It is the *trivial* selection ("the paradigm of an
omitted annotation is `P`"); it carries **no decision table** and **no swap** (contrast RFC-0002 swap
selection / RFC-0004 packing / RFC-0010 decode, which select among real alternatives). Scope nesting
forms a stack; resolution is **innermost-enclosing-wins**, deterministic, and renderable.

`P ∈ { Binary, Ternary, Dense, Vsa }` — a **paradigm tag only** (RFC-0001 §4.2 paradigms). It never
fixes width/dim/dtype/model/sparsity; those remain explicit at every use (the v0 scope decision).

### 4.2 Surface additions (RFC-0006 grammar)

1. `default paradigm P` — a declaration, valid at colony scope and block scope.
2. `{ <params> }` — the **paradigm-less repr**: the ambient paradigm `P` combined with the written
   `<params>` (whose shape must be `P`'s shape) yields the concrete `Repr`. Equivalent longhand:
   `P{<params>}`.
3. Bare decimal literals resolve to `P`'s encoding (§4.3).
4. `with paradigm P { e }` — a block establishing a nested ambient over `e`.

The fully-tagged forms (`Binary{8}`, `<+0->`, …) remain legal everywhere; the ambient only governs the
*omitted* cases. A program may use **no** ambient at all (status quo) — the feature is opt-in.

### 4.3 Elaboration: meaning-preserving by construction (the honesty core)

A **resolution pass** runs in the surface→L0 elaboration (RFC-0007 §4.6), *before* any L0 is emitted:

- Each paradigm-less `{params}` is replaced by `P{params}` for the nearest enclosing ambient `P`. A
  param shape that is not well-formed for `P` is an explicit `ParadigmShapeMismatch` error.
- Each bare decimal literal is replaced by `P`'s constant at the width demanded by its checked context;
  if the width is not determined, it is an explicit `UnresolvedWidth` error (never a built-in default).
- An omitted paradigm with **no enclosing ambient** is an explicit `UnresolvedAmbient` error — there is
  **no implicit global fallback** (that would be silent).

Two invariants make the pass honest, and are **normative**:

- **(I1) The ambient emits no `Swap`.** Resolution only *fills paradigm tags and literal encodings*; it
  is structurally incapable of introducing a representation change (WF1: only a written `Swap` changes a
  repr). Conversions are always author-written.
- **(I2) Resolution is observationally the identity.** For any program `p` using the ambient and its
  longhand twin `p′` (the same program with every omission written out), `elaborate(p) ≡ elaborate(p′)`
  **structurally** — hence `content_hash(elaborate(p)) = content_hash(elaborate(p′))` (RFC-0001 §4.6;
  names/ambient are metadata, not identity). This is the differential obligation in §4.6.

Elaborated nodes carry provenance `AmbientDefault { site }` (vs `Explicit`) so `EXPLAIN` answers "where
did this paradigm come from?" for every node (no black box; ADR-006).

### 4.4 Override blocks & the missing-conversion refusal

`with paradigm P { e }` sets the ambient to `P` inside `e`. It is **not** a conversion: a value entering
or leaving the block whose paradigm differs from the surrounding context must be bridged by an explicit
`swap` (WF1/WF2). The checker computes the paradigm of each cross-block edge and, for any edge A→B with
`A ≠ B` and **no `swap` in scope bridging A→B**, raises an explicit **`MissingConversion { from, to,
site }`** — the never-silent guarantee the maintainer asked for ("detect missing conversions"). It is
the format-edge analogue of the existing out-of-range → explicit error rule (G2).

This makes a multi-paradigm section a **localized, audited** construct: the override declares the
interior paradigm; the boundary swaps are written and visible; an unbridged edge fails the build with a
precise site, never a coerced value.

### 4.5 Cross-module composition (ties to ADR-016)

The ambient is **module-local sugar**. The resolution pass (§4.3) runs **before** a module's exported
signatures are published, so an exported `fn`'s parameter/return reprs are **concrete L0 reprs** — a
caller never inherits, and is never affected by, a callee's `default paradigm`. The cross-module
boundary is therefore exactly ADR-016's content-hash ABI over concrete reprs; the ambient has no
presence at the boundary. This keeps the feature composable and honest across modules: deviations are
declared *and handled* per module, and "handled cross-module" means the boundary is always explicit.

### 4.6 Verification obligation (NFR-7 / M-210)

Add to the differential corpus a **meaning-preservation** property: for a corpus of programs, each
written *with* an ambient and as an explicit longhand twin, assert `elaborate(p) ≡ elaborate(p′)`
(identical L0 ⟹ identical `content_hash`), and that both run identically on every execution path through
the shared M-210 checker (`ObservationalEquiv`). A mutant (an elaboration that *did* insert a swap or
pick a different repr) is then caught as a hash/observable divergence. This is the executable proof that
the ambient is sugar, not behavior.

### 4.7 Honesty / guarantee orthogonality (VR-5)

The ambient is **orthogonal to the guarantee lattice**: it never reads or writes `Meta`/guarantee
strength. A value's `Exact/Proven/Empirical/Declared` tag is independent of which paradigm an omitted
annotation resolved to. No ambient declaration can upgrade a guarantee (VR-5).

## 5. Drawbacks

- **Local readability now depends on a non-local declaration.** A paradigm-less `{8}` cannot be read in
  isolation. Mitigations (recommended, not normative): the canonical formatter (M-142) and LSP inlay
  hints render the resolved longhand on demand; `EXPLAIN` always shows the resolved paradigm + its site.
  The default is *elided*, never *hidden*.
- **More surface to learn** (override blocks, paradigm-less forms) — a real KISS cost, accepted because
  the win on majority-format code is large and the kernel is untouched (KC-3).
- **A footgun if I1/I2 are ever weakened.** If a future revision let the ambient insert a conversion or
  infer a repr from usage, the black-box property would be lost. I1/I2 are therefore **normative
  invariants**, defended by the §4.6 differential.

## 6. Rationale & alternatives

- **Whole-`Repr` default** (default the entire `Binary{8}`, not just the paradigm). Rejected for v0 by
  the maintainer (2026-06-16): paradigm-only keeps the surface honest about *sizes* (the thing most
  likely to vary within one paradigm) while still killing the dominant repetition (the paradigm tag and
  bare-literal encoding). Recorded as a possible later axis (§9).
- **Infer the representation from usage** (Hindley–Milner-style repr inference). **Rejected**: inference
  from usage is precisely the black box G2/ADR-006 forbid — the chosen repr would be implicit and
  non-local. The ambient is the honest inverse: *declared*, not inferred.
- **Auto-insert conversions at format edges** ("just make it work"). **Rejected**: silent swaps violate
  WF1 and never-silent. The override block + `MissingConversion` refusal is the explicit alternative.
- **Status quo (no defaults).** Rejected: it leaves tension A unaddressed and the DX cost real.

## 7. Prior art

- **Unison** — names are metadata over content-addressed definitions (ADR-003); the ambient is the same
  move applied to an *omitted paradigm*: a convenience over an identity that does not depend on it.
- **Rust/Haskell defaulting** (numeric literal defaulting, `default` declarations) — ambient resolution
  of otherwise-ambiguous literals, but Mycelium constrains it to a *declared* paradigm with no inference
  and an explicit-error fallback, which the mainstream versions do not require.
- **MLIR/dialect default layouts** vs Mycelium's reified packing (RFC-0004 §5) — the consistent
  Mycelium stance: a default is fine *iff* it is reified and inspectable, never an opaque pass.

## 8. Unresolved questions

- **R12-Q1 — bare-literal width.** v0 takes width from the checked context and errors otherwise (no
  default width). Should a later revision allow an optional ambient *width* too (drifting toward the
  whole-`Repr` default of §6)? Deferred.
- **R12-Q2 — override conversion selection.** Must the boundary `swap` be fully written, or may an
  override block opt into the RFC-0005 selection machinery to *choose* the conversion policy (still
  reified/EXPLAIN-able, still never-silent)? v0: fully written; the policy-driven form is a candidate
  follow-on.
- **R12-Q3 — canonical form.** Does the M-142 canonical formatter render *expanded* (longhand) or
  *ambient* form? Identity is over expanded L0 regardless (§4.3 I2); this is a presentation choice.
- **R12-Q4 — VSA paradigm ambient (RFC-0003).** A module may `default paradigm Vsa`, but VSA params
  (model/dim/sparsity) are mandatory and the submodule boundary is opt-in; confirm the interaction is
  just "paradigm tag elided, params explicit," with no change to the RFC-0003 boundary.

## 9. Future possibilities

- A **per-axis default set** (paradigm, then dtype, then dims) layering toward the whole-`Repr` default,
  each axis independently defaultable and independently overridable.
- **Policy-driven override boundaries** (R12-Q2) sharing the RFC-0005 decision-table mechanism.
- **LSP "expand/collapse ambient"** as a first-class projection (RR/G11; M-380), since the resolved and
  elided forms are two projections of the same content-addressed L0.

## Meta — changelog

- **2026-06-16 — Draft.** Initial draft. Proposes a **surface-only** ambient (declared, scoped,
  **paradigm-only**) representation default plus **scoped override/conversion blocks**, per the
  maintainer's 2026-06-16 direction (paradigm-only granularity; full v0 scope incl. overrides). The
  honest core is two normative invariants — **(I1)** the ambient emits no `Swap` and **(I2)** resolution
  is observationally the identity (identical L0 ⟹ identical content hash) — defended by a §4.6
  meaning-preservation differential (NFR-7/M-210). Forbids the two black-box failure modes
  (repr-inference-from-usage; silent conversion insertion); cross-paradigm edges stay explicit `Swap`s
  and a missing one is an explicit `MissingConversion` refusal (G2). The trusted kernel is **untouched**
  (KC-3) — L0's frozen node set does not change; this is RFC-0006 surface/term-layer sugar that
  elaborates away. Cross-module: exported signatures are concrete L0 reprs (ADR-016 boundary), so the
  ambient never leaks across modules. **No code and no RFC-0001 change land with this draft** —
  ratification (Draft → Accepted) and the elaborator/checker wiring (the resolution pass, the
  `MissingConversion`/`UnresolvedAmbient`/`ParadigmShapeMismatch` checks, the M-142/LSP rendering, and
  the §4.6 differential) are the maintainer's append-only decision, presented here first. Append-only.
