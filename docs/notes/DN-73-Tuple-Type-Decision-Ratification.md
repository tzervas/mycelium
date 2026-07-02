# Design Note DN-73 — Tuple-Type Decision: Ratifying the Landed Multi-Arg Surface (RFC-0024 §4A.8)

| Field | Value |
|---|---|
| **Note** | DN-73 |
| **Status** | **Accepted** (2026-07-02 — accepted by the wave orchestrator under the maintainer's 2026-07-02 delegation (`Declared` as relayed), per the integration-reconcile promotion gate; **Option A** ratified — the landed M-822/M-826 tuple state is the final decision, reconciliation-only delta. Was **Recommended, pending orchestrator acceptance** 2026-07-02; that history stands unchanged below — this line records the forward transition, append-only, house rule #3.) The maintainer **delegated this decision to the orchestrator** (2026-07-02, in-session directive — `Declared`). |
| **Recommends** | **Option A (§5):** ratify the landed M-822/M-826 surface as the formal RFC-0024 §4A.8 decision — **curried arrows** (`A => B => C`) are the canonical multi-argument function-value form; the **tuple type** `(T, U, …)` is an orthogonal first-class product (not the multi-arg mechanism); `(A, B) => C` is a *distinct* type (an arrow whose domain is the product) with **no implicit interconversion** to/from the curried form. Delta = reconciliation only (§6). |
| **Feeds** | M-921 (post-decision follow-through — the §6 delta or a grounded no-delta record); kickoff `grm` DoD row 3; the Phase-II mass-port grammar-stability gate (roadmap §5 H2a) |
| **Depends on** | RFC-0024 §4A.5/§4A.8/§4A.10/§5 (the tuple-gated boundary); M-822 (currying, landed 2026-06-29); M-826 (tuple type + `f(x)(y)`, landed 2026-06-29); RFC-0037 D4 (the `=>` arrow glyph); `docs/planning/Blocked-Decisions-Ratification-Map.md` group G2 |
| **Date** | 2026-07-02 |
| **Task** | M-920 (kickoff `grm`, Phase-I H2a) |

> **Posture (transparency rule / VR-5 / G2).** The landed-state inventory (§2) is `Exact` where read
> directly from source (file:line cited) and `Empirical` where it rests on the landed test suites
> (M-822/M-826 three-way differentials, recorded in `tools/github/issues.yaml` `landed_basis`). The
> composition claim that `(A, B) => C` already parses-and-registers (§2.4) is **source-read, not
> fixture-tested** — held at `Declared` (by-construction argument) with the missing fixture flagged
> as part of the §6 delta, never silently assumed green. The recommendation's benefit claim ("ported
> signatures never reshape") is `Declared` (design intent). No status of RFC-0024 or any other doc is
> moved by this note (house rule #3).

---

## 1. Why this decision is still formally open

RFC-0024 §4A.8 framed multi-argument arrows as **gated on a tuple type**: *"The prerequisite is
therefore a tuple type (or, equivalently, currying `(A, B) -> C` to `A -> B -> C` …)"*, and §4A.10(f)
requires that multi-arg/partial cases land *"only after the tuple-type prerequisite (§4A.8) is
ratified separately."* RFC-0024 §5 repeats the boundary: *"FLAG: the tuple type is the gating
decision; not granted here."* The ratification map (group G2) carries the same premise: *"Multi-arg
arrows need tuple-type support (prerequisite)."*

The 2026-06-29 serial-lane closeout then landed **both halves without the formal §4A.8 ratification**:

- **M-822** landed multi-argument partial application **via the currying route** — and its
  `landed_basis` records: *"The RFC-0024 4A.8 tuple-gated premise proved unnecessary (currying needs
  no tuple type)."*
- **M-826** landed the **first-class tuple/product type** anyway (as an independent surface feature,
  per the maintainer decision after M-822), plus the `f(x)(y)` chained-application lift.

So the *implementation* resolved §4A.8's disjunction ("tuple **or** currying") by taking **both
branches** — currying as the multi-arg mechanism, the tuple as an orthogonal product type — while the
*formal decision* the RFC demands ("ratified separately") was never recorded. This dossier is that
record's preparation: it inventories what landed (§2), enumerates exactly what the decision still
governs (§3), and recommends the ratification (§5). Nothing here re-implements; the
verification-first rule of kickoff `grm` applies (verify the landed state, land nothing twice).

## 2. Landed-state inventory (what exists today)

All citations are to the current `dev` tip (merge `629aa12`).

### 2.1 Currying — the multi-arg mechanism (M-822; `Exact` source-read + `Empirical` differential)

- A **multi-parameter lambda** `lambda(p1: A, p2: B, …) => body` desugars at the checker to nested
  single-parameter lambdas `lambda(p1: A) => lambda(p2: B) => … => body`
  (`crates/mycelium-l1/src/checkty.rs:3344`–`3358`; doc block at `:3312`). Each arrow in the chain
  lowers by the existing §4A.5 single-arg Reynolds machinery — no new mechanism, no new L0 node
  (KC-3).
- A **multi-parameter named fn used as a value** synthesizes the curried type
  `A => (B => (… => Z))` (right-associative) and wraps into a curried lambda
  (`checkty.rs:3178`–`3210`). A *generic* multi-parameter fn-as-value is a **never-silent refusal**
  (`checkty.rs:3170`–`3175`) — an honest residual, not a silent accept (G2).
- Partial application `f(x)` yields a partially-applied closure; **chained application** `f(x)(y)`
  routes through the §4A.5 `apply` dispatcher (the M-826 first-order-restriction lift).
- Verification: three-way differential agreement (L1-eval ≡ L0-interp ≡ AOT) per the M-822/M-826
  `landed_basis` — `Empirical`.

### 2.2 The tuple type — an orthogonal product (M-826; `Exact` source-read + `Empirical` differential)

- Surface: tuple **literals** `(a, b)`, **types** `(T, U)`, **patterns** with `let`/`match`
  destructuring, nesting, multi-value return. Arity **≥ 2**; a single parenthesized type `(T)` stays
  grouping, never a 1-tuple (`crates/mycelium-l1/src/ast.rs:562`–`567`;
  `crates/mycelium-l1/src/parse.rs:1564`–`1591`).
- Lowering: each arity-N tuple desugars to a synthetic single-constructor data type
  `Tuple$N<T0, …>` over the existing `Construct`/`Match` nodes (`checkty.rs:664`–`721`) —
  `mycelium-core` untouched (KC-3).
- Flagged non-blocking residue (recorded at landing, still open): positional projection `.0` is
  **destructure-only** (no projection expression); **no unit type** `()`; **no 1-tuples**.

### 2.3 The arrow — single-argument, `=>` glyph (`Exact` source-read)

- The function-type arrow is `=>` (RFC-0037 D4 retired `->`; a leftover `->` is a teaching reject —
  `parse.rs:1452`–`1456`). The arrow is **single-argument and right-associative**
  (`Ty::Fn(Box<Ty>, Box<Ty>)`, `checkty.rs:111`; `BaseType::Fn`, `ast.rs:561`).

### 2.4 Composition: `(A, B) => C` — supported by construction, fixture-missing (`Declared`)

By composition of the two landed pieces, `(A, B) => C` parses today as
`BaseType::Fn(Tuple([A, B]), C)` — an arrow whose **domain is the product** (`parse.rs:1447`–`1468`
parses the tuple atom then the `=>` tail), and the tuple-arity walker **recurses into `Fn` types**
so the domain's `Tuple$2` registers (`checkty.rs:814`–`817`). Two honesty flags:

1. **No dedicated conformance fixture exercises this form** (grep over `crates/mycelium-l1/tests/`
   finds none) — so the end-to-end claim stays `Declared` (by-construction) until the §6 fixture
   lands. Never assumed green.
2. **A stale doc comment contradicts the landed state:** `ast.rs:558`–`560` on `BaseType::Fn` still
   says *"multi-argument `(A, B) -> C` is not yet supported and is a never-silent refusal at the
   parser"* — pre-M-826 text. A reconciliation item, §6.

## 3. The residual decision surface (what §4A.8 still governs)

The mechanism questions are closed by the landed code; what remains is the **formal surface
contract** a mass port will write against. Five points:

- **D1 — canonical multi-arg function-value form.** Is the curried arrow `A => B => C` the canonical
  type of an n-ary function *used as a value*? (It is what the checker synthesizes today, §2.1.)
- **D2 — meaning of `(A, B) => C`.** Is the tuple-domain arrow a *distinct* type from the curried
  form — with **no implicit interconversion** (no auto-curry/uncurry coercion) — or are the two
  unified/adapted? Today they are structurally distinct (`Fn(Tuple$2<A,B>, C)` vs
  `Fn(A, Fn(B, C))`) and nothing converts between them.
- **D3 — declaration signature form.** Does `fn f(a: A, b: B) => C` remain the n-ary declaration
  form (saturated calls `f(x, y)` direct; value use curries), i.e. do ported signatures keep their
  written shape through 1.0?
- **D4 — reconciling RFC-0024's stale premise.** §4A.8/§4A.10(f)/§5 still present the tuple as *the*
  gating prerequisite; M-822 disproved that ("currying needs no tuple type"). The record must be
  reconciled **append-only** (an addendum, never a rewrite — house rule #3). Likewise the map group
  G2 line and the `ast.rs` doc comment (§2.4).
- **D5 — the flagged residue.** Positional projection `.0` as an expression, the unit type `()`,
  1-tuples, and the generic-multi-param-fn-as-value refusal: does this decision resolve them, or do
  they stay explicitly open FLAGs?

## 4. Options

### Option A — Ratify the landed state; reconciliation-only delta (recommended)

Adopt D1–D3 exactly as landed: **curried arrows canonical** for multi-arg function values; the
**tuple an orthogonal product**; `(A, B) => C` a **distinct** tuple-domain arrow with **no implicit
interconversion** (a mismatch is a never-silent type error — a user writes an explicit adapter
lambda when they want one). D4 resolves by append-only addenda; D5 residue **stays open** (flagged,
not decided here). Delta = §6 (docs + one doc comment + one fixture; no semantics change).

- *For:* zero signature reshaping (the M-920 user story — mass-ported code compiles unchanged
  through 1.0, `Declared`); matches the `Empirical` three-way-verified behavior already on `dev`;
  no silent adaptation (house rule #2/G2); the lowest-delta path to closing kickoff `grm` DoD row 3.
- *Against:* two spellings for "a 2-ary function" exist (curried vs tuple-domain) and users must
  pick; mitigated by D2's never-silent distinctness — confusion surfaces as an explicit type error
  with both types printed, never as a wrong-shape accept.

### Option B — Make the tuple-domain arrow canonical (uncurried multi-arg)

Declare `(A, B) => C` the multi-arg form; auto-tuple call sites; retire or demote currying.

- *For:* one obvious n-ary spelling; closer to some mainstream surfaces.
- *Against:* **reshapes the landed, verified surface** (contradicts M-822's `Empirical` landed state
  and the "never reshape after the fact" user story); partial application then *does* need new
  mechanism or auto-tupling coercions; strictly larger delta on the serial `mycelium-l1` lane the
  kickoff is trying to close. Rejected.

### Option C — Unify the forms with implicit curry/uncurry adaptation

Accept either spelling anywhere via inserted coercions.

- *For:* maximum call-site convenience.
- *Against:* a **silent representation adaptation** — exactly what house rule #2 and G2 prohibit
  (a swap-like change that is not reified/EXPLAINed); introduces inference ambiguity (is `f(x, y)`
  a saturated curried call or a 1-tuple-argument call?). Rejected on principle, independent of
  delta size.

### Option D — Defer again

- *Against:* M-921 sits in the serial lane blocked on this; kickoff `grm` DoD row 3 and the Phase-II
  mass port are gated on grammar stability — deferral re-opens the moving-grammar risk the kickoff
  exists to close (roadmap §5: no mass porting against a moving grammar). Rejected.

## 5. Recommendation

**Option A.** Ratify the landed M-822/M-826 surface as the formal RFC-0024 §4A.8 decision:

1. **D1:** the curried arrow `A => B => C` is the canonical multi-argument function-value type.
2. **D2:** `(A, B) => C` is a distinct single-argument arrow over the product `Tuple$2<A, B>`;
   **no implicit interconversion** with the curried form — ever, in either direction (G2).
3. **D3:** `fn f(a: A, b: B) => C` remains the n-ary declaration form; saturated calls stay direct;
   value use curries. Ported signatures keep their written shape.
4. **D4:** RFC-0024 §4A.8's tuple-prerequisite premise is reconciled **append-only** (§6), recording
   that the disjunction resolved as *currying = the mechanism, tuple = an independent product*.
5. **D5:** the flagged residue (positional projection expression, unit type, 1-tuples,
   generic-multi-param-fn-as-value) stays **explicitly open** — separate future decisions, not
   silently granted here.

Basis strength: the inventory this ratifies is `Exact`/`Empirical` (§2); the ratification itself is
a surface-contract commitment (`Declared`, as all surface decisions are). Per the maintainer's
**2026-07-02 delegation to the orchestrator**, this note carries the recommendation only; the
decision is recorded when the orchestrator flips the status to Accepted (append-only changelog
entry) — **not before** (VR-5: assent is not upgraded past its basis).

## 6. The M-921 delta under Option A (reconciliation-only)

M-921 ("implement ONLY the delta the ratified decision directs, or record no-delta with evidence")
reduces, under Option A, to:

1. **RFC-0024 append-only addendum** (§4A.8 + §5 boundary + §4A.10(f)): record that the prerequisite
   disjunction resolved via currying (M-822), the tuple landed independently (M-826), and the
   "ratified separately" gate is discharged by this decision. No rewrite of the historical text.
2. **Fix the stale `ast.rs:558`–`560` doc comment** on `BaseType::Fn` (§2.4 flag 2).
3. **A conformance/differential fixture for the tuple-domain arrow** `(A, B) => C` — constructing,
   passing, and applying one — upgrading §2.4's `Declared` by-construction claim to `Empirical`,
   plus a **distinctness reject fixture** (curried value where tuple-domain expected ⇒ explicit
   type error naming both types, per D2).
4. **No semantics/grammar change.** If review of (3) exposes a gap (e.g. the tuple-domain arrow does
   *not* check end-to-end), that is a FLAG back to this decision, not a silent patch.

Estimated as low-delta docs + tests on the serial lane; no new L0 node under any circumstance (KC-3
STOP-and-flag still binds).

## 7. Out of scope / FLAGs to the integrator

- **Explicitly not decided here (D5):** `.0` projection expressions, unit `()`, 1-tuples,
  generic multi-param fn-as-value. Each is a candidate future note; all are never-silent refusals
  or destructure-only today.
- **FLAG (integrator-owned files — not touched by this leaf):** `CHANGELOG.md` entry;
  `docs/Doc-Index.md` registration of DN-73; `tools/github/issues.yaml` M-920 status + this note as
  `doc_refs`; `docs/planning/Blocked-Decisions-Ratification-Map.md` **group G2** — its "Multi-arg
  arrows need tuple-type support (prerequisite)" line is stale per §1 and should be annotated when
  the map is next reconciled (append-only).
- **FLAG:** upon orchestrator acceptance, M-921 unblocks (serial lane) with the §6 list as its scope.

## 8. Definition of Done (this dossier)

- [x] Landed-state inventory grounded to file:line + `landed_basis` (`Exact`/`Empirical`; §2), with
  the one by-construction claim honestly held at `Declared` and its missing fixture flagged (§2.4).
- [x] Residual decision surface enumerated (D1–D5, §3).
- [x] Options with tradeoffs (§4) and a clear recommendation (§5) citing the maintainer's
  2026-07-02 delegation.
- [x] The post-decision delta enumerated for M-921 (§6); shared-file updates FLAGged, not edited
  (§7).
- [ ] **Orchestrator acceptance recorded** (append-only status flip below) — the delegated
  decision's act, not this dossier's.
- [ ] M-921 executes the §6 delta (or records a grounded no-delta) — after acceptance only.

---

## Changelog

| Date | Status | Note |
|---|---|---|
| 2026-07-02 | **Recommended, pending orchestrator acceptance** | Initial record (M-920). Landed M-822/M-826 state inventoried; residual §4A.8 surface enumerated (D1–D5); Options A–D; recommendation = Option A (ratify landed state, reconciliation-only delta). Decision delegated by the maintainer to the orchestrator (2026-07-02); acceptance pending — not self-ratified (house rule #3). |
| 2026-07-02 | **Accepted** | Accepted by the wave orchestrator at the integration-reconcile promotion gate, under the maintainer's 2026-07-02 delegation (`Declared`). **Option A** adopted: the landed M-822/M-826 tuple state is the final tuple-type decision; only the reconciliation-only §4A.8 delta remains, tracked as normal follow-up. Forward transition, append-only (house rule #3); no guarantee tag upgraded past its basis (VR-5). |
