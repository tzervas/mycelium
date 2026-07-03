# Design Note DN-76 — Kernel-Freeze Four-Condition Scorecard (the M-969 gate instrument)

| Field | Value |
|---|---|
| **Note** | DN-76 |
| **Status** | **Accepted** (2026-07-02 — accepted by the wave orchestrator under the maintainer's 2026-07-02 delegation (`Declared`), per the integration-reconcile promotion gate; **Option A** adopted: this scorecard is accepted **as the M-969 freeze-gate instrument**. Acceptance does **not** declare the freeze — the scorecard is 0/4 green and M-969 stays gated on 4/4; the freeze declaration remains a maintainer-class act executed only on green. Was **Recommended, pending orchestrator acceptance** 2026-07-02; that history stands unchanged below — append-only forward transition, house rule #3.) The maintainer **delegated this decision to the orchestrator** (2026-07-02, orchestrator-relayed session directive; `Declared`). |
| **Feeds** | **M-969** (the kernel-freeze declaration — strictly last, executed only on a green scorecard) · **M-959** (reject-ledger completion — condition 1's closing task) · the **ADR-021/ADR-022** Gate-A release-readiness input that DN-56 defines. |
| **Extends** | **DN-56** (`Kernel-Completeness-And-Freeze-Criterion`, Accepted 2026-06-27) — this is the per-condition *audit instrument* DN-56 §5 calls for, at its current (2026-07-02) evidence state. Append-only: DN-56 is not modified; its gate is *scored* here. |
| **Date** | July 2, 2026 |
| **Task** | M-958 (kickoff `frz`, Lane A) |

> **Posture (transparency rule / VR-5 / G2).** Every verdict below is **`Empirical`** — an
> evidence-based read of the repository at `dev` tip `629aa12` (2026-07-02), citing the artifacts
> inspected. Nothing here is `Proven`; nothing is upgraded past its basis. **The kernel is NOT
> frozen today, and this note does not freeze it** — it records exactly how far each DN-56 freeze
> condition is from green, what remains, and who owns the remainder. A discrepancy between a
> stated status and the ledger evidence is **FLAGged, not smoothed over** (house rule #4).

## 1. Scope — which conditions this scores

DN-56 §5 defines a five-condition freeze gate. Condition §5.1 (**census green** — the DN-52 census
with zero silent gaps, no `Undetermined` rows, backed by the DN-50 OQ-2 standing gate) was
**satisfied in W5** (2026-06-27; DN-56 changelog row 2:
`runnable_gate.rs::every_accepted_construct_elaborates_to_ok_or_explicit_residual`, `Empirical`).
The **four conditions still open** at DN-56 ratification are the scorecard's rows, numbered here as
the frz kickoff numbers them (M-958 row):

| Scorecard # | DN-56 §5 # | Condition |
|---|---|---|
| 1 | §5.2 | Reject-ledger completeness |
| 2 | §5.3 | Primitive set closed |
| 3 | §5.4 | Lowering surface closed |
| 4 | §5.5 | KC-3 completeness review passed |

## 2. The scorecard (2026-07-02, `dev` @ `629aa12`)

| # | Condition | Verdict | Tag | One-line basis |
|---|---|---|---|---|
| 1 | Reject-ledger completeness | **OPEN** (partial: parse-level ledger exists, self-policing; checker-level refusals explicit but unledgered) | `Empirical` | 28-fixture reject corpus + `REJECT_EXPECTED` map; no unified `{construct, reason, alternative}` ledger, no reject-path regression guard |
| 2 | Primitive set closed | **OPEN — near green** (trusted-base growth closed; scheduled Π additions still `todo` on the issue ledger — disposition FLAGged) | `Empirical` | ADR-033 FLAG-1 resolved+implemented; Gap A float + Gap B integer landed; `vsa.*` (M-892…M-894) + Gap E lift (M-902…M-904) remain `status:todo` |
| 3 | Lowering surface closed | **OPEN** (RFC-0037 Enacted; DN-54 mechanism substantially landed; three maintainer-gated grm decisions outstanding) | `Empirical` | RFC-0037 Enacted 2026-06-27; DN-54 M-812-cont landed; open: consume model, tuple, Fn-field follow-through, grammar baseline (grm M-915…M-924) |
| 4 | KC-3 completeness review | **OPEN — not startable yet** (sequenced after 2 and 3 close; prior DN-39 instances show the bar working) | `Empirical` | DN-39 Accepted (zero promotions); DN-69 ran the bar and PROMOTEd on merit; the completeness-augmented final review has not run |

**Net: 0 of 4 green.** No condition is asserted closed that the evidence does not close (VR-5).
Condition 2 is the closest; condition 1 has a single owning task (M-959); conditions 3 and 4 are
sequenced behind maintainer decisions and the close of 2/3 respectively.

## 3. Per-condition detail — evidence, remainder, owner

### 3.1 Condition 1 — reject-ledger completeness (DN-56 §2.2/§5.2) — OPEN

**What exists today (`Empirical`, inspected):**

- The **parse-level reject corpus**: `docs/spec/grammar/conformance/reject/` — **28 fixtures**
  (vs 25 accept fixtures), each a named forbidden construct (`01-no-nodule-header` …
  `29-missing-semicolon-terminator`, one gap in numbering: no `16-`).
- The corpus is **self-policing**: `crates/mycelium-l1/tests/conformance.rs` carries the
  `REJECT_EXPECTED` table (per-fixture expected-error fragments) and
  `reject_corpus_all_fails_explicitly` fails if any fixture parses, panics, rejects for the wrong
  reason, or **lacks a table entry** — a reject there is an explicit `ParseError`, never a panic
  (G2; `src/error.rs` reifies position + message).
- **Checker-level refusals are explicit but not ledgered**: `CheckError`
  (`crates/mycelium-l1/src/checkty.rs:154`) is a reified, message-carrying refusal used across
  `checkty`/`elab`/`mono`/`grade` (~200 construction/reference sites) with unit and conformance
  tests over individual refusals — but there is **no unified ledger** enumerating every forbidden
  construct with its reason and surface alternative, and **no regression guard** that fails when a
  reject path is added without a ledger row.

**What remains, and its owner:**

| Remainder | Owner |
|---|---|
| The exhaustive `{construct, reason, alternative}` ledger over parse-level AND check-level rejects | **frz M-959** |
| The regression guard: adding a reject path without a ledger row fails a test | **frz M-959** |
| Audit that the 28-fixture corpus covers every *grammar-level* forbidden construct (the DN-56 §7 "reject-corpus completeness audit") | **frz M-959** (with M-958 — this note — as its input) |

### 3.2 Condition 2 — primitive set closed (DN-56 §4/§5.3) — OPEN, near green

**What closed (`Empirical`, inspected):**

- **ADR-033 FLAG-1 is resolved and implemented** (2026-06-28, r4v/M-810): Path A
  (type-carrying hash) landed — `FieldSpec::Fn { sig: FnSig }` in `mycelium-core`, injective
  full-signature encoding, distinct-hash property test + no-match differential green. Soundness
  tag **`Empirical`** (trial-tested, not mechanized — VR-5). ADR-033 header proposes `Enacted`,
  **pending the maintainer's final nod**. The DN-56 §5.3 wording — "in particular ADR-033's FLAG-1
  soundness is resolved" — is satisfied at the `Empirical` strength.
- **Gap A (scalar float) landed through the double gate**: ADR-040 **Accepted** (2026-07-02,
  maintainer-ratified) + DN-69 (**PROMOTE**, the DN-39 four-clause bar argued clause-by-clause and
  ratified) → M-896…M-900 landed on `dev`: `Repr` value form, float literal, `flt.add/sub/mul/div/
  neg`, `flt.lt/le/gt/ge/eq` + `flt.total_le`, three-way conformance closure (PR #990/#991/#992).
- **Gap B (integer arithmetic + signedness) landed**: `bin.div`/`bin.rem` with explicit
  div-by-zero, shifts, and the M-767 signed-op set `div_s`/`rem_s`/`shr_s`/`lt_s` (PR #994 — the
  current `dev` tip), under the DN-72 `_u`/`_s` naming convention (Accepted + enacted 2026-07-02).
- **The Π registry as it stands** (`crates/mycelium-core/src/prim.rs`, inspected): 31 named prims —
  `bin.*` (11), `cmp.*` (3), `dense.*` (6: elementwise + `dot`/`similarity`, M-890/M-891),
  `flt.*` (11).

**What is NOT closed — the FLAG (G2/VR-5, house rule #4):** the spawning directive states *"enb
(primitives) is now COMPLETE on dev (Gap A float + Gap B integer all landed)."* The Gap A + Gap B
half of that claim is **verified true** (above). But the issue ledger (`tools/github/issues.yaml`,
read-only for this leaf) still carries **scheduled Π/frontier additions at `status:todo`**:

- **M-892/M-893/M-894** — the VSA prim group (`vsa.bind`/`unbind`/`permute`, `vsa.bundle`,
  `vsa.cleanup`/`reconstruct`): absent from Π (grep-verified) and `todo` (enb Gap C, second half —
  the dense half landed).
- **M-902/M-903/M-904** — the Gap E Substrate/consume lift (value form, affine tracker, lifting
  the staged `Residual` to executing forms): `todo`. This is L1-frontier work rather than Π
  growth, but lifting a `Residual` moves the accept/reject frontier the census scored — it must be
  landed or explicitly re-scoped before "closed" is honest.

A primitive set with **scheduled, undispositioned additions is not "closed"** — DN-56 §5.3 demands
*no open kernel-primitive question*, and "will `vsa.*` enter Π before freeze?" is exactly such a
question. Either the additions land, or they are **explicitly deferred by a recorded decision**
(post-freeze Π growth would then be a DN-39 promotion or a `core 2.0.0` event — DN-56 §6), or the
issue ledger is stale and needs reconciling. Any of the three is fine; *silence is not*.

**What remains, and its owner:**

| Remainder | Owner |
|---|---|
| Disposition of the VSA prim group (land M-892…M-894, or record an explicit pre-freeze deferral decision) | **enb** (tail) — disposition call: **orchestrator** (FLAG-1 of this note) |
| Disposition of the Gap E lift (M-902…M-904 — land or explicitly re-scope; DN-71 confirmed the model) | **enb** (tail) — same FLAG |
| ADR-033 Accepted → Enacted final nod | **maintainer** (one nod; grm M-922/M-923 carry the surface follow-through) |
| `FieldSpec::Fn` *surface* lowering (Fn-typed record fields in `.myc`) — in or ledgered-refusal | **grm** M-922/M-923 |

### 3.3 Condition 3 — lowering surface closed (DN-56 §2.4/§5.4) — OPEN

**What closed (`Empirical`, inspected):**

- **RFC-0037 is Enacted** (2026-06-27): the grammar epic landed in `mycelium-l1` + `mycelium-fmt`
  (full corpus migrated; 615 + 11 green; `mycelium.ebnf`/editor grammars/api-index regenerated).
  Its two named follow-ons (D2-b short repr keywords; RFC-0025 operator wiring) are explicitly
  non-blocking for the RFC but sit inside grm's stability close-out.
- **DN-54's mechanism is substantially landed**: `lower`/`derive` surfaces active; M-812-cont
  landed the RHS **elaboration to L0 plus the §6 KC-3 kernel-growth guard** (merge `29b0aed`,
  reconciled `5ace333`); the §10 derive-site design-pass addendum authored (`a6a86f3`). DN-54
  itself remains **Accepted** (not Enacted) — its own status header holds the residuals honestly.

**What remains, and its owner (all grm; three of the four are maintainer-decision-gated):**

| Remainder | Owner |
|---|---|
| DN-54 completion audit (verify the landed checks §4.1/§4.2/§6/§7 against DN-54 as written; re-run the §7 harness) | **grm** M-917 |
| Derive-site consumption model — dossier → maintainer ratification → enact in the extension-checker | **grm** M-918/M-919 (maintainer gate) |
| Tuple-type decision (RFC-0024 §4A.8) — dossier → decision → delta (or grounded no-delta) | **grm** M-920/M-921 (maintainer gate) |
| ADR-033 Fn-field lowering follow-through (see 3.2) | **grm** M-922/M-923 (maintainer gate) |
| RFC-0037 D2-b short repr keywords + RFC-0025 residual operator wiring | **grm** M-915/M-916 |
| Grammar-stable baseline close-out (regenerate `mycelium.ebnf`/editor grammars/api-index; window proposed) | **grm** M-924 |

Condition 3 turns green when grm's DoD is met — the extension surface checked
transparent-by-construction (DN-54 → honestly steppable) and every surface feature carrying a
named, verified lowering post-RFC-0037.

### 3.4 Condition 4 — KC-3 completeness review (DN-39/DN-56 §5.5) — OPEN, sequenced last

**What exists today (`Empirical`):** the review *machinery* is ratified and demonstrably working —
DN-39 (Accepted 2026-06-26) fixed the four-clause conjunctive **default-DENY** bar and returned
**zero promotions** on its first run; DN-69 (Accepted 2026-07-02) ran the same bar as a review
instance and granted the float promotion **on merit, clause-by-clause** — the bar's first PROMOTE,
which is evidence the bar discriminates rather than rubber-stamps in either direction.

**What has NOT run:** the DN-56 §5.5 review itself — the kernel-promotion review over the **final**
post-enb/post-grm kernel, *plus the completeness dimension DN-56 adds* (minimality + auditability +
"every accept/reject/variant/invariant enumerated"). It **cannot** run meaningfully before
conditions 2 and 3 close (a completeness review over a still-moving prim set would be vacuous —
VR-5: no upgrade past basis).

**What remains, and its owner:**

| Remainder | Owner |
|---|---|
| The completeness-augmented KC-3 review over the closed prim set + closed lowering surface | **frz** Lane A (orchestrator-run, DN-39 machinery; the last M-969 precondition) |

### 3.5 DN-39 default-DENY — re-affirmed as holding throughout (`Empirical`)

The M-958 DoD requires this stated plainly: **default-DENY has held at every point since DN-39.**
The trusted base has grown exactly twice since the bar was ratified, and both growths went
*through* the bar, not around it: (1) **ADR-033 `FieldSpec::Fn`** — maintainer sign-off-gated
(R2, 2026-06-27), KC-3 growth deliberate and bounded, FLAG-1 resolved before implementation
counted; (2) **the scalar-float value form** — the ADR-038 §2.6 double gate (ADR-040 Accepted
**and** DN-69's four-clause PROMOTE, ratified 2026-07-02). No promotion was waved through; DN-39's
sole other candidate remains KEEP-OUT. Basis: the status headers of DN-39/DN-69/ADR-033/ADR-040,
all inspected — `Empirical`, not `Proven` (no mechanized audit of "no other growth occurred"
exists; the claim rests on the Π inspection in §3.2 plus the corpus record).

## 4. FLAGs raised (never guessed — G2/VR-5)

- **FLAG-1 (condition 2, blocking green):** the "enb COMPLETE" status conveyed to this task
  conflicts with `tools/github/issues.yaml`, which holds M-892/M-893/M-894 (VSA prims) and
  M-902/M-903/M-904 (Gap E lift) at `status:todo`, and with Π, which contains no `vsa.*` entries.
  The orchestrator must **disposition** these (land · explicitly defer by recorded decision ·
  reconcile a stale ledger) before condition 2 can be scored green. This leaf did not edit
  `issues.yaml` (orchestrator-owned).
- **FLAG-2 (condition 2, one nod):** ADR-033 sits at "propose `Enacted`, pending maintainer final
  nod" — a cheap close; recorded so it is not forgotten under the freeze.
- **FLAG-3 (shared files, per the leaf contract):** `CHANGELOG.md`, `docs/Doc-Index.md`,
  `docs/api-index/`, and `tools/github/issues.yaml` (M-958 → done; DN-76 registration) are
  **untouched** by this PR and need the integrating parent's one-time reconciliation.

## 5. Options and recommendation (the delegated decision)

The maintainer delegated acceptance of this scorecard to the orchestrator (2026-07-02, `Declared`
— §Status). The decision: *what does M-969 execute against?*

- **Option A — accept this scorecard as the M-969 gate instrument, as scored (0/4 green today).**
  M-969 executes only when all four rows flip green with cited evidence; FLAG-1's disposition is
  routed to enb/the orchestrator; M-959 proceeds immediately (it depends only on M-958).
- **Option B — narrow condition 2 to "trusted-base growth closed" and score it green now.** This
  would read `vsa.*`/Gap E as frontend-registry work outside the freeze boundary. **Rejected as
  written**: DN-56 §6 freezes "the ten-node budget + the ratified prim set", and Π *is* the
  ratified prim set — scoring it green with scheduled additions undispositioned upgrades the claim
  past its basis (VR-5). If the orchestrator *records the deferral decision*, Option B collapses
  into Option A with FLAG-1 resolved.
- **Option C — hold the scorecard until enb's tail lands.** Safe but strictly worse: it stalls
  M-959 (condition 1's owning task) behind an unrelated lane, and the scorecard is *designed* to
  be re-scored as conditions close.

**RECOMMENDATION: Option A.** Accept the scorecard as the gate instrument now; unblock M-959;
disposition FLAG-1 in the enb lane; re-score each condition append-only (a dated row per flip,
evidence cited) and execute M-969 only on 4/4 green. This is the reading under which the freeze
declaration is "an evidenced act, not a vibe" (M-958 user story) — and it is the only option that
neither stalls Lane A nor upgrades condition 2 past its basis.

## 5A. Re-scoring — current `integration` tip (2026-07-02, `09891ac`; M-958b) — 4 of 4 GREEN

> **Posture (transparency rule / VR-5 / G2).** This section is an **independent freeze-gate
> re-score** by a separate assessor (M-958b), deliberately not part of the wave that landed the
> `grm`/`frz` work — its job is to catch any condition that is **not** genuinely green, not to close
> the wave. Every verdict is **`Empirical`** — an evidence-based read of the repository at the
> **`integration` tip `09891ac`** (PR #1048, "promote grm-frz-lang to integration"), citing the
> artifacts inspected and the tests run. Nothing is `Proven`. The §2 scorecard (0/4 at `dev`
> `629aa12`) **stands unchanged above** — this is an append-only re-scoring, not a rewrite (house
> rule #3). **This section scores the gate green; it does not declare the freeze** — M-969 is a
> maintainer-class act executed by the orchestrator, and two orchestrator-owned *hygiene* items
> (FLAG-A/FLAG-B below) should be reconciled at the freeze close-out even though neither fails a
> condition.

| # | Condition | Verdict | Tag | One-line basis (current tip) |
|---|---|---|---|---|
| 1 | Reject-ledger completeness | **GREEN** | `Empirical` | DN-80 is the unified `{construct, reason, alternative}` ledger across all four reject strata (parse · check · ambient · runtime-kernel), `Accepted`; the regression guard `reject_ledger.rs` is present and **green** (9 tests, run below) |
| 2 | Primitive set closed | **GREEN** | `Empirical` | DN-76 FLAG-1 fully resolved — `vsa.*` (M-892…894) + Gap E lift (M-902…904) all `status:done`; the 7 `vsa.*` prims are in Π (`prim.rs`) and implemented in interp; the affine tracker is active; ADR-033 FLAG-1 dispositioned by DN-74 (primitive **IN** at `Empirical`) |
| 3 | Lowering surface closed | **GREEN** | `Empirical` | RFC-0037 Enacted; all three grm decisions resolved (DN-71/DN-73/DN-74); every grm *implementation* landed in code (merged PRs #1026/#1029/#1032/#1033 — verified ancestors of HEAD); grammar baseline in sync (`drift.sh` + `grammar.sh` green); DN-54 extension checks §4.1/§6/§7 verified **Landed** by DN-75 (Resolved) |
| 4 | KC-3 completeness review | **GREEN** | `Empirical` | The completeness-augmented KC-3 review was **run** here (§5A.4): the completeness gate (`runnable_gate.rs`) + the reject ledger are green; the trusted-core boundary matches DN-39 §7; every Π growth since DN-39 went through a ratified gate (no silent kernel growth); default-DENY held |

**Net: 4 of 4 GREEN.** The scorecard is green and **M-969 is unblocked** on its evidence. Two
orchestrator-owned bookkeeping items (FLAG-A/FLAG-B) do **not** fail any condition but should be
reconciled for an honest close-out (§5A.5).

### 5A.1 Condition 1 — reject-ledger completeness — GREEN

- **DN-80** (`docs/notes/DN-80-Reject-Ledger-Exhaustive-Never-Silent-Refusal-Inventory.md`,
  **Accepted** 2026-07-02) is the unified `{construct, reason, alternative}` ledger DN-56 §2.2/§5.2
  calls for, spanning the four reject strata: parse (`ParseError` — 30 fixtures), check
  (`CheckError`), ambient (`AmbientError`), and runtime/kernel (`EvalError`/`WfError`). It closes the
  exact gap the §2 scorecard named ("checker-level refusals explicit but not ledgered … no
  regression guard").
- The regression guard `crates/mycelium-std-conformance/tests/reject_ledger.rs` is present and
  **green**: `cargo test -p mycelium-std-conformance --test reject_ledger` → **9 passed, 0 failed**
  (parse-corpus fixture-set match; checkty/grade/fuse `CheckError` construction-count audits; and —
  strongest — `BTreeSet` closed-enum equality over `WfError`/`EvalError`/`AmbientError` variants,
  which catches any add/remove/rename exactly). Honesty (per the test's own header, VR-5): the
  *count* assertions are a line/regex heuristic over source text (source is ground truth); the
  closed-enum assertions are semantically exact.

### 5A.2 Condition 2 — primitive set closed — GREEN

- The §2 scorecard's **FLAG-1** (blocking green) is fully resolved. In `tools/github/issues.yaml`,
  **M-892/M-893/M-894** (the `vsa.*` group) and **M-902/M-903/M-904** (the Gap E Substrate/consume
  lift) are all **`status:done`** — not `todo`.
- The prims exist in the trusted base, not just on the ledger: Π (`crates/mycelium-core/src/prim.rs`)
  now carries the 7 `vsa.*` entries (`vsa.bind`/`unbind`/`permute`/`bundle`/`cleanup`/`reconstruct`/
  `required_dim`, grounded in RFC-0003 §3-6 / ADR-008), registered + implemented in
  `crates/mycelium-interp/src/prims.rs` (`r.register("vsa.bind", …)` etc.). Π totals **38** named
  prims (`bin.*` ×11, `cmp.*` ×3, `dense.*` ×6, `flt.*` ×11, `vsa.*` ×7).
- The Gap E lift landed in the frontend: the affine use-once tracker `crates/mycelium-l1/src/affine.rs`
  (M-903) is **active** in the checker (`checkty.rs` — "the active M-919 affine tracker", `use
  crate::affine::{Tracker, UseOutcome}`); `Substrate` value form + the staged-`Residual` lift are
  present.
- ADR-033 FLAG-1 (the last open primitive-set question) is **dispositioned** by **DN-74** (Accepted):
  Option A — `FieldSpec::Fn` full-signature encoding is the final answer, the primitive is **IN** at
  `Empirical` strength. So DN-56 §5.3's "no open kernel-primitive question remains" is satisfied: the
  in/out decision is made, not pending.
- **Residuals (deferred-with-disposition, not freeze-blocking):** (i) ADR-033's *status label* still
  reads "propose `Enacted`, pending maintainer final nod" — a bookkeeping step, not an open prim
  question (DN-74 already decided the primitive is IN); (ii) **M-805** (multi-kernel question,
  RFC-0036) is a **Phase-6 P3 `capture`** item — a parked future consideration, not a Phase-5 freeze
  blocker and not a question about the *current* frozen prim set.

### 5A.3 Condition 3 — lowering surface closed — GREEN (with a ledger-hygiene FLAG-A)

- **RFC-0037 is Enacted** (grammar epic landed; `mycelium.ebnf`/editor grammars/api-index
  regenerated).
- **The three previously-maintainer-gated grm decisions are resolved:** DN-71 (Substrate/consume
  Model S, ratified), DN-73 (tuple type), DN-74 (ADR-033 Fn-field FLAG-1). Their decision dossiers
  (M-917/M-918/M-920/M-922) are all `status:done`.
- **Every grm *implementation* landed in code** (verified as ancestors of HEAD, merged into
  `integration`): M-915 short-repr keywords (`bin`/`tern`/`emb`/`hvec` → `*Short` tokens in
  `token.rs`); M-916 operator wiring (verified — no residual, `f48b7e7`, PR #1026); M-919 consume
  checker (affine tracker active in `checkty.rs`); M-921 tuple delta (PR #1029, `867bf55`); M-923
  `FieldSpec::Fn` surface lowering + program-level differential (`d0094de`, PR #1032); M-973 DN-54
  §10 attachment Model A (`6a8df01`, PR #1033, wired through the M-919 affine tracker,
  `derive_site_double_consume` red-then-green).
- **Grammar baseline recorded + in sync (M-924):** `scripts/checks/drift.sh` **green** (grammar
  artifacts + `tools/grammar/` current with the lexer `keyword()` table); `scripts/checks/grammar.sh`
  **green** (27 accept, 30 reject fixtures well-formed).
- **User extensions (DN-54) checked transparent-by-construction:** DN-75 (the DN-54 completion audit,
  **Resolved**) verified against the *tree* — §4.1 RHS type-check **Landed** (`checkty.rs:2039-2070`),
  §6 KC-3 kernel-growth guard **Landed** (`elab.rs` codomain = the closed `mycelium_core::Node`
  enum), §7 harness **re-run green**; `elab` reads `Env::lower_rules` (`elab.rs:567`). DN-75's
  disposition **keeps DN-54 `Accepted`** deliberately (not stepped to Enacted).
- **Residuals the §2 scorecard §3.3 flagged (deferred-with-disposition, routed by DN-75, never
  silent):** the DN-54 §7 re-run harness is done; R-2 (`reveal`-ability of derive output) is **gated
  on the DN-38 §5 `reveal` track** — the by-construction argument stands (`Declared`); R-3 (§7.1
  differential as a *generated, DN-20-tiered* corpus vs the landed fixed 4-case table) is a testing
  refinement. None break the never-silent guarantee (the completeness gate below covers them), so
  none is freeze-blocking.
- **FLAG-A (ledger hygiene — orchestrator-owned `issues.yaml`, not editable here).** The issue
  statuses for **M-915/M-916/M-919/M-921/M-923** still read **`status:blocked`** ("serial L1 lane —
  after enb"), while the code for all five **landed** as the merged PRs cited above. The statuses are
  **stale** — they should be stepped to `done`. This is a *bookkeeping* discrepancy, not a functional
  gap (the lowering surface is closed in code, which is ground truth), **but** a freeze declared
  while the tracker shows five prerequisite P1 issues "blocked" is internally inconsistent with "an
  evidenced act, not a vibe" — so the orchestrator should reconcile these before the M-969 close-out.

### 5A.4 Condition 4 — KC-3 completeness review — GREEN (review run here)

The completeness-augmented DN-39 KC-3 review, run over the now-closed prim set + lowering surface:

- **Completeness — every accept accounted.** The DN-50 OQ-2 standing gate
  `crates/mycelium-l1/tests/runnable_gate.rs::every_accepted_construct_elaborates_to_ok_or_explicit_residual`
  is **green** (`cargo test -p mycelium-l1 --test runnable_gate` → 1 passed). It is a data-driven
  table over the DN-52 census construct categories (~19 rows — literals, ops, swaps, `let`, calls,
  data+match, recursion, colony/hypha, generics, trait impls, fold sugar, plus the two explicit-Residual
  cases: dense-swap target, `wild` non-host-call), each asserting `elaborate` → `Ok(node)` OR
  `Err(ElabError::Residual{..})` — never a silent accept-but-unrunnable. `Empirical` (a representative
  table + the three-way corpus; not an exhaustive proof over all programs — stated plainly, as the
  test does).
- **Completeness — every reject accounted.** The DN-80 ledger + `reject_ledger.rs` (§5A.1) close this,
  including exact closed-enum coverage of `WfError`/`EvalError`/`AmbientError`.
- **KC-3 minimality + auditability.** The trusted-core boundary matches **DN-39 §7** unchanged: L0
  Core IR + the reference interpreter (`mycelium-core`/`mycelium-interp`/`mycelium-l1`) + the
  content-addressing primitive + the guarantee lattice + the swap engine; `mycelium-spore` and the
  AOT/MLIR path stay **outside** the TCB (verified consumers / validated-not-trusted). No boundary
  move occurred.
- **KC-3 growth deliberate + bounded — default-DENY held, no silent kernel growth.** Every Π/trusted-base
  growth since DN-39 traces to a ratified gate: `flt.*` via **DN-69 PROMOTE** (the DN-39 four-clause
  bar argued clause-by-clause) + **ADR-040** (the ADR-038 §2.6 double gate); `FieldSpec::Fn` via
  **ADR-033** R2 maintainer sign-off + **DN-74** FLAG-1 disposition; `vsa.*` as the enactment of the
  already-ratified **RFC-0003/ADR-008** paradigm prim set (M-892…894); `bin.*`/`dense.*` per
  RFC-0032/DN-72. **Honest note (VR-5):** unlike `flt.*` (DN-69), the `vsa.*` surfacing did **not**
  get a fresh standalone DN-39 promotion dossier — its basis is the prior RFC-0003/ADR-008
  ratification, not a new KC-3 bar run; it is deliberate and never-silent, but that is the strength of
  its basis (`Empirical`, spec-grounded), stated so it is not over-read as a fresh adjudication. User
  extensions (DN-54) add **no** kernel growth by construction — the §6 guard fixes the elaborator's
  codomain to the closed `mycelium_core::Node` enum, so a user `lower`/`derive` cannot introduce a new
  L0 node.
- **No construct silently unhandled.** `runnable_gate` (accepts) and `reject_ledger` (rejects)
  jointly enforce "every accept → OK or explicit `Residual`; every reject → ledgered refusal" — G2
  held throughout.
- **Verdict: PASS.** The kernel passes the completeness-augmented KC-3 review. Tagged `Empirical`
  (established via the two green standing gates + the boundary inspection + the ratified-growth trace;
  **not** `Proven` — there is no mechanized proof that *no other* growth occurred, exactly as DN-39 §9
  tags the kernel-at-large). No genuine gap was found: no construct that fails to elaborate, no silent
  kernel growth, no unhandled case.

### 5A.5 FLAGs (never guessed — G2/VR-5)

- **FLAG-A (condition 3 — ledger hygiene, does NOT fail the condition; orchestrator-owned).**
  M-915/M-916/M-919/M-921/M-923 are `status:blocked` in `issues.yaml` while their code landed
  (merged PRs #1026/#1029/#1032/#1033 + `token.rs`/`checkty.rs`/`elab.rs`). Step them to `done`
  before the M-969 close-out so the tracker matches the frozen reality. `issues.yaml` is read-only for
  this leaf.
- **FLAG-B (condition 3 — doc hygiene, does NOT fail the condition).** DN-54's *status header* still
  lists RHS-elaboration / §4.1 / §6 as "NOT yet implemented (deferred to M-812-cont)", but DN-75's
  audit confirms all three **Landed** on the tree. DN-75 E-1 already dispositioned this as a one-line
  editorial for DN-54's next changelog — apply it at close-out. (DN-54 correctly stays `Accepted` per
  DN-75.)
- **FLAG-C (condition 2 — one nod, carried forward from §4 FLAG-2).** ADR-033's Accepted → Enacted
  label step is still pending the maintainer's final nod. DN-74 already decided the primitive is IN,
  so this does not gate condition 2; recorded so it is not lost under the freeze.
- **FLAG-D (shared files, per the leaf contract).** `CHANGELOG.md`, `docs/Doc-Index.md`,
  `docs/api-index/`, and `tools/github/issues.yaml` (M-958b close-out; the FLAG-A status steps) are
  **untouched** by this PR and need the integrating parent's one-time reconciliation.

**Bottom line (M-958b, `Empirical`): the four-condition scorecard is 4 of 4 GREEN at `integration`
`09891ac`, so M-969 is unblocked on the evidence. FLAG-A and FLAG-B are close-out hygiene the
orchestrator owns — reconcile them so the freeze is declared over a tracker that matches the frozen
code. This section scores the gate; it does not declare the freeze.**

## 6. Grounding

**DN-56** (the gate this scores; §5 conditions, §6 what freeze forbids, §7 the open ledger) ·
**DN-39** + **DN-69** (the KC-3 bar + its two adjudications: KEEP-OUT, PROMOTE) · **ADR-033**
(FLAG-1 resolution record) · **ADR-040/DN-72** (Gap A/B ratifications) · **RFC-0037** (Enacted —
the grammar migration) · **DN-54** (extension surface; its own honest status header) · **DN-50/
DN-52** (the frontier + census behind §5.1, already satisfied) · kickoffs **`frz`** (Lane A:
M-958/M-959/M-969), **`enb`** (Gaps A/B/C/E), **`grm`** (M-915…M-924) · repo evidence at `dev`
`629aa12`: `crates/mycelium-core/src/prim.rs` (Π), `crates/mycelium-l1/tests/conformance.rs` +
`docs/spec/grammar/conformance/reject/` (the reject corpus), `crates/mycelium-l1/src/checkty.rs`
(`CheckError`), PRs #976/#979/#981/#990–#994. House rules **1** (tags), **3** (append-only),
**4** (grounded assent; the FLAG against the conveyed status), **G2**, **VR-5**, **KC-3**.

## Changelog

| Date | Status | Note |
|---|---|---|
| 2026-07-02 | **Recommended, pending orchestrator acceptance** | Authored (M-958, kickoff `frz` Lane A) as the DN-56 four-condition freeze scorecard at `dev` `629aa12`. All four conditions scored `Empirical` with cited evidence: 0/4 green (1 reject-ledger OPEN → M-959; 2 primitive set OPEN-near-green → FLAG-1 disposition + ADR-033 nod; 3 lowering surface OPEN → grm M-915…M-924; 4 KC-3 completeness review OPEN → frz Lane A, sequenced last). DN-39 default-DENY re-affirmed as holding throughout (`Empirical`). Decision delegated by the maintainer to the orchestrator (2026-07-02, `Declared`); recommendation: **Option A** — accept as the M-969 gate instrument, unblock M-959, disposition FLAG-1, execute M-969 only on 4/4 green. Not self-ratified; enacts nothing; freezes nothing. |
| 2026-07-02 | **Accepted** | Accepted by the wave orchestrator at the integration-reconcile promotion gate, under the maintainer's 2026-07-02 delegation (`Declared`). **Option A** adopted: this scorecard is the M-969 four-condition freeze-gate instrument. Acceptance does **not** freeze the kernel — the score stands at 0/4 green, so M-969 remains gated (held/not closed). FLAG-1 disposition is carried by DN-74 (Accepted, same date). Forward transition, append-only (house rule #3); nothing frozen, no tag upgraded (VR-5/G2). |
| 2026-07-02 | **Accepted** (re-scored; instrument status unchanged) | **Independent re-score (M-958b) at the `integration` tip `09891ac` (PR #1048): 4 of 4 GREEN** — recorded append-only in **§5A** (the §2 0/4 section stands). Condition 1 GREEN (DN-80 unified ledger + `reject_ledger.rs` 9 tests green); condition 2 GREEN (FLAG-1 fully resolved — `vsa.*` M-892…894 + Gap E M-902…904 all `done`, 7 `vsa.*` prims in Π, affine tracker active, ADR-033 FLAG-1 dispositioned IN by DN-74); condition 3 GREEN (RFC-0037 Enacted; grm decisions DN-71/DN-73/DN-74 resolved; all grm implementations landed as PRs #1026/#1029/#1032/#1033; grammar baseline in sync via `drift.sh`+`grammar.sh`; DN-54 §4.1/§6/§7 verified Landed by DN-75); condition 4 GREEN — the completeness-augmented KC-3 review was **run** (`runnable_gate.rs` + `reject_ledger.rs` green; DN-39 §7 boundary unchanged; no silent kernel growth; default-DENY held) → **PASS**. Net: **M-969 unblocked on the evidence**; the freeze is **not** declared here (orchestrator's delegated act). Non-blocking hygiene FLAGs for close-out: FLAG-A (M-915/916/919/921/923 `issues.yaml` statuses stale at `blocked` while code landed — step to `done`), FLAG-B (DN-54 status header stale re: M-812-cont completions — DN-75 E-1 editorial). All verdicts `Empirical`; nothing frozen, no tag upgraded (VR-5/G2); DN-76 stays `Accepted` (the instrument). |
