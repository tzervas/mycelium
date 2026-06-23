# RFC-0032 — Kernel Self-Hosting Enablement Surface

| Field | Value |
|---|---|
| **RFC** | 0032 |
| **Status** | **Draft** (2026-06-23) |
| **Type** | Foundational / normative (once Accepted) — the kernel value-representations and primitive operations the self-hosted `.myc` stdlib must bottom out on to complete the Tier-1/Tier-2 ports |
| **Date** | 2026-06-23 |
| **Feeds** | E13-1 (stdlib in Mycelium) — unblocks M-716 (collections), M-717 (text/fmt), M-718 (width-typed cmp/math) |
| **Decides** | Which new kernel prims + value representations are required; their signatures, semantics, and honest guarantees; whether each addition lives in `core` 1.0.0 vs post-1.0.0 (KC-3); whether width-generic functions belong here or to E11-1; the implementation sequencing |
| **Depends on** | RFC-0031 (Self-Hosted Standard Library Composition — **Accepted**; §5 D4 names the blocked tiers this RFC unblocks); RFC-0001 (the value model — `Repr`/`Value`/`Meta`, the guarantee lattice); RFC-0007 (L1 kernel calculus); ADR-022 (full-language 1.0.0 gate — the core-placement question); KC-2/KC-3 (small auditable kernel — new reprs enlarge the value model); G2/VR-5 (never-silent, honesty tags) |
| **Coupled with** | `crates/mycelium-core/src/repr.rs` (the `Repr` enum — `Binary`/`Ternary`/`Dense`/`Vsa`, no sequence/string today); `crates/mycelium-interp/src/prims.rs` (the prim registry — `core.id`/`bit.*`/`trit.*`); `crates/mycelium-l1/src/checkty.rs` (`prim_kernel_name` surface map + the type system for width-generics); `lib/std/*.myc` (the consumers); E13-1 children M-716/M-717/M-718 |
| **Task** | E19-1 (epic) / M-746 (this RFC's authoring task) |

> **Posture (honesty rule / VR-5).** Advisory stub — decides nothing normatively yet. The required
> prims, the value representations, their `core`-1.0.0-vs-post-1.0.0 placement, and the width-generics
> ownership are **open questions** enumerated in §5. RFC-0031 §5 D4 established (and VR-5 requires)
> that no stdlib module is claimed self-hosted ahead of the kernel surface it bottoms out on; this RFC
> is that surface, scoped honestly. Every addition here **enlarges the value model / trusted base**
> (KC-3), so each is gated on this RFC reaching **Accepted** and on a checked differential test before
> any `.myc` consumer depends on it. Claims about "what the kernel must add" are `Declared` positions
> until checked by implementation.

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

- [ ] The required new prims are named with signatures, semantics, never-silent behaviour (G2), and
  honest guarantee tags.
- [ ] The required value representations are named (or shown ADT-expressible and thus not required),
  with their well-formedness and the indexing/codepoint operations they expose.
- [ ] Each addition's `core`-1.0.0-vs-post-1.0.0 placement is decided and grounded in ADR-022/KC-3.
- [ ] The width-generics ownership (this epic vs E11-1/`s10`) is decided.
- [ ] Each enabler is tied to the E13-1 module(s) it unblocks via `depends_on`.
- [ ] The implementation order is sequenced and grounded.
- [ ] This RFC reaches **Accepted** before any M-747…M-751 implementation leaf begins.
- [ ] All §5 open questions are resolved or explicitly deferred with direction.

## 5. Open questions

1. **Comparison/equality prim** — one `eq` (→ `Bool`) or a full `cmp` (→ a kernel `Ordering`)? Over
   both `Binary{N}` and `Ternary{N}`? Guarantee `Exact` (it is a total decidable relation)? Does it
   reuse the existing content-addressed equality (ADR-003) or define a width-typed structural one?
2. **Binary arithmetic** — surface the registered `bit.and`/`bit.or` (trivial `prim_kernel_name`
   add) plus a binary `add`/`sub`: a fixed-width carry chain with **never-silent overflow** (an
   explicit out-of-range error, mirroring the ternary prims' in-range contract) — or a wrapping
   modular `add` (which would violate G2 unless the wrap is the declared semantics)?
3. **Sequence/array representation** — is a new `Repr::Seq { elem, len }` (or similar) required for an
   indexed `Vec`, or does the recursive-ADT `List` cover enough of `collections` that only `Map`/`Set`
   (hashing/ordering) need kernel support? What is the *minimal* addition? Indexing must be never-silent
   (out-of-bounds → `Option`/error, G2).
4. **Byte/string representation** — a dedicated `Repr` for UTF-8 text, or `Seq<Binary{8}>` + codepoint
   ops in `.myc`? How are invalid-encoding conditions made never-silent (`Result`, RFC-0031 D-tier)?
5. **Width-generic functions** — `fn f<N>(x: Ternary{N}) -> Ternary{N}` (a const-generic over width)
   is needed for a *general* (non-fixed-width) `math`/`cmp` surface. Is this in scope for E19-1, or is
   it a surface-language type-system feature owned by **E11-1** (`s10`)? It touches `mycelium-l1`'s
   checker — the collision surface with the language legs.
   > **Resolved direction (maintainer, 2026-06-23): E11-1/`s10`.** Width-generics is a surface-language
   > type-system feature and is owned by the `s10` leg (which already owns the `mycelium-l1` type
   > system) — keeps exactly one leg editing the checker. M-751 is reassigned to E11-1/`s10` as a
   > pointer; M-746 records the link. (To be ratified into a normative decision when M-746 authors §5.)
6. **KC-3 / 1.0.0 placement** — the core kernel gate (ADR-022 T1) is gate-met / tag-ready. Do the new
   reprs/prims land **in** `core` 1.0.0 (enlarging the just-frozen value model), **post-1.0.0** (a 1.1
   value-model extension), or as a **non-trusted representation-extension** layered above the trusted
   base? KISS/YAGNI + KC-3 weigh here; ADR-022 is the gate of record.
   > **Resolved direction (maintainer, 2026-06-23): IN `core` 1.0.0.** The new reprs/prims land in the
   > core kernel **before** the 1.0.0 tag, so the language is fully self-hosting at 1.0.0. **Consequence
   > (flag, coordinate):** this makes E19-1 a **core-1.0.0 gate prerequisite** — E10-1/`c10`'s "gate-met
   > / tag-ready" status (ADR-022 track T1) now also waits on E19-1; ADR-022 + E10-1 + the `c10` kickoff
   > need a maintainer update so the core tag accounts for E19-1. The KC-3 trusted-base growth is thus
   > deliberate and gated by this RFC + ADR-022. (To be ratified into a normative decision, with the
   > per-addition trusted-base justification, when M-746 authors §5.)
7. **Sequencing** — comparison prim first (smallest, unblocks Tier-1 `cmp` immediately), then binary
   arithmetic, then the representations (largest, KC-3-heaviest)? Or representation-first because it
   unblocks the most E13-1 surface?

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
| 2026-06-23 | **Draft** | Initial stub — open questions enumerated; no normative decisions. Scopes the kernel prims + value representations that unblock E13-1 Tier-1/Tier-2 (RFC-0031 §5 D4). Task: E19-1/M-746. |
