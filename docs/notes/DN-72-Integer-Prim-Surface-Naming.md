# Design Note DN-72 — Integer-Prim Surface Naming: the `_u`/`_s` Signedness Convention

| Field | Value |
|---|---|
| **Note** | DN-72 |
| **Status** | **Accepted** (2026-07-02 — maintainer ratified "clean up now" with the explicit `_u`/`_s` signedness convention; enacted in the same change that authors this note) |
| **Decides** | Every integer-arithmetic **surface** prim name carries an explicit signedness suffix: **`_u`** = unsigned semantics, **`_s`** = signed/two's-complement semantics. The historical mixed `_bin`/`_tc` suffixes are replaced atomically (§3). **Kernel** prim names (`bit.*`/`bin.*`) are **not** renamed — they are content-addressed (DN-10 §3.4); their namespace inconsistency is a deferred FLAG (§5). |
| **Feeds** | Future signed-variant surfacing (M-767: `div_s`/`rem_s`/`shr_s` slot cleanly into the convention); `.claude/memory/lang-lexicon-syntax.md`; docs/spec surface-name references |
| **Depends on** | ADR-028 (Binary is sign-free; **signedness is operations** — signedness-dependent ops are distinct named ops); RFC-0033 §4.1.2/§4.1.3 (the signedness-split requirement the old names implemented); RFC-0032 D2 (the original `add_bin`/`sub_bin`); DN-10 §3.4 (content-addressed kernel prim identity — why kernel names stay) |
| **Date** | 2026-07-02 |
| **Task** | Ratified integer-prim surface-naming cleanup (serial lane over `mycelium-l1/src/checkty.rs` plus `lib/std/*.myc`) |

> **Posture (transparency rule / VR-5 / G2).** The rename map (§3) and the "surface only, kernel
> unchanged" scope (§4) are `Exact` — read directly from `checkty.rs::prim_kernel_name` and
> verified by the full `mycelium-l1` test suite green after the atomic switch. The claim that the
> new names make signedness *predictable for users* is `Declared` (design intent, the ratified
> rationale). The kernel-namespace inconsistency (§5) is `Exact` (source-read) and its resolution
> is deliberately **deferred**, not silently decided (G2).

---

## 1. Problem — the accumulated suffix mix

The integer surface prims accumulated three naming patterns as they landed
(M-748 → M-887 → M-888 → M-889 → M-766):

- `add_bin`/`sub_bin` — **unsigned** (kernel `bit.add`/`bit.sub`, RFC-0032 D2);
- `mul_bin`/`div_bin`/`rem_bin`/`shl_bin`/`shr_bin` — `_bin` again, but **mixed** signedness
  (`mul_bin` is signed/two's-complement per RFC-0033 §4.1.3; `div_bin`/`rem_bin` are unsigned per
  §4.1.2; the shifts are logical/unsigned);
- `add_tc`/`sub_tc`/`neg_bin` — signed/two's-complement, forced to `_tc` because `add_bin`/`sub_bin`
  were already claimed by the unsigned pair (the `checkty::prim_family` naming FLAG, now resolved).

So the suffix carried **no reliable signedness signal**: `_bin` meant "unsigned" in five names,
"signed" in two, and `_tc` existed only to dodge a collision. ADR-028's whole point — signedness
lives in the *operation*, so the operation's *name* is where a user reads it — was undermined at
the surface.

## 2. Decision — explicit `_u`/`_s` suffixes (ratified 2026-07-02)

The maintainer ratified the cleanup: every integer-arithmetic surface prim name ends in **`_u`**
(unsigned semantics) or **`_s`** (signed/two's-complement semantics). This is ADR-028
signedness-as-operations made *legible*: the signedness split that ADR-028 mandates as distinct
named ops is now readable from the name itself, uniformly.

Width-agnostic bit-logical prims (`not`/`xor`/`and`/`or`) and the reduce-to-`Bool` comparisons
(`eq`/`lt`) have no signedness reading and keep their unsuffixed names. The trit-backed
balanced-ternary `add`/`sub`/`mul`/`neg` are untouched (balanced ternary is inherently signed and
symmetric; no `_u`/`_s` split exists to encode).

## 3. The rename map (`Exact` — `checkty.rs::prim_kernel_name`)

| old surface | new surface | kernel (unchanged) | signedness |
|---|---|---|---|
| `add_bin` | `add_u` | `bit.add` | unsigned |
| `sub_bin` | `sub_u` | `bit.sub` | unsigned |
| `div_bin` | `div_u` | `bin.div` | unsigned |
| `rem_bin` | `rem_u` | `bin.rem` | unsigned |
| `shl_bin` | `shl_u` | `bin.shl` | unsigned (logical) |
| `shr_bin` | `shr_u` | `bin.shr` | unsigned (logical) |
| `mul_bin` | `mul_s` | `bin.mul` | signed (two's-complement) |
| `add_tc` | `add_s` | `bin.add` | signed (two's-complement) |
| `sub_tc` | `sub_s` | `bin.sub` | signed (two's-complement) |
| `neg_bin` | `neg_s` | `bin.neg` | signed (two's-complement) |

The switch is **atomic**: the `checkty.rs` surface→kernel map, the `PrimFam`/`prim_sig` match
arms, the `mycelium-l1` conformance/enablement/differential tests, all `lib/std/*.myc` nodules
(13), and the `mycelium-cli` multi-nodule run fixture changed in one commit, with the full
`mycelium-l1` suite green (a missed rename is a failing test — the old names no longer resolve).

The signed variants still to be surfaced (M-767: signed division/remainder, arithmetic right
shift) land as `div_s`/`rem_s`/`shr_s` — the convention pre-assigns their names with no future
collision.

## 4. Scope — surface names only; kernel names deliberately unchanged

Kernel prim names (`bit.*`/`bin.*` in the interpreter's registry / `PrimTable::builtins()`/`Π`)
are **content-addressed identities** (DN-10 §3.4): renaming them would churn prim identities that
landed `.myc` differentials may pin. The user-facing win — predictable signedness at the point of
use — lives entirely at the surface layer, so the surface is what changed.

## 5. Deferred FLAG — the kernel-namespace inconsistency (content-address-impacting)

**FLAG (deferred, maintainer decision required before any kernel rename).** The `bit.*`/`bin.*`
kernel split does **not** cleanly encode signedness: `bin.div`/`bin.rem`/`bin.shl`/`bin.shr` are
*unsigned* while `bin.mul`/`bin.add`/`bin.sub`/`bin.neg` are *signed*, and the unsigned surface
`add_u`/`sub_u` map to `bit.add`/`bit.sub` while the equally-unsigned `div_u`/`rem_u` map to
`bin.div`/`bin.rem`. Reconciling the kernel namespace (e.g. a `bin.udiv`/`bin.sdiv`-style split, or
folding `bit.add`/`bit.sub` into a uniform scheme) is a **content-address-impacting** change
(DN-10 §3.4) — it must be its own decided, migration-planned change, not a rider on a surface
cleanup. Until then the `checkty.rs` map comments and this note are the record of the mismatch
(never silent — G2).

## 6. Definition of Done

- [x] The §3 map applied atomically across `checkty.rs`, the `mycelium-l1` tests, all 13
  `lib/std/*.myc` nodules, and the CLI multi-nodule fixture; zero old-name occurrences remain.
- [x] `cargo test -p mycelium-l1` fully green post-rename (1052 passed, 0 failed);
  `cargo test -p mycelium-cli` green (26 passed).
- [x] `mycfmt --check` clean on all 13 edited `lib/std/*.myc` nodules.
- [x] Kernel prim names verified byte-identical (no `mycelium-core`/`mycelium-interp` changes).
- [x] The kernel-namespace FLAG recorded (§5) rather than silently resolved.
- [ ] Integrator: remaining old-name mentions reconciled. Verified zero hits in
  `.claude/memory/` and `docs/spec/`; the only survivors are DN-41 / DN-42 / DN-52 / RFC-0032
  plus the archived 2026-06 changelog — historical records of decided docs, which stay as
  history (append-only); forward-notes may be added if desired, never rewrites.

---

## Changelog

| Date | Status | Note |
|---|---|---|
| 2026-07-02 | **Accepted** | Initial record. Maintainer ratified the `_u`/`_s` surface-naming convention (ADR-028 signedness-as-operations); rename enacted atomically, kernel names unchanged, kernel-namespace reconciliation deferred (§5 FLAG). |
