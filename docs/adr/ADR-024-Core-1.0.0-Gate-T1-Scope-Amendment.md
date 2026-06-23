# ADR-024 — Core 1.0.0 Gate (Track T1) Scope Amendment: the Self-Hosting Enablement Surface

| Field | Value |
|---|---|
| **ADR** | 024 |
| **Status** | **Accepted** (2026-06-23 — maintainer-ratified). Amends **ADR-022 track T1** (the core/kernel 1.0.0 sub-gate): adds epic **E19-1** (the RFC-0032 kernel self-hosting-enablement surface) to T1's Definition of Done. This is a **scoped amendment**, not a wholesale supersession — ADR-022's dual-version model, tracks T2–T9, and the preserved ADR-021 Gate A/B criteria all **remain in force**. `Accepted → Enacted` with ADR-022 T1 at the `core 1.0.0` tag. |
| **Decides** | That the **core 1.0.0 gate (ADR-022 T1)** additionally requires **E19-1** — the kernel value-representations + primitive operations the self-hosted `.myc` stdlib bottoms out on (RFC-0032) — to land and be differential-tested **before** the `core 1.0.0` tag (M-703). So the language is *fully* `.myc`-self-hosted at the tag, not only its structural core. |
| **Amends** | **ADR-022 §4 / §5 track T1** — extends T1's Definition of Done by one prerequisite (E19-1). The ADR-021 Gate A/B rows carried into T1 are **unchanged and remain met**; this ADR *adds* a requirement, it does not reopen or weaken any existing one. ADR-022 is updated only with an append-only "amended by ADR-024" pointer (its normative §4/§5 criteria text is not rewritten). |
| **Grounds** | RFC-0032 §5 D6 (the in-`core`-1.0.0 placement decision, maintainer 2026-06-23); RFC-0031 §5 D4 (the tiered stdlib migration that names the blocked tiers E19-1 unblocks); ADR-022 (the gate amended); ADR-021 (the kernel Gate A/B carried into T1); KC-2/KC-3 (small auditable kernel — the trusted-base growth is deliberate and bounded); G2/VR-5 (never-silent, honest tags). |
| **Date** | 2026-06-23 |

> **Posture (honesty rule / VR-5).** This ADR records *criteria*, maintainer-ratified; it asserts no
> release. It **adds** E19-1 to T1's Definition of Done — it does not declare E19-1 done, nor move any
> spec to `Enacted`. E19-1's per-issue DoDs (M-747…M-750 + the M-752 conformance gate) carry the
> evidence bar; nothing here is "met" until each lands with a checked three-way differential test. The
> KC-3 trusted-base growth (two prims + two `Repr` variants, RFC-0032 D1–D4) is **deliberate and gated
> by RFC-0032 + this ADR**; each addition carries its own trusted-base justification at implementation.

---

## 1. Why this amendment exists

RFC-0031 (Accepted) tiered the self-hosted-stdlib migration and found the **structural/polymorphic
core** executable today (Tier-0 — landed, M-715), but the heavier tiers **blocked on kernel surface
that does not exist**: a reduce-to-`Bool` comparison prim + binary arithmetic (Tier-1), and
sequence/array + byte/string value representations (Tier-2 — the value model `Repr` =
`Binary`/`Ternary`/`Dense`/`Vsa` has neither). RFC-0032 scopes that surface (epic E19-1).

The maintainer decided (RFC-0032 §5 D6) that this surface lands **in `core 1.0.0`**, so the stdlib is
**fully self-hosted at the tag** rather than only its structural core. Because ADR-022's Status makes
**changing T1's criteria a supersede-only act** (house rule #3 — append-only), that decision is
enacted into the gate **here**, in a focused amending ADR, rather than by editing ADR-022's Accepted
normative content in place.

## 2. The amendment — T1's Definition of Done, extended

ADR-022 track T1 (`core 1.0.0`) previously required: the ADR-021 **Gate A** (A1–A5) and **Gate B**
(B1–B2) criteria, all **met**, plus the maintainer-reserved tag act (M-703). **This ADR adds one
prerequisite:**

> **T1+E19-1.** Before the `core 1.0.0` tag, epic **E19-1** (the RFC-0032 self-hosting-enablement
> surface) lands and is differential-tested: **D1** `eq`/`lt` comparison prims (M-747), **D2** binary
> arithmetic — surface `bit.and`/`bit.or` + never-silent carry-chain `add`/`sub` (M-748), **D3**
> `Repr::Seq` (M-749), **D4** `Repr::Bytes` (M-750), each with a checked three-way differential
> (L1-eval ≡ L0-interp ≡ AOT where it runs to closed L0) and honest tags, and the **M-752 conformance
> gate** green. Width-generics (M-753) is **not** a T1 item — it is owned by E11-1/`s10` (track T2,
> RFC-0032 D5) and gates E13-1 M-718's *general* surface, not the core tag.

The Gate A/B rows are **unchanged and remain met**. `M-703` (the tag act) now `depends_on E19-1`.

## 3. Consequence

- The `core 1.0.0` tag is **no longer simply "tag-ready"** — it waits on E19-1 (M-703 `depends_on`
  E19-1). It still does **not** wait on T2–T9.
- The trusted base grows by the RFC-0032 D1–D4 additions (two prims + `Repr::Seq` + `Repr::Bytes`).
  This is the deliberate KC-3 cost of full self-hosting at 1.0.0, bounded and justified per RFC-0032.
- ADR-022 §4 carries an append-only "amended by ADR-024" pointer; its T1 §4/§5 normative text is not
  rewritten (the supersede-to-change-criteria rule is honored).

## 4. Rationale & alternative considered

**Chosen:** full self-hosting at `core 1.0.0` — the language is genuinely usable with the stdlib in
`.myc` (not Rust-backed) at the headline tag, matching ADR-022 §8 Q1 ("1.0.0 = libs in Mycelium").

**Alternative (not taken):** a **leaner core tag now** (on the existing A/B rows) + the
self-hosting-enablement surface as a **1.1 value-model extension**. This would tag sooner and keep the
just-frozen value model smaller, but the stdlib would remain Rust-backed beyond Tier-0 at 1.0.0 —
weaker on the "fully usable, libs in Mycelium" criterion. The maintainer weighed the KC-3 cost against
that goal and chose full self-hosting (RFC-0032 §5 D6).

## 5. Definition of Done

- [ ] E19-1 lands: M-747/M-748/M-749/M-750 each differential-tested (three-way) with honest tags;
  never-silent (G2) on overflow / out-of-bounds / invalid-encoding; M-752 conformance gate green.
- [ ] ADR-022 carries the append-only "T1 amended by ADR-024" pointer (§4 note + changelog); its T1
  §4/§5 criteria text is otherwise unchanged.
- [ ] `M-703` `depends_on E19-1` (tracker); the core tag is cut only once E19-1 is met.
- [x] This ADR reaches **Accepted** (maintainer-ratified) and is indexed (`docs/adr/README.md`).
- **Enacted** with ADR-022 T1 at the `core 1.0.0` tag (append-only; M-703).

## 6. Grounding / honesty

- RFC-0032 §5 D6 — the in-`core`-1.0.0 placement decision this ADR enacts into the gate.
- RFC-0031 §5 D4 — the tiered migration; E19-1 is the surface its blocked tiers sequence behind.
- ADR-022 §4/§5 (track T1) — the gate amended (scoped; the rest stands).
- ADR-021 (Superseded by ADR-022) — the kernel Gate A/B carried into T1, unchanged here.
- KC-2/KC-3, G2, VR-5 — the trusted-base growth is deliberate; additions are never-silent + honestly
  tagged; nothing is "met" ahead of a checked differential test.

---

### Changelog

| Date | Status | Note |
|---|---|---|
| 2026-06-23 | **Accepted** | Maintainer-ratified scoped amendment of ADR-022 track T1: adds epic E19-1 (the RFC-0032 self-hosting-enablement surface — `eq`/`lt` prims, binary arithmetic, `Repr::Seq`, `Repr::Bytes`) to the `core 1.0.0` Definition of Done, so the stdlib is fully `.myc`-self-hosted at the tag. The ADR-021 Gate A/B rows in T1 are unchanged + remain met; M-703 now `depends_on` E19-1. Enacts RFC-0032 §5 D6 append-only (ADR-022's normative T1 text is not rewritten — only an "amended by ADR-024" pointer). |
