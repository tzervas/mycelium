# Design Note DN-14 — Self-Hosting Gate: Surface Language Readiness for stdlib Authoring

| Field | Value |
|---|---|
| **Note** | DN-14 |
| **Status** | **Resolved** (2026-06-23; see §4 + changelog) |
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
| 6 | **Generic type parameters** (`fn f<A, B>(…)`, `type List<A>`) | `collections`, `iter`, `cmp`, `error`, `math`, `text` | M-657: `checkty.rs` checks unbounded generics (type vars, unification-based instantiation, arity, never-guess); `elab.rs` **stages** the L0 lowering of a generic *instantiation* as an explicit `Residual` (monomorphization follow-up); **M-673 landed the monomorphization elaboration (`mono.rs`) — a concrete generic instantiation now lowers to closed L0 (M-210 three-way differential green)**; RFC-0007 §11/§11.3 | **present** |
| 7 | **Trait-like interfaces** (`trait T { fn … }`) + impl blocks | RFC-0016 §4.1 C1–C6 contract machinery in-language; `iter`, `cmp`, `fmt` | **Checker landed (M-659):** `trait`/`impl` type-check with **coherence** (global uniqueness + single-nodule orphan rule), exact method-set conformance, and bounded-call/trait-method resolution — every violation an explicit `CheckError` (G2), guarantee `Declared`; **M-673 landed the L0 lowering**: dictionary-free **static instance resolution** + a reified EXPLAIN selection record (`mono.rs`) — a trait-method call lowers to a direct call to the coherent instance's method body; the literal RFC-0019 §4.5 runtime-dictionary form remains deferred to a trusted-core ADR (KC-3) | **present** |
| 8 | **Effect annotations (RFC-0014 RT3)** — declared `{time, entropy, io, …}` on surface `fn` | `rand`, `time`, `io`, `fs`, `recover` | **Landed (M-660):** `ast.rs` `FnSig.effects`; `parse.rs` parses the `!{ … }` annotation after the return type (absent ⇒ pure; duplicate-effect = parse refusal); `checkty.rs` `check_effect_coverage` enforces **declared ⊇ performed** (performed = union of every callee's declared effects — a top-level fn OR an unqualified trait method — checked over fn bodies **and** impl-method bodies, so an effect can never be hidden from a caller), under-declaration an explicit `CheckError` (G2/RFC-0014 I3), over-declaration allowed (I5); impl-method effects must equal the trait method's. Guarantee **`Declared`** (a structural coverage check). **No new L0 node** (KC-3); the runtime budget ledger stays `mycelium-interp::budget` (M-353); `wild`-sourced effects **arrived with M-661** (a `wild` block performs the `ffi` effect — see row 9) | **present** |
| 9 | **`wild` / FFI surface** — callable host operations | `fs`, `rand`, `io` (std-sys call sites) | **Gate landed (M-661); execution landed (M-720/M-721; RFC-0028 Accepted):** `@std-sys` is an explicit **nodule-header attribute** (`ast.rs` `Nodule.std_sys`; atomic `@std-sys` token; `parse.rs` `parse_nodule`), **not** a naming convention. `checkty.rs` `Cx::check_wild` admits a `wild` block **iff** the nodule is `@std-sys` **and** the `fn` declares `!{ffi}`; a `wild` elsewhere is a **hard `CheckError`** (G2, not a lint). The `wild` body is the **trusted/opaque** FFI escape — NOT recursively type-checked (audited, not verified — VR-5/ADR-014) — so it needs an expected/ascribed type (synthesis refuses). **Execution now lands:** `elab.rs` `elab_wild` lowers a `wild` host-call body to `Op { prim: "wild:<name>" }` — the reserved host-dispatch namespace, **no new Core-IR node** (KC-3) — and `eval.rs`/`mycelium-interp`/AOT dispatch it through the prim registry (the capability handle; RFC-0028 §4.3). The **three-way differential** (L1-eval ≡ L0-interp ≡ AOT) now covers a deterministic `wild`-backed op (`Empirical` for that op). An ungranted host op is an explicit `UnknownPrim` (never silent, G2); a non-host-call body is an explicit elaboration `Residual`. Guarantee **`Declared`** baseline (real syscalls), `Empirical` for the differentially-covered op. The Mycelium-level `just safety-check` audit (M-724) verifies `@std-sys` + `!{ffi}` + `// SAFETY:` per `wild` site. | **present (audited, std-sys context; type-checks, gates, **executes** three-ways — DN-14 row 9 staged-execution gap closed)** |
| 10 | **Full phyla + cross-nodule imports** — `phylum std`, `use` across nodule boundaries | Any multi-nodule library | **Landed (M-662):** `parse_phylum` parses a `phylum <path>` header + multiple `nodule` blocks (`ast.rs` `Phylum`; a header-less nodule is a phylum-of-one); `checkty.rs` `check_phylum` builds a qualified per-phylum import registry (pub-only) + a pub-blind coherence view, resolves cross-nodule `use` (specific + glob `use a.b.*`) with **never-silent** absent/private/duplicate/glob-ambiguity refusals (G2), and enforces the RFC-0019 §4.5 orphan rule **phylum-wide**; `pub` exports gate cross-nodule visibility (default private-to-nodule). Guarantee `Declared`. | **present** |
| 11 | **Refinement / dependent types for guarantee-matrix encoding** (guarantee index as first-class surface type, checked statically) | Per-op guarantee machinery, RFC-0016 §4.5 | **Stage-1a landed (M-663; RFC-0018 Enacted):** `grade.rs` is a static guarantee-grading checker pass (Pass 3d) enforcing the §4.3 judgment over the lattice `Exact ⊐ Proven ⊐ Empirical ⊐ Declared` — G-App (argument ⊒ parameter demand), G-Weaken (`@ g` may only weaken), meet composition (G-Let/G-Con/G-Op), `G-Match/A` (Design A — data not control), G-Swap (cert reference trusted at the type level — R18-Q4); `elab.rs` no longer `Residual`s an `@ g` index (statically checked + erased — no L0 node, KC-3). Guarantee **`Declared`** (the noninterference theorem stays Declared-with-argument — not upgraded, VR-5). **Stage 1b (grade polymorphism) / stage 2 (refinements) remain future RFCs** | **present (stage 1a)** |

**Verdicts defined:**
- **present** — the feature exists and is exercised in the typechecker/elaborator (grounded in source).
- **gate-fails** — the feature is required but absent or explicitly refused; self-hosting is blocked until it is implemented.
- **missing (partial)** — the feature is partially present (parsed, reserved) but not functional; recorded as a separate status because it is closer to implemented than a pure gate-fail.

---

## 4. Verdict

**Self-hosting is not yet established.**

> **Update — 2026-06-23 (append-only, VR-5).** Since this §4 prose was written (the original M-502
> assessment), §3's table has advanced: rows 8 (effects, M-660), 10 (phyla, M-662), 11 (guarantee
> index stage-1a, M-663), and now **rows 6 (generics) + 7 (traits) → `present` (M-673 —
> monomorphization + dictionary-free static trait resolution lower both to closed L0)**, and now
> **row 9 (`wild`/FFI) → `present` (M-720/M-721; RFC-0028 Accepted — `wild` lowers to a `wild:`-namespaced
> `Op` and *executes* three-ways through the prim-registry capability handle; the staged-execution gap
> is closed)**. **§3's table is authoritative**; the prose below is the original assessment, and the
> full §4 reconciliation + **Status → Resolved** lands with **M-649** (the first self-hosted generic
> stdlib nodule).

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

- **2026-06-23 — §3 row 9 (`wild`/FFI) → `present`: execution lands (M-720/M-721; RFC-0028 Accepted;
  append-only, VR-5).** The staged-execution gap is closed. `elab.rs` `elab_wild` lowers a `wild`
  host-call body to `Op { prim: "wild:<name>" }` — the reserved host-dispatch namespace, **no new
  Core-IR node** (KC-3) — and `eval.rs` / `mycelium-interp` / the AOT env-machine dispatch it through
  the prim registry (the capability handle; RFC-0028 §4.3). The **three-way differential** (L1-eval ≡
  L0-interp ≡ AOT) now covers a deterministic `wild`-backed op (`Empirical` for that op; real syscalls
  stay `Declared`). Never-silent: an ungranted host op is an explicit `UnknownPrim` (the default
  registry grants none), a non-host-call body is an explicit elaboration `Residual` (G2). The
  Mycelium-level `just safety-check` audit (M-724) verifies `@std-sys` + `!{ffi}` + `// SAFETY:` per
  `wild` site. **This supersedes the prior entries' "row 9 stays staged" note** (those entries are kept
  verbatim — append-only — but row 9 is now `present`). `mycelium-std-sys` gains `io` + `sys` modules
  (M-722; `Declared`). The §4 "remaining open items: `wild`/FFI execution" item is hereby discharged.
- **2026-06-23 — §3 rows 6 + 7 → `present` (M-673; append-only, VR-5).** Monomorphization elaboration
  landed (`crates/mycelium-l1/src/mono.rs`): a generic instantiation lowers to closed L0 (row 6), and
  trait-method calls lower via dictionary-free static instance resolution + a reified EXPLAIN record
  (row 7). M-657/M-659 → done. The §4 prose carries an append-only update banner; the full §4
  reconciliation + **Status → Resolved** lands with M-649 (the first self-hosted generic nodule).
- **2026-06-22 — §3 row 11 static guarantee grading landed → row 11 now `present (stage 1a)` (M-663;
  RFC-0018 Enacted; append-only, VR-5).** The guarantee index `@ g` is now a **statically-enforced**
  surface constraint in `mycelium-l1` (RFC-0018 §4.3, stage 1a, Design A). A self-contained checker pass
  (`grade.rs`, Pass 3d) runs after type-checking and enforces the lattice `Exact ⊐ Proven ⊐ Empirical ⊐
  Declared`: G-App (a call argument's grade must satisfy its parameter's demand), G-Weaken (an `@ g`
  annotation/return demand may only *weaken*, never upgrade — VR-5), the meet composition rule
  (G-Let/G-Con/G-Op), `G-Match/A` (Design A — the scrutinee's *control* grade does not degrade the
  result; a destructured field inherits the scrutinee's *data* grade), and G-Swap (the endorsement point
  — the certificate **reference** is trusted at the type level, validity deferred to elaboration/runtime
  per R18-Q4). `elab.rs` no longer returns `Residual` for an `@ g` index — a grade, like a type, is
  statically checked and **erased** (no L0 node — KC-3). Unannotated types default modular/bottom
  (`Declared`), so grading only ever *bites* where an `@ g` is written (un-annotated code is unaffected).
  Guarantee **`Declared`** — the noninterference *theorem* stays **Declared-with-argument**, not upgraded.
  Stage 1b (grade polymorphism) and stage 2 (refinements) remain future RFCs. `cargo fmt`/`clippy
  -D warnings`/`cargo test -p mycelium-l1` + the full `just check` green. This **flips row 11 only**
  (rows 6/7 stay `partial`, row 10 `present`); DN-14 → `Resolved` still awaits the remaining staged rows.
- **2026-06-22 — §3 row 9 `wild`/FFI gate landed → row 9 now `conditionally present (audited, std-sys
  context; type-checks + gates; execution staged)` (M-661; append-only, VR-5).** The audited FFI floor is
  now *enterable* in `mycelium-l1` (RFC-0016 §8-Q6). **`@std-sys` is an explicit nodule-header attribute**
  (`nodule std.sys.fs @std-sys` — `ast.rs` `Nodule.std_sys`; the atomic `@std-sys` token in
  `lexer.rs`/`token.rs`; `parse.rs` `parse_nodule`), **not** a naming convention. The checker
  (`checkty.rs` `Cx::check_wild`) admits a `wild` block **iff** the nodule is `@std-sys` **and** the `fn`
  declares **`!{ffi}`** — `wild` is the `ffi` effect *source*, fed into the M-660 coverage pass — and a
  `wild` outside a `@std-sys` nodule is a **hard `CheckError`** (G2, **not** a lint; the issue's "lint"
  wording is amended to a hard refusal accordingly). The `wild` **body is the trusted/opaque FFI escape —
  NOT recursively type-checked** (it conforms to the expected type by ascription; audited, not verified —
  VR-5/ADR-014), so a synthesis position refuses with "ascribe …". **Execution stays staged** (no FFI host
  in v0 → `elab.rs` lowers `wild` to an explicit `Residual`), consistent with M-657/659/660. Guarantee on
  the gate **`Declared`**. The `myc-sec` `// SAFETY:`-presence audit (ADR-014) is **orthogonal + unchanged**
  (it is the SAFETY-comment audit; the typechecker is the std-sys *context* gate — the two are not coupled).
  Verified by `mycelium-l1` tests (`tests/check.rs` 10 new cases — accept in `@std-sys`/opaque body, reject
  outside `@std-sys`, reject without `!{ffi}`, synthesis-demands-ascription, impl-method gating, staged
  elaboration, attribute-not-convention; lexer/parser unit tests; `accept/18-wild-std-sys.myc`), `just
  check` green for `mycelium-l1`. This **flips row 9 only** (to *conditionally present* — the host-execution
  capability is the remaining staged work; rows 6/7 stay `partial`, row 11 stays `gate-fails`, row 10 stays
  missing-partial); DN-14 → `Resolved` still awaits those remaining rows. The §4 Verdict prose is the dated
  Draft snapshot (left as-is, append-only); this entry records the row-9 flip.
- **2026-06-22 — §3 row 8 effect annotations landed → row 8 now `present` (M-660; append-only, VR-5).**
  Stage-1 **effect annotations** landed in `mycelium-l1` (RFC-0014 §3.4/§4.5, the `!{…}` surface now
  pinned normative for the frontend): `FnSig.effects` (`ast.rs`), the parser's optional
  `!{ eff1, eff2 }` after the return type (`parse.rs` — absent ⇒ pure; a duplicate effect name in one
  annotation is a never-silent **parse** refusal), and the **effect-coverage** checker pass
  (`checkty.rs` `check_effect_coverage`): a fn's **declared** effects must be a **superset** of the
  effects it **performs** (performed = the union of the declared effects of every top-level fn it
  calls — the §8 manual-declare + compositional-**check** line, never inference). **Under-declaration**
  is an explicit `CheckError` naming the effect + the callee (G2/RFC-0014 I3); **over-declaration is
  allowed** (a declaration is a contract — I5); an **impl method's** effect set must **equal** the
  trait method's (exact match). Guarantee **`Declared`** (a structural coverage check, not a theorem).
  **No new L0 node** (KC-3) — effects are checker metadata and do not lower; the **runtime budget
  ledger stays `mycelium-interp::budget` (M-353)**, not wired by this frontend, and **`wild`-sourced
  effects expand the source set with M-661** (`wild` stays rejected here). Verified by the `mycelium-l1`
  effect test suite (`tests/check.rs` incl. a monotonicity property sweep + trait/impl
  effect-conformance; `tests/parse.rs` grammar; `accept/16`/`reject/17` conformance fixtures), `just
  check` green for `mycelium-l1`. This **flips row 8 only** (rows 6/7 stay `partial`, rows 9/11 stay
  `gate-fails`, row 10 stays missing-partial); DN-14 → `Resolved` still awaits those remaining rows.
  (RFC-0014 §4.5; M-660, E7-1)
- **2026-06-22 — §3 row 7 trait CHECKER landed → row 7 now `partial` (M-659; append-only, VR-5).**
  The stage-1 trait/impl **checker** landed in `mycelium-l1`: `trait`/`impl` type-check with **coherence**
  (global uniqueness + single-nodule orphan rule), exact method-set conformance, and bounded-generic +
  trait-method call resolution — every violation an explicit `CheckError` (G2), guarantee **`Declared`**
  (RFC-0019 §4.5, not machine-checked). The single-parameter self-bound sugar `T: Cmp ≡ T: Cmp<T>` is
  supported; multi-param/ambiguous bounds are explicit errors. **Dictionary-passing L0 lowering stays
  STAGED** (explicit `Residual`) → **M-673**, so **row 7 is `partial`**, not `present` — it flips to
  `present` only when M-673 lands the dictionaries (VR-5: no upgrade without the landed lowering). 187
  `mycelium-l1` tests green incl. a coherence property sweep + `accept`/`reject` conformance fixtures.
  (RFC-0007 §12, RFC-0019 §4; M-659, E7-1)
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

<!-- changelog: 2026-06-23 Status → Resolved (M-649; append-only, VR-5) -->
**2026-06-23 — Status → Resolved (M-649; append-only, VR-5).** The first self-hosted generic stdlib nodule (`std.result`) has landed: `lib/std/result.myc` type-checks, passes all four toolchain gates (`mycfmt --check`, `myc-check`, `myc-sec`, `myc-lint`), and its differential tests (`crates/mycelium-l1/tests/std_result.rs`) run to closed L0 on all three paths (L1-eval ≡ elaborate→L0-interp ≡ AOT, validated by the M-210 shared checker).

**§3 table status at Resolved:**
- Rows 1–8, 10–11: all **`present`** (grounded in landed implementations; see prior changelog entries).
- Row 9 (`wild`/FFI): **`conditionally present`** — type-checks + gated in `@std-sys` context; *execution stays staged* (no FFI host in v0; `elab.rs` lowers `wild` to an explicit `Residual`). This is the only remaining open row; it is not a gate-fail for `std.result` (which is pure and does not use `wild`).

**What M-649 closes:** §3 rows 6 + 7 are now `present` (M-673 — monomorphization + dictionary-free static trait resolution lower both to closed L0); `std.result` self-hosts as the first concrete evidence. The §4 prose ("self-hosting is not yet established") reflects the original 2026-06-19 assessment; the §4 update banner (2026-06-23) and this entry are the authoritative current state.

**Remaining open items (honestly recorded, not silently upgraded):**
- **`wild`/FFI execution** — row 9 stays staged; a host-execution capability for `wild` blocks is future work (no tracking conflict with this resolution).
- **Refinement stage-1b/2** — grade polymorphism and full refinement types remain future RFCs (row 11 is `present` at stage-1a only; no change from prior entry).
- **HOF combinators (`map`, `and_then`, `fold`)** — not yet executable in v0 (first-order surface: no function type `A -> B` as a value). Now captured as **clearly-marked pseudocode** in the companion `lib/std/result.pending-hof.md` (written in the finalized **RFC-0024** surface — function types + named-fn-as-value; `mycfmt` v0 preserves only the structured nodule header + code, so the pseudocode lives beside the nodule, not as interior comments — G2), to be swapped into `result.myc` as executable code once the HOF capstone (static defunctionalization; no kernel change, KC-3) lands and is pulled back down. `Declared`/deferred — documentation of the intended API, **not** silent stubs (G2/VR-5).

Guarantee tags: differential agreement is **`Empirical`** (7 trial tests, all green); the type-level contract is **`Declared`**. Status was `Draft`; now **`Resolved`** (append-only — `Draft → Resolved` per the notes status discipline).

<!-- changelog: 2026-06-23 HOF combinators now executable + self-hosted; companion pseudocode removed (append-only, VR-5) -->
**2026-06-23 — HOF combinators (`map`/`and_then`/`fold`) now executable + self-hosted (RFC-0024 / M-685/686/687; append-only, VR-5).** The function-type surface (`A -> B`) and named-fn-as-value are now implemented and lowered by static defunctionalization in `mono.rs` (no kernel change — KC-3). `lib/std/result.myc` now contains all six HOF combinators as fully executable self-hosted code (M-649 companion file `result.pending-hof.md` **removed** — the pseudocode was the honest deferral; the executable version replaces it). Differential agreement across L1-eval ≡ L0-interp ≡ AOT is **`Empirical`** (three-way `std_result` + `hof` differential tests). The type-level contract is **`Declared`** (a structural rewrite, not a theorem; VR-5 — no upgrade). **Remaining honest deferrals (unchanged, not silently upgraded):** transitive HOF (closures, dynamic fn-flow through `match` arms or data fields), partial application, multi-argument arrows `(A,B)->C`, generic-fn-as-value, and a structured `///` doc-comment convention — all remain explicit never-silent `Residual`s or future RFCs.

<!-- changelog: 2026-06-19 Draft created (M-502) -->
