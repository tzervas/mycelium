# RFC-0038 — Inject-Mode Security Axis

| Field | Value |
|---|---|
| **RFC** | 0038 |
| **Status** | **Proposed** — ratified design direction (DN-64 §7 OQ-K…OQ-Q, 2026-06-29). Enacts nothing; named open R&D items are explicit, not silently closed (G2/VR-5). Advances to Accepted when the maintainer ratifies this document; to Enacted when the inject-mode mechanism lands Rust-first. |
| **Type** | Normative / foundational (once Accepted) — introduces the inject-security axis as a first-class, independently configurable policy orthogonal to the certification axis (RFC-0034) |
| **Date** | 2026-06-29 |
| **Task** | M-836…M-842 (DN-64 §7 OQ-K…OQ-Q disposition tasks) |
| **Feeds** | ADR-013 §2 component 4 (spore signature — deferred; this RFC is the design vehicle); ADR-017 (hot-inject mechanism; this RFC adds the security gate on top); RFC-0008 R8-Q4 (adversarial mesh — out of scope here; named) |
| **Depends on** | ADR-003 (content-addressed identity — the dispatch key); ADR-013 (spore as deployable unit — `InjectCert` is its signature component); ADR-016 (ABI dispatch table); ADR-017 (hot-inject mechanism); ADR-006 (EXPLAIN-able dispatch — extended here with inject-mode dimension); DN-18/M-620/M-630 (`NativeArtifact` + VR-4 cross-backend attestation); RFC-0008 §4 (germination — `TrustRoot` set at image start); RFC-0034 §4/§8 (knob matrix + orthogonality — the cert axis is independent); DN-44 §1.1 (disclosed-and-guided residual insecurity) |
| **Does NOT change status of** | RFC-0034 (Enacted — with code), ADR-013 (Accepted), ADR-016 (Accepted), ADR-017 (Accepted), RFC-0008 (Accepted). All references are cross-references, never rewrites (house rule #3 — append-only). |

> **Posture (transparency rule / VR-5 / G2).** All design claims in this RFC are `Declared`
> unless a cited, checked basis exists. Corpus facts that quote an existing Enacted/Accepted
> document are tagged `Exact` (citing that document). Nothing in this RFC is `Proven` or
> `Enacted`; mechanization and code are the basis for future upgrades. Open R&D items are
> named explicitly in §K, §L, §M — none is silently closed (G2). The maintainer dispositions
> in DN-64 §7 are the authoritative source; where this RFC restates them the restatement is
> `Declared` at the same strength the maintainer set.

---

## §1 Motivation

ADR-017 (Accepted) establishes hot-inject as a **correctness and atomicity** mechanism:
content-addressing dissolves the atomicity hazard (a change is a new hash, never an in-place
mutation), and `Image::call()` refuses unknown hashes with an explicit
`InjectError::DispatchMiss` (`Exact` — ADR-017 Decision item 5). **No signing requirement or
unsigned-code refusal is present in the current corpus.**

The gap: a content hash is collision-resistant identity but not authentication. An attacker who
can write a new node to the Image can register an arbitrary definition under its own valid hash.
The concrete attack vector is a malicious `.so` artifact loaded via `dlopen` (the JIT substrate
— `crates/mycelium-mlir/src/jit.rs`, M-340; ADR-014/DN-44 §2 — eight `unsafe` blocks confined
there). Content-addressing prevents a silently-stale entry but does not prevent injecting a
newly-minted malicious definition; the signing gate must verify that the artifact was produced
by a **trusted preparation phase**. (`Exact` for the gap description — DN-64 §4.1.)

DN-44 §1 states the security thesis: "the only security vulnerabilities that can exist are ones
a developer introduces into their own `.myc` programs." Hot-inject of arbitrary unsigned code
in production is a direct counter to that thesis.

This RFC ratifies the design direction the maintainer resolved in DN-64 §7 (OQ-K through OQ-Q,
2026-06-29) and names the remaining open R&D items explicitly.

---

## §2 User stories

- As a **language developer** iterating locally, I want to hot-inject unsigned compiled
  definitions without ceremony, with the unsigned status tagged on every injected call so I am
  never surprised about what guarantee level I am running at.
- As a **production operator**, I want the runtime to refuse injection of any compiled artifact
  that does not carry a valid cert from my project's signing key, so that a malicious `.so`
  cannot be loaded even if an attacker can write to the Image node.
- As a **security auditor**, I want to inspect the inject-security posture (mode, trust-root,
  per-call inject-mode tag) from the EXPLAIN channel — the same interface as every other dispatch
  decision — without consulting logs or secondary tooling.
- As a **phylum author** deploying across colonies in a mesh, I want the receiving colony to
  verify the `InjectCert` against its own trust root, so that trust is never silently inherited
  from a sender whose key I may not recognise.
- As a **developer building a script**, I want the signing requirement to match what I am
  building (a script has a lighter scope than a library or application), so the gate is graded
  to my scope of work, not uniformly expensive.

---

## §3 Invariants (normative as proposed)

These extend ADR-017's never-silent contract to the security axis. They are normative once this
RFC is Accepted; until then they are the design target (`Declared`).

- **I1 — Unsigned status is never silent.** In `loose` mode every injected call carries a
  G2-tag marking unsigned status. No call site may suppress or obscure it. (Extends G2 to the
  inject-security axis.)
- **I2 — Unsigned code yields an explicit refusal in `inoculated` mode.** `InjectError::UnsignedCode(ContentHash)`
  is the designated variant — explicit, carrying the exact rejected hash, never swallowed. The
  pattern is structurally isomorphic to the existing `InjectError::DispatchMiss` (ADR-017
  Decision item 5). (`Declared` — no code yet; the pattern is `Exact`-cited from ADR-017.)
- **I3 — `TrustRoot` is immutable after germination.** A `TrustRoot` set at image start (the
  germination point — RFC-0008 §4) cannot be changed at runtime. An attempt to change it is an
  explicit error, never a silent downgrade. An empty `TrustRoot` means `loose` mode.
- **I4 — Cross-colony trust is never inherited.** A receiving colony verifies `InjectCert`
  validity, trustedness, and currency (not expired or superseded) against its own `TrustRoot`.
  It never inherits or proxies the sender's trust. (`Declared` — direction set by OQ-Q.)
- **I5 — Inject-mode is independently configurable from cert mode.** The two axes are
  orthogonal: `fast`+`inoculated` is valid; `certified`+`loose` is valid. No coupling by
  design (RFC-0034 §8). (RFC-0034 §8 establishes the dispatch mechanism is independent of
  cert-mode — `Exact`; the orthogonality of the *inject-security axis* is a design property of
  this RFC — `Declared`.)
- **I6 — The interpreter fallback path is also gated in `inoculated` mode.** An interpreted
  definition injected into an `inoculated` image requires a valid `InjectCert` — uniform
  enforcement, no back-door through the interpreter path. (Direction: OQ-O, `Declared`;
  threat rationale in §5.)
- **I7 — The inject-mode dimension is EXPLAIN-able.** Every dispatch decision carries its
  inject-mode tag in the `Resolution` enum, inspectable through the same EXPLAIN channel as
  the execution-path tag (ADR-006/G2). (`Declared`.)

---

## §4 The two axes — orthogonality (normative as proposed)

The cert axis (RFC-0034) governs swap-certificate emission and checking. The inject-security
axis governs whether injection requires a signed artifact. They are **orthogonal axes**:
independent knobs that compose freely. (RFC-0034 §8 establishes the dispatch mechanism is
independent of cert-mode — `Exact`; that the inject-security axis composes freely with the cert
axis is a design property of this RFC — `Declared`.)

### §4.1 Extended knob matrix

The RFC-0034 §4 knob matrix is extended with two inject-axis rows (`Declared`):

| Knob | Phase | `fast` (cert) | `certified` (cert) | `loose` (inject) | `inoculated` (inject) |
|---|---|---|---|---|---|
| Swap certificates | runtime | off | full | independent | independent |
| Value provenance tags | runtime | `Exact`/`Declared` only | full | independent | independent |
| Hot-inject gate | preparation/runtime | independent | independent | unsigned permitted | `InjectCert` required |
| Mode tag on results | runtime | never silent (G2) | never silent (G2) | G2-tagged on injected calls | G2-tagged on all calls |
| Interpreter fallback gate | runtime | independent | independent | unsigned permitted | `InjectCert` required (I6) |

The cert axis and inject axis compose freely across all four combinations. No row of the
inject axis implies a row of the cert axis, and vice versa.

### §4.2 The two first-class inject modes

- **`loose`** (local-dev) — unsigned injection permitted; every injected call is G2-tagged
  with its unsigned status so unsigned code is **never silent**. This is the development mode:
  fast iteration, no key ceremony. (`Declared`; direction: OQ-P.)
- **`inoculated`** (production) — injection requires a valid `InjectCert` from a signer in
  the image's `TrustRoot`; unsigned code yields `InjectError::UnsignedCode(ContentHash)`, an
  explicit never-silent refusal. The interpreter fallback path is equally gated (I6). This is
  the **secured, strictly-enforced production tier**. (`Declared`; direction: OQ-O/OQ-P.)

**Naming rationale** (`Exact` — DN-64 §7 OQ-P naming note): `inoculated` is the biological
term for introducing a *verified* organism into a substrate — on-brand with the fungal lexicon,
and it earns its three-test gate (T-map: a sealed/verified admission; T-illuminate: teaches
"only verified code is admitted"; T-learn: human- and LLM-legible). The draft name `sealed` is
superseded by `inoculated` for all forward uses; §4 of DN-64 is preserved as the commissioning
draft (append-only).

---

## §5 Threat model and boundaries (honest disclosure)

### §5.1 What the `inoculated` gate closes

An `inoculated` image refuses injection of any compiled `.so` artifact whose `ContentHash` is
not accompanied by a valid `InjectCert` from a trusted `SignerId`. This closes the attack
vector in §1: a malicious artifact minted by an attacker cannot be injected without a valid
signature from the project's preparation-phase key, even if the attacker can write to the
Image node. (`Declared` — mechanism unbuilt; threat model `Exact`-cited from DN-64 §4.1/§4.5.)

### §5.2 What the gate does not close (G2 / DN-44 §1.1)

The following are **explicitly outside scope** — named, not buried:

- **Compromised signing key.** A key management failure that exposes the project signing key
  nullifies the gate. Key rotation and revocation machinery are open R&D (§L).
- **Byzantine or adversarial injection from a peer colony in a mesh.** Cross-colony injection
  is addressed structurally by I4 (verify against own TrustRoot), but Byzantine-fault-tolerant
  mesh hardening is deferred (RFC-0008 R8-Q4 — its own future RFC).
- **Content-addressed identity replay.** A content hash is not a timestamp. An attacker with
  access to a valid `InjectCert` for a known-vulnerable prior version of a definition can
  inject that prior version. The replay/expiry mechanism is open R&D (§L); until it lands,
  this gap is named and disclosed per DN-44 §1.1.
- **Supply-chain compromise before `myc-prepare`.** If the build pipeline is compromised before
  the preparation step, the cert is issued over malicious code. This is not a language-level
  gap; it is addressed by supply-chain posture (RFC-0035 / DN-44 scope).

### §5.3 Interpreter fallback rationale (OQ-O)

The `inoculated` gate applies to the interpreter fallback path equally (I6). Rationale
(`Declared`): although the reference interpreter (ADR-007) is the trusted, `forbid(unsafe)`
base, an unverified definition injected via the interpreter path still bypasses the
authentication check. `inoculated` enforces uniformly — the gate is about **who authorized
this code**, not about which execution path it takes. A developer who wants unsigned
interpretation uses `loose` mode explicitly (never-silent: the mode tag shows it).

---

## §6 `InjectCert` — the spore's signature component (§N)

### §6.1 Unification with ADR-013 spore signatures (OQ-N)

**The `InjectCert` is the spore's signature component** (ADR-013 §2 component 4: "artifact
metadata — provenance, guarantee/bound certificates, signatures" — deferred). This RFC is the
design vehicle that ratifies what that component is. (`Exact` — ADR-013 §2 names the component
and defers it; `Declared` for the structure below.)

`myc-prepare` produces a **signed spore**: the spore is simultaneously the deployable unit
(ADR-013) and the inject gate. There is no separate artifact to manage — the spore's signature
component is the `InjectCert`. (`Declared` — direction: OQ-N.)

### §6.2 `InjectCert` structure (`Declared`)

```
InjectCert {
    content_hash:       ContentHash,         // the dispatch key (ADR-003/ADR-016/017)
                                             // — signature is over the dispatch key itself
    signer:             SignerId,            // signing authority public-key fingerprint
    signature:          Bytes,              // over content_hash ‖ vr4_attestation_digest
    vr4_attestation:    CrossBackendGate,   // DN-18/M-630 no-opaque-lowering attestation
    issued_at:          Timestamp,          // replay-attack surface — open R&D §L
}
```

Key properties (`Declared` unless noted):

- **The signature is over the dispatch key itself.** `content_hash` in the `InjectCert` is
  exactly the dispatch key (ADR-016/017). No secondary identity can drift from the dispatch
  key. (`Exact` for the dispatch-key identity — ADR-016/017; `Declared` for the
  signature-over-it.)
- **The `vr4_attestation` fuses security with transparency.** The DN-18/M-630
  no-opaque-lowering attestation is carried inside the `InjectCert`. The cert asserts not only
  "this came from an authorized party" but "this came from an authorized party and the lowering
  is auditable — no black-box pass" (ADR-006 / VR-4). Security and transparency are fused in
  one artifact, not two separate channels. (`Exact` for VR-4/CrossBackendGate — DN-18/M-630;
  `Declared` for the fusion design.)
- **Content-addressing is natural revocation.** An edited definition produces a new hash; the
  `InjectCert` for the old hash is automatically invalid for the new hash without any revocation
  list. The content hash is simultaneously the version and the revocation signal. (`Declared` —
  derives from ADR-003/017; the argument is `Exact`, the inject-security application is
  `Declared`.)

### §6.3 `myc-prepare` (`Declared`)

`myc-prepare` is the toolchain preparation step that:

1. Compiles the definition via the native backend (MLIR-LLVM, RFC-0004/ADR-009).
2. Produces a `NativeArtifact` (DN-18/M-620) — content-hash-derived identity, dumpable IR,
   VR-4 attestation.
3. Signs the `ContentHash` concatenated with the `vr4_attestation` digest, using the project
   signing key (scope: §K).
4. Emits an `InjectCert` embedding the `NativeArtifact`'s identity and attestation.

This is the **preparation phase** — the authorization phase, separate from the build phase,
analogous to a release-signing step. (`Declared`.)

---

## §7 `TrustRoot` and `inject_mode` on `Resolution` (normative as proposed)

### §7.1 `TrustRoot` (`Declared`)

`TrustRoot` is the set of trusted `SignerId`s associated with an image. It is:

- Set at germination (the image start event — RFC-0008 §4.4 uses "germinating" descriptively;
  the germination *contract* is an open question per RFC-0008 §8 R8-Q5). That `TrustRoot` is set
  at germination, and that germination is the right initialization point, are both `Declared`
  here (a sensible direction, not a fact RFC-0008 §4 establishes).
- **Immutable** after germination. Any attempt to change it at runtime is an explicit error,
  never a silent downgrade (I3 / G2).
- An **empty `TrustRoot`** means `loose` mode: no signing requirement is enforced. This is
  a deliberate design choice that keeps the default for development unobtrusive while making
  the empty case explicit and inspectable — it is never silent (G2).

### §7.2 Cross-colony trust (OQ-Q, `Declared`)

When a colony receives an inject request from a peer colony in the mesh:

1. The receiving colony extracts the `InjectCert` from the inbound spore.
2. It verifies the cert is well-formed, that `signer` is in its **own** `TrustRoot`, and that
   the cert is not expired or superseded (per the replay/expiry mechanism — open R&D §L).
3. Verification is against the **receiving colony's own trust**, never the sender's. The
   sender's trust posture is not inherited, proxied, or consulted.

Rationale: trust is colony-local. A colony in a mesh does not implicitly extend trust to every
other colony's signing authority. This is I4 operationalized. (`Declared` — direction OQ-Q.)

### §7.3 `inject_mode` dimension on `Resolution` (`Declared`)

ADR-006 / ADR-017 decision 5 establish that dispatch decisions are EXPLAIN-able. The existing
`Resolution` enum (`Compiled | Interpreted | Miss`) is extended with an inject-mode dimension:

```
Resolution::Compiled    { inject_mode: InjectMode, ... }
Resolution::Interpreted { inject_mode: InjectMode, ... }
```

where `InjectMode` is `Loose { unsigned_signer: None }` or `Inoculated { cert: InjectCert }`.
Every dispatch decision is then inspectable for both its execution path and its security posture
through the same EXPLAIN channel. (`Declared` — I7 / ADR-006 / G2.)

---

## §8 Signing authority and scope (§K)

### §8.1 Ratified direction (OQ-K)

The signing requirement is **project-scoped, graded, and dev-configurable by scope of work**.
The key principle: the signing authority and the burden it imposes scale to what the developer
is building. A script has a lighter scope than a nodule; a nodule has a lighter scope than a
library or application. The gate is not uniformly expensive. (`Declared` — direction OQ-K.)

### §8.2 Scope-of-work grades (`Declared`)

The grades correspond to the Mycelium unit taxonomy (DN-02/DN-03):

| Scope of work | Description |
|---|---|
| `script` | a single-file program, no phylum boundary |
| `nodule` | a static module with a `// nodule:` header |
| `library` | a phylum exposing a public API |
| `application` | a deployable application phylum |
| `other` | catch-all for custom signing scopes |

The signing requirement for each grade is **dev-configurable** — a developer building a script
can set a lighter signing policy than one shipping a production library. The mechanism by which
this configuration is expressed (manifest key, `mycelium-proj.toml`, phylum header annotation,
or colony-bootstrap configuration) is the open R&D subject of §K.2 below.

### §8.3 Key location design space (`Declared`)

Three candidate locations for the project signing key:

- **`mycelium-proj.toml`** — the project manifest (analogous to `Cargo.toml`'s publish
  configuration; M-368 is the `mycelium-proj.toml` to spore packaging task).
- **Phylum header** — a signing-key reference in the `// phylum:` / `// nodule:` header,
  scoping the key to one phylum or nodule.
- **Colony-bootstrap configuration** — a trust configuration passed at colony germination
  (RFC-0008 §4), establishing the `TrustRoot` and optionally the project key for preparation.

These are the design space; the selection among them is open R&D (§K.2).

---

## §9 Open R&D items (explicitly open — G2/VR-5)

These items have a maintainer-set direction (DN-64 §7) but are not yet design-committed. Each
is commissioned as R&D (`Declared`); findings stay `Declared` until a checked basis exists.
They are named here so no reader infers they are closed.

### §K.2 — Key management detail (OQ-K R&D component, M-836)

**Commissioned direction:** project-scoped, graded by scope-of-work, dev-configurable (§8).
**What is open:** the concrete key-management story — key generation, rotation, phylum-level
vs. project-level granularity, the manifest declaration syntax, colony-bootstrap key passing,
and the tooling surface (`myc-prepare` UX). These are design decisions that require a separate
deliberation once the signing-authority grading is validated. They feed back into
`mycelium-proj.toml` (M-368), the phylum header syntax (RFC-0030), and potentially RFC-0008's
germination contract.

### §L — Replay/expiry mechanism (OQ-L, M-837)

**Commissioned direction:** R&D — investigate monotonic-counter vs. expiry-timestamp
approaches. **Trade-offs to resolve:**

- **Content-addressing protects identity, not currency.** A valid `InjectCert` for a
  known-vulnerable prior version of a definition is still a valid cert. The replay/expiry
  mechanism must close this gap without making the preparation step significantly more expensive.
- **Monotonic counter approach:** the image tracks a per-signer injection counter; a cert
  carries a minimum-sequence number and is rejected if the image's counter has advanced past it.
  Requires counter state on the image — interaction with checkpoint/dormancy (RFC-0008 §4.4
  cysts) to be resolved.
- **Expiry-timestamp approach:** the cert carries an `issued_at` plus a `valid_for` duration;
  the image checks wall-clock currency. Simpler but requires clock synchronization in
  distributed settings (mesh — RFC-0008 §4.3/RT5).
- **Content-addressing as partial mitigation:** because the cert is over the exact
  `ContentHash`, injecting a new version of a definition (a new hash) invalidates the old cert
  naturally. Replay is a risk only when the same prior-version hash is re-injected — a narrower
  attack surface than a naive expiry analysis suggests.

The `issued_at` field in the `InjectCert` structure (§6.2) is a placeholder for whichever
mechanism is ratified; it is not a commitment to the timestamp approach.

### §M — Inject-mode scoping hierarchy (OQ-M, M-838)

**Commissioned direction:** R&D — investigate reuse of the RFC-0034 `@certification`
global/phylum/nodule scoping mechanism vs. a separate image/colony/runtime level for
inject-mode configuration.

**Options under investigation:**

- **Reuse `@certification` scoping (RFC-0034 §6):** inject mode is declared at the same three
  levels (global, phylum, nodule) using an `@inject_mode` annotation, with the same resolution
  rules. Ergonomic advantage: one scoping mechanism for both axes. Risk: the inject-mode is an
  image-level property set at germination (I3); `@certification` is a compile-time declaration;
  the two have different change semantics.
- **Image/colony/runtime level:** inject mode is a germination-time parameter on the `Image`
  or `colony`, not a compile-time annotation. Consequence: a single spore can be deployed in
  `loose` or `inoculated` mode depending on the colony's configuration, without recompilation.
  This aligns better with the production/development split (a CI colony vs. a dev workstation
  running the same artifact).
- **Hybrid:** compile-time default declared in the manifest; runtime override at germination.

The interaction with RFC-0034 §6 mode resolution must be carefully specified to avoid ambiguity
(I5 — the two axes are independent; their scoping mechanisms must not entangle them).

---

## §10 Corpus groundwork — what exists today (`Exact`)

Several existing corpus elements are load-bearing for this RFC. All citations are `Exact`:

| Element | Location | Load-bearing property |
|---|---|---|
| Spore signature component named | ADR-013 §2 component 4 | Names the slot; design deferred — this RFC fills it |
| `NativeArtifact` with VR-4 attestation | DN-18/M-620 (`crates/mycelium-mlir/src/deploy.rs`) | The concrete artifact type `InjectCert` wraps; `CrossBackendGate` is the `vr4_attestation` type |
| `InjectError::DispatchMiss` | ADR-017 Decision item 5 / `crates/mycelium-mlir/src/inject.rs:185-193` | Precedent for the never-silent refusal pattern; `UnsignedCode(ContentHash)` is a structural extension |
| Dispatch table `ContentHash` to `entry` | ADR-016 / ADR-017 Decision item 1 | The inject point; `content_hash` in `InjectCert` is this same key |
| `Resolution` enum | ADR-006 / ADR-017 Decision item 5 | Extended with `inject_mode` dimension (§7.3) |
| `@certification` scoping | RFC-0034 §6 | Potential reuse for inject-mode scoping (§M) |
| Germination as initialization event | RFC-0008 §4 | The point at which `TrustRoot` is set (I3) |
| R8-Q4 (trust scope) | RFC-0008 §8 | Cross-colony adversarial mesh — explicitly deferred here (§5.2) |
| DN-44 §1.1 honesty corollary | DN-44 §1.1 | Framework for disclosed-and-guided residual insecurity (§5.2) |

---

## §11 Key differentiators (`Declared` unless noted)

- **Content-addressing dissolves versioning and staleness for free** (`Exact` reasoning,
  `Declared` application to inject-security): an edited definition is a new hash; the
  `InjectCert` for the old hash is automatically invalid for the new definition without any
  revocation machinery. The content hash is the version and the revocation. Structurally
  distinct from Java signed JARs (which add a version number plus an explicit revocation list
  on top of identity).
- **The signature is over the dispatch key itself** (`Declared`): `content_hash` in the
  `InjectCert` is exactly the ABI dispatch key (ADR-016/017). No secondary identity can drift.
- **The security gate also asserts lowering auditability** (`Exact` for VR-4; `Declared` for
  fusion design): the VR-4 no-opaque-lowering attestation (DN-18/M-630) is carried inside the
  `InjectCert`. The cert is not just "from an authorized party" but "from an authorized party
  whose lowering is auditable (no black-box pass)" — fusing security and transparency per
  ADR-006.
- **The gate is independent of the swap-cert mode** (`Declared` — by this RFC's design; grounded
  in RFC-0034 §8, which establishes the dispatch mechanism is `Exact`-ly independent of cert-mode):
  a `fast` runtime can enforce `inoculated` injection; a `certified` runtime can operate in `loose`
  mode. The two axes are independently configurable.

---

## §12 FR/NFR/VR/SC mapping

This RFC advances or references the following requirements from
`docs/Mycelium_Project_Foundation.md` (`Declared` unless the relationship is `Exact`):

| Requirement | Relationship |
|---|---|
| **G2** (never-silent) | I1, I2, I3, §7.2 — the inject-mode gate is G2-disciplined throughout; `Exact` for G2 itself |
| **VR-4** (translation validation / no-opaque-lowering) | `vr4_attestation` inside `InjectCert` extends VR-4 to the injection gate; `Exact` for VR-4 scope, `Declared` for extension |
| **VR-5** (honest guarantee strength) | all claims tagged accordingly; open R&D items are not silently closed |
| **SC-4** (no opaque lowering) | the `InjectCert` carries the VR-4 attestation, extending the opaque-lowering guarantee to the deployment gate |
| **NFR-3** (formal auditability) | `inject_mode` on `Resolution` makes the security posture inspectable through EXPLAIN (I7) |
| **NFR-7** (execution-path equivalence) | the interpreter fallback is equally gated in `inoculated` mode (I6), so the equivalence invariant is not circumvented |
| **KC-3** (small auditable kernel) | the `InjectCert` structure and the signing gate are additive over ADR-017; the dispatch mechanism is unchanged |
| **DN-44 §1.1** (honesty corollary) | §5.2 names all residual gaps; none is silent |

---

## §13 Definition of Done

**Design DoD (this RFC at Accepted):** the maintainer ratifies this document; the knob matrix
extension (§4.1), the two first-class inject modes (§4.2), the `InjectCert` structure (§6.2),
`TrustRoot` semantics (§7.1), cross-colony trust (§7.2), `inject_mode` on `Resolution` (§7.3),
and the three named open R&D items (§K.2/§L/§M) are stated normatively and grounded; the
invariants I1-I7 are stated as design targets.

**Implementation DoD (Enacted — with code):** the inject-mode mechanism lands Rust-first:

- `InjectCert` struct in `crates/mycelium-mlir/src/inject.rs` (or a new `inject_cert.rs`)
  with the fields in §6.2.
- `InjectError::UnsignedCode(ContentHash)` variant added.
- `inject_mode: InjectMode` dimension added to `Resolution`.
- `TrustRoot` on `Image`, set at germination, immutable thereafter.
- `myc-prepare` emits a signed spore (key management: tracked per §K.2).
- Conformance suite parameterized over `InjectMode` (`loose`/`inoculated`), verifying:
  (a) `loose` permits unsigned injection with G2 tagging;
  (b) `inoculated` rejects unsigned code with `UnsignedCode(ContentHash)`;
  (c) `inoculated` rejects unsigned interpreted definitions (I6);
  (d) `TrustRoot` immutability — runtime change yields explicit error;
  (e) `inject_mode` appears on every `Resolution` and is EXPLAIN-able;
  (f) cert axis and inject axis compose correctly in all four combinations.

Until Implementation DoD is met, all mechanism claims stay `Declared` (VR-5/G2).

---

## §14 Residual / open (non-R&D)

- The cross-colony mesh hardening (RFC-0008 R8-Q4) is not addressed here; §5.2 names it.
- The interaction between inject-mode and the `@certification` scoping mechanism (§M) remains
  to be specified in a follow-on once §M R&D resolves.
- The `myc-prepare` UX and key-generation tooling surface are deferred to the §K.2 R&D
  outcome.
- ADR-013's full deployable-artifact schema, germination contract, and the `myc-prepare` to
  spore wire-format extension are impl-pending (ADR-013 §4 / DN-18 §4); this RFC names the
  structure but defers the wire-format to that track.

---

## Changelog

| Date | Status | Note |
|---|---|---|
| 2026-06-29 | **Proposed** | Initial draft — ratifies the hot-inject security axis from DN-64 §7 OQ-K…OQ-Q (2026-06-29 maintainer dispositions). Introduces the `loose`/`inoculated` inject modes as an axis orthogonal to RFC-0034's cert axis; defines `InjectCert` as the spore's signature component (ADR-013 §2 comp. 4); specifies `TrustRoot` semantics and cross-colony trust (OQ-Q); extends `Resolution` with `inject_mode`; names three open R&D items (§K.2 key-management detail, §L replay/expiry, §M scoping hierarchy) explicitly — none silently closed (G2/VR-5). Feeds ADR-013, ADR-017, RFC-0008 R8-Q4 (out of scope), RFC-0034 §4/§8 (independent). Commissioned under M-836…M-842. |
