# Design Note DN-138 — Making Primitive/Std-Type Derive-Instances Available in a Freshly-Transpiled File (the DeriveAttr-class top unblock)

| Field | Value |
|---|---|
| **Note** | DN-138 |
| **Status** | **Accepted** (2026-07-13 — strict 9-criterion DN-review gate; the design is ratified, the §4.4 A-vs-B call **resolved to Alt A** on a verified premise. Was **Draft** (2026-07-13); the Draft history stands unchanged below, append-only per house rule #3). Design-only — **builds nothing**; every mechanism/tag stays `Declared`/unbuilt and this note is **not `Enacted`** until the FLAGGED build issues (§8, minted by the integrating parent) land and are differential-witnessed against the real `myc check` oracle. Does **not** edit `crates/**`, `lib/**`, `CHANGELOG.md`, `docs/Doc-Index.md`, or `tools/github/issues.yaml` — those are FLAGGED for the integrating parent (§9). Ratification record: §10. |
| **Decides (proposes, for ratification)** | *How* to make the primitive/std-type derive-instances (`Show`/`Init`/`Ord3` for `Binary{N}`/`Bool`/`Bytes`, and the `eq`/`hash` prim routings) **available in a freshly-transpiled `.myc` file**, so `emit/derives`' shared field-eligibility gate can be extended to accept primitive/std fields and the six landed derive rows compose real, `myc check`-clean code. Recommends **Alt A — prelude-instance-seed the resolution fact** (extend the landed `PRELUDE_TRAIT_SEEDS` spine with a parallel, conditional `PRELUDE_INSTANCE_SEEDS` carrying coherence-key + method-signature only, bodies staying in `lib/std`) over **Alt B — auto-import** (`use std.fmt`/`use std.cmp`) and **Alt C — bespoke resolver**. |
| **Feeds** | `docs/planning/DN-136-phase2-bulk-gap-close-worklist.md` §3 (DeriveAttr = 14.5%, the #2 class, 139 gaps) — the specific unblock that lets the six landed `emit/derives/*` rows (`show`/`init`/`clone_copy`/`eq`/`ord`/`hash`) accept the real corpus derive structs (`CheckError`/`CtorInfo`/`EvaluatorOpts` — `String`/`Vec`/`u64` fields) instead of gapping every non-user-type field. |
| **Grounds on** | DN-128 (Accepted — the standard-derive lowering library; §2 per-derive rules); DN-129 §5 / M-1091 (`PRELUDE_TRAIT_SEEDS` — the conditional prelude-trait seeding spine this extends, `crates/mycelium-l1/src/checkty.rs:2282`, `crates/mycelium-l1/src/preseed.rs`); DN-127 / M-1090 (the landed `Show` primitive impls, `lib/std/fmt.myc:221/227/234`); DN-136 P1-a (the frozen `emit/derives` axis; `field_derive_eligible`, `crates/mycelium-transpile/src/emit/derives/mod.rs:99`); RFC-0019 §4.5 (phylum-wide coherence, width-erased `type_head`, `checkty.rs:305`); ADR-040 §2.3/§2.4 (Float/NaN — the derived-total-Eq/Ord refusal preserved); the landed prims `cmp.eq`/`bytes_eq`/`bool_eq`/`hash.blake3` (`checkty.rs:8879/10382`, `lib/std/content.myc:42`); KC-3; G2; VR-5; KISS/YAGNI. |
| **Verified-against** | `@dev 159417cc` (this worktree's base tip). All `file:line` anchors re-read at that tip (mit #14). |
| **Date** | July 13, 2026 |
| **Task** | Design-first; build FLAGGED §8 (recommend minting WU-1 lib/std authoring, WU-2 availability seed, WU-3 eligibility/routing; WU-4 Vec-recursive deferred). |
| **Definition of Done** | §7. In one line: **Accepted** requires the gate to confirm (a) the availability mechanism (Alt A recommended; Alt B a legitimate lighter near-term), (b) the **heterogeneity finding** (§3 — the six derives split into *trait-dispatched* Show/Init/Ord3 needing a resolvable instance vs *prim-routed* eq/hash needing only a compose route) as the design basis, (c) the **scope boundary** (increment 1 = scalar/`Bytes`/`Bool` fields at width ≤ 64; `Vec[T]`, narrow-width scalar, and hash-over-scalar deferred to increment 2 / §6), and (d) the soundness obligations (§5) as build gates. |

> **Posture (transparency rule / VR-5 / G2).** Draft design note; every claim tagged at its basis
> (tree-facts `Empirical` with `file:line`; proposed mechanism `Declared`). It argues against its own
> recommendation (§4.4) and, per VR-5, **corrects the task's own framing** where the codebase
> disconfirms it (§3): the "seed the instances" premise is right for three of the six derives and the
> wrong shape for the other two, and the "unblocks the whole class" expectation is tempered — `Vec`
> fields (present in every named corpus struct) still gap after increment 1.

---

## §1 The problem, precisely located (Empirical `@159417cc`)

**Five** of the six landed derive rows (`crates/mycelium-transpile/src/emit/derives/*.rs` —
`show`/`init`/`eq`/`ord`/`hash`; the sixth, `clone_copy`, is a satisfied value-semantics **no-op** —
`clone_copy.rs` imports only `{DeriveCtx, DeriveHandler, DeriveOutcome}`, never gates, and is
untouched by this design) each **refuse the whole derive** the moment a struct field is not a Named
user type. The single shared gate for those five is `field_derive_eligible` (`derives/mod.rs:99–110`):

```rust
pub(super) fn field_derive_eligible(mapped_ty: &str) -> bool {
    if matches!(mapped_ty, "Bool" | "Float" | "Bytes") { return false; }
    if mapped_ty.contains(['{', '(', '[']) { return false; }   // excludes Binary{N}, tuples, Vec[T]/Seq
    mapped_ty.chars().next().is_some_and(|c| c.is_ascii_uppercase())
}
```

**Why it excludes primitives (grounded, from the rows' own gap messages).** A freshly-transpiled
`.myc` file has no in-scope instance/function for the primitive's operation. `show::compose`
(`show.rs:39–48`) states it verbatim: a primitive field is *"a primitive repr with no ambient `Show`
instance in this file (`lib/std/fmt.myc`'s primitive impls live in a separate, unimported nodule)"*.
`init::compose` (`init.rs:29`), `eq::compose` (`eq.rs:116`), `ord::compose` (`ord.rs:89`), and
`hash::compose` (`hash.rs:74`) each carry the analogous refusal. So the real corpus derive structs —
`CheckError`, `CtorInfo`, `EvaluatorOpts`, all `String`/`Vec`/`u64` fields — gap on **every** field.
This is the single blocker for the DeriveAttr class (14.5%, #2, 139 gaps —
`docs/planning/DN-136-phase2-bulk-gap-close-worklist.md:94`).

The stated fix is: make the primitive/std-type instances **available** so `field_derive_eligible` can
accept them. This note designs the availability mechanism — and, per VR-5, first corrects two premises
in that framing that the codebase disconfirms.

---

## §2 Verify-first: what actually exists (Empirical `@159417cc`)

| Instance / prim | Exists? | Where | Guarantee (at the body) |
|---|---|---|---|
| `Show[Binary{64}]` | **yes** | `lib/std/fmt.myc:221` (`render = to_dec`, unsigned decimal) | `Exact` over its domain (DN-127) |
| `Show[Bytes]` | **yes** | `lib/std/fmt.myc:227` (identity render) | `Exact` |
| `Show[Bool]` | **yes** | `lib/std/fmt.myc:234` (ASCII word) | `Exact` |
| `Show[Float]` | **no — deliberate honest gap** | `lib/std/fmt.myc:239` (OQ-1) | refused, never fabricated |
| `Show[Seq]` / `Show[Vec]` | **no** | — | — |
| `Init[*]` (any) | **NONE** anywhere in `lib/**` | — | must be authored |
| `Ord3[*]` (primitive) | **NONE** in `lib/std` | — | must be authored |
| `cmp.eq` (`eq` prim, `Binary{N}`) | yes | `checkty.rs:10382` | `Exact` |
| `bytes_eq(Bytes,Bytes)=>Binary{1}` | yes | `checkty.rs:8879` | `Exact` |
| `bool_eq` (`Bool`) | yes | `lib/std/cmp.myc:37` (match-defined) | `Exact` |
| `hash.blake3 : Bytes -> Bytes` | yes | `lib/std/content.myc:42`, `checkty.rs:8905` | `Exact` (raw-bytes BLAKE3) |
| `Binary{N} -> Bytes` (raw) | **no** | only `to_dec`/`digit_byte` (decimal render, not raw bytes) | missing prim |

Two structural facts from the checker that shape every option below:

1. **Coherence keys are width-erased per head** (`type_head`, `checkty.rs:305–318`): `Ty::Binary(_)
   => "Binary"`. So the single `(Show,"Binary")` slot is already occupied by `Show[Binary{64}]` —
   there can be no separate `Show[Binary{8}]` (fmt.myc:211 documents exactly this). A `Binary{8}`
   field's `render` call resolves the instance by head `Binary` but then type-checks its `Binary{8}`
   argument against the instance method's `Binary{64}` parameter → **width mismatch, never silent**.
   *The corpus `u64` fields hit `Binary{64}` exactly and are covered; narrower widths need a
   `width_cast` first (deferred, §6).*
2. **Instance seeding must be conditional** (`checkty.rs:2310–2319`): an *unconditionally* present
   trait/instance trips `mono::is_already_monomorphic`'s trait-emptiness fast-path for **every**
   program, forcing even trait-free code through mono's slow specializing pass — a real regression,
   not a test artifact. The landed trait seeds are therefore inserted **iff** the nodule's own items
   need them. Any instance seed must inherit this conditional-on-need discipline.

---

## §3 The heterogeneity finding — the availability need is not uniform (VR-5 correction)

The task frames the whole class as one instance-availability problem. Verify-first against the six
rows' `compose` functions disconfirms that: the six derives split by **how a field's operation
resolves**, and only three of them actually need a resolvable *instance*.

| Derive (row) | `compose` emits per field | Resolves via | Primitive availability need |
|---|---|---|---|
| **Show** (`show.rs:57`) | `render(f)` | **`Show` trait instance** dispatch | a resolvable `Show[Binary{64}]`/`Show[Bytes]`/`Show[Bool]` |
| **Init** (`init.rs:36`) | bare `init()` (seed-from-expected) | **`Init` trait instance** dispatch | a resolvable `Init[Binary{N}]`/`Init[Bool]`/`Init[Bytes]` |
| **Ord** (`ord.rs:100`) | `cmp(p,q)` | **`Ord3` trait instance** dispatch | a resolvable `Ord3[Binary{N}]`/`Ord3[Bytes]`/`Ord3[Bool]` |
| **PartialEq** (`eq.rs:129`) | `eq_<FieldType>(p,q)` — a **top-level fn name** | deterministic fn resolution | **not an instance** — compose must route a primitive field to the `cmp.eq`/`bytes_eq`/`bool_eq` prim |
| **Hash** (`hash.rs:85`) | `hash_<FieldType>(p)` — a **top-level fn name** | deterministic fn resolution | **not an instance** — compose must route to `hash.blake3` (and scalar→`Bytes`, a **missing prim**, §6) |

The eq/hash rows *deliberately* avoid trait dispatch: `eq.rs`'s module doc (`eq.rs:22–45`) records
that there is **no landed `Eq`/`Hash` prelude trait**, and that self-declaring one inline collides
(`"duplicate trait declaration"` on a second struct; the `eq` method name shadows the `eq` prim). So
for eq/hash the "primitive instance" is a **compose-side routing to an already-landed prim**, not a
seed. `eq_Binary{8}` is not even a legal fn name (braces) — extending eligibility *without* routing
would make eq/hash compose calls to functions that do not and cannot exist.

**Consequence for the eligibility extension (the load-bearing design point):** `field_derive_eligible`
is **shared by the five field-gating rows** (`use super::field_derive_eligible` in each of
`show`/`init`/`eq`/`ord`/`hash`; `clone_copy` does not import it). Extending it to a bare
`bool`-accept of primitives would make **all five** attempt to compose over a primitive field —
correct for Show/Init/Ord3 (once the instance is available) but **broken for eq/hash** (calls to
nonexistent `eq_Binary{8}`/`hash_Binary{8}`). The extension must therefore be a **classification, not
a boolean** (§4.5), so each row routes correctly.

---

## §4 Alternatives for the availability mechanism (the DN's core decision)

Scope: this decision is about the **trait-dispatched** derives (Show/Init/Ord3), which genuinely need
a resolvable instance. eq/hash are handled by §4.5's compose routing regardless of which alternative
is chosen.

### §4.1 Alt A — prelude-instance-seed the resolution fact (RECOMMENDED)

Extend the landed `PRELUDE_TRAIT_SEEDS` spine (`preseed.rs`; `PreludeTraitSeed`) with a parallel
`PRELUDE_INSTANCE_SEEDS` array. Each entry seeds — **conditionally, iff a derive in this nodule needs
it** (§2 fact 2) — an *instance resolution fact*: the `(trait, type-head)` coherence key plus the
instance method's **signature** (`render: Binary{64} => Bytes`, `init: () => Binary{N}`,
`cmp: (Binary{N},Binary{N}) => Binary{8}`). It seeds **no body**: the real body lives and is verified
in `lib/std` (exactly as the trait seeds carry `TraitInfo` interface, not method bodies). So a
transpiled file's `render(field)` type-checks against the seeded fact; at link/eval the `lib/std` body
is the actual implementation.

- **Mechanism cost:** one new array + a `seed_instance_for_nodule` mirroring the existing
  `seed_for_nodule` (`preseed.rs:42`), including its never-silent redeclare-refusal. No new resolution
  path, no new kernel node (KC-3).
- **Namespace:** clean — only the instance fact becomes ambient, not the helper fns (`to_dec`,
  `digit_byte`, …). The transpiled user namespace is untouched.
- **Honesty (VR-5):** the seed is `Declared`-that-the-instance-exists; the body's guarantee
  (`Show[Bytes]` `Exact`, `Init[Binary{N}]`=0 `Exact`, …) lives with the body. **Hard obligation
  (§5):** the seeded signature must be pinned to the `lib/std` body by a differential test, else a
  check-passes/eval-fails divergence.

### §4.2 Alt B — auto-import the std instances

Have the transpiler emit `use std.fmt` / `use std.cmp` (etc.) into every transpiled nodule. The
instances resolve through the **existing** import + coherence machinery — literally how a hand-written
Mycelium file would obtain them.

- **Mechanism cost:** near-zero (emit header lines) — reuses import resolution entirely.
- **Transparency:** *best of the three* — the `use` line makes provenance visible in the emission; an
  agent/human sees exactly where each instance comes from (G2-ideal).
- **Costs:** (i) **namespace pollution** — importing `std.fmt` dumps `to_dec`/`digit_byte`/… into the
  transpiled file's scope, which can **shadow or collide** with the transpiled user code's own names
  (a Rust source fn named `render`/`to_dec` would clash — a real correctness hazard at scale); (ii)
  forces every transpiled file's `myc check` to resolve and type-check the **full** std nodule bodies,
  not just the needed facts; (iii) still cannot supply the instances that **do not exist yet** (Init,
  Ord3) — those must be authored either way.

### §4.3 Alt C — bespoke primitive-instance resolver

Special-case primitive-instance resolution inside the checker (a hardcoded fallback path when
`(trait, head)` is a known primitive). Rejected: it is the **most** mechanism (a second resolution
path parallel to the coherence machinery), the least KISS/KC-3, and the hardest to keep
`EXPLAIN`-able/never-silent (a resolution that "just works" with no visible seed or `use` is exactly
the black box rule #2 forbids).

### §4.4 Ranking + the argument against the recommendation (VR-5)

Objective function (criteria, weighted by the house rules):

| Criterion (house rule) | Alt A (seed fact) | Alt B (auto-import) | Alt C (resolver) |
|---|---|---|---|
| KC-3 minimal / reuse landed machinery (rule 5) | **strong** — extends the seed spine | strong — reuses imports | weak — new path |
| Soundness — instances are REAL, no fabrication (rule 1/§5) | strong (with the sig-pin gate) | strong | medium |
| Namespace safety (no shadowing user code) | **strong** | **weak** — pollutes/shadows | strong |
| Transparency / `EXPLAIN` / never-silent (rule 2, G2) | strong (seed is a named const) | **strongest** (`use` visible) | weak (hidden) |
| Confines ambient surface to what's needed | **strong** | weak (whole nodule) | strong |
| Build effort (near-term) | medium | **lowest** | high |
| **Rank** | **1** | **2** | 3 |

**The honest argument for Alt B over Alt A (against my own recommendation):** Alt B is *simpler to
build*, needs *no new checker code*, and is *more transparent* (the provenance is a literal `use`
line). If the transpiler already namespaces or mangles emitted user names such that the pollution
cannot collide (worth verifying before ruling it out), Alt B's shadowing hazard evaporates and it
becomes the KISS winner. I rank Alt A first because the shadowing hazard is *real today* (transpiled
names are surface-level) and because Alt A confines the ambient surface to exactly the derive-needed
facts — but a maintainer who values the visible-`use` provenance and accepts a name-collision guard
may legitimately pick Alt B. **Both are honest and never-silent; Alt C is not competitive.** This is a
genuine A-or-B call I am flagging for the gate, not a foregone conclusion.

> **GATE RESOLUTION — Alt A (2026-07-13; strict DN-review gate).** The A-vs-B call turns on one
> verifiable fact the §4.4 argument flagged as "worth verifying before ruling it out": *does the
> transpiler mangle/namespace emitted user names, so an auto-`use std.fmt`/`std.cmp` cannot collide
> with user code?* **Verified against the code at `@159417cc`: it does not.** The transpiler emits user
> identifiers **verbatim into a flat file namespace** — no mangling, no per-nodule prefix, no
> auto-rename: `emit.rs:1049` renders `fn {name}…` with only a `pub_prefix` (Empirical); `reserved.rs`
> states it outright — *"The transpiler has **no sanctioned renaming scheme** … a collision is
> **gapped** (`Category::ReservedWord`), never silently emitted or **auto-renamed** (G2/VR-5)"*
> (Empirical); and `emit.rs:2092/2108` records that the transpiler does **not** auto-emit `use std.*`
> and that "even a real `use std.cmp.ne;` import would additionally fail." **Therefore Alt B's
> namespace-pollution hazard is real, not hypothetical:** auto-importing `std.fmt`/`std.cmp` drops that
> nodule's free helper fns (`to_dec`, `digit_byte`, byte-compare helpers, …) into the same flat
> namespace as verbatim-emitted user names, and a Rust source with a fn/type of a colliding name
> produces a duplicate-declaration `myc check` **failure** the transpiler **cannot** deconflict (it
> gaps, it does not rename) — turning a would-be-clean derive back into a gap, strictly worse than the
> gap Alt B set out to close. **Alt A** confines the ambient surface to exactly the `(trait, head)` +
> signature resolution fact, polluting no free-fn name, and reuses the landed conditional
> `PRELUDE_TRAIT_SEEDS` spine. **Decision: adopt Alt A** (the DN's own Rank-1 recommendation, now
> confirmed on the verified premise, not asserted). Alt B is *not* selected — its simplicity is real
> but the collision guard it would require is unavailable in a verbatim-emitting transpiler, so the
> visible-`use` provenance does not offset the correctness hazard. Alt C remains non-competitive.

### §4.5 The `field_derive_eligible` extension — a classification, not a boolean

Replace the shared `field_derive_eligible(mapped_ty) -> bool` with a shared classifier
`field_derive_kind(mapped_ty) -> FieldDeriveKind` (DRY: one predicate; SoC: each row routes on the
kind — respecting DN-136's invariant that a row owns its compose and the driver is untouched):

| `FieldDeriveKind` | Matches | Show/Init/Ord3 route | eq route | hash route |
|---|---|---|---|---|
| `UserNamed` | leading-uppercase, no `{`/`(`/`[`, not a known primitive (current true-branch) | `render(f)`/`init()`/`cmp(p,q)` (unchanged) | `eq_<T>(p,q)` | `hash_<T>(p)` |
| `ScalarBinary` | `Binary{N}` | seeded instance (width ≤ 64; narrower → `width_cast`, **deferred §6**) | `eq` prim (`cmp.eq`) | **deferred §6** (no scalar→`Bytes` prim) |
| `BytesLike` | `Bytes` (from `String`/`str`/`[u8]`) | seeded `*[Bytes]` | `bytes_eq(p,q)` | `hash.blake3(p)` directly |
| `BoolLike` | `Bool` | seeded `*[Bool]` | `bool_eq(p,q)` | via `Bool→Bytes` render |
| `Float` | `Float` | **ineligible** — no `Show[Float]`; ADR-040 refuses total Eq/Ord | refused (ADR-040) | refused |
| `Deferred` | `Seq`/`Vec[T]`, tuples, other bracketed | **ineligible in increment 1** (`Vec` recursive → §6) | ineligible | ineligible |

The ADR-040 Float check that `eq.rs`/`ord.rs` already run **ahead** of the eligibility gate stays
exactly where it is (§5 point 3).

---

## §5 Soundness obligations (build gates — VR-5 / G2 / ADR-040)

These are non-negotiable acceptance conditions on the build, not optional polish:

1. **Seed-signature pinned to the `lib/std` body (Alt A).** Each `PRELUDE_INSTANCE_SEEDS` entry's
   method signature must be **differential-tested equal** to the corresponding `lib/std` instance's
   method signature. A seed that claims a signature the body does not provide is a check-passes /
   eval-fails divergence — the exact hazard the transparency rule exists to prevent. (Under Alt B this
   is automatic, since the real body *is* what resolves — a point in Alt B's favor.)
2. **Instances must be REAL and correct, tagged at basis.** `Show[Binary{64}]` renders unsigned
   decimal (`Exact` over its domain, DN-127); `Show[Bytes]`/`Show[Bool]` `Exact`; `Init[Binary{N}]` =
   all-zeros (`Exact`); `Init[Bool]` = `False` (`Declared` convention — matches Rust `Default::default`
   for `bool`); `Init[Bytes]` = empty (`Exact`); `Ord3[Binary{N}]` via `std.cmp::cmp{N}` (`Exact`);
   `Ord3[Bytes]` lexicographic byte compare (`Exact`, needs a byte-compare authored); `Ord3[Bool]`
   `False < True` (`Declared` convention). No instance may claim `Exact`/`Proven` past its body's
   checked basis (rule 1).
3. **Float stays refused, never seeded.** No `Show[Float]`/`Init[Float]`/`Ord3[Float]` seed; the
   `Float` head is never in `PRELUDE_INSTANCE_SEEDS`; the ADR-040 §2.3/§2.4 NaN refusal in
   `eq.rs`/`ord.rs` is preserved (a derived *total* Eq/Ord over `Float` remains an honest gap — NaN ≠
   NaN, NaN has no order position).
4. **Conditional-on-need seeding.** Instance seeds are inserted **iff** a derive in the nodule needs
   them (§2 fact 2), never unconditionally — else `mono::is_already_monomorphic` regresses for every
   trait-free program.
5. **Never-silent redeclare-refusal.** Reuse `preseed.rs`'s `redeclare_error` shape: a transpiled (or
   hand-written) file that both triggers the seed and declares the instance gets an explicit refusal,
   never a silent shadow (G2).

**Adversarial stress-test of the recommendation (Alt A) — verdict.** (a) *Coherence collision:* the
per-head, conditional seed occupies one `(trait, head)` slot; a same-file redeclaration is refused
never-silently → **safe.** (b) *Width cap:* a narrow-width scalar field's `render`/`cmp` mismatches the
`Binary{64}` method param → **honest width-mismatch gap** (corpus `u64` covered; narrower deferred) →
**safe, disclosed.** (c) *Instance-resolution interaction:* seeding only the resolution fact (key +
sig) plugs into the existing coherence lookup with no new path; obligation (1) pins it to the body →
**sound iff the sig-pin gate holds.** (d) *Value semantics:* `render`/`cmp`/`init`/`eq`/`hash` are pure
value functions (no aliasing/mutation); `init` yields a fresh all-zeros/`Nil`/`False` value → **no
value-semantics interaction hazard.** (e) *Ambiguity/overlap:* the transpiler never emits its own
primitive instances, and the seed is per-head + conditional → **no overlap within a transpiled file.**
The one residual failure mode is signature drift between seed and body — closed by obligation (1). **The
recommendation survives the stress-test with two disclosed, honest residuals (width cap, hash-over-
scalar) pushed to increment 2.**

---

## §6 Scope boundary — what increment 1 does NOT unblock (VR-5, tempering the claim)

The task's expectation is that this "unblocks the whole DeriveAttr class once ratified+built." That is
**over-optimistic and I am tempering it honestly:**

- **`Vec[T]` fields still gap.** `Vec<T>` maps to the `Vec[T]` cons-list (`type Vec[A] = Nil |
  Cons(A, Vec[A])`, `map.rs:110`), a `Data:Vec` type. No `Show[Vec]`/`Init[Vec]`/`Ord3[Vec]` exists,
  and eq/hash over a recursive cons-list is a structural recursion none of the rows compose. **Every
  named corpus struct (`CheckError`/`CtorInfo`/`EvaluatorOpts`) has `Vec` fields**, so those structs
  are **not** fully unblocked by increment 1 — only their scalar/`Bytes`/`Bool` fields are. A struct
  whose fields are *only* scalar/`Bytes`/`Bool` **is** fully unblocked.
- **Narrow-width scalars (`u8`/`u16`/`u32` → `Binary{8/16/32}`)** need a `width_cast` in the Show/Ord3
  compose (the width-erased coherence cap, §2 fact 1). Deferred.
- **Hash over a scalar field** needs a `Binary{N} -> Bytes` raw-byte prim that does not exist (§2
  table). Deferred; hash-over-`Bytes` works today.

These three are **increment 2** — a follow-on DN/issue (WU-4, §8), not this note. Increment 1's honest
value is: it unblocks all scalar/`Bytes`/`Bool` derive fields and every struct composed only of them,
which is the *mechanism* the class needs; the recursive `Vec` closure is the remaining lever.

---

## §7 Definition of Done (for the maintainer / DN-review gate)

**Accepted** requires the gate to confirm:

1. **Mechanism:** Alt A (prelude-instance-seed the resolution fact) — **resolved and adopted by the
   gate over Alt B/C** (§4.4 GATE RESOLUTION, 2026-07-13): Alt B's name-collision guard is unavailable
   in a verbatim-emitting transpiler (verified: no renaming scheme, `reserved.rs`/`emit.rs:1049`), so
   its pollution hazard is real and Alt A is the sound choice. *(Satisfied at ratification.)*
2. **Design basis:** the §3 heterogeneity finding (trait-dispatched vs prim-routed) accepted, and the
   §4.5 classifier (not a boolean) accepted as the `field_derive_eligible` replacement.
3. **Scope:** the §6 boundary accepted — increment 1 = scalar/`Bytes`/`Bool` fields (width ≤ 64);
   `Vec[T]`, narrow-width scalar, hash-over-scalar deferred to increment 2.
4. **Soundness gates:** the §5 obligations accepted as build acceptance conditions (sig-pin
   differential; Float never seeded + ADR-040 preserved; conditional-on-need; never-silent
   redeclare-refusal; every instance tagged at its checked basis).
5. **Build issues minted** (§8) by the integrating parent — `issues.yaml` is read-only to this note.

Only after (1)–(5) does the build (§8) proceed; nothing here is `Enacted` until WU-1..3 land and are
differential-witnessed against the real `myc check` oracle (VR-5).

---

## §8 Build decomposition (FLAGGED — issues minted by the integrating parent)

Strict dependency order (each work-unit pins the next):

| WU | Scope | Owns (files) | Depends on |
|---|---|---|---|
| **WU-1** | **Author the missing `lib/std` primitive instances** — `Init[Binary{N}]`=0, `Init[Bool]`=`False`, `Init[Bytes]`=empty; `Ord3[Binary{N}]` (via `std.cmp::cmp{N}`), `Ord3[Bytes]` (lexicographic), `Ord3[Bool]`. Each with its honest guarantee tag (§5 pt 2) + a `myc check` witness. `Show[*]` already exist (§2). | `lib/std/fmt.myc`, `lib/std/cmp.myc` (or a new `lib/std/derive_prelude.myc`) | landed prims only |
| **WU-2** | **The availability seed (the DN-side change)** — Alt A: add `PRELUDE_INSTANCE_SEEDS` + `seed_instance_for_nodule` (conditional, redeclare-refusing) alongside the trait-seed spine; add the §5-pt-1 sig-pin differential test. *(If Alt B chosen: emit the `use` headers in the transpiler + a name-collision guard instead.)* | `crates/mycelium-l1/src/preseed.rs`, `crates/mycelium-l1/src/checkty.rs` *(or `crates/mycelium-transpile/src/emit/*` for Alt B)* | **WU-1** (bodies must exist to pin against) |
| **WU-3** | **Eligibility + routing (the emit/derives-side change)** — replace `field_derive_eligible: bool` with `field_derive_kind` (§4.5); update each of the five field-gating rows' `compose` to route per kind (`clone_copy` is a no-op — untouched); extend `src/tests/emit.rs` row cases (scalar/`Bytes`/`Bool` derive clean; `Float` refuses; `Vec` defers) against the live oracle. | `crates/mycelium-transpile/src/emit/derives/*.rs` | **WU-2** (seeded instances must resolve for compose to `myc check` clean) |
| **WU-4** *(increment 2, deferred — separate DN/issue)* | `Vec[T]`-recursive instances + eligibility; a `Binary{N} -> Bytes` prim for hash-over-scalar; narrow-width scalar `Show`/`Ord3` via `width_cast`. | — | WU-3 |

Each WU is a small, individually-reviewable PR (DN-65 scope). WU-1/WU-2/WU-3 land in order; WU-4 is a
follow-on lever, not a blocker for the increment-1 unblock.

---

## §9 FLAGs (append-only rows owed to shared files — this note does not edit them)

- **`CHANGELOG.md`** — add a `docs(notes): DN-138 …` entry under the appropriate design-phase section
  (append-only). *Owned by the integrating parent.*
- **`docs/Doc-Index.md`** — register `DN-138` in the notes index with its one-line summary and
  `Grounds on`/`Feeds` cross-refs. *Owned by the integrating parent.*
- **`tools/github/issues.yaml`** — mint the build issues for WU-1/WU-2/WU-3 (and a follow-on for WU-4),
  with `depends_on` in the §8 order and `doc_refs` back to `corpus:DN-138` + the cited `src:` anchors.
  *Owned by the integrating parent — `issues.yaml` is read-only to this note (mit #1: verify the slot
  before minting).*

---

## §10 Ratification record (strict 9-criterion DN-review gate, 2026-07-13)

Ratified **Draft → Accepted** by the strict DN-review gate. Every citation in §1–§8 was re-read
against the code at `@dev 159417cc` (mit #14); the gate's per-criterion verdict:

| # | Criterion | Verdict | Basis (checked) |
|---|---|---|---|
| 1 | **Grounding** | **PASS** (one correction applied) | ~25 anchors verified accurate to ±4 lines: `Show` impls `fmt.myc:221/227/234`; `type_head` width-erasure `checkty.rs:309` (`Ty::Binary(_) => "Binary"`); `PRELUDE_TRAIT_SEEDS` `checkty.rs:2282`; conditional-seed/`mono` regression `checkty.rs:2310–2319`; prims `bytes_eq:8879`/`hash_blake3:8905`/`cmp.eq:10382`; `bool_eq cmp.myc:37`; `content.myc:42` (raw-`Bytes` BLAKE3 only); **NO `impl Init`/`impl Ord3` in `lib/**`** (grep-confirmed); all five field-gating rows' compose refusal+emit lines; `eq.rs:22–45` no-Eq-trait doc; `map.rs:110` `Vec` cons-list; `preseed.rs:42/81` seed+redeclare; ADR-040 §2.3/§2.4. **Correction:** §1/§3/§8 said "six rows … `field_derive_eligible` in each"; `clone_copy.rs:7` imports only `{DeriveCtx, DeriveHandler, DeriveOutcome}` and is a satisfied no-op — **five** rows gate, not six. Fixed in-place (no design impact; §3/§4.5 tables already enumerated the five). |
| 2 | **VR-5 honest scope temper** | **PASS** | §6 tempers the "unblocks the whole class" premise honestly — `Vec` fields (in every named corpus struct) still gap after increment 1; narrow-width + hash-over-scalar deferred. Not overclaimed; corrects the task's own uniform-instance framing (§3). |
| 3 | **G2 never-silent** | **PASS** | Width-mismatch → honest gap (matches the verified width-erased coherence key); redeclare-refusal reuses `preseed.rs:81`'s never-silent `CheckError`; `Float` never seeded. |
| 4 | **Append-only** | **PASS** | Status moved forward only (Draft → Accepted); Draft history retained; not `Enacted` (build unbuilt). Supersede-not-rewrite honored. |
| 5 | **Native-solution / KC-3 (key)** | **PASS** | Alt A extends the **landed** `PRELUDE_TRAIT_SEEDS` spine (verified: 5-entry array `checkty.rs:2282`, `seed_for_nodule preseed.rs:42`) with a parallel conditional instance-seed array — no new resolution path, no new kernel node. Genuinely minimal + reuses landed machinery. |
| 6 | **KC-3 (DRY/KISS/SoC)** | **PASS** | One classifier replaces one predicate (DRY); each row routes on the kind, DN-136's driver-untouched invariant preserved (SoC); Alt C's second resolver rejected (KISS). |
| 7 | **Adversarial + A-vs-B** | **PASS** | **A-vs-B resolved → Alt A** (§4.4 GATE RESOLUTION) on the *verified* premise that the transpiler emits user names **verbatim** (`emit.rs:1049`, `reserved.rs` "no sanctioned renaming scheme … never auto-renamed", `emit.rs:2092/2108`) — so Alt B's pollution hazard is real and unfixable in a non-renaming transpiler. **Sig-drift build gate (§5 obl. 1) sound + sufficient:** the seed carries key+signature only, so a seed↔body signature divergence is a check-passes/eval-fails hazard; a build-time differential pinning seed-sig == `lib/std`-body-sig catches drift before landing, and instance *existence* is subsumed (no body ⇒ no signature to diff ⇒ test fails; WU-2 `depends_on` WU-1). Residuals (coherence collision, width cap, hash-over-scalar) honestly disclosed and deferred. |
| 8 | **DoD stated** | **PASS** | §7 gives five explicit gate conditions + the header-table one-liner. |
| 9 | **Consistency** | **PASS** | Heterogeneity finding applied uniformly across the §3 table and the §4.5 classifier (both enumerate the same five gating derives); `field_derive_kind` classification sound (covers `UserNamed`/`ScalarBinary`/`BytesLike`/`BoolLike`/`Float`/`Deferred`; `Float` ineligible per ADR-040). The §1/§3/§8 "six" overcount was the sole inconsistency vs the tables' correct five — reconciled by the criterion-1 correction. |

**Outcome:** clean PASS. The design is ratified; the A-vs-B call is **resolved to Alt A** with a
checked (Empirical) basis, not an asserted one. Nothing is `Enacted` — WU-1/WU-2/WU-3 must land and be
differential-witnessed against the real `myc check` oracle before any mechanism/tag upgrades past
`Declared` (VR-5). Build issues + the `CHANGELOG`/`Doc-Index` rows remain FLAGGED for the integrating
parent (§9); `issues.yaml`/`CHANGELOG.md`/`Doc-Index.md` were not edited by this ratification.

---

## Changelog (this note)

- 2026-07-13 — **Draft** created (`@dev 159417cc`). Design-only; recommends Alt A (prelude-instance-seed
  the resolution fact) for the DeriveAttr-class top unblock, corrects the task's uniform-instance
  framing (§3 heterogeneity finding), tempers the unblock claim (§6 — `Vec` fields deferred), and
  FLAGs the build decomposition (§8) + shared-file rows (§9). Not self-ratified — the strict DN-review
  gate ratifies (house rule #3).
- 2026-07-13 — **Draft → Accepted** by the strict 9-criterion DN-review gate (§10). All §1–§8 anchors
  re-verified against `@dev 159417cc`; the §4.4 **A-vs-B call resolved to Alt A** on a checked premise
  (the transpiler emits user names verbatim with no renaming scheme — `emit.rs:1049`, `reserved.rs`),
  so Alt B's namespace-pollution hazard is real and Alt A is adopted. One grounding correction applied
  (the "six rows"/`field_derive_eligible`-in-each overcount — `clone_copy` is a no-op that does not
  gate; five rows gate, §1/§3/§8). Design ratified; not `Enacted` — WU-1/WU-2/WU-3 (§8) build + witness
  pending, FLAGged for the integrating parent (§9). Append-only per house rule #3.
