# ADR-036 — Dogfooding and Public-Release Strategy: the 1.0.0 Tag Is Rust; the Public Release Is Dogfooded

| Field | Value |
|---|---|
| **ADR** | 036 |
| **Status** | **Accepted** (2026-07-01 — maintainer-ratified). Establishes, as project strategy, that the **`lang 1.0.0` tag** and the project's **public release** are **two distinct milestones**: the tag is cut on the **Rust reference implementation** (self-hosting gates it only at the existing **core-lib self-host slice**, ADR-022 §8 Q1/track T9 — **unchanged, explicitly preserved** by this ADR); **comprehensive dogfooding** — reimplementing the whole toolchain/stdlib/kernel in Mycelium, beside the Rust originals, differential-validated against them — is a first-class **within-1.0.0** track that runs in **parallel** and gates **nothing** beyond that same core-lib slice; the repository stays **private** until dogfooding is complete, validated, and the Rust components it supersedes are **replaced**, at which point the **public** release happens. This is an **additive** strategy decision, not a track-criteria amendment: it does not reopen, narrow, or expand ADR-022 §5's Definition of Done (unlike ADR-024/ADR-034/ADR-035) — it **layers a new, separate release/publicity gate on top** and gives the existing T9/E18-1 self-hosting-capstone track an explicit non-gating status for everything beyond its already-carved-out slice. |
| **Decides** | (1) The `lang 1.0.0` **tag** is cut on the Rust reference implementation; the only self-hosting bar on the tag remains the **core-lib self-host slice** already fixed by ADR-022 §8 Q1 (T4 + the core-lib slice of T9) — this ADR changes nothing about that bar. (2) **Comprehensive dogfooding** (progressively reimplementing *all* of Mycelium — toolchain, stdlib, kernel — *in* Mycelium, alongside the Rust originals) is a first-class **within-1.0.0** track (E18-1's full scope, beyond its core-lib slice) that runs in parallel with, and gates nothing beyond, that slice. (3) The **validation model**: each Mycelium reimplementation is built beside its Rust reference and **differential-validated against it** — the same interp≡AOT≡JIT discipline (RFC-0029 §7.5, the M-210 checker) extended to a **Rust≡Mycelium** axis; a Rust component is **replaced** by its Mycelium counterpart only once fully tested, benched, validated, implemented, and it satisfies the maintainer. (4) **Release/publicity gate:** the repository stays **private** until dogfooding is **complete and validated**; the actual full **public** release happens only then (Rust replaced by Mycelium). The 1.0.0 *tag* (Rust) and the *public release* (dogfooded) are **distinct milestones** — reaching the tag does **not** flip the repo public. |
| **Amends** | **Nothing in ADR-022's Definition of Done (§5) or Q1 resolution (§8).** This is deliberately **not** a T4/T6/T9-style scope amendment (contrast ADR-024/034/035) — ADR-022 §8 Q1's core-lib-slice T9 bar is **preserved verbatim**; this ADR only adds **append-only pointers** at §8 Q1 and §10 clarifying how the (unchanged) tag gate relates to the (new) dogfooding track and the (new) public-release gate. It also **refines the trigger condition** DN-66's sibling design note **DN-27** (Draft, advisory, untouched by this ADR) anticipated for the public-MIT flip: DN-27 described the flip as following "once `lang 1.0.0` is reached, completed, and satisfied"; this ADR makes that condition **conjunctive with dogfooding validation**, not satisfied by the tag alone — DN-27 remains the advisory topology/mechanics note; this ADR is the binding decision DN-27 itself said would come "when `lang 1.0.0` nears" (DN-27 §*Decides*). |
| **Grounds** | Maintainer decision (2026-07-01, verbal ratification recorded here per house rule #6's Definition-of-Done discipline); **ADR-022 §8 Q1** (the existing, unchanged core-lib self-host slice bar this ADR explicitly preserves) and **§10** (the long-term zero-Rust vision + post-1.0.0 public-MIT-flip vision this ADR makes concrete and conditions on dogfooding validation); **ADR-024/ADR-034/ADR-035** (the precedent of focused, scoped, append-only-pointer amending ADRs — this ADR follows the same discipline even though, unlike them, it does not touch §5's criteria text); **DN-27** (the advisory repo-decomposition/public-release note this ADR's release-gate decision now binds against); **E18-1** (the self-hosting-capstone epic whose full scope — beyond the core-lib slice — is this ADR's dogfooding track; its own Definition of Done already names a "Rust-host == self-host == AOT" three-way differential as one bootstrap-protocol option, which this ADR elevates to the whole-project validation model); **RFC-0029 §7.5** + the **M-210** shared differential checker (the interp≡AOT≡JIT discipline this ADR extends to a Rust≡Mycelium axis); **docs/planning/self-hosting-port-ledger.md** (the living roadmap this track already tracks against); KC-3 (small auditable kernel — the trusted base does not change: the interpreter/Rust reference stays the reference *during* dogfooding, exactly as the AOT/JIT paths stay outside the kernel today), G2/VR-5 (never-silent, honestly-tagged: no Mycelium reimplementation replaces its Rust counterpart on an unchecked basis; no status is upgraded past what a differential actually shows). |
| **Date** | 2026-07-01 |

> **Posture (transparency rule / VR-5).** This ADR records *strategy*, maintainer-ratified; it asserts no
> implementation progress and declares no track "done." It does not move E18-1, T9, or any RFC/ADR to
> `Enacted`. The core-lib self-host slice (ADR-022 §8 Q1) is **unchanged and stays the only self-hosting
> bar on the `lang 1.0.0` tag** — this ADR does not loosen or tighten it. The comprehensive-dogfooding
> track and the public-release gate this ADR establishes are both **forward-looking commitments**: neither
> is met today (the reference implementation is still overwhelmingly Rust — DN-66's per-crate survey
> found zero of 26 `mycelium-std-*` crates cleared even the narrower RFC-0031 D5/D6 bar; E18-1's own
> children, M-739…M-742, remain `status:needs-design`). This ADR is honest about that: it commits to a
> **process** (parallel build, differential validation, replace-on-satisfaction, private-until-validated),
> not to a date or a completion claim.

---

## 1. Why this decision exists

Three things were previously true but not tied together in one place:

- **ADR-022 §8 Q1** already resolved that the `lang 1.0.0` tag requires only the **core stdlib/corelib
  self-host slice** (with T4) — "enough that the language is properly usable without a developer ever
  dropping to hand-written L0/L1" — while the **full toolchain/compiler self-host** (E18-1's remaining
  children, M-739…M-742) explicitly **trails to the long-term arc** (§10). This is the existing,
  unchanged tag gate.
- **DN-27** (Draft, advisory) separately sketched a **post-1.0.0** vision: once `lang 1.0.0` is reached,
  the monorepo decomposes into component + phylum re-export repos and the project "flips to a full set
  of public, MIT-licensed repos" — but left the **binding trigger** for that flip to "a future ADR
  drafted when `lang 1.0.0` nears" (DN-27 *Decides*).
- **E18-1**'s own Definition of Done already names a **"Rust-host == self-host == AOT" three-way
  differential** as one candidate bootstrap protocol for the compiler frontend specifically — but this
  was scoped to the frontend port, not stated as the project's general replace-in-place discipline.

The maintainer's 2026-07-01 decision draws these together and settles the question DN-27 deferred: the
public release is **not** simply "whatever is on `main` when the `lang 1.0.0` tag lands." It is a
**separate, later milestone**, reached only once the comprehensive Mycelium-native reimplementation —
built beside the Rust reference and differentially validated against it — is complete, validated, and
has actually **replaced** the Rust components it targets. Until then the repository stays private. This
gives the project two honest, distinct waypoints instead of one overloaded one: a **tag** (Rust,
functionally complete, the existing gate) and a **release** (dogfooded, public).

## 2. The decision

### 2.1 The `lang 1.0.0` tag is cut on the Rust reference implementation

The tag (ADR-022 §5, `M-738`) is reached when the existing tracks T1–T9 close on their **current**
criteria — including the amendments already layered on by ADR-024 (T1/E19-1), ADR-034 (T6/E15-1 full
native coverage), and ADR-035 (T4 narrowed to the stable-API freeze + core-lib slice). **Self-hosting
does not additionally gate the tag beyond what ADR-022 §8 Q1 already requires** — the core-lib
self-host slice (T4 + the core-lib slice of T9, M-714…M-718). This ADR changes **none** of that; §8 Q1's
resolution text is preserved verbatim (house rule #3), and this ADR is explicit that it does **not**
reopen it.

### 2.2 Comprehensive dogfooding is a first-class, non-gating, within-1.0.0 parallel track

Beyond the core-lib slice, **all** of Mycelium — the full toolchain (lexer/parser/checker/elaborator/
mono/codegen), the full stdlib (all 26 `mycelium-std-*` crates), and everything else that is not the
bare trusted-base kernel — is progressively reimplemented **in Mycelium itself**, built **alongside**
(not instead of, and not blocking) the Rust originals. This is E18-1's full scope (M-739…M-742 plus
whatever further children the comprehensive rewrite requires) and the zero-Rust vision ADR-022 §10
already named. What this ADR adds is the explicit framing: this is **within-1.0.0** work — it starts
now, runs in parallel with the tag-gating tracks, and is tracked with the same rigor — but it **gates
nothing** beyond the slice §8 Q1 already carved out. Progress on it neither blocks nor is blocked by the
tag.

### 2.3 Validation model: Rust≡Mycelium differential, replace only once it satisfies the maintainer

Each Mycelium reimplementation is developed **beside** its Rust reference, never as a blind rewrite, and
is **differential-validated against it** before it is trusted: this extends the project's existing
interp≡AOT≡JIT three-way differential discipline (RFC-0029 §7.5, the shared **M-210** checker) to a
**Rust≡Mycelium** axis — the same never-silent, honestly-tagged bar (G2/VR-5) the AOT/JIT paths already
meet. A Rust component is **replaced** by its Mycelium counterpart only once it is fully **tested,
benched, validated, and implemented**, and it **satisfies the maintainer** — never on an unchecked or
merely-plausible basis. Until replacement, the Rust component **remains the reference and the trusted
base** (KC-3 unchanged); the Mycelium counterpart is the candidate being proven, exactly as the AOT path
is validated against, and does not supersede, the interpreter today.

### 2.4 Release/publicity gate: private until dogfooding is complete and validated; public only then

The repository **stays private** through the `lang 1.0.0` tag and through however much of the
comprehensive-dogfooding track remains open at that point. The **public** release — the "flip to a full
set of public, MIT-licensed repos" DN-27 sketched the mechanics for — happens only **after** dogfooding
is complete and validated per §2.3, i.e. once the Rust components it targets have actually been
**replaced** by their validated Mycelium counterparts. The `lang 1.0.0` **tag** and the **public
release** are therefore **distinct milestones on different clocks**: the tag can land (and is expected
to land) well before the public release, since the tag does not require comprehensive dogfooding (§2.1)
while the public release explicitly does (§2.4).

## 3. Consequences

- **ADR-022 §5/§8 Q1 are unaffected.** No track's Definition of Done changes; the tag gate stays exactly
  what ADR-024/034/035 already left it. §8 Q1 and §10 each carry an append-only "see ADR-036" pointer
  recording this ADR's relationship to them (§4 below) — their own resolution/vision text is **not**
  rewritten.
- **E18-1 gets an explicit, honest non-gating framing for its full scope.** Its core-lib slice
  (shared with T4) remains the only tag-gating piece (unchanged, per §8 Q1); its remaining children
  (M-739…M-742) and whatever further comprehensive-rewrite work follows are now explicitly named as the
  **dogfooding track** this ADR establishes — real, tracked, within-1.0.0, parallel, but not
  tag-gating. This is a clarifying, non-status-changing note (E18-1 stays `status:needs-design`); it
  does not claim any of that work done.
- **A new release milestone exists that ADR-022 does not itself define.** ADR-022 (as amended) defines
  the `lang 1.0.0` **tag**. This ADR defines the **public release** as a later, separate act, gated on
  dogfooding completion + validation (§2.4) — refining DN-27's anticipated trigger from "once `lang
  1.0.0` is reached" to "once `lang 1.0.0` is reached **and** dogfooding is validated." DN-27 itself
  stays `Draft`/advisory and untouched; a future act may formally fold this condition into DN-27 or
  supersede it, but that is not performed here.
- **The trusted base does not move.** Exactly as with the AOT/JIT paths (ADR-034) and the stdlib freeze
  (ADR-035), the interpreter/Rust reference remains authoritative until a Mycelium counterpart is
  differentially validated and the maintainer accepts the replacement (KC-3 untouched).
- **No timeline commitment is made.** This ADR commits to a **process** — parallel build, differential
  validation, replace-on-satisfaction, private-until-validated — not to a date. Per VR-5, no claim here
  should be read as "dogfooding will be done by X"; it is `Declared` intent, not a scheduled fact.

## 4. Cross-reference application (append-only; no rewrites)

- **ADR-022 §8 Q1** — append-only pointer added: "see ADR-036" — the full self-hosting-beyond-the-slice
  track is confirmed as the non-tag-gating, within-1.0.0 dogfooding track; Q1's own resolution text is
  unchanged.
- **ADR-022 §10** — append-only pointer added: the zero-Rust end state and the post-1.0.0 public-MIT-flip
  vision are the same track/gate ADR-036 now governs the timing of (dogfooding-validated, not
  tag-triggered); §10's own vision text is unchanged.
- **E18-1** (`tools/github/issues.yaml`) — append-only body note: E18-1 is the dogfooding-capstone track
  (comprehensive self-hosting beyond the core-lib slice), non-tag-gating except that slice (T9, §8 Q1);
  roadmapped by `docs/planning/self-hosting-port-ledger.md`. No status change.
- **`docs/planning/self-hosting-port-ledger.md`** — header note added: this ledger is the roadmap for the
  ADR-036 comprehensive-dogfooding track (build beside the Rust reference → Rust≡Mycelium differential
  validation → replace → public release).
- **ADR-034, ADR-035** — noted here as the amending-pattern lineage this ADR follows procedurally
  (focused ADR, append-only pointers) even though, unlike them, it does not amend ADR-022's §5 criteria.
- Indexed in `docs/adr/README.md` and `docs/Doc-Index.md` (doc-currency index-coverage requires it).

## 5. Rationale & alternatives considered

**Chosen:** decouple the tag from comprehensive dogfooding entirely, and gate **public visibility**
(not the tag) on dogfooding's completion and differential validation. This lets the Rust
reference-implementation track land on its own, already-defined schedule (§8 Q1 preserved) while giving
the zero-Rust vision (§10) a concrete, honest trigger for when the project actually goes public — rather
than leaving that trigger as DN-27's open "a future ADR will decide this."

**Alternative A (not taken): gate the tag itself on full self-hosting.** This would make T9 (E18-1's
complete scope) a hard 1.0.0 blocker, reversing the reasonable-not-maximal calculus §8 Q1 already
applied (the same calculus ADR-035 just extended to T4). Rejected: it would hold a functionally-complete
Rust 1.0.0 hostage to a much larger, longer-running rewrite, with no correctness benefit the core-lib
slice doesn't already deliver.

**Alternative B (not taken): make the repo public at the `lang 1.0.0` tag** (the reading DN-27's
"once reached" phrasing invited). Rejected: the maintainer's stated preference is to complete and
validate the from-scratch Mycelium reimplementation **before** exposing the project publicly, so the
public-facing artifact is the dogfooded one, not a Rust codebase presented as if the rewrite were
incidental.

**Alternative C (not taken): sequence dogfooding after the tag, not parallel with it.** Rejected: running
dogfooding **beside** the Rust work (rather than after it) lets the two informative differentials
(Rust≡Mycelium here; interp≡AOT≡JIT already in force) develop together, and avoids idling the
comprehensive-rewrite effort until the tag lands.

## 6. Definition of Done

- [x] ADR-022 §8 Q1 carries an append-only "see ADR-036" pointer; §8 Q1's own resolution text is
  unchanged (verified by diff — only a pointer line added).
- [x] ADR-022 §10 carries an append-only "see ADR-036" pointer on the public-release trigger; §10's own
  vision text is unchanged.
- [x] E18-1's issue body (`tools/github/issues.yaml`) carries an append-only, non-status-changing note
  framing it as the dogfooding-capstone track, roadmapped by the port ledger.
- [x] `docs/planning/self-hosting-port-ledger.md` carries a header note identifying it as this track's
  roadmap.
- [x] This ADR is indexed in `docs/adr/README.md` (status line + table) and `docs/Doc-Index.md`
  (doc-currency index-coverage).
- [x] `CHANGELOG.md` `[Unreleased]` records the ratification (append-only).
- [x] `python3 tools/github/manifest-check.py`, YAML load, `scripts/checks/markdown.sh`,
  `scripts/checks/links.sh`, `python3 tools/github/doc_refs_check.py`, and
  `python3 scripts/doc_currency.py` all pass clean on the touched files.
- [x] This ADR reaches **Accepted** (maintainer-ratified) — recorded here.
- **Enacted** only once the **public release** milestone itself is reached (§2.4 — dogfooding complete,
  validated, Rust components replaced, repo flips public) — **not** by this ADR's ratification alone
  (house rule #3: Accepted steps to Enacted only on a checked basis; that basis does not exist yet).

## 7. Grounding / honesty

- Maintainer decision (2026-07-01) — the four-part strategy this ADR records (§2).
- ADR-022 §8 Q1 — the existing, **unchanged** core-lib self-host slice bar this ADR explicitly preserves
  and does not reopen.
- ADR-022 §10 — the zero-Rust long-term vision + post-1.0.0 public-MIT-flip vision this ADR concretizes
  with a checked trigger condition.
- DN-27 — the advisory repo-decomposition/public-release note whose anticipated "future ADR... when
  `lang 1.0.0` nears" is this ADR; DN-27 itself stays `Draft`, untouched.
- E18-1 — the epic whose full scope (beyond the core-lib slice) is the dogfooding track named here; its
  own DoD's "Rust-host == self-host == AOT" differential option is the seed this ADR generalizes.
- ADR-024, ADR-034, ADR-035 — the precedent of focused, append-only-pointer amending ADRs (procedural
  lineage only; this ADR does not itself amend §5 criteria).
- RFC-0029 §7.5, the M-210 shared checker — the interp≡AOT≡JIT discipline extended here to Rust≡Mycelium.
- KC-3, G2, VR-5 — the trusted base is unchanged; no replacement happens on an unchecked basis; no
  status/tag/track is upgraded past what a differential actually shows; no date is asserted.

---

### Changelog

| Date | Status | Note |
|---|---|---|
| 2026-07-01 | **Accepted** | Maintainer-ratified project-strategy decision: the `lang 1.0.0` **tag** is cut on the Rust reference implementation (self-hosting gates it only at the existing core-lib slice, **ADR-022 §8 Q1, unchanged**); **comprehensive dogfooding** (rewriting all of Mycelium in Mycelium, beside the Rust originals) is a first-class **within-1.0.0**, non-tag-gating, parallel track (E18-1's full scope); each reimplementation is **differential-validated** against its Rust reference (extending the interp≡AOT≡JIT discipline, RFC-0029 §7.5/M-210, to a Rust≡Mycelium axis) and **replaces** the Rust original only once tested/benched/validated and maintainer-satisfied; the repository **stays private** until dogfooding is complete and validated, with the **public release** a distinct, later milestone from the tag. Does **not** amend ADR-022 §5's Definition of Done or §8 Q1's resolution (both preserved verbatim, append-only pointers only) — contrast ADR-024/034/035, which did amend track criteria. Refines DN-27's anticipated public-release trigger (DN-27 stays Draft, untouched). E18-1 and the self-hosting port ledger carry append-only, non-status-changing framing notes. |
