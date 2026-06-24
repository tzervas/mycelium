# Design Note DN-28 — Registry Architecture & Reconstruction-Based Distribution

| Field | Value |
|---|---|
| **Note** | DN-28 |
| **Status** | **Draft** (2026-06-23; planning capture, DN-17 posture) |
| **Feeds** | ADR-003 (content-addressed identity — the load-bearing rule); ADR-013 (`spore` is the deployable unit); ADR-018 (versioning policy); M-732 (the registry publish/resolve landed in E16-1); DN-06 (phylum/nodule/spore model); DN-27 (post-1.0.0 public decomposition) |
| **Date** | June 23, 2026 |
| **Decides** | *Nothing normatively* — advisory. Records the maintainer's intended **registry architecture** (2026-06-23): a cheap, lightweight, content-addressed distribution model where the registry stores a phylum's **content-hash DAG** (the dense, verifiable map), and the **actual source bytes live in a git forge / object store** that the consumer fetches and verifies. Captured now so the M-732 v0 and its successors are shaped toward this end-state, not painted into a corner. The binding decision is a future RFC. |
| **Task** | E16-1 follow-up (registry evolution); gated research before the security features |

> **Posture (honesty rule / VR-5).** Advisory, forward-looking. Enacts nothing. The registry that
> **exists today** is the M-732 **local, file-based** store (`publish`/`resolve` over `objects/` +
> `index/`). This note records the *direction* — a GitHub-artifacts-backed v0, then a cheaper
> optimized host — so present decisions don't foreclose it. Where this note describes security /
> secrecy / obfuscation features, those are **future research**, explicitly **not** in any MVP and
> **not** implemented; nothing here may be cited as a shipped guarantee (G2).

---

## 1. Goal

A registry that is **cheap, lightweight, easy, and secure** for the project's actual near-term
reality: at first **just the maintainer** publishing phyla, and — if lucky — a few early adopters.
The design optimizes for **minimal registry storage and network traffic** by trading a little
**consumer-side compute**: the registry need not hold the full source bytes of every phylum, only the
**content-addressed map** that lets a consumer *reconstruct and verify* a phylum from source the
publisher already hosts (a git forge — GitHub / GitLab / Gitea — or an object store).

## 2. User stories / motivating use cases

- As the **maintainer**, I want to publish a phylum without the registry having to store (and serve)
  the full source, so hosting stays cheap — ideally **GitHub Releases / artifacts** as the v0 backing
  store, with a cheaper optimized host later.
- As a **downstream app developer**, I want to resolve a phylum by `name@version`, fetch its
  **content-hash map**, retrieve the referenced source/artifact objects, and have the toolchain
  **verify every object against its hash** before building — so what I built is provably what was
  published (ADR-003), with the network cost of the *map*, not the *monorepo*.
- As a **publisher of proprietary code**, I want a future option to keep **sensitive portions secret**
  (encrypted / obfuscated) while the phylum still **builds and runs** for a consumer, so I can
  distribute without exposing proprietary values — or, alternatively, accept a policy that **public
  registry hosting requires source**.
- As a **model / AI co-author**, I want the registry entry to be a compact, navigable **DAG of
  hashes** rather than a blob, so I can reason about a phylum's composition without ingesting it.

## 3. The core idea — and an honest correction (VR-5 / G2)

**The intent:** the published hash(es) act as a *dense mapping* of an entire phylum/nodule/script,
enabling **lightweight reconstruction** — the consumer does more compute, the network does less.

**The honest mechanism (a hash is one-way; it cannot self-reconstruct content).** What makes this
work is not the hash *alone* but a **content-addressed Merkle DAG** plus an **out-of-band content
store** — exactly the model git, Nix, and IPFS use:

1. A spore already **is** a content-addressed DAG (ADR-003): source files by BLAKE3, the resolved
   dependency edges (each pinned by hash), and the germination surface — the `spore_id` is the hash
   over that DAG. This DAG is the **dense map**: a few kB of hashes that names every byte of the
   phylum without containing them.
2. The **bytes themselves** live in a **content store** the consumer can reach: the publisher's git
   forge (retrievable by commit/tree/blob hash) or an object store keyed by content hash.
3. **Reconstruction = fetch + verify:** the consumer walks the DAG, fetches each referenced object
   from the content store, and **verifies its bytes hash to the address in the map** (the never-silent
   check M-732 already enforces on `resolve`). A mismatch is refused (G2) — you can only reconstruct
   *the* phylum that was published, never a substitute.

So the registry stores the **map** (and metadata + the `name@version → spore_id` index); the
**content** is fetched from elsewhere and integrity-checked locally. That is the cheap/lightweight
win, stated without the impossible "rebuild source from a bare hash" premise.

### 3.1 A Mycelium-native compact encoding (Parquet-inspired)

Beyond "a hash points at content," the maintainer's principle is a **calculated computational
representation** — an *encoding* that is small to transfer and store, where the **consumer's machine
does the reconstruction work**. The registry ships the lightweight encoded value back and forth; the
heavy lifting (decode → rebuild the phylum) happens locally. This is the right cost shape for a
**self-hosted, near-zero-budget registry**: storage and bandwidth scale with the *encoding*, not the
source.

A promising direction is to take inspiration from **Apache Parquet** — columnar layout, dictionary
encoding, run-length / delta encoding, per-column compression, a self-describing footer — and
**improve on it in pure Mycelium** for this domain. A phylum's content DAG + source is highly
**structured and repetitive** (shared identifiers, repeated type/representation tags, the
guarantee-strength lattice, dependency edges), which is exactly where columnar + dictionary encoding
wins. Dog-fooding it in Mycelium also exercises the language on a real systems-encoding workload and
keeps the toolchain self-hosted (DN-26/E18-1 spirit).

**Honesty guardrails for any such encoding (VR-5 / G2):**
- It is a **lossless, content-addressed** encoding: decode must reproduce **byte-identical** source,
  verified against the same content hashes (§3) — never a lossy/approximate "reconstruction."
- The decode cost moved onto the consumer must be **bounded and `EXPLAIN`-able** (no black box; house
  rule #2) — a pathological encoding is a refusal, not an unbounded local blow-up.
- Compression/encoding **never** weakens the integrity guarantee: the hash is over the canonical
  *content*, independent of how it was packed for transit.

### 3.2 One canonical identity, three deliveries (source · dense manifest · binary)

A publisher may deliver a phylum as **source**, as the **dense manifest** (§3.1), and/or as a
**binary** — and wants the registry footprint to be just the lightweight manifest + hashes, with
hashes as the verification method. The natural wish is a **single hash that matches all three**. Here
is the honest analysis (VR-5 / G2):

- **Source ⇄ dense manifest share one identity, by reconstruction.** Both are **lossless
  representations of the same content**. Canonicalize to the source DAG and hash it — that hash is the
  **identity** (this is exactly what `spore_id` already is, ADR-003). Decode the manifest → byte-
  identical source → recompute the DAG hash → it **equals the identity**. So source *or* manifest each
  verify to the **same** `spore_id` by reconstruction. This is the "1:1 reconstruction ⇒ the hash
  checks out" property, and it is **achievable today**.
- **A binary cannot share that identity by reconstruction — and that is the feature, not a bug.**
  Compilation is **one-way**: you cannot recover source from a binary, so a binary cannot recompute
  the source-DAG hash. Crucially, this is the *same* fact that gives **proprietary privacy**: if a
  binary *could* reproduce the source identity, the source would be recoverable and the IP would be
  exposed. The two goals — "one hash verifies the binary by reconstruction" and "source is not
  recoverable from the binary" — are **mutually exclusive**; for the closed-source case, privacy wins.
  So a binary having its **own** content hash is not a KISS compromise — it is the **correct** model.

**The model that satisfies all of it:**
- **One canonical identity = `spore_id`** (the source-DAG hash; already built). **Source** and the
  **dense manifest** each verify to it by lossless reconstruction.
- **Binary = a derived artifact** with its **own** content hash *plus* a small binding record
  `{ built_from: spore_id, … }`. The binding is verifiable **cheaply by a reproducible build** (rebuild
  from source → compare the binary hash); absent a reproducible build it is an honest **`Declared`**
  "trust the publisher" — **never** claimed as `Proven` (VR-5). The binary deliberately does **not**
  reconstruct source (privacy preserved).

So a publisher picks what to ship: **source and/or manifest** (open, reconstructable, verified to
`spore_id`) and/or **binary** (closed, own hash + attestation). The registry stores the **manifest +
the hashes + the binding records** — a tiny, reliable footprint; the bytes (source in a forge,
binaries wherever the publisher hosts them) are fetched and integrity-checked on use. The encoding is
**not encryption** — it is a lossless efficiency format (§3.1); the *secrecy* of a closed binary comes
from **not shipping the source**, not from obfuscating the manifest. The **KISS fallback** (hash each
delivered artifact independently) is a strict subset of this and stays available; the only thing
`spore_id` adds for free is the source⇄manifest *unification*, so we get the elegant property at no
extra cost.

## 4. Relation to what landed (M-732) — the seam is already cut

The v0 registry (M-732) deliberately separates the two addresses this architecture needs:

- **`spore_id`** — the DAG **identity** (what the map *is*).
- **`artifact`** — the **integrity** hash verified on publish *and* resolve.

The evolution is to let a registry "object" be a **reconstruction recipe** (content-store locations +
the content-hash DAG) rather than necessarily the **full bytes** it is today. The local file-store v0
is the proof-of-concept; a **GitHub-artifacts/Releases backend** is the next backing store; a cheaper
bespoke host is the long-term option. The `resolve` integrity check is the invariant that survives
every backend change.

## 5. Scope & decision space

**In scope (when this note is promoted to an RFC):**
- The registry record schema: `name@version → { spore_id, content-hash DAG, content-store refs, metadata }`.
- The **v0 backend = GitHub** (Releases/artifacts as the object store; the index as a repo or a small
  hosted manifest), and the abstraction that lets a cheaper host replace it without changing `resolve`.
- The **forge-fetch + verify** reconstruction path in the toolchain (`myc`/`spore resolve`).
- **Licensing-driven inspectability:** an open-source license ⇒ inspectable source; a closed license ⇒
  the trust reduces to the publisher and whether they ship a **binary** or **buildable source**.
- **Security-advisory hosting (DN-30).** The registry hosts not only packages (phyla + nodules) but also
  the **security findings** the scanning toolkit discovers — as **screened/anonymized/privatized** entries
  (vulnerable logic minimized to a content-addressed pattern fingerprint; what / severity / affected
  content-addressed versions / logic-retaining mitigations / honest confidence). *Find-once,
  report-to-community* keyed by the content-hash DAG. The screening policy (what is safe to publish) is an
  open question — DN-30 §4/§7.

**Out of scope for the MVP / proof-of-concept (future research — §6):**
- **Secrecy / obfuscation** of sensitive portions while remaining buildable (encryption, sealed
  sub-DAGs, capability-gated reconstruction).
- A possible **"source required to publish on the public registry"** capability policy.
- Signing / provenance / supply-chain attestation beyond the content-hash integrity already present.

## 6. Open questions

1. **Index hosting.** Is the `name@version → spore_id` index a git repo, a GitHub Release asset, or a
   tiny hosted service? What is its consistency / immutability story (M-732 already enforces
   `name@version` immutability locally)?
2. **Content-store abstraction.** What is the minimal interface a "content store" must satisfy (get by
   hash, exists, licensing/visibility) so GitHub today and a bespoke host tomorrow are interchangeable?
3. **Forge coupling.** How are git-forge object addresses (commit/tree/blob) reconciled with the
   spore's BLAKE3 content addresses — a translation layer, or a re-hash-on-publish into a
   forge-independent object store?
4. **The compact encoding (§3.1).** Is a Parquet-inspired, Mycelium-native columnar encoding worth
   building over a generic compressor? What is the decode-cost bound, and does dog-fooding it in
   Mycelium pay for itself vs. a stock format? It must stay **lossless + content-verified** (§3.1).
5. **Trust without source.** For closed-license/binary publishers, what is the *honest* trust contract
   surfaced to a consumer (what is verified vs what is merely asserted — VR-5)?
6. **Secrecy mechanism.** If sensitive portions are sealed, how does a build consume them without
   exposing them — and how is that capability *never-silent* about what it cannot inspect (G2)?
7. **Policy.** Should the public registry **require source** (inspectable, buildable) as a hosting
   capability, deferring binary-only distribution to private/trusted channels?

## 7. Definition of Done (this note's gate)

This DN is **done as a planning capture** when it has: (a) recorded the reconstruction-based,
content-addressed architecture with the honest map-vs-content distinction (§3); (b) tied it to the
landed M-732 seam (§4); (c) named the v0 GitHub backend and the cheaper-host evolution; and (d)
enumerated the security/secrecy features as explicit **future research, not MVP** (§5–6). It is
**superseded** (append-only) by the binding **RFC** that ratifies the registry record schema, the
content-store abstraction, and the forge-fetch+verify path — at which point the M-732 v0 evolves
toward that schema. No code or guarantee is claimed by this note (VR-5 / G2).
