# RFC-0034 — Tunable Certification & Transparency Modes

| Field | Value |
|---|---|
| **RFC** | 0034 |
| **Status** | **Proposed** (2026-06-24) — the tunable-certification model (the knob matrix, the two first-class modes `fast`/`certified`, mode resolution + scoping, the provenance-tag-as-adjustable-unit, the generation/consumption split, the compile/runtime phase split, and the never-silent mode invariant) is stated normatively **as proposed**. The maintainer ratifies → **Accepted** (house rule #3 — never skipping steps). The corpus amendments (the always-on→per-mode rewordings and the honesty→transparency reframe) are carried by the **superseding ADR-032** and applied via the staged manifest (§13) only **after** both are Accepted — never silently. |
| **Type** | Foundational / normative (once Accepted) — makes certification/hashing/tag machinery a *tunable policy* over the existing RFC-0001/0002/0005 substrate; the substrate's mechanisms are unchanged |
| **Date** | 2026-06-24 |
| **Decides** | the knob matrix (§4); the two first-class modes `fast`/`certified` + `balanced` intermediate (§5); mode resolution + `global/phylum/nodule` scoping (§6); provenance tagging as an adjustable unit, `fast` omitting `Empirical`/`Proven` (§7); the generation≠consumption split for the inspectability signal (§7); the compile/runtime phase split — spores survive a cert-off runtime (§8); memory-safe-by-default + explicit per-use escape (§9); the named `wrapping`/`fast` Axis-B opt-out (§10); the never-silent mode invariant (§3) |
| **Depends on** | RFC-0001 (the value model — `Repr`/`Value`/`Meta`/`Payload`, the `Exact⊐Proven⊐Empirical⊐Declared` lattice, content-addressing §4.6); RFC-0002 (swap certificate & checker — the machinery a mode gates); RFC-0005 (`PolicyRef`/EXPLAIN — `EXPLAIN` of the *mode* stays mandatory); RFC-0012 (ambient representation & scoped overrides — the scoping mechanism reused for mode resolution); ADR-010/ADR-011 (bound kernels / universal `BoundBasis` — invoked at `certified`); ADR-013/ADR-016/ADR-017 (spore / ABI / hot-inject — the compile-time hash consumers); ADR-014 (unsafe `permitted-but-warned` — sharpened, not superseded, §9); **KC-4** (cost-driven downgrade already authorized — generalized from a kill-switch into a knob); **VR-5 / G2** (downgrade-don't-overclaim; never-silent — the transparency contract); DN-29 (the deliberation anchor; §11 ripple map) |
| **Superseded-by-relationship** | **ADR-032** carries the charter/north-star reframe, the always-on→per-mode supersession of SC-3/FR-M3's *unconditional* reading, and the whole-corpus honesty→transparency vocabulary reframe. This RFC stays **implementation-focused** (DN-29 §11.5 Q12). |
| **Coupled with** | `mycelium-proj` (manifest / nodule-header `@certification` — the mode declaration site, per DN-29); `crates/mycelium-cert/*` (the swap-cert machinery a mode gates); `crates/mycelium-core/src/{guarantee.rs,meta.rs,content.rs,bound.rs}` (tag propagation, content-hash, bound basis); `crates/mycelium-spore/*` (compile-time spore identity); `crates/mycelium-mlir/src/inject.rs` (ABI dispatch keys) |
| **Task** | tunable-certification (DN-29 → RFC-0034 + ADR-032). Epic id to be minted in `tools/github/issues.yaml` at ratification — **not assigned here** to avoid an ID collision (swarm mitigation #1). |

> **Posture (transparency rule / VR-5).** This RFC **decides the model as proposed** (§3–§10): the knobs,
> the modes, the scoping, and the never-silent mode invariant. It **decides the surface, it does not
> implement it** — no kernel code lands alongside it; claims about runtime behaviour are `Declared`
> positions to be checked by implementation (VR-5). The machinery it tunes (RFC-0001 lattice, RFC-0002
> certificates, ADR-010 kernels, ADR-013 spores) is **unchanged** — this RFC only makes *when* it runs a
> policy. The corpus rewordings and the charter reframe are **ADR-032's** act, applied via the staged
> §13 manifest **after** ratification (append-only; never silently). Until then the always-on rules hold.
>
> **Provenance.** Rationale, the knob-by-knob cost audit, the keep-list, and the rejected single-ladder
> strawman are recorded in **DN-29** (`docs/notes/DN-29-Tunable-Certification-and-Honesty-Modes.md`),
> grounded in a machinery/coupling audit of `mycelium-core`/`-cert`/`-spore`/`-mlir` and a corpus survey
> of the always-on framing (SC-3/FR-M3/VR-5/G2/KC-4).

## §1 Scope

The corpus mandates the full certification/hashing/tag machinery **unconditionally**: every value carries a
`GuaranteeStrength` (RFC-0001 §3.4), every swap emits + checks a certificate (SC-3, FR-M3, RFC-0002 §2),
every value/definition is content-hashed (RFC-0001 §4.6). That was right for the maximalist design phase;
in practice it is *messy and expensive* on every line. This RFC makes the machinery a **tunable policy** —
dialable from fully off to fully engaged — **without losing transparency**, by separating two things the
corpus had fused:

- **Transparency** = *operations are never opaque and never overclaim* (you can always see what was
  established; a result never claims a strength it did not compute). Cheap; the everyday default.
- **Certification depth** = *how much you bother to establish, and whether it is machine-checked.*
  Expensive; engaged on request.

In scope: the knob matrix (§4), the modes (§5), resolution/scoping (§6), provenance tagging + the
signal generation/consumption split (§7), the compile/runtime phase split (§8), memory safety (§9), the
Axis-B opt-out (§10), the transparency argument (§11), the keep-list (§12), conformance + the staged
amendment manifest (§13). **Out of scope** (carried by ADR-032): the charter/north-star reframe and the
whole-corpus honesty→transparency vocabulary rewording.

## §2 User stories

- *As an application developer*, I want a **fast, memory-safe, ergonomic** language that does not tax every
  operation with certificate machinery I am not using, **so that** day-to-day code is fast and uncluttered —
  while I keep the option to turn assurance on where it matters.
- *As a library (`phylum`) author shipping to others*, I want to **opt a phylum up to `certified`** so its
  published guarantees are checked and certificate-backed, **so that** consumers get auditable provenance
  for the parts that warrant it.
- *As a developer debugging a swap*, I want the **inspectability signal always captured** (what happened,
  which swap, why, how) even in `fast`, with **consumption I can dial up mid-session**, **so that** I never
  have to re-run or switch modes to inspect history that already occurred.
- *As a safety-conscious developer*, I want **memory safety by default** with **unsafe reachable only via an
  explicit per-use escape**, **so that** every unsafe site is a conscious, auditable choice.
- *As a reviewer / AI agent*, I want the **active mode to be never-silent** — tagged on every result,
  surfaced by tooling, explicit at any cross-mode boundary — **so that** I can always tell what assurance a
  value actually carries.

## §3 Invariants (normative)

1. **Never-silent about the mode (G2).** Every result carries an explicit certification-mode tag; tooling
   always surfaces the active mode; a value produced in one mode entering a stricter-mode computation is an
   **explicit, visible event**, never a silent upgrade (disclosure can only degrade — RFC-0001 §3.4).
2. **No overclaim (VR-5).** No result claims a guarantee strength it did not compute. `fast` results sit at
   the structural tags (`Exact`/`Declared`); they **never** carry `Empirical`/`Proven`, because those cost
   trials/proofs `fast` did not run.
3. **Transparency floor (mode-independent).** The inspectability signal (§7) is generated from the middle
   tier up; memory safety (§9) and never-silent failability (Axis B, §10) hold in **every** mode, including
   `fast`. Tunability never reaches these.
4. **Out-of-range stays an explicit `Option`/error** in every mode (RFC-0001/RFC-0002 never-silent
   fallibility is *not* an Axis-A knob).

## §4 The knob matrix (normative)

Certification is **not** a single scalar; it is a small matrix of independent knobs grouped by *when* the
work happens. They compose; the modes (§5) are presets over them.

| Knob | Phase | Cost | Enables | `fast` default |
|---|---|---|---|---|
| **Spore / deploy identity hash** | compile / deploy | cheap, **per-artifact** | spores (ADR-013), hot-inject + ABI dispatch keys (ADR-016/017), dedup | **available** (deployability survives cert-off — §8) |
| **Memory safety** | always | inherited (Rust kernel) | safe surface; no raw pointers | **safe** (explicit per-use escape — §9) |
| **Never-silent failure** (Axis B) | runtime | O(1) | `Option`/`Result`/`SwapError` out-of-range | **on** (named `wrapping` opt-out — §10) |
| **Provenance tagging** *(adjustable unit)* | runtime | `Exact`/`Declared` O(1); `Empirical`=trials, `Proven`=checked theorem | the `Exact⊐Proven⊐Empirical⊐Declared` lattice | **structural `Exact`/`Declared` only** (§7) |
| **Signal generation** (inspectability trace) | runtime | cheap | the "what happened / which swap / why / how" trace `EXPLAIN` reads | **generated ≥ middle tier** (safe default — §7) |
| **Diagnostic consumption / DX surfacing** | tooling | ~O(1) | how much of the signal surfaces in DX/UX | **lean** (signal still generated, just not shown — §7) |
| **Runtime value hashing / dedup** | runtime | O(size) | runtime identity/dedup of values | **off** |
| **Swap-cert emission** | runtime | O(1) | a certificate object per swap (RFC-0002) | **off** |
| **Swap-cert checking** | runtime | **expensive** | the RFC-0002 checker + ADR-010 bound kernels | **off** |

## §5 The two first-class modes (normative)

`fast` and `certified` are the two **first-class** modes — the two postures a developer chooses between.
`balanced` is an **optional intermediate**, not a headline.

| Mode | First-class? | Spore hash | Mem-safe | Axis-B | Tags | Signal gen | Consumption | Cert emit | Cert check | Character |
|---|---|---|---|---|---|---|---|---|---|---|
| **`fast`** ***(default)*** | **yes** | available | safe | on | `Exact`/`Declared` | ≥ middle tier | lean | off | off | fast, memory-safe, ergonomic; **inspectable + still deployable** |
| **`balanced`** *(intermediate)* | no | available | safe | on | propagated (unchecked) | full | medium | emitted (unchecked) | off | provenance tags + certs, not verified |
| **`certified`** | **yes** | available | safe | on | tracked | full | full audit trail | emitted | **checked** (ADR-010) | **fully auditable**, max assurance |

`certified` is the union of the maximalist phase's machinery (§12), engaged **on request** rather than
always. `balanced` exists for the "honest tags, skip the expensive checking" middle ground.

## §6 Mode resolution & scoping (normative)

- **Scope (v0): `global` ⊐ `phylum` ⊐ `nodule`.** A mode is declared at the project (`mycelium-proj`
  manifest), `phylum`, or `nodule` level, resolving most-specific-wins — reusing the **RFC-0012** ambient
  representation + scoped-override mechanism (no new scoping machinery). The per-op `thaw`-style knob is
  **deferred** (YAGNI until a use case demands it — DN-29 §9 Q5).
- **Declaration site: a `@certification` attribute** on the manifest / nodule header (DN-29). The mode is
  data in the source, **not** a hidden build flag.
- **No hash perturbation (ADR-003).** The mode is metadata; it **must not** enter the content hash of a
  value/definition — switching modes does not change spore identity. (The mode *is* recorded on results as
  the never-silent §3.1 tag; that tag is dynamic `Meta`, excluded from the content hash exactly as other
  dynamic metadata is — RFC-0001 §4.6.)
- **Cross-mode composition** is the explicit, visible event of §3.1: combining a `fast` value into a
  `certified` computation surfaces the mode boundary; the result cannot silently inherit `certified`
  strength it did not earn.
- **Per-knob overrides** under a mode are **deferred**: v0 ships the named modes only; knob-level overrides
  are added once the modes prove out (DN-29 §9 Q7).

## §7 Provenance tagging & the generation/consumption split (normative)

**The provenance tag is an adjustable unit.** `fast` defaults to **not using `Empirical`/`Proven`** — those
cost the *trials* / *checked theorems* `fast` skips — so `fast` results sit at the structural `Exact` (when
the representation is exact) / `Declared` (otherwise) tags. Computing the heavier tags is dialed up per mode
or per unit, **never forced**. This is the honest floor (§3.2): `fast` does not *claim* `Empirical` because
it did not run the trials (VR-5).

**Generation ≠ consumption.** Two distinct things were fused under "verbosity":

- **Signal generation** — the cheap, valuable *inspectability trace* (what happened, which swap, why, how)
  is **always generated from the middle tier up** (a safe default), so the data exists to inspect.
- **Consumption / DX surfacing** — how much of that signal the DX/UX renders — is **tunable**, and `fast`
  defaults to **lean**. Because generation is always-on, a developer can **dial consumption up mid-session
  and the history is already captured** — no re-run, no mode switch, no painting-into-a-corner.

The stance this encodes: *give developers the tools, the options, and the reasoning behind why each exists —
and let them choose whether to use them.* Lightweight-value-on-by-default + an always-generated signal +
tunable consumption is how `fast` stays **both** cheap **and** non-cornering.

## §8 Compile/runtime phase split — spores survive a cert-off runtime (normative)

"Hashing" is **two** knobs, not one, split by *when* the work happens:

- **Compile / deploy phase** — content-hashing a **deployable unit** mints its **spore identity**
  (ADR-013) and the hot-inject / ABI **dispatch keys** (ADR-016/017). Paid **once per built artifact**, not
  per runtime operation; it is what makes units deployable, content-addressed, and dedup-able.
- **Runtime phase** — hashing **values** during execution, propagating tags, emitting/checking swap
  certificates — repeated across the whole functionality set as the program runs.

**Decisive consequence: deployability survives a fully cert-off runtime.** `fast` keeps spores — content-
addressed, deployable units — while paying **no** runtime hash/cert cost, because spore identity is a
compile/deploy concern. Turning off the *compile* spore hash is a separate, deliberate choice (embedded /
no-deploy builds) that **MUST explicitly disable and `EXPLAIN`** the loss of spores/inject — never-silent
about *capabilities*, not just values.

## §9 Memory safety — safe by default, explicit per-use escape (normative)

The surface is **memory-safe by default**. Unsafe memory operations are reachable **only** through an
*explicit, per-use* escape hatch at the call site — the **ADR-014** `permitted-but-warned` precedent,
**sharpened** from a global lint toggle to a **per-use, source-visible** opt-in, so the developer must
consciously think at each use and the escape is grep-auditable. This is **independent of the certification
mode**: even `fast` is memory-safe. ADR-014 is **sharpened, not superseded** — its dev/test warning + the
mandatory `// SAFETY:` justification still apply.

## §10 The Axis-B `wrapping` opt-out (normative)

Never-silent failability (Axis B — out-of-range yields `Option`/`Result`/`SwapError`) is **default-on in
every mode**, including `fast`; it is O(1) and is the cheapest part of the transparency floor (§3.3). A
developer who genuinely wants wraparound arithmetic opts in via a **named, explicit `wrapping`** construct —
exposed in **v0** (DN-29 §9 Q4). The opt-out is *named and visible*, never an ambient default: choosing
`wrapping` is itself never-silent.

## §11 Why it stays transparent (the argument)

Tunability stays **transparent and accurate** at every setting — *transparency is disclosure-of-strength
plus inspectable ops, not universal certification* — and the corpus already contains the hooks:

- The **`Declared`** tier exists precisely for *"asserted, not proven, always flagged"* (RFC-0001 §4.3).
  `fast` tags results `Declared` + `mode: fast` — the intended use of the weakest tier, applied
  systematically.
- **KC-4** already authorizes a cost-driven downgrade *certified → declared-and-property-tested* ("document
  the loss"). Tunable certification **generalizes KC-4 from a one-time kill-switch into a knob.**
- **VR-5 / RFC-0014** already permit *downgrade-and-disclose* (recovery/fallbacks "never fabricate or
  upgrade a guarantee"). A mode is a **systematic, flagged downgrade**, never a hidden overclaim.
- **G2** is satisfied so long as the mode is **inspectable** and **cross-mode operations are explicit**
  (§3.1).

What stays non-negotiable even in `fast`: **the mode is never hidden, and no result ever claims a strength
it did not compute.** That single rule is the whole transparency contract; everything else is opt-in.

## §12 What's kept (the maximalist phase earned these)

None of the maximalist machinery is discarded — it becomes the **implementation of `certified`**, gated
behind the mode that asks for it: the **ADR-010** bound kernels, the **RFC-0002** swap-certificate
split-regime + checker, **ADR-011** (`BoundBasis` universal — within `certified`), the four-point guarantee
lattice, the M-210 checker, content-addressing (ADR-013). A spec moves to *"implemented (Rust-first),
pending ratification"* as pieces land — never silently to `Accepted` (VR-5).

## §13 Conformance & Definition of Done

**Definition of Done (this RFC):** ratified to **Accepted** by the maintainer; the knob matrix (§4), the two
first-class modes (§5), the resolution/scoping rules (§6), the tagging + generation/consumption split (§7),
the phase split (§8), and the §3 invariants are stated normatively and grounded; the companion **ADR-032**
is Accepted in lockstep (it carries the charter/vocabulary reframe); the staged amendment manifest (below)
exists and is dry-run-clean; **no corpus text is rewritten and no code lands** under this RFC (those are the
post-ratification acts).

**Conformance (once implemented):** an implementation conforms iff (a) every result carries a never-silent
mode tag (§3.1); (b) no `fast` result carries `Empirical`/`Proven` (§3.2); (c) memory safety + Axis-B hold
in every mode (§3.3); (d) `EXPLAIN` of the active mode is always available; (e) spores are mintable with the
runtime cert-off (§8); (f) cross-mode composition surfaces the boundary (§6).

**Staged amendment manifest.** The corpus rewordings (ADR-032's act) are applied — **only after both this
RFC and ADR-032 are Accepted** — via the anchor-keyed, single-pass-per-file mechanism of DN-29 §11.4
(`tools/dn29_apply.py` + `docs/notes/dn29-amendment-manifest.json`). The tool is **dry-run by default** and
**never-silent**: it asserts each anchor matches exactly once and fails loudly otherwise. It is **staged,
not run**, under this RFC.

## §14 Residual / open

- **Per-op granularity** (`thaw`-style) and **per-knob overrides** under a mode are deliberately deferred
  (§6) — revisit if a concrete use case demands sub-nodule tuning.
- **Default-by-kind** (a published `phylum` defaulting to `balanced`/`certified`) was considered and
  **rejected for v0** in favour of a single `fast` default with explicit opt-up (DN-29 §9 Q3) — revisit if
  ecosystem experience shows consumers need certified-by-default for shared code.
- The exact `@certification` surface syntax is sketched (§6); its grammar lands with the surface-syntax work
  (it reuses RFC-0012 scoping, so no new resolution semantics are open).

### Changelog

| Date | Status | Note |
|---|---|---|
| 2026-06-24 | **Proposed** | Initial proposal from the settled DN-29 deliberation. Makes certification/hashing/tag machinery a tunable policy over RFC-0001/0002/0005: the knob matrix (§4), two first-class modes `fast`/`certified` + `balanced` intermediate (§5), `global/phylum/nodule` resolution reusing RFC-0012 scoping (§6), provenance tag as an adjustable unit with `fast` omitting `Empirical`/`Proven` + the generation≠consumption split (§7), the compile/runtime phase split so spores survive a cert-off runtime (§8), memory-safe-by-default + explicit per-use escape sharpening ADR-014 (§9), the named `wrapping` Axis-B opt-out (§10), and the never-silent mode invariant (§3). Implementation-focused; the charter/north-star reframe + whole-corpus honesty→transparency vocabulary reword are carried by the superseding **ADR-032** and applied via the staged §13 manifest only after both are Accepted. Decides the surface, implements nothing (VR-5/G2). Anchor: DN-29. |
