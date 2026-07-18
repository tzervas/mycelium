# Spec — `std.swap` (the certified, never-silent representation-change library)

| Field | Value |
|---|---|
| **Status** | **Accepted** (2026-06-20, maintainer-ratified per DN-07 — guarantee matrix asserted in tests; open §7/§8 questions are design/scope calls, not contract violations; was *Implemented (Rust-first) — pending ratification* 2026-06-18, Draft/needs-design 2026-06-17) — the Rust-first code landed as `mycelium-std-swap` (M-516, Batch P5-A; guarantee matrix asserted in tests). The Mycelium-lang migration (M-502-gated) remains. |
| **Module / Ring** | `std.swap` · Ring 1 (RFC-0016 §4.2) · Tier A |
| **Tracks** | `M-516` (#158) — the Phase-5 task this spec delivers |
| **Scope** | The ergonomic, certificate-carrying surface over Mycelium's **landed** certified representation swaps (binary↔ternary, `F32`→`BF16`, Dense↔VSA) and the build/check of the RFC-0002 `SwapCertificate` through the **one** M-210 shared checker. A swap is lexically visible, certificate-emitting, and **never silent**. |
| **Boundary** | A *representation change* (a `Repr`→`Repr` swap on a legal pair) is `std.swap`. A *non-representation, value-level* conversion (a lossy numeric narrowing, an ordering/equality coercion) is `std.cmp`/`convert` (M-532, #172), **not** a swap. Defining new legal pairs or new certificate *kinds* is RFC-0002 + the capability crates, not this module (C5). The selection *policy* that decides which swap to apply is `std.select`/`explain` (M-519, #161); `std.swap` *consumes* the `PolicyRef` it records, it does not author policy. |
| **Depends on** | **RFC-0002** (the `SwapCertificate`, the legal-pair table, the bijection semantics, the shared checker — this module is its library form); RFC-0016 §4.1 (the contract); RFC-0001 (the value model — `Value`/`Repr`/`Meta`, the guarantee lattice, content-addressing §4.6); ADR-010 (bound kernels + certificate); RFC-0005 (`PolicyRef`). |
| **Grounds on** | The landed capability crates this consumes (KC-3, above the kernel): the M-120 binary↔ternary `enc`/`dec`; the M-211 `F32`→`BF16` rounding swap; the M-231 Dense↔VSA swap; and the **M-210** shared certificate checker. The RFC-0002 §5 legal-pair table is the normative source of the matrix rows. |

---

## 1. Summary

`std.swap` is the library form of Mycelium's signature operation (RFC-0002): a representation swap that
yields a value in the target paradigm **and** an inspectable `SwapCertificate`. The user-facing surface is a
small set of named swaps (`bin_to_tern`/`tern_to_bin`, `f32_to_bf16`, `dense_to_vsa`/`vsa_to_dense`) plus
`build`/`check` over the certificate, each returning an explicit `Result`/`Option`. Its **honesty crux**: a
swap **structurally cannot be silent** — an unsupported `(R_src → R_target)` pair or an out-of-range value is
an explicit refusal/error (`Err`/`None`), **never** a clamp, a re-round, or a sentinel (C1/G2). The module is
Ring 1 and adds **no trusted code** (KC-3): it is a *certificate consumer* — it calls the one M-210 checker
and wraps the landed swap crates; it does not enlarge the trusted base, define new legal pairs, or re-prove
bijections.

## 2. Scope & module boundary

- **In scope:** the exported, named swaps over the RFC-0002 §5 **legal pairs** that have landed
  (binary↔ternary, `F32`→`BF16`, Dense↔VSA); `build` (assemble the `SwapCertificate` for a swap instance)
  and `check` (validate a certificate through the M-210 checker); the `Option`-typed inverse for the
  `LosslessWithinRange` bijection class; and the EXPLAIN projection of a swap's certificate.
- **Out of scope (and who owns it):**
  - *Value-level / non-representation conversion* (lossy narrowing, `Ord`/`Eq` coercions) → `std.cmp`/`convert`
    (M-532, #172). The boundary is: same paradigm, different *value* = `cmp`; different *`Repr`* on a legal
    pair = `swap`.
  - *Defining the legal pairs, the certificate format, or the checker* → RFC-0002 + the capability crates +
    the kernel (KC-3). This module does **not** add a pair or a certificate kind (C5).
  - *Authoring the selection policy* that chooses a swap → `std.select`/`explain` (M-519, #161). `std.swap`
    records the `PolicyRef` it was handed (RFC-0005); it does not decide policy.
  - *VSA model↔model swaps and the VSA capacity derivations* → `std.vsa` (M-513) / RFC-0003; `std.swap`
    exports only the Dense↔VSA boundary swap, with its bound sourced from RFC-0003/T0.2.
- **Ring & layering:** Ring 1 (RFC-0016 §4.2) — an ergonomic surface that **wraps** the landed swap crates
  (M-120/211/231) and **calls** the one M-210 checker; it re-exports the RFC-0001 certificate/lattice types
  rather than redefining them. It is a certificate/EXPLAIN **consumer**; no `wild`/FFI; no new trusted code
  (KC-3, C5).

## 3. Exported-op surface (pinned to the landed crate)

**Pinned to the landed `mycelium-std-swap` surface (DN-16, 2026-06-19; §7-Q4 resolved).** Value-semantic,
immutable-by-default; **every fallible op returns `Result` (no `Option`)**. The conversion/check/explain fns
are defined in `crates/mycelium-std-swap/src/lib.rs`; the certificate types + the M-210 `check` are
**re-exported from `mycelium_cert`** (not redefined here). The signatures below are the actual landed surface
(value-typed: `Value` in, `Swapped` out; the source representation is carried inside the dynamic `Value`).

```
// landed surface — crates/mycelium-std-swap/src/lib.rs (cert types re-exported from mycelium_cert)

// A swap result is the target value PLUS its certificate — never the value alone (C1/C3).
pub struct Swapped { value: Value, cert: SwapCertificate }   // non-generic; cert re-exported from mycelium_cert

// --- binary <-> ternary : LosslessWithinRange, Exact-within-range (RFC-0002 §4) ---
fn bin_to_tern(src: &Value, trits_width:  u32, policy: &PolicyRef) -> Result<Swapped, SwapError>
fn tern_to_bin(src: &Value, binary_width: u32, policy: &PolicyRef) -> Result<Swapped, SwapError>  // Err off the image

// --- F32 -> BF16 : Bounded (epsilon), one-way (RFC-0002 §5; ADR-010) ---
fn f32_to_bf16(src: &Value, policy: &PolicyRef) -> Result<Swapped, SwapError>   // carries ErrorBound epsilon
// no exact inverse exported: BF16 -> F32 widening is a `cmp/convert` lift, not a certified swap (boundary)

// --- Dense <-> VSA : Bounded/probabilistic (epsilon, delta) (RFC-0002 §5; RFC-0003/T0.2) ---
fn dense_to_vsa(src: &Value, vsa_dim:    u32, delta: f64, policy: &PolicyRef) -> Result<Swapped, SwapError>
fn vsa_to_dense(src: &Value, components: u32, delta: f64, policy: &PolicyRef) -> Result<Swapped, SwapError>

// --- certificate check / explain : the consumer surface over the ONE M-210 checker ---
// re-exported from mycelium_cert: check, CheckVerdict, Evidence, Fallback, NotValidatedReason,
//                                 RefinementRelation, SwapCertificate, SwapError
fn check_swap(a: &Value, b: &Value, cert: &SwapCertificate) -> Result<GuaranteeStrength, CheckError>  // delegates to cert::check (M-210); returns the established strength
fn explain(cert: &SwapCertificate) -> ExplainRecord                                                   // total; EXPLAIN
// (no `build` fn: a certificate is carried by the conversion fns above / assembled via mycelium_cert)

// Unsupported pair / out-of-range == explicit error, never a fallback. SwapError is re-exported
// from mycelium_cert (variants owned there). CheckError is defined here (matches the crate):
enum CheckError {
    Refuted { detail: String },                                  // concrete counterexample — the swap is wrong
    NotValidated { reason: NotValidatedReason, fallback: Fallback }, // TV incompleteness -> explicit fallback, never a silent pass
}
```

The `NotValidated` arm is load-bearing: translation validation may *fail to validate a correct* swap
(incompleteness, RFC-0002 §2). That is an explicit outcome that routes to a fallback path — **never** a silent
pass.

## 4. Guarantee matrix (the load-bearing deliverable — RFC-0016 §4.5)

Rows = exported ops; one row per exported swap plus the build/check/explain ops. The fallibility column carries
the **legal-pair side-conditions** (RFC-0002 §5). Encoded here as the checked RFC-0003 §4 table; asserted in
tests once code lands — never prose only.

| Op | Guarantee tag | Fallibility (explicit error set / legal-pair side-conditions) | Declared effects | EXPLAIN-able? |
|---|---|---|---|---|
| `bin_to_tern` | `Exact` (within range; RFC-0002 §4 `LosslessWithinRange`) | `Err(OutOfRange)` if `x ∉ image(enc)`; out-of-range is an explicit error, never a clamp (RFC-0002 §4) | `none` (pure) | yes (Bijective cert) |
| `tern_to_bin` | `Exact` (within range; partial right-inverse on the image) | `None` off `image(enc)`; `Option`-typed inverse — total bijection impossible at fixed widths (RFC-0002 §4) | `none` (pure) | yes (Bijective cert) |
| `f32_to_bf16` | `Proven` (ε) **iff** the ADR-010 rounding-error theorem's side-conditions check here; else `Empirical`/`Declared` (VR-5 downgrade) | one-way; legal pair `Dense F32 → BF16` (RFC-0002 §5); rejected pair = `Err(UnsupportedPair)`; never re-rounds silently | `none` (pure; bounded `alloc`) | yes (Bounded cert: ε + BoundBasis) |
| `dense_to_vsa` | `Empirical` (ε, δ) by default — VSA capacity result (RFC-0003/T0.2); `Proven` only if the cited theorem's side-conditions (dimension, sparsity class, model) check here | legal pair `Dense ↔ VSA` (RFC-0002 §5); `Err(PolicyRejected)` if no `PolicyRef`; `Err(NoStatableBound)` if the pair has no statable bound (type error, not a `Declared` gamble — RFC-0002 §5) | `none` (pure of inputs + `PolicyRef`; bounded `alloc`) | yes (Bounded cert: ε, δ + PolicyRef) |
| `vsa_to_dense` | `Empirical` (ε, δ); `Proven` only with checked side-conditions (as above) | legal pair `VSA ↔ Dense`; `Err(PolicyRejected)` / `Err(NoStatableBound)` as above | `none` (pure of inputs + `PolicyRef`; bounded `alloc`) | yes (Bounded cert) |
| `build` | `Exact` (assembles a certificate value; asserts no accuracy itself — the *swap* carries the tag) | `Err(UnsupportedPair)` for an illegal `(src → target)`; `Err(NoStatableBound)` for a pair with no statable bound (RFC-0002 §5) | `none` (pure) | yes (it *produces* the cert) |
| `check` | `Exact` (a verdict, not an approximation): `Ok` = validated, `Err(Refuted)` = disproved, `Err(NotValidated)` = TV incompleteness → explicit fallback | does **not** widen the trusted base — delegates to the one M-210 checker (KC-3); a non-validated correct swap is explicit, never a silent pass (RFC-0002 §2) | `none` (pure; the checker is a Rust consumer) | yes (the verdict references the cert) |
| `explain` | `Exact` (a faithful projection of the certificate; G11 dual human/machine form) | `total` (every certificate explains) | `none` (pure) | yes (it *is* the EXPLAIN record) |

Tag justification (VR-5 — downgrade rather than overclaim):

- **`bin_to_tern` / `tern_to_bin` → `Exact`-within-range.** The bijection class is the only genuinely
  provable swap (RFC-0002 §4, §5): `dec (enc x) = Some x` (left-inverse/injectivity) and the partial
  right-inverse on the image are **SMT-dischargeable for fixed widths**. A *total* bijection is impossible
  (2ⁿ = 3ᵐ holds only trivially), so the inverse is `Option`-typed and out-of-range is an explicit error.
  `Exact` is correct *within range*; nothing is claimed off the image.
- **`f32_to_bf16` → `Proven` (ε) only with checked side-conditions; else downgraded.** The bound is from
  ADR-010 rounding-error theory (RFC-0002 §5). Per RFC-0002 §3 the `strength` tag is **derived from how the
  bound was obtained, never asserted**: `Proven` *iff* the certificate's `BoundBasis = ProvenThm` and that
  theorem's side-conditions are checked here; otherwise `EmpiricalFit` → `Empirical` or `UserDeclared` →
  `Declared`. This is **not** `Exact` — it is a lossy/bounded swap carrying its ε tag.
- **`dense_to_vsa` / `vsa_to_dense` → `Empirical` (ε, δ) by default.** The bound rests on VSA capacity
  results (RFC-0003 / T0.2), which are probabilistic; the default honest tag is `Empirical` (trials, with a
  method). It rises to `Proven` only if the cited capacity theorem's side-conditions (dimension, sparsity
  class, model) are checked at certificate-build time (RFC-0002 §3). A pair with no statable bound is a
  **type error** (`Err(NoStatableBound)`), never a `Declared` gamble (RFC-0002 §5).
- **`build` / `check` / `explain` → `Exact`.** These carry no accuracy semantics of their own: `build`
  assembles a certificate value; `check` returns a verdict (the *accuracy* lives in the swap the cert
  describes); `explain` is a faithful projection (C2 — an op with no accuracy semantics is `Exact`).

## 5. §4.1 contract conformance (C1–C6)

- **C1 — never-silent (G2):** every swap returns the target value **with** its certificate, or an explicit
  `Err`/`None`. An unsupported pair is `Err(UnsupportedPair)`, an out-of-range value is `Err(OutOfRange)` /
  `None` (the `LosslessWithinRange` partial inverse), and a pair with no statable bound is
  `Err(NoStatableBound)` — never a clamp, a re-round, or a sentinel (RFC-0002 §4, §5). TV incompleteness is
  surfaced as `Err(NotValidated)` routing to an explicit fallback, never a silent pass (RFC-0002 §2).
- **C2 — honest per-op tag (VR-5):** each row tags on `Exact ⊐ Proven ⊐ Empirical ⊐ Declared`. The bijection
  swaps are `Exact`-within-range; the lossy/bounded swaps carry their ε (and δ) and reach `Proven` **only**
  when the bound's cited theorem side-conditions are checked, downgrading to `Empirical`/`Declared`
  otherwise (RFC-0002 §3). No swap is tagged `Exact` unless it is exact.
- **C3 — no black boxes / EXPLAIN (SC-3/G11):** every swap emits a `SwapCertificate` (Bijective: lemma_ref +
  params; Bounded: bound + BoundBasis + PolicyRef — the ratified `swap-certificate.schema.json`), and
  `explain` projects it into the dual human/machine form (G11). No swap decision is opaque.
- **C4 — content-addressed, value-semantic (ADR-003):** swaps are pure functions of their inputs (+ the
  `PolicyRef` for the policy-dependent ones); results are immutable values. The Bijective certificate is
  **cacheable by content hash** (RFC-0001 §4.6, RFC-0002 §3) — it references a once-per-swap-kind lemma plus
  concrete params, with no per-value proof. Metadata is not identity (ADR-003).
- **C5 — above the kernel (KC-3):** this module **consumes** the M-210 checker and the landed swap crates
  (M-120/211/231); it does not enlarge the trusted base, define a new legal pair, or re-prove a bijection. It
  is the *certificate consumer* the issue names. No `wild`/FFI.
- **C6 — declared, bounded effects (RFC-0014):** every exported op is pure (`none`) save for the bounded
  `alloc` of building the target representation + certificate; no IO, time, or randomness. The randomness of
  any VSA construction is upstream in `std.vsa` (M-513) and arrives reified via the `PolicyRef`, not pulled
  silently here.

## 6. Grounding

- The module is the **library form of RFC-0002** (RFC-0016 §4.3 `swap` row; §7 prior art "RFC-0002 swap").
  The certificate (Bijective / Bounded), the legal-pair table, and the shared-checker decision are RFC-0002
  §2–§5; the ratified shape is `docs/spec/schemas/swap-certificate.schema.json` (M-010).
- **`LosslessWithinRange`, `Option`-typed inverse, Exact-within-range, out-of-range = explicit error:**
  RFC-0002 §4 (binary↔ternary bijection semantics, T2.1) — the only genuinely bijective/provable swap class.
- **One checker, no enlarged trusted base:** RFC-0002 §2 (the single `(A, B, R, claimed-bound, certificate)`
  checker, a Rust consumer; TV incompleteness → explicit fallback), routed through **M-210**; KC-3 / C5.
- **Bound bases and the derived-not-asserted strength tag:** RFC-0002 §3, §5 (`F32`→`BF16` rounding-error
  theory; Dense↔VSA capacity ε/δ from RFC-0003/T0.2; a pair with no statable bound is a type error), ADR-010
  (bound kernels + certificate); VR-5 (downgrade to stay honest).
- **Contract:** RFC-0016 §4.1 (C1–C6) and §4.5 (the guarantee-matrix obligation); the §4.1 reference and
  matrix obligation in `docs/spec/stdlib/README.md`. The value model + lattice + content-addressing: RFC-0001
  (§4.6). The `PolicyRef`: RFC-0005.
- **Landed grounding crates:** M-120 (binary↔ternary `enc`/`dec`), M-211 (`F32`→`BF16`), M-231 (Dense↔VSA),
  M-210 (shared checker) — per RFC-0016 §4.3 and the README index. Their exact landed signatures are
  described abstractly here and FLAGGED (§7-Q4).

## 7. Open questions (FLAGGED — resolve before ratification)

- **(Q1) Module + swap naming.** Is the phylum `std` and are the ops named `bin_to_tern` / `f32_to_bf16` /
  `dense_to_vsa`, or does the fungal lexicon (DN-02/06) prefer themed names? Ties to **RFC-0016 §8-Q2
  (naming)** — a DN-level decision, not settled here.
- **(Q2) Ergonomics of the certificate at the call site.** Should the certificate be an explicit second
  return everywhere (`Swapped<T>`), or implicit-by-default-but-inspectable (the RFC-0012 ambient lesson)?
  Always-explicit is the honest floor; the ergonomic default is open. Ties to **RFC-0016 §8-Q3 (ergonomics vs
  the contract)**. Disposition: default to explicit `Swapped<T>` until §8-Q3 resolves.
- **(Q3) `BF16`→`F32` widening placement.** A `BF16`→`F32` *widening* is value-exact but is **not** a
  certified swap in RFC-0002 §5 (it is a one-way bounded pair the other direction). Does it belong here as a
  trivially-exact lift, or in `std.cmp`/`convert` (M-532)? Ties to **RFC-0016 §8-Q1 (module set)** and the
  swap/convert boundary; cross-module reconcile with `cmp.md`. Disposition: leave with `cmp`/`convert` unless
  the maintainer rules the boundary the other way.
- **(Q4 — RESOLVED 2026-06-19; DN-16) Exact landed-crate surface.** §3 is now **pinned to the landed
  `mycelium-std-swap` surface** (verified against `crates/mycelium-std-swap/src/lib.rs`): the conversion fns
  (`bin_to_tern`/`tern_to_bin`/`f32_to_bf16`/`dense_to_vsa`/`vsa_to_dense`), `check_swap(a, b, cert) ->
  Result<GuaranteeStrength, CheckError>` (delegating to the **re-exported** `mycelium_cert::check`, the M-210
  checker), and `explain`. Corrections vs the old sketch: there is **no `build` fn**; the certificate types
  (`SwapCertificate`/`SwapError`/`CheckVerdict`/`Evidence`/`Fallback`/`NotValidatedReason`/`RefinementRelation`)
  are re-exported from `mycelium_cert`; `CheckError` carries the richer two-arm shape (`Refuted{detail}` /
  `NotValidated{reason, fallback}`). The M-210 entry point is confirmed (`cert::check`). The conversion fns
  are **value-typed**: each takes `&Value` plus its explicit width/`delta` parameter(s) and a `&PolicyRef`, and
  returns `Result<Swapped, SwapError>` (`Swapped` is **non-generic**: `{ value: Value, cert }`); there are **no
  `Option` returns** — every fallible op is `Result`. (Earlier §3 drafts showed typed-generic pseudo-signatures
  `Dense<F32>`/`Vsa`/`Swapped<T>` and an `Option` inverse — those were schematic and are now corrected.)
- **(Q5) Migration bar for a self-hosted `std.swap`.** When `std.swap` migrates Rust→Mycelium-lang (RFC-0016
  §4.6), must the self-hosted form match the Rust reference on *observable swap results only*, or on the
  *certificate + EXPLAIN bit-for-bit*? Ties to **RFC-0016 §8-Q5 (the migration differential's bar)** /
  NFR-7. A certificate consumer arguably must match the cert, not just the value — FLAGGED for the gate.

## Amendment — 2026-07-18: W-1 binary width canon corrective (append-only)

**Status of this amendment.** Captured 2026-07-18 from the maintainer's binding corrective
(`docs/planning/design-steer-2026-07-17/PROGRAM-HANDOFF-DESIGN-STEER-2026-07-17.md` §2). This section
**amends by addition** — §1-§7 above (Accepted 2026-06-20, DN-07) stand unchanged; nothing in §3's
exported-op surface or §4's guarantee matrix is rewritten. Implementation (the site sweep, the new
exports) is **Phase-C work** (the program handoff's §5 wave W-D) and is **pending**. The doc's own
`Status` field above is left unchanged (**Accepted**, 2026-06-20, DN-07): this is an additive capture of
a binding steer, not an independent re-ratification.

### B.1 What changes for `std.swap`

`bin_to_tern`/`tern_to_bin` (§3 above) are unaffected in *signature* — both already take an explicit
`trits_width`/`binary_width: u32` parameter, so no default-width assumption is baked into the exported op
itself. What the corrective changes is which pair is treated as **canonical** wherever a width is
*exemplified* (docs, examples, the fungal-lexicon std exports) rather than *parameterized*:
`Binary{64} ↔ Ternary{41}` is now that canonical pair (`Binary{32} ↔ Ternary{21}` the recognized
fallback), per `docs/spec/swaps/binary-ternary.md`'s companion 2026-07-18 amendment (§A.2-§A.3 there) —
see that file for the legality arithmetic and the E-W1 enablement gate; not restated here.

### B.2 Std-export sweep (Phase-C, pending)

`lib/std/swap.myc` today exports only the `{8,6}` and `{4,3}` pairs (`bin8_to_tern6`/`tern6_to_bin8`,
`bin4_to_tern3`/`tern3_to_bin4`) — verified against the file 2026-07-18; no `bin32_to_tern21` or
`bin64_to_tern41` family exists yet. Per the corrective, the Phase-C sweep adds `bin32_to_tern21`/
`tern21_to_bin32` **now** (available — no enablement gap) and `bin64_to_tern41`/`tern41_to_bin64`
**behind E-W1**. This is a `lib/` code change and is explicitly **out of scope for this amendment**
(docs capture only) — recorded here as the normative TODO the Phase-C sweep must close, with the new
exports' certificate class unchanged from §6 of `docs/spec/swaps/binary-ternary.md` (both pairs stay
`LosslessWithinRange`, RFC-0002 §4).

### B.3 Length/count canon fix-list (normative TODO, not landed here)

The corrective also standardizes length- and count-typed returns on `Binary{64}` (`usize` parity with
the transpiler's `u64`/`usize` → width-64 mapping). Verified against the tree 2026-07-18,
`lib/std/swap.myc` has **one** concrete inconsistency in this class: `matrix_len`
(`lib/std/swap.myc:222`) returns `Binary{8}`, while the `nonempty` helper's `bytes_len` delegate
(`lib/std/swap.myc:171-174`) already returns `Binary{32}` — the two length-typed helpers in the same
file disagree with each other, and neither yet uses the new `Binary{64}` canon. **This amendment does
not fix the code** (out of the docs-only scope of this capture); it records the fix as **normative for
the Phase-C sweep**: `matrix_len` moves `Binary{8} → Binary{64}`, and `bytes_len` aligns to the same
`Binary{64}` canon (currently `Binary{32}`, the recognized fallback — not wrong, but not canonical once
Phase-C lands).

### B.4 Tracking

The E-W1 enablement item (lifting the `mycelium-std-ternary`/`mycelium-core` conversion-utility `i64`
ceiling so `bin64_to_tern41`/`tern41_to_bin64` are constructible) is proposed to the integrating parent
as **M-1119** — not filed in `tools/github/issues.yaml` by this amendment (`issues.yaml` is
orchestrator-owned; see this batch's FLAG report for the exact entry text). The full sweep-site list is
`PROGRAM-HANDOFF-DESIGN-STEER-2026-07-17.md` §2.1/§2.2 item 5, not duplicated here.

## Meta — changelog

- **2026-06-17 — Draft (needs-design).** Stands up the `std.swap` module design spec (M-516, #158; Ring 1,
  Tier A) as the **library form of RFC-0002**: the ergonomic, never-silent, certificate-carrying surface over
  the **landed** certified swaps (binary↔ternary M-120, `F32`→`BF16` M-211, Dense↔VSA M-231) plus
  `build`/`check`/`explain` through the **one** M-210 checker. Honesty crux: a swap structurally cannot be
  silent — an unsupported `(R_src → R_target)` pair, an out-of-range value, or a pair with no statable bound is
  an explicit refusal/error, never a clamp or sentinel (C1/G2). The load-bearing **guarantee matrix** (§4)
  carries one row per exported op with its legal-pair side-conditions in the fallibility column: the bijection
  swaps tagged `Exact`-within-range with an `Option`-typed inverse (RFC-0002 §4 `LosslessWithinRange`); the
  lossy/bounded swaps carrying their ε (and δ) and reaching `Proven` **only** when the cited bound theorem's
  side-conditions are checked, downgrading to `Empirical`/`Declared` otherwise (VR-5; RFC-0002 §3). §5 maps
  C1–C6; the module is a **certificate consumer** that enlarges no trusted base (KC-3, C5) and authors no
  policy (RFC-0005). Five §7 questions FLAGGED (naming → §8-Q2; call-site ergonomics → §8-Q3; `BF16`→`F32`
  placement → §8-Q1; the abstract landed-crate surface; the self-hosting migration bar → §8-Q5). No code; no
  kernel change (KC-3). Append-only.

- **2026-06-20 — Accepted (maintainer ratification, DN-07).** The maintainer ratified this Rust-first spec: the §4.5 guarantee matrix is asserted in tests, never-silent fallibility and honest per-op tags hold, and the open §7/§8 questions are design/scope calls, not contract violations. No guarantee tag was upgraded without a checked basis (VR-5). Status moves *Implemented (Rust-first) — pending ratification → Accepted*. Append-only; no kernel change (KC-3).

- **2026-07-18 — W-1 corrective captured (Amendment, append-only; see §B above).** Records the
  maintainer's binding width-canon corrective as it applies to `std.swap`: canonical pair moves to
  `Binary{64}↔Ternary{41}` (`Binary{32}↔Ternary{21}` recognized fallback) for exemplified/exported
  widths; `bin_to_tern`/`tern_to_bin` signatures (§3) are unaffected (already width-parameterized); the
  Phase-C std-export sweep (`bin32_to_tern21`/`tern21_to_bin32` now, `bin64_to_tern41`/`tern41_to_bin64`
  behind E-W1) and the `matrix_len`/`bytes_len` length-canon fix-list are recorded as normative TODOs,
  not landed (docs-only capture). E-W1 proposed to the integrating parent as **M-1119**. `Status` field
  unchanged (**Accepted**, 2026-06-20, DN-07) — additive amendment, not a re-ratification.
