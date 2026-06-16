# RFC-0007 — The L1 Kernel Calculus

| Field | Value |
|---|---|
| **RFC** | 0007 |
| **Status** | **Accepted** (r4 — ratified 2026-06-15 per §10: the **v0 kernel calculus** §4.1–4.8 (ten-node budget, registry, stage-0 *dynamic* guarantee check, `matured` totality gate, evaluation-complete-fragment elaboration, acyclicity, `for`-sugar). **Deferred (NOT ratified):** stage-1 static grading (§4.3; R7-Q2), R7-Q1/Q3 → RFC-0001 r4, R7-Q4, traits/LR-2, and concrete surface syntax (KC-2). Supersedes the r3 Draft.) |
| **Type** | Foundational / normative (once Accepted) |
| **Date** | June 10, 2026 |
| **Depends on** | RFC-0006 (layering L0–L3, S1–S6, LR-1…LR-9; DN-02 ratified lexicon); RFC-0001 §4.5/§4.6 (Core IR, WF1–WF5, content-addressing); RFC-0004 §4 (stable-component gate); ADR-003 (Unison identity); research T3.1/T3.2/T3.4/T3.5 (`research/03-language-layer-RECORD.md`) |
| **Coupled with** | `crates/mycelium-l1` (the non-normative prototype validating this draft); `docs/spec/grammar/` (the surface that elaborates here) |

## 1. Summary

L1 is the **kernel calculus** of Mycelium-the-language: the smallest typed term language that the
surface (L2/L3) elaborates into and that the trusted base executes. It is the L0 Core IR's five
nodes plus **five** more — `Lam`, `App`, `Construct`, `Match`, `Fix` — with **data-type
declarations living in a content-addressed registry, not in the term language** (the
GHC-Core/Lean/Coq/Unison convergence, T3.1). General recursion exists (`Fix`); a **structural
totality checker outside the kernel** classifies each definition `total` or `partial` (the
divergence bit, Q4/T3.4), and only `total` definitions may be **`matured`** (promoted stable
components, RFC-0004 §4). v0 elaboration to L0 is defined on the *evaluation-complete fragment*
(§4.6); making the L1 nodes themselves part of the Core IR is the planned RFC-0001 revision and
is **not** done by this document.

## 2. Motivation

RFC-0006 fixed the layering and the invariants; this RFC fixes the *content* of the one new
trusted layer. Everything above L1 is elaboration-defined (S4 — no semantics of its own), so L1's
node budget, typing judgments, and recursion/totality posture are the load-bearing decisions for
the whole language. Research Pass 3 found four independent ecosystems converging on the same
architecture; this RFC commits Mycelium to that convergence point with the project's honesty
rules layered on.

## 3. Guide-level explanation

A surface program (a `nodule`) declares data types, traits, and functions. Elaboration turns
those into: **registry entries** (one content-addressed declaration per data type, `#type#c` per
constructor) and **kernel terms** (one per function body) over exactly ten node kinds:

```text
 L0 (frozen, RFC-0001 §4.5):   Const | Var | Let | Op | Swap
 L1 (this RFC, five more):     Lam   | App | Construct | Match | Fix
```

`Match` is **flat** — one scrutinee, single-level constructor alternatives, at most one default —
because nested surface patterns are compiled away by the elaborator (Maranget-style decision
trees; T3.1). `Fix` gives general recursion; nothing in the kernel proves termination — instead
the **totality checker** (a separate, untrusted-for-semantics tool) certifies a structural
fragment, and that certificate gates privileges (`matured`), never meaning. The trusted
interpreter stays fuel-guarded (CakeML-style clocked semantics, T3.4): a wrong totality checker
can mis-gate a promotion, but can never change what a program computes.

## 4. Reference-level design (normative once Accepted)

### 4.1 Terms

```ebnf
term ::= Const(value)                          (* an L0 Value: repr + payload + Meta *)
       | Var(name)
       | Let(name, term, term)
       | Op(prim, [term])
       | Swap(term, repr, policy_ref)          (* never inserted; always written — S1 *)
       | Lam(param, type, term)
       | App(term, term)
       | Construct(ctor_ref, [term])           (* SATURATED: arity = the ctor's field count *)
       | Match(term, [alt], default?)          (* flat; alt = (ctor_ref, [binder], term) *)
       | Fix(name, type, term)                 (* general recursion; self-reference by name *)
```

- **W6 (saturation).** `Construct` is fully applied; partial construction is expressed with `Lam`.
- **W7 (flat match).** Every `Match` alternative binds exactly the constructor's arity; the same
  constructor appears at most once; a `Match` with no `default` must cover every constructor of
  the scrutinee's type (exhaustiveness is *checked*, never assumed — LR-1).
- **W8 (no silent swap).** Elaboration from any higher layer is Repr-transparent: no rule may
  introduce a `Swap` (restates S1 at the kernel boundary).

### 4.2 Data declarations — registry, not nodes

A data declaration `type T<a…> = C₁(τ…) | … | Cₙ(τ…)` is a **registry entry**, content-addressed
over its α-normalized structure (constructor order is significant; names are not identity —
ADR-003). A constructor reference is `#T#i` (declaration hash + constructor index), exactly
Unison's recipe. Mutually recursive declaration groups hash **as a cycle**: the group is one
hashing unit, members canonically ordered by their hashes computed with the cycle occurrences
replaced by a placeholder (T3.1; the Unison hashes scheme). The same cycle rule applies to
mutually recursive `Fix` definition groups.

### 4.3 Types

```ebnf
type ::= ReprTy(repr)                          (* Binary{n} | Ternary{m} | Dense{d,s} | VSA{…} *)
       | Data(decl_ref, [type])                (* a registry type, possibly applied *)
       | Arrow(type, type)
       | Substrate(tag)                        (* the affine external-resource kind — LR-8 *)
```

A type may carry the **guarantee index** `τ @ g` (LR-6). In this RFC's v0 the index is *checked
dynamically against `Meta`* (the stage-0 semantics of T3.2: runtime tags + meet); the static
graded judgment is **stage 1, a revision of this RFC**, with one rule already fixed: an
annotation may only *weaken* — asserting `@ g` on a term whose tag is weaker than `g` is an
error, never an upgrade (VR-5).

### 4.4 Typing judgments (v0: simple types, monomorphic)

`Γ ⊢ e : τ`, with `Σ` the data registry and `Π` the prim signature table:

```text
 T-Const  Σ ⊢ v wf                      T-Var   (x:τ) ∈ Γ
          ───────────────                       ──────────
          Γ ⊢ Const(v) : ReprTy(v.repr)         Γ ⊢ x : τ

 T-Let    Γ ⊢ e₁ : τ₁    Γ, x:τ₁ ⊢ e₂ : τ₂     T-Lam   Γ, x:τ₁ ⊢ e : τ₂
          ─────────────────────────────────            ─────────────────────────
          Γ ⊢ Let(x, e₁, e₂) : τ₂                      Γ ⊢ Lam(x, τ₁, e) : τ₁→τ₂

 T-App    Γ ⊢ f : τ₁→τ₂    Γ ⊢ a : τ₁          T-Op    Π(p) = (τ₁…τₙ) → τ    Γ ⊢ eᵢ : τᵢ
          ───────────────────────────                  ─────────────────────────────────
          Γ ⊢ App(f, a) : τ₂                           Γ ⊢ Op(p, e₁…eₙ) : τ

 T-Swap   Γ ⊢ e : ReprTy(r₁)    legal swap r₁→r₂ exists or is checked at run time
          ────────────────────────────────────────────────────────────────────────
          Γ ⊢ Swap(e, r₂, π) : ReprTy(r₂)

 T-Con    Σ(#T#i) = C(τ₁…τₙ)    Γ ⊢ eⱼ : τⱼ    T-Fix   Γ, f:τ ⊢ e : τ
          ──────────────────────────────               ────────────────────
          Γ ⊢ Construct(#T#i, e₁…eₙ) : T               Γ ⊢ Fix(f, τ, e) : τ

 T-Match  Γ ⊢ s : T    each alt (#T#i, x̄, eᵢ): Γ, x̄:fields(#T#i) ⊢ eᵢ : τ
          default (if any): Γ ⊢ d : τ    coverage: alts ∪ default ⊇ ctors(T)   (W7)
          ────────────────────────────────────────────────────────────────────
          Γ ⊢ Match(s, alts, default?) : τ
```

Polymorphism (type parameters, traits/LR-2) is **deliberately out of v0**: declarations may be
*parameterized* (and are hashed as such), but v0 type checking is monomorphic; instantiating a
generic is an explicit "deferred" error, never a guess. The trait system is its own later RFC.

### 4.5 The divergence bit & the `matured` gate (Q4/Q7; T3.4)

Every definition is classified **`total`** or **`partial`** by the totality checker:

- a definition with no (direct or mutual) recursion is `total`;
- a self-recursive definition is `total` iff every recursive call passes, in some fixed argument
  position, a **strict structural piece** of that parameter (a binder bound by a `Match` on the
  parameter or on one of its pieces) — Foetus-style structural descent;
- a **mutually-recursive group** (a strongly-connected component of the call graph, the
  `FixGroup` of RFC-0001 r5 / R7-Q3) is `total` iff there is a **mutual structural descent**: an
  assignment of one designated argument position `p(f)` to each member `f` such that *every* call
  from any member `f` to any member `g` passes, in `g`'s position `p(g)`, a **strict structural
  piece** of `f`'s parameter `p(f)` (smallness seeded by a `Match` on `p(f)` or its pieces, the
  same transitive notion as self-descent). This is sound by a single well-founded measure: along
  any path through the group the structural size of the designated argument strictly decreases at
  every call, so no infinite call path exists. Self-recursion is the size-1 case (the group is one
  member; `p(f)` ranges over its positions). The search over position assignments is bounded; a
  group too large to search, or one whose well-foundedness this structural criterion cannot
  witness, stays `partial` (incompleteness is honest — never an unsound `total`);
- everything else is `partial` — an honest classification, not an error.

**"Checked total" formally** = the reference interpreter terminates on it *for every sufficiently
large fuel* (CakeML clock quantification). The checker lives **outside** the trusted kernel: its
only power is gating `matured` (a `matured` definition **must** be `total` — refusing otherwise
is an explicit error), in addition to RFC-0004 §4's existing gates. *Flagged novel (no found
precedent — T3.4):* totality gating AOT promotion specifically; soundness is by analogy with
Lean's kernel-opaque `partial` (a mis-gate affects packaging, never meaning).

### 4.6 Elaboration to L0 (v0) and the path to full L1 execution

L0 has no functions, data, or matching, so **v0 elaboration is defined on the
evaluation-complete fragment**: definitions whose call graph is acyclic and whose bodies, after
inlining and normalization (the simply-typed, `Fix`-free fragment is strongly normalizing),
contain only `Const/Var/Let/Op/Swap` residue. Elaboration of anything outside the fragment is an
**explicit `Residual` error** (never a partial artifact). Programs outside the fragment run on
the **L1 fuel-guarded evaluator** (a big-step environment machine mirroring M-110's contract).
Differential obligation (NFR-7): on the fragment, L1-eval, L0-interp-after-elaboration, and the
M-150 AOT path must agree on the observable, validated through the M-210 shared checker. The
*full* answer — adding the five L1 nodes to the Core IR with their own WF rules — is the planned
RFC-0001 revision (RFC-0006 §4.4 step 2) and supersedes §4.6's fragment restriction when it lands.

> **Narrowed by RFC-0001 r3, then RETIRED by RFC-0001 r4 (enacted 2026-06-15).** The `Residual`
> refusal is **gone for the whole v0 calculus**: r3 made `Construct` + flat `Match` L0 nodes (data +
> matching elaborate; the M-320 Maranget tree lowers nested patterns; `if` → a `Bool` match), and r4
> made `Lam`/`App`/`Fix` L0 nodes — so **functions, self-recursion, and `for`** (a synthesized
> self-recursive `Fix` fold) now elaborate too. The only `Residual`s left are **mutual recursion** (a
> deferred elaboration step, R7-Q3) and a **dynamic guarantee index** `@ g` (§4.3, stage 0). The
> differential obligation now covers the data/matching **and** recursive fragments (L1-eval ≡
> elaborate→L0-interp); the AOT path stays repr-only for now (RFC-0011 §4.4 Q5).

### 4.7 Memory-safety semantics (LR-9)

L1 runtime values are **immutable and acyclic by construction**: recursion is through
*definitions* (`Fix` names, content-addressed), never through heap self-reference — a `Construct`
value can only contain values that existed before it. Reclamation is therefore precise
(reference-counting without cycle hazards; Perceus-style reuse is a backend optimization, T3.5).
`wild` is **denied by default**: in v0 there is no host FFI, so a `wild` block is rejected at
check time with an explicit diagnostic — the capability to run one must be introduced by a later
FFI RFC, and *granting* it will be lexical and auditable (DN-02 §5).

### 4.8 Bounded iteration — elaboration-defined sugar, no new kernel node (r2)

Resolves ADR-012 §7.2 (maintainer decision, 2026-06-10). The kernel stays functional and its
node budget unchanged: **iteration is sugar over structural recursion**, never a kernel form.

**The normative content is the elaboration rule.** A bounded-iteration expression over a value
of a *linearly recursive* data type `T` (v0 shape restriction: every constructor of `T` is
either a **nil** — no fields — or a **cons** — exactly one field of type `T` (the spine) and
exactly one non-`T` field (the element, of type `E`, the same `E` across all cons constructors);
anything else is an explicit refusal, with general catamorphisms deferred to L2-with-lambdas)
elaborates to a **synthesized self-recursive helper**:

```text
 for x in xs, acc = init => body     ⤳     %fold_T(xs, init)   where

 fn %fold_T(s: T, a: A) -> A =
     match s {
         Nil          => a,
         Cons(x, rest) => %fold_T(rest, body[acc ↦ a])
     }
```

with the spine walked head-to-tail (outermost constructor first) and the result the final
accumulator. The helper descends structurally on the spine, so the existing checker (§4.5)
classifies it **`Total` with zero extension** — iteration is bounded *by construction* (a value
is finite and acyclic, §4.7), not by programmer promise. Typing:

```text
 T-For   Γ ⊢ xs : T   (T linear-recursive, element type E)
         Γ ⊢ init : A        Γ, x:E, acc:A ⊢ body : A
         ─────────────────────────────────────────────
         Γ ⊢ for x in xs, acc = init => body : A
```

**Spelling vs semantics (the Q6 split, applied to control flow).** The elaboration rule above
is normative; the concrete spelling `for x in xs, acc = init => body` is **adopted** (maintainer
decision, 2026-06-10): spelling A — `for`-head, explicit accumulator binder — for its familiar
head, *binders not closures*, honest about v0's first-orderness. Like all v0 surface syntax it
remains under RFC-0006 §1's global KC-2 gate, and revisiting it later is an explicit recorded
decision (append-only), not a drift. A named-args `fold(xs, from: …, with: …)` arrives as an
ordinary **L2 library function** once lambdas land (same elaboration, no new syntax); the KC-2
benchmark's iteration tasks (kc2-09/kc2-10) remain as measurements of the choice, not its gate.
`for` joins the v0 reserved-word set (recorded in DN-03).

**What stays out.** `while`, `loop`, `break`, `continue`, and `return` remain **excluded and
unreserved** (DN-02 §6): unbounded iteration would undermine the divergence bit (§4.5), and
early exit is a later, explicit form (fold-to-`Option`), never ambient control flow. Honesty at
the diagnostic level: where these words already produce an error (juxtaposition is never valid
syntax; an unknown name is a check error), the diagnostic *teaches* — "Mycelium iterates by
recursion or `for` (§4.8)" — rather than reporting a generic failure.

## 5. Drawbacks

Ten nodes + a registry is more machinery than L0's five — but it is the smallest budget any
surveyed ecosystem has sustained for a real language (T3.1), and the registry keeps the term
grammar from growing with every data type. The v0 fragment restriction (§4.6) means some typed
programs cannot yet reach the AOT path; that is honest sequencing, not a design limit.

## 6. Rationale & alternatives

- **Recursors instead of `Fix`** (Lean): strongest certification, but a heavy elaborator and
  known kernel-reduction traps; wrong for a fuel-guarded-interpreter architecture (T3.1/T3.4).
- **Letrec-only** (GHC): no total fragment at all — incompatible with the `matured` gate.
- **Data declarations as term nodes** (rejected): every surveyed kernel keeps them in the
  environment; nodes would bloat the term grammar and complicate hashing for zero power.
- **Nested `Match`** (rejected): flat alternatives keep W7 checkable locally and push pattern
  complexity into the (untrusted, inspectable) elaborator, where Maranget compilation lives.

## 7. Prior art

GHC Core (10-constructor `Expr`, flat `Case`, TLDI 2007); Lean 4 (declarations + recursors;
kernel-opaque `partial`); Coq's `constr` (the cautionary larger kernel); Idris 2 (per-definition
totality pragmas); Unison (ABT hashing, `#x.n` cycles, `#T#c` constructors); CakeML (clocked
big-step trusted semantics); Maranget 2008 (match compilation). Full citations: T3.1/T3.4.

## 8. Unresolved questions

- **R7-Q1:** ~~`Fix` node vs a recursive-`Let` flag~~ → **resolved (RFC-0001 r4): a `Fix` node**
  (RFC-0007 §4.1's typed form; maintainer-confirmed).
- **R7-Q2:** the `Match` default's interaction with guarantee indices once stage-1 grading lands
  (does a default arm's body meet-degrade differently from named alternatives?).
- **R7-Q3:** mutual recursion → **partially resolved (RFC-0001 r4):** the content-addressed *identity*
  of a mutually-recursive group is fixed (the canonical cycle ordering, §4.2, is implemented); the
  surface→registry/`Fix`-group *elaboration* stays deferred (the prototype accepts only
  self-recursion). So the hashes will not move underneath the surface when it grows mutual recursion.
- **R7-Q4:** the prim signature table `Π` — currently a fixed builtin table; should become
  declarations with their own content addresses.

## 9. Future possibilities

Stage-1 static grading (T3.2: FlowCaml-style inference over the 4-chain); the trait RFC (LR-2);
sized-type totality beyond structural descent; the RFC-0001 revision folding L1 into the Core IR
and retiring §4.6's fragment restriction.

## 10. Ratification scope (ratified 2026-06-15 — the carve-out, **Accepted**)

This RFC moved `Draft → Accepted` on 2026-06-15 (maintainer sign-off). A completion-review found the
**v0 calculus complete and implementation-validated** — the non-normative `crates/mycelium-l1`
prototype exercises every part (terms, registry, the §4.4 typing judgments, the §4.5 totality/`matured`
gate, §4.6 elaboration with explicit `Residual`, §4.7 acyclicity, §4.8 bounded-iteration desugaring),
and the M-320 work added the Maranget usefulness analysis + decision-tree compiler against it. The split
of what is **now ratified** vs deferred:

**Ratified (the v0 calculus — stage-0).**

- **§4.1 terms** (the ten-node budget: L0's five + `Lam/App/Construct/Match/Fix`) and **§4.2
  declarations-as-registry** (content-addressed, Unison `#T#i` + cycle hashing) — the
  GHC-Core/Lean/Coq/Unison convergence (T3.1), prototype-realized.
- **§4.3 types** (stage-0: the guarantee index `τ @ g` checked **dynamically** against `Meta`; only
  *weakening* allowed) and **§4.4 typing judgments** (v0 simple/monomorphic: `T-Const…T-Match`),
  with W6/W7/W8 — prototype-realized; the `Match`-into-L0 path is now ratified (**RFC-0011** staged
  r3, the named revision this RFC's §9 points at).
- **§4.5 divergence bit + `matured` gate** ("checked total" = CakeML clock quantification; checker
  outside the kernel, gates packaging never meaning) and **§4.6 elaboration** (the
  evaluation-complete fragment, explicit `Residual`; the differential obligation) — prototype-realized.
  *Note:* RFC-0011 (ratified) **narrows** §4.6's `Residual` for data/matching when the r3 enactment
  lands; until then §4.6 is accurate as written.
- **§4.7 memory-safety** (immutable acyclic values; `wild` denied-by-default) and **§4.8 bounded
  iteration** (elaboration-defined `for`-sugar over structural recursion, `Total` by construction —
  the maintainer-adopted spelling A) — prototype-realized.

**Stays gated / deferred (explicit — NOT ratified by accepting v0).**

- **Stage-1 static graded judgment (§4.3).** A revision of this RFC: the static graded typing (vs v0's
  dynamic `Meta` check). Couples to RFC-0006 Q3's open implicit-flows decision; **R7-Q2** (does a
  `Match` default arm meet-degrade differently?) is part of it.
- **R7-Q1 (`Fix` node vs recursive-`Let` flag) and R7-Q3 (mutual recursion in v0).** Both deferred to
  the **RFC-0001 r4** revision (the `Lam/App/Fix`-into-L0 step that RFC-0011 §4.5 sketches and names);
  cosmetic at the hash level (R7-Q1) / additive (R7-Q3, groups already hash per §4.2).
- **R7-Q4 (prim table `Π` as content-addressed declarations).** A later refinement; v0's fixed builtin
  table is sound meanwhile.
- **Concrete surface syntax (L2/L3).** KC-2-gated via RFC-0006 §1 (M-002-external) — the prototype's
  grammar is non-normative; ratifying RFC-0007 ratifies the *calculus*, never a surface spelling.
- **Polymorphism / traits (LR-2).** Explicitly out of v0 (§4.4) — its own later RFC.

**Status line (now in force):** *Accepted — the v0 kernel calculus (§4.1–4.8, stage-0 dynamic
guarantee check); stage-1 static grading, R7-Q1…Q4, and concrete surface syntax remain later
revisions / KC-2-gated.*

## Meta — changelog

- **2026-06-16 — §4.5 mutual-descent classification (M-343 loose end; R7-Q3 fully resolved;
  append-only, completeness-only).** Extends the totality checker's §4.5 classification from
  self-descent only to **mutual structural descent** over a `FixGroup` (RFC-0001 r5): a
  mutually-recursive group is `total` iff a per-member designated argument position descends on
  every inter-member call (one well-founded measure). Self-recursion becomes the size-1 case. This
  closes the half of #105 (R7-Q3) that landed the `FixGroup` elaboration + three-way differential
  but left every mutual group conservatively `partial`; e.g. a `ping`/`pong` pair is now classified
  `total` (admits `matured`), while a non-productive mutual cycle stays `partial`. **No calculus
  content changed** and **soundness is unchanged** — the checker still only ever *gates `matured`,
  never meaning* (the runtime is fuel/`FixGroup`-clocked regardless); this only widens the set of
  groups recognized `total`, never relaxes the bar (incomplete-but-honest: a group the structural
  criterion cannot witness stays `partial`). Enacted in `crates/mycelium-l1::totality`.
- **2026-06-15 — §4.6 `Residual` retired for self-recursion; R7-Q1 resolved, R7-Q3 partially resolved
  (RFC-0001 r4 enacted; editorial, append-only).** `Lam`/`App`/`Fix` are now L0 Core IR nodes, so
  functions + self-recursion + `for` elaborate (only mutual recursion + dynamic guarantee indices stay
  `Residual`). §4.6's note and §8's R7-Q1 (→ a `Fix` node) and R7-Q3 (→ canonical cycle *identity*
  fixed; elaboration still deferred) updated to record the consequence. No calculus content changed.
- **2026-06-15 — §4.6 `Residual` narrowed (RFC-0001 r3 / RFC-0011 enacted; editorial, append-only).**
  Added the §4.6 note recording that the planned RFC-0001 revision has landed for **data and matching**:
  `Construct` + flat `Match` are now L0 Core IR nodes, so those programs **elaborate** (the M-320
  Maranget tree lowers nested patterns to the flat kernel `Match`) instead of returning `Residual`. The
  fragment restriction *narrows* — `App`/`Fix`/`for` keep returning `Residual` until r4. No calculus
  content changed; this records the consequence of the RFC-0011 enactment on §4.6's wording (RFC-0011
  §4.4 / RFC-0001 r3).
- **2026-06-15 (r4) — Accepted (maintainer sign-off).** Moved `Draft → Accepted` with the **§10
  carve-out**: ratified = the v0 calculus §4.1–4.8 (ten-node budget, registry, stage-0 dynamic
  guarantee check, the `matured` totality gate, evaluation-complete-fragment elaboration, acyclicity,
  bounded-iteration sugar), all prototype-realized in `crates/mycelium-l1` and exercised by the M-320
  usefulness + decision-tree work; the `Match`-into-L0 path is the ratified **RFC-0011** staged r3.
  **Deferred (NOT ratified):** stage-1 static grading (§4.3; R7-Q2), R7-Q1/Q3 → RFC-0001 r4, R7-Q4
  (prim-table-as-declarations), traits/polymorphism (LR-2), and concrete surface syntax (KC-2). A
  completion-review found the v0 calculus complete + implementation-validated; no design content
  changed on acceptance. The status line carries the carve-out (VR-5).
- **2026-06-10 (r3) — Draft, `for` spelling adopted (maintainer decision).** §4.8's spelling A
  (`for x in xs, acc = init => body`) moves from *provisional* to **adopted**: the maintainer
  chose to commit it now rather than hold it pending a KC-2 ablation run; the kc2-09/kc2-10
  benchmark tasks remain as measurements, not a gate. Like all v0 surface syntax it stays under
  RFC-0006 §1's global KC-2 gate; revisiting is an explicit recorded decision (append-only).
- **2026-06-10 (r2) — Draft, bounded iteration added (maintainer decision).** New §4.8: bounded
  iteration as **elaboration-defined sugar** over structural recursion — no new kernel node; the
  normative content is the desugaring to a synthesized self-recursive helper, `Total` by the
  existing §4.5 classifier with zero extension; v0 domain is linearly recursive (list-shaped)
  data with explicit refusals beyond it. Provisional spelling A (`for x in xs, acc = init =>
  body`) ships in the non-normative prototype grammar; named-args `fold` is the planned L2
  library form; the ratified spelling is KC-2-evidence-gated (T3.6). `while`/`loop`/`break`/
  `continue`/`return` stay excluded and unreserved, with teaching diagnostics where they already
  error. Resolves ADR-012 §7.2; `for`'s reservation is recorded in DN-03.
- **2026-06-10 — Draft.** Initial draft from the T3.1/T3.4 positions and the ratified DN-02
  lexicon: the ten-node term budget, registry-not-nodes data declarations with Unison cycle
  hashing, v0 monomorphic typing judgments, the divergence bit + `matured` gate (novelty
  flagged), the evaluation-complete-fragment elaboration with explicit `Residual` refusal, and
  the LR-9 acyclicity argument. Prototype: `crates/mycelium-l1` (non-normative until Accepted).
