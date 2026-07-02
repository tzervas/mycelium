# Lane E — Spore Provenance: How a Content-Addressed Deployable Carries Its Full History

| Field | Value |
|---|---|
| **Lane** | E |
| **Topic** | Spore provenance: ratification + build + verification history on a content-addressed deployable |
| **Date** | 2026-06-24 |
| **Confidence** | Per-claim tags throughout (VR-5) |
| **Status** | Research handoff — not normative; informs future RFCs |

---

## 1. Central Question

A Mycelium `spore` is a content-addressed deployable DAG (ADR-013): code + values + the reconstruction
manifest + artifact metadata. The open design question is:

> **How can a content-addressed spore CARRY or REFERENCE its full ratification, build, and verification
> history? And how is a security advisory bound precisely to affected content-addressed versions (the
> DN-28/RFC-0035 model) vs. how npm/PyPI/crates.io/GHSA currently do it?**

More concretely:

- What provenance is **mechanically verifiable** on `spore resolve` vs. merely **asserted**?
- What does "full history" mean for a hash-DAG artifact — and can that history itself be
  content-addressed?
- How precisely can advisory applicability statements target content-addressed identities vs. version
  ranges?

---

## 2. Mycelium Corpus Grounding

### 2.1 `spore` identity (ADR-013, RFC-0001 §4.6) — `Empirical/Declared`

A spore is a content-addressed DAG with four components (ADR-013 §2):

1. **Code** — content-addressed definitions (ADR-003; hash follows Unison §4.6, T4.3)
2. **Values** — initial/captured state as ordinary Mycelium values with `Meta` intact
3. **Reconstruction manifest** (RFC-0003 §6) — one digest-referenced component, the narrow degenerate case
4. **Artifact metadata** — provenance, guarantee/bound certificates, signatures (explicitly listed,
   not yet schematized — "deliberately deferred" per ADR-013 §4)

The identity hash (`spore_id`) is defined in RFC-0001 §4.6:

```
hash(def) = H( normalize(structure(def)) ‖ types_with_repr(def) ‖ static_contract(def) )
```

Dynamic metadata (provenance DAG, measured sparsity, bounds, `policy_used`) is **not hashed** — it
travels with the value but is not identity-bearing. This is the correct model for stability
(renaming ≠ identity change) but it means the `spore_id` alone does not commit to build history.
The `Provenance` type is structured as an acyclic derivation DAG:
`Provenance ::= Root | Derived{ op: ContentHash, inputs: [ProvenanceRef] }`, giving each value a
traceable derivation graph. RFC-0001 §9 notes the provenance DAG "could support full W3C-PROV-style
export (Area 4) for external audit" — this is an explicitly open future possibility.

**Gap (Declared):** There is no current schema for what "artifact metadata — provenance, guarantee/bound
certificates, signatures" means inside a spore. ADR-013 flags this explicitly: "the deployable-artifact
schema, signing story, and germination contract are *new obligations* on the RFC-0008 implementation
stages — deliberately deferred."

### 2.2 Registry model (DN-28) — `Declared` (advisory/forward-looking)

The registry stores a lightweight **content-hash DAG map** (`spore_id` → DAG of hashes), not full
source bytes. The consumer fetches source from a git forge/object store and hash-verifies locally.

Provenance and signing are explicitly **out of scope for the MVP** (DN-28 §5):
> "Signing / provenance / supply-chain attestation beyond the content-hash integrity already present"
> is listed as future research, not MVP.

The M-732 v0 registry deliberately separates two concerns:
- `spore_id` — the DAG identity (what the map *is*)
- `artifact` — the integrity hash verified on publish and resolve

This is the correct seam for future provenance attachment: a provenance record would be a
separate artifact in the registry, keyed by `spore_id` and integrity-checked on resolve.

### 2.3 Security advisory binding (RFC-0035, DN-28 §5) — `Declared`

RFC-0035 §4 and DN-28 §5 share a finding model: advisories are hosted as a **second
content-addressed catalog** reusing the same reconstruction-on-render model as packages. A finding
record includes:

- **Affected**: precisely which content-addressed phylum/nodule versions (the DN-28 content-hash
  DAG + a VEX applicability statement)
- **Screened pattern**: anonymized/minimized to a content-addressed fingerprint so detection
  propagates ecosystem-wide without exposing source
- **Provenance + confidence**: honest tag (proven taint-flow vs. heuristic, RFC-0001 lattice)

The key Mycelium innovation vs. npm/PyPI/crates.io/GHSA: advisory applicability is stated in
terms of **content-addressed `spore_id` values**, not version-string ranges. "Which exact versions
are affected" is **precise**, not guessed (RFC-0035 §3). This is stronger than OSV's version ranges
or GHSA's semver bounds — it is hash-equality, not range-matching.

**Gap (Declared):** The concrete schema for the advisory catalog (the publish/screen/approve workflow,
the exact VEX content-addressing model) is implementation work under E22-1, not yet built.

### 2.4 Honesty tags on build claims (VR-5)

RFC-0001's four-point lattice (`Exact ⊐ Proven ⊐ Empirical ⊐ Declared`) applies to provenance
claims too. DN-28 §3.2 states that for a binary with a `{ built_from: spore_id, … }` binding
record, verifiability depends on whether reproducible builds hold:

- Rebuild from source → compare binary hash → `Proven` (mechanically verified)
- Publisher assertion without a reproducible build → `Declared` ("trust the publisher") — flagged,
  never claimed as `Proven`

This is the right model: the strength of a provenance claim tracks the strength of the verification.

---

## 3. External Prior Art

### 3.1 in-toto Attestation Framework (`Empirical`)

**Source:** [in-toto/attestation on GitHub](https://github.com/in-toto/attestation),
[SLSA on in-toto blog](https://slsa.dev/blog/2023/05/in-toto-and-slsa)

in-toto defines a layered attestation model: **Predicate** (type-specific metadata) →
**Statement** (binds attestation to subjects) → **Envelope** (DSSE: authentication + serialization)
→ **Bundle** (multiple attestations). The Statement binds to artifacts via a `DigestSet`:
`{ "sha256": "ff332109…" }` — the artifact is identified by its content hash.

A SLSA Provenance predicate (predicate type `https://slsa.dev/provenance/v1`) carries:
- `buildDefinition` — repository, git ref, resolved dependencies
- `runDetails` — the builder identity

Crucially, **each attestation references the artifact by digest**, so the provenance is bound to
the specific binary/source that was attested, not to a version string. This is the content-hash
binding model Mycelium needs.

**What in-toto does well:** multiple predicate types can accumulate on the same subject — so a
single `spore_id` could carry a provenance predicate (build), a test-results predicate, a
ratification predicate, an SBOM predicate, etc. This is the "full history via attestation bundle"
pattern.

**What in-toto does not do:** it does not specify the hash algorithm (accommodates any; SHA-256
and SHA-512 common), and the framework does not require chaining — each attestation is
independently signed and logged, not necessarily forming a Merkle-linked chain.

**Adoption (2024–2025, `Empirical`):** GitHub Actions now auto-generates SLSA L3 provenance
attestations for released artifacts (June 2024). Red Hat's Konflux build platform issues in-toto
attestations per pipeline step, with policy enforcement via Conforma (Rego) for full trust-chain
verification.

### 3.2 SLSA (Supply-chain Levels for Software Artifacts) (`Empirical`)

**Source:** [SLSA spec v1.1](https://slsa.dev/spec/v1.1/faq),
[SLSA framework overview](https://www.wiz.io/academy/application-security/slsa-framework),
[Deep dive on SLSA provenance](https://www.legitsecurity.com/blog/slsa-provenance-blog-series-part-2-deeper-dive-into-slsa-provenance)

SLSA defines four build track levels (L0–L3) representing increasing provenance verifiability:
- **L0** — no provenance
- **L1** — basic provenance exists (self-certified)
- **L2** — hosted builds + digital signatures (prevents tampering after build)
- **L3** — hardened, isolated build platform; authenticated, signed provenance (prevents
  tampering *during* build; the builder's identity is verified by a neutral CA)

**Relevance to Mycelium:** SLSA L2/L3 is roughly the model Mycelium needs for spore provenance —
a signed attestation issued by a trusted build platform, binding the `spore_id` to the build
environment, inputs, and policy. The key insight from Red Hat's Konflux deployment:
*neutral-observer signing* (credentials outside the build process) is what makes L3 attestations
mechanically meaningful rather than merely asserted. A build process that signs its own
attestations can lie; a neutral CA that signs based on verified OIDC identity cannot (within the
threat model of the CA).

**What SLSA does not do:** it does not mandate content-addressed artifact identity — it uses
whatever digest the ecosystem provides (SHA-256 for containers, git commit hashes for source). A
content-addressed language like Mycelium with a stable `spore_id` is a stronger substrate than
version-string or git-ref based identity.

### 3.3 Sigstore / Cosign / Rekor (`Empirical`)

**Sources:** [Sigstore docs overview](https://docs.sigstore.dev/cosign/signing/overview/),
[OpenSSF Sigstore blog](https://openssf.org/blog/2024/02/16/scaling-up-supply-chain-security-implementing-sigstore-for-seamless-container-image-signing/),
[GitHub: sigstore/cosign](https://github.com/sigstore/cosign)

Sigstore implements **keyless signing**: ephemeral key pair → Fulcio CA issues a short-lived
certificate binding the public key to an OIDC identity → the (signature, public key, artifact
hash) triple is submitted to Rekor (an append-only transparency log) → private key destroyed.

**Rekor** is the critical component: it is an **immutable, append-only, publicly auditable log**
of signing events. Every entry timestamps and records the artifact hash, the signer's identity,
and the signature. This is what makes provenance claims publicly auditable rather than
privately asserted — a verifier can check that the signer's certificate was valid at signing time
and that the entry appears in the log before a key was compromised.

**Relevance to Mycelium:** The Rekor model is the canonical answer to "what makes build provenance
mechanically verifiable vs. asserted":
- **Asserted:** the publisher claims they built the artifact from source X. No external check.
- **Mechanically verifiable:** the build platform submitted the attestation to Rekor; any
  third party can independently verify the Rekor entry matches the artifact hash and the claimed
  signer identity.

The Mycelium equivalent would be a transparency log entry per spore publish event, binding
`spore_id` to build-system identity + timestamp. This is currently out of scope for the v0
registry (DN-28 §5) but is the natural next step above content-hash integrity.

**Current limitations (as of August 2024, `Empirical`):** Sigstore/Cosign stores attestations
alongside container images in OCI registries. For non-container artifacts (executables, JAR files,
Python packages), attestation storage is still an unsolved challenge. Mycelium's registry (DN-28)
would need its own attestation storage design.

### 3.4 SBOM (SPDX / CycloneDX) (`Empirical`)

**Sources:** [CycloneDX vs SPDX comparison](https://sbomify.com/2026/01/15/sbom-formats-cyclonedx-vs-spdx/),
[CycloneDX guide](https://fossa.com/learn/cyclonedx/)

A Software Bill of Materials enumerates a deployable's components: names, versions, licenses,
hashes, and dependency relationships. Both SPDX and CycloneDX are US government-mandated formats
(2021 executive order) and ISO standards (SPDX is ISO/IEC).

**Relevant to Mycelium:**
- CycloneDX 1.7 (2025) added **citations** — provenance of BOM data itself (which build system,
  which tool, which repo). This is the "provenance of the provenance" property.
- CycloneDX supports VEX integration for advisory applicability alongside the SBOM.
- Both formats support `purl` (Package URL) for component identity — but `purl` is semver/version
  based, not content-addressed. A Mycelium `spore_id` would map to a `purl` with a hash
  qualifier, which SPDX's `packageVerificationCode` or CycloneDX's `hash` fields support.

**Gap:** Neither format natively uses content-addressed identity as the primary key — they treat
version strings as identity and hashes as integrity checks. Mycelium's model inverts this:
the hash *is* the identity. An SBOM for a spore would need the `spore_id` as the primary key
and the version string as discoverable metadata.

### 3.5 OSV Schema + VEX (`Empirical`)

**Sources:** [OSV schema spec](https://ossf.github.io/osv-schema/),
[OSV GitHub](https://github.com/ossf/osv-schema),
[OpenSSF OSV explainer](https://openssf.org/blog/2023/05/02/getting-to-know-the-open-source-vulnerability-osv-format/)

OSV provides a cross-ecosystem advisory format using version ranges (SEMVER/ECOSYSTEM/GIT range
types) to specify affected versions. Key fields: `affected[*].versions` (explicit list),
`affected[*].ranges` (introduced/fixed events), `affected[*].package` (ecosystem + name + purl).

**Critical finding for Mycelium:** OSV does **not** support content-addressed version identity
natively. It targets version-string ranges. The `GIT` range type uses commit hashes, enabling
commit-level precision — but this is git-SHA identity, not content-addressed package identity.
The distinction matters: two packages at the same git commit but built differently (different
build flags, dependency versions) would have the same git-SHA OSV entry but potentially different
vulnerability exposure. Content-addressed `spore_id` would catch this; git-SHA would not.

VEX (Vulnerability Exploitability eXchange) is a companion to SBOMs and OSV: it states per-product
whether a known vulnerability is actually exploitable in that product's context. VEX formats
include OpenVEX, CycloneDX VEX, CSAF, and SPDX VEX — none use content-addressed primary keys
natively.

**Mycelium's contribution (Declared, per RFC-0035):** Using `spore_id` as the VEX applicability
key — "does this vulnerability affect the spore with `spore_id = X`?" — is strictly more precise
than version-range matching. A content-addressed VEX statement is tamper-evident (any change to
the advisory changes its fingerprint, caught by reconstruction-on-render). This is the
"find-once, report-to-community" model in RFC-0035 §4.

**Crates.io / RustSec model (`Empirical`):** RustSec advisories use OSV-format JSON, keyed by
crate name + semver range. The `cargo-deny` tool (already used in Mycelium CI) matches the
project's `Cargo.lock` (pinned versions) against advisory ranges. This is range-matching over
semver, not hash equality. A content-hash check (does this `spore_id` appear in the affected set?)
would be cheaper to compute and more precise.

### 3.6 Reproducible Builds (`Empirical`)

**Sources:** [Reproducible Builds project](https://reproducible-builds.org/reports/2024-04/),
[NixOS reproducible builds](https://reproducible.nixos.org/),
[Debian mandate news](https://needhelp.icu/blogs/debian-reproducible-builds-mandate),
[Arch Linux verifier paper](https://arxiv.org/pdf/2505.21642)

Reproducible builds are the mechanism by which a `built_from: spore_id` binding record becomes
`Proven` rather than `Declared` (DN-28 §3.2): any independent verifier can rebuild from the
declared source and compare the binary hash. If they match, the build claim is verified.

Current status (`Empirical`):
- **NixOS:** high-90s% reproducibility in the nixpkgs ecosystem; content-addressed store paths
  (Nix RFC 62) allow output-hash verification without re-executing the entire derivation.
- **Debian:** Debian 14 "Forky" mandates reproducible builds for the testing branch — first major
  distro to enforce this universally. Downstream: Ubuntu, Kali, Mint, Raspberry Pi OS, Tails.
- **Arch Linux:** independent verifier `rebuilderd` enables third-party reproducibility checks.

**Nix's content-addressed derivations (RFC 62):** In Nix ≥ 2.4, store paths can be
*content-addressed* (hash of the output) rather than *input-addressed* (hash of the derivation).
This means the store path is determined by what was actually produced, not just by the build
specification — the same property Mycelium's `spore_id` provides. Content-addressed Nix paths
enable independent verification without re-running the full build.

**Key insight for Mycelium:** reproducible builds + content-addressed identity is the combination
that makes provenance mechanically verifiable. A `spore_id` is always content-addressed (it is
the hash of the DAG); the question is whether the build process that produced the binary claimed
by `built_from: spore_id` is reproducible. A reproducible build makes the binary hash deterministic
given the source, so any verifier can check the claim without trusting the publisher.

### 3.7 W3C PROV (`Empirical`)

**Source:** [W3C PROV specs (ResearchGate)](https://www.researchgate.net/publication/266369089_The_W3C_PROV_family_of_specifications_for_modelling_provenance_metadata)

W3C PROV defines a generic provenance model: **Entities** (data objects), **Activities** (processes
that create/transform them), and **Agents** (actors responsible for activities). PROV-O is the OWL
ontology form; PROV-JSON/PROV-N are serializations.

RFC-0001 §9 notes the Mycelium provenance DAG "could support full W3C-PROV-style export (Area 4)
for external audit." The current `Provenance` type (`Root | Derived{ op: ContentHash,
inputs: [ProvenanceRef] }`) is a structural subset of PROV's Entity-Activity model, without the
Agent layer or timestamping. Extending to PROV-JSON export would give Mycelium values
interoperability with external audit tooling.

### 3.8 OCI Referrers API (v1.1) (`Empirical`)

**Sources:** [OCI 1.1 release post](https://opencontainers.org/posts/blog/2024-03-13-image-and-distribution-1-1/),
[Docker attestation storage docs](https://docs.docker.com/build/metadata/attestations/attestation-storage/)

OCI Distribution 1.1 (February 2024) added a `referrers` API: given an image digest, retrieve all
associated artifacts (attestations, SBOMs, signatures) that reference it. This is the
"bundle of provenance records hanging off a content-addressed artifact" pattern — analogous to
what Mycelium would need for spore provenance.

The OCI pattern: an image is identified by `sha256:...` digest; provenance/attestation manifests
are stored as separate OCI artifacts with the image digest as their `subject`. The `referrers` API
makes this queryable. The key property: the referrers API is **keyed by the artifact's content
hash**, not by a version string — this is the content-addressed provenance attachment model.

**Relevance to Mycelium:** The DN-28 registry could adopt an analogous "referrers" query:
given `spore_id`, return all associated provenance/advisory/SBOM artifacts. The DN-28 design
already separates the index from the content store; provenance records could live in the same
index, keyed by `spore_id`.

---

## 4. Sketch: Spore-Carried Provenance, Advisory Binding, Applicability to Mycelium

### 4.1 What "full history" means for a spore

A spore's full history comprises at least four layers, each with different verifiability:

| Layer | What it is | Verifiable how | Strength |
|---|---|---|---|
| **Content identity** | `spore_id` = hash of code + values + manifest DAG | Recompute the hash from the fetched content | `Exact` (it either matches or it does not) |
| **Ratification history** | Which RFCs/ADRs the design satisfies; which corpus decisions the code implements | Manual review; cited in artifact metadata | `Declared` (stated, not mechanically checkable without a proof checker) |
| **Build provenance** | Which source, builder, toolchain, and inputs produced this spore | in-toto/SLSA attestation, signed by neutral CA, logged in transparency log | `Proven` (if neutral-CA + reproducible build) or `Declared` (publisher assertion) |
| **Verification history** | Test results, security scans, RFC-0002 certificates for any swaps | Signed attestation predicates keyed by `spore_id` | `Empirical` (test corpus) or `Proven` (machine-checked cert) |

### 4.2 The mechanically verifiable core

On `spore resolve`, the mechanically verifiable minimum is already defined by DN-28 §3 and M-732:

1. **Fetch the content-hash DAG** from the registry (the spore's map, a few kB)
2. **Fetch each referenced object** from the content store (git forge / object store)
3. **Hash-verify each object** against its address in the DAG
4. **Refuse mismatches** — never silent (G2)

This gives `Exact` verification of content identity. Everything above this is additional
attestation.

### 4.3 Proposed provenance architecture for spores

**Confidently grounded in prior art (`Empirical`):**

```
spore_id  →  content-hash DAG  (already: DN-28 / M-732)
         →  provenance bundle  (proposed, keyed by spore_id)
                ├─ build-attestation  [in-toto Statement, SLSA provenance predicate]
                │      subject: { spore_id: "blake3:…" }
                │      buildDefinition: { repo, git-ref, toolchain-hash, … }
                │      runDetails: { builder: "myc-build/1.0", … }
                │      signature: DSSE envelope, Fulcio cert
                │      rekor-entry: https://rekor.sigstore.dev/api/v1/log/entries/…
                ├─ ratification-cert  [Mycelium-native]
                │      ratifies: [ "RFC-0001", "ADR-013", … ]
                │      corpus-state: <hash of docs/ at ratification time>
                │      issued-by: <maintainer key or process>
                ├─ verification-results  [in-toto, RFC-0002 certs]
                │      tests: { suite-hash, pass-rate, coverage, guarantee: "Empirical" }
                │      swap-certs: [ RFC-0002 SwapCertificate per spore swap, if any ]
                └─ SBOM  [CycloneDX or SPDX, primary key = spore_id]
                         components: phyla + nodules (by spore_id, not by semver)
                         hashes: { "blake3": spore_id, "sha256": artifact_hash }
```

The bundle is stored in the registry as a second content-addressed catalog (reusing DN-28's
reconstruction-on-render model): only the hashes + manifest are stored inline; the heavy
attestation bodies are fetched + hash-verified on use, so a tampered attestation fails
reconstruction.

**What is mechanically verifiable on resolve:**
- Content identity (DAG hash): `Exact`
- Build attestation (if logged in Rekor): `Proven` (subject to Rekor's threat model)
- Reproducible build claim: `Proven` if the verifier rebuilds; `Declared` if not
- Ratification cert: `Declared` (maintainer asserts alignment with corpus — cannot be
  machine-checked without a Mycelium proof checker)
- Swap certs (RFC-0002): `Proven` or `Empirical`, per cert strength
- Test results: `Empirical`

### 4.4 Advisory binding — content-addressed vs. version-range

Current practice (npm, PyPI, crates.io, GHSA, OSV):
- Advisories are keyed by **ecosystem + package name + semver range**
- A consumer must resolve "is my installed version X in the affected range?" — this is a
  string-comparison, potentially ambiguous if version ordering is nontrivial
- No hash check — two packages with the same semver but different bits are indistinguishable
- VEX applicability is per-product, still version-string based

Mycelium's model (RFC-0035 §3–§4, DN-28 §5):
- Advisory applicability is stated as a **set of `spore_id` values** (or a predicate over the
  content-hash DAG structure)
- `spore_id` equality is a hash comparison — O(1), unambiguous, no range-matching logic
- A screened finding record is itself content-addressed (its fingerprint = the hash of the
  anonymized vulnerable pattern) — so detection propagates by hash equality, not string matching
- A consumer running `spore resolve` can check: "does this spore_id appear in the affected set
  of any known advisory?" — a single hash-set lookup

**The precision gap (important, Empirical/Declared):** the hash-equality model is only as strong
as the enumeration of affected `spore_id` values. For a newly discovered vulnerability, the
affected set is built by scanning known-affected versions and recording their `spore_id` values —
this requires the scanner to have access to the content-addressed artifacts, not just the version
string. For prospective detection (a vulnerability class that applies to any spore sharing a
pattern), the fingerprint model (screened/anonymized pattern → content-addressed fingerprint)
enables matching without enumerating every affected spore_id. This is RFC-0035's "find-once,
detect-everywhere" model.

### 4.5 Comparison table: advisory binding precision

| System | Version identifier | Advisory precision | VEX | Tamper-evident |
|---|---|---|---|---|
| npm audit | semver range | range-match | via CycloneDX VEX | No (advisory not hash-locked) |
| PyPI / pip-audit | semver range | range-match | No native | No |
| crates.io / cargo-audit | semver + OSV-format | range-match | No | No |
| GHSA | semver range | range-match | Partial (via CycloneDX VEX) | No |
| OSV (cross-ecosystem) | semver / GIT hash | commit-level for GIT type | Via VEX complement | No |
| **Mycelium spore (RFC-0035)** | `spore_id` (BLAKE3 DAG hash) | **hash-equality** | Content-addressed VEX | **Yes** (tamper → hash mismatch) |

---

## 5. Open Research Questions

These are not answered by this survey; they are the next set of decisions for the corpus.

**OQ-1 (Declared — design gap).** What is the concrete schema for spore artifact metadata
(the "provenance, guarantee/bound certificates, signatures" listed in ADR-013 §2 item 4)?
This is the load-bearing design gap: without a schema, "full history" is aspirational, not
implementable. The natural next step is an RFC that extends DN-28's registry record schema.

**OQ-2 (Empirical — from external prior art).** Which in-toto predicate types are relevant for
Mycelium spore provenance, and should Mycelium define its own predicate types
(ratification-cert, swap-cert predicate) or map to existing ones? The in-toto framework is
extensible (custom predicate type URIs) — this seems straightforward but needs a concrete design.

**OQ-3 (Declared — open in DN-28 §6.3).** How are git-forge object addresses (commit/tree/blob
hashes) reconciled with spore BLAKE3 content-addresses? Options: (a) a translation layer that
re-hashes forge objects to BLAKE3, (b) a two-hash binding (BLAKE3 in spore_id, SHA-1/SHA-256 for
forge retrieval), (c) content-addressed git (git SHA-256 repos, which git now supports). Option
(b) is closest to the M-732 seam already cut; option (c) is a clean long-term direction.

**OQ-4 (Declared — open in DN-28 §6.5).** For closed-license/binary publishers, what is the
*honest trust contract* surfaced to a consumer on `spore resolve`? The `Declared` label for
unverified `built_from` claims is the correct answer; the UX of surfacing that label clearly
(what exactly does the user see? what can they do?) is a design decision.

**OQ-5 (Open research).** Is there value in a Merkle-chained attestation format
(each attestation's hash appears in the next one's statement) vs. independent attestations
keyed by `spore_id`? The OCI referrers model and in-toto bundles use independent attestations
keyed by artifact digest — simpler and well-precedented. Merkle-chaining would add ordering
guarantees but at complexity cost. Current evidence favors the independent-keyed model
(`Empirical`, from OCI 1.1 and in-toto adoption).

**OQ-6 (Open research).** Should the Mycelium registry host a Rekor-style transparency log
(append-only, publicly auditable signing events) for `spore publish` events, or rely on an
external Rekor instance? The tradeoff is self-hosting cost vs. external dependency. Sigstore's
public Rekor instance is free but introduces a non-MIT dependency on external infrastructure.
A self-hosted append-only log could be simpler and cheaper at early scale.

**OQ-7 (Declared — from RFC-0035 §9).** What exactly does the safe-fix refinement certificate
(RFC-0002-style) look like for a security advisory fix? The two worked examples (WE-1, WE-2)
required before RFC-0035 can be Accepted are the concrete research tasks here.

**OQ-8 (Open research).** The OSV `GIT` range type provides commit-level precision without full
content-addressing. Is there value in a hybrid model — emit both a `spore_id` set AND a version
range in the advisory, for compatibility with existing OSV consumers — or should Mycelium define
its own advisory schema that treats `spore_id` as the primary key and version string as metadata?
The hybrid approach maximizes ecosystem compatibility; the clean approach maximizes precision.

---

## 6. Highest-Value Finding

**Content-addressed advisory binding is the single largest precision advantage Mycelium has over
the current ecosystem.** npm/PyPI/crates.io/GHSA/OSV all use version-string ranges as the primary
key for vulnerability applicability — inexact, ordering-sensitive, and unable to distinguish two
builds of the same version. Using `spore_id` (BLAKE3 DAG hash) as the primary key for advisory
applicability statements yields hash-equality matching: O(1), unambiguous, and tamper-evident by
reconstruction. The RFC-0035 screened-fingerprint model (anonymized vulnerable pattern → its own
content-addressed fingerprint) extends this to prospective detection without enumerating every
affected spore. No currently deployed ecosystem does this; it is a genuine design advance,
not merely incremental (`Declared` until E22-1 implements it — but the design is well-grounded in
content-addressing theory, `Empirical`).

---

## Sources

- ADR-013: `docs/adr/ADR-013-Spore-Is-The-Deployable-Unit.md`
- DN-28: `docs/notes/DN-28-Registry-Architecture-and-Reconstruction-Distribution.md`
- RFC-0001 §4.6: `docs/rfcs/RFC-0001-Core-IR-and-Metadata-Schema.md`
- RFC-0035: `docs/rfcs/RFC-0035-Security-Scanning-Toolkit.md`
- [in-toto attestation framework](https://github.com/in-toto/attestation)
- [SLSA on in-toto](https://slsa.dev/blog/2023/05/in-toto-and-slsa)
- [SLSA spec v1.1 FAQ](https://slsa.dev/spec/v1.1/faq)
- [SLSA framework overview (Wiz)](https://www.wiz.io/academy/application-security/slsa-framework)
- [Red Hat software provenance with in-toto (2025)](https://developers.redhat.com/articles/2025/05/15/how-we-use-software-provenance-red-hat)
- [Sigstore/Cosign signing overview](https://docs.sigstore.dev/cosign/signing/overview/)
- [OpenSSF: Sigstore for container signing (2024)](https://openssf.org/blog/2024/02/16/scaling-up-supply-chain-security-implementing-sigstore-for-seamless-container-image-signing/)
- [OSV schema spec](https://ossf.github.io/osv-schema/)
- [OSV GitHub](https://github.com/ossf/osv-schema)
- [OpenSSF OSV format explainer](https://openssf.org/blog/2023/05/02/getting-to-know-the-open-source-vulnerability-osv-format/)
- [CycloneDX vs SPDX (sbomify, 2026)](https://sbomify.com/2026/01/15/sbom-formats-cyclonedx-vs-spdx/)
- [CycloneDX guide (FOSSA)](https://fossa.com/learn/cyclonedx/)
- [OCI 1.1 release](https://opencontainers.org/posts/blog/2024-03-13-image-and-distribution-1-1/)
- [Docker attestation storage docs](https://docs.docker.com/build/metadata/attestations/attestation-storage/)
- [Reproducible Builds April 2024 report](https://reproducible-builds.org/reports/2024-04/)
- [NixOS reproducible builds dashboard](https://reproducible.nixos.org/)
- [Arch Linux independent verifier paper](https://arxiv.org/pdf/2505.21642)
- [Nix content-addressed paths RFC 62](https://github.com/NixOS/rfcs/blob/master/rfcs/0062-content-addressed-paths.md)
- [W3C PROV family of specifications](https://www.researchgate.net/publication/266369089_The_W3C_PROV_family_of_specifications_for_modelling_provenance_metadata)
- [Unison 1.0 content-addressed code](https://byteiota.com/unison-1-0-content-addressed-code-hits-production/)
- [RustSec advisory database](https://rustsec.org/)
- [SCAI: Software Supply Chain Attribute Integrity](https://arxiv.org/pdf/2210.05813)
