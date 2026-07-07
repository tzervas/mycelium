# ADR-042 — Rust-Base Freeze Now, Full Mycelium Self-Hosting (Kernel Included) by Decomposition ("zero foreign first-party languages")

| Field | Value |
|---|---|
| **ADR** | 042 |
| **Status** | **Accepted** (2026-07-07 — maintainer directive, in force going forward). Records the **Rust-base freeze + Mycelium-first expansion** policy over **two time horizons**: **(a) NOW** — *freeze new Rust surface* (no new Rust language features/stdlib enter the language project; new functionality is authored in `.myc`); **(b) END-STATE** — the **entire first-party project, INCLUDING the kernel, is rewritten to `.myc`**: **zero foreign first-party languages by the DN-88 component-repo decomposition gate.** The Rust kernel is the **current** trusted base, itself **slated for a `.myc` rewrite** as the deepest, last self-hosting step — **not** a permanent fixture. One sanctioned near-term exception: **`mycelium-tero`**'s Rust PoC/MVP (M-1015→M-1018) may finish in Rust, then is rewritten to Mycelium (M-1019). **North Star (`Declared`): fully-native Mycelium everything** — the goal reaches the **full toolchain, the AOT/codegen backend included** (the current MLIR/LLVM path is itself slated for a native-Mycelium codegen-backend replacement — a far-future `needs-design` track, §3 — not a permanent external dependency); the only foreign residue in the terminal state is the **irreducible OS/hardware ABI seam** (syscalls, hardware intrinsics), where the **minimal-`wild`** wild-free goal bottoms out. **Aspiration (`Declared`): a wild-free `.myc` kernel** — minimise `wild`/FFI, use only where strictly necessary. **Enacts nothing** — the freeze, the port, and the kernel rewrite are ongoing work; `Accepted → Enacted` only when the whole first-party project (kernel included) is `.myc` and zero foreign first-party languages remain at the decomposition gate (house rule #3 — must step through Accepted first). |
| **Decides** | (1) **Rust-base freeze (NOW).** No *new* Rust code is added to the Mycelium language-project surface (language features, stdlib, frontend) going forward; new work is authored in **`.myc`**. This is a freeze on *expanding* the Rust surface, **not** a change to the DN-39 kernel **boundary** and **not** a ban on kernel bug-fix/maintenance. (2) **Mycelium-first expansion + full self-hosting (END-STATE).** New functionality is `.myc`, **and** the existing Rust — **including the kernel itself** — is progressively rewritten to `.myc`, so that **zero foreign first-party languages** remain by the DN-88 decomposition gate. The DN-26 migration of the existing Rust frontend proceeds per RFC-0016 §4.6 / ADR-038 §2.3; the **kernel rewrite is the deepest, last self-hosting step**. The zero-foreign-languages goal is **comprehensive — it reaches the full toolchain, the AOT/codegen backend included**: the current MLIR/LLVM path is slated for a **native-Mycelium codegen backend** (§3 — its own far-future `needs-design` track), *not* treated as a permanent external dependency; the only foreign residue in the terminal fully-native state is the **irreducible OS/hardware ABI seam** (syscalls, hardware intrinsics — minimal `wild`, §2.4). (3) **The `mycelium-tero` near-term exception.** Its Rust PoC/MVP (M-1015→M-1018, epic E39-1) finishes in Rust, then is rewritten to Mycelium (M-1019); like all first-party code it lands in the zero-foreign-languages end-state. (4) **Wild-free `.myc` kernel is the target.** `wild`/FFI blocks are minimised — used **only where strictly necessary** — so a self-hosted kernel maximises the language's own safety while retaining maximal performance; a fully wild-free `.myc` kernel is the stated **goal** (`Declared` — a preference/target, feasibility not yet proven). |
| **Relates / hardens** | **Hardens RFC-0016 §4.6** ("Rust-first now, Mycelium-lang eventually") into a forward freeze on new Rust surface *plus* an explicit end-state that includes the kernel. **Sharpens ADR-038's North Star** — *"Rust where appropriate, Mycelium everywhere else"* — resolving "where appropriate" for the **end-state** toward *everything first-party, kernel included*; ADR-038 §2.1–§2.3 (tag/release split, differential/replace-on-satisfaction) and §2.5 (transpiler progressive hardening — which this ADR promotes to first-class) are **unchanged**. ADR-038 §2.3 framed compiler/kernel self-hosting as a *conditional* aspiration; this ADR records the maintainer's clarified **end-state goal** that it is *committed* by the decomposition gate (`Declared`; the §2.3 conditionality relationship is flagged, §4, not silently overwritten — house rule #3). **DN-39 boundary UNCHANGED — but the kernel's implementation *language* changes** (Rust → Mycelium); rewriting the language of the trusted components neither moves the boundary nor promotes/demotes any component (the load-bearing nuance, §2.2). Feeds the **DN-88 decomposition gate** (zero-foreign-languages precondition). Feasibility base: **DN-14** (all 11 self-hosting gate rows `present`). Reduces the mirror burden of **DN-26 §7.2 / the L0-boundary decision** (the companion append). **Does not amend** ADR-007, ADR-022/036, or ADR-041. |
| **Grounds** | Maintainer directive (2026-07-07, this session — recorded verbatim in intent; `Declared` until `Accepted`, then binds); **RFC-0016 §4.6**; **ADR-038** (North Star + transpiler doctrine + §2.3 conditional self-hosting + §2.8 terminal); **DN-39 §7** (the kernel boundary — its *component set* held UNCHANGED); **DN-88** (the component-repo decomposition gate whose zero-foreign-languages precondition this fixes); **DN-14** (self-hosting-gate evidence); **DN-26** (the bootstrap plan whose mirror burden this reduces; L0-boundary Option-A is the companion decision); **DN-87 / E39-1** (`mycelium-tero`: M-1015 `done`, M-1016/17/18 Rust PoC, M-1019 the `.myc` rewrite; DN-87 v0 has Python ingestion — first-party code in the end-state); **DN-86** (the transpiler's Python front-ends — first-party code in the end-state); **DN-34 / DN-85 / M-1006** (the transpiler ladder this makes first-class); **ADR-019 / `mycelium-mlir` / M-995–M-996** (the current MLIR/LLVM AOT/codegen path a native-Mycelium backend eventually supersedes — §3 / OQ-5); KC-3, G2, VR-5. |
| **Date** | 2026-07-07 |

> **Posture (transparency rule / VR-5 / house rule #4).** This ADR records *governance/posture* from the
> maintainer's explicit 2026-07-07 direction. It **asserts no implementation progress**: no Rust is
> deleted, no module is declared ported, no guarantee tag is upgraded by this document. The **full
> self-hosting end-state (kernel included)** and the **wild-free `.myc` kernel** are **`Declared` goals**,
> not achieved or proven properties — the **feasibility of a self-hosted trusted kernel is unproven**
> (§3 / §6 OQ-1), and the bootstrap/trust story it raises is explicitly `needs-design` (§6 OQ-1,
> recommended as a dedicated follow-on DN). Genuine tensions and open distinctions are **surfaced, not
> buried** (§4, §6): the relationship to ADR-038 §2.3's *conditional* self-hosting; which Phase-I enablers
> are ratified-maintenance vs new surface; and whether "zero foreign languages" reaches *external*
> build-time toolchain dependencies (MLIR/LLVM/OS) or treats them as external deps. Each is flagged for the
> maintainer, not guessed (G2).

---

## 1. Context

Three ratified positions set the frame; the maintainer's 2026-07-07 direction fixes the forward edge of
all three and adds an explicit end-state:

- **RFC-0016 §4.6 — "Rust-first now, Mycelium-lang eventually" (M-346).** Every module lands **Rust-first**
  (Phase 5a), then migrates to Mycelium behind an NFR-7-style differential (Phase 5b). As written it
  governs the *order* of migration but leaves the door open to *new* Rust surface accruing, and does not
  pin the terminal state of the kernel/compiler.
- **ADR-038 — "Rust where appropriate, Mycelium everywhere else."** Pragmatic, progressive dogfooding; the
  Mycelium rewrite of existing Rust proceeds module-by-module, differential-validated (§2.3), with the
  transpiler an **accelerant, not a gate** (§2.5). Its §2.8 terminal (`1.0.0`) is "fully rewritten into
  Mycelium (where appropriate) and 100% operational"; §2.3 made **compiler self-hosting** a *deferred,
  conditional* aspiration (only if stability/perf-proven).
- **DN-39 — the KC-3 kernel boundary, held UNCHANGED.** The trusted core is the **set** {L0 Core IR,
  reference interpreter, content-addressing primitive, guarantee lattice, swap engine}; the 2026-06-26
  promotion review admitted zero promotions on merit.
- **DN-88 — the component-repo decomposition gate.** Once the project is fully rewritten into Mycelium and
  production-ready, the monorepo decomposes into per-component repos + spores (GHCR). Its precondition is
  the ADR-038 §2.8 terminal state.

The maintainer's direction resolves the ambiguity these leave open, over **two horizons**. **Now:** stop
growing the Rust base — new functionality is `.myc`, so effort concentrates on expanding the `.myc` base
rather than widening a Rust surface that must then be ported back (DN-14's self-hosting-gate verdict — all
11 rows `present` — is the evidence the surface can now *carry* new functionality). **End-state:** the
project is **fully Mycelium** — the kernel itself is rewritten to `.myc` as the deepest, last self-hosting
step, so **zero foreign first-party languages** remain by the DN-88 decomposition gate. This sharpens
ADR-038's "where appropriate": for the end-state, *everything first-party* — kernel included — is
appropriate to rewrite. The Rust kernel is the **current** trusted base, **not a permanent fixture**.

## 2. Decision

### 2.1 Two horizons — freeze now, full Mycelium (kernel included) by decomposition

**(a) NOW — the Rust-base freeze.** No new Rust code enters the Mycelium language-project surface (language
features, stdlib, frontend). New functionality is **authored in `.myc`**. This freeze is specifically about
**not expanding** the Rust surface; it is **not** a change to the DN-39 kernel boundary (§2.2), **not** a
ban on **maintenance** of the existing Rust kernel/frontend (bug-fixes, soundness repairs, and the
completion of *already-ratified* kernel capability are ongoing maintenance — §4), and **not** retroactive
(the existing Rust base is untouched and migrates per RFC-0016 §4.6 / ADR-038 §2.3 on its own schedule).

**(b) END-STATE — zero foreign first-party languages by the DN-88 decomposition gate.** The entire
first-party project is progressively rewritten to `.myc`: stdlib, toolchain, the transpiler and its
front-ends, `mycelium-tero`, the **AOT/codegen backend** (a native-Mycelium replacement for the current
MLIR/LLVM path — §3, a far-future track), **and the Rust kernel itself**. Self-hosting **includes rewriting
the kernel into Mycelium** — the deepest and *last* self-hosting step (the
frontend/checker/elaborator/evaluator port, DN-26 Stage-5/6, precedes it; the kernel is the floor beneath
them). By the DN-88 decomposition gate the project carries **zero foreign first-party languages** — all
first-party *code* is `.myc`. The **fully-native North Star** goes one step further and far later: a
native-Mycelium codegen backend supersedes the external MLIR/LLVM toolchain (§3), so that the terminal
state depends on **no** foreign build toolchain and the only foreign residue is the **irreducible
OS/hardware ABI seam** (§2.4). The Rust kernel is therefore the **current** trusted base, itself slated for
a `.myc` rewrite — a milestone, not a permanent boundary.

This does not amend ADR-038 (append-only); it records the maintainer's clarified **end-state goal**
(`Declared`) and sharpens ADR-038 §2.8's "where appropriate" toward "everything first-party." Its
relationship to ADR-038 §2.3's *conditional* compiler/kernel self-hosting ("only if stability/perf-proven")
is flagged in §4, not silently overwritten.

### 2.2 DN-39 preserved precisely — boundary vs implementation language

The load-bearing nuance: **DN-39 ratified the kernel *boundary* — the SET of trusted components** {L0 Core
IR, reference interpreter, content-addressing primitive, guarantee lattice, swap engine}. **That set is
UNCHANGED by this ADR.** What changes is the **implementation *language*** of those components: **Rust →
Mycelium**. Rewriting a trusted component's language:

- **does not move the boundary** — the same components are trusted, in the same roles;
- **does not promote or demote anything** — nothing enters or leaves the TCB;
- **is therefore fully consistent with DN-39, not a contradiction of it.** DN-39's "the kernel never grows
  without a net trust reduction" is about the *membership* of the trusted set, not the language it is
  written in.

A `.myc` kernel is the *same* kernel boundary, re-expressed in Mycelium. (The new *trust* questions this
raises — what executes and vouches for a `.myc` kernel — are real and are handled in §3 / §6 OQ-1; they are
questions about the trust *story*, not about the DN-39 component *set*.)

### 2.3 Mycelium-first expansion — the rationale, and tooling across the two horizons

New functionality is authored in `.myc`. The rationale, recorded faithfully:

1. **Concentrate effort on the `.myc` base** — the language eats its own cooking on *new* work, not only on
   a lagging back-port.
2. **Reduce the mirroring burden** — fewer *new* kernel/Rust types means fewer types to mirror in-language,
   directly easing the DN-26 L0-boundary **Option A** mirror model (its drift tax is bounded by construction
   once the kernel type-set is frozen against new additions — DN-26 L0-boundary §2.3 / §10).
3. **Systematise the port** — the manual hand-port is **parameterised, idempotent, systematically automated
   and schematised in the transpiler** (DN-34 / DN-85 / DN-86; the M-1006 ladder), so future porting is
   *systematic*. This makes ADR-038 §2.5's transpiler hardening a **first-class** workstream.

**Tooling across the two horizons — instrument now, `.myc` at the end.** `mycelium-transpile` (and the
DN-86 multi-language front-ends, the index/gate scripts) is a build *instrument*, not part of the language
the project ships, so the **freeze does not apply to it now** — the maintainer explicitly wants the
transpiler **improved** (point 3). **But it is first-party code**, so in the **end-state** it too is `.myc`:
the transpiler's Python front-ends (DN-86) and `mycelium-tero`'s Python ingestion (DN-87 v0) are named,
alongside the Rust kernel/stdlib, as first-party foreign-language code that the zero-foreign-languages
end-state eventually rewrites to Mycelium. "Not frozen now (free to evolve during development)" and "`.myc`
at the decomposition gate (as first-party code)" are both true across the two horizons.

### 2.4 Wild-free `.myc` kernel is the goal

`wild`/FFI blocks (the denied-by-default unsafe/FFI escape, admitted only in an `@std-sys` nodule under
`!{ffi}` — DN-14 row 9; lexicon §5/§7) are to be **minimised — used only where strictly necessary** — so a
self-hosted kernel maximises the language's own safety while retaining maximal performance. A **fully
wild-free `.myc` kernel is the goal.**

This applies the wild-free-core aspiration to the **`.myc` kernel** once it is self-hosted, and **refines
Decision 2 of the companion DN-26 L0-boundary append.** While the kernel is *Rust*, the DN-26 Option-A
"materialise the finished L0" crossing is a frontend↔kernel **FFI seam** (the one `wild:` prim reserved for
handing L0 to the trusted kernel to run). **Once the kernel is itself `.myc`, that crossing is no longer a
foreign-language FFI seam** — it becomes an **internal concern of the self-hosted kernel**, and its
wild-vs-wild-free status is part of the **kernel-rewrite design** (§6 OQ-1), not a permanent frontend↔kernel
boundary. **Honesty (`Declared`, VR-5):** a fully wild-free kernel is a **target**, not a proven-reachable
state — an irreducible `wild`/host floor is expected to remain at the **OS/hardware ABI seam** (syscalls,
hardware intrinsics). Per ADR-042's North Star that ABI seam is the **one place minimal `wild` is expected
to survive** in the terminal fully-native state (everything above it — kernel, codegen backend included — is
`.myc`). The ADR sets the *preference*; each residual `wild` site is audited (`just safety-check`)
and never silent (G2), and the residual count is tracked toward zero (§6 OQ-1).

## 3. Consequences

- **The `.myc` expansion, the port, and the kernel rewrite become the critical path.** With no new Rust
  surface and an end-state that includes the kernel, forward progress *is* the Mycelium-first authoring plus
  the DN-26 back-port culminating in the kernel rewrite. The transpiler-systematisation (§2.3.3) is
  load-bearing, not optional.
- **A `.myc` kernel raises a bootstrap & trust question — `needs-design` (see §6 OQ-1).** If the kernel is
  written in Mycelium, *what executes and vouches for it?* Candidate answers to **name** (this ADR does
  **not** solve them): **(i)** self-AOT-compilation of the `.myc` kernel to a native binary via the
  MLIR/LLVM backend (the `.myc` kernel compiles itself to native code); **(ii)** a **bootstrap seed** (an
  earlier trusted build — e.g. today's Rust kernel — used once to build the first `.myc` kernel, then
  **eventually eliminated**); **(iii)** the **"Reflections on Trusting Trust"** reflective-trust argument —
  a self-hosted toolchain must reason about the trust it inherits from whatever first built it, and how a
  reproducible / diverse-double-compilation argument discharges it; **(iv)** what the **new trusted base**
  becomes — plausibly *the `.myc` kernel source + the AOT compiler that lowers it* (the trust root moves
  from "the Rust kernel" to "the `.myc` kernel source + its lowering path"). **Recommended:** a dedicated
  follow-on **kernel-self-hosting / bootstrap-trust design note** (the maintainer will design it). The
  **feasibility of a self-hosted trusted kernel is `Declared`, not proven.**
- **A native-Mycelium codegen backend is a major, far-future design track of its own — `needs-design`.**
  "Zero foreign languages" reaching the *full* toolchain (OQ-4, maintainer-resolved) means the current
  MLIR/LLVM/C++ AOT path (the `mycelium-mlir` crate; the ADR-019 libMLIR toolchain decision; recent AOT
  work M-995/M-996) is itself slated for replacement by a **native-Mycelium code-generation backend** — not
  treated as a permanent external dependency. This is a **large, research-grade effort well beyond the
  frontend/kernel port** (a Mycelium-native codegen backend that lowers to native code, superseding
  MLIR/LLVM); it is flagged **`needs-design` with its own future design note** and is **NOT** folded into
  M-1013 (the frontend/eval port) or the kernel-rewrite track. **Honesty (`Declared`, VR-5/G2 — not
  soft-pedalled):** the **feasibility and timeline of a native-Mycelium codegen backend replacing LLVM are
  `Declared`** — an ambitious stated *goal*, not a proven-reachable or de-risked state. This ADR **sets the
  North Star** (fully-native everything; `wild` only at the OS/hardware ABI seam); it does **not** claim the
  path is short or de-risked. The AOT backend is also part of the OQ-1 bootstrap/trust root (what lowers the
  `.myc` kernel to native code), so the two tracks interlock.
- **DN-26 Option-A mirror gets cheaper, then dissolves.** Freezing new kernel/Rust types bounds the mirror's
  drift tax now (DN-26 §10); once the kernel is itself `.myc`, the frontend↔kernel mirror/FFI seam becomes
  an internal `.myc` concern (§2.4) rather than a cross-language boundary at all.
- **Existing Rust migrates unchanged; nothing is deleted here.** RFC-0016 §4.6 Phase-5b and ADR-038 §2.3
  govern *how* the existing Rust moves to Mycelium; the freeze only forbids *adding* to that surface.
- **The transpiler is first-class and itself a port target.** Improving it is a named workstream now
  (DN-85/DN-86/M-1006); as first-party code its Python front-ends are `.myc` in the end-state (§2.3).
- **`mycelium-tero` finishes in Rust, then dogfoods** (M-1015→M-1018 Rust; M-1019 `.myc`), like all
  first-party code.
- **Tensions and open distinctions surfaced (§4, §6):** the ADR-038 §2.3 conditional-self-hosting
  relationship; the Phase-I-enabler classification; and whether "zero foreign languages" reaches external
  build-time toolchain deps — each FLAGged for the maintainer (G2/VR-5), not guessed.

## 4. Tensions surfaced (per house rule #4)

- **ADR-038 §2.3 framed compiler/kernel self-hosting as *conditional* ("only if stability/perf-proven").**
  This ADR records the maintainer's clarified **end-state goal** that full self-hosting (kernel included) is
  *committed* by the DN-88 decomposition gate — a **strengthening** of §2.3's conditional aspiration into a
  directional commitment. It is recorded `Declared` (a goal, feasibility unproven — §3/OQ-1), and it does
  **not** rewrite ADR-038 (append-only, house rule #3). Whether this is a formal supersession of §2.3's
  conditionality, or the condition is deemed met by directive, is a **maintainer call** — flagged, not
  assumed.
- **Some ADR-038 Phase-I usability enablers are Rust-/kernel-side.** The below-grammar enabler list
  (binary integer arithmetic + signed ops, dense/vsa prim surfacing, the scalar-float `Repr` — ADR-040,
  already `Enacted` — `Substrate`/`consume` execution, an R2-lite runtime subset) includes kernel-side work
  that, read naively, looks like "new Rust." The honest reading: **completing an *already-ratified* kernel
  capability is maintenance**, on the §2.1(a) carve-out side (ADR-040's DN-69-reviewed scalar-float `Repr`
  is the precedent); **new, unratified kernel surface now meets a higher bar** — prefer the `.myc`/wild-free
  path where the surface can carry it (DN-14), and route a genuinely-required prim through the DN-39
  promotion bar with the freeze as a thumb on the scale toward "author it in `.myc`." This ADR does **not**
  adjudicate the individual enablers — a maintainer call (§6 OQ-2).

## 5. Definition of Done

**For this ADR (the decision record):**

- [x] Maintainer directive recorded faithfully: the two-horizon, four-point decision (freeze now ·
  Mycelium-first + full self-hosting including the kernel · the `tero` exception · wild-free `.myc` kernel),
  `Status: Accepted` (2026-07-07, in force going forward).
- [ ] Indexed in `docs/adr/README.md` and `docs/Doc-Index.md`; `CHANGELOG.md` records the decision
  *(orchestrator-owned — proposed in the companion report, applied by the integrating parent)*.
- [ ] The companion **DN-26 L0-boundary Option-A** decision appended (the mirror-burden reduction cited).
- [ ] A dedicated **kernel-self-hosting / bootstrap-trust design note** recommended (§3 / §6 OQ-1) is
  minted for the maintainer to author (the trust story is `needs-design`, not solved here).
- [ ] `Accepted → Enacted` **only when** (never by ratification alone — house rule #3): the entire
  first-party project — **kernel included** — is rewritten to `.myc`; **zero foreign first-party languages**
  remain at the DN-88 decomposition gate; the bootstrap/trust story (OQ-1) is designed and discharged; the
  `mycelium-tero` rewrite (M-1019) has landed; and — for the **fully-native North Star** terminal — the
  native-Mycelium codegen backend has superseded MLIR/LLVM (§3 / OQ-5), leaving only the irreducible
  OS/hardware ABI seam. Each is a *checked* basis, not an intent. (The native-codegen terminal is the
  far-future step; the decomposition-gate zero-foreign-first-party-*languages* state is the nearer one.)

**For the policy it enacts (the standing gate, checked continuously):**

- [ ] New language-project functionality lands as `.myc`, not new Rust (auditable at PR review — a "no new
  language-project Rust surface" check for the freeze's life; §6 OQ-3).
- [ ] `wild` sites are justified per-occurrence (`just safety-check`), minimised, and the
  wild-free-`.myc`-kernel goal tracked (§6 OQ-1) — never silently expanded (G2).
- [ ] The transpiler-systematisation workstream (DN-85/DN-86/M-1006) is resourced as first-class.

## 6. Open questions

- **OQ-1 — the bootstrap & trust story for a `.myc` kernel (`needs-design`).** What executes and vouches for
  a self-hosted kernel? Name and design: self-AOT-compilation via MLIR/LLVM; a bootstrap seed eventually
  eliminated; the "Reflections on Trusting Trust" reflective-trust discharge (reproducible /
  diverse-double-compilation); and the new trusted base (the `.myc` kernel source + the AOT compiler that
  lowers it). **Recommend a dedicated follow-on design note**; the maintainer will design it. The
  feasibility of a self-hosted trusted kernel is `Declared`, not proven.
- **OQ-2 — which in-flight ADR-038 Phase-I enablers are ratified-maintenance vs new surface?** (§4.) A
  maintainer adjudication — flagged, not decided here.
- **OQ-3 — does the freeze need a `/pr-review` or CI lens to hold "by construction"?** A lightweight "no new
  language-project Rust surface" review check (analogous to the branch-guard) would make the freeze hold by
  construction. Proposed, not built here.
- **OQ-4 — does "zero foreign languages" reach *external* build-time toolchain dependencies? — RESOLVED
  (maintainer, 2026-07-07): YES, the full toolchain.** The goal is **fully-native Mycelium everything**,
  including the AOT/**codegen backend**: the current MLIR/LLVM/C++ AOT path is **slated for a native-Mycelium
  codegen-backend replacement** (§3 — a major, far-future `needs-design` track), **not** treated as a
  permanent external dependency. The **only** foreign residue in the terminal fully-native state is the
  **irreducible OS/hardware ABI seam** (syscalls, hardware intrinsics) — exactly where the minimal-`wild`
  wild-free goal bottoms out (§2.4), and the one place `wild` is expected to survive. (Recorded as the
  end-state, no longer open; the *how* of the native codegen backend is OQ-5.)
- **OQ-5 — the native-Mycelium codegen backend design (`needs-design`, its own future note).** How a
  Mycelium-native code-generation backend lowers `.myc` to native code and supersedes MLIR/LLVM (§3). A
  large, research-grade track distinct from the frontend/kernel port and interlocking with the OQ-1
  bootstrap/trust root. Recommend a dedicated future design note; feasibility and timeline `Declared`, not
  proven (VR-5/G2).

## 7. Alternatives considered

- **Keep RFC-0016 §4.6 as written (Rust-first, migrate eventually; new Rust still permitted; kernel stays
  Rust permanently).** Rejected: it lets the Rust surface keep growing and pins no end-state, contradicting
  the maintainer's zero-foreign-languages goal.
- **Freeze new Rust but keep the kernel Rust forever.** Rejected by the clarification: the end-state is a
  *fully* Mycelium project — the kernel is a port target (the deepest, last one), not a permanent fixture.
  (This is the premise the first draft of this ADR got wrong and this revision corrects.)
- **Zero-Rust immediately (rewrite the kernel now).** Rejected: the kernel rewrite is the *deepest* step and
  depends on the frontend port (DN-26 Stage-5/6) and the bootstrap/trust design (OQ-1) landing first;
  forcing it now would be building the floor before the walls. The freeze keeps the trusted Rust while no
  *new* Rust accrues, and the kernel rewrite lands last.
- **No `tero` exception (force `mycelium-tero` into `.myc` now).** Rejected: the Rust PoC de-risks the DN-87
  §6 improved-on-RAG bet; M-1019 already gates the rewrite on M-993 maturity.
- **Leave the wild-free-kernel goal and the trust story unstated.** Rejected: naming them — while tagging
  feasibility `Declared` and routing the trust story to a follow-on DN — is the never-silent posture (G2).

## 8. Grounding / honesty

- Maintainer directive, 2026-07-07 (this session) — the doctrine §2 records; `Declared` until `Accepted`.
- RFC-0016 §4.6 — the migration trajectory hardened (Phase-5b differential unchanged).
- ADR-038 §2.1–§2.3/§2.5/§2.8 — the North Star + differential-replace + transpiler doctrine sharpened for
  the end-state; the §2.3 conditional-self-hosting relationship flagged (§4), not amended;
  ADR-007/022/036/041 untouched.
- DN-39 §7 — the kernel *component set* held UNCHANGED; the implementation *language* changes (§2.2); KC-3.
- DN-88 — the decomposition gate whose zero-foreign-languages precondition this fixes.
- DN-14 — the self-hosting-gate evidence; the `wild`/`@std-sys`/`!{ffi}` mechanism cited for §2.4.
- DN-26 — the bootstrap plan whose mirror burden §2.3.2 reduces; the L0-boundary Option-A companion append.
- DN-87 / E39-1 (`mycelium-tero`); DN-86 (transpiler Python front-ends) — first-party foreign-language code
  named for the end-state; checked against `issues.yaml` (2026-07-07).
- DN-34 / DN-85 / M-1006 — the transpiler ladder made first-class.
- KC-3, G2, VR-5 — small trusted base grows only via DN-39 review; nothing silent; no claim (including the
  full-self-hosting / wild-free-kernel goals and this ADR's own status) above its checked basis.

---

## Meta — changelog

- **2026-07-07 — Accepted (maintainer directive).** Records the **Rust-base freeze + Mycelium-first
  expansion** policy over **two horizons**: **NOW** — freeze new Rust language-project surface (new work in
  `.myc`); **END-STATE** — the entire first-party project, **including the kernel**, rewritten to `.myc`:
  **zero foreign first-party languages by the DN-88 decomposition gate.** The Rust kernel is the *current*
  trusted base, itself slated for a `.myc` rewrite (the deepest, last self-hosting step) — not a permanent
  fixture. **DN-39 preserved precisely:** the trusted-component *set* is UNCHANGED; only the implementation
  *language* changes (Rust → Mycelium) — no boundary move, no promotion/demotion. Names the **bootstrap and
  trust question** a `.myc` kernel raises (self-AOT via MLIR/LLVM; a bootstrap seed eventually eliminated;
  the "Trusting Trust" reflective-trust argument; the new trusted base = `.myc` kernel source + AOT
  compiler) as `needs-design` and recommends a dedicated follow-on DN; **feasibility `Declared`, not
  proven.** **Wild-free `.myc` kernel is the goal** (the DN-26 Option-A materialise-L0 crossing becomes an
  internal `.myc`-kernel concern once self-hosted). Zero-foreign-languages is comprehensive — also names the
  transpiler's Python front-ends (DN-86) and tero's Python ingestion (DN-87 v0) as end-state port targets;
  keeps the `mycelium-tero` PoC exception (M-1015→M-1018 Rust; M-1019 `.myc`). Hardens RFC-0016 §4.6,
  sharpens ADR-038's North Star; strengthens ADR-038 §2.3's conditional self-hosting to a directional
  end-state goal (flagged §4, not amended). **Enacts nothing** — `Accepted → Enacted` only when the whole
  first-party project (kernel included) is `.myc`, zero foreign first-party languages remain at the
  decomposition gate, and the bootstrap/trust story is designed + discharged (house rule #3). **OQ-4
  resolved (maintainer, 2026-07-07):** "zero foreign languages" reaches the **full toolchain, AOT/codegen
  backend included** — a native-Mycelium codegen backend supersedes MLIR/LLVM (its own major, far-future
  `needs-design` track, §3 / OQ-5; feasibility + timeline `Declared`, not de-risked — supersedes the
  `mycelium-mlir` / ADR-019 / M-995-996 path); the only terminal foreign residue is the irreducible
  OS/hardware ABI seam (minimal `wild`). Remaining open questions: the trust story (OQ-1), Phase-I-enabler
  classification (OQ-2), a freeze review-lens (OQ-3), the native-codegen-backend design (OQ-5). Doc-Index /
  README / CHANGELOG rows owned by the integrating parent. (VR-5 / G2 / house rule #4.)
