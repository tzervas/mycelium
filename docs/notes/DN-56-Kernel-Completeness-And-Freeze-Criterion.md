# Design Note DN-56 — Kernel Completeness & the Freeze Criterion (no black boxes, by construction)

| Field | Value |
|---|---|
| **Note** | DN-56 |
| **Status** | **Enacted** (2026-07-02) — **THE KERNEL FREEZE IS DECLARED (M-969; see §9 + Changelog).** All five §5 conditions now hold, evidenced (`Empirical`, not `Proven` — VR-5): §5.1 census (never-silent floor, satisfied W5); §5.2 reject-ledger (DN-80 + the M-959 regression guard, green); §5.3 primitive set closed (Π = 38 prims; ADR-033 FLAG-1 dispositioned IN via DN-74; `vsa.*`/Gap-E landed); §5.4 lowering surface closed (RFC-0037 Enacted; DN-54 extension surface checked; DN-71/DN-73/DN-74 resolved; grammar baseline in sync); §5.5 KC-3 completeness review **passed** (run 2026-07-02 — `runnable_gate` + reject-ledger green, no silent kernel growth). Independent four-condition scorecard: **DN-76 §5A, 4/4 green**. Post-freeze diff policy: **DN-39-only** — the frozen kernel changes *only* via a DN-39 default-DENY promotion; any other kernel change is a `core 2.0.0` event (§6). *Prior framework status:* **Accepted** (2026-06-27) — *Proposed → Accepted as the freeze-criterion **framework**, **ratified by the maintainer 2026-06-27** (R5 gate); ratifying the framework did not itself declare the freeze (VR-5) — that is this 2026-07-02 enactment.* Prior: **Proposed** (2026-06-27; lang-design wave capstone synthesizing DN-52/ADR-033/DN-54/DN-55). |
| **Feeds** | the **`core 1.0.0` freeze** — extends **DN-39** (Kernel-Promotion-Review-KC3) with a *completeness* dimension and feeds the **ADR-021 / ADR-022** release-readiness gates (the kernel-freeze is a Gate-A-class criterion). Synthesizes the lang-design wave: **DN-52** (census), **ADR-033** (the last kernel-touching primitive), **DN-54** (user extensions), **DN-55** (polymorphism). |
| **Date** | June 27, 2026 |
| **Decides** | *Proposes, for ratification:* (1) the **thesis** — a minimal, eventually-**frozen** L0 kernel such that *every* construct a developer can write (including user pseudo-macros) lowers to it **transparently by construction**, so a black box is **impossible by design**, not merely discouraged; (2) the **completeness criterion** — every accept, every reject, every variant and invariant enumerated, with **zero silent gaps** on the parsable-vs-runnable frontier; (3) the **minimal primitive set** boundary (what the frozen kernel must contain, and the last open primitive); (4) the **freeze gate** — the concrete, checkable condition under which the kernel may be declared frozen. |
| **Task** | M-815 (lang-design wave capstone) |

> **Posture (transparency rule / VR-5 / G2).** This is a synthesis + criterion note. Every claim is
> tagged at its established strength: the *thesis* and the *freeze gate* are **`Declared`** (a design
> direction + a definition, not a theorem); the *evidence* it rests on is the wave's own honesty —
> DN-52's census is **`Empirical`** (an evidence-based read of the code, no silent gaps *found*, not
> *proven* absent), ADR-033's soundness is **open** (its FLAG-1), DN-55's mechanism is `Empirical`
> (three-way). Nothing is `Proven`. **The kernel is NOT frozen today** — this note defines *when it
> could be*. Saying otherwise would itself be the kind of overclaim the thesis exists to forbid.

## 1. The thesis

Mycelium's deepest invariant is **no black boxes** (G2 / SC-3). Most languages hold this as a
*value* — discouraged-but-possible. Mycelium's goal is stronger: make a black box **impossible by
construction**. That is achievable only if the **L0 kernel is complete and minimal**, so that:

> Every construct a developer can write — surface sugar, generics, traits, effects, `wild`, *and
> user-defined pseudo-macros* — **lowers to L0 transparently**, and the lowered L0 term is a
> first-class, `reveal`-able value. Because there is nothing *below* L0 to hide in, and nothing a
> developer can express that does *not* lower to it, **there is nowhere for a black box to exist.**

This unifies four rules already in the corpus into one goal:

- **KC-3** (small, auditable kernel) — the kernel is *minimal*.
- **The DN-38 lowering law** — every feature lowers to L0 with the same observable meaning; the
  kernel *never grows* for ergonomics.
- **The DN-50 frontier** — no *silent* accept-but-unrunnable; every accept either runs or is an
  explicit `Residual`.
- **No black boxes** (G2) — selections/conversions/lowerings are reified, inspectable, `EXPLAIN`/
  `reveal`-able.

The freeze is what makes the guarantee *structural* rather than *aspirational*: once the kernel is
frozen and the lowering surface above it is closed, "no black boxes" holds **by construction** for
all time, including for code and extensions not yet written.

## 2. The completeness criterion

The kernel is **complete** (a precondition of freeze) when **the accept/reject frontier is fully
enumerated with zero silent gaps**:

1. **Every accept is accounted.** Every construct the parser/checker accepts either (a) **runs**
   three-way (L1-eval ≡ L0-interp ≡ AOT), or (b) carries an **explicit `Residual`/FLAG** — never a
   silent accept-but-unrunnable. *(This is DN-50's OQ-1/OQ-2 ruling; DN-52 is the census.)*
2. **Every reject is accounted.** Every construct the language *forbids* is an **explicit, named
   refusal** (a `CheckError`/parse refusal/reject-corpus entry), never an accept-by-omission. The
   `reject/` conformance corpus is the ledger.
3. **Every variant and invariant is captured.** The value-model variants (`Repr`/`Payload`, the
   ten-node L1 budget — RFC-0007; the kernel prims — RFC-0032) and the invariants they must preserve
   (content-addressing/injectivity — ADR-003; never-silent swap — RFC-0002; totality — RFC-0007 §4.5)
   are each enumerated and checked.
4. **The lowering surface is closed.** Every surface feature has a *named, grammar-checked,
   semantics-preserving* lowering to L0 (DN-38), and **user-defined lowerings cannot escape it** —
   they are transparent surface→L0 rules under the same lowering law (DN-54), so even extensibility
   adds **no** gap.

The criterion is **`Empirical`/checked, not assumed**: it is satisfied by the *census* (§3) showing
no silent gap **and** the conformance corpus (accept + reject) covering the surface — not by a claim.

## 3. Wave evidence — where each leaf lands on the ledger

This wave was structured to advance exactly the four facets of §2:

| Facet | Wave deliverable | Result |
|---|---|---|
| Accepts have no silent gap (§2.1) | **DN-52 census** | **No silent-gaps found** across ~50 construct categories — every accept runs or hits an explicit `Residual`. Two rows **Undetermined** (Dense three-way; cross-nodule three-way — not yet in the differential corpus; not active gaps but **must be closed before freeze**). `Empirical`. |
| The last kernel-touching primitive (§3) | **ADR-033** | Dynamic dispatch — the **one** polymorphism form that cannot fully monomorphize — accounted as a deliberate, KC-3-significant `FieldSpec::Fn` extension. **Open: FLAG-1** (arity-only hashing could collide two function types) — a **soundness item that must be resolved before this primitive can be frozen.** |
| Extensibility adds no gap (§2.4) | **DN-54** | User pseudo-macros are transparent surface→L0 lowerings (`reveal`-able by construction), held to the same verification as built-in passes — so extensibility is *inside* the closed lowering surface, not a hole in it. |
| Polymorphism on the ledger (§2.3) | **DN-55** | Polymorphism (`static specialization`) costs **zero kernel primitives** — it erases entirely in the frontend. The only polymorphism form touching the kernel is the dynamic-dispatch `FieldSpec` (ADR-033). |

The net picture: the kernel-touching surface of the *entire* language is small enough to enumerate,
and this wave closed most of it. What remains before freeze is short and named (§5/§7).

## 4. The minimal primitive set (the freeze boundary)

The frozen kernel is the **trusted base** (`mycelium-core`) + the **L1 ten-node calculus**
(RFC-0007) + the **kernel prims** (RFC-0032 D1/D2: comparison + binary/ternary arithmetic) + the
**value representations** (RFC-0032 D3/D4: `Repr::Seq`/`Repr::Bytes`; the width-cast prim DN-41).
Everything else is **frontend** that lowers away (generics/polymorphism — DN-55, zero primitives;
traits — dictionary-free static resolution, M-673; sugar/objects/derive — DN-38/DN-53/DN-54). The
**one candidate addition** still on the table is **`FieldSpec::Fn`** (ADR-033) for dynamic dispatch —
the last deliberate, sign-off-gated trusted-base growth. **The freeze boundary is: this set, plus
ADR-033 if-and-only-if its soundness is established, and nothing more.**

## 5. The freeze gate

The kernel may be declared **frozen** (`core 1.0.0`-class) when **all** of the following hold —
each checkable, none asserted:

1. **Census green (§2.1):** the DN-52 census shows **zero silent gaps**, with **no `Undetermined`
   rows** (the Dense and cross-nodule three-way rows closed), backed by the **narrow standing gate**
   (DN-50 OQ-2: a test over the accept corpus asserting accept ⇒ runs OR explicit `Residual`).
2. **Reject ledger complete (§2.2):** the `reject/` conformance corpus covers every forbidden
   construct with an explicit named refusal.
3. **Primitive set closed (§4):** no open kernel-primitive question remains — in particular
   **ADR-033's FLAG-1 soundness is resolved** (a checked soundness argument, or the type-descriptor
   variant), so `FieldSpec::Fn` is either in (sound) or out (deferred), not pending.
4. **Lowering surface closed (§2.4):** every surface feature (post the RFC-0037 grammar wave) has a
   named, verified lowering; user extensions (DN-54) are checked transparent-by-construction.
5. **KC-3 review passed (DN-39):** the kernel-promotion review confirms minimality + auditability,
   now *plus* this completeness dimension.

Until **all five** hold, the kernel stays **open** (and honestly so). This gate **extends DN-39** and
is a **Gate-A-class** input to **ADR-021/ADR-022** (it is part of "the kernel stabilizes").

## 6. What freeze buys (and what it forbids)

Post-freeze: the kernel is **immutable** — no new L0 node, no new primitive (the ten-node budget +
the ratified prim set are final). Every future feature is a **frontend lowering** over the frozen
kernel; **no feature, and no user extension, can grow it** (KC-3 becomes an invariant, not a
target). At that point **"no black boxes" is guaranteed by construction**: there is nothing below
L0, nothing that fails to lower to it, and nothing un-`reveal`-able — so a black box is not merely
discouraged, it is *unexpressible*. (Changing the frozen kernel thereafter is a `core 2.0.0` event —
a supersession, never an in-place growth; house rule #3.)

## 7. What remains before freeze (open, named — not hidden)

- **ADR-033 FLAG-1** — the `FieldSpec::Fn` soundness/hashing question (the single open *primitive*).
- ~~**DN-52 `Undetermined` rows** — Dense three-way + cross-nodule three-way must enter the
  differential corpus and resolve to *runs* or *explicit-Residual*.~~ **RESOLVED (W5/freeze-ledger,
  2026-06-27):** Dense → `Explicit-Residual` (elab.rs fix + `differential.rs::dense_swap_is_an_explicit_residual_on_all_paths`);
  cross-nodule → `Runs` (`differential.rs::cross_nodule_program_runs_three_way`). DN-52 §5 updated.
- ~~**The narrow standing gate** (DN-50 OQ-2) — to be wired over the accept corpus.~~ **RESOLVED
  (W5/freeze-ledger, 2026-06-27):** wired as `crates/mycelium-l1/tests/runnable_gate.rs::every_accepted_construct_elaborates_to_ok_or_explicit_residual`
  — a representative data-driven table over construct categories, green. `Empirical`.
- **RFC-0037 ratification + the grammar-supersession migration** — closes the *surface* side of the
  lowering surface.
- **DN-54's extension-checker** — the mechanism that *enforces* transparent-by-construction on user
  lowerings (so §2.4 is checked, not assumed).
- The reject-corpus completeness audit (§2.2).

None of these is a black box; each is an explicit, named, tracked item — which is itself the thesis
in action: the *path to* a no-black-box kernel is, fittingly, fully enumerated and never silent.

## 8. Grounding

- **DN-39** (Kernel-Promotion-Review-KC3) — the KC-3 review this extends with a completeness
  dimension. **ADR-021 / ADR-022** — the release-readiness gates this feeds. **RFC-0007** (the L1
  ten-node budget — the kernel's node set). **RFC-0032** (the kernel prims + value reps). **DN-41**
  (the width-cast prim). **DN-38** (the lowering law). **ADR-003** (content-addressing/injectivity —
  the invariant the primitive set must preserve, the crux of ADR-033 FLAG-1). **RFC-0002** (never-
  silent swap). Wave: **DN-50/DN-52** (frontier + census), **ADR-033** (dynamic dispatch),
  **DN-54** (extensions), **DN-55** (polymorphism). House rules: **KC-3**, **G2**, **VR-5**.

## 9. The freeze declaration (M-969 — 2026-07-02)

**The kernel is declared frozen** (`core 1.0.0`-class), 2026-07-02, as the closing act of Phase-I.
This is a `Declared` decision resting on `Empirical` evidence (VR-5 — not a `Proven` claim of a
theorem-complete kernel); it records that the §5 gate's five conditions are all met and checked, and
fixes the post-freeze diff policy.

**The five §5 conditions, each green with cited basis:**

1. **§5.1 census / never-silent floor** — satisfied (W5): `runnable_gate.rs::every_accepted_construct_elaborates_to_ok_or_explicit_residual` green; DN-52 census has no silent gaps *found*.
2. **§5.2 reject-ledger** — DN-80 (the unified `{construct, reason, alternative}` ledger over all four reject strata) + the M-959 regression guard (`crates/mycelium-std-conformance/tests/reject_ledger.rs`, 9/9 green).
3. **§5.3 primitive set closed** — Π = 38 named prims; ADR-033 FLAG-1 (dynamic-dispatch soundness) dispositioned IN by DN-74; the previously-scheduled `vsa.*` (M-892…894) and Gap-E (M-902…904) additions landed and are `done`.
4. **§5.4 lowering surface closed** — RFC-0037 Enacted; the DN-54 extension surface (`lower`/`derive`) is checked (RHS-elab + §4.1 IL-grammar + §4.2 acyclicity + §6 KC-3 guard + §7 discipline all landed, verified by DN-75); DN-71/DN-73/DN-74 resolved; the DN-54 §10 attachment model Accepted (Model A, DN-81) and enacted (M-973); grammar baseline in sync (M-924).
5. **§5.5 KC-3 completeness review** — **passed** (run 2026-07-02 via the DN-39 machinery, recorded in DN-76 §5A): `runnable_gate` + reject-ledger green, DN-39 §7 boundary unchanged, every Π growth since DN-39 traces to a ratified gate (no silent kernel growth), DN-54 extensions add no L0 node by construction.

**Independent scorecard:** DN-76 §5A re-scored all four open conditions against `integration` tip
`81fa519` — **4 of 4 green** (assessed independently of the wave that closed them, to guard against
completion bias — house rule #4).

**Post-freeze diff policy (what freeze forbids — §6):** the frozen kernel (the `mycelium-core`
trusted base + the L1 ten-node calculus + the ratified Π = 38 prim set) changes **only** via a
**DN-39 default-DENY promotion** (a reviewed, evidenced, deliberate kernel-growth act). Any other
change to the frozen kernel is a **`core 2.0.0`** event, not a patch. Every future language feature is
a **frontend lowering** over the frozen kernel — a black box is not merely discouraged, it is
*unexpressible* by construction (§1).

**What the freeze does NOT claim (VR-5):** it is not a proof the kernel is bug-free or that no
undiscovered construct escapes the census — the census/KC-3 verdicts are `Empirical` (evidence-based,
no gap *found*), not `Proven`. It is a checked, deliberate declaration that the kernel is complete and
stable enough to be a fixed base a public release can stand on, with an explicit, auditable change
policy thereafter.

## Changelog

| Date | Status | Note |
|---|---|---|
| 2026-06-27 | **Proposed** | Authored as the lang-design wave capstone (M-815). States the kernel-completeness criterion (every accept/reject/invariant enumerated, zero silent gaps, closed lowering surface incl. user extensions) and the **freeze gate** (five checkable conditions). Synthesizes DN-52 (census: no silent gaps), ADR-033 (the last kernel-touching primitive + its open soundness), DN-54 (extensions lower by construction), DN-55 (polymorphism zero-primitive). Extends DN-39; feeds ADR-021/022. **Enacts no code; declares no freeze** — defines when one is permissible (VR-5/G2). The kernel is not frozen today; §7 lists what remains, all named and tracked. |
| 2026-06-27 | **Proposed** (updated) | W5/freeze-ledger: two §7 open items **RESOLVED**. (1) DN-52 `Undetermined` rows closed — Dense: elab.rs Expr::Swap guard + `differential.rs::dense_swap_is_an_explicit_residual_on_all_paths` → `Explicit-Residual`; cross-nodule: `differential.rs::cross_nodule_program_runs_three_way` → `Runs`. (2) DN-50 narrow standing gate wired — `runnable_gate.rs::every_accepted_construct_elaborates_to_ok_or_explicit_residual` (data-driven table, `Empirical`). Remaining in §7: ADR-033 FLAG-1, RFC-0037 migration, DN-54 extension-checker, reject-corpus audit. The kernel is still **not frozen** — §5 freeze-gate conditions #1 and the standing gate are now satisfied; #2–5 remain open. |
| 2026-07-02 | **Enacted** | **THE KERNEL FREEZE IS DECLARED (M-969)** — the closing act of Phase-I. All five §5 conditions green with cited `Empirical` basis (§9): census (W5) · reject-ledger (DN-80 + M-959 guard, 9/9) · primitive set closed (Π = 38; ADR-033 FLAG-1 dispositioned IN by DN-74; `vsa.*`/Gap-E landed) · lowering surface closed (RFC-0037 Enacted; DN-54 surface checked; DN-71/73/74 resolved; DN-54 §10 Model A enacted via M-973; grammar baseline M-924) · KC-3 completeness review **passed** (DN-39 machinery, 2026-07-02). Independent scorecard **DN-76 §5A: 4/4 green** (assessed against `integration` 81fa519, independent of the closing wave — house rule #4). Post-freeze diff policy: **DN-39-only** (any other kernel change is a `core 2.0.0` event, §6). Status → **Enacted** (steps through the 2026-06-27 `Accepted` framework — house rule #3). `Declared` act on `Empirical` evidence; **not** a `Proven`-complete-kernel claim (VR-5). |
| 2026-07-03 | **Enacted** (policy addendum, append-only) | **§6 diff policy gains a third, scoped channel: within-freeze behavior-preserving hardening** (RFC-0041 §6, Accepted/maintainer-ratified). The `SIGABRT`-on-deep-input recursive `Drop`/`Clone`/`Canon` on the frozen core types is a *never-silent (G2) unsoundness the freeze currently protects*; RFC-0041 admits **recursion→iteration destruction/traversal transforms on existing frozen types** (no new type/variant/field, no observable value/error/order change — I1–I3) under an explicit bar: M-210 **+** the new error-parity differential green, mutation-witnessed. This is **narrower** than DN-39 (which admits a *new component into the TCB*) and does **not** loosen the `DN-39-only`/`core 2.0.0` rule for anything else — it names the one transform class that *restores* the never-silent guarantee the freeze assumes. Ratified 2026-07-03; enacts per-wave with RFC-0041 W3. Append-only (VR-5/house rule #3). |
| 2026-07-06 | **Enacted** (posture clarification, append-only) | **Maintainer clarification (owner, 2026-07-06): pre-production, the freeze is a *discipline gate*, not a delivered-trusted-core immutability guarantee.** The project has not hit a production release, so the maintainer authorizes kernel modifications **where sound and with the project's overall intent in mind** — routed through the same decision review (a recorded maintainer authorization per change), never silently. The freeze's *process* stands unchanged: changes still arrive via DN-39, the RFC-0041 §6 behavior-preserving channel, or an explicit per-change maintainer authorization recorded in the decision docs — this row makes the third path's basis explicit rather than implied. First uses: the **M-996** AOT env-machine TCO decision (behavior-changing `DepthLimit→FuelExhausted` for divergent tail loops, approved — n.b. `mycelium-mlir` is outside freeze *scope*; the authorization there is the behavior-change decision itself) and **M-997** (evaluation of structural sharing inside the frozen `mycelium_core::Datum` — minted as a tracked option, deliberately **not** exercised absent a measured driver; KC-3 favors holding the authorization in reserve). Status stays **Enacted**; append-only (VR-5/house rule #3). |
