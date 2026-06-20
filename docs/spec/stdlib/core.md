# Spec — `std.core` (the honest value model / prelude)

| Field | Value |
|---|---|
| **Status** | **Accepted** (2026-06-20, maintainer-ratified per DN-07 — guarantee matrix asserted in tests; open §7/§8 questions are design/scope calls, not contract violations; was *Implemented (Rust-first) — pending ratification* 2026-06-18, Draft/needs-design 2026-06-17) — the Rust-first code landed as `mycelium-std-core` (M-515, Batch P5-A; guarantee matrix asserted in tests). The Mycelium-lang migration (M-502-gated) remains. |
| **Module / Ring** | `std.core` (the prelude) · Ring `0` (RFC-0016 §4.2) · Tier `A` |
| **Tracks** | `M-515` (#157) — the Phase-5 task this spec delivers |
| **Scope** | The thin prelude: the re-exported value model (`Value`/`Repr`/`Meta`), `Option`/`Result` + the never-silent error values, and the guarantee-lattice tags (`Exact ⊐ Proven ⊐ Empirical ⊐ Declared`) with their query surface. It owns the *naming and documentation* of the kernel's value model for the library — not the model itself. |
| **Boundary** | Out: a representation change is `std.swap` (M-516), not a `core` operation; numeric ε/δ-bounded helpers are `std.numerics` (M-512); the errors-as-values *combinator* ergonomics (`?`-propagation, `map_err`, recovery triggers) are `std.error` (M-527) / `std.recover` (M-520); content-addressing as a first-class library is `std.content` (M-523). `core` exposes the *types* `Option`/`Result`/error and the lattice tags; it adds no combinators of its own beyond trivial kernel constructors/projections. |
| **Depends on** | RFC-0001 (the value model — `Value`/`Repr`/`Meta`, the guarantee lattice §4.7, content-addressing §4.6, the runtime query surface §4.8); RFC-0016 §4.1 (the contract); RFC-0016 §4.2 (Ring-0 layering) / §4.3 (the Tier-A `core` row). |
| **Grounds on** | `mycelium-core` (M-101 — the landed kernel crate that defines `Value<R>`/`Repr`/`Meta`/`GuaranteeStrength`/`CoreValue`/`Datum` and the §4.8 query surface). Ring-0 **re-exports only**; KC-3: no new trusted code. |

---

## 1. Summary

`std.core` is the **prelude**: the thin Ring-0 surface every other stdlib module imports to talk about values
honestly. It **re-exports** the kernel's value model from `mycelium-core` (RFC-0001) — `Value<R>` with its `Repr`
and `Meta`, the runtime sum `CoreValue`/`Datum`, the `GuaranteeStrength` lattice tags
(`Exact ⊐ Proven ⊐ Empirical ⊐ Declared`), and `Option`/`Result` with the never-silent **error values** — plus
the §4.8 query surface (`repr_of`/`meta_of`/`guarantee_of`/`bound_of`/`provenance_of`). Its **honesty crux** is
structural and *inherited, not invented*: because the kernel forbids a silent `Repr` change (WF1/WF2) and a
spurious guarantee upgrade (§4.7 `meet`), `core` is where that floor is *documented* and *named* for the whole
library — the lattice tags and the explicit-error discipline are the contract every Ring-1/Ring-2 module then
builds on. It is Ring 0 and **adds no trusted code** (KC-3): everything it exports is an alias or re-export of a
kernel item that already landed in M-101.

## 2. Scope & module boundary

- **In scope:** the *re-exported* names — the value model (`Value`/`Repr`/`Meta`, `CoreValue`/`Datum`), the
  guarantee-lattice tags + the `Bound`/`BoundBasis` types, `Option`/`Result` and the canonical error-value
  type(s), and the §4.8 runtime query functions. Plus a documented, curated **prelude set** (the names imported
  by default) and the trivial total constructors/projections (`Some`/`None`/`Ok`/`Err` and their pattern access)
  where these are kernel forms, not new logic.
- **Out of scope (and who owns it):** representation change → `std.swap` (M-516); ε/δ numeric helpers →
  `std.numerics` (M-512); error/option/result *combinators* and `?`-propagation ergonomics → `std.error`
  (M-527); recovery-as-effect → `std.recover` (M-520); content-addressing *as a first-class library* →
  `std.content` (M-523, distinct from the identity model `core` merely surfaces). `core` names the types; the
  verbs live in those modules.
- **Ring & layering:** Ring 0 (RFC-0016 §4.2). `core` **re-exports** `mycelium-core` and **aliases** for
  ergonomic/themed names (Q2); it **wraps nothing** and **builds no new trusted code**. KC-3 statement: every
  exported item resolves to a kernel item from M-101 — `core` enlarges the trusted base by zero lines; a
  `wild`/FFI block here would be a contract violation and there are none.

## 3. Exported-op surface (design sketch)

A re-export prelude, so the "ops" are mostly *names made available* plus the kernel's already-defined total
constructors/projections and the §4.8 queries. This sketch fixes the surface to feed the §4 matrix; it is **not**
a committed grammar, and any exact `mycelium-core` identifier below is **illustrative** — the canonical names are
whatever M-101 exported (FLAG Q2).

```
// illustrative re-exports (names track mycelium-core / RFC-0001; not a committed surface)

// — value model (RFC-0001 §4.1–4.3) —
type Value<R: Repr>          // re-export: payload + meta
type Repr                    // re-export: Binary | Ternary | Dense | VSA  (closed kinds, §4.1)
type Meta                    // re-export: provenance, guarantee, bound, sparsity, physical, … (§4.3)
type CoreValue               // re-export: Repr(Value<R>) | Data(Datum)    (RFC-0001 §4.2 r3)
type Datum                   // re-export: { ctor, fields, guarantee }     (meet-summary, no bound)

// — the guarantee lattice (RFC-0001 §4.7; the documented floor) —
enum GuaranteeStrength { Exact, Proven, Empirical, Declared }   // re-export; Exact ⊐ Proven ⊐ Empirical ⊐ Declared
type Bound; type BoundBasis  // re-export: { kind, basis }; ProvenThm | EmpiricalFit | UserDeclared (§4.3)

// — errors-as-values (the never-silent floor; types only — combinators are std.error) —
enum Option<T> { Some(T), None }            // re-export
enum Result<T, E> { Ok(T), Err(E) }         // re-export
type CoreError                              // re-export: the kernel's canonical error value(s)  [FLAG Q2: exact name]

// — the §4.8 runtime query surface (inspectability; re-export) —
fn repr_of(v: &CoreValue)       -> Repr
fn meta_of(v: &CoreValue)       -> Meta
fn guarantee_of(v: &CoreValue)  -> GuaranteeStrength
fn bound_of(v: &CoreValue)      -> Option<Bound>
fn provenance_of(v: &CoreValue) -> Provenance
```

## 4. Guarantee matrix (the load-bearing deliverable — RFC-0016 §4.5)

Rows = exported items. For a re-export prelude every row is **`Exact`** and **total**: `core` introduces no
operation that *selects*, *converts*, or *approximates*, so none carries accuracy semantics of its own (C2 ⇒
`Exact`, per RFC-0016 §4.1). The value of the table is to **document the floor** — that the lattice tags and the
error-value discipline are inherited, exact, and inspectable — not to disclose an approximation. Encoded as a
checked table (the RFC-0003 §4 template), asserted in tests once code lands — never prose only.

| Op / item | Guarantee tag | Fallibility (explicit error set) | Declared effects | EXPLAIN-able? |
|---|---|---|---|---|
| `Value` / `Repr` / `Meta` (type re-exports) | `Exact` | total (type alias; no op) | none | n/a (data, not a decision) |
| `CoreValue` / `Datum` (type re-exports) | `Exact` | total | none | n/a |
| `GuaranteeStrength` (lattice tags) | `Exact` | total | none | n/a (it *is* the disclosure vocabulary) |
| `Bound` / `BoundBasis` (type re-exports) | `Exact` | total | none | n/a |
| `Option<T>` / `Some` / `None` | `Exact` | total (constructors/projections) | none | n/a |
| `Result<T,E>` / `Ok` / `Err` | `Exact` | total (constructors/projections) | none | n/a |
| `CoreError` (error-value type) | `Exact` | total (it *is* the error vocabulary) | none | n/a |
| `repr_of` / `meta_of` (queries §4.8) | `Exact` | total | none | n/a (surfaces EXPLAIN inputs) |
| `guarantee_of` / `bound_of` / `provenance_of` | `Exact` | total | none | yes — surfaces the value's own tag / bound / provenance DAG |

**Why every row is `Exact` and total (justification, VR-5).** No row carries accuracy/precision/probability
semantics of its own, so the honest tag is `Exact` (RFC-0016 §4.1 C2: "an op with no accuracy semantics … is
simply `Exact`") — this is the *honest floor*, not an upgrade. No row is fallible because re-exporting a type and
reading kernel-maintained metadata cannot fail on well-formed kernel values; there is therefore no error set to
declare here, and the never-silent obligation is met vacuously (there is nothing to fail silently). The
`guarantee_of`/`bound_of` queries are marked EXPLAIN-able because they are the very surface through which a
*downstream* op's `Proven`/`Empirical`/`Declared` tag and its `Bound`/`BoundBasis` become inspectable (RFC-0001
§4.8); `core` does not *produce* those tags, it *exposes* them. No row carries `Proven` — there is no theorem to
check here, so claiming `Proven` would itself violate VR-5; `Exact` is correct and stronger.

## 5. §4.1 contract conformance (C1–C6)

- **C1 — never-silent (G2):** `core` adds no fallible op, so it cannot fail silently. Its *contribution* to C1
  is to **re-export and document** the never-silent vocabulary every other module uses — `Option`/`Result` and
  the kernel error value — so that "a `parse` failure is `Err`, not `0`" has a single, named home. The error
  values it surfaces are the floor; suppression is not expressible because no `core` item swallows one.
- **C2 — honest per-op tag (VR-5):** every exported item is `Exact` (§4), the honest tag for a no-accuracy
  surface. `core` is also where the **lattice itself** (`Exact ⊐ Proven ⊐ Empirical ⊐ Declared`) and the
  `Bound`/`BoundBasis` companion (RFC-0001 §4.3, invariants M-I1–M-I4) are *documented* as the library-wide
  tagging discipline. It re-exports the tags; it never upgrades one.
- **C3 — no black boxes / EXPLAIN (SC-3/G11):** `core` makes no selection/conversion/approximation, so it has no
  decision to EXPLAIN. It instead re-exports the §4.8 **query surface** (`guarantee_of`/`bound_of`/
  `provenance_of`) that *is* the inspection channel other modules' EXPLAIN records read from — the prelude is the
  audit window, not an opaque heuristic.
- **C4 — content-addressed, value-semantic (ADR-003):** the re-exported `Value`/`CoreValue`/`Datum` are the
  kernel's immutable, content-addressed values (RFC-0001 §4.6); identity is structural and metadata is **not**
  identity. `core` adds no mutable state and no new identity scheme — it surfaces the kernel's.
- **C5 — above the small kernel (KC-3):** the defining property of this module. Every export is a re-export or
  alias of an M-101 `mycelium-core` item; `core` adds **zero** lines to the trusted base, has **no** `wild`/FFI
  (ADR-014), and is the structural guarantee that Ring 0 is "kernel-adjacent re-exports, *no new trusted code*"
  (RFC-0016 §4.2).
- **C6 — declared, bounded effects (RFC-0014):** every exported item is pure (`none` in §4) — re-exporting a type
  and reading already-computed metadata has no IO/time/randomness/allocation-beyond-the-obvious. There is no
  effect to declare or bound; `core` is the effect-free floor the effectful modules sit above.

## 6. Grounding

- The value model, the four-point lattice (`Exact ⊐ Proven ⊐ Empirical ⊐ Declared`) and its `meet`
  monotone-downward propagation, the `Bound`/`BoundBasis` schema + invariants M-I1–M-I4, the `Option`/`Result`
  forms, and the §4.8 query surface are all defined in **RFC-0001 (Accepted, r5)** §4.1–4.3, §4.6–4.8 — `core`
  re-exports them, it does not redefine them.
- The Ring-0 "kernel-adjacent re-exports, no new trusted code" placement and the Tier-A `core` row tracing to
  RFC-0001 / M-515 are **RFC-0016 §4.2 / §4.3**; the C1–C6 per-op contract is **RFC-0016 §4.1**; the
  guarantee-matrix obligation is **RFC-0016 §4.5**.
- KC-3 (small auditable kernel; the stdlib lives *above* it), G2 (never-silent), and VR-5 (honest tags) are the
  Foundation requirements this module is the library-level keystone for; ADR-003 grounds the content-addressed
  value semantics (C4). No claim here is asserted without one of these bases.

## 7. Open questions (FLAGGED — resolve before ratification)

- **(Q1) Prelude membership — what is imported by default?** Which exact names are in the auto-imported prelude
  vs. require an explicit `std.core::…` path is an ergonomics decision (RFC-0016 §8-Q3, "ergonomics vs the
  contract"). FLAGGED: too thin forces boilerplate; too broad pollutes the namespace. Disposition: propose a
  minimal set (the value model + `Option`/`Result` + the lattice tags + the query surface) and defer the final
  list to ratification. **Cross-module:** the orchestrator should keep this prelude set consistent across all
  module specs.
- **(Q2) Naming / aliasing of re-exports + the exact error-value name.** Whether `core` exposes the kernel names
  verbatim or aliases some to themed fungal-lexicon names (RFC-0016 §8-Q2; DN-02/06) — and the **exact**
  `mycelium-core` identifier for the canonical error value (the §3/§4 `CoreError` is illustrative, not verified
  against M-101) — is unresolved. Disposition: FLAGGED; this spec describes the error type abstractly and does
  **not** commit a name it has not verified. **Cross-module:** error-value naming must match `std.error` (M-527).
- **(Q3) Types-only boundary vs. `std.error`.** `core` exports `Result<T,E>` and the kernel error value;
  `std.error` (M-527) owns the *combinator* ergonomics over them. The boundary — does `core` re-export any
  `?`-style helper, or strictly the bare types? — needs a one-line maintainer call so Ring 0 stays "types only."
  Disposition: this spec assumes **types only**; FLAGGED for confirmation against M-527.

## Meta — changelog

- **2026-06-17 — Draft (needs-design).** Stands up the Ring-0 `core`/prelude design spec under RFC-0016 (Draft):
  scope + boundary (re-export the value model, `Option`/`Result`/error values, and the guarantee-lattice tags;
  representation change / numerics / combinators / recovery / content-addressing explicitly owned by adjacent
  modules), the exported-name design sketch, and — the load-bearing deliverable — the guarantee matrix (9 rows,
  **all `Exact`/total**, the honest floor for a no-accuracy re-export surface; `guarantee_of`/`bound_of`/
  `provenance_of` marked EXPLAIN-able as the inspection window other modules read). The honesty crux is documented
  as *inherited, not invented*: `core` is where the lattice tags and the never-silent error floor are *named* for
  the whole library (G2/VR-5), while introducing **no trusted code** (KC-3) — every export resolves to an M-101
  `mycelium-core` item (RFC-0001 Accepted r5). C1–C6 conformance stated per clause. Three questions FLAGGED —
  prelude membership (Q1; ties to RFC-0016 §8-Q3), re-export naming + the unverified canonical error-value name
  (Q2; ties to §8-Q2 and to `std.error` M-527), and the types-only boundary vs. `std.error` (Q3) — all for the
  orchestrator to reconcile across modules. No code; no kernel change (KC-3). Append-only.

- **2026-06-20 — Accepted (maintainer ratification, DN-07).** The maintainer ratified this Rust-first spec: the §4.5 guarantee matrix is asserted in tests, never-silent fallibility and honest per-op tags hold, and the open §7/§8 questions are design/scope calls, not contract violations. No guarantee tag was upgraded without a checked basis (VR-5). Status moves *Implemented (Rust-first) — pending ratification → Accepted*. Append-only; no kernel change (KC-3).
