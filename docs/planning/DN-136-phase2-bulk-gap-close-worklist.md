# DN-136 Phase-2 Bulk Gap-Close Worklist — the additive-leaf fan-out plan

| Field | Value |
|---|---|
| **Status** | **Draft — living planning document** (2026-07-13), register style (mirrors `language-completeness-gap-inventory.md`). Updated in place; append rather than overwrite when a figure changes (VR-5). It recommends, does not ratify (house rule #3), and moves no other doc's status. |
| **Task** | Scope DN-136 §Phase-2 (now authorized — Phase-1 interfaces landed at `dev` `b1755fa6`): re-measure the corpus against the frozen `emit/{patterns,derives,calls}`/`type_map::TABLE`/`struct_layouts` interfaces, classify every remaining gap class as BUILD-READY additive / NEEDS-DESIGN / BLOCKED, and produce the parallel Phase-2 leaf worklist + the recommended first wave. |
| **Grounding** | Read against `dev@b1755fa6` (worktree branch `phase2-scope-worklist`, fast-forwarded from `origin/dev` with zero divergent commits). Re-measured by regenerating `gen/myc-drafts/` (`just myc-drafts-regen`) — the identical 13-target corpus (5-file semcore SCC + 12 unported stdlib crates) the DN-124/DN-122 §8.22 phylum-mode baseline and the `language-completeness-gap-inventory.md` §2 draft-corpus numbers used, so this re-measure is **apples-to-apples**, not a different corpus. `generated_from_commit: b1755fa6fa41b1746b9ddc3bd0f5ab7494583e7c` (`gen/myc-drafts/manifest.json`). Every measurement below is `Empirical`; every unbuilt mechanism is `Declared` (VR-5). |
| **Supersedes (as the current synthesis)** | The gap-class distribution and design-status columns of `language-completeness-gap-inventory.md` §2/§3/§4 (dated 2026-07-12, itself already one landing-wave stale — see §4 below) — re-derived here against the current tree (mitigation #14). The inventory doc is left untouched (append-only, house rule #3); it is now historical context for *how* the ranking moved, not the live ranking. |

> **Honesty finding up front (house rule #4 / VR-5).** The single biggest surprise in this
> re-measure is **not** a new gap class — it is that **nine of the gap-inventory's "design-first,
> un-owned" items are now Accepted design notes, several already partly BUILT**, in a same-day
> landing wave (DN-125 through DN-136) that happened *after* the inventory was written. The
> inventory's own §4 "design-first" list is now mostly stale in the *design* dimension (ratified)
> while still accurate in the *build* dimension (unbuilt) for most rows. This worklist re-derives
> the honest current split.

---

## §0 Verify-first (mitigation #14) — what actually changed since the inventory

| Design item | Inventory's framing (2026-07-12) | **Current state (verified against `b1755fa6` source, 2026-07-13)** |
|---|---|---|
| `&mut self`/`&mut T` mutation | "un-owned, top design-first item" | **DN-125 Accepted + BUILT** (`M-1081 status:done`, PR #1527/#1530) — value-threading + call-site rebind landed in `emit.rs`/`map.rs`, 14/14 `tests::mut_thread` green |
| `write!`/`format!` → Display | "design-gated, no Display prim" | **DN-127 Accepted; WU-1/WU-2 BUILT** (`lib/std/fmt.myc` `to_dec`, `Show` prelude seed) — **WU-3 (the transpiler lowering rule) is the only unbuilt residual**, tracked `M-1090 status:todo`, `depends_on: [M-1081]` (now satisfied) |
| Standard-derive library | "facility landed, library unbuilt" | **DN-128 Accepted** (ratifies field-wise `cmp.eq` fold for Eq/PartialEq, `Ord3` fold for Ord, `hash.blake3` fold for Hash, `Clone`=no-op). Show/Init/Clone-Copy rows **landed** (DN-136/P1-a); **Eq/Ord/Hash rows are the unbuilt residual**, `M-1086 status:todo` |
| Records | "Draft, needs ratify" | Unchanged this session — DN-123 still Draft (not part of this landing wave) |
| Impl-level generics | "design-gated, own DN needed" | **DN-130 Accepted** (`impl[T] Trait for Foo[T]`, constructor-keyed coherence) — unbuilt, `M-1087 status:todo`, real dependency on `M-1080` (now done) |
| Bounded-generic surface | "design-gated (M-876)" | **DN-131 Accepted** for the non-function (inherent-impl) slot — **kernel/L1 side BUILT** (PR #1529); **only the transpiler emission is unbuilt** (`emit.rs:355-359` still hard-refuses any bounded type param), `M-1088 status:todo` |
| Struct-variant patterns | "small registry-shape design" | **DN-132 Accepted (P1 only)** — producer (`Pat::Struct` in `map_pattern_inner`) **BUILT**; consumer (`struct_layouts` walking `Item::Enum` variants) **ALSO now built**, but by a *different*, *later* issue (M-1093/DN-134's collision-safe interface, PR #1548) — see the stale-status finding below |
| Emit dispatch collision (Phase-1 itself) | "the priority interface" | **DN-136 Accepted + BUILT** — `emit/{patterns,derives,calls}` + `type_map::TABLE` all landed, byte-identical differential green (this is the frozen base this worklist builds against) |

**A second, independent stale-tracker finding (mit #14):** `M-1089`'s own body (filed against
PR #1535) says "the `StructLayout` variant-awareness item is NOT landed" and stays `status:todo`. But
`transpile.rs::struct_layouts` (read directly at `b1755fa6`) **does** walk `Item::Enum` variants
collision-safely — landed later by **M-1093**/DN-134 (PR #1548), a different issue that closed
M-1089's own named residual as a side effect. **M-1089 appears fully closed and its `status:todo`
is stale** — flagged to the integrator (§8) for verification + flip, not flipped here (this doc
does not edit `issues.yaml`).

**Third finding — four landed leaves have no issue row at all (mit #1/#14).** `M-1092` (DN-135
combinator, merged `c044452d`), `M-1093` (DN-134 struct-variant construction + collision-safe
`struct_layouts`, merged `a4318e53`), `M-1094` (DN-133 qualified-call, merged `eb1b7625`), and the
DN-136 Phase-1 build itself (P1-a `f270eeb8`/`b1755fa6`, P1-c `6ad15b8a`/`7d3724cc`) are all **landed
code with no corresponding `issues.yaml` row** — `git log --grep` confirms every merge, but
`grep 'id: M-109[2-9]'` finds nothing past `M-1091`. FLAGged for minting as `status:done` rows
(§8) rather than silently left untracked.

---

## §1 The re-measured baseline (`Empirical`, `b1755fa6`, identical 13-target corpus)

Aggregated from `gen/myc-drafts/manifest.json` (13 targets: the 5-file semcore SCC + 12 unported
stdlib crates — the same corpus `06b4d7a7`'s phylum baseline and the inventory's `eb6bc0e2` numbers
used):

| Metric | Value | vs. prior same-corpus baseline |
|---|---|---|
| `non_test_items` (denominator) | 773 | 768 (13-target run, `eb6bc0e2`) — corpus drifted +5 items as source changed under it |
| `checked_fraction` (oracle) | **6.34%** (49/773) | 6.64% (51/768) — **flat to slightly down**, within noise |
| `checked_fraction_phylum` | **6.60%** (51/773) | ~7.4% (DN-122 §8.22 WU-A baseline, `06b4d7a7`, a *different* corpus revision) |
| `expressible_fraction` | 19.0% (147/773) | — |
| Δ_basis (phylum − oracle, this run) | **+0.26pp** (2 items) | — |

**Honest, non-sycophantic reading (house rule #4):** `checked_fraction` did **not** move from
Phase-1. This is **expected, not a regression** — DN-136 §3 explicitly designed P1-a as a
byte-identical migration (the differential witness gates on unchanged emission). Phase-1 bought
*future* leaf-parallelism, not present coverage; Phase-2 is where coverage is supposed to move.

**A second correction to the DN-124/inventory narrative:** on *this* corpus, the phylum-mode basis
correction is tiny (+0.26pp), not the ~1pp implied by the `06b4d7a7`-vs-oracle headline. Looking at
the per-target `phylum.ran`/`phylum.ok` fields explains why: **8 of 13 targets report
`phylum ok: false`** (semcore, std-content, std-fs, std-io, std-rand, std-runtime, std-sys,
std-time) — `myc check --phylum` runs but the **whole batch refuses** (e.g. semcore's
`"unknown type Env"`), which credits **0** to the phylum numerator for that entire target exactly
like oracle-mode's file-gating does. So on this corpus, "Import is mostly a measurement artifact"
(DN-124's claim) is **not yet demonstrated** — most targets can't even reach the point where
phylum-mode's extra visibility would matter. This is a distinct, more fundamental blocker
(whole-phylum assembly failure) than the emit-side `Import` gap count, and is **not itself a
Phase-2 additive-leaf candidate** (it is diagnostic/tooling work, tracked loosely under
DN-124/M-1024) — flagged, not sized, here.

---

## §2 Current gap-class ranking

`Empirical`, 959 real gap instances (`real_gap_count` basis — excludes the one `DeriveSatisfied`
non-gap advisory; `TestItem`/`ModuleDecl` are tallied but **excluded from the coverage denominator**,
see the note below the table).

| Rank | Class | Count | Share | vs. inventory §2 (2026-07-12, different-but-comparable corpus) |
|---|---:|---:|---:|---|
| 1 | **Other** (type-coverage + misc) | 203 | 21.2% | was ~23–24% — flat/slightly down |
| 2 | **DeriveAttr** | 139 | 14.5% | was ~11–12% — **up**, now the #2 class (not #4) |
| 3 | **Import** | 130 | 13.6% | was ~14% — flat |
| 4 | **Impl** | 111 | 11.6% | was ~12–13% — flat/slightly down |
| 5 | **Struct** | 57 | 5.9% | was ~6% — flat |
| 6 | **GenericBound** | 54 | 5.6% | was ~5–6% — flat |
| 7 | **ReservedWord** | 53 | 5.5% | was ~5% — flat |
| 8 | **NamedFieldDrop** | 52 | 5.4% | was ~5% — flat |
| 9 | **MultiStmtBody** | 47 | 4.9% | not separately ranked before (new visibility) |
| 10 | **ModuleDecl*** | 39 | 4.1% | was ~4% — flat (*excluded from denominator*) |
| 11 | **TestItem*** | 33 | 3.4% | not separately ranked before (*excluded from denominator*) |
| 12 | InnerAttr | 11 | 1.1% | — |
| 13 | Closure | 9 | 0.9% | — |
| 13 | PayloadVariant | 9 | 0.9% | — |
| 15 | MacroInvocation | 6 | 0.6% | — |
| 16 | Trait | 3 | 0.3% | — |
| 16 | AssocConst | 3 | 0.3% | — |

`*` `TestItem`/`ModuleDecl` are `Category::excluded_from_denominator` — they don't count against
`expressible_fraction`/`checked_fraction` at all (they're not translatable library surface); they
are shown here for completeness because `real_gap_count` (the headline total) still tallies them.

**Why DeriveAttr's share grew even though Show/Init/Clone-Copy landed:** those three rows moved
`Debug`/`Default`/`Clone`/`Copy` **off** the DeriveAttr-gap list (into successful emissions or
`DeriveSatisfied` no-ops) — but `Eq`/`Ord`/`PartialEq`/`PartialOrd`/`Hash` (the still-unbuilt
residual, confirmed absent from `emit/derives/mod.rs::TABLE`) remained gapped the whole time, and
the corpus grew 5 items under the same commit range, netting a **higher share** of a **smaller
absolute pool of "Other"-class gaps that fell away as they got reclassified**. This is exactly the
DN-136 §1 prediction (`Other` "shrank ... as DeriveAttr split out") continuing one step further.

---

## §3 Per-class classification (BUILD-READY additive / NEEDS-DESIGN / BLOCKED)

### BUILD-READY additive leaves

Closes via rows on a frozen DN-136 interface, or a small, narrowly-scoped, non-serializing change
with an already-Accepted design.

| # | Gap class / item | Frozen interface | Ratified basis | Est. corpus impact |
|---|---|---|---|---|
| **B1** | **`derive(PartialEq)`/`derive(Eq)`** | `emit/derives/eq.rs` row | DN-128 (Accepted): field-wise `cmp.eq` fold, refuse-whole-impl on an ineligible field (mirrors `show.rs`/`init.rs`'s `field_derive_eligible` gate exactly); a derived total `Eq` over a `Float` field is refused (ADR-040 NaN semantics) — **the fallible case is already the pattern the two landed rows established** | part of the 139 DeriveAttr gaps; `M-1086`'s DN-128-cited "dependency-free, NOT blocked on M-1090/M-1091" derives |
| **B2** | **`derive(Ord)`/`derive(PartialOrd)`** | `emit/derives/ord.rs` row | DN-128: fold over the **already-seeded** `Ord3` prelude trait (`crates/mycelium-l1/src/ord3.rs`, landed M-1080/DN-122) — **no new prelude-trait seed needed**, unlike the task brief's open question suggested; `Ord3.cmp(a,b) -> Binary{8}` is exactly the lexicographic-fold target `derive_show_impl`'s per-field pattern already generalizes to | same pool as B1 |
| **B3** | **`derive(Hash)`** | `emit/derives/hash.rs` row | DN-128: fold over the **already-landed** `hash.blake3` kernel prim (DN-34 §8.17 CU-…/M-912) — no new kernel prim, no new seed | same pool as B1 |
| **B4** | **`write!`/`format!` → `Show`-render lowering** (M-1090 WU-3) | **not yet one of the 4 frozen axes** — closest fit is a new small macro-invocation recognizer feeding the existing `bytes_concat`-fold codegen `show.rs` already emits; needs a short interface note (a `MacroInvocation`-shaped row family), not a fresh design | DN-127 (Accepted); `to_dec`/`Show` **both already landed** (WU-1/WU-2) — this is purely the transpiler lowering rule, the single highest-leverage item on the historical corpus (DN-34 §8.22: 30/114 pure `Impl` gaps were exactly this bucket) | large — was the single biggest pure-Impl bucket pre-Phase-1; current-corpus recount not yet isolated (folds into `Impl`'s 111) |
| **B5** | **Bounded inherent-impl type-param emission** (M-1088 residual) | not a `TABLE` row — a narrow, single-site change to the impl/type-param-list emission path (`emit.rs:355-359`'s hard refusal) | DN-131 (Accepted); kernel/L1 side **fully built** (PR #1529) — only the transpiler's hard-refusal-on-any-bound needs replacing with pass-through emission of the bound | part of the 54 GenericBound gaps |
| **B6** | **Conversion-method mapping** (`ToOwned`/`Clone`/`ToString`/`Into` → identity or real surface) | `emit/calls/*` row (a method-call recognizer, same shape as the landed `bare`/`qualified_assoc` rows) | M-1037 (todo, already scored build-ready in the prior inventory); confirmed live in the current corpus (`.to_owned()` accounts for 4 of the top "Other" reasons alone) | part of `Other`'s 203; `.to_owned()` alone is 4/203 (~2%), the pattern likely recurs across the untriaged tail |
| **B7** | **`M-1089` status reconciliation** (not a build — a tracking fix) | n/a | Both DoD halves of M-1089 (`Pat::Struct` arm + variant-aware `struct_layouts`) verified landed at `b1755fa6` — flip `status:todo → done` | n/a (bookkeeping) |

**Honest caveat on B4 (VR-5):** DN-136's four frozen axes (`patterns`/`derives`/`calls`/`type_map`)
were scoped to the collision surfaces DN-136 §1 named; a `write!`/`format!` macro invocation is
**not** a `syn::ExprCall`/`Pat`/derive-attr/named-type, so it doesn't literally fit any existing
`TABLE`. It is still **additive** in spirit (a new, disjoint file + a new small dispatch point in
`EmitVisitor`'s macro-handling arm, which today is a single unconditional gap) but is **not yet
proven collision-free** the way the four ratified axes are — recommend a **one-paragraph interface
note** (not a full DN) confirming the macro-dispatch point is similarly a single canonical match
(mirroring DN-136 §1's finding for `dispatch_item`/`walk_expr`) before treating it as a same-wave
parallel leaf. If it is, B4 runs in the same wave; if the macro-handling body turns out to be
shared/serializing, it becomes the wave's one serial leaf (mirroring P1-a's own precedent).

### NEEDS-DESIGN (a fresh Draft DN, or an existing Draft DN needs ratifying, before building)

| # | Gap class / item | Why it needs design | Owning DN / M-id |
|---|---|---|---|
| **D1** | **Unit type `()`** | **NEW finding this session** — 26/203 `Other`-class instances (the single largest specific reason in the whole corpus) are `unit type () has no representable value in this grammar fragment`; the grammar (`docs/spec/grammar/mycelium.ebnf`) has **no** Unit/Nil production at all — this is a genuine, un-owned language question (does a `()`-returning fn get an Idiomatic Remapping — omit the return type entirely — or does Mycelium need a real zero-arity unit form?), not a transpiler oversight | **none — drafted here as DN-137** (§5 below) |
| **D2** | **Records / named fields** | Unchanged this session — DN-123 stays Draft, needs ratify | DN-123 |
| **D3** | **Generic trait-instance impls build** (`impl[T] Trait for Foo[T]`) | Design **is** ratified (DN-130); classified here as design-first-turned-build-ready-but-large (parser/coherence/mono/transpiler, a multi-file cluster, not a single-row leaf) — sits between BUILD-READY and its own mini-wave | DN-130; M-1087 (`depends_on: M-1080`, satisfied) |
| **D4** | **Const items** (`const GUARANTEE_MATRIX`, etc.) | No `item`-level const production in the grammar at all (confirmed: 9+ instances in the current "Other" tail); a genuine grammar/L3 gap, not scoped by any existing DN found in this pass | none found — recommend a FLAG to a design-reasoner, not drafted here (lower leverage than D1, and this pass's effort budget is prioritized) |
| **D5** | **`dyn Trait` / `impl Trait` type positions** | No native trait-object or existential-impl answer decided; distinct from DN-130's parametric-impl-family answer (which is a *static* family, not runtime dispatch) | none found — FLAG only |
| **D6** | **Match-arm guards** (`if ...` in `match`) | Already scoped, not orphaned: DN-132 itself names this as its **P2/P3** residual, explicitly `depends_on` M-833/DN-79 | DN-132 (P1 Accepted, P2/P3 open); M-833/DN-79 |

### BLOCKED (depends on an unbuilt capability named elsewhere; not a Phase-2 leaf target)

| # | Gap class / item | Blocked on |
|---|---|---|
| **K1** | **Cross-nodule qualified-call / qualified-type-path resolution beyond the current file** (`mycelium_stack::with_deep_stack`, `io::Error`, etc. — a meaningful slice of the 130 `Import` gaps + several "Other" qualified-path reasons) | The per-method-granularity follow-up DN-136 §4.4 already named as an OQ (today `imported_type_keys` carries sibling **item** names, not per-method mangled names) — real symtab work, not a row |
| **K2** | **Generic-container type mapping** (`BTreeSet<usize>` and siblings) | No native Set/collection type is mapped yet; entangled with the bounded-generics/collections residual (M-876's broader scope), not a standalone row |
| **K3** | **`n as f64` cast fidelity** | Already tracked as `PENDING-DESIGN(CU-3-fidelity)` (DN-34 §8.17 FLAG-cu3-lossy-swap) — a known, named, deferred residual |
| **K4** | **Cross-nodule runtime execution** (`ModuleDecl`'s 39 instances, `Import`'s runtime-execution half) | M-1024 (`in-progress`) — out of the emit-hook/Phase-2 scope entirely (runtime tier, not transpiler) |

---

## §4 The Phase-2 worklist — the parallel batch

### Parallelism estimate — re-validated against the now-concrete leaf set

DN-136 §6 estimated **≈6–10 concurrent emit leaves**, "one handler-family per Phase-2 gap class."
The concrete BUILD-READY set (§3) gives: **B1, B2, B3 (three independent `derives/` rows), B6 (one
`calls/` row), B5 (one narrow non-table site)** — **5 truly disjoint files today**, each a single
append-only `TABLE` line plus its own file, **verified conflict-free by construction** (DN-136 §2's
own argument: mutually exclusive recognizers, one shared array line per axis). **B4** is a 6th
candidate, conditional on the one-paragraph macro-dispatch interface check (§3). **B7** is a
same-wave bookkeeping item (no code, runs in parallel trivially). So the **validated estimate for
this wave is 5–6 concurrent leaves**, at the low end of DN-136's 6–10 `Declared` range — consistent
with it (this is the *first* Phase-2 wave; later waves adding a range-pattern/`@`-binding row, more
type-map rows, D3's generic-impl cluster, etc. would bring later waves up toward the range's high
end). This wave's estimate upgrades DN-136 §6's figure from `Declared` (structural) to `Empirical`
**once** this wave's octopus merge is actually run conflict-free (DN-136 §8's own stated validation
condition) — **not yet run**, still `Declared` here (VR-5).

### The recommended FIRST wave (5 leaves, each independently spec'd + differential-witnessed)

| Leaf | Owns | Spec | Differential witness |
|---|---|---|---|
| **L1 — `derive(PartialEq)`/`derive(Eq)`** | `crates/mycelium-transpile/src/emit/derives/eq.rs` (new file) + one `TABLE` row in `derives/mod.rs` | `recognizes: name == "PartialEq" \|\| name == "Eq"`. `emit`: per-field `cmp.eq` fold (mirror `show::compose`'s `bytes_concat_chain` shape, folding with `and` instead — `p0 == q0 and p1 == q1 and ...`), refuse the whole impl via `field_derive_eligible` (reuse verbatim) **plus** a NaN-refusal check: any field mapped to `Float` gates the entire derive to `Gap` (DN-128's ADR-040 clause), not just ineligible-repr fields. Composes `impl Eq[T] for T` if `Eq` fired, or the equivalent for `PartialEq` (check DN-128 §2 for which of the two, or both, the library targets — recommend emitting under whichever attribute name fired, matching the `Show`/`Debug` precedent of naming the impl after the *matched* derive text) | New `cases()` entries: (a) all-primitive-field struct derives clean; (b) a `Float` field refuses whole; (c) an ineligible-repr field refuses whole (byte-identical shape to the existing Show/Init cases) |
| **L2 — `derive(Ord)`/`derive(PartialOrd)`** | `crates/mycelium-transpile/src/emit/derives/ord.rs` (new file) + one `TABLE` row | `recognizes: name == "Ord" \|\| name == "PartialOrd"`. `emit`: per-field lexicographic fold targeting `Ord3` (`cmp.lt`/`cmp.eq`-style short-circuit chain, or a direct `Ord3.cmp` composition per DN-128 §2's stated fold) → `impl Ord3[T] for T { fn cmp(a: T, b: T) => Binary{8} = ...; }`; same `field_derive_eligible`/Float-NaN refusal as L1 | Analogous `cases()` triple (clean multi-field, Float refusal, ineligible-repr refusal); a lexicographic-order assertion case (field 0 dominates field 1) |
| **L3 — `derive(Hash)`** | `crates/mycelium-transpile/src/emit/derives/hash.rs` (new file) + one `TABLE` row | `recognizes: name == "Hash"`. `emit`: fold field renders (reuse `Show`'s `render`) through `hash.blake3`, or hash the `bytes_concat`-ed field representation directly if `hash.blake3` takes `Bytes` (check `mycelium-core/src/prim.rs`'s exact signature before emitting) — refuse on ineligible field, same gate | `cases()` triple mirroring L1/L2; confirm `hash.blake3`'s actual arity/signature first (a spec-blocking read, not a guess — VR-5) |
| **L4 — Conversion-method mapping** | `crates/mycelium-transpile/src/emit/calls/conversion.rs` (new file) + one `TABLE` row in `calls/mod.rs` | `recognizes`: a `MethodCall` (not `ExprCall` — **check which axis's shape actually matches**; `calls/mod.rs`'s `CallHandler` is typed over `syn::ExprCall`, but `.to_owned()`/`.clone()`/`.to_string()`/`.into()` are `syn::ExprMethodCall` receiver-call shapes, the SAME axis `prim_map::TABLE` (the method-prim table) already owns, not `emit/calls`. **Route this row into `prim_map::TABLE` instead** (M-1037's actual home) — a spec correction from the naive "it's calls-shaped" read to the axis that structurally fits | A `cases()` entry per conversion method: `.to_owned()`/`.clone()` → identity (drop the call, keep the receiver); `.to_string()` → `Show`-render if implemented, else gap; `.into()` → identity when source/target coincide, else gap (never guess a real conversion) |
| **L5 — Bounded inherent-impl type-param emission** | `crates/mycelium-transpile/src/emit.rs:355-359` (the single hard-refusal site) — **not a new file**, a narrow in-place change; low collision risk because it is a single, already-isolated `if bound.is_some() { return Err(...) }`-shaped guard, not a shared dispatch body | Replace the unconditional refusal with: emit the bound if present (`T: Bound` → the same `T: Bound` text in the `.myc` type-param list), matching DN-131's ratified pass-through (no new checker work needed — the L1/kernel side already accepts it) | A `cases()` entry: `impl<T: Clone> Foo<T> { ... }` → `impl[T: Clone] Foo[T] { ... }`, plus the existing unbounded-impl case stays unchanged (regression guard) |

**L6 (conditional — B4, `write!`/`format!`) is deliberately held out of this first wave** pending
the one-paragraph macro-dispatch interface confirmation (§3 caveat) — recommend it as the **second**
wave's lead item once confirmed, since it is the single highest-leverage remaining item.

**B5 note:** on closer read, L5 is *not* one of the four DN-136 axes either (it's a direct
`emit.rs` edit) — but it is a **minimal, single-guard, non-serializing** change (DN-136 §1's own
distinction: item/expr-level dispatch is *already* one canonical match; this is one `if` inside an
already-narrow function, not a body every leaf edits). Flagged as low-but-nonzero collision risk;
recommend it lands **after** L1–L4 merge (or in an isolated worktree with a fast rebase) rather than
claiming zero risk by fiat.

---

## §5 Draft DN for the one genuinely-new design item

**D1 (unit type `()`)** is scoped and drafted as **`docs/notes/DN-137-Native-Answer-To-Unit-Type.md`**
(Draft, not ratified here — house rule #3; the DN-review gate ratifies). See that file for the full
alternatives/adversarial analysis. Summary (DN-137 was itself strict-gate-patched — the first Draft's
base_type recommendation was demoted for a strictly-smaller answer): Mycelium's `fn_sig`/`fn_item`
productions **mandate** an explicit `=> type_ref` on every function — no "omit the return type" sugar,
unlike Rust's implicit `-> ()`. DN-137 recommends **a prelude nullary-constructor data type
`type Unit = Unit;`** (DN-111 **Native Equivalent** — the arity-0 member of the M-826 tuple/product
family) — reusing the **existing** ADT machinery (`constructor`'s field-parens are already optional;
`lib/compiler/ast.myc` uses payload-free constructors pervasively) with **no new kernel node and no
grammar change**, over a new `base_type` (rejected: not minimal, and M-826-inconsistent — a base_type
carries kernel-managed representation `Unit` doesn't have), a dishonest sentinel-reuse (rejected on the
G2 veto), or loosening the mandatory-annotation grammar (out of scope). The build leaf is then a pure
prelude declaration + one `type_map::TABLE` row — **no `mycelium-core` edit**.

---

## §6 FLAGs (owned elsewhere — this doc edits none of them)

`Doc-Index.md`, `CHANGELOG.md`, and `issues.yaml` are integration-owned (concurrent-PR pattern:
leaves FLAG, the integrating parent applies once). **FLAG to the integrator:**

- **FLAG-1 (Doc-Index):** add a Planning-docs row for this worklist, and a Design-Notes row for
  DN-137 (Draft, 2026-07-13).
- **FLAG-2 (CHANGELOG):** append-only `[Unreleased]` entries for both new docs (dated 2026-07-13).
- **FLAG-3 (issues.yaml — M-ids to mint, verified free at filing time — mit #1):**
  - **M-1092** — DN-135 Result/Option-combinator lowering. **Already landed** (`c044452d`,
    PR #1547) — file as `status:done`.
  - **M-1093** — DN-134 struct-variant construction + collision-safe `struct_layouts`. **Already
    landed** (`a4318e53`, PR #1548) — file as `status:done`. Cross-reference: this issue's landing
    is what actually closed M-1089's own residual (§0 finding).
  - **M-1094** — DN-133 qualified-associated-fn call emission. **Already landed** (`eb1b7625`,
    PR #1546) — file as `status:done`.
  - **M-1095** — DN-136 P1-a emit hook-dispatch interface build (patterns/derives/calls axes).
    **Already landed** (`b1755fa6`, PR #1551) — file as `status:done`, `depends_on:
    [M-1092, M-1093, M-1094]` (per DN-136 §5's own stated dependency, satisfied).
  - **M-1096** — DN-136 P1-c `map.rs` type-map table. **Already landed** (`7d3724cc`, PR #1550) —
    file as `status:done`.
  - **M-1097 through M-1101** — the five Phase-2 first-wave leaves (§4): L1 (`derive Eq/PartialEq`),
    L2 (`derive Ord/PartialOrd`), L3 (`derive Hash`), L4 (conversion-method `prim_map` row), L5
    (bounded inherent-impl type-param emission) — file as `status:todo`, `depends_on:
    [M-1095, M-1096]` (the frozen interfaces), each `doc_refs: corpus:DN-128` (L1–L3),
    `corpus:M-1037` (L4), `corpus:DN-131` (L5).
  - **M-1102** — DN-137 (unit-type native answer) — a Draft-DN tracking id, not yet build-ready
    (design-first, per §5).
  - **Status-flip requests (not new ids):** `M-1089` → `done` (§0 finding, verify against
    `transpile.rs::struct_layouts`'s `Item::Enum` walk before flipping).
- **FLAG-4 (cross-refs):** add `corpus:DN-136-phase2-bulk-gap-close-worklist` pointers from
  `language-completeness-gap-inventory.md` §3 (rows 2, 3, 5, 10, 11 all moved design status) and
  from DN-136 §6 (this doc is the promised Phase-2 map). Add `doc_refs` `src:` anchors (all at
  `b1755fa6`): `emit/derives/mod.rs:66` (the `TABLE` needing L1–L3's rows),
  `emit/calls/mod.rs` + `prim_map.rs` (L4's actual home), `emit.rs:355-359` (L5),
  `crates/mycelium-l1/src/ord3.rs` (L2's fold target), `crates/mycelium-core/src/prim.rs`
  (`hash.blake3`'s signature, to be confirmed before L3 specs its exact fold).

---

## §7 Changelog

- **2026-07-13** — initial Draft. Re-measured the DN-136 Phase-2 corpus (`gen/myc-drafts`
  regenerated at `dev@b1755fa6`, same 13-target corpus the phylum baseline used):
  `checked_fraction` 6.34% oracle / 6.60% phylum (773-item denominator) — flat vs. the pre-Phase-1
  baseline, as expected (Phase-1 was a byte-identical migration). Re-derived the gap-class ranking
  (Other 21.2%, DeriveAttr 14.5% now #2, Import 13.6%, Impl 11.6%, ...). Found and corrected nine
  stale design-status claims (mitigation #14): DN-125/127/128/130/131/132 all landed
  Accepted+partly-built in a same-day wave the inventory doc predates; M-1089's own "not landed"
  residual is actually closed by M-1093's later work; M-1092/1093/1094 plus the DN-136 Phase-1
  build itself (P1-a/P1-c) are landed code with no `issues.yaml` row. Classified the remaining gap
  mass as 7 BUILD-READY items (Eq/Ord/Hash derive rows, `write!`/`format!` lowering, bounded-generic
  emission, conversion-method mapping, an M-1089 bookkeeping fix), 6 NEEDS-DESIGN items (one new —
  the unit-type `()` gap, drafted as DN-137; the rest already tracked), and 4 BLOCKED items.
  Validated DN-136 §6's "6–10 concurrent leaves" estimate at 5–6 for this concrete first wave
  (still `Declared` until the octopus merge actually runs conflict-free). Specs the recommended
  first wave (L1–L5) with differential-witness plans. Recommends, does not ratify (house rule #3);
  every figure `Empirical`/`Declared` at its basis (VR-5). FLAGs the Doc-Index/CHANGELOG/issues rows
  up (§6), including 11 M-ids to mint (5 landed-but-unfiled, 5 todo, 1 design-tracking).
