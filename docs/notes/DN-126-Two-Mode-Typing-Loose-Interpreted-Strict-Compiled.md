# Design Note DN-126 — Two-Mode Typing: Loose/Duck-Typed on the Interpreted Path, Strict-Gates-Compile, and Type-Hint-Driven Mechanical Strictification

| Field | Value |
|---|---|
| **Note** | DN-126 |
| **Status** | **Accepted** (2026-07-12, ratified under explicit maintainer delegation — "ratify based on objective reasoning and the project's needs/intents, keep to core principles, report results"; mirrors the DN-115/117/118/122/124/123/125 precedent). Was **Draft** (2026-07-12, same day). **Accepted, not Enacted** (house rule #3) — it **builds nothing** yet; every mechanism below stays `Declared`/unbuilt until the FLAGGED build issue (M-1077) lands and is differential-witnessed. This note still does not edit `crates/**`; the integration close-out applies `Doc-Index.md`/`CHANGELOG.md`/`issues.yaml` per this ratification (recorded here, append-only). |
| **Ratification basis (recorded verbatim, 2026-07-12)** | Loose mode = the existing bidirectional checker run in a **non-refusing posture** over the **unchanged repr-dynamic evaluator**; strict mode = the same checker with demotion off — **strict typing gates compile unconditionally, as today**. Mechanical strictification writes down **only a principal inferred type** (sound-by-conservatism — a non-principal/undecidable hole is *definitionally* surfaced via the DN-04 residual channel, never guessed, per the §4.1 principality invariant). The **runnable-floor boundary** (§3.3) stays HARD in loose mode: name/arity/parse/FFI refusals never demote — only *type-level* refusals do. **Zero kernel growth** (KC-3): the design reuses the existing checker, `Cx::infer`, the repr-dynamic evaluator, the `Declared` floor, and the DN-04 surfacing channel — no new type theory, no new kernel primitive. **Type-strictness is a genuinely THIRD, orthogonal axis** — distinct from ADR-032/RFC-0034's certification-depth (`fast`/`certified`) and from RFC-0018's guarantee-grade lattice (`Exact⊐Proven⊐Empirical⊐Declared`); neither existing axis already provides it (§2's three-axis verdict, ratified). Alt A (demotion-switch, single-checker) is ratified over Alt B (inference-only pre-pass, rejected on DRY) and Alt C (a new gradual-type-system with blame, rejected on KC-3/YAGNI — retained as the documented growth path only). Carries forward to Python's gradual typing (PEP 484) with the identical mechanism (§7). Gate PASS (9/9) — ratified on the merits under maintainer delegation; this note's own reasoning (§1–§9) is not re-litigated, only executed and recorded (VR-5). |
| **Decides (proposes, for ratification)** | (1) the **three-axis verdict** — type-resolution *strictness* is a **new, orthogonal axis**, distinct from certification-depth (ADR-032/RFC-0034 `fast`/`certified`) and guarantee-grade (RFC-0018 `Exact⊐Proven⊐Empirical⊐Declared`); neither existing axis already provides it (§2); (2) the **two-mode model** — loose = the existing bidirectional checker in a *non-refusing posture* over the *unchanged repr-dynamic evaluator*; strict = the same checker with demotion off (§3); (3) the **runnable-floor boundary** — name/arity refusals stay HARD in loose mode; only *type-level* refusals demote to DN-04 flags (§3.3); (4) the **mechanical-strictification algorithm** + its **principality invariant** and the precise **truly-ambiguous residual** it must surface never-silently (§4); (5) the ranked alternatives with an objective table, recommending **Alt A (demotion-switch, single-checker)** (§5); (6) the **honesty boundary** — a mode-level never-silent tag, guarantee-grades orthogonal, value semantics untouched (§6). |
| **Feeds** | M-1077 (this note is its design gate — the DoD's "a DN scoping the loose-mode mechanism … is ratified before implementation"); the Python→Rust→Mycelium porting path (§7); the lexicon-alias DX layer (related future work, §8, out of scope here). |
| **Depends on / grounds on** | **ADR-032** (Enacted — tunable `fast`/`certified`, never-silent mode tag; the *certification-depth* axis this note holds orthogonal); **RFC-0034** (the mode mechanism — `CertMode`); **RFC-0018** (Enacted stage-1a — the guarantee-grade lattice + the graded pass `crate::grade`; the *provenance* axis this note holds orthogonal); **RFC-0013 / DN-04** (structured diagnostics — the never-silent surfacing channel for the ambiguous residual); **RFC-0007 §4.4/§4.6** (the v0 bidirectional checker + the fuel-guarded evaluator); **ADR-003** (content-addressed value identity — the value-semantics invariant loose mode must not disturb); KC-3 (small kernel); VR-5/G2 (downgrade-don't-overclaim; never-silent); KISS/YAGNI. |
| **Date** | July 12, 2026 |
| **Author** | design-reasoner (Opus). Owns only this note. |
| **Task** | M-1077 — loose/gradual typing DX mode + Python-transpile enabler (P3 backlog; maintainer top-priority requirement 2026-07-12). |

> **Grounding + honesty (house rule #4 / VR-5 / G2).** This is a **design recommendation**, not a
> decision (house rule #3). **No sycophancy:** §2 *corrects the task's own framing* — the requirement
> named "RFC-0018's fast/certified grading" as the possible existing scaffold, but `fast`/`certified`
> is ADR-032/RFC-0034 (certification depth) and grading is RFC-0018 (guarantee provenance); *neither*
> is the type-strictness axis, and I say so on the merits. §5 ranks three real alternatives and
> rejects a fourth (overloading an existing axis) explicitly. §4/§9 argue *against* any "just infer
> and compile it" shortcut: an unsound loose run has **no principal type**, so it is *definitionally*
> in the surfaced residual and must never be mechanically promoted.

---

## §0 The question, in one line

**How does Mycelium let a developer run a not-yet-fully-typed program on the fast interpreted path
(duck-typed, infer-and-hint, do-not-gate-iteration) while keeping strict typing an unconditional gate
for compilation — and make the loose→strict promotion *mostly mechanical*, surfacing only the truly
ambiguous cases — without a new type theory, a new kernel primitive, or any relaxation of the
certified path or the value-semantics/honesty guarantees?**

## §1 The requirement (M-1077), restated

- **Duck typing in INTERPRETED mode.** The fast interpreted path (`crates/mycelium-l1/src/eval.rs`)
  allows loose/gradual typing: infer plus hint types, do **not** gate iteration on full type
  resolution.
- **Strict typing as a HARD REQUIREMENT for COMPILATION.** The compiled/AOT path
  (`crates/mycelium-mlir`) requires full strict typing; strict typing gates compile.
- **Type-hint-driven mechanical strictification.** Given interpreted-mode type hints, promoting to
  strict (compile-ready) typing should be mostly **mechanical** (automated), surfacing **only the few
  truly AMBIGUOUS/undecidable cases** for human resolution (never-silent, via DN-04 diagnostics).
- **Inherently enables Python transpilation** (Python's gradual typing maps onto the interpreted-loose
  mode). Frame the mechanism **source-language-general**.

## §2 Verify-first: is this answered-by-design, or a genuinely new mode? (the verdict)

**Verdict: a genuinely NEW axis — but one that reuses every existing mechanism and adds no new
kernel/soundness surface.** Grounded in the code and corpus at `dev@fa53dc46`:

### §2.1 There are already TWO orthogonal axes — and neither is type-strictness

1. **Certification-depth axis (ADR-032 / RFC-0034): `fast` ⇄ `certified`.** This governs *how much
   assurance machinery runs* — whether swaps emit/check certificates, whether `Empirical`/`Proven`
   trials/proofs run. `CertMode::Fast` "sits at the structural `Exact`/`Declared` tags and never claims
   an `Empirical`/`Proven` it did not earn" (`crates/mycelium-core/src/cert_mode.rs:13`). **Crucially:
   `fast` mode still fully type-checks.** Both `fast` and `certified` run the *same* checker
   (`checkty`); the mode changes certification depth, not type resolution. So `fast` is **not** loose
   typing. (This is the task-framing correction: the requirement floated "fast/certified grading" as
   the candidate scaffold; on inspection it is the wrong axis.)

2. **Guarantee-grade axis (RFC-0018, Enacted stage-1a): `Exact ⊐ Proven ⊐ Empirical ⊐ Declared`.**
   This is a graded coeffect system tracking *data provenance* — "what evidence backs this result"
   (RFC-0018 §4.5, Design A). It is enforced by a post-pass (`crate::grade`,
   `crates/mycelium-l1/src/grade.rs`) and is **erased at elaboration** (RFC-0018 changelog 2026-06-22:
   "a grade, like a type, is statically checked and **erased** — no L0 node"). A value's grade is
   independent of whether its *type* was inferred or fully resolved. So grading is **not** loose
   typing either — it is orthogonal provenance.

Neither axis controls **type-resolution strictness** — whether *every expression must resolve to a
concrete `Ty`* before the program may run. Today that is **unconditionally required**: see §2.2.

### §2.2 Today the checker is a HARD gate before the evaluator (strict-only)

The evaluator takes a **checked** environment: `Evaluator::new(env: &Env)`
(`crates/mycelium-l1/src/eval.rs:584`), where `Env` is the output of `check_nodule`. Every execution
path is `parse → check_nodule → (monomorphize) → Evaluator` (e.g.
`crates/mycelium-l1/tests/compiler_stage1.rs:57`, `compiler_stage3_ast.rs:37`). The checker **refuses**
— "generics, `spore`, value-level integers without context, and a `wild` block outside the audited FFI
floor are *refused with a reason*, never guessed at" (`checkty.rs:1-8`); a refusal is a flat
`CheckError { site, message }` (`checkty.rs:181`). There is **no** posture in which a
type-mismatch/unresolved-type program runs. So "loose typing on the fast path" is genuinely absent
today — it is a new capability, not a re-labelling.

### §2.3 …but the scaffold to build it on is entirely present (why this is small)

Three landed facts make the new axis cheap and soundness-neutral:

- **The evaluator is repr-dynamic, not type-directed.** It is a big-step machine over
  `repr + payload + Meta` values (`eval.rs:1-30`; the L0 small-step semantics dispatch on `Const(v)`
  reprs, `crates/mycelium-interp/src/lib.rs`). It never consults a static `Ty` to *run* — it consults
  the `Env` only for **name/arity** resolution (function table, `DataInfo` constructor arities). So
  "duck typing at runtime" is **already the runtime's nature**; only the checker's *gate* stands in the
  way. And the runtime is never-silent by construction: "states the typechecker proves unreachable
  still fail as explicit `L1Error::Stuck` errors, never panics or defaults" (`eval.rs:1-30`) — the
  loose path already has its runtime backstop.
- **A bidirectional inference engine already exists.** `checkty` runs bidirectional check/infer
  (`Cx::check` plus `Cx::infer`, `checkty.rs:5111`; "bidirectional" appears throughout, e.g.
  `checkty.rs:3229`). Loose-mode "infer and hint" reuses `Cx::infer` verbatim; it does not add
  inference.
- **The honesty home for an inferred type already exists.** `CertMode::Fast`'s structural
  `Exact`/`Declared` floor (`cert_mode.rs:30`) is exactly where an *inferred-but-unchecked* type sits:
  `Declared`. And the never-silent surfacing channel exists — RFC-0013/DN-04 structured diagnostics
  (`crates/mycelium-lsp/src/diagnostics`), whose governing rule is "diagnostics are additive
  presentation over explicit errors — never a substitute" (DN-04 §1).

**Conclusion (the verdict, for ratification).** Type-strictness is a **third, orthogonal axis**. It is
**not** answered-by-design by RFC-0018 or ADR-032. But building it is a **DX-surfacing plus
gate-posture** design — a driver-level posture flag over the existing checker, the existing infer
engine, the existing repr-dynamic evaluator, the existing `Declared` floor, and the existing DN-04
surface — **not** a new type theory and **not** a new kernel primitive (contrast Alt C, §5). No
soundness surface is added to the certified/compile path, because that path is left exactly as it is
today.

## §3 The two-mode model

### §3.1 Loose (interpreted / `fast`) mode — "check-lite, run anyway"

Run the bidirectional checker in a **non-refusing posture**: it still resolves names, still infers
types where it can, but a **type-level** failure is **demoted** from `CheckError` (a hard refusal) to a
**hint diagnostic** (a DN-04/RFC-0013 warning), and evaluation proceeds over the unchanged
repr-dynamic evaluator. Precisely, three tiers of type knowledge per binding/expression:

- **Inferred** — a type the engine synthesizes (`Cx::infer` returns a principal `Ty`). Carried as a
  `Declared` hint (it was inferred, not checked-against-a-demand). Drives tooling/diagnostics.
- **Hinted** — a user-written annotation. In loose mode it is a *hint*, not a *gate*: a violated
  annotation is **flagged** (never-silent), not refused. (In strict mode it is a demand — VR-5's
  weaken-only rule, unchanged.)
- **Unchecked (a type hole `⊤?`)** — a construct the checker cannot resolve to a type (a value-level
  integer without context, an unresolved generic, a genuinely duck-typed value used across
  incompatible shapes). Left as a hole, run on runtime repr, **flagged** with the reason it could not
  be resolved. The runtime backstop is the existing `L1Error::Stuck` if the hole manifests badly.

The evaluator is **byte-identical** to strict mode (same trusted prim/swap engines, NFR-7): loose mode
changes the *checker's gate*, never the *runtime*. Two execution paths still never mean two semantics.

### §3.2 Strict (compiled / `certified`-eligible) mode — the current checker, unchanged

Every expression resolves to a concrete `Ty`; totality is checked; grades are checked (RFC-0018);
`matured` gates. **The bar to compile = the loose-mode soft-flag set is empty.** The AOT path
(`mycelium-mlir`) and monomorphization (`mono`, which requires resolved types) only ever accept a
fully strict-checked program. **Strict typing gates compile — unconditionally, exactly as today.**

**The load-bearing structural relationship: strict = loose with the demotion switch off.** Strict is
*not a different checker*; it is the same checker with the type-level-refusal demotion disabled. This
is the KISS/YAGNI-optimal design and it is what makes mechanical strictification nearly free (§4): the
strict checker *already computes* the exact set of demotions that fired in loose mode.

### §3.3 The runnable-floor boundary (a first-class part of the model, from §9.1)

Not every refusal may demote. The evaluator genuinely cannot run some programs, and pretending
otherwise would be a black box (G2). So loose mode partitions the checker's refusals:

- **HARD even in loose mode (still a refusal — the "runnable floor"):** an unresolved *name* (a
  function/constructor/type-name that does not exist — the evaluator would be `Stuck`); a
  constructor-arity mismatch; a parse error; a `wild`/FFI-gate violation. These are *"the program is
  not runnable"* errors, not *"the program is not typed"* errors.
- **SOFT in loose mode (demote to a DN-04 flag, run anyway):** a type mismatch; a missing annotation
  where strict needs one; an unresolved *type* (a generic not instantiated, a value-level integer
  without context); an unmet guarantee-grade demand (the RFC-0018 pass runs advisory in loose mode).

This partition is the precise definition of "duck typing interpreted": names and arities are real
(the program can run), types are advisory (iteration is not gated on them).

## §4 The mechanical-strictification algorithm

**Input:** a loose-mode program that ran, its inferred-type hint table (`Cx::infer` results), and any
user annotations. **Goal:** produce a strict-checkable program, or the minimal human residual.

```text
strictify(program):
  1. Run the checker in STRICT posture. Collect the soft-flag set F
     (exactly the demotions that loose mode would have turned into DN-04 hints).
     (Free, because strict = loose-with-demotion-off — the checker already computes F.)

  2. For each flag f in F, attempt MECHANICAL resolution:
     (a) Inferred-type materialization — if Cx::infer synthesized a UNIQUE (principal)
         Ty for the hole, WRITE THAT TYPE DOWN as an annotation. (Deterministic: the
         inference already ran; strictification only records what infer found.)
     (b) Grade default — an un-annotated binding takes RFC-0018 stage-1a's existing
         modular/bottom default (param demands Declared, return advertises Declared).
         Already mechanical; no human needed.
     (c) Use-site pinning — a value-level integer / un-instantiated generic whose type
         is FORCED to a unique value by a use site adopts it (the strict checker's own
         defaulting, applied where context now exists).

  3. RESIDUAL = the flags where step 2 is NON-UNIQUE or UNDECIDABLE (see §4.1).
     Surface EACH via a DN-04/RFC-0013 structured diagnostic carrying:
       - the candidate type set (empty = no type exists; >1 = ambiguous),
       - the witnessing use sites,
       - the reason it is non-principal.
     NEVER guess one (G2/VR-5).

  4. Re-run the strict checker on the materialized program. If F' (the new soft-flag
     set) is empty, the program is strict — it compiles. Otherwise F' subset-of RESIDUAL
     is the exact human-resolution worklist.
```

### §4.1 The precise characterization of the "truly ambiguous" residual

The residual is **exactly** the set of type-level flags for which mechanical resolution is *non-unique
or undecidable* — and this is *decidable relative to the checker* by construction:

- **Non-principal / empty inference.** `Cx::infer` returns **⊥** (no type satisfies the constraints —
  a genuinely ill-typed program that only *ran* because the loose path took one branch) or a
  **non-principal set** (more than one type satisfies all constraints and the alternatives are
  observationally distinct — e.g. an integer literal used at two incompatible widths; a duck-typed
  value passed to two functions demanding different structural shapes).
- **Generic instantiation not forced by any use.** Polymorphism the loose run never disambiguated
  (`Cx::infer` cannot prove a principal instantiation).
- **A path strict typing rejects that the loose run exercised anyway** (the adversarial case, §9.1) —
  which, by the invariant below, is a *subset* of "non-principal / empty inference."

**The principality invariant (the soundness spine — never violated):**

> Mechanical strictification writes down **only a principal inferred type**. If a hole has no principal
> type (⊥ or non-unique), it is *definitionally* in the surfaced residual. Where the checker cannot
> *prove* principality (e.g. parts of the generic fragment), the hole is **conservatively surfaced**,
> never mechanically resolved.

This makes the algorithm **sound-by-conservatism**: it is impossible for mechanical strictification to
promote a program to a *different* (compiling-but-wrong) program, because the only thing it ever writes
is a type the inference proved unique. Everything else is a human decision, surfaced never-silently.

**Honesty tag (VR-5).** A mechanically-inserted type is tagged `Declared` (it was inferred) until the
strict re-check (step 4) passes — at which point the *strict check itself* is the checked basis and the
grade follows RFC-0018 normally. The mechanical step **never upgrades a tag**; it only records an
inferred type for the strict checker to then verify.

## §5 Ranked alternatives (objective table)

| Criterion (weight) | **Alt A — demotion-switch, single checker** | Alt B — inference-only pre-pass layer | Alt C — new gradual type-system (`Ty::Dynamic` plus blame) |
|---|---|---|---|
| KC-3 / new soundness surface (high) | **None** — a posture flag over the existing checker; certified path untouched | Low — a second pass, no kernel change | **High** — new kernel `Ty` variant, runtime cast nodes, blame tracking |
| DRY / single inference source (high) | **Yes** — reuses `Cx::infer` verbatim | **No** — duplicates/forks inference into a pre-pass to keep in sync | Yes, but a *new* system to maintain |
| Reuses landed scaffold (med) | **Fully** — checker, infer, repr-dynamic eval, `Declared` floor, DN-04 | Partly — new pass, still needs DN-04 | Little — new machinery throughout |
| Mechanical-strictification cost (high) | **Near-free** — strict = loose-with-demotion-off computes F directly | Moderate — must reconcile pre-pass annotations with strict checker | High — cast/blame boundaries must be resolved away |
| Honesty preservation (VR-5/G2) (high) | **Clean** — principality invariant; mode-level never-silent tag | Clean, but two gates to keep honest | Clean but heavyweight (blame is a new honesty surface) |
| YAGNI for a dev accelerator (med) | **Best fit** | Acceptable | Over-built for the stated need |
| Power ceiling (low) | Sufficient (infer-and-hint) | Sufficient | Maximal (true gradual typing plus blame) |

**Recommendation — Alt A (Rank 1).** A driver-level **posture flag** that classifies a subset of
`CheckError`s as *demotable* (type-level) vs *hard* (runnable-floor, §3.3), demotes the former to DN-04
hints in loose mode, and disables demotion in strict mode. Mechanical strictification is §4. Zero new
type theory, zero kernel change, the certified path exactly as today.

- **Alt B (Rank 2)** — a separate inference-only pre-pass that annotates the AST before the unchanged
  strict checker. More code, and it *splits inference from the checker* (DRY violation): the "loose
  bar" becomes a second artifact to keep in lockstep with the checker's own inference. Rejected on DRY.
- **Alt C (Rank 3)** — a genuinely new gradual type system (a `Dynamic` type plus a consistency
  relation plus runtime casts with blame, Siek–Taha style). Maximal power (true gradual typing with
  blame tracking), but violates KC-3 (new kernel variant plus cast nodes plus blame), adds a soundness
  surface, and is YAGNI for a dev-iteration accelerator. Rejected — **retained as the documented growth
  path** *if* the project ever wants blame-tracked gradual typing as a first-class language feature.
- **Alt D (rejected outright)** — overload RFC-0018 grades or the `fast`/`certified` mode to *mean*
  type-strictness. Rejected because it **conflates orthogonal axes** (§2): a `fast`-mode program is
  fully type-checked today; a `Declared` grade is provenance, not type-resolution. Overloading would
  corrupt the honesty semantics of both existing axes. This is the §2 verdict as a design constraint.

## §6 The honesty boundary (VR-5 / G2 / ADR-003)

Three invariants the design must hold, each a DoD gate:

1. **Mode-level never-silent tag (like ADR-032's mode tag).** A program run in loose mode is
   **tagged loose**, never silently presented as if strict-checked. "This program compiles under strict
   typing" is itself a `Declared` claim until the strict check passes (§4 step 4). The never-silent
   obligation lives at the **mode level** — exactly mirroring ADR-032's never-silent `CertMode` tag —
   not at the value-grade level.

2. **Guarantee-grades stay orthogonal and keep running.** A loose-inferred type does **not** touch a
   value's `Meta` guarantee. Guarantee propagation (`GuaranteeStrength::propagate`, the meet rule) runs
   at the *evaluator* level in **both** modes, so a loose-mode result carries the same honest meet-grade
   it would in strict mode. The type-inference `Declared`-ness (a *hint* tag on the type) is a
   **separate** thing from the value's *guarantee* grade. The DN must not conflate them, and the
   implementation must keep them in separate fields. **Backstop:** `CertMode::Fast` already caps at
   `Exact`/`Declared` and never fabricates `Empirical`/`Proven` (`cert_mode.rs:74`), so loose mode
   cannot over-claim a guarantee even if a type was inferred away.

3. **Value semantics untouched (ADR-003).** Loose mode changes only the checker's gate; the evaluator,
   the content-addressed identity, and the immutable-acyclic `Data` invariant are byte-identical. A
   loose-mode value has the same identity/hash as its strict counterpart. **DoD:** a differential test
   asserting *loose-run observable == strict-run observable* for every program that strict-checks.

## §7 Python carry-forward (source-language-general framing)

Python's gradual typing (PEP 484 hints over a duck-typed runtime) maps **exactly** onto loose mode: an
inferred/duck type is a `Declared` hint, not a compile gate. The M-1077 payoff composes **existing**
stages — **Python → Rust** (py2rust) → **Rust → Mycelium** (the existing transpiler,
`crates/mycelium-transpile`) — with py2rust's own inferred/duck types entering as loose-mode hints
*rather than* being required to resolve to Mycelium's strict kernel vocabulary up front. This builds
**no new Python frontend**; it reuses two pipeline stages plus the loose acceptance mode this note
scopes. Framed source-language-general: **loose mode is the acceptance surface for any gradually-typed
source** (Python, TypeScript, …) — the strict gate then applies whenever that source is compiled, and
§4's mechanical strictification is the automated path from "ported and running" to "compiles", with the
`Declared`/`Empirical` boundary preserved (no fabricated "full Python support" from a narrow fixture).

## §8 Related future work (out of scope here) — the lexicon-alias DX layer

The companion DX idea (a conventional-term → fungal-native alias layer with INFO-level *teaching*
diagnostics — e.g. accept "module"/"crate" and teach `nodule`/`phylum`) shares this note's DN-04
surfacing channel and its never-silent posture, but is a **surface-vocabulary** concern, not a
**typing-mode** one. Noted as related future work; **kept out of scope** to keep DN-126 focused on the
typing axis. It deserves its own DN if pursued.

## §9 Adversarial stress-test (VR-5 — attack the recommendation)

### §9.1 Unsound loose→strict promotion (the sharpest attack)

*Attack:* a duck-typed value V runs fine on the fast path because at runtime it happened to have a repr
the called op accepted (the loose run took one branch), but there is **no** static type making the
whole program strict-typeable — e.g. V is used at `Binary{8}` in one branch and `Data("Foo")` in
another, reconciled only because execution took one branch. If mechanical strictification "just picked a
type", it would silently compile a *different* program. **This must surface, never silently compile.**

*Verdict — the design survives, by the §4.1 principality invariant.* Such a V has **no principal
inferred type** (`Cx::infer` returns ⊥ or a non-unique set), so it is *definitionally* in the surfaced
residual. Mechanical strictification only ever writes a *principal* type; the unsound case has none, so
it is impossible to promote it. The never-silent obligation is discharged by construction, not by a
heuristic. *Narrowing this produced §3.3 (the runnable floor) and the "sound-by-conservatism" rule:
where principality cannot be **proven** (generic fragment), the hole is conservatively surfaced, never
resolved.* No hole remains here.

### §9.2 Value-semantics interaction

*Attack:* does loose mode perturb content-addressed identity or the immutable-acyclic `Data` invariant
(ADR-003)? *Verdict — no.* Loose mode changes the checker's gate, not the evaluator; the runtime, the
identity, and the hash are byte-identical across modes (§6.3). The one obligation is the differential
DoD test (loose observable == strict observable for any strict-checkable program). No hole.

### §9.3 The honesty-tag boundary

*Attack:* could a loose-inferred (`Declared`) type leak into a context that then claims
`Exact`/`Proven`, over-claiming a guarantee? *Verdict — no.* Type-strictness and guarantee-grade are
orthogonal (§6.2): the value's `Meta` guarantee is set by `GuaranteeStrength::propagate` in the
evaluator regardless of mode, and `CertMode::Fast` caps loose mode at `Exact`/`Declared`
(`cert_mode.rs:74`). The type hint's `Declared`-ness never touches the guarantee grade. The one design
obligation: keep the *type-hint tag* and the *guarantee grade* in **separate fields** (do not overload
one `Declared`). Flagged as an implementation invariant plus a DoD gate. No hole once that separation
holds.

### §9.4 Residual honest holes (surfaced, not hidden)

- **Principality decidability in the full generic fragment** is `Declared` — the v0 monomorphic
  fragment computes principality, but stage-1b grade/type polymorphism (RFC-0018 §4.7, not yet built)
  may not. The design is **conservative** there (surface, don't resolve), so it is *safe* but may
  surface *more* than strictly necessary. Acceptable (over-surface, never mis-promote); tighten as 1b
  lands.
- **Which exact `CheckError` sites are "demotable" vs "runnable-floor"** needs an enumeration pass over
  `checkty`'s refusal sites (they are flat `CheckError { site, message }` today, `checkty.rs:181`, so
  the partition is a classification table, not an enum split). This is a build-time inventory, flagged
  as the first implementation task, not a design hole.

## §10 Definition of Done (for the maintainer to ratify — house rule #6)

Ratification of DN-126 = the maintainer records an append-only decision that:

1. **Accepts the three-axis verdict** (§2): type-strictness is a new, orthogonal axis, *not* provided
   by RFC-0018 grading or ADR-032 `fast`/`certified`. (Or rejects it with a stated basis.)
2. **Selects the mechanism** — Alt A (demotion-switch, single-checker, §5) recommended — or picks
   another ranked alternative on the merits.
3. **Ratifies the runnable-floor boundary** (§3.3): name/arity/parse/FFI refusals stay hard in loose
   mode; only type-level refusals demote.
4. **Ratifies the mechanical-strictification principality invariant** (§4.1): mechanical only for a
   *principal* inferred type; everything non-principal/undecidable is surfaced via DN-04, never guessed.
5. **Ratifies the honesty boundary** (§6): mode-level never-silent tag; guarantee-grades orthogonal and
   still propagated; value semantics/ADR-003 untouched; type-hint tag kept separate from guarantee grade.

Then M-1077 implementation may proceed to its own DoD (the two fixtures plus the py2rust end-to-end
fixture, per the issue), with these guarantee tags: the loose-mode acceptance is `Declared`; the
differential loose==strict observable and the strict-gate-still-refuses fixture are `Empirical` once
they run; no fabricated "full Python support" claim (VR-5/G2).

**Applied at the 2026-07-12 ratification close-out (append-only note, original text above left
as-authored):** `Doc-Index.md` DN-126 row added at status **Accepted**; `CHANGELOG.md` carries the
ratification entry; **M-1077** (already filed, tracking this design) gets `doc_refs: corpus:DN-126`
plus an append-only close-out note recording DoD item 1 ("a DN scoping the loose-mode mechanism … is
ratified before implementation") satisfied — the build itself (the two fixtures + the py2rust
end-to-end fixture) remains unbuilt, `status: todo` unchanged.

---

## Meta — changelog

- **2026-07-12 — Draft filed (DN-126).** Design-reasoner recommendation for M-1077. Delivers the
  three-axis verdict (§2, correcting the task framing — `fast`/`certified` is ADR-032, grading is
  RFC-0018, neither is type-strictness), the two-mode model (§3) with the runnable-floor boundary
  (§3.3), the mechanical-strictification algorithm plus principality invariant plus truly-ambiguous
  residual characterization (§4), the ranked alternatives with an objective table (§5, Alt A
  recommended, Alt C the documented growth path, Alt D rejected), the honesty boundary (§6), the
  Python source-language-general carry-forward (§7), the lexicon-alias note as out-of-scope future work
  (§8), the adversarial stress-test (§9 — the unsound-promotion attack survives by the principality
  invariant), and the DoD for maintainer ratification (§10). `Empirical` claims read against
  `dev@fa53dc46` with `file:line`; every proposed mechanism `Declared`. Not Accepted, not Enacted.
