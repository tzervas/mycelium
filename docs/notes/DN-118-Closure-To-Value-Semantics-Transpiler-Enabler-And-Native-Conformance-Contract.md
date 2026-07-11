# Design Note DN-118 — Closure-to-Value-Semantics Transpiler Enabler and Native-Conformance Contract

| Field | Value |
|---|---|
| **Status** | **Accepted** (2026-07-11, orchestrator-ratified on the merits, delegated authority — the scoping agent's own attempt to file this note was guard-blocked; the orchestrator authored and accepted it in the same pass per the maintainer's standing delegation for design-note ratification, mirroring the DN-115/DN-117 precedent). Ratifies **Option A** — a transpiler **closure-EMIT pass** (`crates/mycelium-transpile`) that emits the Mycelium `lambda` surface and lets `myc check`/mono's already-landed whole-program defunctionalization (RFC-0024 §4A, M-704) resolve captures, plus a **future** RFC-0018-framework `@value_closures` native-conformance contract (P2, not built by this note). **Explicitly NOT Enacted** — P1 (the transpiler emit pass) lands with this note's leaf; P2 (the contract) and P3 (borrowck-backed capture-mutation checking) are scoped, future work (house rule #3: `Enacted` requires the whole decision fully implemented and landed, which this note is not). |
| **Kind** | Design note + implementation record (leaf-scoped). P1 code lands with this note (`crates/mycelium-transpile/src/{emit,visit,gap}.rs` + `src/tests/emit.rs`); P2/P3 are scoped, not built. |
| **Decides** | (1) **Option A is ratified**: the transpiler gap is a **closure-EMIT gap**, not a defunctionalization gap — defunctionalization of env-capturing closures is *already* implemented in the Mycelium **language** (RFC-0024 §4A, M-704 `done`), so the transpiler must **emit the `lambda` surface and stop there**; it must **never** build its own defunctionalizer (that would duplicate `mono.rs` and re-hit an unrelated, already-diagnosed failure — §2). (2) **The FnMut/`&mut` safety boundary**: a closure whose body syntactically shows it mutating a captured (non-parameter) binding in place is **never auto-emitted** — `syn` carries no borrowck facts, so this is classified conservatively and FLAGGED with a suggested idiom (fold/accumulator/functional update), per the DN-109 D5/D7 ratchet. (3) **The single-parameter-only Phase-1 scope**: a multi-parameter closure is *also* flagged in Phase 1 — a **verify-first correction** of this note's own original plan (§4.3) — because `lambda(x: T, y: U) => …` parses but checks as fully curried (M-822/RFC-0024 §4A.8), and this transpiler's existing, unchanged call-site emission cannot produce the required chained application. (4) **The tag boundary**: the emitted `.myc` text is `Empirical` (mechanically produced, `myc check`-verified for the shapes this note's tests cover); the *general* "every value-safe-classified closure is semantically faithful" guarantee stays `Declared` until a P2 RFC-0018-framework certificate/checker discharges it — never upgraded past a checked basis (VR-5). (5) **Phasing**: P1 (this leaf, transpiler emit pass) → P2 (a future `@value_closures` contract under RFC-0018's grading framework, semcore-scoped) → P3 (a future borrowck-backed capture-mutation checker, closing the residual method-call/mutation false-negative classes P1's syntactic scan cannot see). (6) **Append-only supersede**: DN-34 §3's "environment-capturing closures are auto-`Impossible`" row is **STALE** as of RFC-0024 §4A/M-704 landing — this note supersedes that characterization going forward (DN-34's own text is left unchanged, house rule #3; §8 below is the pointer). |
| **Grounds in** | RFC-0024 §4A (M-704, `done` — the closure defunctionalization this note builds ON, not around); RFC-0018 (Stage-1 Static Guarantee Grading — the framework a future P2 `@value_closures` certificate would use); DN-109 §3 D5/D7 (the idiom-classification ratchet: Mechanical/Heuristic/Judgment, and the `&mut`-aliasing trap specifically); DN-34 §3 (the construct-mapping sketch whose closures row this note supersedes going forward); DN-111 (the native-equivalence taxonomy this note's classification follows); the M-1041 `ExprVisitor`/`walk_expr` shared dispatch (`crates/mycelium-transpile/src/visit.rs`, ridden not duplicated); `crates/mycelium-l1/src/mono.rs` (`ClosureSpecialization`, the whole-program `apply$A$B` dispatcher this note's Phase-0 probe exercised directly); `crates/mycelium-l1/src/tests/facility_stage1_hygiene.rs` (the `apply$Fn` synthetic-`Env` note that this note's §2 traces to its real, narrower cause). Read against `dev @ 7ecfb4d7`. |
| **Guarantee posture** | `Empirical` for every claim checked against the codebase/real toolchain in this note (§1's verify-first probes, §5's live-oracle tests, the `checked_fraction` movement in §6) — measured, not asserted. `Declared` for the P2/P3 scoping (unbuilt, future work) and for the general "closure translation is semantically faithful" claim beyond the syntactic scan's proven boundary. No `Proven` claim anywhere (no checked theorem). |

---

## §1 Verify-first — premises confirmed against the codebase (mitigation #14)

Before writing any code, every premise of the task brief was checked against `dev @ 7ecfb4d7`
(this leaf's actual working base), not assumed:

1. **Defunctionalization is already implemented in the language.** `crates/mycelium-l1/src/mono.rs`
   has a full `ClosureSpecialization` mechanism (RFC-0024 §4A, M-704 `done`): every escaping closure
   lowers to a per-arrow `Fn$A$B` tag-sum data type (one constructor per distinct closure, fields =
   captured free variables in first-occurrence order) plus a generated `apply$A$B(clo, x)` dispatcher,
   emitted **once per arrow, whole-program**, at `MonoSelections::finish()`. Confirmed by reading
   `mono.rs`'s `closures`/`closure_specs` fields and `emit_closures` (lines ~437–1067) and the RFC-0024
   changelog's M-704 entry (three-way differential agreement, `crates/mycelium-l1/tests/closures.rs`).
2. **The surface `lambda(p: T) => body` is active.** `crates/mycelium-l1/src/token.rs` lexes
   `Tok::Lambda`; `crates/mycelium-l1/src/parse.rs::parse_lambda_expr` parses
   `lambda_expr ::= 'lambda' '(' params? ')' '=>' expr` (grammar `docs/spec/grammar/mycelium.ebnf`
   line ~206). Captures are **not** declared explicitly — they are the free `let`/param binders the
   checker's own free-variable walk discovers (mono's job, not the parser's).
3. **The `apply$Fn` failure is real but NARROWER than a general limitation.**
   `crates/mycelium-l1/src/tests/facility_stage1_hygiene.rs`'s fixture-4 doc (lines 288–309) names the
   exact prior finding: `elaborate_lower_rule`'s **ad-hoc single-function synthetic `Env`** — used
   ONLY to elaborate a `lower`-rule's RHS (DN-54/M-812's generative-lowering sugar) — does not register
   mono's generated `apply$Fn$…` dispatcher function, so a `lower` rule whose RHS is an
   immediately-applied `lambda` (an IIFE) fails with `unknown function/constructor/prim
   apply$Fn$Binary8$Binary8`. This is confirmed, in the cited fixture's own words, to be a **standing
   gap in how a `lower` rule's RHS is elaborated at all** — orthogonal to closures generally and
   independent of the value-parametric hygiene work that fixture belongs to.
4. **A whole-program `nodule` with a closure IS `myc check`-clean and runs correctly** — the premise
   this note's Option A depends on. Built `target/debug/myc-check` and `target/debug/myc` fresh from
   this leaf's worktree and ran, end to end:

   ```mycelium
   // nodule: closure_demo
   nodule closure_demo;

   fn make_masker(n: Binary{16}) => Binary{16} =
       let f = lambda(x: Binary{16}) => and(x, n) in
       f(n);

   fn main() => Binary{16} =
       make_masker(0b1111_0000_1111_0000);
   ```

   `myc check` → `myc: 1 nodule(s) checked clean` (exit 0). `myc run` → evaluates to
   `Binary{16}` bits `1111000011110000` — the correct `and` of the literal with itself — and
   reports ``myc: ran `main` in closure_demo.myc`` (exit 0). **Both the checker and the evaluator
   resolve the generated `apply$…` dispatcher whole-program with zero issue** — confirming the
   `apply$Fn` failure in point 3 is specific to the synthetic `Env` mechanism inside
   `elaborate_lower_rule`, not to `myc check`/`myc run`'s real, whole-program compilation path (the
   one the transpiler's output goes through).
5. **The transpiler's actual gap, prior to this leaf, was a missing `Expr::Closure` emit arm** —
   not a missing defunctionalizer. `crates/mycelium-transpile/src/emit.rs`'s `EmitVisitor` (the
   M-1041 `ExprVisitor` implementation backing `emit_expr_inner`) had no `visit_closure` override, so
   `Expr::Closure` fell to the generic `fallback`, gapping every closure as
   `Category::Other`/`"unsupported expression form"`. Confirmed by transpiling a representative Rust
   closure (`fn make_adder(n: u16) -> u16 { let f = |x: u16| x & n; f(n) }`) through
   `mycelium-transpile` at this leaf's pre-change base: `0 emitted, 1 gap(s) recorded, 0.0%
   expressible`.

**Conclusion (grounding this note's Option A, §3): the residual gap is exactly and only "the
transpiler does not emit `lambda` for `Expr::Closure`" — not "Mycelium cannot express env-capturing
closures" (it can, and does, per points 1–4) and not "the transpiler needs its own defunctionalizer"
(building one would duplicate `mono.rs` and, per point 3, risks re-deriving the exact
synthetic-`Env` failure a *different*, unrelated mechanism already hit).**

## §2 The stale characterization this note corrects (DN-34 §3, append-only)

`docs/notes/DN-34-Rust-to-Mycelium-Transpiler-Strategy.md` §3's construct-mapping sketch (line 65)
reads: *"only *named* fns-as-value via RFC-0024 (Proposed / pending ratification, not Accepted);
**environment-capturing closures are auto-`Impossible`** and must be flagged (research-18 §3; DN-14 —
transitive HOF stays `Residual`)."* DN-109 §3 D5 (line 101) repeats the same classification:
*"env-capturing closures | **Judgment/Impossible** | Closures are auto-`Impossible`, must be flagged
(DN-34 §3; research-18 §3)."*

Both rows predate RFC-0024 §4A/M-704 (2026-06-29, `done`) — at the time they were written, closures
genuinely had no lowering path (`Residual`, per the RFC-0024 changelog's own "before" state). **They
are stale now.** Per house rule #3 (append-only decisions), this note does **not** edit DN-34 or
DN-109's text — it supersedes the *characterization* going forward, recorded here and cross-referenced
(§8), exactly as `Enacted`/`Superseded` transitions are meant to work: forward-only, cited, never a
silent rewrite.

The corrected classification (per this note, DN-109's own §3.2 ratchet vocabulary): an env-capturing
closure whose every capture is provably value-safe (read-only / moved / `Copy`) is now **Mechanical**
(auto-fire-eligible, §3.3-B's v0 bucket) for the *language* side (closures type-check and lower via
RFC-0024 §4A) — but, per §5 below, the *transpiler's own classification step* stays a syntactic,
conservative gate: value-safety is Mechanical only when it can be positively shown from `syn` syntax
alone; anything it cannot prove remains **Judgment** (flagged), never silently upgraded.

## §3 Option A — the closure-EMIT pass (ratified; what P1 builds)

**Decision:** `crates/mycelium-transpile` gets a `visit_closure` arm (`EmitVisitor::visit_closure`,
riding the M-1041 `ExprVisitor`/`walk_expr` shared dispatch — `crates/mycelium-transpile/src/visit.rs`)
that:

1. Rejects `async`/`const`/`static` closures outright (no Mycelium `lambda` correspondence —
   `lambda_expr` is plain and synchronous; ADR-003 has no reference type for a `static` closure's
   promoted captures to alias).
2. Requires every closure parameter to be an **explicitly-typed** identifier (`Pat::Type` wrapping a
   plain `Pat::Ident`) — Mycelium's `lambda_expr`'s `params` production is exactly `Ident ':'
   type_ref`, and this transpiler has no type-inference pass to recover an omitted Rust closure-param
   type (most Rust closures infer their param types from call-site usage; VR-5 — absence, never a
   guess).
3. Rejects a zero-parameter closure (no v0 `lambda` form — the grammar's own note) and, per the §4.3
   verify-first correction, a **multi**-parameter closure too (see §4.3).
4. Runs the DN-109 D5/D7 safety gate (§5) BEFORE ever emitting text.
5. Emits `lambda(name: Type) => body`, with captured names left as **ordinary in-scope references**
   in the body — this pass does **not** synthesize an env record or a capture list; mono's
   whole-program defunctionalization (§1 point 1) resolves the capture set itself, at `finish()`,
   after every nodule in the check is loaded.

**What this explicitly is NOT:** a transpiler-side defunctionalizer. No tag-sum type, no `apply`
dispatcher, no capture-set computation is built or emitted by `crates/mycelium-transpile` — that
machinery already exists, whole-program, in `mono.rs` (§1 point 1), and re-building even a partial
version of it in the transpiler would (a) duplicate landed, tested language machinery and (b) risk
re-deriving the exact ad-hoc-`Env` failure mode `elaborate_lower_rule` already hit for an unrelated
reason (§1 point 3) — the two failure modes look superficially similar (both surface as an unresolved
`apply$…` name) but have different root causes, and building a transpiler-side partial `Env` was
exactly the path that would hit the *transpiler's own* version of the same trap.

## §4 The transpiler/language split, stated precisely

| Concern | Owner | Status |
|---|---|---|
| Env-capturing closure type-checks and lowers to `Fn$A$B` + `apply$A$B` | **Language** (`mycelium-l1`, RFC-0024 §4A) | `done` (M-704), verified `Empirical` per §1 point 4 |
| `lambda(p: T) => e` surface syntax | **Language** (parser/lexer) | Active (RFC-0037 D5) |
| Rust `Expr::Closure` → Mycelium `lambda` surface text | **Transpiler** (`mycelium-transpile`) | **This note, P1** |
| Capture-mutation (`FnMut`/`&mut`) safety classification at translation time | **Transpiler** (syntactic, conservative) | **This note, P1** — a translation-time judgment call, not a language feature |
| A checked, borrowck-backed capture-mutation *proof* (closing the residual method-call/false-negative classes) | **Future** — needs real ownership facts (`syn` has none) | **P3**, not this note |
| A native-conformance *certificate* that a translated closure is semantically faithful | **Future** — RFC-0018 framework | **P2**, not this note |

### §4.3 Verify-first correction: multi-parameter closures are ALSO Phase-1 gaps

This note's design phase originally assumed a multi-parameter `lambda(x: T, y: U) => …` could be
emitted directly (the grammar's `params?` production accepts any arity, and `docs/spec/grammar/
mycelium.ebnf`'s own comment says the checker "desugars" a multi-param lambda to the curried form).
**Verified empirically against the real oracle and found WRONG as a call-site assumption**
(mitigation #14 — surfacing the disconfirming finding rather than shipping the original plan): a
directly multi-param `lambda(x: T, y: U) => …` DOES parse and DOES check as a value — but the checker treats
that value as **fully curried** (M-822/RFC-0024 §4A.8: each application takes exactly one argument),
so an ordinary Rust-derived multi-arg call site `f(a, b)` — this transpiler's existing, **unchanged**
`Expr::Call` emission (`visit_call`, out of this leaf's scope) — fails `myc check`:

```text
check-error: check error in `combine`: `f` has function type and takes exactly 1 argument in
stage-1; got 2 (partial application / multi-arg HOF is deferred — RFC-0024 §5, never a silent
coercion)
```

A faithful multi-parameter closure needs BOTH a curried **declaration**
(`lambda(x: T) => lambda(y: U) => body`) AND a chained **call-site rewrite** (`f(a)(b)`) — and
`visit_call`'s call-target match only accepts a bare/qualified `Expr::Path` today, not a nested
`Expr::Call`, so it cannot even emit a chained call yet. That is a distinct, larger,
call-site-aware unit of work (touching `Expr::Call`, not `Expr::Closure`), so — rather than emit a
plausible-but-`myc check`-failing form — **P1 gaps a multi-parameter closure explicitly**
(`Category::Closure`, a cited, honest reason), leaving curried multi-arg closure support to a later
phase. Only the **single-parameter** form is Mechanical/auto-emitted in P1.

## §5 The FnMut/`&mut` safety boundary (DN-109 D7) — the load-bearing gate

DN-109 §3 D7 names the core hazard directly: *"`&mut T` in-place mutation to value-semantics
functional update | **Judgment** (Heuristic *only* with borrowck facts) | If two references alias,
functional update is **observably different**. `syn` cannot prove non-aliasing … The core VR-5 trap."*
This applies to closure captures exactly: mono's defunctionalization captures a closure's environment
as a **value snapshot at construction** (a tag-sum struct field, set once — §1 point 1). An
`FnMut`-style Rust closure that mutates a capture **across calls** would, if silently auto-emitted,
translate to a Mycelium program that reads the SAME (stale, construction-time) value every call — a
silent semantic divergence from the source program's behavior, not merely a `myc check`-time
rejection. This is exactly the never-silent failure mode G2/VR-5 exist to prevent.

**The gate (implemented in `visit_closure`, before any emission):** a small, deliberately conservative
syntactic scanner (`scan_block_for_capture_mutation`/`scan_expr_for_capture_mutation`,
`crates/mycelium-transpile/src/emit.rs`) walks the closure body, tracking which names the closure
itself binds (its own parameters, plus every `let`-bound name in scope so far — so a **purely
internal** accumulator that never escapes the closure is never mistaken for a capture). Any
syntactically-detectable sign that a name OUTSIDE that local set is mutated — a direct or compound
assignment (`total = …` / `total += …`), an explicit `&mut` (`&mut total`), or use as a method-call
**receiver at all** (Rust's `&self` vs `&mut self` dispatch is unknowable from `syn` syntax alone, so
this is the conservative direction: over-flag rather than risk missing a `.push()`-style in-place
mutation) — is treated as "cannot prove value-safe" and the closure is FLAGGED (`Category::Closure`,
never auto-emitted), with a message naming the captured binding and suggesting the idiom to use
instead (a fold/accumulator parameter, or a functional update returning the new value).

**Honesty about the scan's own limits (never overclaimed):** this is a `syn`-only heuristic, not a
borrowck-backed proof. It cannot, for example, distinguish `captured.len()` (read-only) from
`captured.push(x)` (mutating) by method name alone — both are conservatively flagged as "cannot prove
value-safe" since both are method calls on a captured receiver. This is the correct DIRECTION of
error (over-flag, never silently emit an unsafe case) but means some genuinely-safe closures are
flagged too; closing that residual gap for real needs actual ownership facts (rustc/rust-analyzer
`mir_borrowck`, per DN-34 §3's own division of labor for the general Rust→affine mapping) — deferred
to **P3**, a future, distinct unit of work.

## §6 Guarantee tags — never upgraded past a checked basis (VR-5)

- **The emitted `.myc` text**: `Empirical`. Verified, not asserted: the move/`Copy`-capture fixture
  (`closure_move_copy_capture_emits_lambda`) is run through the REAL `myc-check` oracle
  (`closure_move_copy_capture_checks_clean`, a live-oracle test mirroring the crate's existing
  `binop_operand_gated_forms_check_clean` pattern) and confirmed `Clean` — the `apply$Fn` gap this
  note traces to its real cause (§1 point 3) is CLOSED for this shape, whole-nodule, exactly as §1
  point 4's standalone probe predicted.
- **`checked_fraction` movement** (measured, not modeled): over a 2-file closure-bearing micro-corpus
  (a move/`Copy`-capture closure calling itself, plus the representative fixture from this note's
  Phase-0 probe), `--vet` against the real oracle reports **before this leaf: `checked_fraction`
  0.0% (0/2)**; **after: `checked_fraction` 100.0% (2/2)**. This is a small-N, illustrative measurement
  (this note's own micro-corpus, not the full `checked_fraction`/M-1000 vet-loop corpus) — `Empirical`
  for exactly what it measures, not a claim about the whole workspace's coverage.
- **The FnMut-flag classification**: `Empirical` that the scan fires on the tested shapes
  (`closure_fnmut_compound_assign_capture_gapped`, `closure_fnmut_explicit_mut_ref_capture_gapped`,
  `closure_captured_method_receiver_gapped`) and does NOT false-positive on a purely local mutation
  (`closure_purely_local_mutation_not_misclassified_as_closure_gap`) — all four are regression tests,
  not a proof the scan never misses a real hazard elsewhere (§5's honesty note).
- **"Every value-safe-classified closure is a semantically faithful translation"**: `Declared` — a
  design claim this note's syntactic gate is built to *approximate soundly in the flag direction*
  (never silently emit an unsafe case), not a claim discharged by any checked theorem. Upgrading this
  to `Proven`/checked needs P2's contract (a checker-enforced `@value_closures` certificate) or P3's
  borrowck integration — neither exists yet; this tag is not upgraded past that absent basis (VR-5).

## §7 Phasing

- **P1 (this note's leaf, landed here).** The transpiler closure-EMIT pass: the `visit_closure` arm,
  the DN-109 D5/D7 safety gate, `Category::Closure`, and the fixture + live-oracle test corpus
  (`crates/mycelium-transpile/src/tests/emit.rs`). Scope: `crates/mycelium-transpile/` only — no
  `mycelium-l1`/semcore touch (parallel-safe with the semcore lane by construction).
- **P2 (future, semcore-scoped).** An RFC-0018-framework `@value_closures` native-conformance
  contract/certificate — a *checked* grade (not the P1 heuristic scan) that a given closure
  translation is value-safe, likely riding RFC-0018's stage-1 graded-judgment machinery
  (`crate::grade`, per RFC-0018's own `Enacted` stage-1a). Not designed in detail by this note; the
  RFC-0018 framework it would ride is cited (§ Grounds in) but this note does not add a
  `@value_closures` construct anywhere.
- **P3 (future).** Borrowck-backed capture-mutation checking — closing the §5 residual (the
  method-call-receiver over-flag direction, and any capture-mutation shape `syn` syntax genuinely
  cannot see) via real ownership facts (rustc/rust-analyzer `mir_borrowck`), per DN-34 §3's own
  division of labor for the Rust→affine mapping generally.

## §8 Append-only supersede pointer (DN-34 §3, DN-109 §3 D5)

This note **supersedes the characterization**, not the text, of:

- `docs/notes/DN-34-Rust-to-Mycelium-Transpiler-Strategy.md` §3, the `fn` / closures row (line 65):
  *"environment-capturing closures are auto-`Impossible` and must be flagged."* **STALE as of
  RFC-0024 §4A/M-704 (2026-06-29).** The transpiler gap this row anticipated is a **closure-EMIT gap**
  (a missing `Expr::Closure` arm in `crates/mycelium-transpile/src/emit.rs`), not a language
  impossibility — closed for the single-parameter, value-safe case by this note's P1 (§3, §6).
  DN-34's own text is left unchanged (house rule #3; it is a `Draft (advisory)` strategy-capture note,
  never itself `Accepted`) — this note is the forward pointer a future reader of DN-34 §3 should
  follow.
- `docs/notes/DN-109-Idiom-Optimal-Transpilation-And-Structural-Remapping.md` §3 D5 (line 101), same
  claim, same stale basis, same forward pointer.

Neither DN-34 nor DN-109's own `Accepted`/`Draft` status is changed by this note — this is a
**correction-by-append**, exactly the append-only discipline house rule #3 requires: the old rows
stand as a historical record of what was true before RFC-0024 §4A/M-704 landed; this note is the
citable, dated correction for what has been true since.

## §9 Definition of Done

- **P1 (this note's leaf):** `visit_closure` lands in `crates/mycelium-transpile/src/emit.rs`
  (riding the M-1041 shared `ExprVisitor`); `Category::Closure` added to `gap.rs`; the DN-109 D5/D7
  safety scan implemented and tested (both the closed-gap and the flagged-FnMut directions, plus a
  negative control against false-positiving on a purely local mutation); `cargo fmt` /
  `cargo clippy -D warnings` / `cargo test -p mycelium-transpile` green; a live-oracle test proves the
  `apply$Fn` gap closed end-to-end for the move/`Copy`-capture shape; `checked_fraction` movement
  measured and reported (§6); this note filed at `Accepted` (not `Enacted` — P1 alone does not
  discharge the whole DN-118 decision); DN-34 §3 / DN-109 §3 D5 supersede pointers recorded (§8,
  append-only). All satisfied by this leaf.
- **P2/P3:** out of this note's Definition of Done — each is its own future-scoped unit of work (§7),
  not started here.

---

## Ratification (maintainer-delegated, orchestrator-selected on the merits, 2026-07-11)

Per the task brief: *"The scoping agent's DN-118 write was guard-blocked. Author DN-118 as Status:
Accepted (2026-07-11, orchestrator-ratified on the merits, delegated authority)"* — this note is
authored directly at `Accepted`, per the maintainer's standing delegation of design-note ratification
authority to the orchestrating agent for a scoped, verify-first design (mirroring the DN-115/DN-117
same-leaf ratification precedent, both cited in their own headers as "delegated ratification"). The
ratification covers exactly: Option A (§3) as the transpiler-side design; the FnMut/`&mut` safety
boundary (§5) as the load-bearing correctness gate; the tag boundary (§6, nothing upgraded past a
checked basis); the P1/P2/P3 phasing (§7); and the DN-34 §3 / DN-109 §3 D5 append-only supersede
pointers (§8). It does **not** ratify a P2 `@value_closures` construct into existence (none is added
by this note) and does **not** move this note, DN-34, or DN-109 to `Enacted`.

---

## Changelog (this note)

- **2026-07-11 — Accepted (authored + ratified, same leaf).** DN-118 filed directly at `Accepted`
  per the maintainer's delegated ratification authority (the scoping agent's own attempt to file this
  note was guard-blocked). Records Option A (the closure-EMIT pass, NOT a transpiler defunctionalizer),
  the transpiler/language split (§4), the verify-first correction narrowing P1 to single-parameter
  closures only (§4.3), the FnMut/`&mut` safety boundary (§5), the honest tag boundary (§6: emission
  `Empirical`, general value-safety `Declared`), the P1/P2/P3 phasing (§7), and the DN-34 §3 / DN-109
  §3 D5 append-only supersede pointers (§8). P1 lands in the same leaf
  (`crates/mycelium-transpile/src/{emit,visit,gap}.rs` + `src/tests/emit.rs`): the `apply$Fn` gap
  closed end-to-end for the move/`Copy`-capture, single-parameter shape (live-oracle-verified), the
  FnMut/`&mut`-capture case flagged (never auto-emitted), `checked_fraction` moved 0.0% → 100.0% over
  a 2-file micro-corpus. `cargo fmt` / `cargo clippy -D warnings` / `cargo test -p mycelium-transpile`
  green. Not `Enacted` (house rule #3 — P2/P3 remain future work; the integrating parent's call).
  Branch `claude/leaf/dn118-p1-closure-emit`, held for review (not merged).
