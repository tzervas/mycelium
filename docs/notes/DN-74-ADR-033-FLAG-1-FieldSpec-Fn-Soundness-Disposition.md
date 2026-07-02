# Design Note DN-74 — ADR-033 FLAG-1 Disposition: `FieldSpec::Fn` Soundness, the Kernel-Freeze Primitive-Set Condition

| Field | Value |
|---|---|
| **Note** | DN-74 |
| **Status** | **Accepted** (2026-07-02 — accepted by the wave orchestrator under the maintainer's 2026-07-02 delegation (`Declared`), per the integration-reconcile promotion gate; **Option A** ratified as the FLAG-1 disposition: Path A full-signature encoding is the final answer, the primitive is IN at `Empirical` strength, and the surface lowering + program-level differential ride M-923 (held/blocked, not closed here). This note does **not** itself step ADR-033's status. Was **Recommended, pending orchestrator acceptance** 2026-07-02; that history stands unchanged below — append-only forward transition, house rule #3.) The maintainer **delegated this decision to the orchestrator** (2026-07-02, per the M-922 tasking directive; `Declared`). |
| **Decides** | *Proposes, for orchestrator acceptance:* the disposition of **ADR-033 FLAG-1** — the `FieldSpec::Fn` (Fn-typed record-field) soundness question — which gates DN-56 §5's **primitive-set-closure freeze condition** (the last open primitive-set question). Recommendation: **Option A** (§6) — ratify the already-landed Path A type-carrying encoding as the final FLAG-1 resolution, count the primitive **in** the freeze boundary at its honest `Empirical` tag, and direct M-923 to land the surface lowering plus the program-level differential; with **Option B** (kernel-in, surface stays a ledgered explicit refusal) as the named fallback if M-923 slips the H2a window. |
| **Task** | M-922 (kickoff `grm`, Phase-I H2a); unblocks M-923 + the DN-56 primitive-set condition |
| **Depends on** | ADR-033 (esp. §10 — the Path A/Path B analysis and the 2026-06-28 in-session ratification + implementation); DN-56 §4/§5 (freeze boundary + gate); RFC-0019 §4.5 (the dynamic-dispatch normative target); DN-55 (polymorphism costs zero kernel primitives — dispatch is the one exception); KC-3; G2; VR-5; `docs/planning/Blocked-Decisions-Ratification-Map.md` groups G4/G5 |
| **Date** | 2026-07-02 |

> **Posture (transparency rule / VR-5 / G2).** The landed-state inventory (§2–§3) is `Exact`
> (source-read, file:line cited) or `Empirical` (test-run green, re-verified 2026-07-02). The
> options analysis (§5) and recommendation (§6) are `Declared`-with-argument. Nothing in this note
> is `Proven`: the kernel encoding's injectivity-over-typed-structure remains unmechanized, and
> per VR-5 no claim below upgrades past its checked basis. Where a numbering discrepancy exists
> between source documents (§1.2), it is surfaced, not silently normalized.

---

## 1. What this dossier is deciding — and one numbering correction

### 1.1 The question

DN-56 §4 fixes the freeze boundary: the ten L0 nodes + the ratified prims + the value reprs, and
**one** candidate addition — `FieldSpec::Fn` (ADR-033), the single polymorphism form that cannot
monomorphize away (DN-55). DN-56 §5's primitive-set-closure condition requires that no open
kernel-primitive question remain: *"ADR-033's FLAG-1 soundness is resolved (a checked soundness
argument, or the type-descriptor variant), so `FieldSpec::Fn` is either in (sound) or out
(deferred), not pending."*

FLAG-1 (ADR-033 §6) was the observation that the original `FieldSpec::Fn { arity }` design hashed
only the parameter *count*, so two function types of equal arity but different parameter/return
types collided on content identity — letting a type-confused dictionary projection through the
kernel silently (the precise exploit shape is ADR-033 §10.1). This dossier's job (M-922) is to
frame the disposition of that question with its soundness evidence so the freeze gate has a
decided primitive-set condition.

### 1.2 Numbering discrepancy, surfaced (G2)

The `grm` kickoff (M-922 row and its prerequisites table) calls this **"DN-56 kernel-freeze
condition 2"**. DN-56 §5 as written numbers the conditions: #1 census, **#2 reject ledger**,
**#3 primitive set closed** (the ADR-033 FLAG-1 condition), #4 lowering surface, #5 KC-3 review.
The FLAG-1-gated condition is therefore **DN-56 §5 condition #3** by the source document's own
numbering; the kickoff's "condition 2" is an off-by-one label for the same item (`Exact` — both
texts quoted from source). This note uses "the primitive-set-closure condition" throughout to stay
unambiguous, and flags the kickoff-table label for correction by its owner (FLAG-5, §7).

---

## 2. Verified landed state — the kernel side (verification-first, per the kickoff discipline)

**The soundness fix ADR-033 §10 recommended (Path A, type-carrying hash) is already implemented in
`mycelium-core`, ratified in-session by the maintainer on 2026-06-28, and green today.** This
dossier re-verified rather than assumed it (the 2026-06-29 serial closeout landed adjacent work;
nothing is re-proposed here that already landed):

| Item | State | Basis |
|---|---|---|
| `FieldSpec::Fn { arity, sig: FnSig }` with `FieldTyRef` leaves (`Repr` / `Data` / nested `Fn`) | Landed | `Exact` — `crates/mycelium-core/src/data.rs:145–184` |
| Full-signature injective encoding: `FIELD_FN` + arity + `FN_SIG_PARAMS`/`FN_SIG_RET` + `FTR_REPR`/`FTR_DATA`/`FTR_DATA_CYCLE`/`FTR_FN` tags | Landed | `Exact` — `data.rs:533–592` |
| Well-formedness: `arity == params.len()` checked at build, never-silent (`RegistryError::FnArityMismatch`), recursively through nested sigs | Landed | `Exact` — `data.rs:385–447` |
| FLAG-3 widening (ADR-033 §10.6 Q3): SCC/`succ` + cycle ordering traverse `Data` edges *inside* `Fn` signatures; in-cycle sig leaves use the cycle placeholder | Landed | `Exact` — `data.rs:648–763` (`collect_data_refs_*`) |
| §10.6 Q1 (return type IS encoded — full signature, not params-only) | Landed as recommended | `Exact` — `data.rs:557` + test `fn_distinct_return_reprs_hash_distinctly` |
| §10.6 Q2 (full `FieldTyRef` leaves, not the cheaper `Repr`-only variant) | Landed as recommended | `Exact` — `data.rs:145–151` |
| Property/regression coverage: 12 `fn_*` tests — distinct-param-repr hashes, distinct-return-repr hashes, `FIELD_FN` vs `FIELD_REPR` non-collision, stability/determinism, arity-mismatch explicit error, dangling sig ref explicit error, self-recursive sig no-loop, out-of-cycle sig resolution, nested higher-order sig distinctness, repr-only sig creates no declaration dep | Green | `Empirical` — `crates/mycelium-core/src/tests/data.rs`; `cargo test -p mycelium-core` **306 + 11 passed, 0 failed**, re-run 2026-07-02 |

**Consequence for the soundness hole:** the §10.1 collision (`MkDict_Eq8` ≡ `MkDict_Eq16` on
content identity) is closed at the kernel level — distinct signatures produce distinct declaration
hashes, so their `CtorRef`s differ and a `Match` against one cannot silently match a value of the
other. Tag: **`Empirical`** (trial-tested via the property suite above; **not `Proven`** — the
injectivity-over-typed-structure argument is the disjoint-tag reasoning of ADR-033 §2.3/§10.2,
unmechanized; VR-5 forbids the upgrade).

---

## 3. Verified landed state — what is NOT landed (the honest gap inventory)

The kernel primitive exists; **nothing produces it yet, and the ADR's own Enacted bar is not met**:

1. **No surface lowering.** `mycelium-l1`'s `field_spec` maps `Ty::Fn(_, _)` in a field position
   to `None`, so the owning declaration is skipped and any use surfaces as an explicit `Residual`
   ("staged", never silent — G2 holds today). `FieldSpec::Fn` is **never constructed by
   elaboration**; its only producers are unit tests. `Exact` —
   `crates/mycelium-l1/src/elab.rs:836–841` (comment cites RFC-0024 §4 / M-687 staging).
2. **No program-level three-way differential.** ADR-033 §5.1's L1 ≡ L0 ≡ AOT differential over a
   dynamic-dispatch program does not exist (`crates/mycelium-l1/tests/differential.rs` has no
   dictionary/Fn-field case — it cannot, while item 1 holds). The never-silent cross-typed
   no-match property is covered at the **kernel unit level** only. `Exact` (test-corpus grep).
3. **ADR-033 status is "propose `Enacted`, pending maintainer final nod"** (its header, since
   2026-06-28) — i.e. the FLAG-1 *resolution* is ratified and implemented, but the ADR's §8 + §10.7
   Enacted checklist (which includes item 2 above) is not fully discharged, so the status has
   honestly not stepped. `Exact` — ADR-033 header + changelog.
4. **Soundness tag ceiling.** The strongest supportable claim is `Empirical`. A mechanized
   injectivity proof over the encoding (the ADR's named upgrade path to `Proven`) has not been
   attempted. `Exact` (absence verified by search; no proof artifact exists under `proofs/`).

---

## 4. The decision surface

Given §2–§3, "resolve FLAG-1 for the freeze gate" decomposes into three sub-decisions:

- **D1 — Is the FLAG-1 soundness question itself closed?** (Is Path A, as landed, the accepted
  resolution — or does the freeze demand more, e.g. mechanization?)
- **D2 — Is `FieldSpec::Fn` in or out of the frozen kernel?** (DN-56 demands "in (sound) or out
  (deferred), not pending".)
- **D3 — What must M-923 land, and what does ADR-033's status honestly become?** (Surface lowering
  vs ledgered refusal; `Enacted` only when its own checklist is genuinely met.)

---

## 5. Options

### Option A — Ratify Path A as final; primitive IN; M-923 lands the surface lowering (RECOMMENDED)

**D1:** accept the landed Path A encoding as the final FLAG-1 resolution. DN-56's condition text
offers two closing routes — "a checked soundness argument, **or the type-descriptor variant**" —
and Path A **is** the type-descriptor variant, landed and `Empirical`-verified (§2). The condition
is satisfiable *as written* without a mechanized proof; VR-5 requires the honest tag, not the
maximal one.
**D2:** `FieldSpec::Fn` is **in** the freeze boundary, at tag `Empirical`.
**D3:** M-923 lands, in order: (i) the `Ty::Fn`-field lowering in `field_spec` (building the
`FnSig` from the checked type — the checker already carries `Ty::Fn(param, ret)`); (ii) the §5.1
three-way differential over a minimal dictionary-dispatch program; (iii) a program-level
never-silent no-match conformance case (build `MkDict_Eq8`, project under an `Eq16`-typed match,
assert explicit error); (iv) only then step ADR-033 → `Enacted` (its §8 + §10.7 checklist met).
Items (ii)+(iii) green lift the dispatch-soundness claim from kernel-unit `Empirical` to
program-level `Empirical` — still not `Proven`, and honestly so.

*For:* closes the last primitive-set question with the evidence already in hand; unblocks
RFC-0019 §4.5 (heterogeneous immutable collections — the driver ADR-033 §1.2 documents); the
identity-bearing variant is already permanent (removing it would be a *new* trusted-core change),
so "in" matches physical reality; the freeze gate's own wording is met.
*Against / risk:* M-923 is serial-lane work gated on this acceptance; if it slips, H2a's close-out
(M-924) waits on it. Mitigated by the Option B fallback being pre-named (below).

### Option B — Kernel-in, surface refusal ledgered (the honest fallback)

**D1/D2** as Option A. **D3:** instead of the lowering, M-923 records the *current* explicit
`Residual` as a **reject-ledger row** (a refusal is a decision, not a leftover — the M-923 DoD
already permits this branch): fn-typed record fields refuse explicitly at the surface until a
named follow-on lands the lowering. ADR-033 stays `Accepted` with an append-only note; the freeze
condition is still closed ("in (sound)" at the kernel; the surface path is a ledgered, dated
refusal, not a pending question).
*For:* zero serial-lane cost inside H2a; every G2 obligation already holds today (§3 item 1).
*Against:* RFC-0019 §4.5 stays inexpressible from the surface; object-style ports hit the refusal;
"in the kernel but unreachable from the language" is a weaker freeze claim (though an honest one).

### Option C — Rule the primitive OUT (supersede ADR-033, remove the variant) — NOT recommended

*Against, decisively:* the variant is landed, tested, and **identity-bearing** — ADR-033 §1.3
records that a `FieldSpec` variant is a permanent, append-only trusted-base change; removal would
itself be a fresh trusted-core modification invalidating existing hashes, a larger KC-3 event than
keeping it. It would re-block RFC-0019 §4.5 with no soundness gain (the soundness hole is already
closed). It would also discard a maintainer ratification (2026-06-28) without new disconfirming
evidence — none exists. Listed for completeness because DN-56 names "out (deferred)" as a legal
disposition; the facts on the ground make it strictly worse than B.

### Option D — Hold the condition open pending a mechanized `Proven` injectivity proof — NOT recommended

*Against:* over-strong relative to the gate's own text (which accepts the type-descriptor
variant); inverts VR-5's direction (VR-5 says don't *upgrade* past the basis — it does not say
refuse to *decide* at an honest `Empirical`); and starves the freeze on an open-ended proof task
no roadmap item owns. The mechanization stays on the books as the named upgrade path (ADR-033
§10.5), not as a gate.

---

## 6. Recommendation

**Adopt Option A**, with **Option B as the pre-named fallback** if M-923 cannot land inside the
H2a window: ratify the landed Path A type-carrying encoding as the final ADR-033 FLAG-1
disposition; count `FieldSpec::Fn` **in** the frozen primitive set at tag **`Empirical`** (kernel
property suite green, re-verified 2026-07-02; not `Proven` — unmechanized); direct M-923 to land
the surface lowering + program-level differential + never-silent cross-typed no-match case, and
step ADR-033 → `Enacted` only when its own §8/§10.7 checklist is genuinely complete. Either
branch (A, or B's ledgered refusal) closes DN-56's primitive-set-closure condition as "decided,
not pending" — which is exactly what the freeze gate demands.

Grounds: the gate's own wording names the landed variant as a sufficient closing route (`Exact`,
DN-56 §5 quoted in §1.1); the soundness evidence is checked and green (`Empirical`, §2); the
irreversibility asymmetry between A/B and C is structural (`Exact`, ADR-033 §1.3); the residual
gaps are enumerated and assigned (§3 → M-923), none silent (G2).

**Acceptance protocol (not self-ratified):** the orchestrator, holding the maintainer's 2026-07-02
delegation, accepts or amends this recommendation; on acceptance, this note's status steps to
`Accepted` (append-only), M-923 is unblocked with the D3 work-list above, and the map-group and
DN-56 bookkeeping lands via the FLAGs below — by their owners, not by this leaf.

---

## 7. FLAGs (files this leaf does not own — integration applies these once)

- **FLAG-1:** `docs/planning/Blocked-Decisions-Ratification-Map.md` groups **G4/G5** — record this
  dossier + the disposition once accepted (the map's own note: ratify FLAG-1 once, not twice).
- **FLAG-2:** **DN-56** — on acceptance, an append-only §7/changelog note marking the primitive-set
  condition's ADR-033 dependency as resolved-by-DN-74 (owner-applied; this note does not edit DN-56).
- **FLAG-3:** **ADR-033** — on acceptance, an append-only changelog row citing DN-74 as the
  delegated FLAG-1 disposition record; status step to `Enacted` only via M-923's checklist.
- **FLAG-4:** `CHANGELOG.md`, `docs/Doc-Index.md`, `tools/github/issues.yaml` (M-922 status),
  `docs/api-index/` — orchestrator-owned; not touched here.
- **FLAG-5:** kickoff `grm` prerequisites/M-922 rows label the primitive-set condition
  "condition 2"; DN-56 §5 numbers it #3 (§1.2) — the kickoff owner should reconcile the label.
- **FLAG-6:** `docs/notes/DN-73` is unallocated on this branch tip; DN-74 was assigned by the
  tasking directive (presumably DN-73 is reserved for a sibling dossier leaf). If DN-73 goes
  unused, that is a numbering gap to note, not an error.

---

## 8. Grounding

| Claim | Tag | Basis |
|---|---|---|
| DN-56's primitive-set condition accepts "the type-descriptor variant" as a closing route | `Exact` | DN-56 §5 (quoted §1.1) |
| The kickoff's "condition 2" label vs DN-56 §5's #3 numbering | `Exact` | Both texts read from source (§1.2) |
| Path A (full-sig `FieldSpec::Fn`) is landed in the kernel, including Q1/Q2/Q3 as §10.6 recommended | `Exact` | `data.rs:145–184, 385–447, 533–592, 648–763`; §2 table |
| The kernel property suite covering the FLAG-1 collision is green | `Empirical` | `cargo test -p mycelium-core`: 306 + 11 passed, 0 failed (2026-07-02 re-run); 12 `fn_*` tests in `src/tests/data.rs` |
| The §10.1 type-confusion hole is closed at the kernel level | `Empirical` | Distinct-signature distinct-hash + tag non-collision tests; **not `Proven`** (unmechanized — VR-5) |
| No surface lowering exists; fn-typed fields are an explicit staged `Residual` | `Exact` | `elab.rs:836–841` (`Ty::Fn(_, _) => return None`) |
| No program-level dispatch differential exists | `Exact` | `differential.rs` corpus grep (no Fn-field/dictionary case) |
| ADR-033 status is propose-`Enacted`, pending final nod | `Exact` | ADR-033 header + changelog (2026-06-28 rows) |
| The maintainer delegated this decision to the orchestrator (2026-07-02) | `Declared` | The M-922 tasking directive (recorded there; no doc-section citation available — stated plainly per G2) |
| Option C's removal cost exceeds its benefit | `Exact` (cost) + `Declared` (judgement) | ADR-033 §1.3 (identity-bearing permanence); §5 argument |
| Option A's recommendation over B/C/D | `Declared`-with-argument | §5–§6; disconfirming risk (M-923 slip) named with its fallback |

---

## Changelog

| Date | Status | Note |
|---|---|---|
| 2026-07-02 | **Recommended, pending orchestrator acceptance** | Authored as the M-922 dossier (kickoff `grm`, H2a). Verified the landed kernel state (Path A full-sig encoding + 12-test property suite, 306+11 green re-run today) and the honest gap inventory (no surface lowering — explicit `Residual`; no program-level differential; ADR-033 at propose-`Enacted`). Four options framed; **Option A recommended** (ratify Path A as final FLAG-1 disposition, primitive IN at `Empirical`, M-923 lands lowering + differential), Option B pre-named as fallback. Cites the maintainer's 2026-07-02 delegation to the orchestrator (`Declared`); **not self-ratified** — acceptance steps this note, not this leaf. Surfaced the DN-56-§5-#3 vs kickoff-"condition 2" numbering discrepancy. FLAGs raised for every owner-owned artifact (map G4/G5, DN-56, ADR-033, CHANGELOG/Doc-Index/issues.yaml/api-index, kickoff label, DN-73 gap). Enacts no code; steps no other doc's status. (VR-5; G2; KC-3; append-only.) |
| 2026-07-02 | **Accepted** | Accepted by the wave orchestrator at the integration-reconcile promotion gate, under the maintainer's 2026-07-02 delegation (`Declared`). **Option A** adopted as the ADR-033 FLAG-1 disposition: Path A full-signature encoding is final; the `FieldSpec::Fn` primitive is IN at `Empirical` strength (no `Proven` upgrade — VR-5). The surface lowering + program-level differential (M-923) stay open/held — this acceptance dispositions the soundness question, it does not close the impl lane. ADR-033's own status is untouched here. Forward transition, append-only (house rule #3). |
