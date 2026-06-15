# Native-Ternary Forward-Compatibility Mapping (M-370)

| Field | Value |
|---|---|
| **Status** | **Living note** (build-phase forward map, 2026-06-15) — documentation + stub target, not a built backend |
| **Owns** | the documented mapping from Mycelium's **ternary value-semantics contract** to a hypothetical future **3-state hardware backend**, and what stays invariant across that move |
| **Grounding** | **ADR-005** (ternary = logical substrate now, native HW later); **R7** (forward-compatible value-semantics contract); RFC-0004 §2 (`ternary` MLIR dialect → backends); RFC-0001 §4.1 (`Repr::Ternary`, `Trit`); NFR-1 / **G3** (packing is inspectable metadata, not hidden lowering); NFR-7 (interpreter is the reference semantics) |
| **Companion** | `crates/mycelium-mlir` (the AOT path: `dialect::emit` textual `ternary` dialect skeleton + `llvm` direct-LLVM backend that already lowers `trit.neg`); `docs/planning/phase-3.md` §9 (M-301/M-370) |

> **Scope / honesty.** ADR-005 is explicit that there is **no competitive ternary hardware** today
> (Setun emulated ternary; BitNet is weight *quantization*, not general ternary compute). This note is
> therefore a **forward-compatibility contract**, not a backend: it records what Mycelium must keep
> invariant so that *if* a 3-state target ever arrives, the value semantics port without a rewrite —
> and points at the existing dialect skeleton as the **stub target**. No 3-state backend is built or
> claimed (VR-5).

## 1. The ternary value-semantics contract (what must stay invariant)

The contract a native 3-state backend must preserve — the same one the interpreter (the reference
semantics, NFR-7) and the current emulated path obey:

1. **Balanced ternary, value-first.** A `Repr::Ternary{m}` value is `m` balanced trits
   `{-1, 0, +1}` with `value = Σ tᵢ·3ⁱ` (RFC-0001 §4.1; `mycelium-core::ternary`). Negation is
   digit-wise sign flip (exact, no carry); `add`/`sub`/`mul` are the balanced-ternary arithmetic with
   an explicit out-of-range **refusal**, never a silent wrap (M-111). A backend must compute the
   *same values* — the trit encoding is an implementation detail below the value.
2. **Packing is inspectable metadata, not the type (G3 / NFR-1 / DN-01).** Which physical packing a
   value uses (`I2_S`/`TL1`/`TL2` on binary HW today; a native 3-state cell tomorrow) lives in
   `meta.physical` (`PhysicalLayout`), chosen at a lowering stage by the reified selector (M-250) and
   recorded — never baked into the type or the content hash. A native backend swaps the *packing
   target*, not the value semantics or the selection mechanism.
3. **No opaque lowering (G2 / RFC-0004 §2/§6).** Every stage is dumpable/diffable. The `ternary`
   dialect renders one op per binding with all attributes visible (`dialect::emit`); a native backend
   adds a *lowering target* below the dialect, not a hidden pass.
4. **Interpreter is the reference; every backend is validated against it (NFR-7 / VR-4).** AOT, JIT,
   and any future native-ternary artifact must be **observably equivalent** (`repr + payload +
   guarantee`) to the interpreter, checked through the *same* M-210 translation-validation machinery
   the swaps and the interp↔AOT/JIT differentials already use. A 3-state backend is gated on passing
   that differential — it does not get to mean a second semantics.

## 2. The mapping: emulated-on-binary → native 3-state

| Layer | Today (binary HW, emulated) | Future 3-state HW backend | Invariant across the move |
|---|---|---|---|
| **Value** | `Repr::Ternary{m}`, balanced trits | same | the value `Σ tᵢ·3ⁱ` and the op semantics |
| **Ops** | `trit.neg/add/sub/mul` in the interpreter / direct-LLVM (`neg` lowered; arithmetic next) | native 3-state ALU ops | per-op result + the honest out-of-range refusal |
| **Storage** | BitNet-class packing (`I2_S`/`TL1`/`TL2`), 2-bit / base-3 byte codecs (`mycelium-mlir::pack`) | native 3-state cells / words | `meta.physical` records the choice; the selector (M-250) is unchanged, only its candidate set grows a native scheme |
| **Lowering** | Core IR → `ternary` dialect (textual) → LLVM (direct-LLVM today; libMLIR later) → x86/ARM | Core IR → `ternary` dialect → **native-ternary target** | the dialect boundary (RFC-0004 §2) — a new target *below* it, dumpable |
| **Validation** | interp↔AOT / interp↔JIT differential through M-210 | interp↔native differential through the **same** M-210 checker | NFR-7: observable equivalence is the gate |

**The stub target.** The `ternary` dialect (`mycelium-mlir::dialect::emit`) is the
forward-compatibility seam: it already names a `ternary.*` op per binding with explicit
`repr`/`value`/`policy`/`layout` attributes. A native 3-state backend is *a new lowering of this
dialect* (alongside the LLVM lowering), so the dialect is the stub the future target attaches to —
no Core IR or value-semantics change required. Adding a native `PhysicalLayout` variant + a selector
candidate is the only surface change, and both are *metadata* (G3).

## 3. What a native backend would NOT change (the portability guarantee, R7)

- The Core IR, `Value<Repr, Meta>`, the guarantee lattice, and the content-addressed identity
  (ADR-003) — a native target is a backend, not a language change.
- The selection mechanism (RFC-0005) and EXPLAIN — a native packing is one more reified candidate.
- The honesty rule — a native op's guarantee tag is whatever its basis supports, never upgraded by
  virtue of running on dedicated hardware (VR-5).
- The interpreter-as-reference contract — the native artifact is validated against it (NFR-7), never
  the other way around.

## 4. Open / deferred

- The native-ternary **arithmetic lowering** (`trit.add/sub/mul` with carry/overflow) is the next
  direct-LLVM slice (M-301) and a prerequisite for a meaningful native target; `neg` is done.
- A concrete native `PhysicalLayout::NativeTernary{…}` variant + selector candidate is **deferred**
  until a real target exists (ADR-005) — adding it now would be a `Declared` placeholder with no
  basis.
- BitNet-class *acceleration* on binary HW (FR-C3 / G3, M-360) is the nearer-term performance path;
  this note is the *hardware-portability* contract, orthogonal to it.

## Meta — changelog

- **2026-06-15 (M-370 initial map):** documented the ternary value-semantics contract (§1), the
  emulated→native mapping with the `ternary` dialect as the stub target (§2), the portability
  guarantee (§3), and the deferred native-arithmetic/layout items (§4). Documentation only — no
  3-state backend built (ADR-005 / VR-5).
