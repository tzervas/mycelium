# Appendix — Layer-3 (Transpiler Translation Rules) Grounded Inventory

> Supporting inventory for `docs/planning/zero-hand-port-delta-ledger.md` §1-§2. Draft, 2026-07-10.
> Grounded evidence backing the ledger's transpiler-reality synthesis; not independently ratified.

**Scope:** `crates/mycelium-transpile` — does each Rust construct have a CORRECT rule that emits
myc-check-clean idiomatic Mycelium, or gap / fabricate / leave-unmapped? Honest metric =
`checked_fraction` (myc-check-clean), NOT `expressible_fraction` (text emitted).

**Method:** grounded in ALREADY-COMMITTED artifacts (no fresh transpiler/build run, a batch was
building concurrently). Sources: transpiler source (`emit.rs`, `gap.rs`, `prim_map.rs`, `map.rs`,
`transpile.rs`), the committed draft manifest (`gen/myc-drafts/MANIFEST.md` plus
`manifest.json`, commit `13b00ff9`), the M-1006 ladder record (`docs/notes/DN-34` §8.7-§8.18),
`docs/planning/self-hosting-port-ledger.md`, `docs/CURRENT-STATE.md`, `tools/github/issues.yaml`.
Emission is `Declared`; every vet number is `Empirical` (real `myc check` oracle). VR-5.

---

## 1. Last measured checked_fraction plus corpus (MEASURED, `Empirical`)

| Snapshot | Corpus | non-test items | checked (myc-check-clean) | `checked_fraction` | `expressible_fraction` |
|---|---|---:|---:|---:|---:|
| **Latest, committed manifest** (`gen/myc-drafts/`, commit `13b00ff9`) | **17 wave-1 targets** (5 semcore SCC files plus 12 unported stdlib crates) | 760 | 59 | **7.8%** | 13.4% |
| DN-34 §8.17 (kernel prim-gap closure, `and`/`or` operand-gated plus CU-3 cast) | 17 wave-1 | ~760 | -- | **7.76%** (5.79%->7.76%, +15) | -- |
| DN-34 §8.18 (`Expr::Cast` widen arm) | 17 wave-1 | -- | -- | **+0 on corpus** (qualifying casts sit in already-gapped files; file-gated) | -- |
| DN-34 §8.9 baseline (wave-1 rip-through) | 17 wave-1 | 759 | 28 | **3.7%** | 6.1% |
| `docs/CURRENT-STATE.md` (M-991 verdict) | boot10 port surface | -- | -- | **3.7% union** (~0-8% per target) | -- |

**The honest current number: `checked_fraction` roughly 7.8% (59/760) over the 17-target wave-1
corpus** (committed manifest, latest). The ladder climbed 3.7% to 7.8% via M-1006 phases
§8.10-§8.18.

**DISCREPANCY FLAG (VR-5):** an earlier brief cited "~6.7% on the 24-target corpus." That could
NOT be grounded. No 24-target corpus exists in the committed artifacts, the corpus is **17
targets**; "24" in DN-34 (around line 479) is a *gap-category taxonomy refinement* (8 to 24
categories), not a corpus size. The last honest checked_fraction is **7.8% / 17 targets**, not
6.7% / 24. If a 6.7%/24-target run exists it is uncommitted here (possibly a concurrent batch),
**requires a fresh measurement to confirm**; do not cite 6.7%/24 as grounded.

**File-gating caveat:** `checked_fraction` numerator is file-gated all-or-nothing, one poison item
zeros a whole file's credit (`vet.rs::checked_clean_items_is_file_gated_all_or_nothing`). So 7.8%
is an honestly-conservative *lower bound* (DN-34 §8.7), not a per-item ceiling.

---

## 2. Rule-coverage inventory (from `emit.rs` / `transpile.rs` dispatch)

**No fabrication remains (G2 confirmed).** Every construct without a faithful rule routes to a
never-silent `GapReason`/`Category`. The two former fabrication classes were fixed in the enb wave
(M-1001, DN-34 §8.8): fake `use extern_crate.Sym;` emissions and verbatim reserved-word emissions
are now GAPPED, not emitted. `prim_map.rs` enforces this structurally, a `wired:false` row (CU-5
`wrapping_*`) ALWAYS refuses; a `wired:true` row emits only behind a `ReceiverGate` (never fires
on an unconfirmed operand type). CU-1/CU-6/CU-7/CU-8/CU-9 were deliberately EXCLUDED from the
table rather than guessed (value-shape/width mismatches, documented FLAGs, not fabrications).

| Rust construct | Rule in transpiler? | Emits check-clean? | Notes |
|---|---|---|---|
| `fn` (single-expr body) | YES `emit_fn`/`render_fn` | often | modifier-gated (`check_fn_modifiers`) |
| `Expr::Binary` `&`/`\|` to `and`/`or` | YES (operand-type-gated, §8.17) | YES for `Binary{N}` | the main checked-fraction lever landed |
| `Expr::Binary` `==`/`!=`/`<`/`>` | YES, `Binary{1}`->`Bool` bridge | YES | |
| `Expr::Cast` widen (`Binary{N} as Binary{M}`, M>N) | YES -> `width_cast` (§8.18, DN-41) | YES (faithful) | |
| `Expr::Cast` narrow (M<N) | YES -> `truncate` (DN-51) | YES | |
| `Expr::Cast` float-crossing | GAP `Other` | -- | PENDING-DESIGN CU-3-fidelity |
| `Expr::If`/`Match`/`Unary`/`Paren`/`Tuple(>=2)`/`Array`/`Block`/`Field` | YES | partial | |
| `Expr::Reference` (`&x`) | ERASED (value semantics, ADR-003) | YES | |
| `Expr::MethodCall` (float `is_nan`...) | YES via `prim_map` wired rows (CU-2) | YES | |
| `Expr::MethodCall` `wrapping_*` | GAP `Conversion` (wired:false) | -- | CU-5, no grammar surface |
| `Expr::Struct` literal plus `Expr::Field` proj | YES if layout resolvable (§8.12/§8.13) | YES | positional ctor, names dropped |
| String literal / `String` type | YES -> `Bytes` (§8.14) | YES | ladder's largest win |
| `Tuple()` unit / `Repeat` | GAP `Other` | -- | no surface |
| `Item::Enum`/`Struct` (named fields) | YES -> positional `constructor` | YES (fidelity note) | `NamedFieldDrop` records dropped names |
| `Item::Impl` (external trait, e.g. `Widen`) | EMITS but FAILS check | **NO** | `Category::Impl`, undefined-trait check error |
| `Item::Trait` | GAP mostly | -- | trait Self-bodies unsupported (M-876) |
| `Item::Use` | GAP `Import` (M-1001) | -- | no cross-nodule symbol table |
| `Item::Macro` (def/invocation) | GAP `MacroDef`/`MacroInvocation` | -- | no expand step (M-875) |
| `Item::Const`/`Static`/`Type`/`Union`/`ExternCrate`/`ForeignMod`/`Mod`/`TraitAlias` | GAP (`Other`/`Struct`/`Trait`) | -- | unmapped, never-silent |
| Reserved-word identifier | GAP `ReservedWord` | -- | human-rename decision, never auto-renamed |
| Multi-statement body | GAP `MultiStmtBody` (partly desugared §8.12) | -- | language is expr-oriented |
| Generic bounds / where-clauses | GAP `GenericBound`/`WhereClause` | -- | M-876 |
| Deep recursion | GAP `RecursionBudget` (RFC-0041) | -- | refuse before host stack overflow |

---

## 3. Gap-class taxonomy table (ranked; union 812 gaps / 17 targets, DN-34 §8.9, later phases reduced some)

| Gap class | Freq / blast-radius | Transpiler-only vs DOWNSTREAM (L1/L2) | Tracked issue |
|---|---|---|---|
| **Other** (type-coverage: String[partly closed §8.14], **signed ints** ADR-028 sign-free, `usize`/`isize`, `char`, closures, refs[closed §8.11]) | **#1, 322 gaps (40%)**, dominant everywhere | **DOWNSTREAM (LANGUAGE surface)**, needs kernel/grammar type vocabulary, not a transpiler rule | M-874 (needs-design); ADR-028 (sign-free) |
| **Impl** (external-trait / whole-impl, `Widen`-class) | **#2, 119 (15%)**; emits-but-fails-check, poisons whole file | **DOWNSTREAM (LANGUAGE)**, no impl-of-external-trait / trait Self-body surface; synthetic trait-def tried and FAILED | M-876 (needs-design) |
| **Import** (`use`) | **#3, 117 (14%)**; was universal poison, now gapped | **TRANSPILER-side** (symbol-table gap), needs a cross-nodule symbol table / project-mode vetting; single-file oracle cannot resolve | M-1001 (done, gapped honestly) |
| **Struct** (named-field record) | **#4, 80 (10%)** | **DOWNSTREAM (LANGUAGE)**, no record surface; positional ctor only (partly emitted via `NamedFieldDrop`) | M-876 |
| **GenericBound** (bounded generics) | **#5, 59 (7%)** | **DOWNSTREAM (LANGUAGE)**, bounded-generic surface undesigned | M-876 |
| **Macro** (Invocation plus Def) | 64 (8%) | **TRANSPILER-only**, expand-first macro expansion; blocks *emission* not *check* (lower checked-fraction priority) | M-875 (needs-design) |
| **DeriveAttr** | 19 | **MIXED**, derive expansion (transpiler) plus target trait surface (downstream) | M-875/M-876 |
| **PayloadVariant** | 21 | DOWNSTREAM (LANGUAGE), enum payload surface | M-876 |
| **ReservedWord** | 14 (gapped) | DOWNSTREAM (human port decision), no sanctioned auto-rename | M-1001 (done) |
| **Conversion** (CU-3 float cast, CU-5 wrapping) | small | DOWNSTREAM (KERNEL/ENGINE), prims not landed (CU-3 float-fidelity pending-design; CU-5 no grammar) | DN-34 §8.16/§8.17; RFC-0034 §10 |
| **MultiStmtBody** | 3 (mostly desugared §8.12) | TRANSPILER (bounded by expr-oriented language) | -- |
| **Trait**, **WhereClause**, **AssocConst**, **RecursionBudget** | tail | DOWNSTREAM (language) / guard | M-876 |

---

## 4. The load-bearing ratio: transpiler-only vs downstream

Of the 812 union gaps (DN-34 §8.9):

- **DOWNSTREAM (language/kernel/engine surface, L1/L2, NOT a transpiler rule):** Other 322 plus
  Impl 119 plus Struct 80 plus GenericBound 59 plus PayloadVariant 21 plus ReservedWord 14 plus
  Conversion plus tail, roughly **615+ gaps, ~75-80%**.
- **TRANSPILER-only** (a rule/mechanism the transpiler must gain): Import/symbol-table 117 plus
  Macro 64 plus partial Derive/MultiStmt, roughly **185-200 gaps, ~20-25%**.

**KEY INSIGHT (grounded, DN-34 §8.8 M-991 verdict):** the dominant blockers are **downstream
language-surface gaps, not transpiler defects.** Roughly 3 of every 4 "transpiler gaps" are the
LANGUAGE lacking a surface (record types, signed ints, external-trait impls, bounded generics,
String[now partly closed], closures) or the KERNEL lacking a prim (CU-3/CU-5). No transpiler rule
can close them, they are the E18-1 `needs-design` worklist (M-874/M-875/M-876). This is exactly the
M-991 go/no-go finding: **NO-GO as an automated bulk porter; GO as a never-silent gap-profiling
instrument.** The transpiler is downstream of L1/L2.

**Empirical proof of the ceiling:** §8.10/§8.11/§8.12 each moved `checked_fraction` by **0**
despite faithful new emission rules, the emitted constructs referenced types the language cannot
check (§8.11: "no transpiler-only change moves `checked_fraction` on this corpus"). The only moves
came from LANGUAGE/KERNEL surface: §8.14 (String to Bytes, kernel-backed) and §8.17 (kernel
`and`/`or` prims landed under kernel-unfrozen). Transpiler-side hardening is near-exhausted on this
fixed corpus.

---

## 5. Highest-leverage transpiler work (ranked)

1. **Cross-nodule symbol table / project-mode vetting** (Import, 117 gaps, ~14%), the one
   genuinely transpiler-only class with real blast radius. Would let `use` resolve and unblock
   multi-file drafts. Tracked adjacent to M-1001 (currently gapped honestly, not resolved).
2. **Expand-first macro expansion** (Macro, 64), M-875 (needs-design). Blocks emission not check,
   so lower checked-fraction priority, but high emission-coverage leverage.
3. Everything else with big blast radius (Other/Impl/Struct/GenericBound roughly 580 gaps) is
   **NOT transpiler work**, it is LANGUAGE-surface design (M-874/M-876) plus KERNEL prims
   (CU-3/5). Per the north star (translate RULES in the transpiler, close EXPRESSIBILITY gaps in
   the language), the highest-leverage overall move is closing language/kernel surface, then
   re-running the ladder.

---

## 6. Issue reconciliation (VR-5)

- **M-1000** (wire transpile-to-myc-check vet loop), **done**.
- **M-1001** (close top vet-blocking gap classes; gapped `use`/reserved-word, the anti-fabrication
  fix), **done**.
- **M-1002/M-1003** (stand up `gen/myc-drafts/` plus rip-through), **done**.
- **M-1006** (phased whole-corpus ladder), **in-progress** (phases §8.10-§8.18 recorded in
  DN-34).
- **M-873** (transpiler PoC) / **M-991** (accelerator evaluation), **done** (NO-GO bulk / GO
  instrument).
- **M-874** macro/type surface, **M-875** macro-expand (expand-first), **M-876** surface
  completeness (bounded generics, records, trait Self-bodies), all **needs-design** (the
  downstream language work).
- **M-1032 (macro-expand) and M-1037 (conversion-op mapping): NOT PRESENT in this worktree's
  `issues.yaml`** when this appendix was drafted (highest tracked M-10xx was M-1022 at the time
  of the L3 read). By the time this ledger was reconciled at integration, M-1032 and M-1037 had
  landed on `dev` (confirmed present) and the reconciliation in the ledger + the new tracked
  issues uses those ids directly. This row is kept for the honest record of the discrepancy at
  the time of analysis (VR-5).
