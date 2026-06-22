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

> **Note (2026-06-22 — append-only; M-660): effect annotations are a separate signature surface.**
> A fn signature may *also* carry an **effect annotation** `!{eff1, eff2}` after its return type
> (`-> ret !{…}`; absent ⇒ pure) — orthogonal to the `@ g` guarantee index above. Its surface +
> v0 coverage check (declared ⊇ performed) is pinned in **RFC-0014 §3.4/§4.5** and implemented
> Rust-first in `crates/mycelium-l1/` (no new L0 node — effects are checker metadata, KC-3).

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

> **Granularity superseded by RFC-0017 (2026-06-18; append-only — soundness unchanged).** This
> section's gate (`matured ⟹ total`) and totality classifier are **retained verbatim**; what RFC-0017
> changes is the **scope `matured` attaches to**. Maturation is no longer per-definition (`matured fn`
> is **retired**): it is declared at `nodule`/`phylum` scope (header) or program/package scope
> (manifest), and the gate is **quantified over that scope** — *every definition reachable in a matured
> scope must be `total`* (and AOT-eligible, RFC-0004 §4), **except** definitions marked `thaw`
> (kept interpreted; RFC-0017 §4.3). The conjunction is sound directly from the per-definition gate
> below, so nothing in the soundness argument moves. Read this section as the per-*definition*
> obligation that RFC-0017 §4.2 universally quantifies.

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

## 11. Stage-1 generic type parameters — the §4.4 deferral, discharged (append-only)

> **Append-only amendment (2026-06-22; M-656).** §4.4 states polymorphism is "deliberately out of
> v0 … its own later RFC", and §9/§10 name that RFC. That RFC is **RFC-0019 (Accepted 2026-06-18)**.
> This section records the consequence for the L1 calculus and pins the **minimally-sufficient
> stage-1 surface** that `crates/mycelium-l1` v1 must check. It changes **no v0 calculus content**
> (the ten-node budget, the §4.1 terms, the §4.4 monomorphic core judgments are all retained
> verbatim); it discharges the *deferral* by recording the stage-1 scope and its honest grounding.
> The §4.4 sentence "instantiating a generic is an explicit 'deferred' error, never a guess" is
> **superseded** by §11.3 (the error becomes a checked pass, never a guess — VR-5/G2 hold either way).

### 11.1 What is ratified, and by which document

Generics and traits were never ratified by *this* RFC — §4.4/§10 explicitly route them to a later
RFC. **RFC-0019** is that RFC and is **Accepted**: it ratifies (all **Declared-with-argument**,
not machine-checked — VR-5; `research/10` is the basis for a future `Proven` upgrade):

- **dictionary-passing elaboration** of bounded generics and traits to the *existing* L1 nodes
  (`Construct`/`Match`/`Lam`/`App`/`Let`/`Var`) — the **kernel node budget does not grow** (KC-3;
  RFC-0019 §4.3/§4.4);
- **coherence** = orphan rule + global uniqueness + reject-overlap, the only mechanism consistent
  with content-addressed identity (ADR-003; RFC-0019 §4.5; `research/10` T10.2–T10.3);
- the **Repr-polymorphism restriction set** (RFC-0019 §4.6; locally checkable, S1-preserving;
  `research/10` T10.5–T10.7) — code obeying it is admitted, code violating it gets
  `UnresolvedReprPolymorphism`.

This §11 adds nothing normative to RFC-0019; it **commits `mycelium-l1` to implementing the
stage-1 fragment of it** (M-657 for the unbounded core; M-659 for the bounded/trait core).

### 11.2 The stage-1 fragment `mycelium-l1` v1 must check (minimally sufficient)

The deferral is discharged in two honest steps, matching the dependency order of E7-1:

1. **Unbounded parametric generics (M-657 — this discharge's first step).** Generic *type*
   declarations and generic *function* declarations whose type parameters carry **no trait bound**:

   ```mycelium
   type List<A> = Nil | Cons(A, List<A>)
   fn is_empty<A>(xs: List<A>) -> Binary{1} = match xs { Nil => 0b1, Cons(_, _) => 0b0 }
   fn first_or<A>(xs: List<A>, d: A) -> A = match xs { Nil => d, Cons(x, _) => x }
   ```

   A type parameter `A` is an **abstract type variable**: the checker treats it as opaque — no
   representation-specific `Op` (`binary_and`, a `Swap`, a width-indexed primitive) may be applied
   to a value of type `A`, because its representation is not known (this is exactly the §4.6
   restriction of RFC-0019 specialized to the unbounded case; it falls out of the abstract-variable
   discipline and needs no separate machinery). Instantiation `List<Binary{8}>` substitutes the
   concrete type for `A`; arity is checked (a `List<Binary{8}, Ternary{3}>` is an explicit error).

   **Two honest boundaries of the v0 surface** (so the examples are not over-claimed): (a) a genuinely
   *partial* `head<A>(xs: List<A>) -> A` is **not** a total function — there is no `A` to return for
   `Nil`, and the exhaustiveness gate (W7) refuses a non-covering `match`; the total form is
   `first_or` above, or a `head` returning an `Option<A>`. (b) **Higher-order** generics like
   `map<A,B>(f: A -> B, …)` need a surface **arrow type**, which is an L2-with-lambdas concern — the
   v0 type surface has no `A -> B`, so `map` is deferred to L2, not part of this stage-1 fragment.

2. **Bounded generics + traits (M-658/M-659 — the second step).** `fn f<T: Eq<T>>(…)` and
   `impl Eq<Binary{8}> for Binary{8} { … }` elaborate via the RFC-0019 §4.3 dictionary-passing
   translation. This §11 records that they belong to the same discharge; their typing/elaboration
   rules are RFC-0019's, not restated here.

The v0 monomorphic judgments (§4.4 `T-Const…T-Match`) are the **base case** of the stage-1 judgment
(an empty type-parameter list is the identity). Every v0 program that checked before still checks.

### 11.3 Elaboration: checker now, monomorphization staged (the honest split, S1/VR-5)

M-657 lands in two honest halves (the implementation-of-record, `crates/mycelium-l1`):

- **The checker is complete** for the unbounded fragment: type-parameter declarations, generic
  functions, call-site **instantiation by unification**, arity, and the never-guess refusals all
  type-check (§11.2). Reusing the v0 monomorphic judgments as the base case keeps KC-3 — no new
  kernel node, no new trusted machinery.
- **Elaboration to L0 of a generic *instantiation* is staged** behind an explicit, never-silent
  `Residual`. A *monomorphic* program elaborates unchanged; a program that instantiates a generic
  type or calls a generic function elaborates to a clean `Residual` ("monomorphization staged"),
  **never** a partial or half-monomorphized artifact (G2). DN-14 §3 row 6 therefore moves to
  *type-checks; elaboration staged* (§11.4), not silently to `present`.

**Why staged, and the two elaboration strategies.** The L0 data **registry** (`FieldSpec` =
`Repr | Data(name)`) has no representation for an *abstract type-parameter field*, so a generic
declaration cannot be lowered abstractly. Two sound discharges exist, and this is the recorded
choice between them:

1. **Monomorphization** — the issue's "monomorphic-instantiation elaboration": specialize each
   concrete use into an ordinary monomorphic registry entry. Stays entirely within the frontend
   (no trusted-core change); the honest cost (RFC-0019 §4.4) is that a generic's content-addressed
   identity *fragments across specializations*. **This is the chosen path**; the `Residual` above is
   its not-yet-implemented placeholder.
2. **Uniform / dictionary-passing** (RFC-0019 §4.4's "one body, one hash" property) — would need a
   new abstract-parameter `FieldSpec` variant in the **trusted core** (`mycelium-core`), a
   KC-3-sensitive change. **Deferred**: it is the cleaner long-term identity story, but it is its own
   decision, not folded into this stage-1 frontend work.

**The S1 obligation (never-silent swap) is enforced at the checker, not deferred.** A `Ty::Var`
value is **representation-opaque**: applying a representation-specific `Op`/`Swap` to it is a
**check error** (the prim/`swap` signatures demand a concrete repr; `Var ≠ Binary{n}`), so the
Repr-polymorphic case is refused *before* elaboration — the elaborator is never asked to insert a
`Swap` for an instantiation (S1/W8). A mismatched or undetermined instantiation is a checked error,
**never a guess** (G2). This restates S1 at the polymorphic level and is the honest boundary.

### 11.4 DN-14 §3 row 6 — gate formally captured

DN-14 §3 row 6 ("Generic type parameters — `fn f<A,B>(…)`, `type List<A>`") was recorded
**gate-fails** with the evidence `checkty.rs:~167/~286` (the explicit deferral refusals). This
amendment is the **spec gate** that converts those refusals into checked passes. On M-657 landing,
row 6 moves to **"type-checks (checker present); L0 elaboration of generic instantiations staged
(monomorphization — §11.3)"** — *not* a bare `present`, because a stdlib nodule that *instantiates*
a generic cannot yet self-host through to L0 (that needs the monomorphization follow-up). The
honesty discipline (VR-5/G2) is unchanged — the refusal sites become real checks, and anything
outside the stage-1 fragment (Repr-polymorphism, higher-order generics, multi-parameter traits,
associated types — RFC-0019 §10 deferrals) stays an **explicit** refusal, never a silent accept.

### 11.5 Honesty posture of this amendment

This §11 is a **spec record**, not a new soundness claim. It leans entirely on RFC-0019's results,
which are **Declared-with-argument** (not machine-checked); accepting this amendment **does not
upgrade** that tag (VR-5). The `mycelium-l1` implementation it commits to is **Rust-first, pending
ratification** of the v1 polymorphic judgment as a checked basis — implementing the fragment is
evidence (`Empirical`, via the conformance + property corpus), not a `Proven` upgrade of RFC-0019's
coherence/S1-preservation arguments. No claim here is stronger than its basis.

## 12. Stage-1 trait interfaces + `impl` blocks — the LR-2 surface (append-only)

> **Append-only amendment (2026-06-22; M-658).** The companion to §11: §11 pins the *unbounded*
> generics surface, this section pins the **trait / bounded-generics** surface that `crates/mycelium-l1`
> v1 must check. Like §11 it adds **no v0 calculus content** and leans on **RFC-0019 (Accepted
> 2026-06-18)**, which is the LR-2 RFC; the trait checker is M-659.

### 12.1 The stage-1 trait surface `mycelium-l1` v1 must check

```mycelium
// A trait: a set of method signatures over one type parameter (single-parameter only in v1).
trait Eq<A> { fn equal(x: A, y: A) -> Binary{1} }

// An instance (witness) for a concrete type — the Rust idiom `impl Trait for T`.
impl Eq<Binary{8}> for Binary{8} { fn equal(x: Binary{8}, y: Binary{8}) -> Binary{1} = … }

// A bounded generic: a type parameter constrained by a trait (extends §11's unbounded core).
fn contains<T: Eq<T>>(needle: T, haystack: List<T>) -> Binary{1} = …
```

The existing grammar already parses `trait Ident type_params? { fn_sig* }` (RFC-0019 §3.1); v1 adds the
`impl_item` and bounded `type_param` productions (RFC-0019 §4.1 — additive; every v0 program still
parses). What v1 must **check** (RFC-0019 §4.5/§4.9):

- **Trait declarations** — method signatures type-check; the trait is a registry entry (RFC-0019 §4.2),
  not a kernel node (KC-3).
- **`impl Trait for T` blocks** — every method is provided at the right signature; **coherence is
  enforced**: **global uniqueness** (at most one instance per `(Trait, Type)`) and the **orphan rule**
  (the `impl` shares a `nodule`/`phylum` with the trait *or* the type). A violation is an explicit
  `CheckError` naming the conflict — **never** a silent shadowing (the content-addressed-identity
  argument, ADR-003 / RFC-0019 §2.2).
- **Bounded generics** — `fn f<T: Trait>(…)` resolves the instance at each call site
  (`inst(Trait, C) ↝ dict`); a missing instance is an explicit error naming the `(Trait, Type)` pair.

### 12.2 `impl` is reserved (M-658) — never a silent identifier

DN-03 §1 chose `impl` (over `embody`/`instance`) for both trait instances (`impl Trait for T`,
RFC-0019 §3.2) and inherent methods (`impl T { fn … }`, M-664). As of **M-658** `impl` is a **reserved
keyword** in the lexer (`token.rs` `keyword()` → `Tok::Impl`) — it can **never** lex as an identifier
(G2); a program using `impl` as a name is an explicit parse error (reject-corpus
`reject/14-impl-reserved-ident.myc`). The parser productions that *consume* `Tok::Impl` land with the
trait checker (M-659) and the inherent-method work (M-664); until then `impl` at item position is an
explicit refusal, never a silent accept.

### 12.3 Elaboration: dictionary-passing, staged the same way as §11.3

Bounded generics + traits elaborate by **dictionary-passing** (RFC-0019 §4.3): a trait becomes a
`Dict_Trait<A>` data declaration, an `impl` becomes a `Construct` dictionary, a bounded `fn` takes the
dictionary as an explicit first argument, and a method call is a field projection — **all existing L1
nodes, kernel budget unchanged** (KC-3). But a `Dict_Trait<A>` is *parameterized*, so lowering an
instantiated dictionary to L0 hits the **same** abstract-type-parameter-field obstacle as §11.3 — so
**elaboration is staged identically**: the *checker* (coherence, instance resolution, dictionary
*typing*) is M-659; the L0 lowering of a generic/trait *instantiation* stays an explicit never-silent
`Residual` until the **monomorphization follow-up (M-673)**, which discharges generics *and* traits
together. No new kernel node; no silent artifact (G2).

### 12.4 Scope & honesty (what is *not* in v1)

v1 is **single-parameter** traits only; **multi-parameter traits, associated types, and
newtype-derived coherence waivers are deferred to v2** (RFC-0019 §10 — they complicate the orphan
rule / need a roles mechanism). **Repr-polymorphism** (`R: Repr`) and **guarantee-indexed methods**
(LR-6 stage-2) stay the explicitly-refused / `Declared` cases of RFC-0019 §4.6/§4.7. DN-14 §3 row 7
gate is captured here. Honesty (VR-5): RFC-0019's coherence and S1-preservation results are
**Declared-with-argument**; this amendment **does not upgrade** them — implementing the checker is
`Empirical` evidence, not a `Proven` basis.

### 12.5 Implementation status (M-659 — implemented Rust-first, pending ratification)

As of M-659 the §12.1 surface is **implemented in `crates/mycelium-l1`** (the reference frontend),
pending RFC ratification — not flipped to `Enacted`. What landed (all explicit, never-silent — G2):
the parser productions (`impl Trait<args> for T { fn … }`; bounded fn type-params `<T: Cmp + Ord<T>>`
— with the single-parameter self-bound sugar `T: Cmp ≡ T: Cmp<T>`; bounds on `type`/`trait`
parameters are an explicit parse refusal); the trait checker — trait + instance registries on the
checked `Env`, a head-granular **coherence key** (`type_head`: width/shape erased, so stage-1
conservatively rejects two instances on the same head even at different widths — a documented,
deferrable refinement), the **trait pass** (method-sig resolution, duplicate trait/method refusals),
the **impl pass** (trait-arg arity, **global uniqueness**, the **single-nodule orphan rule** — local
trait OR local data type OR a primitive repr; cross-nodule enforcement staged with the phylum work,
M-662 — and exact method-set + per-method signature/body checking), **bounded-call satisfiability**
and **unqualified trait-method resolution** (concrete instance or bound-in-scope; ambiguity and
undetermined are explicit refusals, never a guess). The §4.6 Repr-polymorphism restriction is
unchanged — a bound does **not** grant representation-specific ops. **Elaboration stays staged**: a
trait-method / bounded-generic call's dictionary-passing L0 lowering is an explicit `Residual`
(→ M-673), exactly mirroring §11.3's generic-instantiation staging; **no new kernel node** (KC-3).
Honesty (VR-5): the checker entry points are tagged `Declared` (a structural registry check, not a
theorem); this status note records implementation as `Empirical` evidence, **not** a `Proven` basis,
and does **not** advance the decision's status.

## Meta — changelog

- **2026-06-22 — §12.5 added: the stage-1 trait checker is implemented Rust-first (M-659; append-only,
  no calculus change).** Records that `crates/mycelium-l1` now implements the §12.1 surface — the
  `impl`/bounded-`fn` parser productions, the trait/instance registries + coherence (global uniqueness +
  single-nodule orphan rule, head-granular keying), bounded-call satisfiability + unqualified
  trait-method resolution, all with explicit never-silent refusals (G2) — while **staging** the
  dictionary-passing L0 lowering to an explicit `Residual` (→ M-673), mirroring §11.3. No new kernel
  node (KC-3); RFC-0019's Declared-with-argument coherence result is **not** upgraded (the
  implementation is `Empirical` evidence; the decision status is unchanged — VR-5).
- **2026-06-22 — new §12: stage-1 trait interfaces + `impl` blocks; `impl` reserved (M-658; RFC-0019
  ripple, append-only, no calculus change).** The companion to §11: §12 pins the **trait /
  bounded-generics** surface `crates/mycelium-l1` v1 must check (single-parameter `trait`/`impl Trait
  for T` declarations + coherence = orphan rule + global uniqueness, per **RFC-0019** §4.5/§4.9), and
  records that **`impl` is now a reserved lexer keyword** (`Tok::Impl`; DN-03 §1) — never a silent
  identifier (G2), with reject-corpus `reject/14-impl-reserved-ident.myc`. Elaboration is **staged
  identically to §11.3** (dictionary-passing types-check in the checker — M-659; the L0 lowering of an
  instantiated dictionary is a never-silent `Residual` until monomorphization — M-673, which discharges
  generics + traits together). Multi-parameter traits / associated types / Repr-polymorphism stay
  deferred (RFC-0019 §10). DN-14 §3 row 7 captured. No v0 calculus content changed; leans on RFC-0019's
  Declared-with-argument results without upgrading them.
- **2026-06-22 — §11.2–§11.4 corrected to the *as-implemented* split (M-657; honesty fix, same
  session, pre-`main`).** The M-656 draft of §11 over-described elaboration as already **uniform**
  ("one body, one content-addressed hash"). The landed M-657 implements the **checker** in full
  (type variables, unification-based instantiation, arity, never-guess refusals) but **stages
  elaboration**: a generic *instantiation* lowers to an explicit, never-silent `Residual`
  ("monomorphization staged"), because the L0 registry `FieldSpec` cannot represent an abstract
  type-parameter field without a trusted-core change. §11.3 now records the two strategies
  (monomorphization = chosen, in-frontend; uniform/dictionary = deferred, needs a `mycelium-core`
  change) and §11.4 records DN-14 row 6 as *type-checks; elaboration staged* (not `present`). The
  §11.2 examples were also corrected (a total `first_or`/`is_empty` replaces the non-exhaustive
  `head`; higher-order `map` is noted as L2-with-lambdas — v0 has no surface arrow type). No v0
  calculus content changed; this only aligns the amendment's wording with the honest implementation
  (VR-5/G2).
- **2026-06-22 — §4.4 generics deferral discharged → new §11 (M-656; RFC-0019 ripple, append-only,
  no calculus change).** §4.4's "polymorphism/traits deliberately out of v0 — its own later RFC" is
  now routed to its destination: **RFC-0019 (Accepted 2026-06-18)** ratifies dictionary-passing
  elaboration (kernel budget unchanged, KC-3), orphan-rule coherence (ADR-003), and the
  Repr-polymorphism restriction set — all **Declared-with-argument**. New **§11** records the
  consequence and pins the **minimally-sufficient stage-1 surface** `crates/mycelium-l1` v1 must
  check: (a) **unbounded** parametric generics (`type List<A>`, `fn head<A>`, `fn map<A,B>`) with
  type parameters as abstract variables — M-657; (b) **bounded** generics + traits via RFC-0019
  dictionary-passing — M-658/M-659. The §4.4 "instantiating a generic is a deferred error" sentence
  is **superseded** by §11.3 (a checked pass, never a guess — VR-5/G2). The never-silent-swap
  obligation (S1/W8) is restated at the polymorphic level: instantiation never inserts a `Swap`;
  a Repr-polymorphic body that would need one is an explicit `UnresolvedReprPolymorphism`, never a
  silent insertion. DN-14 §3 row 6 gate captured (§11.4). **No v0 calculus content changed**; the
  amendment leans on RFC-0019's Declared-with-argument results and **does not upgrade** them (§11.5).
- **2026-06-19 — §8 R7-Q3 surface grammar decided (RP-6 → DN-13; M-391; append-only, surface
  commitment).** The remaining R7-Q3 sub-question — the *surface grammar* for a group of ≥2
  mutually-recursive top-level functions — is resolved: **nodule-wide mutual visibility, no new
  syntax** (RP-6 candidate 2; DN-13). Every top-level `fn` in a `nodule` is mutually visible; the
  elaborator auto-groups each call-graph SCC of ≥2 into the `FixGroup` of RFC-0001 r5 (the lowering
  already enacted by M-343), materializing the inferred grouping as a concrete L0 node (no black box).
  The grammar spec (`docs/spec/grammar/mycelium.ebnf`) records the scoping rule as a comment — **no
  production change**. M-391 pins it: two further surface-written mutual-recursion programs in the
  M-210 three-way differential, an identity assertion, and a never-silent regression. No calculus
  content changed (KC-3).
- **2026-06-18 — §4.3 stage-1 deferral superseded + §8 R7-Q2 resolved (RFC-0018 Accepted; ripple,
  append-only).** §4.3 deferred the static graded judgment ("stage 1, a revision of this RFC"); that
  stage-1 grading is now specified and **Accepted in RFC-0018**, which **supersedes §4.3's deferral**
  (the stage-0 dynamic check specified here remains the runtime semantics). **R7-Q2** (does a `Match`
  default arm meet-degrade differently from named alternatives once grading lands?) is **resolved**:
  under RFC-0018's adopted Design A (`G-Match/A`), the default arm is meet-folded identically to named
  alternatives — no special degradation. No normative change to this RFC's v0 rules; editorial
  cross-ref update.
- **2026-06-18 — §4.5 `matured` *granularity* superseded by RFC-0017; concrete-surface-syntax
  carve-out discharged (KC-2 verdict DN-09); append-only, soundness unchanged.** Two consequences of
  maintainer decisions recorded today, neither altering the calculus: **(1)** RFC-0017 (Accepted) lifts
  `matured` from per-definition to `nodule`/`phylum`/program **scope** (header + manifest), **retires
  `matured fn`**, and reserves **`thaw`** for de-maturation — §4.5 gains a note; the
  `matured ⟹ total` gate + the totality classifier are **retained verbatim**, RFC-0017 §4.2 merely
  *quantifies* them over the matured scope. **(2)** DN-09 records the **KC-2 verdict = proceed**, which
  discharges §10's "concrete surface syntax (KC-2)" deferral: the v0 grammar is now the **committed L3
  text surface** (RFC-0006 §10 Q1 resolved) — ratifying RFC-0007 still ratifies the *calculus*, and the
  surface is now committed by DN-09/RFC-0006 r5 rather than gated. No calculus content changed.
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
