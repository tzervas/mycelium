# Design Note DN-77 — Inject-Mode Build-Scope Dossier (RFC-0038 §8 → the M-961 slice)

| Field | Value |
|---|---|
| **Note** | DN-77 |
| **Status** | **Accepted** (2026-07-02 — accepted by the wave orchestrator under the maintainer's 2026-07-02 delegation (`Declared`), per the integration-reconcile promotion gate; the recommended **Option B** directed coherent slice is adopted as M-961's build scope, and M-961 built exactly that subset (see RFC-0038's 2026-07-02 changelog row: the §4.2/§5.1/§6.2/§7.1/§7.3/§8.4/§8.6/§8.5 claims flipped `Declared → Enacted` for the built slice; everything else stays `Declared`). RFC-0038 itself stays **Accepted** untouched. Was **Recommended, pending orchestrator acceptance** 2026-07-02; that history stands unchanged below — append-only forward transition, house rule #3.) The maintainer **delegated this build-scope decision to the wave orchestrator** (2026-07-02, session directive). |
| **Feeds** | **M-961** (enact the confirmed inject-mode subset); RFC-0038 §13 Implementation DoD; the deferral ledger below feeds M-836/M-837/M-838/M-839/M-842/M-847/M-849. |
| **Date** | July 2, 2026 |
| **Decides** | *Nothing normatively.* Records (1) the delegation + decision framing (§1); (2) the verified as-built baseline (§2, `Empirical`); (3) the option analysis (§3); (4) the recommended Phase-I-buildable subset (§4, ⟐); (5) the deferral ledger — every deferral flagged and mapped to its R&D issue, none dropped (§5, G2); (6) open questions + FLAGs (§6); (7) guarantee posture + DoD (§7). |
| **Task** | M-960 (kickoff `frz`, Lane B — Phase-I H2) |

> **Posture (transparency rule / VR-5 / G2).** Advisory dossier. Every claim below is tagged; the
> strongest tag anywhere in this note is `Empirical` (code/corpus inspection) — nothing is `Proven`,
> and the recommendation itself is `Declared` (an argued judgment, not a checked fact). Assent
> discipline (house rule #4): the recommendation follows the evidence of what Phase I can honestly
> enact, not the convenience of a smaller diff — where the directed subset has a real gap (the
> signature-scheme tension, §3.3), the gap is argued openly rather than papered over.

---

## §1 Purpose, delegation, and honest scope

RFC-0038 (Accepted 2026-06-29) ratifies the inject-mode security axis as **design**: the
`loose`/`inoculated` modes, `InjectCert`, `TrustRoot`, the enforcement-granularity axis, the scope
hierarchy, and the colony trust topology are all `Declared` — "design ratified, mechanism unbuilt"
(`frz` kickoff basis line). M-961 enacts a subset, flipping `Declared → Enacted` **only for what is
actually built** (VR-5). M-960 — this dossier — decides *which* subset.

**Delegation (`Exact` for the fact of it; recorded here so the authority chain is inspectable).**
The `frz` kickoff listed "RFC-0038 build-scope" as a maintainer decision that M-960 *prepares* but
does not take. On 2026-07-02 the maintainer **delegated the decision to the wave orchestrator**
(session directive). Accordingly this dossier drafts the options and a clear recommendation, and its
status is **"recommended, pending orchestrator acceptance"** — the orchestrator's acceptance (not
this note) is the gate M-961 builds against. If the orchestrator declines or amends, this note gains
an append-only changelog row recording that disposition; nothing is rewritten.

**Scope.** In: the build-now vs. defer-as-R&D partition over RFC-0038 §4–§8 for the M-961 slice.
Out: any design change to RFC-0038 (append-only — a change would be a supersession, not this note);
the deferred R&D itself (M-836/M-837/M-838/M-839/M-842/M-847/M-849 keep their own issues); the
kernel-freeze lane (M-958/M-959/M-969 — the inject lane is parallel to and does not gate the
freeze).

---

## §2 As-built baseline — what exists today (`Empirical`, verified 2026-07-02)

Verification-first (kickoff rule): the following was checked by direct source inspection, not
assumed from the RFC's §10 table.

| Element | Location | State |
|---|---|---|
| `Resolution` enum (`Compiled \| Interpreted \| Miss` pattern) | `crates/mycelium-mlir/src/inject.rs` | Exists; **no `inject_mode` dimension yet** |
| `InjectError` (`DispatchMiss(ContentHash)` · `Compile` · `Interp`) | `crates/mycelium-mlir/src/inject.rs` | Exists; **no `UnsignedCode` / `BadSignature` variants yet** |
| `Image::inject` / `Image::call` (compiled path + interpreter fallback, never-silent miss) | `crates/mycelium-mlir/src/inject.rs` | Exists — the gate's insertion point on both paths (I6) |
| `NativeArtifact` + `CrossBackendGate` (the `vr4_attestation` type) | `crates/mycelium-mlir/src/deploy.rs` (DN-18/M-620/M-630) | Exists — the attestation `InjectCert` carries |
| `InjectCert` / `TrustRoot` / `InjectMode` / `myc-prepare` signing | — | **Not built anywhere** (grep: no definitions) |
| `mycelium-sec` crate | `crates/mycelium-sec/` | Exists, but is the **M-367 tooling scanner** (`wild`-block audit, secrets/supply-chain orchestration) — it is *not* the inject gate; a plausible home for verify helpers, not a prebuilt one |

Consequence: the directed subset is **additive** over a real, tested insertion point — `inject.rs`
already carries the never-silent-refusal pattern (`DispatchMiss`) that `UnsignedCode`/`BadSignature`
structurally extend (RFC-0038 §10). Nothing below assumes unbuilt scaffolding exists.

---

## §3 Option analysis

Three candidate scopes for M-961, ordered by surface area. The maintainer's own direction already
points at Option B (the `frz` M-960/M-961 rows name the `whole`-app default + `InjectCert`/
`TrustRoot` verify + the two refusals + the deviation manifest); the analysis below tests that
direction rather than assuming it (house rule #4).

### §3.1 Option A — refusals-only stub (too small)

Build only the `InjectMode` knob and the two refusal variants: `inoculated` refuses **all**
injection (no verify path at all), `loose` permits with G2 tagging. No `InjectCert`, no `TrustRoot`.

- *For:* smallest diff; the refusal surface becomes real.
- *Against:* it enacts a **lockout**, not a **security axis** — an `inoculated` image could run no
  injected code at all, so no deployer could actually use the mode, and RFC-0038's central claim
  (signed injection admitted, unsigned refused) would stay wholly `Declared`. The M-961 user story
  ("production code requires a *valid* inject signature") is unmet. **Rejected.**

### §3.2 Option B — the directed coherent slice (⟐ recommended; detailed in §4)

The `whole`-grain compile/load-time application-signature default, the `InjectCert`→`TrustRoot`
verify path on both execution paths, the two never-silent refusals, `inject_mode` on `Resolution`,
and the default-plus-deviations manifest as the EXPLAIN surface. Everything else defers, flagged.

- *For:* this is the smallest slice in which the security axis is **usable end-to-end** — a deployer
  can sign an app, germinate an `inoculated` image with a `TrustRoot`, have valid injection admitted
  and unsigned/wrong-signer injection refused never-silently, and EXPLAIN the posture. Every piece
  is grounded on the §2 baseline; no piece depends on unresolved R&D (§K.2/§L/§M) for its *mechanism*
  (the one tension — the signature scheme — is resolved in §3.3). It matches RFC-0038 §8.6's own
  default row: application ⇒ `inoculated`/`whole`.
- *Against:* it leaves the granularity axis only partially enforced (§4 item 6) and the replay gap
  open (§L) — both are *disclosed*, not closed, per DN-44 §1.1. Acceptable: disclosure is the
  RFC's own posture for these (§5.2).

### §3.3 Option C — subset + production key-management (too much, and it front-runs R&D)

Option B plus a committed production signature scheme, key generation/rotation tooling, and the
`mycelium-proj.toml` key-declaration syntax.

- *For:* "batteries included" — a deployer gets the whole story at once.
- *Against:* key generation, rotation, manifest syntax, and the `myc-prepare` UX are **exactly the
  §K.2 open R&D** (M-836) that RFC-0038 §9 names as requiring "a separate deliberation". Building
  them now would silently close an explicitly-open R&D item (a G2/VR-5 violation) and would front-run
  the maintainer's M-836 disposition. **Rejected.**

**The signature-scheme tension (the one real design gap in Option B, argued openly).** A verify path
needs *some* signature primitive, yet the scheme/key story is §K.2 R&D. Two sub-options:

- **B-1 (⟐ recommended):** M-961 builds the gate against a small **`SignatureScheme` seam** (verify
  interface + a deterministic test scheme for the conformance suite; the production cipher plugs in
  when M-836 resolves). The `Enacted` flip then honestly covers the **gating mechanism** — mode
  gating, `TrustRoot` verify flow, refusals, EXPLAIN — while "production-grade signing" visibly stays
  `Declared` pending §K.2. No new cryptography dependency lands without its own supply-chain review.
- **B-2:** commit a concrete production scheme now (e.g. an Ed25519 crate). Rejected for this slice:
  the dependency choice is a supply-chain + KC-3 decision entangled with M-836's key-management
  deliberation; taking it here would decide half of §K.2 as a side effect. It is the **first**
  follow-on once M-836 lands, and the seam is designed so no gate code changes when it does.

This mirrors the repo's established pattern for unbuilt surface (DN-63/M-963: activate the directed
subset; the unbuilt refuses never-silently) — the seam refuses/flags rather than fakes.

---

## §4 The recommended Phase-I-buildable subset (⟐ — `Declared`; for orchestrator acceptance)

M-961 builds exactly the following seven items; RFC-0038's claims flip `Declared → Enacted` **only**
for these (VR-5), each verified per the RFC-0038 §13 conformance clauses cited.

1. **The two first-class modes** — `InjectMode::{Loose, Inoculated}` gating injection on **both**
   the compiled and interpreter-fallback paths (I6; §4.2/§5.3). `loose` permits unsigned injection
   with G2 tagging; empty `TrustRoot` ⇒ `loose`, explicit and inspectable (§7.1). [§13 (a)(c)]
2. **`InjectCert`** (§6.2 fields: `content_hash`, `signer`, `signature`, `vr4_attestation`,
   `issued_at`) with the signature over the dispatch key ‖ attestation digest. `issued_at` lands as
   the §L **placeholder** it is declared to be — carried, not yet enforced (§5 row L). [§13 (b)(h)]
3. **`TrustRoot` on `Image`** — set at germination, **immutable** thereafter (runtime mutation is an
   explicit error, I3/G2). Verification is always against the image's **own** `TrustRoot` (I4 —
   the local rule; the cross-colony *mesh* path defers, §5 row M-842). [§13 (d)]
4. **The two never-silent refusals** — `InjectError::UnsignedCode(ContentHash)` and
   `InjectError::BadSignature(ContentHash, SignerId)`, distinct (missing cert vs. wrong/untrusted
   signer), both carrying the exact rejected hash, on both paths (§5.1/§8.7). [§13 (b)(h) — the
   blacklist half of (h) defers with §8.8]
5. **`inject_mode` on `Resolution`** — every dispatch decision EXPLAIN-able for its security posture
   (§7.3, I7/NFR-3), composing freely with the RFC-0034 cert axis (§4.1). [§13 (e)(f)]
6. **`whole` enforcement grain as the `inoculated` application default** (§8.4/§8.6): the
   application signature is checked once at compile/load; its calls are then trusted — declared,
   mode-tagged, EXPLAIN-able, never a hidden weakening. The granularity **axis** (the knob + its
   tagging) is built; the `module`/`call` **enforcement paths** are deferred and **refuse
   never-silently** when selected (an explicit "grain not yet enforced" error — the DN-63 pattern),
   so no posture is silently downgraded to `whole`. [§13 (g), scoped to `whole` + never-silent
   refusal of the rest] The deferred `module`/`call` enforcement-path build-out is **M-847's**
   tracked scope (RFC-0038 §8.4-§8.7, `needs-design`) — this item's stub-plus-refusal *coordinates*
   with M-847 rather than substituting for it (§5 row 7, §6 F-1 correction).
7. **The default-plus-deviations manifest** (§8.5) — the effective policy rendered as a declared
   default plus enumerated deviations, surfaced via EXPLAIN. Phase-I scope: project-level default
   with per-inject override (§8.7's per-inject signing in an otherwise-`loose` context rides this);
   the **full seven-level hierarchy + config surface** is §M R&D and defers (§5 row M-838). This
   minimal manifest slice is a Phase-I precursor to, and *coordinates with* (does not duplicate),
   **M-847**, whose DoD covers "the granularity/scope/override + deviation-manifest realized
   Rust-first" for RFC-0038 §8.4-§8.7 as a whole. [§13 (g), manifest clause]

**Signature scheme:** per §3.3 B-1 — the `SignatureScheme` seam + test scheme; production cipher
gated on M-836. **Testing:** the §13 conformance suite parameterized over `InjectMode`, three-way
differential where a path is executable (M-961 DoD); a property test per bound.

**What this makes true for a deployer (the M-961 user story, restated honestly):** an `inoculated`
image refuses unsigned and wrong-signer injection never-silently and admits verified injection —
with the verification primitive swappable and its production hardening (`key management, replay`)
explicitly pending the flagged R&D below.

---

## §5 Deferral ledger — every deferral flagged, none dropped (G2)

Each row names what stays `Declared`, its owning R&D issue (verified present in
`tools/github/issues.yaml`, 2026-07-02), and why deferral is honest rather than a scope drop.

| Deferred surface | RFC-0038 § | Owning issue | Status today | Why deferred (grounding) |
|---|---|---|---|---|
| Key-management detail — keygen, rotation, project/phylum key granularity, manifest declaration syntax, `myc-prepare` UX, production cipher commitment | §8.2/§8.3/§K.2 | **M-836** | needs-design | Explicitly open R&D; RFC-0038 §9 requires "a separate deliberation"; §3.3 B-1 seam keeps M-961 unblocked without closing it |
| Replay/expiry mechanism (monotonic counter vs. expiry timestamp; cyst/checkpoint interaction) | §L | **M-837** | needs-design | Open R&D; `issued_at` is a declared placeholder; the replay gap stays **named and disclosed** per §5.2/DN-44 §1.1 — disclosed, not silently shipped as closed |
| Scoping-hierarchy config surface + resolution algorithm (`@certification` reuse vs. `@inject` annotation vs. germination parameter; I5 non-entanglement) | §8.5/§M | **M-838** | needs-design | §8.5 fixed the *shape*; the residual R&D is exactly the config surface — item 7 builds the manifest rendering without pre-deciding it |
| `myc-prepare` signed-spore emission + ADR-013 wire-format/schema (spore signature component end-to-end) | §6.3/§14 | **M-839** | needs-design | Depends on ADR-013's impl-pending artifact schema (RFC-0038 §14); M-961 verifies certs, `myc-prepare` *production* of them rides M-839/M-836 |
| Cross-colony mesh verify path (peer-inbound spore extraction/verification flow) | §7.2 | **M-842** | needs-design | The I4 own-`TrustRoot` *rule* is built (item 3); the mesh *transport/flow* needs RFC-0008's germination/mesh contracts (R8-Q5) |
| Colony trust topology — controller mode/stack, masterless propagation, node invalidation/blacklist | §8.8 | **M-849** | needs-design | RFC-0038 itself marks the controller protocol, propagation, and blacklist TTL semantics "open infrastructure R&D"; Phase-7 milestone |
| `module`/`call` enforcement-grain paths | §8.4-§8.7 | **M-847** (see §6 F-1 correction) | needs-design, depends_on [M-836, M-838, M-840] | Phase I ships the `whole` default (§8.6 app row) and refuses `module`/`call` never-silently (item 6); the enforcement-path build-out (plus the wider granularity/scope-resolution/deviation-manifest system) is M-847's tracked scope (RFC-0038 §8.4-§8.7) — items 6-7 coordinate with it, not duplicate it |
| Byzantine/adversarial mesh hardening | §5.2 | RFC-0008 **R8-Q4** (its own future RFC) | out of scope | RFC-0038 §5.2 already excludes it; restated so the exclusion survives into the build scope |

Related, not deferrals: **M-840** (inoculated gates the interpreter fallback — design) is
`status:done` and its design lands *in* item 1; **M-841** (naming: `inoculated` replaces `sealed`)
is `status:todo` docs work already ratified by RFC-0038 §4.2 — M-961 code uses `inoculated`
throughout, and M-841's remaining sweep is unaffected. Neither is dropped.

---

## §6 Open questions / FLAGs (for the orchestrator)

- **F-1 — CORRECTED 2026-07-02 (was: "untracked deferral, mint an M-id").** The original claim was
  factually wrong: verified in `tools/github/issues.yaml` (2026-07-02) that **M-847** — "Inject
  enforcement granularity + scope resolution + deviation manifest (RFC-0038 §8.4-§8.7)",
  `needs-design`, `depends_on: [M-836, M-838, M-840]`, DoD "the granularity/scope/override +
  deviation-manifest realized Rust-first" — **already owns** the `module`/`call` enforcement-grain
  build-out. **No new M-id should be minted**; doing so would create duplicate tracking (G2). M-838
  remains correctly cited (§5 row above) as the narrower scoping-config-surface R&D that M-847
  depends on. Item 6's never-silent refusal stub for `module`/`call` and item 7's Phase-I manifest
  slice (§4) are a minimal, *coordinating* instantiation of the surface M-847 will complete — not a
  separate or duplicate effort.
- **F-2 — shared-file updates.** `CHANGELOG.md`, `docs/Doc-Index.md`, `docs/api-index/`, and the
  M-960 `issues.yaml` close-out are orchestrator-owned; this note lands without touching them
  (FLAGged here for the integration-tier reconciliation).
- **F-3 — the B-1/B-2 cipher choice** (§3.3) is part of this recommendation; if the orchestrator
  prefers committing a production scheme now, that acceptance should trigger a supply-chain
  `/security-review` on the new dependency before M-961 builds against it.
- **F-4 — `TrustRoot` germination point.** §7.1 notes the germination *contract* is open
  (RFC-0008 R8-Q5). Item 3 builds `TrustRoot` as an `Image` construction parameter — the direction
  §7.1 declares — without waiting on R8-Q5; if the germination contract later moves, the parameter
  relocates under that RFC's supersession, not silently here.

---

## §7 Guarantee posture + Definition of Done

**Posture.** §2 is `Empirical` (direct source inspection, paths cited). The delegation fact in §1
is `Exact` (a recorded directive). Everything in §3–§5 tagged ⟐/`Declared` is argued judgment for
acceptance — **no claim here is `Proven`**, and nothing in this note moves any RFC-0038 claim's tag
(only M-961's landed code does that, and only for §4's seven items).

**DoD (this dossier).** (a) Buildable subset vs. deferred R&D enumerated with a recommendation (⟐)
— §3/§4; (b) every deferral mapped to its R&D issue — §5 (the `module`/`call` enforcement-grain row
maps to **M-847**, corrected 2026-07-02 per §6 F-1 — no unmapped surface remains); (c) no silent
scope drop (G2) — §5 + §6; (d) delegation cited and the status marked "recommended, pending
orchestrator acceptance" — header + §1. Met when the orchestrator records acceptance/amendment as an
append-only changelog row below.

---

## Changelog

| Date | Status | Note |
|---|---|---|
| 2026-07-02 | **Recommended, pending orchestrator acceptance** | Initial dossier (M-960, kickoff `frz` Lane B). Records the maintainer's 2026-07-02 delegation of the RFC-0038 build-scope decision to the wave orchestrator; verifies the as-built baseline (`inject.rs`/`deploy.rs`/`mycelium-sec`, `Empirical`); analyzes three scope options and recommends (⟐) the directed coherent slice — `loose`/`inoculated` gating, `InjectCert`→`TrustRoot` verify via a `SignatureScheme` seam (B-1; production cipher gated on M-836), never-silent `UnsignedCode`/`BadSignature` refusals on both paths, `inject_mode` on `Resolution`, the `whole`-grain application default with never-silent refusal of unbuilt grains, and the default-plus-deviations manifest. Deferral ledger maps §K.2/§L/§M, `myc-prepare` wire-format, cross-colony flow, and §8.8 topology to M-836/M-837/M-838/M-839/M-842/M-849; the `module`/`call` enforcement build-out FLAGged as untracked (F-1). Enacts nothing; RFC-0038 untouched. (Append-only; VR-5; G2.) |
| 2026-07-02 | **Recommended, pending orchestrator acceptance** (correction, adversarial-verification pass) | **Correction:** F-1 and deferral-ledger row 7 wrongly claimed the `module`/`call` enforcement-grain build-out had "no dedicated M-id." Verified in `tools/github/issues.yaml` that **M-847** ("Inject enforcement granularity + scope resolution + deviation manifest (RFC-0038 §8.4-§8.7)", `needs-design`, `depends_on: [M-836, M-838, M-840]`) already owns exactly that surface. F-1 and §5 row 7 revised to cite M-847 as the existing owner — no new M-id minted, avoiding duplicate tracking (G2). §4 items 6-7 (the `whole`-grain default's never-silent refusal stub and the Phase-I manifest slice) are unchanged in the recommended buildable subset, now explicitly noted as *coordinating with*, not duplicating, M-847's tracked scope. The recommendation direction (Option B, the Phase-I-buildable inject subset) is unchanged; only the grounding for F-1/row 7 was wrong and is corrected here (append-only; house rule #4). |
| 2026-07-02 | **Accepted** | Accepted by the wave orchestrator at the integration-reconcile promotion gate, under the maintainer's 2026-07-02 delegation (`Declared`). **Option B** (the Phase-I-buildable inject subset) adopted as M-961's build scope; M-961 built exactly that slice Rust-first (`crates/mycelium-mlir/src/{inject_gate,inject_cert,inject}.rs`) and RFC-0038's matching subset claims flipped `Declared → Enacted` (see RFC-0038 changelog, same date). The `module`/`call` grains + full scope-resolution + deviation manifest remain `Declared`/unbuilt, owned by M-847 (coordinates-with, not duplicated-by; F-1). Forward transition, append-only (house rule #3). |
