# Design Note DN-129 — Mycelium's Native Answer to the Problems Rust `Default` and `Error` Solve: Prelude-Seeded `Init` and Errors-as-Values with a `Show` Bound

| Field | Value |
|---|---|
| **Note** | DN-129 |
| **Status** | **Draft** (2026-07-12). Design-reasoner pattern (enumerate → evaluate → recommend-ranked → adversarially test); **recommends, does not ratify** (house rule #3 — Draft→Accepted is the maintainer's). **Builds nothing** — every mechanism/tag is `Declared`/unbuilt until landed and differential-witnessed. Does not edit `crates/**` or any integration-tier shared file. |
| **Decides (proposes, for ratification)** | (1) **The `Default` problem** (a canonical zero/identity value of a type) → a **single-parameter, param-only prelude trait** — proposed name **`Init`** (`Init[T] { fn init() => T; }`) — **seeded conditionally in the linked env exactly like `Fuse`/`Ord3`** (`checkty.rs:1358`/`1372`), Native Equivalent. (2) **The `Error` problem** (a value that describes a failure, is renderable, and can chain) → **errors-as-values** (`Result[A,E]`/`Option[A]`, already the ratified story) with a **thin marker trait `Fault`** requiring a `Show` render (DN-127) for the *renderable* obligation — Idiomatic Remapping, **not** a re-invention of the `std::error::Error` supertrait/`source()` machinery. Both traits are prelude-seeded by **one shared mechanism** this note specifies once (DN-127's `Show` and DN-128's derived impls ride it). |
| **Native-solution class (DN-110/DN-111)** | `Default` → **Native Equivalent** (a value-producing trait, exactly RFC-0019's single-param trait shape, monomorphized away by DN-55). `Error` → **Idiomatic Remapping** (Rust's `Error` *trait-object + `source()` chain* remapped onto Mycelium's already-native errors-as-values + a `Show`-bounded marker; the `dyn Error` boxing is deliberately **not** ported — ADR-033 dynamic dispatch is the one escape, not the default). |
| **Feeds** | DN-34 §8.22 finding 3 (the 14 reserved-word `impl Default` sub-issues + the 12 empty `impl std::error::Error for X {}` markers — "no native `Default`/`Error` trait wall", grep-confirmed); DN-128 (`derive Default` lowers to an `Init` impl — this note supplies the trait it targets); DN-127 (`Fault`'s `Show` bound; the shared prelude-seed mechanism); DN-122 §13 (the single-param param-only admitted class this reuses). |
| **Grounds on** | DN-122 §13.1 (single-param param-only foreign/prelude-trait impls check clean today — no new checker work); the landed `Fuse` seed (`checkty.rs:1358`–`1369`) + `Ord3` seed (`checkty.rs:1372`–`1381`, M-1080); RFC-0019 (traits, dictionary-free static resolution, phylum-wide orphan rule); DN-55 (static specialization — zero kernel primitives); `lib/std/error.myc` (errors-as-values ergonomics over `Result`/`Option` — the existing native error story); FLAG-core-4/M-535 (`StdError` = Rust-host machinery, **not** ported — no `.myc` equivalent); DN-02 (the three-test keyword/name gate); `default` = a lowercase-only lexer keyword (`token.rs:491`); KC-3; G2; VR-5; KISS/YAGNI. |
| **Verified-against** | `@dev fa53dc46` / cited sites re-checked at `b36ebdbe`. |
| **Date** | July 12, 2026 |
| **Task** | Design-first; build FLAGGED §7 (recommend minting: the `Init` + `Fault` prelude seeds + the shared seed helper). |

> **Posture (transparency rule / VR-5 / G2).** Draft design note; claims tagged at basis (tree-derived
> facts `Empirical` + `file:line`; proposed mechanism `Declared`). Argues against its own recommendation
> (§6). No sycophancy: where the honest answer is "no trait is needed" (Error), it says so.

---

## §1 The two PROBLEMs (not Rust's mechanisms)

**`Default`** solves: *give me the canonical, zero-argument, "empty/identity" value of a type* — `0` for a
counter, `""`/empty for a collection, a struct with each field defaulted. Rust's mechanism: a
`trait Default { fn default() -> Self; }` with a blanket `#[derive(Default)]`.

**`Error`** solves: *a value that describes a failure, can be rendered for a human, and can name its cause* —
so heterogeneous failures compose (`Box<dyn Error>`, `?`, `source()` chains). Rust's mechanism: a
`trait Error: Debug + Display { fn source(&self) -> Option<&(dyn Error)>; }` plus trait-object boxing.

**The gap, grounded (DN-34 §8.22 finding 3, `Empirical @b36ebdbe`).** The transpile corpus hits both walls
directly: **14 reserved-word sub-issues** are "almost entirely `impl Default for X { fn default(..) }`" that
fail because **Mycelium has no native `Default` trait** (`grep -n '"default"' checkty.rs` finds only the token,
`DN-34-…:1153`–`1159`); and **12 empty-marker `impl std::error::Error for X {}` gaps** hit "the identical
wall: no native `Error` trait either, `grep`-confirmed" (`DN-34-…:1160`–`1162`). I re-verified both (mit #14):
`checkty.rs @fa53dc46` seeds only `Fuse` and `Ord3` in the prelude; `lib/std/core.myc:48` FLAG-core-4 records
the `StdError` marker + blanket impl + `impl_std_error!` macro as **Rust-host machinery with no `.myc`
equivalent** (M-535) — confirming there is no native error trait to target.

---

## §2 The naming constraint — `default` is a taken keyword

`default` is an **active lowercase keyword** (`crates/mycelium-l1/src/token.rs:491`, `"default" =>
Tok::Default` — it declares the nodule/phylum ambient paradigm, RFC-0012; sugar-index row `default | active`).
Mycelium type/trait names are **capitalized** (`Fuse`, `Ord3`), and identifiers are case-distinct from the
lowercase keyword set — so a **trait** named `Default` would not itself collide, but the **method** `fn
default()` an impl must write **does** collide (that is exactly the §8.22 finding-3 reserved-word failure).
Two consequences:

1. The native trait's **method** cannot be `default` — it must be a non-keyword (e.g. `init`).
2. To avoid teaching a false parallel (DN-02 T-illuminate — `default` already *means* "ambient paradigm" in
   this language), I propose the **trait** name `Init` (not `Default`), method `init`: `Init[T] { fn init() =>
   T; }`. This sidesteps the keyword entirely and reads natively ("the initial value of `T`"). **Naming is
   DN-02-gated** — `Init`/`Zero`/`Blank`/`Fresh` are candidates; the maintainer picks at ratification. I do
   **not** claim `Init` is final; I claim `default` is **unavailable** and a rename is **mandatory** (that part
   is `Empirical`, grounded in `token.rs:491`).

---

## §3 `Default` → the `Init` prelude trait (Native Equivalent)

The native answer is a **value-producing single-parameter, param-only trait**, seeded in the prelude:

```
// prelude — trait shape
trait Init[T] { fn init() => T; }
// user / derived impl
impl Init[Counter] for Counter { fn init() => Counter{ n: 0 }; }
```

- **Why this is cheap (grounded).** `Init[T] { fn init() => T }` is **single-parameter and param-only** — its
  signature names only the trait's own param `T`, no foreign concrete type. That is exactly the DN-122 §13.1
  class the checker **admits today with no new checker code** (`register_instances` + the orphan rule;
  `checkty.rs:4335`–`4338`). The foreign-sig guard does not fire. `impl Init for LocalType` checks clean now.
- **Seeding mechanism (the shared spine — §5).** `Init` is seeded into the linked env **iff** some nodule
  declares an `impl Init[…] for …`, mirroring `Fuse` (`checkty.rs:1358`–`1369`) and `Ord3`
  (`checkty.rs:1372`–`1381`, M-1080) verbatim. Zero `use`, zero manifest emission (the DN-122 OQ-6
  prelude-seed resolution — KISS, least soundness surface: the coherence closure is `{this phylum, <prelude>}`,
  one uniform home, DN-112 §9).
- **Monomorphizes away (KC-3).** By DN-55 static specialization, `Init` adds **zero kernel primitives** — every
  `init()` call resolves to the concrete impl body at check/mono time. No dictionary, no runtime dispatch.

---

## §4 `Error` → errors-as-values + a `Show`-bounded `Fault` marker (Idiomatic Remapping)

Rust's `Error` trait bundles three obligations: **Debug** (structural), **Display** (human render), and
**`source()`** (cause chain), behind trait-object boxing. Mycelium already solves the *composition* problem
natively and differently:

- **Errors are values.** `lib/std/error.myc` is the ratified errors-as-values ergonomics layer over
  `Result[A,E]`/`Option[A]` (propagate/map/recover, RFC-0013/RFC-0014). An "error" is any `Data` type used as a
  `Result`'s `E` — there is **no need for a trait to *be* an error**. This is the native mechanism; the design
  question is only what, if anything, the *`Error`-as-abstraction* adds.
- **The renderable obligation** = DN-127's `Show`. A failure a human reads is a `Show`-able value. So the
  minimal native `Error` abstraction is a **marker trait `Fault` with a `Show` bound**:
  `trait Fault[T]: Show[T] {}` — "a `T` that is a fault carries a human render." An empty `impl Fault for
  MyError {}` (the 12-marker shape) then checks clean **because** `MyError: Show`, closing the §8.22
  empty-marker wall **without** re-inventing `source()`.
- **`source()` / cause chains are deliberately deferred (YAGNI).** Rust's `source()` exists to walk a
  `dyn Error` chain at runtime; Mycelium's errors-as-values compose by **explicit wrapping** (`E` can carry a
  `cause: OtherErr` field — a plain value, `reveal`-able), so the chain is in the data, not behind a trait
  object. A `Fault::source`-style method is **not** in the MVP (no evidence the corpus needs it; the 12 markers
  are all empty). If demand appears, it is an append-only extension, flagged now, not built.
- **`dyn Error` boxing is NOT ported.** Heterogeneous `Box<dyn Error>` is dynamic dispatch — ADR-033's single
  escape, not the native default. The native answer to "a function that can fail many ways" is a **sum type**
  `E = A(..) | B(..) | …` (`Result[T, E]`), which is more transparent (every failure mode named, G2) than an
  opaque boxed trait object. This is a **deliberate divergence**, stated plainly.

**So `Error` decomposes cleanly:** structural render → DN-128's `derive Debug`; human render → DN-127's `Show`;
the abstraction itself → a one-line `Fault: Show` marker; composition → the already-native sum-type `Result`.

---

## §5 The shared prelude-seed mechanism (specified once, used by `Show`/`Init`/`Fault`)

DN-127 (`Show`), this note (`Init`, `Fault`), and every future prelude trait need the **same** conditional
seeding. Rather than three ad-hoc copies, specify it once:

- A prelude trait `Tr` is defined once (a `TraitInfo` builder like `crate::fuse::prelude()` /
  `crate::ord3::prelude()`), and `Env::link` seeds it **iff** `self.nodules.iter().any(|(_, e)|
  e.traits.contains_key(Tr::TRAIT_NAME))` — the exact `Fuse`/`Ord3` conditional at `checkty.rs:1358`/`1372`.
- **Recommendation:** factor the repeated `if any-nodule-declares { traits.insert(name, prelude()) }` block
  into a small helper `seed_prelude_trait(&mut traits, &self.nodules, NAME, builder)` so `Show`/`Init`/`Fault`
  are one call each. This is a DRY refactor of landed code (KC-3-neutral — no new concept, just deduplication),
  **not** a new mechanism. The three traits then differ only by their `prelude()` builder + `TRAIT_NAME`.
- **`Fault`'s supertrait bound.** `Fault[T]: Show[T]` is a **supertrait** — check whether RFC-0019 stage-1
  admits a single-param trait with a single-param supertrait bound. **Honest open item (OQ-2):** if supertrait
  bounds are not yet in the trait checker, the MVP `Fault` is a **bare marker** (`trait Fault[T] {}`) and the
  `Show` obligation is enforced by the `derive`/convention, not the type system, until supertraits land. I
  flag this rather than assume it (VR-5).

---

## §6 Adversarial stress-test (VR-5 / house rule #4)

1. **"Do we even need an `Init` trait — why not just a plain `fn counter_init() => Counter`?"** *Attacked
   (the YAGNI case against my own recommendation).* *Result: trait justified, narrowly.* A plain nullary
   constructor suffices for a *concrete* default, and for much code that is the simpler native answer (KISS —
   state it: **if a type only ever needs one hand-written default, write a fn, skip `Init`**). The trait earns
   its place **only** for (a) `derive Default` (DN-128 needs a *target* trait to lower into) and (b) generic
   code that is polymorphic over "any `T` with a default." Absent both, `Init` is YAGNI. So `Init` is
   recommended **because DN-128 requires it**, not as a universal mandate — an honest, bounded justification.
2. **"`Fault: Show` supertrait may not check."** *Result: HONEST OQ-2* (§5) — MVP degrades to a bare marker if
   supertraits aren't ready; not assumed.
3. **"Is dropping `source()` going to bite ports that rely on cause chains?"** *Attacked:* real Rust error
   enums do carry `#[source]` fields. *Result: HELD — the cause is carried as a value field, not lost.* A
   `#[source] cause: IoError` field maps to a plain `cause: IoError` `Data` field (transparent, `reveal`-able);
   only the *trait-object `source()` accessor* is dropped, and its information is strictly *more* visible as a
   named field. No data loss (G2). If a generic "walk the cause chain" API is later needed, it is additive.
4. **"`Init` naming — is `Init` actually better than reusing `Default` capitalized?"** *Attacked:* `Default`
   (capitalized trait) might not collide with `default` (keyword). *Result: NARROWED.* The **trait** name might
   be free, but the **method** `default` is definitely taken, and DN-02 T-illuminate warns against a trait whose
   name echoes an unrelated keyword's meaning (ambient paradigm). `Init`/`init` is cleaner, but this is a
   **naming preference, DN-02-gated, maintainer's call** — the load-bearing claim is only that `default` (the
   method) is unavailable (`Empirical`, `token.rs:491`).
5. **"Two new prelude traits inflate the prelude — soundness surface?"** *Result: HELD.* Each is single-param
   param-only (DN-122 §13.1 admitted class), conditionally seeded (present only if used), monomorphized away
   (DN-55). The coherence closure stays `{this phylum, <prelude>}` — no cross-phylum import, no diamond. The
   soundness argument is DN-122 §13.1's, unchanged.

---

## §7 Definition of Done + FLAGs

**Ratifying this note = accepting:** (1) `Default`-problem → a prelude-seeded single-param `Init` trait
(name DN-02-gated; method ≠ `default`); (2) `Error`-problem → errors-as-values + a `Show`-bounded `Fault`
marker (no `source()`, no `dyn Error` boxing in the MVP — deliberate divergences, stated); (3) a shared
`seed_prelude_trait` helper factored from the landed `Fuse`/`Ord3` seeds. It enacts nothing.

**DoD for `Enacted`** (house rule #6): (1) `Init` + `Fault` prelude builders + the shared seed helper in
`mycelium-l1`, with `Fuse`/`Ord3` migrated to the helper (DRY, behavior-identical, tests green); (2) a
T-B1-style visibility test — `impl Init for LocalType` and `impl Fault for MyErr` check clean with no `use`;
(3) OQ-2 resolved (supertrait bound admitted, or `Fault` shipped as a bare marker with the boundary recorded);
(4) DN-128's `derive Default` witnessed lowering into an `Init` impl (the cross-note gate).

**FLAGs (append-only; integrating parent applies — I do not edit these files):**
- **CHANGELOG.md** — Draft-DN row for DN-129.
- **docs/Doc-Index.md** — register DN-129 (Draft).
- **tools/github/issues.yaml** — mint (READ-ONLY here; recommend, parent assigns; M-1081 taken by DN-125):
  - **`M-⟨new-c⟩` — `Init` + `Fault` prelude traits + shared `seed_prelude_trait` helper** (`mycelium-l1`
    `checkty.rs`/`fuse.rs`/`ord3.rs`-parallel modules). Blocks DN-128's `derive Default`.
  - **OQ-2 (supertrait bound for `Fault: Show`)** — track as a residual; MVP degrades to a bare marker.
- **DN-02** — the `Init`/`Fault` names need a T-map/T-illuminate/T-learn pass (flag to the lexicon owner; do
  not finalize here).

---

## Meta — changelog

- **2026-07-12 — Created (Draft, design-reasoner pattern).** Scopes Mycelium's native answers to the
  `Default` and `Error` problems (DN-34 §8.22 finding 3: 14 `impl Default` reserved-word + 12 empty
  `impl std::error::Error` gaps, no native trait for either — grep-confirmed). `Default` → a prelude-seeded
  single-param `Init` trait (Native Equivalent, method ≠ the taken `default` keyword, `token.rs:491`), seeded
  like `Fuse`/`Ord3` (`checkty.rs:1358`/`1372`, M-1080). `Error` → errors-as-values (the ratified
  `Result`/`Option` story, `error.myc`) + a `Show`-bounded `Fault` marker (Idiomatic Remapping — `source()`
  and `dyn Error` boxing deliberately not ported, causes carried as transparent value fields). Specifies one
  shared `seed_prelude_trait` helper (DRY refactor of landed seeds) that `Show`/`Init`/`Fault` all ride.
  Adversarially held; `Init` justified **narrowly** (earns its place only via DN-128's `derive Default` +
  generic default-polymorphism, else a plain fn is the KISS answer). OQ-2 (supertrait bound) an honest
  residual. Names DN-02-gated. All mechanism `Declared`; tree-facts `Empirical` + `file:line`. **Recommends,
  does not ratify** (house rule #3). Enacts nothing. (Append-only; VR-5; G2.)
