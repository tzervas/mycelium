# ADR-037 — The Spore Registry's v0 Remote Backend Is GHCR/OCI, Distributing the DN-28 Dense-Map

| Field | Value |
|---|---|
| **ADR** | 037 |
| **Status** | **Enacted** (2026-07-01 — implemented + demonstrated; stepped through **Accepted** first, same day). The remote backend landed in `mycelium-spore` (`remote` module, M-871), and its Definition of Done (§3) is met: publish→resolve→fetch-and-verify round-trips green against **both** a local `registry:2` OCI registry (`scripts/dist/spore-oci-selftest.sh`, 57 unit tests) **and** the **live GitHub Packages registry** — the example phyla `hello` + `std` were published to `ghcr.io/tzervas/{hello,std}:0.0.0-dogfood` and resolved back with identical, hash-verified `spore_id`s (`hello=blake3:6ce6d13…`, `std=blake3:52b6982…`, byte-identical local↔GHCR — deterministic content-addressing confirmed across backends). **Update (2026-07-01): M-872 landed** — remote publish now enforces `name@version` immutability best-effort (a list-tags → pull → compare-`spore_id` pre-check refuses a *differing* re-publish as `Conflict`; an identical re-publish is idempotent; a first publish proceeds). Honest ceiling (Declared): OCI tags are server-side mutable, so it is a *client-side* guard, not a proven server invariant — grounded oras "not found" vs auth-error classification keeps it never-silent (an auth failure is never read as "nothing published"). Fixes the **remote/networked backend** for the `spore` registry (M-732's local file store gains a remote sibling): a published spore is distributed as an **OCI artifact** in the **GitHub Packages container registry (GHCR)**, decomposed into the **DN-28 dense-map** — each content-object (source file, by BLAKE3 address) is its own OCI blob (so identical objects **dedup** across versions and phyla by construction), and the dense-map DAG (spore_id + object references + dependency edges + germination surface) is the OCI **config** blob; the `name@version` index is the OCI **tag**. `oras` is the **v0 wire-transport driver** (a pluggable transport; a pure-Rust OCI client is named future work, not faked). This is the binding backend decision **DN-28 anticipated** ("the binding decision is a future RFC"); it does not amend any ADR-022 Definition-of-Done criterion. |
| **Decides** | (1) The spore registry's **remote backend is GHCR** — spores are hosted in the **GitHub Packages** registry as **OCI 1.1 artifacts** (`ghcr.io/<owner>/<phylum>:<version>`), fulfilling the maintainer's release-strategy directive to host phylum/nodule/spore packaging **in the GitHub Packages registry** to prove out + test the registry design (ADR-036 / [[dogfooding-release-strategy]]). (2) The distribution shape is the **DN-28 dense-map**: N content-objects → N OCI blobs (BLAKE3-addressed, deduped by identical bytes), the dense-map manifest → the OCI config blob (`application/vnd.mycelium.densemap.v1`), the artifact type is `application/vnd.mycelium.spore.v1`. (3) **Fetch-and-verify is mandatory + never-silent (G2/VR-5):** on resolve, every fetched object's bytes must BLAKE3 to its declared content address **and** the reconstructed DAG must recompute (via the single canonical `mycelium_spore::content_address`) to the recorded `spore_id`; any mismatch/missing object/extra layer is an explicit error, never a silent partial. (4) **`oras` is the v0 transport driver** — the Rust code owns the *design* (dense-map decomposition, addressing, verification, dedup, the `name@version`→tag index); `oras` owns only the OCI HTTP wire protocol. The transport is behind a trait (`OciTransport`) so a pure-Rust OCI client can replace `oras` append-only without touching the registry logic. **`oras` absent is an explicit, actionable error, never a silent skip.** (5) The **local file store (M-732) is unchanged** and remains the default; a `--registry` value with an explicit `oci://` / `ghcr://` scheme selects the remote backend, a bare path stays the local store — the route is never guessed. |
| **Amends** | **Nothing in ADR-022's Definition of Done.** This is a backend/implementation decision, not a tag-criteria amendment (contrast ADR-024/034/035). It **realizes** the direction DN-28 (Draft, advisory) recorded and **refines** DN-28's sketched "GitHub-artifacts-backed v0": DN-28 floated GitHub *Releases* as the object store; this ADR selects the GitHub Packages *container* registry (GHCR/OCI) instead — still GitHub-artifacts-backed, but the **Packages registry the maintainer's release strategy names**, and a better fit for the dense-map (OCI content-addressed blob dedup is exactly DN-28 §3's "store each object once"). DN-28 stays **Draft/advisory** and is **not rewritten** (append-only, house rule #3) — it gains a forward pointer to this ADR as the binding v0-backend decision it said "is a future RFC." |
| **Grounds** | Maintainer decision (2026-07-01, verbal ratification recorded here per house rule #6); the maintainer's release strategy — host phylum/nodule/spore in the **GitHub Packages registry** to prove out + test the registry, **no crates.io**, repo private until dogfooded (ADR-036, [[dogfooding-release-strategy]]); **ADR-003** (content-addressed identity — the load-bearing rule the OCI blob/dense-map layout preserves end-to-end); **ADR-013** (`spore` is the deployable unit — this ADR gives it a networked home); **DN-28 §1–§3** (the dense-map/fetch-and-verify distribution model this ADR implements over OCI); **M-732** (the local `publish`/`resolve` this extends — the two content-addresses, both checked, carry over verbatim); the **OCI Image/Distribution Spec 1.1** (artifacts + arbitrary blobs, the standardized wire); KC-3 (small auditable kernel — the transport is a subprocess driver, no HTTP client enters the kernel; the dense-map format is a hand-rolled, injective, strictly-parsed encoding consistent with `content_address`/`parse_entry`, **no new runtime dependency**); G2/VR-5 (never-silent, honestly-tagged — every selection/verification is explicit and `EXPLAIN`-able; a range constraint stays `Unsupported`, not mis-resolved; the whole remote path is `Empirical`/`Declared`, never claimed `Proven`). |
| **Date** | 2026-07-01 |

> **Posture (transparency rule / VR-5).** This ADR records a *backend decision* and the shape of its v0
> implementation. The remote path's guarantees are **`Empirical`** (verified by round-trip tests against a
> local OCI registry + a live GHCR dogfood) and **`Declared`** where they rest on `oras`/GHCR behavior we
> do not prove — never `Proven`. The ADR reached **Enacted** on exactly the basis its Definition of Done
> (§3) names: the `mycelium-spore` remote backend implemented + tested + a **live GHCR** publish/resolve
> round-trip demonstrated (M-871, epic E26-1). Remote `name@version` immutability is now enforced
> best-effort (M-872 landed 2026-07-01) — a *client-side* pre-check, **Declared** not `Proven` (OCI tags
> are server-side mutable). Still **not** claimed: SemVer range resolution (stays `Unsupported`), and any
> `Proven`-tier guarantee on the `oras`/GHCR wire.

---

## 1. Why this decision exists

The maintainer's release strategy (ADR-036, [[dogfooding-release-strategy]]) requires that Mycelium's own
packaging units — **phylum** (library/package), **nodule** (static unit), **spore** (deployable artifact,
ADR-013) — be **hosted in the GitHub Packages registry**, both to make 1.0.0 installable *without*
crates.io and to **dogfood the registry design itself** before the repo flips public. The registry that
exists today (M-732) is a **local file store** only — DN-28 recorded the *intended* remote direction but,
being advisory Draft, decided no backend and shipped no wire. This ADR makes that binding, minimal,
never-silent decision so the dogfood can proceed.

Three facts constrain the choice:

- **GitHub Packages has no generic-file package type.** Its ecosystems are npm/Maven/NuGet/RubyGems/…
  and the **container registry (GHCR)**. The only fit for an arbitrary content-addressed `spore` is
  **GHCR as an OCI-artifact registry** (OCI 1.1 lets a manifest carry an arbitrary `artifactType` and
  arbitrary blob layers). So "host in GitHub Packages" ⇒ GHCR/OCI. (This is the one place the maintainer's
  "GitHub Packages" directive and DN-28's sketched GitHub *Releases* object store diverge — resolved here
  in favor of the directive; §4.)
- **DN-28's dense-map maps onto OCI cleanly.** DN-28 §3 wants the registry to store each content-object
  **once** (dedup) and have the consumer **fetch-and-verify** a DAG of hashes. OCI blobs are exactly
  content-addressed, dedup-by-digest storage; an OCI manifest is exactly a small DAG-of-descriptors. The
  dense-map → config, objects → layers mapping is natural, not a forcing.
- **The kernel must stay small (KC-3) and dependency-light.** A full OCI-distribution HTTP client
  (bearer-token auth dance, chunked blob upload) in Rust is a large surface + a heavy new dependency. For
  v0 we drive the wire with **`oras`** (the purpose-built OCI-artifact CLI) as a subprocess behind a
  trait, keeping the *registry design* — the part DN-28 is about — in auditable Rust, and leaving the
  standardized HTTP mechanics to a standardized tool. The pure-Rust client is named future work.

## 2. The mapping (spore ⇄ OCI artifact)

```text
spore  (name@version, ADR-013 content-addressed DAG)
  │
  ├─ each source object  (bytes, BLAKE3 == SourceFile.hash)  ─────►  one OCI blob (layer)
  │      mediaType application/vnd.mycelium.spore.object.v1            (dedup by digest across versions)
  │      title = <blake3-hex>.myco   (so resolve maps blob → content-hash)
  │
  ├─ the dense-map  { format, spore_id, kind, name, version,   ─────►  the OCI config blob
  │      surface[], objects[{rel_path, content_hash}],                 mediaType application/vnd.mycelium.densemap.v1
  │      deps[{name, phylum, hash, version}] }                         (the DN-28 "dense, verifiable map")
  │
  └─ name@version  ───────────────────────────────────────────►  the OCI tag
         ghcr.io/<owner>/<name>:<version>            artifactType application/vnd.mycelium.spore.v1
```

**Two content addresses, both checked — carried over from M-732 verbatim:** `spore_id` (identity, the DAG
hash) and each object's BLAKE3 (integrity). On **resolve**: pull the manifest by tag → read the dense-map
config → for **every** object, fetch its blob and assert `BLAKE3(bytes) == content_hash` → reconstruct the
`SourceFile` set → recompute the identity via **the single canonical `mycelium_spore::content_address`**
(never a re-implementation — DRY, the v0/v1-split lesson) and assert it equals the recorded `spore_id`. A
missing object, a byte-mismatch, an extra/undescribed layer, or a recomputed-`spore_id` mismatch is an
explicit error (G2). The dense-map format is a hand-rolled, length-prefixed (netstring-style), injective
encoding with a **strict, never-silent parser** — the same discipline as `content_address` and
`registry::parse_entry` (unrecognized/duplicate/missing fields are named errors, not silent defaults).

## 3. Definition of Done (this ADR → Enacted)

1. `mycelium-spore` gains a remote backend (`remote` module): dense-map encode/decode (injective +
   strict-parse, property-tested over adversarial inputs), `build_dense_map`, `verify_and_reconstruct`, an
   `OciTransport` trait with an `oras` driver **and** an in-memory test double, and `publish_remote` /
   `resolve_remote` — all never-silent (G2), guarantees tagged `Empirical`/`Declared`.
2. `spore publish --registry <oci://…|ghcr://…>` and `spore resolve <name> <ver> --registry <…>` route to
   the remote backend by explicit scheme; a bare path keeps the M-732 local store. `oras` absent ⇒ an
   explicit, actionable error (never a silent skip).
3. Round-trip verified against a **local OCI registry** (dependency-free CI-shaped test) **and** a **live
   GHCR** publish→resolve of the example phyla (`examples/hello-phylum`, `lib/std`) — the dogfood.
4. Docs: the Spore-Build-and-Publish-Contract gains a remote-backend section; DN-28 gets its forward
   pointer; `CHANGELOG` + a `just` recipe + a dogfood script land.

## 4. Alternatives weighed (honest, VR-5)

- **GitHub Releases object store (DN-28's literal sketch).** Truer to DN-28's words, but Releases are **not
  the Packages registry** the maintainer's strategy names, and give no content-addressed dedup — rejected
  in favor of the directive + the cleaner dense-map fit. (DN-28 stays advisory; not contradicted, refined.)
- **Whole-spore single OCI artifact (one blob).** Simpler, but forgoes DN-28's per-object dedup and the
  "dense map of hashes" property the maintainer chose — rejected: the point of this dogfood is to exercise
  the *dense-map* design, not a tarball.
- **Pure-Rust OCI client now.** Most self-contained, but a large surface + heavy HTTP/auth dependency into
  a kernel-adjacent crate for v0 — deferred as named future work behind the `OciTransport` trait, so the
  swap is append-only.
- **crates.io / a bespoke server.** Excluded by strategy (no crates.io; private until dogfooded).

## 5. Consequences

- The spore registry becomes **networked and installable** without crates.io, hosted in GitHub Packages —
  the release-artifact model of ADR-036 gains its concrete registry.
- The registry **design is dogfooded**: publishing the real example phyla to GHCR exercises the dense-map,
  dedup, and fetch-and-verify paths against a production registry before the repo goes public.
- `oras` becomes a **v0 runtime prerequisite** of the remote path (documented; never-silent if absent).
  This is an explicit, bounded coupling, retired when the pure-Rust client lands.
- **No kernel change, no new runtime dependency** in `mycelium-spore` (the dense-map is hand-rolled; the
  transport is a subprocess) — KC-3 preserved.
- Guarantees on the remote path are **`Empirical`/`Declared`** — round-trip-tested, not proven; a SemVer
  range stays `Unsupported` (ADR-018 deferred), never mis-resolved.

Relates to [[dogfooding-release-strategy]]. Feeds DN-28 (its binding v0-backend decision), ADR-013, M-732.
