# Design Note DN-14 — Self-Hosting Gate: Surface Language Readiness for stdlib Authoring

| Field | Value |
|---|---|
| **Note** | DN-14 |
| **Status** | **Draft** (2026-06-19) |
| **Feeds** | M-502 (#150); RFC-0016 (Core/Standard Library); RFC-0007 (L1 Kernel Calculus); DN-07 (RFC-0016 ratification); DN-13 (RP-6 / mutual recursion) |
| **Date** | June 19, 2026 |
| **Decides** | *Nothing normatively* — assessment note only. Enumerates the surface-language features required to author a stdlib module in Mycelium-lang (dogfooding); checks each against the M-391 + M-343 implementations (what is actually present in `crates/mycelium-l1/`); gives an honest verdict per feature. Self-hosting is **NOT declared** until all gate-fails are resolved. |
| **Task** | M-502 (#150) — self-hosting readiness gate |

> **Posture (honesty rule / VR-5).** This note is *advisory*. `present` verdicts are grounded in
> the actual codebase (cites the specific source module where the feature lives). `gate-fails`
> verdicts are recorded where a required capability either does not exist in `crates/mycelium-l1/`
> or is explicitly refused by the typechecker. No feature is pre-declared `present` on intent
> alone — the evidence is the code.

---

## 1. Why this note

M-346 conditions the whole stdlib decomposition on a precondition: the library migration order is
"Rust-first, Mycelium-lang eventually (dogfooding; free of other languages)." RFC-0016 §4.6 names
`diag` + `recover` as the **first migration targets** (the most honesty-load-bearing modules),
and M-502 (#150) gates the self-hosted batch on a concrete readiness verdict.

This note makes that precondition *checkable*: it enumerates what surface-language features a
stdlib module needs to be authored in Mycelium-lang, checks each against what M-391 and M-343
actually implemented (the L1 surface prototype in `crates/mycelium-l1/`), and records an honest
per-feature verdict. As DN-07 §5 records: *"The concrete L3 authoring surface stays KC-2-gated
(A2 ruling; RFC-0006 §10), so the M-502 self-hosting verdict honestly stays not-yet."* This note
documents why.

---

## 2. What a stdlib module needs

The following capabilities are required to author a non-trivial stdlib module (e.g., `std.error`,
`std.collections`, or `std.diag`) in Mycelium-lang, based on their specs
(`docs/spec/stdlib/*.md`) and the RFC-0016 §4.1 C1–C6 contract:

1. **Value types and literals** — integers, booleans, representation literals (Binary/Ternary),
   tuples; needed by any module that constructs or inspects values.
2. **ADTs (algebraic data types) and pattern matching** — `type Foo = Bar | Baz(T)` and `match`;
   needed by `error`/`result`/`option`-style modules and any data-carrying type.
3. **Functions, recursion, and mutual recursion** — `fn` declarations, self-reference, and
   mutually-recursive groups; needed by all algorithmic code.
4. **Let bindings and lambda abstractions** — `let` / `fn` anonymous-form; needed by combinators
   and higher-order ops.
5. **Nodule-level organization** — the `nodule path` header, single-nodule scoping; needed to
   express any library unit.
6. **Generic type parameters** — `fn map<A, B>(f: A -> B, xs: List<A>) -> List<B>`; needed by
   `collections`, `iter`, `cmp`, `error`, and most Ring-2 modules.
7. **Trait-like interfaces (guarantee/EXPLAIN contract)** — `trait T<…> { fn … }` syntax plus
   the ability to *implement* traits on types; needed to express the C1–C6 RFC-0016 §4.1
   guarantee/EXPLAIN contract in the language itself (RFC-0016 §4.1, LR-2).
8. **Effect annotations (RFC-0014 RT3)** — declared effects like `{time, entropy}` on functions;
   needed by `rand`, `time`, `io`, `fs`, and any module with non-pure ops (RFC-0014 §4.3, RT3).
9. **`wild` blocks / FFI surface** — the `wild { … }` keyword for calling host operations;
   needed for any module that bottoms out in a syscall (e.g., `fs`, `rand`, `io`) per LR-9 and
   the `std-sys` split (RFC-0016 §8-Q6).
10. **Full phyla and cross-nodule imports** — `phylum std` declarations + `use` across nodule
    boundaries; needed to organize a multi-file library (DN-06; RFC-0016 §8-Q2).
11. **Refinement / dependent types for guarantee-matrix encoding** — the ability to express
    per-op guarantee tags (e.g., `Exact ⊐ Proven ⊐ Empirical ⊐ Declared`) as first-class
    surface types, so the guarantee matrix is checkable in the language itself; needed for the
    RFC-0016 §4.5 obligation (VR-5 / LR-6).

---

## 3. Readiness assessment

Evidence base: `crates/mycelium-l1/src/` (M-343 elaborator; M-391 mutual-recursion; M-320 nested
patterns; `checkty.rs` — the v0 monomorphic typechecker). The checker's own comments are the
primary evidence for refusals.

| # | Feature | Required for | Evidence / Grounding | Verdict |
|---|---|---|---|---|
| 1 | **Value types** — integers (i64), boolean via ADT, repr literals (`Binary{n}`, `Ternary{m}`, `Dense{d,s}`, `VSA{…}`), tuples via ADT | Any stdlib module | `ast.rs` `BaseType`; `checkty.rs` `Ty`; `eval.rs`; M-343 | **present** |
| 2 | **ADTs + pattern matching** (including nested patterns, Maranget usefulness check) | `error`, `collections`, `recover`, most modules | `ast.rs` `TypeDecl`/`Ctor`; `checkty.rs` `check_type_decl`; `decision.rs` Maranget; M-320/M-343 | **present** |
| 3 | **Functions + self-recursion + mutual recursion** (nodule-wide; Tarjan SCC → `FixGroup`) | All | `elab.rs` `FixGroup`; `checkty.rs` Pass 2 + Pass 3; DN-13; M-343 + M-391 | **present** |
| 4 | **Let bindings + lambda abstractions** (`let`, anonymous `fn`-forms, `for` sugar) | All combinators, `iter`, `error` | `ast.rs` `Expr::Let`; `elab.rs` `elab_lam`; `ast.rs` `Expr::For`; M-343 | **present** |
| 5 | **Nodule-level organization** (`nodule` header, single-nodule scoping, `use path`) | Any library unit | `ast.rs` `Nodule`/`Item::Use`; `nodule.rs`; DN-06; M-343 | **present** |
| 6 | **Generic type parameters** (`fn f<A, B>(…)`, `type List<A>`) | `collections`, `iter`, `cmp`, `error`, `math`, `text` | M-657: `checkty.rs` checks unbounded generics (type vars, unification-based instantiation, arity, never-guess); `elab.rs` **stages** the L0 lowering of a generic *instantiation* as an explicit `Residual` (monomorphization follow-up); RFC-0007 §11 | **partial — type-checks; elaboration staged** |
| 7 | **Trait-like interfaces** (`trait T { fn … }`) + impl blocks | RFC-0016 §4.1 C1–C6 contract machinery in-language; `iter`, `cmp`, `fmt` | `checkty.rs` line 297: `Item::Trait(_)` is skipped (no check arm); RFC-0007 Accepted scope explicitly defers "traits/LR-2" per RFC-0007 status field; AST parses `TraitDecl` but `checkty` ignores it | **gate-fails** |
| 8 | **Effect annotations (RFC-0014 RT3)** — declared `{time, entropy, io, …}` on surface `fn` | `rand`, `time`, `io`, `fs`, `recover` | No effect-annotation syntax in `ast.rs` `FnSig` or `FnDecl`; `checkty.rs` has no effect-checking pass (RFC-0007 §4.3: "stage 1, a revision of this RFC"); RFC-0014 effects exist only in the L0 interpreter budget layer (`mycelium-interp`) | **gate-fails** |
| 9 | **`wild` / FFI surface** — callable host operations | `fs`, `rand`, `io` (std-sys call sites) | `checkty.rs` line ~454: *"`wild` is denied by default (LR-9): no host FFI capability exists in v0, so a wild block cannot be checked or run — this refusal is the design, not a gap"*; `ast.rs` `Expr::Wild` parses but typechecker rejects | **gate-fails** |
| 10 | **Full phyla + cross-nodule imports** — `phylum std`, `use` across nodule boundaries | Any multi-nodule library | `ast.rs` `Item::Use(Path)` is parsed; `lib.rs` notes "v0 is single-nodule"; `checkty.rs` does not resolve cross-nodule paths; `ast.rs` notes `phylum` is a **reserved keyword** (DN-06) but no phylum-level elaboration exists | **missing (partial)** |
| 11 | **Refinement / dependent types for guarantee-matrix encoding** (guarantee index as first-class surface type, checked statically) | Per-op guarantee machinery, RFC-0016 §4.5 | `ast.rs` `TypeRef.guarantee: Option<Strength>` parses the index; `checkty.rs` note: "stage-0 semantics… runtime tags + meet"; RFC-0007 §4.3: "static graded judgment is stage 1, a revision of this RFC" — stage-1 is not implemented | **gate-fails** |

**Verdicts defined:**
- **present** — the feature exists and is exercised in the typechecker/elaborator (grounded in source).
- **gate-fails** — the feature is required but absent or explicitly refused; self-hosting is blocked until it is implemented.
- **missing (partial)** — the feature is partially present (parsed, reserved) but not functional; recorded as a separate status because it is closer to implemented than a pure gate-fail.

---

## 4. Verdict

**Self-hosting is not yet established.**

Of the 11 required features, **5 are present** (features 1–5: value types, ADTs + pattern
matching, functions + recursion including mutual recursion, let/lambda, nodule-level organization).
**5 are gate-fails** (features 6–9, 11: generics, traits, effect annotations, `wild`/FFI,
static guarantee index). **1 is partially missing** (feature 10: cross-nodule phyla).

The **blocking gates** for any non-trivial stdlib module authored in Mycelium-lang are:

- **Generic type parameters** — without polymorphism, no `List<A>`, no `Option<T>`, no `Result<T,E>`
  at the surface level; the `collections`, `iter`, `error`, `cmp`, `text`, `math` modules cannot
  be authored. (RFC-0007 §4.4 defers this as stage-1; no surface-level generics in v0.)
- **Trait interfaces** — without `trait` / `impl` blocks functioning in the typechecker, the
  RFC-0016 §4.1 C1–C6 guarantee/EXPLAIN contract cannot be expressed as a surface constraint;
  modules cannot declare conformance in-language. (RFC-0007 defers traits/LR-2 from the accepted
  v0 scope.)
- **Effect annotations** — without declared effects at the surface (`fn f() -> T / {time}`), the
  RFC-0014 RT3 contract cannot be expressed or checked; `rand`, `time`, `io`, `fs` cannot carry
  the honesty invariant. (Deferred to RFC-0007 stage-1.)
- **`wild` / FFI surface** — without an auditable `wild` block that the typechecker accepts, no
  module can bottom out in a syscall; `fs`, `rand`, `io` cannot be authored in Mycelium v0. (Denied by design in
  v0 per LR-9; the `std-sys` split — RFC-0016 §8-Q6 — is the roadmap path.)
- **Static guarantee index** — without stage-1 static graded type checking, the guarantee matrix
  (RFC-0016 §4.5) cannot be expressed as checked surface types; guarantee tags remain runtime
  metadata, not surface obligations.

Until these gate-fails are resolved, stdlib modules authored in Mycelium-lang will not have access
to polymorphism (collections/iter/error/cmp), trait-based conformance verification (the C1–C6
contract), declared effects (pure vs effectful modules), host interop (io/fs/rand), or a
statically-checked guarantee lattice. The current surface is sufficient for **single-nodule,
monomorphic, pure, representation-only programs** — adequate for validating the kernel calculus
(M-343's mandate) but not for authoring a production stdlib module.

The M-391 + M-343 deliverables are exactly what they claim: the nodule-scoped elaborator,
type inference, L0 lowering, and mutual recursion. The self-hosting gate is a separate,
later-phase goal, honestly `not-yet`.

---

## 5. Non-blocking open questions (not self-hosting gates)

The following RFC-0016 §8 questions are open but do **not** block the M-502 verdict (they do
not change whether stdlib authoring in Mycelium-lang is currently possible):

- **Q3 (ergonomics vs contract)** — the RFC-0012 ambient-representation implicit-vs-explicit
  tension. Accepted as a *direction* in DN-07 §3-Q3; discharged as per-ring design pass M-540.
  Does not block self-hosting (it is a library-ergonomics call, not a language capability gap).
- **Q4 (`runtime`/`colony` sequencing)** — deferred to Phase-7 (the RFC-0008 fungal concurrency
  constructs). A `runtime` phylum is reserved vocabulary; does not block other modules.
- **Lexicon consistency** (Q2 — the `core`↔`error` error-value name, `phylum std` naming) —
  a DN-level design call, not a language capability blocker.

---

## Meta — changelog

- **2026-06-22 — §3 row 7 spec gate landed; `impl` reserved (M-658; append-only, no row flip yet).**
  The **trait** spec gate is in place: **RFC-0007 §12** pins the stage-1 trait surface (single-parameter
  `trait`/`impl Trait for T` + coherence = orphan rule + global uniqueness, per RFC-0019), and **`impl`
  is now a reserved lexer keyword** (`Tok::Impl`) — never a silent identifier (G2; reject-corpus
  `reject/14-impl-reserved-ident.myc`). **Row 7 stays `gate-fails`** — only the landed M-659 trait
  checker (declaration + `impl`-block checking + coherence + dictionary-passing typing) flips it (and,
  like row 6, the L0 elaboration of an instantiated dictionary is staged → M-673). Spec gate, not an
  implementation (VR-5). (RFC-0007 §12; M-658, E7-1)
- **2026-06-22 — §3 row 6 → *partial* (M-657 checker landed; elaboration staged; append-only).**
  The generics **checker** is implemented in `crates/mycelium-l1` (RFC-0007 §11): type parameters as
  abstract variables, generic data + function declarations, **call-site instantiation by
  unification**, arity checks, and the never-guess refusals (undetermined parameter; a
  representation-specific op on a type parameter — the RFC-0019 §4.6 restriction). **L0 elaboration of
  a generic instantiation is staged** behind an explicit never-silent `Residual` (monomorphization —
  RFC-0007 §11.3), so row 6 is **partial**, *not* `present`: a stdlib nodule that *instantiates* a
  generic type-checks but does not yet self-host through to L0. Row 6 flips to `present` when the
  monomorphization follow-up lands (tracked under E7-1). Honest, never silent (VR-5/G2). (M-657, E7-1)
- **2026-06-22 — §3 row 6 spec gate landed (M-656; append-only, no row flip yet).** The **spec gate**
  for generics is in place: **RFC-0007 §11** (append-only amendment) discharges the §4.4 deferral by
  routing it to **RFC-0019 (Accepted)** and pinning the minimally-sufficient stage-1 generics surface
  for `mycelium-l1` v1 — (a) unbounded parametric generics (`type List<A>`, `fn head<A>`), type
  parameters as abstract variables (M-657); (b) bounded generics + traits via dictionary-passing
  (M-658/M-659). **Row 6 stays `gate-fails`** here — only the landed M-657 implementation (checker +
  elaborator, green `just check`) flips it to `present` (VR-5/honesty: a spec gate is not an
  implementation). This note records the unblock, not the closure. (RFC-0007 §11; M-656, E7-1)
- **2026-06-21 — M-649 DEFERRED (post-1.0, ADR-021 §5; M-648/M-649 editorial sweep).** M-649 (Self-hosting Stage-2: first stdlib module in Mycelium-lang) is scoped post-1.0 per ADR-021 §5. Gate status: **5 present / 5 absent**. Present: value types + ADTs, pattern matching, functions + recursion, let/lambda, nodule organization. Absent (gate-fails): (1) generic type parameters (no `List<A>`/`Option<T>` without RFC-0019 enactment), (2) trait interfaces (`impl Trait` blocked — RFC-0019 deferred LR-2), (3) effect annotations (declared effects `fn f() -> T / {time}` — deferred RFC-0014 stage-1), (4) `wild`/FFI surface (denied by design in v0, LR-9; `std-sys` phylum is the roadmap path), (5) static guarantee index (stage-1 graded type checking — RFC-0018 accepted, not yet enacted). These five block all non-trivial stdlib modules from being authored in Mycelium-lang. M-649 stays OPEN with DEFERRED status — it is not blocked, it is scoped to Phase-6 (Stage-1 generics/traits RFC amendments). This note stays **Draft** (M-649 verdict is `not-yet`; self-hosting is not declared until gate-fails resolve). Append-only.
<!-- changelog: 2026-06-21 Tracking IDs assigned (E7-1 epic, M-656..M-664); append-only -->
**2026-06-21 — Tracking IDs assigned (append-only).** The five gate-fails (§3 rows 6–9, 11) and
the one missing-partial (§3 row 10) now have tracking issues under epic **E7-1** (L1 Stage-1
Language Completeness, Phase 5). Dependency order: M-656 (RFC-0007 spec: generics) → M-657 (L1
generics impl) → M-658 (RFC-0007 spec: traits + `impl`) → M-659 (L1 traits impl); then in
parallel: M-660 (effect annotations, row 8), M-663 (static guarantee / RFC-0018 enactment, row 11),
M-662 (phylum construct + cross-nodule, row 10). M-661 (wild/FFI, row 9) depends on M-660. M-664
(`consume`/`grow`/`impl` surface keywords) depends on M-659. Each row in §3 flips to `present`
when its tracking issue lands and `just check` confirms green. DN-14 Status → `Resolved`
(append-only) after all 5 gate-fail rows and the 1 missing-partial row are `present`. This note
itself does not flip any row — only a landed, confirmed implementation may do so (VR-5/honesty rule).

<!-- changelog: 2026-06-19 Draft created (M-502) -->
