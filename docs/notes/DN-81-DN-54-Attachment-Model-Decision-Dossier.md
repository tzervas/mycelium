# Design Note DN-81 — DN-54 §10 Derive-Site **Attachment Model** Decision Dossier (Model A vs Model B)

| Field | Value |
|---|---|
| **Note** | DN-81 |
| **Status** | **Recommended — pending orchestrator acceptance** (2026-07-02, under the maintainer's 2026-07-02 delegation of design decisions to the orchestrator). This dossier's §5 recommendation — **Model A (sibling-item injection)** — becomes the decision once the orchestrator accepts; it does **not** self-ratify (house rule #4). DN-54 stays `Accepted`; nothing here advances any status. Enactment is a **separate serial-lane follow-up** (§6), not this note. |
| **Decides** | *Nothing normatively yet.* Frames the DN-54 §10 **attachment-model** decision precisely (§2); enumerates Model A vs Model B with grounded tradeoffs (§3–§4); records a recommendation with its evidence (§5); adversarially stress-tests that recommendation (§5.3); lists the enactment work the chosen model directs and the DN-75 residuals it closes vs leaves open (§6); FLAGs the M-918 `issues.yaml` conflation and the DN-71 §7 naming recommendation (§7). |
| **Feeds** | The DN-54 **Enacted** gate (DN-54 §9 — the attachment model is one of its named residuals); the **enacting follow-up task** (§6 — the serial `mycelium-l1` lane); DN-75 residual ledger **R-1 → R-2 → R-3** (§6.3); the DN-76 line-144 maintainer gate (*"Derive-site consumption model … enact in the extension-checker"*); kickoff `grm` DoD (E30-1). |
| **Depends on** | DN-54 §10 (M-824 attachment design-pass — the Models A/B, §10.2 criteria, §10.5 recommendation, §10.6 open questions); DN-75 (M-917 completion audit — R-1…R-4 residual ledger, the per-section `Empirical` verdicts); DN-71 §3/§7 (the two-constructs-sharing-"consume" distinction + the naming recommendation); the M-919 landing (PR #1028 — the now-**active** affine checker in `check_lower_rule_rhs_type`); RFC-0019 (traits/coherence/monomorphization); ADR-003 (content-addressed identity); KC-3 (house rule #5); RFC-0001 §4.3 (provenance metadata). |
| **Date** | July 2, 2026 |
| **Task** | M-972 (kickoff `grm`, epic E30-1, Phase-I H2a — the DN-54 §10 attachment-model dossier; branch `claude/leaf/E30-1-M972-dn54-attachment-dossier`) |

> **Posture (transparency rule / VR-5 / G2).** This is a **design dossier for the orchestrator's
> acceptance**, not an enactment and not a self-ratification. Claims about what the code and corpus
> **already do** are `Exact` (source-read, cited to `file:line` at the branch base — the current
> `origin/dev` tip). Design recommendations are `Declared` (argued from the cited basis, awaiting
> acceptance). Prior-art parallels inherited from DN-54 §5 are `Empirical`-at-source. No claim is
> upgraded past its basis. The recommendation is argued **on merit** and its strongest counter-case
> is developed in a dedicated adversarial section (§5.3) before it is allowed to stand — assent is a
> claim and is tagged at its strength (house rule #4).

---

## §1 Why this dossier exists — the misfiled "resolved"

M-919 (the extension-checker enactment, PR #1028, landed to `dev` 2026-07-02) discovered that the
DN-54 §10 **attachment model was never actually resolved**. The `tools/github/issues.yaml` **M-918**
entry records it as *"DN-54 derive-site consumption-model dossier — **RESOLVED: DN-71 Model S**"*
with a Definition of Done reading *"MET BY: DN-71 Model S (ratified 2026-07-02) — one model, no
fork."* (`issues.yaml:10763–10791`, `Exact`).

That framing is wrong, and DN-71 says so in its own text. **DN-71 §3 and §7** (`Exact`) draw the
distinction explicitly:

> *"the word 'consume' collides across two distinct constructs: (1) `consume <expr>` — the affine
> `Substrate` acquisition expression … this memo's subject (Gap E); (2) derive-site consumption —
> DN-54 §10's attachment question — `grm` M-918's subject."* (DN-71 §3)

DN-71 §7 states plainly these *"are two different constructs sharing the word 'consume'"* and
**recommends M-918 be renamed to the "attachment model"** to end the collision (DN-71 §7 item 5,
FLAG-5). DN-71 Model S is the **affine `Substrate`/`consume` execution model** — an interpreter-level
opaque handle with static use-once checking. DN-54 §10's attachment model is a **different**
construct: *where does a `derive Name for T` rule's generated L0 live in the program* — **Model A
(sibling-injection)** vs **Model B (registry)** (DN-54 §10.3–§10.5).

DN-75 (the M-917 audit, `Resolved`) already confirmed this residual is genuinely open: its §5 ledger
routes **R-1** (attachment model + parametric instantiation + item-shaped RHS) as still-open work,
and DN-76 line 144 still lists *"Derive-site consumption model — dossier → maintainer ratification →
enact in the extension-checker"* as a live **`grm` M-918/M-919 (maintainer gate)**. No ratification of
Model A vs Model B exists anywhere in the corpus (DN-75 §3 §10 row, DN-54 §9 gate, DN-76 line 144).

This dossier supplies the missing attachment-model decision. Per the maintainer's 2026-07-02
delegation, the orchestrator's acceptance of §5 **is** the ratification.

---

## §2 The decision, framed precisely

### §2.1 What is actually open

`lower Name[params] = <rhs>` defines **what** the generated L0 is; the rule's RHS is elaborated to a
closed L0 `Node` by `elaborate_lower_rule` (`crates/mycelium-l1/src/elab.rs:532`, landed M-812-cont,
`Empirical` per the DN-54 §7 harness). `derive Name for T` **instantiates** the rule at a concrete
type. The open question is **attachment**: *where does the instantiated L0 live in the program, and
how does surrounding code refer to it?*

**Current code state (`Exact`, branch base `origin/dev`):** a `derive` use site is **name-resolution
and target-type checked** — `check_derive_application` (`checkty.rs:2162`) rejects an unknown rule
name and an ill-formed `for_ty`, never-silent (G2) — but it produces **no L0 and registers
nothing**. `Item::Derive(_)` is skipped at elaboration (`checkty.rs:839`,
`Item::Use(_) | Item::Default(_) | Item::Derive(_) => {}`). So the derive site is a checked no-op
today: DN-54 §4.4 (content-addressed dedup of derive output), §4.5 (`reveal`-ability of a use), and
§4.6 row 6 (use-site output IL-grammar check) all have **no output to act on** until an attachment
model lands (DN-75 §3 rows; R-2). The attachment model is the missing hinge.

Three tightly-coupled facets ride on the choice (DN-54 §10.1):

- **(a) Attachment** — where the derived L0 lives / how it is named and reached (the primary axis:
  Model A vs Model B).
- **(b) Item-not-Expr RHS gap** — the DN-54 §3.2 motivating example's RHS is an `impl` block, an
  **item**, not an `Expr`; `parse_lower_decl` calls the expression parser (`parse.rs`), so the
  canonical use case (derive a trait impl) is **not expressible** in v0. Both models need a shared
  `parse_lower_item_rhs` parser variant (DN-54 §10.3).
- **(c) Parametric instantiation** — a `lower Name[T]` whose RHS mentions `T` has no monomorphic L0
  until a `derive` provides the concrete type; an undetermined `T` at the derive site surfaces as the
  ordinary `ElabError::Residual` (never-silent — DN-54 §10.1(c)). Both models route this through
  `mono.rs` (the existing monomorphizer; its coherence-key/head-erasure logic is at `mono.rs:876`).

### §2.2 What a satisfactory model must do (DN-54 §10.2 criteria, `Declared`)

1. **Named + reachable output** — a call site of `derive Checksum for MyPacket` must resolve the
   result without re-running the rule; identity derivable from `(rule_name, concrete_type_args)`
   (a content-addressability constraint, ADR-003).
2. **No new L0 node (KC-3)** — the attachment mechanism adds **no** kernel node.
3. **Coherence-compatible (RFC-0019 §4.2)** — the orphan rule + global-uniqueness must still hold; a
   derived `impl` must be treated by the coherence checker exactly like a hand-written `impl`.
4. **Never-silent (G2)** — a conflicting or ill-typed derive is an explicit error, never a silent
   accept or silent discard.
5. **Reveal-able (§4.5)** — attachment must not hide the generated L0; `reveal` shows the same L0
   regardless of model.

### §2.3 Interaction with the now-enforced affine checker (M-919 — new evidence)

M-919 (PR #1028) made the static affine use-once tracker **active** inside a `lower` rule's RHS
check: `check_lower_rule_rhs_type` now seeds an active `Tracker::seeded(&[])`
(`crates/mycelium-l1/src/affine.rs` module docs; `checkty.rs:2100`), closing a real coverage gap
(a rule RHS can bind a `Substrate` via a helper `fn` call — DN-54 §3.3 permits ordinary fn calls —
and a same-rule double-consume previously type-checked silently under the prior inert tracker).

This matters directly for the attachment decision, and it is **evidence that did not exist when the
M-824 §10.5 recommendation was written.** Whichever model is chosen, a *derived* `impl`'s method
bodies must receive the **same** use-once enforcement as hand-written method bodies — otherwise the
gap M-919 just closed reopens at the derive site. The tracker is active precisely in
`check_fn_body`'s single-walk `Cx` (`affine.rs:132,148`). So the model that routes a derived impl
through `check_fn_body` inherits the affine guarantee **by construction**; a model that stores derived
impls in a side-table must additionally wire that table's method bodies into the affine pass or
silently lose the M-919 coverage. §5.2 develops this as the decisive new argument.

---

## §3 The two candidate models (from DN-54 §10.3, restated + updated)

### §3.1 Model A — Sibling-item injection

`derive Name for T` is elaborated as if the user had written the rule's RHS as a **sibling item in
the same nodule**, with `T` substituted for the rule's type parameter. After `mono.rs`
monomorphizes the RHS at `T`, the resulting closed item (e.g. an `impl` block) is inserted into the
nodule's item list as a co-equal declaration, content-addressed from `(rule_name, T)` and registered
in `Env` under the same namespace as hand-written `impl` blocks.

- **Item-not-Expr gap:** `parse_lower_item_rhs` accepts item-shaped RHS; `mono.rs` substitutes `T`
  through the item body exactly as it substitutes into a generic `fn` body (RFC-0019 §4.3).
- **Coherence:** the injected `impl` is visible to the RFC-0019 coherence pass as a sibling item, so
  global-uniqueness holds unchanged — a second `derive Checksum for MyPacket` is a duplicate `impl`,
  caught never-silent.
- **Affine (M-919):** the injected impl's method bodies go through `check_fn_body`, which carries the
  active affine tracker — the use-once guarantee is inherited **for free** (§2.3).
- **`reveal`:** the injected item went through the full elaboration pipeline, so `reveal` shows its
  exact L0 with no special case.
- **KC-3:** no new L0 node; the injection is an elaboration-phase rewrite (RFC-0007 §4.1 nodes only).

### §3.2 Model B — Registry of derived impls (side-table)

`derive Name for T` does **not** inject a sibling item; instead a **`DerivedImplTable`** in `Env`
(a new data structure, not an L0 node) is populated with `(rule_name, concrete_T) → L0_node`. The
consuming paths — trait dispatch, coherence, `reveal`, and the affine pass — must each be extended to
consult the table in addition to the hand-written `impl` namespace.

- **Item-not-Expr gap:** same `parse_lower_item_rhs` variant; output stored in the table rather than
  injected.
- **Coherence:** requires coherence to cover **both** namespaces (hand-written impls + the table); a
  derive whose entry would conflict with a hand-written impl must be an explicit dual-namespace check.
- **Affine (M-919):** requires the table's stored method bodies to be run through the affine pass
  explicitly — a *new* wiring, not inherited from `check_fn_body`.
- **`reveal`:** requires a new query arm over the table.
- **KC-3:** the table is an elaboration artifact (no new L0 node), but dispatch, coherence, `reveal`,
  and the affine pass all grow a table-aware arm.
- **One advantage:** the table preserves an explicit record of *which impls came from `derive`* vs
  hand-written — useful for tooling/IDE provenance (DN-54 §10.5 disconfirming argument).

---

## §4 Honest tradeoff table (`Declared`; extends DN-54 §10.4 with the M-919 axis)

| Criterion | Model A — Sibling injection | Model B — Derived-impl registry |
|---|---|---|
| **KC-3 / new L0 node** | None | None (table is an elaboration artifact) |
| **New `Env`/dispatch machinery** | `parse_lower_item_rhs`; `elaborate_derive_as_sibling`; a content-key de-dup guard (trivial via ADR-003) | `parse_lower_item_rhs`; `DerivedImplTable`; **extended coherence + dispatch + reveal + affine** — four new table-aware arms |
| **Coherence (RFC-0019 §4.2)** | Free — injected impl enters the existing pass; global-uniqueness by construction | Dual-namespace check required; explicit invariant that both are covered |
| **Affine use-once (M-919, new)** | **Free — method bodies flow through `check_fn_body`'s active tracker** | Requires wiring the table's bodies into the affine pass, or the M-919 coverage silently reopens at the derive site |
| **`reveal` (§4.5)** | Free — first-class item in the elaborated nodule | New query arm over the table |
| **Never-silent (G2)** | Coherence + affine errors on the existing paths | More surface for a missed case (four arms must each be complete) |
| **KISS / KC-3 preference (house rule #5)** | Strongly favored — fewer concepts, existing paths | Weaker — one new structure, four new arms to keep in sync |
| **Provenance / tooling** | Provenance not intrinsic; recoverable via ADR-003 hash + RFC-0001 §4.3 metadata (OQ-A) | Provenance intrinsic (the table *is* the record) |
| **Separate-compilation caching** | Served by substrate content-addressing (§4.4) — the injected L0 hash-dedups in the store | A persistent table is a *possible* future cache home, but content-addressing already dedups |

---

## §5 Recommendation

### §5.1 Recommended — **Model A (sibling-item injection)**

`Declared` — design recommendation, pending orchestrator acceptance. This confirms and strengthens
DN-54 §10.5's own Model-A recommendation, and it aligns with DN-71 §7's shared-commitment #2
(*"reuse existing machinery … never a parallel bespoke path"*) and DN-33 §8.1 Q4's ratified
*"subsume, not a separate path"* discipline.

### §5.2 The evidence

1. **Strictly less machinery (KISS / KC-3, house rule #5).** Model B's four table-aware arms
   (coherence, dispatch, `reveal`, affine) are **all eliminated** under Model A because the injected
   impl is a peer of hand-written impls on the existing paths. A `DerivedImplTable` is a new concept
   that must be maintained across four surfaces; Model A adds none.
2. **Coherence by construction (RFC-0019 §4.2).** A derived impl *is* an impl entry, so the existing
   global-uniqueness / orphan machinery (`mono.rs:876` head-erasure coherence key) covers it without
   extension — including the never-silent duplicate-derive refusal (criterion 4).
3. **Affine coverage by construction (M-919 — the decisive new argument).** This evidence post-dates
   the M-824 design pass. Model A routes a derived impl's method bodies through `check_fn_body`,
   which now carries the **active** affine tracker (`affine.rs:132,148`), so the exact use-once
   enforcement M-919 just extended into rule RHSs is inherited at the derive site **for free**. Under
   Model B, the derive site's method bodies live in a side-table that `check_fn_body` never walks, so
   the M-919 double-consume coverage would either need re-wiring or silently regress — a G2 risk on
   the very construct M-919 was landed to protect. This single axis, absent from the original §10.4
   table, tips an already-favored recommendation to a firm one.
4. **Reveal is already exact (§4.5).** The injected item went through the full pipeline, so `reveal`
   needs no special case; DN-54 §4.5's by-construction argument holds directly.
5. **The item-not-Expr parser gap is shared** — `parse_lower_item_rhs` is needed by both models, so
   it is not a differentiator (DN-54 §10.5).

### §5.3 Adversarial stress-test — the strongest case *against* Model A

*(House rule #4: I am Opus; I argue the strongest case against my own recommendation before letting
it stand, and revise if it breaks.)*

**Objection 1 — the cross-phylum orphan problem (OQ-D).** Model A "injects a sibling item" — but
*into which nodule/phylum?* If `lower Checksum` is defined in phylum P and `derive Checksum for
MyPacket` is written in phylum Q where neither `Checksum` nor `MyPacket` is locally owned,
sibling-injection lands an `impl` in Q that the RFC-0019 §4.2 **orphan rule** may forbid. Model B's
content-addressed table could key globally and dodge the "where does the item textually live"
question entirely. This is a genuine place where Model A's "it's just a sibling item" simplicity
leaks — DN-54 §10.6 OQ-D flags exactly this.

*Examination.* On inspection this **reinforces** Model A rather than defeating it. A derived impl
that would violate the orphan rule **should be refused** — that is the coherence invariant working as
designed (criterion 3). Model A makes that refusal happen through the **existing** coherence pass,
never-silent (G2). Model B would have to *re-implement* orphan checking over the table, or risk
**silently admitting an orphan derived impl** — a G2 violation. So the objection strengthens Model A
on the never-silent axis. **But it does impose a binding obligation:** the enacting RFC must settle
OQ-D (the phylum-level attachment scope — where an injected impl "lives" for a cross-phylum derive)
**explicitly**, not leave it open. §6.4 records that as a required enactment step, not a deferred
open question. *Recommendation holds; obligation added.*

**Objection 2 — provenance loss.** Model A discards the "which impls came from `derive`" record once
injection happens (DN-54 §10.5's own disconfirming argument). *Counter:* content-addressing (ADR-003)
encodes origin in the hash, and RFC-0001 §4.3 provides a `provenance` metadata field the elaborator
can populate with `(rule_name, instantiation_args)`. This dossier **adopts** OQ-A: recommend the
enacting RFC populate the provenance field so Model A matches Model B's one genuine advantage without
Model B's four extra arms. *Recommendation holds; OQ-A upgraded from "open" to "adopt".*

**Objection 3 — separate-compilation caching.** For a content-addressed language (ADR-003), a
persistent registry is arguably the more natural long-term home for cross-compilation-unit derived-
impl caching; sibling-injection re-elaborates at each derive site. *Counter:* the substrate's
content-addressing **already** dedups the injected L0 by hash (DN-54 §4.4) — Model A does not redo
cached work, because the elaborated `Node` is content-addressed in the same store regardless of
model. The "registry for caching" motivation is served by content-addressing, not by a separate `Env`
table. *Recommendation holds — but recorded, append-only:* if a *future* separate-compilation design
needs a persistent derived-impl **index** (distinct from the value store), that is a future DN, not a
reason to adopt Model B now. Do not foreclose it; do not pre-build it (YAGNI).

**Objection 4 — deferred-inference derives.** Could Model B's late-bound table lookup handle a derive
whose target type is only known after later inference, where Model A's eager injection would hit a
`Residual`? *Counter:* v0 has no deferred-inference derive; both models surface an undetermined `T`
identically as `ElabError::Residual`, never-silent (DN-54 §10.1(c)). No advantage to B. *Holds.*

**Verdict.** The strongest objection tested (Objection 1, cross-phylum orphan / OQ-D) does not
defeat Model A — on examination it reinforces the never-silent case for it — but it converts OQ-D
from a "defer to the RFC" open question into a **required** enactment deliverable (§6.4). The
recommendation stands, **revised** only to bind OQ-D and to adopt OQ-A (provenance metadata) into the
enactment scope.

---

## §6 Enactment work the chosen model directs (for the serial-lane follow-up)

This dossier is docs-only. The following is the enactment scope Model A directs, for the separate
`mycelium-l1` serial-lane task (the DN-76 line-144 gate's "enact in the extension-checker" step;
tracked as the M-919 successor — see §7 on the mislabeled M-918/M-919). All `Declared`.

### §6.1 Parser — item-shaped RHS

Add `parse_lower_item_rhs` accepting item-shaped RHS forms. **OQ-B (enumerate the legal set):** the
minimum for the DN-54 §3.2 use case is `impl Trait for T { … }`; the RFC must enumerate the supported
item forms explicitly (no silent over-generalization — G2). Keep `type` aliases and standalone `fn`
items out of v1 unless a use case is named (YAGNI); OQ-C (mixed expr-and-item rules) stays a future
extension the parser architecture must not preclude.

### §6.2 Elaboration — `elaborate_derive_as_sibling`

Replace the `Item::Derive(_) => {}` no-op (`checkty.rs:839`) and the elaboration skip with a path
that: monomorphizes the rule RHS at the derive site's concrete `for_ty` via `mono.rs`; inserts the
resulting closed `Item` into the nodule's item list; content-addresses it by `(rule_name, for_ty)`;
and de-dups via the ADR-003 content key (a duplicate is a no-op if identical, a coherence error if
different). Undetermined `T` continues to surface `ElabError::Residual` (never-silent).

### §6.3 Coherence + affine + reveal — inherited, but tested

- **Coherence:** confirm the injected impl enters the RFC-0019 coherence pass; add a reject-
  conformance case that a duplicate `derive` (or a derive colliding with a hand-written impl) is a
  never-silent global-uniqueness refusal.
- **Affine (M-919):** confirm the injected impl's method bodies flow through `check_fn_body`'s active
  affine tracker; add the derive-site analogue of M-919's reject test — a derived impl whose method
  double-consumes a `Substrate` is refused, citing DN-71 by name (the derive-site twin of
  `lower_rule_rhs_double_consume_of_a_helper_acquired_substrate_is_refused`).
- **`reveal` (§4.5):** rides the DN-38 §5 inspector track (R-2); when it lands, no special case for
  derived items.

### §6.4 OQ-D — required, not deferred (per §5.3 Objection 1)

The enacting RFC **must** settle the cross-phylum attachment scope: where an injected impl lives for
a `derive` in a phylum different from the rule's definition, and how the orphan rule applies. This is
a deliverable, not an open question — leaving it open would leave a never-silent hole in the coherence
story (§5.3).

### §6.5 Provenance — adopt OQ-A

Populate the RFC-0001 §4.3 `provenance` field of a derived impl with `(rule_name, instantiation_args)`
so Model A retains Model B's provenance advantage (§5.3 Objection 2). Cheap; makes `reveal` provenance
real rather than aspirational (cf. DN-71 FLAG-9's parallel ruling for the affine handle).

### §6.6 DN-75 residuals — closed vs left open

- **Closes / directs to closure (once enacted):**
  - **R-1** — derive-site attachment model chosen (Model A); item-shaped RHS (`parse_lower_item_rhs`);
    parametric-instantiation path (`mono.rs`). This dossier is the ratifiable input R-1 was waiting on.
  - **R-2** — §4.4 content-addressed dedup of derive output, §4.5 `reveal`-exercisability, §4.6 row 6
    (use-site output IL-grammar check) — all become actionable because derive output now *exists*;
    bind them into the enacting task's DoD (the DN-75 §5 R-2 FLAG target).
  - **R-3** — §7.1/§7.2 generated-corpus differential and DN-20 LOW/HIGH tiering becomes fully
    meaningful once derive sites elaborate (DN-75 §5 R-3); bind to the enacting task's DoD, or a
    small dedicated M-task if scope is held tight.
- **Leaves open (not governed by the attachment choice):**
  - **R-4** — §7.3 `certified`-mode `delaborate ∘ lower = id` plus §7.4 `Proven`-per-run TV witness.
    Gated on ADR-032 `certified` mode; rides that track; must appear on the DN-54 Enacted checklist.
    The attachment model does not touch it.
  - **OQ-C** (mixed expr-and-item rules), **OQ-E** (effect annotation on item-RHS rules, RFC-0014) —
    future extensions, recorded, not in v1 scope.

### §6.7 DN-54 status

Enactment of Model A per §6, plus R-2/R-3 closure, is what steps DN-54 toward `Enacted` — together
with R-4's certified-mode track and the §7 verification harness green and ratified (DN-54 §9). This
dossier advances **nothing**; DN-54 stays `Accepted`. House rule #3: `Enacted` is the integrating
parent's step through `Accepted`, only when genuinely complete.

---

## §7 FLAGs — for the orchestrator (issues.yaml + DN-71 are not edited here; G2, flagged not guessed)

**FLAG-1 — the M-918 `issues.yaml` entry conflates two different constructs; correct it.** The M-918
entry (`tools/github/issues.yaml:10763–10791`) titles the DN-54 §10 **attachment-model** question
*"RESOLVED: DN-71 Model S"* and its DoD reads *"MET BY: DN-71 Model S (ratified 2026-07-02) — one
model, no fork."* This is wrong on the corpus's own terms: **DN-71 §3/§7** state that DN-71 Model S
(the affine `Substrate`/`consume` execution model) and DN-54 §10's attachment model *"are two
different constructs sharing the word 'consume'"*. DN-71 Model S does **not** resolve Model A vs
Model B. **Recommended correction (orchestrator-owned):** re-point M-918 to this dossier (DN-81) as
the genuine attachment-model resolution (recommendation: Model A, pending orchestrator acceptance);
record DN-71 Model S as the *separate* affine-construct resolution it actually is; drop the "RESOLVED:
DN-71 Model S" wording from the attachment-model slot. (M-918's `status:done` is only correct for the
affine cross-check, not for the attachment model — which was open until this dossier.)

**FLAG-2 — adopt DN-54 §10's "attachment model" naming (DN-71 §7 item 5 recommendation).** DN-71 §7
FLAG-5 recommends reserving "consume/consumption" for the affine `Substrate` construct and naming the
derive-site question the **"attachment model"** (DN-54 §10.3's own heading). This dossier adopts that
naming throughout. Recommend the orchestrator apply it when correcting M-918 (title → "DN-54 §10
derive-site **attachment-model** dossier") so the collision does not re-entrench.

**FLAG-3 — M-919's "step DN-54's status honestly" is satisfied, but its title reads as more than it
did.** M-919 (`issues.yaml:10793`, *"Enact DN-71 Model S in the extension-checker + step DN-54's
status honestly"*) landed a real affine-coverage fix (the active tracker in `check_lower_rule_rhs_type`)
and correctly held DN-54 at `Accepted`. But its title, read as *"the extension-checker enactment that
completes DN-54,"* overstates: the attachment-model enactment (§6) is a **separate** follow-up that
did not happen in M-919. Recommend the orchestrator record the §6 enactment as a distinct successor
task (the M-919 sibling under E30-1), so DN-76 line 144's *"enact in the extension-checker"* step is
tracked to the right, still-open work.

**FLAG-4 — M-972 not yet minted.** This task (M-972, the attachment-model dossier) is new; its
`issues.yaml` entry, `doc_refs` (`corpus:DN-54`, `src:docs/notes/DN-81-*`), `CHANGELOG`, and
`Doc-Index` are **orchestrator-owned** and not touched here (leaf discipline). Flagged for the
integrating parent.

Shared-file updates this note does **not** make (leaf discipline — FLAGged up, not edited):
`CHANGELOG.md`, `docs/Doc-Index.md`, `docs/api-index/`, `tools/github/issues.yaml` (M-972 mint +
status; the M-918/M-919 corrections of FLAG-1…FLAG-3) — all owned by the integrating parent.

---

## §8 Guarantee posture (VR-5) + Definition of Done

- **`Exact`** — the current-code claims (§2.1 derive-is-a-checked-no-op; §2.3 the active affine
  tracker; the `file:line` citations), read at the branch base.
- **`Declared`** — the Model A recommendation (§5) and every design judgment in §3–§6. Argued from
  the cited basis; not ratified until the orchestrator accepts.
- **`Empirical`-at-source** — the prior-art parallels inherited from DN-54 §5 (Lombok/proc-macro/
  hygiene contrasts).
- **No upgrade** — the recommendation is not stated more strongly than "recommended, pending
  acceptance"; the adversarial §5.3 is the disconfirming-evidence surface house rule #4 requires.

**Definition of Done (M-972 / this note).** Met when: **(a)** the attachment decision is framed
precisely with the current-code state (§2 — done); **(b)** Model A vs Model B are enumerated with
grounded tradeoffs including the new M-919 affine axis (§3–§4 — done); **(c)** a clear recommendation
with evidence is recorded (§5 — done); **(d)** the recommendation is adversarially stress-tested in a
dedicated section, not self-ratified (§5.3 — done); **(e)** the enactment work plus the DN-75
residuals closed vs left open are listed (§6 — done); **(f)** the M-918 conflation and the DN-71 §7
naming recommendation are FLAGged (§7 — done). Status stays **Recommended — pending orchestrator
acceptance**; acceptance is the orchestrator's act, after which this note → the decision it records
(append-only). Enacts no code; moves no other doc's status.

---

## §9 Relation to corpus

- **DN-54 §10** (Accepted; M-824 addendum) — the attachment design-pass this dossier decides: §10.1
  the open facets, §10.2 the criteria, §10.3 Models A/B, §10.4 the tradeoff table, §10.5 the Model-A
  recommendation, §10.6 OQ-A…OQ-E, §10.7 the sequencing gate.
- **DN-71 §3/§7** (Accepted) — the two-constructs distinction and the "attachment model" naming
  recommendation (FLAG-5); the shared commitments (§7 items 1–5) any coherent reading must hold.
- **DN-75** (Resolved; M-917 audit) — the residual ledger R-1…R-4 this choice governs (R-1/R-2/R-3
  closed/directed; R-4 left open); the `Empirical` per-section verdicts (§3).
- **DN-76** line 144 — the live maintainer gate this dossier feeds.
- **M-919 / PR #1028** — `crates/mycelium-l1/src/checkty.rs` `check_lower_rule_rhs_type` plus
  `affine.rs` (the now-active tracker) — the §2.3/§5.2 decisive new evidence.
- **RFC-0019** (Enacted) — traits/coherence/monomorphization; Model A's "coherence by construction"
  and the `mono.rs` instantiation path.
- **ADR-003** — content-addressed identity; the derive-output de-dup key (§4.4) and the
  separate-compilation-caching counter (§5.3 Objection 3).
- **RFC-0001 §4.3** — the `provenance` metadata field OQ-A adopts (§6.5).
- **KC-3** (house rule #5) — no new L0 node; the KISS preference favoring Model A.
- **G2** (never-silent) and **VR-5** (no upgrade past basis) — the refusal + tag discipline throughout.

---

## Meta — changelog

- **2026-07-02 — Created (Recommended — pending orchestrator acceptance) — authored (M-972, kickoff
  `grm`, epic E30-1).** The DN-54 §10 **attachment-model** decision dossier. Frames the decision
  precisely against the current code (a `derive` site is a checked no-op — `Item::Derive` skipped at
  elaboration, `checkty.rs:839`; the attachment model is the missing hinge, §2). Enumerates **Model A
  (sibling-item injection)** vs **Model B (derived-impl registry)** with an honest tradeoff table
  (§3–§4) **extended by the M-919 affine axis** — the active use-once tracker in `check_fn_body`,
  new evidence post-dating the M-824 design pass. Recommends **Model A** (§5): strictly less
  machinery, coherence and affine and reveal by construction, KC-3/KISS. Adversarially stress-tests
  the recommendation (§5.3) — the strongest objection (cross-phylum orphan / OQ-D) reinforces rather
  than defeats it but binds OQ-D as a required enactment deliverable; adopts OQ-A (provenance
  metadata). Lists the enactment work (§6 — `parse_lower_item_rhs`, `elaborate_derive_as_sibling`,
  coherence and affine and reveal tests, OQ-D, provenance) and the DN-75 residuals it closes
  (**R-1 → R-2 → R-3**) vs leaves open (**R-4**, certified-mode; OQ-C/OQ-E). FLAGs the M-918
  `issues.yaml` conflation (*"RESOLVED: DN-71 Model S"* wrongly equates the affine construct with the
  attachment model — DN-71 §3/§7), the DN-71 §7 "attachment model" naming recommendation, and the
  M-919 title overstatement (§7). Status **Recommended — pending orchestrator acceptance** under the
  maintainer's 2026-07-02 delegation; **not** self-ratified (house rule #4). Enacts no code; advances
  no status; DN-54 stays `Accepted`. CHANGELOG / Doc-Index / issues.yaml / docs/api-index owned by
  the integrating parent (FLAGged, not edited). (Append-only; VR-5; G2.)
