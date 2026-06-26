# Design Note DN-43 — Surfacing `bytes.slice` / `bytes.concat` to the `.myc` Surface

| Field | Value |
|---|---|
| **Note** | DN-43 |
| **Status** | **Proposed** (2026-06-26) — surfaces two **already-landed** kernel prims, `bytes.slice` / `bytes.concat` (RFC-0032 D4, M-750), to the `.myc` surface as `bytes_slice(b, start, end)` / `bytes_concat(b1, b2)`. This is the missing **surface mapping** that wave-n1/wave-n2 flagged as **FLAG-text-3**: the byte prims exist and are never-silent in the kernel, but `prim_kernel_name`/`try_check_seq_bytes_prim` never mapped a surface name onto them, so a `.myc` program could not call them. **Maintainer ratifies → Accepted** (house rule #3); this note **proposes** the surfacing and records the Rust-first implementation that lands with it ("implemented (Rust-first), pending ratification" — never silently `Accepted`/`Enacted`). |
| **Feeds** | **M-717** (E13-1 text/fmt: self-hosted UTF-8) — closes the **last** Definition-of-Done remainder: a `Bytes`-native slice/concat over the kernel `Bytes`, replacing the deferred `Bytes8` byte-cons-list workaround. **M-799** (E19-1 follow-on — the surfacing task). Extends **RFC-0032 D4** (the byte-string kernel prims) by giving its `slice`/`concat` members a surface, exactly as **DN-41 / M-798** surfaced `width_cast` for D1/D2. |
| **Date** | June 26, 2026 |
| **Decides** | *Proposes, for ratification:* (1) the **surface names** `bytes_slice` (→ kernel `bytes.slice`) and `bytes_concat` (→ kernel `bytes.concat`); (2) the **static signatures** `bytes_slice(b: Bytes, start: Binary{W}, end: Binary{W}) -> Bytes` and `bytes_concat(b1: Bytes, b2: Bytes) -> Bytes`, type-checked in the same dedicated `Seq`/`Bytes` branch as `bytes_get`/`bytes_len` (a non-`Bytes` receiver, a non-`Binary{W}` index, or a wrong arity is an explicit never-silent static refusal — G2); (3) the **honest per-op guarantee tags** (`concat` `Exact`/total; `slice` `Exact` over the in-range domain, with the out-of-range/inverted-range **contract** `Declared`/never-silent); (4) the **never-silent contract** — an out-of-range or inverted `[start, end)` slice is an explicit runtime refusal that the `.myc` surface lifts into an `Option`/`Result`, **never** a silent clamp or truncation (G2/VR-5); (5) **EXPLAIN-ability** (the prims are already reified in the registry); and (6) the **placement** (`mycelium-l1` checker surface only — the kernel prims and their registry registration already exist in `mycelium-interp`). |
| **Task** | M-799 (E19-1 `kpr` follow-on — the byte slice/concat surface; design + Rust-first surfacing) |

> **Posture (transparency rule / VR-5 / G2).** This note is **Proposed** — a design direction for the
> maintainer to ratify. It **does not** move any decision to `Accepted`/`Enacted` on its own authority,
> and it **upgrades no guarantee past its basis**. Crucially, this is a **surfacing** decision, not a
> new-prim design: the `bytes.slice` / `bytes.concat` kernel prims were **already designed,
> implemented, registered, and never-silent-tested** under RFC-0032 D4 / M-750 — see
> `crates/mycelium-interp/src/prims.rs` (`prim_bytes_slice`, `prim_bytes_concat`, registered in the
> builtin `PrimRegistry`). There is therefore **less to decide here than for a new prim** (DN-41 had to
> design `width_cast` from scratch; this note only wires an existing, tested prim to a callable surface
> name). The Rust-first implementation that accompanies this note is "implemented (Rust-first), pending
> ratification" — the surface is **landed and tested**; the *spec status* stays **Proposed** until the
> maintainer ratifies.

---

## §1 Purpose & the gap (FLAG-text-3)

Wave-n1 ported `lib/std/text.myc` (self-hosted UTF-8) over the RFC-0032 D4 byte prims. Wave-n2-p2
(DN-41 / M-798) surfaced `width_cast` and closed FLAG-text-1 (`byte_at`) and FLAG-text-2 (multi-byte
`decode_one`). The **one** remaining flagged gap was **FLAG-text-3**:

> `bytes_slice` / `bytes_concat` are STILL not surface-callable — the `width_cast` prim does NOT unblock
> these (they need their own surface). The `Bytes8` byte-cons model stays declared-only; slice/concat
> ops remain deferred.

The diagnosis in that FLAG was **partially** off, and this note corrects it honestly (VR-5): the gap is
**not** a missing kernel prim. The kernel prims `bytes.slice` and `bytes.concat` **already exist**,
are **already registered** in the builtin prim registry, and are **already never-silent** (an
out-of-range or inverted slice is an explicit `EvalError::PrimType` refusal — `prims.rs::prim_bytes_slice`,
lines ~728–753; `prim_bytes_concat`, lines ~755–770). What was missing is only the **surface mapping** —
two entries in `prim_kernel_name` plus two type-check arms in `try_check_seq_bytes_prim` — exactly the
shape DN-41 added for `width_cast`. This note supplies that mapping; no kernel change is needed.

## §2 The surface — names, signatures & semantics

```
bytes_slice(b: Bytes, start: Binary{W}, end: Binary{W}) -> Bytes   // → kernel bytes.slice
bytes_concat(b1: Bytes, b2: Bytes) -> Bytes                        // → kernel bytes.concat
```

(The surface names are `_`-joined — a `.` is the lexer's path separator — matching `bytes_get` /
`bytes_len` / `seq_get` / `seq_len`.)

| Op | Semantics | Guarantee |
|---|---|---|
| **`bytes_concat`** | the byte-wise concatenation `b1 ++ b2` | **`Exact`** — total, lossless; the byte sequence is exactly the two operands' bytes in order |
| **`bytes_slice`** | the sub-slice `b[start .. end)` (half-open), `start`/`end` unsigned `Binary{W}` byte offsets | **`Exact`** over the in-range domain (`start ≤ end ≤ len`); a range that is **out of bounds or inverted** is a **never-silent refusal**, never a silent clamp/truncation |

The index width `W` is any `Binary{W}` (an unsigned magnitude), matching `bytes_get`. The kernel
`as_index` decodes it MSB-first and refuses a width exceeding the `usize` index space (defense-in-depth;
`prims.rs::as_index`).

## §3 The never-silent contract (G2 / VR-5)

`bytes_slice`'s in-bounds predicate is `start ≤ end ≤ len`. The kernel enforces it **explicitly**:

```rust
if start > end || end > bytes.len() {
    return Err(EvalError::PrimType { /* "slice range [{start}, {end}) is out of bounds or inverted …" */ });
}
```

— an **inverted** range (`start > end`) and an **out-of-bounds** range (`end > len`) are *both* explicit
refusals. There is **no silent clamp** to `[min(start,len) .. min(end,len))` and **no truncation** to the
empty slice; the offending range is reported (the message carries `start`, `end`, and `len`). The `.myc`
surface lifts this kernel refusal into the never-silent `Option`/`Result` the text module already uses
for `byte_at` (the same pattern that makes `bytes_get`'s out-of-bounds refusal honest at the surface).

`bytes_concat` is **total** — every pair of `Bytes` concatenates — so it has no failure mode and no
`Option` lift; its guarantee is `Exact`.

This refusal is exhibited **on all three execution paths** (L1-eval ≡ L0-interp ≡ AOT) by the
conformance test `crates/mycelium-l1/tests/std_bytes_slice.rs`, mirroring the `width_cast`
narrowing-overflow refusal test (DN-41 / `std_widthcast.rs`).

## §4 The `.myc` surface (`lib/std/text.myc`)

The text nodule gains two functions over the kernel `Bytes`, **replacing** the deferred `Bytes8`
byte-cons-list workaround that FLAG-text-3 described:

```
fn slice(b: Bytes, start: Binary{32}, end: Binary{32}) -> Bytes = bytes_slice(b, start, end)
fn concat(b1: Bytes, b2: Bytes) -> Bytes = bytes_concat(b1, b2)
```

- `concat` is a thin total wrapper over the `Exact` kernel prim (`Exact`).
- `slice` delegates to the never-silent kernel prim. The thin form above returns `Bytes` directly (the
  kernel refusal surfaces as a never-silent evaluation error on out-of-range — never a silent clamp).
  A bounds-checked `Option<Bytes>` form (`Some` in range, `None` out of range — the analog of `byte_at`)
  is the honest surface a *caller* uses when it wants the refusal as data rather than an error; the text
  module documents both. The guarantee is `Exact` over the in-range domain; the bounds contract is
  `Declared`/never-silent.

The `Bytes8` cons-list type that wave-n1 declared as a *future* slice/concat model is no longer the
mechanism (it was a declared-only workaround); `slice`/`concat` now operate on the **kernel** `Bytes`
directly. (The `Bytes8` declaration may be retired in a later cleanup; this note does not delete it, to
keep the change minimal and the decision append-only.)

## §5 Why this closes M-717's slicing DoD clause

M-717's Definition of Done requires "Never-silent encoding/slicing errors (Result); honest tags" and a
`.myc` text module that handles "slices, concatenation". Its body records that, after wave-n2-p2, **"the
slicing DoD clause is the only remainder"** (the encoding DoD was already MET; FLAG-text-1/2 CLOSED).
Surfacing `bytes_slice`/`bytes_concat` and adding `slice`/`concat` to `text.myc` — with the out-of-range
refusal exhibited three-way — satisfies that last clause:

- **slices**: `slice(b, start, end)` over the kernel `Bytes`, never-silent on out-of-range (G2);
- **concatenation**: `concat(b1, b2)` over the kernel `Bytes`, `Exact`;
- **honest tags**: `concat` `Exact`; `slice` `Exact` in-range + `Declared`/never-silent bounds contract.

With this landed, M-717's only open items are the **UTF-8 validity layer** (rejecting overlong /
surrogate / `> U+10FFFF` encodings — a *separate* flagged increment, not a slicing concern) — so the
**slicing** DoD clause specifically is **MET**, and **FLAG-text-3 is closed**.

> The M-717 status transition (in `tools/github/issues.yaml`) is **orchestrator-owned** — this leaf
> proposes the surfacing and lands the code/tests; the integrating parent records the M-717 body update
> and any status move (append-only). FLAG up, do not edit (house rule: parent-owned files read-only).

## §6 Definition of Done (this note)

- **DN-43 written** (this file), Status **Proposed**, honestly framed as a *surfacing* (not a new prim).
- **Surface mapping landed** (Rust-first): `prim_kernel_name` maps `bytes_slice`/`bytes_concat`; the
  checker types them in `try_check_seq_bytes_prim` (`bytes_slice`: `(Bytes, Binary{W}, Binary{W}) -> Bytes`;
  `bytes_concat`: `(Bytes, Bytes) -> Bytes`), with arity/receiver/index refusals (G2).
- **`.myc` surface landed**: `slice`/`concat` in `lib/std/text.myc` over the kernel `Bytes`.
- **Three-way conformance** (`std_bytes_slice.rs`): in-range slice (value preserved), out-of-range slice
  → never-silent refusal on **all three** paths (L1-eval, L0-interp, AOT), and `concat` of two `0x…`
  literals — all green.
- **`just check` green** (Rust-first; spec **Proposed**, pending maintainer ratification).

---

## Changelog
- 2026-06-26 — **Proposed** (M-799 / DN-43). Surfaces the already-landed RFC-0032 D4 `bytes.slice` /
  `bytes.concat` kernel prims to `.myc` as `bytes_slice` / `bytes_concat`; adds `slice`/`concat` to
  `lib/std/text.myc`; closes FLAG-text-3 and M-717's slicing DoD clause (Rust-first, spec Proposed).
