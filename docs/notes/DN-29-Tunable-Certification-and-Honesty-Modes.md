# Design Note DN-29 — Tunable Certification & Honesty Modes

| Field | Value |
|---|---|
| **Note** | DN-29 |
| **Status** | **Draft** (2026-06-24; deliberation anchor — advisory, non-committal) |
| **Feeds** | the eventual **RFC-0034** (binding decision) + a superseding ADR; conditionalizes RFC-0001 §3.3/§3.4/§4.6, RFC-0002 §2, RFC-0005 §2, ADR-010, ADR-011, ADR-013/016/017; grounds in **KC-4**, **VR-5**, **G2**, ADR-014 |
| **Date** | June 24, 2026 |
| **Decides** | *Nothing normatively* — advisory. Records the maintainer's intended **shift** (2026-06-24): make the certification / honesty / hashing machinery **tunable from fully-off → fully-engaged** via scoped config, instead of mandatory-everywhere, **while preserving honesty and keeping the good procedures** the early maximalist phase produced. Captures the design space, a strawman ladder, the feature-dependency consequences, and the open questions, so the binding RFC-0034 is shaped toward this end-state. The binding decision is the future RFC. |
| **Task** | value-model / honesty-model evolution (pre-RFC-0034 deliberation) |

> **Posture (honesty rule / VR-5).** Advisory, forward-looking. **Enacts nothing; moves no status; changes
> no normative text.** What exists **today** is the all-on machinery (SC-3 / FR-M3: every swap emits +
> checks a certificate; every value is content-hashed; every value carries a guarantee tag). This note
> records a *direction* — a tunable model — and the argument that it stays honest. The binding change is
> RFC-0034 + a superseding ADR; until then the all-on rules hold. No code or guarantee is claimed here
> (VR-5 / G2).

---

## §1 Purpose & the reframe

The corpus mandates the full honesty/certification/hashing machinery **unconditionally**: every value
carries a `GuaranteeStrength` (RFC-0001 §3.4), every swap emits + checks a certificate (**SC-3**,
**FR-M3**, RFC-0002 §2), every value/definition is content-hashed (RFC-0001 §4.6). The maintainer's
assessment (2026-06-24): *"certification everywhere is messy and expensive"* — devs/users should dial it
from **fully off → fully engaged**, and disable the machinery entirely when undesired, **keeping** the
genuinely good procedures the maximalist phase produced.

**The keystone: honesty ≠ mandatory maximal certification.** Honesty = *never claim more than you
established.* Certification *depth* = *how much you bother to establish.* Splitting these makes the
expensive machinery a **tunable policy**, and honesty **survives at every setting** because the corpus
already contains the hooks:

- **KC-4** (Foundation §2.4) already authorizes downgrading *certified → declared-and-property-tested*
  on cost grounds — "document the loss." Tunable certification **generalizes KC-4 from a one-time
  kill-switch into a knob.**
- **VR-5** ("downgrade to stay honest; never upgrade without a checked basis") + the **`Declared`** tier
  (RFC-0001 §4.3 — *"always flagged; never silently trusted"*) make "off" a *systematic, flagged
  downgrade to `Declared`*, **not** a lie.
- **G2 (never-silent)** binds the *mode itself*: results carry an explicit certification-mode tag;
  tooling always surfaces the active level; a cross-mode operation is an explicit `Option`/error.
- **ADR-014** (relaxed the global `forbid unsafe` → `permitted-but-warned`) is the precedent for
  relaxing a global rule honestly.

## §2 Two orthogonal axes (the load-bearing distinction)

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

## §3 Strawman level ladder (Axis A — for discussion, not decided)

| Level | Guarantee tags | Swap certs | Bound proofs | Content-hash | Features kept / lost |
|---|---|---|---|---|---|
| **L0 Raw** | none (all `Declared`) | none | none | *sub-toggle* | fastest; **loses spores + hot-inject** if hashing also off |
| **L1 Tagged** | tracked + propagated, **unchecked** | none | none | on | identity + dedup; honest tags, unverified |
| **L2 Certified** *(proposed default)* | tracked | emitted + checked (`Empirical`/`Bounded`) | pragmatic (ADR-010 tier-i) | on | swaps validated; spores/inject work |
| **L3 Proven** *(today's behaviour)* | tracked | + `Proven` w/ checked side-conditions | full | on | maximal assurance |

**The honest catch (from the dependency map, §4).** Content-hashing is what **spores** (ADR-013) and
**hot-inject / ABI dispatch** (ADR-016/017) *consume* as identity. So hashing is best modeled as its
**own sub-toggle**, and any level that turns it off MUST **explicitly disable and `EXPLAIN`** the loss of
those features — never-silent about *capabilities*, not just values. Guarantee-tagging (O(1)) and Axis-B
never-silence are cheap; the real cost is cert **checking** + the proof kernels + hashing pervasiveness.

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

- **Manifest** (`crates/mycelium-proj/src/manifest.rs`): `[project].certification = "l2"`.
- **Header** (`crates/mycelium-proj/src/header.rs`): `// @certification: l2` — extend the **closed**
  `HEADER_KEYS` (closed-key discipline: an unknown key is an explicit error, G2).
- **Resolver** (`crates/mycelium-proj/src/resolve.rs`): reuse the inheritance + `Origin{Local |
  ProjectManifest}` provenance + the existing `explain()` — *"where did this level come from?"* is
  answerable (no black box).
- **Scoping ladder:** global default → `[project]` (per-phylum) → `@certification` (per-nodule) →
  *(future)* a `thaw`-style per-op override. **Cert level is associated metadata (ADR-003) ⇒ it does NOT
  perturb content-address identity** (same code+payload → same hash regardless of cert level).
- **Secondary axis — Cargo features** (`--no-default-features` / `--features cert-full`): *compile out*
  the heavy machinery for embedded/perf builds. Distinct from the runtime per-scope level; not the
  primary knob (features are crate-wide binary, not per-scope scalar).
- **Not RFC-0005 selection policy for v0** — overkill for a scalar knob (it is for multi-candidate
  selection); revisit only if cert level ever becomes data-driven per-op.

## §6 Honesty-preservation argument

Tunability is honest **because honesty is disclosure-of-strength, not universal certification** — and the
corpus already says so:

- The **`Declared`** tier exists precisely for *"asserted, not proven, always flagged."* "Off" computes a
  result and tags it `Declared` + `mode: Off`. That is the *intended* use of the weakest tier, applied
  systematically.
- **VR-5 / RFC-0014** already permit *downgrade-and-disclose* (recovery/fallbacks "never fabricate or
  upgrade a guarantee"). A tunable mode is a *systematic, flagged downgrade*.
- **G2** is satisfied so long as the mode is **inspectable** and **cross-mode operations are explicit**:
  a value computed at L0 carries `mode: Off`; combining it into an L3 computation is an explicit,
  visible event (never a silent upgrade — disclosure can only degrade, RFC-0001 §3.4).
- **ADR-010's "tier-i pragmatic checker"** already trades a full prover for checked arithmetic; tunability
  extends the ladder downward to "tier-0: no check, honest about it."

What stays non-negotiable even at L0: **the mode is never hidden, and no result ever claims a strength it
did not compute.** That single rule is the whole honesty contract; everything else becomes opt-in.

## §7 Corpus reach (preview — nothing amended here)

The binding RFC-0034 + a superseding ADR would *conditionalize* (not delete) these always-on statements:

- **RFC-0001** §3.3 (no implicit conversions), §3.4 (mandatory lattice propagation), §4.6
  (content-addressing as identity), §4.3 well-formedness M-I1…M-I4.
- **RFC-0002** §2 (per-swap certificate mandatory) — becomes per-level.
- **RFC-0005** §2 (mandatory EXPLAIN for selections) — EXPLAIN of the *mode* stays mandatory.
- **ADR-010 / ADR-011** (bound kernels / universal `BoundBasis`) — invoked when certification ≥ L2.
- **ADR-013 / 016 / 017** (spore / ABI / hot-inject) — gated on the hashing sub-toggle; their identity
  contract is stated per-level.
- **Foundation** SC-3 / FR-M3 / KC-4, and the honesty-rule wording in **CLAUDE.md** / **CONTRIBUTING.md**
  — reworded from "always" to "at the active certification level; the level itself is never silent."

## §8 What we keep (the maximalist phase earned these)

The overzealous phase produced **good, reusable procedures** — they become the *implementation of the
upper tiers*, not a tax on everyone: the ADR-010 bound kernels, the RFC-0002 swap-certificate
split-regime, **ADR-011** (`BoundBasis` is universal — within certified levels), the four-point guarantee
lattice, the M-210 checker. None of this is discarded; it is **gated behind the level** that asks for it.

## §9 Open questions (the deliberation agenda)

1. **Ladder shape** — 4 tiers as above, or fewer/more? Is L1 (tagged-but-unchecked) worth its own tier?
2. **Hashing's place** — its own sub-toggle (recommended) vs. folded into L0? Accept losing spores/inject
   at the lowest setting, or make hashing the *floor* that never turns off?
3. **Default level** — `L2 Certified` (safe default) vs `L0/L1` (cheap default, opt-in rigor)?
4. **Axis B** — keep never-silent always-on, or also expose a wrapping/fast opt-in?
5. **Per-op granularity** — ship v0 at global/phylum/nodule scope; defer the per-op `thaw`-style knob?
6. **Form/sequencing** — DN-29 (this) → settle → **RFC-0034** + superseding ADR + the amendments. Confirm
   the two-step (deliberate-then-decide) vs. going straight to a `Proposed` RFC.

---

> **Provenance.** Grounded in a machinery/coupling audit of `mycelium-core`/`-cert`/`-spore`/`-mlir`, a
> reuse survey of RFC-0012 / RFC-0017 / `mycelium-proj`, and a corpus survey of the always-on honesty
> framing (SC-3/FR-M3/VR-5/G2/KC-4). Advisory only; **superseded** (append-only) by the binding RFC-0034
> when the deliberation settles. No normative claim is made by this note (VR-5 / G2).
