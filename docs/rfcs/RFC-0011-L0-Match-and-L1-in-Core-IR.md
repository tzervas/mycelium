# RFC-0011 — L0 `Match` & the L1-in-Core-IR Revision

| Field | Value |
|---|---|
| **RFC** | 0011 |
| **Status** | **Accepted — r3 ENACTED** (2026-06-15: the staged r3 — data-and-matching core, flat `Match` — is now **folded into RFC-0001 (r3) and implemented in lockstep**. The §4.1–4.3 diff below is in force: `Construct` + flat `Match` + the content-addressed data registry + WF6/WF7/WF8 are L0 Core IR; the §4.6 `Residual` is narrowed for data/matching (RFC-0007 §4.6 note added); the M-320 elaborator now emits `Match`/`Construct`, the M-110 interpreter evaluates them, and the M-210 differential covers the data fragment (L1-eval ≡ elaborate→L0-interp). `Lam/App/Fix` remain the named **r4**. The prior "decision only; enactment sequenced" status is superseded by this entry — append-only.) |
| **Type** | Foundational / normative (once Accepted) |
| **Date** | 2026-06-15 |
| **Depends on** | RFC-0001 §4.5/§4.6 (frozen Core IR, WF1–WF5, content-addressing); RFC-0006 §3/§4.4 step 2 (layering; the named revision); RFC-0007 §4.1–4.6 (the L1 calculus this folds in: terms, registry, typing, the §4.6 fragment restriction this retires); ADR-003 (Unison identity); `crates/mycelium-l1` (the non-normative prototype: `usefulness`, `decision`, `checkty`, `elab`) |
| **Proposes a revision to** | **RFC-0001 r2 → r3** (extends the §4.5 node grammar; supersedes that grammar on acceptance, not before) |

---

## 1. Summary

The L0 Core IR is **frozen** at five nodes — `Const | Var | Let | Op | Swap` (RFC-0001 §4.5).
The L1 kernel calculus (RFC-0007) designs five more — `Lam | App | Construct | Match | Fix` — but
RFC-0007 §4.6 deliberately stops short of putting them *into* L0: its v0 elaboration covers only the
"evaluation-complete fragment" (`Const/Var/Let/Op/Swap` residue), and anything with data, matching, or
recursion is an explicit `Residual` refusal that runs on the L1 evaluator instead. RFC-0006 §4.4 step 2
and RFC-0007 §9 both name the missing step: *"add the L1 node set to the Core IR with WF rules — the
planned RFC-0001 revision."* This RFC **is that proposal.**

It is the **keystone** for two stalled half-tasks. **M-320** built the Maranget decision-tree compiler
(`mycelium-l1::decision`) but its leaves are *not yet emitted as L0 kernel nodes* because L0 has no
matching node. **M-310** (full-LSP) cannot do real document sync because there is no text→`Node` path
for any program that matches or constructs data. Both ride on giving the surface a path into L0.

Because revising a **frozen** layer is a normative act, this RFC **does not flip RFC-0001's status**.
It (a) specifies exactly what would change in RFC-0001 §4.5/§4.6, (b) recommends a **staged** revision
(`r3` = the data-and-matching core; `r4` = functions + recursion) so the kernel grows in two auditable
steps rather than one (KC-3), (c) recommends that the kernel node be the **flat `Match`** (RFC-0007's
already-designed form) with the Maranget tree kept as the *untrusted, inspectable* compilation
artifact, and (d) leaves the decision — and the `r3` bump — to the maintainer. The elaborator wiring
(`Match`/`Construct` → L0) lands **only after** ratification; until then the prototype keeps returning
`Residual`, honestly.

## 2. Motivation

L0 is the semantic ground truth: the reference interpreter (M-110) executes L0, and every certificate
and differential speaks about L0 values (RFC-0006 §3). As long as `Match`/`Construct`/`Fix` have no L0
form, three things are stuck:

1. **The surface has no path into the trusted base for the language's defining feature** — algebraic
   data + exhaustive `match` (LR-1). Programs that match run only on the L1 evaluator, never reach the
   AOT path, and are never checked by the M-210 differential against an L0 lowering.
2. **M-320's decision-tree compiler dead-ends.** It compiles a checked nested `match` into a flat
   `switch`/`leaf` tree over occurrences and verifies it (`has_reachable_fail`-free), but the tree is
   an internal analysis artifact — its leaves cannot become runnable L0 nodes (phase-3.md §9.9, scope).
3. **M-310's document sync is blocked.** Real LSP `didOpen`/`didChange` over Mycelium source needs a
   deterministic text→`Node` pipeline; without an L0 target for matching/data it can only ever sync the
   evaluation-complete fragment.

The design work is largely **done** — RFC-0007 §4.1–4.4 already gives the terms, the registry, the
typing judgments (`T-Con`, `T-Match`, `T-Fix`), and the WF rules (W6/W7/W8). What remains is the
*normative merge into the frozen grammar* plus the content-addressing and differential consequences.
This RFC packages that merge as a decision for the maintainer, with a recommended staging.

## 3. Guide-level explanation

Today the two layers look like this (RFC-0007 §3):

```text
 L0 (frozen, RFC-0001 §4.5):   Const | Var | Let | Op | Swap
 L1 (RFC-0007, five more):     Lam   | App | Construct | Match | Fix
 elaboration L1→L0:            only the Const/Var/Let/Op/Swap fragment; the rest ⇒ Residual error
```

The revision moves some or all of the L1 nodes **down** into L0, so the reference interpreter, the
content-addresser, and the differential cover them directly, and §4.6's fragment restriction retires
to the same extent.

**The recommended staging** splits the move into two RFC-0001 revisions, smallest-first (KC-3):

```text
 RFC-0001 r3  (this RFC's primary proposal — the data-and-matching core):
     L0 gains:  Construct | Match   + a content-addressed data registry (Σ)
     retires §4.6 Residual for:  data construction, matching (incl. `if`-as-match)
     unblocks:  M-320 (emit decision-tree leaves as L0 Match/Construct), M-310 (sync the match/data surface)

 RFC-0001 r4  (a named follow-on, this RFC §4.5 sketches it but does not finalize):
     L0 gains:  Lam | App | Fix
     retires §4.6 Residual entirely (full L1-in-Core-IR; RFC-0007 §9)
     resolves:  R7-Q1 (Fix node vs recursive-Let), R7-Q3 (mutual recursion)
```

Why split? `Construct`/`Match` are exactly what the two stalled half-tasks need, and a single flat
`Match` over a constructed scrutinee is **evaluable with no lambdas** (it is a case expression — its
arms are terms over `Let/Op/Const/Var/Swap`). Functions and general recursion (`Lam/App/Fix`) are the
larger, more independent jump — they carry their own questions (R7-Q1/Q3, the totality-gate interaction)
and do not block M-320/M-310 — so deferring them to `r4` lets each revision stay single-expert-auditable.
A maintainer who prefers the clean one-shot fold (RFC-0007 §9) can instead ratify the **full** five-node
move as `r3`; that alternative is laid out in §6.

**The kernel node is the flat `Match`, not the compiled tree.** RFC-0007 §6 already decided *"flat
alternatives keep W7 checkable locally and push pattern complexity into the (untrusted, inspectable)
elaborator, where Maranget compilation lives."* This RFC keeps that: the **trusted** L0 node is the flat
`Match` (one scrutinee, single-level constructor/literal alternatives, at most one default, coverage
*checked*); the M-320 Maranget decision tree stays an **untrusted compilation/analysis artifact above
the kernel** — it lowers nested surface patterns *to* the flat `Match` and cross-checks exhaustiveness,
but it is not itself a kernel form. So the kernel grammar gains **one** matching node, not a `Switch`
sub-language, and the trusted form is the one RFC-0007 already typed (`T-Match`).

## 4. Reference-level design (normative once Accepted)

This section is written as a **diff against RFC-0001 r2** (the frozen text). Nothing here is in force
until the maintainer accepts this RFC and bumps RFC-0001 to r3 (§4.7); the prototype's `Residual`
behavior is unchanged until then.

### 4.1 §4.5 node grammar — the added nodes (r3)

RFC-0001 §4.5's `Node` grammar gains two productions (the L1 forms from RFC-0007 §4.1, verbatim in
shape):

```ebnf
Node ::= Const { value: Value }
       | Var   { id: VarId }
       | Let   { id: VarId, bound: Node, body: Node }
       | Op    { prim: Prim, args: [Node] }
       | Swap  { src: Node, target: Repr, policy: PolicyRef }
       | Construct { ctor: CtorRef, args: [Node] }              (* NEW (r3): saturated; SC-3-transparent *)
       | Match     { scrutinee: Node, alts: [Alt], default: Option<Node> }   (* NEW (r3): flat *)

Alt      ::= { ctor: CtorRef, binders: [VarId], body: Node }    (* constructor arm *)
           | { lit:  Value,   body: Node }                      (* literal arm — Binary{n}/Ternary{m} *)
CtorRef  ::= "#" DeclHash "#" Nat                               (* Unison #T#i — RFC-0007 §4.2 *)
```

`Lam`, `App`, `Fix` are **not** added in r3 (deferred to r4, §4.5). A `Match` whose arms or default
contain a node with no r3 L0 form (an `App`, a `Fix` call) remains an elaboration `Residual` until r4 —
the fragment restriction narrows, it does not vanish.

### 4.2 §4.3 registry — data declarations (r3)

RFC-0001 §4.6 content-addressing is extended with the **data registry `Σ`** exactly as RFC-0007 §4.2
specifies (this RFC adopts that text as normative, it does not redesign it):

- A declaration `type T<a…> = C₁(τ…) | … | Cₙ(τ…)` is a **registry entry**, content-addressed over its
  α-normalized structure (constructor order significant; names are not identity — ADR-003).
- A constructor reference is `#T#i` (declaration hash ‖ constructor index).
- Mutually recursive declaration groups **hash as a cycle** (the Unison scheme; one hashing unit,
  members ordered by their placeholder-substituted hashes).

The registry is **environment, not term** (RFC-0007 §6 / §4.2): declarations are *not* L0 nodes, so the
term grammar does not grow with each data type, and WF4 (content-addressability of nodes) is preserved —
a `Construct`/`Match` node hashes over its structure plus the `CtorRef` hashes it mentions.

### 4.3 Typing & well-formedness (r3)

The typing rules are RFC-0007 §4.4's `T-Con` and `T-Match`, lifted into RFC-0001 §4.5's judgment
`Γ ⊢ e : Value<R>` / `Γ ⊢ e : τ` (with `Σ` the registry). The new **kernel WF invariants** lift
RFC-0007's W6/W7/W8 to RFC-0001 §4.5's WF list:

- **WF6 (saturation).** `Construct{ctor, args}` is fully applied: `len(args)` = the constructor's field
  count. Partial construction is *not* an L0 form (it needs `Lam`, r4) — an under-applied constructor is
  an explicit error, never a curried value. *(RFC-0007 W6.)*
- **WF7 (flat, checked-exhaustive match).** Every `Match` alternative binds exactly the constructor's
  arity; each constructor appears at most once; a `Match` with no `default` **must** cover every
  constructor of the scrutinee's type, and a literal `Match` (over the non-enumerated `Binary{n}` /
  `Ternary{m}` domain) **must** carry a `default`. Coverage is *checked, never assumed* (LR-1; this is
  M-320's `usefulness` analysis at the kernel boundary). *(RFC-0007 W7.)*
- **WF8 (no silent swap through elaboration).** No elaboration step that produces `Match`/`Construct`
  may introduce a `Swap`; representation changes stay lexically written (S1/SC-3). *(RFC-0007 W8.)*

WF1–WF5 are unchanged and apply to the new nodes (every `Repr` change is still a `Swap`; every node is
still content-addressable; lowering still preserves `Meta`).

### 4.4 Elaboration, evaluation & the differential (r3)

- **The elaboration target is the flat `Match`.** The M-320 Maranget compiler (`decision::Tree` of
  `Switch`/`Leaf` over occurrences) is the *untrusted, inspectable* lowering from nested surface patterns
  to the flat kernel `Match` + the scrutinee projections it needs. Its existing **fail-free cross-check**
  (`has_reachable_fail` after exhaustiveness) becomes the bridge: an exhaustive surface match must compile
  to a flat L0 `Match` whose coverage WF7 independently re-checks — usefulness, the tree compiler, and the
  kernel WF must all agree (defense in depth, never silent; phase-3.md §9.9).
- **§4.6 fragment restriction narrows.** RFC-0007 §4.6's `Residual` refusal is **retired for data
  construction and matching**: `elaborate` emits `Construct`/`Match` L0 nodes for those, and keeps
  returning `Residual` only for `App`/`Fix` (until r4). The wording change to RFC-0007 §4.6 is part of
  this revision.
- **The reference interpreter (M-110) gains `Construct`/`Match` evaluation** — small-step, mirroring the
  L1 evaluator's `try_match` semantics (first-matching alternative; binders bound left-to-right; literal
  arm on `repr+payload` equality; default on no match). The L1 evaluator already implements this; r3
  makes L0 the trusted definition and the L1 evaluator a *second path* over the same fragment.
- **Differential obligation (NFR-7).** On the data-and-matching fragment, **L1-eval**,
  **L0-interp-after-elaboration**, and (where the kernel subset reaches it) the AOT path must agree on the
  observable, validated through the single shared **M-210 checker** — the same obligation RFC-0007 §4.6
  states for the smaller fragment, now extended to cover matching/data. A divergent elaboration must be
  caught (the mutant-witness convention).

### 4.5 The r4 sketch — functions & recursion (NOT finalized here)

For completeness (and so the maintainer can weigh staged-vs-one-shot, §6), r4 would add `Lam | App | Fix`
(RFC-0007 §4.1), retiring §4.6 entirely. r4 must also resolve the questions RFC-0007 left open and pin to
"the RFC-0001 revision": **R7-Q1** (`Fix` node vs a recursive-`Let` flag — cosmetic at the hash level),
**R7-Q3** (mutual recursion in v0; declaration/`Fix` groups hash per RFC-0007 §4.2), and the interaction
of the `matured` totality gate (RFC-0007 §4.5) with an L0 that can now *represent* recursion. This RFC
**does not** decide r4; it only records that the staged path ends there. A maintainer choosing the
one-shot fold collapses r3+r4 into a single r3 (§6, Alternative C).

### 4.6 What does **not** change

- The four paradigm **kinds stay closed** (RFC-0001 §4.1); this RFC adds *term* nodes and a *data
  registry*, not a fifth `Repr` kind.
- The **guarantee lattice & honesty propagation** (RFC-0001 §4.7) are untouched: a `Match` result takes
  the `meet` of the chosen arm's guarantee with the scrutinee's, like any operation; no node here upgrades
  a guarantee.
- **No `Swap` semantics change** (WF8). Data/matching are Repr-transparent.

### 4.7 Process — how this gets ratified (append-only)

1. The maintainer reviews this RFC and **chooses** the staging (recommended r3-then-r4) **or** the
   one-shot fold (§6 Alternative C) **or** the low-level-`Switch` form (§6 Alternative B).
2. On acceptance: this RFC moves `Draft → Accepted`, and **RFC-0001 is bumped r2 → r3** with the §4.1/§4.3
   diff above folded into its frozen text (the append-only "supersedes the r2 §4.5 grammar" note), and
   RFC-0007 §4.6 gets the narrowing-of-`Residual` wording.
3. **Only then** does the elaborator wiring land (M-320 remaining half: `Match`/`Construct` → L0; the M-110
   interpreter cases; the M-210 differential extension). Until step 2, the prototype's `Residual` is
   correct and stays.

This RFC **does not perform** steps 2–3. Drafting it is append-only and leaves frozen-L0 frozen.

## 5. Drawbacks

- **The kernel grows.** Even the staged r3 adds two nodes + a registry to a five-node kernel — real
  surface against KC-3. Mitigation: the registry keeps the *term* grammar from growing per data type
  (RFC-0007 §6); the Maranget tree stays *out* of the kernel; and the staging means the maintainer ratifies
  the smallest useful increment first.
- **Two revisions instead of one** (the recommended staging). More process than a single fold, and r3
  leaves a *narrowed* fragment restriction (App/Fix still `Residual`) that some will read as half-done.
  Accepted as the price of an auditable kernel-growth path; Alternative C (§6) is the one-shot option for a
  maintainer who weights "one clean revision" higher.
- **Frozen-L0 precedent.** Revising RFC-0001 at all sets the precedent that the frozen layer *can* move.
  Mitigated by the append-only discipline (r2→r3 supersession, never rewrite) and by this RFC being the
  *named, pre-anticipated* revision (RFC-0006 §4.4 step 2), not an ad-hoc change.

## 6. Rationale & alternatives

- **Why the flat `Match` as the kernel node (Alternative A, recommended)?** RFC-0007 already typed it
  (`T-Match`) and gave its WF (W7); promoting it keeps the trusted form aligned with the in-draft L1 design
  and the M-210 differential, and keeps Maranget compilation *above* the kernel where RFC-0007 §6 put it.
- **Alternative B — a low-level `Switch`/`Leaf` over occurrences as the kernel form** (promote M-320's
  `decision::Tree` instead of the surface `Match`). The kernel form would be the already-checked,
  exhaustiveness-verified, fail-free *compiled* artifact — arguably closer to L0's mechanical spirit. But it
  still needs `Construct` + projection in L0, it is *further* from RFC-0007's `T-Match` (a second typing
  story), and it puts an occurrence/projection sub-language into the trusted base. Rejected as the
  recommendation, recorded as a real option the maintainer may prefer on minimalism grounds.
- **Alternative C — the one-shot fold (all five L1 nodes as r3, retire §4.6 entirely; RFC-0007 §9).** The
  cleanest single revision and exactly what RFC-0006 §4.4 step 2 / RFC-0007 §9 name. Rejected as the
  *recommendation* only because it front-loads `Lam/App/Fix` (which M-320/M-310 don't need) and their open
  questions (R7-Q1/Q3, the totality-gate interaction) into one larger kernel jump — a KC-3 cost. A
  maintainer who prefers one clean revision over two small ones should take C; this RFC's §4.5 already
  sketches its content.
- **Why not "just keep matching in L1 forever" (do nothing)?** Then the language's defining feature never
  reaches the trusted base, the AOT path, or the differential — and M-320/M-310 stay dead-ended. The
  layering was always designed to fold L1 into L0 (RFC-0006 §3); deferring indefinitely is the non-option.

## 7. Prior art

The same convergence RFC-0007 cites: **GHC Core** (a ~10-constructor `Expr` with a flat `Case` — the
direct precedent for promoting a small fixed node set with flat matching into the trusted core);
**Unison** (`#T#c` constructor refs, cycle hashing — the registry model, ADR-003); **Maranget 2008**
(decision-tree match compilation living *above* the kernel — why the tree stays untrusted); **Lean/Coq**
(the cautionary tale of a larger kernel — why r3 is staged and `Lam/App/Fix` are weighed separately).
No new sources; this RFC is a *merge proposal*, not new design (T3.1).

## 8. Unresolved questions

- **Q1 (staging).** r3-then-r4 (recommended) vs the one-shot fold (Alternative C)? **Maintainer's call.**
- **Q2 (kernel form).** Flat `Match` (recommended A) vs low-level `Switch`/`Leaf` (B)? **Maintainer's call.**
- **Q3 (literal-match defaults).** WF7 requires a `default` for `Binary`/`Ternary` literal matches (the
  domain is not enumerated). Confirmed consistent with M-320's existing W7 check — flagged for the
  maintainer to ratify as kernel WF, not just prototype behavior.
- **Q4 (deferred to r4, not this RFC).** R7-Q1 (`Fix` vs recursive-`Let`), R7-Q3 (mutual recursion), and the
  `matured`-gate/recursion-in-L0 interaction (RFC-0007 §4.5).
- **Q5 (AOT reach). RESOLVED (M-342, 2026-06-16).** The data + recursion fragment
  (`Construct`/`Match`/`Lam`/`App`/`Fix`) now lowers to ANF and runs on the `aot::run` **env-machine**,
  so the three-way differential (L1-eval ≡ L0-interp ≡ AOT) spans the full v0 calculus. The *native
  direct-LLVM* backend stays the bit/trit subset and refuses the rest with an explicit `UnsupportedNode`
  (VR-5) — data/closure native codegen is the deferred MLIR→LLVM work. (A follow-on, M-347, tracks
  making the env-machine's recursion stack-robust / more efficient.)

## 9. Future possibilities

r4 (functions + recursion) completes L1-in-Core-IR and retires §4.6 (RFC-0007 §9). Beyond that:
guarantee-indexed match arms (RFC-0007 R7-Q2 — does a `default` arm meet-degrade differently?), the
stage-1 static graded judgment (RFC-0007 §4.3), and `Match`/`Construct` in the AOT/SIMD path once the
backend grows data support.

## Meta — changelog

- **2026-06-15 — the §4.5 r4 sketch is now REALIZED in RFC-0001 r4 (note; append-only).** The named
  follow-on this RFC sketched (§4.5: `Lam`/`App`/`Fix` into L0, retiring §4.6 entirely) is enacted as
  **RFC-0001 r4** — R7-Q1 resolved (a `Fix` node), R7-Q3's content-addressed cycle *identity* fixed
  (elaboration of mutual recursion still deferred), and the `matured`-gate/recursion interaction
  restated (the interpreter clocks every `Fix`; the gate is packaging, never meaning). With r4,
  RFC-0007 §4.6's `Residual` is retired for the whole v0 calculus except mutual recursion + dynamic
  guarantee indices. This RFC's own scope (the r3 data-and-matching fold) is unchanged.
- **2026-06-15 — r3 ENACTED (the §4.7 steps 2–3, performed in lockstep).** With RFC-0006/0007
  ratified, the staged r3 is now **folded and implemented together** (spec never leads code): (2)
  **RFC-0001 r2 → r3** — §4.5 gains `Construct` + flat `Match` + `Alt` (supersedes the r2 grammar) +
  WF6/WF7/WF8; §4.6 gains the content-addressed data registry Σ (`CtorRef = #T#i`, Unison
  self-recursive placeholder hashing; mutual recursion deferred to r4); §4.2 gains `Datum` + the
  runtime sum `CoreValue` (a **sibling** type — `Value` unchanged — with a **meet-summary guarantee,
  no bound**, the one genuinely-open value-model choice, maintainer-confirmed); §4.7 gains the datum
  guarantee addendum. RFC-0007 §4.6 gets the `Residual`-narrowing note. (3) The wiring lands:
  `mycelium-core` (registry, `Datum`/`CoreValue`, the nodes, content-addressing/serialization,
  AOT-repr-only `is_aot_lowerable` per Q5), `mycelium-interp` (`Construct`/`Match` small-step +
  `eval_core`, the `Exact`-scrutinee meet identity + the explicit non-`Exact` refusal), and
  `mycelium-l1::elab` (the M-320 Maranget tree lowering nested patterns to nested flat L0 `Match`,
  `if`→Bool-match; App/Fix/`for` stay `Residual`, r4). The **M-210 differential** extends to the
  data fragment (L1-eval ≡ elaborate→L0-interp on the `CoreValue` observable, via `L1Value::to_core`,
  with a mutant-witness) — closing **M-310's** text→`Node` blocker (residual R1) and **M-320's**
  decision-tree-leaf emission. 497 workspace tests pass. r4 (`Lam/App/Fix`) remains the named future
  revision.
- **2026-06-15 — Accepted (decision recorded; enactment sequenced).** The maintainer chose the
  **staged** path (this RFC's recommendation): **RFC-0001 r3** = the data-and-matching core
  (`Construct` + flat `Match` + the content-addressed registry, with WF6/WF7/WF8), `Lam/App/Fix`
  deferred to a later **r4**; and the **flat `Match`** as the kernel node (Alternative A — the Maranget
  tree stays untrusted). **What this entry records is the *decision*, not its enactment.** RFC-0011
  depends on RFC-0007, and the maintainer subsequently directed that **RFC-0006 and RFC-0007 be
  completed and ratified before the core-language implementation**. To avoid a grounding inversion (an
  Accepted RFC-0001 r3 citing still-`Draft` RFC-0007 as normative), the §4.7 enactment steps — the
  RFC-0001 r2 → r3 text-fold (§4.1/§4.3), the RFC-0007 §4.6 `Residual` narrowing, and the M-320
  elaborator wiring (`Match`/`Construct` → L0, the M-110 interpreter cases, the M-210 differential
  extension) — are deferred and land **together as the core-lang step**, in this order:
  **Phase-3 exit-gate assembly → M-360 SIMD → complete + ratify RFC-0006/0007 → enact r3 + wire**.
  RFC-0001 stays **r2/frozen** until that step; the `mycelium-l1` prototype keeps returning `Residual`
  for these nodes, honestly, in the meantime. r4 (`Lam/App/Fix`, retiring §4.6 entirely) remains a named
  future revision.
- **2026-06-15 — Draft.** Initial draft. Proposes the named RFC-0001 revision (RFC-0006 §4.4 step 2 /
  RFC-0007 §9): fold the L1 data-and-matching core (`Construct` + flat `Match` + the content-addressed
  registry, with WF6/WF7/WF8) into the frozen L0 as **RFC-0001 r3**, staged ahead of an r4 that adds
  `Lam/App/Fix`. Recommends the flat `Match` as the kernel node (Maranget tree stays untrusted, RFC-0007
  §6) and the staged two-revision path (KC-3), with the one-shot fold (Alternative C) and the low-level
  `Switch` form (Alternative B) recorded for the maintainer. **Does not flip RFC-0001's frozen status or
  land the elaborator wiring** — ratification and the r3 bump are the maintainer's append-only decision
  (§4.7); the prototype keeps returning `Residual` until then. Unblocks (post-ratification) M-320's
  decision-tree-leaf emission and M-310's document sync.
- Maintain append-only with status transitions (Draft → Accepted → Superseded), mirroring the RFC
  discipline (RFC README). On acceptance, perform §4.7 steps 2–3 (the RFC-0001 r3 bump, the RFC-0007 §4.6
  wording, and the elaborator wiring) as separate append-only changes.
