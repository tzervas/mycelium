# Value-Model Integration Report & Design

> **Mycelium recording note (2026-06-24).** This is a **recorded research input** — an
> external Grok + Claude value-model research bundle (2026-06-23), preserved here as the
> evidence base for **RFC-0033** and **ADR-025…031** (all landing **Proposed**). It is not
> itself normative. Four corpus corrections apply when reading it:
> 1. **Paths re-mapped to the real tree.** There is no `crates/mycelium-value`; the trusted
>    arbitrary-width ternary lands in **`crates/mycelium-core/src/ternary/`** (reconciled
>    against the existing `Trit` + `core::ternary` M-111 codec — no duplicate `Trit`, and
>    `Limb27`/`PackedTernary` are an explicit YAGNI follow-on). Tasks `VM-NNN` map to
>    canonical **M-754…M-784** under epic **E20-1** (see `docs/planning/value-model-*`).
> 2. **The "silent precision ceiling" is `embeddonator`'s, not Mycelium's.** Mycelium's
>    `core::ternary` is **already never-silent** about the 40-trit cap (`max_magnitude`
>    returns `None` at m ≥ 41; `add`/`mul` return `None` on overflow). The new `BigTernary`
>    **removes the cap by adding a growable path** — it does not fix a silent bug in
>    Mycelium's own code.
> 3. **The RFC-0001 §4.3 `bound.basis` amendment reconciles with the existing, ratified
>    `ADR-011 — BoundBasis-Is-Universal`** — it is not asserted fresh. (That reconciliation
>    is value-model phase V5 / M-781, not landed here.)
> 4. **Collections (`Seq`/`Bytes`) overlap `RFC-0032` / `E19-1`**, which already scopes
>    `Repr::Seq`/`Repr::Bytes`; RFC-0033 references and aligns with it rather than
>    re-deciding it.

**Project:** Mycelium · **Layer:** value model (trusted kernel, KC-3)
**Date:** 2026-06-23
**Status:** Design-complete for the decisions below; implementation pending
**Inputs:** Grok's `mycelium-value-model-design-decisions.md` (draft recs);
the value-model design review (verdicts); a source audit of the `embeddonator`
crates (lift-and-shift candidates).
**Audience:** Claude Code (implementation), the PM/Evaluator personas, future readers.

> Placement note: Mycelium is private, so paths here mirror **conventional** layout
> (`research/value-model/`, `docs/value-model/rfcs|adrs/`, `crates/mycelium-value/`).
> Re-file to match the real tree; the relative structure is what matters.

---

## 1. Purpose & scope

This package turns a set of value-model design questions into **normative decisions**,
a **corrected `Repr`/`Payload`**, an explicit **blast-radius reconciliation plan**, and
**ready-to-lift Rust** for the balanced-ternary substrate. It exists because these are
ABI- and content-address-affecting choices that gate near-term dogfooding: getting them
wrong means re-hashing or migrating every stored value later. The governing constraints
throughout are the two non-negotiables — **never-silent** (every conversion/approx/
out-of-range is an explicit `Option`/`Result`) and **minimal trusted kernel** (every
addition to KC-3 justified and minimal) — plus the four-point guarantee lattice
**Exact ⊐ Proven ⊐ Empirical ⊐ Declared** and the two swap-certificate kinds
(`LosslessWithinRange`, `Bounded`).

In scope: collections (`Seq`/`Bytes`), the four paradigms' precision/width audit
(Binary, Ternary, Dense, VSA), and the embeddonator lift. Out of scope: higher-level
ADTs (`Vec<Struct>`, `Map`), surface syntax, the codegen/lowering pipeline (touched only
where a decision constrains it).

---

## 2. Decisions at a glance

| # | Decision | Verdict vs Grok | One-way door? | Corpus blast radius |
|---|---|---|---|---|
| A1 | `Seq`/`Bytes` length **in the type** | **Agree** | **Yes** | none (growables are above-kernel) |
| A2 | Elements are **repr-values only** | **Agree** | **Yes** | none |
| A3 | `get` returns `(Repr, in_bounds bit)`; `Option` lifted above kernel | **Agree** (+ blessed adapter) | no | none |
| B-Bin | Binary: width-general; **signedness as operations, not a `Repr` field** | **Amend** (Grok put `signed:bool` in Repr) | the *signed-in-Repr* choice is | content-address identity, swap matrix (if signed-in-Repr taken) |
| B-Ter | Ternary: **arbitrary-width limbed now**, never-silent fixed-width overflow | **Agree** (high priority) | the *capped* choice is | swap certs (`LosslessWithinRange` semantics) |
| B-Den | Dense: **granularity-aware quant descriptor in Repr, scale/zero-point arrays in Payload** | **Disagree** (Grok's scalar is dishonest) | **Yes** | swap certs (`Bounded`), **RFC-0001 §4.3 `bound.basis`**, guarantee lattice, M-I bounds, content-address |
| B-VSA | VSA: **element space in Repr + block-sparse + complex payload carrier** | **Disagree** (Grok's `Vec<f64>` blocks ≥3 families) | **Yes** | Payload enum (kernel), content-address, swap certs |

The two **disagreements** (Dense, VSA) are the load-bearing findings: both are cases
where the draft is *under-general to the point of silently misrepresenting standard
data*, which is a never-silent failure at the representation level — the worst place
for one. Both must be reconciled **now** because they change content-address identity.

---

## 3. Per-decision rationale

### A1 — Length in the type. **AGREE.** Genuine one-way door
**Why.** Mycelium's lowering, codegen, allocation, and content-addressing all rest on
"every `Repr` is fixed-size." `Seq{elem, len:u32}` preserves that exactly as
`Binary{width}` does, and it is the dominant low-level convention: LLVM array types
carry length in the type (`[N x T]`); you cannot construct a dynamically-sized LLVM
array type — the documented pattern is pointer + element count. Rust `[T; N]` is a
distinct type per `N`; `Vec<T>` is `(ptr, capacity, length)` built *above* it. Sized-
type languages (Idris/Agda `Vect n`, ATS) and MLIR's static-vs-dynamic dimension split
do the same. Growable `Vec` becomes a higher structure over a fixed-capacity `Seq`
(capacity + length, chunked/rope, or COW). **Cost to KC-3:** one `u32`, already
justified by `width`/`dim`.
**Rejected:** *length-in-payload* (`Seq{elem}`). Gives a kernel-level growable but
breaks the fixed-size-`Repr` invariant every consumer leans on, importing heap/realloc
concerns into the trusted base for a capability (growth) that does not belong there.
Reversing the invariant later is corpus-wide; choosing length-in-type is the
reversible-upward move.

### A2 — Repr-values-only elements. **AGREE.**
**Why.** Homogeneous repr-value sequences keep the kernel closed over itself: a `Seq`
of repr-values is still a fixed-size `Repr`, swaps compose element-wise, content-
addressing stays structural. The kernel makes "collections" real without learning
user-defined structure.
**Rejected:** *general algebraic-data elements* (`Vec<MyStruct>`, `Map<K,V>` in the
kernel). Forces `Repr` to describe a structured container over the whole value model
and to reference a **data/type registry** recursively — the single largest trusted-
surface explosion available, and it breaks the clean "swaps are between paradigms"
model (no certified swap for an arbitrary user struct). Heterogeneous containers are
recursive ADTs **above** the kernel, using `Seq` for storage. `Map` in particular must
never be a kernel repr (its hashing/ordering policy is not canonically content-
addressable).

### A3 — `get → Option`. **AGREE**, with one amendment
**Why.** A kernel primitive returns a *representation value*; `Option`/`Result` are
*algebraic data* that live above the kernel. Returning `(elem_repr, in_bounds:Binary{1})`
and lifting the `Option` in one line is exactly the ratified `eq → Binary{1}` then
Bool-ADT pattern — consistency in the most sensitive layer is itself the argument. It
also mirrors how minimal/verified systems expose status: CPU overflow/carry flags,
SMT-LIB overflow *predicates* (`bvuaddo`/`bvsaddo` return a Bool beside the wrapping
result), WASM checked-arithmetic proposals, seL4/CompCert TCB-minimization.
**Amendment (non-trusted).** The lift must be a **single, generated, blessed adapter**
(`lift_option(repr, bit) -> Option<Repr>`), reused at every call site, so never-silent
is mechanically enforced rather than re-implemented per call. On `in_bounds = 0` the
returned repr is a defined-but-ignored zero; `get`'s guarantee tag is **Exact**.
**Rejected:** (1) `Option` as a kernel primitive return — drags a generic tagged sum
into KC-3, the very thing A2 excludes, and is circular (need ADTs to express a kernel
op's result). (2) sentinel/poison value — silent in-band signal, violates never-silent.
(3) trap/panic on OOB — destroys totality; `get` must be total with explicit status.

### B-Binary — Width-general; signedness as **operations**, not a `Repr` field. **AMEND.**
**Agree with Grok on:** width-general `Binary{width:u32}`, separate bitwise (always
unsigned bitvec) vs arithmetic ops, never-silent overflow, no bignum growth in the
kernel (`BigInt` is an ADT above).
**Amend:** Grok puts `signed:bool` in the `Repr`. The cleaner, smaller-surface design
is **sign-agnostic storage + signedness-carrying operations**. Two's-complement
`add/sub/mul/neg` are bit-identical for signed/unsigned; only division, comparison,
right-shift, and overflow detection differ — which is exactly why SMT-LIB shares
`bvadd/bvsub/bvmul` and splits only `bvsdiv/bvudiv`, `bvslt/bvult`, `bvashr/bvlshr`, and
the overflow predicates, and why the Z3 guide states there is *no* signed/unsigned
distinction in the bit-vector *value* — only in the *operations*. So: `Binary` **is**
the bitvector; "signed integer" is an *interpretation* imposed by the op set (or a
higher typed view), not a property of the stored value.
**Why it matters / blast radius:** if `signed` lives in the `Repr`, the *same bit
pattern* with `signed=true` vs `false` becomes two distinct content-addressed values
and a pure reinterpretation requires a swap — fragmenting the address space. Keeping
`Binary{width}` sign-free avoids that. *If* the team wants a nominal signed type for
surface ergonomics, that is defensible — but then it is a deliberate content-address
decision to take once, now, not by default.
**Rejected:** two's-complement wrap on overflow (never-silent violation); bignum-
growable `Binary` in the kernel (imports allocation, same reason as A1-option-2); a
separate `Int{width,signed}` repr (redundant; doubles the swap matrix).
**Width:** keep `u32` (≈4.29e9 bits); arithmetic past a fast-path threshold (e.g. 128
bits) routes to a limbed slow path with never-silent fixed-width overflow.

### B-Ternary — Arbitrary-width limbed now. **AGREE. High priority.**
**Why.** The current ~40-trit cap (i64 internal) is an **accidental silent precision
ceiling**, exactly what never-silent forbids — and the audit found it is worse than a
cap: `embeddonator`'s `dimensional::Tryte::max_value(n) = (3i64.pow(n)−1)/2` silently
overflows i64 for `n ≥ 41`. The packing math is favorable: 40 balanced trits fit a
signed i64 (`3^40 ≈ 1.216e19`, balanced range `±6.07e18 < i64 9.22e18`;
`40·log2 3 = 63.398` bits), and 5 trits pack into a byte (`3^5=243<256`, 1.6 bits/trit,
99.06% efficient). Balanced-ternary carries are `{−1,0,+1}` and most addition-table
entries don't carry — carry chains are not materially worse than binary.
**Design.** Ship a **digit-serial `BigTernary`** (obviously correct, never-overflowing
oracle) first; add a **limbed `PackedTernary` (40 trits/u64)** as the perf path,
**differentially tested** against the oracle (mirror embeddonator's
`bt_phase1_packed_equivalence`). The never-silent boundary is the *fixed-width* type:
any op exceeding `±(3^N−1)/2` returns `None`/`Result`; the growable path never
overflows. See `crates/mycelium-core/src/ternary/` — the trit/limb primitives are
lifted from embeddonator and the arbitrary-width arithmetic is implemented and
algorithm-validated (10k fuzz vs arbitrary-precision int).
**Rejected:** keep-the-cap-and-document (silent ceiling, NEVER-SILENT violation);
digit-serial *as the production path* (one-trit-per-element wastes memory and forfeits
word-parallelism — keep it only as the equivalence oracle); redundant signed-digit /
carry-save (O(1) carry but multiple representations of one number — destroys canonical
content-addressing; balanced ternary's **non-redundancy** must be preserved).
**Blast radius:** binary↔ternary base conversion (2,3 coprime — no bit shortcut) lands
in the **swap machinery**; arbitrary width changes what "within range" means for the
`LosslessWithinRange` certificate (always-lossless for growable, range-bounded for
fixed). Reconcile the certificate semantics now.

### B-Dense — Granularity-aware quant **descriptor in Repr**, scale arrays in **Payload**. **DISAGREE with Grok.**
**Agree on the dtype expansion:** add `I8/U8, I16/U16, I4/U4, F8E4M3/F8E5M2` (+`TF32`
as a compute/storage marker). These are table-stakes across ONNX, OCP FP8/MX, and the
GGUF/bitsandbytes ecosystems. (FP6 `E3M2/E2M3` and FP4 `E2M1` are emerging via OCP MX —
register, lower priority.)
**The disagreement.** Grok's `quant_meta: Option<{scale:f64, zero_point:i64}>` — a
*single scalar pair per value* — is honest **only** for per-tensor INT8/UINT8. Every
mainstream sub-8-bit / modern embedding-quant format is **per-channel or per-block**,
and a scalar literally cannot represent them, so using one to "represent" such a value
is **silently wrong** — a never-silent violation at the `Repr` level:
- **ONNX** `QuantizeLinear` defines per-tensor (scalar), **per-axis** (1-D scale tensor,
  length = channel count), and **blocked** (scale shaped like input with the blocked
  dim divided by block size).
- **GGUF K-quants** (Q4_K): 256-weight super-block, 32-weight sub-blocks, FP16
  super-scale + FP16 super-min **plus** eight 6-bit sub-scales/sub-mins (144 B/256
  values, 4.5 bpw). No single scale.
- **OCP MX** (MXFP8/6/4, MXINT8): one shared **E8M0** 8-bit scale per **32-element
  block**. Per-tensor is documented as *insufficient* sub-8-bit.
- **NF4 / QLoRA**: block-wise absmax, **blocksize 64**, scales themselves double-
  quantized (`c2` blocksize 256).
**Corrected design (concrete in §4).** A granularity descriptor —
`QuantScheme::{PerTensor, PerChannel{axis}, PerBlock{axis, block}}` + `symmetric` +
`scale_dtype` — goes **in the `Repr`** (small, fixed-size, determines layout, and
*must* be part of content-address identity so per-tensor and per-block values can't
collide). The scale/zero-point **arrays** are `O(dim/block)`, so they are **Payload**,
not `Repr` (variable-length scale vectors in `Repr` would re-break the A1 invariant).
Grok's "metadata in Repr" instinct is right for the *descriptor*, wrong for the *data*;
the scalar is wrong on both counts.
**Rejected:** scalar-per-tensor-only (dishonest/under-general); all-meta-in-Payload
(per-tensor and per-block then share an address despite being non-interchangeable —
silent aliasing); defer quant entirely (the content-address door makes deferral *more*
expensive than doing the descriptor now). If staging: ship the **descriptor in Repr**
now and implement `PerTensor` + `PerBlock` payload paths first.
**Blast radius (sharp):** dequant is a `Bounded` swap whose error bound **depends on
granularity** → touches the `Bounded` certificate AND **RFC-0001 §4.3 `bound.basis`**
(the basis now includes block structure); dequant tags below `Exact` (Empirical /
carry a `Bounded` cert) → guarantee lattice + **M-I bound series gain a granularity
parameter**; and the descriptor is part of **content-address identity of every
quantized value** — ship it before anyone stores Dense values.

### B-VSA — Element space in `Repr` + block-sparse + complex carrier. **DISAGREE with Grok.**
**Why Grok's "model strings are enough" is wrong.** `model:String` is the right
extensibility mechanism for binding/bundling *operations*, but a model string **cannot
retrofit a vector space the Payload/Repr cannot hold**. `Hypervector(Vec<f64>)` +
`Sparse{max_active}` structurally **blocks at least three recognized families**:
- **Complex (FHRR, MAP-C, GHRR/resonator phasors).** FHRR HVs are complex unit-modulus
  phasors; binding is component-wise complex multiply, unbinding the complex conjugate.
  `Vec<f64>` can't hold them honestly: interleaving re/im makes `Repr.dim` lie (2D for a
  D-dim HV) and breaks "elementwise"; storing only phases means binding's modular angle
  arithmetic is not the declared op. Either way the kernel silently misrepresents the
  algebra. FHRR is a *core* model, not fringe.
- **Block-sparse (BSDC-S/SEG, SBC).** Partition D into L blocks, ~one active per block.
  `Sparse{max_active}` is a *global* count cap and cannot express block structure (which
  blocks, one-hot-per-block), so it cannot distinguish a valid SBC vector from an
  invalid one, and block-local binding is inexpressible. **embeddonator already
  implements this** (`block_sparse.rs`: `Block{pos:u64, neg:u64}` over `BLOCK_SIZE=64`,
  never-silent `try_new`/`validate`, bind/bundle) — proof the family is real and the
  current sparsity enum is insufficient.
- **Integer/graded (MAP-I).** Storable in `f64` but the element type can't be
  *declared* integer, so tags/swaps can't reason about exactness.
**Corrected design (concrete in §4).** Add `VsaElem ∈ {Binary,Bipolar,Integer,Real,
Complex}` and a richer `VsaSparsity ∈ {Dense, SparseGlobal{max_active},
SparseBlock{blocks, active_per_block}}` to `Repr::Vsa`, and add a
`HypervectorC(Vec<Complex<f64>>)` Payload arm. The registry then chooses the algebra
*on top of* a carrier that can honestly store it. (`SparseBlock{…, active_per_block}`
covers both strict SBC, `=1`, and BSDC, `>1` — the latter is what `block_sparse.rs`
implements today.)
**Rejected:** Grok's "current design mostly sufficient + model strings" (strings can't
store complex or block structure — families stay unrepresentable); interleaved
`Vec<f64>` complex with a lying `dim` (never-silent violation); drop complex/block
families from scope (FHRR and SBC are not fringe; the "complete coverage" goal fails).
**Blast radius:** the **Payload enum is trusted** — adding `HypervectorC` and the new
`Repr::Vsa` fields enlarges KC-3 (justified: the alternative is a value model that
*cannot represent* families it claims to support — a correctness failure, not a feature
gap); changes **content-address identity** of every VSA value (do it now); needs swap
certificates for complex↔real (`Bounded`) and bipolar↔binary (`LosslessWithinRange`).

---

## 4. The corrected value model (concrete deltas)

> Illustrative MSRV-1.92 Rust. `Repr` stays flat and fixed-size; variable-length data
> stays in `Payload`. New/changed parts marked.

```rust
pub enum Repr {
    // ── Binary: width-general, SIGN-FREE (signedness is an op/typed-view concern) ──
    Binary { width: u32 },

    // ── Ternary: width-general; arithmetic is arbitrary-width (see ternary/) ──
    Ternary { trits: u32 },

    // ── Dense: dtype expanded; quant is a GRANULARITY DESCRIPTOR (data lives in Payload) ──
    Dense {
        dim: u32,
        dtype: Dtype,
        quant: Option<QuantDesc>,          // CHANGED: descriptor, not a scalar pair
    },

    // ── VSA: element space + richer sparsity made explicit ──
    Vsa {
        model: String,                     // registry key for the algebra (unchanged role)
        dim: u32,
        elem: VsaElem,                     // NEW: the vector space
        sparsity: VsaSparsity,             // CHANGED: adds SparseBlock
    },

    // ── Collections (ratified shape) ──
    Seq   { elem: Box<Repr>, len: u32 },   // length IN THE TYPE
    Bytes { len: u32 },                    // dedicated; never-silent UTF-8 decode lifted above
}

pub enum Dtype {
    F16, BF16, F32, F64, TF32,             // floats (+ tf32 marker)
    I8, U8, I16, U16, I4, U4,              // integer quant
    F8E4M3, F8E5M2,                        // fp8 (table-stakes)
    // FOLLOW-ON (OCP MX): F6E3M2, F6E2M3, F4E2M1
}

// Descriptor in Repr (small, fixed-size, part of content-address identity).
pub struct QuantDesc {
    pub scheme: QuantScheme,
    pub symmetric: bool,                   // zero_point present iff !symmetric
    pub scale_dtype: Dtype,                // E8M0 (MX), F16 (GGUF), F32 (NF4), …
}
pub enum QuantScheme {
    PerTensor,                             // scalar scale  (honest for INT8/UINT8)
    PerChannel { axis: u32 },              // 1-D scale, length = channel count
    PerBlock   { axis: u32, block: u32 },  // MX(32), NF4(64), GGUF super-block(256)
}

pub enum VsaElem { Binary, Bipolar, Integer, Real, Complex }
pub enum VsaSparsity {
    Dense,
    SparseGlobal { max_active: u32 },
    SparseBlock  { blocks: u32, active_per_block: u32 }, // =1 SBC, >1 BSDC
}

pub enum Payload {
    Bits(Vec<bool>),
    Trits(Vec<Trit>),
    Scalars(Vec<f64>),
    Hypervector(Vec<f64>),
    HypervectorC(Vec<Complex<f64>>),       // NEW: FHRR / MAP-C carrier
    // Quantized Dense payload = packed codes + the scale/zero-point ARRAYS whose
    // count is a function of (dim, scheme). Exact encoding TBD in the RFC; the key
    // point is the arrays are Payload, not Repr.
    QuantDense { codes: Vec<u8>, scales: Vec<f64>, zero_points: Vec<i64> },
}
```

---

## 5. Blast-radius reconciliation plan

These are the cross-corpus touches the decisions force. The project prefers doing mass
reconciliation **early**; each is a task in the implementation plan.

1. **`LosslessWithinRange` certificate (binary↔ternary).** Redefine to distinguish the
   growable-lossless case from the fixed-width range-bounded case once arbitrary-width
   ternary lands. *Touches:* swap-certificate definitions.
2. **`Bounded` certificate (dequant).** Bound computation must read `QuantScheme` (per-
   block is tighter than per-tensor). *Touches:* swap-certificate machinery, **RFC-0001
   §4.3 `bound.basis`** — basis now includes block structure.
3. **Guarantee lattice.** Pin dequant ≤ `Exact` (Empirical, or carry a `Bounded` cert).
   Complex→real-magnitude is `Bounded`; bipolar↔binary is `LosslessWithinRange`.
4. **M-I bound series.** Add a *granularity parameter* so invariant bounds differ for
   per-tensor vs per-channel vs per-block.
5. **Content-address identity.** Three identity-affecting changes land together so the
   address space is hashed once: Dense `QuantDesc` in `Repr`; VSA `elem` + `SparseBlock`
   in `Repr`; (if taken) Binary `signed` in `Repr`. **Do before any values are stored.**
6. **OQ-3 interaction.** The shared-`Bound`-supertype resolution (basis as a common
   required field, with the one-line RFC-0001 §4.3 amendment) is consistent with item 2;
   land them in the same amendment so `bound.basis` is defined once for both the existing
   ambiguity and the new block-structure dimension.

---

## 6. Embeddonator leverage (summary)

Detail in `research/15-embeddonator-leverage-map-RECORD.md`. Bottom line: the **Ternary primitive layer is
liftable near-verbatim and is exhaustively tested upstream**; the **VSA block-sparse and
SIMD dense vectors are liftable for the VSA paradigm** (and `block_sparse.rs` is the
reference impl validating the B-VSA verdict); the **arbitrary-width ternary integer
arithmetic did not exist upstream** and is provided here (`big_ternary.rs`), algorithm-
validated. Keep `unsafe` SIMD out of KC-3.

---

## 7. Risks & caveats

- **Not compiled in-environment.** No Rust toolchain was available; the ternary
  algorithms were validated by a 1:1 Python port + 10k fuzz vs arbitrary-precision int.
  DoD requires `cargo +1.92 test`/`clippy`/`fmt` green.
- **Private repo, conventional paths.** Re-file to the real tree.
- **Provisional task IDs.** Tasks use a local `VM-NNN` prefix; assign canonical `M-NNN`
  on import to avoid colliding with the existing 25-task Phase 0/1 set.
- **External standards move.** GHRR (2024) and newer VSA algebras keep appearing; the
  design makes the *element space* honest (binary/bipolar/integer/real/complex + block)
  which covers recognized families — exotic future algebras still register as model
  strings over those carriers.
- **Dense payload encoding** for the scale/zero-point arrays is specified at the
  *location* level (Payload, not Repr) here; the exact byte layout is left to the RFC and
  should align with whichever external formats Mycelium ingests (ONNX/GGUF/MX/NF4).
