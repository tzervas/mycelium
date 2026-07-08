# Spec — `std.cmp` / `convert` (ordering/equality + non-representation value conversions)

| Field | Value |
|---|---|
| **Status** | **Accepted** (2026-06-20, maintainer-ratified per DN-07 — guarantee matrix asserted in tests; the landed `MycEq`/`MycOrd`/`MycPartialOrd` trait naming is documented in §3 (a parity note, not honesty); open §7 questions are design/scope calls, not contract violations; was *Implemented (Rust-first) — pending ratification* 2026-06-18, Draft/needs-design 2026-06-17) — the Rust-first code landed as `mycelium-std-cmp` (M-532, #172, Batch P5-B). The Mycelium-lang migration (M-502-gated) remains. |
| **Module / Ring** | `std.cmp` / `convert` · Ring `2` (RFC-0016 §4.2) · Tier `B` |
| **Tracks** | `M-532` (#172) — the Phase-5 task this spec delivers (RFC-0016 §4.4 `cmp`/`convert` row) |
| **Scope** | The ordering/equality traits (`eq`, `ord`, partial/total order, `min`/`max`/`clamp`/`sort`-key) and **ordinary value conversions** — lossless widening (e.g. `i8 → i32`) and explicitly-fallible narrowing (e.g. `i32 → i8`). These are *value* conversions: same representation paradigm, value re-typed within it. |
| **Boundary** | A **representation change** (binary↔ternary, `F32→BF16`, Dense↔VSA) is **`std.swap` (M-516)** — certificate-carrying and visible (RFC-0002), **not** a `convert`. `convert` never crosses `Repr` paradigms and never emits a swap certificate; `swap` is the only door across that line. |
| **Depends on** | RFC-0016 §4.1 (the C1–C6 contract) and §4.4 (the `cmp`/`convert` row); RFC-0001 (the value model — `Value`/`Repr`/`Meta`, the guarantee lattice, content-addressing §4.6); RFC-0002 (swap certificates — the boundary's other side); ADR-003 (content-addressed identity — what `eq` respects). |
| **Grounds on** | Ring-0 `core` (`Option`/`Result`/error values, the lattice tags; M-515) and the kernel value model; no new trusted code (KC-3). |

---

## 1. Summary

`std.cmp` / `convert` provides the ordinary ordering, equality, and value-conversion surface every program needs: `eq`/`ord` traits, derived helpers (`min`, `max`, `clamp`, sort keys), and value conversions between scalar/value types. Its **honesty crux** is the structural one RFC-0016 §4.4 names: a **lossy / narrowing conversion is an explicit fallible `Result`, never a silent narrowing or truncation** (C1/G2) — `i32 → i8` that does not fit is `Err`, not a wrapped or clamped byte. Equality is the second crux: `eq` respects **content-addressed identity where it applies** (ADR-003) — equal content is equal, and metadata is **not** identity. The module is **Ring 2**, written to the contract over Ring 0/1; it adds **no trusted code** (KC-3) — it consumes the kernel value model and `core` re-exports only.

## 2. Scope & module boundary

- **In scope:** equality (`eq`/`ne`) and ordering (`partial_cmp`/`cmp`, `lt`/`le`/`gt`/`ge`) traits and their derived helpers (`min`, `max`, `clamp`, comparison keys for `std.collections` sorted variants); **value conversions** — lossless **widening** (the value's domain is a subset of the target's: `i8 → i32`, `u8 → u16`, `bool → i32`) and explicitly-fallible **narrowing** (the value may not fit: `i32 → i8`, `i64 → i32`, `usize → u32`).
- **Out of scope (and who owns it):**
  - **Representation changes — `std.swap` (M-516).** Any conversion that changes the `Repr` *paradigm* (binary↔ternary, dense `F32→BF16`, Dense↔VSA) is a **swap**: visible, certificate-carrying, with a per-instance bound (RFC-0002 §2–§5). `convert` deliberately does **not** offer these; offering them under a `convert` name would hide a certified op behind an ordinary one — exactly the silent default C1 forbids. **The boundary clause:** *`convert` re-types a value within one representation paradigm and carries no certificate; `swap` crosses paradigms and must carry a `SwapCertificate` (RFC-0002). A `convert` is never a swap, and a swap is never offered as a `convert`* (grounding: RFC-0002 §1 "turns RFC-0001's `Swap` node into a verifiable operation"; RFC-0016 §4.4 — "non-representation conversions (distinct from `swap`)"; M-516 owns the swap surface).
  - **Parsing / text decoding — `std.text` (M-524).** `str → i32` is a `parse` (`Result`, RFC-0016 §4.4 text row), not a numeric `convert`.
  - **Hashing-for-maps vs. content identity — `std.content` (M-523).** `cmp` *consumes* the ADR-003 identity relation for `eq`; it does not define the content-addressing primitives (M-523 owns those).
  - **Rounding/approximation math — `std.math` (M-525).** A conversion that *rounds* (e.g. `f64 → i32` truncation toward zero) carries a guarantee tag and is fallible on domain (`NaN`/overflow); the rounding *functions* themselves live in `math`.
- **Ring & layering:** Ring 2 (RFC-0016 §4.2). It **re-exports** the lattice/`Result` types from Ring-0 `core`, **wraps** nothing trusted, and **builds new** only the trait surface + total/fallible conversion functions written over the kernel value model. KC-3: no enlargement of the trusted base — `convert` produces ordinary values, never a certificate, so it adds nothing the certificate checker must trust.

## 3. Exported-op surface (design sketch)

A value-semantic, immutable-by-default surface. Total ops return their value directly; fallible ops return `Result`. No effects. **Illustrative — not a committed grammar.**

```
// illustrative signatures (not a committed surface; trait NAMES match the landed crate)

// --- equality / ordering traits (landed as MycEq / MycOrd / MycPartialOrd) ---
trait MycEq         { fn eq(&self, other: &Self) -> bool }        // total; respects ADR-003 identity
trait MycOrd        { fn cmp(&self, other: &Self) -> Ordering }   // total order
trait MycPartialOrd { fn partial_cmp(&self, other: &Self) -> Option<Ordering> } // partial (e.g. floats: NaN => None)

enum Ordering { Less, Equal, Greater }

fn min<T: MycOrd>(a: T, b: T) -> T
fn max<T: MycOrd>(a: T, b: T) -> T
fn clamp<T: MycOrd>(x: T, lo: T, hi: T) -> Result<T, ClampError>  // lo>hi is an explicit error, not a silent swap

// --- value conversions (NOT representation swaps) ---
trait Widen<To>  { fn widen(self) -> To }                       // lossless, total: domain subset of codomain
trait Narrow<To> { fn narrow(self) -> Result<To, NarrowError> } // fallible: may not fit

enum NarrowError { OutOfRange { value, target_min, target_max }, NotRepresentable { reason } }
enum ClampError  { InvertedBounds { lo, hi } }
```

**Naming — the `Myc` prefix (landed-surface parity).** The equality/ordering traits export as `MycEq` / `MycOrd` / `MycPartialOrd` (not `Eq` / `Ord` / `PartialOrd`) in the Rust-first crate `mycelium-std-cmp`: the prefix avoids a namespace collision with Rust's own `std::cmp::{Eq, Ord, PartialOrd}` (the crate cannot re-use those names without shadowing), per the RFC-0016 §8-Q2 naming lexicon. The *contract* is the trait semantics above (reflexive/total equality respecting ADR-003 identity; total vs partial order), not the spelling; the Mycelium-lang surface (post-M-502) may drop the prefix. This is a documentation-parity note, **not** an honesty matter — no guarantee tag is affected.

The `Widen`/`Narrow` split is the surface form of the honesty crux: widening is *structurally* total (the type system witnesses `domain ⊆ codomain`), so it has no error arm; narrowing is *structurally* fallible (the value may not fit), so its result type **is** a `Result` — a caller cannot narrow without handling the out-of-range arm (C1).

### 3.1. Width-generic comparison helpers (self-hosted Mycelium-lang, `lib/std/cmp.myc`)

A *distinct layer* from the trait surface above: `lib/std/cmp.myc` (RFC-0031 §5 D4 Tier-1) exports **width-generic, total** comparison operations over `Binary{N}`, one definition per op, monomorphized to the call-site width (M-753 / DN-42). All are `Exact` over the finite `Binary{N}` domain. Differential agreement (L1-eval ≡ L0-interp ≡ AOT) is `Empirical` (trials, `std_cmp.rs`). **Never-silent (G2):** width-mismatch refusals are explicit (a mixed-width compare fails, not coerced).

```
// Unsigned width-generic comparisons (RFC-0033 §4.1.2 basis: `eq`/`lt` kernel prims)
fn cmp{N}(a: Binary{N}, b: Binary{N}) => Ordering        // three-way unsigned
fn le{N}(a: Binary{N}, b: Binary{N}) => Bool             // unsigned <=
fn ge{N}(a: Binary{N}, b: Binary{N}) => Bool             // unsigned >=
fn max{N}(a: Binary{N}, b: Binary{N}) => Binary{N}       // unsigned max
fn min{N}(a: Binary{N}, b: Binary{N}) => Binary{N}       // unsigned min

// CU-4 completeness surface (RFC-0033 §4.1.2 / RFC-0032 D1)
fn ne{N}(a: Binary{N}, b: Binary{N}) => Bool             // negation of `eq`
fn gt{N}(a: Binary{N}, b: Binary{N}) => Bool             // unsigned >

// Signed two's-complement comparisons (via `lt_s` prim + `eq`)
fn cmp_s{N}(a: Binary{N}, b: Binary{N}) => Ordering      // three-way signed
fn le_s{N}(a: Binary{N}, b: Binary{N}) => Bool           // signed <=
fn ge_s{N}(a: Binary{N}, b: Binary{N}) => Bool           // signed >=
```

## 4. Guarantee matrix (the load-bearing deliverable — RFC-0016 §4.5)

Rows = exported ops. Columns = `{ guarantee tag · fallibility (explicit error set) · declared effects · EXPLAIN-able? }`. Encoded as a checked table (the RFC-0003 §4 template), asserted in tests once code lands — never prose only.

| Op | Guarantee tag | Fallibility (explicit error set) | Declared effects | EXPLAIN-able? |
|---|---|---|---|---|
| `eq` / `ne` | `Exact` | total (`bool`) | none | n/a |
| `cmp` (total order) | `Exact` | total (`Ordering`) | none | n/a |
| `partial_cmp` | `Exact` | total — `None` is the *defined* incomparable result (e.g. `NaN`), not a failure | none | n/a |
| `lt` / `le` / `gt` / `ge` | `Exact` | total (`bool`) | none | n/a |
| `min` / `max` | `Exact` | total | none | n/a |
| `clamp` | `Exact` | `Err(ClampError::InvertedBounds)` when `lo > hi` — never a silent reorder | none | n/a |
| `widen` (lossless, e.g. `i8 → i32`) | `Exact` | **total** — domain ⊆ codomain, no error arm | none | n/a |
| `narrow` (fallible, e.g. `i32 → i8`) | `Exact` *(exact when it returns `Ok`)* | `Err(NarrowError::OutOfRange { value, target_min, target_max })` — **never a silent truncation/wrap** | none | yes (the `NarrowError` carries the rejected value + bounds — a reified, inspectable diagnostic) |
| `narrow` not-representable (e.g. `f64 → i32` on `NaN`/`±∞`/overflow) | `Exact` *(when `Ok`)* | `Err(NarrowError::NotRepresentable { reason })` | none | yes (reason record) |
| `ne{N}` (negation of `eq`) | `Exact` | total (`bool`) | none | n/a |
| `gt{N}` (unsigned, projection of `cmp`) | `Exact` | total (`bool`) | none | n/a |
| `cmp_s{N}` (signed three-way) | `Exact` | total (`Ordering`) | none | n/a |
| `le_s{N}` (signed comparison) | `Exact` | total (`bool`) | none | n/a |
| `ge_s{N}` (signed comparison) | `Exact` | total (`bool`) | none | n/a |

**Tag justification.** Every row is `Exact` — `cmp`/`convert` carries **no** accuracy/precision/probability semantics, so VR-5/C2 makes it `Exact` rather than any downgraded tag (RFC-0016 §4.1 C2: "an op with no accuracy semantics … is simply `Exact`"). The honesty here is **not** an approximation tag; it is the **fallibility column**: `widen` is total *because* it is lossless, and `narrow` is `Result` *because* it may lose information — the explicit narrowing-error set (`OutOfRange`, `NotRepresentable`) is the never-silent guarantee (C1/G2). The `EXPLAIN-able?` column is `yes` precisely where a conversion can *reject* (the narrowing rows): the error value is the reified artifact saying *why* (C3), not an opaque sentinel. Comparison ops need no EXPLAIN record — they neither select, convert, nor approximate (C3 trigger absent). Note the deliberate contrast with a **swap** (out of scope, M-516/RFC-0002): a lossy swap's row would tag the *bound* (`Proven`/`Empirical`/`Declared`) and be `EXPLAIN`-able via a `SwapCertificate`; a `convert` has neither — which is exactly why a representation change cannot live in this module.

## 5. §4.1 contract conformance (C1–C6)

- **C1 — never-silent (G2):** A narrowing conversion that cannot represent its input returns `Err(NarrowError)` carrying the rejected value and the target bounds — never a wrap, clamp, sign-flip, or truncation. `clamp` with inverted bounds is `Err`, not a silent swap of `lo`/`hi`. Widening is total *only because* it is provably lossless (no information is dropped). This is the module's whole reason to split `Widen`/`Narrow` at the type level (RFC-0016 §4.4 — "a lossy `convert` is an explicit fallible op, never a silent narrowing").
- **C2 — honest per-op tag (VR-5):** Every op is `Exact` and the matrix says so — `cmp`/`convert` carries no accuracy semantics, so there is nothing to downgrade; the honesty load is carried by the fallibility column, not a probabilistic tag. No op is tagged `Proven`/`Empirical`/`Declared` (none claims a bound).
- **C3 — no black boxes / EXPLAIN (SC-3/G11):** The narrowing rows are `EXPLAIN`-able: a `NarrowError` is a reified, inspectable diagnostic (the value, the target range or the not-representable reason), so a rejected conversion explains *why* it was rejected. Comparison ops do not select/convert/approximate, so they carry no EXPLAIN obligation. A `convert` never emits a swap certificate — that artifact belongs to `swap` (RFC-0002), which is the boundary this spec defends.
- **C4 — content-addressed, value-semantic (ADR-003 / RFC-0001):** `eq` respects **content-addressed identity where it applies**: two values with equal content are equal, and **metadata is not identity** (ADR-003; Foundation §5.1 — "content-addressing … names-as-metadata"). All ops are pure functions of their inputs (no mutation, no hidden state); conversions return new values, leaving inputs untouched (immutable-by-default). `cmp` *consumes* the ADR-003 identity relation; it does not redefine it (that is `std.content`, M-523).
- **C5 — above the small kernel (KC-3):** Ring 2. The module consumes the kernel value model and Ring-0 `core` re-exports; it introduces **no** trusted code, no `wild`/FFI (pure value logic only), and — critically — produces **no certificate**, so it adds nothing the certificate checker (RFC-0002 §2) must trust. The trusted base is unchanged.
- **C6 — declared, bounded effects (RFC-0014):** Every op is **effect-free** (the "Declared effects" column is `none` throughout): comparison and value conversion are pure functions. No IO, time, randomness, or unbounded allocation. Nothing to declare or budget.

## 6. Grounding

- The **contract** (C1–C6) and the `cmp`/`convert` row: RFC-0016 §4.1 and §4.4 (the honesty crux "a lossy `convert` is an explicit fallible op, never a silent narrowing"). The guarantee-matrix obligation: RFC-0016 §4.5 (the RFC-0003 §4 template).
- The **value model** (`Value`/`Repr`/`Meta`, `Option`/`Result`, the guarantee lattice, content-addressing §4.6): RFC-0001, via Ring-0 `core` (M-515).
- The **boundary** between `convert` and `swap`: RFC-0002 §1–§5 (the `SwapCertificate`, the legal `(R_src → R_target)` pairs, "per-swap (not once-for-all) validation"), owned by **M-516** (`std.swap`). The named representation pairs that are swaps-not-converts — Binary↔Ternary, Dense `F32→BF16`, Dense↔VSA — are RFC-0002 §5's legal-pair table.
- **Equality / identity:** ADR-003 (content-addressing; Foundation §5.1, §5.5 — content identity, names/metadata as non-identity; "formatting is a *projection*, not a mutation of identity").
- **Never-silent:** G2 (the foundational never-silent guarantee) via C1.

## 7. Open questions (FLAGGED — resolve before ratification)

- **(Q1) Float ordering: `PartialOrd` only, or an opt-in total order?** IEEE floats are not totally ordered (`NaN`). The sketch makes floats `PartialOrd` (`NaN => None`), honest by default, but sorted collections (`std.collections`, M-511) need a total key. Disposition: **propose** a separate opt-in `total_cmp` (NaN-bucketed, a *named* total order) rather than silently imposing one — but the exact policy is cross-module with M-511 and ties to RFC-0016 §8-Q3 (ergonomics vs. explicitness). FLAGGED.
- **(Q2) Is `f64 → i32` (rounding) a `convert` or a `math` op?** It both *narrows* (fallible: `NaN`/overflow) and *rounds* (a `math` rounding-mode concern). The sketch places the fallible narrowing here and the rounding-mode choice in `std.math` (M-525); the seam between the two needs maintainer sign-off so the rounding tag is not double-owned. FLAGGED — coordinate with M-525.
- **(Q3) Does `eq` over a lossy-swappable pair ever make sense?** Two values in *different* representation paradigms are not `eq`-comparable here (that would smuggle a swap into `cmp`); comparing them requires an explicit `swap` first (RFC-0002), then `eq`. Confirm this is the intended discipline (it follows from the boundary clause) and that no implicit cross-paradigm `eq` is exposed. Disposition: **assert the discipline, FLAG for ratification** that no cross-`Repr` `eq` overload is offered. Ties to RFC-0016 §8-Q1 (the v0 surface).

## Meta — changelog

- **2026-06-17 — Draft (needs-design).** Stands up the `std.cmp` / `convert` module spec under **RFC-0016 (Draft)**, M-532 (#172): the ordering/equality trait surface (`eq`/`cmp`/`partial_cmp` + `min`/`max`/`clamp`) and the **non-representation value-conversion** surface (lossless `widen`, explicitly-fallible `narrow`). The **honesty crux** is the never-silent narrowing (C1/G2) — a lossy conversion is an explicit `Result` (`NarrowError::{OutOfRange, NotRepresentable}`), never a silent truncation/wrap — and `eq` respects content-addressed identity (ADR-003, metadata-is-not-identity). The **boundary clause** is load-bearing: a representation change (binary↔ternary, `F32→BF16`, Dense↔VSA) is a certificate-carrying `std.swap` (M-516 / RFC-0002), **not** a `convert`; this module crosses no `Repr` paradigm and emits no certificate. The guarantee matrix (RFC-0016 §4.5) carries nine rows, all `Exact` (no accuracy semantics — the honesty is in the fallibility column), with the narrowing rows `EXPLAIN`-able via their reified error. §4.1 conformance (C1–C6) stated per clause; three questions FLAGGED (float total order vs. partial, the `convert`/`math` rounding seam, cross-paradigm `eq`), tied to RFC-0016 §8-Q1/Q3. No code; no kernel change (KC-3, Ring 2, no trusted base growth). Append-only.
- **2026-06-20 — Accepted (maintainer ratification, DN-07).** The maintainer ratified this Rust-first spec: the §4.5 guarantee matrix (nine `Exact` rows) is asserted in tests, never-silent narrowing (`Result`, never silent truncation) holds, and the open §7 questions (float total order, the `convert`/`math` rounding seam, cross-paradigm `eq`) are design/scope calls, not contract violations. The **landed trait naming** (`MycEq`/`MycOrd`/`MycPartialOrd`) is now documented in §3 — a `Myc`-prefix that avoids a Rust `std::cmp` namespace collision (RFC-0016 §8-Q2); a parity note, **not** an honesty matter. Status moves *Implemented (Rust-first) — pending ratification → Accepted*. Append-only; no kernel change (KC-3).

- **2026-06-27 — Self-hosted width-generic ordering prototype now executes three-way (M-718, rsm S2).** A *distinct artifact* from this Rust-first spec: `lib/std/cmp.myc` (RFC-0031 §5 D4) is a self-hosted Mycelium-lang prototype exporting **width-generic** `cmp` / `le` / `ge` / `max` / `min` over `Binary{N}` — one definition each, monomorphized to the call-site width (`le`/`ge`/`max`/`min` delegate to `cmp`; M-753 / DN-42). It now executes three-way (L1-eval ≡ L0-interp ≡ AOT; `crates/mycelium-l1/tests/std_cmp.rs` + the consolidated `std_generic_conformance.rs` at ≥ 2 widths), with width-mismatch refusals exercised (a mixed-width compare is an explicit never-silent refusal naming the offending width — G2). It is a **subset** of this spec's surface (no `convert`/`clamp`/`sort`-key, no float total-order story) and does **not** change this spec's status; the full Mycelium-lang migration (M-502-gated) still remains. Agreement `Empirical`; nothing upgraded to `Proven`. Append-only; no kernel change (KC-3).

- **2026-07-08 — Self-hosted signed comparison and completeness surface now documented (CU-4).** The width-generic `ne{N}`, `gt{N}`, `cmp_s{N}`, `le_s{N}`, `ge_s{N}` helpers (RFC-0033 §4.1.2 / RFC-0032 D1) close the comparison basis — signed three-way via `lt_s` prim + `eq`, negation of `eq`, unsigned `gt` projection of `cmp`. All total, all `Exact`; differential agreement (L1-eval ≡ L0-interp ≡ AOT) is `Empirical` (trials, `lib/std/cmp.myc:111-130` + `crates/mycelium-l1/src/checkty.rs:7307-7312`). Added to §3 surface-sketch (§3.1) and guarantee matrix (§4) as width-generic helpers distinct from the Rust-first trait surface; this spec's status and scope remain unchanged. Append-only; no kernel change (KC-3).
