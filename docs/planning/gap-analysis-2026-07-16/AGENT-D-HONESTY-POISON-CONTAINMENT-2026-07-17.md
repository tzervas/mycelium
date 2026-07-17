# Design Agent D — Honesty-poison containment (2026-07-17)

| Field | Value |
|-------|--------|
| **Status** | **Draft** (council research — **not** Accepted; does not ratify) |
| **Agent** | D — Honesty-poison containment (meet-contamination isolation) |
| **Model** | grok-4.5 (high effort research) |
| **Honesty** | Design recommendations **`Declared`**; code/corpus citations **`Empirical`** or **`Exact`** as tagged. Nothing `Proven`. |
| **Scope** | Mycelium only; no product code; no merge PRs; no status moves |
| **Council** | [DESIGN-COUNCIL-SWAPS-TAGS-2026-07-17.md](./DESIGN-COUNCIL-SWAPS-TAGS-2026-07-17.md) |
| **Extends** | [DN-141](../../notes/DN-141-Tagging-Meta-Honesty-Lattice-UX.md) slice E · [AGENT-C](./AGENT-C-AX-STACK-SYNTHESIS-2026-07-17.md) **X4/X2** · companion [02-airlocks](../../companion/02-guarantee-airlocks.md) · [04-axes](../../companion/04-three-trust-axes.md) |
| **Siblings** | [Agent A](./AGENT-A-SWAPS-ERGONOMICS-2026-07-17.md) (swap fallibility) · DN-141 (lattice UX) · Agent C (AX ranks) |

> **Posture (VR-5 / G2 / house rule #3 / rule #4).** Recommends a path so a **downgraded**
> honesty/guarantee rating cannot poison whole applications, pipelines, or datasets — while
> retaining accuracy, preferring **deterministic** machinery, and avoiding ceremony or greenwash.
> Does **not** move RFC/DN/ADR status. Argues against the recommendation (§8) on the merits.

---

## 0. Premises verified (mitigations #1 + #14)

| Premise | Basis | Tag |
|---------|-------|-----|
| Lattice `Exact ⊐ Proven ⊐ Empirical ⊐ Declared`; meet = weakest-wins | RFC-0001 §3.4/§4.7; `GuaranteeStrength::meet` | `Exact` (normative) / `Empirical` (code) |
| Composition: `propagate = meet(inputs…, intrinsic)` | `guarantee.rs::propagate`; RFC-0018 G-Let/G-Con/G-Op | `Empirical` |
| Annotation only weakens; Swap sole endorsement | RFC-0018 G-Weaken / G-Swap; `grade.rs` | `Exact` (Enacted stage-1a) |
| Modular bottom: unannotated = `Declared` | RFC-0018 R18-Q5; `ret_grade`/`param_grade` | `Empirical` |
| `fast` floors Proven/Empirical → Declared; structural Exact passes | RFC-0034 §7; `CertMode::gate_guarantee` | `Empirical` |
| Gen ≠ consumption for inspectability | RFC-0034 §7 Enacted | `Exact` (decision) |
| Three orthogonal trust axes | companion 04; DN-126; ADR-032 | framing `Exact` |
| Airlock patterns A–D are guide, not landable API | companion 02 | pattern `Declared` |
| DN-141 Rank-1 = A+F+E (+D later); rejects ambient grade upgrade | DN-141 §5 | Draft `Declared` |
| AX-stack X4 = `std.airlock`; X2 = grade catalog + tag-EXPLAIN | AGENT-C §2 | Draft `Declared` |
| Free DN slot for future promotion | highest note = DN-141 | `Empirical` — **this file is a planning note, not DN-142** |

**What D owns that A/B/C do not:** *how contamination spreads through a whole program /
pipeline / dataset*, and the **firewall architecture** (deterministic boundaries + minimum
EXPLAIN package) that keeps Exact cores shippable without killing DX or laundering tags.

---

## 1. Pain — how meet-contamination poisons "the whole plant"

### 1.1 Mechanism (math, not metaphor)

```
result_g = meet(g₁, g₂, …, gₙ, g_op)     // GuaranteeStrength::propagate
```

Meet is the **greatest lower bound** on a total chain: **one** `Declared` (or one `Empirical`
resonator leaf) forces every ancestor composition that includes it to that floor or lower.
Companion 02: *"a Declared leaf is unsterilized dust in a cleanroom."* That pessimism is
**intentional** (Biba integrity orientation — RFC-0018; VR-5). The pain is not the meet rule;
it is the **absence of named containment boundaries**, so dust fills the building instead of
one room.

### 1.2 Threat model

| Threat | Mechanism | Failure if uncontained |
|--------|-----------|------------------------|
| **T1 Meet cascade** | Lattice meet across composition | One dust leaf strips a whole cleanroom pipeline |
| **T2 Silent greenwash** | Cast / hope / bare `@ Exact` on weak data | Downstream treats weak data as Exact — accuracy lie |
| **T3 Failed-check as success** | Swap `Err`, cert incomplete, airlock `none` presented as value | Partial conversion looks total |
| **T4 Dataset / export poison** | Spore/publish/table mixes grades without boundary | Consumers import a flat bag with no meet story |
| **T5 Mode conflation** | Author treats `fast` as "no tags" or "all Declared forever" | Over-fear (quality kill) or under-disclosure |
| **T6 Ambient laundry** | Nodule-wide default upgrade | Whole module silently claims Exact (DN-141 rejects Alt C) |
| **T7 Toolchain pressure** | Transpile invents stronger tags to green a metric | Portfolio honesty collapse |
| **T8 VSA resonator dust** | FR-C2 caps decode ≤ Empirical; composite meets that leaf | Structural Datum falsely Exact |
| **T9 Cross-fn modular bottom** | G-App uses declared return grade; unannotated callee = Declared | Precision lost at every call unless signatures write `@ g` |

**Not a threat:** honest meet to Declared when data *is* only Declared; structural Exact on
literals/bijection; explicit `Option`/`Result` failure. Pain is **uncontrolled propagation**
and **false strength**, not the lattice itself.

### 1.3 Contamination surfaces (inventory)

| Surface | How poison spreads | Basis |
|---------|-------------------|-------|
| **P-D1 Pipeline meet** | One weak intermediate in `let`/fold/`Op` degrades whole expression | `Exact` meet; `Empirical` G-Let/G-Op |
| **P-D2 Dataset / batch fold** | `meet_all` over corpus: one Declared row zeros batch Exact claims | `meet_all` API `Empirical`; partition rules `Declared` |
| **P-D3 Cross-fn modular bottom** | Stage-1a does not infer return grades across calls | `Empirical` — `grade.rs` G-App |
| **P-D4 Cert-mode floor bleed** | `fast` floors Empirical/Proven → Declared into certified consumer | `Empirical` — `gate_guarantee` |
| **P-D5 VSA / resonator dust** | Resonator ≤ Empirical by construction (FR-C2) | `Empirical` recon; companion 02 scenario |
| **P-D6 Transpile Declared flood** | Emit stays Declared until differential; mixed with native Exact cores | VR-5 policy `Exact`; transpile docs `Empirical` |
| **P-D7 Spore / deploy graph** | Mixed-grade graph under one spore identity over-advertises | ADR-013 + companion 02 D — envelope `Declared` |
| **P-D8 Airlock laundry (future)** | Seal remints without total Exact predicate or Swap cert | companion 02 counter-arg; DN-141 Attack 2 |

---

## 2. Mechanisms catalog — isolation / firewall / airlock / dual-path

Each pattern: **what it isolates**, **deterministic vs EXPLAIN-only**, **landed vs design**.

### M1 — Dual-path (no meet across trust tiers)

Keep safety-critical Exact cores on a **separate data path** that never receives the weak leaf
as an *input* to a composition. Weak data may drive **control** (Design A: scrutinee grade does
not taint result — RFC-0018 G-Match/A) or a side-channel report, but does not enter Exact
constructor field lists.

| | |
|--|--|
| Isolates | Critical composite fields / certified kernels |
| Determinism | Checker enforces `@ Exact` field/param demands (G-App / G-Con) |
| Landed | Stage-1a demand checking **yes**; dual-path as *named pattern* **no** |
| Risk | Authors still accidentally meet via one shared `let` |

### M2 — Predicate seal / remint airlock (companion 02 B; DN-141 E; AX-X4)

Accept weak payload; run a **total Exact-decidable predicate** (or real Swap cert); on pass,
mint a **new** value at the earned grade; on fail, `Option`/`Result` never-silent.

```text
// conceptual — pattern, not frozen API
fn seal_width(x: Binary{64} @ Declared) -> Option[Binary{64} @ Exact] =
  match lt(x, 0b1_0000_0000) {
    0b1 => some(remint_exact(x)),  // basis = predicate discharge
    _   => none
  }
```

**Remint allowed only when:** (1) `pred` total + Exact-decidable, **or** (2) Swap certificate
valid under active cert mode, **or** (3) basis-carrying strengthen (DN-141 D) supplies
`BoundBasis` that `Meta::new` accepts (M-I1…M-I4).

**Forbidden:** `as Exact`, comment-only seal, remint on "looks pure."

| | |
|--|--|
| Isolates | Declared/Empirical → Exact/Proven handoff at a named gate |
| Determinism | Remint rule is a hard checker law |
| Landed | Discipline with ad-hoc `Option` **yes**; stdlib phylum **no** |
| Risk | Laundry if remint is cast without basis |

### M3 — Cert-mode firewall (companion 02 A; RFC-0034 / ADR-032)

Exploratory work under `@certification(fast)`; production consumers demand `certified` and
refuse silent cross-mode composition (mode tag never-silent; cross-mode is explicit
`Option`/error per RFC-0034 §3).

| | |
|--|--|
| Isolates | Unchecked cert machinery from checked consumers |
| Determinism | Mode resolution + mode tag on Meta; **not** a grade upgrade |
| Landed | `CertMode`, `gate_guarantee`, mode on Meta — **yes** |
| Risk | Authors conflate mode with grade (DN-141 P3) |

### M4 — Phylum / nodule sea-wall (pub API grade advertisement)

`pub` exports **must write** `@ g` under opt-in (or `certified`-default) lint; internal helpers
stay modular-bottom. Callers meet only the **advertised** boundary grade (G-App uses declared
return grade).

| | |
|--|--|
| Isolates | Implementation Declared dust from Exact library consumers |
| Determinism | Signature is the contract (S2); checker proves body ⊒ demand |
| Landed | G-Weaken on return **yes**; lint profile **no** (DN-141 OQ-4) |

### M5 — Meet-at-named-boundaries-only (firewall topology)

Inside a quarantine region, intermediate meets still happen for *local* honesty; the **region
export** is the only place a grade re-enters the ambient program — and only via M2 seal or M4
signature.

**Deterministic meet policy (recommended core):**

| Context | Meet allowed? | On refuse |
|---------|---------------|-----------|
| Same airlock cell / local pure composition | Yes (weakest-wins) | N/A |
| Across **seal** boundary | No — only sealed remint or explicit weak export | type/check error |
| Into **certified** demand from weaker basis | No unless airlock or Swap cert | refuse or Option |
| Into **dataset partition** marked Exact-only | No for weaker leaves | partition reject |
| Author writes **explicit** weak export | Yes, result is weakest; EXPLAIN records allow | — |

| | |
|--|--|
| Isolates | Pipeline-local contamination from plant-wide meet |
| Determinism | Prefer typed rules + export seal; region attribute alone is weaker |
| Landed | **No** (design) |
| Risk | Ceremony if ambient; must be **opt-in quarantine**, not default on every nodule |

### M6 — Spore grade envelope (deploy gate)

Spore packages content-addressed code (ADR-013) **plus** a **declared grade envelope** (min
advertised strength of public exports + cert mode of build). Install/run in a `certified`
colony refuses weaker envelopes without an explicit import airlock.

| | |
|--|--|
| Isolates | Deploy-time contamination of production colonies |
| Determinism | Envelope check is pure metadata compare |
| Landed | Spore identity **yes**; grade envelope **no** |
| Risk | Greenwash if self-declared without checker attestation |

### M7 — Dataset / provenance partition

Never `meet_all` over heterogeneous rows for a *claim*. Partition by grade (or basis id);
report **per-partition** strengths; only homogeneous Exact partitions support Exact aggregates.

| | |
|--|--|
| Isolates | One bad row from batch-level Exact claims |
| Determinism | Partition keys from Meta.grade / basis |
| Landed | `meet_all` API **yes**; partition helpers **no** |

### M8 — Swap + reconstruction manifest (lossy path honesty)

Dense/VSA lossy paths attach reconstruction policy so EXPLAIN shows indexed retrieval vs
compositional reconstruction (companion 02 C). The airlock is the **typed policy**, not a cast.
Pairs with Agent A regime → total / `Option` / `Result` (AX-X3).

### M9 — Organizational / CI quarantine

CI flags new `@ Declared` on certified paths; seal-density reports; transpile emissions stay
in `gen/` or draft phyla until differential upgrade. Necessary but **not sufficient** alone.

### M10 — Tag-EXPLAIN + isolation EXPLAIN package (DN-141 F; AX-X2)

Always **generate** meet-trace + gate_guarantee + basis ids + boundary decisions; **consume**
lean / normal / audit so operators see *why* a result is Declared without re-running.

**Isolation EXPLAIN package** (minimum when isolation is dynamic):

| Field | Meaning |
|-------|---------|
| `boundary_kind` | `airlock` · `firewall` · `quarantine` · `meet_refuse` · `swap_check` |
| `input_grade` / `demand_grade` | lattice points at the boundary |
| `decision` | `pass_remint` · `pass_weak` · `refuse` · `fallback` |
| `basis_ref` | predicate id, Swap cert hash, trial method, or empty if refuse |
| `policy_used` / `cert_mode` | as already on Meta |
| `never_silent` | always true for refuse/fallback |

**Rule:** if consumer signature demands `@ Exact` and value is weaker → **type/grade error**.
EXPLAIN is not a substitute for isolation at that boundary. EXPLAIN substitutes only when the
consumer **accepts** the weak grade knowingly.

| | |
|--|--|
| Isolates | Nothing mechanically — **makes poison visible** (necessary companion to M1–M7) |
| Determinism | Trace schema deterministic; consumption tunable (RFC-0034 §7) |
| Landed | Mode EXPLAIN **yes**; grade + isolation packages **no** |

### Interaction with swaps (Agent A binding)

| Swap outcome | Must not look like | Containment binding |
|--------------|--------------------|---------------------|
| `Ok` + cert Validated | — | grade from BoundBasis / regime (RFC-0002) |
| `Ok` + cert incomplete | Exact success | explicit fallback; EXPLAIN `swap_check`/`fallback` |
| `Err` / out-of-range / partial inverse | total target type | regime → `Option`/`Result` (Agent A) |
| Lossy Dense/VSA | Exact | reconstruction manifest (M8) |

**Rule D↔A:** failed or incomplete check **must not** type or present as Exact downstream.

---

## 3. Quality retention (do not kill Exact cores)

Containment fails if the only safe program is "everything Declared."

| Practice | Effect |
|----------|--------|
| **Separate Exact path** | Critical branch never meets Empirical/Declared leaves |
| **Partition datasets** | Exact structural tables vs Empirical ML tables side-by-side |
| **Seal, don't strip** | Airlock remints when basis exists; does not force global floor |
| **fast display ≠ loss of Exact structure** | `gate_guarantee` floors non-structural strengths; structural Exact still passes |
| **Modular bottom stays** | Everyday code writes zero tags; ceremony only at exports/seals/spores |

**Anti-pattern:** "to avoid poison, ban Exact" or "to avoid ceremony, omit Declared flags."

### Author mental model (ceremony budget)

```
1. Write freely under modular bottom / fast — Declared is honest, not shameful.
2. Exact cores: construct only from Exact parts (literals, sealed atoms, @ Exact APIs).
3. Need weak data in a strong program → dual-path OR seal at the door — never bare meet.
4. Export a library claim → write @ g on pub API; checker proves body.
5. Ship a spore → envelope must match colony policy or explicit airlock import.
6. See "why Declared?" → tag-EXPLAIN (lean badge; audit DAG) — never guess.
7. @certification · loose/strict typing · @ g  are THREE dials (companion 04).
```

---

## 4. Options ranked

### Objective function

| Criterion | Weight | Notes |
|-----------|--------|-------|
| **C1 VR-5** — no silent upgrade; remint only with basis | **hard gate** | laundry = FAIL |
| **C2 G2** — isolation failure / weak export never silent | **hard gate** | |
| **C3 Exact-core preservation** | high | mandate |
| **C4 Ceremony bound** | high | KISS / DN-141 C3 |
| **C5 Deterministic first** | high | type/mode/phylum before convention |
| **C6 Axis clarity** | high | companion 04 |
| **C7 Incremental land** | medium | wave-friendly |
| **C8 No greenwash** | high | mandate |

### Alternatives

| Alt | Summary | C1 | C2 | C3 | C4 | C5 | C8 | Role |
|-----|---------|----|----|----|----|----|----|------|
| **0** | Docs-only (companion 02 as folklore) | pass | weak | weak | best | weak | med | baseline only |
| **A** | Dual-path + signature sea-wall only (M1+M4) | pass | pass | med-high | good | good | good | necessary layer |
| **B** | Full ambient quarantine on every nodule | pass | pass | high | **poor** | med | good | ceremony death |
| **C** | Join/strongest-wins "optimistic" composition | **FAIL** | fail | "high" fake | good | — | **FAIL** | **reject** |
| **D** | Seal phylum + meet-at-export (M2+M5 light) | pass† | pass | high | good | best | good† | **core** |
| **E** | Spore envelope + mode firewall (M3+M6) | pass | pass | med (deploy) | good | best | good | deploy layer |
| **F** | EXPLAIN-only (M10 without seals) | pass | pass* | weak | best | med | med | visibility, not isolation |
| **G** | Dataset partition (M7) + VSA policy (M8) | pass | pass | high (data/ML) | good | good | good | domain layer |
| **H** | **Rank-1 stack: A+D+E+F+G sequenced; B/C reject** | pass | pass | high | good | best | good | **recommend** |

†D passes C1/C8 only if remint soundness hinge holds. \*F passes C2 only if generation is
always-on and lean never hides *export* Declared.

### Named D-ranks (for AX join)

| Rank | Summary | Closes |
|------|---------|--------|
| **D1** | Meet-boundary table + quarantine exports | T1, T4 partial |
| **D2** | `std.airlock` seal/recertify | T1, T2 |
| **D3** | Certified consumer firewall (mode × grade) | T5, T1 |
| **D4** | Isolation EXPLAIN package | T2 opacity, G2 |
| **D5** | Dataset / spore grade partitions | T4 |
| **D6** | Lint profiles (certified public APIs) | T6 process |
| **Reject** | Global Declared floor / ambient Exact / `as Exact` laundry | O3 or O2 fail |

---

## 5. Recommendation (Draft) — **Containment Topology (CT) stack = D-core**

### Rank 1 — Deterministic firewalls at named boundaries; EXPLAIN everywhere; dual-path for Exact cores

Ship as **one design story**, land as **slices that extend AX-stack X2/X4** (do not fork a
second lattice).

```
                    ┌─────────────────────────────────────┐
                    │  Colony / spore envelope (M6)         │  deploy gate
                    └─────────────────┬───────────────────┘
                                      │
              ┌───────────────────────┼───────────────────────┐
              ▼                       ▼                       ▼
        certified phylum         fast phylum            draft/transpile
        pub @ Exact|Proven       modular bottom         Declared floor
              │                       │                       │
              │   seal (M2) ◄─────────┴─── only at export ────┘
              │
        Exact core dual-path (M1)     quarantine bag (M5 light)
              │                              │
              └──────── meet only inside bag / never across bag without seal
```

**D-core = D1 + D2 + D3 + D4**, with D5/D6 as packaging and process follow-ons.

| Slice | Content | AX join | Closes |
|-------|---------|---------|--------|
| **CT-0** | Dual-path tutorial + cast-upgrade lint + mixed `meet_all` lint | docs | T1 education |
| **CT-1 / D4** | Tag-EXPLAIN + isolation EXPLAIN; **boundary ≥ normal** even in lean `fast` | **X2** / **X11** | T2 opacity |
| **CT-2 / D2** | `std.airlock` seal + **checker remint hinge** + laundry CI | **X4** | T1, T2 |
| **CT-3 / D1+D3** | Meet-boundary table + signature sea-wall + certified firewall | **X9** / **X10** | T1, T5, T9 |
| **CT-4 / D5** | Spore grade+mode envelope + colony refuse | **X12** | T4, T7 deploy |
| **CT-5 / D5** | Dataset partition + VSA seal-to-codebook tests | **X12** | T4, T8 |
| **CT-6 / D6** | `public-api-must-annotate-grade` for certified exports | **X13** | T6 process |

**Sequencing:** CT-0/1/2/3 first (tables + EXPLAIN + seal hinge before sugar); CT-4/5 after
dogfood need; D2 remint DoD is the fragile joint.

### Explicit non-goals

- Optimistic join composition (**Alt C**) — reject.
- Ambient quarantine on every nodule (**Alt B**) — reject.
- Ambient nodule `@guarantee: Exact` — reject (DN-141 Alt C).
- Collapsing cert mode into grade — reject.
- Stage-1b grade polymorphism as prerequisite — defer.
- Global Declared floor to "be safe" — reject (kills O3/C3).

### Why this wins

| Criterion | How Rank-1 hits it |
|-----------|-------------------|
| C1/C2 | Remint hinge + never-silent Option + envelope refuse |
| C3 | Dual-path + seal keeps Exact cores free of dust *by construction* |
| C4 | Everyday code stays zero-tag modular bottom; ceremony only at **exports, seals, spores** |
| C5 | CT-2/3/4 are checker/metadata rules; CT-1 is deterministic trace |
| C6 | Envelope and EXPLAIN show three axes separately |
| C7 | CT-0→5 independently shippable |
| C8 | Laundry CI + no cast + Declared visible at boundaries |

---

## 6. Failure modes (stress catalog)

| FM | Attack | Mitigation in Rank-1 | Residual |
|----|--------|----------------------|----------|
| **FM1 Airlock laundry** | `seal` remints without real predicate | hard remint hinge; CI seal density; no `as Exact` | process slip if hinge softens |
| **FM2 Certified/fast cross-mode** | meet fast-Declared into certified Exact silently | mode tag + explicit cross-mode Option; envelope | axis confusion without CT-1 |
| **FM3 Dataset provenance** | one Declared row → batch Exact claim | M7 partition; lint `meet_all` | authors bypass helpers |
| **FM4 VSA resonator dust** | Empirical unbind meets structural Datum | FR-C2 cap + seal-to-codebook; dual-path | catalog miss on new op |
| **FM5 Transpile Declared flood** | gen drafts met into native Exact | quarantine drafts; differential before upgrade; sea-wall | dogfood pressure to fake Proven |
| **FM6 Envelope greenwash** | self-declared Exact envelope without checker | envelope from **attested** pub grades | tooling gap until CT-4 |
| **FM7 Lean DX hide** | lean elides Declared on exports | boundary sites ≥ normal | local noise still elided (OK) |
| **FM8 Ceremony revolt** | authors demand ambient Exact nodule | refuse; offer lint profile instead | political pressure |
| **FM9 Modular bottom "fake Exact body"** | body meets Declared params → cannot advertise Exact | G-Weaken already refuses; seal params first | education |

---

## 7. Adversarial stress-test (argue against Rank-1)

### Attack 1 — "Companion 02 + DN-141 E is enough; this is duplicate"

**Partial concede:** CT-2 **is** DN-141 E / AX-X4. **Counter:** DN-141 scopes *UX ceremony*;
this note scopes *plant-wide topology* (dataset, spore, dual-path, meet-at-boundary, transpile
flood). Without CT topology, E becomes a lonely helper nobody places correctly.

### Attack 2 — "Seal phylum becomes the laundry API everyone feared"

**Real.** If remint hinge is docs-only, Rank-1 **fails C1**. Mitigation is **checker-enforced**,
not tutorial. If engineering cannot land the hinge, **drop CT-2 remint**, keep dual-path +
EXPLAIN + signature sea-wall only (degraded Rank-1′).

### Attack 3 — "Spore envelope is YAGNI before deploy maturity"

**Agree on sequencing:** CT-4 after CT-1/2/3. Early dogfood uses phylum sea-wall alone.

### Attack 4 — "Partition helpers won't be used; meet_all stays the footgun"

**Likely.** Lint + EXPLAIN on mixed-grade `meet_all` is the real lever; helpers are sugar.
Weight CT-0/CT-1 over CT-5 if bandwidth is scarce.

### Attack 5 — "Deterministic isolation is impossible for ML; EXPLAIN-only should be Rank-1"

**For pure ML paths, dual-path may be empty.** Rank-1 still requires Exact *cores* (parsers,
width checks, content hashes) to dual-path; ML outputs stay Empirical with full EXPLAIN
package. EXPLAIN-only as the *sole* plant strategy fails C3 for mixed systems.

### Attack 6 — "Quarantine regions (M5) reintroduce ambient ceremony"

**Agree if ambient.** Rank-1 uses **light** M5: opt-in export-only seal, not nodule-wide
quarantine attributes.

### Stress-test verdict

Rank-1 survives with **sequencing** (CT-0/1/2/3 first; CT-4/5 later) and a **hard remint hinge**.
Fragile joints: (1) CT-2 laundry, (2) CT-4 attestation honesty, (3) dual-path education.
Degraded Rank-1′ (no remint sugar) still beats Alt 0 on C3 if dual-path + EXPLAIN + sea-wall land.

---

## 8. Open questions for the maintainer

| ID | Question | Default if silent |
|----|----------|-------------------|
| **OQ-D1** | Is containment topology a **new DN-142** or an **amendment** of DN-141? | **DN-142** after council steer (keeps DN-141 UX-scoped) |
| **OQ-D2** | Remint hinge: only **total Exact predicates**, or also **Empirical trial bases** for remint-to-Empirical? | Exact→Exact via total pred; Empirical remint only with attached trial basis (pairs DN-141 D) |
| **OQ-D3** | Typed carrier `Quarantined[T]` vs export-only seal without carrier? | export-only seal first (YAGNI) |
| **OQ-D4** | Spore envelope: **min pub grade** only, or full **export grade map**? | min + mode first; map later |
| **OQ-D5** | Should `certified` colonies **hard-refuse** `fast` spores, or allow with explicit import airlock? | allow with **explicit** airlock import |
| **OQ-D6** | Transpile: automatic **draft phylum quarantine** until differential? | yes for `gen/` and myc-drafts |
| **OQ-D7** | Lean DX: confirm **boundary ≥ normal** even when local lean elides Declared? | yes |
| **OQ-D8** | Name: `std.airlock` vs `std.seal` vs `std.quarantine`? | `std.airlock` phylum with `seal` fn (companion 02 + DN-141 OQ-7) |
| **OQ-D9** | Default meet policy cut: meet free inside nodule, refuse at `pub` without seal — or stricter cells? | nodule-internal free; `pub` refuse without seal or weak export |
| **OQ-D10** | NotValidated UX: std combinator with Agent A, or recover/diag? | share with Agent A OQ; named fallback EXPLAIN |

---

## 9. Definition of Done (maintainer steer of *direction*)

This planning note's direction is **steered** (not "Accepted" as a DN) when the maintainer confirms:

1. Rank-1 CT / D-core stack (or a superseding rank with same C1/C2/C8 hard gates) is the
   containment direction.
2. OQ-D1…D10 answered or explicitly deferred with owners.
3. Remint soundness hinge accepted as **checker-enforced** or CT-2 remint deferred.
4. No status move on DN-141 / RFC-0018 / RFC-0034 implied; build slices remain `Declared` until
   witnessed.
5. Integrating parent applies Doc-Index / CHANGELOG / issues rows only after DN promotion
   (if OQ-D1 = DN-142).

---

## 10. Suggested work items (wave re-rank)

Extends AGENT-C X2/X4 and DN-141 §9 — **do not double-mint**; prefer **widening** those issues.

| Priority | Suggested track | Title | Slice | Depends |
|----------|-----------------|-------|-------|---------|
| P0 | widen DN-141 F / AX-X2 | Grade provenance EXPLAIN + **boundary ≥ normal** + isolation package schema | CT-1 / D4 | RFC-0034 §7 |
| P0 | widen DN-141 E / AX-X4 | `std.airlock` seal + **checker remint hinge** + laundry CI | CT-2 / D2 | companion 02; grade pass |
| P0 | docs/lint | Dual-path tutorial + cast-upgrade lint + mixed `meet_all` lint | CT-0 | companion 02 |
| P1 | new (after OQ-D1) | DN-142 Containment Topology (if steered) | design | this note |
| P1 | lint / checker | Meet-boundary table + certified consumer firewall | CT-3 / D1+D3 | stage-1a grade |
| P1 | lint | `public-api-must-annotate-grade` for certified exports | CT-6 / D6 | DN-141 OQ-4 |
| P1 | structural catalog | Exact-safe constructor catalog (dual-path feed) | CT-3 | DN-141 A |
| P2 | spore | Grade+mode envelope on spore metadata + colony refuse | CT-4 / D5 | ADR-013; CT-1 |
| P2 | data/VSA | `partition_by_grade` + resonator→codebook seal tests | CT-5 | FR-C2; CT-2 |
| P3 | transpile | Draft-phylum quarantine / depend edge lint vs Exact cores | CT-0 | OQ-D6; DN-34 |
| — | non-goal | ambient quarantine regions; join-composition; ambient Exact nodule | — | reject |

**Wave placement:** design-council pause → maintainer steer → promote DN-142 if wanted → land
CT-0/1/2 in same wave as AX-X2/X4 → CT-3 with catalog → CT-4/5 after dogfood need. **Do not**
interleave product remint code under this design phase until steer (council brief).

---

## 11. Cross-links to A / B / C AX-stack

| AX / source | Agent D extension |
|-------------|-------------------|
| **X1** legal-pair / policies (A) | Lossy swap policies feed M8 reconstruction honesty |
| **X2** grade catalog + tag-EXPLAIN (B/C) | **CT-1** adds boundary ≥ normal + isolation package |
| **X3** regime → Option/Result (A) | Seal failures reuse same fallibility story |
| **X4** `std.airlock` (B/C) | **CT-2** hardens remint hinge + laundry CI + plant topology placement |
| **X5** cert ambient for swap (A) | Orthogonal; mode still never upgrades grade at seal |
| **X6** optional sugar (A/B) | Only after CT-1 so omission ≠ Exact |
| **X7** basis-carrying tags (B) | OQ-D2 Empirical remint couples to X7 |
| **X8** LSP insert-swap (A) | LSP should also suggest **seal** at Exact demand failures |

**New AX rows proposed for C's table (if C revises):**

| Rank | Slice | Source | Closes |
|------|-------|--------|--------|
| **X4a** | Remint hinge in checker (not docs) | D CT-2 | FM1 laundry |
| **X4b** | Dual-path Exact-core patterns + lint | D CT-0/3 | P-D1/P-D5 |
| **X9** | Meet-boundary / export quarantine table | D D1 | T1 spine |
| **X10** | Certified consumer firewall (mode × grade) | D D3 | T5 |
| **X11** | Isolation EXPLAIN package | D D4 | G2 dynamic |
| **X12** | Spore/dataset grade partitions | D D5 | T4 |
| **X13** | Certified public-API grade lint profiles | D D6 | T6 |

---

## 12. FLAGs for integrating parent (read-only here)

| Artifact | Row (dated 2026-07-17) |
|----------|------------------------|
| `docs/Doc-Index.md` | planning: AGENT-D honesty-poison containment (Draft) under gap-analysis-2026-07-16 |
| `CHANGELOG.md` | `docs(planning): agent-D honesty-poison containment CT/D-core stack (draft)` |
| `tools/github/issues.yaml` | mint/widen only after maintainer steer + free-id verify; link this path |
| DN-141 | optional cross-link from §5/§9 to this topology note once steered |
| AGENT-C | optional X4a/X4b/X9–X13 rows after council |
| `CLAUDE.md` | no house-rule change |

---

## 13. Changelog (this note)

| Date | Change |
|------|--------|
| 2026-07-17 | **Draft** — Design Agent D: threat model T1–T9, surfaces P-D1…P-D8, mechanisms M1–M10, alts 0/A–H + D-ranks, Rank-1 CT/D-core stack, EXPLAIN minimum package, failure modes, adversarial stress-test, OQs, wave items, AX-stack extensions X4a–X13. |

---

## Meta

- **Honesty of this note:** design `Declared`; citations `Empirical`/`Exact` as tagged. Nothing `Proven`.
- **Supersedes:** nothing. **Amends:** nothing normatively until a DN is promoted and Accepted.
- **Related:** companion 02/04 · DN-141 · DN-29 (Superseded) · ADR-032 / RFC-0034 · RFC-0018 stage-1a · RFC-0001 lattice · AGENT-A/B/C · ADR-013 spores.
