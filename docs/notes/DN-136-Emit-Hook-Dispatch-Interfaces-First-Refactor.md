# Design Note DN-136 — The Emit Hook-Dispatch Interface (Interfaces-First Refactor for the Bulk Gap-Close Drive)

| Field | Value |
|---|---|
| **Note** | DN-136 |
| **Status** | **Accepted** (2026-07-12) — ratified through the strict 9-criterion DN-review gate on a **clean re-pass** after the derive-guarantee + DoD-witness + citation patch (see §10 and the ratification changelog). Ratification is the **delegated DN-review gate** (maintainer standing automation + explicit task this session), not a self-ratify by the note (house rule #3 preserved — the note itself only *recommends*; the independent gate *ratifies*). **Accepted authorizes the interfaces-first Phase-1 build** (§5) against the frozen contracts (Alt B; the ordered-pass-preservation invariant as a build-blocking contract; the §4.1 collision-safe `struct_layouts` `resolve`). It remains **design only** — **builds nothing** and edits no `crates/**`; every mechanism is `Declared`/unbuilt until its FLAGGED build issue lands and is differential-witnessed (VR-5), and a guarantee tag upgrades `Declared → Empirical` only on that green differential. `Doc-Index.md`/`CHANGELOG.md`/`issues.yaml` rows remain **FLAGGED** (§9), applied by the integrating parent. Prior status: Draft (2026-07-12). |
| **Task** | Design **Phase 1** of the remaining-gaps bulk drive: predevelop the shared **interface points** in `crates/mycelium-transpile` so bulk gap-closing leaves become **purely additive against frozen contracts** (no shared-`emit.rs` serialization). Deliver the emit hook-dispatch interface (ranked alternatives + a migration-preserves-soundness plan), a per-shared-interface stable-vs-needs-work assessment, the Phase-1 build decomposition + parallelism, the Phase-2 bulk-drive map, and an adversarial stress-test. **Does not touch `emit.rs`** (it is under concurrent edit by the residual-tail leaves M-1092/M-1093/M-1094 — DN-133/134/135). Parallel-cluster slot: **DN-136** (mit #1 — DN-133/134/135 taken by the residual-tail cluster; DN-136 verified free at `origin/dev` (`1bc7956b`, re-affirmed at `c044452d`)). |
| **Decides** | *Proposes, for ratification:* (1) the **verified diagnosis** — `emit.rs`'s dispatch is **already half-centralized** (`walk_expr`/`ExprVisitor`, `visit.rs:50/126`; `dispatch_item`, `transpile.rs:404`; the `prim_map::TABLE` registry, `prim_map.rs:140`), so the residual collision surface is **three sub-dispatch axes** *inside* the big handler bodies (pattern-kind in `map_pattern_inner`, derive/trait-shape in `emit_impl`, call-shape in `visit_call`/`visit_method_call`) — **not** a whole-module rewrite. (2) The **recommended interface**: generalize the **already-landed `prim_map::TABLE` pattern** into a **static per-axis handler-table** (Alt B), one submodule file per handler, consulted by the **unchanged single ordered pass** — ranked over a trait-object registry (Alt A) and a chained-visitor (Alt C). (3) The **one hard invariant that makes hook-ification sound**: the *ordered-pass-preservation* invariant — the table changes **who** handles a construct, never **when** or **in what order** constructs are visited, and the single `&mut EmitCtx` stays threaded through the one left-to-right pass. Parallelism is **development-time** (leaves author disjoint handler files), **never emission-time**. (4) The **per-interface** assessment (§4): `struct_layouts` **needs** the DN-134 collision-safe interface (design it here as the shared contract both M-1089/M-1093 consume); `map.rs` type-dispatch **needs** a light type-map table; `gap.rs Category`, `symtab.rs` (M-1084), and the prelude-trait seed set are **additive-as-is**. |
| **Feeds** | The remaining-gaps bulk drive (the `language-completeness-gap-inventory.md` §3 worklist) — turns each Phase-2 gap-class into an additive leaf; DN-133/134/135/M-1092/1093/1094 (the residual-tail, whose landed `local_mangled` ordering + `struct_layouts` collision-safety are the two soundness witnesses this interface must preserve); DN-132/DN-128 (pattern + derive landed capabilities the migration must not regress); DN-99 (surface-gap register). |
| **Grounds on** | **KC-3** (small, auditable kernel — the interface is *contained*: it reuses `walk_expr` and the `prim_map` table shape, adds no new kernel/`Ty`/eval surface); **DRY** (one dispatch policy per axis, generalizing the *existing* `prim_map` scan — not a second parallel dispatcher); **KISS/YAGNI** (house rule #5 — the smallest pattern that meets the additivity objective wins; the proven `&[Row]`-table beats a trait-object framework); **G2/never-silent** (every unrecognized construct falls through to the existing explicit gap — a table miss is a `GapReason`, never a silent no-op); **VR-5** (every claim tagged at its basis; `Declared` until built + differential-witnessed; no `Proven` claim); **DN-133** (the `local_mangled` single-left-to-right-pass ordering invariant — `emit.rs:174–196,315–324`); **DN-134** (the mandatory collision-safe `struct_layouts` population — `transpile.rs:332`; DN-134 §3 step 1); **DN-111** (native-equivalence taxonomy — handlers emit the ratified native form, they do not invent one); mitigations **#1** (verified DN slot free), **#11** (isolated worktree), **#13** (stale-base caught + corrected — see §0). |
| **Definition of Done** | §8. In one line: **Accepted** requires the maintainer to ratify (a) Alt B as the emit interface, (b) the ordered-pass-preservation invariant as a build-blocking contract, and (c) the §4 per-interface verdicts — after which the emit-hook seam leaf and the §5 parallel interface leaves may build against the frozen contracts. This note **does not** move to Accepted itself. |

---

## §0 Verify-first (mitigations #1, #13, #14)

Grounded against **`origin/dev@c044452d`** (current dev; re-pinned in the post-gate amendment — the
combinator emitter M-1092/DN-135 landed since the original `1bc7956b` pin, adding ~397 lines and shifting
every `emit.rs` anchor below ~2000, so all anchors are refreshed to `c044452d`). **A stale-base correction
is recorded honestly (mit #13):** the isolated worktree first branched from an older dev tip (`#1532`,
`08d8fc21`), at which DN-133/134/135 did *not* exist and `local_mangled`/`cross_nodule_resolve_mangled`
were absent — which would have made the task's "landed DN-133/134/135" premises read as false. Re-basing
onto `origin/dev` (mit #13's discipline: develop off the working tier, never a stale tip; first `1bc7956b`,
then merged to `c044452d` at the amendment) brought them in. **After correction, every task premise
verifies** (below). This is exactly the failure mit #13 exists to catch — flagged, not silently papered
over.

| Premise (task) | Verified at `c044452d`? | Anchor |
|---|---|---|
| `emit.rs` is a large shared dispatch every gap leaf edits | **Yes** — 4943 lines; `EmitVisitor` is a single `impl` | `emit.rs:1803` |
| A single-canonical expr dispatch already exists | **Yes (sharpens the design)** — `walk_expr`/`ExprVisitor` | `visit.rs:50,126` |
| The method-prim / combinator axis is already a registry | **Yes (the template)** — `prim_map::TABLE` + linear `lookup` | `prim_map.rs:140,221` |
| Qualified-call has a `local_mangled` observed-emission ordering | **Yes** — `EmitCtx.local_mangled`, populated in one left-to-right pass | `emit.rs:174–196,315–324` (DN-133/M-1094) |
| `struct_layouts` needs collision-safe enum-variant keying | **Yes** — still `Item::Struct`-only, bare-name-keyed | `transpile.rs:332`; DN-134 §3 step 1 |
| Derive has a per-derive atomicity guarantee (**corrected — not item-level all-or-nothing**) | **Yes, but two-level, not whole-item** — a single derived impl is all-or-nothing over its *fields* (`derive_show_impl`/`derive_init_impl` refuse the whole impl on any ineligible field), while across the derive *set* each derive independently emits its impl **or** records a sub-gap and the **item still emits** the composable derives | `derive_show_impl:3809`, `derive_init_impl:3861`, `lower_struct_derives:3899`, `emit_struct:4212/4222` (DN-128/M-1086) |
| DN slot | **DN-136 free** (highest is DN-135) | `docs/notes/` |

**Non-sycophantic finding (house rule #4).** The task frames Phase 1 as "the emit.rs hook-dispatch
refactor" as though the dispatch is monolithic. It is not: `walk_expr` and `prim_map::TABLE` already did
the hard centralizing work. The honest, smaller truth is that **Phase 1 is generalizing one proven
pattern (`prim_map`) to three more axes** — a contained change, not a rewrite. Designing it as a big-bang
refactor would violate KISS/YAGNI and needlessly risk the landed soundness gates. The recommendation
below is sized to the *actual* residual.

---

## §1 The problem, precisely located

A gap-closing leaf that adds "emit construct X" today must edit one of the shared handler **bodies**,
because the sub-construct recognition is an inline `match`/`if`-chain *inside* the body:

| Dispatch axis | Where it lives now | Shape today | Who serializes on it |
|---|---|---|---|
| **method-prim / combinator** | `visit_method_call` `emit.rs:2253` | **`prim_map::TABLE` scan** (already a registry) + inline conversion/desugar arms | *already additive* for a table row; the inline arms still collide |
| **call-shape** (bare / qualified-assoc / cross-nodule mangled) | `visit_call` `emit.rs:2150` | inline `match` on `c.func` shape + `local_mangled` resolution (DN-133) | every new call-shape leaf |
| **pattern-kind** (or / tuple / struct-variant / range / `@`) | `map_pattern_inner` `emit.rs:3492` | inline `match` on `Pat::*` | every pattern leaf (M-823/M-826/M-1089 serialized here) |
| **derive-rule + trait-shape** | `emit_impl` `emit.rs:4601`; derive lowering `lower_struct_derives:3899` / `emit_struct:4108` / `emit_enum:3982` | inline recognizers (`mvp_prelude_trait_shape`, width-cast, `Narrow`, per-derive arms) | every derive/trait-shape leaf |

The item-level (`dispatch_item`, `transpile.rs:404`) and expr-level (`walk_expr`) dispatches are **already
one canonical match each** and are **not** the collision surface — a new *item kind* is already one
function, a new *expr variant* is one `ExprVisitor` method. The collision is the four sub-axes above.
**The interface job is to lift each sub-axis's inline recognizer into a static handler table whose rows
live in per-construct files.**

---

## §2 The emit hook-dispatch interface — alternatives, evaluation, recommendation

### Objective function (the criteria table)

| Criterion | Weight | Why it matters here |
|---|---|---|
| **Additivity** | **critical** | A leaf adds a handler *without* editing a shared body — the whole point |
| **Soundness-preservation** | **critical (veto)** | Must NOT regress the landed gates: DN-133 `local_mangled` ordering, DN-134 `struct_layouts` collision-safety, DN-128 derive **per-derive atomicity** (per-impl all-or-nothing over fields; per-derive independence across the set, item still emits), DN-132 pattern arity/resolvability |
| **Never-silent (G2)** | **critical (veto)** | A table miss must fall through to the existing explicit `GapReason`, never a silent drop |
| **KC-3 / KISS** | high | Smallest contained mechanism; no new kernel/framework ceremony |
| **Testability** | high | The landed `cases()` corpus + differential/conformance harnesses must pass unchanged |
| **Merge-collision surface** | high | The residual shared line(s) per leaf must be trivially reconcilable |

### Alt A — `ConstructEmitter` trait + a registry of trait objects

A `trait ConstructEmitter { fn recognizes(&self, …) -> bool; fn emit(&self, …, ctx: &mut EmitCtx) -> Result<String, GapReason>; }`; each construct is a unit struct implementing it, collected into a
`&[&dyn ConstructEmitter]` registry the driver scans.

- **Additivity:** good — a new file adds an `impl` + one registry entry.
- **Soundness:** neutral — sound *iff* the driver threads the single `&mut EmitCtx` and keeps the ordered
  pass (same invariant as Alt B). Trait objects add nothing here.
- **KC-3/KISS:** **weaker** — introduces a trait + dynamic dispatch + object lifetimes for zero capability
  the data-table lacks; the handlers are pure functions of `(node, &mut ctx)`, so the `self`-object is
  dead weight. YAGNI flags the framework.
- **Testability:** equal.
- **Verdict:** works, but heavier than the problem. **Rank 2.**

### Alt B — Static per-axis handler table (generalize `prim_map::TABLE`) — **RECOMMENDED**

Each sub-axis gets a `pub const TABLE: &[Handler]` where a `Handler` is a **plain data row** pairing a
**recognizer** with a **handler fn pointer** — exactly the shape `prim_map::PrimMapping` (`prim_map.rs:88`)
already ships. The driver method (unchanged in structure) does the same **first-match-wins linear scan**
`visit_method_call` already does over `prim_map` (`emit.rs:2253` → `prim_map::lookup`), threading the
single `&mut EmitCtx`. Each row's handler fn lives in its **own submodule file**:

```
emit/
  patterns/    or_pat.rs   tuple_pat.rs   struct_variant_pat.rs   …   (table in patterns/mod.rs)
  derives/     eq.rs  ord.rs  hash.rs  clone.rs  show.rs  init.rs   …  (table in derives/mod.rs)
  calls/       bare.rs  qualified_assoc.rs  cross_nodule.rs         …  (table in calls/mod.rs)
  method_prims/  (this IS prim_map.rs today — already the template)
```

A `Handler` row (pattern axis shown; the others mirror it):

```
pub struct PatternHandler {
    /// Pure recognizer — does this row own this `Pat` shape? No emission, no ctx mutation.
    pub recognizes: fn(&syn::Pat) -> bool,
    /// The lowering. Threads the SINGLE &mut EmitCtx; returns the native form or an explicit gap.
    pub emit: fn(&syn::Pat, self_ty: Option<&str>, ctx: &mut EmitCtx) -> Result<String, GapReason>,
    pub slug: &'static str,        // for EXPLAIN / diagnostics (G2)
    pub citation: &'static str,    // the DN/M-id grounding this row (VR-5)
}
pub const TABLE: &[PatternHandler] = &[ or_pat::ROW, tuple_pat::ROW, struct_variant_pat::ROW, … ];
```

- **Additivity:** **best** — a leaf adds **one file** (disjoint) + **one row** (`…::ROW`) to a `const
  TABLE` array. The array is the *only* shared line, and it is **append-only, one token per row** —
  trivially reconcilable (the merge conflict class of adding an enum variant, not of editing a function
  body). N leaves adding N rows conflict only on that one line, resolved by concatenation.
- **Soundness:** **preserved by construction** *given the §3 invariant* — the table is a **compile-time
  static lookup**; the driver still owns the ordered pass and the `&mut EmitCtx`. The recognizer is
  **pure** (no ctx, no emission), so it cannot perturb ordering; only the driver's scan order (source
  order of rows, first-match-wins) matters and is deterministic.
- **G2:** a scan that matches no row falls through to the existing `fallback`/`GapReason` — identical to
  `visit_method_call`'s current fall-through past the `prim_map` scan (`emit.rs:2253`, `prim_map::lookup`
  at `2270`; the conversion/desugar arms and generic fall-through follow it).
- **KC-3/KISS:** **best** — it is *literally the pattern already in the tree* (`prim_map`), so it adds no
  new concept; a reviewer who understands `prim_map` understands every axis.
- **Testability:** the `cases()` corpus (`src/tests/emit.rs`, 89 `Case` entries at `c044452d`) and the differential/conformance
  harnesses pass unchanged — the emitted text per case is invariant (§3).
- **Verdict:** **Rank 1.** Smallest, proven, additive, ordering-safe.

### Alt C — Chained separate `impl ExprVisitor` hooks

Each construct is its own `impl ExprVisitor`, composed by a chain-of-responsibility over `walk_expr`.

- **Additivity:** ok, but a `Visitor` is *one impl per full method set*, so "one construct = one impl"
  forces empty stubs for every other method, and composition needs a hand-rolled chain the language does
  not give free.
- **Soundness:** the chain must still thread one ctx and one order — no gain over B, more moving parts.
- **KC-3/KISS:** **weakest** — most ceremony, and it fights the fact that the sub-axes are *inside*
  methods, not new expr variants. **Rank 3.**

### Ranked recommendation

**Alt B ≻ Alt A ≻ Alt C.** Adopt **Alt B (static per-axis handler tables, one file per handler, consulted
by the unchanged single ordered pass)**, because it (1) generalizes an *already-landed, already-reviewed*
pattern (`prim_map`), (2) shrinks the per-leaf collision surface to one append-only array line, and (3)
preserves every landed soundness gate *by leaving the ordered pass and the single `&mut EmitCtx` exactly
where they are*. Alt A is the fallback **iff** a future handler needs richer per-handler state than a fn
pointer (none does today — YAGNI: do not build the trait until a row needs it). The argument *against*
Rank 1 is in §7.

---

## §3 The migration plan — preserves every current test + the resolution/gap invariants

The migration is **mechanical and behavior-preserving**: each inline recognizer arm is *moved*, not
rewritten, into a row+file; the driver method keeps its structure and its ctx threading. The gate: the
`cases()` corpus and the differential harness emit **byte-identical** text before/after (the M-1089/M-1093
differential-witness discipline). Per landed capability:

**1. Patterns (M-823 or-pattern, M-826 tuple-pattern, M-1089/DN-132 struct-variant).** Move each `Pat::*`
arm of `map_pattern_inner` (`emit.rs:3492`) into `emit/patterns/<kind>.rs::ROW`. **The Maranget
usefulness/exhaustiveness pass runs unchanged, in the driver, *after* a handler returns its positional
pattern** — handlers produce a positional `Pattern` fragment; they do **not** own exhaustiveness. DN-132's
arity/`..`-rest and resolvability gates stay in the driver (they are cross-pattern properties, not
per-row). **The struct-variant row consumes the shared `struct_layouts` interface (§4.1), never its own
layout map.**

**2. Derive (M-1086/DN-128 std-derive + M-812 facility).** Move each derive lowering into
`emit/derives/<d>.rs::ROW`. What is landed at `c044452d`: `derive(Debug)` → `impl Show`
(`derive_show_impl:3809`), `derive(Default)` → `impl Init` (`derive_init_impl:3861`), and `Clone`/`Copy`
as satisfied value-semantics no-ops (a `DeriveSatisfied` sub-gap); `Eq`/`Ord`/`Hash` are **not yet
recognized** (collected as a sub-gap). **Critical invariant preserved — corrected to the actual *two-level*
guarantee (the strict-gate finding; an earlier draft mis-stated this as item-level all-or-nothing, which
the code does NOT do):**
- **(i) Per-derived-impl atomicity lives in the *rule*.** `derive_show_impl`/`derive_init_impl` refuse the
  **whole** impl the moment any field is ineligible (`return Err`, `emit.rs:3809`/`3861`, gated by
  `field_derive_eligible:3777`) — never a partial single impl. This is the real "never-partial" guarantee,
  and it is per-impl, not per-item.
- **(ii) Per-derive independence across the set lives in the *driver*.** `lower_struct_derives`
  (`emit.rs:3899`) walks the derive set and, for each derive, pushes its composable impl (`Ok`) **or** a
  sub-gap (`Err`); `emit_struct` (`emit.rs:4212/4222`) then appends the composable impls, does
  `sub_gaps.extend(derive_gaps)`, and **still returns `Ok(Emitted{…})`**. So `#[derive(Debug, Ord)]` with
  `Ord` unrecognized emits the struct + the Debug impl + an `Ord` sub-gap — **the item does NOT gap.**

**The interface must preserve BOTH levels:** each derive row carries its own per-impl field-atomicity
(a rule stays all-or-nothing over *its* fields), and the driver keeps the per-derive-in-set orchestration
(compose the eligible derives, sub-gap the rest, item still emits). **A leaf may add a derive row but may
NOT change either level** — neither collapse a rule's per-field atomicity into a partial impl, nor move the
set-orchestration out of `lower_struct_derives` into a row — a build-blocking review check (§8).

**3. Qualified-call (DN-133/M-1094) — the ordering-sensitive one.** `visit_call` (`emit.rs:2150`) resolves
a `Type::method(...)` site against `EmitCtx.local_mangled` — the set of `{Type}__{method}` mangled decls
emitted **so far in this file's single left-to-right item pass** (`emit.rs:174–196`), plus the
cross-nodule `imported_type_keys` table. **The call-shape *recognition* (bare vs qualified-assoc vs
cross-nodule) moves to `emit/calls/*::ROW`; the *resolution against `local_mangled`* stays in the driver
with the threaded `&mut EmitCtx`.** A row is a pure "given the resolved target name, emit the call" step;
the ordered population of `local_mangled` (via `record_local_mangled_assoc_fn`, `emit.rs:315`, from
`emit_impl`'s success path) is untouched. This is the crux the §3 invariant protects — see §7.

**4. Combinator (DN-135/M-1092).** **Already additive** — a Result/Option combinator over a closure lowers
via a method-prim row; `prim_map::TABLE` (`prim_map.rs:140`) is the landed table. A new combinator = a new
row today. The migration only *documents* this axis as the template the other three copy; it changes no
combinator behavior.

**Invariant witnesses the differential must show green (VR-5):**
- **Ordering:** a file with a forward-referenced qualified call (`f()` calling `T::g()` whose `impl`
  appears later) gaps identically before/after — because the pass order and `local_mangled` population are
  unchanged.
- **Collision-safety:** the `struct A{..}` + `enum E{ A{..} }` case (DN-134) refuses identically.
- **Derive:** a mixed derive set (e.g. `#[derive(Debug, Ord)]`, `Ord` unrecognized) emits the composable
  derives and sub-gaps the rest byte-identically — **the item still emits**; and a `derive(Debug)` on a
  struct with one ineligible field refuses the **whole Debug impl** identically (per-impl field-atomicity).

---

## §4 The other shared interfaces — stable-vs-needs-work assessment

### §4.1 `struct_layouts` population — **NEEDS AN INTERFACE (design it here as the shared contract)**

`struct_layouts` (`transpile.rs:332`) is still **`Item::Struct`-only, bare-name-keyed `HashMap<String,
Vec<Option<String>>>`**. Both the **pattern-completion** arm (M-1089/DN-132) and the **construction** arm
(M-1093/DN-134) must extend it to enum struct-variants — the *same* population change. DN-134 §3 step 1
already ratified a **hard collision-safety mandate**: the moment enum variants join a bare-name-keyed map,
`struct A{foo,bar}` + `enum E{ A{foo} }` **silently binds `E::A{foo}` against the unrelated `A` layout** —
a G2 wrong-index violation.

**Design it as THE shared read-only interface both arms (and every future variant leaf) consume**, with
DN-134's collision-safe discipline baked in — *not* re-derived per leaf:

```
/// The one shared layout interface. Populated once (transpile.rs), read-only to every emit handler.
/// Collision-safe by construction (DN-134 §3 step 1): a name that would shadow is refused, never bound.
pub struct StructLayouts { /* … */ }
impl StructLayouts {
    /// Populate from Item::Struct AND Item::Enum Fields::Named variants, collision-safe.
    pub fn build(items: &[Item]) -> Self;
    /// Resolve a ctor name to its field layout, or None (→ the existing explicit gap). Never a
    /// silently-shadowing hit.
    pub fn resolve(&self, ctor_name: &str) -> Option<&StructLayout>;
}
```

Adopt DN-134's **recommended default (b): never-silent refusal of ambiguity** — on population, a
variant-ctor name that collides with a struct name or another variant marks that name **ambiguous**, so
`resolve` **gaps** rather than binding (KISS: no resolution-side qualifier threading). Escalate to DN-134
**(a) qualified-identity keying** (`Enum::Variant`) **only if** a real port target needs same-name
struct/variant construction to *emit*. **Whichever residual-tail leaf lands the population owns this
guarantee; this note freezes the *contract* (collision-safe `resolve`) so pattern and construction leaves
build against it in parallel instead of racing the map's shape.**

### §4.2 `map.rs` `MapTypeVisitor` type-mapping — **NEEDS A LIGHT TYPE-MAP TABLE**

`walk_type`/`TypeVisitor` (`visit.rs`) already centralizes the *`syn::Type`-variant* dispatch, but the
**type-name → Mycelium-type mapping** is an inline body inside `MapTypeVisitor::visit_path` (`map.rs:162`;
the `MapTypeVisitor` struct at `map.rs:148`).
The inventory queues several type-vocab additions (signed ints, `usize`/`isize`, `char` — rows 8/9), each
of which would edit `visit_path` → a collision seam. **Recommend a small `&[TypeMapRow]` table** (same
`prim_map` shape) consulted by `visit_path`, one row per mapped type in its own file. Low-risk,
mechanical; do it in the Phase-1 interface swarm (§5) since it parallelizes cleanly with the emit-hook
work (disjoint file: `map.rs`).

### §4.3 `gap.rs` `Category` taxonomy — **ADDITIVE-AS-IS (stable)**

`Category` (`gap.rs:17`) is a plain enum; adding a class is a **3-line append** (the variant + its
`as_str` + its classification arm in `excluded_from_denominator`/`category_counts`). This is enum-growth,
not a serializing body-edit — the collision class of `struct_layouts` does not apply. **No interface
needed.** (Minor: the `as_str` match is a shared line; append-only, trivially reconciled — leaves may add
a `Category` variant directly, unlike the emit bodies.)

### §4.4 `symtab.rs` resolution API (M-1084) — **STABLE / ADDITIVE (with a named follow-up)**

`SymbolTable::resolve` (`symtab.rs:273`) / `candidate_lookup_keys` (`symtab.rs:330`) / `use_candidates`
(`symtab.rs:121`) landed with M-1084 and
are a clean, narrow API consumed by `dispatch_use` and `imported_type_keys` (`transpile.rs:364`). **Stable
enough to build against as-is.** The **per-method-granularity follow-up is real and named**: today
`imported_type_keys` carries the batch siblings' *emitted item names*, **not yet each mangled per-method
name** (`emit.rs:193` comment), so a genuinely cross-file `Type::method` resolves only when the sibling
*item* name is visible. That is a DN-133 tier-(ii) residual, **not** a Phase-1 interface blocker — record
it as an OQ, do not fold it into the freeze.

### §4.5 Prelude-trait registry (Show/Init/Fault seeds) — **ADDITIVE-AS-IS (stable at the emit layer)**

The DN-127/DN-128/DN-129 prelude traits are **prelude-seeded at the checker/prelude layer** — only the
*trait itself* is seeded; no cross-nodule ambient resolution is invented for their instances (an unseeded
`impl Show[T]`/`impl Init[T]` for a primitive field fails never-silently — `derive_show_impl:3809` /
`derive_init_impl:3861` gap exactly on that missing ambient instance, per DN-127 §7/OQ-1 and DN-129 §5).
Adding a prelude trait is
a **small append to the seed set**, not an `emit.rs`-dispatch collision. **No interface needed** at the
emit layer; the seed set is the shared line (append-only). If the seed set grows large, it can later take
the same `&[Seed]` table shape — YAGNI: not now.

**Summary:**

| Interface | Verdict | Phase-1 action |
|---|---|---|
| `emit.rs` sub-axis dispatch | **NEEDS** (the priority) | Alt B handler tables (§2/§3) — serial, one leaf |
| `struct_layouts` | **NEEDS** | freeze the collision-safe `resolve` contract (§4.1) |
| `map.rs` type-map | **NEEDS (light)** | small `&[TypeMapRow]` table (§4.2) — parallel |
| `gap.rs Category` | additive-as-is | none |
| `symtab.rs` (M-1084) | stable | none (record the per-method OQ) |
| prelude-trait seeds | additive-as-is | none |

---

## §5 Phase-1 build decomposition + parallelism

The interface build is itself a small swarm. **One axis is serial + soundness-critical; the rest
parallelize.**

| # | Interface leaf | Owns (disjoint) | Parallel? | Depends on |
|---|---|---|---|---|
| **P1-a** | **Emit-hook seam** — the Alt B pattern+derive+call-shape handler tables + the §3 invariant, migrating the landed rows behind a green differential | `emit/` submodule tree + the driver methods | **SERIAL — one careful leaf** (touches the shared driver; must preserve DN-133 ordering + DN-128 gate) | the residual-tail (M-1092/1093/1094) must land first — do **not** refactor `emit.rs` while it is under edit |
| **P1-b** | **`struct_layouts` collision-safe interface** (§4.1) | `transpile.rs::struct_layouts` + a `StructLayouts` type | Coordinated with M-1089/M-1093 (whoever lands the population owns it); the *contract* freezes here | DN-134 (ratified) |
| **P1-c** | **`map.rs` type-map table** (§4.2) | `map.rs` (disjoint file) | **Parallel** with P1-a | — |
| **P1-d** | **`Category` + docs** any new gap-class variants needed by Phase-2 | `gap.rs` (append-only) | **Parallel** | — |

**Sequencing.** P1-b/c/d run in parallel now (disjoint files). **P1-a waits for the residual-tail
(M-1092/1093/1094) to merge** — refactoring `emit.rs` concurrently with those leaves would reintroduce the
exact serialization this note removes (and violate mit #11's disjoint-ownership). Once the tail lands,
P1-a is one focused leaf whose DoD is "byte-identical differential + the three §3 invariant witnesses
green." **The emit-hook refactor is deliberately the last Phase-1 step, not the first** — freeze the
data-only interfaces (b/c/d) first so P1-a lands against a stable base.

---

## §6 Phase-2 bulk-drive map — the additive parallel swarm

With the interfaces frozen, each remaining gap class becomes an **additive leaf**: a new handler file +
one append-only table row, editing **no** shared body. Mapping the inventory's gap ranking (§2/§3 of
`language-completeness-gap-inventory.md`) onto the frozen axes:

| Gap class (inventory) | Frozen interface it lands against | Additive leaf shape | Parallel-safe? |
|---|---|---|---|
| **DeriveAttr** (~11–12%) | `emit/derives/*` table | one file + row per derive rule | **yes** (N derive leaves) |
| **Impl / method-body** (`&mut self`, `write!`) | `emit/calls/*` + method-prim table (+ the design-gated `&mut self` DN) | rows once the mapping DN ratifies | yes (per-shape) |
| **Struct / records** (~6% + NamedFieldDrop) | `struct_layouts` contract + `emit/patterns/struct_variant` + `visit_struct` construction | consumes the frozen layout interface | **yes** (pattern & construction independent) |
| **Import** (~14%, mostly measurement) | `symtab.rs` (stable) + the per-method OQ | resolution rows | yes |
| **GenericBound** (~5–6%) | (design-gated: M-876 bounded-`Var`) then emit rows | rows post-design | yes |
| **ReservedWord** (~5%) | prelude-seed / `reserved.rs` (additive) | seed/rename rows | yes |
| **Other / type-coverage** (~23%) | `map.rs` type-map table + `emit/calls` | type-map rows + call rows | **yes** (the biggest additive win) |
| **ModuleDecl** (~4%) | runtime (M-982/M-1024) — out of the emit-hook scope | — | n/a |

**Parallelism unlocked (estimate, `Declared`).** **Today:** effectively **1 emit leaf at a time** — every
pattern/derive/call/combinator leaf serializes on the shared `map_pattern_inner`/`emit_impl`/`visit_call`
bodies (the M-823→M-826→M-1089→M-1092→M-1094 residual-tail *is* that serialization). **Post-freeze:** the
emit collision surface per leaf shrinks to **one append-only array line + one disjoint file**, so
concurrency is bounded by the append-merge of the four `TABLE` arrays, not by body edits. Realistically
that unlocks **≈6–10 concurrent emit leaves** — roughly one handler-family per Phase-2 gap class (derives,
patterns, call-shapes, type-maps, combinators, records-construction), each a disjoint file with a one-line
table registration. The array-line reconciliation is the `issues.yaml`-append class of merge (mit #2), but
*simpler*: one token per row, no semantic dedup, so an octopus merge of N handler leaves is conflict-free
after a trivial array concatenation by the integrating parent. **This figure is `Declared` (a structural
estimate), not measured — it is validated when the first Phase-2 wave runs N handler leaves concurrently
and the octopus merge is conflict-free.**

---

## §7 Adversarial stress-test (house rule #4 / VR-5) — does hook-ification reintroduce an ordering/serialization hazard?

**The sharpest attack (the one the task names): the DN-133 `local_mangled` observed-emission ordering.**
Qualified-call resolution is *order-dependent* — `T::g()` resolves to a local mangled decl **only if an
earlier item in the single left-to-right pass already emitted `T__g`** (`emit.rs:174–196,315–324`). A naïve
reading of "hook registry + parallel additive leaves" suggests **runtime** concurrency or handler
**reordering**, either of which would **break** this: if handlers ran out of source order, or each held
its own context, a forward-reference call that today gaps could spuriously resolve (or vice-versa) — a
silent behavior change, a G2 violation.

**Verdict: the hazard is real, and Alt B avoids it *iff* the §3 ordered-pass-preservation invariant is a
ratified, build-blocking contract — which this note makes it.** The resolution is a distinction the design
must state explicitly and the review must enforce:

- **The parallelism is development-time, never emission-time.** Leaves author disjoint handler *files* in
  parallel; the *emitter still runs one deterministic left-to-right pass* over items/exprs, threading the
  *single* `&mut EmitCtx`. The table changes **who** handles a construct (a compile-time static lookup),
  never **when** or **in what order** the pass visits constructs, and never how many contexts exist.
- **Recognizers are pure** (`fn(&node) -> bool`, no ctx, no emission) — so a row **cannot** perturb the
  pass order or the `local_mangled` population; only the driver mutates ctx, exactly as today.
- **`local_mangled` population stays in the driver** (`record_local_mangled_assoc_fn` from `emit_impl`'s
  success path) — a call-shape *row* is a pure "emit given the resolved name" step; it never owns the
  ordered set.

So the honest answer is **conditional, not unqualified**: Alt B preserves DN-133 soundness **only under
the invariant**, and a design that hook-ified the dispatch into a runtime-reordering or per-handler-context
model **would** reintroduce the serialization/ordering hazard. That is precisely why the invariant is
elevated to a build-blocking DoD item (§8), not left implicit — and why Alt B (a *data table over the
existing pass*) is ranked above any framework that tempts a handler to carry its own execution model.

**Two more stress cases, both survived under the invariant:**
- **Derive two-level atomicity (DN-128) — corrected model.** Two distinct hazards, both refused. (a) If a
  *rule* dropped its per-field atomicity, it could emit a *partial* impl (some fields rendered, one skipped)
  — refused: each rule stays all-or-nothing over its own fields (`derive_show_impl:3809`/`derive_init_impl:3861`),
  a build-blocking review check. (b) If the *set orchestration* migrated into a row, a row could gap the
  whole item (or conversely emit past a sibling's sub-gap) — refused: `lower_struct_derives` (`3899`) +
  `emit_struct` (`4212/4222`) keep the compose-eligible-sub-gap-the-rest-item-still-emits behavior in the
  driver; a row cannot change it. Note this is **not** an item-level all-or-nothing gate (the earlier draft's
  error, corrected) — the honest guarantee is per-impl-atomic + per-derive-independent.
- **`struct_layouts` collision (DN-134).** If pattern and construction leaves each built their own layout
  map, one could omit the collision-safety discipline. Refused: §4.1 freezes **one shared `resolve`
  contract** with DN-134 (b) baked in; both arms consume it, neither re-derives it.

**Residual honest caveat (VR-5).** The differential-witness (byte-identical emission before/after) is the
*empirical* proof the migration preserved behavior; until it runs green, every claim here is **`Declared`**.
The invariant is a *design contract*, not a checked theorem — there is no `Proven` claim in this note.

---

## §8 Definition of Done (the ratification gate — for the maintainer, not self-applied)

**This note reaches Accepted only when the maintainer (or the strict DN-review gate) confirms:**
1. **Alt B** (static per-axis handler tables, one file per handler, unchanged single ordered pass) is
   ratified as the emit interface, over Alt A/C, on the §2 objective table.
2. **The ordered-pass-preservation invariant (§3/§7) is ratified as a build-blocking contract** — the
   emit-hook leaf's review MUST reject any change that (a) reorders handler invocation, (b) introduces
   emission-time concurrency, (c) gives a handler its own `EmitCtx`, or (d) moves `local_mangled`
   population out of the driver, or (e) move the per-derive **set-orchestration** (`lower_struct_derives`)
   out of the driver into a row, or collapse a derive rule's per-impl field-atomicity into a partial impl.
3. **The §4 per-interface verdicts** are accepted (`struct_layouts` + `map.rs` type-map need interfaces;
   `Category`/`symtab`/prelude-seeds are additive-as-is), and the `struct_layouts` collision-safe
   `resolve` contract (§4.1, DN-134 (b) default) is frozen.

**Then, the build DoD (per interface leaf, VR-5):**
- **P1-a** lands only with a **byte-identical differential** over the `cases()` corpus + the M-1089/M-1093
  witnesses, and **the three §7 invariant witnesses green** (forward-ref qualified-call ordering,
  struct/variant collision refusal, and the derive witness: a mixed derive set emits the composable derives
  and sub-gaps the rest byte-identically — item still emits — while a single-ineligible-field Debug refuses
  the whole Debug impl identically). Guarantee tag upgrades `Declared →
  Empirical` **only** on that green differential.
- **P1-b/c/d** land with `just check`-green change-scoped tests; each adds a row/variant, no shared-body
  edit.
- **Phase-2 validation:** the parallelism estimate (§6) is confirmed `Empirical` when the first wave runs
  ≥3 handler leaves concurrently and the octopus merge is conflict-free after array concatenation.
- No claim is tagged `Proven` (no checked theorem); the tracker is `Declared`, the codebase is ground
  truth (mit #14).

---

## §9 FLAGs (owned elsewhere — this note edits none of them)

`Doc-Index.md`, `CHANGELOG.md`, and `issues.yaml` are integration-owned (concurrent-PR pattern: leaves
FLAG, the integrating parent applies once). **FLAG to the integrating parent:**

- **FLAG-1 (Doc-Index):** add a Design-Notes row — `DN-136 — The Emit Hook-Dispatch Interface
  (Interfaces-First Refactor for the Bulk Gap-Close Drive) (Draft, 2026-07-12)`.
- **FLAG-2 (CHANGELOG):** append-only `[Unreleased]` `docs(dn)` entry for DN-136 (dated 2026-07-12).
- **FLAG-3 (issues.yaml — M-ids to mint; verify free at filing, mit #1):**
  - **P1-a emit-hook seam** — a build M-id, `depends_on: [M-1092, M-1093, M-1094]` (the residual-tail must
    land first). **Highest existing id is M-1091** (at both `1bc7956b` and `c044452d`); M-1092/1093/1094 are referenced by
    DN-133/134/135 + landed code but **their issue rows are not yet filed** in `issues.yaml` at this tip —
    confirm/file those too.
  - **P1-b `struct_layouts` collision-safe interface** — coordinate with M-1089/M-1093 (shared
    population); this may be a note on those, not a new id.
  - **P1-c `map.rs` type-map table** and **P1-d `Category` growth** — small build M-ids.
- **FLAG-4 (cross-refs):** add `corpus:DN-136` pointers from `language-completeness-gap-inventory.md` §4
  (the design-first vs build-now split gains this interface row) and from DN-133/134/135 (the residual-tail
  whose gates this interface freezes). Add `doc_refs` `src:` anchors (all at `c044452d`):
  `emit.rs:1803,2150,2253,2270,3492,3809,3861,3899,4108,4212,4601`, `visit.rs:50,126`,
  `prim_map.rs:88,140,221`, `transpile.rs:332,404`, `gap.rs:17`, `map.rs:148,162`, `symtab.rs:121,273,330`.
- **FLAG-5 (shared-checkout hygiene — coordination note, not a doc edit).** During this session's
  verify-first, an early `git reset --hard origin/dev` was inadvertently issued in the **shared main
  checkout** (`/root/git/isolated/mycelium`, on protected `dev`) instead of this worktree; it moved the
  **local** `dev` pointer to the fetch-time `origin/dev` (`1bc7956b`). No push occurred (the remote `dev`
  is untouched and canonical, now at `c044452d`), the tree stayed clean, and a follow-up re-align was
  correctly **blocked** by the sandbox (mit #15 — not retried/circumvented). The orchestrator should
  `git -C <shared> fetch && git -C <shared> reset --hard origin/dev` (or fast-forward) to re-align the
  local `dev` pointer at its convenience. This worktree is now based on current dev `c044452d` (the
  original `1bc7956b` pin merged forward at the post-gate amendment).

## §10 Changelog

- **2026-07-12** — **Draft → Accepted (ratified, strict 9-criterion DN-review gate, delegated ratification).**
  Re-ran the full gate against the code at `c044452d` on the patched note (`f425e4da`). **Clean pass:**
  every previously-failing item is fixed and verified against source — (1 · grounding) all refreshed
  anchors resolve exactly (`derive_show_impl:3809`, `derive_init_impl:3861`, `field_derive_eligible:3777`
  each `return Err` on an ineligible field; `lower_struct_derives:3899`; `emit_struct:4212/4222` does
  `sub_gaps.extend(derive_gaps)` then **still** `Ok(Emitted{…})`; `map_pattern_inner:3492`,
  `emit_impl:4601`, `emit_enum:3982`, `visit_call:2150`, `visit_method_call:2253`, `prim_map::lookup:2270`,
  `map.rs:148/162`, `symtab.rs:121/273/330`, `transpile.rs:332/404`, `gap.rs:17`, `prim_map.rs:88/140/221`,
  `visit.rs:50/126`); (7b · adversarial-derive) the two-level guarantee now matches the code (per-impl
  field-atomicity in the rule + per-derive independence in the driver, item still emits) — no item-level
  all-or-nothing claim remains; (8 · DoD) the derive witness is now differential-consistent (asserts the
  *current* behavior, so byte-identical before/after is satisfiable); (9 · consistency) the derive model is
  stated uniformly across §0/§2/§3/§7/§8. Previously-passing criteria (2 VR-5 · 3 G2 · 4 append-only ·
  5/6 KC-3 · 7a ordered-pass · 7c `struct_layouts` collision) re-confirmed unaffected. **One residual nit
  corrected in this ratification commit:** the descriptive `cases()` count read `88`; the live count at
  `c044452d` is **89** `Case` literals (three independent counts agree; maintainer confirmed the
  residual-tail fix added one case) — updated §2 and the amendment changelog line. Ratification is the
  delegated DN-review gate (maintainer standing automation + explicit task), **not** a self-ratify (house
  rule #3): the note recommends, the independent gate accepts. Accepted authorizes the interfaces-first
  Phase-1 build (§5); the note still builds nothing (`Declared` until each FLAGGED build issue lands +
  differential-witnessed, VR-5). Doc-Index/CHANGELOG/issues rows stay FLAGGED (§9) for the integrating parent.
- **2026-07-12** — **Draft amendment (post strict-gate review).** The gate PASSED the architecture (Alt B
  verified a true `prim_map` generalization; the §7 ordered-pass invariant and the §4.1 `struct_layouts`
  collision-safety held under adversarial stress) but FAILED two items, both fixed here surgically without
  touching the architecture: **(1) the DN-128 derive guarantee was described FALSELY** as item-level
  all-or-nothing ("if any derive in the set gaps, the item gaps"). The code does **not** do that — verified
  against `lower_struct_derives:3899` + `emit_struct:4212/4222` (`sub_gaps.extend(derive_gaps)` then **still**
  `Ok(Emitted)`) and `derive_show_impl:3809`. The honest guarantee is **two-level**: per-derived-impl
  atomicity in the *rule* (whole impl refused on any ineligible field, never partial) + per-derive
  independence in the *driver* (compose the eligible, sub-gap the rest, **item still emits**). Rewrote §0
  table, §2 objective row, §3.2, the §3 witnesses, §7, and §8 DoD to the corrected model; **(2) replaced
  the self-contradictory §8 DoD witness** ("mixed derive gaps the whole item") with the differential-consistent
  witness ("emits the composable derives and sub-gaps the rest byte-identically; single-ineligible-field
  Debug refuses the whole Debug impl"). **Grounding hygiene:** re-pinned from `1bc7956b` to current dev
  `c044452d` (M-1092/DN-135 combinator landed, +~397 lines) and refreshed every anchor — `map.rs:145→162`,
  `symtab.rs:160→121/273/330`, the `prim_map` scan to `visit_method_call:2253`/`lookup:2270` (not the
  `visit_call:2178` misattribution), `cases()` `~161→89` (89 `Case` literals at `c044452d`; an earlier
  amendment said 88 — the residual-tail fix added one case, maintainer-confirmed), and all shifted `emit.rs` anchors
  (`map_pattern_inner:3492`, `emit_impl:4601`, `emit_struct:4108`, `emit_enum:3982`, the derive fns).
  Status stays **Draft** (re-gate pending; no self-ratify — house rule #3).
- **2026-07-12** — initial **Draft**. Designed Phase 1 of the remaining-gaps bulk drive: the emit
  hook-dispatch interface. Verified (mit #1/#13/#14) against `origin/dev@1bc7956b` that the dispatch is
  already half-centralized (`walk_expr`, `prim_map::TABLE`) so the residual collision surface is three
  sub-axes; recommended **Alt B** (static per-axis handler tables generalizing the landed `prim_map`
  pattern, one file per handler, consulted by the unchanged single ordered pass) over a trait-object
  registry (Alt A) and a chained-visitor (Alt C); wrote the migration-preserves-soundness plan for the
  landed pattern/derive/qualified-call/combinator capabilities; assessed the other shared interfaces
  (`struct_layouts` + `map.rs` type-map **need** interfaces, `Category`/`symtab`/prelude-seeds are
  additive-as-is); decomposed Phase-1 (one serial soundness-critical emit-hook leaf + parallel data-only
  interface leaves) and mapped the Phase-2 additive bulk swarm (≈6–10 concurrent emit leaves unlocked,
  `Declared`). Adversarial stress-test: hook-ification **would** reintroduce the DN-133 `local_mangled`
  ordering hazard under a runtime-reordering/parallel-execution model — Alt B avoids it **only** under the
  ratified ordered-pass-preservation invariant (development-time parallelism, emission-time single ordered
  pass, single `&mut EmitCtx`), which this note makes a build-blocking DoD item. Recommends, does not
  ratify (house rule #3); every mechanism `Declared` until built + differential-witnessed (VR-5). FLAGs the
  Doc-Index/CHANGELOG/issues rows up (§9).
