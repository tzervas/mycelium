# Design Note DN-128 — Mycelium's Native Answer to the Problem Rust `#[derive(Debug/Clone/PartialEq/Default/…)]` Solves: A Standard-Derive Lowering Library over the DN-54 `derive` Facility

| Field | Value |
|---|---|
| **Note** | DN-128 |
| **Status** | **Accepted** (2026-07-12, ratified under explicit maintainer delegation — mirrors the DN-115/117/118/122/123/124/125/126/127 precedent). Was **Draft** (2026-07-12, same day). **Accepted, not Enacted** (house rule #3) — **builds nothing** yet; every mechanism/tag stays `Declared`/unbuilt until the FLAGGED build issue (M-1086, minted at this close-out) lands and is differential-witnessed. Does not edit `crates/**`; `Doc-Index.md`/`CHANGELOG.md`/`issues.yaml` are applied by this ratification's integration close-out (recorded here, append-only). |
| **Ratification basis (recorded verbatim, 2026-07-12)** | Mycelium's native answer to `#[derive(...)]` is a **standard-derive lowering LIBRARY** — per-derive `lower` rules (DN-54, already landed/Accepted) as **structural folds** over the type's fields/variants (Native Equivalent, DN-111): `Clone` = a value-semantics identity no-op (ratified: the transpiler should *drop* `#[derive(Clone)]` as a satisfied no-op, not generate a trivial impl); `PartialEq`/`Eq` = field-wise `cmp.eq` conjunction; `Hash` = field-wise `hash.blake3` fold; `PartialOrd`/`Ord` = lexicographic `Ord3` fold; `Debug` = structural render via DN-127's `Show`; `Default` = field-wise `init()` via DN-129's `Init`. **A derived total `Eq` over a `Float` field is REFUSED never-silently** (NaN/ADR-040 — a derived `Eq` must use `flt.eq`'s partial semantics, matching Rust's own `derive(Eq)` refusal for `f64`). The load-bearing **OQ-1 (does a `lower` RHS have field reflection?) stays honestly OPEN** — Alt C (compiler-internal field-walk composed by `lower` rules) is ratified as the recommendation precisely because it survives either OQ-1 answer, over Alt A (pure `.myc` library, contingent on OQ-1 resolving positive) and Alt B (compiler-internal only, loses standard/user uniformity). Zero kernel growth beyond the already-landed DN-54 facility (KC-3). `Debug`/`Default` rules are explicitly sequenced **after** DN-127/DN-129 land; the dependency-free derives (`Eq`/`Ord`/`Hash`/`Clone`) are not blocked. Gate PASS clean — ratified on the merits under maintainer delegation; this note's own reasoning (§1–§6) is not re-litigated, only executed and recorded (VR-5). |
| **Decides (proposes, for ratification)** | Mycelium's **native solution** to the problem Rust `#[derive(Debug, Clone, PartialEq, Default, …)]` solves — *auto-generating the mechanical, structural impls of standard traits* — is a **standard-derive lowering LIBRARY**: a set of `lower StdDerive[T] = <L0 RHS>` rules (the DN-54 facility, already landed) that generate the impls, plus the transpiler mapping `#[derive(D)]` → `derive D for T`. The **facility exists** (DN-54, `lower`/`derive` active, RHS→L0 elaboration landed); the **library of standard rules is unbuilt** — the transpiler today **drops** every `#[derive(...)]` as a `DeriveAttr` sub-gap (`emit.rs:2284/2406/2513/3003`). This note designs that library, per-derive, as **Native Equivalent**. |
| **Native-solution class (DN-110/DN-111)** | **Native Equivalent** — Mycelium's `derive`/`lower` (DN-38/DN-54) *is* the native construct for generative lowering; `#[derive(D)]` maps to `derive D for T` and each standard `D` is a `lower` rule producing explicit, content-addressed, `reveal`-able L0 (no opacity, KC-3-clean by DN-54 §6). The per-derive faithfulness is itself classified below (some are exact structural equivalents; `Debug`/`Default` depend on DN-127/DN-129). |
| **Feeds** | DN-34 §8.22 (the `DeriveAttr` drop-and-record sub-gaps + DN-99 register rows 3/50 "std-derive `lower` lib"); DN-54 (the generative-lowering facility this builds the standard rules on); DN-127 (`derive Debug`/`derive Show` target the `Show` render surface); DN-129 (`derive Default` targets the `Init` prelude trait); DN-99 rows 3 (derive-attr) + 50 (impl/derive-undefined-trait + std-derive `lower` lib). |
| **Grounds on** | DN-54 (Accepted — `lower Name[params] = <rhs>` + `derive Name for T` active; RHS→L0 elaboration + IL-grammar type-check + KC-3 growth-guard + acyclicity all landed, M-812/M-812-cont merge `29b0aed`, DN-75 audit); DN-38 §8.1 (`derive`/`reveal`/`via` naming); DN-55 (static specialization — the generated impls monomorphize away, zero kernel primitives); RFC-0019 (traits + phylum-wide coherence); the landed `cmp.eq`/`bytes.eq` prims (structural equality building blocks); KC-3; G2 (a underivable field is an explicit gap, never a fabricated impl); VR-5; KISS/YAGNI. |
| **Verified-against** | `@dev fa53dc46` / cited sites re-checked at `b36ebdbe`. |
| **Date** | July 12, 2026 |
| **Task** | Design-first; build FLAGGED §7 (recommend minting: the std-derive `lower` library + the transpiler `#[derive]`→`derive` mapping). |

> **Posture (transparency rule / VR-5 / G2).** Draft design note; claims tagged at basis (tree-facts
> `Empirical` + `file:line`; proposed rules `Declared`). Argues against its own recommendation (§6): where a
> derive is better handled by a hand-written impl or a compiler-internal rule than a user-`lower` rule, it
> says so.

---

## §1 The PROBLEM (not Rust's mechanism)

`#[derive(...)]` solves: *the compiler mechanically writes the obvious, structural impl of a standard trait
for a data type*, so a developer does not hand-write `Clone`/`PartialEq`/`Debug`/`Default` boilerplate. Rust's
mechanism is a proc-macro that expands to an `impl` at the token level, before type info.

Mycelium's **native** generative construct is already ratified and landed: **`derive Name for T`** applies a
**`lower` rule** that produces an **explicit, content-addressed, `reveal`-able L0 artifact** (DN-38 §8.1;
DN-54 Accepted). This is strictly better than a token macro (DN-54 §2: typed AST/L0, not token substitution;
every use is `reveal`-able by construction). **So the native answer is not a new mechanism — it is the
*standard library of `lower` rules* on the existing one.**

**The gap, grounded (`Empirical @b36ebdbe`).** The **facility** is complete: DN-54 (`lib` header + DN-75
audit) confirms `lower Name[params] = <rhs>` and `derive Name for T` are **active keywords** with the full
pipeline landed — RHS elaboration to L0 (M-812-cont, merge `29b0aed`), IL-grammar RHS type-check, KC-3
growth-guard, cross-rule acyclicity, and the §7 verification discipline. But the **standard-derive library is
absent**: I re-verified (mit #14) that `crates/mycelium-transpile/src/emit.rs` **drops** every `#[derive(...)]`
attribute as a `Category::DeriveAttr` sub-gap at four sites — enum (`:2284`), struct (`:2406`), fn (`:2513`),
and impl-method (`:3003`) — "dropped non-doc attribute(s) on …" with no lowering emitted. DN-99 register row 3
(`derive-attr | idiom`) and row 50 (`transpiler-impl-undefined-trait+derive`) both name "**std-derive `lower`
lib**" as the unbuilt lever (`DN-99-…:66`, `:113`, `:262`). DN-34 §8.22 counted this class under-weighted (a
top-4 gap). **The library is the missing piece.**

---

## §2 Per-derive design (the standard set)

Each standard derive is a `lower` rule producing an explicit L0 impl. Faithfulness classified per-derive
(DN-111), because they are **not** uniform:

| Derive | Native rule (sketch) | Class | Depends on | Notes |
|---|---|---|---|---|
| **`Clone`** | `lower Clone[T] = <identity copy>` | **Native Equivalent (trivial)** | nothing | Value semantics (ADR-003) make every value a value — "clone" is the identity/structural copy the kernel already does. Arguably a **no-op** in Mycelium (see §6.1). |
| **`PartialEq`/`Eq`** | `lower Eq[T] = <field-wise cmp.eq ∧-fold>` | **Native Equivalent** | landed `cmp.eq`/`bytes.eq` prims | Structural equality = conjunction of field equalities; recursion over fields; the single-param `Eq` shape (DN-122 §13.1 admitted class). |
| **`Debug`** | `lower Debug[T] = <render Ctor(field,…) via Show>` | **Native Equivalent** (structural render) | **DN-127** (`Show`/`render`/`to_dec`) | The structural render `MyType{a, b}` → `bytes_concat("MyType(", render(a), ", ", render(b), ")")`. Blocked on DN-127's render surface. |
| **`Default`** | `lower Default[T] = <Init impl, each field init()>` | **Native Equivalent** | **DN-129** (`Init` trait; method ≠ `default`) | Field-wise default; blocked on DN-129's `Init` prelude trait (and the keyword-rename convention). |
| **`Hash`** | `lower Hash[T] = <field-wise hash.blake3 fold>` | **Native Equivalent** | landed `hash.blake3` prim | Structural hash via the landed `hash.blake3` (M-912); lower priority (YAGNI until a port needs it). |
| **`PartialOrd`/`Ord`** | `lower Ord[T] = <lexicographic Ord3 fold>` | **Native Equivalent** | landed `Ord3` (M-1080) | Lexicographic field comparison via the just-seeded `Ord3` prelude trait. |

**Common shape (KISS).** Every rule is a **structural fold over the type's fields/variants**: `Clone`=copy,
`Eq`=`∧`-fold of `cmp.eq`, `Debug`=`bytes_concat`-fold of `render`, `Default`=constructor of field `init()`s,
`Hash`=fold of `hash.blake3`, `Ord`=lexicographic `Ord3` fold. This is one code shape (a field/variant walk)
parameterized by the per-derive leaf operation — so the library is small and DRY.

---

## §3 Where the rules live — the crux (compiler-internal vs user-`lower`)

DN-54's `lower` facility is **user-extensible** — a user writes `lower Checksum[T] = …`. But the **standard**
derives (`Clone`/`Eq`/`Debug`/…) are a different question: should they be (a) **user-space `lower` rules**
shipped in a std prelude nodule, or (b) **compiler-internal** lowerings the elaborator knows structurally?

**Ranked alternatives + objective function:**

| Criterion (weight) | Alt A: **std-prelude `lower` rules** (`.myc` library) | Alt B: compiler-internal structural lowerings (Rust elaborator) | Alt C: hybrid — internal for the field-walk primitive, `lower` rules compose it |
|---|---|---|---|
| **KC-3 / DN-54 §6 (no kernel growth)** — high | ✓ user-space, zero kernel; the derive IS a `lower` | ✗ risks the elaborator special-casing per-type (kernel-adjacent growth) | ✓ if the field-walk is expressed in existing L0 |
| **Reflection requirement** — **load-bearing** | **needs field/variant reflection in `.myc`** (a `lower` RHS must enumerate `T`'s fields) — **does `.myc` have this?** (OQ-1) | ✓ the elaborator already has the typed AST — field enumeration is free | ✓ internal field-walk, `.myc` composes |
| **`reveal`-ability (DN-54 by-construction)** | ✓ | ✓ (a structural lowering is still explicit L0) | ✓ |
| **Uniformity with user derives** | ✓ standard = user, one mechanism | ✗ two mechanisms (built-in vs user) | partial |
| **Faithfulness / KISS** | ✓ if reflection exists | ✓ | medium (two layers) |

**The load-bearing open question (OQ-1, honest — I could not fully resolve it from the tree):** a `lower
StdDerive[T] = <rhs>` rule must, in its RHS, **enumerate the fields/variants of the parameter `T`** to build
the structural fold. DN-54 §3's allowed RHS references are "the type `T` being derived for, trait bounds,
const/width params" (`DN-54-…:127`–`129`) — it is **not evident that a `lower` RHS can reflect over `T`'s
field list** (the `Checksum` example lowers to a fixed L0 pattern, not a field-count-dependent one). If `.myc`
**lacks** field reflection in a `lower` RHS, then **Alt A is not expressible today** and the standard derives
must be **Alt B / Alt C** (compiler-internal field-walk), with the *user-facing surface* still `derive D for
T`. **I flag this rather than assume it (VR-5, house rule #4).**

**Recommendation (ranked, conditional on OQ-1): Alt C ≻ Alt B ≻ Alt A.** Alt C keeps the derive **surface**
uniform (`derive D for T`, a `lower`-rule *name*) while the **structural field-walk** — the one thing a `.myc`
RHS may not express — is a compiler-internal elaboration step the standard rules invoke. This gives DN-54's
`reveal`-ability and KC-3 posture **and** works whether or not `.myc` gains field reflection. Alt A (pure
`.myc` library) is the ideal *if* OQ-1 resolves positive (field reflection lands) — recommend re-evaluating
then. Alt B alone loses the "standard = user, one mechanism" uniformity DN-38 §8.1 prizes.

---

## §4 The transpiler mapping

Independent of §3's where-do-rules-live question, the transpiler change is the same and small:

- **Replace the drop-and-record** at the four `Category::DeriveAttr` sites (`emit.rs:2284/2406/2513/3003`):
  for a recognized standard derive `D ∈ {Clone, PartialEq, Eq, Debug, Default, Hash, PartialOrd, Ord}`, **emit
  `derive D for T`** (the native use-site) ahead of the type, iff the target trait/rule is available (DN-127's
  `Show` for `Debug`; DN-129's `Init` for `Default`; the landed `Ord3` for `Ord`). Otherwise **keep the honest
  gap** (record the `GapReason`, do not fabricate — G2), exactly as DN-122 §13.2 WU-A does for foreign-trait
  impls.
- **Emit↔check agreement** (mirror DN-122 §13.2 T-A3): the emitter's "is this a recognized, available standard
  derive" predicate must match what the checker will accept, so the emitter never ships a `derive D` the
  checker refuses.
- **Non-standard derives** (`#[derive(Serialize)]`, custom proc-macros) stay honest gaps — out of scope, named
  never-silent.

---

## §5 Kernel / std / transpiler split

| Layer | Owns | Grounding |
|---|---|---|
| **Kernel** | **Nothing new** (KC-3). Reuses `cmp.eq`/`bytes.eq` (equality), `hash.blake3` (M-912), the DN-54 `lower`→L0 elaborator. | §2; DN-54 §6 |
| **Elaborator (`mycelium-l1`, if Alt C)** | The compiler-internal **structural field-walk** primitive the standard `lower` rules invoke (field/variant enumeration → fold) — only if OQ-1 says `.myc` can't reflect. Still emits explicit, `reveal`-able L0 (DN-54 §5). | §3 Alt C |
| **Std (prelude nodule)** | The standard-derive `lower` rule *names* + their RHS (composing the field-walk with the per-derive leaf op); `impl` targets for `Debug`→`Show` (DN-127), `Default`→`Init` (DN-129). | §2, §3 |
| **Transpiler (`emit.rs`)** | `#[derive(D)]` → `derive D for T` mapping at the four `DeriveAttr` sites; availability check; emit↔check agreement; honest gap for non-standard/unavailable. | §4; `emit.rs:2284/2406/2513/3003` |

**Disjoint work-units** (`Declared` sizes):
- **WU-1 (resolve OQ-1):** determine whether a `lower` RHS can enumerate `T`'s fields; picks Alt A vs Alt C.
- **WU-2 (std-derive rules):** `Clone`/`Eq`/`Ord`/`Hash` first (no cross-note dep — landed prims + `Ord3`);
  then `Debug` (after DN-127) and `Default` (after DN-129).
- **WU-3 (transpiler mapping):** the four-site `#[derive]`→`derive` swap + emit↔check test + honest-gap
  retention. Property tests: (T-1) `#[derive(PartialEq)]` on a product struct emits `derive Eq for T` and
  `myc check`s clean; (T-2) `#[derive(Debug)]` emits + checks clean **after** DN-127 lands (red-but-honest
  before); (T-3) `#[derive(Serialize)]` stays an honest gap; (T-4) emit↔check agreement.

---

## §6 Adversarial stress-test (VR-5 / house rule #4)

1. **"`Clone` is a no-op in Mycelium — deriving it is dead weight."** *Attacked — argues against building a
   `Clone` rule at all.* *Result: CONCEDED, in part.* Under value semantics (ADR-003) every value already
   copies structurally; there is no `&`-vs-owned distinction to bridge, so a Mycelium `derive Clone` is either
   the identity or unnecessary. **Recommendation:** the transpiler should **drop `#[derive(Clone)]` as a
   *satisfied* no-op** (record it never-silently as "Clone is implicit under value semantics", *not* a gap and
   *not* a generated impl) — the honest native answer is "you already have it." This is a sharper answer than
   mechanically generating a trivial impl. `Copy` likewise.
2. **"Does a `lower` RHS actually have field reflection? (OQ-1)"** *Result: UNRESOLVED, flagged §3.* This is
   the load-bearing risk; the recommendation (Alt C) is **designed to survive either answer**, which is why it
   is ranked first over the more elegant Alt A.
3. **"`derive Debug`/`derive Default` create a hard dependency on DN-127/DN-129 — is that ordering real?"**
   *Result: HELD and sequenced.* WU-2 explicitly ships the dependency-free derives (`Eq`/`Ord`/`Hash`/`Clone`)
   first and gates `Debug`/`Default` behind DN-127/DN-129 landing. The transpiler keeps the honest gap for the
   gated ones meanwhile (no fabrication).
4. **"Structural `Eq` over floats is unsound (NaN ≠ NaN)."** *Attacked:* a field-wise `cmp.eq` fold over a
   `Float` field inherits ADR-040's partial equality. *Result: NARROWED — honest.* A derived `Eq` on a type
   with a `Float` field must use `flt.eq` (partial, NaN-false) and **carry that semantics transparently**
   (`Empirical`, ADR-040 §2.4) — it is **not** the same as Rust's `PartialEq` vs `Eq` split, and a derived
   *total* `Eq` over a float field must be **refused** (G2), matching Rust's own refusal to `derive(Eq)` for
   `f64`. Flagged as a per-field soundness gate in the `Eq` rule.
5. **"Variant/enum derives — does the field-walk handle sum types?"** *Attacked:* §2's rules sketch products;
   enums need a per-variant match. *Result: HELD but scoped.* The field-walk must handle both product (struct)
   and sum (enum-variant) shapes — the `Eq`/`Debug` folds become a `match self { V(a,…) => … }`. This is
   more work than the product case; WU-2 ships product-struct derives first, enum derives second (the
   match-pattern struct-variant gap DN-34 §8.22 finding 5 flagged is adjacent — coordinate).

---

## §7 Definition of Done + FLAGs

**Ratifying this note = accepting:** the standard-derive **library** is the native answer (a set of `lower`
rules over the landed DN-54 facility, transpiler-mapped from `#[derive]`), with the **per-derive faithfulness
classes** of §2, the **Alt-C compiler-internal-field-walk** recommendation (conditional on OQ-1), the
**`Clone` = satisfied-no-op** finding (§6.1), and the **float-`Eq` refusal** gate (§6.4). It enacts nothing.

**DoD for `Enacted`** (house rule #6): (1) OQ-1 resolved (field-reflection-in-`lower` determined → Alt A/C
picked); (2) the dependency-free derive rules (`Eq`/`Ord`/`Hash`) `myc check`-clean + three-way
differential-witnessed; (3) `Debug`/`Default` rules landed **after** DN-127/DN-129; (4) the transpiler
four-site `#[derive]`→`derive` mapping with T-1..T-4 green + emit↔check agreement + the `Clone`-no-op handling;
(5) an M-1006/DN-124 re-measure quantifying the `DeriveAttr`-class `checked_fraction` movement (`Declared`
until run).

**FLAGs (append-only; integrating parent applies — I do not edit these files):**
- **CHANGELOG.md** — Draft-DN row for DN-128.
- **docs/Doc-Index.md** — register DN-128 (Draft).
- **tools/github/issues.yaml** — mint (READ-ONLY; recommend, parent assigns; M-1081 taken by DN-125):
  - **`M-⟨new-d⟩` — resolve OQ-1 (field reflection in a `lower` RHS) + std-derive `lower` library**
    (`Eq`/`Ord`/`Hash`/`Clone` first; `mycelium-l1` + std prelude nodule). Feeds DN-99 row 50.
  - **`M-⟨new-e⟩` — transpiler `#[derive]`→`derive` mapping** (`emit.rs`, the four `DeriveAttr` sites) +
    emit↔check test + `Clone`-no-op handling. Depends on M-⟨new-d⟩.
  - `Debug`/`Default` derive rules — **blocked on DN-127 (M-⟨127-a⟩) / DN-129 (M-⟨129-c⟩)**; note the
    cross-note dependency in the issue bodies.
- **DN-99** — rows 3 + 50 should reference DN-128 as the owning design note (flag to the register owner).

**Applied at the 2026-07-12 ratification close-out (append-only note, original FLAGs above left
as-authored):** `Doc-Index.md` DN-128 row added at status **Accepted**; `CHANGELOG.md` carries the
ratification entry; **M-1086** minted (OQ-1 resolution + the std-derive `lower` library — `Eq`/`Ord`/
`Hash`/`Clone` first, dependency-free; `Debug`/`Default` rules sequenced after M-1090/DN-127 and
M-1091/DN-129 land, `depends_on: []` since the dependency-free subset is not blocked) — the transpiler
`#[derive]`→`derive` mapping is folded into the same tracking issue (WU-3). DN-99 rows 3/50
cross-reference recorded as a follow-up (not applied to `DN-99` itself here — FLAGged forward).

---

## Meta — changelog

- **2026-07-12 — Created (Draft, design-reasoner pattern).** Scopes Mycelium's native answer to
  `#[derive(Debug/Clone/PartialEq/Default/…)]`: a **standard-derive lowering LIBRARY** over the **landed**
  DN-54 `derive`/`lower` facility (the facility is complete — M-812/M-812-cont, DN-75 audit; the library is
  unbuilt — the transpiler **drops** `#[derive]` as a `DeriveAttr` sub-gap at `emit.rs:2284/2406/2513/3003`,
  DN-99 rows 3/50). Per-derive design + DN-111 faithfulness class (§2); the load-bearing OQ-1 (does a `lower`
  RHS have field reflection?) flagged honestly, with **Alt C (compiler-internal field-walk composed by `lower`
  rules)** recommended precisely because it survives either OQ-1 answer; the transpiler four-site
  `#[derive]`→`derive` mapping (§4). Adversarially held: **`Clone` is a satisfied no-op under value semantics**
  (drop-as-satisfied, not generate), a **derived total `Eq` over a `Float` field must be refused** (ADR-040
  NaN semantics), enum/sum-type field-walk scoped second. Cross-note deps: `Debug`→DN-127 (`Show`),
  `Default`→DN-129 (`Init`). All rules `Declared`; tree-facts `Empirical` + `file:line`. **Recommends, does
  not ratify** (house rule #3). Enacts nothing. (Append-only; VR-5; G2.)
- **2026-07-12 — Ratified Accepted (delegated ratification, gap-close-2 batch).** Status moved Draft →
  Accepted under explicit maintainer delegation (mirrors DN-115/117/118/122/123/124/125/126/127). The
  per-derive design, Alt C (compiler-internal field-walk) recommendation, `Clone`-satisfied-no-op finding,
  and float-`Eq` refusal gate are accepted as designed; OQ-1 stays an open build precondition. Builds
  nothing yet — **M-1086** minted for the implementation. Append-only; VR-5/G2.
