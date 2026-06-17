# Research Record 08 — Cross-Language Stdlib Comparison & "Honest Stdlib" Prior Art (RFC-0016 §7)

> **What this file is.** A durable record discharging **RFC-0016 §7's standing pre-ratification grounding
> obligation**: the cross-language standard-library comparison (Rust / Python / Go / OCaml–Haskell *module
> sets*) and the "honest stdlib" prior art (refinement-typed / verified / effect-tracked standard
> libraries), traced into the evidence base so the **RFC-0016 module taxonomy** (§4.3/§4.4) and its
> **per-op contract** (§4.1, C1–C6) rest on checked precedent rather than assertion. Conducted 2026-06-17
> as part of the maintainer's RFC-0016 ratification (DN-07; M-501, #149). Findings are labeled
> **T8.1–T8.7** (continuing the T0–T7 scheme). This record is the **§7 discharge** named in DN-07 §4.

## Scope

RFC-0016 §7 lists the prior art the standard-library design leans on and explicitly defers the *traced*
comparison to a Research Record (this file). Two governing questions:

1. **Module set (the breadth/orthogonality target).** What do mature standard libraries put in their `core`
   vs `std` rings, and how do they organize the common surface (collections / text / math / io / time /
   rand / cmp / fmt)? This grounds the RFC-0016 Tier-B "table-stakes" taxonomy (§4.4) and the Ring 0/1/2
   layering (§4.2).
2. **Honesty (the differentiator).** Which standard libraries make accuracy / partiality / effects part of
   the *typed, checked* interface (rather than erased or silently coerced)? This grounds RFC-0016's §4.1
   contract — never-silent (C1/G2), honest per-op guarantee tags (C2/VR-5), declared/bounded effects
   (C6/RFC-0014) — and the §8-Q5 differential bar.

Seven precedents, split into the **module-set comparison** (T8.1–T8.4) and the **"honest stdlib"** prior
art (T8.5–T8.7).

## Part A — cross-language module-set comparison

### T8.1 — Rust (`core` / `alloc` / `std`; traits; `Result`/`Option`; `no_std`)
Rust splits its library into a **three-ring** stack: `core` (no allocator, no OS — pure language support:
`Option`/`Result`, `cmp`/`Ord`, `iter`, `fmt`, `mem`, numeric primitives), `alloc` (heap collections:
`Vec`/`String`/`BTreeMap`), and `std` (OS-facing: `fs`/`net`/`io`/`time`/`thread`/`sync`). Errors are
**values** (`Result<T, E>`/`Option<T>`), never exceptions; the `?` operator makes propagation the floor.
Organization is **trait-based** (coherent, orphan-ruled). **Maps to RFC-0016 §4.2 + §4.4:** the closest
model for Mycelium's ring layering (Ring 0 `core`/prelude ≈ Rust `core`; Ring 2 the OS-facing commons ≈
`std`), the errors-as-values ergonomics (C1), and the **`no_std` floor** that RFC-0016 §9 mirrors as the
"Ring 0 alone" embedded target. The `core`↔`std` split is the precedent for RFC-0016 §8-Q6's `std-sys`
carve-out (Rust isolates OS/`unsafe` reach in `std`, leaving `core` pure). *(Background knowledge; the ring
split + errors-as-values are definitional. Empirical/Declared.)*

### T8.2 — Python ("batteries included"; the breadth target *and* a cautionary case)
Python's stdlib is the canonical "batteries included" breadth reference: `collections`, `itertools`,
`functools`, `math`/`statistics`/`decimal`/`fractions`, `json`, `csv`, `datetime`, `random`/`secrets`,
`re`, `pathlib`/`os`, `io`, `unittest`. **Maps to RFC-0016 §4.4 (breadth) and is the explicit *cautionary*
case for §4.1:** Python is the breadth target for the Tier-B commons, but its **silent coercions** (mixed
int/float arithmetic; truthiness), **sentinel returns** (`str.find` → `-1`, `dict.get` → `None`), and
**exception-by-default** control flow are exactly what C1 (never-silent, explicit `Option`/`Result`)
forbids. Notably Python *did* split entropy honesty after the fact (`random` for simulation vs `secrets`
for security) — corroborating RFC-0016's `rand` design (seeded vs entropy generators *typed* distinct, the
RT3 declared effect). *(Background knowledge. Empirical/Declared.)*

### T8.3 — Go (small, orthogonal, explicit error returns)
Go's stdlib favors **small orthogonal packages** (`fmt`, `strings`/`strconv`, `bytes`, `sort`, `errors`,
`io`, `os`, `time`, `math/rand`, `encoding/json`, `testing`) with **explicit `(value, err)` returns** — no
exceptions for ordinary failures. `testing` ships *in* the stdlib (table-driven tests as a first-class
expectation). **Maps to RFC-0016 §4.4 + the §4.1 ethos:** the orthogonality + explicit-error posture that
the contract codifies, and the precedent for RFC-0016 shipping **`testing`** as a first-class module
(M-534) rather than a third-party add-on. Go's `math/rand` vs `crypto/rand` split again corroborates the
typed entropy distinction. *(Background knowledge. Empirical/Declared.)*

### T8.4 — OCaml / Haskell (value semantics, totality posture, type-class/trait organization)
OCaml (immutable-by-default core; `Stdlib`/`Base`/`Core` with `option`/`result`; modules + functors) and
Haskell (`Prelude` + `base`: `Maybe`/`Either`, `Data.Map`, `Foldable`/`Traversable`, type classes; purity
with effects in `IO`) demonstrate **value semantics + a totality/partiality posture** in a real stdlib.
Haskell's community even maintains a "**partial functions are a wart**" discipline (`head`/`fromJust` are
discouraged; `safe`/`NonEmpty` exist precisely to make partiality explicit) — the cultural form of C1.
**Maps to RFC-0016 §4.1 (C1 partiality) + §4.2:** the immutable-by-default substrate Mycelium's value
semantics shares, the `option`/`result` errors-as-values floor, and the type-class/trait organization the
`cmp`/`convert`/`fmt` modules adopt. *(Background knowledge. Empirical/Declared.)*

**Module-set synthesis (→ RFC-0016 §4.4 Tier B).** Across the four, a stable **common spine** recurs:
collections, text/string, math, iter, error/option/result, io/serialize, fs, time, rand, cmp/convert, fmt,
testing — *exactly* the Tier-B taxonomy RFC-0016 §4.4 names. The taxonomy is therefore **grounded as
table-stakes**, not invented (the §8-Q1 "safe v0 floor" sits inside this consensus set). The *ring* split
(Ring 0 pure / Ring 2 OS-facing) is the Rust/`no_std` precedent; the **honesty contract** is what Mycelium
adds *on top* of this otherwise-conventional set (Part B).

## Part B — "honest stdlib" prior art (the differentiator)

### T8.5 — Refinement-typed / verified standard libraries (Liquid Haskell, F\*/Low\*, Idris, Agda/Coq/Lean)
The closest prior art for a *standard library whose interface carries checked guarantees*: **Liquid
Haskell** refines `base` types with logical predicates discharged by an SMT solver (e.g. a `len`-indexed
list, safe-indexing preconditions); **F\*** (and Low\*/KaRaMeL → verified C in HACL\*) gives a stdlib whose
functions carry **pre/postconditions + effect labels** verified before extraction; **Idris** carries
totality + dependent indices in its prelude; **Agda/Coq/Lean** ship *fully proof-carrying* standard
libraries (`stdlib`/`mathlib`) where a lemma's statement *is* its checked contract. **Maps to RFC-0016
§4.1-C2 + §8-Q5:** these establish the pattern Mycelium's per-op **guarantee tag** generalizes — a contract
attached to an op and *checked*, with the crucial honesty caveat that the strong tag (`Proven`) is licensed
**only** when the side-conditions are discharged (the LiquidHaskell/F\* SMT-discharge step). Where Mycelium
differs (and must argue its own soundness, flagged novel): the tag is a **4-point lattice** (Exact ⊐ Proven
⊐ Empirical ⊐ Declared) that *degrades honestly* for ops with only empirical or declared bases (e.g. VSA
reconstruction) — these verified stdlibs have no "Empirical"/"Declared" rung; they are Proven-or-untyped.
That degradation rung is Mycelium's contribution, grounded in the verified-numerics "bound, not equality"
pattern (T8.6). *(Background knowledge of these systems; the refinement/verification properties are
definitional. Empirical/Declared — explicitly never Proven by this record.)*

### T8.6 — Verified numerics as the "bound, not equality" precedent (Gappa, Flocq, FPTaylor, Interval libs)
Already cited in Record 01 / ADR-010: Gappa, Flocq, FPTaylor, Rosa/Daisy, Herbie, and interval-arithmetic
libraries establish that verified numerics prove **certified error bounds**, not bit-equalities — "an ideal
real spec + a proven bound." **Maps to RFC-0016 `numerics`/`swap`/`vsa` (Tier A) + the guarantee lattice:**
this is the prior art for `Proven`-with-a-checked-bound and, by contrast, for the **`Empirical`** rung
(Gaussian-approximate VSA capacity bounds that are *measured*, not proven) — the honesty distinction the
4-point lattice exists to record. Confirms RFC-0016's differentiator modules are the *library form* of an
already-grounded research base, not new claims. *(Traceable to Record 01 + ADR-010. Empirical/Declared.)*

### T8.7 — Effect-tracked / total standard libraries (Koka, Lean, Unison's abilities, Roc)
**Koka** tracks effects (incl. `div` divergence) in the type of every stdlib function — the precedent for
RFC-0016-C6 "declared/bounded effects on a signature" and RFC-0007's divergence-only `total`/`partial` bit.
**Lean** gates kernel reduction on totality; **Unison** models I/O via typed *abilities* and ships a
content-addressed standard library (the ADR-003 identity model RFC-0016-C4 inherits); **Roc** makes
platform effects explicit at the boundary. **Maps to RFC-0016-C4/C6 + §8-Q4:** effects-as-typed-interface
(the `io`/`fs`/`time`/`rand` declared-effect rows), content-addressed library identity (the whole §4.1-C4
posture, and why `runtime` can be *reserved vocabulary* addressed by hash until Phase 7), and the totality
posture the migration differential (§8-Q5) leans on. *(Background knowledge; effect-typing + content
addressing are definitional for these systems. Empirical/Declared.)*

## How the findings discharge §7 (→ RFC-0016 ratification)

- **The Tier-B taxonomy is table-stakes, grounded (T8.1–T8.4).** The common spine (collections/text/math/
  iter/error/io/fs/time/rand/cmp/fmt/testing) is the *intersection* of four mature stdlibs — RFC-0016 §4.4
  is the consensus set, and §8-Q1's "safe v0 floor" sits inside it. Not invented (G2).
- **The ring layering is the Rust/`no_std` precedent (T8.1).** Ring 0 pure / Ring 2 OS-facing ≈ `core`/`std`;
  the `std-sys` carve-out (§8-Q6) is the Rust `core`↔`std` isolation of OS/`unsafe` reach.
- **The honesty contract is the differentiator, and it has prior art (T8.5–T8.7).** Checked per-op
  contracts (refinement/verified stdlibs), certified *bounds* not equalities (verified numerics), and
  effects/identity as typed interface (Koka/Unison) each ground one clause of §4.1 (C2/C6/C4). Mycelium's
  **novel** contribution — the 4-point lattice with honest *degradation* (Empirical/Declared rungs absent
  from the Proven-or-untyped verified stdlibs) — is flagged as needing its own soundness argument
  (consistent with RFC-0006 §8-Q3's "grading + runtime certificates has no found precedent").
- **Cautionary cases sharpen C1 (T8.2).** Python's silent coercions / sentinel returns are the concrete
  anti-patterns the never-silent contract forbids; the `random`/`secrets` and `math/rand`/`crypto/rand`
  splits corroborate the typed-entropy `rand` design.

This record **discharges the RFC-0016 §7 obligation** (DN-07 §4): the taxonomy + contract are now traced to
prior art in `research/`, clearing the standing grounding gate before the maintainer flips RFC-0016
`Draft → Accepted`.

## Uncertainty register

- T8.1–T8.7 are checked against **background knowledge** of these widely-used, stable systems; their
  load-bearing properties (ring splits, errors-as-values, refinement/verification, effect typing, content
  addressing) are **definitional / well-established**, not version-specific. No primary docs or source trees
  were fetched in this environment. Every finding is tagged **Empirical/Declared — never Proven** (VR-5).
- The record grounds the taxonomy's *external* prior art; it does **not** upgrade any Mycelium guarantee.
  The genuinely novel parts (the 4-point honest-degradation lattice; grading + runtime certificates) are
  flagged as carrying **no found precedent** and owing their own soundness argument — recorded, not papered
  over (the planning analogue of G2).
- Where a comparison would benefit from a precise module-by-module table per language, that is left as a
  future deepening; the consensus *spine* (the load-bearing claim for §8-Q1) does not depend on it.

## Changelog
- **2026-06-17 — Created (discharges RFC-0016 §7).** Traces the cross-language stdlib module-set comparison
  (Rust `core`/`alloc`/`std`; Python batteries-included + cautionary; Go small/orthogonal/explicit-error;
  OCaml/Haskell value-semantics/totality) as **T8.1–T8.4**, and the "honest stdlib" prior art
  (refinement-typed/verified stdlibs — Liquid Haskell/F\*/Idris/Agda-Coq-Lean; verified numerics as the
  "bound not equality" precedent; effect-tracked/content-addressed stdlibs — Koka/Lean/Unison/Roc) as
  **T8.5–T8.7**, into the evidence base. Grounds the RFC-0016 §4.4 Tier-B taxonomy (table-stakes
  consensus), the §4.2 ring layering + §8-Q6 `std-sys` split (Rust `core`/`std` precedent), and the §4.1
  honesty contract (C2/C4/C6), while flagging the 4-point honest-degradation lattice as Mycelium's novel,
  precedent-free contribution owing its own soundness argument. Discharges DN-07 §4's standing §7 gate.
  Findings tagged Empirical/Declared, never Proven (VR-5). Append-only.
