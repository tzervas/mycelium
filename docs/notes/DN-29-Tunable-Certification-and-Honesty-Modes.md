# Design Note DN-29 — Tunable Certification & Honesty Modes

| Field | Value |
|---|---|
| **Note** | DN-29 |
| **Status** | **Draft** (2026-06-24; deliberation anchor — advisory, non-committal · rev. 2026-06-24 — owner-steered: phase-decomposed knobs, `fast`-default, north-star reframe · rev. 2 2026-06-24 — `fast`+`certified` first-class modes, safe-by-default + explicit per-use unsafe escape, diagnostic-verbosity knob, "honesty"→"transparency & auditability" reframe · rev. 3 2026-06-24 — provenance tag as adjustable unit (`fast` omits `Empirical`/`Proven`), signal **generation** split from **consumption**, **§11 ripple map** (40-hit corpus inventory + batched-replacement mechanism)) |
| **Feeds** | the eventual **RFC-0034** (binding decision) + a superseding ADR; conditionalizes RFC-0001 §3.3/§3.4/§4.6, RFC-0002 §2, RFC-0005 §2, ADR-010, ADR-011, ADR-013/016/017; grounds in **KC-4**, **VR-5**, **G2**, ADR-014 |
| **Date** | June 24, 2026 |
| **Decides** | *Nothing normatively* — advisory. Records the maintainer's intended **shift** (2026-06-24): make the certification / honesty / hashing machinery **tunable from fully-off → fully-engaged** via scoped config, instead of mandatory-everywhere, **while preserving honesty and keeping the good procedures** the early maximalist phase produced. Captures the design space, the knob model, the feature-dependency consequences, and the open questions, so the binding RFC-0034 is shaped toward this end-state. The binding decision is the future RFC. **Rev. 2026-06-24 (owner-steered, §10):** the single ladder is **decomposed into independent knobs split by *phase*** — compile/deploy-time **spore identity hashing stays available even when all runtime certification is off** — composing into named **profiles**; the default profile is **`fast`** (opt-in certification); and the note now **captures a north-star reframe** (Mycelium as a fast, memory-safe, ergonomic multi-paradigm language with certification *baked in as optional*), with the Foundation/charter ripple **flagged** for the binding RFC-0034 + superseding ADR. |
| **Task** | value-model / honesty-model evolution (pre-RFC-0034 deliberation) |

> **Posture (transparency rule / VR-5).** Advisory, forward-looking. **Enacts nothing; moves no status;
> changes no normative text.** What exists **today** is the all-on machinery (SC-3 / FR-M3: every swap
> emits + checks a certificate; every value is content-hashed; every value carries a guarantee tag). This
> note records a *direction* — a tunable model — and the argument that it stays **transparent and
> accurate** at every setting. It also proposes a **vocabulary reframe** (§1, §6): the project's "honesty"
> principle is restated as **transparency & auditability** — same mechanism, pragmatic framing. The binding
> change is RFC-0034 + a superseding ADR; until then the all-on rules hold. No code or guarantee is claimed
> here (VR-5 / G2).

---

## §1 Purpose & the reframe

The corpus mandates the full transparency/certification/hashing machinery **unconditionally**: every value
carries a `GuaranteeStrength` (RFC-0001 §3.4), every swap emits + checks a certificate (**SC-3**,
**FR-M3**, RFC-0002 §2), every value/definition is content-hashed (RFC-0001 §4.6). The maintainer's
assessment (2026-06-24): *"certification everywhere is messy and expensive"* — devs/users should dial it
from **fully off → fully engaged**, and disable the machinery entirely when undesired, **keeping** the
genuinely good procedures the maximalist phase produced.

**Vocabulary reframe (owner-steered, 2026-06-24): "honesty" → "transparency & auditability."** The
corpus's "honesty rule / never lie" framing is recast in pragmatic engineering terms — it was always about
**transparent, inspectable operations**, not a moral claim. The restatement:

- **By default (`fast`): transparent & inspectable — *non-certified auditability*.** Every operation is
  debuggable and inspectable: you can see *what happened, what went wrong, why, and how*, **without** any
  certification machinery running. This is the everyday value — a fast language whose operations are never
  opaque.
- **With certification (`certified`): a *fully auditable* framework.** Engaging certification upgrades the
  inspectable trail into a checked, certificate-backed, fully-auditable record. **Optional**, on request.

The **mechanism is unchanged** — never-silent (G2), the four-point provenance lattice
(`Exact⊐Proven⊐Empirical⊐Declared`), `EXPLAIN`-able selections, and downgrade-don't-overclaim (VR-5). Only
the *framing* changes: from "stay honest" to "stay transparent and accurate." (Charter-level vocabulary —
**captured here, flagged** for RFC-0034 + the superseding ADR; CLAUDE.md house-rule 1, CONTRIBUTING, and
the VR-5/G2 wording are not rewritten here — §7.)

**The keystone: transparency ≠ mandatory maximal certification.** Transparency = *operations are never
opaque and never overclaim* (you can always see what was established). Certification *depth* = *how much
you bother to establish, and whether it's checked.* Splitting these makes the expensive machinery a
**tunable policy**, and transparency **survives at every setting** because the corpus already contains the
hooks:

- **KC-4** (Foundation §2.4) already authorizes downgrading *certified → declared-and-property-tested*
  on cost grounds — "document the loss." Tunable certification **generalizes KC-4 from a one-time
  kill-switch into a knob.**
- **VR-5** ("downgrade to stay accurate; never upgrade without a checked basis") + the **`Declared`** tier
  (RFC-0001 §4.3 — *"always flagged; never silently trusted"*) make "off" a *systematic, flagged
  downgrade to `Declared`*, **not** a hidden overclaim.
- **G2 (never-silent)** binds the *mode itself*: results carry an explicit certification-mode tag;
  tooling always surfaces the active level; a cross-mode operation is an explicit `Option`/error.
- **ADR-014** (relaxed the global `forbid unsafe` → `permitted-but-warned`) is the precedent for
  relaxing a global rule transparently — and the model for memory safety here: **safe by default, with an
  *explicit per-use* escape hatch** (§3.1) so unsafe mem ops are a conscious, visible choice at the use
  site, never an ambient default.

**The reframe is bigger than a knob — it repositions the north star (owner-steered, 2026-06-24).** The
intended end-state is **Mycelium as a fast, efficient, memory-safe, easy-to-use multi-paradigm language**,
with the certification / honesty / hashing machinery **baked in as *optional* capabilities** engaged when
needed — not a tax paid on every line. The maximalist phase's procedures are *retained as an adjustable
framework*; the day-to-day posture relaxes toward speed and ergonomics. This is a charter-level shift (it
touches the Foundation §1 framing, **SC-3**, **FR-M3**), so it is **captured here and flagged** for the
binding RFC-0034 + superseding ADR (§7, §10) — DN-29 does not amend the charter itself (append-only; VR-5).
Three pillars are now first-class *goals*, mostly **inherited, to be kept** rather than newly built:
**memory safety** (the kernel is Rust, `#![forbid(unsafe)]`-rooted save ADR-014's warned escape; the surface
exposes no raw pointers), **speed/efficiency** (the `fast` profile pays for none of the runtime cert/hash
machinery), and **ergonomics** (certification is opt-in, never in the way).

## §2 The load-bearing distinctions — two axes × two phases

The phrase "sign and hash everything" hides **two independent knobs**. Separating them is the crux:

- **Axis A — certification depth** *(the expensive target).* Swap-certificate emission/checking, the
  ADR-010 bound-proof kernels, guarantee-*proof* machinery, and the *pervasiveness* of content-hashing.
  Dial **off → full**.
- **Axis B — failure semantics** *(cheap correctness; should usually stay on).* Never-silent out-of-range
  via `Option`/`Result` (e.g. `crates/mycelium-core/src/binary.rs::int_to_bits` returns `None` out of
  range; `ternary/mod.rs`; the swap `SwapError` variants). This is **O(1) logic, not certification
  overhead** — turning it off buys little speed and costs correctness. Recommended default-on, with an
  explicit, named opt-in for fast/wrapping arithmetic if ever wanted.

The corpus today conflates these. The cost the maintainer is reacting to is **Axis A** (cert checking +
proof kernels + hashing everywhere) — *not* the cheap Axis-B range checks.

**The phase split (owner-steered keystone, 2026-06-24).** Axis A is not one cost — it splits by ***when*
the work happens**, and the two phases are independently switchable:

- **Compile / deploy phase** — *per-artifact, cheap, valuable; keep available even at the lowest runtime
  setting.* Content-hashing a **deployable unit** mints its **spore identity** (ADR-013) and the
  hot-inject / ABI **dispatch keys** (ADR-016/017). This is paid **once per built artifact**, not per
  runtime operation, and it is exactly what makes units deployable, content-addressed, and dedup-able.
- **Runtime phase** — *per-operation, pervasive, the expensive part; off by default.* Hashing **values**
  during execution, propagating guarantee tags, **emitting** swap certificates, and **checking** them
  against the ADR-010 bound kernels — repeated across the whole functionality set as the program runs.

The decisive consequence: **deployability survives a fully cert-off runtime.** You can ship spores —
content-addressed, deployable units — *without* paying any runtime hash/cert tax, because spore identity is
a compile/deploy concern. This is why "hashing" is not one knob but **two**, and why the lowest *runtime*
profile need not forfeit spores (correcting the §3 strawman's "L0 loses spores" coupling).

So the design space is a small **matrix of independent knobs** (below), not a single linear scalar — they
*compose*, and convenience **profiles** are presets over them.

## §3 The knob matrix + profiles (owner-steered model — for discussion, not decided)

Rather than one linear ladder, the design space is a set of **independent knobs grouped by phase**; named
**profiles** are presets over them. (This supersedes the earlier single L0–L3 strawman, kept below in §3.2
only as the "preset" intuition.)

### §3.1 The knobs

| Knob | Phase | Cost | Consumes / enables | Recommended floor |
|---|---|---|---|---|
| **Spore / deploy identity hash** | compile / deploy | cheap, **per-artifact** | spores (ADR-013), hot-inject + ABI dispatch keys (ADR-016/017), dedup | **available even when runtime is fully off** |
| **Memory safety** | always | inherited (Rust kernel) | the safe surface; no raw pointers | **safe by default; *explicit per-use* unsafe escape hatch** (see below) |
| **Never-silent failure** (Axis B) | runtime | O(1) | `Option`/`Result`/`SwapError` out-of-range | **on** (named `wrapping`/`fast` opt-out) |
| **Provenance tagging** *(adjustable unit)* | runtime | `Exact`/`Declared` O(1); `Empirical`=trials, `Proven`=checked theorem | the `Exact⊐Proven⊐Empirical⊐Declared` lattice | **`fast`: structural `Exact`/`Declared` only — `Empirical`/`Proven` not used**; heavier tags dialled up per mode/unit |
| **Signal generation** (inspectability trace) | runtime | cheap | the "what happened / which swap / why / how" trace `EXPLAIN` reads | **safe default; always generated ≥ middle tier** so consumption can always be dialled up |
| **Diagnostic consumption / DX surfacing** | tooling | ~O(1) | how much of the generated signal surfaces in DX/UX | **tunable; `fast` lean** (signal still generated, just not shown) |
| **Runtime value hashing / dedup** | runtime | O(size) | runtime identity/dedup of values | off |
| **Swap-cert emission** | runtime | O(1) | a certificate object per swap | off |
| **Swap-cert checking** | runtime | **expensive** | M-210 checker + ADR-010 bound kernels | off |

The knobs compose; out-of-range / cross-knob interactions stay never-silent (§6). Two are *not* really
"Axis A cost" and stay on cheaply: **memory safety** (inherited) and **Axis-B never-silence** (O(1)).

**Memory safety — safe by default, explicit per-use escape (owner-decided, 2026-06-24).** The surface is
**memory-safe by default**; unsafe mem ops are reachable **only** through an *explicit, per-use* escape
hatch at the call site (the ADR-014 `permitted-but-warned` precedent, sharpened from a global toggle to a
**per-use** opt-in) — so the dev must consciously think at each use, and the escape is visible/auditable in
the source. This is independent of the certification profile: even `fast` is memory-safe.

**Provenance tagging is an adjustable unit; and generation is split from consumption (owner-steered,
2026-06-24 rev. 2).** Two refinements keep the lightweight value on by default *without* cornering anyone:

- **The provenance tag is tunable per unit.** `fast` defaults to **not using `Empirical`/`Proven`** — those
  cost *trials* / *checked theorems*, the very work `fast` skips — so `fast` results sit at the structural
  `Exact`/`Declared` tags. Computing the heavier tags is dialled up per mode or per unit, never forced. This
  is the honest floor: `fast` doesn't *claim* `Empirical` because it didn't run the trials (VR-5).
- **Generation ≠ consumption.** The cheap, valuable *signal* — the inspectability trace (what happened,
  which swap, why, how) — is **always generated from the middle tier up** (a *safe default*), so the data
  exists to inspect. What's tunable is **consumption**: how much of that signal the DX/UX surfaces. `fast`
  defaults to lean output, but the signal is still generated, so a dev can **dial consumption up
  mid-session and the history is already there** — no re-run, no mode switch, no painting-into-a-corner.

The stance this encodes: **give devs the tools, the options, and the reasoning behind why each exists — and
let them choose whether to use them.** Lightweight-value-on-by-default + an always-generated signal +
tunable consumption is how `fast` stays *both* cheap *and* non-cornering.

### §3.2 Modes (presets over the knobs)

**Two first-class modes — `fast` and `certified` (owner-decided, 2026-06-24)** — are the anchors the
language is designed around; **`balanced` is an optional intermediate**, not a headline.

| Mode | First-class? | Spore hash (compile) | Mem-safe | Axis-B | Diagnostics | Tags | Runtime hash | Cert emit | Cert check | Character |
|---|---|---|---|---|---|---|---|---|---|---|
| **`fast`** ***(default)*** | **yes** | available | safe | on | lean | off | off | off | off | fast, memory-safe, ergonomic, **inspectable + still deployable** |
| **`balanced`** *(intermediate)* | no | available | safe | on | medium | propagated (unchecked) | off | emitted (unchecked) | off | provenance tags + certs, not verified |
| **`certified`** | **yes** | available | safe | on | full audit trail | tracked | on | emitted | **checked** (ADR-010) | today's all-on behaviour; **fully auditable**, max assurance |

The two first-class modes name the two postures a dev actually chooses between: **`fast`** — a fast,
memory-safe, ergonomic language whose ops are still transparent/inspectable (non-certified auditability) —
and **`certified`** — the same language with the full, checked, certificate-backed auditable framework
engaged. `certified` is the union of the maximalist phase's machinery (§8), now **engaged on request**
rather than always. The earlier L0–L3 ladder maps roughly: `fast ≈ L0`-but-**with spores + tags-available
kept**, `balanced ≈ L1`, `certified ≈ L2/L3` (L3 = `certified` with `Proven` side-conditions checked).

**The real catch, corrected.** The §3-strawman coupled "lowest level ⇒ lose spores." The phase split
(§2) **breaks that coupling**: spore identity is a *compile/deploy* hash, so **`fast` keeps spores** while
paying no *runtime* hash/cert cost. Turning off the *compile* spore hash is a separate, deliberate choice
(embedded/no-deploy builds) that MUST **explicitly disable and `EXPLAIN`** the loss of spores/inject —
never-silent about *capabilities*, not just values.

## §4 Feature-dependency graph (what each tier actually costs)

Grounded in the machinery audit (`crates/mycelium-core/src/content.rs`, `guarantee.rs`, `meta.rs`,
`bound.rs`; `crates/mycelium-cert/*`; `crates/mycelium-spore/*`; `crates/mycelium-mlir/src/inject.rs`):

- **Content-hash (BLAKE3, `content.rs`)** — hashes `Repr` + literal `Payload` (dynamic metadata
  excluded). **Consumed by:** spores (`spore_id`), hot-inject (dispatch keys), dedup. Turn off ⇒ those
  features go dark (must be flagged, never silent).
- **Guarantee lattice (`guarantee.rs`/`meta.rs`)** — O(1) `meet`/propagate, enforced by `Meta::new`.
  Cheap; "off" means *don't compute proofs*, results pinned `Declared`.
- **Swap certificates (`mycelium-cert`, `bound.rs`)** — emitted + checked per swap. The M-210 checker
  together with the ADR-010 bound kernels are the expensive part Axis A dials.
- **Never-silent failures (Axis B)** — `Option`/`Result`/`SwapError`; orthogonal, cheap, default-on.
- **Existing optionality precedent** — only the `mlir-dialect` Cargo feature exists today; **no** cert
  feature-gates, **no** runtime mode system. This is greenfield.

## §5 The knob's home + scoping (reuse, zero kernel change)

Ride the **RFC-0012 ambient-scoping + `crates/mycelium-proj` manifest/header resolver** — the exact path
RFC-0017's `@matured` took (Enacted; the machinery exists):

- **Manifest** (`crates/mycelium-proj/src/manifest.rs`): `[project].certification = "fast"` (the default
  if omitted) — a **profile** name (`fast`/`balanced`/`certified`, §3.2); individual knobs (§3.1) may be
  overridden under it (e.g. `cert.spore_hash = true`, `cert.runtime_check = false`) for the rare
  off-profile mix.
- **Header** (`crates/mycelium-proj/src/header.rs`): `// @certification: certified` — extend the **closed**
  `HEADER_KEYS` (closed-key discipline: an unknown key is an explicit error, G2).
- **Resolver** (`crates/mycelium-proj/src/resolve.rs`): reuse the inheritance + `Origin{Local |
  ProjectManifest}` provenance + the existing `explain()` — *"where did this profile/knob come from?"* is
  answerable (no black box).
- **Default = `fast` (owner-decided, 2026-06-24).** A project that specifies nothing gets `fast`
  (memory-safe, never-silent, spores available; runtime cert/hash/check off); certification is **opt-in**
  per-phylum / per-nodule. (Resolves §9 Q3.)
- **Scoping ladder:** global default (`fast`) → `[project]` (per-phylum) → `@certification` (per-nodule) →
  *(future)* a `thaw`-style per-op override. **Profile/knob state is associated metadata (ADR-003) ⇒ it
  does NOT perturb content-address identity** (same code+payload → same spore hash regardless of profile).
- **Secondary axis — Cargo features** (`--no-default-features` / `--features cert-full`): *compile out*
  the heavy machinery for embedded/perf builds. Distinct from the runtime per-scope level; not the
  primary knob (features are crate-wide binary, not per-scope scalar).
- **Not RFC-0005 selection policy for v0** — overkill for a scalar knob (it is for multi-candidate
  selection); revisit only if cert level ever becomes data-driven per-op.

## §6 Transparency & auditability argument *(was: "honesty-preservation")*

Tunability stays **transparent and accurate** at every setting — *transparency is disclosure-of-strength
plus inspectable ops, not universal certification* — and the corpus already says so. The two tiers of the
guarantee:

- **`fast` = transparent + non-certified auditability.** Ops are inspectable and never overclaim, but the
  audit trail is *unchecked* — you can see *what happened and why*, you just don't get a certificate proving
  it. Cheap, debuggable, the everyday default.
- **`certified` = fully auditable.** The same trail, now checked and certificate-backed — a fully auditable
  framework, engaged on request.

Why it holds:

- The **`Declared`** tier exists precisely for *"asserted, not proven, always flagged."* `fast` computes a
  result and tags it `Declared` + `mode: fast`. That is the *intended* use of the weakest tier, applied
  systematically.
- **VR-5 / RFC-0014** already permit *downgrade-and-disclose* (recovery/fallbacks "never fabricate or
  upgrade a guarantee"). A tunable mode is a *systematic, flagged downgrade*.
- **G2** is satisfied so long as the mode is **inspectable** and **cross-mode operations are explicit**:
  a value computed in `fast` carries `mode: fast`; combining it into a `certified` computation is an
  explicit, visible event (never a silent upgrade — disclosure can only degrade, RFC-0001 §3.4).
- **ADR-010's "tier-i pragmatic checker"** already trades a full prover for checked arithmetic; tunability
  extends the ladder downward to "tier-0: no check, transparent about it."

What stays non-negotiable even in `fast`: **the mode is never hidden, and no result ever claims a strength
it did not compute.** That single rule is the whole transparency contract; everything else becomes opt-in.

## §7 Corpus reach (preview — nothing amended here)

The binding RFC-0034 + a superseding ADR would *conditionalize* (not delete) these always-on statements:

- **RFC-0001** §3.3 (no implicit conversions), §3.4 (mandatory lattice propagation), §4.6
  (content-addressing as identity), §4.3 well-formedness M-I1…M-I4.
- **RFC-0002** §2 (per-swap certificate mandatory) — becomes per-level.
- **RFC-0005** §2 (mandatory EXPLAIN for selections) — EXPLAIN of the *mode* stays mandatory.
- **ADR-010 / ADR-011** (bound kernels / universal `BoundBasis`) — invoked when certification ≥ L2.
- **ADR-013 / 016 / 017** (spore / ABI / hot-inject) — gated on the hashing sub-toggle; their identity
  contract is stated per-level.
- **ADR-014** (unsafe `permitted-but-warned`) — sharpened toward **safe-by-default with an *explicit
  per-use* escape hatch** (§3.1); memory safety is a first-class default, not merely "warned."
- **Vocabulary: "honesty" → "transparency & auditability"** — **CLAUDE.md house-rule 1 ("The honesty
  rule")**, **CONTRIBUTING.md**, and the **VR-5 / G2** phrasing reframe from a moral "honesty / never lie"
  register to pragmatic *transparent, inspectable ops + (optional) certified auditability*. The
  **mechanism is unchanged** (never-silent, the provenance lattice, `EXPLAIN`, downgrade-don't-overclaim);
  only the framing/wording moves. Charter-level — **flagged, not amended here** (append-only).
- **Foundation §1 north star** — the headline framing shifts from *"certified, never-silent substrate with
  honest per-operation guarantees"* toward *"a fast, memory-safe, ergonomic multi-paradigm language, with
  certified/auditable semantics as optional, baked-in capabilities."* Memory-safety + speed + ergonomics
  rise to first-class goals (§1). **Charter-level — flagged, not amended here** (append-only); the binding
  RFC-0034 + superseding ADR carry it.
- **Foundation** SC-3 / FR-M3 / KC-4, and the transparency-rule wording in **CLAUDE.md** /
  **CONTRIBUTING.md** — reworded from "always" to "at the active mode; the mode itself is never silent."

## §8 What we keep (the maximalist phase earned these)

The overzealous phase produced **good, reusable procedures** — they become the *implementation of the
upper tiers*, not a tax on everyone: the ADR-010 bound kernels, the RFC-0002 swap-certificate
split-regime, **ADR-011** (`BoundBasis` is universal — within certified levels), the four-point guarantee
lattice, the M-210 checker. None of this is discarded; it is **gated behind the level** that asks for it.

## §9 Open questions (the deliberation agenda)

**Resolved (owner-steered, 2026-06-24 — folded into §2/§3/§5/§10):**
- ~~Q1 **Ladder shape**~~ → **decomposed into independent knobs + 3 profiles** (`fast`/`balanced`/
  `certified`, §3); the single linear ladder is retired as the model (kept only as preset intuition).
- ~~Q2 **Hashing's place**~~ → **split by phase** (§2): compile/deploy **spore-identity hash kept available
  even when runtime cert is fully off**; runtime value-hashing is its own off-by-default knob. The lowest
  *runtime* profile no longer forfeits spores.
- ~~Q3 **Default level**~~ → **`fast`** (memory-safe, never-silent, spores available; runtime cert/hash/
  check off); certification is **opt-in** (§5).

**Resolved (owner-steered, 2026-06-24 rev. 2 — folded into §1/§3/§6/§10):**
- ~~Q2′ **First-class modes**~~ → **`fast` and `certified` are both first-class modes** (the two anchors);
  `balanced` is an optional intermediate (§3.2).
- ~~Q8 **Memory safety**~~ → **safe by default with an *explicit per-use* unsafe escape hatch** (§3.1);
  independent of certification profile.
- ~~Q9 **"Honesty" framing**~~ → **reframed to "transparency & auditability"** (§1/§6); mechanism
  unchanged, charter-vocabulary ripple flagged (§7).
- ~~Q10 **Diagnostic verbosity**~~ → its own **tunable knob, mode-defaulted** (lean at `fast` → full audit
  trail at `certified`), independently overridable; **tagging stays on** as the inspectability substrate
  (§3.1).

**Still open:**
4. **Axis B** — model says **default-on with a named `wrapping`/`fast` opt-out** (§3.1); confirm that
   opt-out is exposed in v0 vs. deferred.
5. **Per-op granularity** — ship v0 at global/phylum/nodule scope; defer the per-op `thaw`-style knob?
6. **Form/sequencing** — DN-29 (this) → settle → **RFC-0034** + superseding ADR + the amendments. Confirm
   the two-step (deliberate-then-decide) vs. going straight to a `Proposed` RFC.
7. **Mode knob-overrides** — expose individual knob overrides under a mode (§5) in v0, or ship the named
   modes only and add knob-level overrides later?

## §10 Decisions captured this revision (2026-06-24, owner-steered — still advisory)

These are **recorded directions**, not normative enactments (the binding act is RFC-0034 + the superseding
ADR; VR-5/G2 — DN-29 enacts nothing):

1. **Phase-decomposed knobs over a single ladder.** Certification is a small matrix of independent knobs
   grouped by *compile/deploy* vs *runtime* phase (§3.1), with `fast`/`balanced`/`certified` profiles as
   presets (§3.2).
2. **Spores survive a cert-off runtime.** Compile/deploy spore-identity hashing is decoupled from runtime
   certification, so deployable, content-addressed units are available at every runtime profile (§2).
3. **`fast` is the default; certification is opt-in** (§5).
4. **North-star reframe captured.** Mycelium repositions toward a fast, memory-safe, ergonomic
   multi-paradigm language with certification baked in as optional (§1). The Foundation §1 / SC-3 / FR-M3
   ripple is **flagged for RFC-0034 + the superseding ADR** (§7) — *not* amended here (append-only).
5. **`fast` and `certified` are both first-class modes** (§3.2); `balanced` is an optional intermediate.
6. **Memory-safe by default + explicit per-use unsafe escape hatch** (§3.1) — a conscious, visible opt-in
   at each use site (ADR-014 precedent, sharpened to per-use); independent of the certification mode.
7. **"Honesty" reframed to "transparency & auditability"** (§1/§6): default `fast` = transparent +
   inspectable *non-certified auditability*; `certified` = the same trail as a *fully auditable* framework.
   Mechanism unchanged (never-silent, provenance lattice, `EXPLAIN`, VR-5); the CLAUDE.md house-rule 1 /
   CONTRIBUTING / VR-5 / G2 vocabulary ripple is **flagged for RFC-0034** (§7) — not rewritten here.
8. **Provenance tag is an adjustable unit** (§3.1): `fast` defaults to **not using `Empirical`/`Proven`**
   (they cost trials/proofs it skips), sitting at structural `Exact`/`Declared`; heavier tags dial up per
   mode/unit. Honest floor — `fast` never *claims* a tag it didn't earn (VR-5).
9. **Signal generation is split from consumption** (§3.1): the cheap inspectability signal is **always
   generated ≥ middle tier** (safe default), while **consumption** (DX/UX surfacing, diagnostic noise) is
   tunable and lean at `fast` — so a dev can dial consumption up mid-session with the history already
   captured. The stance: *give the tools + the reasoning; let devs choose to use them.*
10. **Ripple map built** (§11): a 40-hit, 12-file inventory of every corpus location the binding RFC-0034 +
    superseding ADR must touch, plus an **anchor-keyed, single-pass-per-file** replacement mechanism that
    avoids positional mangling. Inventory is `Empirical/Declared` (re-verify against source at RFC-0034
    time); spine anchors (SC-3, FR-M3, VR-5) validated.

## §11 Ripple map — what RFC-0034 + the superseding ADR must amend

**Status: `Empirical/Declared` inventory** (a line/regex sweep, 2026-06-24; spine anchors SC-3 / FR-M3 /
VR-5 spot-validated against source). It scopes the amendment surface; it is **not** the amendment (that is
the binding RFC-0034 + ADR — append-only). Re-verify each `file:line` against source at drafting time, as
line numbers drift.

### §11.1 Change-type taxonomy (what each edit does)

| Type | Meaning | Example targets |
|---|---|---|
| **conditionalize-per-mode** | "every/always/mandatory" → "at the active mode (e.g. cert ≥ `certified`); the mode itself is never silent" | SC-3, FR-M3, RFC-0002 §2, RFC-0005 §2, G2/SC-3 invariants |
| **reword honesty→transparency** | "honesty / honest / never lie" → "transparency & auditability / accurate" (mechanism unchanged) | CLAUDE.md house-rule 1, CONTRIBUTING §honesty-rule, Glossary, VR-5 colloquial |
| **north-star reframe** | headline "certified, never-silent substrate / honest guarantees" → "fast, memory-safe, ergonomic language; certification optional" | CLAUDE.md §what-this-repo-is, README opening, Foundation §1 mission |
| **tag-now-adjustable** | "every value carries / meet-propagates a tag" → "tag is computed at cert ≥ L1; `fast` is structural `Exact`/`Declared`, composition is identity" | RFC-0001 §3.4, §4.3 (M-I4), §4.6 lattice + worked example |
| **memory-safety sharpen** | ADR-014 `permitted-but-warned` → "safe by default + explicit *per-use* escape hatch" | CONTRIBUTING §unsafe, ADR-014 (sharpened, not superseded) |

### §11.2 Surface summary (sweep, 2026-06-24)

- **~40 hits across ~12 files.** Change-type distribution (approx): reword-honesty **~14**, conditionalize
  **~12**, north-star **~8**, tag-adjustable **~6**, memory-safety **~2**.
- **High-collision files (>5 hits) — amend as one batched pass each:** `README.md` (~10), `Foundation` (~9),
  `CLAUDE.md` (~8).
- **Spine mandates (validated):** `Foundation:53` SC-3, `Foundation:73` FR-M3, `Foundation:113` VR-5 (note:
  VR-5's *formal* def is narrow — Gaussian-approx ⇒ `Empirical`; the colloquial "downgrade to stay honest"
  rule lives in `Glossary:94` + CLAUDE.md). Plus RFC-0001 §3.3/§3.4/§4.3/§4.6, RFC-0002 §2, RFC-0005 §2.

### §11.3 Multi-category lines (highest mangle-risk — rewrite whole, don't sequential-edit)

These single lines need **two or three** change-types at once; sequential edits drift positions and risk
mangling — each is rewritten as **one** replacement:

- `CLAUDE.md:14-15` — *north-star* **+** *honesty→transparency* ("certified, never-silent … honest, per-operation guarantees").
- `README.md:52-53` — *honesty→transparency* **+** *north-star* **+** *tag-adjustable* ("Honesty is a typed, monotone property … meet").
- `Foundation:53` (SC-3) — *conditionalize* **+** *G2/never-silent reword*.
- `Foundation:73` (FR-M3) — *conditionalize* ("always emits a certificate") **+** vocabulary.
- `RFC-0001:27` — *honesty→transparency* **+** *tag-adjustable* **+** VR-5-consequence reword.

### §11.4 Batched-replacement mechanism (avoids positional mangling)

Execution (built + run **with** RFC-0034, not now):

1. **Manifest** `{file: [{anchor, replacement, category}, …]}` — `anchor` is a **unique content substring**,
   never a line/offset, so matches are position-independent.
2. **One pass per file:** read once → apply all replacements to the in-memory string → write once. Replacing
   one anchor never invalidates another's match (content-keyed, not positional) ⇒ no recalc, no rescan,
   order-independent.
3. **Never-silent guard on the tool itself (G2):** assert each anchor matches **exactly once** before
   applying; a missing/ambiguous anchor *fails loudly* (the tooling obeys the rule the corpus does). The
   multi-category lines (§11.3) appear as a **single** anchor→replacement each, so they are rewritten whole.

A small one-shot generator (e.g. `tools/dn29-apply.py`) consumes the manifest; the manifest is authored
from this map once RFC-0034 fixes the final wording.

### §11.5 Process decisions the sweep surfaced (owner to confirm)

- **Vocabulary scope = whole-corpus** (not CLAUDE.md only): CONTRIBUTING, Foundation, RFCs, Glossary all
  reframe honesty→transparency. *(lean: yes — partial reframe would read inconsistent.)*
- **The superseding ADR amends the Foundation/charter; RFC-0034 stays implementation-focused.** *(lean: yes
  — keeps the charter change in the decision record, per append-only.)*
- **Backward-compat footnotes** on the Accepted RFCs/ADRs (RFC-0001/0002/0005, ADR-010/011/013/016/017):
  a §-end note "mandates apply at `certified`; `fast`/`balanced` relaxations per RFC-0034 + ADR-xxxx." *(lean:
  yes — append-only, preserves the originals while pointing forward.)*

---

> **Provenance.** Grounded in a machinery/coupling audit of `mycelium-core`/`-cert`/`-spore`/`-mlir`, a
> reuse survey of RFC-0012 / RFC-0017 / `mycelium-proj`, and a corpus survey of the always-on honesty
> framing (SC-3/FR-M3/VR-5/G2/KC-4). Advisory only; **superseded** (append-only) by the binding RFC-0034
> when the deliberation settles. No normative claim is made by this note (VR-5 / G2).
>
> **Revision history.** *2026-06-24* — original Draft (deliberation anchor; #537). *2026-06-24 (rev.)* —
> owner-steered refinement: phase-decomposed knob matrix + profiles replacing the single ladder (§2/§3),
> compile-time spore hashing decoupled from runtime certification (§2), `fast` default (§5), north-star
> reframe captured with the charter ripple flagged (§1/§7/§10), open questions Q1–Q3 resolved (§9). Status
> stays **Draft** (still advisory; a Draft is iterated in place, not status-advanced — append-only honored).
> *2026-06-24 (rev. 2)* — owner-steered: **`fast` + `certified` elevated to first-class modes** (`balanced`
> intermediate, §3.2); **memory-safe by default + explicit per-use unsafe escape hatch** (§3.1); a
> **diagnostic-verbosity knob** tied to the mode, tagging kept on (§3.1); and the **"honesty" → "transparency
> & auditability" reframe** (§1/§6) with the CLAUDE.md/CONTRIBUTING/VR-5/G2 vocabulary ripple flagged (§7).
> Mechanism unchanged; still **Draft**, still advisory. *2026-06-24 (rev. 3)* — owner-steered:
> **provenance tag is an adjustable unit** (`fast` omits `Empirical`/`Proven`, sits at `Exact`/`Declared`;
> §3.1/§10.8); **signal generation split from consumption** (signal always generated ≥ middle tier, DX/UX
> consumption tunable and lean at `fast`; §3.1/§10.9); and a **§11 ripple map** — a ~40-hit/~12-file
> `Empirical/Declared` inventory of the RFC-0034 amendment surface (change-type taxonomy, high-collision +
> multi-category lines, spine anchors validated) plus an **anchor-keyed single-pass-per-file** batched
> replacement mechanism with a never-silent guard. Still **Draft**, still advisory.
