# Kickoff `kpr` — Kernel self-hosting-enablement surface (E19-1)

> Read `CLAUDE.md` (house rules win) + `.claude/kickoffs/README.md` + `RFC-0032` + `RFC-0031` §5 D4
> first. This kickoff builds the **kernel prims + value representations** that unblock the blocked
> tiers of E13-1 (self-hosted stdlib). It is **design-gated**: RFC-0032 (M-746) must reach Accepted
> before any implementation leaf starts — exactly as RFC-0031/M-714 gated E13-1.

## Metadata

| Field | Value |
|---|---|
| **UID** | `kpr` |
| **Head branch** | `claude/head/kpr` |
| **Status** | **gate cleared** — M-746 done (RFC-0032 **Accepted**, §5 D1–D7); the implementation leaves M-747…M-750 are `todo` and ready to start |
| **Swarm mode** | Sonnet |
| **Depends on** | E13-1 §5 D4 (RFC-0031, **Accepted** — names the blockers); **gate: RFC-0032/M-746 must reach Accepted first**. Coordinates with `c10` (mycelium-core / kernel-T1 ownership) and `s10` (mycelium-l1 type system / width-generics). |

---

## Scope

Build the **minimal kernel surface** the self-hosted `.myc` stdlib must bottom out on to complete
E13-1's blocked tiers. RFC-0031 §5 D4 tiered the migration: Tier-0 (structural/polymorphic core)
landed (M-715); Tier-1/Tier-2 are blocked on kernel surface that does not yet exist:

- **Tier-1** — width-typed `cmp`/`Eq`/`Ord` + binary `math`: no reduce-to-`Bool` comparison prim, no
  binary arithmetic (only `bit.not`/`bit.xor` + `trit.*` are surfaced).
- **Tier-2** — efficient `collections` + `text`/`fmt`: the value model `Repr` =
  `Binary`/`Ternary`/`Dense`/`Vsa` has no sequence/array and no byte/string value.

Every addition **enlarges the value model / trusted base (KC-3)**, so the leg is design-gated by
RFC-0032 and each addition's `core`-1.0.0-vs-post-1.0.0 placement is an RFC-0032 decision (ADR-022 is
the gate of record).

---

## Epic / issue IDs driven

- Epic **E19-1** (`claude/head/kpr` is its head)
- Issues: **M-746** (RFC-0032 authoring — the GATE) → M-747 (comparison prim) · M-748 (binary
  arithmetic) · M-749 (sequence/array repr) · M-750 (byte/string repr) · M-751 (width-generic fns —
  ownership decided by RFC-0032 Q5) → M-752 (enablement conformance + `.myc` smoke ports)

---

## Owned trees (collision profile)

- **Owned:** `crates/mycelium-interp/src/prims.rs` (new prims) · `crates/mycelium-l1/src/checkty.rs`
  `prim_kernel_name` (the localized surface map) · `docs/rfcs/RFC-0032-*.md`.
- **Coordinated (NOT freely owned):**
  - `crates/mycelium-core/**` — the value-model additions (M-749/M-750) are KC-3-significant and
    overlap **`c10`**'s kernel-T1 scope. RFC-0032 Q6 decides core-1.0.0-vs-post-1.0.0 placement; do
    **not** add a `Repr` variant to the gate-met value model without that decision landed.
  - `crates/mycelium-l1/` type system (M-751 width-generics) overlaps **`s10`** (E11-1 surface
    language). RFC-0032 Q5 decides whether E19-1 owns it or it is reassigned to E11-1/`s10`. If
    reassigned, M-751 closes as a pointer.
- **Read-only (orchestrator/other-leg owned):** `tools/github/issues.yaml`, `CHANGELOG.md`,
  `docs/Doc-Index.md`, `docs/api-index/`, `docs/rfcs/README.md`, workspace `Cargo.toml`, `lib/std/**`
  (that is E13-1/`lib10`'s tree — the unblock is *demonstrated* here via smoke ports under
  `crates/mycelium-l1/tests/`, the consumers land in `lib10`).

---

## Swarm & parallelization pattern

**Serial design gate, then parallel-where-disjoint leaves.** M-746 (RFC-0032) must complete and the
RFC must reach Accepted before any implementation leaf starts. Once Accepted:

- **M-747 (comparison prim)** and **M-748 (binary arithmetic)** both edit
  `mycelium-interp/src/prims.rs` + `prim_kernel_name` — **serial on those two files** (one lands, the
  next rebases), or one leaf does both (they are small and adjacent).
- **M-749 (sequence repr)** and **M-750 (byte/string repr)** touch `mycelium-core` and are
  KC-3-heaviest — **serial on core**, and M-750 may build on M-749 (a string can be `Seq<Binary{8}>`
  per RFC-0032 Q4), so M-749 → M-750.
- **M-751 (width-generics)** — only if RFC-0032 Q5 assigns it here; it is `mycelium-l1` type-system
  work, **serial-on-L1** and coordinated with `s10`.
- **M-752 (conformance)** is the integration gate: runs after the enablers land.

**Collision surface (orchestrator-owned):** `CHANGELOG.md`, `docs/Doc-Index.md`,
`tools/github/issues.yaml`, `docs/api-index/`, the RFC index. Leaves never touch these.

```
Gate:    M-746 (RFC-0032 → Accepted)
           ↓
Wave A:  M-747 ∥-ish M-748   (serial on prims.rs)        [Tier-1 prims]
Wave B:  M-749 → M-750        (serial on mycelium-core)   [Tier-2 reprs]
Wave C:  M-751                (serial-on-L1, if owned here; else → E11-1/s10)
Wave D:  M-752                (conformance + .myc smoke ports — after A/B/C)
```

---

## Sequencing & dependencies

- **M-746 gates everything** (the value-model boundary + KC-3 placement must be right first).
- Smallest-unblock-first within the implementation: **M-747** (comparison prim) is the cheapest and
  unblocks Tier-1 `cmp` immediately; **M-748** unblocks binary math; **M-749/M-750** are the
  heaviest (new reprs) and unblock Tier-2.
- Cross-leg continuity rides the **issues**: as each enabler lands, flip the corresponding E13-1
  precondition (M-716 ⟸ M-749; M-717 ⟸ M-750; M-718 ⟸ M-747/M-748/M-751) with a pointer to the
  demonstrating test — never by touching `lib10`'s `lib/std/**`.

---

## Definition of Done

- [ ] RFC-0032 reaches **Accepted** (M-746): all §5 open questions resolved or explicitly deferred
  with direction; the value-model boundary + each addition's KC-3/1.0.0 placement + width-generics
  ownership + sequencing fixed.
- [ ] Each named enabler (M-747…M-750, and M-751 if owned here) lands with a **three-way differential
  test** (L1-eval ≡ L0-interp ≡ AOT where it runs to closed L0) and honest tags; never-silent (G2)
  on overflow / out-of-range / invalid-encoding.
- [ ] M-752: a `.myc` **smoke port per unblocked E13-1 tier** demonstrates the unblock (a width-typed
  `eq`, a binary `add`, an indexed `Vec` op, a string slice) following the
  `std_result`/`std_option`/`std_cmp` harness pattern.
- [ ] The E13-1 issues (M-716/M-717/M-718) have their preconditions flipped with a pointer to the
  demonstrating test (the unblock is *demonstrated*, never asserted — VR-5/G2).
- [ ] `just check` green; CHANGELOG append-only; RFC-0032 status/index reconciled; honest
  "implemented, pending ratification" framing where RFC-0032 is not yet Enacted.

---

## Landing

Tiered `dev → integration → main` (see `.claude/kickoffs/README.md`). Work sub-branches off the head
(or off `dev`), merge **freely** (octopus/`--no-ff`, no PR) into the head; land the completed head via
PR. **The core-touching leaves (M-749/M-750) and any width-generics work (M-751) require a maintainer
sign-off on the RFC-0032 KC-3/placement + ownership decision before merge** (architecturally
significant — flag, don't guess). Orchestrator reconciles `CHANGELOG.md`, `docs/Doc-Index.md`,
`docs/api-index/`, `tools/github/issues.yaml`, and the RFC index after the octopus merge.

---

## Agent prompt (self-contained brief)

You are running kickoff `kpr` — building the kernel prims + value representations that unblock the
blocked tiers of E13-1 (self-hosted stdlib). The repo is `/home/user/mycelium`. Your working branch is
`claude/head/kpr`; branch off `dev` (or `main` if `dev` is current) and push before spawning leaves.

**Step 0 — DONE (M-746, 2026-06-23): RFC-0032 is Accepted (§5 D1–D7).** The gate is cleared; start at
the enablers below. The ratified decisions: **D1** `eq`/`lt` prims (→`Bool`, Exact; `cmp`/`Ordering`
derives in `.myc`); **D2** surface `bit.and`/`bit.or` + never-silent carry-chain `add`/`sub`; **D3**
`Repr::Seq` (indexed sequence, never-silent `get`); **D4** `Repr::Bytes` (byte/string, never-silent
decode); **D5** width-generics → E11-1/`s10` (M-751 closed → **M-753**); **D6** in `core 1.0.0`
(extends ADR-022 T1 — append-only via supersession, **pending the mechanism**: confirm before relying
on the gate text); **D7** sequencing comparison → binary-arith → `Repr::Seq` → `Repr::Bytes`. *(Historical
brief for re-authoring, now satisfied: the two maintainer-settled questions were —)*
- **Q6 (placement) = IN `core` 1.0.0** — the reprs/prims land in the kernel before the 1.0.0 tag.
  **Consequence you must carry through:** E19-1 becomes a **core-1.0.0 gate prerequisite**, so
  E10-1/`c10`'s "tag-ready" status (ADR-022 T1) now waits on E19-1 — **FLAG that ADR-022 + E10-1 + the
  `c10` kickoff need a maintainer update** (do not rewrite that gate yourself; append-only, c10-owned).
- **Q5 (width-generics) = E11-1/`s10`** — M-751 is reassigned there as a pointer; record the link in
  RFC-0032 + file/link the s10 task, then close M-751.

The **remaining architecturally-significant questions are Q3 (sequence repr vs the recursive-ADT
`List` — the minimal addition) and Q4 (byte/string repr — dedicated `Repr` vs `Seq<Binary{8}>`)** —
**use `AskUserQuestion` for those two; do not guess.** Q1 (comparison-prim shape), Q2 (binary-overflow
semantics), and Q7 (sequencing) are engineering calls you may decide and document. Mirror how M-714
authored RFC-0031 (resolve the open questions into normative decisions, mark the DoD, move the status
footer Draft → Accepted, append the changelog row).

**Then the enablers (only after RFC-0032 is Accepted), per the RFC's sequencing:**

1. **M-747 — comparison/equality prim** over `Binary{N}`/`Ternary{N}` in
   `crates/mycelium-interp/src/prims.rs`, surfaced via `prim_kernel_name` in
   `crates/mycelium-l1/src/checkty.rs`. Guarantee `Exact`; three-way differential test (mirror
   `crates/mycelium-l1/tests/differential.rs`); an unsupported domain is an explicit never-silent
   error (G2).
2. **M-748 — binary arithmetic:** surface the already-registered `bit.and`/`bit.or` (one-line
   `prim_kernel_name` add) + a binary `add`/`sub` with the RFC-0032-decided overflow semantics
   (never-silent unless declared wrapping). Three-way differential incl. an overflow-refusal case.
3. **M-749 — sequence/array repr** in `crates/mycelium-core` (Repr + well-formedness + payload) +
   interp/AOT eval + L1 surface; indexing never-silent (out-of-bounds → `Option`/error). **Maintainer
   sign-off on the RFC-0032 placement before merge.**
4. **M-750 — byte/string repr** + codepoint ops (dedicated `Repr` or `Seq<Binary{8}>` per RFC-0032
   Q4); never-silent invalid-encoding (`Result`). Builds on M-749 if strings are sequences.
5. **M-751 — width-generic fns** (`fn f<N>(x: Ternary{N})`) — **REASSIGNED to E11-1/`s10`** (Q5,
   settled). Do **not** implement here: record the reassignment in RFC-0032 (M-746), file/link the
   `s10` task, and close M-751 as a pointer. (E13-1 M-718 depends on it landing under `s10`.)
6. **M-752 — conformance:** a `.myc` smoke port per unblocked tier under
   `crates/mycelium-l1/tests/`, proving the E13-1 preconditions are genuinely satisfied; flip the
   M-716/M-717/M-718 precondition notes with a pointer to each test.

**House rules (mandatory):**
- Never edit: `tools/github/issues.yaml`, `docs/Doc-Index.md`, `CHANGELOG.md`, `docs/rfcs/README.md`,
  `docs/api-index/`, `CLAUDE.md`, `CONTRIBUTING.md`, `lib/std/**` (that is `lib10`'s tree). FLAG up.
- Treat `crates/mycelium-core/**` as **coordinated with `c10`** and the `mycelium-l1` type system as
  **coordinated with `s10`** — flag the cross-leg edit, do not collide.
- Run `just check` (not just `cargo test`) before every commit; report any skip.
- Honesty rule (VR-5): no guarantee tag upgraded without a checked basis; new reprs/prims are
  never-silent (G2) and carry honest tags; a spec moves to "implemented, pending ratification", never
  silently to `Accepted`/`Enacted`.
- Flag-don't-guess (G2/VR-5): the representation shape + KC-3 placement + width-generics ownership are
  maintainer decisions — `AskUserQuestion`, never assume.
