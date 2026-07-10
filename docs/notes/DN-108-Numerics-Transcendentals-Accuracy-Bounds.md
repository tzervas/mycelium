# Design Note DN-108 — Numerics Transcendentals under the Transparency Rule (the ε/δ surface, ENB-5 / #42)

| Field | Value |
|---|---|
| **Note** | DN-108 |
| **Status** | **Draft** (2026-07-10). Authored as a **design-forward reasoner note** for the DN-99 register row **#42** (float-eps-delta-numerics / transcendentals, `open`, runtime-`enb`, ENB-5) — one of the four DN-99 `open` surface gaps. It **works the decision forward to a ranked recommendation**; it **enacts nothing**, **ratifies nothing**, and **moves no other doc's status** (house rule #3, append-only). Tags are `Empirical` where read against the tree (dev tip `5130badc`, 2026-07-10), `Declared` for any design not yet built/ratified (VR-5). |
| **Decides** | *Proposes, for ratification (does not self-ratify):* (1) transcendental/irrational-result functions (`sqrt`, `exp`, `ln`, `sin`, `cos`, `pow`) surface as **`flt.*` interpreter prims** that return a plain `Repr::Float` **value** whose honest accuracy bound rides in **the existing `Bound`/`Approx` provenance certificate** (ADR-010) — **no new numeric surface type, no new kernel node** (FR-N1 / KC-3); (2) the v0 per-op guarantee tag is **`Declared`**, not `Empirical` — the ε is *asserted* from the host libm's documented ULP bound, there being no measured reference-case corpus yet (VR-5: no `Empirical` without trials; §6/§7.2); (3) out-of-domain inputs (`ln(x ≤ 0)`, `sqrt(x < 0)`, `pow` indeterminate forms) are a **never-silent `Result`/`NumErr::Domain` refusal**, never a silent NaN or fabricated value (G2); (4) **v0 refuses composition of an *approximate* transcendental input** (`ApproxRule::Refuse` — the ADR-040 §2.5 posture), because transcendental ε-propagation is nonlinear and has no checked rule yet — `sin(exp(x))` over an already-approximate `x` refuses rather than compounding a fabricated bound; only `Exact` (literal) inputs feed v0 transcendentals; (5) the **fast/certified mode split (RFC-0034/ADR-032)** is the *growth path* (`certified` earns `Empirical`→`Proven` as a corpus/theorem lands), **not** a v0 requirement (YAGNI). It does **not** edit `issues.yaml`, `CHANGELOG.md`, `Doc-Index.md`, `lib/**`, or `crates/**` — the integrator / cloud semcore lane own those (FLAGGED in §9). |
| **Feeds** | DN-99 register row #42 + §4 Track-A **A4** (ENB-5); ADR-010 (the ε/δ kernel + shared certificate this note *consumes*, adds nothing to); ADR-040 / DN-69 (the scalar-float landing whose `flt.*` + `ApproxRule` pattern this note extends); ADR-032 / RFC-0034 (fast·balanced·certified mode — the growth path); DN-104 / M-1027 (the constructor seal that lets `Approx::proven` port faithfully — the path to a `Proven` transcendental); RFC-0001 §4.7 (bound composition / the guarantee lattice); `lib/std/numerics.myc` FLAG-num-1 (the ε/δ magnitude surface this row would close). |
| **Grounds on** | KC-3 (small kernel — reuse the ADR-010 `Bound`/`Approx` certificate and the M-204 `ApproxRule` prim dispatch; **no new L0 node, no new numeric type, no new checking pass** — the prims are new registry entries, the bound is the existing record), DRY (the `flt.*` group + `flt_bound()` shape is the template), G2/never-silent (domain refusals + composition refusals both print the fix; no silent NaN, no fabricated ε), VR-5 (the tag is exactly what the basis supports — v0 = `Declared` because the ε is asserted; upgraded only by a corpus / a checked theorem), KISS/YAGNI (defer interval arithmetic and the first-class `Bounded<Float>` surface; defer the mode split to when `certified` has content). |
| **Date** | July 10, 2026 |
| **Task** | DN-99 #42 / ENB-5 (A4) — transcendental + ε/δ float numerics design-forward. |

> **Grounding + honesty (house rule #4 / VR-5 / G2).** This note **works a decision forward and
> recommends, ranked**; it does **not** take the decision (house rule #3 — the maintainer ratifies).
> Every claim about existing machinery carries a `file:line` read against dev tip `5130badc`
> (`Empirical`). Every design not yet built is `Declared`. **No tag is upgraded past its basis, and the
> central recommendation is deliberately the *weaker* honest tag** (`Declared`, not `Empirical`) — §6
> confronts head-on the ergonomic pressure to upgrade it and refuses. **No sycophancy:** §7 states the
> two concerns that most threaten the recommendation (nonlinear composition; libm non-correct-rounding
> defeating the bit-identical differential) plainly, and §5's rank-1 is the *simplest* option that meets
> the objective, not the most feature-rich — even though the maintainer's own register row #42 sizes the
> gap "XL" and lists a "per-op ε/guarantee matrix" that a naive reading would build up front.

---

## §1 Purpose and frame

DN-99 row #42 is one of four genuinely-`open` surface gaps: Mycelium's stdlib has **no transcendental
surface** (`sin`/`cos`/`exp`/`ln`/`sqrt`/`pow`). `lib/std/math.myc` is integer/binary/ternary only
(`badd`/`bmul`/`band`/…, verified `math.myc:53-115`); `lib/std/numerics.myc` ports **only the strength
lattice** and explicitly **FLAGs the ε/δ magnitude surface** as not-yet-closed (`numerics.myc:49-55`,
FLAG-num-1). The crux the maintainer names: a transcendental result is **at best `Proven`** (with a
checked error theorem), else `Empirical` (a measured bound), else `Declared` (asserted) — it is **never
`Exact`**, and it must **never be a silent approximation** (G2). The design question is precisely: *how
does a `sin` result carry its ε/δ tag per-op, and what does v0 land?*

**What already exists (verified — this note adds nothing to the kernel).**
- The **ε/δ certificate** (ADR-010, Accepted): `ErrorBound{eps, norm}` (`numerics/error.rs:221`),
  `ProbBound{delta}`, the shared record `Bound{kind, basis, strength}` reducing to
  `{ε, δ, strength ∈ {Exact,Proven,Empirical,Declared}}`, `BoundBasis{ProvenThm|EmpiricalFit|
  UserDeclared}` whose `strength()` is the *only* honest tag source, and the tier-i Rust checker.
- The carrier: `Approx<T>` (`std-numerics/src/lib.rs:260`) — a value + bound + basis-derived strength;
  `attach`/`declared`/`empirical`/`proven` builders, the last **sealed** behind a `ProvenThm` witness
  token (`sealed::Sealed`, `lib.rs:101-155`) so `Proven` is unreachable without a checked citation.
- The **interpreter composition dispatch**: `ApproxRule ∈ {Refuse, Passthrough, Error(ErrorOp)}`
  (`interp/src/prims.rs:41`), applied at every prim boundary (M-204). `ErrorOp ∈ {Add,Sub,Neg,
  Scale,Mul}` (`numerics/cert.rs:42`) — **note: no transcendental op yet**.
- The **scalar-float precedent** (ADR-040, **Enacted** 2026-07-02): `flt.add/sub/mul/div/neg` return a
  `Repr::Float` value carrying `Bound{ErrorBound{eps: 0.0, Linf}, EmpiricalFit{40 trials}, Empirical}`
  (`prims.rs:2050-2090`, `FLT_CONFORMANCE_TRIALS = 40`). Two facts are load-bearing for #42:
  **(i)** ADR-040 §2.5 **deliberately keeps transcendentals OUT of the kernel** — "libm is NOT involved"
  (`prims.rs:2079`); #42 is exactly the deferred surface. **(ii)** `flt.*` composition **accepts an input
  only if it is `Exact` (a literal) or carries the zero-deviation `Empirical` form; any *other* bound is
  an explicit `EvalError::ApproxCompositionUnsupported` refusal** (`prims.rs:2085-2090`) — the honest
  "refuse, don't guess" default this note inherits.

So #42 is **not a from-scratch numerics design**. The kernel, the certificate, the carrier, the seal,
and the prim-boundary dispatch are all landed. #42 is the **narrow question of how a genuinely-approximate
(nonzero-ε, non-correctly-rounded) prim attaches an honest bound and composes** — everything the
scalar-float landing deferred.

**Premise-check correction (mitigation #14, verify-first).** `numerics.myc:52-54` (dated 2026-07-09)
states the `.myc` runtime "has NO scalar-float VALUE form yet … no value can be constructed/evaluated."
This is **stale**: ADR-040 is **Enacted** (float literal M-897, arithmetic M-898 landed 2026-07-02;
`Repr::Float` + `Payload::Float(x)` live at `prims.rs:2109`). The float *value* form exists. This note
proceeds on the verified state (float values evaluate), and FLAGs the stale comment in §9.

### §1.1 Requirements (what a transcendental MUST satisfy)

- **R1 — never `Exact`.** A transcendental result's tag is `Proven`/`Empirical`/`Declared`, never
  `Exact` (the value is not the exact real; VR-5).
- **R2 — bound carried per-op, never dropped.** The ε (and, where relevant, δ) travels with the value in
  the provenance certificate even when the *surface* type is a bare `Float` (the flt.* discipline).
- **R3 — never-silent domain.** Out-of-domain inputs (`ln(x ≤ 0)`, `sqrt(x < 0)`, `pow(0, ≤0)` /
  `pow(<0, non-integer)`) are an explicit `Result`/`NumErr` refusal, never NaN-in-band, never a
  fabricated value (G2, rule #2).
- **R4 — honest tag under composition.** A composed result's tag is **≤** the meet of its inputs'
  strengths *and* its bound is a **true upper bound** on the composed deviation — or the composition
  **refuses**. No fabricated compounding rule (the ADR-040 §2.5 / M-204 posture).
- **R5 — tag matches basis (v0).** If the v0 ε is asserted (from libm docs), the tag is **`Declared`**;
  it becomes `Empirical` only with a measured reference corpus, `Proven` only with a checked theorem +
  the `ProvenThm` seal (DN-104). No upgrade without a checked basis (VR-5).
- **R6 — small kernel.** Reuse ADR-010's certificate and M-204's dispatch; add no new numeric type and
  no new checking pass (KC-3, rule #5).

### §1.2 Definition of Done (the v0 gate — for maintainer ratification, §8)

Transcendental prims `flt.sqrt/exp/ln/sin/cos/pow` registered; each returns a `Repr::Float` value with a
`Bound{ErrorBound{eps, Linf}, BUserDeclared, Declared}`; domain errors refuse never-silently; approximate
inputs refuse composition (`ApproxRule::Refuse`); a differential/conformance harness pins the *domain
refusals* and the *tag* (not bit-identical values — see §7.3); the stale `numerics.myc` float comment is
refreshed. `Empirical`/`Proven` and `sin(exp(x))`-style composition are explicitly **out of v0**.

### §1.3 User stories

- *As a numerics porter*, I want `sqrt(2.0)` to return a `Float` I can use, **so that** I can port a Rust
  stdlib routine — but with the ε visible on inspection so I never mistake it for exact.
- *As a certified-mode consumer*, I want `ln(x)` for `x ≤ 0` to **refuse** with a typed domain error,
  **so that** an out-of-domain call is a caught `Result`, not a silent NaN that poisons a downstream
  bound.
- *As a maintainer auditing transparency*, I want the v0 transcendental tag to read `Declared` when the
  bound is asserted, **so that** the corpus never claims measured accuracy it does not have (VR-5).
- *As a future certified-numerics author*, I want the ε to ride the **same** `Bound` certificate the
  kernel already checks, **so that** upgrading a bound to `Proven` is a basis+witness change, not a
  surface rewrite.

---

## §2 What must a transcendental return? (the representation question, isolated)

A transcendental `f: Float → Float` computes an approximation `ŷ` of the real `f(x)`. Honesty requires
the result object to expose, per-op: **(a)** the value `ŷ`; **(b)** an error bound `ε` such that
`|ŷ − f(x)| ≤ ε` in some norm, with **(c)** a `basis` recording *how ε was obtained*, which **(d)**
*derives* the `strength` tag (never a caller-set field). This is **exactly** the `Bound`/`Approx`
certificate ADR-010 already defines. The open design choice is therefore **not "what fields"** — it is
**where the certificate lives relative to the value** (op-level provenance vs a first-class surface pair
vs an interval), **what the v0 basis/strength is**, and **how composition behaves**.

---

## §3 Enumerated alternatives (real options, each with its mechanism)

**(a) Op-level bound in provenance — extend the `flt.*` pattern.** The prim returns a plain
`Repr::Float` value; the `Bound{ErrorBound{eps}, basis, strength}` is attached to the op's result
metadata exactly as `flt_bound()` does for arithmetic (`prims.rs:2090+`), flowing through the M-204
`ApproxRule` machinery. Surface type is `Float`; the ε is inspectable via `EXPLAIN`/the certificate.
*Mechanism already exists for arithmetic; #42 adds the transcendental prims + their basis.*

**(b) First-class `Bounded<Float>` / `(value, error)` surface pair.** Transcendentals return an
`Approx[Float]` value the **caller must destructure** — the ε is in the type, not just provenance. In
`.myc` this means graduating the `Approx[A]` carrier (already ported for the *strength* half,
`numerics.myc:158`) to carry the float ε magnitude and be the return type of `sqrt`/`sin`/….
*Mechanism: promote `Approx[Float]` to a returned surface type; requires the float ε magnitude in `.myc`
(FLAG-num-1) and a caller-side unwrap idiom.*

**(c) Interval / ball arithmetic.** Return `[lo, hi]` (or `(center, radius)`) and propagate rigorously
through every op with directed rounding (the `round::add_up`/`AffineForm` machinery, `numerics/error.rs`,
partially exists). A transcendental returns a proven enclosure of `f(x)`.
*Mechanism: a new interval numeric type + directed-rounding transcendental enclosures (Gappa/RLIBM-class
work); rigorous but a large new surface + kernel.*

**(d) Mode split (fast `Declared` vs certified `Proven`-with-theorem), per RFC-0034/ADR-032.** The same
prim behaves differently by active certification mode: `fast` returns a bare/`Declared`-tagged float (no
recompute), `certified` returns the `Approx` carrier with a checked bound and a `ProvenThm` witness.
*Mechanism exists as a mode concept (ADR-032 footnote in ADR-010); needs the bound content per mode.*

**(e) Defer transcendentals entirely (do-nothing).** Keep #42 `open`; ports that need `sqrt`/`sin`
stay on the Rust oracle or use rational/integer approximations by hand. *No code; the honest status quo.*

---

## §4 Evaluation against the objective

Objective function (weighted by the house rules): **R1–R6** above + ergonomics + small-kernel + value
semantics. Scored `++ / + / ~ / − / −−` (best→worst) with the grounding.

| Criterion (basis) | (a) op-level prov. | (b) `Bounded<Float>` | (c) interval | (d) mode split | (e) defer |
|---|---|---|---|---|---|
| R1 never-`Exact` (VR-5) | ++ (basis derives tag) | ++ | ++ | ++ | ~ (n/a) |
| R2 bound carried per-op (rule #1) | + (in provenance; inspectable, but *droppable* by a careless surface) | ++ (in the type — can't drop) | ++ | + | −− |
| R3 never-silent domain (G2) | ++ (`Result`/`NumErr`) | ++ | ++ | ++ | ~ |
| R4 honest composition (§1.1) | + (refuse until a rule exists — honest, but limits use) | + | ++ (rigorous by construction) | + | ~ |
| R5 tag matches basis (VR-5) | ++ (`Declared` v0; upgrade path clean) | ++ | + (tempts a `Proven` claim before the enclosure is checked) | ++ | ~ |
| R6 small kernel (KC-3) | ++ (reuse cert + dispatch; **0 new types**) | + (graduate `Approx[Float]` surface) | −− (new numeric type + rounding transcendentals) | + | ++ |
| Ergonomics (`sin(x): Float`) | ++ (bare float) | − (caller unwraps every call) | − (interval everywhere) | ++ (fast = bare) | −− |
| Value semantics (ADR-003) | ++ | ++ | + | ++ | ~ |
| Ships in v0? (YAGNI, rule #5) | ++ (prims + basis only) | ~ (needs float-ε surface) | −− (XL) | ~ (needs per-mode content) | ++ (nothing) |

**Reading.** (a) dominates on small-kernel + ships-in-v0 + ergonomics, at the cost of R2/R4 (the bound is
in provenance not the type, and composition must *refuse* rather than propagate). (b) is strictly more
honest on R2 (the ε can't be silently dropped) but costs ergonomics and needs the float-ε `.myc` surface
first. (c) is the rigorous end-state but XL and kernel-heavy. (d) is not a rival to (a) — it is a *layer*
over it. (e) is the honest do-nothing baseline.

---

## §5 Recommendation — RANKED (Draft; the maintainer ratifies)

**Rank 1 — (a) op-level bound in provenance, tag `Declared` in v0.** Extend the `flt.*` group with
`flt.sqrt/exp/ln/sin/cos/pow`, each returning a `Repr::Float` value carrying
`Bound{ErrorBound{eps, Linf}, BUserDeclared, Declared}`. This is the **KISS/DRY/KC-3 answer**: it reuses
ADR-010's certificate, the `flt_bound()` shape, the M-204 `ApproxRule` dispatch, and the DN-104 seal path
for later `Proven` — **zero new numeric types, zero new kernel nodes, zero new checking passes**. It ships
the *ergonomic* `sin(x): Float` while keeping the ε inspectable (R2 met at the provenance level, R1/R3/R5
met fully).

**Rank 2 — (d) the fast/certified mode split, as the GROWTH PATH layered on Rank 1 (not a v0 item).**
Once a measured corpus earns `Empirical` and a checked bound + `ProvenThm` earns `Proven`, `certified`
mode returns the stronger-tagged carrier and `fast` keeps the bare `Declared` float. This is already how
ADR-032/RFC-0034 frames kernel invocation; v0 does not need it (the tag machinery is identical at both
modes — only the *bound content* differs). **Fold into Rank 1's roadmap, do not build up front.**

**Rank 3 — (b) first-class `Bounded<Float>` surface.** Strictly more honest on R2 (the ε cannot be
dropped), and the right answer *if* provenance-level bounds prove too droppable in practice. Deferred: it
needs the float-ε magnitude in `.myc` (FLAG-num-1) and a caller-unwrap idiom, and the strength half is
already modeled — revisit after v0 exposes whether the provenance bound is enough.

**Rank 4 — (c) interval / ball arithmetic.** The rigorous end-state (proven enclosures, R4 by
construction), but XL, kernel-heavy, and premature (YAGNI). Track as the long-horizon `certified`
substrate, not a v0 or near-term item.

**Rank 5 — (e) defer entirely.** Rejected as the recommendation, but recorded as the honest fallback:
**if the maintainer wants the float-value-in-`.myc` self-hosted surface fully settled before adding an
approximate prim class**, #42 legitimately stays `open` and ports keep using the Rust oracle. This is a
real fork (§7.4).

### §5.1 Minimal v0 scope that lands

- **Functions (6):** `flt.sqrt`, `flt.exp`, `flt.ln`, `flt.sin`, `flt.cos`, `flt.pow` — the DN-99 #42
  named set. Registered as interpreter prims mirroring the `flt.*` group (`prims.rs:210-237`).
- **Tag default: `Declared`.** Basis `BUserDeclared` → `strength()` = `Declared`. The ε is the host
  libm's **documented** worst-case ULP bound (e.g. a small constant × ULP), **asserted, not measured** —
  so the honest tag is `Declared` (§6, §7.2). *Not `Empirical`, not `Proven`.*
- **Bound sourcing:** the asserted ε is a named constant with a cited source (the platform libm doc), in
  the `Linf` norm; recorded in the certificate exactly as `flt_bound()` records the arithmetic bound.
- **Domain errors (never-silent):** `ln(x ≤ 0)`, `sqrt(x < 0)`, `pow(0, ≤0)`, `pow(x < 0, non-integer)`
  → `Result`/`NumErr::Domain` (add a `Domain` variant to `NumErr`; today's variants are `BadEps/BadDelta/
  NoRule/NormMismatch/Overflow`, `numerics.myc:200`). No in-band NaN, no fabricated value.
- **Composition:** `ApproxRule::Refuse` for an *approximate* transcendental input — only `Exact` (literal)
  inputs feed a v0 transcendental; `sin(exp(x))` over an approximate `exp(x)` **refuses**
  (`ApproxCompositionUnsupported`) rather than compounding a fabricated bound (§6.1).
- **Witness:** a conformance/differential harness pinning **(1)** every domain refusal and **(2)** the
  `Declared` tag + the recorded ε constant — **not** bit-identical values against a reference (§7.3).
- **Out of v0:** `Empirical`/`Proven` tags, the transcendental ε-propagation `ErrorOp`, the mode split,
  δ (probability) bounds (transcendentals are deterministic — no δ), interval/`Bounded<Float>` surface.

---

## §6 Adversarial stress-test (rule #4 / VR-5 — run the recommendation through what breaks it)

### §6.1 Does the tag stay honest under composition? (`sin(exp(x))`) — the central concern

**Sequence:** `exp(x)` yields `ŷ₁` with a nonzero-ε `Declared` bound; feed `ŷ₁` into `sin`. Naively one
might **sum** the errors and return a `Declared` result — *this would be a lie.* Transcendental error
propagation is **nonlinear**: `ε_out ≈ |f'(ξ)|·ε_in + ε_op`. For `sin`, `|cos| ≤ 1` is benign; but for
`exp`, `|exp'| = exp` **amplifies** the input error unboundedly, and a naive additive rule would *understate*
`ε_out` — an unsound bound, the exact VR-5 failure (a tag/bound stronger than its basis). **Verdict:** v0
must **refuse** composition of an approximate transcendental input (`ApproxRule::Refuse`), inheriting the
ADR-040 §2.5 posture (`prims.rs:2085-2090` already refuses non-zero-deviation float composition). This is
the honest outcome, and it **bounds v0's usefulness**: `sin(exp(x))` does not evaluate in v0 — only
`sin(<literal>)` does. The sound composition rule (a transcendental `ErrorOp` carrying the derivative
bound / a mean-value or affine enclosure) is **real future work**, tracked as an open question (§7.1). *This
is the top surviving concern: the recommendation is honest precisely because it refuses, and that refusal
is a genuine functional limit the maintainer must accept for v0.*

### §6.2 Is `Empirical` defensible without measured trials? — no.

The maintainer's mandate is explicit and correct: **v0 has no measured reference-case corpus** for
transcendental accuracy (nothing analogous to `FLT_CONFORMANCE_TRIALS = 40` hand-derived IEEE cases,
`prims.rs:2079`). The v0 ε is *asserted* from libm documentation. Claiming `Empirical` (which means "fit
to trials") would upgrade the tag past its basis — the canonical VR-5 violation, ranked with an ungrounded
claim (rule #4). **Verdict:** v0 is **`Declared`**. `Empirical` is *earned* by landing a transcendental
reference-case corpus (the direct analogue of the flt.* corpus); `Proven` is earned by a checked
correctly-rounded/error-analysis theorem (CR-libm / RLIBM / Gappa-class) plus the DN-104 `ProvenThm`
seal. The upgrade path is clean *because* the bound rides the same certificate — but v0 does not walk it.

### §6.3 Where does ergonomics tempt a silent upgrade? — the `fast`-mode footgun.

Users want `sin(x): Float`. The temptation is to have `fast` mode return a **bare float with the bound
dropped** — which reads downstream as an *unmarked, exact-looking* value: the precise G2/VR-5 failure.
**Verdict:** `fast` mode is **`Declared`-without-recomputation, never bound-dropped and never
`Exact`-tagged.** The bound stays in provenance at every mode (R2); `fast` only skips the *tier-i
re-derivation/checking*, not the *disclosure*. Rank-1 keeps the surface type `Float` (ergonomic) **and**
the certificate attached (honest) — the two are not in tension once the bound lives in provenance, exactly
as `flt.*` already demonstrates.

### §6.4 libm is not correctly-rounded — it defeats the bit-identical differential. (second surviving concern)

Unlike `flt.*` arithmetic (IEEE-754 correctly-rounded, *bit-reproducible* across platforms —
`prims.rs:2075`), **transcendentals are not correctly-rounded on most libms** (the table-maker's dilemma):
`sin`/`exp` results differ bit-for-bit across platforms and libm versions. So the zero-deviation-vs-spec
trick that lets `flt.*` claim a reproducible `Empirical` bound **does not transfer** — there is no single
spec bit-pattern to diff against. **Consequences for v0:** (1) the three-way differential (L1-eval ≡
L0-interp ≡ AOT) **cannot assert bit-identical transcendental values** — it must pin the *tag*, the
*domain refusals*, and at most a *tolerance-band* agreement (itself an `Empirical`-class comparison, not
`Exact`). (2) To make results reproducible at all, v0 should **pin a single vendored implementation** (a
specific correctly-rounded or version-locked routine) rather than call the ambient host libm — otherwise
"the value" is platform-dependent, which is itself a never-silent hazard. This is a real design
constraint that pushes toward a vendored routine even in v0, and it is why the §5.1 witness pins tag +
refusals, not values. Folded into §7.3.

---

## §7 Risks and open questions (survivors of §6, stated plainly — VR-5/G2)

1. **OQ-1 — the transcendental ε-propagation `ErrorOp` (composition).** v0 refuses approximate-input
   composition (§6.1). The sound rule (a mean-value/derivative-bound or affine enclosure per function,
   extending `ErrorOp` beyond `Add/Sub/Neg/Scale/Mul`) is the single biggest follow-up; without it,
   transcendentals only accept `Exact` inputs. **Open.** (Ranks (c)/interval is one principled way to get
   R4 by construction.)
2. **OQ-2 — earning `Empirical`/`Proven`.** What is the transcendental reference-case corpus (the flt.*-analogue)
   and its trial-count constant? What theorem + witness earns `Proven` (CR-libm / RLIBM / Gappa), and does
   the DN-104 seal suffice to gate it? **Open.**
3. **OQ-3 — reproducibility vs the host libm (§6.4).** Does v0 vendor a version-locked / correctly-rounded
   routine (reproducible, heavier) or call the ambient libm (light, platform-dependent)? The differential
   harness can only pin bit-identical values under the former. **A fork the maintainer should settle —
   leaning vendored, for a never-silent, reproducible result.**
4. **OQ-4 — the (e)/defer fork (sequencing).** Should #42 wait until the float-value self-hosted `.myc`
   surface is fully settled (numerics.myc FLAG-num-1 / the `.myc`-level ε magnitude), or land the interp
   prims now with the bound in Rust provenance? Rank-1 lands now; (e) waits. **Maintainer's call.**
5. **OQ-5 — δ (probability) is not applicable to deterministic transcendentals.** ADR-010's `ProbBound`
   is for failure-probability composition (VSA/retrieval); a scalar transcendental has an ε, not a δ.
   v0 carries ε only. Recorded so no one wires a spurious δ. **Closed by statement.**
6. **RISK-1 — provenance bound is droppable.** Rank-1 puts the ε in provenance, not the type; a careless
   surface could drop it (the R2 weakness vs Rank-3 (b)). Mitigated by the never-silent discipline + the
   `EXPLAIN` path; escalate to (b) if it proves insufficient.

---

## §8 Definition of Done — what "Accepted" requires of the maintainer (house rule #6)

This note is **Draft** and stays Draft until the maintainer:
1. **confirms the representation choice** — Rank-1 (op-level bound in the existing ADR-010 certificate; no
   new numeric type), or selects an alternative (§3/§5) with rationale;
2. **ratifies the v0 tag as `Declared`** (not `Empirical`/`Proven`) and the asserted-ULP bound sourcing
   (§5.1, §6.2);
3. **settles OQ-3 (vendored vs host libm)** and **OQ-4 (land-now vs defer to (e))** — the two genuine
   forks;
4. **authorizes the ENB-5 tracking issue + a companion ADR** (row #42 is `DN? = yes`; a kernel-prim class
   with an accuracy contract is an ADR-level decision, parallel to ADR-040) — the reasoner does **not**
   file `issues.yaml` or self-ratify (rule #3/#4);
5. accepts the v0 **functional limit** that approximate-input composition refuses (§6.1) and the
   differential pins **tag + refusals, not bit-identical values** (§6.4).

Until then, no code lands from this note; it recommends, it does not enact.

---

## §9 Doc-Index + changelog + issues (FLAGGED up — NOT applied here)

`docs/Doc-Index.md`, `CHANGELOG.md`, and `tools/github/issues.yaml` are **integration-owned** (the
concurrent-PR pattern: the reasoner FLAGs, the integrating parent applies once). This note edits none of
them. **FLAGs to the integrator:**
- **Doc-Index row:** add a Design-Notes entry `DN-108 — Numerics Transcendentals under the Transparency
  Rule (Draft, 2026-07-10)`, feeding DN-99 #42 / ENB-5, ADR-010, ADR-040.
- **CHANGELOG:** an append-only entry for DN-108 created (Draft) — design-forward note for row #42.
- **issues.yaml linkage:** wire DN-108 as the design note for the **ENB-5** backlog item (DN-99 §8) /
  **M-1028** (row #42); mark row #42's DN-flag satisfied-by DN-108 (Draft). File the ENB-5 tracking issue
  + companion ADR per §8.4. Do **not** flip row #42's status (append-only; ratification-gated).
- **Stale-comment refresh (mitigation #14):** `lib/std/numerics.myc:52-54` FLAG-num-1's "no scalar-float
  VALUE form yet" is stale post-ADR-040-Enacted (§1); flag for a doc-hygiene refresh in the self-hosted
  numerics workstream (not this note's write scope).

---

## §10 Changelog

- **2026-07-10** — DN-108 created (**Draft**). Design-forward reasoner note for DN-99 register row #42
  (transcendental ε/δ numerics, ENB-5 / A4). Framed the requirements (R1–R6) + DoD + user stories;
  enumerated five real alternatives (op-level provenance bound / first-class `Bounded<Float>` / interval
  arithmetic / mode split / defer); evaluated them against the transparency rule, never-silent G2,
  ergonomics, small-kernel, and value-semantics; **recommended, ranked**, Rank-1 = op-level bound in the
  existing ADR-010 certificate with a **`Declared`** v0 tag (asserted ULP, no measured corpus — VR-5),
  never-silent domain refusals, and **composition-refusal** of approximate inputs (ADR-040 §2.5 posture);
  ran the adversarial stress-test (nonlinear composition; `Empirical`-without-trials; the `fast`-mode
  silent-upgrade temptation; libm non-correct-rounding defeating the bit-identical differential) and
  folded the survivors into OQ-1..5 / RISK-1. `Empirical` where cited against the tree (dev tip
  `5130badc`); `Declared` for all unratified design. Authored **READ + this DN only** — no edit to
  `crates/**`, `lib/**`, `issues.yaml`, `CHANGELOG.md`, or `Doc-Index.md` (FLAGGED §9). Append-only;
  status advances only by maintainer ratification (house rule #3 / #4). The reasoner recommends; it does
  not ratify.
