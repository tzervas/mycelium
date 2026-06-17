# Spec — Self-hosting readiness gate (M-502)

| Field | Value |
|---|---|
| **Status** | **Draft (needs-design)** (2026-06-17) — the *checkable verdict*, not a pre-declaration. The verdict below is **"not yet established"** and stays so until the surface actually supports authoring a stdlib module in Mycelium-lang (VR-5; never pre-declared). |
| **Tracks** | **M-502** (#150) — the M-346 precondition made checkable. Gates the Mycelium-lang *migration half* of M-510…M-520 (RFC-0016 §4.6 Phase 5b). |
| **Scope** | Enumerate the surface-language capabilities a stdlib module needs in order to be **authored in Mycelium-lang itself** (dogfooding, "free of other languages"), assess each against the landed corpus, and emit an honest **ready / not-yet** verdict. Does **not** gate the Rust-first modules (Batches P5-A/P5-B proceed against RFC-0016 now). |
| **Depends on** | RFC-0016 §4.6 (the Rust-first → Mycelium-lang migration the gate sits inside); RFC-0006/0007 (the surface + L1 calculus a module is written in); RFC-0011 (L0 `Match` + data-in-core); RFC-0012 (ambient representation); DN-06 (`phylum`/`nodule`); M-359 (the manifest); M-320 (the L1 term-language extension) |
| **Grounds on** | the Doc-Index status of each cited doc; `tools/github/issues.yaml` (M-320 #92, the surface track); the RFC-0016 §4.6 migration discipline |

---

## 1. What this gate decides

RFC-0016 builds every module **Rust-first** (ADR-007 — the trusted toolchain) and **migrates** it to
Mycelium-lang only "as the surface self-hosts" (§4.6). M-346's precondition — the stdlib is "decomposed once
the surface language is self-hosting enough to write stdlib modules in Mycelium itself" — is a **claim that
must not be pre-declared**. This gate makes it *checkable*: a capability checklist (§2) with an honest
per-capability status, composed into a single **ready / not-yet verdict** (§3). The verdict is the planning
analogue of the honesty rule — a `Proven`-style "self-hosting" claim is allowed **only** when the surface
actually clears the checklist; absent that, the verdict stays **not established** and says so (VR-5/G2).

It gates a *specific, narrow* thing: the **Mycelium-lang authoring** of a module (RFC-0016 §4.6 Phase 5b —
the `diag`/`recover` self-hosting targets M-510/M-520, and any later migration). It does **not** gate the
Rust-first specs or implementations (Batches P5-A/P5-B), which depend on RFC-0016, not on self-hosting.

## 2. The capability checklist (what authoring a stdlib module in Mycelium-lang requires)

Each row: the capability, why a stdlib module needs it, the corpus that owns it, and its **current** landed
status (read off the Doc-Index, not asserted). "Status" is honest and may be `not yet` — that is the point.

| # | Capability needed to author a module in Mycelium-lang | Why a stdlib module needs it | Corpus basis | Current status |
|---|---|---|---|---|
| 1 | **Data declarations + matching** (algebraic data, a registry `Σ`, flat `Match`) | every module defines + destructures values (`Option`/`Result`, collections, records) | RFC-0011 (L0 `Match`, `Construct`, content-addressed registry; WF6/WF7/WF8) | **landed (kernel/IR)** — RFC-0001 **r3 ENACTED**; M-210 differential covers the data fragment |
| 2 | **Functions + closures + recursion** (`Lam`/`App`/`Fix`; the `for` fold) | combinators, folds, the recursion every non-trivial module uses | RFC-0001 **r4** (`Lam`/`App`/`Fix`, closed-closure value model); RFC-0007 §4.8 (`for` fold) | **landed (kernel/IR)** — RFC-0001 r4 Accepted; the v0 calculus is ratified |
| 3 | **A concrete surface syntax** to *write* a module in (L3) — declarations, signatures, guarantee annotations | a human/agent authors `.myc` source; without it there is no Mycelium-lang to author *in* | RFC-0006 (L0–L3 layering; concrete L3 syntax) | **not yet** — RFC-0006 ratifies the *layering*; **concrete L3 syntax is KC-2-gated / deferred** (§10) |
| 4 | **Leaf emission / a working term-language prototype** (the interpreter executes authored terms) | a self-hosted module must *run* + be differential-tested against its Rust reference | M-320 (L1 term-language extension, interpreter/prototype) | **not yet** — **M-320 is open** (#92); leaf emission unblocked by RFC-0011 but the surface extension is in flight |
| 5 | **Honest guarantee tags expressible in the surface** (the `Exact ⊐ Proven ⊐ Empirical ⊐ Declared` lattice on a signature) | the §4.1 contract (C2) requires every op carry its tag; a self-hosted module must *say* its tags in-language | RFC-0001 (the lattice in `Meta`); RFC-0006 (guarantee annotations in the grammar) | **partial** — the lattice + `Meta` exist; the *surface annotation* rides on capability #3 |
| 6 | **Declared/bounded effects in the surface** (C6 — IO/time/rand/budget on a signature) | Tier-B modules (`io`/`fs`/`time`/`rand`) declare effects; the surface must express them | RFC-0014 (declared + bounded effects); RFC-0006 (effect surface) | **partial** — the effect *model* is Accepted/enacted (Rust); the *surface* form rides on #3 |
| 7 | **Ambient representation / scoped overrides** (to keep honesty's verbosity tolerable in-language) | offsets tension A so an authored module is not drowned in explicit reprs | RFC-0012 (ambient representation; I1/I2) | **Accepted, enactment-gated** — design normative; enactment is M-344 |
| 8 | **Organization + packaging surface** (`phylum`/`nodule`; the `mycelium-proj.toml` manifest; nodule headers) | a module *is* a `nodule` in the `std` `phylum`, declared by header + manifest | DN-06 (`phylum`/`nodule`); M-359 (manifest); the Nodule-Header spec | **landed (design)** — DN-06 **Resolved**; the manifest + header are specified |

## 3. The readiness verdict (honest — VR-5)

**Verdict: NOT YET established.** Self-hosting a stdlib module is **close on the substrate, not yet on the
surface.**

- **What is ready (the substrate):** the *semantic* foundation a module needs — data + matching (#1),
  functions/closures/recursion (#2), the guarantee lattice + effect model, and the organization/packaging
  surface (#8) — is **landed** in the kernel/IR and the Accepted corpus. The thing a module *means* is
  expressible.
- **What is not ready (the surface):** the *concrete syntax* to **author** a module in Mycelium-lang (#3)
  and a **running term-language prototype** to execute + differential-test it (#4) are **not yet landed** —
  the concrete L3 syntax is KC-2-gated (RFC-0006 §10) and **M-320 (#92) is open**. Capabilities #5/#6 (the
  surface forms of tags + effects) and #7 (ambient, enactment-gated M-344) ride on those.

Because authoring requires #3 and #4, the honest verdict is **not-yet**: no module can be *truthfully*
called "self-hosted" today, and claiming so would itself violate the honesty rule.

## 4. What this gate does and does not block

| Track | Gated by M-502? | Disposition |
|---|---|---|
| **RFC-0016 ratification (M-501)** + the per-module **specs** (this wave) | **no** | design-first; depends on the contract, not on self-hosting. Proceeds now. |
| **Rust-first module implementations** (Batches P5-A/P5-B) | **no** | ADR-007 trusted toolchain; the Rust reference is what a future self-hosted form is *differentialled against*. Proceeds now. |
| **Mycelium-lang authoring** of any module (RFC-0016 §4.6 Phase 5b) — incl. self-hosting `diag`/`recover` (M-510/M-520) | **yes** | **waits** on the verdict flipping to *ready* (i.e. #3 + #4 land). Until then a "self-hosted" claim is `not established`. |

This matches `docs/planning/phase-5.md` §3: M-502 gates only the *Mycelium-lang half*; the Rust-first work
does not wait on it.

## 5. How the verdict gets upgraded (the re-check trigger)

The verdict is **append-only with a status transition**, mirroring the ADR/RFC discipline. It flips
`not-yet → ready` only when the checklist actually clears — concretely when **M-320 (#92)** lands a running
term-language prototype and the **concrete L3 surface (RFC-0006 §10)** is no longer gated, such that a
*real* stdlib module (the smallest honest candidate — `core`/prelude or `diag`) can be authored in
Mycelium-lang **and** pass its NFR-7-style migration differential against the Rust reference (RFC-0016 §4.6
Phase 5b). The first module to clear it is the *evidence* that upgrades this verdict — never a forward
declaration. (The exact differential bar is RFC-0016 §8-Q5.)

## 6. Open questions (FLAGGED — carried from RFC-0016 §8)

- **(Q-a) The migration differential's bar.** What a self-hosted module must match (observable results only?
  tags + EXPLAIN bit-for-bit?) before the verdict flips for that module. → **RFC-0016 §8-Q5 / NFR-7**.
- **(Q-b) The smallest honest first target.** Whether the readiness *proof* is `core`/prelude (thinnest) or
  `diag` (the charter's named first self-hosting target, M-510). → ties RFC-0016 §4.6 + §8-Q1.
- **(Q-c) Surface coverage threshold.** How much of #5/#6 (tags + effects *in the surface*) must land before
  authoring is "enough" — a partial surface might author `core` but not `io`. → **RFC-0016 §8-Q3**.

## Meta — changelog

- **2026-06-17 — Draft (needs-design).** Stands up the M-502 self-hosting readiness gate as a **checkable
  verdict**: an eight-row capability checklist (data+matching · functions/closures/recursion · concrete L3
  surface · a running term-language prototype · surface guarantee tags · surface effects · ambient repr ·
  organization/packaging), each assessed against the landed corpus. Honest verdict: **not yet established**
  — the *substrate* (data/recursion/closures via RFC-0011/RFC-0001 r4, the lattice + effect model, DN-06
  packaging) is ready, but the *surface* to author + run a module (concrete L3 syntax, KC-2-gated; M-320
  #92, open) is not. Records what the gate blocks (the Mycelium-lang migration half of M-510…M-520) vs what
  proceeds regardless (RFC-0016 ratification, the per-module specs, the Rust-first implementations), the
  re-check trigger, and three FLAGs (→ RFC-0016 §8-Q1/Q3/Q5). Never pre-declared (VR-5). Append-only.
