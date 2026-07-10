# Design Note DN-109 — Layer-4 Idiom-Optimal Transpilation and Layer-5 Structural Remapping (module to nodule / crate to phylum)

| Field | Value |
|---|---|
| **Note** | DN-109 |
| **Status** | **Accepted** (2026-07-11, maintainer ratification — see the dated "Ratification / Maintainer decision" note below: the L4/L5 design accepted, fork F1/§7-b reframed per the gap-closure-default and native-solution-mapping principles, one fork (§7-d) flagged as not cleanly resolved). Originally **Draft (advisory)** (2026-07-10). At Draft time this RECOMMENDED a design; **ratified nothing, shipped no code.** |
| **Feeds / builds on** | **DN-34** (Rust to Mycelium transpiler strategy, the gap-profiling instrument, not a bulk porter; §8.7-§8.8 M-991 verdict), **DN-85** (multi-language transpilation program), **DN-06** (`phylum`/`nodule`/`colony` lexicon), **RFC-0006** (L0 to L1 to L2 layer cake plus invariants S1-S5), **RFC-0001/0012** (paradigms plus ambient repr), **ADR-003** (content-addressed identity over elaborated L0), **ADR-014** (`wild`), **G2/VR-5** (never-silent / honest tags). |
| **Decides** | *Nothing normatively.* Recommends (L4) a three-bucket idiom-decision framework, mechanical / heuristic / judgment, bound by a single "no-silent-upgrade" rule; and (L5) a structure-preserving default plus a machine-readable **remap manifest** provenance artifact, with the transpiler additions required to support it. |
| **Guarantee** | Every claim here is **`Declared`**, this is a design proposal, not a measurement or a proof. |

> **Posture (transparency rule / VR-5 / G2).** This is a recommendation for a maintainer to
> ratify, not a ratification. It extends DN-34 without superseding it: DN-34 §8.7's **M-991
> verdict** — *the transpiler is a gap-profiling instrument, not a bulk porter; emitted `.myc`
> stays `Declared` until a differential upgrades it* — is the binding frame both layers below must
> not violate. The single worst failure this note guards against is an "optimization" or
> "restructuring" that **silently changes observable behavior or upgrades a guarantee tag**. When
> in doubt: **flag, don't guess.**

---

## §1 Frame — the two coupled questions

The maintainer's directive: *"opt for the Mycelium-optimal paradigm, implementation, and idioms
wherever possible by strategic, systematic, intelligent analysis"* (L4), and *"restructure the
crate's modules into an optimized set of nodules... mapped and documented and explained for
how/where/why"* (L5).

A **correct** 1:1 translation is DN-34's current output (flat `out_dir/<stem>.myc`,
stem-collision last-writer-wins). Correct is not the goal; **idiomatic-and-still-correct** is. The
two layers are coupled because they share one discipline: *an automatic transformation may fire
only when it is provably semantics-preserving under value-semantics, inserts no `swap` (S1),
upgrades no guarantee tag (VR-5), and is recorded in an EXPLAIN-able manifest.* Everything else is
a flag.

### §1.1 Definition of Done (this note, for maintainer ratification)

1. The L4 three-bucket classification (§3) is accepted or amended, in particular **which decision
   points may auto-fire in v0** vs remain flags.
2. The L5 remap-manifest schema (§5.2) is accepted as the provenance artifact of record, and the
   structure-preserving 1:1 default (§5.3) is accepted as v0.
3. The forks in §7 are resolved (they are *not* decided here).
4. A follow-on epic is minted with per-transpiler-change DoD = "the differential witnesses no
   behavior change vs the Rust original, and the manifest EXPLAINs every non-1:1 decision."

### §1.2 User stories

- *As the maintainer driving zero-hand-port*, I want the transpiler to emit **idiomatic** Mycelium
  (value semantics, paradigm-native forms) where it can prove that safe, so the hand-refine
  residue shrinks, **but never at the cost of a silent behavior change** I'd have to hunt for
  later.
- *As a reviewer of a ported phylum*, I want a single machine-readable ledger that tells me, for
  every target nodule and every non-obvious idiom choice, **which Rust location it came from and
  why the transpiler chose this form**, so I can audit the port without diffing against the Rust
  by eye.
- *As a future contributor*, I want restructuring (module-to-nodule consolidation/split/relocation)
  to be **opt-in and reviewable**, not an opaque auto-reshuffle, so the phylum's shape is a
  decision I can trace.

---

## §2 Grounding the baseline (mitigation #14, verified against code, 2026-07-10)

- `crates/mycelium-transpile/src/bin/mycelium-transpile.rs::run_batch` writes **flat,
  stem-named** output (`out_dir/<stem>.myc` plus `<stem>.gap.json`); directory structure is
  **discarded**; two files sharing a stem (e.g. two `mod.rs`) collide **last-writer-wins with a
  loud warning** ("no path-qualification in this PoC's per-file naming"). So today there is
  **no** structural mapping and **no** provenance beyond the per-file gap report, L5 starts from
  *worse than* structure-preserving 1:1.
- `batch.rs` transpiles each file independently; the only cross-file artifact is a **union gap
  report** plus summary. There is no nodule/phylum planner, no many-to-one mapping, no remap
  ledger.
- DN-34 §3 maps *constructs*, not *structure*; §8.11 already lands **shared-reference type
  erasure** (`&T` dropped), which is the first L4 idiom move and a useful precedent, it is safe
  precisely because a shared immutable borrow is a no-op under value semantics.
- **No prior note** covers module-to-nodule / crate-to-phylum remapping or a provenance manifest,
  this is new design space (grep of `docs/notes/` plus `issues.yaml`, 2026-07-10).

---

## §3 L4 — the idiom-decision framework (mechanical / heuristic / judgment)

Every idiom decision point is classified by **who can soundly decide it**:

- **MECHANICAL** to a transpiler rule, auto-emit. Statically decidable *and*
  semantics-preserving *and* no tag upgrade. Fires in v0.
- **HEURISTIC** to a rule that emits the idiomatic form **with an EXPLAIN flag for review**, OR
  (v0-safe mode) emits the conservative form and flags the optimization as a *candidate*. Never
  silent.
- **JUDGMENT** to **flag, don't guess.** Needs semantic/human input the source does not carry.
  The transpiler emits a located gap/flag and the conservative-or-no emission; it never invents
  the answer.

### §3.1 The decision points, classified (`Declared`)

| # | Rust to Mycelium decision | Class | Why |
|---|---|---|---|
| D1 | `u8/u16/...` to `Binary{8/16}`; `bool` to prelude `Bool` | **Mechanical** | The Rust type dictates the width; direct, lossless. |
| D2 | `Option`/`Result`/`?` to never-silent `Option`/`Result` plus explicit `match` | **Mechanical** | Rust is already explicit; `?` lowers to a `match` (DN-34 §3). Faithful. |
| D3 | `struct`/`enum` to data decl (`type`/`Construct`/`Match`) | **Mechanical** | Data registry (RFC-0011); Maranget exhaustiveness already exists. |
| D4 | `&T` shared immutable borrow to value (erase the reference) | **Mechanical** (already landed §8.11) | A shared read is a no-op under value semantics. |
| D5 | named `fn`-as-value to L1 `Lam`; **env-capturing closures** | **Judgment/Impossible** | Closures are auto-`Impossible`, must be flagged (DN-34 §3; research-18 §3). |
| D6 | `unsafe` to `wild` | **Mechanical to detect, flag to place** | Never silently transpiled (ADR-014); emitted as a flagged `wild` site. |
| D7 | `&mut T` in-place mutation to value-semantics functional update | **Judgment** (Heuristic *only* with borrowck facts) | If two references alias, functional update is **observably different**. `syn` cannot prove non-aliasing (needs rustc MIR `mir_borrowck`, DN-34 §6-Q3). The core VR-5 trap, see §6.1. |
| D8 | iterator chains (`.map().filter().collect()`) to `for` / structural recursion | **Heuristic if provably bounded, else Judgment** | Mycelium `for` is **bounded, Total-by-construction**; a lazy/unbounded iterator has **no faithful target**. `while`/`loop` are *unreserved* (divergence bit, lexicon §6/DN-02). Silent map bounds an unbounded computation. |
| D9 | `panic!`/`unwrap`/`expect` to never-silent `Option`/`Result` or explicit refusal | **Mechanical to detect, Judgment to retarget** | `unwrap` erases the error path; a silent extraction violates G2. What to do on `None` is a human decision, flag it. |
| D10 | paradigm/representation choice **binary** (from integer types) | **Mechanical** | Inferable from the Rust type. |
| D11 | paradigm choice **Dense** (f32 arrays in an ML workload) | **Heuristic plus flag** | The *workload* is not in the type; `[f32; N]` is not "embedding". Emit `Dense` only as a flagged candidate. |
| D12 | paradigm choice **Ternary / VSA** | **Judgment, always declared** | Rust has **no** ternary or hypervector type. Any ternary/VSA target is a **human declaration**, never a transpiler inference. Inferring it is an unsound guarantee upgrade (§6.2). |
| D13 | numeric `as` / `.into()` width or repr change to `swap` | **Judgment, never auto-insert** | **S1 forbids any inferred `swap`.** A `swap` names `to:` and `policy:` (rounding/saturation) that Rust's `as` fixes silently. Emit a **flagged candidate swap**, never an auto-swap. |
| D14 | error/iterator/Display trait impls to paradigm-native forms | **Heuristic plus flag** | Often a genuine idiom win, but changes the public surface, needs review. |

### §3.2 The one binding rule (the ratchet)

**An automatic idiom transformation may fire only if ALL hold:** (1) statically
**semantics-preserving under value-semantics**; (2) it **inserts no `swap`** (S1); (3) it
**upgrades no guarantee tag** (VR-5, e.g. it may not turn a truncating conversion into a
claimed-exact one); (4) it is **recorded EXPLAIN-ably** (the manifest, §5.2, an idiom-choice
column). Anything failing any clause is **Heuristic (flag) or Judgment (flag)**, never a silent
rewrite. This keeps the transpiler a **gap-profiling plus provenance instrument** (M-991 verdict),
augmented with a *safe* mechanical-idiom set, not an idiomatic-rewrite engine.

### §3.3 L4 alternatives, evaluated

- **A. Keep correct 1:1 mechanical mirror; defer all idiom optimization** (DN-34's current
  posture). Simplest (KISS), safest, but not the maintainer's goal (leaves value-semantics/
  paradigm wins on the table).
- **B. Tiered framework (§3.1) with the §3.2 ratchet plus EXPLAIN.** *Recommended.* v0
  auto-fires **only the Mechanical bucket** (D1-D4, D6-detect, D10); Heuristic and Judgment emit
  **flags**, not silent rewrites, until a differential upgrades each rule case-by-case. Grows the
  auto-set by evidence, honestly.
- **C. Aggressive idiomatic rewriter** (auto-pick paradigms, auto-`swap`, auto value-semantics on
  `&mut`). **Rejected**, violates S1 (D13), VR-5 (D12), and the §6 aliasing/termination traps.

**Ranked L4 recommendation: B > A > C.** v0 scope = **Mechanical bucket only auto-applies**;
Heuristic plus Judgment ship as flags. This collapses B toward A *for v0 output* but installs the
ratchet that lets the auto-set grow safely as differentials accrue.

---

## §4 L5 — structural remapping (module to nodule / crate to phylum)

Rust module/crate structure need not map 1:1 to Mycelium nodules/phyla. The maintainer's **hard
requirement**: every restructuring must be **mapped, documented, and explained (how/where/why)**,
machine-readable, human-auditable, EXPLAIN-able. §5 designs that artifact; this section sets the
rules.

### §4.1 The restructuring operations

`Keep` (1:1 mod-to-nodule), `Consolidate` (N mods to 1 nodule), `Split` (1 mod to N nodules),
`Relocate` (move a component to a different nodule), `CrateToPhylum` (a crate to a phylum with a
reshaped nodule set).

### §4.2 Safe vs risky

- **SAFE (mechanical/heuristic):** structure-preserving `Keep` (Rust `mod` path to nodule dotted
  path); flattening an **inline `mod foo { ... }`** into its parent nodule (inline modules are
  pure namespacing, value semantics untouched); `Split` of a nodule along item boundaries (nodules
  are namespacing). These do not change *what is exported* or *how items elaborate*.
- **RISKY (flag for review):** any op that changes the **public API surface**, `pub use`
  re-exports, visibility widening (a Rust-private item becoming cross-nodule visible),
  `Consolidate` with name collisions, cyclic module dependencies (Mycelium cross-nodule rules may
  forbid), macro-generated module content, anything that changes which `swap`/guarantee is
  observable at a boundary. **Never auto-applied.**

### §4.3 Identity note (ADR-003)

Content-addressed identity is over **elaborated L0**, not file layout, so a pure
`Keep`/`Split`/inline-flatten is **identity-neutral**. **FLAG** any restructuring that would
change elaboration (and therefore identity), that is a semantic edit, not a reshuffle.

---

## §5 L5 — what the transpiler must gain, and the provenance artifact

### §5.1 Transpiler additions (v0)

The transpiler today is **flat plus per-file**. To support L5 it needs three additions:

1. **A path model.** Preserve the Rust `mod` tree to a **nodule dotted path**
   (`a/b/mod.rs` to `nodule a.b;`), replacing the flat stem-collision scheme. This *alone* fixes
   the current data-loss bug (two `mod.rs` overwriting each other).
2. **A nodule-planner stage** (runs *before* emission): consumes the discovered file/module tree,
   produces a **remap plan** (the manifest, §5.2), a bounded working-set artifact that the
   emitter then follows (fits the `/forward` DN-96 context-windowing: plan persists, drops from
   context, emitter consumes it). v0 plan = pure `Keep`; `Consolidate`/`Split`/`Relocate` entries
   are added **only** when a human directive or a reviewed heuristic requests them.
3. **A manifest emitter**, writes the machine-readable ledger plus a rendered human view.

### §5.2 The remap manifest (the provenance artifact of record)

A committed sidecar per transpiled phylum, machine-readable `remap.json` **and** a rendered
`REMAP.md` (byte-derivable from the JSON). **Item-granular**, not just file-granular. Proposed
schema (`Declared`):

```text
remap.json
  phylum:  { source_crate, target_phylum }
  nodules: [ { target_nodule, operation: Keep|Consolidate|Split|Relocate|CrateToPhylum,
               sources: [ { rust_path, rust_span, moved_items:[...] } ],
               rationale,                 // WHY, the how/where/why the maintainer requires
               safety: Safe|Review,       // §4.2 classification
               api_surface_changed: bool, // true forces Review
               identity_neutral: bool,    // §4.3 / ADR-003
               guarantee: "Declared" } ]
  idiom_choices: [ { target_span, rust_span, decision: D#,   // the L4 EXPLAIN trail (§3.2 clause 4)
                     class: Mechanical|Heuristic|Judgment, chose, alternatives, reason } ]
```

This is the **unification of** the L5 restructuring ledger and the L4 idiom-choice EXPLAIN trail
into one auditable record. It is `reveal`-adjacent (never a black box, S4) and is the reviewer's
worklist. (This also subsumes what would otherwise be a separate "successful-choice EXPLAIN"
diagnostic — the ledger's DX appendix D7 finding — so no separate `explain.json` is proposed.)

**Design choice (flagged as a fork, §7):** make this a **new** artifact, or **extend the existing
`summary.json` / `union.gap.json`** the batch runner already emits. Leaning: extend, the gap
report already carries `{file, line, reason, category}`; the manifest is the same ledger with
target-side plus rationale columns. Not decided here.

### §5.3 L5 alternatives, evaluated

- **A. Keep flat stem-based 1:1** (current). **Rejected**, discards structure, loses provenance,
  collision-lossy.
- **B. Structure-preserving 1:1 (`mod` path to nodule dotted path) plus the remap manifest, from
  v0.** *Recommended.* Fixes the collision bug, satisfies the "mapped/documented/explained"
  mandate on day one, and makes the manifest the substrate every later restructuring records
  into. `Consolidate`/`Split`/`Relocate` are **opt-in**, plan-reviewed, and **never change the
  public API without a Review flag**.
- **C. Full auto-restructuring** (cluster analysis auto-consolidates/splits). **Rejected for v0**
  (YAGNI plus unsound-upgrade risk); revisit later as **Heuristic-with-review** once the manifest
  plus differential exist.

**Ranked L5 recommendation: B > C (deferred) > A (reject).** The **manifest is mandatory from v0
even for pure 1:1**, it is the provenance ledger the maintainer requires and the ratchet substrate
for all later restructuring.

---

## §6 Adversarial stress-test (VR-5 trap), where this breaks

### §6.1 The `&mut` to value-semantics aliasing trap (top concern #1)

Rust `&mut` in-place mutation where two references **alias**, or where a caller **observes** the
mutation through a pointer, becomes **observably different** under a value-semantics
functional-update rewrite. Detecting `&mut` is trivial; **proving non-aliasing is not**, `syn` is
syntax-only and cannot see it (rustc MIR `mir_borrowck` is the authoritative source, DN-34
§3/§6-Q3). Therefore a value-semantic rewrite of `&mut` is **Judgment (flag), not Heuristic**,
unless/until the transpiler acquires a borrowck front-end (fork §7-b). Where "idiomatic value
semantics" quietly rewrites aliased mutation, behavior changes silently, the exact failure the
note exists to prevent.

### §6.2 The swap-insertion / guarantee-upgrade / termination trap (top concern #2)

Three faces of one failure, an "optimization" that changes the **observable guarantee,
representation, or termination**:

- **Swap insertion (S1):** Rust `as`/`.into()` *looks* like a `swap`, but S1 forbids inferred
  swaps, and a `swap`'s `policy:` (rounding/saturation) is a semantic choice Rust's `as` makes
  silently. Auto-emitting a swap both breaks S1 and can present a truncation as exact.
- **Guarantee upgrade (VR-5):** choosing `Dense`/`VSA`/`Ternary` (D11/D12) commits to a
  representation *guarantee* the Rust source never states. Inferring it upgrades `Declared`
  toward a claimed representation-appropriateness that has no basis, a VR-5 violation.
- **Termination (divergence bit):** mapping an unbounded `loop`/`while`/lazy-iterator (D8) to
  Mycelium `for` **silently bounds an unbounded computation**. `while`/`loop` are *deliberately
  unreserved* (DN-02 §6); a "faithful" `for` map is not faithful. Must be flagged, never mapped.

All three are why §3.2's ratchet is **conjunctive** (all four clauses), any single clause failing
sends the decision to a flag.

---

## §7 Forks left for the maintainer (NOT decided here, house rule #3 / VR-5)

- **(a) Restructuring target level.** Restructure at **L2 surface** (human-refinable, more
  legible for the review loop) vs **Core IR** (mechanical, less reviewable). DN-34 §6-Q2 already
  flags this; L5 leans L2.
- **(b) Acquire a rustc/rust-analyzer front-end?** Without borrowck facts, the `&mut`-to-value-
  semantics rewrite (D7) and the iterator-boundedness heuristic (D8) stay **Judgment/flags
  forever**. A front-end promotes them to Heuristic/Mechanical but is a large scope commitment
  (DN-34 §6-Q1/Q3). Big call — this is **F1** in the companion ledger
  (`docs/planning/zero-hand-port-delta-ledger.md` §7), the single most pivotal open fork across
  the whole zero-hand-port program.
- **(c) Manifest: new artifact or extend `summary.json`/`union.gap.json`?** (§5.2, schema
  unification.)
- **(d) Does non-1:1 restructuring belong in the transpiler at all,** or in a separate
  human-driven post-transpile "nodule refactor" pass, keeping the transpiler a pure gap-profiler
  (M-991 / KISS-YAGNI boundary)? The manifest works either way; ownership of the *decision engine*
  is the question.
- **(e) v0 auto-fire set.** §3.3-B v0 auto-fires the Mechanical bucket only. Confirm, or move
  specific Heuristic rules (e.g. D8-bounded, D11-Dense) into auto-with-mandatory-flag.

---

## §8 Definition of Done for maintainer ratification (rule #6)

"Accepted" requires the maintainer to: (1) accept/amend the §3.1 classification and the **v0
auto-fire set** (§7-e); (2) accept the **remap-manifest schema** (§5.2) and resolve the
new-vs-extend fork (§7-c); (3) accept **structure-preserving 1:1 plus mandatory manifest** as v0
(§5.3-B); (4) rule on forks §7-a/b/d; (5) authorize a follow-on epic whose per-change DoD =
*"the differential witnesses no behavior change vs the Rust original, and the manifest EXPLAINs
every non-1:1 structural op and every non-Mechanical idiom choice."* Until then this note is
**Draft**; all guarantees remain **`Declared`**.

## §9 Tracked issues filed alongside this note (2026-07-10)

The integrating session filed the following against the L4/L5 design and the companion L2/DX
findings (`docs/planning/zero-hand-port-delta-ledger.md`); none of these are self-ratifying, they
are scoped implementation work whose design is either already settled (mechanical DRY refactors)
or explicitly gated on this note's ratification (the remap manifest, the idiom EXPLAIN trail):

- **M-1041** — visitor/fold trait for the AST walkers (the L2 meta-gap plus DX-D1, converged).
- **M-1042** — structured transpiler output plus path-qualified names (DX-D2/D3; the §5.1 path
  model's transpiler-CLI prerequisite).
- **M-1043** — per-item `// src: file:line` provenance breadcrumbs (DX-D3b).
- **M-1044** — the remap manifest (`remap.json`/`REMAP.md`, §5.2) — gated on this note's fork §7-c
  resolving.
- **M-1045** — actionable `suggested_idiom` on gap diagnostics (DX-D6).
- **M-1046** — "closest-to-clean" investment ranking in the vet report (DX-D8).
- **M-1047** — transpiler DX polish (`mycfmt` post-pass DX-D4, dry-run/summary mode DX-D5, minor
  arg-parsing DX-D12, combined).

## §10 FLAGs (append-only rows the integrating parent applies)

- **CHANGELOG.md** — an "Added (design, pending ratification)" row for DN-109 plus the M-1041..
  M-1047 filings, applied by the integrator landing this batch.
- **docs/Doc-Index.md** — DN-109 registered by the integrator landing this batch.
- **CLAUDE.md** — no change proposed.
- **Maintainer note:** the original directive referenced "DN-99"; DN-99 was already taken (the
  spw stdlib-port surface-gap register). This design lands as **DN-109**, confirmed free by grep
  of `docs/notes/` at reconciliation time (highest landed was DN-108).

## Ratification / Maintainer decision (2026-07-11)

> **Ratified** — part of the maintainer's batch approval "approving and ratifying the rest of that set
> from 101–109."
>
> **F1 reframe (maintainer, same thread).** Per the maintainer's gap-closure principle (established
> ratifying DN-106): the `&mut`/aliasing (D7) and unbounded-loop (D8) cases this note's fork §7-b names
> are **deliberate exclusions**, not gaps a borrowck frontend is *required* to close.
>
> **Refinement (maintainer, same thread).** *"Mycelium has different native ways to solve the same
> problems those excluded constructs solved — value-semantics/functional-update for mutable state,
> structured/bounded control for unbounded loops, explicit never-silent `swap` for representation
> change, errors-as-values, etc. So porting an excluded construct = map its underlying PROBLEM →
> Mycelium's native SOLUTION (auto-emit where safe/mechanical; where it needs judgment … flag WITH the
> suggested native idiom … so the dev applies the known native solution). Bare never-silent refusal is
> only the last resort."* And the **transparency requirement**: the full lowering pipeline
> (surface→…→lowest compiled form) must work across the language's intentionally-different
> conventions/environments **and** stay transparent/revealable.

**Recorded decision (append-only — this note's original §1–§10 text above is unchanged; this section
resolves the §8 DoD items + the §7 forks, per house rule #3):**

1. **§3.1 classification + the v0 auto-fire set (§7-e) accepted.** The Mechanical bucket (D1–D4,
   D6-detect, D10) auto-fires in v0; Heuristic and Judgment emit flags (never silent rewrites), exactly
   as §3.3's ranked recommendation B specifies.
2. **The remap-manifest schema (§5.2) accepted** as the provenance artifact of record; **structure-
   preserving 1:1 (§5.3-B) accepted as v0**, mandatory manifest from v0 even for pure `Keep`.
3. **Fork §7-a (restructuring target level) accepted per the note's own lean: L2 surface** (more
   legible for the review loop), not Core IR.
4. **Fork §7-c (manifest: new artifact or extend `summary.json`/`union.gap.json`) accepted per the
   note's own lean: extend** the existing gap-report artifact rather than introduce a new one (§5.2's
   "Leaning: extend" text) — an implementation detail for **M-1044**, not blocking this ratification.
5. **Fork §7-b / F1 (acquire a rustc/rust-analyzer borrowck frontend) — REFRAMED, not left open, per
   the gap-closure default + the native-solution-mapping refinement above.** §6.1/§7-b originally framed
   D7 (`&mut` aliasing) and D8 (unbounded-iterator boundedness) as staying "Judgment/flags forever"
   *without* a borrowck frontend, implying the frontend is close to a *requirement* for handling them at
   all. That framing is corrected: **D7 and D8 are exactly the deliberate-exclusion set** (in-place
   aliasable mutation, unbounded iteration) that Mycelium's native solutions already cover —
   destructure-and-reconstruct functional update (the D7 problem's native solution, and precisely what
   DN-106 ratified for the record-update case) and bounded `for`/structural recursion (the D8 problem's
   native solution, RFC-0007 §4.8). Per the refinement: where the Rust↔Mycelium problem→solution mapping
   is **safe/mechanical**, the transpiler auto-emits it (D4's `&T`-erasure precedent, already landed);
   where it needs **judgment** the source doesn't carry — `syn` cannot prove D7's non-aliasing, D8's
   boundedness is not always syntactically evident — the transpiler **flags WITH the suggested native
   idiom** (§3.1's `suggested_idiom` extension to a gap diagnostic, filed as **M-1045**) rather than a
   bare, unhelpful refusal. **A borrowck/rust-analyzer frontend is therefore an OPTIONAL PRECISION AID**
   — it would let the transpiler auto-detect *which* sites are provably safe to auto-map vs which need
   the flagged-suggestion path (upgrading some D7/D8 sites from Judgment to Heuristic/Mechanical with a
   checked basis) — **not a blocker for mechanical porting**, since the flagged-suggestion path already
   gets a developer to Mycelium's native solution without it. Recorded as a **tracked forward question**
   (not resolved to "acquire" or "don't acquire" — that remains a real, large scope commitment per
   DN-34 §6-Q1/Q3 and the delta ledger's F1), filed as **M-1052** (below).
6. **Fork §7-d (does non-1:1 restructuring belong in the transpiler, or a separate human-driven
   post-transpile pass?) — FLAGGED, not cleanly resolved.** Neither the maintainer's ratification message
   nor this note's own text states a lean for §7-d (unlike §7-a/§7-c, which the note's own prose already
   leans on). Per the standing instruction to flag rather than guess a maintainer's position: **this
   fork stays genuinely open**, tracked as a residual of this ratification rather than silently assumed
   either way. It does not block DN-109's Accepted status (the manifest, per §5.2, "works either way" —
   only the *decision engine's ownership* is unresolved) but is called out here for the maintainer's
   confirmation.
7. **New requirement (this ratification, not in the original §1–§10 text): the full lowering pipeline
   must stay transparent/revealable across the language's intentionally-different conventions.** Beyond
   §3.2's EXPLAIN-able-manifest ratchet (which covers idiom *choices*), the maintainer requires that the
   **entire** surface→…→lowest-compiled-form pipeline remain inspectable end-to-end, including at the
   points where Mycelium's native solution genuinely differs in convention/environment from the Rust
   source (value semantics vs `&mut`, bounded `for` vs unbounded `loop`, explicit `swap` vs `as`,
   errors-as-values vs panics/exceptions) — a dev must be able to reveal, at any stage, why the pipeline
   chose the native form it did. This **ties directly to the desugar/expand-on-demand capability** filed
   under DN-106's ratification (**M-1051**): the same on-demand-reveal mechanism DN-106 requires for
   surface sugars is the natural vehicle for this pipeline-wide transparency requirement too — cross-
   linked so the two don't diverge into separate tooling.
8. **DN-109 moves Draft (advisory) → Accepted** on this basis (items 1–4, 5's reframe, and 7 resolve the
   note's design; item 6 is flagged as a residual open question, not a blocker).
9. **Follow-up filed:** **M-1052** — "DN-109 F1 (reframed) — optional borrowck/rust-analyzer precision
   aid for auto-detecting deliberate-exclusion sites (D7/D8), vs the flagged-suggested-native-idiom
   default path" (`status:todo`, `doc_refs: corpus:DN-109, corpus:DN-34`, `tools/github/issues.yaml`).

## Changelog (this note)

- **2026-07-11** — **Ratified (maintainer, house rule #3).** Status **Draft (advisory) → Accepted** —
  part of the batch ratification of DN-101–DN-109. §3.1/§7-e (Mechanical-only v0 auto-fire), §5.2/§5.3-B
  (remap manifest + structure-preserving 1:1), §7-a (L2 surface), and §7-c (extend the existing gap-
  report artifact) accepted per the note's own recommendations/leans. **Fork §7-b/F1 REFRAMED**: D7/D8
  are the deliberate-exclusion set with native Mycelium solutions (functional update, bounded `for`); a
  borrowck frontend is an optional precision aid, not a porting blocker — the default path is
  flag-with-suggested-native-idiom (`suggested_idiom`, M-1045). **Fork §7-d FLAGGED** as genuinely
  unresolved (decision-engine ownership), not silently assumed. Adds a new pipeline-wide
  transparency/revealability requirement, cross-linked to DN-106's desugar/expand capability (M-1051).
  Follow-up filed as **M-1052**. Append-only — the original design record above is unchanged; this is
  an added ratification note.
- 2026-07-10 — filed Draft, from the L4/L5 analysis (`docs/planning/zero-hand-port/delta-L4L5-idiom-structural-DRAFT.md`).
