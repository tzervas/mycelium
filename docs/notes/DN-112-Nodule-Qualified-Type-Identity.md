# Design Note DN-112 — Nodule-Qualified Type Identity (the ctor-seal capability boundary, made real)

| Field | Value |
|---|---|
| **Note** | DN-112 |
| **Status** | **Accepted** (2026-07-10, delegated ratification — see the dated "Ratification (maintainer-delegated, orchestrator-selected on the merits, 2026-07-10)" section below). **Accepted ratifies the MECHANISM only, NOT Enacted** (house rule #3: `Enacted` requires stepping through `Accepted` first and means *fully implemented/landed*; this note has no implementation to land — M-1036 tracks that). Originally **Draft** (2026-07-10). A design-reasoner mini-DN that worked **M-1036** forward to a ranked recommendation for maintainer ratification. It decides the **type-identity mechanism** DN-104 §6 left open — the residual that turns M-1027's landed `priv` constructor seal (DN-104, Draft) from a never-silent API-discipline nudge into an **enforced capability boundary**. Tags are `Empirical` where read against the code at `dev@45927ea4`, `Declared` for any design not yet implemented/witnessed (VR-5). |
| **Decides (proposes, for ratification)** | (1) the **`Ty::Data` identity mechanism** — how two same-named data types declared in **different nodules** become *distinct* types (ranked: §4); (2) whether the **`type_head` coherence-key** twin gap is **in- or out-of-scope** for M-1036 — decided **IN-SCOPE** (§5, G2 — stated, not silently dropped); (3) **printer/EXPLAIN rendering** — own-nodule types render bare, cross-nodule render qualified, never-silent (§6); (4) confirmation that the change keeps **`mono.rs` mangling collision-free** (§7); (5) the **KC-3 framing + blast radius** — a check-time identity refinement, no L0/runtime change (§3); (6) the **guarantee posture** — `Empirical` on the general fix, `Declared` on a point-patch (§8). |
| **Feeds** | M-1036 (this note is its design gate); DN-104 §6 CRITICAL residual (the real fix it names); DN-99 §A3 / register row #37 (sealed-constructor visibility) + FR-N3 (the unforgeable-capability driver); M-1023 (`Approx::proven` — the port that must not rely on the seal as a boundary until this lands); M-1050 (`pub(path)` scoped visibility — DN-104 §3 option B — which `depends_on` M-1036 and inherits the shadow bypass until this lands); RFC-0006 §4.3 / M-662 / M-1024 (the bare-name resolution + cross-nodule link machinery this note refines). |
| **Grounds on** | KC-3 (small kernel — a check-time identity refinement, no new L0 node, no runtime/representation change), DRY (reuse the existing `Ty::Data` `String` slot + the M-662 per-nodule registry as the home carrier), G2 (never-silent — a same-named cross-nodule mismatch is an explicit `CheckError`; the printer discloses the home), VR-5 (no tag upgraded past its basis — `Declared` until the flipped differential + property test witness it `Empirical`), KISS/YAGNI (nodule-qualified name over a content-addressed type id). |
| **Date** | July 10, 2026 |
| **Task** | M-1036 — nodule-qualified type identity (the real fix for M-1027's ctor-seal capability-gate bypass). |

> **Grounding + honesty (house rule #4 / VR-5 / G2).** This note is a **design recommendation**, not a
> decision (house rule #3 — the maintainer ratifies). Empirical claims are read against the tree at
> `dev@45927ea4` and cite `file:line`; the recommended mechanism, the `type_head` scope call, and the
> guarantee posture are `Declared` until implemented + witnessed. **No sycophancy:** §4 ranks three real
> alternatives on merit and §9 argues *against* the Rank-1 recommendation (the string-overload
> transparency concern that could reasonably flip Ranks 1 and 2), rather than presenting a strawman plus
> the answer. The scoping's own candidate ("add `home` to `DataInfo`") is confirmed **necessary but not
> sufficient** on its own (§4) — surfaced, not glossed.

---

## §1 The problem (verified against the code)

M-1027 (ENB-4, DN-104) landed a per-constructor `priv` seal: `pub type T = priv Mk(..)` exports the
type *name* but withholds the constructor from cross-nodule **construction**. The mechanism is a
per-nodule withheld set (`NoduleImports.sealed`, `crates/mycelium-l1/src/checkty.rs:1269`) populated from
the phylum export table (`Exports.sealed_ctors`, `checkty.rs:1228`) and checked at the two
constructor-application sites via `check_ctor_seal` (`checkty.rs:3993`).

**The seal is bypassed** because **type identity is by bare name**, not nodule-qualified:

- `Ty::Data(String, Vec<Ty>)` (`checkty.rs:107`) carries only a **bare name string**. Its `PartialEq`
  (`checkty.rs:427`) is `n1 == n2 and args match` — two data types with the same bare name are the
  **same type**, regardless of which nodule declared them.
- `resolve_ty` (`checkty.rs:648`) resolves a named type by looking it up in the **caller's own** merged
  registry (`checkty.rs:689`, `Ty::Data(name.clone(), resolved)` at `:707`) — bare name, re-resolved in
  the calling nodule's scope (own decls shadow imports — RFC-0006 §4.3 / M-662, pre-existing).
- The withheld set (`NoduleImports.sealed`) is keyed by **bare constructor name** and populated **only**
  along the `use`-import path — it is never consulted for a caller's **own locally declared** type
  (`checkty.rs:1259`–`1268`).

So a foreign nodule that declares its **own same-named unsealed type** — never importing the sealed
original — forges a value the checker accepts wherever the sealed type is expected. Pinned as
`crates/mycelium-l1/tests/ctor_seal.rs:203`
(`known_gap_a_same_named_local_shadow_type_bypasses_the_seal`), which currently **asserts the unsound
`Ok`**. Its own comment (`ctor_seal.rs:204`–`209`) confirms the root cause: *"`resolve_ty` looks the name
up in the caller's `Cx.types`, not the callee's declaring scope."*

**Consequence (Empirical, witnessed by the pinned test):** DN-104's seal, as landed, is a never-silent
opt-in API-discipline nudge for a well-behaved `use home.Ctor` caller — **not** the FR-N3 unforgeable
capability boundary DN-104 §1/§3 originally framed. This note decides the identity mechanism that closes
that gap.

## §2 User stories

- **As a stdlib author** relying on M-1027's sealed-constructor capability-gate (e.g. M-1023's
  `Approx::proven`), I want the seal to withhold construction from **any** foreign nodule — including one
  that declares its own same-named local type — so that the "unforgeable" claim DN-104 makes is *true*,
  not bypassable by an adversarial or accidentally-colliding name.
- **As a phylum author** with two unrelated nodules that each legitimately declare a type named `Node`
  (no capability intent), I want them to remain **distinct, non-conflicting** types, so that an
  accidental name collision across unrelated nodules is never a false type-equality (or a false
  coherence-overlap refusal) — the fix must not over-merge.
- **As an agent or human reading an `EXPLAIN`/diagnostic**, I want a cross-nodule type rendered with its
  home so I can see *which* `Node` a mismatch is about, while my own-nodule types stay legibly bare —
  never-silent, but not noisy for the common case.

## §3 KC-3 framing plus blast radius (verified)

**This is a check-time (L1) identity refinement — no L0 node, no runtime/representation change, value
semantics unaffected.** `Ty` is the L1 type-checker's type; the runtime `Value`/`Repr` lives in
`mycelium-core` and is a **different** type family (`mycelium-core/src/data.rs` uses `FieldTy::Data`, not
`checkty::Ty::Data` — the whole-workspace `Ty::Data` grep hit on core was a false positive). No swap, no
representation, no L0 grammar node is added (KC-3). The mangling change (§7) is compile-time
monomorphization *naming*, not a runtime layout change.

**Blast radius (verified at `dev@45927ea4`).** `Ty::Data` appears in exactly **7 non-test source files**
of `mycelium-l1` and nowhere else in the workspace:

| File | `Ty::Data` textual occurrences |
|---|---|
| `crates/mycelium-l1/src/checkty.rs` | 37 (incl. the enum def, `Display`, `PartialEq`, `resolve_ty`, `type_head`) |
| `crates/mycelium-l1/src/mono.rs` | 28 (mangling plus the mono solver) |
| `crates/mycelium-l1/src/elab.rs` | 9 |
| `crates/mycelium-l1/src/decision.rs` | 2 |
| `crates/mycelium-l1/src/eval.rs` | 1 |
| `crates/mycelium-l1/src/fuse.rs` | 1 |
| `crates/mycelium-l1/src/usefulness.rs` | 1 |

Total ≈ **79 textual occurrences** across 7 source files (the scoping's "~7 files / ~68 match sites" is
confirmed in the right order of magnitude; the exact identity-bearing match/construct arms are a subset
of the 79). **Single crate, check-time only.** Under the Rank-1 mechanism (§4) most of these arms need
**no change** (they destructure `Ty::Data(n, args)` and the identity rides inside `n`); the functional
edits concentrate in `resolve_ty`, the registry/`DataInfo`, the printer, and a small audit set.

## §4 The identity mechanism — three alternatives, ranked

The scoping's candidate is *"add `home: String` to `DataInfo`, stamped at `resolve_ty`."* **Verified
finding: that is necessary but not sufficient on its own.** `DataInfo` is the registry entry
(`checkty.rs:249`); it is **not** what flows through checking and mono. Type *equality* and *mangling*
operate on the `Ty::Data` **value** (`PartialEq` at `checkty.rs:427`, `mangle_ty` at `mono.rs:2996`),
which carries only the bare name. So `DataInfo.home` sitting unused changes no type's identity. **The
discriminator must reach the `Ty::Data` value itself.** The three real ways to do that:

### Alt 1 — Nodule-qualified name in the existing `Ty::Data` `String` (RECOMMENDED, Rank 1)

`resolve_ty` stamps the **declaring nodule's home** into the identity by producing a **qualified name**
in the existing `String` slot (e.g. `Ty::Data("a::T", [..])` for `T` declared in nodule `a`); `DataInfo`
gains `home: String` as the provenance record the stamp is read from, populated at declaration
registration / M-662 link time. The registry key and `DataInfo.name` are qualified **consistently** with
the stamp.

- **Mechanism.** The home carrier is `DataInfo.home` (the scoping's field); the *identity* is the
  qualified string in `Ty::Data`. Because the discriminator rides **inside the existing `String`**,
  `PartialEq`, `subst_ty` (`checkty.rs:323`), `has_var` (`:375`), and — critically — `mangle_ty` are
  **unchanged and collision-free by construction** (a different home means a different string means a
  different mangle; §7). The ~79 arms that merely destructure `Ty::Data(n, args)` need **no change**.
- **What changes.** (a) `resolve_ty` — stamp the qualified name from the resolved `DataInfo.home`;
  (b) `DataInfo` plus the registry key plus M-662 link — qualify consistently; (c) the diagnostic printer
  — strip the own-home prefix (§6); (d) an **audit** of the handful of literal `n == "<builtin>"`
  comparisons (`"Result"`, `"Option"`, `"Bool"`, `Tuple$N`, `mono.rs` `n == dname`) — builtins/synthetics
  stay under a single reserved home so they never split per-nodule (§9, the load-bearing invariant).
- **KC-3 verdict.** Smallest kernel delta that actually closes the gap; reuses the existing
  injective-mangling machinery. **Recommended.**

### Alt 2 — Explicit `home` field in `Ty::Data` (Rank 2)

Restructure to a struct variant `Ty::Data { name, home, args }` (or a `QualName` newtype), making home a
**structural** part of identity. `PartialEq`/`type_head`/`mangle_ty` fold `home` in explicitly.

- **Pro.** No string-overload: the identity is unambiguously structured, robust against a user type whose
  name contains the qualifier separator, and legible in a debugger. Arguably the more honest shape for a
  **security** mechanism (G2 — the identity is not a parseable convention).
- **Con.** Touches **all ~79 `Ty::Data` arms** (every match must bind/ignore the new field), and
  `mangle_ty` must fold `home` in (a one-line addition, still collision-free). Larger mechanical churn
  for the same semantics.
- **When to prefer.** If the maintainer judges the string-overload too implicit for a capability-security
  boundary — a legitimate G2 call (§9). This note ranks it **second** on KISS/blast-radius, but the gap
  to Rank 1 is *narrow* and defensible either way.

### Alt 3 — Content-addressed type id (Rank 3)

Identity = a hash / interned id of the resolved declaration (aligns with RFC-0007 §4.2's
content-addressing aspiration).

- **Pro.** Most general: distinguishes even same-name **same-home** structurally-different declarations,
  and is stable under renaming.
- **Con.** The largest kernel change; it degrades `EXPLAIN`/mangling **readability** (a hash where a name
  was — a mild black-box tension with G2), and it solves a distinction (**structural**) that has **no
  witnessed demand** — the witnessed demand (the pinned test, FR-N3) is purely **home** distinction.
  **YAGNI** for this residual.

### Objective function (criteria table)

| Criterion | Alt 1 — qualified name | Alt 2 — explicit `home` field | Alt 3 — content id |
|---|---|---|---|
| Closes the seal-bypass gap (must-have) | Yes | Yes | Yes |
| KC-3 small-kernel delta | **Best** (rides existing `String`) | Good (new field, plus churn) | Weak (largest change) |
| Mangling collision-freedom (§7) | **Free** (string differs) | Free (fold home in) | Free (id differs) |
| Blast radius / churn | **~4 edit clusters, ~79 arms untouched** | All ~79 arms plus mangle | Broad |
| EXPLAIN / printer transparency (G2) | Good (strip own home) | Good | **Weak** (hash) |
| Explicitness of identity (G2 robustness) | Fair (string convention) | **Best** (structural) | Best (structural) |
| Matches witnessed demand (YAGNI) | **Exact** (home) | Exact (home) | Over-general (structural) |

**Ranked recommendation: Alt 1 then Alt 2 then Alt 3.** Alt 1 closes the witnessed gap with the smallest
check-time delta and gets mangling plus `type_head` collision-freedom *for free* (§5, §7); Alt 2 is the
honest fallback if the maintainer weights structural explicitness over churn (§9); Alt 3 is over-general
for the residual (YAGNI).

## §5 The `type_head` coherence-key twin gap — DECIDED **IN-SCOPE** (G2)

`type_head` (`checkty.rs:296`) computes the trait/impl **coherence key** and returns `Data:{n}` for a
data type (`checkty.rs:312`) — **the exact same bare-name weakness.** Two same-named-different-home types
share a coherence key, so `impl Trait for a::Node` and `impl Trait for b::Node` would be seen as
**overlapping** (a false global-uniqueness refusal, `checkty.rs:3443`), *or* one impl would wrongly cover
both. This is a **second bypass surface** on the same root cause — an impl-coherence forge twin of the
constructor forge.

**Decision: fixing `type_head` is IN-SCOPE for M-1036.** Rationale:

1. **It is the same root cause** (bare-name identity). Leaving it bare would ship a fix where the *seal*
   distinguishes homes but *coherence* does not — a latent, documented-open twin gap that contradicts
   this note's own objective (a real boundary). Deferring it silently would violate G2; deferring it
   *explicitly* would still leave the capability story half-closed.
2. **Under Rank 1 it is fixed as an automatic consequence** — `type_head` reads `n` from `Ty::Data`, so a
   qualified `n` makes the key `Data:a::Node` vs `Data:b::Node` with **zero extra code**. Under Alt 2 it
   is a one-line fold of `home` into the key. Near-free either way.
3. **The correctness change is desirable** — two genuinely-distinct types *should* each be allowed their
   own impl; the current bare-name key **wrongly** refuses the second as overlapping.

**But the coherence blast radius earns its own witness.** The change alters which impls count as
overlapping (the orphan/global-uniqueness rules, `checkty.rs:3443`–`3531`). The **code** is free; the
**test** is not. The DoD (§10) therefore names a **dedicated coherence witness**: two
same-named-different-home types each carry a distinct impl of the same trait **without** a false-overlap
refusal, and a genuine same-home overlap **still** refuses. In-scope, enumerated, not deferred.

## §6 Printer / EXPLAIN rendering (never-silent — transparency rule)

Identity is **always fully qualified internally**. Rendering:

- **Own-nodule types render bare** (`Node`) — legible for the common case.
- **Cross-nodule types render qualified** (`a::Node`, or the chosen home separator) — never-silent: a
  diagnostic about a same-named cross-nodule mismatch *shows the home*, so the two `Node`s are
  distinguishable in the error text (directly serving the third user story, §2).

**Design sub-choice, flagged (§11).** `Display for Ty` (`checkty.rs:141`) is today **context-free** (it
has no "current nodule"). Own-bare/cross-qualified rendering needs the current nodule. Recommendation:
the **diagnostic/EXPLAIN printer** carries the current-nodule context and does own-bare/cross-qualified;
the **context-free `Display`** renders **fully qualified** (never hides the home — honest fallback, G2),
*not* bare. This keeps `Display` never-silent while the richer printer stays legible. A maintainer may
prefer a single always-qualified renderer (simpler, noisier) — flagged, not silently chosen.

## §7 `mono.rs` mangling collision-freedom (verified plus confirmed)

`mangle_ty` (`mono.rs:2996`) mangles a nullary data type as `format!("{n}#")` (`mono.rs:3033`) and an
applied one as `n` plus `$`-joined mangled args (`:3034`–`3040`) — **using only the bare name `n`**. So
**today, two same-named-different-home types mangle identically** — a real (currently-latent, because the
types cannot yet coexist) monomorphization collision. The identity change must keep mangling injective:

- **Under Rank 1 (qualified name in the `String`):** `mangle_ty` is **unchanged and collision-free by
  construction** — `a::Node` and `b::Node` are different strings, so `a::Node#` differs from `b::Node#`.
  **Caveat (flagged):** the home separator must be a **mangling-safe** character (the mangler already uses
  `#`/`$` as non-identifier separators; a `::`/`.` in the raw name should be normalized in `mangle_ty` to
  a reserved char so the emitted symbol stays a valid downstream identifier — a one-line `.replace`, the
  same pattern `Vsa`'s `-`→`_` map already uses at `mono.rs:3015`). This is the *only* mangling edit
  Rank 1 needs, and it is additive.
- **Under Alt 2 (explicit `home` field):** fold `home` into `mangle_ty` explicitly (`{home}${n}#`).
- **`mono.rs` `n == dname` comparisons** (`mono.rs:1801`, `:1841`, `:2467`, `:2531`): these compare a
  `Ty::Data` name against a decl name. **They stay correct iff `DataInfo.name` / registry keys are
  qualified *consistently* with the `Ty::Data` stamp** — the load-bearing consistency invariant (§9).

**Verdict:** collision-freedom holds under the recommended mechanism, with one additive separator-safety
edit to `mangle_ty`. Confirmed against `mono.rs:2996`–`3050`.

## §8 Guarantee posture (VR-5)

- **General fix (recommended) → `Empirical`.** The flipped pinned regression
  (`ctor_seal.rs:203`, renamed to drop `known_gap`, asserting the **refusal**) **plus** a **property test
  over generated same-name-collision programs** (the DoD, §10) witness the boundary empirically. Not
  `Proven` — no theorem of unforgeability is discharged (VR-5); the claim is "the checker refuses the
  witnessed collision families", earned by trials, not proved.
- **Narrower point-patch → `Declared`.** If the maintainer instead chooses a *point-patch* (e.g. only
  populate the withheld set from *local same-named* decls, without general nodule-qualified identity),
  that does **not** generalize to the whole identity surface (coherence, mangling, cross-nodule
  signatures) and stays **`Declared`** — an honest, un-upgraded tag. **Recommendation: the general fix**
  (Alt 1), so the posture is `Empirical` and the `type_head`/mangling twins close with it.

## §9 Adversarial stress-test (VR-5 / house rule #4 — argue against the recommendation)

**The sharpest finding — the prelude/builtin uniform-home invariant is the single most likely way this
regresses.** If `resolve_ty` naively qualifies **everything** by the *current* nodule, then every builtin
/ synthetic type (`Bool`, `Option`, `Result`, `Tuple$N`) **splits per-nodule** — `a::Bool` differs from
`b::Bool` — and the type system **fractures**: unification fails everywhere, every cross-nodule `Result`
is a mismatch, the literal `n == "Result"` / `n == "Option"` / `n == "Bool"` comparisons scattered
through `checkty.rs`/`mono.rs` all silently stop matching. **The mechanism MUST exempt prelude/synthetic
types** — keep them under a **single reserved home** (e.g. `@prelude`, or leave them bare) so every nodule
sees the *same* builtin. This is the backward-compat requirement the M-1036 DoD hints at
("same-named-but-unrelated ... unchanged"), sharpened to its real failure mode. **This is a stronger
argument for Alt 2's explicit field** than for Rank 1: with a `home: Option<String>` field, `None` = the
prelude/uniform home is *structurally* the default, harder to get wrong than a string convention that a
careless `resolve_ty` path could over-qualify. It is the honest case where Rank 1 and Rank 2 could
reasonably swap.

**Where `home`-stamping otherwise holds (verified reasoning against the code):**

1. **Re-exports / `use` chains.** The home must be the **declaring** nodule, resolved through the
   registry `DataInfo.home` — **never** the use-site nodule. `resolve_ty` looks types up in the merged
   registry (`checkty.rs:689`), and M-662 `link` merges "each name from its home nodule"
   (`checkty.rs:1064`), so the home is available at the declaration, not the import. Stamping from the
   *resolved* `DataInfo` (not the current nodule) makes a re-exported `a::T` stay `a::T` through nodule
   `b`. **Constraint, not a break** — but a mis-implementation that stamps the use-site would silently
   re-open the bypass. Flagged as the second load-bearing invariant.
2. **Generic instantiation across nodules.** `List<Foo>` with `List` from the prelude and `Foo` from
   nodule `b` resolves to `Data("@prelude::List", [Data("b::Foo", [])])` — head home and arg home each
   stamped from their **own** declaration, recursively. Two nodules that both write `List<b::Foo>` unify
   (same string). **No break** — the stamp is per-head, recursion handles args.
3. **Recursive / self-referential types** (`Cons(A, List<A>)` — the recursive `Self` field the
   list-detection heuristic keys on, `checkty.rs:3908`–`3923`). The recursive occurrence is resolved
   under the **declaring** nodule's scope at registration, so it carries the **same** qualified name as
   the head — the `fname == name` recursion check compares qualified-to-qualified and still fires. **No
   break, but the list/recursion heuristics must be confirmed to compare the qualified names** (a DoD
   check line).
4. **Type aliases.** Verified: Mycelium v0 has **no transparent type-alias** form — `type_decl` is a data
   declaration, not an alias (`checkty.rs` grammar / DN-104 §2). So there is no alias-identity confusion
   today. **Flag forward:** if transparent aliases land later, an alias must resolve to the **aliased**
   type's home, not the alias's — noted so it is not a silent future bypass.
5. **The `mono.rs` `n == dname` consistency invariant** (§7). If `DataInfo.name`/registry keys are
   qualified but a `Ty::Data` stamp is not (or vice-versa), every such comparison silently mis-branches.
   Consistency is the property that must be tested, not assumed — a DoD property.

**Net verdict:** the recommendation survives, **conditioned on** two invariants that are the real risk
surface — (i) the **prelude/builtin uniform-home exemption** and (ii) **home = declaring nodule, stamped
from the registry, never the use-site** — plus the mangler separator-safety edit (§7). Both invariants
are enumerated in the DoD as witnessed properties, not left to care.

## §10 Definition of Done (what "Accepted" requires of the maintainer)

Ratifying this note = accepting the **mechanism**; the implementation (a later pipeline step, M-1036)
must then satisfy:

1. **Identity.** `Ty::Data` distinguishes two same-named types from different nodules (Alt 1: qualified
   name in the `String`, `DataInfo.home` the carrier, stamped at `resolve_ty` from the **declaring**
   nodule). Prelude/synthetic types stay under a single reserved home (§9 invariant i).
2. **The seal becomes real.** `ctor_seal.rs:203` is **flipped to assert the refusal**, renamed to drop
   `known_gap`; the exploit (`use a.use_t; type T = Mk(..); use_t(forge())`) is a never-silent type
   mismatch, not a values-forged pass. DN-104 §6's CRITICAL residual is closed and its capability-gate
   framing is restorable/ratifiable.
3. **`type_head` coherence (in-scope, §5).** Two same-named-different-home types each carry a distinct
   impl of the same trait with **no** false-overlap refusal; a genuine same-home overlap **still**
   refuses — a dedicated coherence witness.
4. **Mangling (§7).** A property/differential witness that two same-named-different-home types mangle
   **distinctly** (mono collision-freedom), with the separator-safety edit to `mangle_ty`.
5. **Property test** over **generated same-name-collision programs** (the general witness that earns
   `Empirical`): across nodules × sealed/unsealed × own/imported, the boundary holds and unrelated
   same-named types still check unchanged (no false-positive collision across unrelated phyla — the
   backward-compat story, §9 invariant i).
6. **Printer/EXPLAIN (§6).** Own-nodule bare, cross-nodule qualified in diagnostics; context-free
   `Display` renders qualified (never hides the home).
7. **Guarantee tag = `Empirical`** for the general fix (VR-5; `Declared` only if a point-patch is chosen —
   §8).
8. **`.myc` parity** noted, not silently assumed: the self-hosted frontend mirrors identity per DN-26 as
   the checkty cross-nodule port progresses (DN-104 §6 already flags the enforcement mirror rides that
   port).
9. **CONDITION (added by ratification, 2026-07-10 — see the Ratification section below).** The
   implementation MUST include a dedicated regression test that pins the **builtin/prelude
   uniform-home invariant** named as the sharpest stress-test finding (§9): `Bool`/`Option`/`Result`/
   `Tuple$N` and every other prelude/synthetic type stay resolvable under **one** reserved home across
   every nodule — a resolution path that over-qualifies a builtin per-current-nodule must be **caught**
   by this test, not merely honored by convention. This converts §9's risk finding from an unwitnessed
   convention into a tested property; it is **additional to**, not a replacement for, item 5's general
   same-name-collision property test.

## §11 Open questions

- **OQ-1 (mechanism, for the maintainer).** Rank 1 (qualified name in the existing `String`) vs Rank 2
  (explicit `home` field)? §9's prelude-exemption argument is a genuine reason a maintainer could pick
  Rank 2 for structural robustness. This note recommends Rank 1 on KISS/blast-radius but the margin is
  narrow — a real fork, stated on merits. **RESOLVED at ratification (2026-07-10): Rank 1 accepted,
  CONDITIONED on §10 item 9's builtin-invariant test — see the Ratification section below.**
- **OQ-2 (home separator).** Which qualifier separator (`::`, `.`, a reserved char), and its
  normalization in `mangle_ty` (§7)? Must be mangling-safe and not a legal identifier char.
- **OQ-3 (`Display` context).** Context-aware own-bare/cross-qualified printer with a fully-qualified
  context-free `Display` fallback (recommended, §6), vs a single always-qualified renderer?
- **OQ-4 (interaction with M-1050 `pub(path)`).** M-1050 layers scoped visibility on top of this
  identity fix (it `depends_on` M-1036). Confirm the qualified identity is the substrate `pub(path)`
  resolves *against* — noted so the two land in the right order (identity first, per the M-1036 body).

## Ratification (maintainer-delegated, orchestrator-selected on the merits, 2026-07-10)

**Recorded decision (append-only — this note's original §1–§11 text above is unchanged; this section
adds the ratification, per house rule #3).** The maintainer delegated the choice among this note's
ranked alternatives ("ratify the options best fit objectively speaking for this project"); the
integrating orchestrator selected the option below on the merits stated in §4–§9, and this section
records that selection as the ratification. This is a **delegated ratification, not a self-ratification
by the reasoner** — the maintainer authorized the delegation; the selection is grounded entirely in this
note's own objective-function analysis (§4's criteria table), not asserted without basis.

1. **§4 Rank 1 (Alt 1) accepted.** The nodule-qualified name carried in the **existing** `Ty::Data`
   `String` slot — stamped at `resolve_ty` from the resolved `DataInfo.home` (the declaring nodule,
   never the use-site) — is ratified as the type-identity mechanism for M-1036. It is **collision-free
   by construction** for `PartialEq`/`subst_ty`/`mangle_ty` (a different home is a different string is a
   different mangle), leaves the ~79 `Ty::Data` destructure sites that merely bind `(n, args)`
   **untouched**, and is the smallest kernel delta that closes the witnessed seal-bypass gap (KC-3 —
   "KC-3-minimal" per the §4 objective table's own verdict row). Rank 2 (explicit `home` field) and
   Rank 3 (content-addressed id) are **not** selected — Rank 2 remains the honest fallback per §9's own
   stress-test framing should a future implementation surface the string-overload concern in practice
   (not merely hypothetically); Rank 3 is YAGNI for the witnessed demand (§4).
2. **§5 `type_head` coherence-key fix — IN-SCOPE, confirmed.** The impl-coherence twin gap closes as an
   **automatic consequence** of Rank 1 (`type_head` reads the same qualified `n`) with zero additional
   code, per §5's own reasoning. Ratified as stated — not deferred, not silently dropped (G2).
3. **CONDITION added to the Definition of Done (§10 item 9, appended above).** The implementation MUST
   ship a **dedicated regression test pinning the builtin/prelude uniform-home invariant** — `Bool`,
   `Option`, `Result`, `Tuple$N`, and every other prelude/synthetic type must stay resolvable under
   **one** reserved home across every nodule, and a resolution path that over-qualifies a builtin
   per-current-nodule must be **caught**, not merely avoided by convention. This is the direct answer to
   §9's own sharpest adversarial finding (the prelude-fracture regression risk) — it converts that risk
   from an unwitnessed convention into a tested property, and it is a condition on M-1036's DoD, not an
   optional nice-to-have.
4. **Guarantee posture confirmed — `Empirical` post-fix, NOT `Proven` (VR-5).** Per §8, once M-1036 lands
   the general fix (Rank 1) witnessed by the flipped `ctor_seal.rs` differential, the §10-item-5
   same-name-collision property test, **and** this ratification's §10-item-9 builtin-invariant test, the
   posture is `Empirical` — earned by trials over a witnessed corpus, not `Proven` (no theorem of
   unforgeability is discharged). A narrower point-patch, if one were chosen instead, stays `Declared`
   per §8's own framing — unchanged by this ratification.
5. **CRITICAL — Accepted ratifies the MECHANISM only, NOT Enacted (house rule #3 / VR-5).** No code has
   landed for M-1036 at this ratification; the seal-bypass gap (`ctor_seal.rs::known_gap_a_same_named_
   local_shadow_type_bypasses_the_seal`) remains open until M-1036 implements and witnesses the
   mechanism above. This note's own guarantee-posture claims stay `Declared` until then — nothing here
   is upgraded past its checked basis by this ratification.
6. **Unblocks M-1036.** `tools/github/issues.yaml` M-1036 gains a note (applied by the integrating
   parent, not this note) recording that its mechanism is DN-112 Rank 1 plus the §10-item-9
   builtin-home-invariant test.

---

## §12 Changelog

- **2026-07-10 (later same day) — Ratified (maintainer-delegated, orchestrator-selected on the merits,
  house rule #3).** Status **Draft → Accepted** (design ratification, **NOT Enacted** — VR-5, guarantees
  stay `Declared` until M-1036 implements and witnesses the mechanism). §4 Rank 1 (nodule-qualified name
  in the existing `Ty::Data` `String` slot) accepted; §5's `type_head` coherence fix confirmed in-scope
  as an automatic consequence; a new DoD condition (§10 item 9) added requiring a dedicated
  builtin/prelude uniform-home-invariant regression test, directly answering §9's sharpest adversarial
  finding; guarantee posture confirmed `Empirical` post-fix, `Declared` on any narrower point-patch.
  OQ-1 resolved to Rank 1, conditioned. See the "Ratification (maintainer-delegated, orchestrator-selected
  on the merits, 2026-07-10)" section above for the full recorded decision.
- **2026-07-10** — DN-112 created (**Draft**). A design-reasoner mini-DN working M-1036 forward:
  recommends **Alt 1 — nodule-qualified name in the existing `Ty::Data` `String` slot** (Rank 1; Alt 2
  explicit-field second, Alt 3 content-id third — §4 objective table), decides the **`type_head`
  coherence twin gap IN-SCOPE** (§5), confirms **mono mangling collision-freedom** with one additive
  separator-safety edit (§7), specifies **own-bare/cross-qualified never-silent rendering** (§6), and
  sets the **guarantee posture** (`Empirical` on the general fix, `Declared` on a point-patch — §8). The
  sharpest stress-test finding: the **prelude/builtin uniform-home invariant** is the real regression
  risk and is the strongest argument for Rank 2 (§9). Grounded against `dev@45927ea4` (Empirical
  `file:line` cites); the recommended mechanism is `Declared` until implemented plus witnessed.
  **Recommends, does not ratify** — status advances only by maintainer ratification (house rule #3).
  Authored the DN only — no edit to `issues.yaml`, `CHANGELOG.md`, or `Doc-Index.md` (integration-owned;
  FLAGGED up).
