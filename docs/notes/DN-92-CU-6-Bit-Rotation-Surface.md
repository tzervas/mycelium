# Design Note DN-92 — CU-6 Bit-Rotation Surface: dedicated `bit.rotl`/`bit.rotr` prims vs a derived width-safe surface

| Field | Value |
|---|---|
| **Note** | DN-92 |
| **Status** | **Draft** (2026-07-08) — works up the CU-6 rotate/reverse decision (DN-34 §8.16 `FLAG-math-3`) for maintainer ratification. Recommends **Option A** (a dedicated `bit.rotl`/`bit.rotr` (+`bit.reverse`/`bit.swap_bytes`) kernel-prim family) and presents **Option B** (a derived `std.math` surface) with its tradeoffs. **Enacts nothing; ships no code; moves no other decision's status** (house rule #3). |
| **Decides** | *Proposes, for ratification:* (1) **prim-vs-derived** — whether faithful bit-rotation is surfaced as a dedicated kernel-prim family (Option A) or a derived width-safe `std.math` surface (Option B); (2) the **rotate-amount convention** — `n mod N`, with `n = 0` and `n ≥ N` **total** (identity / full wrap-around), never a refusal on the identity case; (3) **bit-reverse and byte-reverse** (`reverse_bits`/`swap_bytes`) in the same family; (4) the **honest per-op guarantee tags** (total bit-permutations are `Exact`; three-way agreement `Empirical`); (5) the **never-silent contract** (`n = 0`/`n ≥ N` are defined, total identities — never the `shift-amount ≥ N` refusal the naive derivation trips); (6) **surface naming** (no `_u`/`_s` suffix — rotation is sign-free per ADR-028/DN-72). |
| **Feeds** | **CU-6 (surface)** — closes `lib/std/math.myc` `FLAG-math-3` (the one bit-manipulation gap left after `bpopcount`/`bclz`/`bctz` landed, DN-34 §8.16); the **trx2** transpiler's rotate emission (DN-34 §8.16 in-progress item 4 — "Rotate emits once a rotate prim lands"); the gate-D1 decision-point blocker. |
| **Depends on** | **DN-34 §8.16/§8.17** (the CU-6 deferral, `FLAG-math-3`, and the "rotate IS expressible, the naive `or()` is wrong" correction); **RFC-0036** (kernel-primitives consolidation — the add-a-prim bar and the `bit.*` prim taxonomy); **DN-39** (the four-clause KC-3 promotion bar — distinguished below from a *verified* compute-prim addition); **DN-41** (the `bit.width_cast` width-witness ABI — the precedent that a focused width-parameterised prim beats a general reflection facility); **DN-72** (the `_u`/`_s` surface-naming convention — and why rotation takes no suffix); **KC-3** (small-auditable-kernel, house rule #5 — the composition-vs-prim tension); **ADR-028** (Binary is sign-free — rotation/reversal are pure bit-permutations); **RFC-0033 §4.1.2/§4.1.3** (the `bin.shl`/`bin.shr` shift set whose `≥ N` refusal is the crux). |
| **Date** | July 8, 2026 |
| **Task** | trx2 decision-point closure — gate D1 (CU-6 rotate/reverse surface). |

> **Posture (transparency rule / VR-5 / G2).** This note is **Draft** — a design direction for the
> maintainer to ratify. It **upgrades no guarantee past its basis** and **self-Accepts nothing**
> (house rule #3). The source facts it rests on are `Exact` (read directly 2026-07-08 from
> `crates/mycelium-interp/src/prims.rs`, `crates/mycelium-l1/src/checkty.rs`,
> `crates/mycelium-core/src/binary.rs`, `lib/std/math.myc`, `lib/std/cmp.myc`); the prior-art survey
> (x86 / LLVM / Rust / RISC-V) is `Declared` (cited external references, §3); the recommendation and
> its rationale are **`Declared`-with-argument** (a grounded design position, not a proven theorem).
> The proposed prim guarantee tags (`Exact` for the total permutations) are the *supportable* strength
> for a total, decidable, bijective map on a finite domain — **not** `Proven` absent a discharged
> theorem; three-way differential agreement is `Empirical` (trials), exactly as the sibling CU-6
> counts (`bpopcount`/`bclz`/`bctz`) carry it.

---

## §1 Problem — the one CU-6 gap left open

The CU-6 prim-gap wave landed population-count and the zero-count pair as **kernel prims**
(`bit.popcount`/`bit.clz`/`bit.ctz`, PR #1275) and their width-generic `std.math` surface
(`bpopcount`/`bclz`/`bctz`, PR #1291) — DN-34 §8.16. It **deferred** the rest of the
bit-manipulation family — rotate (`rotate_left`/`rotate_right`) and reverse
(`reverse_bits`/`swap_bytes`) — to dedicated design work, recorded as `lib/std/math.myc`
`FLAG-math-3` and DN-34 §8.16's "Deferred to dedicated design work" ruling. This note is that
dedicated design work.

**The crux (grounded — `Exact`, source-read).** The obvious width-generic derivation of a left
rotation over `Binary{N}` is

```text
rotl(x, n)  ==  or( shl_u(x, n), shr_u(x, N − n) )
```

and it is **wrong on the identity case**. `bin.shl`/`bin.shr` (surface `shl_u`/`shr_u`) are
*never-silent*: a shift amount `≥ N` is an **explicit `EvalError::PrimType` refusal**, not a
silently-zeroed result (`crates/mycelium-interp/src/prims.rs` §"never-silent logical shift", lines
897–920; RFC-0033 §4.1.3). So at `n = 0` the complementary shift is `shr_u(x, N − 0) = shr_u(x, N)`
— a full-width shift, amount `= N ≥ N` — which **refuses**. The naive rotate therefore *refuses on
the very case that must be the identity* (`rotl(x, 0) == x`). This is the DN-34 §8.16 correction
verbatim: rotate *is* expressible, but the naive `or(shl_u, shr_u)` is **not** a clean derivation
from the surfaced prims (VR-5 — never faked).

Two further facts sharpen the derivation problem (both `Exact`, source-read):

1. **Rotate needs `N` as a *runtime value*** to form `N − n` (and to normalise `n mod N`). Inside a
   width-generic `.myc` body the width `N` is a **type-level** monomorphisation parameter, not a
   `Binary{N}` value — and there is **no width-reflection surface** to obtain it (grep of
   `lib/std/*.myc` 2026-07-08: no `width_of` / `width!` / reflection intrinsic exists). `FLAG-math-3`
   names exactly this: rotate is "gated on either a dedicated `bit.rotl`/`bit.rotr` prim **or** a
   width-reflection surface."
2. **The arithmetic the derivation needs is not surfaced either.** The `n mod N` normalisation needs
   `rem_u` and the `N − n` step needs `sub_u`; `bsub` is surfaced but width-generic **division/
   remainder is explicitly *not* surfaced** in `std.math` yet (`math.myc` `FLAG-math-1`). So even
   with `N` in hand the mod-normalisation step is currently unavailable at the surface.

The surface *can* already express the `n = 0` **guard** — `.myc` has `match` over enum/`Bool`
values (`lib/std/cmp.myc` uses it throughout) — so a conditional is not the blocker. The blocker is
**getting `N` as a value** and the **mod/sub arithmetic** over it.

---

## §2 The decision — two options

### Option A — a dedicated `bit.rotl`/`bit.rotr` (+ `bit.reverse`/`bit.swap_bytes`) prim family *(recommended)*

Add a focused kernel-prim family, each a total bit-permutation over `Binary{N}`, with the width `N`
known intrinsically to the kernel (it is `a.len()` of the value operand — the kernel already reads it
for `bit.popcount`/`clz`/`ctz`). The prim performs the `n mod N` normalisation **itself**, so the
`n = 0` / `n ≥ N` cases are **total by construction** — the identity/full-wrap is computed, never
refused.

- **`bit.rotl : (Binary{N}, Binary{N}) → Binary{N}`** — rotate-left. Second operand is the rotate
  amount `n`, read as an unsigned `N`-bit magnitude (same operand shape as `bin.shl`); the prim
  applies `n' = n mod N` and rotates by `n'`. `n' = 0` (i.e. `n ≡ 0 (mod N)`) is the identity.
- **`bit.rotr : (Binary{N}, Binary{N}) → Binary{N}`** — rotate-right (mirror).
- **`bit.reverse : (Binary{N}) → Binary{N}`** — reverse the bit order of the whole value. Total,
  self-inverse.
- **`bit.swap_bytes : (Binary{N}) → Binary{N}`** — reverse the byte order. Defined **iff** `N mod 8 = 0`;
  a non-byte-multiple width is an **explicit never-silent refusal** (`EvalError::PrimType`), never a
  guessed partial-byte swap (G2).

Surface (`std.math`, following the `b`-prefix convention of `bpopcount`/`bclz`/`bctz`/`bmul`; no
`_u`/`_s` suffix — rotation/reversal are sign-free, ADR-028/DN-72 §2): `brotl` / `brotr` /
`breverse` / `bswap_bytes`, each a width-generic wrapper delegating to the kernel prim.

**Namespace:** the `bit.*` family, alongside `bit.popcount`/`bit.clz`/`bit.ctz`/`bit.and`/`bit.or`/
`bit.xor`/`bit.not` — the *total* bit-manipulation ops — **not** the `bin.*` arithmetic family
(`bin.add`/`bin.shl`/…). Rotation and reversal have no overflow/refusal mode on the value domain
(only `swap_bytes`' width precondition), matching the `bit.*` totality profile exactly.

### Option B — a derived width-safe `std.math` surface

Keep the kernel unchanged (KC-3 minimal) and express rotation in `.myc` as a composition of existing
prims, made total by an explicit `n mod N` normalisation and an `n = 0` guard. Sketch:

```text
fn brotl{N}(x: Binary{N}, n: Binary{N}) => Binary{N} =
  let k = rem_u(n, width_value{N});        // n mod N   — needs rem_u AND N-as-a-value
  match is_zero(k) {                        // the n = 0 guard (match exists in .myc)
    True  => x,                             // identity — never touches shr_u(x, N)
    False => or( shl_u(x, k), shr_u(x, sub_u(width_value{N}, k)) )
  };
```

This is faithful **only if three currently-absent pieces are added first**:

1. a **width-reflection surface** `width_value{N} : Binary{N}` (or an intrinsic) to materialise `N`
   as a value — the "width-reflection surface" `FLAG-math-3` names;
2. the **surfaced `rem_u`** (`n mod N`) and **`sub_u`** (`N − n`) over `Binary{N}` — `sub_u`/`bsub`
   exists, but width-generic **remainder is not surfaced** in `std.math` (`math.myc` `FLAG-math-1`);
3. `reverse_bits`/`swap_bytes` have **no shift-composition at all** that is efficient or
   width-generic — a portable bit-reverse is an O(log N) shift-and-mask ladder (§3), so Option B
   would *still* need dedicated prims for the reverse half, or leave `FLAG-math-3` half-open.

So Option B does **not** avoid a kernel/language addition — it trades a focused, host-mapped rotate
prim for a **more general** width-reflection facility (plus surfaced division), and it does not close
the reverse family at all.

---

## §3 Prior art (`Declared` — cited external references)

Every mainstream rotate surface agrees on the **`n mod width`** convention and makes **`n = 0` an
identity** — the exact convention Option A bakes into the prim, and the exact totality the naive
derivation fails to provide.

- **x86 `ROL`/`ROR`.** The rotate count is **masked to the operand size** — 5 bits (`mod 32`) for
  8/16/32-bit operands, 6 bits (`mod 64`) with `REX.W`. A masked count of 0 performs no bit movement
  and leaves flags unaffected. (Intel SDM Vol. 2, `RCL/RCR/ROL/ROR`;
  <https://www.felixcloutier.com/x86/rcl:rcr:rol:ror>.)
- **LLVM funnel shifts `llvm.fshl`/`llvm.fshr`** — the modern canonical rotate lowering. Rotate is
  the special case with the first two arguments equal: `fshl(x, x, amt)` is rotate-left,
  `fshr(x, x, amt)` rotate-right. "The shift value is treated as an unsigned amount **modulo the
  size** of the arguments"; at `amt ≡ 0` the result is the input (identity). (LLVM LangRef,
  `llvm.fshl`/`llvm.fshr`; <https://llvm.org/docs/LangRef.html#llvm-fshl-intrinsic>.) This is the
  clean AOT lowering target Option A's prim maps onto directly.
- **Rust `u32::rotate_left`/`rotate_right`.** The docs warn "this isn't the same operation as the `<<`
  shifting operator" — bits **wrap around** rather than being discarded, and the amount is **not**
  range-restricted the way a wrapping shift's RHS is: `n` may exceed the bit width and rotates by the
  full amount (`rotate_left(1024)` on a `u32` returns the input unchanged, `1024 mod 32 = 0`);
  `rotate_left(0)` is the identity. (<https://doc.rust-lang.org/std/primitive.u32.html#method.rotate_left>.)
- **RISC-V Zbb `rol`/`ror`/`rori`.** The rotate amount is the **least-significant `log2(XLEN)` bits**
  of the source/immediate — 5 bits (`mod 32`) on RV32, 6 bits (`mod 64`) on RV64 — the same
  mod-width convention; single-instruction rotates. (RISC-V Bit-Manipulation ISA 1.0, Zbb;
  <https://five-embeddev.com/riscv-bitmanip/1.0.0/bitmanip.html>.)
- **Bit-reverse / byte-reverse are native single instructions, not shift-derivable.** ARM/AArch64
  **`RBIT`** reverses all bits in one instruction; RISC-V Zbb **`brev8`** reverses bits within each
  byte (a full-width reverse is `rev8 ∘ brev8`). Byte-reverse is likewise one instruction: x86
  **`BSWAP`**, RISC-V **`rev8`**, ARM **`REV`**. A width-generic bit-reverse in portable code costs
  an O(log N) shift-and-mask ladder — the standard argument for exposing it as an intrinsic rather
  than deriving it. (<https://developer.arm.com/documentation/ddi0602/2022-09/Base-Instructions/RBIT--Reverse-Bits->;
  <https://www.felixcloutier.com/x86/bswap>.)

**The load-bearing generalisation:** all four rotate surfaces take the amount **modulo width** and
treat **`n = 0` (and any `n ≡ 0 mod N`) as the identity** — never an error. A never-silent language
must make the identity case *total*, and reversal is a *primitive* on every target. Both facts point
at Option A.

---

## §4 Recommendation — Option A, with the mod-`N` convention baked into the prim

**Recommend Option A.** The reasoning, grounded:

1. **It is the *smaller net* kernel addition, not the larger one.** The KC-3 tension (house rule #5)
   is real, but Option B does not actually keep the kernel unchanged — it requires a **width-reflection
   surface** (materialising `N` as a value) plus a **surfaced remainder**, and still leaves the
   reverse family needing prims (§2/§3). A focused, totally-specified rotate/reverse family is a
   *narrower and more auditable* addition than a general width-reflection facility. This is the DN-41
   precedent applied: DN-41 chose a **focused width-parameterised prim** (`bit.width_cast`, width
   carried by a witness operand) over a general reflection facility for exactly this reason.
2. **The width is already in the kernel's hand.** The kernel reads `N = a.len()` for
   `bit.popcount`/`clz`/`ctz` today; a rotate prim gets `N` for free and can compute `n mod N` and
   `N − n` internally — so the `n = 0` / `n ≥ N` cases become **total by construction**, the
   never-silent property the naive `.myc` derivation cannot achieve (it hits the `shr_u(x, N)`
   refusal). Totality-at-the-surface is *cheaper and safer* in the prim than pushed onto every caller.
3. **It matches the CU-6 precedent already ratified.** `bit.popcount`/`bit.clz`/`bit.ctz` landed as
   kernel prims on the maintainer's "single host instruction, not efficiently derivable in `.myc`"
   ruling (`prims.rs` lines 146–147). Rotate and reverse are the **same class**: single host
   instructions (x86/RISC-V/ARM; LLVM `fshl`/`fshr`), and their `.myc` derivation is *blocked*
   (width-reflection) or *inefficient* (the O(log N) reverse ladder), not merely slower. Adding them
   as `bit.*` prims is consistent with the boundary the maintainer already drew for this exact family.
4. **The AOT path is clean.** Option A's `bit.rotl`/`bit.rotr` lower directly to LLVM
   `fshl`/`fshr` (and reverse to a target `RBIT`/`BSWAP` or the portable ladder) — a single,
   well-defined intrinsic, no branch. Option B's guarded composition lowers to a compare-and-branch
   the backend must then re-recognise as a rotate idiom.

**On the DN-39 bar — an honest distinction (VR-5).** DN-39's four-clause bar governs **promotion to
the *axiomatically-trusted* core** — declaring a component exempt from verification — and its
dispositive clause (2) is "unverifiable-from-outside." Rotate/reverse are **verifiable** (a
differential/property oracle establishes them), so under a literal reading of clause (2) they would be
"verify, don't trust." **That clause does not bar Option A**, because Option A does *not* axiomatise
rotate as trusted — the prims are **verified** kernel prims (property + three-way differential tested,
exactly like `bit.popcount`/`clz`/`ctz`), not TCB axioms. The operative test for a *verified compute
prim* is RFC-0036's "irreducibly primitive **or** perf-critical single-instruction not cleanly
derivable at the surface" — and rotate/reverse clear the second limb (the surface derivation is
*blocked*, not merely slow). I flag this explicitly rather than paper over it: the KC-3 small-kernel
value still cuts against any addition, and the honest weighing (point 1) is that Option A is the
*smaller* addition, which is why it wins — not that the addition is free.

**Rotate-amount convention (both options — a total, never-silent surface):**

- `n' = n mod N`. `n = 0` (and any `n ≡ 0 mod N`) is the **identity**; `n ≥ N` wraps via the mod —
  **never a refusal** on either. This matches x86/LLVM/Rust/RISC-V (§3) and is the never-silent
  requirement: the identity case is *defined and total*, not the `shift ≥ N` error.
- `swap_bytes` carries the **one** width precondition: `N mod 8 = 0`, else an explicit refusal
  (never a guessed partial-byte swap). `reverse_bits` is total on every `N`.
- No `_u`/`_s` suffix (DN-72 §2): rotation and reversal are pure bit-permutations with no signedness
  reading (ADR-028 — Binary is sign-free), like `not`/`xor`/`and`/`or`.

---

## §5 Guarantee tags (honest per-op)

| Op | Guarantee | Basis |
|---|---|---|
| `bit.rotl` / `bit.rotr` | **`Exact`** | A total bijective permutation of the `N`-bit vector on the whole `Binary{N}` domain (the `n mod N` normalisation is total); decidable, no overflow mode. Same profile as `bit.and`/`bit.or` (total, `Exact`). |
| `bit.reverse` | **`Exact`** | Total, self-inverse bit-permutation on the whole domain. |
| `bit.swap_bytes` | **`Exact`** on `N mod 8 = 0`; **refusal** otherwise (never-silent) | Total byte-permutation on byte-multiple widths; the width precondition is an explicit `EvalError::PrimType`, `Declared`/never-silent (asserted + exhibited by a refusal test, not a proven theorem). |
| Three-way agreement (L1-eval ≡ L0-interp ≡ AOT) | **`Empirical`** | Trials over the differential/conformance corpus — exactly as `bpopcount`/`bclz`/`bctz` carry it (DN-34 §8.16). |

No tag is `Proven`: totality/bijectivity is argued (a finite decidable permutation), not
machine-checked (VR-5).

---

## §6 User stories

- **As a systems programmer** porting a hash/CRC/cipher round from Rust, **I want** `rotate_left`/
  `rotate_right` over `Binary{N}` that behave like Rust's (wrap-around, `n mod N`, `n = 0` = identity),
  **so that** the ported code is bit-exact and I never hit a surprise refusal on a zero rotate.
- **As a language user** relying on the never-silent guarantee, **I want** `brotl(x, 0) == x` to be a
  *defined identity*, **so that** the "never-silent" promise holds on the identity case instead of the
  surface refusing (the naive `shr_u(x, N)` bug).
- **As the trx2 transpiler**, **I want** a landed rotate prim to emit against (`FLAG-math-3` /
  DN-34 §8.16 item 4 — "Rotate emits once a rotate prim lands"), **so that** Rust `rotate_left`/
  `rotate_right`/`reverse_bits`/`swap_bytes` call-sites stop being an emission gap.
- **As a kernel auditor**, **I want** the rotate/reverse family to be a *small, totally-specified,
  host-mapped* `bit.*` prim set rather than a general width-reflection facility, **so that** the
  frozen-toward-1.0 kernel (RFC-0036) grows by the minimum auditable surface.

---

## §7 Definition of Done

The decision is *ratifiable* when this note presents (done here): the prim-vs-derived recommendation
with grounded rationale, the alternative with tradeoffs, the rotate-amount convention, the reverse
family, honest guarantee tags, and the never-silent contract. **Implementation DoD** (for the
follow-on that lands the ratified option — not this note):

1. **Prims + surface landed** (Option A): `bit.rotl`/`bit.rotr`/`bit.reverse`/`bit.swap_bytes` in the
   `crates/mycelium-interp` prim registry and the `mycelium-l1` `checkty` surface map; `std.math`
   `brotl`/`brotr`/`breverse`/`bswap_bytes` wrappers; `FLAG-math-3` closed in `math.myc`.
2. **Property tests (every bound):** `rotl(x, N) == x` and `rotl(x, 0) == x` (**rotate-by-`N` /
   rotate-by-`0` is the identity** — the never-silent totality); `rotr(rotl(x, n), n) == x`
   (**`rotl ∘ rotr == id`**); `rotl(x, n) == rotl(x, n mod N)` (mod-width convention); `reverse` is
   self-inverse; `swap_bytes` refuses on `N mod 8 ≠ 0`. Each a property test, **not** a single case.
3. **Three-way differential (L1/L0/AOT):** the `n = 0` identity, a mid rotate, and `n ≥ N` wrap agree
   across L1-eval, L0-interp, and the AOT leg (the CU-6 pattern) — the `Empirical` basis for the tag.
4. **Never-silent verified:** a test asserting `n = 0` / `n ≥ N` are **total** (no refusal), and that
   `swap_bytes` on a non-byte width is an **explicit refusal** (not a silent partial swap).
5. **Guarantee tags** at the strength §5 states; the Π prim count and the `checkty` surface map
   updated in lockstep; DN-34 §8.16's `FLAG-math-3` row moved to "landed."

---

## Meta — changelog

- **2026-07-08 — Created (Draft).** Works up the CU-6 rotate/reverse decision (DN-34 §8.16
  `FLAG-math-3`, gate D1) for maintainer ratification. Establishes the crux (`Exact`, source-read):
  the naive `or(shl_u, shr_u)` rotate **refuses on `n = 0`** because `bin.shr` refuses a shift amount
  `≥ N`, and a `.myc` derivation is blocked because the width `N` is a type-level parameter with **no
  width-reflection surface** and width-generic remainder is unsurfaced. Recommends **Option A** — a
  focused `bit.rotl`/`bit.rotr`/`bit.reverse`/`bit.swap_bytes` prim family (width known intrinsically,
  `n mod N` normalised in-prim so `n = 0`/`n ≥ N` are **total** identities, `swap_bytes` refuses on
  non-byte widths), namespaced `bit.*` with `b`-prefixed `std.math` wrappers and **no** `_u`/`_s`
  suffix (sign-free, ADR-028/DN-72). Presents **Option B** (derived surface) and shows it needs a
  *larger* addition — a width-reflection facility + surfaced remainder — and still cannot close the
  reverse half. Grounds the convention in x86/LLVM `fshl`/`fshr`/Rust/RISC-V prior art (all `n mod
  width`, `n = 0` = identity; `Declared`). Distinguishes the DN-39 *promotion* bar (axiomatic trust)
  from a **verified** compute-prim addition, honestly (VR-5): the prims are verified, not trusted, so
  clause (2) does not bar Option A, and the KC-3 weighing is that Option A is the *smaller* addition.
  Guarantee tags: total permutations `Exact`, three-way agreement `Empirical`, `swap_bytes` width
  precondition `Declared`/never-silent; nothing `Proven`. **Enacts nothing; ships no code; moves no
  other decision's status** (house rule #3). (Append-only; VR-5; G2.)
