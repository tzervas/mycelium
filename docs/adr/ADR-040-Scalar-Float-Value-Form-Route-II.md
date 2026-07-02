# ADR-040 — Scalar-float value form (route ii): first-class `Repr::Float`, IEEE-754 binary64

| Field | Value |
|---|---|
| **ADR** | 040 |
| **Status** | **Proposed** (2026-07-02 — drafted per ADR-038 §2.6 / task M-895; **maintainer ratifies** — this ADR is not self-ratified). The *route* is already decided: ADR-038 §2.6 (Accepted 2026-07-01) fixed **route (ii) — a first-class scalar-float `Repr`** — and this ADR does **not** re-open it; it fixes the float *design* inside that route (width set, NaN/rounding semantics, never-silent boundaries, content-address coordination). Kernel entry is separately gated by the DN-39 default-DENY promotion bar; the four-clause dossier is **DN-69** (companion note, same date). |
| **Decides** | *(proposed)* A first-class scalar-float value form: `Repr::Float { width }` with the width set **binary64 (F64) only at introduction**, carried by an append-only, frozen-tag width registry (the `ScalarKind::tag()` discipline) so later widths extend without shifting any existing address. Arithmetic is IEEE-754 binary64, **round-to-nearest-even only**; rounding mode is a property of **operations**, never hidden state (the ADR-028 signedness-as-operations parallel). **NaN is canonicalized to the single positive quiet-NaN bit pattern at value construction** (NaN payload bits are not identity-bearing and not observable); `+0.0`/`-0.0` stay bit-distinct. Arithmetic specials (±inf, NaN) are **in-band, inspectable, propagating values** — the recommended overflow policy (§2.4, FLAG-2) — while every **conversion boundary** (float↔integer, parsing, any future width change) is never-silent: out-of-range/NaN → explicit `Option`/error. The content-address impact is **documented, not spent** (§3): identity commitments are settled here, and the identity-set change rides the **single E20-1 rehash**, coordinated with the deferred ADR-030/031 one-way doors and **deferred to the first value-persistence feature** (RFC-0033 §7 / ADR-038 §2.6). |
| **Grounds** | **ADR-038 §2.6** (route (ii) decided; the double gate — float ADR + DN-39 review); **RFC-0033 §7** (single-rehash dogfood gate), **§4.1.4** (the one-way-door pattern this ADR mirrors), **§4.3.2** (floats-as-Dense-dtypes = route (i), rejected by ADR-038), and its 2026-07-01 changelog pull-forward note; **ADR-028** (semantics carried by operations, not the `Repr` — the rounding-mode parallel); **ADR-030/ADR-031** (the deferred content-address one-way doors this ADR coordinates with; note ADR-031's M-775…M-780 → **E20-1** milestone erratum); **DN-39** (the default-DENY promotion bar) + **DN-69** (this candidate's four-clause dossier); `docs/spec/stdlib/self-hosting-readiness.md` §0 blocker-1 (the motivating gap, `Exact`); `docs/spec/stdlib/cmp.md` Q1 (float partial order / named total order); IEEE 754-2019 (binary64, RNE, totalOrder); `crates/mycelium-core/src/repr.rs` + `src/content.rs` (the current `Repr`/`Canon` facts, source-read). |
| **Date** | 2026-07-02 |

> **Posture (VR-5 / house rules #1, #3, #4).** **Proposed** — drafted by an agent for maintainer
> ratification; nothing here is self-ratified and no implementation lands with this ADR. The route-(ii)
> decision is ADR-038's and is treated as settled input, not re-argued. Every claim below carries its
> honest tag; no `Proven` is used without a checked side-condition (there is none here — this is a
> design document, so the strongest tags in play are `Exact` for gap statements verified against
> source/grammar, `Empirical` for source-read facts, and `Declared` for asserted platform properties).
> Open decision points are FLAGged in §7, not silently resolved.

## 1. Context — the gap, and the decided route

The float gap is the first blocker in the self-hosting readiness ledger
(`docs/spec/stdlib/self-hosting-readiness.md` §0, item 1, `Exact`):

> **No float value form/ops** (no float literal/type/prims; F16–F64 exist only as Dense dtypes) →
> blocks `math` (f64 half), `numerics`.

Floats exist today only as **Dense dtypes** (`Repr::Dense { dim, dtype: ScalarKind }`,
`crates/mycelium-core/src/repr.rs:93`; RFC-0033 §4.3.2) — tensor storage formats, not scalar values. A
Mycelium program cannot write `1.5`, hold a scalar float value, or call float arithmetic.

ADR-038 §2.6 (Accepted) already decided **how** this closes: **route (ii), a first-class scalar-float
`Repr`** — explicitly *not* a Dense-dtype workaround (route (i)) and *not* a library-only encoding —
double-gated by this ADR plus a DN-39 promotion review, with the content-address impact coordinated
with the deferred ADR-030/031 one-way doors so the identity-set change lands as **one rehash, once**,
deferred to the first value-persistence feature (RFC-0033 §7). This ADR supplies the gated design; the
DN-39 dossier is DN-69.

## 2. Decision (proposed)

### 2.1 Value form and width set

`Repr::Float { width: FloatWidth }` — a new scalar `Repr` variant in `mycelium-core`, sibling to
`Binary`/`Ternary`, with a single-scalar payload arm.

**Width set at introduction: `FloatWidth::F64` (IEEE-754 binary64) only.** Grounding: every scalar
float consumer named in the corpus needs f64 and only f64 — `math`'s f64 half
(`docs/spec/stdlib/math.md` §on ε), `numerics`' `DECLARED_FLOAT_EPS` (f64), `cmp`'s f64→i32 seam
(DN-16). No corpus consumer needs a *scalar* f16/bf16/f32; the sub-64-bit widths exist for **tensor
storage economy**, which is `Dense`'s concern (RFC-0033 §4.3.2), not a scalar concern. Per the DN-39
clause-(4) posture (small + auditable; kernel entry is earned **per width**), the width set is the
minimum the evidence supports. `Empirical` (corpus grep — every scalar-float citation found is f64;
FLAG-1 if the maintainer wants f32 from day one).

**Extensibility is append-only and address-stable.** `FloatWidth` follows the frozen-tag registry
discipline of `ScalarKind::tag()` (`repr.rs:50–61` — "existing codes are frozen so a definition's
identity never shifts when the registry grows"): adding a later width mints a new frozen tag and
leaves every existing f64 address unchanged. `Empirical` (source-read of the `Canon` prefix-tag
encoder; becomes a checked regression test at implementation — see §5).

Whether `FloatWidth` is a dedicated enum or a constrained reuse of `ScalarKind` is an implementation
choice for M-896 within this constraint (dedicated enum recommended — reusing `ScalarKind` would admit
scalar F16/BF16 by construction, widening the ratified surface silently; FLAG-6).

### 2.2 Rounding — a property of operations, never hidden state (the ADR-028 parallel)

All float arithmetic at introduction is IEEE-754 binary64 **round-to-nearest-even (RNE)**, the IEEE
default. There is **no dynamic rounding-mode register**: a mutable mode that changes what `flt.add`
means from a distance is a black box (house rule #2) and breaks per-op auditability. Exactly as
ADR-028 keeps signedness in **operations** rather than the `Repr`, any future non-RNE rounding is a
**distinct named op** (e.g. a `_rtz`-suffixed op), not a mode switch and not a `Repr` field. This also
keeps the content address of a float value independent of any ambient mode — one bit pattern, one
address, regardless of how it was computed. `Declared` (design assertion; the ADR-028 precedent is
`Exact` as a citation).

### 2.3 NaN, signed zero, and identity determinism

- **NaN is canonicalized to the single positive quiet NaN (bits `0x7FF8_0000_0000_0000`) at value
  construction.** NaN payload bits are **not identity-bearing and not observable**. Basis: NaN
  payload/sign bits produced by hardware arithmetic are platform-dependent (Rust/LLVM document
  non-deterministic NaN bit patterns — `Declared`, from the Rust reference; not independently checked
  here). The interpreter is the trusted base that roots the differential oracle (NFR-7, DN-39 §6), so
  value identity must be deterministic: without canonicalization, `0.0/0.0` could yield different
  content addresses on different hosts — a silent identity fork. Canonicalization is a reified,
  documented normalization at the value boundary, not a silent swap: no observable float operation
  distinguishes NaN payloads, so no information an operation could see is dropped (`Declared`; the
  no-payload-observing-op property becomes a checked invariant of the prim set at implementation).
- **`+0.0` and `-0.0` stay bit-distinct in identity.** They are observably distinct (`1.0/x` sign,
  `copysign`), so collapsing them would alias two distinguishable values — the silent-aliasing failure
  ADR-030 rejects for quant descriptors. Two addresses, honestly distinct. (FLAG-4 notes the
  equality-vs-identity seam this creates; `==` remains IEEE equality, under which `+0.0 == -0.0`.)
- **Existing seam, surfaced (G2):** today's `Canon::f64` (`content.rs:156–158`) hashes raw
  `to_bits()` with **no** NaN canonicalization, so Dense/Hypervector payloads containing NaN already
  have platform-bit-dependent identities (`Empirical`, source-read). This ADR's canonicalization rule
  should be settled **uniformly** (scalar float and the existing f64 payload paths) in the same E20-1
  identity settlement — FLAG-5.

### 2.4 Never-silent boundaries

The never-silent rule (house rule #2; RFC-0033 §4.1.3 for the integer analog) binds two different
kinds of boundary differently:

- **Conversion boundaries — explicit `Option`/error, no exceptions.** Float→integer where the value
  is NaN, ±inf, or out of the target range → explicit error/`None`, never truncation-by-default.
  Integer→float where the integer exceeds exact representability (|n| > 2^53 for binary64) → the
  inexactness is explicit: a checked-exact variant errors, and any rounding variant is a reified,
  `EXPLAIN`-able conversion carrying its bound — never a silent lossy cast. Malformed float literals →
  parse diagnostics (M-897). Any future width-to-width conversion is a swap with an explicit cert.
- **Arithmetic specials — in-band, inspectable, propagating values (recommended; FLAG-2).** IEEE
  overflow (→ ±inf), division by zero (→ ±inf/NaN), and invalid operations (→ NaN) produce
  **first-class, distinguished, sticky values**, not errors. The rationale, argued not assumed:
  integer overflow is never-silent because wraparound **aliases an ordinary in-range value** — the
  corruption is invisible. Float overflow lands on a **distinguished sentinel that propagates** and is
  directly inspectable (`is_finite`/`is_nan` classification prims ship with the op set) — the signal
  is in-band, which is the *mechanism* of never-silent, not an exception to it. Trapping on ±inf/NaN
  would instead make standard IEEE algorithms (which use inf as a legitimate value) inexpressible.
  This reading of "never-silent" for floats is a real semantic decision the maintainer must ratify
  explicitly, because the M-895 task wording ("out-of-range/overflow → explicit Option/error") admits
  a trap-on-overflow reading — both options are laid out in FLAG-2, with this one recommended.
- **Comparison.** Float comparison is **partial**: ordering against NaN yields an explicit
  no-order result (`Option`-shaped), never a silent `false`-as-less-than. A **named, opt-in total
  order** (IEEE-754 `totalOrder`) ships as a distinct op for sorting/keying — imposing it silently is
  exactly what `docs/spec/stdlib/cmp.md` Q1 rejects. Coordinates with M-899/M-511 (the total-order
  proof debt stays `Empirical` until proven).

### 2.5 Kernel op surface at introduction — deliberately minimal

The kernel prim set this ADR ratifies: arithmetic `add`/`sub`/`mul`/`div`/`neg` (RNE), partial
comparison plus the named total order (§2.4), classification (`is_nan`/`is_finite` at minimum), and
the never-silent conversions of §2.4. Exact prim names follow the existing registry conventions at
implementation time (M-898/M-899). **Transcendentals and every libm-dependent function (`sqrt`, `exp`,
`log`, trig) stay OUT of the kernel prim set** — they are `std.math`'s surface (M-525), they carry the
M-541 audit debt, and excluding them is what keeps the DN-39 clause-(4) "small + auditable" case
honest (DN-69 §3.4). Surfacing them later is append-only registry growth under their own honest tags,
not an amendment to this ADR.

### 2.6 Guarantee-tag posture for the prims

Per-op tags at introduction (VR-5 — none may be upgraded without a checked basis):

- `flt.add`/`sub`/`mul`/`div`/`neg`: the semantic *definition* is "the correctly-rounded IEEE-754
  binary64 result under RNE" (`Exact` as a definition — it is the spec). The *implementation claim*
  that the host's f64 arithmetic delivers that result is **`Empirical`** at introduction
  (property/differential tests against IEEE reference cases), with the underlying "Rust f64 is IEEE
  754 binary64" platform statement held at **`Declared`** (asserted by the Rust reference; not
  independently verified here). No `Proven` is claimed anywhere in this ADR.
- Comparison/total order: partial-order behavior `Empirical` (property-tested, NaN cases in
  conformance); the `totalOrder` total-order *property* stays `Empirical` until a proof lands (M-511).
- Conversions: range/exactness checks `Empirical` via property tests on the documented bounds
  (2^53, target-range edges).

## 3. Content-address impact — documented, not spent (the rehash-coordination section)

A new `Repr` variant **joins content-address identity**: implementation adds a frozen `REPR_FLOAT` tag
arm to `Canon::repr` (`content.rs:227`) and a scalar payload arm to `Canon::payload`, and a float
value's address becomes `blake3(canon(Float{width}) ‖ canon(payload-bits, NaN-canonical))`. This ADR
fixes the identity commitments now so they are **settled once**:

- **Identity-bearing:** the `Float` variant tag, the frozen width tag, the payload bit pattern
  (with NaN canonicalized per §2.3; `-0.0`/`+0.0` distinct).
- **Not identity-bearing:** NaN payload/sign bits (canonicalized away); all dynamic `Meta`
  (unchanged, RFC-0001 §4.6).

**No rehash is spent by this ADR, and none is spent by landing the variant.** Because the `Canon`
encoder is prefix-tagged and existing tags are frozen, *adding* the `Float` arm changes no existing
value's address (`Empirical`, source-read; pinned by an address-stability regression test in M-896 —
its DoD already requires verifying **no rehash occurred**). The coordination obligation (RFC-0033 §7 /
ADR-038 §2.6) is therefore about **settlement, not immediate mechanics**: this ADR's identity
commitments (§2.3, the width registry, and the FLAG-5 uniform-NaN question) must be settled alongside
the deferred ADR-030 (`Dense.quant`) and ADR-031 (`Vsa.elem`/`Vsa.sparsity`) one-way doors so that the
**single E20-1 rehash** — which those doors *do* require, since they reshape existing identity-bearing
fields — happens **once**, with the float commitments already final, **before any value is persisted
for dogfooding**. The rehash itself **defers to the first value-persistence feature**: until some
feature actually persists values, no rehash is spent (ADR-038 §2.6, honored unchanged). Note
ADR-031's milestone erratum: the rehash work is owned by epic **E20-1**; "M-775…M-780" IDs are
unminted — this ADR cites E20-1.

This is the same one-way-door pattern RFC-0033 §4.1.4 prescribes for integer signedness: identity
choices are made by explicit decision, never by drift, and revisiting any of them after values persist
requires a superseding decision plus a coordinated rehash.

## 4. User stories

- As a **stdlib author**, I want scalar floats to be first-class values with literals and arithmetic,
  so that `math`'s f64 half and `numerics` become expressible in `.myc` instead of blocked
  (readiness §0 blocker-1).
- As a **language user**, I want float overflow, invalid operations, and NaN comparisons to be
  distinguished, inspectable states — never a silently wrong ordinary number — so that numeric bugs
  surface where they happen.
- As a **certified-mode user**, I want every float op's accuracy claim tagged at its honest strength
  (`Empirical` until audited/proven), so that the results I audit trace to a basis, not an assertion.
- As the **maintainer**, I want the float value form's identity commitments settled in one reviewed
  decision, coordinated with the other one-way doors, so that the content-address rehash happens once
  and never by accident.
- As a **kernel auditor**, I want the float kernel entry to be the minimum earned surface (one width,
  a small prim set, no libm), so that KC-3 stays true after the variant lands.

## 5. Consequences

- Unblocks M-896…M-900 (Repr + value form, literal, arithmetic prims, comparison prims, three-way
  conformance closure) once this ADR is Accepted **and** the DN-39 review (DN-69) passes — the
  double gate of ADR-038 §2.6.
- The kernel/TCB grows by one value form (the DN-39-audited surface: one `Repr` variant, one payload
  arm, two `Canon` arms, the §2.5 prim set). DN-69 argues this is net-trust-reducing versus the
  workarounds; the KC-3 delta is reviewed at M-896 per its DoD.
- The swap matrix grows: float↔binary and any future float↔dense-scalar conversions are explicit,
  certed swaps (never-silent). Dense's float story is unchanged — `Dense` keeps its dtypes; scalar
  float is a separate form, and no implicit scalar↔rank-0-tensor identification is introduced.
- Implementation must add the address-stability regression test (§3) and the NaN-canonicalization
  property test (no constructor path yields a non-canonical NaN bit pattern).
- `docs/spec/stdlib/cmp.md` Q1 gets its answer shape (partial default, named opt-in total order);
  M-511's proof debt becomes load-bearing and stays honestly `Empirical` until discharged.

## 6. Rejected

- **Route (i) — scalar floats as `Dense{dim:1}`** and **route (iii)-style library encodings over
  `Bytes`/`Binary` bit-casts**: rejected by ADR-038 §2.6 (Accepted); recorded here only for
  completeness — not re-adjudicated. (DN-69 §3.3 details why both fail the never-silent and
  no-black-box rules.)
- **Wide width set at introduction ({F16, BF16, F32, F64}).** No grounded scalar consumer below f64;
  each width multiplies the swap matrix and the audited kernel surface (DN-39 clause 4). Widths are
  added append-only when a consumer exists.
- **A dynamic rounding-mode register.** Hidden state that changes op meaning at a distance — a black
  box (house rule #2); contradicts the ADR-028 semantics-in-operations precedent.
- **Raw-bits NaN identity (no canonicalization).** Imports platform nondeterminism into content
  addresses — the trusted base's identity would fork across hosts (§2.3).
- **Collapsing `-0.0` into `+0.0` in identity.** Aliases observably distinct values — the ADR-030
  silent-aliasing failure mode.
- **Trap-on-overflow arithmetic as the default.** Presented as the FLAG-2 alternative rather than
  silently discarded; rejected in the recommendation because IEEE specials are in-band never-silent
  signals and trapping makes standard float algorithms inexpressible (§2.4).

## 7. Open decisions — FLAGs for the maintainer

| # | FLAG | Recommendation (tagged) |
|---|---|---|
| FLAG-1 | **Width set**: F64-only at introduction, or {F32, F64}? | F64-only (`Empirical` — no corpus consumer needs scalar f32; append-only extension stays cheap). |
| FLAG-2 | **Arithmetic overflow policy**: IEEE in-band specials (±inf/NaN as values) vs trap-on-overflow (`Option`/error from `flt.add` etc.). The M-895 task wording admits either reading. | IEEE in-band specials, with classification prims and never-silent conversion boundaries (§2.4 rationale; `Declared` — a design argument, not a checked fact). |
| FLAG-3 | **Literal semantics**: decimal→binary64 conversion is inherently inexact (`0.1` has no exact binary64). Accept documented correctly-rounded (RNE) literal conversion, or require exactness annotations? | Documented correctly-rounded conversion, stated in the literal's spec and `EXPLAIN`-able — the alternative (erroring on inexact literals) rejects nearly all real decimal literals. |
| FLAG-4 | **Identity vs equality seam**: `+0.0`/`-0.0` are two addresses but IEEE-equal; canonical NaN is one address but IEEE-unequal to itself. Confirm this documented divergence between content identity and `==`. | Accept and document (identity is bit-level with canonical NaN; `==` is IEEE). Any keyed/dedup use routes through the named total order (FLAG via cmp Q1). |
| FLAG-5 | **Uniform NaN canonicalization**: extend §2.3's rule to the existing `Canon::f64` payload paths (Dense/Hypervector), which today hash raw platform bits — settle in the same E20-1 identity settlement? | Yes — one NaN-identity rule for the whole value model, settled once (`Declared`; touching those paths is itself identity-affecting for NaN-bearing tensors, hence E20-1-coordinated). |
| FLAG-6 | **`FloatWidth` carrier**: dedicated enum vs constrained reuse of `ScalarKind`. | Dedicated enum (reuse admits scalar F16/BF16 by construction — silent surface widening). Implementation-level; M-896 decides within the §2.1 constraint. |
| FLAG-7 | **DN-39 dossier recording format**: DN-39 prescribes no format for later review instances; a new dated note (DN-69) cross-linking DN-39 was chosen. Confirm. | Keep DN-69 as the instance record; DN-39 itself stays append-only untouched. |

## 8. Definition of Done (the ratification gate)

This ADR is **Accepted** when the maintainer ratifies: (a) the width set (FLAG-1), (b) the NaN/zero
identity rules (§2.3, FLAG-4/5), (c) the overflow policy (FLAG-2), (d) the never-silent conversion
boundaries (§2.4), (e) the minimal kernel op surface (§2.5), and (f) the §3 rehash-coordination
posture (documented-not-spent; single E20-1 rehash; deferred to first value-persistence) — together
with a **passed DN-39 promotion review** (DN-69 ratified PROMOTE). Acceptance enacts no code; it
opens the M-896…M-900 gate (ADR-038 §2.6). It is **Enacted** only when the implementation lands per
the M-896…M-900 DoDs (including the no-rehash regression evidence) — stepping through Accepted first,
never skipping (house rule #3). MIT-licensed like every first-party artifact (house rule #6); the
IEEE-754 citations are references, not incorporated text.

---

### Changelog

| Date | Status | Note |
|---|---|---|
| 2026-07-02 | **Proposed** | Initial draft (task M-895, kickoff `enb` Gap A). Route (ii) per ADR-038 §2.6 (not re-opened). Proposes: `Repr::Float{width}`, F64-only frozen-tag width registry; RNE-only, rounding-as-operations (ADR-028 parallel); canonical-NaN identity, bit-distinct signed zeros; in-band IEEE specials + never-silent conversion boundaries; minimal kernel op surface (no libm); content-address impact documented-not-spent, coordinated with ADR-030/031 in the single E20-1 rehash deferred to first value-persistence (RFC-0033 §7). Seven maintainer FLAGs (§7). Companion DN-39 dossier: DN-69. Maintainer ratifies; not self-ratified. |
