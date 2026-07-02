# ADR-038 — Pragmatic Dogfooding: the Function-First Release Strategy ("Rust Where Appropriate, Mycelium Everywhere Else")

| Field | Value |
|---|---|
| **ADR** | 038 |
| **Status** | **Proposed** (2026-07-01 — authored from the maintainer's same-day session directives, **for maintainer ratification; not self-ratified**). Records the project's **North Star** — *"Rust where appropriate, Mycelium everywhere else"* (pragmatic dogfooding, **not** zero-Rust dogmatism) — and the **function-first** release strategy that follows: **the public release is gated on functional usability and is *version-independent*** — it happens at whatever semantic version fits the work completed (a **`0.x`**, well before `1.0.0`), **not** at any version milestone (§2.8). The **public semantic version tracks the Mycelium-rewrite progress**, climbing `0.x → 1.0.0` **in the open**, where **`1.0.0` ≡ "fully rewritten into Mycelium (where appropriate) and 100% operational."** So: **Phase I** → reach functional usability, then **go public at a `0.x`**; **Phase II** → the progressive Mycelium rewrite continues **publicly**, advancing the version toward `1.0.0`, with **compiler self-hosting a deferred, conditional aspiration**. **For now the version stays `0.0.0`; the concrete semver scheme is deferred until the project is actually ready to publish** (§2.8). Execution doctrine (§2.7): **Fable-class models are reserved solely for planning + complex design; implementation and lighter work run on Opus/Sonnet/Haiku scoped to task complexity.** In doing so it (a) **refines ADR-036 §2.4** (append-only — the public-release gate becomes functional usability, decoupled from any version number, not completed Rust-replacement) and (b) records that **ADR-036 superseded RFC-0031 §5 D1's "compiler stays Rust forever" boundary** (append-only note there). Two ambiguities are **FLAGged for the maintainer, not guessed** (§2.8): the `lang 1.0.0` label collision with ADR-022's functional-completeness milestone (FLAG-V1) and whether `1.0.0` requires compiler self-hosting (FLAG-V2). Becomes binding on **Accepted**; until then ADR-036 §2.4 as written stands. |
| **Decides** | (1) The **North Star**: *Rust where appropriate, Mycelium everywhere else* — dogfooding is pragmatic and progressive, never dogmatic; Rust remains wherever it is the appropriate tool (trusted base, FFI floor, performance-critical kernels) for as long as it is appropriate. (2) **Phase I (→ functional usability → go public at a `0.x`):** make the language **fully functional and usable** — close the **below-grammar enabler gaps** so a Mycelium program can exercise the whole ratified surface + stdlib end-to-end (`myc run`); implement **in Rust where appropriate, in Mycelium where the surface is ready**. The repo is **published/decomposed into public repos when fully usable — at whatever version fits, expected a `0.x`, NOT gated on `1.0.0`** (§2.8); until then it stays **private**, all crates at **version `0.0.0`** with **`publish = false`**. (2a) **Versioning axis (§2.8):** the **public release is version-independent** (gated on usability, not a version number); the **public semver tracks the Mycelium-rewrite progress**, climbing `0.x → 1.0.0` in the open; **`1.0.0` ≡ "fully rewritten into Mycelium (where appropriate) and 100% operational."** **For now the version stays `0.0.0` and the concrete semver scheme is deferred until actually ready to publish.** (3) **Phase II (post-public, progressive):** the full Mycelium-lang rewrite of the remaining corpus proceeds **after** the public flip, **publicly**, module by module under ADR-036 §2.3's differential/replace-on-satisfaction discipline, advancing the version toward `1.0.0`; **compiler self-hosting** happens **only if** it demonstrably improves stability and/or performance, and **only after** the Rust→Mycelium transpiler is fully polished. (4) The **transpiler doctrine**: progressive hardening (prove on small crates, then progressively larger — get-it-right-first-time), the transpiler is an **accelerant, not a gate**; **pre-port polish** (clean ambiguous Rust before porting each crate); **manifest-driven transcoding** only where ROI-positive after correction overhead. (5) The **float enabler route (ii)**: a first-class scalar-float `Repr`, via a future float ADR + a DN-39 promotion review, coordinated with the deferred ADR-030/031 one-way doors so the content-address **rehash happens once**, deferred to the first value-persistence feature. (6) The **execution doctrine**: **Fable-class models are reserved *solely* for planning + complex design** (they do not implement); **implementation and all lighter work run on Opus/Sonnet/Haiku, scoped to the intensity and complexity of the task** (heaviest → Opus, mid → Sonnet, light → Haiku); **full PM prep (user stories + Definition of Done) precedes any implementation agent**. |
| **Refines / supersedes** | **Refines ADR-036 §2.4** (append-only): §2.4's public-release trigger — "dogfooding complete and validated … Rust components actually replaced" — is revised to **functional usability** (Phase I's gate); ADR-036 §2.1–§2.3 (the tag is cut on the Rust reference; dogfooding is a parallel non-tag-gating track; Rust≡Mycelium differential validation + replace-only-on-maintainer-satisfaction) **remain in force unchanged**, and ADR-036's text is untouched except an append-only pointer at §2.4 + a changelog row. **Records** (makes explicit, append-only at RFC-0031 §5 D1) that **ADR-036 supersedes RFC-0031 §5 D1's permanence claim**: D1's "`mycelium-l1` stays Rust forever / a self-hosted compiler is explicitly out of scope" is **lifted as a *forever* boundary** — ADR-036 §2.2 already scopes the *full toolchain* (lexer/parser/checker/elaborator/mono/codegen) for progressive Mycelium reimplementation; this ADR fixes the terms (deferred, conditional, post-public). D1's boundary **remains the operative engineering rule throughout Phase I**. **Does not amend ADR-022** (§5 tracks, §8 Q1, as amended by ADR-024/034/035 — all unchanged; the `lang 1.0.0` **tag** gate is not touched by this ADR). |
| **Grounds** | Maintainer decision (2026-07-01, this session — the locked doctrine recorded verbatim in intent; `Declared` until ratified as this ADR's `Accepted`); **ADR-036** (the tag/release split + dogfooding track this ADR refines at exactly one point, §2.4); **ADR-022** (as amended by ADR-024/034/035 — the unchanged `lang 1.0.0` tag gate Phase I closes) and its §10 long-term vision; **ADR-035** (T4 narrowed to the stable-API freeze + core-lib slice — the precedent of usability-calibrated, reasonable-not-maximal gates this ADR extends to the release itself); **RFC-0031** (§5 D1 the boundary refined here; D3–D6 the migration mechanics unchanged); **DN-27** (the decomposition/public-flip mechanics note — its binding *trigger* now this ADR's Phase-I gate; its *mechanics* ADR stays future work at Phase-II kickoff); **RFC-0033 §7** (the dogfood-gate single-rehash discipline the float route must honor) + **ADR-030/ADR-031** (the deferred content-address one-way doors the float `Repr` coordinates with) + **DN-39** (the default-DENY kernel-promotion bar a scalar-float `Repr` must clear); **DN-34 §8 / M-873** (the transpiler PoC + measured 12.4% union coverage, `Empirical` — the evidence base for "accelerant, not gate"); `docs/spec/stdlib/self-hosting-readiness.md` §0 (the 2026-07-01 surface-sufficiency verification: the surface suffices for the structural majority; the ~5 real blockers are below-grammar — the Phase-I enabler list); `docs/planning/road-to-1.0.0-and-mycelium-rewrite.md` (the umbrella roadmap that sequences this strategy); KC-3, G2, VR-5 (small trusted base; never-silent; no claim above its checked basis). |
| **Date** | 2026-07-01 |

> **Posture (transparency rule / VR-5).** This ADR records *strategy*, authored from the maintainer's
> explicit 2026-07-01 direction; it is **Proposed** and binds only on the maintainer's ratification
> (`Accepted`) — nothing here is self-ratified, and the append-only pointers placed on ADR-036 and
> RFC-0031 say so. It asserts **no implementation progress**: no track closes, no tag moves, no module
> is declared usable or ported by this document. Every forward-looking statement (phase contents,
> orderings, the roadmap's horizons) is `Declared` intent; the cited implementation facts (crate
> versions at `0.0.0`, `Empirical`; transpiler coverage ≈12.4%, `Empirical` per DN-34 §8.5; the
> below-grammar gap list, `Exact`/`Empirical` per the readiness §0 verification) carry their own tags.
> The Phase-I gate is **maintainer-checked**, not self-declared (house rule #6: "done" requires a
> stated, checkable Definition of Done — §5 below).

---

## 1. Context

Three ratified decisions frame the release strategy, and two of them now pull against the maintainer's
refined intent:

- **ADR-022** (as amended by ADR-024/034/035) defines the **`lang 1.0.0` tag**: the Rust reference
  implementation, functionally complete across tracks T1–T9 (self-hosting gating only the core-lib
  slice, §8 Q1). This gate is healthy and **unchanged** here.
- **ADR-036** split the **tag** from the **public release** — a genuine improvement — but fixed the
  public-release gate at *comprehensive dogfooding complete, validated, and the Rust components
  actually replaced* (§2.4). Taken literally, the project stays private until essentially the whole
  toolchain/stdlib is rewritten in Mycelium — a multi-wave rewrite standing between a fully usable
  language and any user ever seeing it.
- **RFC-0031 §5 D1** drew the irreducible-Rust boundary and, for the compiler frontend, stamped it
  **permanent** ("stays Rust forever"; "a self-hosted *compiler* is explicitly out of scope") — which
  ADR-036 §2.2 then contradicted in the forward direction by scoping the *full toolchain* for
  progressive Mycelium reimplementation. The corpus currently holds both statements; the supersession
  was real but implicit, and implicit supersessions are exactly what house rule #3 exists to prevent.

Meanwhile the ground truth moved under the old plan's assumptions: the 2026-07-01 surface-sufficiency
verification (`docs/spec/stdlib/self-hosting-readiness.md` §0) found the language **surface** already
sufficient for the structural majority of the stdlib (~19/26 crates; 8 nodules executing three-way),
with the real blockers a short list of **below-grammar enablers** (float value form, binary integer
arithmetic/signed ops, dense/vsa prim surfacing, `Substrate`/`consume` execution, R2 runtime
vocabulary, plus a string literal and a `hash.*` prim). And the transpiler PoC (M-873, DN-34 §8)
measured ≈12.4% mechanical union coverage — real as an accelerant, nowhere near a substitute for
deliberate porting.

The maintainer's 2026-07-01 direction resolves the tension: **the user-facing gate is usability, not
purity of implementation language.** Dogfooding remains a first-class commitment — but it is
*pragmatic* (Rust where appropriate) and *progressive* (it continues past the public flip), never a
dogma that holds a usable language hostage to a rewrite.

## 2. Decision

### 2.1 North Star — "Rust where appropriate, Mycelium everywhere else"

The project's standing implementation-language rule. Mycelium is used wherever the surface is ready
and the module is expressible; Rust remains wherever it is the *appropriate* tool — the trusted base
(KC-3), the FFI/platform floor, performance-critical kernels, and any module whose Mycelium port has
not yet cleared the RFC-0031 §5 D5 bar. This is **pragmatic dogfooding, not zero-Rust dogmatism**:
"zero Rust" (ADR-022 §10) survives as a *horizon*, not a gate — approached module-by-module under
ADR-036 §2.3's differential/replace-on-satisfaction discipline, for exactly as long as each
replacement is demonstrably at-least-as-good. A Rust component that keeps earning its place keeps it.

### 2.2 Phase I — function-first: reach functional usability, then go public (at whatever version fits)

Phase I's objective is a language that is **fully functional and usable**: a Mycelium *program* can
exercise the **whole ratified surface and the stdlib** end-to-end through the shipped toolchain. The
work is the **below-grammar enabler closure** (the readiness-§0 gap list — sequenced as horizon H1 of
the umbrella roadmap: binary integer arithmetic → dense/vsa prim surfacing → the scalar-float `Repr`
(route (ii), §2.6) → `Substrate`/`consume` execution → an R2-lite runtime-vocabulary subset — plus
`myc run`, a textual string literal, and `hash.*` surfacing), the remaining ADR-022 functional-
completeness tracks, and enough polish that a newcomer can build and run real programs. Implementation
language per §2.1: Rust where appropriate, Mycelium where the surface is ready (the opportunistic
`.myc` ports continue in parallel — they are welcome, they are simply **not the gate**).

**Publication discipline.** The project is **published — and decomposed into public repos (DN-27
mechanics) — when it is fully usable, NOT when it reaches any particular version number** (§2.8: the
public release is version-independent; it happens at whatever semantic version fits the work
completed — expected to be a **`0.x`**, well before `1.0.0`). Until the Phase-I usability gate closes:
the repository stays **private**; every crate stays at **version `0.0.0`**; and **`publish = false`**
is set across the workspace. (`Empirical`, 2026-07-01: all 52 crates are at `0.0.0`; `publish = false`
is currently set on only 3 — the workspace-wide sweep is an H0 hygiene task in the roadmap.) **For now
the version stays `0.0.0`**; the concrete public semantic-versioning scheme is **deferred until the
project is actually ready to publish** (§2.8 — don't design it early). The flip itself (whatever `0.x`
version it lands at) happens together with the DN-27 decomposition and per-repo GHCR publication
(extending ADR-037's registry backend per-repo); the decomposition's binding *mechanics* ADR (per-repo
topology, history, re-export form — DN-27 §5) is authored at that boundary, not here.

### 2.3 Phase II — post-public, progressive: the full Mycelium rewrite

After the flip, the rewrite of the remaining corpus proceeds **progressively** — module by module,
transpiler-accelerated (§2.5), each port differential-validated and replacing its Rust original only
on the maintainer's satisfaction (ADR-036 §2.3, unchanged). This happens **in the open**, and it is
the axis the **public semantic version climbs** (§2.8): `0.x → 1.0.0` as the rewrite lands, reaching
**`1.0.0` only when the project is fully rewritten into Mycelium (where appropriate) and 100%
operational**. There is no calendar deadline; the version *is* the honest progress measure.

**Compiler self-hosting** is inside Phase II and doubly conditioned: it proceeds **only if** it
demonstrably improves stability and/or performance over the Rust frontend (evidence-gated — a
benchmark/defect case the maintainer accepts, not an aesthetic preference), **and only after** the
Rust→Mycelium transpiler is 100% polished (§2.5's progressive hardening complete). Absent that
evidence, the Rust frontend stays — that *is* the North Star applied to the compiler. This is a valid
**aspiration**, not a commitment; RFC-0031 §5 D3's no-circularity staging holds until a bootstrap
protocol is ratified at Phase-II time.

### 2.4 Refinement of ADR-036 §2.4 (append-only) — the release gate is usability, not replacement

ADR-036 §2.4 as written keeps the repository private "until dogfooding is **complete and
validated** … once the Rust components it targets have actually been **replaced**." This ADR
**refines that gate**: the public release is triggered by **Phase-I functional usability** (§2.2,
checked per §5) — at **whatever semantic version fits the work completed (a `0.x`), decoupled from any
`1.0.0` label** (§2.8) — and the Mycelium replacement **continues progressively post-public** (§2.3).
ADR-036 §2.1 (tag on the Rust reference), §2.2 (dogfooding as a parallel, non-tag-gating track), and
§2.3 (the Rust≡Mycelium differential + replace-on-satisfaction validation model) are **unchanged and
remain in force** — only the *release trigger* moves, and it moves to an axis **independent of the
version number**. The public release is no longer coupled to a rewrite-sized wait *or* to a version
milestone: it follows functional usability. Applied append-only: ADR-036's body is untouched except a
dated pointer under §2.4 and a changelog row; this refinement binds when this ADR reaches **Accepted**
(house rule #3 — a Proposed ADR revises nothing by itself).

### 2.5 Transpiler doctrine — progressive hardening; accelerant, not gate

The Rust→Mycelium transpiler (M-873 PoC, DN-34 §8) is developed by **progressive hardening**: prove
it end-to-end on **small crates first**, then progressively larger ones — **get it right first time**
at each size before moving up, rather than sprinting to breadth on a loose mapper. It is an
**accelerant for the ports, never a gate** on any phase: no milestone in either phase waits on
transpiler coverage. Two operating rules:

- **Pre-port polish.** Before porting each crate, **clean the ambiguous Rust first** — resolve the
  idioms the mapper mis-reads (implicit conversions, macro-heavy surfaces, signedness assumptions)
  in the Rust source, then transpile. Correcting the input once beats hand-correcting the output
  every regeneration.
- **Manifest-driven transcoding** (batch/mechanical translation driven by a per-crate manifest) is
  used **only where ROI-positive after correction overhead** — measured against the hand-port
  alternative, per crate, honestly (the DN-34 §8.5 coverage data is the baseline; `Empirical`).

### 2.6 Float enabler — route (ii): first-class scalar-float `Repr`, one rehash

The float gap (no float literal/type/prims — blocks `math`'s f64 half and `numerics`) closes via
**route (ii): a first-class scalar-float `Repr`** — not a Dense-dtype workaround (route (i)) and not
a library-only encoding. Because a new `Repr` variant **enlarges the trusted base and joins
content-address identity**, it is doubly gated: a dedicated **future float ADR** (design: width set,
NaN/rounding semantics, never-silent boundaries) plus a **DN-39 promotion review** (the default-DENY
four-clause bar — a scalar float must *earn* kernel entry). Its content-address impact is
**coordinated with the deferred ADR-030/031 one-way doors** (Dense quant descriptor; VSA element
space) so the identity-set change lands as **one rehash, once** — the RFC-0033 §7 / M-780 single-
rehash discipline, honored unchanged. The rehash itself **defers to the first value-persistence
feature**: until some feature actually persists values, no rehash is spent.

### 2.7 Execution doctrine — Fable plans, Opus/Sonnet/Haiku implement (scoped to complexity)

The standing division of labor for agent-driven execution of this strategy:

- **Fable-class models are reserved *solely* for planning and complex design** — the planning/
  reasoning brain: PM preparation, decomposition, sequencing, architecture, decision-prep for the
  maintainer, and the authoring of governing strategy/roadmap/spec work. **Fable does not do
  implementation or lighter work.**
- **Implementation and all lighter work are done by Opus, Sonnet, and/or Haiku, scoped to the
  intensity and complexity of the task** — the heaviest/most-intricate implementation to **Opus**,
  mid-complexity change-focused work to **Sonnet**, small/mechanical/repetitive work to **Haiku**.
  Pick the smallest model that honestly fits the task (KISS/YAGNI applied to model choice); escalate
  a task that proves harder than scoped rather than forcing it.
- **Full PM prep precedes any implementation agent** (whichever of Opus/Sonnet/Haiku runs it): every
  task carries explicit **user stories** and a **Definition of Done** (house rule #6) before the
  agent is spawned — no implementation agent receives an under-specified task.

This **refines the CLAUDE.md swarm-mode table** (which still governs assignment *within* an
implementation swarm — the Opus/Sonnet/Haiku/Hybrid orchestrator↔leaf modes): the added constraint
is that the **planning tier is Fable-only and never implements**, and the implementation tier is
Opus/Sonnet/Haiku **sized to task complexity**. (Recorded as strategy; the CLAUDE.md wording update
itself is a small follow-up FLAGged for the maintainer, not performed by this ADR.)

### 2.8 Versioning axis — the public release is version-independent; `1.0.0` marks the completed rewrite

The project's **public semantic version is the progress scale of the Mycelium-lang rewrite**, not a
proxy for the public-release event. Three points:

- **Public release is gated on functional usability (§2.2), NOT on any version number.** The project
  goes public at **whatever semantic version fits the work completed** — expected to be a **`0.x`**
  (substantially-Rust, fully functional), **well before `1.0.0`**. "Wait for v1 to go public" is
  rejected: `1.0.0` as a publicity gate is arbitrary and would withhold public value for a label.
- **The public semver climbs `0.x → 1.0.0` in the open, tracking the rewrite.** Each module replaced
  by its validated Mycelium counterpart (ADR-036 §2.3) advances the version along the rewrite scale;
  the progressive rewrite (Phase II) happens **publicly**, not behind the private wall.
- **`1.0.0` ≡ "fully rewritten into Mycelium (where appropriate) and 100% operational."** The `1.0.0`
  release is the completion of the zero-Rust-where-appropriate end state (ADR-022 §10) — reached
  progressively, post-public.

**For now: the version stays `0.0.0`; the concrete semver scheme is deferred (maintainer, this
session).** This section fixes the *principle* — public is version-independent; the number tracks the
rewrite; `1.0.0` = fully-rewritten-and-operational. It deliberately does **not** design the semver
mapping (how rewrite progress maps to `0.x` increments, pre-release tags, per-repo vs unified
versioning). That is **deferred until the project is actually ready to publish** (YAGNI — no early
design of a scheme whose inputs aren't known yet). Until then everything remains `0.0.0` +
`publish = false`.

**FLAG-V1 — the `lang 1.0.0` label collision (maintainer reconciliation needed; not resolved here).**
ADR-022 (Accepted) names its full-language, Rust-reference **functional-completeness milestone**
`lang 1.0.0`, and ADR-023 already frames it as *a release-event, not a crate version*. Under this
section that **event** is exactly the **functional-usability / public-release trigger** (§2.2) — but
its historical **`1.0.0` numeric label collides** with the project semver `1.0.0` reserved here for the
completed rewrite. Proposed mapping (`Declared`, for ratification): ADR-022's milestone keeps its role
as the *functional-completeness event that triggers the public flip at a `0.x`*, and the numeric
`1.0.0` is **reserved for rewrite-completion**. Binding reconciliation — whether ADR-022's event is
re-labeled (e.g. "full-language functional-completeness") via its own append-only amendment — is a
**maintainer decision**; this ADR does not rewrite ADR-022's Accepted text (house rule #3). Until
reconciled, read every "`lang 1.0.0` tag" reference (ADR-022/036, and this ADR/roadmap) as the
**functional-completeness event**, distinct from the project semver `1.0.0` defined here.

**FLAG-V2 — does `1.0.0` require compiler self-hosting?** §2.3 makes compiler self-hosting a
*conditional* aspiration (only if stability/perf-proven). So "fully rewritten … and 100% operational"
is ambiguous at the compiler: is `1.0.0` reachable with the compiler still in Rust (the North Star's
"appropriate Rust"), or does `1.0.0` require the self-hosted compiler? Proposed reading (`Declared`):
`1.0.0` = "everything-appropriate rewritten + operational," with the compiler counted as *appropriate
Rust* unless/until §2.3's condition triggers — so `1.0.0` does **not** require compiler self-hosting.
Confirm at ratification (maintainer decision — surfaced, not guessed; house rule #4).

## 3. Consequences

- **A usable language ships years-of-rewrite sooner.** The public flip no longer waits on the
  comprehensive rewrite; users, feedback, and the public MIT corpus (DN-27's motivation) arrive at
  usability. The cost, honestly stated: the public repo at flip time is **substantially Rust**, and
  is presented as such — pragmatic dogfooding is part of the public story, not something to obscure
  (G2 applies to positioning too).
- **ADR-036 survives intact minus one clause.** Its architecture (tag/release split, parallel
  dogfooding, differential validation, replace-on-satisfaction) is *strengthened* by being freed of
  the one clause that overloaded it; §2.4's original text stays in place with an append-only pointer.
- **RFC-0031 D1 loses its "forever," keeps its force.** Through Phase I nothing changes for any
  D1-row module. The corpus stops holding two contradictory statements about the compiler's future.
- **Compiler self-hosting can now be honestly discussed** — as a conditional, evidence-gated,
  post-public aspiration rather than either a taboo (old D1) or an implied commitment (ADR-036 §2.2
  read maximally).
- **The V-wave deferral bends, minimally.** M-766/M-767 (binary two's-complement arithmetic +
  signedness-split ops) and the float-`Repr` work are pulled forward from the post-1.0-deferred
  RFC-0033 V-wave into Phase I (they are usability enablers); the §7 dogfood-gate/single-rehash
  discipline is honored unchanged (§2.6). The rest of the V-wave stays deferred.
- **Risk — "usable" is softer than "replaced."** A replacement-complete gate is trivially checkable;
  a usability gate needs a real Definition of Done and a maintainer check (§5) to avoid becoming a
  vibe. That is why the gate is maintainer-ratified against explicit criteria, not self-declared.
- **Risk — public before rewrite means public APIs harden earlier.** Post-flip, per-repo SemVer
  (ADR-018) starts constraining the very modules Phase II wants to rewrite. Mitigated by ADR-036
  §2.3's replace-only-when-validated (a Mycelium port is API-compatible by construction — the
  differential enforces it) and by ADR-023's stability carve-outs.
- **The version number becomes an honest progress signal (§2.8).** By reserving `1.0.0` for the
  completed rewrite and letting the public semver climb `0.x → 1.0.0` in the open, the version itself
  communicates how far the dogfooding has actually got — no overloaded "1.0 = done" that hides a
  substantially-Rust reality. Cost: the project goes public at a `0.x`, which some read as "beta";
  that reading is *correct* and honest (G2 applied to the version label), not a defect to paper over.
- **Two flagged ambiguities remain for the maintainer (§2.8 FLAG-V1/V2):** the `lang 1.0.0` label
  collision with ADR-022's functional-completeness milestone, and whether `1.0.0` requires compiler
  self-hosting. Both are surfaced, not resolved — reconciled at ratification, not guessed.

## 4. Alternatives considered

- **Keep ADR-036 §2.4 as written (release = replacement complete).** Rejected by the maintainer:
  it holds a usable language hostage to a rewrite whose value is engineering-internal, and it
  front-loads the least-parallelizable work (compiler-adjacent ports) before any user exists.
- **Zero-Rust as the gate, accelerated by mandatory transpilation.** Rejected: the transpiler's
  measured coverage (≈12.4% union, `Empirical`) makes "transpile everything" a fiction today;
  forcing it would produce volume without correctness — the opposite of get-it-right-first-time.
- **Publish now (pre-usability).** Rejected: publishing a language that cannot run a full program
  end-to-end squanders the first impression the private phase exists to protect; `0.0.0` +
  `publish = false` + private is precisely the honest signal.
- **Unconditional compiler self-hosting in Phase II.** Rejected: self-hosting a compiler is a cost
  center unless it pays in stability/perf; the North Star demands the evidence first.

## 5. Definition of Done

**For this ADR (the decision record):**

- [ ] Maintainer ratification: status `Proposed → Accepted` (the strategy binds; §2.4's refinement of
  ADR-036 takes effect).
- [x] Append-only pointer + changelog row on **ADR-036** (§2.4 refinement recorded; body untouched).
- [x] Append-only note + changelog row on **RFC-0031 §5 D1** (the supersession made explicit; D1
  operative through Phase I).
- [x] Append-only prioritization note on **RFC-0033** (M-766/M-767 + float-`Repr` pull-forward; §7
  discipline unchanged).
- [x] The umbrella roadmap (`docs/planning/road-to-1.0.0-and-mycelium-rewrite.md`) revised to sequence
  this strategy (H0/H1/H2/H2a horizons; Phase II separated) and referencing this ADR as governing.
- [x] Indexed in `docs/adr/README.md` and `docs/Doc-Index.md`; `CHANGELOG.md` records the proposal.
- [ ] `Enacted` only when **both** phase gates below have been reached and checked — never by
  ratification alone (house rule #3).

**For Phase I (the usability gate — the release trigger this ADR defines; maintainer-checked):**

- [ ] The below-grammar enabler set (roadmap H1) is closed: binary integer arithmetic + signed ops;
  dense/vsa prims surfaced; scalar-float `Repr` landed per §2.6 (ADR ratified, DN-39 review passed);
  `Substrate`/`consume` executes; the R2-lite runtime subset is active; textual string literal;
  `hash.*` surfaced — each never-silent, honestly tagged, conformance-tested.
- [ ] `myc run` executes a real multi-nodule Mycelium project end-to-end through the shipped
  toolchain.
- [ ] The `lang 1.0.0` tag gate (ADR-022 §5, as amended) closes on its own criteria.
- [ ] The maintainer ratifies "fully functional + usable" against the above, then the flip executes:
  the public **version is decided at that point (a `0.x`; §2.8 — the scheme was deferred until here)**,
  `publish` flips per the decomposition ADR, DN-27 mechanics run. (The flip is **not** gated on
  reaching any particular version number.)
- [ ] FLAG-V1 / FLAG-V2 (§2.8) resolved by the maintainer — the `lang 1.0.0` label reconciliation and
  whether `1.0.0` requires compiler self-hosting (these can be ratified with the ADR or deferred to
  the publish-time semver decision, but are not left silently open — G4).

**For Phase II / `1.0.0` (the versioning terminal — §2.8):** ports proceed publicly under §2.3/§2.5;
the public semver climbs `0.x → 1.0.0`; compiler self-hosting only on its double condition (§2.3).
**`1.0.0` is reached when the project is fully rewritten into Mycelium (where appropriate) and 100%
operational** (subject to FLAG-V2). Past `1.0.0`, the North Star continues in steady-state
maintenance.

## 6. Grounding / honesty

- Maintainer direction, 2026-07-01 (this session) — the doctrine §2 records; `Declared` until this
  ADR is Accepted, at which point it is the project's ratified strategy.
- ADR-036 — refined at §2.4 only; §2.1–§2.3 load-bearing and unchanged. ADR-022 (+024/034/035) — the
  tag gate, untouched. ADR-035 — the reasonable-not-maximal precedent.
- RFC-0031 §5 — D1 boundary (operative in Phase I; permanence lifted), D3 staging, D5/D6 port
  mechanics (unchanged). DN-27 — flip mechanics, trigger now Phase-I usability.
- RFC-0033 §7 + ADR-030/031 + DN-39 — the float route's gates (§2.6); single-rehash discipline
  honored.
- DN-34 §8 / M-873 (`Empirical`) — transpiler coverage data grounding §2.5's "accelerant, not gate".
- `docs/spec/stdlib/self-hosting-readiness.md` §0 (2026-07-01) — the surface-sufficiency verification
  grounding Phase I's enabler list (`Exact`/`Empirical` per item).
- Crate versions `0.0.0` across the workspace, `publish = false` on 3 of 52 crates (`Empirical`,
  checked 2026-07-01) — §2.2's discipline is mostly-held, with the sweep tracked in the roadmap.
- KC-3, G2, VR-5 — the trusted base grows only through DN-39-style review; nothing silent; no claim
  (including this ADR's own status) above its checked basis.

---

### Changelog

| Date | Status | Note |
|---|---|---|
| 2026-07-01 | **Proposed** (refined, same session, pre-ratification) | **Versioning-axis + execution-doctrine refinement** (maintainer, same-session direction — folded into the still-Proposed ADR before ratification; the authoring trail is recorded here rather than silently overwritten). **(a) Public release decoupled from the version number (§2.8):** the public flip is gated on **functional usability alone** and happens at **whatever semantic version fits (a `0.x`), well before `1.0.0`** — "v1 as a publicity gate is arbitrary." The **public semver tracks the Mycelium-rewrite progress**, climbing `0.x → 1.0.0` **in the open**, with **`1.0.0` ≡ "fully rewritten into Mycelium (where appropriate) and 100% operational."** **For now the version stays `0.0.0`; the concrete semver scheme is deferred until actually ready to publish.** Two ambiguities FLAGged, not guessed: FLAG-V1 (the `lang 1.0.0` label collision with ADR-022's functional-completeness milestone) and FLAG-V2 (whether `1.0.0` requires compiler self-hosting). **(b) Execution doctrine (§2.7):** **Fable-class models reserved *solely* for planning + complex design (never implement); implementation and lighter work on Opus/Sonnet/Haiku scoped to task complexity** (a CLAUDE.md swarm-table refinement, FLAGged as a small follow-up). Held `Proposed` at the maintainer's instruction ("once adapted I'll say it's ratify-ready"). |
| 2026-07-01 | **Proposed** | Authored from the maintainer's 2026-07-01 session directives, for ratification. Records the North Star (*Rust where appropriate, Mycelium everywhere else* — pragmatic, not dogmatic); the function-first two-phase strategy (Phase I → `lang 1.0.0` and the public release gated on **functional usability**, private + `0.0.0` + `publish = false` until then; Phase II → post-public **progressive** rewrite); the append-only **refinement of ADR-036 §2.4** (release trigger: usability, not completed replacement — §2.1–§2.3 unchanged); the explicit record that **ADR-036 superseded RFC-0031 §5 D1's "compiler Rust forever"** (self-hosting now a deferred, conditional, post-public aspiration — evidence-gated on stability/perf, after transpiler polish); the transpiler doctrine (progressive hardening, pre-port polish, manifest transcoding only where ROI-positive; accelerant, not gate); float route (ii) (scalar-float `Repr` via future float ADR + DN-39 review, one rehash coordinated with ADR-030/031, deferred to first value-persistence); and the execution doctrine with mandatory PM prep. Companion roadmap: `docs/planning/road-to-1.0.0-and-mycelium-rewrite.md` (revised same day). |
