# Design Note DN-41 — The `Binary` Width-Cast Prim (Zero-Extension Widen / Checked Narrow)

| Field | Value |
|---|---|
| **Note** | DN-41 |
| **Status** | **Accepted** (2026-06-26; **ratified by the maintainer** — width-cast prim sanctioned; impl Rust-first landed waveN2) — designs the never-silent `Binary` width-cast kernel prim (`width_cast` surface / `bit.width_cast` kernel) that wave-n1 flagged as missing: the prim needed to re-width an unsigned `Binary{N}` value to `Binary{M}` so a `Binary{8}` byte index can be compared against a `Binary{32}` length. **Maintainer ratifies → Accepted** (house rule #3); this note **proposes** the design and records the Rust-first implementation that lands with it ("implemented (Rust-first), pending ratification" — never silently `Accepted`/`Enacted`). |
| **Feeds** | **M-717** (E13-1 text/fmt: multi-byte UTF-8 decode + `byte_at`) — the concrete unblock: `lt(width_cast(idx8, len32), len32)`, a `Binary{8}` byte index widened to `Binary{32}` to be `lt`-compared against a `bytes_len`/`seq_len` result (both `Binary{32}` — RFC-0032 D3/D4). Extends **RFC-0032 D1/D2** (the comparison + binary-arithmetic kernel enablers, §5) with the missing width-bridging prim the comparison prims need when operands differ in width. Grounded in **ADR-028** (Binary is sign-free) — which makes widening a pure zero-extension. |
| **Date** | June 26, 2026 |
| **Decides** | *Proposes, for ratification:* (1) the **prim** `bit.width_cast(value: Binary{N}, into: Binary{M}) -> Binary{M}` and its surface name `width_cast`; (2) the **semantics** — widen (`M > N`) zero-extends; same-width (`M == N`) is identity; narrow (`M < N`) keeps the low `M` bits **iff** the dropped high `N − M` bits are all zero; (3) the **width-witness** ABI — the target width `M` is carried by the *second operand's width* (its bits are unused), so `M` threads to the kernel through the existing surface→kernel dispatch with no result-type plumbing; (4) the **honest per-op guarantee tags** (widen/identity/in-range-narrow `Exact`; the narrowing-fit **contract** `Declared`/never-silent); (5) the **never-silent contract** (a lossy narrow is an explicit `EvalError::Overflow`, never a silent truncation — G2/VR-5); (6) **EXPLAIN-ability**; and (7) the **placement** (`crates/mycelium-interp` prim registry + the `mycelium-l1` checker/elaborator surface — the same trusted base as the RFC-0032 enablers). |
| **Task** | M-798 (E19-1 `kpr` follow-on — the width-bridge enabler wave-n1 flagged; design + Rust-first implementation) |

> **Posture (transparency rule / VR-5 / G2).** This note is **Proposed** — a design direction for the
> maintainer to ratify. It **does not** move any decision to `Accepted`/`Enacted` on its own
> authority, and it **upgrades no guarantee past its basis**: the widen/identity/in-range-narrow cases
> are `Exact` (a total, lossless, decidable map on the unsigned magnitude — `Binary` is sign-free per
> ADR-028); the *narrowing-fit contract* (out-of-range narrow refuses) is `Declared`/never-silent
> (asserted and exhibited by the refusal test, **not** a proven theorem). The Rust-first
> implementation that accompanies this note is "implemented (Rust-first), pending ratification" — the
> prim is **landed and tested**, the *spec status* stays **Proposed** until the maintainer ratifies.

---

## §1 Purpose & the gap

Wave-n1 (the self-hosted-stdlib collections + text/fmt port, RFC-0032 D1–D4 enablers) hit a wall in
the text path (M-717): a UTF-8 / `byte_at` routine has a **byte index** that is naturally a small
`Binary{8}` (or other narrow width), but the **length** it must be bounds-checked against is a
`Binary{32}` (the `bytes_len`/`seq_len` result shape — `prims.rs::u32_as_binary32`, RFC-0032 D3/D4).
The comparison prims `eq`/`lt` (RFC-0032 D1) require **equal-width, same-paradigm** operands and
**refuse** a width mismatch as an explicit type error (`prims.rs::cmp_repr_operands`). There was **no
kernel prim to bridge the widths** — so `lt(idx8, len32)` could not be written, and the bounds check
that every safe indexing routine needs could not be expressed over the kernel. `width_cast` is that
missing bridge.

The fix is deliberately a **single, small, never-silent kernel prim** (KC-3), not a family of
per-width conversions: one prim, width-polymorphic in both `N` and `M`.

## §2 The prim — signature & semantics

```
width_cast(value: Binary{N}, into: Binary{M}) -> Binary{M}      // surface name: width_cast
bit.width_cast                                                   // kernel registry name
```

`value` is the unsigned `Binary{N}` magnitude to re-width (bits MSB-first). `into` is a **width
witness**: only its `Binary{M}` *width* is read — **its bits are ignored** — and `M` is the result
width. The three cases (all on the unsigned magnitude, because `Binary` is **sign-free** — ADR-028):

| Case | Rule | Guarantee |
|---|---|---|
| **Widen** (`M > N`) | **zero-extension** — pad `M − N` zero bits on the MSB side | **`Exact`** — total, lossless; the unsigned value is unchanged |
| **Identity** (`M == N`) | a copy | **`Exact`** |
| **Narrow** (`M < N`) | keep the low `M` bits **iff** the dropped high `N − M` bits are all zero | **`Exact`** when it fits; **never-silent refuse** when it does not |

Because `Binary` carries no sign (ADR-028), widening cannot change the value and has no sign-extension
variant to choose — zero-extension is the *only* width-increasing map, and it is exact. (A future
signed view, if one is ever added above the kernel, would be a **separate, distinct named op** —
ADR-028's "signedness is operations, not the `Repr`" — never a silent reinterpretation of this one.)

## §3 The width-witness ABI — why the target width is a second operand

A kernel prim sees only `(prim: &str, args: &[&Value])` — it has **no result-type hint** (the
elaborator/evaluator map the surface name through `prim_kernel_name` using the *argument* nodes, never
the checked result type). The target width `M` must therefore be conveyed *through an argument*. Three
options were considered:

1. **Encode `M` in the prim name** (`bit.width_cast.<M>`) — rejected: `M` is unbounded, so the
   registry match table cannot be static, and the name carries data (a smell).
2. **Thread the checked result type into the `Op` node** — rejected for this change: it touches the
   prim-dispatch contract in three places (checker, elaborator, evaluator) for a single prim — larger
   blast radius than the enabler warrants (KISS/YAGNI; KC-3).
3. **A width witness operand** (chosen): the second operand's *width* is `M`; its value is unused.
   This is exactly how a typed IR carries a cast's target type alongside the value (LLVM `zext … to
   iN`), threads `M` to the kernel through the **existing** dispatch unchanged, and — crucially —
   **the motivating call already has the witness in hand**: `lt(width_cast(idx8, len32), len32)`
   reuses the very `Binary{32}` length it is about to compare against as the cast's width witness. No
   dummy operand is manufactured; the natural call site supplies it.

**Honest cost of the choice (flagged, not hidden):** the witness's *bits* are genuinely ignored, so
two calls with the same value-operand and same-width-but-different-valued witnesses are observationally
identical — the result's provenance is composed over the **value operand only** (`compose_result(&args[..1], …)`),
so the witness's value does not enter the result's content hash (correct: it does not affect the
result). The witness is a *type/width carrier*, and the checker enforces it is a `Binary{M}` (a
non-`Binary` witness is a static refusal), so it cannot be misused as a value silently.

## §4 Never-silent contract (G2/VR-5)

A **narrowing whose dropped high bits are not all zero does not fit `Binary{M}`** and is an **explicit
`EvalError::Overflow`** — never a silent truncation to the low `M` bits. This exactly mirrors the
`bit.add`/`bit.sub` out-of-range contract (RFC-0032 D2): a fixed-width result that falls outside the
representable range is a named refusal, not a wrap. The refusal fires identically on **all three
execution paths** (L1-eval, L0-interp, AOT — they share the one prim registry), pinned by
`std_widthcast.rs::narrow_overflow_refuses_on_every_path`. Type mismatches (a non-`Binary` value or
witness) are **static** refusals at check time. The fit/overflow distinction is a **runtime** contract
(the value is not known statically), so the program type-checks and the refusal is at evaluation —
exactly as `add_bin` overflow does.

## §5 EXPLAIN-ability (no black boxes)

`width_cast` is a reified `Op` node like every other prim: it appears in the L0 term, its inputs and
the chosen result width are inspectable, and its provenance is the standard `Derived{ op:
hash("bit.width_cast"), inputs: [hash(value)] }` threading (`compose_result`). There is **no hidden
coercion** — the widen pads explicit zero bits, the narrow's drop test is an explicit high-bit check,
and the refusal names the prim. A future `EXPLAIN` over a width-cast node reports "zero-extend
`Binary{N}` → `Binary{M}`" or "checked narrow `Binary{N}` → `Binary{M}` (refused: value exceeds
`2^M − 1`)" directly from the node + result.

## §6 Guarantee posture (lattice `Exact ⊐ Proven ⊐ Empirical ⊐ Declared`; VR-5)

- **`Exact`** — widen, identity, and in-range narrow: each result *equals the unsigned reference value
  exactly* (zero-extension preserves the magnitude; an in-range narrow drops only zero bits). Total
  and decidable over its in-range domain. Grounded in ADR-028 (sign-free `Binary` ⇒ unsigned magnitude
  is the whole semantics).
- **`Declared` / never-silent** — the *narrowing-fit contract* (out-of-range narrow → explicit
  refuse). Asserted and **exhibited** by `narrow_overflow_refuses_on_every_path`, **not** a proven
  theorem; flagged as such.
- **`Empirical`** — the **three-way agreement** (L1-eval ≡ L0-interp ≡ AOT) is established by *trial*
  on the `std_widthcast.rs` programs, not proven.

No tag is upgraded past its basis. The prim composes an **approximate** input by **refusing**
(`ApproxRule::Refuse`) — width-cast has no defined ε-propagation rule, so an approximate value operand
is refused rather than fabricating a bound (G2).

## §7 Implementation (Rust-first, pending ratification)

| Layer | Location | What |
|---|---|---|
| Prim | `crates/mycelium-interp/src/prims.rs` (`prim_width_cast`, registered `bit.width_cast`) | The kernel implementation: arity 2, both operands `Binary`, widen=zero-extend / narrow=checked-drop / `EvalError::Overflow` on a lossy narrow; result repr is the witness's own `Binary{M}` (cloned, not reconstructed). |
| Surface (type) | `crates/mycelium-l1/src/checkty.rs` (`try_check_seq_bytes_prim` `width_cast` arm) | `width_cast(value: Binary{N}, into: Binary{M}) -> Binary{M}`; both operands type-checked as `Binary`; result width = witness width `M`; non-`Binary` operand or wrong arity = static refusal. |
| Surface (name) | `crates/mycelium-l1/src/checkty.rs` (`prim_kernel_name`) | `"width_cast" => "bit.width_cast"`. |
| Conformance | `crates/mycelium-l1/tests/std_widthcast.rs` | Three-way differential: widen 8→32 (incl. max byte), identity, narrow-that-fits (incl. boundary 255), narrow-overflow refusal on all three paths, the M-717 composite `lt(width_cast(idx8,len32), len32)` (true + false), and the static type/arity refusals. |

The AOT path runs the prim **unchanged** (it dispatches `Rhs::Op` through the same `PrimRegistry` as
the reference interpreter — `aot.rs`), so the differential is genuinely three-way with no AOT-specific
gap. (No AOT limitation to flag.)

## §8 Definition of Done

- [x] The prim `bit.width_cast` is designed (signature, the three width cases, the width-witness ABI)
      and **grounded** (ADR-028 sign-free ⇒ zero-extension; RFC-0032 D1/D2 the enabler context; M-717
      the unblock).
- [x] Per-op guarantee tags stated at honest basis (widen/identity/in-range-narrow `Exact`; the
      narrowing-fit contract `Declared`/never-silent; three-way agreement `Empirical`) — no upgrade
      past basis (VR-5).
- [x] The never-silent contract is specified (lossy narrow → explicit `EvalError::Overflow`, never a
      silent truncation) **and** exhibited on all three paths (G2).
- [x] EXPLAIN-ability stated (reified `Op`, no hidden coercion).
- [x] Rust-first implementation landed across interp + l1, with the three-way differential test green
      (`cargo test -p mycelium-l1 --test std_widthcast`), `cargo fmt`/`clippy -D warnings` clean.
- [ ] **Maintainer ratifies `Proposed → Accepted`** (the spec status move — house rule #3). Until
      then the spec stays **Proposed** and the code is "Rust-first, pending ratification".
- [ ] **M-717 handoff:** the text/fmt port (E13-1) consumes `width_cast` to express the byte-index
      bounds check (`lt(width_cast(idx, len), len)`), completing the multi-byte UTF-8 / `byte_at`
      routine the enabler unblocks.

> **Append-only (house rule #3).** This note **supersedes nothing** and moves no decision status from
> itself. It extends RFC-0032's enabler set with a Proposed follow-on prim; RFC-0032 stays Accepted,
> ADR-028 stayed Proposed as of this note's authoring (this note *depends on* ADR-028's direction, it
> does not advance it) — **ADR-028 later reached Accepted (2026-07-01, RFC-0033's ratification act)**,
> a status move made elsewhere, not by this note. CHANGELOG / Doc-Index / issues.yaml / docs/api-index
> are owned by the integrating parent.

---

## Meta — changelog

- **2026-06-27 — extended by DN-51 (auto-widen-in-arithmetic + explicit `truncate`); append-only.**
  This prim stays exactly as designed (explicit `width_cast`; widen=zero-extension `Exact`; checked
  narrow refuses on out-of-range). **DN-51** (Accepted 2026-06-27) builds on it two ways: (1) it makes
  the widen **automatic inside binary arithmetic** when operands differ in width (still lossless +
  reified/`EXPLAIN`-able, so never *silent* — a deliberate, recorded softening of "casts are explicit");
  and (2) it **adds** an explicit **`truncate`** narrow (intentional high-bit drop) *alongside* this
  prim's checked-narrow — truncation only ever via the named op, so "never a *silent* truncation"
  (this note's decision #5) is preserved. DN-51 does not change this prim's semantics or status; it is a
  forward pointer. Grounding caveat carried forward: the zero-extension widen relies on **ADR-028**
  (Binary sign-free), still *Proposed* but committed-in-practice here.
- **2026-06-26 — Created (Proposed) — authored (M-798, E19-1 `kpr` follow-on).** Designs the
  never-silent `Binary` width-cast kernel prim `bit.width_cast` (surface `width_cast`) that wave-n1
  flagged as missing: the width-bridge `lt`/comparison needs to compare a narrow `Binary{N}` index
  against a `Binary{32}` length (M-717 multi-byte UTF-8 / `byte_at`). Records the **signature**
  (`width_cast(value: Binary{N}, into: Binary{M}) -> Binary{M}`), the **width-witness ABI** (the
  second operand's width is `M`; its bits are unused — chosen over name-encoding/result-type-threading
  for KISS/KC-3), the **semantics** (widen=zero-extension `Exact`; identity `Exact`; narrow keeps the
  low `M` bits iff the dropped high bits are zero — `Exact` when it fits, never-silent
  `EvalError::Overflow` when it does not), grounded in **ADR-028** (sign-free `Binary` ⇒ widening is
  zero-extension). **Per-op tags at honest basis** (widen/identity/in-range-narrow `Exact`; the
  narrowing-fit contract `Declared`/never-silent; three-way agreement `Empirical` — no upgrade past
  basis, VR-5). **EXPLAIN-able** (reified `Op`, no hidden coercion). **Rust-first implementation
  landed** in `crates/mycelium-interp/src/prims.rs` (`prim_width_cast`) + surfaced/type-checked in
  `crates/mycelium-l1/src/checkty.rs` (`try_check_seq_bytes_prim` + `prim_kernel_name`), with a full
  **three-way differential** (L1-eval ≡ L0-interp ≡ AOT) + never-silent narrowing-overflow refusal on
  all three paths + the M-717 motivating composite, in `crates/mycelium-l1/tests/std_widthcast.rs`.
  DoD = the Proposed → Accepted gate (maintainer ratifies) + the M-717 handoff. **Spec stays
  Proposed; code is "Rust-first, pending ratification" — never silently Accepted/Enacted.** CHANGELOG
  / Doc-Index / issues.yaml / docs/api-index owned by the integrating parent. (Append-only; VR-5; G2.)
