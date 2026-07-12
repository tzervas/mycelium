# Design Note DN-132 — The L3 Pattern-Surface Cluster: Struct-Variant, Range, and `@`-Binding Patterns

| Field | Value |
|---|---|
| **Note** | DN-132 |
| **Status** | **Draft** (2026-07-12). Authored as a **design-forward reasoner note** working the **DN-119 L3-G1/G2/G3 pattern-surface residual** — struct/named-field patterns, range patterns, and `@`-binding patterns — forward to a **ranked recommendation**. It **works the decision forward and recommends, ranked**; it **enacts nothing**, **ratifies nothing**, and **moves no other doc's status** (house rule #3, append-only — the maintainer ratifies). It **does not edit** `crates/mycelium-l1/**` (semcore serial lane — read-only), `crates/mycelium-transpile/**`, `issues.yaml`, `CHANGELOG.md`, or `Doc-Index.md` — FLAGGED in §11. Tags are `Empirical` where read against the tree (dev tip `fa53dc46`, 2026-07-12), `Declared` for any design not yet built/ratified (VR-5). |
| **Decides** | *Proposes, for ratification (does not self-ratify):* (1) the **verified residual** — of the three DN-119 L3 pattern-surface forms, **only struct-variant patterns are a genuine transpiler gap with a buildable follow-on** (DN-34 §8.22 finding #5, 6/114 Impl-class gaps); range and `@`-binding patterns should **not** get dedicated new L1 grammar productions of their own, but they are **not yet expressible either** — their natural remapping is onto a **guard-arm surface that is ratified but unbuilt** (DN-79, Accepted 2026-07-02; M-833, `status: todo`): `Arm` (`ast.rs:957–962`) has no guard field and the transpiler emitter currently refuses every match-arm guard (`emit.rs:1060–1064`, "match-arm guard (`if ...`) has no Mycelium equivalent"). So range/`@` should become transpiler-emitted never-silent `when`-guard idioms **once M-833/DN-79 lands** — a **prerequisite dependency**, not already-served machinery. (2) The **native-solution class** of each (DN-111 taxonomy): all three are **Idiomatic Remappings** — P1 onto the positional-`Ctor` machinery that exists **today** (zero kernel-semantic growth, KC-3, exactly the shape of the landed or-pattern (M-823) and tuple-pattern (M-826) sugar); P2/P3 onto the guarded-arm machinery **DN-79/M-833 has already ratified but not yet built** — no *new* growth of this DN's own, but gated on that prerequisite landing first. (3) The **struct-variant-pattern lowering**: a `Pat::Struct` arm in the transpiler that resolves each field name to its declaration index via a **variant-aware** `StructLayout`, emitting a positional `Pattern::Ctor` with wildcards at omitted indices (`..` rest → all-remaining wildcard); reusing the existing Maranget usefulness/exhaustiveness pass untouched. (4) The **`StructLayout` variant-awareness extension** (`emit.rs:28` — the DN-34 §8.22 blocker): the flat `HashMap<String, Vec<Option<String>>>` gains per-ctor/per-variant layouts so an enum struct-variant (`E::A { x }`) resolves its field names. (5) The **do-not-build boundary**: a range token + `Pattern::Range` + interval-exhaustiveness in the checker, and a `Pattern::Bind` variant, stay **rejected as L1 grammar** (KISS/YAGNI; DN-119 §8 honest `checked_fraction` boundary — pattern-surface grammar moves the vet number ~0). (6) The **honest tag boundary + open questions** (§7/§8): exhaustiveness holes under the guard idiom, nested/overlapping patterns, and the DN-104 seal interaction. It **references DN-123** for the field-name↔index map (does not duplicate it) and **DN-104** for the seal semantics. |
| **Feeds** | DN-119 **L3-G1/G2/G3** (struct/named-field, range, `@`-binding patterns — the genuine L1-grammar residual, DN-119 §3/§7 Phase A) — this note is that Phase-A design; DN-123 (records / named-fields surface lever — the field-name↔index map + struct-pattern OQ-4 this note builds the pattern half of); DN-104 (M-1027, per-constructor visibility seal — the pattern-position-sealing residual, §7); DN-34 §8.22 (the `Impl`-gap breakdown — finding #5, the struct-variant-pattern buildable follow-on this note scopes); M-823 (or-patterns, RFC-0020 §9 — the checker-desugar precedent); M-826 (tuple patterns — the synthetic-ctor-desugar precedent); DN-79 (M-833, guard-clause semantics — Accepted 2026-07-02, unbuilt; the P2/P3 prerequisite dependency this note relies on). |
| **Grounds on** | **DN-111 §3.2** (native-equivalence taxonomy — all three forms classify as Idiomatic Remapping); **DN-106 GP2** (gap-closure default = the mechanically-lowering sugar, not a kernel primitive — the ratified basis for desugar-to-positional over a new `Pattern` variant); **KC-3** (small auditable kernel — zero L0/`Ty`/eval growth for P1; P2/P3 rely on the guard-field growth **DN-79/M-833 already ratified**, not new growth of this DN's own); **KISS/YAGNI** (no range/interval-exhaustiveness subsystem for a confirmed ~0 vet payoff, DN-119 §8); **DRY** (reuse the existing `StructLayout`, the Maranget `usefulness` pass, and the or/tuple desugar shape — no parallel resolver); **DN-79** (M-833 — the guard-clause semantics dossier P2/P3's `when`-idiom recommendation depends on landing); **G2/never-silent** (an unresolved field name, a wrong `..`-rest arity, a duplicate field name, and the range-guard exhaustiveness loss are all never-silent refusals/flags); **VR-5** (the whole proposed design is `Declared` until built + differential-witnessed; pattern-surface grammar's `checked_fraction` impact is `LOW` per DN-119 §8, never upgraded — and P2/P3's recommendation is correctly downgraded from "already served" to "prerequisite-gated"). |
| **Date** | July 12, 2026 |
| **Task** | Work the DN-119 L3-G1/G2/G3 pattern-surface residual forward — verify-first inventory + ranked recommendation + surface grammar/lowering + adversarial stress-test + DoD. Read-only except this DN plus its FLAGGED Doc-Index/CHANGELOG rows. Parallel-cluster DN slot assigned by coordinator: **DN-132** (mit #1 — DN-125..131 taken by sibling clusters). |

> **Grounding + honesty (house rule #4 / VR-5 / G2).** This note **works a decision forward and
> recommends, ranked**; it does **not** take the decision (house rule #3 — the maintainer ratifies).
> Its central finding — reported on the evidence, not shaped to manufacture a large deliverable
> (house rule #4: *be corrected over being wrongly affirmed*) — is that **this cluster is mostly a
> "decline to add grammar" decision, not a build.** Only one of the three forms (struct-variant
> patterns) is a genuine, buildable transpiler gap; the other two should **not** get dedicated new L1
> grammar of their own, but they are **not yet expressible either** — their natural remapping is a
> **guarded-arm surface that DN-79/M-833 has already ratified but not yet built** (a corrected finding,
> below — an earlier draft of this note mistakenly claimed the guard surface "already exists"; `Arm` has
> no guard field and the transpiler refuses every guard today). Adding dedicated L1 range/`@` grammar
> (beyond the ratified guard prerequisite) would still grow the kernel and the exhaustiveness checker
> further, for a payoff DN-119 §8 already *measured* at ~0 `checked_fraction`. The honest deliverable is:
> **build the struct-variant-pattern remapping now (the DN-34 §8.22 follow-on); adopt the range/`@`
> `when`-guard idiom once M-833/DN-79 lands; and decline the dedicated range-token/`Pattern::Range`/
> `Pattern::Bind` grammar regardless** — following the evidence, not the "implement L3 comprehensively"
> phrasing (VR-5 applied to the directive), and not overclaiming machinery that does not exist yet
> (VR-5 applied to this note's own earlier draft).

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
Mycelium's pattern grammar is positional-constructor + literal by design (value-semantic, positional
identity, ADR-003); a pattern **guard** is a **separately-ratified-but-unbuilt** extension (DN-79,
Accepted 2026-07-02; M-833, `status: todo`), not part of today's grammar. Every one of the three is an
**Idiomatic Remapping** (DN-111 §3.2 / DN-110 "Solution"): P1 onto machinery that **already exists** —
positional `Pattern::Ctor` (with wildcards); P2 onto a **guarded arm** (`p when cond => …`, DN-79's
ratified `when` keyword) and P3 onto a **named binder + guard**, both of which remap onto machinery
that is **ratified in direction but not yet built** — `Arm` (`ast.rs:957–962`) has no guard field today
and the transpiler emitter refuses every match-arm guard (`emit.rs:1060–1064`). P1 takes the identical
shape the landed **or-pattern** (M-823, checker-desugared to repeated arms) and **tuple pattern** (M-826,
desugared to a single synthetic-ctor `Ctor` match) sugar already take — zero kernel growth (KC-3); P2/P3
take that same shape **once the DN-79/M-833 guard prerequisite lands**.

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
  token (`DotDot`/`DotDotEq` — grep: 0 hits). **Correction (reviewer-caught, `Empirical`):** an earlier
  draft of this note claimed there is also no `@`/`At` token — **false**: `Tok::At` **exists**
  (`token.rs:304`), already lexed for guarantee annotations (`@tier`, `@std-sys`, `@forage`, …). So P1's
  `{ }` delimiters are already lexed, **and P3's `@` glyph is already lexed too** — P3 needs only a new
  **parser arm** reusing the existing `Tok::At`, not a lexer change. **P2 alone needs a new lexer token**
  (`DotDot`/`DotDotEq`, confirmed absent).
- **`Arm` has no guard field, and the emitter refuses every guard (the load-bearing finding).**
  `Arm` (`ast.rs:957–962`) is exactly `{ pattern: Pattern, body: Expr }` — **no guard slot**. The
  transpiler's `map_arm` loop (`emit.rs:1059–1064`) explicitly rejects any Rust arm carrying a guard
  (`arm.guard.is_some()`), returning a `GapReason` whose message reads, verbatim: *match-arm guard
  ("if ...") has no Mycelium equivalent (arm grammar has no guard slot)*. So **the "guard idiom" P2/P3
  remap onto does not exist yet** — it is the subject of **DN-79** (Guard-Clause Semantics Dossier,
  Accepted 2026-07-02, "Nothing is implemented by this acceptance") and **M-833** (`status: todo`),
  whose ratified surface keyword is `when`, not `if`. Landing it requires a new `Arm.guard` field, plus
  the `when` keyword itself, an exhaustiveness interaction (guarded arms don't count toward Maranget
  coverage), and guarantee-tag propagation (DN-79 §4's meet rule) — genuine L1 grammar growth, already
  ratified in direction but not yet built. P2/P3's guard-idiom recommendation is therefore correctly
  stated as a **dependency on M-833/DN-79 landing first**, not as already-expressible machinery.
- Exhaustiveness: `usefulness.rs` runs Maranget usefulness over `Pat::Ctor`/`Pat::Lit`/`Pat::Wild`
  (`usefulness.rs:38/181`) — **positional**. There is no guarded-arm construct today to test against
  exhaustiveness; **once M-833/DN-79 lands**, a guarded arm will **not** contribute to exhaustiveness (a
  guard makes an arm conditional — a match of only-guarded arms will need a wildcard, enforced
  never-silently, the same W7 coverage guard M-1035 uses for open-`Bytes`). This is a **prospective**
  design consequence of the not-yet-built guard machinery, not a currently-observed behavior.

**Transpiler (Rust ports) — `map_pattern_inner` and its residual (`emit.rs:2159`–`2232`):**
- Handles: `Pat::Wild`, `Pat::Ident` **without** `subpat` (the guard `pi.subpat.is_none()`, `:2162`),
  `Pat::Path`, `Pat::TupleStruct` (positional ctor), `Pat::Lit` (bool/int/str), `Pat::Or`, `Pat::Tuple`,
  `Pat::Paren`, `Pat::Reference`.
- **Falls to the `_` catch-all gap** (`:2228`, `Category::Other` "unsupported match pattern form"):
  `Pat::Struct` (P1), `Pat::Range` (P2), and `Pat::Ident` **with** `subpat` (P3 — the `@`-binding is a
  `syn::PatIdent` carrying a `subpat`, guarded out by `:2162`). All three are honest never-silent gaps
  **today** — and a `syn` arm carrying an `if`-guard hits the separate, earlier guard refusal at
  `emit.rs:1060–1064` before pattern-mapping even runs, regardless of which of P1/P2/P3 the pattern is.
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
2. **P2 (range patterns) does not need dedicated L1 grammar, but it is *not yet expressible* either.**
   A range match's natural remapping is a **guarded arm** (`n when n >= lo and n <= hi => …`), but
   guarded arms are **not** currently expressible — `Arm` has no guard field and the emitter refuses
   every guard (verified above). So P2's honest state is: **gated on M-833/DN-79 landing first**; until
   then it is a raw, catch-all transpiler gap like any other unhandled `Pat::Range`. This is a correction
   from an earlier draft of this note, which incorrectly stated the L1 grammar and transpiler "already
   support" the guard idiom (VR-5 — the claim is downgraded to match its actual basis).
3. **P3 (`@`-binding) does not need dedicated L1 grammar either, but its common case has the same
   M-833/DN-79 dependency as P2.** `x @ 1..=5` remaps to a **named guard** (`x when x >= 1 and x <= 5 =>
   use(x)`, once `when` lands), where the arm binder *is* `x`; the range half reuses P2's idiom and its
   prerequisite. Only `x @ StructuralPattern` (bind + non-range shape) is a residual beyond that, and it
   is rare (§4).

---

## §3 The native-solution class of each (DN-111)

Applying the DN-111 §3.2 `{exact?}×{native?}` generator + the DN-110 handles:

| Form | DN-111 class | Native machinery it remaps onto | Kernel growth |
|---|---|---|---|
| P1 struct-variant pattern | **Idiomatic Remapping** (DN-110 "Solution") | positional `Pattern::Ctor(name, [wildcards + binders at resolved indices])`; `..` rest → all-remaining wildcards | **zero** (reuses `Ctor` + usefulness, all landed today) |
| P2 range pattern | **Idiomatic Remapping** | a **guarded arm** `p when p >= lo and p <= hi` over the existing comparison ops (RFC-0025) | **zero *new* growth of this DN's own** — but **gated on M-833/DN-79** (the `Arm.guard` field + `when` keyword are ratified, Accepted 2026-07-02, but **not yet built**; `status: todo`) |
| P3 `@`-binding | **Idiomatic Remapping** | for `x @ range`: a **named guard** (binder `x` + P2's guard); for `x @ shape`: bind-and-nested-match | **zero *new* growth for the range case** — same M-833/DN-79 dependency as P2 |

P1 has a native answer that reuses landed machinery **today**. P2/P3 have a native answer that reuses
machinery **DN-79/M-833 has already ratified in direction but not yet built** — so this cluster's
recommendation for P2/P3 is *correctly* a construct that should be a mechanically-lowering remapping
**once its prerequisite lands**, not a new kernel/grammar production of its own (DN-106 GP2). It is
**not** already-available machinery today.

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

- **B1 (recommended, PREREQUISITE-GATED on M-833/DN-79) — transpiler-emitted never-silent `when`-guard
  idiom; no *dedicated* L1 range grammar.** `lo..=hi` → an arm guard `p when p >= lo and p <= hi`;
  `lo..hi` → `p when p >= lo and p < hi`. **Not yet expressible** — this depends on `Arm.guard` +
  the `when` keyword landing first (DN-79, Accepted 2026-07-02, "Nothing is implemented by this
  acceptance"; M-833, `status: todo`). Once that lands: the emitter binds the scrutinee to a fresh name
  if the arm had no binder, and never-silently flags the **exhaustiveness cost** (§7 OQ-1). B1 introduces
  **no additional grammar growth of its own** beyond the already-ratified M-833/DN-79 guard field — it
  is a genuine dependency, not a zero-cost idiom available today.
- **B2 (rejected unless demand witnessed) — a `DotDot`/`DotDotEq` token + `Pattern::Range(lo, hi,
  inclusive)` + interval exhaustiveness in `usefulness.rs`.** This is real lexer + checker growth **on
  top of** B1's M-833/DN-79 prerequisite (an interval-lattice specialization so `0..=127 | 128..=255` is
  exhaustive on a bounded scalar). DN-119 §8 *measured* pattern-surface grammar at ~0 `checked_fraction`,
  so this is KC-3 cost for a confirmed ~0 payoff — **YAGNI** until a port driver's *exhaustive range
  coverage* proves the guard idiom's exhaustiveness loss (§7 OQ-1) too costly. B2 remains rejected
  regardless of whether M-833/DN-79 lands.

### P3 — `@`-binding patterns

- **C1 (recommended, PREREQUISITE-GATED on M-833/DN-79) — remap the common `x @ range` to a named
  guard; gap `x @ shape`.** `x @ lo..=hi` → `x when x >= lo and x <= hi => …` (the binder *is* `x`, so
  the whole value is bound for free while the range constrains it — elegant, no *new* growth of this
  DN's own, reuses B1's dependency and its `when` keyword). **Not yet expressible**, same M-833/DN-79
  gate as B1. The rarer `x @ SomeCtor(y)` (bind + a non-range structural shape) stays a **never-silent
  gap** until witnessed regardless (its faithful desugar — bind the scrutinee *and* destructure — needs
  a let-in-arm the surface does not have; not worth the grammar today).
- **C2 (rejected) — a `Pattern::Bind(name, Box<Pattern>)` variant.** Kernel growth for the rare
  structural case; the common range case needs nothing beyond B1/C1's shared M-833/DN-79 dependency.
  YAGNI.

### Overall

- **R1 (recommended) — build P1 (A1) now; adopt B1/C1 as the never-silent idiom emission ONCE
  M-833/DN-79 lands; decline the range-token/`Pattern::Range`/`Pattern::Bind` grammar (B2/C2)
  regardless.** Until M-833/DN-79 lands, P2/P3 remain honest, raw catch-all transpiler gaps (the
  current, actual state) — R1 does not claim they are already served.
- **R2 (rejected) — full L1 grammar for all three (A2 + B2 + C2).** Grows kernel + lexer + exhaustiveness
  checker for a DN-119-§8-measured ~0 `checked_fraction`; fails KISS/YAGNI + KC-3.
- **R3 (partial) — P1 only, no P2/P3 handling at all, ever.** Under-delivers relative to R1: R1 still
  names the B1/C1 idiom as the *eventual* answer once its prerequisite lands, where R3 declines P2/P3
  permanently.

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

### §5.4 Range/`@` idiom lowering (P2/P3) — **PREREQUISITE: gated on M-833/DN-79 landing first**

Unlike §5.1–§5.3 (the P1 build, buildable today), this sub-section designs a lowering that **cannot be
built until `Arm` gains a guard field and the `when` keyword lands** (DN-79, Accepted 2026-07-02, "Nothing
is implemented by this acceptance"; M-833, `status: todo`). It is recorded here so the design is ready
the moment that prerequisite lands, not because it is buildable now.

- P2 (once M-833/DN-79 lands): the emitter maps `syn::Pat::Range { lo, hi, limits }` at an arm to a
  **guard** on the arm's binder (introducing a fresh binder if the arm was a bare `Pat::Range`), using
  the **ratified `when` keyword**: `p when p >= lo and p <= hi` (inclusive) or `p when p >= lo and p <
  hi` (half-open). It **must** pair this with the never-silent exhaustiveness flag of §7 OQ-1.
- P3 (once M-833/DN-79 lands): `x @ lo..=hi` maps to `x when x >= lo and x <= hi` (binder `x`, P2's
  guard). `x @ <non-range>` stays a never-silent gap.
- **Until M-833/DN-79 lands**, both P2 and P3 fall to the existing `_` catch-all gap (`emit.rs:2228`,
  "unsupported match pattern form") — the honest interim state, not a claimed idiom.

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

1. **R1 (recommended) — build P1 (A1 + §5 variant-aware layout) now; adopt P2/P3 as never-silent `when`-
   guard idioms (B1/C1) ONCE M-833/DN-79 lands; decline the L1 range/`@`/`Bind` grammar (B2/C2)
   regardless.** Scores highest: C1 (all three problems have a native answer — P1 today, P2/P3 once their
   ratified prerequisite is built), C2 (**veto passed for P1** — zero kernel growth; **P2/P3's growth is
   M-833/DN-79's already-ratified guard field, not new growth this DN introduces** — the veto is
   satisfied in the sense that this DN adds nothing new, but P2/P3 are honestly *not buildable* until
   that separate, already-decided growth lands), C4 (no *dedicated* range/`@` grammar for the ~0-payoff
   forms), C5 (P1 restores field-name readability now; P2/P3 will emit readable guards once buildable),
   C6 (every residual — including the M-833/DN-79 dependency itself — flagged, never silently assumed
   available). C3 is honestly `LOW` (DN-119 §8 — pattern-surface work is faithfulness, not vet). *This is
   the recommendation*, corrected from an earlier draft that mischaracterized P2/P3 as "already
   expressible, zero grammar growth" — the honest basis is a **named dependency**, not existing
   machinery (VR-5).
2. **R3 (partial) — P1 only, ever.** Sound but permanently declines P2/P3 even after M-833/DN-79 lands,
   where a bounded idiom would then check-clean; R1 dominates by naming the idiom as the *eventual*
   answer rather than a permanent decline.
3. **R2 (rejected) — full grammar for all three.** Fails C2 (**veto** — range interval-exhaustiveness
   is a real checker subsystem, on top of the M-833/DN-79 guard-field growth) and C4, for a C3 ≈ 0.
   Rejected.

**Argument against my own recommendation (VR-5, no sycophancy).** R1's weakness: it **declines to add
range and `@` grammar the directive's literal "implement L3 comprehensively" could be read to demand**,
it leaves `x @ shape` and *exhaustive* range coverage as gaps, and — the corrected, load-bearing
weakness — **P2/P3 are not buildable at all until a separate, currently-unbuilt decision (M-833/DN-79)
lands**, so R1's near-term deliverable is narrower than "the L3 cluster is closed": it is P1 now, P2/P3
later. Four honest rebuttals and one concession: (a) the decline of B2/C2 is grounded in DN-119 §8's
*measured* ~0 `checked_fraction` for pattern-surface grammar and in KC-3, not in convenience; (b) the
guard idiom **will** express P2/P3's problem natively once built (Python itself has no range pattern —
§9 — which validates the idiom as the *native* answer, not a shortfall); (c) the exhaustiveness loss
will be **flagged never-silently** once the idiom is built, so no correctness will be silently traded;
(d) the M-833/DN-79 dependency is **not a new ask this DN invents** — it is a decision the maintainer's
own delegated orchestrator already ratified 2026-07-02, so this DN is not proposing new kernel growth,
only correctly attributing an existing one. **Concession:** if a port driver appears whose correctness
*depends on* a bounded scalar's range-exhaustiveness (e.g. a `u8` matched by `0..=127 | 128..=255` with
no wildcard), then B2 (the range token + interval exhaustiveness) becomes warranted — R1 names that
trigger explicitly (§7 OQ-1) rather than pre-building it. The maintainer should ratify the
**decline-with-a-named-trigger for B2/C2, plus the explicit M-833/DN-79 dependency for B1/C1**, not a
claim that P2/P3 are already served.

---

## §7 Adversarial stress-test — where it breaks, and the open questions

**OQ-1 — Exhaustiveness holes under the guard idiom (the load-bearing one, P2/P3; PROSPECTIVE — the
guard idiom does not exist yet).** **Once M-833/DN-79 lands**, a guarded arm will **not** contribute to
Maranget exhaustiveness (a guard is conditional — this follows from DN-79 §3's design, not from any
currently-observed behavior, since no guarded arm can be constructed today). So a Rust match that is
exhaustive *because* of range patterns — `match b: u8 { 0..=127 => …, 128..=255 => … }` — will lower to
two **guarded** arms and become **non-exhaustive** in Mycelium (no wildcard), which the L1 checker will
reject never-silently (the W7 open-domain coverage guard, M-1035 precedent). **This is the honest
prospective cost of B1, and it must be never-silent, two ways, once B1 is built:** (i) the transpiler,
when it lowers a range pattern in a match with **no** irrefutable default arm, must either
**synthesize the required wildcard arm** *only if it can prove the ranges partition the scalar* (it
generally cannot, without interval reasoning) or **refuse the match with a precise gap** ("range-pattern
match relies on range-exhaustiveness; Mycelium lowers ranges to guards which do not prove exhaustiveness
— add a wildcard arm or witness B2"). It must **never** silently emit a non-exhaustive match, and
**never** fabricate a wildcard that changes semantics (G2/VR-5). (ii) This exact case is **the named
trigger for B2**: if such matches prove frequent, the interval-exhaustiveness grammar earns its kernel
cost. Recorded, not pre-built — and **not buildable at all until M-833/DN-79's own prerequisite lands
first**.

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
  differential witnesses it (DN-123 §5 tag boundary, shared). **P2/P3's B1/C1 idiom is `Declared` at a
  strictly weaker basis than P1's:** it is not merely an unbuilt lowering of existing machinery, it is a
  lowering *onto* machinery (`Arm.guard` + `when`) that **does not exist in the tree today** and depends
  on **M-833/DN-79** (Accepted 2026-07-02, `status: todo`) landing first — a dependency, not a design
  choice this DN controls. This distinction is load-bearing (a reviewer-caught correction, VR-5): the
  earlier draft understated it by calling the guard idiom "already expressible."
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
as `case n if <guard>`. This is **independent evidence** that the B1 `when`-guard idiom is the *native*
answer to "match a range," not a Mycelium shortfall (§6 self-argument rebuttal (b)) — **independent of
whether Mycelium's own guard machinery is built yet** (it is not; §2, M-833/DN-79). Python's `if`-guard
existing today doesn't change that Mycelium's `when`-guard does not exist today; it only strengthens the
case for *what to build* once M-833/DN-79 lands. Python's `as`-pattern
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
   self-hosted/DN-123 dependency) (§5.1–§5.3) — **done**; the range/`@` idiom lowering (§5.4) is
   designed but **PREREQUISITE-GATED on M-833/DN-79 landing first** — recorded ready, not buildable yet.
5. Adversarial stress-test — exhaustiveness holes, nested/overlapping, the DN-104 seal interaction,
   `..`-rest arity, field-order canonicalization (§7) — **done**.
6. Honest tag boundary + `checked_fraction` = `LOW` (§8); Python carry-forward (§9) — **done**.

**For maintainer ratification (what "Accepted" requires — the reasoner does not self-ratify, house
rule #3/#4):**
7. **Confirm the per-form verdict (§4/§6 R1):** build P1 (struct-variant patterns) **now** as the
   mechanically-lowering remapping; adopt P2 (range) and the common P3 (`x @ range`) as **never-silent
   `when`-guard idioms** **once M-833/DN-79 lands** (a confirmed prerequisite dependency, not
   already-available machinery — `Arm` has no guard field today, the emitter refuses every guard); in
   the interim, P2/P3 stay honest catch-all transpiler gaps; **decline** the L1 range token /
   `Pattern::Range` / interval-exhaustiveness and the `Pattern::Bind` grammar (B2/C2) **until the OQ-1
   named trigger is witnessed**, independent of the M-833/DN-79 timeline.
8. **Confirm the `StructLayout` variant-awareness shape (§5.1):** key layouts by emitted ctor name and
   populate from `Item::Enum` struct-variants — the DN-34 §8.22 blocker's resolution. Ratify that this
   is a **population** change to the existing `HashMap<String, Vec<Option<String>>>`, not a new type.
9. **Confirm the OQ resolutions as build preconditions:** OQ-1 (**once M-833/DN-79 lands and B1/C1
   become buildable**, range-guard exhaustiveness loss is never-silent — synthesize a wildcard *only*
   when provably safe, else refuse), OQ-4 (`..`-rest arity from a confirmed layout only, P1), OQ-5
   (canonicalize to declaration order — inherit DN-123 OQ-1, P1), and OQ-3(a) (pattern-position matching
   on a sealed ctor stays allowed under DN-132, P1).
10. **Confirm the coordination dependencies:** the self-hosted P1 half is **sequenced after DN-123**
    (named-field type surface — the name↔index map source); OQ-3(b/c) **coordinates with DN-104** before
    any cross-phylum sealed-ctor pattern-name reveal; OQ-2 redundancy-precision loss is documented in the
    DN-119 Phase-I idiom catalogue.
11. **Authorize the filing** (the integrator owns `issues.yaml`): file the **P1 transpiler build** as a
    tracking issue (the DN-34 §8.22 buildable follow-on — see FLAG-5 for the suggested slot, immediately
    actionable), the self-hosted P1 half as a DN-123-dependent issue, and the P2/P3 idiom emission as a
    small transpiler issue **explicitly `depends_on: [M-833]`** (not immediately actionable — blocked
    until M-833/DN-79 lands); cross-ref DN-119 L3-G1/G2/G3, DN-123, DN-104, DN-34 §8.22, M-833/DN-79.

**Enacted** (a separate, later transition — never skipped to) requires: the P1 transpiler `Pat::Struct`
arm + variant-aware layout landed and differential-witnessed on the DN-34 §8.22 targets; **M-833/DN-79
landed** (the P2/P3 prerequisite) followed by the P2/P3 `when`-guard-idiom emission landed with the OQ-1
never-silent exhaustiveness flag and `*_reject`/`*_flag` tests; and the self-hosted P1 half landed (after
DN-123) with usefulness/exhaustiveness green and the OQ-3/OQ-4/OQ-5 refusals pinned as tests. **P1 does
not depend on M-833/DN-79 and may Enact independently of the P2/P3 timeline** (see the FLAG-5 note on
splitting P1 out for standalone ratification if the maintainer prefers not to hold it on M-833/DN-79).

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
  DN-123-dependent issue. File the **P2/P3 `when`-guard-idiom emission** as a small standalone
  transpiler issue with **`depends_on: [M-833]`** — it is not actionable until M-833/DN-79 lands, so
  it should be filed `status: blocked` (or `todo` with the explicit `depends_on`), not left implying
  it is immediately buildable. **Orchestrator/integration-owned — not minted here** (mitigation #1/#2,
  ID + union-merge hygiene).
- **FLAG-6 (P1/P2-P3 ratification split — reviewer-suggested, `Declared`):** because P2/P3's
  recommendation is now correctly gated on **M-833/DN-79** (a separate, currently-unbuilt decision)
  while **P1 has no such dependency and is independently verified sound** (§5.1–§5.3, unchanged by this
  correction), the maintainer may prefer to **ratify P1 alone now** (split DN-132's §10 items 7–11 into
  a P1-only subset) and defer P2/P3's ratification until M-833/DN-79 actually lands, rather than holding
  the whole DN in Draft on a dependency P1 does not share. This DN does not perform that split (house
  rule #3 — the maintainer ratifies); it only flags the option as available and grounded.

---

## §12 User stories

- *As a **port author**, I want `match f { Foo { x, y } => … }` and `Self::NotFound { path, .. }` to port
  mechanically, so that* the ≈6/114 Impl-class struct-variant-pattern gaps close and the ported `.myc`
  reads against field names, not brittle positions.
- *As a **port author**, I want `match n { 0..=9 => … }` and `x @ 1..=5` to emit a readable, check-clean
  guarded arm **once M-833/DN-79's guard-clause machinery lands**, so that* range and range-bind matches
  port without a raw catch-all gap — **and** I am told never-silently when the lowering costs me
  range-exhaustiveness (I add a wildcard, or the match is honestly refused, never silently mis-lowered).
  Until that lands, I am told honestly that these forms are a catch-all gap, not silently misinformed
  that they already work.
- *As a **maintainer**, I want this pattern cluster to be a mechanically-lowering remapping with **zero
  *new* kernel/exhaustiveness-checker growth of this DN's own** (P2/P3 depend on M-833/DN-79's
  already-ratified guard field, not a fresh ask), so that* the small auditable kernel (KC-3) and
  value-semantic positional identity (ADR-003) are preserved — no range-interval subsystem enters the
  trusted base for a ~0-`checked_fraction` payoff, and I am not asked to approve growth I already
  approved under a different name.
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
- 2026-07-12 — **Correction (load-bearing, reviewer-caught during the strict DN-review gate; still
  Draft):** the P2/P3 (range/`@`-binding) analysis was **factually wrong** — the original text claimed
  the guard idiom (`p if cond => …`) was "already expressible," "already supported by the L1 grammar and
  the transpiler," and "zero grammar growth" (§2, §3, §4 B1/C1, §5.4, §6, Decides (1)/(3)). Verified
  against the tree: `Arm` (`ast.rs:957–962`) has **no guard field**, and the transpiler emitter
  (`emit.rs:1060–1064`) **refuses every match-arm guard** ("match-arm guard (`if ...`) has no Mycelium
  equivalent"). Guard clauses are **ratified-but-unbuilt**: **DN-79** (Guard-Clause Semantics, Accepted
  2026-07-02, "Nothing is implemented by this acceptance") + **M-833** (`status: todo`), whose ratified
  surface keyword is **`when`**, not `if`. Corrected: §1/§2/§3/§4/§5.4/§6/§7 OQ-1/§8/§9/§10/§12 and the
  header `Decides`/`Feeds`/`Grounds on` rows now state P2/P3's `when`-guard-idiom recommendation as a
  **prerequisite dependency on M-833/DN-79 landing**, not existing machinery — the guard-field/`when`-
  keyword growth belongs to M-833/DN-79 (already ratified, separately), not to this DN, so this DN adds
  no *new* kernel growth of its own, but P2/P3 are honestly **not yet buildable**. Also corrected: the
  **mis-citation "DN-102 (guarded arms)"** replaced with **DN-79** throughout (DN-102 is the unrelated
  `?`-try-operator desugar); the **false claim "no `@`/`At` token"** corrected — `Tok::At`
  **exists** (`token.rs:304`, already lexed for guarantee annotations/`@tier`/`@std-sys`/`@forage`), so
  P3 needs only a new **parser arm** reusing it, not a lexer change (only P2's range token is genuinely
  absent). **P1 (struct-variant patterns, §5.1–§5.3) is unchanged by this correction** — independently
  verified sound and immediately buildable; **FLAG-6** added noting the maintainer may ratify P1 alone
  now and defer P2/P3 pending M-833/DN-79. Honest tags applied throughout (VR-5); no status transition
  (stays Draft); append-only edit — nothing removed, only corrected/added (house rule #3/#4).
