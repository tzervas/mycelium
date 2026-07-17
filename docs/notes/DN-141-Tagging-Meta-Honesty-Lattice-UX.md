# Design Note DN-141 — Tagging · Meta · Transparency-Lattice UX · Isolation (Airlock / Firewall / Quarantine / Meet-Boundary)

| Field | Value |
|---|---|
| **Note** | DN-141 |
| **Status** | **Draft** (2026-07-16; integrated 2026-07-17) — design recommendation only; **does not ratify** (house rule #3). Builds nothing. Mechanism claims stay `Declared` until build issues land and are differential-witnessed (VR-5). Owns **only** this note; `CHANGELOG.md` / `docs/Doc-Index.md` / `tools/github/issues.yaml` / `CLAUDE.md` are **FLAGGED** for the integrating parent (§11). |
| **Decides (proposes, for ratification)** | A **layered UX stack** for the guarantee lattice + Meta + cert-mode presentation + **honesty-poison isolation** so authors are not buried in ceremony while VR-5/G2 hold and a weak grade cannot silently poison whole applications or datasets: (1) **keep** RFC-0018 stage-1a modular bottom (`@` optional; unannotated = `Declared` demand/advertisement) as the *source* contract; (2) **deterministic structural inference** for body grades plus a **catalog of structural Exact / capped-at-Empirical ops** so most tags never need writing; (3) **basis-carrying strengthen syntax** (only path that may raise a grade without Swap/seal); (4) **isolation as a core slice** — airlock / firewall / quarantine / meet-boundary with **deterministic rules for where meet is allowed**, not folklore; (5) **airlock sugar** as surface over `Option`/`Result` + predicate (companion 02 pattern B; **no laundry**); (6) **generation≠consumption for tag + isolation EXPLAIN** (RFC-0034 §7 generalized); (7) **tooling presentation** that separates the three trust axes (DN-126 / companion 04). Rejects ambient nodule-wide grade *upgrade* defaults, silent auto-`Proven`/`Empirical`, quality-kill global floors, and greenwash remints. |
| **Feeds** | RFC-0018 stage 1b (grade polymorphism — sequenced, not replaced); companion `02-guarantee-airlocks` (airlock patterns → landable surface); RFC-0034 §7 generation≠consumption (tag-EXPLAIN + isolation packages); council [Agent D](../planning/gap-analysis-2026-07-16/AGENT-D-HONESTY-POISON-CONTAINMENT-2026-07-17.md) threat model; Agent A swap fallibility (A-PC rules); Agent C AX-stack; suggested build items §10. |
| **Depends on / grounds on** | **RFC-0001** (`Meta`, `GuaranteeStrength`, M-I1…M-I4, provenance); **RFC-0018** Enacted stage-1a (`crate::grade`, modular bottom, G-Weaken, G-Swap sole endorsement); **RFC-0034 / ADR-032** Enacted (`CertMode`, `gate_guarantee`, mode never-silent, gen≠consumption); **RFC-0005** (mandatory EXPLAIN of selection/mode); **ADR-011** (`BoundBasis` universal); **DN-126** Accepted (third trust axis — typing strictness); companion **02** airlocks + **04** three axes; **VR-5 / G2 / KC-3 / KISS / YAGNI**; house rule #1 (transparency lattice). |
| **Verified-against (slot + premises)** | Free slot: highest free note id is **DN-141** (`docs/notes/` has DN-140; no prior DN-141; `issues.yaml` has no competing `corpus:DN-141` at draft). Premises re-checked against landed code/corpus (mitigation #14): `grade.rs` modular bottom + meet (`crates/mycelium-l1/src/grade.rs`); `CertMode::gate_guarantee` (`crates/mycelium-core/src/cert_mode.rs`); `Meta::new` M-I1…M-I4 (`meta.rs`); transpiler blanket `Declared` (`mycelium-transpile/src/lib.rs`); language-reference §3 `T @ Strength`. |
| **Date** | 2026-07-16 (Draft open); 2026-07-17 (isolation elevated to core slice in body) |
| **Author** | design-reasoner (Design Agent B / council — tagging · Meta · lattice UX · isolation). Integrator weave with Agent D / companion 02. |
| **Task** | Ergonomic improvements + **deterministic machinery** for tagging and **containment of honesty poison** without ceremony burial, quality kill, or greenwashing. |

> **Posture (transparency rule / VR-5 / G2).** Draft design note. Tree-facts tagged `Empirical`
> with `file:line` or corpus anchors; design proposals `Declared`. No claim is upgraded past its
> basis. **No sycophancy:** §7 argues *against* the recommendation (ambient grade scopes,
> "infer stronger by default," and laundry seals all fail VR-5 on the merits). Status stays
> **Draft** until maintainer ratification (§9 DoD). **This note does not claim Accepted.**

---

## §0 The questions, in two lines

**How does Mycelium keep the four-point transparency lattice, full `Meta`, and tunable cert modes
as load-bearing invariants — while making everyday authoring *not* a tax of `@ Declared` /
`@ Exact` / basis comments on every binding — without ever silently overclaiming?**

**How does a downgraded honesty rating stay isolated so it cannot poison whole applications or
datasets — retaining Exact cores and accuracy — via deterministic walls and transparent EXPLAIN,
never greenwash?**

---

## §1 Pain inventory — what must be written or maintained by hand today

Grounded inventory. Each row is a real friction surface, not a strawman.

| # | Pain | Who pays | Basis |
|---|---|---|---|
| P1 | **Public API grade advertisement is manual.** Stage-1a treats missing `@ g` as modular bottom `Declared` (param demand + return advertisement). Precision requires writing `@ Exact` / `@ Proven` / … on signatures. | library authors | `Empirical` — `grade.rs:39-49`, `ret_grade`/`param_grade` |
| P2 | **Body grades are inferred, but strengthen is ceremony.** Meet-of-parts is automatic; raising a grade still requires a certified `Swap` (or an annotation that can only *weaken*). Authors who want an airlock write ad-hoc `Option` + comment, not a named pattern. | application authors | `Empirical` — RFC-0018 §3.2–3.3; companion 02 pattern B is conceptual |
| P3 | **Three trust axes look like one dial.** Guarantee grade · cert depth (`fast`/`certified`) · typing strictness (DN-126) are orthogonal; authors and docs still conflate them ("fast mode tags" vs "loose types" vs "`Declared`"). | everyone | `Declared` framing pain + `Exact` three-axis table (companion 04; DN-126 §2) |
| P4 | **`Meta` is rich; surface is thin.** Runtime `Meta` carries provenance, bound+basis, sparsity, physical, reconstruction, `policy_used`, `cert_mode`, `wrapping_opt`. Surface language exposes mainly `T @ g`. Bound basis pairing (M-I1…M-I4) is enforced in Rust `Meta::new` but has no ergonomic *author* path for "I claim Empirical with these trials." | advanced / certified path | `Empirical` — `meta.rs:84-110`; RFC-0001 §4.3 |
| P5 | **`Proven`/`Empirical` are expensive to *earn* and easy to *type*.** VR-5 forbids typing them without basis; today the *refuse* path is checker-grade demand, not guided basis attachment. Temptation: annotate `@ Proven` and hope. | certified authors | `Declared` UX gap; rule itself `Exact` (VR-5 / RFC-0018 G-Weaken) |
| P6 | **Cert mode gates tags, but the story is easy to misread.** `CertMode::Fast` floors intended `Empirical`/`Proven` → `Declared` while leaving structural `Exact` alone. Authors may think "fast = no tags" or "fast = all Declared." | everyone | `Empirical` — `cert_mode.rs:61-84` |
| P7 | **Transpiler / API-index / emit discipline is hand-prose `Declared`.** Every emit path documents `Declared` in module docs; co-emission of `Strength`/`GuaranteeStrength` is special-cased; lattice variant names are reserved words. High ceremony for tool authors, risk of tag drift. | toolchain | `Empirical` — `mycelium-transpile/src/{lib,emit,map,remap}.rs`; DN-140 reserved-word facet |
| P8 | **Meet contamination has no first-class isolation surface.** One `Declared` leaf meets a pipeline to `Declared` (correct, intentional). Quarantine, firewall, and meet-boundary rules are *patterns* (companion 02; Agent D), not stdlib/checker tables. | library + app | `Exact` meet rule; `Declared` isolation API |
| P9 | **EXPLAIN is mandatory for selection/mode; grade provenance and isolation decisions are second-class in DX.** Generation≠consumption exists for the inspectability signal (RFC-0034 §7); "why is this value `Declared`?" and "why did this boundary refuse?" are not first-class consumption tiers. | DX / LSP | `Declared` gap vs `Exact` mode-EXPLAIN mandate |
| P10 | **Documentation dual-write.** Spec matrices, tutorial `@ Declared` examples, companion airlocks, guarantee comments in Rust — same lattice restated with different ceremony levels; easy to overclaim in one surface. | maintainers | `Empirical` corpus density |
| P11 | **Honesty poison at application/dataset scale.** A weak grade or failed swap check can cascade (meet) or be laundered; Exact cores die or lie. Containment objective is council-first-class (Agent D); lattice UX must supply the **remint and wall** surface. | app + data authors | companion 02; Agent D T1–T7 |

**What is *not* pain (do not "fix"):**

- The lattice itself (`Exact ⊐ Proven ⊐ Empirical ⊐ Declared`) — load-bearing (RFC-0001, house rule #1).
- Meet-weakest-wins — intentional cleanroom discipline (companion 02).
- Never-silent mode tag on every `Meta` — already cheap and correct (RFC-0034 §3.1).
- Modular bottom for unannotated code — the reason existing code is not grade-hostile (RFC-0018 R18-Q5).

---

## §2 What can be inferred / defaulted **deterministically** without lying (VR-5)

### §2.1 Safe to default or infer (no author text required)

| Fact | Rule | Tag of *this* claim |
|---|---|---|
| Literal / written constant | grade `Exact` (G-Const) | `Empirical` — `grade.rs:18-19` |
| Unannotated param | demand `Declared`; bind body var at `Declared` | `Empirical` — `grade.rs:62-66` |
| Unannotated return | advertise `Declared` | `Empirical` — `grade.rs:56-59` |
| Pure structural composition | result = meet of parts | `Empirical` — G-Let/G-Con/G-Op |
| `wild` / FFI | floor `Declared` | `Empirical` — `grade.rs:27-28` |
| Annotation may only weaken | G-Weaken; upgrade without basis = type error | `Exact` — RFC-0018 §3.1–3.2 |
| `Swap` sole endorsement (plus seal with total predicate) | grade raise only with certificate or remint basis | `Exact` — RFC-0018 §3.3; companion 02 |
| `fast` floors non-structural strengths | `Proven`/`Empirical` → `Declared`; `Exact` passes | `Empirical` — `gate_guarantee` |
| Bound basis must match strength | M-I1…M-I4 in `Meta::new` | `Empirical` — `meta.rs:113-118` |
| Resonator-class decode | basis ≤ `Empirical` (FR-C2) | `Empirical` — `recon.rs` FR-C2 |
| Mode of production | from active `@certification` scope; rides `Meta`, not content hash | `Exact` — RFC-0034 / ADR-032 |
| Transpiler emission honesty | stay `Declared` until differential-witnessed | `Exact` — house VR-5 + transpile `lib.rs` |

### §2.2 Must **not** be inferred (would lie)

| Temptation | Why forbidden |
|---|---|
| Infer `@ Proven` from "looks pure" | No theorem + checked side-conditions (VR-5) |
| Infer `@ Empirical` from "has tests somewhere" | Trials must be *named method + ≥1 run* attached to the claim, not ambient CI |
| Ambient nodule default `@ Exact` for all returns | Silent upgrade of modular bottom; breaks S2 "grade is what the signature writes" |
| Collapse cert mode into grade | Orthogonal axes (DN-126 §2; companion 04) |
| Elide `Declared` flag because mode is `fast` | `Declared` is the transparent floor, not a tax to hide (RFC-0001; ADR-032) |
| Auto-upgrade after airlock without re-mint | Meet is not endorsement; only `Swap`/checked predicate mint is |
| Treat failed swap / `NotValidated` as Exact success | Greenwash (Agent A A-PC1…A-PC3; Agent D T3) |
| Global Declared floor "to be safe" | Quality kill (Agent D O3) |

### §2.3 The load-bearing split (restate for UX)

```
  WRITE (author ceremony)     INFER (machine, deterministic)     EARN (basis machinery)
  ───────────────────────     ──────────────────────────────     ─────────────────────
  signature @ g demands       body meet / literals / wild        ProvenThm side-conds
  airlock / seal intent       fast gate_guarantee floor          EmpiricalFit trials
  basis attachment            M-I1…M-I4 reconciliation           Swap certificate check
  cert-mode scope             mode tag on Meta                   cert emit+check
  quarantine / export wall    meet-boundary table                isolation EXPLAIN package
```

**Ceremony belongs only in WRITE and EARN.** INFER must be total, local (stage-1a), and never
stronger than justified. Isolation adds **deterministic walls** so meet does not become an
uncontrolled application-wide floor.

---

## §3 Isolation core slice — airlock · firewall · quarantine · meet-boundary

This section is **normative design proposal** for ratification of isolation UX. Companion 02 is
the narrative source; Agent D is the threat model and ranked D-core package. DN-141 owns the
**lattice/tag surface** that authors type and that EXPLAIN presents.

### §3.1 Alignment with companion 02 (reject laundry)

Companion 02 holds the counter-argument: documenting airlocks must not teach **Declared laundry**.
Mitigations that this note **requires** of any landable surface:

1. Remint only via **total Exact-decidable predicate**, **Swap certificate**, or **basis-carrying
   strengthen** (M-I1…M-I4).
2. Seal success is always **EXPLAIN-able** (basis ref present).
3. No `as Exact` token; lint against cast-upgrade.
4. Prefer **recertify-with-evidence** over **assert-stronger**.
5. CI may flag seal density in certified phyla (process, not silent upgrade).

If mitigations slip, **revoke airlock sugar**, keep structural catalog + tag-EXPLAIN.

### §3.2 Four isolation shapes (first-class concepts)

| Shape | Role | Companion 02 | Agent D rank |
|-------|------|--------------|--------------|
| **Airlock (seal)** | Weak in → check → remint stronger or never-silent fail | Pattern B | D2 |
| **Firewall** | Mode/cert-depth boundary; certified refuses unchecked meet | Pattern A | D3 |
| **Quarantine** | Module / phylum / spore / dataset sea-wall | Pattern D | D1/D5 |
| **Meet-boundary** | Table of where lattice meet is allowed vs refused | implied cleanroom | D1 |

### §3.3 Deterministic rules — where meet is allowed

Meet remains the default **inside** a cleanroom cell. Isolation specifies **refuse** sites so
poison cannot cross without a decision.

| Context | Meet allowed? | On refuse / dynamic path |
|---------|---------------|---------------------------|
| Local pure composition (same airlock cell) | Yes (weakest-wins) | N/A |
| Across **seal** boundary | No — only remint or explicit weak export | type/check error |
| Into **certified** demand from weaker / unchecked basis | No unless airlock or Swap cert | refuse or `Option` |
| Into **export** / Exact-only **dataset partition** | No for weaker leaves | partition / export reject |
| Author writes **explicit** weak export / `meet_allow` | Yes; result is weakest | EXPLAIN records allow |
| Swap site with partial regime / failed check | N/A — fallibility first (Agent A A3) | no Exact presentation |

**Static** refuse is preferred (checker table). **Dynamic** isolation (runtime seal, partition
check, cert `NotValidated` fallback) **must** emit the isolation EXPLAIN package (§3.5).

### §3.4 Quality retention (do not kill Exact cores)

- Keep critical Exact paths **separate** from Empirical/Declared leaves (companion 02 ML scenario).
- Partition datasets: Exact structural tables beside Empirical ML tables — not one silent flat meet.
- Seal when basis exists; do **not** force a global Declared floor.
- `fast` display floors non-structural strengths; structural Exact still passes (`gate_guarantee`).

### §3.5 Isolation EXPLAIN package (minimum)

When isolation is dynamic, generate:

| Field | Meaning |
|-------|---------|
| `boundary_kind` | `airlock` · `firewall` · `quarantine` · `meet_refuse` · `swap_check` |
| `input_grade` / `demand_grade` | lattice points |
| `decision` | `pass_remint` · `pass_weak` · `refuse` · `fallback` |
| `basis_ref` | predicate id, cert hash, trial method, or empty on refuse |
| `policy_used` / `cert_mode` | Meta fields |
| `never_silent` | always true for refuse/fallback |

Consumption tiers match §5.3 (lean / normal / audit). Shared schema with Agent D §5 and Agent E
EXPLAIN panel backlog.

### §3.6 Conceptual seal surface (not frozen grammar)

```text
// conceptual — Declared design, not frozen grammar
// desugars to match pred(x) { true => some(remint x); false => none }
seal[g'](x @ g, pred) -> Option[T @ g']
```

Remint allowed only under §3.1 rules. Stdlib phylum first (`std.airlock.seal`) preferred over
new keyword (OQ-1).

---

## §4 Options (real alternatives, not strawmen)

### Alt 0 — Status quo + documentation only

Keep stage-1a + RFC-0034 as-is; invest only in companion/tutorial/LSP copy.

- **Pros:** zero surface risk; KC-3 pure.
- **Cons:** P2/P5/P8/P9/P11 stay; airlocks remain folklore; three-axis confusion (P3) only partially fixed by companion 04.
- **Verdict:** necessary baseline, insufficient alone.

### Alt A — Structural-default inference catalog (extend INFER, no new WRITE)

Publish and enforce a **closed catalog** of ops whose *intrinsic* grade is structural `Exact`
(identity, bijective bit ops, pure constructors of Exact parts, content-hash, length) vs
**capped** (`resonator ≤ Empirical`, lossy swap ≤ certificate grade, `wild = Declared`).
Tooling auto-fills EXPLAIN "why this grade" from the catalog. Authors still write `@ g` only on
API boundaries they care about (status quo WRITE).

- **Pros:** pure INFER; VR-5-safe; reuses `gate_guarantee` + grade pass; reduces P1 ceremony *de facto*.
- **Cons:** does not reduce signature ceremony for libraries that *do* want `@ Exact` APIs; catalog maintenance.
- **KC-3:** no kernel growth — table + lints.

### Alt B — "Write only to strengthen" signature sugar (invert modular bottom *display*)

Keep modular bottom semantics, but DX presents unannotated as "inferred floor" and offers
explicit strengthen-only sugar so the common mental model is "tags appear when you claim more,"
not "tags appear when you confess less."

- **Pros:** matches VR-5 psychology (Declared is default truth).
- **Cons:** pure sugar over Alt 0; risk of teaching that missing tag means Exact (must not); needs careful LSP wording.
- **KC-3:** sugar only if desugar ≡ current modular bottom.

### Alt C — Ambient nodule-level guarantee default (like `@certification` / ambient paradigm)

e.g. `// @guarantee: Exact` on nodule header so unannotated returns demand Exact.

- **Pros:** low per-fn ceremony for homogeneous certified nodules.
- **Cons:** **silent upgrade of modular bottom** for every unannotated fn — antithetical to RFC-0018 R18-Q5 and S2. Cross-nodule imports become footguns. Meets contamination becomes ambient laundry.
- **Verdict:** **reject** as a *default upgrade*. A *lint profile* ("this nodule's *public* API must write `@ g`") is fine; ambient *semantic* upgrade is not.

### Alt D — Basis-carrying strengthen syntax (WRITE + EARN coupled)

Surface forms that **cannot parse/check without a basis payload**, e.g. conceptual:

```text
// conceptual — Declared design, not frozen grammar
fn f(x: T) -> U @ Empirical(method: "proptest-corpus-v3", n: 10_000) = …
fn g(x: T) -> U @ Proven(thm: "bound.dense.l2", side: checked) = …
// bare @ Proven without basis ⇒ hard refuse (never silent)
```

Desugars to: grade demand + `Bound`/`BoundBasis` construction that `Meta::new` accepts (M-I2/M-I3).
`fast` still floors via `gate_guarantee`.

- **Pros:** closes P5; makes VR-5 *syntactic*; pairs with M-I1…M-I4.
- **Cons:** grammar work; theorem/trial registries must exist; YAGNI until certified path is dogfooded.
- **KC-3:** surface + elaborator only if bound records already exist (they do in core).

### Alt E — Airlock sugar + isolation table (companion 02 + Agent D → landable surface)

Land **seal/recertify** plus checker **meet-boundary / export quarantine** tables (§3).

- **Pros:** directly attacks P2/P8/P11; teaches cleanroom; zero new endorsement channel if remint rules hold.
- **Cons:** laundry risk if remint hinge slips; design cost for meet-boundary table.
- **KC-3:** sugar over `Option` + existing grade rules; tables in checker preferred to kernel nodes.

### Alt F — Tag + isolation EXPLAIN consumption tiers (generalize RFC-0034 §7)

Always **generate** grade provenance trace (meet tree + gate_guarantee + basis ids) and isolation
packages when boundaries fire. **Consume** via DX tiers: `lean` · `normal` · `audit`. Mode EXPLAIN
stays mandatory at every tier.

- **Pros:** closes P3/P9 without source ceremony; gen≠consumption already ratified for mode.
- **Cons:** LSP/runtime work; trace schema design.
- **KC-3:** tooling + query surface (RFC-0001 runtime query sketches `guarantee_of`/`meta_of`).

### Alt G — Full stage-1b grade polymorphism + cross-fn inference

Whole-program return-grade inference (RFC-0018 §4.7 FlowCaml-style).

- **Pros:** maximum ceremony reduction for internal code.
- **Cons:** whole-program; deferred by design in stage-1a; large soundness surface; not YAGNI for the present pain.
- **Verdict:** **sequence later**, do not block UX stack.

---

## §5 Evaluation against objective and house rules

### §5.1 Objective function (explicit)

| Criterion | Weight | Notes |
|---|---|---|
| **C1 VR-5 integrity** — no silent upgrade; downgrade free | **hard gate** | any option that fails is out |
| **C2 G2 never-silent** — mode, grade, airlock failure, isolation refuse visible | **hard gate** | |
| **C2b No greenwash / no quality kill** — council N7–N8 | **hard gate** | |
| **C3 Ceremony reduction** — fewer required `@` on happy path | high | measured as "tags written per public API" + "tags written per internal let" |
| **C4 KC-3 / KISS / YAGNI** — no parallel lattice; reuse Meta/grade/CertMode | high | |
| **C5 Axis clarity** — grade ≠ cert depth ≠ typing strictness | high | DN-126 / companion 04 |
| **C6 Isolation completeness** — meet-boundary + seal + EXPLAIN | high | §3; Agent D |
| **C7 Incremental land** — each slice valuable alone | medium | wave-friendly |
| **C8 Dogfood / transpile honesty** — does not pressure tools to fake Proven | medium | P7 |

### §5.2 Scorecard (ordinal ranks; hard-gate failures marked FAIL)

| Alt | C1 | C2/C2b | C3 | C4 | C5 | C6 | C7 | C8 | Rank role |
|---|---|---|---|---|---|---|---|---|---|
| 0 docs-only | pass | pass | weak | best | partial | weak | best | pass | baseline |
| **A catalog** | pass | pass | med | best | helps | partial | best | pass | **core INFER** |
| B strengthen-display | pass* | pass | low-med | good | helps | n/a | good | pass | sugar on A |
| C ambient upgrade | **FAIL** | risk | high | poor | confuses | **FAIL** | med | risk | **reject** |
| **D basis syntax** | pass | pass | med (certified path) | good | helps | n/a | staged | pass | **core EARN** |
| **E airlock + isolation table** | pass† | pass | high (P8/P11) | best | helps | **best** | good | pass | **core isolation WRITE** |
| **F tag+isolation EXPLAIN** | pass | pass | high (P9) | good | **best** | high | good | pass | **core DX** |
| G stage-1b | pass | pass | highest long-term | heavy | ok | partial | poor now | pass | **later** |

\*B passes only if desugar ≡ modular bottom and LSP never implies Exact-by-omission.
†E passes only if remint requires total Exact predicate or real Swap cert — not `as Exact`.

### §5.3 Grounding map

| Claim in recommendation | Basis |
|---|---|
| Modular bottom stays | RFC-0018 R18-Q5 / `grade.rs` module note — `Exact` (corpus) |
| fast floors Empirical/Proven | `gate_guarantee` — `Empirical` (code) |
| Three axes orthogonal | DN-126 §2 Accepted; companion 04 — `Exact` (decision) / mechanisms `Declared` until built |
| Airlock pattern + laundry reject | companion 02 — pattern `Declared`; discipline enforceable today |
| Isolation catalog | Agent D Draft; companion 02 A–D |
| Gen≠consumption | RFC-0034 §7 Enacted — `Exact` (decision) |
| Reject ambient grade upgrade | VR-5 + S2 + R18-Q5 — argument `Declared` (this note), premises `Exact` |
| Swap fallibility binding | Agent A A-PC1…A-PC6 — `Declared` design |

---

## §6 Ranked recommendation (Draft)

### Rank 1 — **Layered stack: A + F + E(isolation) now-sequence; D when certified dogfood needs it; B as optional sugar; G later; C never**

**Ship as one design, land as independently reviewable slices:**

1. **Slice A — Structural grade catalog + "why this grade" generator**
   Codify intrinsic grades for prims/std ops; feed grade EXPLAIN and API-index rows; refuse catalog
   overclaim in CI. *Addresses P1 de facto, P7, P10.*

2. **Slice F — Tag + isolation EXPLAIN consumption tiers**
   Generalize RFC-0034 §7 from mode signal to **grade provenance + isolation package + cert_mode +
   basis id**. Lean badges in `fast` DX; audit trail on demand. *Addresses P3, P6, P9, P11 opacity.*

3. **Slice E — Isolation core: meet-boundary table + quarantine exports + `std.airlock` seal**
   Land §3 as checker tables + tested phylum: total predicate → `Option[T @ g']` with remint rules
   documented; lint against bare cast-to-stronger; certified firewall refuse weak-into-certified
   without seal. *Addresses P2, P8, P11.* Sequencing inside E: **tables and EXPLAIN schema before
   sugar that could look like laundry.**

4. **Slice D — Basis-carrying `@ Empirical(…)` / `@ Proven(…)`**
   Only when a real consumer needs certified APIs with attached basis (not before). Bare `@ Proven`
   becomes a hard error once D lands. *Addresses P4, P5.*

5. **Optional B** — presentation sugar only after F exists (so omission never reads as Exact).

6. **Explicit non-goals:** Alt C ambient upgrade; auto-Empirical-from-CI; collapsing axes;
   stage-1b as a prerequisite for ergonomics; global Declared floor; laundry seals.

### §6.1 Why this wins the objective table

- **C1/C2/C2b hard gates:** every strengthen path is meet-safe inference, mode floor, Swap cert,
  or predicate-remint with never-silent failure — never ambient; isolation EXPLAIN is mandatory on
  dynamic boundaries; Exact cores stay reachable via separate paths.
- **C3:** everyday code keeps writing **zero** tags (modular bottom); libraries write tags only at
  **exported** strengthens; airlocks replace essay comments.
- **C4:** reuses `grade`, `Meta`, `CertMode`, `BoundBasis`, RFC-0005 EXPLAIN — no second lattice.
- **C5:** F's presentation is *required* to show three axes as three badges, not one slider.
- **C6:** meet-boundary + seal + quarantine + firewall are first-class, not footnotes.
- **C7:** A→F→E→D is independently shippable; each closes named pains.
- **C8:** catalog + transpile stay `Declared` until witnessed; D does not let tools mint Proven.

### §6.2 Author mental model (teach this)

```
1. Omit @ g          → you advertise Declared (honest floor). No shame; VR-5 default.
2. Write @ Exact     → only if the body/catalog proves Exact (checker enforces).
3. Need stronger from weaker data → airlock (seal) or Swap with cert — never cast.
4. Weak data near Exact cores → quarantine / separate path / meet-boundary refuse — never hope.
5. @ Empirical/@ Proven → bring basis (slice D) or don't write them.
6. @certification    → how much machinery ran; does not invent a stronger basis.
7. loose/strict type → whether the checker refuses; independent of grade (DN-126).
8. Failed check / partial swap → Option/Result/fallback; never looks Exact (Agent A).
```

### §6.3 Lattice + isolation presentation (DX contract — slice F)

| Tier | What user sees | Never hides |
|---|---|---|
| **lean** (`fast` default) | single badge: grade + ⚠ if Declared; mode icon only if non-default; boundary-fired mark if isolation ran | mode if not `fast`; any Declared; refuse/fallback |
| **normal** | one-line: `Declared ← meet(Empirical resonator, Exact width)` or `refuse: meet_boundary export` | basis absence; isolation decision |
| **audit** | provenance DAG + isolation package + `policy_used` + bound + cert_mode + wrapping_opt | nothing in Meta query surface |

---

## §7 Adversarial stress-test (argue against the recommendation)

### Attack 1 — "A+F+E is still three projects; Alt 0 is enough"

**Concede partially:** if engineering bandwidth is zero, companion 02+04 + tutorial already reduce
P3. **Counter:** P5/P8/P11 are *mechanism* gaps; docs cannot refuse bare `@ Proven`, standardize
airlocks, or encode meet-boundary tables. Rank-1 without basis syntax D still beats Alt 0 on
C3/C5/C6.

### Attack 2 — "Airlock sugar becomes Declared laundry"

**Real risk** (companion 02 counter-argument). **Mitigations baked into Rank 1:** (i) remint only
via total Exact-decidable predicate or Swap cert; (ii) seal success still EXPLAIN-able; (iii) CI
flag on seal density in `certified` phyla; (iv) no `as Exact` token; (v) tables land before sugar.
If mitigations slip, **revoke E sugar**, keep A+F + meet-boundary refuse only.

### Attack 3 — "Basis syntax (D) is premature ceremony"

**Agree on sequencing:** D is slice 4, not slice 1. Shipping D before catalogs/EXPLAIN teaches
authors to decorate everything. **If** certified dogfood never needs attached basis in surface,
**drop D** and keep bound construction in Rust/kernel only — YAGNI wins.

### Attack 4 — "Ambient nodule Exact (C) is what users will ask for"

**Likely request; still reject.** It fails C1 and C2b: unannotated code would advertise Exact
without a written claim, and ambient "clean" is laundry. Offer instead: lint
`public-api-must-annotate-grade` as an **opt-in profile** — honest tax, not silent upgrade.

### Attack 5 — "Stage-1b (G) would erase ceremony better"

**True for internal code long-term.** False as *near-term* ergonomics: whole-program inference
delays airlocks and EXPLAIN, and does not solve basis attachment, axis confusion, or dataset
quarantine. Sequence G after A/F/E; do not block.

### Attack 6 — "fast already removed tag cost; this DN is redundant with ADR-032"

**False.** ADR-032/RFC-0034 solved *certification machinery cost* and mode disclosure. They
explicitly kept the lattice and modular tagging. The residual pain is **author/DX ceremony,
axis clarity, and poison isolation**, not cert CPU.

### Attack 7 — "Meet-boundary tables will kill composition"

**Risk of over-quarantine.** Mitigate: meet remains free **inside** cells; refuse is at **export,
seal, certified demand, Exact-only partitions**. Explicit `meet_allow` / weak export stays for
honest Declared pipelines. Quality retention (§3.4) is a DoD criterion.

### Stress-test verdict

Rank-1 survives with **sequencing discipline** (A/F before E sugar; E tables before E sugar;
D last; C rejected; G deferred). Fragile joints: **E remint rule** and **meet-boundary default
strictness** — treat both as hard DoD gates, not docs footnotes.

---

## §8 Open questions for the maintainer

| ID | Question | Default if silent |
|---|---|---|
| **OQ-1** | Is **airlock** a **stdlib phylum** (`std.airlock.seal`) or a **surface keyword**? | stdlib phylum first (KC-3, YAGNI on grammar) |
| **OQ-2** | Should bare `@ Proven` / `@ Empirical` become **hard errors** once basis syntax (D) exists, or remain weaken-only annotations with separate basis fields? | hard error for bare strengthen-to-Proven/Empirical (VR-5 syntactic) |
| **OQ-3** | Lean DX: is a visible `Declared ⚠` on **every** unannotated value too noisy under `fast`? | show on **bindings that cross a public API or meet with a stronger demand**; elide pure-local Declared in lean (signal still generated) |
| **OQ-4** | Opt-in lint profile `public-api-must-annotate-grade` — want it for `certified` phyla by default? | yes for `certified` public exports; no for `fast` |
| **OQ-5** | Does tag/isolation EXPLAIN generation stay **always-on** even in `fast` (like middle-tier signal), or mode-gated? | always generate cheap meet-trace + boundary events; floor *consumption* lean in `fast` (mirror RFC-0034 §7) |
| **OQ-6** | Transpiler: should emit grow **auto** `@ Declared` on every fn return for VR-5 visibility, or keep omission (= modular bottom)? | **omit** (status quo); document equivalence; do not spray `@ Declared` |
| **OQ-7** | Name bikeshed: `seal` / `airlock` / `recertify` / `remint`? | `seal` (short; companion 02); alias `recertify` in docs |
| **OQ-8** | Meet-boundary default: free meet inside nodule + refuse at `pub`, or stricter cell model first? | free inside nodule; refuse at `pub` / certified demand / Exact partitions (Agent D OQ-1 default) |
| **OQ-9** | Dataset/spore grade partitions: spore contract fields now, or library convention first? | library convention first; spore fields when packaging dogfood needs them |

---

## §9 Definition of Done (maintainer ratification → Accepted)

This note moves **Draft → Accepted** only when the maintainer confirms:

1. **Rank-1 stack** (A+F+E isolation, D sequenced, C rejected, G deferred) is the direction — or
   records a superseding rank with the same C1/C2/C2b hard gates.
2. **OQ-1…OQ-9** are answered or explicitly deferred with owners (not silently assumed).
3. **E remint soundness hinge** is accepted: no grade raise without total Exact predicate or Swap
   certificate (companion 02 + RFC-0018 G-Swap).
4. **Meet-boundary determinism** is accepted: table-driven allow/refuse; dynamic paths carry
   isolation EXPLAIN packages.
5. **Axis orthogonality** restated: this DN does not merge grade · cert mode · typing strictness.
6. **No greenwash / no quality-kill** restated as DoD: failed checks never present as Exact;
   Exact cores remain reachable.
7. **No code status change** is implied: Accepted ≠ Enacted; build slices remain `Declared` until
   landed and witnessed.
8. Shared-file rows (§11) applied by integrating parent only.

**Enacted** (future, not this note): each slice A/F/E/D has a build issue, tests, and honest tags.

---

## §10 Suggested work items (wave re-rank)

| Priority | Suggested id (mint free) | Title | Slice | Depends |
|---|---|---|---|---|
| P0 | *(FLAG mint)* | Structural grade catalog for prims + std ops + CI overclaim guard | A | RFC-0018 stage-1a landed |
| P0 | *(FLAG mint)* | Grade + isolation EXPLAIN schema + lean/normal/audit consumption | F | RFC-0034 §7; RFC-0001 query; Agent D §5 |
| P0 | *(FLAG mint)* | Meet-boundary / export quarantine table + conformance fixtures | E-table | RFC-0018 meet; Agent D D1 |
| P1 | *(FLAG mint)* | `std.airlock` seal/recertify + tests + lint vs cast-upgrade | E-seal | companion 02; grade pass; E-table |
| P1 | *(FLAG mint)* | Certified consumer firewall (mode × grade refuse) | E-fw | RFC-0034; E-table |
| P2 | *(FLAG mint)* | Basis-carrying `@ Empirical`/`@ Proven` surface + bare-tag refuse | D | A, bound registries, certified dogfood need |
| P2 | *(FLAG mint)* | LSP/hover three-axis badges (grade · mode · typing) + boundary marks | F | DN-126; companion 04; Agent E |
| P3 | *(FLAG mint)* | Opt-in `public-api-must-annotate-grade` lint profile | lint | OQ-4 |
| P3 | *(FLAG mint)* | Spore/dataset grade partition fields | quarantine | Agent D D5; spore contract |
| P3 | *(existing track)* | RFC-0018 stage 1b grade polymorphism | G | after A/F/E settle UX |
| — | docs-only | Tutorial + language-reference: three axes + isolation cleanroom | 0 | companion 02/04; Agent A fallibility pass |

**Out of scope / do not mint from this DN:** ambient `@guarantee` semantic upgrade (Alt C);
collapsing `fast` into loose typing; forcing `@ Declared` spray in transpile emit; auto-insert
swap or seal.

---

## §11 FLAGs for integrating parent (read-only here)

| Artifact | Row to apply (dated 2026-07-17 for isolation weave) |
|---|---|
| `docs/Doc-Index.md` | DN-141 — Tagging · Meta · transparency-lattice UX · isolation (Draft) |
| `CHANGELOG.md` | council integrate: poison containment + isolation core in DN-141 body |
| `tools/github/issues.yaml` | mint P0–P2 items in §10 after free-id verify; `doc_refs: corpus:DN-141` |
| `CLAUDE.md` | no house-rule change; optional Map pointer to DN-141 under transparency/UX |
| companion | deep-link from `02-guarantee-airlocks` / `04-three-trust-axes` once Accepted |
| council peers | Agent A A-PC rules; Agent D D-core; Agent C AX isolation ranks; Agent E EXPLAIN panel |

---

## §12 Changelog (this note)

| Date | Change |
|---|---|
| 2026-07-16 | **Draft** — Design Agent B council deliverable: pain inventory, safe-inference set, alts 0/A–G, Rank-1 recommendation, adversarial stress-test, OQs, DoD, work-item flags. Slot DN-141 verified free. |
| 2026-07-17 | **Draft (integrated body)** — Honesty-poison isolation elevated from pattern footnote to **§3 core slice** (airlock / firewall / quarantine / meet-boundary, deterministic meet rules, EXPLAIN package, quality retention, laundry reject). Cross-linked Agent A/D/E and companion 02. Status remains **Draft** — not Accepted. |

---

## Meta

- **Honesty of this note:** design recommendations `Declared`; code/corpus citations `Empirical`
  or `Exact` as tagged inline. Nothing `Proven`.
- **Supersedes:** nothing. **Amends:** nothing normatively until Accepted.
- **Related:** DN-29 (Superseded deliberation) · ADR-032 / RFC-0034 (Enacted modes) · RFC-0018
  (Enacted stage-1a) · DN-126 (Accepted typing axis) · companion 02/04 · RFC-0001/0005 · council
  Agents A/C/D/E under `docs/planning/gap-analysis-2026-07-16/`.
