# Design Note DN-132 — The L3 Pattern-Surface Cluster: Struct-Variant, Range, and `@`-Binding Patterns

| Field | Value |
|---|---|
| **Note** | DN-132 |
| **Status** | **Draft** (2026-07-12). Authored as a **design-forward reasoner note** working the **DN-119 L3-G1/G2/G3 pattern-surface residual** — struct/named-field patterns, range patterns, and `@`-binding patterns — forward to a **ranked recommendation**. It **works the decision forward and recommends, ranked**; it **enacts nothing**, **ratifies nothing**, and **moves no other doc's status** (house rule #3, append-only — the maintainer ratifies). It **does not edit** `crates/mycelium-l1/**` (semcore serial lane — read-only), `crates/mycelium-transpile/**`, `issues.yaml`, `CHANGELOG.md`, or `Doc-Index.md` — FLAGGED in §11. Tags are `Empirical` where read against the tree (dev tip `fa53dc46`, 2026-07-12), `Declared` for any design not yet built/ratified (VR-5). |
| **Decides** | *Proposes, for ratification (does not self-ratify):* (1) the **verified residual** — of the three DN-119 L3 pattern-surface forms, **only struct-variant patterns are a genuine transpiler gap with a buildable follow-on** (DN-34 §8.22 finding #5, 6/114 Impl-class gaps); range and `@`-binding patterns are already served natively by the **existing guard-arm surface** and should stay transpiler-emitted never-silent idioms, not new L1 grammar productions. (2) The **native-solution class** of each (DN-111 taxonomy): all three are **Idiomatic Remappings** onto the positional-`Ctor` + guarded-arm machinery — zero kernel-semantic growth (KC-3), exactly the shape of the landed or-pattern (M-823) and tuple-pattern (M-826) sugar. (3) The **struct-variant-pattern lowering**: a `Pat::Struct` arm in the transpiler that resolves each field name to its declaration index via a **variant-aware** `StructLayout`, emitting a positional `Pattern::Ctor` with wildcards at omitted indices (`..` rest → all-remaining wildcard); reusing the existing Maranget usefulness/exhaustiveness pass untouched. (4) The **`StructLayout` variant-awareness extension** (`emit.rs:28` — the DN-34 §8.22 blocker): the flat `HashMap<String, Vec<Option<String>>>` gains per-ctor/per-variant layouts so an enum struct-variant (`E::A { x }`) resolves its field names. (5) The **do-not-build boundary**: a range token + `Pattern::Range` + interval-exhaustiveness in the checker, and a `Pattern::Bind` variant, stay **rejected as L1 grammar** (KISS/YAGNI; DN-119 §8 honest `checked_fraction` boundary — pattern-surface grammar moves the vet number ~0). (6) The **honest tag boundary + open questions** (§7/§8): exhaustiveness holes under the guard idiom, nested/overlapping patterns, and the DN-104 seal interaction. It **references DN-123** for the field-name↔index map (does not duplicate it) and **DN-104** for the seal semantics. |
| **Feeds** | DN-119 **L3-G1/G2/G3** (struct/named-field, range, `@`-binding patterns — the genuine L1-grammar residual, DN-119 §3/§7 Phase A) — this note is that Phase-A design; DN-123 (records / named-fields surface lever — the field-name↔index map + struct-pattern OQ-4 this note builds the pattern half of); DN-104 (M-1027, per-constructor visibility seal — the pattern-position-sealing residual, §7); DN-34 §8.22 (the `Impl`-gap breakdown — finding #5, the struct-variant-pattern buildable follow-on this note scopes); M-823 (or-patterns, RFC-0020 §9 — the checker-desugar precedent); M-826 (tuple patterns — the synthetic-ctor-desugar precedent); DN-102 (guarded arms). |
| **Grounds on** | **DN-111 §3.2** (native-equivalence taxonomy — all three forms classify as Idiomatic Remapping); **DN-106 GP2** (gap-closure default = the mechanically-lowering sugar, not a kernel primitive — the ratified basis for desugar-to-positional over a new `Pattern` variant); **KC-3** (small auditable kernel — zero L0/`Ty`/eval growth; the positional `Pattern::Ctor` + guard machinery is untouched); **KISS/YAGNI** (no range/interval-exhaustiveness subsystem for a confirmed ~0 vet payoff, DN-119 §8); **DRY** (reuse the existing `StructLayout`, the Maranget `usefulness` pass, and the or/tuple desugar shape — no parallel resolver); **G2/never-silent** (an unresolved field name, a wrong `..`-rest arity, a duplicate field name, and the range-guard exhaustiveness loss are all never-silent refusals/flags); **VR-5** (the whole proposed design is `Declared` until built + differential-witnessed; pattern-surface grammar's `checked_fraction` impact is `LOW` per DN-119 §8, never upgraded). |
| **Date** | July 12, 2026 |
| **Task** | Work the DN-119 L3-G1/G2/G3 pattern-surface residual forward — verify-first inventory + ranked recommendation + surface grammar/lowering + adversarial stress-test + DoD. Read-only except this DN plus its FLAGGED Doc-Index/CHANGELOG rows. Parallel-cluster DN slot assigned by coordinator: **DN-132** (mit #1 — DN-125..131 taken by sibling clusters). |

> **Grounding + honesty (house rule #4 / VR-5 / G2).** This note **works a decision forward and
> recommends, ranked**; it does **not** take the decision (house rule #3 — the maintainer ratifies).
> Its central finding — reported on the evidence, not shaped to manufacture a large deliverable
> (house rule #4: *be corrected over being wrongly affirmed*) — is that **this cluster is mostly a
> "decline to add grammar" decision, not a build.** Only one of the three forms (struct-variant
> patterns) is a genuine, buildable transpiler gap; the other two are already expressible through the
> **guarded arm surface that already exists**, so adding L1 range/`@` grammar would grow the kernel and
> the exhaustiveness checker for a payoff DN-119 §8 already *measured* at ~0 `checked_fraction`. The
> honest deliverable is: **build the struct-variant-pattern remapping (the DN-34 §8.22 follow-on),
> emit the range/`@` guard-idiom never-silently, and decline the rest** — following the evidence, not
> the "implement L3 comprehensively" phrasing (VR-5 applied to the directive).

---

## §1 The problem, precisely

Three Rust (and, §9, Python) pattern-surface forms sit in the DN-119 L3 grammar residual (DN-119 §3,
rows L3-G1/G2/G3). Each solves a distinct readability/matching problem a mechanical port must express:

| # | Form (DN-119 id) | Rust example | The underlying **problem** it solves |
|---|---|---|---|
| P1 | **Struct-variant pattern** (L3-G1) | `match f { Foo { x, y } => … }`, `Self::NotFound { path, .. }`, `E::A { x } => …` | *Destructure a named-field product/variant by field name (order-insensitive, partial via `..`), so the match reads against the field names not brittle positions.* |
| P2 | **Range pattern** (L3-G2) | `match n { 0..=9 => …, 'a'..='z' => … }` | *Match a contiguous interval of a scalar with one arm (and, on a bounded scalar, contribute to exhaustiveness).* |
| P3 | **`@`-binding pattern** (L3-G3) | `match n { x @ 1..=5 => use(x), p @ Some(_) => … }` | *Bind the whole matched value to a name **while** also constraining its shape/range.* |

These three cross over — all are L3 concrete pattern-matching grammar, all share the same desugar
target (positional `Pattern::Ctor` + guarded arms), and P3 composes with both P1 and P2 — so they are
scoped as **one** DN (the task's framing), with **per-form** ranked recommendations because they do
**not** all warrant the same answer.

**The Mycelium-native answer (DN-111 / DN-110 taxonomy).** None of the three is a *Native Equivalent* —
Mycelium's pattern grammar is positional-constructor + literal + guard by design (value-semantic,
positional identity, ADR-003). Every one is an **Idiomatic Remapping** (DN-111 §3.2 / DN-110
"Solution") onto machinery that **already exists**: positional `Pattern::Ctor` (with wildcards) for
P1, a **guarded arm** (`p if cond => …`) for P2, and a **named binder + guard** for P3. This is the
identical shape the landed **or-pattern** (M-823, checker-desugared to repeated arms) and **tuple
pattern** (M-826, desugared to a single synthetic-ctor `Ctor` match) sugar already take — zero kernel
growth (KC-3).

---

## §2 Verify-first — what already parses/lowers vs. the genuine residual (`Empirical`, dev tip `fa53dc46`)

Read directly against `crates/mycelium-l1/src/{ast.rs,parse.rs,token.rs,checkty.rs,usefulness.rs}` and
`crates/mycelium-transpile/src/emit.rs` at `fa53dc46`.

**L1 self-hosted grammar — positional-only, confirmed (mitigation #14, the code is ground truth):**
- `Pattern` enum (`ast.rs:988`) has exactly: `Wildcard`, `Lit`, `Ctor(String, Vec<Pattern>)`
  (**positional**), `Ident`, `Tuple` (M-826), `Or` (M-823). **No** `Struct`/record, **no** `Range`,
  **no** `Bind` variant (grep: 0 hits for `Struct(`/`Range(`/`Bind(` in `ast.rs`).
- `parse_pattern_guarded` (`parse.rs:2148`) dispatches `_`, ident/ctor (`Ident` then `(`→`Ctor`),
  literals, and `(`→tuple/grouping. After a ctor-name ident it consumes **only `(`** — there is **no
  `{`-arm** (struct pattern). `parse_arm` (`parse.rs:2113`) gathers `|`-alternatives into `Pattern::Or`.
- `token.rs`: `LBrace`/`RBrace`/`Dot`/`Pipe` **exist** (`:280/282/330/332`), but there is **no** range
  token (`DotDot`/`DotDotEq` — grep: 0 hits) and **no** `@`/`At` token. So P1's `{ }` delimiters are
  already lexed; **P2 and P3 need new tokens** (a lexer change, not just a parser arm).
- Exhaustiveness: `usefulness.rs` runs Maranget usefulness over `Pat::Ctor`/`Pat::Lit`/`Pat::Wild`
  (`usefulness.rs:38/181`) — **positional**. Guarded arms already do **not** contribute to
  exhaustiveness (a guard makes an arm conditional; a match of only-guarded arms needs a wildcard —
  enforced never-silently, the same W7 coverage guard M-1035 uses for open-`Bytes`).

**Transpiler (Rust ports) — `map_pattern_inner` and its residual (`emit.rs:2159`–`2232`):**
- Handles: `Pat::Wild`, `Pat::Ident` **without** `subpat` (the guard `pi.subpat.is_none()`, `:2162`),
  `Pat::Path`, `Pat::TupleStruct` (positional ctor), `Pat::Lit` (bool/int/str), `Pat::Or`, `Pat::Tuple`,
  `Pat::Paren`, `Pat::Reference`.
- **Falls to the `_` catch-all gap** (`:2228`, `Category::Other` "unsupported match pattern form"):
  `Pat::Struct` (P1), `Pat::Range` (P2), and `Pat::Ident` **with** `subpat` (P3 — the `@`-binding is a
  `syn::PatIdent` carrying a `subpat`, guarded out by `:2162`). All three are honest never-silent gaps.
- `collect_pattern_bound_names` (`emit.rs:2085`) **already future-proofs `Pat::Struct`** (`:2103`) — a
  deliberate no-silent-gap guard placed so a later `map_pattern` arm cannot reintroduce the env-fix
  hole. So the binder-collection half of P1 is already wired; only the **emit** half is absent.
- `StructLayout = Vec<Option<String>>` (`emit.rs:28`), built by `struct_layouts` (`transpile.rs:315`)
  **only for `Item::Struct`** (`:318`), keyed by type name. **It is not variant-aware** — one flat
  layout per type name, matching a single-ctor product struct, **not** a multi-variant enum's
  struct-variants. This is the exact blocker DN-34 §8.22 finding #5 named (`:1177`–`1189`).

**Net finding (register-lag corrected, mitigation #14).** The genuine residual is **narrow and
uneven**:
1. **P1 (struct-variant patterns) is a real, buildable transpiler gap** — DN-34 §8.22 sized it at
   6/114 (≈5%) of this corpus's Impl class, "the one genuine, moderately-scoped follow-on candidate,"
   blocked on `StructLayout` variant-awareness + a `map_pattern` arm. This DN designs it. Its self-hosted
   `.myc` half depends on DN-123's named-field **type** surface (a positional type has no field names to
   resolve against — §5).
2. **P2 (range patterns) is *not* a `checked_fraction` gap** — a range match is already expressible as a
   **guarded arm** (`n if n >= lo and n <= hi => …`), which the L1 grammar and the transpiler already
   support. The residual is *faithfulness/exhaustiveness*, not expressibility (§7 OQ-1).
3. **P3 (`@`-binding) is *not* a gap for its common case** — `x @ 1..=5` is exactly a **named guard**
   (`x if x >= 1 and x <= 5 => use(x)`), where the arm binder *is* `x`; the range half reuses P2's
   idiom. Only `x @ StructuralPattern` (bind + non-range shape) is a residual, and it is rare (§4).

---

## §3 The native-solution class of each (DN-111)

Applying the DN-111 §3.2 `{exact?}×{native?}` generator + the DN-110 handles:

| Form | DN-111 class | Native machinery it remaps onto | Kernel growth |
|---|---|---|---|
| P1 struct-variant pattern | **Idiomatic Remapping** (DN-110 "Solution") | positional `Pattern::Ctor(name, [wildcards + binders at resolved indices])`; `..` rest → all-remaining wildcards | **zero** (reuses `Ctor` + usefulness) |
| P2 range pattern | **Idiomatic Remapping** | a **guarded arm** `p if p >= lo and p <= hi` over the existing comparison ops (RFC-0025) | **zero** (no new pattern/token if idiom-only) |
| P3 `@`-binding | **Idiomatic Remapping** | for `x @ range`: a **named guard** (binder `x` + P2's guard); for `x @ shape`: bind-and-nested-match | **zero** for the range case |

Every one has a native answer that reuses landed machinery. **This is the definition of a construct
that should be a mechanically-lowering remapping, not a new kernel/grammar production** (DN-106 GP2).

---

## §4 The real alternatives (per form + overall)

### P1 — struct-variant patterns

- **A1 (recommended) — desugar to positional `Pattern::Ctor` via a variant-aware name↔index map.**
  Transpiler: add a `Pat::Struct` arm to `map_pattern_inner` that, using a **variant-aware**
  `StructLayout`, resolves each named field to its declaration index and emits
  `Ctor(name, subs)` where `subs[i]` is the field's sub-pattern if named/bound and `Wildcard` otherwise
  (`..` rest ⇒ every unnamed index wildcard). Self-hosted `.myc`: a parse-time `{ … }`-arm +
  checker pre-pass desugar to `Pattern::Ctor` (the or/tuple-pattern precedent), resolving names against
  DN-123's named-field **type declaration** surface. Reuses the Maranget usefulness pass unchanged.
  **Zero kernel growth.**
- **A2 (rejected) — a first-class `Pattern::Struct(name, Vec<(String, Pattern)>, rest: bool)` variant
  carried through checker/usefulness/eval/elab.** Grows the kernel and forces a name-aware
  specialization in `usefulness.rs`. Contradicts the positional-identity design (DN-106 fork B, KC-3).
  Ruled out on the merits.

### P2 — range patterns

- **B1 (recommended) — transpiler-emitted never-silent guard idiom; no L1 grammar.** `lo..=hi` → an arm
  guard `p if p >= lo and p <= hi`; `lo..hi` → `p if p >= lo and p < hi`. Already expressible; the
  emitter binds the scrutinee to a fresh name if the arm had no binder. Never-silent flag on the
  **exhaustiveness cost** (§7 OQ-1). **Zero kernel/lexer growth.**
- **B2 (rejected unless demand witnessed) — a `DotDot`/`DotDotEq` token + `Pattern::Range(lo, hi,
  inclusive)` + interval exhaustiveness in `usefulness.rs`.** This is real lexer + checker growth (an
  interval-lattice specialization so `0..=127 | 128..=255` is exhaustive on a bounded scalar). DN-119
  §8 *measured* pattern-surface grammar at ~0 `checked_fraction`, so this is KC-3 cost for a confirmed
  ~0 payoff — **YAGNI** until a port driver's *exhaustive range coverage* proves the guard idiom's
  exhaustiveness loss (§7 OQ-1) too costly.

### P3 — `@`-binding patterns

- **C1 (recommended) — remap the common `x @ range` to a named guard; gap `x @ shape`.** `x @ lo..=hi`
  → `x if x >= lo and x <= hi => …` (the binder *is* `x`, so the whole value is bound for free while
  the range constrains it — elegant, zero growth, reuses B1). The rarer `x @ SomeCtor(y)` (bind + a
  non-range structural shape) stays a **never-silent gap** until witnessed (its faithful desugar —
  bind the scrutinee *and* destructure — needs a let-in-arm the surface does not have; not worth the
  grammar today).
- **C2 (rejected) — a `Pattern::Bind(name, Box<Pattern>)` variant.** Kernel growth for the rare
  structural case; the common range case needs nothing. YAGNI.

### Overall

- **R1 (recommended) — build P1 (A1), emit P2/P3 as never-silent idioms (B1/C1), decline the rest.**
- **R2 (rejected) — full L1 grammar for all three (A2 + B2 + C2).** Grows kernel + lexer + exhaustiveness
  checker for a DN-119-§8-measured ~0 `checked_fraction`; fails KISS/YAGNI + KC-3.
- **R3 (partial) — P1 only, no P2/P3 handling at all.** Under-delivers: leaves P2/P3 as raw
  catch-all gaps in the transpiler when a bounded idiom would emit + check-clean.

---

## §5 Surface grammar + lowering design (the P1 build)

The load-bearing deliverable. Two halves — a **transpiler half** (Rust ports, disjoint
`mycelium-transpile` lane, the DN-34 §8.22 follow-on) and a **self-hosted half** (semcore-serial
`mycelium-l1` lane, gated on DN-123).

### §5.1 `StructLayout` variant-awareness (the DN-34 §8.22 blocker)

Today `StructLayout = Vec<Option<String>>` keyed `HashMap<String, StructLayout>`, one flat layout per
**type name**, populated only for `Item::Struct`. An enum struct-variant (`E::A { x, y }`) has **no
entry**, so a `Self::A { x } => …` pattern cannot resolve `x`'s index. The extension (recommended
minimal shape, DRY over a parallel map):

- **Key layouts by the emitted *constructor* name, not the type name.** For a struct `Foo`, the ctor is
  `Foo` (unchanged). For an enum `E` with a struct-variant `A { x, y }`, register a layout under the
  emitted variant-ctor name (`A`, per `emit_enum`'s existing positional variant emission), so
  `struct_layout("A")` yields `[Some("x"), Some("y")]`. This makes the registry **variant-aware without
  a new type** — it is still `HashMap<String, Vec<Option<String>>>`; only the **population** changes
  (`struct_layouts` in `transpile.rs:315` must also walk `Item::Enum` variants with `Fields::Named`).
- The existing **resolvability gate** (`struct_layout` returns `None` unless the name is an emitted,
  in-file, resolvable ctor — `emit.rs:223`) carries over unchanged: a struct-variant pattern on a
  foreign/unresolved ctor gaps for want of a confirmed layout, never a mis-resolved emit (VR-5/G2).

### §5.2 The `map_pattern` `Pat::Struct` arm

New arm in `map_pattern_inner` (`emit.rs:2159`), mirroring the `Pat::TupleStruct` arm's guard shape:

1. Resolve the pattern's path to a ctor name; `guard_ident` it (as the existing ctor arms do).
2. Fetch `struct_layout(name)`; if `None`, gap never-silently ("struct-variant pattern on a ctor with
   no confirmed in-file layout" — the honest resolvability gap, DN-123 §2).
3. For each named field in the pattern, resolve its name to its declaration index via the layout; a
   **name not in the layout** is a never-silent refusal (not a wildcard), and a **duplicate field name**
   is a never-silent refusal (G2; DN-123 OQ-4c).
4. Build a positional `subs: Vec<Pat>` of `layout.len()` slots: the field's mapped sub-pattern at its
   resolved index, `_` (wildcard) at every unmentioned index. **`Foo { x, .. }` rest** ⇒ every
   unmentioned index is a wildcard (the `..` is the *permission* to omit; without `..`, an omitted
   field is still a wildcard in a match — Rust requires `..` for omission but the lowered positional
   pattern is identical, so the transpiler accepts either and emits the same positional `Ctor`).
5. Emit `format!("{}({})", name, subs.join(", "))` — a positional `Ctor` pattern the checker already
   handles. Exhaustiveness runs on the positional form **unchanged** (the Maranget pass never sees a
   struct pattern).

### §5.3 The self-hosted `.myc` half (gated on DN-123)

For hand-written / self-hosted `.myc` to write `Foo { x, y }` patterns, the parser needs a `{ … }`-arm
after a ctor name (the `{ }` tokens already lex), and a checker pre-pass must desugar it to
`Pattern::Ctor`. **But a positional `type E = A(T) | B(U)` declaration has no field names to resolve
against** — so the self-hosted struct pattern is **only well-posed once DN-123's named-field type
declaration surface exists** (`type Foo { x: T, y: U }`), which is where the parse-time name↔index
table comes from. **This DN references DN-123 for that map and does not duplicate it** (task framing);
the self-hosted P1 half is **sequenced after DN-123**, while the **transpiler P1 half has no such
dependency** (its map comes from the Rust `Fields::Named`) and is the immediately-buildable follow-on.

### §5.4 Range/`@` idiom lowering (P2/P3)

- P2: the emitter maps `syn::Pat::Range { lo, hi, limits }` at an arm to a **guard** on the arm's
  binder (introducing a fresh binder if the arm was a bare `Pat::Range`): `p if p >= lo and p <= hi`
  (inclusive) or `p >= lo and p < hi` (half-open). It **must** pair this with the never-silent
  exhaustiveness flag of §7 OQ-1.
- P3: `x @ lo..=hi` maps to `x if x >= lo and x <= hi` (binder `x`, P2's guard). `x @ <non-range>` stays
  a never-silent gap.

---

## §6 Objective function + ranked recommendation

**Objective function (the criteria this cluster is scored against):**

| Criterion | Weight | What it rewards |
|---|---|---|
| C1 — Native expressibility of each form's *problem* | high | the problem has a native answer (production OR flagged idiom) |
| C2 — Small auditable kernel (KC-3) + value-semantic positional identity (ADR-003) | **veto** | no production grows the kernel/exhaustiveness checker unnecessarily |
| C3 — `checked_fraction` leverage | medium | moves the honest vet number |
| C4 — KISS/YAGNI (grammar only when the idiom proves too costly) | high | no production added for a ~0-payoff form |
| C5 — Faithfulness / round-trip readability (DN-119 C5) | medium | ported code reads against field names / ranges |
| C6 — Never-silent (G2) | **veto** | every gap/exhaustiveness cost is flagged, never a silent mis-lower |

**Ranked recommendation:**

1. **R1 (recommended) — build P1 (A1 + §5 variant-aware layout), emit P2/P3 as never-silent guard
   idioms (B1/C1), decline the L1 range/`@`/`Bind` grammar.** Scores highest: C1 (all three problems
   expressible), C2 (**veto passed** — zero kernel growth), C4 (no grammar for the ~0-payoff forms), C5
   (P1 restores field-name readability; P2/P3 emit readable guards), C6 (every residual flagged). C3 is
   honestly `LOW` (DN-119 §8 — pattern-surface work is faithfulness, not vet). *This is the
   recommendation.*
2. **R3 (partial) — P1 only.** Sound but leaves P2/P3 as raw catch-all gaps where a bounded idiom would
   check-clean; R1 dominates.
3. **R2 (rejected) — full grammar for all three.** Fails C2 (**veto** — range interval-exhaustiveness
   is a real checker subsystem) and C4, for a C3 ≈ 0. Rejected.

**Argument against my own recommendation (VR-5, no sycophancy).** R1's weakness: it **declines to add
range and `@` grammar the directive's literal "implement L3 comprehensively" could be read to demand**,
and it leaves `x @ shape` and *exhaustive* range coverage as gaps. Three honest rebuttals and one
concession: (a) the decline is grounded in DN-119 §8's *measured* ~0 `checked_fraction` for
pattern-surface grammar and in KC-3, not in convenience; (b) the guard idiom **does** express P2/P3's
problem natively (Python itself has no range pattern — §9 — which validates the idiom as the *native*
answer, not a shortfall); (c) the exhaustiveness loss is **flagged never-silently**, so no correctness
is silently traded. **Concession:** if a port driver appears whose correctness *depends on* a bounded
scalar's range-exhaustiveness (e.g. a `u8` matched by `0..=127 | 128..=255` with no wildcard), then B2
(the range token + interval exhaustiveness) becomes warranted — R1 names that trigger explicitly (§7
OQ-1) rather than pre-building it. The maintainer should ratify the **decline-with-a-named-trigger**,
not a permanent exclusion.

---

## §7 Adversarial stress-test — where it breaks, and the open questions

**OQ-1 — Exhaustiveness holes under the guard idiom (the load-bearing one, P2/P3).** A guarded arm
does **not** contribute to Maranget exhaustiveness (correct — a guard is conditional). So a Rust match
that is exhaustive *because* of range patterns — `match b: u8 { 0..=127 => …, 128..=255 => … }` — lowers
to two **guarded** arms and becomes **non-exhaustive** in Mycelium (no wildcard), which the L1 checker
rejects never-silently (the W7 open-domain coverage guard, M-1035 precedent). **This is the honest cost
of B1, and it must be never-silent, two ways:** (i) the transpiler, when it lowers a range pattern in a
match with **no** irrefutable default arm, either **synthesizes the required wildcard arm** *only if it
can prove the ranges partition the scalar* (it generally cannot, without interval reasoning) or
**refuses the match with a precise gap** ("range-pattern match relies on range-exhaustiveness; Mycelium
lowers ranges to guards which do not prove exhaustiveness — add a wildcard arm or witness B2"). It must
**never** silently emit a non-exhaustive match, and **never** fabricate a wildcard that changes
semantics (G2/VR-5). (ii) This exact case is **the named trigger for B2**: if such matches prove
frequent, the interval-exhaustiveness grammar earns its kernel cost. Recorded, not pre-built.

**OQ-2 — Nested / overlapping patterns (P1/P2).** *Nested:* `Foo { x: Some(y), z: 0..=9, .. }` — P1
recurses (`map_pattern` on each field sub-pattern, already recursion-budget-guarded, `emit.rs:2149`),
and a nested range field inherits OQ-1 (a range inside a struct field is *also* a guard, so the whole
arm becomes guarded — the transpiler must lift the field-range into an arm-level guard on the resolved
binder, or gap; this composition must be specified, not assumed). *Overlapping:* `1..=5 | 3..=8` as
guards — the Maranget **redundancy** check cannot see guard overlap, so redundant-arm detection is
**lost** for range guards (a Rust warning becomes silent in Mycelium). Honest cost: redundancy
*precision* degrades under the idiom; flag it in the port-guide idiom catalogue (DN-119 Phase I), do
not claim redundancy checking over range guards.

**OQ-3 — The DN-104 seal interaction (P1).** DN-104's per-constructor visibility seal (`priv Mk`)
withholds **cross-nodule construction** of a sealed ctor but **exports the type name for pattern
position** (DN-104 §"Decides" (3)); DN-104 §6 lists **pattern-position sealing** as an explicit open
residual. Three sub-cases this DN must pin:
- (a) A struct-variant **pattern** `Mk { x }` on a sealed ctor lowers to the **same positional
  `Pattern::Ctor`** as any other pattern — it **constructs nothing**, so the seal's construction-gate is
  **untouched**; matching a sealed ctor stays allowed (consistent with DN-104: the seal gates
  *construction*, and matching is not construction). **Recommendation to ratify: pattern-position
  matching on a sealed ctor stays allowed under DN-132**, unchanged from DN-104's current semantics.
- (b) The field-**name reveal** — which field names a *foreign* nodule may write in `Mk { x, .. }` — is
  **phylum-local surface metadata** (DN-123 OQ-2/OQ-4b): the struct-variant pattern must **not** leak a
  sealed ctor's field names cross-nodule beyond what the positional surface already reveals. Since the
  transpiler's name↔index map is *file-local* (`struct_layout` gated on in-file resolvability), this
  holds by construction for the transpiler half; the self-hosted half inherits DN-123 OQ-2's
  coordination dependency (names phylum-local).
- (c) **If DN-104 later closes its §6 residual by *also* sealing pattern position** (withholding
  cross-nodule *matching* on a sealed ctor), the struct-variant pattern **inherits that decision**
  automatically (it lowers to the same `Ctor` pattern the seal check would gate). **FLAG: coordinate
  with DN-104 before the self-hosted P1 half claims cross-nodule sealed-ctor pattern reveal.**

**OQ-4 — `..`-rest arity correctness (P1).** The `..` rest expands to wildcards at *every* unmentioned
index — this is only correct if the layout arity is **exact**. A stale/wrong layout would emit a `Ctor`
of the wrong arity (a check failure at best, a mis-match at worst). The resolvability gate (§5.1) makes
this safe: a layout is used only when the ctor is a confirmed in-file emission, so its arity is ground
truth; otherwise the pattern gaps. **Never a partial or guessed arity** (VR-5/G2).

**OQ-5 — Field-order canonicalization (P1, inherited from DN-123 OQ-1).** Rust struct patterns are
order-insensitive (`Foo { y, x }` ≡ `Foo { x, y }`); the lowering resolves each name to its
**declaration index** and places sub-patterns positionally, so pattern field order is canonicalized to
declaration order — the **same obligation** DN-123 OQ-1 pins for construction. A missing/duplicate name
is a never-silent refusal, not a mis-ordered emit. **This DN adopts DN-123 OQ-1's resolution wholesale
for patterns** (does not restate the identity argument).

---

## §8 Honest tag boundary + `checked_fraction` impact

- **The whole proposed design is `Declared`** — a proposed lowering not yet built. It upgrades
  piecewise: the P1 transpiler arm becomes `Empirical` *for structure* once it emits and `myc
  check`-passes on the 6/114 DN-34 §8.22 targets; **name-faithful** round-trip stays `Declared` until a
  differential witnesses it (DN-123 §5 tag boundary, shared).
- **`checked_fraction` impact: `LOW`** — DN-119 §8 *measured* that pattern-surface grammar/emission
  moves the vet number ~0; P1's payoff is **faithfulness + closing 6 honest transpiler gaps**, not vet
  leverage. Any claim that "this cluster raises native capability (measured)" is **downgraded** to "this
  cluster raises *faithfulness* and closes catch-all gaps; the measured lever is elsewhere" (VR-5).
- **No tag is upgraded past its basis.** The residual counts (6/114, DN-34 §8.22) are quoted
  `Empirical`-per-source, **not re-counted here** (declared residual uncertainty). The range/`@`
  exhaustiveness loss (§7 OQ-1) is a `Declared` design consequence until B1 is built and witnessed.

---

## §9 Python carry-forward (the eventual second source language)

Python's structural pattern matching (PEP 634, `match`/`case`) maps **directly and favorably** onto
this cluster's recommendation — and *validates* it:

| Python `case` form | Maps to this DN's | Note |
|---|---|---|
| **Class pattern** `case Point(x=0, y=0)` (keyword sub-patterns) | **P1** struct-variant pattern | Python's keyword-patterns are exactly named-field patterns — the P1 remapping serves them |
| **Class pattern** positional `case Point(0, 0)` | existing positional `Ctor` | already handled |
| **Capture pattern** `case x` / **wildcard** `case _` | existing `Ident` / `Wildcard` | already handled |
| **OR pattern** `case 1 \| 2` | landed `Pattern::Or` (M-823) | already handled |
| **`as` pattern** `case [Point() as p]` | **P3** `@`-binding | Python's `as` = Rust's `@`; the P3 named-binder remapping serves it |
| **Guard** `case n if n in range(1, 6)` | **P2** guard idiom | **Python has no range pattern** — it *uses a guard*, exactly the B1 idiom |
| **Value pattern** `case Color.RED` | existing `Path`/`Ident` ctor | already handled |
| Mapping/sequence patterns `case {..}` / `case [..]` | out of scope here | a distinct future DN (dict/list destructuring) |

**The load-bearing carry-forward point:** Python **itself has no range pattern** — it expresses ranges
as `case n if <guard>`. This is **independent evidence** that the B1 guard idiom is the *native* answer
to "match a range," not a Mycelium shortfall (§6 self-argument rebuttal (b)). Python's `as`-pattern
being the same construct as Rust's `@` means P3's remapping is **source-general**, not Rust-specific.
Python's class-keyword-patterns being P1 means the struct-variant build serves both source languages.
**No Python-specific grammar is proposed here** (DN-119 §11 — Python is a separate DN + a DN-111
classification pass); this section only records that the recommendation *carries forward cleanly*, which
is a point *for* R1 (source-general, not Rust-shaped).

---

## §10 Definition of Done

**For this DN (done at authoring):**
1. Verified pattern-surface state recorded against the tree, with the residual separated per-form
   (§2) — **done**.
2. Native-solution class of each form fixed (DN-111, §3) — **done**.
3. Ranked recommendation per-form + overall, with an objective table and a self-argument (§4/§6) —
   **done**.
4. Surface grammar + lowering design for the P1 build (variant-aware layout, `map_pattern` arm,
   self-hosted/DN-123 dependency, range/`@` idiom lowering) (§5) — **done**.
5. Adversarial stress-test — exhaustiveness holes, nested/overlapping, the DN-104 seal interaction,
   `..`-rest arity, field-order canonicalization (§7) — **done**.
6. Honest tag boundary + `checked_fraction` = `LOW` (§8); Python carry-forward (§9) — **done**.

**For maintainer ratification (what "Accepted" requires — the reasoner does not self-ratify, house
rule #3/#4):**
7. **Confirm the per-form verdict (§4/§6 R1):** build P1 (struct-variant patterns) as the
   mechanically-lowering remapping; emit P2 (range) and the common P3 (`x @ range`) as **never-silent
   guard idioms**; **decline** the L1 range token / `Pattern::Range` / interval-exhaustiveness and the
   `Pattern::Bind` grammar (B2/C2) **until the OQ-1 named trigger is witnessed**.
8. **Confirm the `StructLayout` variant-awareness shape (§5.1):** key layouts by emitted ctor name and
   populate from `Item::Enum` struct-variants — the DN-34 §8.22 blocker's resolution. Ratify that this
   is a **population** change to the existing `HashMap<String, Vec<Option<String>>>`, not a new type.
9. **Confirm the OQ resolutions as build preconditions:** OQ-1 (range-guard exhaustiveness loss is
   never-silent — synthesize a wildcard *only* when provably safe, else refuse), OQ-4 (`..`-rest arity
   from a confirmed layout only), OQ-5 (canonicalize to declaration order — inherit DN-123 OQ-1), and
   OQ-3(a) (pattern-position matching on a sealed ctor stays allowed under DN-132).
10. **Confirm the coordination dependencies:** the self-hosted P1 half is **sequenced after DN-123**
    (named-field type surface — the name↔index map source); OQ-3(b/c) **coordinates with DN-104** before
    any cross-phylum sealed-ctor pattern-name reveal; OQ-2 redundancy-precision loss is documented in the
    DN-119 Phase-I idiom catalogue.
11. **Authorize the filing** (the integrator owns `issues.yaml`): file the **P1 transpiler build** as a
    tracking issue (the DN-34 §8.22 buildable follow-on — see FLAG-5 for the suggested slot), the
    self-hosted P1 half as a DN-123-dependent issue, and the P2/P3 idiom emission as a small transpiler
    issue; cross-ref DN-119 L3-G1/G2/G3, DN-123, DN-104, DN-34 §8.22.

**Enacted** (a separate, later transition — never skipped to) requires: the P1 transpiler `Pat::Struct`
arm + variant-aware layout landed and differential-witnessed on the DN-34 §8.22 targets; the P2/P3
guard-idiom emission landed with the OQ-1 never-silent exhaustiveness flag and `*_reject`/`*_flag`
tests; and the self-hosted P1 half landed (after DN-123) with usefulness/exhaustiveness green and the
OQ-3/OQ-4/OQ-5 refusals pinned as tests.

Status stays **Draft** until 7–11 are ratified.

---

## §11 FLAGs (append-only reconciliations this note does not perform)

`docs/Doc-Index.md`, `CHANGELOG.md`, `tools/github/issues.yaml`, and the DN-99 register are
**integration-owned** (the concurrent-PR pattern: leaves FLAG, the integrating parent applies once).
This DN edits **none** of them. **FLAG to the integrator:**

- **FLAG-1 (Doc-Index):** add a Design-Notes row `DN-132 — L3 Pattern-Surface Cluster: Struct-Variant,
  Range, and @-Binding Patterns (Draft)` to `docs/Doc-Index.md`.
- **FLAG-2 (CHANGELOG):** add a `docs(dn)` `[Unreleased]` entry for DN-132 (append-only, dated
  2026-07-12).
- **FLAG-3 (DN-119):** DN-119 §7 Phase A (pattern-surface completion, L3-G1/G2/G3) should cross-reference
  DN-132 as its design note. **Integration/semcore-owned — not edited here.**
- **FLAG-4 (DN-123):** DN-123 OQ-4 (struct-pattern exhaustiveness + DN-104 seal) should cross-reference
  DN-132 as the note that builds the pattern half; DN-123 remains the records-surface/name↔index-map
  owner. **Not edited here.**
- **FLAG-5 (issues.yaml — the buildable follow-on M-id):** file the **P1 struct-variant-pattern
  transpiler build** — the DN-34 §8.22 finding-#5 follow-on (variant-aware `StructLayout` +
  `map_pattern` `Pat::Struct` arm, ≈6/114 Impl-class gaps). Highest current M-id is **M-1080**; suggest
  minting **M-1081** (verify free against `issues.yaml` at mint time — mitigation #1), with `doc_refs`
  pointing at DN-132, DN-34 §8.22, `src:crates/mycelium-transpile/src/emit.rs:2159`, and
  `src:crates/mycelium-transpile/src/transpile.rs:315`. File the **self-hosted P1 half** as a separate
  DN-123-dependent issue, and the **P2/P3 guard-idiom emission** as a small standalone transpiler issue.
  **Orchestrator/integration-owned — not minted here** (mitigation #1/#2, ID + union-merge hygiene).

---

## §12 User stories

- *As a **port author**, I want `match f { Foo { x, y } => … }` and `Self::NotFound { path, .. }` to port
  mechanically, so that* the ≈6/114 Impl-class struct-variant-pattern gaps close and the ported `.myc`
  reads against field names, not brittle positions.
- *As a **port author**, I want `match n { 0..=9 => … }` and `x @ 1..=5` to emit a readable, check-clean
  guarded arm, so that* range and range-bind matches port without a raw catch-all gap — **and** I am
  told never-silently when the lowering costs me range-exhaustiveness (I add a wildcard, or the match is
  honestly refused, never silently mis-lowered).
- *As a **maintainer**, I want this pattern cluster to be mechanically-lowering remapping with **zero**
  kernel/exhaustiveness-checker growth, so that* the small auditable kernel (KC-3) and value-semantic
  positional identity (ADR-003) are preserved — no range-interval subsystem enters the trusted base for
  a ~0-`checked_fraction` payoff.
- *As a **Python-source port author** (eventual), I want `case Point(x=0)`, `case [..] as p`, and
  `case n if …` to map onto the same struct-variant / `@`-binding / guard machinery, so that* the second
  source language reuses this cluster's remapping unchanged (source-general, not Rust-shaped).

---

*DN-132 — Draft. Works the decision forward and recommends, ranked; ratification is the maintainer's
(house rule #3).*

## Changelog

- 2026-07-12 — DN-132 created (**Draft**): the L3 pattern-surface cluster (struct-variant, range,
  `@`-binding patterns — DN-119 L3-G1/G2/G3). Verify-first finding (only struct-variant patterns are a
  genuine buildable transpiler gap — DN-34 §8.22 finding #5; range/`@` are already served by the
  guarded-arm surface); native-solution class per DN-111 (all three Idiomatic Remappings, zero kernel
  growth); the P1 lowering design (variant-aware `StructLayout`, the `map_pattern` `Pat::Struct` arm, the
  self-hosted half's DN-123 dependency, the range/`@` guard-idiom); ranked recommendation R1 (build P1,
  emit P2/P3 as never-silent idioms, decline the range/`@`/`Bind` grammar until the OQ-1 trigger);
  adversarial OQ-1..5 (exhaustiveness holes, nested/overlapping, the DN-104 seal interaction, `..`-rest
  arity, field-order canonicalization); honest tag boundary (`checked_fraction` = `LOW`, DN-119 §8);
  Python carry-forward (class-keyword → P1, `as` → P3, guard → P2 — Python has no range pattern either);
  DoD + user stories. References DN-123 (name↔index map) and DN-104 (seal) without duplicating them.
  `Empirical` where read against dev tip `fa53dc46`; `Declared` for the proposed design. Authored READ +
  DN only — no edit to `crates/mycelium-l1/**`, `crates/mycelium-transpile/**`, `issues.yaml`,
  `CHANGELOG.md`, or `Doc-Index.md` (semcore/integration owned; FLAGGED up per §11). Append-only; status
  advances only by maintainer ratification (house rule #3).
