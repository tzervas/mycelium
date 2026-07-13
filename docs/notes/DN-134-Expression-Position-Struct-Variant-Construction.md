# Design Note DN-134 — Expression-Position Struct-Enum-Variant Construction (the positional-ctor native form)

| Field | Value |
|---|---|
| **Note** | DN-134 |
| **Status** | **Draft** (2026-07-12). Works the decision forward and **recommends, ranked**; it does **not** self-ratify (house rule #3 — the DN-review gate / maintainer ratifies Draft → Accepted). **Builds nothing** — every mechanism is `Declared`/unbuilt until the FLAGGED build issue (M-1093) lands and is differential-witnessed. Does not edit `crates/**`; `Doc-Index.md`/`CHANGELOG.md`/`issues.yaml` rows are **FLAGGED** for the integrating parent, not applied here. |
| **Decides** | *Proposes, for ratification:* (1) the **verified residual** — a named-field **enum struct-variant** built in **expression position** (`TimeErr::ClockUnavailable { reason: "…" }`, `Self::NotFound { path }`) is a genuine transpiler gap: `visit_struct` (`emit.rs:2324`) resolves the ctor via `struct_layout(name)` (`emit.rs:2353`), which is populated **from `Item::Struct` only** (`transpile.rs:315–332`) — so an *enum variant's* layout is never found and the construction gaps. This is the **construction twin** of DN-132/M-1089's **pattern** side (they share the identical blocker). (2) The **native-solution class** (DN-111 §3.2): an **Idiomatic Remapping** onto the positional `constructor` call **that already exists** — the enum emitter already lowers a struct-variant `Ctor { a: T, b: U }` to a positional `Ctor(T, U)` (names dropped + recorded, `emit.rs:3113–3141`); construction just *calls* that ctor with the field values in **declaration order**. Zero kernel growth (KC-3). (3) The **shared `StructLayout` variant-awareness** — the **same** population change DN-132/M-1089 specifies (`struct_layouts` also walks `Item::Enum` `Fields::Named` variants, keyed by emitted ctor name; a population change to the existing `HashMap<String, Vec<Option<String>>>`, not a new type) serves **both** the pattern arm (M-1089) and this construction arm (M-1093). (4) The **`visit_struct` extension**: with the variant-aware layout available, the existing field-resolution loop (`emit.rs:2362–2382`) already maps named fields to positional args by declaration order — so the construction arm is *near-free* once the shared population lands; the new work is the enum-variant path + the never-silent refusals. (5) The **honesty boundary** — cross-nodule resolvability (the `std-sys-host` case constructs `TimeErr` from another nodule) and the DN-104 **construction-side seal** (OQ-3(b)). It **references DN-123** for the field-name↔index map (does not duplicate it), **DN-132** for the pattern twin, **DN-104** for the seal, **DN-131** for the generic-variant interaction, and **DN-113** for cross-nodule resolution. |
| **Feeds** | The `std-sys-host` 6/6 path — closes residual #2 in `OsClock::wall_now` (`lib.rs:63–65`); the corpus DN-34 §8.22 "Other" gap class (expression-position struct-variant construction is the construction complement of the §8.22 finding #5 pattern gaps); DN-132/M-1089 (shares the `StructLayout` population — a coordination point); DN-123 (records / named-fields surface — the construction half of the field-name↔index map DN-132 built the pattern half of). |
| **Grounds on** | **DN-111 §3.2** (native-equivalence taxonomy — Idiomatic Remapping onto positional `Ctor`, the identical class DN-132 P1 took for the pattern side); **DN-106 GP2** (gap-closure default = the mechanically-lowering desugar over a new kernel form); **KC-3** (zero L0/`Ty`/eval growth — the positional ctor and the `HashMap` layout both exist); **DRY** (reuse the *same* `struct_layouts` population DN-132/M-1089 specifies, and `visit_struct`'s existing field-resolution loop — no parallel resolver); **ADR-003** (value-semantic positional identity — a named-field product *is* its positional tuple, so name-drop is faithful, exactly `emit.rs:3113–3141`'s ruling); **DN-123** (the field-name↔declaration-index map + field-order canonicalization, OQ-1 — inherited, not duplicated); **DN-104** (per-constructor visibility seal — construction of a sealed ctor outside its module is the never-silent refusal OQ-3(b)); **DN-131** (bounds on the type-parameter slot — a struct-variant of a generic enum composes with the impl-slot generics unchanged); **DN-113** (cross-phylum/cross-nodule import resolution — the resolvability dependency for a variant defined in another nodule); **G2/never-silent** (an unresolved ctor name, a missing/duplicate field, a sealed cross-module construction are all never-silent refusals); **VR-5** (`Declared` until built + differential-witnessed; no `Proven` claim). |
| **Date** | July 12, 2026 |
| **Task** | Scope the expression-position struct-variant-construction residual that (with DN-133) blocks `std-sys-host` 6/6, and that is a corpus-wide "Other" gap class — verify-first, native solution, ranked recommendation, emission spec, adversarial stress-test, DoD. Read-only except this DN + its FLAGGED rows. Parallel-cluster slot: **DN-134** (mit #1 — DN-125..133 taken; DN-134 verified free). |

> **Grounding + honesty (house rule #4 / VR-5 / G2).** This note **works a decision forward and
> recommends, ranked**; it does not take it (house rule #3). Its central finding is that this is the
> **construction mirror of DN-132's pattern side** — *the same* variant-aware `StructLayout` blocker, *the
> same* positional-`Ctor` Native-Equivalent target, and the enum emitter already emits the positional ctor
> those constructions call (`emit.rs:3113–3141`). So the honest deliverable is small and precise: **route
> the shared `struct_layouts` variant-awareness (DN-132/M-1089) into `visit_struct` and add the
> enum-variant construction arm** — while stating plainly the two real bounds (cross-nodule resolvability;
> the DN-104 construction seal) that DN-132's *pattern* side did not face.

---

## §1 The problem, precisely

`crates/mycelium-std-sys-host/src/lib.rs:63–65` — the production `ClockSource::wall_now` adapter:

```rust
Err(TimeErr::ClockUnavailable {
    reason: "OS wall clock read a time before the Unix epoch",
})
```

`TimeErr::ClockUnavailable { reason }` is a **named-field enum variant** built in **expression position**
(here inside a nested `match`). It solves the problem: *construct one variant of a sum type by field name,
so the construction reads against the field names not brittle positions.*

**Why it gaps today (verify-first, mitigation #14).** Confirmed against `dev @ 08d8fc21`:

1. `visit_struct` (`emit.rs:2324`) takes the **last path segment** as the ctor name (`ClockUnavailable`
   for `TimeErr::ClockUnavailable`; also correct for `Self::NotFound`), then resolves
   `struct_layout("ClockUnavailable")` (`emit.rs:2353`).
2. `struct_layout` reads the thread-local layout map built by `struct_layouts` (`transpile.rs:315–332`),
   which walks **`Item::Struct` only** — never `Item::Enum` variants. So an enum struct-variant's layout is
   **never present**, and `visit_struct` returns the "not an in-file single-ctor struct that emits" gap
   (`emit.rs:2353–2361`).
3. **The positional ctor it would call already exists.** `emit_enum` already lowers a `Fields::Named`
   variant to a positional `Ctor(T, U)` with the names dropped and recorded as a never-silent
   `NamedFieldDrop` sub-gap (`emit.rs:3113–3141`). So the *type surface* the construction needs is emitted;
   only the *construction site* can't find the field→index map.

This is **exactly** the DN-132/M-1089 blocker — DN-132 §5.1 names the same `struct_layouts`-is-struct-only
gap for the **pattern** side (`emit.rs:28`, the `map_pattern_inner` `Pat::Struct` arm). This note is its
**construction** mirror.

**The Mycelium-native answer (DN-111 / DN-110 taxonomy).** An **Idiomatic Remapping** (DN-111 §3.2 / DN-110
"Solution") — *not* a Native Equivalent, because Mycelium's `constructor` is **positional-only** by design
(value-semantic positional identity, ADR-003; there is no record/named-field construction surface — that is
DN-123's separate lever). A named-field variant construction remaps onto the positional ctor call in
**declaration order**:

```
Err(TimeErr::ClockUnavailable { reason: "…" })   ⟶   Err(ClockUnavailable("…"))
```

— identical in class to DN-132 P1's pattern remapping, and to the landed struct→positional-ctor rule the
struct emitter already applies (`emit.rs:3221–3236`). The field name `reason` is dropped and recorded
never-silently (G2), and the value is placed at `reason`'s declaration index.

---

## §2 Alternatives (real, ranked)

**Alt A — Positional-ctor remapping via the shared variant-aware `StructLayout` (RECOMMENDED).** Populate
`struct_layouts` from `Item::Enum` `Fields::Named` variants too (the *same* change DN-132/M-1089 makes for
the pattern side), then add an enum-variant construction path to `visit_struct` that resolves each named
field to its declaration index and emits the positional `Ctor(args…)`. Reuses the existing field-resolution
loop; zero kernel growth.

**Alt B — A record/named-field construction surface.** Give Mycelium a first-class `Ctor { a: v }`
construction grammar (the DN-123 records lever, construction side). This is a *genuine language feature*,
not a remapping — it would let named-field construction survive as named. But it grows the grammar + checker
+ eval for a payoff DN-123 already scopes as its own lever, and it is **not needed** to close the residual
(the positional remapping is faithful under positional identity, ADR-003). YAGNI/KC-3 decline for *this*
gap; retained as DN-123's separate decision.

**Alt C — Status quo (per-site gap).** Leave every expression-position struct-variant construction an honest
gap. Closes nothing; `std-sys-host` stays ≤ 5/6.

### §2.1 Objective function (criteria table)

| Criterion (weight) | Alt A (positional remap) | Alt B (record surface) | Alt C (gap) |
|---|---|---|---|
| **Closes `std-sys-host` residual #2** (×3) | **Yes** (3) | Yes (3) | No (0) |
| **Native / idiomatic (DN-111)** (×3) | **Idiomatic Remapping onto existing positional `Ctor`** (3) | Native Equivalent, but a *new* surface (2) | n/a (0) |
| **Kernel cost (KC-3)** (×3) | **Zero — `HashMap` population + emit arm** (3) | **High — new grammar + checker + eval** (0) | Zero (3) |
| **DRY with DN-132/M-1089** (×2) | **Shares the exact `struct_layouts` population** (2) | Parallel surface (0) | n/a (1) |
| **Faithfulness / VR-5** (×2) | **Faithful under positional identity (ADR-003)** (2) | Faithful (2) | n/a (2) |
| **Corpus "Other" reach** (×1) | Broad — every struct-variant construction (1) | Broad (1) | None (0) |
| **Weighted total** | **32** | 17 | 9 |

**Recommendation (ranked): Alt A ≫ Alt B ≫ Alt C.** Alt A wins on the same grounds DN-132 P1 won the
pattern side: it is an Idiomatic Remapping onto machinery that **already exists**, it grows nothing, and it
**shares the identical `struct_layouts` population** DN-132/M-1089 already specifies — so building both the
pattern arm and the construction arm off one population change is strictly DRY. Alt B (a real record
construction surface) is the *right* answer for the DN-123 records lever, but it is a language feature for a
different decision, not this residual's fix — declined here on KC-3/YAGNI, deferred to DN-123.

---

## §3 The emission spec (what M-1093 builds)

1. **Shared `StructLayout` variant-awareness (coordinated with DN-132/M-1089).** `struct_layouts`
   (`transpile.rs:315`) additionally walks `Item::Enum` variants with `Fields::Named`, inserting each under
   its **emitted ctor name** (the variant ident) with the same `Vec<Option<String>>` field-name layout.
   *This is one population change serving both the M-1089 pattern arm and the M-1093 construction arm* —
   whichever leaf lands first adds it; the other reuses it (see §7 coordination).
2. **`visit_struct` enum-variant path (`emit.rs:2324`).** The method already takes the last path segment as
   the ctor name (correct for `Enum::Variant` and `Self::Variant`). With the variant-aware layout present,
   `struct_layout(ctor_name)` now resolves for an enum struct-variant, and the **existing** field-resolution
   loop (`emit.rs:2362–2382`) maps each named field to its declaration index and builds the positional args
   — so the construction emits `Ctor(args…)` with **no** change to the loop itself.
3. **Never-silent refusals (G2):** a ctor name with no resolvable layout → the existing gap; a **missing**
   field for a declared index → the existing partial-constructor gap (`emit.rs:2372–2380`); a **duplicate**
   field name → a new never-silent refusal; `..rest` struct-update → the existing "no record-update surface"
   gap (`emit.rs:2328–2333`). Field name-drop is recorded as a `NamedFieldDrop` sub-gap, mirroring the enum
   emitter.
4. **Field-order canonicalization** to **declaration order** (inherits DN-123 OQ-1) — the args are emitted
   in the variant's declared field order regardless of source write-order, matching the positional ctor's
   arity/order.

**Guarantee tag:** `Declared` until built; **`Empirical`** once `myc check`-clean and differential-witnessed
(three-way, DN-26) on the construction targets; **no `Proven` claim** (VR-5).

---

## §4 Adversarial stress-test (VR-5 / house rule #4)

1. **Cross-nodule variant (the actual `std-sys-host` case).** `TimeErr` is imported from `std.time`
   (`lib.rs:6–9`), **not** defined in the `std-sys-host` nodule. Under the M-1006 **file-gated** vet loop,
   `named_field_emit_allowed` gates out an out-of-file ctor because emitting `ClockUnavailable(…)` would
   introduce an unresolved reference that poisons the file's `myc check` (the exact gate `emit_enum`
   applies, `emit.rs:3149–3159`). **Honest bound, stated plainly:** the construction emits **clean for a
   same-nodule variant**; for a **cross-nodule** variant it is either (a) **gated** under bare file-vet
   profiling, or (b) **clean on the real 6/6 port path**, where `std-sys-host` is a `.myc` nodule that
   **imports `std.time`** and the ctor resolves via DN-113 cross-nodule import resolution. This is a
   **dependency on DN-113**, not an incorrectness — the emission is faithful whenever the ctor resolves and
   an honest gap when it does not. *(This bound is new relative to DN-132's pattern side, which matches
   in-scrutinee and never introduces an out-of-file constructor reference — a real asymmetry, reported.)*
2. **DN-104 construction-side seal (OQ-3(b)).** DN-132 OQ-3(a) ruled that *matching* a sealed ctor stays
   allowed (matching is not construction). **Construction is the opposite:** a per-constructor visibility
   seal (DN-104/M-1027) exists precisely to forbid **constructing** a sealed variant outside its module. So
   an expression-position construction of a sealed ctor from outside its home nodule must be a **never-silent
   refusal** (G2), not an emission. This is the construction-side complement DN-132 flagged but did not have
   to enforce. Named as OQ-3(b), §6.
3. **Generic-enum variant (DN-131).** `Result::Ok(x)` / a generic enum's struct-variant composes with the
   impl-slot generics unchanged — the ctor is monomorphized like any other (M-673 dictionary-free); the
   layout is keyed by ctor name independent of type args. **Held** — no new interaction.
4. **Tuple-variant construction is out of scope.** `TimeErr::Overflow` (unit) and `E::V(a, b)`
   (tuple-variant) are `Expr::Path`/`Expr::Call`, not `Expr::Struct` — they already emit via the call/path
   arms and are not this note's residual. **Held** — scope is exactly the `Fields::Named` variant.
5. **`Self { .. }` vs `Self::Variant { .. }`.** Bare `Self { .. }` (a struct's own ctor) already works via
   the `raw == "Self"` branch (`emit.rs:2341–2352`); `Self::Variant { .. }` takes the last segment
   (`Variant`) and resolves via the variant layout — **held**, no special-case needed.
6. **Positional-identity faithfulness (ADR-003).** Dropping `reason`'s name and placing its value at the
   declared index is faithful *because* a Mycelium named-field product **is** its positional tuple
   (ADR-003; the exact ruling `emit.rs:3113–3141` already makes for the type side). **Held** — construction
   and declaration agree on order.
7. **Disconfirming-evidence check.** The task noted this is "distinct from DN-132/M-1089 which is the
   *pattern* side — this is the *construction* side." **Confirmed and sharpened:** they are distinct *arms*
   but share **one** blocker (`struct_layouts` is struct-only) and **one** target class (positional `Ctor`).
   The genuinely *new* content on the construction side is the two bounds above (cross-nodule resolvability;
   the DN-104 construction seal) — which the pattern side does not face. Reporting that the two are more
   *shared* than the framing implies (one population change serves both) is the honest call (house rule #4).

**Verdict:** Alt A survives. Its two honest bounds — cross-nodule resolvability (DN-113 dependency) and the
DN-104 construction seal (a never-silent refusal to *add*, OQ-3(b)) — are stated as dependencies/refusals,
not hidden, and neither is an incorrectness.

---

## §5 Open questions (never-silent)

- **OQ-1 — field-order canonicalization** (inherits DN-123 OQ-1): declaration order is the canonical arg
  order; a source out-of-order write is reordered, never emitted positionally as written.
- **OQ-2 — cross-nodule resolvability** (stress #1): the DN-113 dependency for constructing a variant whose
  enum lives in another nodule; clean on the real port path, gated under bare file-vet profiling.
- **OQ-3(b) — DN-104 construction seal** (stress #2): constructing a sealed ctor outside its home nodule is
  a never-silent refusal; the construction-side complement of DN-132 OQ-3(a).

---

## §6 Definition of Done (the gate for M-1093; what "Accepted" then "Enacted" require)

**Draft → Accepted** (maintainer / DN-review gate): the §2 ranked recommendation and §4 stress-test are
accepted or amended; the §1 finding (same blocker + target as DN-132's pattern side; the two new bounds) is
confirmed.

**M-1093 Done (Enacted basis):**

- **Shared population:** `struct_layouts` walks `Item::Enum` `Fields::Named` variants keyed by ctor name (or
  reuses M-1089's landing of the same change — §7).
- **Construction arm:** `visit_struct` resolves an enum struct-variant's layout and emits the positional
  `Ctor(args…)` in declaration order via the existing field-resolution loop.
- **Never-silent tests:** unresolved ctor → gap; missing field → gap; **duplicate** field → refusal;
  `..rest` → gap; a sealed cross-module construction → refusal (OQ-3(b)); field name-drop recorded.
- **Cross-nodule honesty:** a same-nodule variant emits clean; a cross-nodule variant is gated/clean per the
  DN-113 resolvability state (stress #1) — the test asserts the never-silent gate, not a fabricated
  reference.
- **Witness:** `myc check`-clean on the construction targets (a same-nodule fixture, plus the `std-sys-host`
  `OsClock::wall_now` body on the real port path); **differential-witnessed** (three-way, DN-26) before any
  `Empirical` upgrade past `Declared` (VR-5).

---

## §7 Build-leaf decomposition + the `std-sys-host` 6/6 path

- **Lane:** `crates/mycelium-transpile/src/{transpile.rs, emit.rs}` — **serial** (single-owner files). One
  leaf: **M-1093** (construction arm + never-silent refusals).
- **Coordination with DN-132/M-1089 (the shared `struct_layouts` population).** M-1089 (pattern arm) and
  M-1093 (construction arm) **share** the `Item::Enum`-variant population change (`transpile.rs:315`). Two
  clean options, both DRY: **(i)** land the population change with **whichever leaf lands first** and have
  the other consume it (a `depends_on` edge from the second to the first); or **(ii)** carry the population
  change as a tiny **shared precursor** both depend on. Recommend **(i)** with `M-1093 depends_on: [M-1089]`
  if M-1089 lands first (its pattern arm already specifies the exact population), else the reverse — the
  integrating parent picks the order and sets the single `depends_on` edge. Either way the population change
  is written **once** (mitigation #2/#7 — never duplicated across the two leaves).
- **Where it sits in 6/6:** **DN-134/M-1093 closes residual #2** — the `Err(TimeErr::ClockUnavailable {
  reason })` construction in `OsClock::wall_now` (`lib.rs:63–65`). With **DN-133/M-1092** closing residual
  #1 (the combinator chain in `OsEntropy::fill_bytes`) and the existing 5-capability punch-list,
  `std-sys-host` reaches **6/6**. M-1092 (`visit_method_call`) and M-1093 (`visit_struct`) touch different
  methods of `emit.rs` and can land in either order (no shared hunk); M-1093's only ordering edge is the
  shared population with M-1089.

---

## §8 Changelog

- 2026-07-12 — DN-134 created (**Draft**). Scopes expression-position struct-variant construction
  (`std-sys-host` #2 + corpus "Other" class) as the **construction twin** of DN-132's pattern side;
  recommends **Alt A, the positional-ctor remapping via the shared variant-aware `StructLayout`** over a new
  record construction surface (DN-123's separate lever) and status-quo gap; emission spec + adversarial
  stress-test (cross-nodule resolvability, DN-104 construction seal) + DoD; build leaf **M-1093** (shares the
  `struct_layouts` population with DN-132/M-1089). `Declared` — builds nothing; the DN-review gate/maintainer
  ratifies. FLAGs Doc-Index/CHANGELOG/issues rows for the integrating parent.
