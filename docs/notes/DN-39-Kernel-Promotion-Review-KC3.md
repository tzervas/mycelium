# Design Note DN-39 — KC-3 Trusted-Core Promotion Review (Should Anything Join the Kernel? — No)

| Field | Value |
|---|---|
| **Note** | DN-39 |
| **Status** | **Accepted** (2026-06-26; **ratified by maintainer**) — the recommendation is **ratified: no promotions; the kernel boundary stays UNCHANGED** (KC-3 held on merit), the four-clause default-DENY bar and the *"a deterministic encoding is the most testable artifact, so it is the last thing to axiomatize into the kernel"* principle adopted. Prior: **Draft (advisory)** (2026-06-26; direction capture). Accepted ratifies the *recommendation* — it still **enacts nothing** (the kernel was already unchanged) and moves no other doc's status; the spore injectivity follow-up it named is a separate library change already landed (#617). Records the maintainer-commissioned **KC-3 trusted-core promotion review**: should any non-kernel functionality be promoted into the trusted core? **Recommendation: NO — zero promotions, the kernel boundary stays UNCHANGED.** The one candidate surfaced (the spore-identity pre-image encoding `content_address`, `crates/mycelium-spore/src/lib.rs`) is a decisive **KEEP-OUT**, failing the bar on clause (2) [its sole obligation — pre-image injectivity — is *verifiable* and self-detecting, so it must be **verified, not trusted**] and clause (3) [promotion would enlarge the unverified-but-trusted surface and freeze a self-described provisional format]. Default-DENY is sustained **on merit, not by default**. Per-verdict guarantee tags held at their basis (KEEP-OUT clause-(2) ground `Proven`; the original injectivity *risk* now `Proven` — a witness was constructed in PR #617 — and **FIXED**; boundary-unchanged `Proven` for the one candidate, `Declared` for the kernel-at-large since only one candidate was surfaced; VR-5 — no tag upgraded past its basis). Append-only; house rule #3. Enacts no code, moves no decision status. |
| **Feeds** | the **KC-3 small-auditable-kernel** invariant (CLAUDE.md house rule #5; `docs/Mycelium_Project_Foundation.md` KC-3) and the **trusted-core / TCB boundary** it governs (L0 Core IR + the reference interpreter + the content-addressing primitive + the guarantee lattice + the swap engine). Reaffirms the **L0/interpreter (trusted — roots the differential oracle, NFR-7) vs deploy-phase-encodings (verified, checkable)** split. Intersects **ADR-003** (content-addressed identity), **ADR-013 / M-368** (spore packaging), and **RFC-0008 R2** (the future wire-schema that supersedes `mycelium-spore-v0`). The actionable follow-up it names (encoding injectivity) is a **library** item under mycelium-spore — explicitly *not* a kernel change — and has since **landed on `dev`** (PR #617). |
| **Date** | June 26, 2026 |
| **Decides** | *Nothing normatively* — advisory + recommendation capture for maintainer ratification. Records (1) the **bar** the review applied (four conjunctive clauses — foundational · unverifiable-from-outside · net-trust-reducing · small-and-auditable — **default-DENY**, any one failed clause ⇒ KEEP-OUT); (2) the **result** — **kernel boundary UNCHANGED, zero promotions**, KC-3 held; (3) the **one candidate** (`content_address`) and its clause-by-clause **KEEP-OUT**; (4) the **security finding** — the v0 encoding had a real content-address injectivity flaw (a supply-chain substitution vector), now **FIXED** in PR #617; (5) the **generalizable principle** — *a deterministic encoding is the most testable artifact in the system, so it is the last thing that should be axiomatized into the kernel*; and (6) the **open-question ledger** + honest per-verdict guarantee posture. |
| **Task** | KC-3 trusted-core promotion review (maintainer-commissioned, 2026-06-26; advisory — no `issues.yaml` slot, enacts nothing) |

> **Posture (transparency rule / VR-5 / G2).** This note records an **advisory review**: should any
> non-kernel functionality be **promoted into the trusted core**, with explicit 2nd/3rd-order ripple +
> security-model/TCB reasoning? It **enacts nothing** — no kernel boundary moves, no RFC/ADR/DN status
> changes, no code or property test ships *from this note*. The grounding split is load-bearing and held
> throughout: the **KEEP-OUT verdict's clause-(2) ground** is `Proven`-grade (the source was read —
> `content_address` is a deterministic total function over the already-trusted blake3 primitive, the
> integrity re-hash round-trips are load-bearing, and injectivity sub-properties are property-tested); the
> **injectivity flaw** is reported `Proven`-and-**FIXED** (a concrete colliding witness was constructed in
> PR #617, which then closed it — the original risk is *no longer* the `Empirical`/plausible it would have
> been pre-witness; this note states both the original strength and the upgrade honestly, house rule #4);
> the **kernel-at-large** unchanged conclusion is `Declared` (only one candidate was surfaced — the review
> did not independently enumerate every possible promotion). Every gap is named, not buried (G2). Assent is
> on independently-verified merit, not deference (VR-5 applied to agreement, house rule #4).

---

## §1 Purpose & honest scope

The maintainer commissioned a **KC-3 promotion review**: with the kernel deliberately kept small (house
rule #5 — *small, auditable kernel*; `docs/Mycelium_Project_Foundation.md` KC-3 "the kernel never grows
without a net trust reduction"), should **any** non-kernel functionality be **promoted into the trusted
core**? The review treats a "promotion" as the strongest possible move — declaring a component
**axiomatically trusted**, i.e. exempt from the certificate/property verification machinery — and asks
of each candidate not just "is it important?" but "does importing it into the TCB *reduce* net trust, and
is it *unverifiable from outside* so that trusting it is the only option?" The reasoning is required to
be **second- and third-order**: what must move *with* a promoted component, what new trust dependencies
it creates, and what it does to the **threat model / TCB surface** — not merely a first-order "yes/no".

**The recommendation is NO — zero promotions; the kernel boundary stays UNCHANGED.** Exactly **one**
candidate was surfaced, and it is a decisive **KEEP-OUT**. KC-3 holds **on merit**: the burden of proof
for promotion was genuinely not met, not waved through by default-DENY.

This note **enacts nothing**. It is a recommendation the maintainer ratifies; adopting any follow-up
(the §5 injectivity hardening — already landed as a *library* change, PR #617) is a separate, forward-only
decision under mycelium-spore (ADR-003 / M-368 scope), **never** a kernel change.

---

## §2 The bar — four conjunctive clauses, default-DENY

A candidate is promoted into the trusted core **only if it clears all four** clauses below. The default
is **DENY**: a **single** failed clause ⇒ **KEEP-OUT**. The clauses are deliberately demanding because a
larger TCB is a strictly worse security position unless the import *removes* more trust than it adds.

1. **(1) Foundational.** The component must be *foundational to the kernel* — part of the trusted base
   that defines execution semantics — not merely important to some subsystem above the kernel.
   *Foundational-to-deployment ≠ foundational-to-the-kernel.*
2. **(2) Unverifiable-from-outside.** The component's correctness obligation must be one that **cannot be
   established by already-trusted checking machinery**. If a trusted property test or the round-trip
   oracle *can and does* establish the obligation, the component **must be verified, not trusted** — a
   thing that can be soundly checked must not be axiomatized. *This is the dispositive clause.*
3. **(3) Net-trust-reducing.** Keeping the component **out** must force an unsafe escape hatch / black
   box / silent-trust leak / duplicated trust. If keeping it out forces none of these — and promoting it
   *enlarges* the unverified-but-trusted surface — the clause **fails**.
4. **(4) Small + auditable.** The component must be small *and* auditing it must not import a foreign
   concern (presentation, encoding, I/O) into the kernel. SoC: KC-3's "kernel never grows" cuts against
   importing concerns the interpreter never needs.

---

## §3 Result — boundary UNCHANGED, zero promotions, KC-3 held

| Candidate | Verdict | One-line ground |
|---|---|---|
| `content_address` — the canonical-DAG pre-image encoding for spore identity (`crates/mycelium-spore/src/lib.rs`; called from `build_spore`; integrity twin `artifact_hash` in `registry.rs`; round-trip checks on publish/resolve) | **KEEP-OUT** | Fails clause (2): a deterministic total function over the already-trusted blake3 primitive whose only obligation (pre-image injectivity) is self-detecting (re-hash on publish/resolve) and property-testable — it must be **verified, not trusted**. Also fails (3): promotion would axiomatize a self-described provisional `mycelium-spore-v0` format and enlarge the unverified TCB. |

**No other components were submitted as candidates in this review batch** (see OQ-4). The
**kernel-at-large** unchanged conclusion therefore rests on this single adjudication plus the default-DENY
posture — held `Declared`, not `Proven`, and flagged as such (G2).

---

## §4 The one candidate — `content_address`, clause-by-clause KEEP-OUT

The candidate is the **canonical pre-image encoding** that `content_address` (in `mycelium-spore`) builds
before handing it to BLAKE3 — the byte-string whose hash *is* a spore's identity. It surfaced because it
is the one place in the reviewed slice that **defines a trusted invariant** ("identity = injective
pre-image") rather than merely *consuming* the already-trusted hash primitive.

- **(1) Foundational — PARTIALLY met / marginal.** The encoding fixes the *meaning* of artifact
  identity: which bytes enter the hash (kind, germination surface, code-by-hash DAG, dependency
  hash-edges) and which are excluded (name/version/authors metadata — the metadata-invariance test
  confirms it). The whole content-addressed deployment chain (publish/resolve, immutability conflict
  detection, dep pinning, ADR-003 reproducibility) trusts distinct spores to map to distinct addresses,
  so injectivity is foundational-*in-effect* to **deployment** integrity. **But** `mycelium-spore` is
  explicitly **"KC-3: above the kernel"** (its own module doc) — a package-manager/deployment library, a
  *consumer* of the content-addressing primitive, not the primitive itself. Foundational-to-deployment is
  **not** foundational-to-the-kernel. Marginal pass at best.
- **(2) Unverifiable-from-outside — FAILS (dispositive).** The encoding is a deterministic, total
  function of `(kind, surface, sources, deps) → String → BLAKE3`. The hash primitive
  (`mycelium_core::ContentHash` + blake3) is **already** in the TCB; this function only *arranges bytes*
  for it. Its sole correctness obligation — **injectivity** (no two semantically-distinct spores collide;
  no one spore drifts to two addresses) — is checkable by mechanisms that are **already trusted**, so it
  must be **verified, not trusted**: the address is **recomputed on every publish and resolve** (the
  integrity layer reads the object back and asserts the re-hash matches — a load-bearing G2 check that
  makes drift self-detecting), and injectivity is a **property, not an axiom** (metadata-invariance and
  code-change-sensitivity are already property-tested). A component a trusted check *can and does*
  establish must not be axiomatized. **Clause (2) is where the candidate dies.**
- **(3) Net-trust-reducing — FAILS.** Keeping it **out** forces no escape hatch, no black box, no
  silent-trust leak. The opposite: it already lives behind a never-silent, self-detecting integrity
  boundary and a property-test guard. Promotion would *remove* the function from verification and *add* a
  new axiom ("this exact byte format is injective forever") — enlarging the unverified-but-trusted
  surface. And the format is **explicitly provisional** (the header carried `mycelium-spore-v0`; the
  module doc announces supersession when RFC-0008 R2 lands). **Axiomatically trusting a format whose own
  docstring announces its future supersession is incoherent** — you cannot freeze a deliberately-unfrozen
  interface into the kernel.
- **(4) Small + auditable — ADVERSE.** The function is small (~two dozen lines), so "small" in isolation.
  But admitting it pulls a **string-serialization detail** into the kernel and, via clause-(2) logic,
  would invite every other "defines-a-trusted-invariant" format (the registry index entry format, the
  object-path layout) to follow — KC-3's "kernel never grows" cuts directly against importing
  presentation/encoding concerns the interpreter never needs.

**Net: fails (2) and (3) outright, (4) adverse, (1) marginal. Any single failure ⇒ KEEP-OUT; here there
are three.** Confidence high.

**Ripple (2nd/3rd-order), why promotion would be a regression, not a win.** Promotion would convert a
*checked* identity guarantee into an *asserted* one — a downgrade on the lattice
(`Proven`-by-construction → `Declared`-axiom) **masquerading as an upgrade** (a VR-5 inversion). It would
drag the encoding's *fallible inputs* into the trusted story with it: the filesystem walk's path
canonicalization (real, fallible I/O) and the `ProjectKind` string spellings (`"phylum"`/`"program"`/
`"script"`) would become **identity-bearing axioms** — a future rename would then *silently re-address
every spore*. You cannot trust an encoding without trusting its inputs' canonicalization, and that
canonicalization lives in untrusted I/O. Downstream resolve consumers and any future germination contract
(ADR-013 §4) would silently inherit "the byte format is injective" as an axiom, and the RFC-0008 R2
supersession would become a **TCB/kernel change** (re-axiomatization) rather than a cheap library version
bump behind a re-hash — converting an append-only, self-detecting migration into the most expensive kind.

---

## §5 Security finding — a real injectivity flaw (now FIXED)

The review surfaced a **real, present** injectivity question in the **v0** encoding, and it is the reason
the candidate even came up. The v0 pre-image was a space/newline-delimited string with **no
length-prefixing and no escaping**: code lines were `"  {path} {hash}\n"` and dependency lines were
`"  {name} {phylum} {hash}\n"`, over **author-influenced** inputs (the source `path` is a
project-relative filename; the dep `name`/`phylum`/`hash` are free-text manifest strings). A crafted
path or name containing a **space or newline** can shift a field boundary so that **two distinct DAGs
collapse to one pre-image string** — hence **one address**. That is a **second-pre-image-style aliasing
collision**: a **supply-chain substitution vector**, because spore identity *is* the content-addressing
supply-chain root for deployment (dep pinning, resolve-by-hash, immutability conflict detection). All
three dep fields are free-text manifest strings, so the attack needs **no** pre-image search and **no**
filesystem access.

**Guarantee tag (honest, with the upgrade stated).** The injectivity *risk* would, on the encoding shape
alone, have been `Empirical`/plausible — the standard concern for an unescaped delimited format, **not** a
proven break. It is now reported **`Proven`**: a **concrete colliding witness was constructed** as part of
the fix (PR #617), exhibiting the collision in a test rather than merely arguing it. Per house rule #4 the
direction this caveat cuts is surfaced explicitly — a witness *strengthens* the KEEP-OUT, and only after
the witness existed was the `Empirical → Proven` upgrade earned (VR-5: not upgraded past its basis until
the witness was in hand).

**Status: FIXED.** PR #617 (`fix(spore): close content-address injectivity flaw + unify encoder
(v0 → v1)`, commit `b160c4e`) **landed on `dev`**: the encoding moved to a **v1 length-prefixed
(netstring-style `<bytelen>:<bytes>`) form** where every variable-length field spans exactly its byte
count, so no embedded space/newline can forge a boundary, and section record-counts are recorded as
defense-in-depth ⇒ the pre-image is uniquely decodable, **injective by construction**. The fix also
**unified the encoder** to a **single canonical encoding function** (DRY — the parallel `verify`-path copy
stamping a stale `v0` while `build_spore` stamped `v1` is exactly the duplication that produced the split),
and added an **adversarial injectivity property test** over paths/names containing spaces and newlines.
The header bump `v0 → v1` re-addresses every spore — an **append-only** supersession of an explicitly
provisional format, acceptable pre-1.0 with no live registry. **The fix is the *opposite* of promotion:
it is the "verify, don't trust" response the bar's clause (2) demands.**

---

## §6 The generalizable principle — encodings are the *last* thing to axiomatize

The single review, though it changes **no code from this note**, hardens a **reusable principle that
reshapes how the whole trust surface is reasoned about** — the real "impacts everything" takeaway:

> **A deterministic encoding is the most testable artifact in the system, so a deterministic byte-format
> is the LAST thing that should ever be axiomatized into the kernel.** The correct response to a found
> injectivity risk is **more verification** (length-prefix or escape the fields so boundaries are
> unambiguous, plus an adversarial property test) — **never trusted-exemption.**

This is **VR-5 applied to the trust boundary itself**: declaring "this is trusted-correct" when its
correctness is *checkable* is **upgrading assent past its basis**. It reaffirms the standing split:

- **L0 Core IR + the reference interpreter** are **trusted** — they *define execution semantics*, the
  ground truth the differential oracle is rooted in (NFR-7). They must be trusted because there is no
  more-primitive checker to verify them *against*.
- **Deploy-phase encodings** (spore identity, registry index, object-path layout) are **verified** —
  they are **orthogonal** to the IR oracle and **checkable** (re-derivable, round-trip-asserting,
  property-testable). Grafting a deployment-encoding axiom onto an execution-semantics trust root would
  muddy *what the oracle vouches for*.

Keeping the boundary fixed keeps the encoding's whole **fallible input chain** (path canonicalization,
`ProjectKind` spellings) on the **verified** side of the line, preserves the **never-silent supply-chain
gate** (the publish/resolve re-hash), and spares the kernel a **perpetual obligation** to audit a string
format's injectivity against adversarial inputs — an obligation the property-test guard discharges for
free.

---

## §7 Recommended kernel boundary — UNCHANGED

```
TRUSTED CORE (TCB) — recommended UNCHANGED:
  • L0 Core IR
  • the reference interpreter (mycelium-interp / mycelium-l1 / mycelium-core)
  • the content-addressing primitive (mycelium_core::ContentHash + blake3)
  • the guarantee lattice (Exact ⊐ Proven ⊐ Empirical ⊐ Declared)
  • the swap engine

VALIDATED-AGAINST-THE-ORACLE, NOT TRUSTED (NFR-7):
  • the AOT / MLIR path
  • mir-passes

ABOVE THE KERNEL — verified consumer, NOT promoted:
  • mycelium-spore (and its content_address encoding)
```

No promotions. `mycelium-spore` and its `content_address` encoding stay **explicitly above the kernel** — a
*verified consumer* of the content-addressing primitive, **not** the primitive itself. The AOT/MLIR path
and mir-passes remain **validated against the oracle, not trusted** (NFR-7). KC-3 — "the kernel never
grows without a net trust reduction" — is **held**: the sole candidate did not meet the burden of proof,
so default-DENY stands **on merit**.

---

## §8 Open-question ledger

- **OQ-1 (resolved as a library item, not a promotion).** The v0 injectivity flaw — the actionable
  follow-up. **Done:** added (as a *verified library* change under ADR-003 / M-368 scope) a length-prefixed
  encoding + adversarial injectivity property test (PR #617). Recorded here for completeness — explicitly
  the **opposite** of a kernel promotion.
- **OQ-2 (supersession must stay a library bump).** When the RFC-0008 R2 wire-schema supersedes the
  provisional spore encoding, **confirm the migration stays an append-only, re-hash-behind library
  version bump** (self-detecting at the publish/resolve round-trips) and is **NOT** treated as a TCB/kernel
  change. The boundary recommendation here must **never** be read as pulling the encoding inward at
  supersession time.
- **OQ-3 (`ProjectKind` spelling stability).** The `ProjectKind` string spellings
  (`"phylum"`/`"program"`/`"script"`) are currently **identity-bearing** in the pre-image. Should they be
  **pinned by an explicit ADR/test** so a future rename cannot silently re-address every spore? A
  stability concern for the **verified library**, not a trust-promotion concern.
- **OQ-4 (other formats not adjudicated this batch).** No other components were submitted as candidates.
  If the broader survey intended additional slices (the registry index entry format, the object-path
  layout), they were **not adjudicated here**; this recommendation covers only the one candidate provided,
  and the kernel-at-large default-deny conclusion rests on that plus the unchanged-boundary default
  (`Declared`, per §1's posture).

---

## §9 Guarantee posture & house-rules note

**ENACTS NOTHING.** This is an advisory KC-3 promotion-review **recommendation** that the maintainer
ratifies; it changes no code, no kernel boundary, and no decision status. **Append-only:** nothing here
supersedes or rewrites any ADR/RFC/DN — adopting any follow-up (the §5/OQ-1 injectivity hardening, already
landed as a library change) is a separate, **forward-only** decision/issue under mycelium-spore, not a
kernel change.

**Per-verdict guarantee tags (transparency rule / lattice `Exact ⊐ Proven ⊐ Empirical ⊐ Declared`):**

- **KEEP-OUT for `content_address` — `Proven`-grade w.r.t. its clause-(2) ground.** The source was read
  and the load-bearing facts confirmed: `content_address` is a deterministic total function over the
  already-TCB blake3 primitive; the integrity re-hash round-trips exist and are load-bearing on both
  publish and resolve; injectivity sub-properties are already property-tested. The "must be verified, not
  trusted" conclusion follows from **checked facts**, so clause (2)'s failure is *established*, not
  asserted.
- **The injectivity flaw / supply-chain substitution path — `Proven` and FIXED.** On encoding shape alone
  it would have been `Empirical`/plausible; it is reported **`Proven`** because a concrete colliding
  **witness was constructed** in PR #617 — which then **closed** it (v1 length-prefixing + a single
  canonical encoder + an adversarial property test, landed on `dev`). House rule #4: the strength and the
  upgrade are stated honestly; the witness was required before the `Empirical → Proven` upgrade (VR-5).
- **Recommended boundary = UNCHANGED — `Proven` for the one candidate, `Declared` for the
  kernel-at-large.** The unchanged conclusion is established for the single candidate adjudicated; for the
  kernel-at-large it rests on the default-DENY posture plus the fact that **only one candidate was
  submitted** (the review did not independently enumerate every possible promotion — OQ-4). Tagged
  `Declared` accordingly; not upgraded past its basis.
- **Grounding / assent (house rule #4, VR-5 applied to agreement).** The KEEP-OUT is affirmed on
  **independently-verified merit** — the source and tests were re-read, not the verdict restated — not by
  deference. **No sycophantic upgrade:** the risk was held to `Empirical` until a witness existed, then
  upgraded only on that checked basis. KC-3 default-DENY is held because the burden of proof for promotion
  was **genuinely not met**, not because deny is the easy answer.

**Definition of Done.** The Draft → Accepted gate: the maintainer ratifies (a) the recommendation of
**zero promotions / kernel boundary UNCHANGED**, (b) the **four-clause default-DENY bar**, and (c) the
**generalizable principle** (§6 — encodings are verified, not axiomatized). Accepting the *recommendation*
neither enacts code nor upgrades any guarantee tag past its stated basis (VR-5). CHANGELOG / Doc-Index /
issues.yaml / docs/api-index owned by the integrating parent.

---

## Meta — changelog

- **2026-06-26 — Created (Draft, advisory) — authored.** Records the maintainer-commissioned **KC-3
  trusted-core promotion review** (task #33): should any non-kernel functionality be promoted into the
  trusted core, with explicit 2nd/3rd-order ripple + security-model/TCB reasoning? **Recommendation: NO —
  zero promotions, kernel boundary UNCHANGED, KC-3 held on merit.** Captures (§2) the **four-clause
  default-DENY bar** (foundational · unverifiable-from-outside · net-trust-reducing · small-auditable; any
  one failed clause ⇒ KEEP-OUT); (§3–§4) the **one candidate** — the spore-identity pre-image encoding
  `content_address` (`crates/mycelium-spore/src/lib.rs`) — and its decisive **KEEP-OUT** (fails clause (2)
  [its sole obligation, pre-image injectivity, is verifiable and self-detecting via publish/resolve
  re-hash round-trips and property tests — must be **verified, not trusted**] and clause (3) [promotion
  would axiomatize a self-described provisional `mycelium-spore-v0` format and enlarge the unverified TCB],
  with (4) adverse and (1) marginal), plus the 2nd/3rd-order ripple (fallible-input drag-in: path
  canonicalization + `ProjectKind` spellings become identity-bearing axioms; a VR-5 lattice inversion;
  RFC-0008 R2 supersession would become a TCB change); (§5) the **security finding** — the v0 unescaped,
  non-length-prefixed delimited encoding over author-influenced path/name/hash admitted a field-boundary
  aliasing collision (a supply-chain substitution vector), tagged **`Proven`** (a concrete witness was
  constructed) and **FIXED** in **PR #617** (`b160c4e`, landed on `dev`: v1 length-prefixed encoding +
  single canonical encoder + adversarial injectivity property test); (§6) the **generalizable principle**
  (a deterministic encoding is the most testable artifact in the system ⇒ the last thing to axiomatize
  into the kernel; the L0/interpreter-trusted vs deploy-encodings-verified split reaffirmed; VR-5 applied
  to the trust boundary); (§7) the **recommended boundary** (UNCHANGED — L0 Core IR + interp/l1/core +
  content-addressing primitive + guarantee lattice + swap engine; mlir/mir-passes validated-not-trusted;
  mycelium-spore stays above the kernel); (§8) the **open-question ledger** (encoding injectivity follow-up
  — done as a library item; RFC-0008 R2 supersession must stay a library bump; `ProjectKind` spelling
  stability pin; other formats not adjudicated this batch); and (§9) the **honest per-verdict guarantee
  posture** (KEEP-OUT clause-(2) ground `Proven`; flaw `Proven` & FIXED; boundary `Proven` for the one
  candidate / `Declared` for the kernel-at-large). DoD = the Draft → Accepted gate (maintainer ratifies the
  recommendation + the bar + the principle). **Enacts nothing; moves no status; changes no normative
  text.** CHANGELOG / Doc-Index / issues.yaml / docs/api-index owned by the integrating parent.
  (Append-only; VR-5; G2.)
- **Ratified Draft → Accepted (2026-06-26).** The maintainer ratified the recommendation: **no
  promotions — the kernel boundary stays UNCHANGED** (KC-3 held on merit), the four-clause default-DENY
  bar and the *"deterministic encodings must be verified, not trusted"* principle adopted. The status
  move enacts no code and upgrades no guarantee (the kernel was already unchanged; the KEEP-OUT verdict
  and the boundary-unchanged conclusion keep their `Proven`/`Declared` basis — VR-5). The spore
  injectivity follow-up it named is a separate, already-landed library change (#617).
