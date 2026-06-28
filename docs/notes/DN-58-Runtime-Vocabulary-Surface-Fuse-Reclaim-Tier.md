# Design Note DN-58 — Runtime-Vocabulary Surface Forms (`fuse` / `reclaim` / `tier`)

| Field | Value |
|---|---|
| **Note** | DN-58 |
| **Status** | **Accepted** (2026-06-28; **ratified by the maintainer 2026-06-28**) — the L1 *surface forms* for `fuse`, `reclaim`, `tier` (which RFC-0027 §10.5 / ADR-020 deferred) are approved. Maintainer decisions: **§A `fuse`** and **§B `reclaim`** accepted as proposed (the `Fuse` lawful-merge trait + bare `fuse(a,b)`; the `reclaim(policy) { scope }` modifier dispatching to `std.runtime` supervision). **§C `tier`** with three explicit calls — **F-C1 = attribute form** (`@tier(…)`), **F-C2 = `compiled`/`interpreted`** mode vocabulary (not `native`), **F-C3 = per-definition hint** (decoupled from build-target profiles). ~~Enacts no code~~ — **implemented (Rust-first, 2026-06-28):** surface + type-check + elaboration landed (M-667, r4v wave); repr-type `fuse` executes three-way (`Empirical`); reclaim runtime supervision wiring + data-type fuse prim registration are **residual** (follow-on M-817). **NOT `Enacted`** — execution is partial (VR-5); → `Enacted` gated on M-817 landing + M-710 closed. Prior: **Draft** (2026-06-27). |
| **Task** | M-667 (L1 surface) · M-710 (execution end-to-end) |
| **Feeds** | the RFC-0008 §4.6 R1 vocabulary activation — the construct-by-construct path ADR-020 §"For Phase-7 construct activation" lays out: this note is step (1) "an implementation RFC commits the construct's typing + elaboration (per RFC-0006 §4.3)" for `fuse`/`reclaim`/`tier`. Unblocks the `r4v` wave (kickoff) and E12-1/T3. |
| **Date** | June 27, 2026 |
| **Decides** | *Nothing normatively — design-direction capture for ratification.* Proposes (A) the `fuse` expression surface (lawful binary merge over a declared semilattice op, RT6); (B) the `reclaim` surface (a reified reclamation/supervision policy attached to a structured scope, RT7 — the shape RFC-0027 §10.5 left open); (C) the `tier` surface (a per-definition execution-mode request bridging RFC-0004 `ExecutionMode`, observable-equivalent by NFR-7); and (D) the common typing/elaboration plan (parse → AST → `checkty` → `elab` dispatch to `mycelium-std-runtime`, never-silent on every refusal). |

> **Posture (transparency rule / VR-5 / G2).** This note **records design proposals**, not ratified
> decisions. It enacts no code, moves no RFC/ADR/DN status (house rule #3). Every claim is tagged at
> its honest strength: the **existing** facts (RFC-0008 §4.5 operational meanings, the landed
> `mycelium-std-runtime` supervision/scheduler/reclamation machinery, RFC-0004 `ExecutionMode`,
> RFC-0027's deferral) are `Empirical`/`Exact` cited to source; the **proposed** surface forms,
> grammar productions, AST shapes, and elaboration rules are **`Declared`-with-argument** — design
> proposals, not ratified, no tag upgraded past its basis (VR-5). Every open sub-question is named and
> FLAGGED for the maintainer (G2). The `reclaim` surface in particular (§B) is the part RFC-0027 §10.5
> explicitly deferred — it carries the most uncertainty and the heaviest flags.

---

## §0 Why this note exists (the deferral it closes)

RFC-0008 §4.5 ratified the *operational meanings* of the runtime vocabulary and DN-03 §4 ratified the
*names* through the DN-02 three-test gate — but both fixed a status rule: these remain **"reserved
vocabulary, not active syntax"**; activation "still requires each construct's implementation RFC
committing each construct's typing and elaboration per RFC-0006 §4.3." RFC-0027 §10.5 then sharpened
this for `reclaim`: *"This RFC specifies the memory model only. The `reclaim` surface construct (L1
typing + elaboration) is left to a follow-on RFC (`lane-B` OQ-2; ADR-020 §consequences defers it)."*
ADR-020's activation pattern names step (1) as "an implementation RFC commits the construct's typing +
elaboration."

So the surface forms are *deliberately unlocked* — and a wave that wrote AST variants now would be
inventing that surface, exactly the design the specs reserved. **This note proposes those surface forms
so the maintainer can ratify them before `r4v` writes code.** The runtime machinery is *not* the gap:
the scheduler (M-709), dataflow deadlock-freedom (M-711), and structured supervision (M-713) are landed
in `mycelium-std-runtime` (`Empirical`, cited at `crates/mycelium-std-runtime/src/lib.rs` §"Execution
maturity"); the gap is purely the L1 surface → those APIs.

**Precedent followed:** the `lambda` treatment in the RFC-0037 grammar epic (surface lands; semantics
behind a never-silent `Residual` until the algorithm wave) and the DN-53 `object` desugar spec
(surface form + grammar + AST + lowering + transparency invariant). This note mirrors DN-53's shape.

---

## §A `fuse` — lawful binary merge (RT6)

### §A.1 Operational basis (Empirical — RFC-0008 RT6, RFC-0027 §10.6)

`fuse` is **genuine merge**: two states converge into one through a **declared merge operation that is
commutative, associative, and idempotent** (the monotonic-semilattice condition; convergence is the
CRDT strong-eventual-consistency theorem, mechanized in Isabelle/HOL — RFC-0008 RT6). The merged
value's `Meta` composes honestly: **guarantee = meet of the inputs' guarantees** (and the merge op's
own intrinsic strength, RFC-0001 §4.7); **provenance = derived-from-both**. A merge that cannot satisfy
the laws is **not** a fusion — it is an explicit conflict surfaced to a policy (RT3). RFC-0027 §10.6
fixes the provenance shape: a `fuse` result is `Derived{op:"fuse_join", inputs:[left_root, right_root]}`.

### §A.2 Surface form (proposed — Declared)

`fuse` is an **expression** combining two values of the same fusible type:

```
// expression position; the merge op is resolved from the type's declared lawful-merge instance
fuse(a, b)
```

The lawful-merge operation is **not** spelled at the call site — it is carried by the type via a
trait that declares the merge op *and the semilattice-law obligation*. Proposed trait (name open,
§A.6):

```
// a type is fusible iff it declares a commutative/associative/idempotent merge
trait Fuse {
    // the declared merge op; laws are an obligation discharged per §A.4
    fn join(self: Self, other: Self) => Self;
}
```

The bare `fuse(a, b)` keeps the call site honest and order-free (RT2): because the op is a semilattice
join, `fuse(a, b)` and `fuse(b, a)` are observably identical, so no argument-order meaning is implied.

### §A.3 Grammar production (proposed)

```ebnf
fuse_expr   ::= 'fuse' '(' expr ',' expr ')' ;
```

`fuse` joins the expression grammar at primary/call position; it consumes exactly two comma-separated
sub-expressions (DN-57: `,` separates siblings, the call is `;`-terminated as any component).

### §A.4 Typing (proposed — Declared)

- `fuse(a, b)` type-checks iff `a : T`, `b : T`, and `T : Fuse` (a lawful-merge instance exists).
  Result type `T`.
- **Result `Meta` = `meet(meta(a), meta(b), strength(T::join))`** — honesty degrades, never upgrades
  (RFC-0001 §4.7; `meet(Proven, Empirical) = Empirical`).
- The semilattice-law obligation on `T::join` is discharged on the lattice (VR-5): **`Empirical`** when
  commutativity/associativity/idempotence are property-tested; **`Proven`** only with the checked
  side-conditions (RFC-0008 RT6's Isabelle/HOL basis applied to *this* instance). A `Fuse` instance
  whose laws are neither tested nor proven yields a `Declared` merge — flagged, never silently `Proven`.
- **No `Fuse` instance ⇒ `CheckError`** (never-silent, G2): "`fuse` requires a lawful-merge instance
  (commutative/associative/idempotent) for `T` — RFC-0008 RT6; declare `impl Fuse for T` or surface
  the conflict to a policy (RT3)."

### §A.5 Elaboration (proposed — Declared)

`fuse(a, b)` elaborates to: (1) a call to the resolved `T::join`; (2) a `Meta` `meet`-composition node;
(3) a `Provenance` `Derived{op:"fuse_join", inputs:[root(a), root(b)]}` node (RFC-0027 §10.6, so the
δ-CRDT Merkle anti-entropy story is available downstream for free). **No new L0 node** (KC-3): `join`
is an ordinary fn call; `meet` and `Derived` are existing `Meta`/`Provenance` constructors. Three-way
differential (L1-eval ≡ L0-interp ≡ AOT) per fusible shape; the result is `reveal`-able to exactly the
join-call + meta-meet + provenance form.

### §A.6 Open sub-questions (FLAGGED)

- **F-A1 — trait name.** `Fuse` (matches the keyword) vs `Mergeable` vs `Semilattice` vs `Join`. The
  DN-02 three-test gate should pick; `Fuse` is the plain-first default. **FLAG: maintainer.**
- **F-A2 — explicit-op form.** Should an explicit-op call site `fuse(a, b) via op` exist for ad-hoc
  merges (a value fusible *more than one lawful way*)? Default proposal: **no** — one canonical lawful
  merge per type keeps RT2 order-irrelevance unambiguous; ad-hoc merges go through a named fn. **FLAG.**
- **F-A3 — n-ary `fuse`.** `fuse(a, b, c)` as sugar for left-fold? Associativity makes it well-defined.
  Default: defer (binary only) to keep the first cut minimal. **FLAG.**

---

## §B `reclaim` — reclamation/supervision policy on a scope (RT7)

> **This is the surface RFC-0027 §10.5 explicitly deferred.** It carries the most uncertainty; the
> proposal below is a minimal first cut, heavily flagged. Tag: **`Declared`**, low confidence.

### §B.1 Operational basis (Empirical — RFC-0008 RT7, ADR-020, `mycelium-std-runtime`)

`reclaim` operates on the **structured-concurrency scope tree**: failure handling and reclamation
policies attach to scopes, OTP-style, and are **themselves reified policies** (RT3). It reclaims **stale
runtime units, never memory** (LR-9 makes memory reclamation automatic; DN-03 §4 fixes this scope). The
machinery is landed: `mycelium-std-runtime::supervision` provides `run_supervised`,
`supervise_with_restart` (bounded-cascade restart), `CancelTree` (structured cancellation),
`SupervisionRecord` (the audit/EXPLAIN record), `SupervisedFailure` (outcome classification)
(`Empirical`, cited `crates/mycelium-std-runtime/src/supervision.rs`). ADR-020 sequences **`reclaim`
surface before `hypha` raw-spawn surface** ("supervision before raw spawning makes the error model
cleaner").

### §B.2 Surface form (proposed — Declared, low confidence)

`reclaim` attaches a reified reclamation/supervision **policy** to a structured scope. Because
`colony { … }` is the landed scope construct (M-666), the proposed form is a `reclaim`-modified scope:

```
// attach a reclamation/supervision policy to a structured scope
reclaim(policy) {
    colony { hypha … }
}
```

where `policy` is a reified RT3 policy value (e.g. a `RestartIntensity`/supervision policy from
`std.runtime`). The block's hyphae run under that policy; on child failure the policy decides
restart/escalate/reclaim, and every decision is recorded in a `SupervisionRecord` (EXPLAIN). Reclaimed
units are stale *runtime units*, never memory (LR-9).

### §B.3 Grammar production (proposed)

```ebnf
reclaim_expr ::= 'reclaim' '(' expr ')' block ;
```

### §B.4 Typing + elaboration (proposed — Declared)

- `reclaim(policy) { body }` type-checks iff `policy : <supervision-policy type>` (the reified RT3
  policy; exact type per §B.6) and `body` is a scope expression (a `colony`/structured block).
- Elaborates to `mycelium-std-runtime::supervision::run_supervised` / `supervise_with_restart` with the
  policy, threading a `SupervisionRecord` for the EXPLAIN trail (RFC-0027 §9 record schema:
  `scope_id`, `event_type`, `value_meta_hash`, `trigger`). **No silent drop/pause** (G2): a reclamation
  or restart is always recorded. **No new L0 node** (KC-3) — dispatch to existing runtime APIs.
- Guarantee: a supervised scope's failure-handling claim is `Empirical` (the supervision machinery is
  property-tested, M-713); never `Proven` without a mechanized basis.

### §B.5 Transparency invariant

`reveal reclaim(policy) { body }` shows the `run_supervised(policy, body)` dispatch + the
`SupervisionRecord` wiring — the surface teaches "this scope is supervised by a *reified, inspectable*
policy," never an opaque runtime prerogative (RT3 + house rule #2).

### §B.6 Open sub-questions (FLAGGED — this is the deferred surface)

- **F-B1 — modifier vs block-attribute vs colony-clause.** `reclaim(policy) { colony … }` (proposed) vs
  `colony reclaim(policy) { … }` (a colony clause) vs a `@reclaim(policy)` attribute on a `hypha`/
  definition. The colony-clause form may compose better with M-666. **FLAG: maintainer — primary
  open question.**
- **F-B2 — the policy type.** What is `policy`'s surface type? A `std.runtime` `RestartIntensity`/
  `Supervisor` value? A reified RT3 policy enum? This binds to the M-356 supervision API. **FLAG.**
- **F-B3 — `reclaim` as expression vs item.** Does `reclaim` ever appear at item position (supervising
  a whole nodule's hyphae), or only as an expression around a scope? Default: expression-only first
  cut. **FLAG.**
- **F-B4 — interaction with `cyst`/checkpointing.** Reclamation of a dormable (encysted) computation:
  out of scope for the first cut; defer to the `cyst` activation. **FLAG (defer).**

---

## §C `tier` — execution-mode request (RFC-0004 `ExecutionMode`)

### §C.1 Operational basis (Empirical — RFC-0008 §4.2, RFC-0004 §9)

`tier` is **not a new mechanism**: switching interpreted ↔ native is RFC-0004's existing
`ExecutionMode` story — **observable-equivalent by NFR-7** (the §3 single certificate checker guarantees
the two paths agree). Compilation is **per-definition**, gated by the §4 stable-component check
(RFC-0004 §9.1). Crucially, a representation switch (dense ↔ sparse) is a **`Swap` (S1)**, *not* `tier`
— `tier` only selects the *execution mode*, never the representation, and never changes observable
semantics (it is a **performance request**, RT2/NFR-7).

### §C.2 Surface form (proposed — Declared)

Because tiering is per-definition and semantics-preserving, the proposed surface is a **definition-level
execution-mode request** — an attribute, not an expression that could imply a semantic effect:

```
// request compiled (AOT) execution for this definition when it is AOT-eligible
@tier(compiled)
fn hot_path(x: Binary{8}) => Binary{8} = …

// the explicit default; interpreted is always available (RFC-0004 §9.1)
@tier(interpreted)
fn dev_iter(…) => … = …
```

`tier` requests a mode; it never *forces* one in violation of the §4 gate. If a `@tier(compiled)`
definition is **not yet AOT-eligible** (not content-addressed/hash-frozen/spec-ratified per RFC-0004
§4), the request is honored as soon as it becomes eligible and, until then, is **never-silent**: the
compiler emits an EXPLAIN record stating the definition stays interpreted *and why* (which §4 obligation
is undischarged). NFR-7 guarantees the chosen mode is observationally identical.

### §C.3 Grammar production (proposed)

```ebnf
tier_attr   ::= '@' 'tier' '(' tier_mode ')' ;
tier_mode   ::= 'compiled' | 'interpreted' ;   // RFC-0004 ExecutionMode surface spelling (F-C2: 'compiled', not 'native')
```

(If the maintainer prefers a keyword-statement form over an attribute, §C.5 F-C1 covers it.)

### §C.4 Typing + elaboration (proposed — Declared)

- `@tier(mode)` is checked as an attribute on a definition; `mode ∈ {compiled, interpreted}` maps to
  RFC-0004 `ExecutionMode`. No effect on the definition's *type* (NFR-7: mode is non-semantic).
- Elaboration records the requested `ExecutionMode` on the definition's metadata; the §4
  stable-component gate + the §3 certificate checker decide actual AOT compilation. **No new L0 node**
  (KC-3) — `tier` annotates, it does not lower to a runtime op. An ineligible `compiled` request →
  EXPLAIN-recorded fallback to interpreted (G2), never a silent downgrade.
- Guarantee: interpreted↔native observable equivalence is `Empirical` by differential testing,
  `Proven`-per-artifact only via the §3 translation-validation certificate (RFC-0004 §3) — never
  `Proven` by assertion.

### §C.5 Open sub-questions (FLAGGED)

- **F-C1 — attribute vs expression vs `tier`-block.** `@tier(native) fn …` (proposed, definition-level)
  vs an expression `tier(native) { … }` vs a `tier native;` statement. The attribute form best matches
  "tiering is per-definition + semantics-preserving" (RFC-0004 §9.1). **FLAG: maintainer.**
- **F-C2 — mode vocabulary.** `native`/`interpreted` vs `compiled`/`interpreted` vs RFC-0004's exact
  `ExecutionMode` spelling (no concrete enum is spelled in RFC-0004; this note proposes the surface
  words). **FLAG.**
- **F-C3 — build-profile interaction.** RFC-0004 §9 adds build-target profiles (`interpret` /
  `build --slim` / `build --target …` / `build --fat`); does `@tier` interact with them or stay a pure
  per-definition hint? Default: per-definition hint; profiles are a build-CLI concern. **FLAG.**

---

## §D Common typing/elaboration plan (the `r4v` implementation shape)

The wave that implements this note (kickoff `r4v`, M-667 → M-710) follows the serial-on-L1 discipline:

1. **`parse.rs`** — remove `Fuse`/`Tier`/`Reclaim` from the reserved teaching-reject arms
   (item ~467–475, expr ~1210–1224); add `parse_fuse_expr`, `parse_reclaim_expr`, and the `@tier`
   attribute path per §§A.3/B.3/C.3.
2. **`ast.rs`** — add `Expr::Fuse { left, right }`, `Expr::Reclaim { policy, body }`, and a
   `tier: Option<ExecutionMode>` field (or attribute) on the definition node.
3. **`checkty.rs`** — typing per §§A.4/B.4/C.4 (the `Fuse` instance lookup; the policy-type check; the
   attribute check). Every refusal never-silent (G2).
4. **`elab.rs`** — dispatch per §§A.5/B.4/C.4 to `mycelium-std-runtime` (no `Residual` for the landed
   paths; a never-silent EXPLAIN where a gate is undischarged). **Zero new L0 nodes (KC-3).**
5. **conformance + differential** — migrate reject fixture #12 (drop `fuse`/`tier`/`reclaim` from the
   reserved-reject set; the others stay), add accept fixtures + three-way differential per construct.

**Staging note (honesty):** M-667 may land the surface with `fuse`/`tier` fully executing (machinery is
there) while `reclaim` lands behind a never-silent `Residual` until F-B1/F-B2 are ratified — the
`lambda` precedent. The DoD below permits that split rather than forcing a guess on the deferred form.

---

## §E Definition of Done (this note)

This note is **`Accepted`** when the maintainer ratifies: **(A)** the `fuse` expression surface +
`Fuse` trait + meet-on-guarantee typing (§A); **(B)** the `reclaim` surface shape — *resolving at least
F-B1 (the modifier/clause/attribute choice) and F-B2 (the policy type)* (§B); **(C)** the `tier`
execution-mode-request surface + mode vocabulary (§C); and **(D)** the common elaboration plan (§D).
Acceptance **enacts no code** — it unblocks the `r4v` wave to implement M-667/M-710 against a ratified
surface, and feeds the RFC-0008 §4.6 R1 activation record (ADR-020 guarantee-matrix row per construct).
Per the append-only rule it then moves `Draft → Accepted`; `→ Enacted` is gated on the `r4v` landing.

**Grounding:** RFC-0008 §4.1 (RT6/RT7) + §4.2 (`tier`) + §4.5 (vocabulary table) + §4.6 (R1 staging);
RFC-0027 §10.5 (the `reclaim` deferral this closes) + §10.6 (`fuse`↔RC, the provenance shape);
RFC-0004 §3 (certificate checker) + §4 (stable component) + §9 (the interpreted↔compiled continuum,
`ExecutionMode`); ADR-020 §"Phase-7 construct activation" (the per-construct activation pattern +
`reclaim`-before-`hypha` ordering); DN-03 §4 (ratified names + reserved-not-active status);
`crates/mycelium-std-runtime/src/{lib,supervision,scheduler,reclamation}.rs` (the landed machinery).

---

## Meta — changelog
- **2026-06-28 — implemented (Rust-first), Accepted → NOT YET Enacted (partial, VR-5)**:
  `fuse`/`reclaim`/`@tier` surface ACTIVE in `mycelium-l1` (r4v wave, M-667 done). Parse +
  AST + checker + elab dispatch landed. Repr-type `fuse` executes three-way (`Empirical`).
  RESIDUAL: `reclaim` elab stub + data-type fuse prim registration → follow-on M-817.
  NOT `Enacted` — execution partial; → `Enacted` gated on M-817 + M-710 closed. Status stays
  **`Accepted`** with this implementation note (append-only). Append-only.
- **2026-06-28 — moved from `Draft` to `Accepted`**: ratified by the maintainer (in-session).
  Maintainer decisions recorded (§A/§B/§C). Enacts no code; surface tagged `Declared`-with-argument.
  Append-only.
- **2026-06-27 — created at `Draft`**: proposes the L1 surface forms for `fuse`/`reclaim`/`tier`
  (the follow-on RFC-0027 §10.5 calls for, scoped to L1 surface), to unblock the `r4v` wave
  (M-667/M-710) against a ratified surface rather than a guessed one. Enacts no code; all proposed
  surface tagged `Declared`-with-argument; the `reclaim` surface (§B) flagged as the deferred,
  lowest-confidence part. Append-only.
