# ADR-042 — Rust-Base Freeze + Mycelium-First Expansion ("the last new Rust is `mycelium-tero`; everything new is `.myc`")

| Field | Value |
|---|---|
| **ADR** | 042 |
| **Status** | **Accepted** (2026-07-07 — maintainer directive, in force going forward). Records the **Rust-base freeze + Mycelium-first expansion** policy: **no new Rust enters the Mycelium *language project* itself** from here on — the existing Rust kernel stays the trusted base (DN-39 boundary unchanged), and **new language-project functionality is authored in `.myc`**. One sanctioned exception: **`mycelium-tero`**'s Rust PoC/MVP (M-1015→M-1018) may finish in Rust, then is rewritten into Mycelium (M-1019) — the *last* new Rust in this project. **Aspiration (`Declared`): a wild-free core** — minimise `wild`/FFI, use it only where strictly necessary; "if Mycelium is capable of a fully wild-free core, that is the goal." **Enacts nothing** — the freeze and the port are ongoing work; `Accepted → Enacted` only when the language-project surface is Rust-frozen in fact and the `.myc` expansion + tero rewrite have landed (house rule #3 — must step through Accepted first). |
| **Decides** | (1) **Rust-base freeze.** No *new* Rust code is added to the Mycelium language-project surface (language features, stdlib, frontend) going forward; the existing Rust **kernel stays the trusted base** — this is a freeze on *expanding* the Rust surface, **not** a change to the DN-39 kernel boundary and **not** a ban on kernel bug-fix/maintenance. (2) **Mycelium-first expansion.** New language-project functionality is authored in **`.myc`**; the `docs/planning/DN-26` migration of the *existing* Rust frontend proceeds per RFC-0016 §4.6 / ADR-038 §2.3 unchanged. (3) **The `mycelium-tero` exception.** Its Rust PoC/MVP (M-1015→M-1018, epic E39-1) is allowed to finish in Rust and is then **rewritten into Mycelium (M-1019)** — explicitly the *last* new Rust admitted to this project. (4) **Wild-free core is the target.** `wild`/FFI blocks are minimised — used **only where strictly necessary** — to maximise the language's own safety while retaining maximal performance; a fully wild-free core is the stated **goal** (`Declared` — a preference/target, feasibility not yet proven). |
| **Relates / hardens** | **Hardens RFC-0016 §4.6** ("Rust-first now, Mycelium-lang eventually") into a forward *freeze* on new Rust surface (the §4.6 migration of already-landed Rust is unchanged). **Sharpens ADR-038's North Star** — *"Rust where appropriate, Mycelium everywhere else"* — for *new* work: everywhere new is Mycelium-first; ADR-038 §2.1–§2.3 (tag/release split, parallel dogfooding, differential/replace-on-satisfaction) and §2.5 (transpiler progressive hardening — which this ADR promotes to first-class) are **unchanged**. **DN-39 kernel boundary UNCHANGED** — the frozen kernel *is* the DN-39 TCB (L0 Core IR + reference interpreter + content-addressing primitive + guarantee lattice + swap engine); KC-3 held, no promotion/demotion. Feasibility base: **DN-14** (all 11 self-hosting gate rows `present`, incl. row 9 `wild`/FFI executes). Reduces the mirror burden of **DN-26 §7.2 / the L0-boundary decision** (Decision 2 in the companion append). **Does not amend** ADR-007 (kernel-in-Rust strategy), ADR-022/036 (the 1.0.0 tag + dogfooding split), or ADR-041 (MSRV). |
| **Grounds** | Maintainer directive (2026-07-07, this session — the doctrine recorded verbatim in intent; `Declared` until this ADR is `Accepted`, at which point it binds); **RFC-0016 §4.6** (the migration trajectory this hardens); **ADR-038** (the North Star + transpiler doctrine + Phase-I/II sequencing this sharpens); **DN-39 §7** (the kernel boundary held UNCHANGED — the freeze does not move it); **DN-14** (the self-hosting-gate evidence that `.myc` can now carry new functionality); **DN-26** (the bootstrap plan whose mirror burden this reduces; its L0-boundary Option-A pick is the companion decision); **DN-87 / E39-1** (`mycelium-tero`, M-1015 `done`, M-1016/17/18 the Rust PoC, M-1019 the `.myc` rewrite — the sanctioned exception); **DN-34 / DN-85 / DN-86 / M-1006** (the transpiler ladder this makes first-class); KC-3, G2, VR-5 (small trusted base; never-silent; no claim above its checked basis). |
| **Date** | 2026-07-07 |

> **Posture (transparency rule / VR-5 / house rule #4).** This ADR records *governance/posture*, authored
> from the maintainer's explicit 2026-07-07 direction. It **asserts no implementation progress**: no Rust
> is deleted, no module is declared ported, no guarantee tag is upgraded by this document. The **wild-free
> core** is a **`Declared` goal**, not an achieved or proven property — and some seams (notably the single
> "materialise the finished L0" kernel crossing named in the DN-26 L0-boundary decision) **may prove to
> require `wild`**; this ADR fixes the *preference/target*, not a feasibility claim (VR-5 — not upgraded
> past its basis). One genuine tension is surfaced, not buried (§4): a few ADR-038 Phase-I usability
> enablers are Rust-/kernel-side — the ADR states how the freeze reads against them and **FLAGs** the
> residue for the maintainer rather than silently deciding it (G2).

---

## 1. Context

Three ratified positions set the frame, and the maintainer's 2026-07-07 direction hardens the forward
edge of all three:

- **RFC-0016 §4.6 — "Rust-first now, Mycelium-lang eventually" (M-346).** The stdlib/library migration
  order: every module lands **Rust-first** (Phase 5a, the trusted toolchain), then migrates to Mycelium
  behind an NFR-7-style differential (Phase 5b). As written, §4.6 governs the *order* of migration but
  leaves the door open to *new* Rust surface accruing indefinitely ahead of the port.
- **ADR-038 — "Rust where appropriate, Mycelium everywhere else."** The North Star: pragmatic, progressive
  dogfooding, not zero-Rust dogmatism. Rust stays wherever it is the appropriate tool (trusted base, FFI
  floor, performance kernels); the Mycelium rewrite of existing Rust proceeds module-by-module,
  differential-validated (§2.3), with the transpiler an **accelerant, not a gate** (§2.5).
- **DN-39 — the KC-3 kernel boundary, held UNCHANGED.** The trusted core is L0 Core IR + the reference
  interpreter + the content-addressing primitive + the guarantee lattice + the swap engine; "the kernel
  never grows without a net trust reduction," and the 2026-06-26 promotion review admitted **zero**
  promotions on merit.

The maintainer's direction resolves the forward-looking ambiguity these leave open. RFC-0016 §4.6 tells
you the *order* to migrate what already exists; it does not stop the Rust surface from *growing*. ADR-038
says "Rust where appropriate" but does not fix, for *new* functionality, whether new Rust is still
"appropriate." The direction is: **stop growing the Rust base.** The existing kernel — the DN-39 TCB — is
the appropriate, trusted Rust and it stays; but **new language-project functionality is written in
Mycelium**, so effort concentrates on expanding the `.myc` base rather than widening a Rust surface that
then has to be ported back. DN-14's self-hosting-gate verdict (all 11 rows `present`, `wild`/FFI executes)
is the evidence that the surface can now *carry* new functionality — the precondition RFC-0016 §4.6's
Phase-5b and ADR-038's Phase II both waited on.

## 2. Decision

### 2.1 Rust-base freeze — no new Rust in the language project

**No new Rust code enters the Mycelium *language project* itself going forward.** The existing Rust
**kernel stays the trusted base** — DN-39's boundary is **unchanged** (L0 Core IR + reference interpreter +
content-addressing + guarantee lattice + swap engine). This freeze is specifically about **not expanding
the Rust surface** with new language features or stdlib; it is:

- **not** a change to the DN-39 kernel boundary (nothing is promoted into or demoted out of the TCB);
- **not** a ban on **maintenance** of the existing Rust kernel/frontend — bug-fixes, soundness repairs,
  and the completion of *already-ratified* kernel capability are ongoing maintenance, not new-surface
  expansion (the same "outside ongoing maintenance and future-dev integration" carve-out house rule #3
  applies to `Enacted`, and DN-39/ADR-041 apply to the kernel);
- **not** retroactive — the existing Rust base is untouched and migrates per RFC-0016 §4.6 / ADR-038 §2.3
  on its own schedule.

### 2.2 Mycelium-first expansion — new functionality is `.myc`

New language-project functionality is **authored in `.myc`**. The rationale, recorded faithfully:

1. **Concentrate effort on the `.myc` base.** Stopping the growth of the Rust base shifts focus to
   expanding the Mycelium base — the language eats its own cooking on *new* work, not only on a lagging
   back-port.
2. **Reduce the mirroring burden.** Fewer *new* kernel/Rust types means fewer types to mirror in-language.
   This directly eases the **Option A in-language mirror model** chosen for the DN-26 L0
   `Value`/`Repr`/`FieldSpec` boundary (the companion decision): the mirror's standing maintenance tax
   (a hand-kept copy that must grow when a kernel type grows — DN-26 L0-boundary §2.3) is bounded *by
   construction* when the kernel type-set is frozen against new additions.
3. **Systematise the port.** The manual, hand-port work is to be **parameterised, idempotent,
   systematically automated and schematised in the transpiler** (DN-34 / DN-85 / DN-86; the M-1006
   ladder), so future porting is *systematic*, not artisanal. This makes ADR-038 §2.5's "progressive
   hardening" of the transpiler a **first-class** workstream, not merely an accelerant.

### 2.3 The one sanctioned exception — `mycelium-tero`

`mycelium-tero`'s Rust PoC/MVP is the **single** exception. Its Layer-1 index (M-1015) has **landed**; the
query engine, API fronts, and VSA layer (M-1016/M-1017/M-1018, epic E39-1) may **finish in Rust**. Once
proven out, the engine's core is **rewritten into Mycelium** (M-1019 — already `status:blocked` on M-993
by design, the `.myc` dogfood milestone). `mycelium-tero` is thereby the **last new Rust admitted to this
project**.

**Tooling is instrument, not language-project surface — and is *not* frozen.** `mycelium-transpile` (and
the DN-86 multi-front-end ingestion, the index/gate scripts, and the general check tooling) is a build
*instrument*, not part of the Mycelium language the project ships. The freeze does **not** apply to it —
indeed the maintainer explicitly wants the transpiler **improved** (§2.2 point 3). The line the freeze
draws is: *the language project's own kernel/stdlib/frontend surface* (frozen against new Rust) vs.
*instruments that build or serve it* (free to evolve in whatever language fits).

### 2.4 Wild-free core is the goal

`wild`/FFI blocks (the denied-by-default unsafe/FFI escape, admitted only in an `@std-sys` nodule under
`!{ffi}` — DN-14 row 9; lexicon §5/§7) are to be **minimised — used only where strictly necessary** — so
the language's own safety is maximised while retaining maximal performance. **"If Mycelium is capable of a
fully wild-free core, that is the goal."**

This **refines Decision 2 (the DN-26 L0-boundary Option-A pick):** even the *single* "materialise the
finished L0" `wild` crossing that Option A reserves is to be **scrutinised for a wild-free path** before it
is spent — `wild` is the last resort, not the default seam. **Honesty (`Declared`, VR-5):** this is a
**target**, not a proven-reachable state. The self-hosted frontend must still hand its finished L0 to the
trusted kernel to *run* it, and DN-14 row 9 confirms `wild` is the mechanism for exactly that callback; it
is entirely possible that the materialisation crossing (or an I/O floor) **cannot** be made wild-free
without moving the kernel boundary (which DN-39 forbids). The ADR sets the *preference* — wild-free where
achievable, `wild` only where strictly necessary, each site audited (`just safety-check`: `@std-sys` +
`!{ffi}` + `// SAFETY:`) and never silent (G2). Whether a **fully** wild-free core is achievable is an
open question (§6 OQ-1), not a claim.

## 3. Consequences

- **The `.myc` expansion + the port become the critical path.** With no new Rust surface, the project's
  forward progress *is* the Mycelium-first authoring and the DN-26 back-port. This is the intended effect
  (concentrate on the `.myc` base), and it makes the transpiler-systematisation (§2.2.3) load-bearing
  rather than optional.
- **The DN-26 mirror model gets cheaper over time, not more expensive.** Freezing new kernel/Rust types
  bounds the Option-A mirror's drift tax by construction: the mirror only ever chases a **frozen**,
  append-only, frozen-tag target (DN-26 L0-boundary §2.3), so the strongest argument *against* Option A
  (the standing duplication cost) is materially weakened by this ADR.
- **Existing Rust migrates unchanged; nothing is deleted here.** RFC-0016 §4.6 Phase-5b and ADR-038 §2.3
  (differential/replace-on-satisfaction) still govern how the *existing* Rust kernel/frontend/stdlib moves
  to Mycelium; the freeze only forbids *adding* to that surface. Compiler self-hosting remains ADR-038
  §2.3's deferred, doubly-conditional aspiration — the freeze does not force it faster, it only ensures no
  *new* Rust is added to the pile that would eventually need porting.
- **The transpiler is first-class.** ADR-038 §2.5 kept the transpiler an "accelerant, not a gate"; this
  ADR keeps that posture but elevates *improving* it (parameterised, idempotent, schematised porting) to a
  named, first-class workstream (DN-85/DN-86/M-1006), because the port is now the critical path.
- **`mycelium-tero` finishes in Rust, then dogfoods.** E39-1 proceeds on its Rust PoC through M-1018 and
  closes the loop at M-1019 (`.myc` rewrite) — a bounded, named exception with an explicit terminal, not
  an open-ended license.
- **A genuine tension with ADR-038's Phase-I enablers — FLAGGED, not silently resolved (§4).** A few
  below-grammar usability enablers on ADR-038's Phase-I list are Rust-/kernel-side; how the freeze reads
  against them needs the maintainer's call (see §4 + §6 OQ-2), not this ADR's guess (G2/VR-5).
- **Wild-free is a direction the codebase now optimises toward** — new `.myc` authoring prefers safe
  constructs and treats each `wild` site as a cost to justify, but the ADR makes no claim that zero `wild`
  is reachable (§6 OQ-1).

## 4. The Phase-I-enabler tension (surfaced, per house rule #4)

ADR-038 §2.2 / §5 lists **below-grammar enablers** for Phase-I functional usability — binary integer
arithmetic + signed ops, dense/vsa prim surfacing, the scalar-float `Repr` (ADR-040, already `Enacted`),
`Substrate`/`consume` execution, an R2-lite runtime-vocabulary subset, plus `myc run` / a textual string
literal / `hash.*` surfacing. **Several of these are Rust-/kernel-side work** (prim surfacing in
`mycelium-interp`, `Substrate` execution in the kernel, runtime vocabulary). Read naively, closing them
looks like "new Rust," which the freeze forbids.

The honest reading this ADR records (and the residue it FLAGs):

- **Completing *already-ratified* kernel capability is maintenance, not new-surface expansion.** ADR-040's
  scalar-float `Repr` is the precedent: it landed via the DN-38 §2.6 double gate + a DN-39 promotion
  review (DN-69) and is `Enacted`. Finishing a *ratified* enabler (its own ADR/DN cleared) is completing
  the committed trusted base, on the maintenance side of §2.1's carve-out — **not** the "new language
  features/stdlib" the freeze targets.
- **New, unratified kernel surface now meets a higher bar.** Any *new* enabler that is not already ratified
  should now be evaluated **against the freeze** — prefer the `.myc`/wild-free path where the surface can
  carry it (DN-14), and where a kernel prim genuinely is required, it goes through the existing DN-39
  promotion bar with the freeze as an explicit thumb on the scale toward "author it in `.myc` instead."
- **This ADR does not adjudicate the individual enablers.** Which remaining Phase-I enablers are
  "ratified-maintenance" vs "new-surface-needing-a-decision" is a **maintainer call** (§6 OQ-2), recorded
  here rather than silently pre-scoped. The freeze's *intent* is clear (stop growing the Rust base); its
  application to the in-flight Phase-I list is flagged, not guessed.

## 5. Definition of Done

**For this ADR (the decision record):**

- [x] Maintainer directive recorded faithfully: the four-point decision (freeze · Mycelium-first · the
  `tero` exception · wild-free goal), `Status: Accepted` (2026-07-07, in force going forward).
- [ ] Indexed in `docs/adr/README.md` and `docs/Doc-Index.md`; `CHANGELOG.md` records the decision
  *(orchestrator-owned — proposed in the companion report, applied by the integrating parent)*.
- [ ] The companion **DN-26 L0-boundary Option-A** decision appended (the mirror-burden reduction this ADR
  cites is grounded in that pick).
- [ ] `Accepted → Enacted` **only when** (never by ratification alone — house rule #3): the language-project
  surface is Rust-frozen in fact (no new Rust language features/stdlib have landed outside the `tero`
  exception + ratified-maintenance); the `mycelium-tero` rewrite (M-1019) has landed; and the Phase-I
  enabler residue (§4 / §6 OQ-2) has been adjudicated. Each condition is a *checked* basis, not an intent.

**For the policy it enacts (the standing gate, checked continuously):**

- [ ] New language-project functionality lands as `.myc`, not new Rust (auditable at PR review — the
  `/pr-review` transparency lens gains a "no new language-project Rust surface" check for the freeze's life).
- [ ] `wild` sites are justified per-occurrence (`just safety-check`), minimised, and the wild-free-core
  goal tracked (§6 OQ-1) — never silently expanded (G2).
- [ ] The transpiler-systematisation workstream (DN-85/DN-86/M-1006) is resourced as first-class.

## 6. Open questions

- **OQ-1 — Is a *fully* wild-free core achievable?** The wild-free-core aspiration is `Declared`. The
  self-hosted frontend must hand its finished L0 to the trusted kernel to run it (DN-14 row 9's `wild`
  callback; the DN-26 L0-boundary "materialise the finished L0" crossing), and an I/O floor may
  irreducibly need `wild`. Whether these can be made wild-free **without** moving the DN-39 boundary (which
  is forbidden) is unproven. Track the residual `wild`-site count toward zero; record honestly if a floor
  proves irreducible (VR-5 — no upgrade of the goal to a claim without a checked basis).
- **OQ-2 — Which in-flight ADR-038 Phase-I enablers are ratified-maintenance vs new-surface?** (§4.) The
  freeze's application to the below-grammar enabler list (dense/vsa prims, `Substrate`/`consume`
  execution, R2-lite runtime) is a maintainer adjudication — this ADR flags it, does not decide it.
- **OQ-3 — Does the freeze need a `/pr-review` or CI lens to hold "by construction"?** Mitigation-style,
  the freeze is currently a policy an agent must remember; a lightweight "no new language-project Rust
  surface" review check (analogous to the branch-guard) would make it hold by construction. Proposed,
  not built here.

## 7. Alternatives considered

- **Keep RFC-0016 §4.6 as written (Rust-first, migrate eventually; new Rust still permitted).** Rejected by
  the maintainer: it lets the Rust surface keep growing ahead of the port, enlarging the eventual back-port
  and the mirror-maintenance burden — the opposite of concentrating effort on the `.myc` base.
- **Zero-Rust immediately (rewrite the kernel too).** Rejected: it violates DN-39 (the kernel is the
  trusted differential oracle; KC-3) and ADR-007 (Rust kernel as trusted base). The freeze deliberately
  *keeps* the trusted Rust and only stops *new* Rust — pragmatic dogfooding (ADR-038), not dogmatism.
- **No `tero` exception (force `mycelium-tero` into `.myc` now).** Rejected: the memory substrate's Rust
  PoC de-risks the DN-87 §6 improved-on-RAG bet before the `.myc` port; M-1019 already gates the rewrite on
  M-993 maturity. A single bounded exception with an explicit terminal is cheaper and more honest than
  forcing a premature port.
- **Leave the wild-free goal unstated (minimise `wild` only implicitly).** Rejected: naming it as the
  explicit target — while tagging it `Declared` and flagging the possibly-irreducible seams — is the
  never-silent posture (G2); an unstated preference cannot be audited against.

## 8. Grounding / honesty

- Maintainer directive, 2026-07-07 (this session) — the doctrine §2 records; `Declared` until this ADR is
  `Accepted`, at which point it is the project's ratified forward policy.
- RFC-0016 §4.6 — the migration trajectory hardened here (its Phase-5b differential discipline unchanged).
- ADR-038 §2.1–§2.3/§2.5 — the North Star + differential-replace + transpiler doctrine sharpened, not
  amended; ADR-007/022/036/041 untouched.
- DN-39 §7 — the kernel boundary held UNCHANGED; KC-3; no promotion/demotion by this ADR.
- DN-14 — the self-hosting-gate evidence (all 11 rows `present`, row 9 executes) that `.myc` can carry new
  functionality; the `wild`/`@std-sys`/`!{ffi}` mechanism cited for §2.4.
- DN-26 — the bootstrap plan whose mirror burden §2.2.2 reduces; the L0-boundary Option-A pick is the
  companion decision appended in the same wave.
- DN-87 / E39-1 (`mycelium-tero`: M-1015 `done`, M-1016/17/18 the Rust PoC, M-1019 the blocked `.myc`
  rewrite) — the sanctioned exception, checked against `issues.yaml` (2026-07-07).
- DN-34 / DN-85 / DN-86 / M-1006 — the transpiler ladder made first-class by §2.2.3.
- KC-3, G2, VR-5 — small trusted base grows only via DN-39 review; nothing silent; no claim (including the
  wild-free-core goal and this ADR's own status) above its checked basis.

---

## Meta — changelog

- **2026-07-07 — Accepted (maintainer directive).** Records the **Rust-base freeze + Mycelium-first
  expansion** policy: no new Rust in the language project (kernel stays the trusted base, DN-39 boundary
  unchanged); new functionality authored in `.myc`; the mirror burden of the DN-26 L0-boundary reduced;
  the hand-port systematised in the transpiler (first-class). One sanctioned exception — `mycelium-tero`'s
  Rust PoC (M-1015→M-1018) finishes in Rust then is rewritten to Mycelium (M-1019), the last new Rust.
  **Wild-free core is the goal** (`Declared` — a target, not a proven property; some seams may prove to
  need `wild`, flagged). Hardens RFC-0016 §4.6, sharpens ADR-038's North Star; DN-39 boundary UNCHANGED;
  amends nothing (ADR-007/022/036/041 untouched). **Enacts nothing** — `Accepted → Enacted` only when the
  surface is Rust-frozen in fact + the tero rewrite lands + the Phase-I enabler residue is adjudicated
  (house rule #3). One tension surfaced (Phase-I enablers are partly Rust-side — §4, OQ-2) rather than
  guessed. Doc-Index / README / CHANGELOG rows owned by the integrating parent. (VR-5 / G2 / house rule #4.)
