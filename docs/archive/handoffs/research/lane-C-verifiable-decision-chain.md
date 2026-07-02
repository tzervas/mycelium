# Lane C — The Append-Only Decision Ledger as a Content-Addressed, Verifiable Structure

| Field | Value |
|---|---|
| **Lane** | C |
| **Status** | Research handoff — Draft (2026-06-24) |
| **Confidence** | Mixed; see per-claim tags (VR-5) |
| **Date** | 2026-06-24 |
| **Feeds** | Future RFC for verifiable decision-ledger structure; ADR-003 (content-addressed identity); RFC-0001 §4.6 (content-addressing); ADR-013 (spore); DN-28 (registry) |

---

## §1 — Central Questions

This lane investigates three tightly coupled questions:

1. **First-class verifiable structure.** Can the design-decision ledger (RFC/ADR/DN documents + the
   `Draft → Accepted → Enacted → Superseded` status machine) be made a first-class,
   content-addressed, mechanically verifiable structure expressible inside Mycelium's own
   value-semantics model — rather than an editorial convention enforced by code review?

2. **Mechanically checkable vs. merely asserted integrity properties.** Which of the corpus's
   current integrity invariants (append-only, no-rewrite-of-Accepted, design↔code↔verification
   linkage) are *mechanically checkable today*, and which remain asserted-by-convention?

3. **Provenance chain verifier.** What would a "provenance chain" verifier over the decision ledger
   actually check? What are its inputs, outputs, and honest confidence tags?

---

## §2 — Mycelium Corpus Grounding

### §2.1 The current ledger: editorial discipline, not a data structure

The Mycelium decision corpus (`docs/rfcs/`, `docs/adr/`, `docs/notes/`) is a **textual,
human-maintained, git-resident ledger**. Its integrity properties are:

- **Append-only status transitions.** CLAUDE.md house rule 3: `Draft/Proposed → Accepted →
  Enacted → Superseded`; notes → `Resolved`. To revise an Accepted/Enacted decision, supersede
  it — never rewrite. This is enforced **editorially** (CONTRIBUTING.md + PR review) and
  **partially** by the `/pr-review` skill checking that no accepted doc's normative body was
  silently altered.

- **Forward-reference discipline.** Superseding ADRs carry explicit `Supersedes:` fields;
  superseded ADRs carry a `Superseded by:` footnote (e.g., ADR-032 → ADR-021; RFC-0001 r5 →
  r4 grammar). These links are textual, not cryptographically bound.

- **Design↔code↔verification linkage.** `tools/github/issues.yaml` tracks `depends_on` edges
  between epics/milestones and RFC/ADR decisions; `doc_refs:` grammar (e.g.,
  `corpus:RFC-0001#4.6`, `src:crates/mycelium-core/src/content.rs:42`) creates traceable links
  validated by `python3 tools/github/doc_refs_check.py`. The validated grammar is a lightweight
  authenticated link — checkable by a tool, but not cryptographically bound to the document
  content.

- **Git squash discipline.** The squash-only `main` policy (CLAUDE.md "Commits & PRs") means
  each landing on `main` is a single atomic commit. Git's SHA-based DAG provides content
  integrity over the committed text — every commit ID is the SHA of its content plus its parent
  links. This is an **implicit Merkle DAG** over the decision ledger, but it is not surfaced as
  a first-class queryable structure.

**Confidence tag: `Empirical`.** The above is an accurate description of the current corpus
discipline, verified by reading CLAUDE.md, ADR/README.md, the RFC/ADR files, and
`tools/github/`. No mechanically-enforced integrity boundary exists between "editorial rule
followed" and "structural invariant violated."

### §2.2 Content-addressing already in the corpus — at the code level

RFC-0001 §4.6 specifies content-addressing for *code definitions* (`hash(def) = H(normalize(structure) ‖ types_with_repr ‖ static_contract)`). The hash excludes human names, spans, comments, and dynamic metadata. This is the Unison model (ADR-003, Area 1), and it is **implemented** in `crates/mycelium-core/src/content.rs`. The provenance DAG over values is already a typed structure:

```
Provenance ::= Root
             | Derived { op: ContentHash, inputs: [ProvenanceRef] }
```

This is an authenticated derivation DAG for *values*. The decision ledger does not yet have an
analogous structure. **Gap:** the value-level provenance DAG and the decision-level provenance
chain are two parallel structures that are not yet connected.

**Confidence tag: `Empirical` (code observed; Declared for the gap claim).**

### §2.3 The spore model and its relevance

ADR-013 defines a `spore` as a content-addressed deployable DAG: `{code (content hashes) +
values + reconstruction manifest + artifact metadata (provenance, guarantee certs, signatures)}`.
The spore's `spore_id` is the hash of this DAG. DN-28 extends this to the registry: the
registry stores the *map* (content-hash DAG) while the bytes live in a content store (git forge
or object store), exactly the git/Nix/IPFS pattern.

The decision ledger could be treated analogously: each RFC/ADR document is a content-addressed
object; the corpus is a Merkle DAG of those objects; the "registry" is the index of
`decision-id → content-hash` (the `Accepted` state of each document at each point in time). The
spore mechanism already provides the primitives; what is missing is the ledger schema and the
append-only structural enforcement.

**Confidence tag: `Declared`.** The analogy is plausible and internally consistent, but no such
schema is designed or implemented. This is a direction, not a claim.

### §2.4 Tunable certification and the mode-tagged ledger

RFC-0034 / ADR-032 separate *transparency* (operations never opaque, never overclaim) from
*certification depth* (how much is machine-checked). The same distinction applies directly to
the decision ledger:

- **Transparency floor (mode-independent):** every decision document is inspectable; its
  provenance (which RFC it supersedes, which ADR it enacts) is always queryable. This is
  already nearly achieved — it is the editorial discipline.

- **Certification layer (opt-in):** the actual hash of each document at the time it became
  `Accepted` is recorded; a consistency proof over the ledger log shows the `Accepted` corpus
  at time T₂ is a superset of the corpus at time T₁ (append-only, nothing removed). This is
  not yet present.

The RFC-0034 invariant "never-silent about the mode" maps directly: a verifiable ledger should
always be able to report *which claims are checked vs. asserted*, and a verifier that can only
check structural links (not semantic soundness) must be explicit about that boundary.

**Confidence tag: `Empirical` (analogy reads cleanly from the corpus; extension is `Declared`).**

### §2.5 The guarantee-strength lattice as a ledger-claim lattice

The `Exact ⊐ Proven ⊐ Empirical ⊐ Declared` lattice (RFC-0001 §3.4) is already used to tag
value-level guarantees. It maps naturally onto decision-ledger claims:

| Claim type | Lattice tag | Example |
|---|---|---|
| Document content unchanged since `Accepted` | `Exact` (hash matches) | SHA of `RFC-0001.md` at PR #N |
| Status transition legal (Draft→Accepted) | `Proven` (automated checker) | `doc_refs_check.py` passes |
| Design←→code linkage intact | `Empirical` (tool-validated doc_refs) | `doc_refs:` grammar valid |
| Editorial discipline followed | `Declared` (review-asserted) | PR review passed |

This framing makes the current ledger's integrity tags explicit: most claims today sit at
`Declared`, with a few at `Empirical` (the doc_refs checker). A mechanically verifiable ledger
would lift status-transition integrity to `Proven` and content-integrity to `Exact`.

---

## §3 — External Prior Art (Cited)

### §3.1 Certificate Transparency and verifiable append-only logs

Certificate Transparency (RFC 6962, [https://www.rfc-editor.org/rfc/rfc6962](https://www.rfc-editor.org/rfc/rfc6962)) is the
canonical reference for append-only verifiable logs in production:

- A log is a Merkle hash tree (binary, SHA-256). Each new entry extends the tree; the log
  periodically publishes a **Signed Tree Head (STH)**: `{timestamp, tree_size, root_hash,
  signature}`.
- **Inclusion proofs** (`O(log n)` hashes): prove entry `e` is in tree of size `n`.
- **Consistency proofs** (`O(log n)` hashes): prove tree at size `m` is a prefix of tree at
  size `n` — the append-only invariant *mechanically*, not editorially.
- Monitors watch for unauthorized entries; auditors verify the log has not forked.

Russ Cox's exposition ([https://research.swtch.com/tlog](https://research.swtch.com/tlog)) clarifies the minimal structure:
inclusion and consistency proofs together eliminate the need to trust the log operator; any
third party can verify both. The Go module transparency log (sum.golang.org) and Sigstore's
Rekor ([https://docs.sigstore.dev/logging/overview/](https://docs.sigstore.dev/logging/overview/)) apply this pattern directly to software
supply chains.

**Directly applicable to Mycelium's decision ledger:** each RFC/ADR at a given status transition
event could be a log entry; consistency proofs would mechanically enforce that the `Accepted`
corpus only ever grows (append-only); a verifier would check STH signatures against a trusted
key.

**Confidence (external): `Proven`.** CT is a ratified IETF standard (RFC 6962; superseded by
RFC 9162 for the v2 protocol) with large-scale deployment. The mechanism is established
technique, not open research.

### §3.2 Git's object model as an implicit Merkle DAG

Git stores every blob, tree, commit, and tag as a content-addressed object (SHA-1 / SHA-256).
The commit DAG is a Merkle DAG: each commit's hash covers its content plus its parent hashes,
so altering any past commit changes all downstream hashes — detectable without a trusted third
party ([https://www.wasilzafar.com/pages/series/software-engineering/software-engineering-part10-git-internals.html](https://www.wasilzafar.com/pages/series/software-engineering/software-engineering-part10-git-internals.html)). The squash-only `main` policy (CLAUDE.md) already exploits this:
each landing on `main` is a single atomic commit whose hash is the integrity anchor for that
state of the corpus.

**The gap vs. CT:** git provides *content integrity* (the objects are immutable once committed)
but not *append-only log proofs*. A reviewer can rewrite history (`git push --force`) and
present the altered repo to a third party — the new hashes are internally consistent but
different. CT's consistency proofs prevent this: a third party holding the old STH can detect
the rewrite. The CLAUDE.md rule prohibiting force-pushes is an editorial mitigation, not a
cryptographic one.

**Confidence (external): `Proven` for the content-integrity property; `Empirical` for the
gap claim (no force-push guard is cryptographic in a bare git repo without external CT anchor).**

### §3.3 Nix / Guix: content-addressed derivation chains

Nix and Guix represent software builds as content-addressed derivation DAGs
([https://lwn.net/Articles/962788/](https://lwn.net/Articles/962788/); [https://guix.gnu.org/en/blog/2024/identifying-software/](https://guix.gnu.org/en/blog/2024/identifying-software/)). Each
derivation is identified by a hash over its inputs + build recipe; outputs are either
input-addressed (hash of the derivation) or content-addressed (hash of the output itself).
Guix emphasizes **provenance tracking**: the verifiable path from source to binary. The model
maps cleanly to Mycelium's spore: the spore's `spore_id` is analogous to Nix's store path,
and the reconstruction recipe (DN-28 §3) is analogous to a derivation.

**For the decision ledger:** a decision DAG could be built on the same model. Each RFC/ADR
document at a given version is a content-addressed object (hash of normalized text). The
`Accepted` event is a derivation: `hash(doc_content) + hash(prior_Accepted_event)`. The
resulting chain is a content-addressed log of decisions — exactly the Nix derivation chain
applied to design rationale rather than build artifacts.

**Confidence (external): `Proven` for Nix/Guix as a pattern; `Declared` for the mapping to
Mycelium's decision corpus (no implementation exists).**

### §3.4 in-toto / SLSA: supply chain attestation chains

in-toto ([https://github.com/in-toto/attestation](https://github.com/in-toto/attestation)) provides a framework for
**cryptographically signed attestations** at each step of a software supply chain. SLSA
([https://slsa.dev/spec/v1.1/faq](https://slsa.dev/spec/v1.1/faq)) layers a policy framework on top: at SLSA Level 3+,
every pipeline step is attested via in-toto envelopes (DSSE-signed), forming a
*verifiable provenance chain* from design through build through deployment.

The key structure is a **link**: `{name, materials (input content hashes), products (output
content hashes), command, environment, byproducts, signed}`. A verifier walks the link chain
from a policy root to the target artifact, checking that each step is signed by an authorized
key and that content hashes match.

**Mapping to Mycelium's decision ledger:**
- A `Design → RFC → Implementation → Verification` chain is a 4-step in-toto layout.
- `materials` at the RFC step = content hash of the design document.
- `products` at the Implementation step = content hashes of the implementing code commits.
- `materials` at the Verification step = the implementation commit hash + the property-test
  results file.
- A verifier checks: (a) the RFC is signed as `Accepted` by the correct key; (b) the
  implementation's content hash is the `products` the RFC step declared; (c) verification
  artifacts reference those exact implementation hashes.

**Confidence (external): `Proven` for in-toto/SLSA as an established pattern; `Declared` for
the Mycelium-ledger adaptation.**

### §3.5 Sigstore / Rekor: append-only transparency log for signing events

Rekor ([https://docs.sigstore.dev/logging/overview/](https://docs.sigstore.dev/logging/overview/)) is Sigstore's append-only
supply-chain transparency log. It stores signed metadata about artifacts using a Merkle
tree (backed by Google's Trillian). Every signing event produces a log entry; log consistency
is verifiable by any third party. The `rekor-monitor` tool watches for unauthorized entries.

**Directly applicable:** Mycelium's status transitions (`Draft → Accepted`) are signing events.
A Rekor-style log over these transitions would make the append-only invariant
*mechanically verifiable* by external auditors, not just reviewers with repo access. Each entry
would carry: `{doc_id, from_status, to_status, doc_content_hash, timestamp, maintainer_sig}`.

**Confidence (external): `Proven` for Rekor as an established pattern; `Declared` for the
Mycelium adaptation.**

### §3.6 Authenticated data structures: theory

Papamanthou et al. ("Authenticated Data Structures" — [https://www.researchgate.net/publication/220770494_Authenticated_Data_Structures](https://www.researchgate.net/publication/220770494_Authenticated_Data_Structures))
and the subsequent literature on auditable data structures ([https://arxiv.org/pdf/2306.01886](https://arxiv.org/pdf/2306.01886))
establish the formal framework: an ADS is a structure that allows an *untrusted* server to
store data and produce *verifiable* query results, where a client holding only a small digest
can check the server's responses. For a decision ledger, the "query" is "what decisions are
currently Accepted?" and the "proof" is an inclusion proof against a signed digest.

**Confidence (external): `Proven` for the theoretical foundation; `Declared` for any
Mycelium-specific instantiation.**

### §3.7 Merkle-CRDTs: distributed append-only logs

Merkle-CRDTs ([https://research.protocol.ai/blog/2019/a-new-lab-for-resilient-networks-research/PL-TechRep-merkleCRDT-v0.1-Dec30.pdf](https://research.protocol.ai/blog/2019/a-new-lab-for-resilient-networks-research/PL-TechRep-merkleCRDT-v0.1-Dec30.pdf))
extend the Merkle DAG with CRDT merge semantics, enabling distributed append-only logs that
merge without conflicts. This is relevant if Mycelium's decision ledger needs to be
replicated across multiple independent repositories (e.g., post-1.0.0 decomposition per
DN-27). For the single-maintainer design phase, this is premature; it becomes relevant at
post-1.0.0 governance scale.

**Confidence (external): `Proven` for the Merkle-CRDT pattern in IPFS/OrbitDB; `Declared` for
Mycelium applicability.**

---

## §4 — Concrete Proposal Sketch

### §4.1 What is checkable mechanically today vs. what is asserted

| Integrity property | Current state | Mechanism | Honest tag |
|---|---|---|---|
| Document content unchanged after acceptance | **Asserted** — editorial, no hash pinned | Git SHA covers the commit, but no per-doc acceptance hash is extracted | `Declared` |
| Status transition is legal (Draft→Accepted only, etc.) | **Partially checkable** — ADR/README.md defines the set; a linter could check the status field text | No linter exists; checked by PR reviewer | `Declared` (aspiring to `Empirical`) |
| Superseded doc still has normative body intact | **Asserted** — PR review checks for silent edits | No structural check; the `Superseded by:` footnote is a text convention | `Declared` |
| `doc_refs:` grammar valid (corpus↔issue links) | **Checkable** — `python3 tools/github/doc_refs_check.py` | Tool exists, runs in CI | `Empirical` |
| Design↔code linkage (RFC enacted → implementation PR cited) | **Asserted** — convention; the `Enacted` status change should cite the PR, but no verifier checks the PR hash against the doc | No hash binding exists | `Declared` |
| Implementation↔verification linkage (code PR → property-test results) | **Asserted** — CI passes on the branch, but no signed attestation binds the test results to the commit hash | No attestation | `Declared` |

**The single largest gap:** there is no mechanism that *pins the content hash of a document at
the moment it becomes `Accepted`* and makes that pin verifiable later. Without this pin, a
subsequent silent edit to the document body is undetectable to an external verifier who was
not present at the time of acceptance.

### §4.2 The minimal verifiable ledger — a three-layer proposal

Layer 0 (already available): git SHA provides content integrity at the commit level. Extract
it: whenever a doc transitions to `Accepted` or `Enacted`, record:

```
AcceptanceRecord {
    doc_id:      "RFC-0034",
    from_status: "Proposed",
    to_status:   "Accepted",
    doc_sha256:  BLAKE3(canonical_text_of_doc_at_acceptance),
    commit_sha:  "abc123...",  // the git commit that landed the status change
    timestamp:   "2026-06-24T00:00:00Z",
}
```

This record, stored in `tools/github/decision-log.json` (append-only by convention + CI
check), gives a lightweight, independently verifiable acceptance ledger. The SHA of the
document at acceptance is the integrity anchor.

Layer 1 (Merkle-log, CT-style): Feed the `AcceptanceRecord` sequence into a Merkle tree (the
CT/Rekor/tlog model). Publish periodic Signed Tree Heads. A verifier who holds the STH from
time T₁ can check via a consistency proof that the ledger at time T₂ extends T₁ — the
append-only invariant is now *mechanically verifiable*, not just editorially enforced.

Layer 2 (in-toto attestation chain): For each `{Design → RFC → Implementation → Verification}`
link, produce an in-toto layout with signed links:
- **Design link:** `materials = {}, products = {RFC-0034.md: sha256}`, signed by maintainer.
- **Implementation link:** `materials = {RFC-0034.md: sha256}, products = {commit: sha}`, signed by CI.
- **Verification link:** `materials = {commit: sha}, products = {test-results.json: sha}`, signed by CI.

A layout verifier checks the full chain: the RFC is `Accepted` (Layer 0), the implementation
references the exact `Accepted` RFC hash (Layer 1 inclusion proof), and the verification
artifacts reference the exact implementation hash (signed link).

### §4.3 Expressing the ledger inside Mycelium's value-semantics model

The `Provenance` type already in RFC-0001 §4.6 is the right shape:

```
Provenance ::= Root
             | Derived { op: ContentHash, inputs: [ProvenanceRef] }
```

A decision ledger entry is a `Derived` provenance node:
- `op` = the hash of the transition event (`{doc_id, from, to, timestamp}`)
- `inputs` = `[ContentHash(doc_text), ContentHash(prior_ledger_tip)]`

The ledger DAG is then a chain of `Derived` nodes, where each node's hash covers the previous
tip — this is exactly a hash chain / Merkle log expressed in Mycelium's own provenance type.
The `GuaranteeStrength` tag on this structure is `Exact` (hashes match) or `Declared` (link
asserted but not hashed). The editorial requirement becomes a type invariant: an
`AcceptanceRecord` node with `guarantee = Exact` means the doc hash was pinned.

**This is the bridge from "editorial convention" to "expressible in Mycelium's value model."**
The ledger is not a new primitive — it is a `Provenance` DAG over decision events, using the
same content-addressing and guarantee-lattice machinery the corpus already defines for values.

**Confidence tag: `Declared`.** This is a design sketch; no implementation exists. The type
mapping is structurally sound, but soundness of the full verification chain requires the
Layer-0 tooling and a formal definition of "decision event canonicalization" (the hashing
precondition).

### §4.4 What a provenance-chain verifier would check

A verifier tool (`myc verify-decisions`) operating at `certified` mode would:

1. **Load the signed acceptance ledger** (`decision-log.json` + STH signature if Layer 1 is
   present).
2. For each `AcceptanceRecord`:
   a. Recompute `BLAKE3(canonical_text)` of the named doc at the named commit.
   b. Check it matches the stored `doc_sha256`. Tag: `Exact` if match, error if not.
   c. Check the `from_status → to_status` transition is in the legal set
      `{Draft→Proposed, Proposed→Accepted, Accepted→Enacted, Accepted→Superseded,
       Enacted→Superseded, Draft→Resolved}`.
      Tag: `Proven` (the state machine is a finite, checkable set).
   d. Check the `commit_sha` exists in the git DAG and its tree contains the doc file
      matching the `doc_sha256`. Tag: `Exact`.
3. **Append-only check (Layer 1):** verify the Merkle consistency proof that the current
   ledger tip is an extension of the last known STH. Tag: `Proven` (Merkle proof is
   machine-checkable).
4. **Design↔code linkage (Layer 2):** for each `Enacted` record, walk the in-toto attestation
   chain. Report unlinked enactments as `Declared` (the link is asserted, not proven).
5. **Output:** a structured report with per-claim `GuaranteeStrength` tags. Claims that cannot
   be checked are reported as `Declared`, never silently omitted (G2/VR-5).

This verifier is **compositional with the existing `GuaranteeStrength` lattice** — the
verifier's output is itself a Mycelium value with a provenance record, so it can be embedded in
the ledger as evidence.

### §4.5 Scope and applicability

**What this proposal changes:** the acceptance ledger is a new artifact (`decision-log.json`)
maintained alongside `issues.yaml`; Layer-0 tooling is a small Python script. No changes to
the RFC/ADR document format are required (the hash is extracted, not injected).

**What it does not change:** the editorial discipline (house rules 1–6) remains the primary
mechanism; the ledger is a verification layer on top, not a replacement.

**Integration with Mycelium language features:** the `Provenance` DAG type in RFC-0001 §4.6
is the natural host for ledger entries. Expressing them as first-class Mycelium values means
the decision corpus can be queried, filtered, and verified using the language's own facilities
— a self-hosting property analogous to `myc` verifying its own stdlib specs (DN-26).

---

## §5 — Open Research Questions

**OQ-C1: Canonicalization of decision documents.** Content-hashing requires a canonical
form. RFC/ADR documents evolve over time with footnotes and status-field updates (the
append-only amendments are intentional). What is the canonical form for hashing at acceptance?
Options: (a) hash the entire file at the commit that changed the status to `Accepted` —
simple, but includes formatting noise; (b) extract and hash only the normative sections
(everything after `## Context`) — requires a structured parser. Choice affects what "document
unchanged" means. *Status: open.*

**OQ-C2: Key management and trust root.** The CT model requires signed STHs; Sigstore's
Rekor uses a cosigning ceremony. Who holds the signing key for Mycelium's decision ledger?
Currently: the single maintainer. At post-1.0.0 scale (DN-27), governance may require multiple
signatories. The trust model determines the security level of the entire chain. *Status: open.*

**OQ-C3: Retroactive anchoring of existing decisions.** ADR-001 through ADR-009 live in
`Mycelium_Project_Foundation.md §8`, not standalone files. Anchoring them requires deciding
whether to hash the Foundation at the time of original authoring (unavailable as a pinned
record) or to accept that pre-ledger decisions are `Declared` trust anchor. *Status: open;
the honest answer is the latter.*

**OQ-C4: Semantic soundness vs. structural integrity.** The verifier proposed in §4.4 checks
*structural* integrity (hashes match, state machine respected). It does not check *semantic*
soundness (the Accepted RFC is logically consistent, the implemented code actually satisfies
the design). Semantic soundness requires theorem proving and is beyond a ledger verifier.
The honest boundary: the verifier is at `Proven` for structural claims and `Declared` for
semantic claims. *Status: clear as a boundary; implications for what "verified" means in
Mycelium's attestation model are open.*

**OQ-C5: Integration with `spore` and the registry.** DN-28 treats the package registry as
a content-hash DAG + content store. A decision ledger could be a first-class entry in this
registry: `decisions@<version> → {ledger-tip-hash, STH, acceptance-log}`. A consumer of the
Mycelium toolchain could then verify the decision corpus the same way they verify a phylum —
fetch, hash-verify, walk the DAG. *Status: design direction, not implementation; feeds a
future RFC (the decision-registry RFC).*

**OQ-C6: Granularity of the provenance chain.** The in-toto model in §4.2 covers
`Design → RFC → Implementation → Verification`. Should each RFC revision (r1, r2, r3 …)
also be a ledger entry? Each revision is an append to the doc's own history; pinning each
revision's hash would give a full document-level revision history (a per-doc Merkle log).
This is technically straightforward but increases the ledger's surface area. *Status: open.*

**OQ-C7: Relationship to RFC-0035 / DN-30 security scanning.** DN-28 §5 notes that the
security advisory catalog reuses the reconstruction model: findings are content-addressed,
hash-verified entries in a separate catalog. If both the decision ledger and the security
catalog are content-hash DAGs, they share the same structural backbone. A unified
*"Mycelium trust registry"* hosting decisions + security findings + package manifests is a
coherent generalization. *Status: speculative direction; feeds future research.*

---

## §6 — Summary of Findings

The highest-value finding is the **structural alignment** between Mycelium's existing machinery
and what a verifiable decision ledger would require:

1. **The `Provenance` DAG type (RFC-0001 §4.6) is the correct native host** for ledger
   entries. No new primitive is needed; the hash chain over decision events is a specialization
   of the existing derivation DAG.

2. **The `GuaranteeStrength` lattice applies directly** to ledger claims, making the
   verifier's output composable with the rest of the corpus's honesty discipline. Today's
   decision claims sit at `Declared`; a Layer-0 ledger lifts content-integrity to `Exact`
   and state-machine checks to `Proven`.

3. **The biggest mechanical gap is the absence of an acceptance hash.** No record of
   `hash(RFC-0034.md at the moment it became Accepted)` exists today. This is the single
   cheapest fix with the highest integrity payoff — a small Python tool and a
   `decision-log.json` file (append-only by CI check).

4. **The CT/Rekor/in-toto patterns are directly applicable** (established technique, not open
   research) to layers 1 and 2. The question is not "can this be done?" but "at what scope and
   when?"

5. **Semantic soundness (design↔code correctness, not just design↔code linkage) remains open
   research**, requiring theorem-proving infrastructure the corpus explicitly defers (KC-4
   framing). The verifier cannot claim `Proven` for semantic properties without machine-checked
   proofs.

---

## Meta — Changelog

- **2026-06-24 — Draft.** Research handoff produced for Lane C (verifiable decision chain).
  No normative status changed; all proposals are `Declared` or `Empirical` direction only.

---

## Sources (external)

- [RFC 6962 — Certificate Transparency](https://www.rfc-editor.org/rfc/rfc6962) (IETF; Laurie, Langley, Kasper 2013)
- [Transparent Logs for Skeptical Clients — Russ Cox](https://research.swtch.com/tlog)
- [Rekor — Sigstore software supply chain transparency log](https://docs.sigstore.dev/logging/overview/)
- [in-toto Attestation Framework](https://github.com/in-toto/attestation)
- [SLSA FAQ v1.1](https://slsa.dev/spec/v1.1/faq)
- [Nix / Guix content-addressed derivations — LWN](https://lwn.net/Articles/962788/)
- [GNU Guix — Identifying Software (2024)](https://guix.gnu.org/en/blog/2024/identifying-software/)
- [Unison — The Big Idea (content-addressed code)](https://www.unison-lang.org/docs/the-big-idea/)
- [Authenticated Data Structures — Papamanthou et al.](https://www.researchgate.net/publication/220770494_Authenticated_Data_Structures)
- [Auditable Data Structures: Theory and Applications (2023)](https://arxiv.org/pdf/2306.01886)
- [Merkle-CRDTs: Merkle-DAGs meet CRDTs — Protocol Labs](https://research.protocol.ai/blog/2019/a-new-lab-for-resilient-networks-research/PL-TechRep-merkleCRDT-v0.1-Dec30.pdf)
- [Git Internals — The Object Model & DAG](https://www.wasilzafar.com/pages/series/software-engineering/software-engineering-part10-git-internals.html)
- [IPFS — Content Addressed, Versioned, P2P File System](https://arxiv.org/pdf/1407.3561)
