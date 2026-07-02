# Spec — Self-hosting readiness gate (M-502)

| Field | Value |
|---|---|
| **Status** | **Draft** — the original (2026-06-17) verdict below was **"not yet established."** **⚠ SUPERSEDED by the §0 currency update (2026-07-01):** that verdict's own flip-trigger (§5) has FIRED — the surface now authors stdlib modules in Mycelium-lang, so the general "not-yet" claim is stale. See §0 for the current, grounded verdict. (Append-only: the original §1–§5 assessment is preserved as the 2026-06-17 snapshot; §0 records what changed. VR-5/G2.) |
| **Tracks** | **M-502** (#150) — the M-346 precondition made checkable. Gates the Mycelium-lang *migration half* of M-510…M-520 (RFC-0016 §4.6 Phase 5b). |
| **Scope** | Enumerate the surface-language capabilities a stdlib module needs in order to be **authored in Mycelium-lang itself** (dogfooding, "free of other languages"), assess each against the landed corpus, and emit an honest **ready / not-yet** verdict. Does **not** gate the Rust-first modules (Batches P5-A/P5-B proceed against RFC-0016 now). |
| **Depends on** | RFC-0016 §4.6 (the Rust-first → Mycelium-lang migration the gate sits inside); RFC-0006/0007 (the surface + L1 calculus a module is written in); RFC-0011 (L0 `Match` + data-in-core); RFC-0012 (ambient representation); DN-06 (`phylum`/`nodule`); M-359 (the manifest); M-320 (the L1 term-language extension) |
| **Grounds on** | the Doc-Index status of each cited doc; `tools/github/issues.yaml` (M-320 #92, the surface track); the RFC-0016 §4.6 migration discipline |

---

## 0. Currency update (2026-07-01) — the verdict has flipped for the SURFACE; remaining blockers are below-grammar

> **Grounded re-verification (2026-07-01, `Empirical`/`Exact` as tagged).** The original §1–§5 "not
> yet established" verdict (2026-06-17) is **superseded**. Its §5 flip-trigger — "a real module is
> authored in Mycelium-lang and passes the differential" — **has fired**: `lib/std/result.myc`
> (M-649) and `lib/std/option.myc` (M-715) were authored **directly in Mycelium** with no Rust source,
> and 8 `lib/std/*.myc` nodules execute three-way (L1-eval ≡ L0-interp ≡ AOT;
> `crates/mycelium-l1/tests/std_*.rs`). DN-14 (self-hosting gate) is **Resolved** with all capability
> rows `present`. RFC-0031 records Tier-0/Tier-1 nodules executing.
>
> **Current verdict: the language SURFACE (grammar + checker) is SUFFICIENT to author the structural
> majority of stdlib modules in pure Mycelium.** The classic suspected gaps have all landed —
> generics `[T]`, width-generics `{N}` (DN-42), traits (M-673 elaboration RUNs), effects `!{…}`
> (M-660), HOF **including capturing closures** (M-704 — the "closures are Impossible" line elsewhere
> is stale; capture is implemented in `mono.rs`), FFI via `wild`/`@std-sys` executing three-way
> (M-720/721), sequences/`Bytes` (RFC-0032). ~19 of 26 crates are expressible or already-demonstrated;
> ~5–7 are blocked.
>
> **The remaining blockers are NOT grammar/surface gaps — they are below-grammar (kernel-prim
> surfacing, value-model implementation, or staged execution):**
> 1. **No float value form/ops** (no float literal/type/prims; F16–F64 exist only as Dense dtypes) →
>    blocks `math`(f64 half), `numerics`. `Exact` (grammar + `interp/src/prims.rs`). Tracked: E20-1 /
>    RFC-0033 (full self-hosting deferred post-1.0 by ADR-035).
>    **→ CLOSED (2026-07-02, M-900 re-verification, `Empirical`).** The float value form/ops gap is
>    landed: `Repr::Float{F64}`/`Payload::Float` (M-896), the decimal float literal + nullary
>    `Float` type (M-897), `flt.{add,sub,mul,div,neg}` (M-898), and
>    `flt.{lt,le,gt,ge,eq,total_le}` (M-899) all close the **full three-way**
>    (L1-eval ≡ elaborate→L0-interp ≡ AOT) over surface `.myc` programs — the literal (incl. exponent
>    forms, the round-trip corpus, out-of-range/pattern/type-mismatch refusals), arithmetic,
>    comparisons, in-band specials (±inf, div-by-zero, `0/0` → canonical NaN, overflow → inf), NaN
>    *propagation* through arithmetic and re-canonicalization on `neg`, signed zeros (`+0`/`−0`
>    distinct under the total order, IEEE-equal under `flt_eq`), and canonical-NaN identity — **88
>    passing tests**, `crates/mycelium-l1/tests/enablement.rs` (verified + one closeout test added
>    for M-900: `flt_arith_nan_propagates_and_recanonicalizes_three_way`). **No AOT refusal was
>    needed for any float form** — every float form closes three-way with no exception to record
>    (contrast the dense group, M-890/M-891, whose nullary-main surface is still inexpressible
>    pending a dense literal) — recorded honestly per G2/VR-5. **No content-address rehash was
>    spent**, re-confirmed green against the M-896 golden-digest pin
>    (`crates/mycelium-core/src/tests/content.rs::adding_float_spent_no_rehash_existing_addresses_stable`,
>    RFC-0033 §7). **Tag discipline unchanged (`Empirical`, ADR-040 §2.6, VR-5 — never upgraded to
>    `Proven`):** the float *definition* (correctly-rounded RNE) is `Exact` as a definition; the
>    *host-delivers-those-bits* implementation claim stays `Empirical`, evidenced by the
>    hand-derived IEEE reference corpus in `mycelium-interp/src/tests/prims.rs`, and the
>    `flt_total_le` total-order property stays `Empirical` pending the M-511 proof (never `Proven`
>    without a checked theorem). **Residual FLAGs (not closed by this task, carried forward):**
>    `is_nan`/`is_finite` classification prims are still OPEN (`mycelium-core/src/prim.rs` —
>    M-899 shipped comparison/total-order only; NaN is detectable today via `¬flt_eq(x, x)`,
>    finiteness via `flt_lt(-inf, x) ∧ flt_lt(x, +inf)`; the float *gate* itself does not need
>    dedicated classification prims to close, so this is a follow-up, not a blocker); the
>    `flt.*`/`Float` surface-name ratification is deferred to the `integration` tier. This closes
>    blocker 1 of the 5 numbered + 2 untracked items in this section — the remaining 4 numbered
>    blockers (binary `mul`/`div`/`shl`/`shr` + signed ops, dense/VSA op-prims, RFC-0008 R2 runtime
>    vocabulary, `Substrate`/`consume` execution) and the 2 untracked items are **UNCHANGED** by
>    this task (out of scope — M-900 is the Gap-A float capstone only; several of them landed under
>    sibling `enb` tasks M-887…M-894/M-901…M-914 but re-verifying those is not this gate record).
> 2. **No binary `mul`/`div`/`shl`/`shr` prims + no signed-op set** → blocks the integer numeric half.
>    `Exact`. Tracked: M-718 FLAG · ADR-028 (signedness-as-operations) · E20-1.
> 3. **Dense/VSA op-prims not surfaced to L1** (types/literals exist; no `dense.*`/`vsa.*` ops in the
>    prim registry) → blocks `dense`, `vsa`. `Exact`. **Tracking unclear — recommend minting an issue.**
> 4. **RFC-0008 R2 runtime vocabulary inactive** (`mesh`/`graft`/`cyst`/`xloc`/`forage`/`backbone`
>    reserved-not-active) → blocks `runtime`'s full surface. Tracked: E12-1 / DN-63 / M-828.
> 5. **`Substrate`/`consume` execution staged** (elaborates to a never-silent `Residual`; no v0 value
>    form) → blocks `fs`/`io`'s affine-handle model. `Exact` (grammar comment). **No dedicated
>    execution issue found — recommend minting.**
> Plus two small **untracked** items: **no textual string literal** (only `0x…` `BytesLit` — an
> *ergonomic* gap, not an expressive one) and a **`hash.*` prim** for `content` (blake3 lives in
> `mycelium-core` but isn't surfaced). `Empirical`.
>
> **By-design non-gaps (RFC-0031 D1):** `mycelium-std-sys`/`sys-host` and the `io`/`fs`/`time`/`rand`
> FFI floor are *irreducibly Rust* behind the quarantined `@std-sys` `wild` boundary — "pure Mycelium"
> there legitimately means "`.myc` + the audited `wild` FFI edge," which works today. `core`/`l0`/`l1`/
> `cert`/`interp::prims`/`mlir` stay Rust forever.
>
> **Bottom line:** "not enough surface to self-host the stdlib" is **PARTIALLY-TRUE and mostly stale** —
> true only for the small below-grammar set above (all value-model/prim/execution items already
> deferred post-1.0 by ADR-035), false as a claim about the grammar/surface. The full port bar (D5/D6,
> Rust-crate retirement) remains post-1.0 per DN-66/ADR-035 — this update corrects the *surface-
> sufficiency* claim only, not the port-completion status. (Append-only; supersedes the 2026-06-17
> snapshot below; does not delete it. VR-5/G2.)

## 1. What this gate decides

RFC-0016 builds every module **Rust-first** (ADR-007 — the trusted toolchain) and **migrates** it to
Mycelium-lang only "as the surface self-hosts" (§4.6). M-346's precondition — the stdlib is "decomposed once
the surface language is self-hosting enough to write stdlib modules in Mycelium itself" — is a **claim that
must not be pre-declared**. This gate makes it *checkable*: a capability checklist (§2) with an honest
per-capability status, composed into a single **ready / not-yet verdict** (§3). The verdict is the planning
analogue of the honesty rule — a `Proven`-style "self-hosting" claim is allowed **only** when the surface
actually clears the checklist; absent that, the verdict stays **not established** and says so (VR-5/G2).

It gates a *specific, narrow* thing: the **Mycelium-lang authoring** of a module (RFC-0016 §4.6 Phase 5b —
the `diag`/`recover` self-hosting targets M-510/M-520, and any later migration). It does **not** gate the
Rust-first specs or implementations (Batches P5-A/P5-B), which depend on RFC-0016, not on self-hosting.

## 2. The capability checklist (what authoring a stdlib module in Mycelium-lang requires)

Each row: the capability, why a stdlib module needs it, the corpus that owns it, and its **current** landed
status (read off the Doc-Index, not asserted). "Status" is honest and may be `not yet` — that is the point.

| # | Capability needed to author a module in Mycelium-lang | Why a stdlib module needs it | Corpus basis | Current status |
|---|---|---|---|---|
| 1 | **Data declarations + matching** (algebraic data, a registry `Σ`, flat `Match`) | every module defines + destructures values (`Option`/`Result`, collections, records) | RFC-0011 (L0 `Match`, `Construct`, content-addressed registry; WF6/WF7/WF8) | **landed (kernel/IR)** — RFC-0001 **r3 ENACTED**; M-210 differential covers the data fragment |
| 2 | **Functions + closures + recursion** (`Lam`/`App`/`Fix`; the `for` fold) | combinators, folds, the recursion every non-trivial module uses | RFC-0001 **r4** (`Lam`/`App`/`Fix`, closed-closure value model); RFC-0007 §4.8 (`for` fold) | **landed (kernel/IR)** — RFC-0001 r4 Accepted; the v0 calculus is ratified |
| 3 | **A concrete surface syntax** to *write* a module in (L3) — declarations, signatures, guarantee annotations | a human/agent authors `.myc` source; without it there is no Mycelium-lang to author *in* | RFC-0006 (L0–L3 layering; concrete L3 syntax) | **ready — concrete L3 text surface committed (DN-09; RFC-0006 r5)** — the KC-2 verdict = proceed (DN-09, 2026-06-18) closes the design gate; the v0 grammar is the ratified L3 surface, refined append-only. Honest scope: the *design* gate (KC-2) is cleared; the surface still needs a *parser implementation* before a `.myc` module can actually be authored and run (the execution gate for M-510/M-520 remains). |
| 4 | **Leaf emission / a working term-language prototype** (the interpreter executes authored terms) | a self-hosted module must *run* + be differential-tested against its Rust reference | M-320 (L1 term-language extension, interpreter/prototype) | **landed (prototype/IR)** — M-320's **codegen half is complete**: the elaborator compiles nested `match` (Maranget) to flat L0 `Match`, and the L1 prototype parses→checks→elaborates→runs the data+recursion fragment, **differential-tested L1-eval ≡ L0-interp ≡ AOT through the shared M-210 checker** (M-302: datum results now validate through `check_core`, not a bespoke compare). The remaining authoring blocker is #3, not #4. |
| 5 | **Honest guarantee tags expressible in the surface** (the `Exact ⊐ Proven ⊐ Empirical ⊐ Declared` lattice on a signature) | the §4.1 contract (C2) requires every op carry its tag; a self-hosted module must *say* its tags in-language | RFC-0001 (the lattice in `Meta`); RFC-0006 (guarantee annotations in the grammar) | **partial** — the lattice + `Meta` exist; the *surface annotation* rides on capability #3 |
| 6 | **Declared/bounded effects in the surface** (C6 — IO/time/rand/budget on a signature) | Tier-B modules (`io`/`fs`/`time`/`rand`) declare effects; the surface must express them | RFC-0014 (declared + bounded effects); RFC-0006 (effect surface) | **partial** — the effect *model* is Accepted/enacted (Rust); the *surface* form rides on #3 |
| 7 | **Ambient representation / scoped overrides** (to keep honesty's verbosity tolerable in-language) | offsets tension A so an authored module is not drowned in explicit reprs | RFC-0012 (ambient representation; I1/I2) | **Accepted, enactment-gated** — design normative; enactment is M-344 |
| 8 | **Organization + packaging surface** (`phylum`/`nodule`; the `mycelium-proj.toml` manifest; nodule headers) | a module *is* a `nodule` in the `std` `phylum`, declared by header + manifest | DN-06 (`phylum`/`nodule`); M-359 (manifest); the Nodule-Header spec | **landed (design)** — DN-06 **Resolved**; the manifest + header are specified |

## 3. The readiness verdict (honest — VR-5)

**Verdict: NOT YET established.** Self-hosting a stdlib module is **close on the substrate, not yet on the
surface.**

- **What is ready (the substrate):** the *semantic* foundation a module needs — data + matching (#1),
  functions/closures/recursion (#2), the guarantee lattice + effect model, and the organization/packaging
  surface (#8) — is **landed** in the kernel/IR and the Accepted corpus. The thing a module *means* is
  expressible.
- **What is now ready on the design front (#3):** the *concrete L3 text surface* is **committed (DN-09;
  RFC-0006 r5)** — the KC-2 design gate is closed (2026-06-18). The v0 grammar is the ratified L3
  surface. Capabilities #5/#6 (the surface forms of tags + effects) and #7 (ambient, enactment-gated
  M-344) are no longer blocked by the KC-2 design gate; they ride on the surface being *implemented*
  (a working parser / elaborator for `.myc` source), which is the remaining execution gate.
- **What is not yet ready (execution gate):** even with the surface *designed*, authoring a real `.myc`
  module requires a **parser** that handles the committed L3 grammar and feeds it into the term-language
  prototype. That implementation step is not yet landed. The **running term-language prototype** (#4)
  executes the *elaborated kernel form*, not human-authored `.myc` *source*, because the parser for that
  source is the remaining blocker. Capabilities #5/#6 (surface tags + effects) and #7 (ambient,
  enactment-gated M-344) follow the implementation, not just the design.

Because authoring a *module* still requires a *working implementation* of #3 — a parser for the committed
concrete syntax — the overall execution verdict remains **not-yet**: the design gate is cleared but no
module can be *truthfully* called "self-hosted" until the surface is implemented, and claiming so would
violate the honesty rule (VR-5).

## 4. What this gate does and does not block

| Track | Gated by M-502? | Disposition |
|---|---|---|
| **RFC-0016 ratification (M-501)** + the per-module **specs** (this wave) | **no** | design-first; depends on the contract, not on self-hosting. Proceeds now. |
| **Rust-first module implementations** (Batches P5-A/P5-B) | **no** | ADR-007 trusted toolchain; the Rust reference is what a future self-hosted form is *differentialled against*. Proceeds now. |
| **Mycelium-lang authoring** of any module (RFC-0016 §4.6 Phase 5b) — incl. self-hosting `diag`/`recover` (M-510/M-520) | **yes** | **waits** on the *implementation* of #3 (a working parser for the committed L3 concrete syntax — the design gate is cleared by DN-09, but the execution gate remains). Until then a "self-hosted" claim is `not established`. |

This matches `docs/planning/phase-5.md` §3: M-502 gates only the *Mycelium-lang half*; the Rust-first work
does not wait on it.

## 5. How the verdict gets upgraded (the re-check trigger)

The verdict is **append-only with a status transition**, mirroring the ADR/RFC discipline. With M-320's
term-language prototype now landed (#4) **and the KC-2 design gate now cleared (DN-09; RFC-0006 r5)** (#3
design gate closed), the *single remaining* gate is the **concrete L3 surface implementation** (#3
execution gate): a working parser for the committed grammar that lets `.myc` source actually be authored
and elaborated. The verdict flips `not-yet → ready` only when a *real* stdlib module (the smallest honest
candidate — `core`/prelude or `diag`) can be **authored in Mycelium-lang source** **and** pass its
NFR-7-style migration differential against the Rust reference (RFC-0016 §4.6 Phase 5b) — the same M-210
checker the kernel fragment already validates through (now extended to datums, M-302). The first module to
clear it is the *evidence* that upgrades this verdict — never a forward declaration.
(The exact differential bar is RFC-0016 §8-Q5.)

## 6. Open questions (FLAGGED — carried from RFC-0016 §8)

- **(Q-a) The migration differential's bar.** What a self-hosted module must match (observable results only?
  tags + EXPLAIN bit-for-bit?) before the verdict flips for that module. → **RFC-0016 §8-Q5 / NFR-7**.
- **(Q-b) The smallest honest first target.** Whether the readiness *proof* is `core`/prelude (thinnest) or
  `diag` (the charter's named first self-hosting target, M-510). → ties RFC-0016 §4.6 + §8-Q1.
- **(Q-c) Surface coverage threshold.** How much of #5/#6 (tags + effects *in the surface*) must land before
  authoring is "enough" — a partial surface might author `core` but not `io`. → **RFC-0016 §8-Q3**.

## Meta — changelog

- **2026-07-02 — §0 blocker 1 (float value form/ops) CLOSED (M-900, `Empirical`); the other §0
  blockers are UNCHANGED.** Kickoff `enb` Phase-I H1 Gap A (M-895…M-900) landed the scalar-float
  value form (M-896), the decimal float literal + nullary `Float` type (M-897),
  `flt.{add,sub,mul,div,neg}` (M-898), and `flt.{lt,le,gt,ge,eq,total_le}` (M-899); M-900 (this
  entry) re-verifies the **full three-way** (L1-eval ≡ elaborate→L0-interp ≡ AOT) closure over the
  whole group — literal, arithmetic, comparisons, in-band specials, NaN propagation/
  re-canonicalization, signed zeros, canonical-NaN identity — 88 green tests in
  `crates/mycelium-l1/tests/enablement.rs` (one test added this task:
  `flt_arith_nan_propagates_and_recanonicalizes_three_way`, closing the one genuinely-uncovered
  three-way corner: NaN *propagated* through arithmetic, not only *produced* by `0/0`). **No AOT
  refusal was needed for any float form** (recorded honestly — there was nothing to refuse). **No
  content-address rehash was spent**, re-confirmed against the M-896 golden-digest pin in
  `crates/mycelium-core/src/tests/content.rs` (RFC-0033 §7). Tag discipline unchanged: `Empirical`
  per ADR-040 §2.6 (VR-5 — never upgraded to `Proven`; the `flt_total_le` total-order property
  stays `Empirical` pending the M-511 proof). §0 item 1 is edited in place with a `→ CLOSED`
  annotation (append-only — the original 2026-07-01 blocker text is preserved, not deleted).
  **FLAGged residuals, explicitly NOT closed by this task:** the `is_nan`/`is_finite`
  classification prims remain OPEN (workaround: `¬flt_eq(x,x)` / `flt_lt(-inf,x) ∧ flt_lt(x,+inf)`
  — the float *gate* does not need them to close); the `flt.*`/`Float` surface-name ratification is
  deferred to the `integration` tier; and §0's other 4 numbered blockers (binary `mul`/`div`/`shl`/
  `shr` + signed ops, dense/VSA op-prims, RFC-0008 R2 runtime vocabulary, `Substrate`/`consume`
  execution) plus its 2 untracked items are **out of this task's scope** and stay as last recorded
  (several landed under sibling `enb` tasks — re-verifying them is not this gate record). Append-only.
- **2026-06-18 — KC-2 design gate cleared (DN-09; RFC-0006 r5); capability #3 design-ready; overall verdict held at *not yet* (execution gate remains).** The **KC-2 verdict = proceed** (DN-09, 2026-06-18) closes the design gate on capability #3 (concrete L3 surface syntax): the v0 grammar is now the ratified L3 text surface (RFC-0006 r5), refined append-only. The maintainer A2 ruling from 2026-06-17 that "the authoring surface stays KC-2-gated" is **superseded** by this verdict — the design gate is cleared. **However, the overall verdict stays NOT YET established**: the gate that now blocks is the *implementation* of #3 (a working parser for the committed grammar), not the design decision. A stdlib module cannot be truthfully "self-hosted" until `.myc` source can actually be parsed, elaborated, and run (the execution gate). Capabilities #5/#6 surface forms and #7 ambient-repr enactment follow the implementation. §2 capability #3 status, §3, §4, and §5 re-check trigger updated to reflect the design/execution gate distinction. Honest scope: the T3.6 ablation remains open (DN-09 §4); "proceed" does not upgrade to the strong Q1 confirmation (VR-5). Append-only.
- **2026-06-17 — RFC-0016 ratified; the §8-Q5 differential bar fixed; verdict held at *not yet* (A2 ruling).**
  With RFC-0016 now **Accepted** (DN-07), the gate's two carried FLAGs are partly discharged: **Q-a / §8-Q5
  (the migration differential's bar) is RESOLVED** to a **two-level bar** — M-210 observable-result
  equivalence as the universal floor + per-module tag/EXPLAIN equivalence for honesty-load-bearing modules
  (the §5 re-check trigger now reads against this ratified bar). The maintainer also ruled (**A2**) that the
  **concrete L3 authoring surface (capability #3) stays KC-2-gated** (RFC-0006 §10) — the deciding experiment
  M-002 (#3) is unrun (needs LLM API; de-risked by the new `tools/llm-harness/` validation harness). So the
  **overall verdict stays NOT YET established**: #4 (term prototype) landed, #3 (the in-language source
  surface) remains the single open gate, and no module is truthfully "self-hosted." Q-c (§8-Q3 surface
  coverage) folds into the now-scheduled per-ring ergonomics pass (M-540). The verdict is **not flipped** (a
  flip awaits #3 + a passing migration differential). Append-only.
- **2026-06-17 — Capability #4 landed; verdict held at *not yet* (honest, VR-5).** Refines capability #4
  (leaf emission / term-language prototype) `not yet → landed (prototype/IR)`: M-320's codegen half is
  complete — the elaborator compiles nested `match` to flat L0 `Match`, the L1 prototype runs the
  data+recursion fragment, and that fragment is differential-tested L1-eval ≡ L0-interp ≡ AOT through the
  shared **M-210 checker**, now over *datum* results too (M-302's `check_core`). The **overall verdict
  stays NOT YET established**: the *single* remaining gate is the **concrete L3 authoring surface** (#3,
  RFC-0006 §10, KC-2-gated) — the substrate runs, but there is no in-language source surface to author a
  module in, so no module is truthfully "self-hosted." Updates §3 (separates the still-gated #3 from the
  now-landed #4) and the §5 re-check trigger (keyed on #3 alone). Append-only status transition; the
  verdict is not flipped (a flip awaits #3 + a passing migration differential).
- **2026-06-17 — Draft (needs-design).** Stands up the M-502 self-hosting readiness gate as a **checkable
  verdict**: an eight-row capability checklist (data+matching · functions/closures/recursion · concrete L3
  surface · a running term-language prototype · surface guarantee tags · surface effects · ambient repr ·
  organization/packaging), each assessed against the landed corpus. Honest verdict: **not yet established**
  — the *substrate* (data/recursion/closures via RFC-0011/RFC-0001 r4, the lattice + effect model, DN-06
  packaging) is ready, but the *surface* to author + run a module (concrete L3 syntax, KC-2-gated; M-320
  #92, open) is not. Records what the gate blocks (the Mycelium-lang migration half of M-510…M-520) vs what
  proceeds regardless (RFC-0016 ratification, the per-module specs, the Rust-first implementations), the
  re-check trigger, and three FLAGs (→ RFC-0016 §8-Q1/Q3/Q5). Never pre-declared (VR-5). Append-only.
