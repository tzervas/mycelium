# DN-90 — A Unified, Extensible Strategy for Arbitrarily-Large Values / Byte-Sizes Across the Value Model

| Field | Value |
|---|---|
| **Note** | DN-90 |
| **Status** | **Draft** (2026-07-08; BACKLOG / plan-around — a design artifact, decides nothing normatively. Advisory, `Declared` throughout — house rule #3, never jumps to Accepted). |
| **Task** | proposed tracking id **not minted** (mitigation #1 — the orchestrator/maintainer verifies a free `E*`/`M-xxx` slot before minting). This note is deliberately **non-blocking** and lives on a **non-conflicting worktree** (maintainer directive, 2026-07-08). |
| **Related** | **RFC-0033** (Value Model, Collections & Precision — the per-representation size fields this note unifies: `Binary{width}`, `Ternary`, `Seq/Bytes{len:u32}`, `Dense/Vsa{dim:u32}`) · **ADR-025** (Seq/Bytes length-in-type `u32`; growables are higher structures over a fixed-capacity `Seq` — **Accepted** 2026-07-01) · **ADR-026** (Seq elements are repr-values — **Accepted**) · **ADR-028** (Binary is sign-free ⇒ unsigned magnitude is the whole semantics) · **ADR-029** (arbitrary-width balanced-ternary arithmetic — the digit-serial reference `BigTernary` implements) · **ADR-030** (Dense quant granularity descriptor — a **content-address one-way door**) · **DN-41** (`width_cast` — the only kernel mechanism to change a `Binary{n}` width; the never-silent checked-narrow model this note generalizes) · **DN-42** (width generics — the width-polymorphic surface `width_cast` lowers) · **DN-34** §8.16 + the CU-7 ruling (the verify-first correction: `ternary::add`/`mul` are already arbitrary-width; only the *conversion* utilities are i64-capped; `BigTernary` exists-but-unsurfaced; **FLAG-cu7-e20-1-gate**) · **RFC-0036** §FLAG-C (the `WidthRel` prim-signature gap `width_cast` exposes) |
| **Grounding** | `crates/mycelium-core/src/binary.rs:140,257,299,362,585` (the five `*_MAX_WIDTH = 64` caps) · `crates/mycelium-core/src/ternary/mod.rs:14-30,108,118,126,164,197` (the CU-7 i64-cap-vs-digit-serial split) · `crates/mycelium-core/src/ternary/big_ternary.rs:1,20-23,46` (`BigTernary`, never-silent fixed-width boundary) · `docs/adr/ADR-025-*.md` §Decision · `docs/adr/ADR-030-*.md` §Status/§Decides · `docs/rfcs/RFC-0033-*.md` §4.3.2/§7 · `tools/github/issues.yaml:7325-7394` (E20-1 epic, `in-progress`; the M-780 rehash deferred post-1.0) |
| **Guarantee** | **`Declared`** throughout — a proposal / design direction. Nothing here is implemented, enacted, or checked; every normative claim cites a ratified ADR/RFC/DN or is marked an open question (house rule #4). The *current-state* survey (§2) is **`Empirical`** (grounded in cited source at `file:line` — but the citations are ground truth, not this note). |

> **The maintainer's ask (captured, 2026-07-08).** Mycelium today handles size **ad-hoc per
> representation** — Binary caps at 64-bit arithmetic, ternary conversion caps near 40 trits, Seq/Bytes
> length and Dense/VSA dimension are each a `u32` field. The maintainer wants **"an intelligent,
> sensible, future-extensible way of handling arbitrarily-large values / byte sizes across the board
> (within reason — a foundation we extend as needed)"**, **designed before implementing**, on a
> non-conflicting lane. This note records that design space. It **decides nothing** — it surveys the
> honest current state (§2), proposes a coherent cross-representation *shape* (§3), **evaluates** (does
> not choose) the provider/consumer service architecture the maintainer floated against at least one
> alternative (§4), **flags** the E20-1 content-address coupling as a hard design gate (§5), and
> separates the incrementally-landable pieces from the held pieces (§6).

---

## 1. Problem statement

Size is a **first-class parameter of every `Repr`** — `Binary{width}`, `Ternary` (trit count),
`Seq{elem, len}` / `Bytes{len}`, `Dense{dim, …}` / `Vsa{dim, …}` — yet each representation reaches
its *upper bound* by a **different, ad-hoc mechanism**, and those mechanisms are **implementation
artifacts**, not value-model decisions:

1. **Machine-codec ceilings (Binary, Ternary-conversion).** `Binary` arithmetic caps at **64 bits**
   because its exact codec routes through an `i64`/`u64`/`i128` intermediate; balanced-ternary
   *conversion* utilities cap near **40 trits** because `3^m` overflows `i64` at `m ≥ 41`. Neither cap
   is a semantic limit — both are "how wide can the machine-int codec stay exact" boundaries.
2. **`Repr`-field ceilings (Seq/Bytes length, Dense/VSA dim).** Length and dimension are each a
   fixed-size **`u32`** in the `Repr` type (ADR-025; RFC-0033 §4.3.2). The `u32` is a KC-3 economy
   choice, not a considered "largest value we ever want" — and there is **no growable form** above it
   in the value model.
3. **Growable paths that already exist but are unsurfaced or unbounded-but-hidden.** The digit-serial
   `ternary::add`/`mul` are **already arbitrary-width** (no `i64` in the loop) and `BigTernary`
   (M-756, ADR-029) is a growable balanced-ternary integer that **never overflows** — yet **neither is
   surfaced as a Mycelium value form or prim** (DN-34 §8.16 CU-7). The growable capability exists in
   the Rust kernel and is invisible to the language.

The result is that "arbitrarily-large value" means four different things in four places, each capped
by a different accident, with one representation (ternary) silently *already* growable in Rust and
capped only at the language surface. There is no single, honest, extensible notion of "this value can
grow, here is the tier it is in, here is exactly where it refuses." That absence is what this note
addresses — **not** by reimplementing the growable paths that exist, but by giving them one coherent
model and surfacing them uniformly and never-silently.

## 2. Current state — verify-first survey (`Empirical`, cited)

Documented honestly *before* proposing, per the maintainer directive (don't propose reimplementing
implemented/partial work). Every row is a `file:line` citation; the source is ground truth.

### 2.1 Binary{N} — capped at 64 (arithmetic/shift/float-conv); bitwise unbounded

- Five **separate** per-op-family caps, all `= 64`, in `crates/mycelium-core/src/binary.rs`:
  `MUL_MAX_WIDTH` (`:140`), `DIV_MAX_WIDTH` (`:257`), `SHIFT_MAX_WIDTH` (`:299`), `TC_MAX_WIDTH`
  (`:362`, the `add`/`sub`/`neg` two's-complement cap), `FLOAT_CONV_MAX_WIDTH` (`:585`). Each gates
  `len > *_MAX_WIDTH → None` — a **never-silent refusal**, never a truncation.
- The cap's rationale is the `i64`/`u64`/`i128`-intermediate exactness bound (`binary.rs:135-140`).
  There is **no single `BINARY_MAX_WIDTH` umbrella** and there is **no arbitrary-width Binary value
  form** today.
- **Bitwise / structural** ops are deliberately **width-unbounded** (`binary.rs:543`) — so the value
  model already tolerates unbounded-width `Binary` for the ops that need no codec.
- **`width_cast` (DN-41)** is the **only** kernel mechanism to change a `Binary{n}` width
  (`RFC-0036:229`); **DN-42 width generics** is the width-polymorphic surface it lowers. Its
  never-silent **checked-narrow** contract (widen = zero-extension `Exact`; narrow refuses unless the
  dropped high bits are zero) is the template this note generalizes to a cross-representation
  "checked size-narrow." **RFC-0036 §FLAG-C** flags that the `WidthRel` prim-signature enum cannot yet
  express `width_cast`'s per-instance output width — a real, open signature-model gap.

### 2.2 Ternary — the i64-cap-vs-digit-serial split (already corrected by DN-34/CU-7)

- **i64-capped conversion utilities** (decimal-literal encoding + oracle tests only), in
  `crates/mycelium-core/src/ternary/mod.rs`: `max_magnitude(m) -> Option<i64>` returns `None` at
  `m ≥ 41` (`:108`); `trits_to_int` Horner-folds into `i64` (`:118`); `int_to_trits` round-trips a
  value through `i64` (`:126`). These are the ~40-trit ceiling.
- **Digit-serial arithmetic — NOT i64-capped, arbitrary-width, never-silent:** `add` (`:164`), `sub`,
  `mul` (`:197`) ripple/accumulate over `&[Trit]` with **no `i64` in the loop**; overflow is a
  non-zero final carry → `None` (structural, never a wrap). The module docstring (`mod.rs:14-30`) is an
  explicit **CU-7 correction** stating these are correct at *any* width `m`.
- **`BigTernary` (M-756, ADR-029)** — the growable form (`big_ternary.rs:46`, a canonicalized
  LSB-first `Vec<Trit>`) whose arithmetic **never overflows** (a carry out of the top digit becomes a
  new digit); the only never-silent boundary is a **checked narrow back to a fixed width**
  (`checked_to_width` / `checked_add_fixed` return `None` when the result needs `> N` trits,
  `big_ternary.rs:20-23`). It is **re-exported in Rust** (`mycelium_core::ternary::BigTernary`) **but
  is NOT a Mycelium value form or prim** — explicitly excluded from the prim table
  (`prim_map.rs:49`); no `.myc` growable-Ternary type exists (CU-7 growable value form deferred).
- **Verify-first correction (DN-34 §8.16, landed):** the "~40-trit cap on runnable `trit.*`" was
  **inaccurate** — the runnable arithmetic is already arbitrary-width; a width-80 three-way
  (L1/L0/AOT) test locks that in. Only the *conversion* utilities and the *growable value form* remain
  the real gaps.

### 2.3 Bytes / Seq — length-in-type `u32` (ADR-025, Accepted)

- `Repr::Seq { elem, len: u32 }` and `Repr::Bytes { len: u32 }` carry length **in the `Repr` type** —
  a fixed-size `u32` parameter, not variable-length data (ADR-025 §Decision, **Accepted** 2026-07-01).
- **Growables** (`Vec`/`DynamicSeq`) are **higher structures over a fixed-capacity `Seq`** (capacity +
  length, chunked, or COW), **not** kernel primitives (ADR-025 §Decision/§Rejected). So the value
  model **already** locates "grow beyond the fixed size" *above* the kernel — an important precedent
  for §3/§4.
- Elements are homogeneous **repr-values** (ADR-026, Accepted); the length cap is thus a pure `u32`
  bound with growth deferred to a structure.

### 2.4 Dense / VSA — dimension is `u32`; no growable dim

- `Repr::Dense { dim: u32, dtype, quant }` and `Repr::Vsa { model, dim: u32, elem, sparsity }`
  (RFC-0033 §4.3.2, `:130,147`). **`dim` is a `u32`** — no cap tighter than `u32::MAX`, and **no
  arbitrary/growable dimension form** exists.
- Per-element scale/zero-point arrays are `O(dim)` and live in `Payload`, never `Repr` (RFC-0033
  `:139-143`) — the same "big data lives in the payload, the size scalar lives in the type" split as
  Seq/Bytes.
- **ADR-030** locks the Dense `QuantDesc` **into the `Repr`** and therefore **into content-address
  identity** — a **content-address one-way door** that lands in the single E20-1 rehash **before any
  Dense value is persisted** (ADR-030 §Status/§Decides; RFC-0033 §7). This is the coupling §5 flags.

### 2.5 E20-1 — the content-address settlement (in-progress; rehash deferred post-1.0)

- Epic `E20-1` (`issues.yaml:7325-7394`) is **`in-progress`**. Its design half is done (RFC-0033 +
  ADR-025/026/027/028 Accepted 2026-07-01; ADR-029/030/031 Accepted 2026-06-24), but the
  **single content-address rehash (M-780)** and the swap/guarantee reconciliation (M-781) are
  **UNMET and deferred to a post-1.0 wave**. Only V0 (BigTernary, M-754…M-757) lands before 1.0.0.
- E20-1 **gates** any change to content-address identity — specifically the Quantized-Dense and
  element-space/block-sparse/complex-VSA paths, held today as explicit never-silent refusals
  (`issues.yaml:5735-5737,5767`). A **growable `Repr` payload shape** is exactly such a change.

## 3. A coherent cross-representation strategy (proposal, `Declared`)

**The unifying observation.** All four representations already share one shape: a **size scalar in the
`Repr`/type** (`width` / trit-count / `len` / `dim`) plus, where the data is large, a **payload that is
`O(size)`**. What differs is only *how each reaches its ceiling* and *whether a growable tier exists*.
So the strategy is not a new data model — it is **one size-capability model layered over the shape that
already exists**, applied uniformly.

### 3.1 A size-tier lattice (the never-silent core)

Define, across every representation, a small **size tier** with a never-silent narrow between tiers.
This is a direct generalization of DN-41's `width_cast` (widen = exact; narrow = checked, refuses on
loss) from `Binary` widths to *all* size parameters:

| Tier | Semantics | Cost | Guarantee posture |
|---|---|---|---|
| **Machine** (`fast` default) | size backed by a machine int (Binary ≤ 64; ternary conversion ≤ 40; `u32` len/dim) | fastest; the current behavior | `Exact` **within** the cap; **never-silent refuse** at the boundary (already true — §2) |
| **Wide** (fixed, digit-serial) | an explicit fixed size *beyond* the machine cap, via a digit-serial codec (ternary already has this in `add`/`mul`; a binary analogue is constructible) | bounded, checked; the size is still fixed and known | `Exact` within the declared width; **never-silent refuse** on out-of-range narrow-back |
| **Growable** (arbitrary) | a `Vec`-of-digit / capacity-backed form whose ops **never overflow** (a carry becomes a new digit; a push grows capacity) — `BigTernary` is the exemplar | allocation + digit-serial cost; unbounded | `Exact` unbounded; the **only** never-silent boundary is a **checked narrow to a fixed tier** |

**Invariants that bind at every tier (house rules #1/#2, ADR-025/028):**

1. **Never-silent (G2).** Exceeding a representable bound is **always** an explicit `Option`/error —
   never a silent cap, wrap, or truncation. Every representation already does this at its machine cap
   (§2.1–§2.4); the tier model just makes the *escape upward* (widen to a larger tier) and the
   *checked descent* (narrow back) equally explicit and equally reified.
2. **Honesty-tagged (VR-5).** The size tier is part of a value's guarantee story: a machine-tier result
   is `Exact` within its cap; a growable-tier result is `Exact` unbounded; a narrow-back's fit
   contract is `Declared`/never-silent (asserted + exhibited, not proven) — exactly DN-41's posture,
   lifted.
3. **Growth lives above the fixed kernel, size lives in the type (ADR-025).** The kernel stays
   small (KC-3): fixed-size `Repr` + a payload; the **growable tier is a higher structure** over a
   fixed-capacity form, precisely as ADR-025 already decides for `Seq`. The tier model does not put
   variable-length data in the kernel `Repr`; it standardizes the *structure* that grows over it.
4. **A tier change is a swap (never silent).** Moving machine → wide → growable (or narrowing back)
   **is a representation swap** in Mycelium's existing sense — reified, `EXPLAIN`-able, and tagged.
   The value model already has the mechanism; this reuses it rather than inventing an allocator path.

### 3.2 How each representation maps onto the tiers

- **Binary.** Machine = today's `≤ 64` exact codec plus unbounded bitwise. Wide = a digit-serial
  `add`/`mul`/`div` beyond 64 (the ternary digit-serial code is the proof-of-shape; a `Binary`
  analogue is a bounded, checked computation). Growable = a big-integer form (a `Vec<limb>` that never
  overflows), narrowed back via a `width_cast`-style checked narrow. **`width_cast`/DN-42 are the
  existing widen/narrow bridge** — the tier model extends their reach, it does not replace them.
- **Ternary.** Machine = the i64-capped conversion utilities. Wide = the **already-arbitrary-width**
  digit-serial `add`/`mul` (fixed declared width, checked). Growable = **`BigTernary`, already built**
  (ADR-029) — the task is to **surface** it as a Mycelium value form/prim, not to reimplement it
  (§6). This is the representation where the growable tier is *most* ready — and *most* blocked (§5).
- **Bytes / Seq.** Machine = `len: u32` in the `Repr`. Growable = the ADR-025 higher structure
  (`DynamicSeq`/COW over a fixed-capacity `Seq`). The tier model gives that growth a *named,
  never-silent* boundary (a checked narrow of a dynamic seq back to a fixed `Seq{len}` refuses if it
  does not fit) and a consistent honesty tag — rather than each collection reinventing it.
- **Dense / VSA.** Machine = `dim: u32`. A wide/growable `dim` is the **most E20-1-coupled** case: the
  `dim` and (for Dense) the `QuantDesc` are in the `Repr` and thus in **content-address identity**
  (ADR-030) — so a growable-`dim` form changes identity and is **gated on the E20-1 rehash** (§5).

### 3.3 What the strategy explicitly is *not*

- **Not** a new machine-int size beyond `u32`/64 baked into every `Repr` (that would be a
  content-address change with no growth story — worst of both).
- **Not** variable-length data in the kernel (ADR-025 rejects it).
- **Not** a reimplementation of the growable paths that exist (`BigTernary`, digit-serial ternary,
  ADR-025 higher structures) — it is a **uniform model plus surface** over them.

## 4. EVALUATE (do not decide) — the provider/consumer service candidate vs. an alternative

The maintainer floated a **provider/consumer service** architecture as a *possibly*-fitting shape.
Presented here as **one candidate with tradeoffs, to assess — not decided** — with a contrasting
alternative. **This section chooses nothing** (house rule #3).

### 4.1 Candidate A — a size/capacity **provider service** (consumer ops request capacity)

A runtime/kernel **service** *provides* size, growth, and allocation policy to *consumer*
representation-ops, decoupling "how large can this get, and how does it grow" from each
representation's arithmetic. A `mul` on a growable value would *consume* capacity from the provider;
the provider owns the growth/refusal policy and could have pluggable backends (machine / wide /
growable / GPU-arena).

- **What it buys.** A **single extensible policy point** — one place to change growth/refusal
  semantics for *all* representations; **pluggable backends** (a GPU/arena provider for Dense/VSA, a
  bignum provider for Binary/Ternary) behind one interface; uniform accounting/limits (a natural home
  for a "within reason" global size budget, resource-limit enforcement, provenance of allocations).
- **What it costs.** (1) **KC-3 tension** — a service crossing into the small, auditable kernel adds
  indirection and a **black-box risk** (house rule #2): every capacity decision must stay
  `EXPLAIN`-able, which a service tends to hide. (2) **Value-semantics friction** — Mycelium is a
  **value-semantics** language; threading a *service handle/capability* through op signatures pulls
  toward reference/effect semantics and away from "a value is a value." (3) **Coupling** — ops become
  coupled to a runtime service's lifecycle and configuration; reproducibility/determinism now depends
  on the provider's state. (4) **Blast radius** — every representation-op signature grows a provider
  parameter (or an ambient capability), a large surface change.

### 4.2 Candidate B (contrast) — a **type-level size-tier** parameter, growth as a swap (no service)

Encode the size tier **in the `Repr`/type** (a tier tag / a `Growable` marker) and resolve size policy
**statically at check/elaboration time**, with tier changes expressed as Mycelium's existing
**never-silent swap**. No runtime service; the "provider" is the type system plus the swap mechanism.

- **What it buys.** Stays **inside value semantics** (the tier is *in the value*, not a handle);
  keeps the **kernel small** (no service); **reuses the swap mechanism** (reified, `EXPLAIN`-able,
  tagged — houses rules #1/#2 for free); **aligns with ADR-025** ("growables are higher structures
  over a fixed-capacity form" — a *structure/type* concern, not a service) and with **DN-41**
  (`width_cast` is already a type-directed, never-silent size change). Determinism is preserved (no
  external state).
- **What it costs.** **Less runtime flexibility** — a pluggable GPU/arena backend is harder to slot in
  behind a static type than behind a service; a **global "within reason" budget** / resource-limit is
  less natural (it becomes a check-time or per-swap policy rather than one accounting service);
  more of the growth machinery is compile-time, which is less dynamically reconfigurable.

### 4.3 The assessment (not a decision)

Candidate B is **closer to the grain of existing decisions** — value semantics, KC-3 small kernel,
the swap mechanism, ADR-025's above-the-kernel growth, DN-41's type-directed checked narrow — and buys
never-silence plus `EXPLAIN`-ability with less new surface. Candidate A **buys pluggable backends and a
single policy/accounting point** (attractive for GPU/arena Dense-VSA and for a global size budget)
**at the cost of** KC-3 indirection, value-semantics friction, and a black-box risk that must be
actively countered. A **hybrid** is conceivable (type-level tiers for the *semantics*; an optional
provider *only* for backend allocation of the growable payload, kept strictly `EXPLAIN`-able) — noted
as a third point in the space, **also undecided**. **The maintainer chooses; this note only lays out
the tradeoffs** (VR-5 — no assent past its basis).

## 5. ⚑ FLAG — the E20-1 content-address coupling is a hard design gate (do NOT guess it)

**This is the load-bearing flag of the note.** A **growable `Repr` payload** — the growable tier of
§3.1 for any representation whose size scalar or payload shape lives in content-address identity —
**couples to the E20-1 content-address one-way doors** and **must not be designed around or guessed**.
The coupling is already flagged in the codebase as **FLAG-cu7-e20-1-gate** (DN-34 §8.16):

- **Why it couples.** Content-address identity is computed over the `Repr` (and, for the one-way-door
  cases, descriptor fields). ADR-030 locks the Dense `QuantDesc` **into** identity; RFC-0033 §7
  mandates a **single content-address rehash (M-780) before any value is persisted**; changing a
  representation's size/payload shape to a growable form **changes that identity**. Doing it *after*
  values are persisted is the expensive one-way door E20-1 exists to avoid.
- **Which pieces are BLOCKED on the E20-1 settlement (hold — do not implement):**
  1. A **genuinely growable Ternary value form** (CU-7) — surfacing `BigTernary` as a growable
     `Repr::Ternary` (no fixed `N`) touches the rehash (DN-34 §8.16 explicitly defers it; RFC-0033
     defers CU-7 post-1.0).
  2. A **growable / beyond-`u32` Dense or VSA `dim`**, and any **quantized-Dense** growth — gated on
     the QuantDesc rehash (ADR-030; `issues.yaml:5735-5737`) and the VSA element-space/block-sparse
     widening (`issues.yaml:5767`).
  3. Any **growable-tier form whose payload shape enters the content address** for Binary/Bytes/Seq —
     to the extent a new growable payload changes identity, it lands in the **same single rehash**.
- **The gate.** These pieces are **held until E20-1 lands its content-address rehash (M-780)** — which
  the epic **defers to a post-1.0 wave** (`issues.yaml:7385-7394`). Attempting them earlier risks a
  *second* rehash (a one-way door E20-1 is explicitly built to make once). **The maintainer settles the
  E20-1 timing; this note does not guess it** (G2/VR-5). ⚑

## 6. Decidable-now vs. held

Separating the **incrementally-landable** pieces (no content-address change) from the **E20-1-coupled**
pieces (held per §5). *All items below are `Declared` proposals; none is scheduled by this note.*

### 6.1 Decidable now (no content-address change; can land incrementally)

- **The size-tier model plus honesty tags plus never-silent narrow as a documented framework** (this
  note, ratified) — a design artifact, changes no `Repr`.
- **Surfacing the `BigTernary` *arithmetic* as a computation** (not a persisted growable value form) —
  the digit-serial `add`/`mul` are already arbitrary-width and never-silent (§2.2); exposing them as
  checked ops that return a **fixed-width** result via a `checked_to_width`-style narrow **does not
  change content-address identity** (the persisted value stays a fixed `Ternary`). This lifts the
  ~40-trit *conversion* ceiling behind an explicit checked boundary — the clean incremental win.
- **A `Binary` wide-tier digit-serial `add`/`mul`/`div` beyond 64**, returning a fixed-width result via
  a `width_cast`-style checked narrow — a bounded computation, no new persisted `Repr`; extends the
  DN-41/DN-42 bridge.
- **Closing RFC-0036 §FLAG-C** (the `WidthRel` prim-signature gap) so `width_cast`'s per-instance
  output width is expressible — a prerequisite for any width-polymorphic size op, and independent of
  E20-1.
- **A uniform never-silent narrow-back API** across Seq/Bytes/Dense/VSA (checked "shrink to a fixed
  size, refuse on loss") — reuses ADR-025's existing above-the-kernel growth; adds the consistent
  boundary plus tag, no `Repr` change.

### 6.2 Held on the E20-1 settlement (do not implement until M-780 — §5)

- The **growable `Repr::Ternary`** value form (CU-7).
- **Growable / beyond-`u32` Dense/VSA `dim`**; quantized-Dense growth; VSA element-space/block-sparse
  widening.
- Any **growable-tier persisted payload** for Binary/Bytes/Seq that alters content-address identity.

## 7. User stories

- *As a **Mycelium numerics author**, I want* to compute with integers larger than 64 bits / 40 trits
  *without* silently losing precision, *so that* a big-integer or high-precision-ternary algorithm is
  expressible and its overflow is an explicit refusal I can handle, not a hidden wrap.
- *As a **kernel maintainer**, I want* one documented size-tier model with never-silent narrows and
  honest tags spanning all four representations, *so that* I add "grow beyond the machine cap" once,
  consistently, instead of re-deciding it ad-hoc per representation.
- *As a **value-model reviewer**, I want* the growable-value work explicitly partitioned into
  land-now vs. E20-1-gated, *so that* no one changes content-address identity ahead of the single
  rehash and triggers a costly second one-way door.
- *As the **maintainer**, I want* the provider/consumer service presented as one evaluated candidate
  against a type-level alternative — not silently adopted — *so that* I choose the architecture on
  merit with the KC-3 / value-semantics tradeoffs surfaced.

## 8. Definition of Done (this note's gate)

- [x] The **current state** of size handling across all four representations is surveyed **honestly,
      with `file:line` citations**, before any proposal (§2) — implemented/partial work (digit-serial
      ternary, `BigTernary`, ADR-025 growables) is documented as **existing**, not proposed for
      reimplementation.
- [x] A **coherent cross-representation strategy** (§3) is stated — one size-tier lattice over the
      shared "size-scalar-in-`Repr` plus `O(size)` payload" shape — with never-silent plus
      honesty-tagged invariants grounded in ADR-025/028 and DN-41.
- [x] The provider/consumer service is **evaluated, not decided** (§4), with pros/cons and **at least
      one contrasting alternative** (type-level tiers) plus a noted hybrid; the assessment is explicit
      that the maintainer chooses (VR-5).
- [x] The **E20-1 content-address coupling is FLAGGED as a design gate** (§5, FLAG-cu7-e20-1-gate),
      with the **blocked pieces identified** — not guessed (G2).
- [x] **Decidable-now vs. held** is separated (§6).
- [x] **User stories** (§7) plus this **Definition of Done** are present (house rule #6); the note is
      **`Declared`/Draft** throughout and cites its basis (house rule #4).
- [ ] **Maintainer reviews** the design space and either directs the decidable-now pieces (§6.1) onto a
      wave or requests revisions. This note **stays Draft** until then (house rule #3 — never
      self-advances). *(Open — the note is a backlog artifact.)*
- [ ] **FLAG handoffs to the orchestrator** (this note does not apply them): the Doc-Index row plus the
      CHANGELOG entry plus a minted tracking id are **orchestrator-owned** (close-out). *(Open.)*

> **Append-only (house rule #3).** This note **supersedes nothing** and moves no decision status. It
> **builds on** RFC-0033, ADR-025/026/028/029/030, DN-41/DN-42, and DN-34's CU-7 correction, and it
> **defers** every content-address-coupled piece to the E20-1 settlement (§5). CHANGELOG / Doc-Index /
> issues.yaml / docs/api-index are owned by the integrating parent — this leaf flags, it does not edit
> them.

---

## Meta — changelog

- **2026-07-08 — Created (Draft) — authored on the `trx2` BACKLOG lane (non-blocking, non-conflicting
  worktree; maintainer directive 2026-07-08).** Drafts a **unified, extensible strategy for
  arbitrarily-large values / byte-sizes across the whole value model**. Surveys the honest current
  state with `file:line` citations (§2): Binary caps at 64 for arithmetic (five `*_MAX_WIDTH = 64`
  consts, bitwise unbounded), ternary conversion caps near 40 trits (i64) **while** the digit-serial
  `add`/`mul` are already arbitrary-width and `BigTernary` (ADR-029) is a built-but-unsurfaced growable
  form, Seq/Bytes length plus Dense/VSA dim are each a `u32` `Repr` field with growth deferred above
  the kernel (ADR-025). Proposes **one size-tier lattice** (machine / wide / growable) over the shared
  size-scalar shape, never-silent and honesty-tagged, with tier changes as swaps (§3). **Evaluates
  (does not decide)** the maintainer's provider/consumer **service** candidate against a **type-level
  size-tier** alternative plus a hybrid (§4). **Flags the E20-1 content-address coupling as a hard
  design gate — FLAG-cu7-e20-1-gate** — and identifies the blocked pieces (growable `Repr::Ternary`,
  growable/beyond-`u32` Dense/VSA dim, quantized-Dense growth, any identity-changing growable payload)
  held until the single content-address rehash (M-780, deferred post-1.0) (§5). Separates
  **decidable-now** (surface `BigTernary` arithmetic behind a checked narrow, a `Binary` wide-tier,
  close RFC-0036 §FLAG-C, a uniform never-silent narrow API) from **held** pieces (§6). Includes user
  stories (§7) plus a Definition of Done (§8). **Status Draft; `Declared` throughout — decides nothing,
  never self-advances (house rule #3); every normative claim cites a ratified ADR/RFC/DN or is marked
  open (house rule #4).** CHANGELOG / Doc-Index / issues.yaml / docs/api-index owned by the integrating
  parent. (Append-only; VR-5; G2.)
