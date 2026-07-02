# Design Note DN-71 — `Substrate`/`consume` Execution Model: Confirm-Memo (M-901)

| Field | Value |
|---|---|
| **Note** | DN-71 |
| **Status** | **Accepted** (2026-07-02 — **maintainer sign-off recorded**: Model S (§4/§6) is approved. Authored 2026-07-02 by the M-901 leaf as **Draft**, a confirm-memo pending sign-off; that history stands unchanged — this line records the forward transition, append-only, house rule #3. The Gap-E implementation lane M-902…M-904 unblocks) |
| **Decides** | Nothing normatively — it **proposes** the confirmed execution model for the affine `Substrate`/`consume` construct (value form, use-once enforcement, lowering, drop protocol), enumerates alternatives with an honest tradeoff table, records a recommendation, and lists every open decision as a FLAG for the maintainer. The maintainer's sign-off (accept / revise / flag-blocking, §9) is what confirms it. |
| **Feeds** | M-902 (`Substrate` v0 value form) · M-903 (affine tracker) · M-904 (lift the staged `Residual`) — the `enb` Gap-E lane; the future `grm` M-918 DN-54 dossier (§7 cross-check); RFC-0027 OQ-5 (drop-without-consume protocol, FLAG-4) |
| **Depends on** | DN-54 §10 (M-824 derive-site design-pass — the pattern + its Model-A commitments, §3); DN-03 §1 + DN-02 §2 (the `consume`/`substrate` lexicon rulings); RFC-0006 LR-8/LR-9 (affine external-resource kind; leaks structurally excluded); DN-33 §8.1 Q4 (maintainer-ratified: `substrate`/`consume` uniqueness subsumes into the MEM-4 static-uniqueness mechanism); DN-59 + RFC-0027 OQ-5 (reclamation; drop-without-consume deferred); `docs/spec/stdlib/io.md` + `fs.md` (the consuming specs); `docs/spec/stdlib/self-hosting-readiness.md` §0 item 5 (the blocker this lane closes); M-664 (the landed `consume` surface) |
| **Date** | 2026-07-02 |
| **Task** | M-901 (epic E28-1, kickoff `enb`, Phase-I H1 Gap E), branch `claude/leaf/E28-1-M901-consume-model-memo` |

> **Posture (transparency rule / VR-5 / G2).** This memo is a design pass, not an enactment. Claims
> about what the code and corpus **already do** are `Exact` (source-read, cited to file/section).
> Claims about what the model **should be** are `Declared` (design intent, awaiting maintainer
> ratification). Nothing is tagged `Proven` — no side-condition-checked theorem exists for any claim
> here. Where the corpus is ambiguous or two documents pull in different directions, the ambiguity is
> FLAGged (§8), never resolved by silent choice (G2 — flag, don't guess).
>
> **Maintainer sign-off (append-only, 2026-07-02).** The maintainer **accepts** Model S (§4,
> recommended §6): the interpreter-level opaque affine `Substrate` handle (no new L0 node, no
> `Repr` growth), static use-once enforcement in the L1 checker with a never-silent runtime
> consumed-flag backstop, and `consume` lowering as the observational-identity move through
> existing paths. The gate that held M-902…M-904 at `status:blocked` (§1) is now cleared; all
> three move to `status:todo` (tools/github/issues.yaml). The individual FLAG-1…FLAG-9
> dispositions (§8) are **not itemized** in this sign-off and remain for the integrator/maintainer
> to record explicitly before any FLAG-dependent implementation choice is made (G2 — not guessed).

---

## 1. Purpose — the Gap-E design gate

`docs/spec/stdlib/self-hosting-readiness.md` §0 item 5 (`Exact`): *"`Substrate`/`consume` execution
staged (elaborates to a never-silent `Residual`; no v0 value form) → blocks `fs`/`io`'s
affine-handle model."* The `enb` kickoff's Gap E closes this with three implementation tasks —
M-902 (value form), M-903 (affine tracker), M-904 (lift the `Residual`) — all gated on this memo:
the maintainer confirms the execution model **before** code, so the affine semantics are ratified,
not improvised mid-implementation (the M-901 user story; maintainer gate #3 in the `enb`
prerequisites).

This memo delivers the four things M-901's Definition of Done names: the model (§4), alternatives
plus a recommendation (§5–§6), the cross-check against `grm` M-918 (§7), and the FLAG list (§8).

## 2. What exists today (`Exact` — source-read)

1. **Type + type rule landed (M-664).** `Ty::Substrate(tag)` exists in the L1 checker
   (`crates/mycelium-l1/src/checkty.rs`, `Substrate{tag} — the affine external-resource kind
   (LR-8). No value forms exist in v0`). `check_consume` enforces the type rule: the operand of
   `consume <expr>` must have type `Substrate{tag}`; any other operand type is an explicit refusal;
   the result is the moved `Substrate{tag}`. The doc-comment is explicit that v0 has **no
   value-level affine-usage tracker** (only pattern-binder linearity, `check_linear`), so the
   single-use property is currently **asserted by the construct, not checked** — guarantee
   `Declared`, honestly recorded in `crates/mycelium-l1/src/grade.rs` (where `consume` is a
   grade-transparent move: it neither upgrades nor downgrades the operand's tag).
2. **Execution staged as a never-silent `Residual`.** `crates/mycelium-l1/src/elab.rs`:
   `Expr::Consume(_)` elaborates to an explicit `Residual` — *"`Substrate` has no v0
   value/representation lowering (LR-8; DN-03 §1; M-664)"* — and `BaseType::Substrate` is refused
   as *"not a representation type"*. Every `Substrate` site refuses identically (G2/VR-5).
3. **Lexicon + reference semantics ratified.** DN-03 §1 (Resolved): `consume` = *"acquire and take
   exclusive ownership of an affine `substrate`"*; its T-map is *"a fungus consumes substrate
   exactly once = affinity."* `docs/reference/language-reference.md` §Substrate: *"an affine
   external resource (LR-8) — consumed exactly once … Misuse (double-consume) is a checker refusal,
   never silent."* RFC-0006 LR-8 reserves affinity **for external resources only** (not a general
   ownership system); LR-9 commits that *"the only leak vector — an unreleased external resource —
   is closed by the affine `Resource` kind."*
4. **The consuming specs assume static enforcement.** `docs/spec/stdlib/io.md` (`Declared`, spec):
   the io half moves bytes against *"an affine `substrate` handle consumed exactly once (LR-8) …
   single-consumption is in the type, not a convention"*; a chunked `read` returns the handle so
   re-reading a spent handle is *"a type error, not a runtime check."* `fs.md` layers filesystem
   access on the same handles.
5. **A ratified static-analysis home already exists.** DN-33 §8.1 Q4 (Accepted — maintainer-ratified
   defaults): `substrate`/`consume` uniqueness **subsumes into** the MEM-4 static-uniqueness elision
   mechanism, *"not a parallel path (DRY)"* — *"the borrow analysis treats a known-affine binding as
   owned-unique."* DN-59 confirms the same subsumption (its §"increment staging") and carries
   RFC-0027 **OQ-5** — the drop-without-consume protocol — as explicitly deferred (*"runtime error,
   silent no-op, or explicit EXPLAIN event"* — dependent on the future `graft` RFC).
6. **Tracking.** M-918 (`grm` kickoff, the DN-54 derive-site consumption-model dossier) is a
   **reserved, unminted slot**: `tools/github/issues.yaml` contains no `id: M-918` entry as of
   2026-07-02 (`Exact`, time-indexed — verified by grep); it exists only as a proposed row in
   `.claude/kickoffs/grm.md`. See §7.

## 3. The stated target — DN-54 §10 — and what it does and does not contain

M-901's issue body directs: *"Confirm the execution model: the DN-54 §10 derive-site `consume`
design-pass (M-824) as the target."* Read carefully, DN-54 §10 is the **derive-site attachment**
design pass — how a `derive Name for T`'s generated L0 attaches to and is *consumed by* the
surrounding program (Models A/B, the §10.2 criteria, the Model-A recommendation). It contains **no
affine-`Substrate` content**: a grep of DN-54 for `consume`/`Substrate`/`affine` finds only
incidental uses of the word "substrate" in the Unison content-addressing sense (`Exact`). The word
"consume" collides across two distinct constructs:

1. **`consume <expr>`** — the affine `Substrate` acquisition expression (DN-03 §1 / LR-8 / M-664) —
   *this memo's subject* (Gap E).
2. **derive-site consumption** — DN-54 §10's attachment question — *`grm` M-918's subject*.

This memo therefore interprets "DN-54 §10 as the target" as: **the Gap-E execution model must be
built to the DN-54 §10 design-pass pattern and its Model-A commitments** — reuse existing
elaboration/checking paths rather than minting new machinery; **no new L0 node** (§10.2 criterion 2,
KC-3); never-silent residuals and refusals (criterion 4); results reachable/inspectable (criteria 1
and 5); and a maintainer-ratified design pass before enactment (§10.7). That interpretation is
**FLAG-1** (§8) — it is the coherent reading, but it is the memo author's reading, not the
maintainer's confirmed intent (`Declared`; G2 — surfaced, not assumed).

**Memo location (FLAG-2).** The task allowed either a new DN or a DN-54 addendum. A **new note
(DN-71)** was minted, following the established convention for design dossiers/memos (DN-65 memo,
DN-68 invariant note, DN-69 promotion dossier — all new dated notes; DN-54's own §10.7 routes its
follow-up work to a separate implementing RFC, not to further DN-54 growth). Appending an
affine-`Substrate` section to DN-54 would also have hard-coded the FLAG-1 conflation into the
corpus. FLAGged for confirmation.

## 4. The proposed confirmed model — Model S (static-affine, existing-paths, no new L0 node)

All of §4 is `Declared` (design for ratification). Each component names the implementing task.

### 4.1 Value form (M-902) — an interpreter-level opaque affine handle

A `Substrate{tag}` value is an **opaque, runtime-only handle**: a value-world citizen of the
evaluators (the L1 evaluator and `mycelium-interp` share the value world), carrying its `tag`, an
opaque host handle identity, and a `consumed` state bit (the §4.2 backstop). It is:

- **not a kernel node and not a `Repr`** — no L0 `Node` growth, no `Repr` growth (KC-3; DN-54 §10.2
  criterion 2; the DN-39 default-DENY promotion bar — compare ADR-040/DN-69, where even floats
  required a full dossier for kernel entry). `elab.rs` already states the ground truth: `Substrate`
  *"is not a representation type"* — it names an external resource, not a value representation
  (LR-8).
- **not content-addressed data** (ADR-003 identity is for values; a handle names an external
  resource whose identity is *not* its content — two opens of the same file are two handles).
- **creatable only through the acquiring surface** — the quarantined `@std-sys` `wild` FFI floor
  today (RFC-0031 D1), and `graft(cap) -> Result<Substrate, GraftErr>` when R2 activates
  (`runtime.md` §API). There is **no literal** and no constructor in safe surface code.
- **inspectable, never a black box**: the tag and acquisition provenance are EXPLAIN-visible; the
  handle contents stay opaque (they are the host's, not the value world's).

Invalid states are unrepresentable or explicit errors (the M-902 DoD): the only states are
*live* and *consumed*, and the only transition is §4.2's move.

### 4.2 Use-once enforcement (M-903) — static affine checking, with a never-silent runtime backstop

**Primary: a static affine-usage pass in the L1 checker.** Every binding of type `Substrate{tag}`
is **moved at most once along any control path**. A "use" is any move: a `consume`, an argument
pass, a return, a constructor/field capture, a channel send. A second use on any path is a
compile-time refusal — a structured RFC-0013 diagnostic naming **both** the first-move site and the
violating site (the M-903 DoD: *"diagnostics name the violation site"*). Grounding for choosing
static as the target rather than an option:

- `io.md` (spec, `Declared`): *"single-consumption is in the type, not a convention"* — re-use of a
  spent handle is *"a type error, not a runtime check."*
- `language-reference.md` (`Declared`): *"Misuse (double-consume) is a checker refusal, never
  silent."*
- DN-33 §8.1 Q4 (**maintainer-ratified**): affine-binding uniqueness subsumes into the same static
  mechanism family MEM-4 uses — a known-affine binding is owned-unique. The M-903 pass is the
  refusal-side twin of that ruling, built on the checker's existing linearity machinery
  (`check_linear` precedent), not a parallel analysis (DRY/KC-3).

**Backstop: a runtime consumed-flag check.** The §4.1 value carries a `consumed` bit; the evaluators
refuse (an explicit `Report`, never silent) any move of an already-consumed handle. Under a correct
static pass this backstop is unreachable from checked code; a tripped backstop is therefore an
**internal invariant break surfaced loudly** (G2) — the same defensive posture as `elab.rs`'s
unreachable-`Lambda` arm. It is also the property-test surface for M-903's DoD (*"no path consumes
twice undetected"*): the property drives generated programs through both enforcement layers and
asserts no double-consume escapes both.

**Leak side (drop-without-consume):** *not* re-decided here — it is RFC-0027 **OQ-5**, explicitly
deferred by DN-59 pending the `graft` RFC. §8 FLAG-4 records the recommended v0 posture
(deterministic scope-exit release with a recorded reclamation/EXPLAIN event; a **silent no-op is
G2-excluded** in any case) and asks the maintainer to either ratify that v0 posture or keep OQ-5
fully open with M-902 shipping the narrower "live/consumed" states only.

### 4.3 Execution (M-904) — `consume` lowers through existing paths; the `Residual` is lifted

`consume <expr>` at L0 is **the move itself — observationally the operand value** (its affine
obligation having been discharged statically at L1 per §4.2, and grade-transparently per
`grade.rs`). No new L0 node; no new prim is *required*. The handle value is created by the
acquiring host-call (the existing `wild` host-call form) and flows through existing nodes as an
opaque evaluator value. Concretely, M-904 replaces the `Expr::Consume` residual arm in `elab.rs`
with elaboration of the operand — after which the Gap-E fragment's elab `Residual` is gone, per the
M-904 DoD. Two shape decisions ride on FLAGs:

- **FLAG-7** — whether `consume` should instead lower to an existing-node **`Op` prim**
  (`substrate.consume`) so the consumption is a first-class runtime event in the EXPLAIN/provenance
  record (per-op provenance is a house capability). The identity-move form is recommended
  (§6) as the KISS/KC-3 default; the prim form is the alternative if the maintainer wants
  consumption events reified at runtime rather than derivable from the static structure.
- **FLAG-8** — the AOT path: `Substrate` is not AOT-lowerable in v0; the MLIR path's refusal stays
  **explicit and recorded** (the M-904 DoD's sanctioned narrowing: *"gone, or explicitly narrowed
  and recorded"*). Three-way conformance closes over the two interpreters; the AOT refusal is a
  recorded decision, not a silent gap.

Reject-case conformance (double-consume, consume-of-non-Substrate, consume-at-item-position) stays
and grows per M-903/M-904's DoD rows; `reject/18-consume-not-an-item.myc` already pins the
item-position refusal (`Exact`).

### 4.4 Guarantee posture after landing

The type rule stays `Exact` (it is checked today). The use-once property moves from `Declared`
(asserted) to **`Empirical`** once the M-903 property test and reject-conformance corpus exercise
it — **not `Proven`**: no mechanized soundness proof of the affine pass is proposed for v0 (VR-5;
the upgrade path, if ever wanted, is a Phase-3-style mechanization like DN-33 §8.1 Q3's). The
`grade.rs` move-transparency of `consume` is unchanged.

## 5. Alternatives considered (`Declared`; honest tradeoff table)

| Criterion | **Model S — static affine + runtime backstop** (§4) | Model D — dynamic-only tracker | Model K — kernel value form (new L0 node / `Repr`) | Model E — encode handles as existing data (e.g. `Binary` ids) |
|---|---|---|---|---|
| KC-3 / kernel growth | None — checker pass + evaluator value | None | **New L0 node or `Repr` arm — default-DENY** (DN-39); would need an ADR-040-class dossier | None |
| Double-consume diagnostic | Compile-time, names both sites; runtime backstop as invariant guard | Runtime only — fails late, per-execution-path | Same open question, plus kernel surface | **Not refusable** — a handle becomes copyable data; misuse is silent (G2 violation) |
| Fit with the ratified corpus | Matches `io.md` ("a type error, not a runtime check"), `language-reference.md` ("checker refusal"), DN-33 Q4 (subsume into static uniqueness) | **Contradicts** `io.md` + `language-reference.md`'s stated semantics | LR-8 says external-resource *kind*, not a representation; `elab.rs` already refuses it as a repr type | Contradicts LR-8 (affinity is *in the type*) |
| Machinery cost | Affine pass on existing linearity machinery; consumed-bit; property test | Consumed-bit + tests only (cheapest) | Kernel node + `Canon`/content-address story + AOT story (largest) | Cheapest, and wrong |
| Leak vector (LR-9) | Rides FLAG-4 (recorded deterministic release); structurally visible either way | Same FLAG-4 dependency, but leaks also only detectable at runtime | Same | Leaks invisible (a "handle" is just data) |
| Staging risk | Moderate — the static pass is the one genuinely new analysis | Low | High | Low |

**Why the losers lose:** Model E is a silent-misuse machine — rejected outright (G2). Model K
buys nothing the evaluator-level value doesn't already provide and spends kernel-growth budget the
DN-39 bar exists to protect — rejected (KC-3; the ADR-033 precedent shows kernel growth is a
deliberate, maintainer-ratified, one-variant-at-a-time affair). Model D is coherent and cheap but
**contradicts the already-written semantics** of `io.md` and the language reference, and forgoes the
maintainer-ratified DN-33 Q4 subsumption; adopting it as the *target* would be a silent downgrade of
two `Declared` spec commitments. Model D **is** acceptable as an explicitly-recorded *staging step*
(land the backstop + reject conformance first, the static pass immediately after) if the maintainer
prefers to de-risk M-903 — that option is FLAG-3, never a silent fallback.

## 6. Recommendation (`Declared` — for maintainer ratification)

**Adopt Model S**: an interpreter-level opaque affine handle (no new L0 node, no `Repr` growth,
creation only via the acquiring `wild`/`graft` surface); **static** use-once enforcement in the L1
checker with structured both-sites diagnostics; a never-silent runtime consumed-flag backstop as the
property-test surface; `consume` lowering as the observational-identity move through existing paths
(FLAG-7 default), with the v0 AOT refusal explicit and recorded (FLAG-8); drop-without-consume held
at RFC-0027 OQ-5 with the recorded-deterministic-release posture proposed (FLAG-4).

This is the DN-54 §10 Model-A discipline applied to Gap E: fewer new concepts, existing paths,
coherence with the ratified corpus by construction, and every refusal explicit.

**Disconfirming considerations (stated, not buried — VR-5):** (a) the static affine pass is the one
piece of genuinely new analysis in the lane — if it slips, the lane's serial schedule (M-902 →
M-903 → M-904) slips with it; Model D staging (FLAG-3) is the honest de-risk. (b) The
identity-move lowering leaves no *runtime* trace of consumption for EXPLAIN; if per-op consumption
provenance is wanted at runtime, FLAG-7's prim form is the better choice despite the extra surface.
(c) `io.md`'s "type error, not a runtime check" is itself a `Declared` spec sentence, not ratified
execution semantics — the maintainer may legitimately weaken it; this memo follows the written
corpus rather than second-guessing it.

## 7. Cross-check against `grm` M-918 — the two kickoffs must not fork the consume model

**Scope relation (`Exact` on the kickoff texts):** `grm` M-918 is the *DN-54 derive-site
consumption-model dossier* — options + recommendation from the M-824 §10 design pass (attachment of
derive-generated L0: Models A/B). This memo is the *affine `Substrate` execution model* (Gap E).
Per §3, these are **two different constructs sharing the word "consume."** They cannot fork a
*single* model because they are not the same model — but both kickoffs direct that they share one,
so this memo pins the **shared commitments** any coherent reading requires both artifacts to hold:

1. **No new L0 node** — both attach/execute through existing kernel forms (DN-54 §10.2 criterion 2;
   KC-3).
2. **Reuse existing machinery** — the existing checker/linearity/monomorphization/coherence paths,
   never a parallel bespoke path (DN-54 §10.5's Model-A rationale; DN-33 §8.1 Q4's "subsume, not a
   separate path").
3. **Never-silent staging** — anything underdetermined stays an explicit `Residual`/refusal until
   ratified (G2).
4. **Maintainer-ratified before enactment** — a design pass gates the implementation (DN-54 §10.7;
   this memo's §9).
5. **One name per term (DN-02/DN-03)** — `consume` keeps exactly its DN-03 §1 affine-acquisition
   meaning in both documents. Recommendation to `grm`: M-918's dossier should consistently name its
   subject the **attachment model** (DN-54 §10's own §10.3 heading), reserving "consume/consumption"
   for the affine construct — ending the collision instead of entrenching it.

**Status (`Exact`, 2026-07-02):** M-918 is **reserved and unminted** — no `id: M-918` exists in
`tools/github/issues.yaml`; the slot appears only as a proposed row in `.claude/kickoffs/grm.md`
(which itself demands the reciprocal cross-check: *"the `enb` M-901 cross-check (same model — the
two kickoffs must not fork it: FLAG if they diverge)"*). **FLAG-5:** when `grm` mints M-918, its
dossier and this memo MUST be reconciled explicitly — M-918 cites DN-71 §7, and any divergence from
commitments 1–5 (or a ratified attachment model that invalidates an assumption used here) re-opens
this memo append-only. No divergence exists today, trivially, because M-918's dossier does not yet
exist — that is an `Exact` vacuous truth, recorded as such, not evidence of agreement (VR-5).

## 8. FLAG list — open decisions needing maintainer sign-off

Every FLAG is `Declared` (a question with a recommended answer, not a decision).

| FLAG | Question | Recommendation |
|---|---|---|
| FLAG-1 | **Interpretation of "DN-54 §10 as the target."** DN-54 §10 contains no affine-`Substrate` content (§3); this memo reads the directive as "build Gap E to the §10 design-pass pattern + Model-A commitments." Is that the intended reading? | Confirm the §3 reading; if the maintainer instead intended a substantive dependency on the (unratified) §10 attachment model, say which parts bind Gap E. |
| FLAG-2 | **Memo location.** New note DN-71 vs a DN-54 §11 addendum. | Keep DN-71 (convention: DN-65/68/69 precedents; avoids entrenching the FLAG-1 conflation). (Note: this memo was renumbered DN-70 → DN-71 pre-integration — DN-70 is the M-905 D-lite scope-split memo that landed concurrently; no self-cite ambiguity remains.) |
| FLAG-3 | **Enforcement locus + staging.** Is static enforcement (Model S) confirmed as the target? May M-903 land the runtime backstop first as an explicitly-recorded staging step (Model D as a step, never the target)? | Confirm Model S as target; permit backstop-first staging only with the static pass in the same milestone and the interim recorded (G2). |
| FLAG-4 | **Drop-without-consume (RFC-0027 OQ-5).** Runtime error, or deterministic scope-exit release + recorded reclamation/EXPLAIN event? (Silent no-op is G2-excluded regardless.) OQ-5 was deferred pending the `graft` RFC — ratify a v0 posture now, or hold? | v0: deterministic scope-exit release with a recorded event (aligns LR-9's "leaks structurally excluded" with the *affine* reading); keep OQ-5's full protocol open for the `graft` RFC. |
| FLAG-5 | **M-918 reconciliation.** M-918 is reserved/unminted; when `grm` mints it, the two artifacts must cross-cite and hold §7's commitments 1–5; divergence re-opens this memo (append-only). Also: should M-918 adopt "attachment model" naming (§7 item 5)? | Adopt both: the reconciliation obligation and the naming recommendation. |
| FLAG-6 | **"Affine" vs "consumed exactly once."** Glossary/LR-8/`io.md` say *exactly once* (linear, =1); the kind is everywhere called *affine* (≤1). Which wording is canonical? | Resolve via FLAG-4: *at most one explicit consume* (affine), with the unconsumed case closed by the recorded deterministic release — so "exactly once" survives as "consumed exactly once **or** released exactly once, never silently neither." Amend Glossary/LR-8 wording append-only once ratified. |
| FLAG-7 | **L0 shape of `consume`.** Observational-identity move (recommended) vs an existing-node `Op` prim (`substrate.consume`) that reifies consumption as a runtime EXPLAIN event. | Identity move (KISS/KC-3); revisit if runtime consumption provenance is wanted. |
| FLAG-8 | **AOT posture.** `Substrate` not AOT-lowerable in v0 — the MLIR refusal stays explicit + recorded (M-904 DoD's sanctioned narrowing). | Accept the recorded v0 refusal; an AOT story is a later, separate decision. |
| FLAG-9 | **Value-form residence + provenance.** The handle value lives at evaluator level (shared value world), not in `mycelium-core`; should it carry acquisition provenance metadata for EXPLAIN (RFC-0001 §4.3-style), mirroring DN-54 §10.6 OQ-A? | Evaluator level, yes; carry the acquisition provenance (cheap, and it is what makes §4.1's "inspectable" true rather than aspirational). |

## 9. Definition of Done (this memo's gate — mirrors DN-54 §10.7)

This memo is complete as a design document when the maintainer has reviewed it and signalled one of:

1. **Accept** Model S (with FLAG dispositions recorded here, append-only) → M-902…M-904 unblock and
   implement exactly the confirmed model.
2. **Revise** — request a new pass with different candidates or criteria.
3. **Flag-blocking** — name specific FLAGs (§8) that must be settled before any code.

The sign-off is recorded as an append-only status/changelog entry on this note plus the M-901 issue
close-out (integrator-owned). Nothing in this memo advances any ADR/RFC/DN status; DN-54 remains
`Accepted`, RFC-0027 OQ-5 remains open, and this note itself stays `Draft` until the maintainer's
signal (house rule #3).

**Sign-off recorded (2026-07-02): Accept.** The maintainer accepts Model S per the note above.
This note moves `Draft` → `Accepted` (house rule #3, stepping forward, not skipped); it moves to
`Resolved` once M-902…M-904 land per their DoDs and the M-901 issue closes out.

---

## Meta — changelog

- **2026-07-02 — Created (Draft) — authored (M-901, kickoff `enb`, epic E28-1).** The Gap-E
  confirm-memo: current-state inventory (`Exact`, source-read); the DN-54 §10 target reading + the
  consume-terminology collision surfaced (FLAG-1); the proposed Model S (interpreter-level opaque
  affine handle · static use-once checking with a never-silent runtime backstop · identity-move
  lowering through existing paths, no new L0 node · drop protocol held at RFC-0027 OQ-5);
  alternatives D/K/E with an honest tradeoff table; the `grm` M-918 cross-check with five shared
  commitments and the reconciliation FLAG; nine FLAGs for maintainer sign-off. Confirms nothing —
  maintainer sign-off pending; M-902…M-904 stay blocked. CHANGELOG / Doc-Index / issues.yaml /
  api-index untouched — FLAGged to the integrating parent. (Append-only; VR-5; G2.)
- **2026-07-02 — Accepted (maintainer sign-off recorded: Accept).** Model S (§4/§6) approved as
  drafted. Unblocks M-902…M-904 (tools/github/issues.yaml status:blocked → status:todo).
  FLAG-1…FLAG-9 dispositions not itemized in this sign-off — remain open, not guessed (G2).
  Append-only; VR-5; G2.
- **2026-07-02 — Implementation landed (integration-reconcile note).** Model S now **executes at
  the L1-eval level**: M-902 (Substrate v0 opaque affine handle), M-903 (static use-once affine
  tracker + never-silent runtime backstop) and M-904 (identity-move `consume` lowering + v0 drop
  posture) landed in `crates/mycelium-l1/src/eval.rs` on `dev` this wave — exactly the §4/§6 model,
  no new L0 node. The `grm` M-918 cross-check is RESOLVED: both landed on the same Model S; the
  consume model was not forked. Guarantee strength stays `Empirical` (trial-backed; no `Proven`
  upgrade — VR-5). Append-only; house rule #3.
