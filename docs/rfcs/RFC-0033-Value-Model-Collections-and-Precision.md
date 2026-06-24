# RFC-0033 — Value-Model Collections & Precision

| Field | Value |
|---|---|
| **RFC** | 0033 |
| **Status** | **Proposed** (2026-06-24) — the value-model collections (`Seq`/`Bytes`) and the four paradigms' precision/width semantics (Binary sign-free; Ternary arbitrary-width; Dense granularity-descriptor quant; VSA explicit element-space + block-sparsity + complex carrier), the never-silent boundaries between them, and the content-address identity set are stated normatively **as proposed**. The maintainer ratifies → **Accepted** (house rule #3 — never skipping steps). Per the swarm-integration rule, kernel pieces move to *"implemented (Rust-first), pending ratification"* as they land — **never silently to `Accepted`** (VR-5). |
| **Type** | Foundational / normative (once Accepted) — extends RFC-0001's value model with collections and honest precision/width semantics so the ABI need not break to introduce growable/quantized/complex/large-width values later |
| **Date** | 2026-06-24 |
| **Feeds** | E20-1 (Value-Model Collections & Precision) — M-754…M-784; the dogfooding gate (values may be persisted only once the content-address one-way doors land — §7) |
| **Decides** | `Repr::Seq`/`Repr::Bytes` shape and never-silent `get`/decode; Binary sign-free storage with signedness-as-operations; Ternary arbitrary-width arithmetic with a never-silent fixed-width boundary; Dense granularity-aware quant **descriptor in `Repr`** + scale/zero-point **arrays in `Payload`**; VSA explicit element space + block-sparsity + complex `Payload` carrier; the swap/guarantee/M-I reconciliation that follows; the content-address identity set |
| **Depends on** | RFC-0001 (the value model — `Repr`/`Value`/`Meta`/`Payload`, the guarantee lattice, content-addressing — r2 already universalized `bound.basis` per ADR-011); RFC-0002 (swap certificate & split regime — `LosslessWithinRange`/`Bounded`); RFC-0003 (VSA submodule boundary — models, sparsity); RFC-0032 (kernel self-hosting surface — **already decides `Repr::Seq`/`Repr::Bytes`**; §3 here aligns with and extends it, it does not re-decide it); ADR-010 (verified numerics); **ADR-011 (BoundBasis is universal — OQ-3 already closed; §6 extends it, does not reopen it)**; KC-2/KC-3 (small auditable kernel — every `Repr`/`Payload` addition justified); G2/VR-5 (never-silent, honest tags) |
| **Coupled with** | `crates/mycelium-core/src/repr.rs` (the `Repr` enum — Dense `dtype`/quant, VSA `elem`/sparsity, `Seq`/`Bytes`); `crates/mycelium-core/src/value.rs` (`Payload` — `HypervectorC`, quantized-Dense arm); `crates/mycelium-core/src/ternary/` (the arbitrary-width `BigTernary`, reconciled against the existing M-111 codec); `crates/mycelium-core/src/bound.rs` (the `Bound`/basis the dequant cert reads — per ADR-011); `crates/mycelium-core/src/content.rs` (content-address identity); `crates/mycelium-vsa/` (block-sparse lift); `docs/spec/swaps/*` (the swap certificates) |
| **Task** | E20-1 (epic) / M-785 (this RFC + ADR-025…031 — the E20-1 design gate; the V0 ternary impl is M-754…M-759) |

> **Posture (honesty rule / VR-5).** This RFC **decides** the value-model collections + precision
> surface **as proposed** (§3–§7): the `Repr`/`Payload` shapes, the never-silent boundaries, and the
> content-address identity set. It **decides the surface, it does not implement it** — the only kernel
> piece landing alongside this RFC is the arbitrary-width `BigTernary` (E20-1 V0, off the
> content-address critical path), reconciled into `core::ternary`. The two **content-address one-way
> doors** (Dense quant descriptor §4.3; VSA element-space §4.4) and the swap/guarantee reconciliation
> (§6) are **Proposed and unimplemented**; they land together in a single rehash (M-780) only after
> ratification, **before any value is persisted for dogfooding** (§7). Every `Repr`/`Payload` addition
> **enlarges the trusted base** (KC-3) and is justified per §2.3 (admissible only when the alternative
> would make the value model *unable to honestly represent* a standard datum — a correctness condition).
> Until each piece is implemented + differential-tested, claims about its behaviour are `Declared`
> positions checked by implementation (VR-5); never-silent (G2) is mandatory on every boundary.
>
> **Provenance.** Rationale, the per-decision verdicts (two of which deliberately **disagree** with the
> input draft on honesty grounds), and the rejected alternatives are recorded in
> `research/14-value-model-integration-report-RECORD.md` and
> `research/15-embeddonator-leverage-map-RECORD.md` (a recorded external research input; not normative).
> This RFC is the **normative** statement; the ADRs `ADR-025…031` are the terse decision records.
> Normative keywords (MUST / MUST NOT / SHOULD / MAY) are per RFC 2119.

---

## §1 Scope

Defines `Repr::Seq`, `Repr::Bytes`, the precision/width semantics of the four paradigms, and the
never-silent boundaries between them, such that the value-model ABI does not require a breaking change
to introduce growable collections or quantized/complex/large-width values later. It does **not** define
higher-level ADTs (`Vec<Struct>`, `Map`), surface syntax, or the codegen/lowering pipeline (touched
only where a decision constrains it).

**Relation to RFC-0032 (collections).** RFC-0032 §5 (D3/D4) already **decides** `Repr::Seq` (indexed
sequence, never-silent `get`) and `Repr::Bytes` (string/byte value, never-silent decode) and places
them **in `core` 1.0.0**. §3 of this RFC **restates those shapes normatively and extends them** with
the element-generality rule (§3.1.1), the `lift_option` adapter discipline (§3.1.2), and the
`Map`-exclusion (§3.1.4). Where this RFC and RFC-0032 both speak to `Seq`/`Bytes`, **RFC-0032 is the
governing placement decision**; this RFC adds the value-model invariants. Any conflict is an error to
be reconciled in this RFC's favour only by an explicit changelog entry here.

## §2 Invariants (normative)

- **§2.1** Every `Repr` MUST describe a **fixed-size** layout. No `Repr` variant MAY carry a
  variable-length field. (`Seq.len`/`Bytes.len` are `u32` *type* parameters, not variable-length data.)
- **§2.2** Every operation that can fail, approximate, or exceed range MUST surface that explicitly via
  `Option`/`Result` or a primitive status bit lifted above the kernel. Silent default, wrap,
  saturation, or truncation on the arithmetic path is FORBIDDEN (G2).
- **§2.3** Additions to the trusted kernel MUST be justified as minimal (KC-3). A new `Repr`
  field/variant or `Payload` arm is admissible only when (a) the alternative would make the value model
  *unable to honestly represent* a standard datum it claims to support (a correctness condition), or
  (b) it is a single fixed-size field with clear layout meaning.

## §3 Collections

### §3.1 `Seq` — length in the type
`Repr::Seq { elem: Box<Repr>, len: u32 }`. The length is part of the type (§2.1). (Shape governed by
RFC-0032 D3; this section adds the value-model invariants.)
- **§3.1.1** Element `Repr` MUST be a repr-value (Binary, Ternary, Dense, Vsa, Seq, Bytes). It MUST NOT
  be a heterogeneous/algebraic-data value; such containers are ADTs **above** the kernel.
- **§3.1.2** `get(seq, i)` is a kernel primitive returning `(elem: Repr, in_bounds: Binary{1})`. It
  MUST NOT return an algebraic `Option`. The `Option<Repr>` MUST be assembled by a single blessed
  adapter `lift_option` immediately above the kernel (one generated call site, mechanically enforced —
  a CI grep finds no other construction site). On `in_bounds = 0` the returned `elem` is a defined zero
  of the element repr and MUST be ignored. `get` carries guarantee tag **Exact**. (This mirrors the
  ratified `eq → Binary{1}` then Bool-lift pattern in RFC-0032 D1 — consistency in the most sensitive
  layer is the argument.)
- **§3.1.3** Growable sequences (`Vec`, `DynamicSeq`) are higher-level structures over a fixed-capacity
  `Seq` (capacity + length, chunked, or COW). They are NOT kernel reprs.
- **§3.1.4** `Map<K,V>` MUST NOT be a kernel repr (its hashing/ordering policy is not canonically
  content-addressable).

### §3.2 `Bytes` — dedicated, never-silent UTF-8
`Repr::Bytes { len: u32 }`, distinct from `Seq<Binary{8}>` for packing and a direct decode path.
(Shape governed by RFC-0032 D4.)
- **§3.2.1** UTF-8 decode is a kernel primitive returning `(str, valid: Binary{1})`; the
  `Result<String, Utf8Error>` MUST be lifted above the kernel. Invalid UTF-8 MUST NOT be silently
  replaced (no `U+FFFD` substitution).

## §4 Precision & width

### §4.1 Binary
- **§4.1.1** `Repr::Binary { width: u32 }`. The stored value is a **bitvector** with no signedness;
  signedness is a property of **operations**, not the `Repr`.
- **§4.1.2** Bitwise ops treat the value as an unsigned bitvector. Arithmetic ops are provided in
  two's-complement form; operations whose result differs by signedness (division, ordering, arithmetic
  vs logical shift, overflow detection) MUST be distinct named ops. Operations identical across
  signedness (`add`/`sub`/`mul`/`neg`) MAY be shared. (Mirrors SMT-LIB/Z3: there is no signed/unsigned
  distinction in the bitvector *value* — only in the *operations*.)
- **§4.1.3** Fixed-width arithmetic overflow MUST be never-silent (`Option`/`Result`). Bignum growth
  MUST NOT occur inside `Binary`; arbitrary-precision integers are a `BigInt` ADT above the kernel.
- **§4.1.4 (one-way door)** A nominal signed integer type, if introduced, is a higher typed-view over
  `Binary{width}` **by default**. Placing a `signed` flag in the `Repr` is permitted ONLY by an
  explicit superseding decision, because it changes the content-address identity of every integer value
  and doubles the swap matrix. (Default is **not** to add `signed` to `Repr`.)

### §4.2 Ternary
- **§4.2.1** `Repr::Ternary { trits: u32 }`, balanced ternary `{−1, 0, +1}`.
- **§4.2.2** Arithmetic MUST be arbitrary-width. The reference implementation is digit-serial
  (`BigTernary`); a limbed implementation (≥40 trits/u64) MAY be added and, if added, MUST be
  differentially proven bit-exact against the reference (YAGNI until a profile shows the digit-serial
  path is hot).
- **§4.2.3** The growable form MUST NOT overflow (carry-out becomes a new trit). For the fixed-width
  type `Ternary{trits:N}`, any operation whose true result exceeds `±(3^N − 1)/2` MUST return
  `None`/`Result`. No silent wrap or truncation.
- **§4.2.4** The canonical representation MUST be non-redundant (no carry-save / signed-digit
  redundancy), so content-addressing is well-defined.

> **Honesty note (§4.2).** Mycelium's existing `core::ternary` (M-111) is **already never-silent**
> about the fixed-width cap: `max_magnitude` returns `None` for `m ≥ 41` and `add`/`mul` return `None`
> on overflow. `BigTernary` therefore **removes the ~40-trit cap by adding a growable path** — it does
> *not* fix a silent bug in Mycelium's code. (The silent-overflow defect described in the research
> record is `embeddonator`'s `dimensional::Tryte::max_value`, a *different* upstream codebase, and is
> the reason that file is on the do-not-lift list.)

### §4.3 Dense (content-address one-way door)
- **§4.3.1** `Repr::Dense { dim: u32, dtype: Dtype, quant: Option<QuantDesc> }`.
- **§4.3.2** `Dtype` MUST include at minimum: `F16, BF16, F32, F64, TF32, I8, U8, I16, U16, I4, U4,
  F8E4M3, F8E5M2`. OCP-MX FP6/FP4 (`F6E3M2, F6E2M3, F4E2M1`) MAY be added. (Today's `ScalarKind` is the
  float-only subset; this widens it — an extensible registry per RFC-0001 §4.1.)
- **§4.3.3** When `dtype` is a quantized variant (any integer or fp8 type), `quant` MUST be `Some`
  (enforced at construction; an invalid `None` MUST be unconstructable).
- **§4.3.4** `QuantDesc { scheme: QuantScheme, symmetric: bool, scale_dtype: Dtype }` with
  `QuantScheme ∈ { PerTensor, PerChannel{axis:u32}, PerBlock{axis:u32, block:u32} }`. The descriptor is
  in the `Repr` and is part of content-address identity.
- **§4.3.5** Scale and zero-point **values** are variable-length (`O(dim)` per-channel, `O(dim/block)`
  per-block) and therefore MUST live in the `Payload`, never in the `Repr` (§2.1). A single scalar
  `scale`/`zero_point` in the `Repr` is FORBIDDEN — it cannot honestly represent per-channel/per-block
  formats (ONNX per-axis/blocked, GGUF K-quants, OCP MX, NF4), and using one to "represent" such a
  value is a never-silent violation at the `Repr` level.
- **§4.3.6** `zero_point` is present iff `!symmetric`.

### §4.4 VSA / HDC (content-address one-way door)
- **§4.4.1** `Repr::Vsa { model: String, dim: u32, elem: VsaElem, sparsity: VsaSparsity }`.
- **§4.4.2** `VsaElem ∈ { Binary, Bipolar, Integer, Real, Complex }`. The element space MUST be
  explicit so guarantee tags and swaps can reason about exactness.
- **§4.4.3** `VsaSparsity ∈ { Dense, SparseGlobal{max_active:u32}, SparseBlock{blocks:u32,
  active_per_block:u32} }`. `SparseBlock` with `active_per_block = 1` is SBC; `> 1` is BSDC. (Extends
  today's `SparsityClass`; RFC-0003 §5 governs the VSA model boundary.)
- **§4.4.4** A complex-valued model (e.g. FHRR, MAP-C) MUST use the `HypervectorC` Payload
  (`Vec<Complex<f64>>`). Encoding a complex HV as interleaved reals in `Hypervector(Vec<f64>)` with a
  doubled or mis-stated `dim` is FORBIDDEN (never-silent).
- **§4.4.5** `model` selects the binding/bundling algebra from the registry, layered on a carrier
  (§4.4.2–§4.4.4) capable of honestly storing it. A `model` string MUST NOT be used to imply a vector
  space the `Repr`/`Payload` cannot store.

## §5 Payload (normative additions)
`Payload` gains `HypervectorC(Vec<Complex<f64>>)` (the FHRR / MAP-C carrier) and a quantized-Dense arm
carrying packed codes together with the scale/zero-point arrays whose counts follow §4.3.4. The exact
byte layout of the quantized arm is specified in a follow-up (and SHOULD align with whichever external
formats Mycelium ingests — ONNX/GGUF/MX/NF4), but the scale/zero-point data MUST stay in `Payload`, not
`Repr` (§2.1). The `Payload` enum is part of the trusted base (KC-3); these arms are admitted under
§2.3(a) — the alternative is a value model that *cannot represent* families it claims to support.

## §6 Swaps & guarantees (reconciliation — normative)
- **§6.1** Binary↔Ternary is `LosslessWithinRange`: lossless for the growable path; range-bounded for
  fixed width. The certificate MUST distinguish these. (Base conversion has no bit shortcut — 2 and 3
  are coprime — so it lands in the swap machinery.)
- **§6.2** Dequantization (quantized Dense → float Dense) is `Bounded`; its error bound MUST be
  computed from `QuantScheme` (per-block tighter than per-tensor). **Per ADR-011, every `Bound` already
  carries a `basis` (it is universal — OQ-3 is closed).** This RFC does **not** reopen OQ-3; it
  **extends** the dequant `bound.basis` to additionally record the quantization **block structure**, so
  the basis honestly reflects how the bound was obtained (VR-5/G5). That extension is the only
  `bound.basis` change; it does not alter ADR-011's universal-basis decision.
- **§6.3** Dequant results MUST be tagged at most `Empirical` (never `Exact`/`Proven`), or carry a
  `Bounded` certificate. Complex→real-magnitude is `Bounded`; bipolar↔binary is `LosslessWithinRange`.
- **§6.4** The M-I bound series MUST gain a granularity parameter so invariant bounds are stated per
  `QuantScheme` (per-tensor vs per-channel vs per-block).

## §7 Content-address identity (normative)
The following are part of value identity and MUST be hashed into the content address (the
identity-bearing `Repr` + payload, per RFC-0001 §4.6 / `Node::content_hash`): `Seq.len`, `Bytes.len`,
`Dense.dtype`, `Dense.quant` (descriptor), `Vsa.elem`, `Vsa.sparsity`. Because changing any of these
**rehashes stored values**, §4.3.4, §4.4.2–§4.4.3 (and §4.1.4 if ever taken) MUST be settled and land
in a **single rehash** (E20-1 V5 / M-780) **before any value is persisted for dogfooding**. This is the
DOGFOOD GATE.

## §8 Conformance
An implementation conforms iff: (a) no `Repr` variant carries variable-length data; (b) every boundary
in §3–§4 is never-silent; (c) the ternary reference and any limbed form are differentially equivalent;
(d) quantized Dense cannot be constructed with a scalar scale or a missing descriptor; (e) complex VSA
cannot be constructed over a real payload. The conformance test matrix is in
`docs/planning/value-model-backlog.md` §6.

## §9 Open questions
- **OQ-3 — CLOSED (not by this RFC).** `bound.basis` universality is already decided by **ADR-011**
  (RFC-0001 → r2). §6.2 only *extends* the dequant basis with block structure; it does not reopen OQ-3.
- **OQ-4 — `NormKind` enumeration (declared-open).** Which norms parameterize Dense/quant error bounds
  (L2/L∞/relative) is **not** resolved here.
- **OQ-5 — policy predicate grammar (declared-open).** The selection/conversion policy predicate
  grammar for choosing quant schemes / VSA models is **not** resolved here.
- These are explicitly **declared-open** (G4), not silently resolved.

---

### Changelog

| Date | Status | Note |
|---|---|---|
| 2026-06-24 | **Proposed** | Initial proposal. Value-model collections (`Seq`/`Bytes`, aligned with RFC-0032 D3/D4) + the four paradigms' precision/width: Binary sign-free (§4.1, one-way door on `signed`-in-`Repr`); Ternary arbitrary-width with never-silent fixed-width boundary (§4.2 — removes the cap, core is already cap-honest); Dense granularity-descriptor quant in `Repr` + scale arrays in `Payload` (§4.3, content-address one-way door, **disagrees with the input draft's scalar quant**); VSA explicit element-space + block-sparsity + complex carrier (§4.4, content-address one-way door, **disagrees with the input draft's model-strings-suffice**). §6 reconciliation **extends** ADR-011's universal `bound.basis` (OQ-3 already closed — not reopened). §7 content-address identity set + dogfood gate (single rehash M-780). OQ-4/OQ-5 declared-open. Companion records: `research/14`, `research/15`; decision records `ADR-025…031`; plan `docs/planning/value-model-implementation-plan.md`. Task E20-1/M-785. |
