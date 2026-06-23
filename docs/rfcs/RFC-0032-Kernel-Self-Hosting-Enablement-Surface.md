# RFC-0032 — Kernel Self-Hosting Enablement Surface

| Field | Value |
|---|---|
| **RFC** | 0032 |
| **Status** | **Accepted** (2026-06-23) — the kernel prims (eq/lt comparison, binary arithmetic), the value representations (`Repr::Seq`, `Repr::Bytes`), the width-generics ownership (→ E11-1/`s10`), the **in-`core`-1.0.0** placement, and the sequencing are ratified (§5 D1–D7). Was Draft (2026-06-23). |
| **Type** | Foundational / normative (once Accepted) — the kernel value-representations and primitive operations the self-hosted `.myc` stdlib must bottom out on to complete the Tier-1/Tier-2 ports |
| **Date** | 2026-06-23 |
| **Feeds** | E13-1 (stdlib in Mycelium) — unblocks M-716 (collections), M-717 (text/fmt), M-718 (width-typed cmp/math) |
| **Decides** | Which new kernel prims + value representations are required; their signatures, semantics, and honest guarantees; whether each addition lives in `core` 1.0.0 vs post-1.0.0 (KC-3); whether width-generic functions belong here or to E11-1; the implementation sequencing |
| **Depends on** | RFC-0031 (Self-Hosted Standard Library Composition — **Accepted**; §5 D4 names the blocked tiers this RFC unblocks); RFC-0001 (the value model — `Repr`/`Value`/`Meta`, the guarantee lattice); RFC-0007 (L1 kernel calculus); ADR-022 (full-language 1.0.0 gate — the core-placement question); KC-2/KC-3 (small auditable kernel — new reprs enlarge the value model); G2/VR-5 (never-silent, honesty tags) |
| **Coupled with** | `crates/mycelium-core/src/repr.rs` (the `Repr` enum — `Binary`/`Ternary`/`Dense`/`Vsa`, no sequence/string today); `crates/mycelium-interp/src/prims.rs` (the prim registry — `core.id`/`bit.*`/`trit.*`); `crates/mycelium-l1/src/checkty.rs` (`prim_kernel_name` surface map + the type system for width-generics); `lib/std/*.myc` (the consumers); E13-1 children M-716/M-717/M-718 |
| **Task** | E19-1 (epic) / M-746 (this RFC's authoring task) |

> **Posture (honesty rule / VR-5).** This RFC **decides** the kernel self-hosting-enablement surface
> normatively (§5 D1–D7): the comparison prims, binary arithmetic, the `Repr::Seq` and `Repr::Bytes`
> value representations, the width-generics reassignment to E11-1/`s10`, the **in-`core`-1.0.0**
> placement, and the sequencing. It **decides the surface, it does not implement it** — no prim or repr
> exists yet; each lands under E19-1 (M-747…M-750) with a checked three-way differential test and an
> honest tag before any `.myc` consumer (E13-1) depends on it. Every addition **enlarges the value
> model / trusted base** (KC-3); per **D6** that growth is *deliberate* and lands **in `core 1.0.0`**,
> extending ADR-022 track T1's Definition of Done (the core tag waits on E19-1). That T1 criteria change
> is captured **append-only via supersession** (house rule #3) — D6 records the decision and **ADR-024**
> enacts it into the gate (ADR-022's text is not edited in place). Until each addition is implemented +
> differential-tested, claims about its behaviour are
> `Declared` positions checked by implementation (VR-5); never-silent (G2) is mandatory on every one
> (overflow, out-of-bounds, invalid-encoding → explicit `Option`/error).

---

## 1. Problem / Goal

RFC-0031 (Accepted) ratified the self-hosted-stdlib composition model and **tiered the migration by
language-surface readiness** (§5 D4). Tier-0 — the structural/polymorphic core (`Option`/`Result` +
finite-type `Ordering`/`Eq`/`Ord`) — is executable today and landed (M-715). The heavier tiers are
**blocked on kernel surface that does not yet exist**:

- **Tier-1** — width-typed `cmp`/`Eq`/`Ord` over `Binary{N}`/`Ternary{N}`, and binary `math`. The
  kernel *surfaces* `bit.not`/`bit.xor` (binary) and `trit.neg/add/sub/mul` (ternary) — note
  `bit.and`/`bit.or` are **registered** in the prim registry but **not yet surfaced** via
  `prim_kernel_name` (Q2) — but there is **no reduce-to-`Bool` comparison/equality prim** and **no
  binary arithmetic** (`add`/`sub`), so a width-typed `eq`/`cmp`
  or a binary `add` has nothing to bottom out on.
- **Tier-2** — `collections` (efficient `Vec`/`Map`/`Set`) and `text`/`fmt`. The value model
  (`Repr` = `Binary`/`Ternary`/`Dense`/`Vsa`) has **no sequence/array value** and **no byte/string
  value**, so an indexed `Vec` or a UTF-8 `str` has no representation. *(Note: a purely functional
  cons-`List` is expressible as a recursive `.myc` ADT today — `type List<A> = Nil | Cons(A, List<A>)`
  — so part of `collections` is not blocked; the §5 questions delimit exactly which additions are
  required for the *efficient/indexed* surface vs which the existing ADT machinery already covers.)*

This RFC scopes the **minimal kernel surface** that unblocks those tiers, decides where each addition
lives relative to the 1.0.0 kernel gate (ADR-022; KC-3), and sequences the implementation. It is the
design gate for epic E19-1.

## 2. User stories

- As a **stdlib author**, I want the exact set of new kernel prims and value representations named,
  with signatures and guarantees, so that the blocked Tier-1/Tier-2 `.myc` ports have a concrete,
  honest surface to compile against — not a vague "needs a prim".
- As a **kernel engineer**, I want each addition's `core`-1.0.0-vs-post-1.0.0 placement decided
  against KC-3/ADR-022 before I touch `mycelium-core`, so that a new `Repr` does not silently reopen
  the gate-met kernel value model.
- As a **compiler engineer**, I want the width-generics question (own here vs E11-1/surface-language)
  resolved so that the `mycelium-l1` type-system work is owned by exactly one work leg (no collision
  with `s10`).
- As a **maintainer**, I want each enabler tied to the specific E13-1 module it unblocks (M-716/717/718)
  via `depends_on`, so that cross-leg continuity rides the issues, never a shared-file edit.
- As a **downstream user**, I want every new prim/repr to be never-silent (G2 — overflow/out-of-range
  is an explicit `Option`/error, never a wrap or sentinel) and honestly tagged, exactly like the rest
  of the substrate.

## 3. Scope and decision space

### In scope

- The **reduce-to-`Bool` comparison/equality** prim(s) over `Binary{N}`/`Ternary{N}` (Tier-1 `cmp`).
- The **binary arithmetic** surface (Tier-1 binary `math`): surfacing the already-registered
  `bit.and`/`bit.or`, and adding a binary add/sub (carry-chain) with never-silent overflow.
- A **sequence/array value representation** + indexing (Tier-2 efficient `Vec`/`Map`/`Set`), and the
  **byte/string value representation** + codepoint ops (Tier-2 `text`/`fmt`) — or the finding that a
  subset is ADT-expressible and needs no new `Repr`.
- The **KC-3 / 1.0.0 placement** decision for each addition (core 1.0.0, post-1.0.0, or a
  representation-extension phylum) — grounded in ADR-022.
- The **implementation sequencing** (which enabler unblocks the most E13-1 work first).

### Out of scope

- The `.myc` stdlib ports themselves — those are E13-1 (M-716/717/718), gated on this RFC.
- The full numeric tower / floating-point math semantics — `std.numerics` ε/δ design is its own spec.
- Runtime/concurrency reprs — Phase-7 (RFC-0008), not a stdlib-enablement concern.
- Changes to the existing `Binary`/`Ternary`/`Dense`/`Vsa` reprs or the guarantee lattice — those are
  RFC-0001; supersede it to change them.

## 4. Definition of Done

- [x] The required new prims are named with signatures, semantics, never-silent behaviour (G2), and
  honest guarantee tags. → **§5 D1 (comparison), D2 (binary arithmetic).**
- [x] The required value representations are named, with their well-formedness and the
  indexing/codepoint operations they expose. → **§5 D3 (`Repr::Seq`), D4 (`Repr::Bytes`).**
- [x] Each addition's `core`-1.0.0-vs-post-1.0.0 placement is decided and grounded in ADR-022/KC-3.
  → **§5 D6 (in `core 1.0.0`; ADR-022 §4 amendment).**
- [x] The width-generics ownership (this epic vs E11-1/`s10`) is decided. → **§5 D5 (→ E11-1/`s10`).**
- [x] Each enabler is tied to the E13-1 module(s) it unblocks via `depends_on`. → **§5 D7 + issues.yaml
  (M-716 ⟸ M-749, M-717 ⟸ M-750, M-718 ⟸ M-747/M-748/M-753).**
- [x] The implementation order is sequenced and grounded. → **§5 D7.**
- [x] This RFC reaches **Accepted** before any M-747…M-750 implementation leaf begins. → **this
  revision; the leaves move `blocked → todo`.**
- [x] All §5 questions are resolved. → **§5 D1–D7.**

## 5. Decisions (D1–D7) — ratified

> Each decision is **normative** for E19-1. Q5 (D5) and Q6 (D6) were maintainer calls (2026-06-23);
> Q3 (D3) and Q4 (D4) the representation shapes (maintainer, 2026-06-23); Q1/Q2/Q7 (D1/D2/D7)
> engineering calls. Honesty (VR-5): each is a `Declared` design position until its implementation
> lands with a checked three-way differential test; never-silent (G2) is mandatory on every one.

### D1 — Comparison/equality prims (Q1)

Two kernel prims over `Binary{N}` and `Ternary{N}`, each returning `Bool`, guarantee **`Exact`** (a
total decidable relation): **`eq(a, b)`** (structural width-typed equality — equal payload over equal
`Repr`) and **`lt(a, b)`** (the total order: unsigned magnitude for `Binary{N}`, balanced-integer
value for `Ternary{N}`). Everything else derives **in `.myc`**: `std.cmp`'s `cmp(a,b) -> Ordering`
(`match eq(a,b) { True => Eq, False => match lt(a,b) { True => Lt, False => Gt } }`), `le`/`gt`/`ge`,
`min`/`max`/`clamp`. Mismatched widths/paradigms are an explicit never-silent prim error (G2 — never a
silent `false`). `eq` is the width-typed structural relation; it is consistent with but does not
replace ADR-003 content-addressed identity (which is the kernel's own value identity). Surfaced via
`prim_kernel_name` as `eq`/`lt`. **Unblocks** E13-1 M-718 (width-typed `cmp`/`Eq`/`Ord`) + the M-716
`Map`/`Set` ordering basis.

### D2 — Binary arithmetic (Q2)

(a) **Surface the already-registered `bit.and`/`bit.or`** via `prim_kernel_name` (a one-line addition
each — they exist in `PrimRegistry::with_builtins`, just unsurfaced). (b) **Add binary `add`/`sub`**
over `Binary{N}` as a **fixed-width carry chain with never-silent overflow**: a result outside
`[0, 2^N)` is an **explicit out-of-range error**, exactly mirroring the `trit.*` prims' in-range
contract (G2 — *never* a silent wrap). Guarantee **`Exact`** on the in-range result. A wrapping/modular
`add` is **rejected** (it would violate G2 unless the wrap were the declared semantics, which it is
not here — an explicit `wrapping_add` could be a separate, declared op later). Surfaced as `and`/`or`/
`add_bin`/`sub_bin` (names finalized at implementation to avoid clashing with the `trit`-backed
`add`/`sub` already mapped — a dispatch-by-`Repr` `add`/`sub` is the preferred surface if the checker
can resolve it). **Unblocks** E13-1 M-718 (binary `math`).

### D3 — Sequence/array value representation (Q3)

**Add a first-class `Repr::Seq` to `mycelium-core`** (with a matching `Payload::Seq(Vec<Value>)`), an
**indexed sequence** of a homogeneous element type — `Repr::Seq { elem: Box<Repr>, len: u32 }` (shape
finalized at implementation), well-formed iff every element matches `elem` and the count matches `len`.
It exposes **never-silent indexing** (`get(s, i) -> Option<elem>` — out-of-bounds is `None`, **never** a
panic or a silent default; G2), `len`, `push`/`pop` (capacity/empty conditions never-silent), and a
fold/iterate basis. This is the substrate for an **O(1)-indexed `Vec`** and the ordered/〔hashed〕
`Map`/`Set` (over D1's `lt`/`eq`). It is the **largest KC-3 trusted-base addition** here — justified by
D6 (full self-hosting at 1.0.0) and gated by the maintainer sign-off in the `kpr` kickoff. **Unblocks**
E13-1 M-716 (collections). *(The recursive-ADT `List<A> = Nil | Cons(A, List<A>)` remains valid `.myc`
for the functional/linked case and needs no kernel support — `Repr::Seq` is specifically the indexed,
O(1) substrate the maintainer chose for the efficient `Vec`/`Map`/`Set` surface.)*

### D4 — Byte/string value representation (Q4)

**Add a dedicated `Repr::Bytes` to `mycelium-core`** (with `Payload::Bytes(Vec<u8>)`) — a first-class
byte string, well-formed for any byte content. Text operations layer on it: **codepoint/UTF-8 decode is
written in `.myc`** over the byte surface, and **invalid encoding is never-silent** (a `Result`, per
RFC-0031's `Declared`-tier honesty — `decode_utf8(b) -> Result<…, DecodeError>`, never a silent
replacement char unless explicitly requested). `Repr::Bytes` exposes `len`, never-silent `get`/`slice`
(out-of-range → `Option`/error, G2), and byte concatenation. Chosen over modelling strings as
`Seq<Binary{8}>` so text has a clear, efficient first-class value (a second deliberate KC-3 addition,
D6-gated). **Unblocks** E13-1 M-717 (text/fmt).

### D5 — Width-generic functions ownership (Q5)

**Reassigned to E11-1/`s10`** (maintainer, 2026-06-23). Width-generic function parameters over
representation width (`fn f<N>(x: Ternary{N}) -> Ternary{N}`) are a **surface-language type-system
feature** that edits the `mycelium-l1` checker — the `s10` leg's collision surface — so `s10`/E11-1
owns it (keeps exactly one leg editing the type system). E19-1's **M-751 is closed as a pointer** to the
new E11-1 task **M-753**; E13-1 M-718's `depends_on` points at M-753 for the general (non-fixed-width)
`math`/`cmp` surface. E19-1 stays the prims+reprs leg.

### D6 — Placement: in `core 1.0.0` (Q6)

**The reprs/prims land in the core kernel before the `core 1.0.0` tag** (maintainer, 2026-06-23), so
the stdlib is **fully `.myc`-self-hosted at the tag** rather than only its structural core. This
extends **ADR-022 track T1's Definition of Done** to also require E19-1 (an additive extension — the
met Gate A/B rows are not reopened), so **the core tag (M-703) waits on E19-1**.

**Governance (append-only — house rule #3).** Extending T1's DoD is a *criteria* change to the
**Accepted** ADR-022, whose Status requires **superseding** to change criteria. Therefore this RFC
**records the decision** (here, D6) and it is enacted into the gate by the focused amending
**ADR-024** (Accepted 2026-06-23) — **not** by editing ADR-022's §4/§5 criteria in place (ADR-022
carries an append-only "amended by ADR-024" pointer). The tracker carries the operational linkage
(M-703 `depends_on` E19-1). The KC-3 trusted-base growth (D1–D4 — two prims + two `Repr` variants) is
**deliberate and gated by this RFC + ADR-024**; each addition carries its own trusted-base
justification at implementation. Chosen over a leaner core tag + a 1.1 value-model extension.

### D7 — Sequencing (Q7)

Smallest-unblock-first, KC-3-lightest-first:
1. **M-747** (comparison `eq`/`lt` prims) — smallest; unblocks Tier-1 `cmp` immediately.
2. **M-748** (binary arithmetic) — small; unblocks binary `math`.
3. **M-749** (`Repr::Seq`) — the first `Repr` addition; unblocks `collections`.
4. **M-750** (`Repr::Bytes`) — the second `Repr` addition (independent of D3 — strings are bytes, not
   sequences-of-bytes, per D4); unblocks `text`/`fmt`.
5. **M-752** (conformance + `.myc` smoke ports) — after the enablers.
M-753 (width-generics) runs **in parallel under `s10`/E11-1** (disjoint — the `mycelium-l1` type
system, not the prims/reprs). Each `Repr`-touching leaf (M-749/M-750) takes the maintainer sign-off
named in the `kpr` kickoff before merge.

## 6. Grounding / honesty

- RFC-0031 (Accepted, 2026-06-23) — §5 D4 tiered the migration and named exactly these blockers; this
  RFC is the kernel surface that §5 D4 sequences the blocked tiers behind.
- RFC-0001 (the value model) — `Repr`/`Value`/`Meta` + the guarantee lattice; the substrate any new
  repr/prim must compose with.
- `crates/mycelium-core/src/repr.rs` (checked 2026-06-23) — `Repr` = `Binary`/`Ternary`/`Dense`/`Vsa`;
  no sequence/string today (the Tier-2 gap is real, not assumed).
- `crates/mycelium-interp/src/prims.rs` (checked 2026-06-23) — registered prims `core.id`,
  `bit.not/and/or/xor`, `trit.neg/add/sub/mul`; no comparison, no binary arithmetic (the Tier-1 gap).
- `crates/mycelium-l1/src/checkty.rs::prim_kernel_name` — surfaces only `not`/`xor`/`add`/`sub`/`mul`/
  `neg`; `bit.and`/`bit.or` are registered but unsurfaced (a one-line addition, Q2).
- ADR-022 (full-language 1.0.0 gate) — the core-placement decision (Q6) is grounded here.
- KC-2/KC-3, G2, VR-5 — non-negotiable: new reprs/prims enlarge the trusted base (justify per KC-3),
  are never-silent (G2), and carry honest tags (VR-5).

---

### Changelog

| Date | Status | Note |
|---|---|---|
| 2026-06-23 | **Accepted** | M-746: §5 D1–D7 ratified — D1 `eq`/`lt` comparison prims (Exact), D2 binary arithmetic (surface `bit.and`/`bit.or` + never-silent carry-chain `add`/`sub`), D3 `Repr::Seq` (indexed sequence, never-silent `get`), D4 `Repr::Bytes` (string/byte value, never-silent decode), D5 width-generics → E11-1/`s10` (M-751 → pointer to M-753), D6 placement **in `core 1.0.0`** (extends ADR-022 T1 — enacted append-only by the focused amending ADR-024, not an in-place edit), D7 sequencing (comparison → binary-arith → `Repr::Seq` → `Repr::Bytes` → conformance). Enablers M-747…M-750 move `blocked → todo`. |
| 2026-06-23 | **Draft** | Initial stub — open questions enumerated; no normative decisions. Scopes the kernel prims + value representations that unblock E13-1 Tier-1/Tier-2 (RFC-0031 §5 D4). Task: E19-1/M-746. |
