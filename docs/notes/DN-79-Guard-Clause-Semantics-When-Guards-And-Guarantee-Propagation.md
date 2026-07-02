# Design Note DN-79 — Guard-Clause Semantics Dossier: `when <cond>` Guards and Guarantee Propagation (M-833)

| Field | Value |
|---|---|
| **Note** | DN-79 |
| **Status** | **Accepted** (2026-07-02 — accepted by the wave orchestrator under the maintainer's 2026-07-02 delegation (`Declared`), per the integration-reconcile promotion gate; the §6 **RECOMMENDATION** for `when`-guard semantics + guarantee propagation is adopted as the ratified direction. Nothing is implemented by this acceptance — M-833's RFC-0020 amendment and any code remain strictly post-acceptance follow-up (held, not part of this wave). Was **Recommended, pending orchestrator acceptance** 2026-07-02; that history stands unchanged below — append-only forward transition, house rule #3.) The maintainer **delegated this ratification decision to the kickoff orchestrator** (2026-07-02, §1.3). |
| **Feeds** | RFC-0020 §4.1 S2 / §4.4 (the append-only amendment M-833 specifies); RFC-0018 §4.3/§4.5 (G-Match under Design A — extended, not modified); the conformance corpus (guard cases, §6.3); DN-56 freeze sequencing (FLAG-4, §7) |
| **Depends on** | DN-64 §6 OQ-G + §7 disposition table (the maintainer decision, 2026-06-29); research/27-dn64-ergonomics-rnd-RECORD.md §3; RFC-0020 §4.4 (Maranget compilation, Accepted scope); RFC-0018 §4.1/§4.3/§4.5 (guarantee meet-semilattice, G-Match Design A — Enacted stage 1a); RFC-0007 §4.1 W7 (flat kernel `Match`); RFC-0006 §4.1 S1–S6; VR-5; G2; KC-3 |
| **Date** | July 2, 2026 |
| **Decides** | *Nothing normatively.* Enumerates (1) the surface + elaboration semantics for `when <cond>` pattern guards (§2); (2) the exhaustiveness/usefulness options E1–E3 (§3); (3) the guarantee-propagation options P1a/P1b/P2 (§4); (4) the interaction ledger (§5); and records a ratification-ready recommendation package (§6) with open questions and FLAGs (§7). |
| **Task** | M-968 (kickoff `frz`, Lane D — l1-semantics closeout tail; design-first for M-833) |

> **Posture (transparency rule / VR-5 / G2).** Advisory dossier. Per-claim tags throughout: `Exact`
> claims cite committed corpus text or committed code; `Declared` claims are design directions or
> recorded directives with no checked basis; nothing in this note is `Proven` (no theorem with checked
> side-conditions is invoked). The recommendation is argued on merit against the recorded maintainer
> disposition — where the recommendation departs from a *literal* reading of that disposition (the
> `Empirical`-guard cell, §4.4), the departure is surfaced explicitly, never chosen silently (G2,
> house rule #4).

---

## §1 Purpose, basis, and current state

### 1.1 The question

DN-64 §6 **OQ-G** asked: should `when <cond>` guard clauses on patterns be ratified, and does the
guarantee tag of the guard weaken the arm's tag — if the guard is `Declared`, does the arm become
`Declared`? (`Exact` — DN-64 §6 OQ-G.)

### 1.2 The maintainer disposition (faithful)

DN-64 §7 (2026-06-29) records the disposition verbatim: **"Yes, ratify guards.** A guard's tag
propagates to the arm: **unless mechanically proven, the arm drops to `Declared`** (VR-5)." —
Disposition **Direct (M-833)**, feeding RFC-0020 §4.1 S2 and `research/27`. (`Exact` — DN-64 §7
row G.) The grounded analysis in research/27 §3.2 elaborates the mechanism as the **meet rule**:
the arm's effective grade is `grade(guard) ∧ grade(arm_body)`, with no guard-specific upgrade path
(`Swap` remains the sole endorsement point, RFC-0018 §3.3/G-Swap). (`Exact` as a record of what
research/27 §3.2 says; the rule itself is a `Declared` design direction until ratified.)

### 1.3 The delegation (why this note recommends rather than prepares-only)

The M-968 task as planned (kickoff `frz`, Lane D) prepares guard clauses *for maintainer
ratification*. On **2026-07-02 the maintainer delegated this decision to the kickoff
orchestrator** (session directive relayed to this task's brief). This note honors both the
delegation and the append-only discipline: it drafts the options **with a clear recommendation**
(§6), cites the delegation here, and holds status at **"recommended, pending orchestrator
acceptance"** — the acceptance itself is the orchestrator's recorded act, not this dossier's.
(`Declared` — a relayed directive; this note is its written record. If the delegation is
misstated, correcting it is a one-line append here, not a rewrite.)

### 1.4 Current state of the corpus and code (the baseline being amended)

- **RFC-0020 §4.4** (Accepted scope) commits Maranget-style decision-tree compilation of L2
  patterns to the flat L1 `Match`, usefulness-checked (inexhaustive and useless rows are explicit
  diagnostics — LR-1/S5). Its surface-pattern table has **no guard row**; guards do not exist in
  the committed L2 surface. (`Exact` — RFC-0020 §4.4.)
- **Or-patterns are active** (M-823 / R20-Q3): the committed grammar's arm production is
  `arm ::= pattern ('|' pattern)* '=>' expr`, and or-arms **desugar to one row per alternative**
  with binding/type equality checked across alternatives. (`Exact` —
  `docs/spec/grammar/mycelium.ebnf`, `arm` production.)
- **`when` is not lexed**: no `Tok::When` exists in `crates/mycelium-l1/src/lexer.rs`/`token.rs`,
  and `when` appears nowhere in the committed EBNF. Ratification must reserve it (DN-02/DN-03
  lexicon registration; the *spelling* `when` is already fixed by the OQ-G disposition). (`Exact`
  for the code/grammar absence; the reservation step is `Declared`.)
- **L1 `Match` is flat** (RFC-0007 §4.1 W7): one scrutinee, single-level alternatives, at most one
  default. Guards do not exist in L1 and — under everything below — **never will**: the guard is
  L2 elaboration-only sugar, adding **no kernel node** (KC-3). (`Exact` for W7; `Declared` for the
  no-new-node commitment.)
- **The usefulness checker** (`crates/mycelium-l1/src/usefulness.rs`) implements the standard
  Maranget `U(P, q)` over a normalized `Pat ∈ {Wild, Ctor, Lit}` matrix, with witness reporting.
  It has no guard concept. (`Exact` — the module's committed doc-comment and types.)
- **RFC-0018 stage 1a is Enacted** with **Design A ratified** (R18-Q1, data-lineage only): the
  G-Match result grade is the meet of all branch *body* grades; the scrutinee's grade does **not**
  taint the result. (`Exact` — RFC-0018 header + §4.5 Design A.)

---

## §2 Surface and elaboration semantics (the concrete proposal)

### 2.1 Grammar

Extend the committed `arm` production (append-only amendment to the EBNF plus RFC-0020 §4.4's
table, both under M-833 post-acceptance):

```ebnf
arm ::= pattern ('|' pattern)* ('when' expr)? '=>' expr
```

One `when` guard per **arm**, applying to the whole or-group. A per-alternative guard form
(`p1 when c1 | p2 when c2 => e`) is **not** proposed: it forks the canonical form for one concept
(DN-03 §3 one-canonical-form) and is expressible today as two arms. (`Declared` direction.)

`when` becomes a **reserved word** (currently an ordinary identifier): it must lex as a keyword so
its appearance in an arm is never a silent identifier (G2). The spelling is the disposition's; no
naming question remains open. (`Declared`; the lexicon-registration step is mechanical.)

### 2.2 Static semantics of the guard expression

- The guard is typed with the arm's **pattern bindings in scope** and must have type `Bool`:
  `Γ, x̄ ⊢ c : Bool @ g_c` under RFC-0018 §4.3's ordinary grading (its grade `g_c` is the meet of
  its inputs' grades through its operations). (`Declared` direction; the grading machinery it
  reuses is `Exact`/Enacted.)
- **S1 holds inside guards**: a representation-changing operation in a guard follows the normal
  explicit-swap rules — a missing conversion is a `MissingConversion` error, never a silent swap
  (RFC-0006 §4.1 S1). (`Exact` basis; its application to guards is `Declared`.)
- **Guards are restricted to the pure fragment in v1** (no effectful operations). Two grounds:
  (a) or-arms desugar to one row per alternative (§1.4), so an arm-level guard is **duplicated
  per alternative** and may evaluate more than once for one logical arm — pure guards make the
  duplication observationally invisible; an effectful guard would double-fire observably, which
  the never-silent rule would force us to either reject or reify, and rejection is the smaller
  surface; (b) match-order fallthrough (§2.3) re-enters the residual tree after a failed guard,
  which is only order-insensitive for pure guards. Lifting the restriction later is an append-only
  extension gated on the effect-system surface (DN-60). (`Declared` — see FLAG-3.)

### 2.3 Dynamic semantics and elaboration to L1

Arms are tried in source order. When an arm's pattern matches, its bindings are established and
its guard (if any) is evaluated; on `True` the arm body runs; on `False` the bindings are
discarded and matching **continues with the remaining rows** (fallthrough), exactly as if the
guarded row had not matched. Guard evaluation order is observable and normative — and dumpable
per S4. (`Declared` direction, standard guard semantics per Maranget's treatment of guarded rows.)

**Elaboration (no kernel growth — KC-3).** A guarded row lowers through the existing decision-tree
compilation with one extra split, using the RFC-0020 §4.5 `if`-chain precedent (`Bool` is a
two-constructor registry type):

```text
… p when c => e, rest …
  ⤳ (at the decision-tree leaf where p has matched, bindings x̄ bound)
Match(c, [(True, e)], Some(residual))
```

where `residual` is the decision tree compiled from the **remaining** rows. Two strategies for
`residual`:

- **(a) Continuation duplication** — inline the residual tree at every guard-failure edge. Simple,
  but worst-case exponential blowup with stacked guards. (`Declared`.)
- **(b) Join-point sharing (recommended)** — bind the residual once (`Let(k, residual_thunk, …)`)
  and invoke it at each guard-failure edge; L1 `Let`/`Lam` suffice, no new node. Code-size linear
  in the number of guards. (`Declared`.)

Either strategy is an **implementation choice**, not observable semantics: the elaborator owes
observational equivalence and an S4-dumpable elaboration (the dump shows the guard split, so
`EXPLAIN` can answer "why did this arm not fire" — the guard's `False` edge is reified, house
rule #2). The recommendation is (b). (`Declared`.)

**W7 compliance is preserved**: every `Match` node emitted (including the guard's `Bool` match) is
flat, saturated, and coverage-complete — the guard match always carries both a `True` alternative
and a default/`False` edge. (`Declared`, mechanical consequence of the lowering above.)

---

## §3 Exhaustiveness and usefulness — options

The load-bearing fact: **a guard can fail at runtime**, so a guarded row's coverage contribution
is conditional on an arbitrary Boolean the checker cannot in general decide. Three options:

### E1 — Guard-erased coverage (conservative; RECOMMENDED)

For **exhaustiveness**, a guarded row contributes **nothing**: `U(P, [_])` runs with all guarded
rows **removed** from the matrix. A match whose full coverage depends on a guard's truth is
**rejected** with the standard witness diagnostic naming the uncovered case — the S5
explicit-partiality outcome; the author adds an unguarded arm or a default. For **usefulness**:

- a guarded row's own reachability is checked with its guard erased (the row is useful iff its
  pattern adds coverage on the assumption the guard *may* succeed); a guarded row whose pattern is
  fully shadowed by prior **unguarded** rows is redundant — its guard can never be consulted —
  and is diagnosed as today;
- a guarded row never shadows later rows (for the redundancy check of row *i*, prior guarded rows
  are removed, since each may fail).

The Maranget framework is **preserved unchanged**: only matrix *construction* changes (which rows
enter which check); the `U(P, q)` algorithm and witness machinery in `usefulness.rs` are untouched.
This is the position of the standard treatment of guarded rows in Maranget-style checkers (and the
Rust precedent), and the direction research/27 §3.2 already analyzed. Over-rejection is possible
(complementary guards `when c` / `when !c` are rejected without a default) — the cost is an
explicit extra arm, never a soundness hole. (`Declared` direction; the algorithm-preservation
claim is `Exact` with respect to which components change.)

### E2 — Guard-complement completeness analysis (deferred R&D)

A decision procedure (SMT or a syntactic complement check) proves that a set of guards on the same
pattern jointly covers, letting `when c` / `when !c` count as exhaustive. Rejected for v1:
(a) it puts a prover in the elaboration path (KC-3 pressure, and the coverage claim would need a
**checked** basis to be more than `Declared` — VR-5); (b) E1 loses no expressiveness, only
requires one explicit arm; (c) it is a clean **append-only future extension** — E2-accepted
matches are a strict superset of E1-accepted ones, so adding E2 later breaks nothing. Deferred as
R&D; the tracking id is the orchestrator's to mint (FLAG-5 — `issues.yaml` is orchestrator-owned,
mitigation #1). (`Declared`.)

### E3 — Guards count as covering, runtime match-failure (REJECTED)

Treat a guarded row as covering its pattern and panic/throw if all guards fail at runtime. This is
a **silent partiality**: it violates LR-1 (exhaustive `match`, no silent fall-through), S5
(explicit partiality), and G2 (never-silent). Not viable in this language. (`Exact` — the cited
invariants are committed; the conflict is direct.)

---

## §4 Guarantee propagation — options

### 4.1 The rule shape

Under RFC-0018 §4.5 **Design A** (ratified, R18-Q1), the G-Match result grade is the meet of all
branch-body grades. The guard extension adds the guard's grade to its own arm's contribution:

```text
G-Match/Guard (Design A, extended)

  Γ ⊢ s : T @ g_s
  for each arm i with pattern bindings x̄ᵢ:
      (guard, if any)   Γ, x̄ᵢ ⊢ cᵢ : Bool @ g_cᵢ
      (body)            Γ, x̄ᵢ ⊢ eᵢ : τ @ g_eᵢ
      contribution      gᵢ = g_cᵢ ∧ g_eᵢ        (unguarded arm: gᵢ = g_eᵢ)
  ────────────────────────────────────────────────────────────────
  Γ ⊢ Match(s, arms, default?) : τ @ (g₁ ∧ … ∧ gₙ)
```

The scrutinee's grade `g_s` still does not appear (Design A unchanged). (`Declared` direction; the
Design-A baseline is `Exact`/Enacted.)

### 4.2 The deliberate asymmetry — why the guard taints when the scrutinee does not

Design A is data-lineage-only: branching on `Declared` data does not degrade an arm that returns
`Exact` data (the scrutinee is pure control). A guard is different in kind: it is a **refinement
claim about the matched value that the arm body relies on** — `x when x > 0 => …` hands the body
not just control but the *assumption* `x > 0`. The trustworthiness of that assumption is part of
the arm's result lineage: if the guard's inputs are `Declared`, the refinement the body computed
under is only asserted. The maintainer's disposition makes exactly this call, and it is coherent
with Design A rather than a contradiction of it — the guard's grade travels **as an input the arm
consumed**, not as a `pc` taint (no implicit-flow machinery is introduced; Design B stays
rejected). (`Declared` — design rationale grounded in the DN-64 §7 disposition; house rule #4:
this paragraph is an argument, not a checked fact.)

### 4.3 The options

- **P1a — Arm-level meet (RECOMMENDED).** `gᵢ = g_cᵢ ∧ g_eᵢ` as in §4.1. Simple, uniform with the
  RFC-0018 §4.1 composition law, and the reading research/27 §3.2 recorded. A `Declared` guard
  makes the arm `Declared`; an `Exact`/`Proven` guard passes its strength into the meet; **an
  `Empirical` guard caps the arm at `Empirical`**. (`Declared` direction; the meet law it reuses
  is `Exact`.)
- **P1b — Binding-level meet (finer; NOT recommended).** Meet `g_cᵢ` only into the grades of the
  pattern bindings the guard refines; an arm body that ignores the bindings keeps its own grade.
  More precise data-lineage, but it directly contradicts the disposition's plain statement ("if
  the guard is `Declared`, the arm becomes `Declared`" — research/27 §3.1) in the constant-body
  case, and the precision buys little: an arm that ignores its bindings rarely needs a guard.
  Enumerated for completeness. (`Declared`.)
- **P2 — Binary drop-to-`Declared` (the literal reading).** Unless `g_cᵢ ∈ {Exact, Proven}`, the
  arm is `Declared` — i.e. an `Empirical` guard flattens the arm to `Declared` rather than to
  `Empirical`. Sound under VR-5 (over-degradation never upgrades) but **lossy**: `Empirical` is by
  definition trials-backed evidence, and flattening it to `Declared` discards real, honestly-tagged
  strength. House rule #1's own framing is "**downgrade to stay accurate**" — P1a's meet *is* the
  accurate downgrade; P2 overshoots it. (`Declared`.)

### 4.4 The one interpretation delta (surfaced, not silently chosen)

P1a and P2 differ in exactly one cell: the **`Empirical` guard**. The disposition's literal words
("unless it can be mechanically proven, it must drop to `Declared`") support P2; the disposition's
recorded analysis (research/27 §3.2, "the arm is exactly the meet of the guard's grade and the
body's grade… No separate upgrade path is needed") supports P1a. This dossier recommends **P1a**
on the grounds in §4.3 and because the meet is the lattice's single composition law everywhere
else (one rule, no special case — KISS, house rule #5). The delegated acceptor should confirm this
cell deliberately; if the stricter P2 is intended, it is a one-line supersession of this
recommendation, not a rework. (`Declared`; G2 — the delta is the one place recommendation and
literal wording diverge.)

### 4.5 No guard-specific upgrade path

`Swap` remains the **sole** endorsement point (RFC-0018 §3.3, G-Swap — the only rule that may
raise a grade, and only against a valid certificate). A guard evaluating to a `Proven`-graded
`Bool` contributes at most `Proven` to the meet; it can never upgrade a weaker body. "Mechanically
proven" is operationalized as: `g_c` is whatever the Enacted RFC-0018 checker derives for the
guard expression — `Proven` arises only through the existing certified paths (house rule #1: no
`Proven` without a checked side-condition). No new machinery. (`Exact` basis — RFC-0018 §3.3/§4.3;
its sufficiency for guards is the research/27 §3.2 conclusion, `Declared`.)

---

## §5 Interaction ledger

| Interaction | Consequence | Tag |
|---|---|---|
| **Or-patterns (M-823, active)** | `p1 \| p2 when c => e` desugars per the existing one-row-per-alternative rule to two guarded rows sharing `c`; the §2.2 purity restriction makes the duplicated guard observationally single. Binding/type equality across alternatives already enforced; the guard typechecks once against the shared binding set. | `Declared` (desugar direction); `Exact` (the or-desugar baseline) |
| **`if` elaboration precedent (RFC-0020 §4.5)** | The guard's `Bool`-match lowering reuses the committed `if`→`Match` scheme verbatim; no new lowering concept. | `Exact` precedent |
| **Totality / `for` (RFC-0007 §4.8)** | Guards introduce no recursion or new partiality beyond §3's checked exhaustiveness; `for` elaboration is untouched. | `Declared` |
| **EXPLAIN / S4** | The guard split is a dumpable elaboration step; a guard-failure edge is reified, so selection of the fired arm is inspectable (house rule #2). | `Declared` direction on the `Exact` S4 obligation |
| **Monomorphization tag context (M-967 / OQ-S)** | `g_c` is graded per instantiation like any expression; the per-instantiation tag-context threading M-967 lands is what keeps a guard's grade from merging across instantiations. No guard-specific work, but M-967 is the carrier. | `Declared` |
| **Kernel freeze (DN-56)** | Guards add **no L1 node** (KC-3-neutral, kernel diff = none) but **do extend the L2 grammar** (`when` reserved + the `arm` production). If the freeze's "lowering surface closed" condition is declared before M-833 lands, guard implementation is a post-freeze *surface* change — compatible with a DN-39-only *kernel* diff policy, but the sequencing must be an explicit orchestrator/maintainer call, not assumed. | `Declared` — **FLAG-4** |
| **Cert modes (RFC-0034)** | Guards are grade-generic; nothing here reads or writes the `fast`/`certified` axis. | `Declared` |

---

## §6 Recommendation (the ratification-ready package)

**⟐ RECOMMENDED — for orchestrator acceptance under the 2026-07-02 delegation:**

1. **Ratify** `when <cond>` guards with the §2 surface: one guard per arm, after the or-group
   (`arm ::= pattern ('|' pattern)* ('when' expr)? '=>' expr`); `when` reserved; guards typed
   `Bool` with bindings in scope; **pure fragment only** in v1; S1 holds inside guards.
2. **Exhaustiveness = E1** (guard-erased coverage): guarded rows contribute nothing to coverage;
   inexhaustive-with-guards is an explicit rejection with witness; usefulness per §3-E1; the
   Maranget `U(P, q)` core is unchanged. E2 deferred as R&D (orchestrator mints the id); E3
   rejected outright.
3. **Guarantee propagation = P1a** (arm-level meet): `gᵢ = g_cᵢ ∧ g_eᵢ` under Design A, scrutinee
   still untainted; no guard-specific upgrade path (`Swap` stays the sole endorsement point). The
   `Empirical`-guard cell (§4.4) is the one deliberate confirmation the acceptor owes.
4. **Elaboration**: guard-failure edges via the `if`-precedent `Bool` match with **join-point
   sharing** (§2.3-b); S4-dumpable; W7-compliant; **no new L1 node** (KC-3).
5. **M-833 implementation scope (post-acceptance, not this task)**: the append-only RFC-0020 §4.4
   amendment (guard row in the surface table + the E1 usefulness rule + the P1a propagation rule);
   EBNF `arm` production update; `when` lexed; conformance corpus cases — at minimum:
   inexhaustive-guarded-match rejection (E1), `Declared`-guard degradation, `Proven`-guard meet
   with `Proven` body, `Empirical`-guard cap (the §4.4 cell, locked by test), shadowed-guarded-arm
   redundancy diagnostic, or-pattern-plus-guard desugar, `MissingConversion` inside a guard (S1),
   and the guard-split `EXPLAIN` dump (S4).

Grounding summary: the *decision to have guards* and the *tag-meet direction* are the maintainer's
(DN-64 §7 OQ-G, `Exact` as a record); the E1/P1a/join-point selections are this dossier's
`Declared` recommendations built on `Exact` corpus invariants (W7, S1–S5, LR-1, the RFC-0018 meet
law, Design A) and the research/27 §3 analysis. Nothing exceeds its basis (VR-5).

---

## §7 Open questions and FLAGs

- **FLAG-1 (the gate).** Orchestrator acceptance under the 2026-07-02 delegation is the
  ratification act. This note holds at "recommended, pending orchestrator acceptance" until that
  acceptance is recorded append-only (here and in M-833/M-968 tracking).
- **FLAG-2 (the `Empirical` cell).** P1a vs P2 (§4.4) — recommended P1a; the divergence from the
  disposition's literal wording is surfaced for a deliberate call.
- **FLAG-3 (guard purity).** v1 restricts guards to the pure fragment (§2.2). Whether effectful
  guards are ever admitted is deferred to the effect-system surface (DN-60) — an append-only
  extension question, not blocking.
- **FLAG-4 (freeze sequencing).** Guards extend the L2 grammar; the DN-56 "lowering surface
  closed" condition and M-833's landing must be explicitly ordered by the orchestrator/maintainer
  (§5). Not this dossier's call.
- **FLAG-5 (E2 R&D id).** The guard-complement completeness analysis (§3-E2) needs a tracking id;
  `tools/github/issues.yaml` is orchestrator-owned — minting is FLAGged up, not done here
  (mitigation #1).
- **FLAG-6 (shared files untouched).** `CHANGELOG.md`, `docs/Doc-Index.md`, `docs/api-index/`,
  and `tools/github/issues.yaml` (M-968 → done; M-833 status note) are orchestrator-owned and
  deliberately not edited by this task; the integrating parent applies them once.

## §8 Definition of Done (this dossier) and guarantee posture

**DoD (met by this note):** the exhaustiveness semantics enumerated with options and a selection
(§3); the guarantee-propagation semantics enumerated with options, the formal rule, and a
selection (§4); the surface + elaboration semantics concrete enough to amend RFC-0020 §4.4 from
directly (§2, §6.5); interactions and sequencing risks ledgered (§5); decision-gated status held —
**no implementation, no grammar edit, no RFC edit** in this task; every claim tagged; the one
interpretation delta and the delegation both recorded explicitly (G2).

**Guarantee posture:** all `Exact` claims cite committed corpus/code locations named inline;
everything normative-sounding about *guards themselves* is `Declared` until the orchestrator's
acceptance and remains `Declared` after it until M-833's implementation lands with its tests
(acceptance ratifies the design; it upgrades no tag — VR-5).

---

## Changelog

- **2026-07-02 — Drafted** (M-968, kickoff `frz` Lane D). Options + recommendation for `when`
  guard clauses and guarantee propagation (M-833 / DN-64 OQ-G). Status: **recommended, pending
  orchestrator acceptance** under the maintainer's 2026-07-02 delegation. Nothing implemented.
- **2026-07-02 — Accepted** (recommended → Accepted). Accepted by the wave orchestrator at the
  integration-reconcile promotion gate, under the maintainer's 2026-07-02 delegation (`Declared`).
  The §6 recommendation is the ratified direction for `when`-guard semantics + guarantee
  propagation; M-833's RFC-0020 amendment and any implementation are post-acceptance follow-up
  (held, not landed this wave). Forward transition, append-only (house rule #3).
