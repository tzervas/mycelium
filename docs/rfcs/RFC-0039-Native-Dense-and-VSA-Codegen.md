# RFC-0039 — Native Dense & VSA Codegen

| Field | Value |
|---|---|
| **RFC** | 0039 |
| **Status** | **Accepted** (2026-06-30 — maintainer-ratified; the four §8 open questions resolved 2026-06-30) · Proposed (2026-06-30) |
| **Type** | Foundational / normative — the native-codegen design for `Repr::Dense` and `Repr::Vsa`, and the dynamic-VSA JIT |
| **Date** | 2026-06-30 |
| **Feeds** | E25-1 (native AOT full-language coverage) — issues **M-853** (Dense), **M-854** (VSA), **M-855** (dynamic-VSA JIT + ADR-009 lift) |
| **Decides** | How `Repr::Dense` element-wise ops + the ADR-030 quant descriptor lower natively (with the quant-granularity / accumulator-width / packing-schedule recorded as inspectable `Meta.physical`, DN-01); how `Repr::Vsa` bind/bundle/permute lower over the BSC/HRR/FHRR/MAP(-I/-B)/SBC models (honoring ADR-031's element space / sparsity / complex carrier); how the dynamic-VSA JIT runs (data-dependent dimension, runtime model selection, cleanup/resonator loops) as an explicit never-silently-selected `ExecMode`; and the honesty/verification contract (every native Dense/VSA op differential-checked through the M-210 checker against the `mycelium-dense`/`mycelium-vsa` reference, mutant-witnessed, honest per-op tags). |
| **Grounds** | RFC-0029 §3 (which **excludes** native Dense/VSA codegen — this RFC fills that gap, it does not contradict it); RFC-0001 §4.1/§4.3 (the `Repr` = Binary/Ternary/Dense/Vsa value model + `Meta.physical`/`Meta.bound`); RFC-0003 §4.1/§5/§6 (the VSA per-op guarantee matrix erratum + the cited-theorem-plus-checked-instantiation capacity strategy + the resonator manifest); RFC-0004 §5 (schedule-staged packing as `Meta.physical`), §6 (no-opaque-lowering — every stage dumpable), §11 (the additive direct-LLVM increment pattern); RFC-0002 (the swap/cert discipline — honest `{ε, δ, strength}`); RFC-0009 (resonator factorization — Empirical-only, never-silent `NoConverge`); ADR-030 (Dense quant granularity descriptor in `Repr`, scales in `Payload`); ADR-031 (VSA element space / block-sparsity / complex carrier in `Repr`); ADR-009 (hybrid execution — the dynamic-VSA JIT deferral this RFC's §6 lifts, append-only); ADR-034 §6 (re-gating that names this RFC the vehicle); ADR-006 (EXPLAIN obligation); G2/VR-5 (never-silent, honest tags); KC-3 (small auditable kernel — the native path stays outside the trusted base). |
| **Coupled with** | `crates/mycelium-mlir/` (the AOT/JIT backend where `Repr::Dense`/`Repr::Vsa` are currently REFUSED — `src/llvm.rs:2170`, `src/dialect/native.rs:349-350`; the `ExecMode` enum at `src/mode.rs:55-72`); `crates/mycelium-dense/` + `crates/mycelium-vsa/` (the trusted-base reference semantics this native path is differential-checked against); `crates/mycelium-cert/` (the M-210 observational-equivalence checker — `src/check.rs`) |
| **Task** | E25-1 (epic) / M-853 (first child) |

> **Posture (transparency rule / VR-5).** This RFC **decides design only**. Acceptance ratifies the
> *design* — it asserts **no implementation** and moves **nothing else** to Accepted/Enacted: not RFC-0029,
> not ADR-009, and not this RFC itself past Accepted (RFC-0039 → Enacted only when the path is complete +
> stable, house rule #3). M-853/M-854/M-855 stay design-gated on this RFC (now Accepted) **before** they
> implement (§7). The native Dense/VSA path is **not built**: `Repr::Dense` and `Repr::Vsa` are *today* an
> explicit, never-silent refusal in every native backend (`AotError::UnsupportedRepr` in `llvm.rs`;
> `DialectError::Unsupported("…Dense/VSA stay on the interpreter / direct-LLVM path")` in
> `dialect/native.rs`), routed to the interpreter — which **remains the trusted base and the
> reference** (ADR-007/NFR-7). This RFC specifies how that refusal is *replaced* by lowering whose
> correctness is measured against `mycelium-dense`/`mycelium-vsa` through the M-210 checker. Every
> claim here about "what the path will do" is `Declared` (a design intent) until a checked three-way
> differential fires, at which point the *implementation* claim becomes `Empirical` per increment
> (never `Proven` absent a checked equivalence proof). Where the reference itself carries a `Proven`
> tag (the single-op MAP-I `bundle` capacity bound; the Dense Higham rounding bound), the native path
> **preserves it only where the checked side-condition still holds** and **drops to Empirical/Declared
> where codegen introduces any new approximation** (VR-5 — never upgrade past the basis). Grounded in
> `crates/mycelium-mlir/`, `crates/mycelium-dense/`, `crates/mycelium-vsa/`, `crates/mycelium-cert/`
> (checked 2026-06-30).

---

## 1. Problem / Goal

RFC-0029 sanctioned the AOT optimization, codegen-maturity, and JIT design for the **bit/trit** value
model — and **explicitly excluded** Dense and VSA from native codegen. Its §3 "Out of scope" and §5
Q6 (the honest codegen map) both record the boundary: the MLIR-dialect path "**Explicitly refused**
(never silent) … and routed to direct-LLVM/interp: … Dense/VSA". That refusal is live in the code
today (checked 2026-06-30):

- **Direct-LLVM** (`crates/mycelium-mlir/src/llvm.rs`): `const_lane` (~line 2170) returns
  `AotError::UnsupportedRepr` for any repr other than `Binary{w}`/`Ternary{m}`; the module header
  lists "`Swap`, Dense/VSA representations" among the deliberate explicit refusals.
- **MLIR-dialect** (`crates/mycelium-mlir/src/dialect/native.rs`): `const_lane` (~lines 349-350)
  returns `DialectError::Unsupported("repr {…} is not in the element-wise dialect fragment (Dense/VSA
  stay on the interpreter / direct-LLVM path)")`.
- **JIT** (`crates/mycelium-mlir/src/jit.rs`): compiles via the same `lower_program`, so it inherits
  the same Dense/VSA refusal.

ADR-034 (Accepted 2026-06-30) **re-gated** native AOT into the `lang 1.0.0` Definition of Done with
scope expanded to **full-language native-codegen coverage** — which includes Dense, VSA, and JIT for
dynamic VSA/HDC. ADR-034 §6 names two scope pieces that "need their **own normative design before
implementation**": native Dense + VSA codegen (out of RFC-0029's scope), and the ADR-009 JIT-deferral
lift for dynamic VSA. **This RFC is that design vehicle** (the maintainer's choice, 2026-06-30:
RFC-0039 rather than an RFC-0029 amendment, with the ADR-009 lift recorded **here** in §6 — and, per
the OQ-1 resolution, the §6 cross-reference **is** the full record: **no separate ADR-009 amendment**).

The goal: lift the Dense/VSA refusal into native lowering whose output is **observably equivalent**
to the `mycelium-dense`/`mycelium-vsa` reference, never-silent at every residual boundary, EXPLAIN-able
at every stage, and honestly tagged. **Performance is the goal of the native path; correctness is
always measured against the interpreter + the reference crates (NFR-7).**

## 2. User stories

- As a **language user**, I want a `Repr::Dense` embedding program compiled natively to produce the
  same values (and the same per-element rounding bound) as `mycelium-dense`, so that I can trust
  native Dense output without re-auditing under a second semantics.
- As a **VSA/HDC author**, I want `bind`/`bundle`/`permute` over MAP/BSC/HRR/FHRR/SBC to compile
  natively while **keeping each model's honest per-op tag** (RFC-0003 §4.1 — `permute` Exact,
  HRR/FHRR `unbind` Empirical, `bundle` Proven only where the side-condition holds), so that native
  speed never silently upgrades a guarantee.
- As a **downstream app developer**, I want JIT compilation for *dynamic* VSA workloads
  (data-dependent hypervector dimension, runtime model selection, cleanup/resonator loops), so that
  exploratory/streaming HDC pipelines get native performance without an offline AOT step — and I want
  that JIT **never silently selected** (only by naming the mode).
- As a **maintainer**, I want every native Dense/VSA op carried by a checked three-way differential
  (interp ≡ direct-LLVM ≡ MLIR-dialect-where-it-runs, plus interp ≡ JIT for the dynamic path) through
  the shared M-210 checker, mutant-witnessed, so that native correctness is *by construction*, never
  asserted.
- As an **auditor**, I want the Dense quant granularity / accumulator width / packing schedule, and
  the VSA model / element-space / sparsity, recorded as inspectable `Meta.physical` and dumpable at
  every lowering stage (RFC-0004 §6), so that no Dense/VSA lowering is a black box (ADR-006).

## 3. Scope and decision space

### In scope

- **Native Dense lowering** of the `mycelium-dense` element-wise surface (`add`/`sub`/`neg`/`scale`;
  `dot`/`similarity` as bare measurements) plus the ADR-030 quant model, with the granularity
  descriptor / accumulator width / packing schedule recorded as `Meta.physical` (§5.1).
- **Native VSA lowering** of `bind`/`unbind`/`bundle`/`permute` over the **1.0.0-mandatory standard
  models** — **MAP-I, BSC, HRR, FHRR** (the literature/`torchhd` standards; OQ-3 resolved) — honoring
  ADR-031's element space / sparsity / complex carrier, and **preserving the RFC-0003 §4.1 per-op
  guarantee matrix** (§5.2). The niche models (**SBC, MAP-B**) extend post-mandate, each still
  differential-checked + honest-tagged when added.
- **The committed widening to the full quantized-Dense + element-space-VSA coverage** (§5.1/§5.2,
  OQ-4 resolved — "both"): native codegen covers the **un-quantized F32/BF16 Dense + real-`Vec<f64>`
  VSA fragment now** (M-853/M-854), **and** this RFC **commits** to widening to the quantized Dense
  (ADR-030) and element-space/block-sparse/complex VSA (ADR-031) variants — **gated only on E20-1
  landing those `Repr` fields** (native codegen cannot precede the repr existing). The unbuilt variants
  stay an explicit never-silent refusal until E20-1 lands them; **E20-1 is the enabling dependency for
  the full-coverage half** (§7).
- **The dynamic-VSA JIT** (§5.3): data-dependent dimension, runtime model selection, cleanup/resonator
  loops, as an explicit `ExecMode` (no silent/`Auto` selection), and the **ADR-009 deferral lift** for
  this path recorded here (append-only cross-reference, §6).
- **The honesty/verification contract** (§5.4): the differential-equivalence bar, the M-210 checker
  registration, the mutant-witness, and the never-silent refusal at every uncovered fragment.

### Out of scope

- The interpreter and the reference crates (`mycelium-core`, `mycelium-dense`, `mycelium-vsa`) — the
  trusted base; this RFC concerns the native path *above* them, validated against them.
- The **bit/trit** native path, the optimization passes (inlining/CSE/DCE), and the BitNet
  packed-ternary acceleration surface — those are RFC-0029's scope (already Accepted); this RFC does
  not re-decide them.
- The `Swap`-node native lowering (M-852) and the closures/recursion increments (M-850/M-851) — sibling
  E25-1 increments with their own design grounding (RFC-0004 §11, RFC-0002); this RFC's only contact
  with `Swap` is that a Dense↔VSA or Dense-dtype native swap reuses the **same** M-210 cert discipline
  (§5.4), it does not re-specify the swap node.
- *Implementing* ADR-030's `QuantDesc` / ADR-031's `VsaElem`/`VsaSparsity`/`HypervectorC` in the value
  model — those `Repr` fields are the **E20-1 content-address rehash's** work (the reprs do not yet
  carry them; see the honesty note in §5.1/§5.2). This RFC **commits native codegen to widen to** the
  quantized/element-space coverage **as E20-1 lands those fields** (the in-scope OQ-4 commitment above —
  it is *sequenced behind E20-1*, not deferred indefinitely), and until then refuses (never-silent) any
  quant/element-space mode the reference cannot yet represent. *Building the reprs themselves* stays
  E20-1's job, not this RFC's.
- Hardware-specific Dense intrinsics beyond the schedule-staged packing set (general GPU/tile/MMA) —
  deferred beyond 1.0.0 (YAGNI), consistent with RFC-0029's BitNet boundary.

## 4. Definition of Done

- [ ] §5.1 (native Dense lowering) specifies: the covered `mycelium-dense` op surface; how the ADR-030
  quant descriptor / accumulator width / packing schedule are recorded as inspectable `Meta.physical`;
  and the never-silent refusal for every unsupported dtype/quant mode (`mycelium-dense` is the
  differential reference).
- [ ] §5.2 (native VSA lowering) specifies: the covered models + ops; how ADR-031's element space /
  sparsity / complex carrier are honored; and the **preservation rule** for the RFC-0003 §4.1 per-op
  guarantee tags (Proven kept only where the checked side-condition holds; Empirical/Declared where
  codegen adds approximation).
- [ ] §5.3 (dynamic-VSA JIT) decides: the data-dependent-dimension / runtime-model-selection /
  cleanup-resonator-loop execution model, as an explicit never-silently-selected `ExecMode`, with the
  correctness bar `JIT ≡ interpreter` (Empirical) over the dynamic subset.
- [ ] §5.4 (honesty/verification contract) specifies: the three-way (and interp≡JIT) differential
  through the M-210 checker; the mutant-witness; and that every uncovered fragment stays an explicit
  refusal routed to the interpreter (G2).
- [x] §6 records the ADR-009 dynamic-VSA JIT deferral lift (append-only cross-reference); OQ-1 is
  resolved — the §6 record **is** the vehicle, **no** separate ADR-009 amendment (maintainer 2026-06-30).
- [x] All §8 open questions are **resolved** (maintainer 2026-06-30 — OQ-1…OQ-4; see §8).
- [x] This RFC reaches **Accepted** (maintainer ratification, 2026-06-30) — the gate every
  M-853/M-854/M-855 implementation task waits on; each stays design-gated on it (§7) **before** it
  implements.

## 5. Normative decisions

### 5.1 Native Dense lowering (M-853)

**Reference.** `crates/mycelium-dense::DenseSpace` is the differential reference. Its element-wise
surface (checked 2026-06-30): `add_values`/`sub_values` (**Proven** — per-element relative ε from
Higham 2002 Thm 2.2, `F32_OP_REL_EPS = 2⁻²⁴`, `BF16_OP_REL_EPS = 2⁻⁸ + 2⁻²³`), `neg_value` (**Exact**
— symmetric grids), `scale_value` (**Proven**), and `dot`/`similarity` (bare `f64` measurements
carrying no `Meta` tag). The reference supports `ScalarKind::{F32, Bf16}` and **refuses F16/F64**
explicitly (`DenseError::UnsupportedDtype`); it checks per-element side-conditions (finite, exactly
on-grid, normal-not-subnormal, no rounding overflow) and refuses, never silently coerces, on any
violation.

**Decision.** Native Dense lowering (a new `crates/mycelium-mlir/src/dense_codegen.rs`,
**direct-LLVM first; the MLIR-dialect path later** — mirroring the RFC-0004 §11 additive-increment
pattern) **removes the Dense refusal** (`llvm.rs:2170`; `native.rs:349-350`) for the covered fragment
and lowers each `mycelium-dense` element-wise op to explicit per-element IR (one op per element —
dumpable, no opaque pass; RFC-0004 §6). The lowering is **never-silent**:

- **Covered ops:** `add`/`sub`/`neg`/`scale` and the `dot`/`similarity` reductions. The native result
  must be **observably equal** to `DenseSpace`'s — same `(repr, payload, guarantee)` — and where the
  reference carries a Proven Higham bound, the native op **carries the same bound only if the same
  per-element side-conditions are checked at codegen/runtime**; otherwise it drops to Declared (VR-5).
- **Quant as `Meta.physical` (ADR-030 + DN-01).** The Dense **quant granularity descriptor**
  (`QuantDesc{scheme: PerTensor|PerChannel{axis}|PerBlock{axis,block}, symmetric, scale_dtype}` — in
  `Repr` per ADR-030), the **accumulator width** chosen for a reduction, and the **packing schedule**
  (the layout chosen at the lowering stage, RFC-0004 §5) are recorded as inspectable `Meta.physical`
  on the lowered artifact — the same schedule-as-metadata discipline ternary packing already uses, not
  a typed property and not a hidden choice. A wrong `Meta.physical`/schedule tag must be caught by the
  §5.4 reference-equivalence check (the RFC-0004 §8 E3 obligation, extended to Dense).
- **Dequant** (the ADR-030 `Bounded` swap whose error bound depends on granularity) is **not** a new
  codegen op here; it lowers through the **same** M-210 cert discipline as any other swap (§5.4), with
  its `bound.basis` extending ADR-011 with block structure exactly as the reference does. The native
  path **must not** upgrade a per-block dequant bound to a per-tensor one (silent aliasing — the very
  failure ADR-030 rejects).
- **Never-silent refusals (G2).** Any dtype the reference refuses (F16/F64 today), any quant scheme
  the reference cannot yet represent, and any side-condition violation are **explicit**
  `AotError`/`DialectError` refusals routed to the interpreter — never a silent coercion or a
  wrong-precision lowering. The native path **does not** ship a second, divergent Dense semantics
  (DRY; the interpreter + `mycelium-dense` are the single meaning).

**Coverage sequencing (OQ-2 + OQ-4, resolved 2026-06-30 — "both").** Native Dense codegen ships the
**un-quantized F32/BF16 element-wise fragment first** (M-853), **and** this RFC **commits to widening**
to the full ADR-030 quant/accumulator/packing set (the int/fp8/TF32 dtypes; `PerTensor` + `PerBlock`
schemes first per ADR-030's own staging) **as E20-1 lands the `QuantDesc` descriptor** — the widening
is *sequenced behind E20-1*, not deferred indefinitely. Until each quantized dtype/scheme is present
and differential-checked, it is an explicit never-silent refusal.

> **Honesty note (VR-5).** ADR-030's `QuantDesc` is **Accepted but not yet implemented** in the value
> model (it lands in the E20-1 content-address rehash, pre-persistence). `mycelium-dense` today stores
> `Payload::Scalars` with no `quant` field. So the §5.1 quant-as-`Meta.physical` decision is the
> **design contract native codegen honors once the descriptor exists** (the committed widening above);
> until then native Dense covers only the un-quantized F32/BF16 element-wise fragment, and **explicitly
> refuses** any quantized Dense value (never silently treating it as un-quantized). Guarantee:
> `Empirical` per increment once the differential is checked; `Declared` until then.

### 5.2 Native VSA lowering (M-854)

**Reference.** `crates/mycelium-vsa` implements six models — **MAP-I** (`mapi.rs`), **MAP-B**
(`mapb.rs`), **BSC** (`bsc.rs`), **HRR** (`hrr.rs`), **FHRR** (`fhrr.rs`), **SBC** (`sbc.rs`) — behind
the `VsaModel` trait (`bind`/`unbind`/`bundle`/`permute`/`unpermute`/`similarity`). The honest per-op
guarantee matrix is **RFC-0003 §4.1 (the r3 erratum)**, which `mycelium-vsa/tests/matrix.rs` encodes
authoritatively:

- `permute`/`unpermute`: **Exact** for every model (a fixed coordinate bijection).
- `bind`/`unbind`: **Exact** for MAP-I/MAP-B (elementwise product, self-inverse) and BSC (XOR,
  self-inverse); HRR/FHRR `bind` **Exact** (deterministic convolution / phase-add) but **`unbind`
  Empirical** (the approximate, non-self-inverse inverse — the residual weak link, RR-13); SBC algebra
  part **Proven** (per-block index add/subtract).
- `bundle`: **Proven** for MAP-I (the capacity theorem, below) and SBC (Bloom analysis), **on-
  expectation** for BSC, **Empirical** for MAP-B (depth-1 only — RR-13 forbids deep nesting under
  Proven) and HRR/FHRR.

**Mandatory model set (OQ-3, resolved 2026-06-30).** The **commonly-used standard models are
1.0.0-native-mandatory — MAP-I, BSC, HRR, FHRR** (the standards in the VSA/HDC literature and in
`torchhd` use). The **niche/less-common models extend post-mandate — SBC, MAP-B** (and any others):
they are interpreter-served (never-silent refusal in the native path) until each is native-lowered,
and each still lands with a checked differential + honest tag when added. This RFC therefore scopes
native VSA 1.0.0 to {MAP-I, BSC, HRR, FHRR} and treats {SBC, MAP-B} as a committed later extension.

**Coverage sequencing (OQ-4, resolved 2026-06-30 — "both").** The mandatory models' **real-`Vec<f64>`
fragment ships now** (M-854), **and** this RFC **commits to widening** to the ADR-031 element-space /
block-sparse / complex-carrier variants **as E20-1 lands the `VsaElem`/`VsaSparsity`/`HypervectorC`
`Repr` fields** — sequenced behind E20-1, not deferred indefinitely. Until each carrier is present and
differential-checked, the block-sparse/complex variant is an explicit never-silent refusal. (FHRR's
*complex* carrier in particular widens with E20-1; its real-phase encoding is covered in the interim,
see the honesty note.)

**Decision.** Native VSA lowering (a new `crates/mycelium-mlir/src/vsa_codegen.rs`, **direct-LLVM
first; dialect later**) **removes the VSA refusal** for the covered fragment and lowers `bind`/`bundle`/
`permute` over each mandatory model's carrier as explicit, dumpable IR. The binding rules:

- **Honor ADR-031's element space + sparsity + complex carrier.** The `model` selects the algebra over
  a carrier that can honestly store it (`VsaElem{Binary,Bipolar,Integer,Real,Complex}`,
  `VsaSparsity{Dense,SparseGlobal{max_active},SparseBlock{blocks,active_per_block}}`, and the
  `HypervectorC` complex `Payload` arm for FHRR/MAP-C). The lowering selects the per-element machine
  type from the element space (e.g. a complex carrier for FHRR — **not** a lying real `dim`); the
  chosen carrier + sparsity layout is recorded as inspectable `Meta.physical`.
- **Guarantee-preservation rule (VR-5 — the load-bearing decision).** The native op **carries the
  reference's RFC-0003 §4.1 tag *if and only if* it introduces no new approximation**:
  - `permute` and the algebraically-exact `bind`/`unbind` (MAP/BSC) and SBC index algebra stay
    **Exact** — they are integer/bit/index permutations and elementwise products with no rounding.
  - The MAP-I `bundle` **Proven** capacity tag is carried **only** by replaying the reference's
    **checked instantiation**: `capacity::proven_capacity_bound(items, dim, δ)` issues
    `BoundBasis::ProvenThm` *iff* `dim ≥ requiredDim(items, δ)`, else `None` (honest downgrade). Native
    `bundle` **must run the same side-condition check** and stamp Proven only when it passes — never on
    a different (e.g. SIMD-reordered) accumulation that the checked instantiation did not cover.
  - HRR/FHRR `unbind` and `bundle` stay **Empirical**, bounded by the reference's trial-validated
    `EmpiricalProfile`s (the documented coverage windows — odd `m ≤ 5`, `d ≥ 1024`, single-factor,
    codebook ≤ 16, etc.). A native lowering whose floating-point reduction order differs from the
    reference's **must not** claim a tighter bound; if it falls outside the profile's coverage window,
    it is an explicit refusal, not a silent Empirical-anyway.
  - Any operation where native codegen changes the numeric result (e.g. a different complex-FFT
    convolution path for HRR `bind` with a different rounding error than the reference's direct
    convolution) **drops to Declared** until a fresh differential establishes Empirical — never
    inheriting the reference tag by assumption.
- **Never-silent refusals (G2).** Models/ops/element-spaces the reference does not implement, and any
  capacity side-condition that fails, are explicit `AotError`/`DialectError` refusals routed to the
  interpreter.

> **Honesty note (VR-5) — two corrections the implementation must respect.** (1) ADR-031's
> `VsaElem`/`VsaSparsity`/`HypervectorC` are **Accepted but not yet implemented** — `mycelium-vsa`
> today stores `Vec<f64>` and FHRR encodes *phase angles* in `f64` (not a `Complex` carrier). So the
> element-space/complex-carrier decision above is the **design contract native codegen honors once the
> carrier exists**; until then native VSA covers the real-`Vec<f64>` fragment and **explicitly refuses**
> any block-sparse or complex value. (2) The M-853/M-854 task bodies say "capacity bounds already have
> Lean 4 + Liquid Haskell proofs (M-832)"; the honest reading is narrower: the **single-op** MAP-I
> `bundle` capacity bound is Proven via the **M-001 confirmed** strategy (`proofs/lh-bundle`,
> LiquidHaskell `SAFE`, axiomatized cited theorem + Z3-checked arithmetic instantiation; replayed in
> `capacity.rs`). The **multi-hop / compositional** capacity work (**M-832**) is **in-progress
> research** that *discovers candidate* bounds and *emits undischarged proof obligations* — its
> underlying multi-hop theorem (OQ-A / M-827) is **open and axiomatized**, and M-832 **never stamps
> Proven** (VR-5). Native VSA codegen therefore **preserves Proven only for the single-op bundle
> side-condition that is actually checked**, and treats any composed/multi-hop bound as Empirical at
> most — never upgrading on the strength of unfinished research.

### 5.3 Dynamic-VSA JIT execution (M-855)

**Decision.** The in-process JIT (`crates/mycelium-mlir/src/jit.rs`, the M-340 `compile → dlopen →
call` path) gains a **dynamic-VSA execution mode** for workloads the AOT path cannot serve statically:

- **Data-dependent hypervector dimension** — the dimension is a runtime value (e.g. read from input or
  a streaming source), so the kernel is specialized at JIT time, not AOT time.
- **Runtime model selection** — the VSA model is chosen at runtime (e.g. a pipeline that switches
  MAP-I/BSC by data); the JIT compiles the selected model's kernel, **never** silently picking a model
  the program did not name.
- **Cleanup / resonator loops** — the iterative decode (RFC-0009 resonator factorization; the
  `cleanup` associative memory) JIT-compiled, **preserving RFC-0009's never-silent contract**: a
  resonator run yields `Factorization{factors, trace}` **only** on `Converged` with the confidence/
  margin gates clear, and emits an explicit `Oscillating`/`Stalled`/`BudgetExhausted` error (with the
  trace, for EXPLAIN) otherwise — its bound stays **Empirical** (FR-C2), never `Proven`.

**Never-silent selection (G2).** The dynamic-VSA JIT is an explicit `ExecMode` variant
(`crates/mycelium-mlir/src/mode.rs` already defines `ExecMode {Interpreter, Aot, Jit}` with the
dispatcher `run(mode, …)` reaching JIT **only** when `ExecMode::Jit` is named — there is **no `Auto`
arm and no heuristic fallback**). This RFC's dynamic-VSA path keeps that discipline: it is reachable
only by naming the mode (API/flag), and an out-of-subset node is an explicit `ModeError`/`AotError`
refusal whose recovery (re-run under `Interpreter`) is the caller's deliberate choice, never automatic.

**Correctness bar.** `JIT ≡ interpreter` over the dynamic-VSA subset (**Empirical**, via the §5.4
checker), plus an explicit refusal outside the subset. The interpreter remains the reference for the
dynamic path exactly as for the static one.

### 5.4 The honesty / verification contract (M-858 durability; per-increment in M-853/854/855)

- **Differential equivalence is the bar.** Every native Dense/VSA op is validated by the
  **interp ≡ direct-LLVM ≡ MLIR-dialect-where-it-runs** three-way differential (and **interp ≡ JIT**
  for the dynamic path), each pair checked through the **shared M-210 observational-equivalence
  checker** — `mycelium_cert::check(reference, candidate, relation, claimed, evidence)` returning
  `CheckVerdict::Validated{strength}` or `NotValidated{reason, Fallback::UseReference}`. The relation
  is `ObservationalEquiv` for an exact native op (Dense `add`/`neg`; VSA `bind`/`permute`) and
  `BoundedSimilarity`/`Bijection` for a bounded/bijective native swap, carrying the emitted
  `SwapCertificate` as `Evidence::Swap`. The harness pattern is the existing
  `tests/threeway_differential.rs` + `tests/jit_differential.rs` (corpus → run each path →
  `assert_eq!(observable(ref), observable(candidate))` → `check(...) == Validated{…}`), extended with a
  Dense corpus and a per-model VSA corpus (including a **capacity-bound parity case** that asserts the
  native `bundle` issues Proven iff the reference does, and Empirical/refusal otherwise).
- **The reference stays the reference.** `mycelium-dense`/`mycelium-vsa` (and the interpreter above
  them) are the trusted base; the native path is the **performance layer**, never the source of
  meaning. A `NotValidated` verdict means `Fallback::UseReference` — refuse the native result, run the
  reference (NFR-7).
- **Mutant-witnessed.** Each native fragment's differential is witnessed by `cargo-mutants`: a
  mutation of the Dense/VSA lowering (a quant-granularity flip; a bind/bundle operand swap; a
  reduction-order change) **must be caught** by the suite for the coverage claim to be `Empirical`;
  absent a demonstrated catch the claim stays `Declared` (RFC-0029 §7.5, extended here). This is the
  M-858 durability gate for the new fragments.
- **Honest tags, never upgraded.** Every native op's tag is **derived from how it was verified**, never
  asserted: `Exact` for the algebraically-exact ops, `Proven` only by replaying the reference's checked
  side-condition, `Empirical` once the differential + mutant-witness fire, `Declared` until then. No
  native lowering upgrades a guarantee past the reference's basis (VR-5).
- **Every uncovered fragment is an explicit refusal (G2).** No silent miscompile and no silent
  fallback: an unsupported dtype/quant/model/element-space/op is an explicit, EXPLAIN-able
  `AotError`/`DialectError`/`ModeError` routed to the interpreter, with a message naming what was
  refused and where it runs instead.
- **No-opaque-lowering (RFC-0004 §6 / ADR-006).** Every Dense/VSA lowering stage is dumpable/diffable
  (the per-element textual IR via `emit_llvm_ir`/`emit_mlir`), and the `Meta.physical` schedule +
  `Meta.bound` cert are inspectable. No Dense/VSA pass is a black box.

## 6. The ADR-009 dynamic-VSA JIT deferral lift (append-only cross-reference)

ADR-009 (Hybrid execution; Accepted) established: *one Core IR, multiple backends — interpreter (the
reference semantics), JIT, AOT; AOT preferred for stable components; interpretation/JIT for
development, exploration, and **dynamic VSA**.* ADR-009 thus already names dynamic VSA as a JIT use
case, and RFC-0029 §5.3 records that "ADR-009 already sanctions JIT, so **no superseding ADR is
required**" for the JIT *mechanism*. ADR-034 §6, however, frames the dynamic-VSA JIT specifically as
"**lifting ADR-009's deferral** for that path," and names this RFC (or a focused ADR-009 amendment, the
maintainer's call) as the place to record it.

**This RFC records the lift here, append-only:** the dynamic-VSA JIT execution mode (§5.3) is the
realization of ADR-009's "interpreter/JIT for dynamic VSA" clause for the data-dependent-dimension /
runtime-model-selection / cleanup-resonator workloads — moving it from sanctioned-but-deferred to
designed (now Accepted). This is a **cross-reference, not a rewrite**: ADR-009's text is unchanged; the
append-only pointer recorded against it (to be applied by the integrating parent — see the flagged
edits accompanying this RFC) is *"the dynamic-VSA JIT deferral is lifted by RFC-0039 §6 (Accepted
2026-06-30); the JIT mechanism itself was already sanctioned (RFC-0029 §5.3)."*

**OQ-1 resolved (maintainer 2026-06-30): this §6 cross-reference IS the vehicle — there is NO separate
ADR-009 amendment.** The §6 record here, plus the Foundation ADR-009 cross-reference note, are the full
capture of the dynamic-VSA JIT deferral lift; a focused ADR-009 amendment (the ADR-024 single-gate-row
pattern) was considered and is **not** taken (the JIT *mechanism* needed no superseding ADR — RFC-0029
§5.3 — and the *deferral lift* is fully recorded by this RFC, append-only against ADR-009).

## 7. Increment map (design-gated on this RFC)

**RFC-0039 is now Accepted (2026-06-30)**, so the design gate each increment waits on is **met** —
each remains **design-gated on this RFC** in the sense that it implements *to* RFC-0039's normative
decisions, with a checked three-way differential and honest tags, never-silent at every residual
boundary (per its DoD and ADR-034 §6). **The full-coverage half — quantized Dense (ADR-030) and
element-space/block-sparse/complex VSA (ADR-031) — has a second enabling dependency: E20-1** (which
lands the `QuantDesc` / `VsaElem` / `VsaSparsity` / `HypervectorC` `Repr` fields). The un-quantized /
real fragment proceeds now; the quantized / element-space fragment widens **as E20-1 lands those
reprs** (OQ-4 "both"), refusing the unbuilt variants never-silently in the interim.

| Task | Scope | Reference | Gate |
|---|---|---|---|
| **M-853** | Native Dense lowering (`dense_codegen.rs`, direct-LLVM first; dialect later) — removes the Dense refusal (`llvm.rs:2170`, `native.rs:349-350`); un-quantized F32/BF16 first, widening to ADR-030 quant / accumulator width / packing schedule as inspectable `Meta.physical` **as E20-1 lands `QuantDesc`**; never-silent on unsupported quant modes. | `mycelium-dense` (+ `mycelium-cert::dense`) | RFC-0039 Accepted (met); three-way differential green; `cargo-mutants` catches a quant-granularity mutation → Empirical. Quant half also gated on **E20-1**. |
| **M-854** | Native VSA lowering (`vsa_codegen.rs`, direct-LLVM first; dialect later) — `bind`/`bundle`/`permute` over the 1.0.0-mandatory **MAP-I/BSC/HRR/FHRR** (SBC/MAP-B extend post-mandate); real-`Vec<f64>` fragment first, widening to ADR-031 element-space/block-sparse/complex **as E20-1 lands those carriers**; preserves the RFC-0003 §4.1 Proven/Exact/Empirical tags only where the checked basis holds. | `mycelium-vsa` (+ `mycelium-cert::dense_vsa`) | RFC-0039 Accepted (met); three-way differential green incl. a capacity-bound parity case; `cargo-mutants` catches a bind/bundle mutation. Element-space half also gated on **E20-1**. |
| **M-855** | Dynamic-VSA JIT (`jit.rs`) — data-dependent dim, runtime model selection, cleanup/resonator loops, as an explicit never-silently-selected `ExecMode` (no `Auto` arm); records the ADR-009 lift (§6, no separate amendment — OQ-1). | interpreter (interp ≡ JIT) | RFC-0039 Accepted (met; §6 lift recorded); `JIT ≡ interpreter` over the dynamic subset (Empirical) + explicit refusal outside it; `cargo-mutants` catches a JIT-codegen mutation. |

## 8. Resolved decisions (was: open questions)

The four Proposed open questions, **resolved by the maintainer 2026-06-30** (the ratification that
moved this RFC to Accepted). Recorded append-only with their resolutions; the body sections above
(§3/§5/§6/§7) are updated to match.

- **OQ-1 (RFC vs. ADR-009 amendment) — RESOLVED.** The **§6 cross-reference IS the vehicle**; there is
  **no separate ADR-009 amendment**. The §6 append-only record, plus the Foundation ADR-009
  cross-reference note, are the full capture of the dynamic-VSA JIT deferral lift. (The JIT *mechanism*
  needed no superseding ADR — RFC-0029 §5.3 — and the *deferral lift* is fully recorded here against
  ADR-009, append-only. The ADR-024 single-gate-row amendment pattern was considered and not taken.)
- **OQ-2 (Dense coverage sequencing) — RESOLVED.** **Yes** — native Dense codegen scopes to the
  **F32/BF16 (un-quantized, real) fragment first**, and the **full ADR-030 int/fp8/TF32
  quant/accumulator/packing set widens as E20-1 lands the `QuantDesc` descriptor** (`PerTensor` +
  `PerBlock` first, per ADR-030's own staging). The unbuilt quantized dtypes/schemes stay an explicit
  never-silent refusal until each is present + differential-checked (§5.1).
- **OQ-3 (1.0.0-mandatory VSA models) — RESOLVED.** The **commonly-used standard models are
  1.0.0-native-mandatory — MAP-I, BSC, HRR, FHRR** (the standards in the literature / `torchhd` use).
  The **niche/less-common models — SBC, MAP-B** (and any others) — **extend post-mandate** (each still
  differential-checked + honest-tagged when added; interpreter-served + never-silent refusal in the
  native path until then). §5.2 marks the set accordingly.
- **OQ-4 (quant/element-space sequencing vs. E20-1) — RESOLVED: BOTH.** Native codegen covers the
  **un-quantized/real fragment NOW** (M-853/M-854), **AND** this RFC **commits** to widening to the
  quantized Dense (ADR-030) and element-space/block-sparse/complex VSA (ADR-031) variants — **gated
  only on E20-1 landing those `Repr` fields** (native codegen cannot precede the repr existing). So
  "both" is **sequenced by the E20-1 dependency, not deferred indefinitely**: **E20-1 is the enabling
  dependency for the full-coverage half** (§7), and the unbuilt variants stay a never-silent refusal in
  the interim (§3 in-scope commitment; §5.1/§5.2).

## 9. Grounding / honesty

- **RFC-0029 §3/§5 Q6** — the explicit Dense/VSA exclusion this RFC fills; not contradicted, extended.
- **RFC-0001 §4.1/§4.3** — the `Repr` value model (`Dense{dim,dtype}`, `VSA{model,dim,sparsity}`) +
  `Meta.physical` (schedule, not type) + `Meta.bound`.
- **RFC-0003 §4.1 (r3 erratum) / §5 / §6** — the authoritative VSA per-op guarantee matrix (the tags
  the codegen preserves), the cited-theorem-plus-checked-instantiation capacity strategy, and the
  resonator/cleanup manifest semantics. Encoded in `mycelium-vsa/tests/matrix.rs`.
- **RFC-0004 §5/§6/§11** — schedule-staged packing as `Meta.physical` (DN-01), no-opaque-lowering for
  all backends, and the additive direct-LLVM increment pattern this RFC's increments follow.
- **RFC-0002** — the swap/cert discipline (honest `{ε, δ, strength}`; never assert a basis) the native
  Dense dequant / Dense↔VSA swaps reuse via the M-210 checker.
- **RFC-0009** — resonator factorization: Empirical-only, never-silent `NoConverge` — the contract the
  dynamic-VSA JIT cleanup/resonator loops preserve.
- **ADR-030 / ADR-031** — the Dense quant descriptor and the VSA element-space/sparsity/complex-carrier
  designs the codegen must honor (Accepted; the `Repr` fields are **landed by E20-1** — native codegen
  **commits to widen** to them as E20-1 lands them, OQ-4 "both"; the honesty notes in §5.1/§5.2 record
  that the descriptors are not yet in the value model).
- **E20-1** — the content-address rehash that lands the ADR-030 `QuantDesc` / ADR-031
  `VsaElem`/`VsaSparsity`/`HypervectorC` `Repr` fields; the **enabling dependency for the full-coverage
  (quantized-Dense + element-space-VSA) half** of M-853/M-854 (§7; OQ-4).
- **ADR-009** — the hybrid-execution decision; §6 records the dynamic-VSA JIT deferral lift
  append-only (OQ-1 resolved: §6 is the vehicle, no separate ADR-009 amendment).
- **ADR-034 §6** — the re-gating that names this RFC the vehicle for native Dense+VSA codegen and the
  ADR-009 lift; the RFC-vs-amendment question it flagged is resolved here (OQ-1, §6/§8).
- **ADR-006 / G2 / VR-5 / KC-3** — EXPLAIN obligation, never-silent, honest tags never upgraded past
  basis, small auditable kernel (the native path stays outside the trusted base).
- **Code, checked 2026-06-30:** `crates/mycelium-mlir/src/{llvm.rs:2170, dialect/native.rs:349-350,
  jit.rs, mode.rs:55-72}` (the live Dense/VSA refusals + the never-silent `ExecMode`);
  `crates/mycelium-dense/src/lib.rs` (the Dense reference + Higham bounds);
  `crates/mycelium-vsa/src/{mapi.rs,mapb.rs,bsc.rs,hrr.rs,fhrr.rs,sbc.rs,capacity.rs,resonator.rs}`
  (the VSA reference + the single-op Proven `capacity::proven_capacity_bound`);
  `crates/mycelium-cert/src/check.rs` (the M-210 checker); `proofs/lh-bundle` (M-001, the confirmed
  single-op capacity proof); `proofs/vsa-multihop-bound` + M-832 (the **in-progress, undischarged**
  multi-hop research — *not* a Proven basis).

---

### Changelog

| Date | Status | Note |
|---|---|---|
| 2026-06-30 | **Accepted** | Maintainer-ratified; the four §8 open questions resolved. **OQ-1:** the §6 cross-reference IS the vehicle for the ADR-009 dynamic-VSA JIT deferral lift — NO separate ADR-009 amendment (§6 record + the Foundation ADR-009 cross-ref note are the full capture). **OQ-2:** native Dense scopes to the F32/BF16 un-quantized fragment first; the full ADR-030 int/fp8/TF32 quant/accumulator/packing set widens as E20-1 lands `QuantDesc`. **OQ-3:** the standard models MAP-I/BSC/HRR/FHRR are 1.0.0-native-mandatory; the niche SBC/MAP-B extend post-mandate (each differential-checked + honest-tagged when added). **OQ-4 (BOTH):** native codegen covers the un-quantized/real fragment now AND commits to widening to quantized-Dense (ADR-030) + element-space/block-sparse/complex VSA (ADR-031) — gated only on **E20-1** landing those `Repr` fields (the enabling dependency for the full-coverage half), refusing the unbuilt variants never-silently in the interim. The Proven-scope correction is unchanged (single-op MAP-I bundle Proven only; the multi-hop M-832 work stays in-progress research, never Proven — VR-5). Acceptance ratifies the **design** only — asserts no implementation; M-853/M-854/M-855 stay design-gated on this RFC (now Accepted) before they implement; RFC-0039 → Enacted only when the path is complete + stable (house rule #3). Task: E25-1 / M-853. |
| 2026-06-30 | **Proposed** | Created. Native-codegen design for `Repr::Dense` (M-853 — element-wise ops + ADR-030 quant as inspectable `Meta.physical`/DN-01) and `Repr::Vsa` (M-854 — bind/bundle/permute over MAP/BSC/HRR/FHRR/SBC, honoring ADR-031, preserving the RFC-0003 §4.1 per-op tags only where the checked basis holds — VR-5), plus the dynamic-VSA JIT (M-855 — data-dependent dim / runtime model selection / cleanup-resonator loops as an explicit never-silently-selected `ExecMode`), and the M-210-checked, mutant-witnessed, interpreter-referenced honesty contract. Fills the Dense/VSA gap RFC-0029 §3 excludes (does not contradict it). Records the ADR-009 dynamic-VSA JIT deferral lift append-only (§6), with OQ-1 flagging whether a focused ADR-009 amendment is also wanted. Asserts **no** implementation; nothing moves to Accepted/Enacted by authoring. Task: E25-1 / M-853. |
