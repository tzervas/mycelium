# Design Note DN-76 — Kernel-Freeze Four-Condition Scorecard (the M-969 gate instrument)

| Field | Value |
|---|---|
| **Note** | DN-76 |
| **Status** | **Recommended, pending orchestrator acceptance** (2026-07-02) — the maintainer **delegated this decision to the orchestrator** (2026-07-02, orchestrator-relayed session directive; the delegation record is `Declared` — it is asserted by the spawning directive, not written into the corpus). This note is **not self-ratified**: it scores, recommends, and stops. Acceptance is the orchestrator's act; the freeze declaration itself (M-969) remains a **maintainer-class act executed by the orchestrator only when this scorecard is green**, per the frz kickoff Lane A. |
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
