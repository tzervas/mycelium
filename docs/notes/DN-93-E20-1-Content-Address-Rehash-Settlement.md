# DN-93 — The E20-1 Content-Address Rehash Settlement (batch every identity-changing one-way door into a single rehash)

| Field | Value |
|---|---|
| **Note** | DN-93 |
| **Status** | **Draft** (2026-07-08; works up a maintainer **strategic-timing** decision — recommends a coordination discipline, decides nothing normatively. Advisory, `Declared` throughout — house rule #3, never jumps to Accepted, never spends the rehash). |
| **Task** | proposed tracking id **not minted** (mitigation #1 — the orchestrator/maintainer verifies a free `E*`/`M-xxx` slot before minting). Authored on the `trx2` design lane (non-blocking, non-conflicting worktree; maintainer directive 2026-07-08). |
| **Related** | **RFC-0033 §7** (the content-address identity set + the single-rehash DOGFOOD GATE — the normative basis this note operationalizes) · **§4.1.4** (the one-way-door pattern) · **§4.2.2/§4.2.3** (arbitrary-width / growable ternary) · **§4.3.2/§4.3.4** (Dense dtype + `QuantDesc` in identity) · **§4.4.2/§4.4.3** (VSA element-space + sparsity in identity) · **ADR-030** (Dense-quant granularity descriptor — a **content-address one-way door**, Accepted) · **ADR-031** (VSA element-space / block-sparse / complex carrier — a **content-address one-way door**, Accepted; note the M-775…M-780 → **E20-1** milestone-ID erratum) · **ADR-029** (ternary arithmetic is arbitrary-width; `BigTernary` the reference, V0 landed) · **ADR-040 §3** (scalar-float identity commitments ride the same rehash — *"documented, not spent"* — the exemplar of the discipline this note generalizes; now Enacted) · **ADR-038 §2.6** (the float route (ii) + "rehash happens once, deferred to the first value-persistence feature") · **DN-90 §5/§6** (the growable-tier design gate + the decidable-vs-held split — this note's direct predecessor) · **`docs/planning/deferred-design-CU-gates.md` D3/D6** (the two gates `blocked` on "the E20-1 content-address settlement" — what this note makes decidable) |
| **Grounding** | `docs/rfcs/RFC-0033-*.md:183-189` (§7 identity set + the DOGFOOD GATE) · `:95-158` (§4.1–§4.4 the four paradigms + their one-way doors) · `docs/adr/ADR-030-*.md` §Status/§Decision (Dense `QuantDesc` in identity) · `docs/adr/ADR-031-*.md` §Status/§Milestone-ID-erratum (VSA elem/sparsity in identity; E20-1 owns the rehash) · `docs/adr/ADR-040-*.md:159-187` (§3 documented-not-spent) · `crates/mycelium-core/src/content.rs:23,33-54,124-176` (the `Canon` model: BLAKE3, domain-separated, **length-prefixed, frozen** prefix tags `REPR_BINARY=0x10 … REPR_FLOAT=0x16`, append-only — additive arms spend no rehash) · `crates/mycelium-core/src/repr.rs:104-120` (`enum Repr`, the `u32` size fields) · `tools/github/issues.yaml` E20-1 epic (`in-progress`; V1–V6 rehash + reconciliation deferred post-1.0) |
| **Guarantee** | **`Declared`** throughout — a proposed coordination discipline / design direction. Nothing here is implemented, enacted, or checked; every normative claim cites a ratified ADR/RFC/DN or is marked an open question (house rule #4). The *current-state* survey (§2/§3) is **`Empirical`** (grounded in cited source at `file:line` — the citations are ground truth, not this note). The prior-art in §5 is **`Declared`** (external systems, cited, not verified against their code here). |

> **The decision this note works up (captured, 2026-07-08).** Several deferred features each want to
> change the value **content-address / identity set**, and the project already decided they must ride a
> **single E20-1 rehash** (M-780) rather than many — that rehash is deferred to *"the first
> value-persistence feature"* (RFC-0033 §7; ADR-038 §2.6; ADR-040 §3). DN-90 §5/§6 flagged this as the
> hard gate under the growable tier and under the D3/D6 CU gates, but did **not** settle *how* the one
> rehash absorbs all the doors, *when* it fires, or *what must be decided before it vs. by it*. This note
> settles that coordination discipline **ratifiably**: it enumerates the doors that must be batched (§2),
> shows the `Canon` mechanism that makes "no rehash spent yet" true (§3), recommends the freeze-and-land
> discipline + trigger (§4), presents the alternatives with prior art and a rationale (§5), and shows the
> settlement makes **D3 (Dense dtype/quant)** and **D6 (growable ternary)** decidable (§6). It **decides
> nothing normatively and spends no rehash** — the strategic timing is the maintainer's (house rule #3).

---

## 1. Problem statement — the meta-blocker

Mycelium's value **content address** is a closure hash over the identity-bearing `Repr` + payload
(`Node::content_hash`, RFC-0001 §4.6). RFC-0033 §7 fixes the identity set — `Seq.len`, `Bytes.len`,
`Dense.dtype`, `Dense.quant`, `Vsa.elem`, `Vsa.sparsity` — and rules that **because changing any of
these rehashes stored values, the changes MUST be settled and land in a single rehash before any value
is persisted for dogfooding** (the DOGFOOD GATE, RFC-0033 §7 `:183-189`). Multiple accepted-but-deferred
features each want exactly such a change:

- the **ADR-030 Dense-quant** door (dtype expansion + `QuantDesc` into identity),
- the **ADR-031 VSA** door (element-space + sparsity into identity),
- the **ADR-040 scalar-float** identity commitments (a new `Repr::Float` variant + NaN/zero identity rules),
- a **growable `Repr::Ternary`** value form (ADR-029; the **D6** CU gate),
- **growable / beyond-`u32` Dense & VSA `dim`** and **quantized-Dense growth** (the **D3** CU gate; DN-90 §5/§6).

The project has *decided the policy* (one rehash, deferred to first persistence) but has **not settled
the coordination discipline** that makes it operational: what the batch contains, the trigger predicate,
the freeze-point, the ordering, and the before-vs-settled split. That absence is the **meta-blocker**:
the D3 and D6 gates are `blocked` in `deferred-design-CU-gates.md` with `depends_on: the E20-1
content-address settlement` (`:52,87`), and DN-90's whole growable tier is held on it (DN-90 §5). Until
the discipline is ratified, every downstream door stays parked on an *unscheduled event of unknown
shape*. This note supplies the missing discipline so the event has a shape and the downstream doors
become decidable — **without** spending the rehash (which stays the maintainer's timing call, §4/§8).

## 2. The pending-identity-change manifest — the doors that must be batched (verify-first, cited)

Every door below changes the content-address identity of some value form, so each **rehashes stored
values if applied after those values are persisted** (RFC-0033 §7). That single fact — not a shared
schedule — is why they batch: a value persisted under scheme *v0* would need re-addressing **once per
door** if the doors landed separately, i.e. *N* flag-days instead of one (the closure-coupling argument,
§5, exactly Git's). The manifest is the batch; it is **append-only** (a door joins only by an explicit
accepted decision) and **verify-first** (each row cites the deciding record + its identity delta).

| # | Door (deciding record) | Identity change it introduces | Why it must be in the single batch |
|---|---|---|---|
| **D-Dense** | **ADR-030** (Accepted) — Dense granularity-descriptor quant | `Dense.dtype` expands (float-only → `I8/U8/I16/U16/I4/U4/F8E4M3/F8E5M2/TF32`, later MX FP6/FP4) **and** `Dense.quant: QuantDesc{scheme, symmetric, scale_dtype}` enters the `Repr` (RFC-0033 §4.3.2/§4.3.4). Per-tensor and per-block values must not alias. | Reshapes `REPR_DENSE` (0x12) identity of every quantized Dense value. A separate rehash later = a second flag-day over the same `Dense` address space. |
| **D-VSA** | **ADR-031** (Accepted) — VSA element-space / block-sparse / complex | `Vsa.elem ∈ {Binary,Bipolar,Integer,Real,Complex}` and `Vsa.sparsity ∈ {Dense, SparseGlobal, SparseBlock}` enter `Repr::Vsa`; a `HypervectorC` payload arm is added (RFC-0033 §4.4.2–§4.4.4). | Reshapes `REPR_VSA` (0x13) identity of every VSA value. Same address space as D-Dense's siblings; batching keeps it one flag-day. |
| **D-Float** | **ADR-040 §3** (Enacted) — scalar-float `Repr::Float` | A new `Repr::Float{width}` variant joins identity under **frozen tag `REPR_FLOAT=0x16`**; NaN canonicalized at construction, `±0.0` bit-distinct, an append-only width registry (`FloatWidth::tag()`). | **Additive** — it spends *no* rehash by itself (§3). But its identity *commitments* (NaN/zero rules, the width registry, **FLAG-5** uniform-NaN across scalar-float and the existing f64 `Payload` paths) MUST be **final before** the rehash so the batch settles them once. The canonical "documented, not spent, rides the same rehash" exemplar. |
| **D-BigTernary** | **ADR-029** (Accepted; V0 landed) — arbitrary-width ternary; the **D6** gate | Surfacing `BigTernary` as a **growable `Repr::Ternary`** (no fixed `N`) changes the ternary payload shape that enters the address; today `Ternary{trits:u32}` is fixed (RFC-0033 §4.2.2/§4.2.3). | Reshapes `REPR_TERNARY` (0x11) identity of ternary values that adopt the growable form. The digit-serial arithmetic + `BigTernary` are **already built** (§3); only the *surfaced value form* is the door. |
| **D-Dim** | **DN-90 §5/§6**; the **D3** gate — growable / beyond-`u32` Dense & VSA `dim`, quantized-Dense growth | A wide/growable `dim` (beyond the `u32` field, `repr.rs:14-25`) and the quantized-Dense element-space/block-sparse payload widening change the `dim`/payload encoding that enters the address. | Reshapes `REPR_DENSE`/`REPR_VSA` identity; couples to D-Dense's `QuantDesc`. DN-90 §5 already flags it (FLAG-cu7-e20-1-gate). |
| **D-Signed** *(conditional)* | **RFC-0033 §4.1.4** — a `signed` flag in `Repr::Binary` | Only **if ever taken**: a `signed` flag in the `Repr` changes the identity of every integer value and doubles the swap matrix. Default is **not** to add it (signedness is operations, not the `Repr` — ADR-028). | RFC-0033 §7 names §4.1.4 explicitly as a door that, *if taken*, joins the same single rehash. Recorded here as a conditional manifest slot, not a proposal to take it. |
| **D-Payload-NaN** | **ADR-040 FLAG-5** — existing f64 `Payload` NaN identity | Whether the existing `Hypervector`/`Scalars` f64 `Payload` paths canonicalize NaN identically to scalar-float; a payload-level identity commitment. | If it changes existing payload identity, it rehashes those values — so it settles **uniformly** in the same batch (ADR-040 §2.3 FLAG-5, `:100-103`). |

**Verify-first honesty (§2 is `Empirical`).** D-BigTernary's arithmetic is *already* arbitrary-width and
`BigTernary` is *already* landed (ADR-029 V0, M-754…M-757) — the door is **surfacing** it, not building
it (DN-90 §2.2). D-Float is *already Enacted as an additive variant* (ADR-040) and spent no rehash. The
manifest tracks the *identity commitments still open*, never re-proposes landed work (the DN-90 /
mitigation-14 verify-first discipline).

## 3. The `Canon` mechanism — additive (no rehash) vs. reshaping (needs the rehash)

The settlement rests on one source fact about how the address is computed
(`crates/mycelium-core/src/content.rs:23,33-54,124-176`, `Empirical`, source-read):

- The kernel hash is **BLAKE3** over a **domain-separated, length-prefixed** encoding (`Canon`), with
  **one frozen prefix byte per syntactic form** (`REPR_BINARY=0x10, REPR_TERNARY=0x11, REPR_DENSE=0x12,
  REPR_VSA=0x13, REPR_SEQ=0x14, REPR_BYTES=0x15, REPR_FLOAT=0x16` — `content.rs:42-54`). The comment is
  explicit: *"existing codes are frozen so a definition's identity never shifts when the registry grows"*
  (`:47`).

This splits every manifest door into two mechanically-distinct kinds — the distinction that lets the
rehash cost **zero** if timed right:

1. **Additive doors** — a *new* `Repr` variant under a *new* frozen tag (D-Float is the exemplar).
   Because the encoder is prefix-tagged and prior tags are frozen, *adding* an arm changes **no existing
   value's address** — **no rehash is spent** (ADR-040 §3 `:171-173`, pinned by M-896's address-stability
   regression). These can land at any time; they only need their identity commitments *final* before the
   batch so nothing about them is re-decided later.
2. **Reshaping doors** — a change to an *existing* identity-bearing field's encoding for a value form
   that may already be persisted (D-Dense, D-VSA, D-BigTernary, D-Dim). These *do* reshape existing
   addresses, so they are the doors the single rehash actually exists for (ADR-040 §3 `:177-179`).

**The decisive consequence.** Because Mycelium is **pre-persistence** — no feature persists
content-addressed values yet (ADR-038 §2.6; the trigger has not fired) — **the reshaping doors, if they
all land before the first value is persisted, reshape nothing** (there is nothing persisted to reshape).
The "single rehash" is then a **no-op migration over zero persisted values**: you never actually pay it.
The rehash cost is only ever incurred if a reshaping door is deferred *past* first persistence — which is
exactly the second flag-day the batch exists to avoid. This reframes the settlement from *"schedule one
expensive migration"* to *"settle every door's shape now, land them all before first persistence, and the
migration is free."*

## 4. The recommended coordination discipline (the settlement — `Declared`, for ratification)

A **freeze-and-land discipline** so ONE rehash (or, timed right, a no-op) absorbs every reshaping door,
with the additive doors' commitments final beside them. Four parts.

**(a) The pending-identity-change manifest (a single, append-only registry).** Maintain the §2 table as
the *authoritative* list of every accepted-but-unspent identity commitment: its deciding ADR/DN, its
identity delta, and its additive-vs-reshaping class. A door **joins only by an explicit accepted
decision** (append-only, house rule #3); it **leaves only by landing** in the batch. This is the
freeze list. *Recommended home:* a short planning doc (e.g. `docs/planning/content-address-rehash-manifest.md`)
or an RFC-0033 §7 appendix — **maintainer/orchestrator's to place** (this leaf owns only DN-93; §8 FLAG).

**(b) The trigger predicate (refining "the first value-persistence feature").** The prior records say
the rehash "defers to the first value-persistence feature" (RFC-0033 §7; ADR-038 §2.6; ADR-040 §3) but
never define it. Recommended crisp predicate: **the trigger fires at the first feature that writes a
content-addressed value to storage that must survive a scheme change** — concretely, the first of {a
value store / cache keyed by content address across sessions; a spore artifact (ADR-013) carrying
values; on-disk value serialization for dogfooding}. Pure in-memory computation and swap machinery are
**not** triggers (they never persist an address). Ratifying this predicate is what converts D3/D6 from
"blocked on an undefined event" to "gated on a *named, recognizable* milestone" (§6).

**(c) The freeze-point + ordering.** The sequence the discipline enforces:
1. **Decide each door's shape** (its ADR/DN) — done for D-Dense (ADR-030), D-VSA (ADR-031), D-Float
   (ADR-040); **still-open before the batch:** the D-BigTernary growable-form shape (D6 — needs a DN),
   the D-Dim growable-dim shape (D3 / DN-90), the quantized-Dense payload byte layout (RFC-0033 §5 defers
   it to a follow-up), FLAG-5's uniform-NaN resolution, and D-Signed *only if taken*.
2. **Freeze the manifest** when the first value-persistence feature (per (b)) is scheduled: after the
   freeze, no new reshaping door may join without either making the freeze or accepting its **own** future
   rehash (a second flag-day — to be avoided).
3. **Land all reshaping doors together, before first persistence**, so the migration runs over zero
   persisted values (§3) — the no-op path.
4. **Run the rehash mechanics once** (a no-op if zero persisted; a single closure re-address with a
   temporary old→new translation if any values already persisted — the Git precedent, §5).
5. **End-state: a single scheme, manifest closed.** Any later identity change is, by definition, a new
   superseding decision plus its own coordinated rehash (RFC-0033 §4.1.4's "explicit decision, never
   drift").

**(d) Decided-before vs. settled-by the rehash (the split).** *Decided before* (each door's own ADR/DN):
field shapes, canonicalization rules, encoding order, the identity-bearing-vs-not split. The rehash
**does not get to re-decide any door's shape** — those are ADR-level and mostly already ratified.
*Settled by the rehash itself* (the M-780/E20-1 mechanics): recomputing persisted addresses under the
final scheme, the temporary translation during migration, and the single-scheme end-state. If the manifest
is frozen with every shape decided and nothing yet persisted, step (d) is empty — the whole cost was
paid in *decisions*, not in a migration.

## 5. Alternatives considered + rationale (prior art folded in, `Declared`)

Three coordination strategies span the design space; each maps onto a named prior-art system. **This note
recommends the batched single rehash (Alternative A); it does not decide the timing** (§8).

### Alternative A — one batched rehash / freeze-and-land *(recommended — §4)*

**Precedent: Git's SHA-1 → SHA-256 hash-function transition.** Git makes the *repository*, not the
individual object, the unit of migration — a coordinated, one-repo-at-a-time flag-day with a temporary
bidirectional translation table, staged to an end-state that removes the old scheme entirely. The
strategic reason it is one-shot rather than per-object: **an object's name is the hash of its
content-plus-transitive-dependencies, so you cannot rehash one object without its whole closure** — the
coherent unit is the whole store, migrated once (`Declared`; https://git-scm.com/docs/hash-function-transition).
Mycelium's content address is the same closure hash (`Node::content_hash`), so the same argument holds
door-for-door: the coherent unit is *all reshaping doors at once*.

- **Buys:** a clean single-scheme end-state (no permanent indirection, no multi-version decode); and —
  because Mycelium is pre-persistence (§3) — a rehash that can be a **no-op** if all doors land before
  first persistence. Matches the already-ratified policy (RFC-0033 §7).
- **Costs:** requires the *discipline* (a frozen manifest, a recognized trigger) or a door slips past the
  freeze and forces a second flag-day; concentrates several decisions into one freeze window.

### Alternative B — per-feature rehash (each door migrates when it lands)

**Precedent: Nix RFC 62 content-addressed derivations** — input-addressed and content-addressed store
paths **coexist**, gated per-derivation behind an experimental flag, migrating incrementally with a
permanent indirection layer (`Declared`; https://github.com/NixOS/rfcs/blob/master/rfcs/0062-content-addressed-paths.md).

- **Buys:** no coordination window; each door lands independently.
- **Costs:** *N* migrations over the same closure-coupled address space = up to *N* flag-days for values
  that outlive several doors; permanent dual-scheme indirection. **Rejected** by the project's existing
  "one rehash" policy (RFC-0033 §7) and by the closure argument — the recommendation ratifies *why*.

### Alternative C — self-describing / versioned identity (never rehash; version the address)

**Precedent: IPFS CIDv1** — a self-describing identifier (`multibase ‖ version ‖ multicodec ‖ multihash`)
whose embedded version/codec lets the scheme evolve by minting new self-declaring addresses, old and new
coexisting, **avoiding any flag-day** at the cost of every consumer handling multiple schemes forever
(`Declared`; <https://github.com/multiformats/cid>). **Unison** is the limiting case: content-addressed,
**append-only immutable** identity where a "change" is an *addition* of new-hash content beside the old, so
migration dissolves entirely (`Declared`; <https://www.unison-lang.org/docs/the-big-idea/>).

- **Buys:** no future rehash ever; new doors are additive by construction. Mycelium's `Canon` **already
  has a weak form** of this — frozen prefix tags make *additive* doors free (§3); D-Float exploits it.
- **Costs:** to make *reshaping* doors additive too, the address must carry a scheme-version and every
  reader must decode all live versions **permanently** — the IPFS multi-scheme tax, forever. For a small,
  known, pre-persistence door set this is strictly more machinery than the batched no-op buys back.

**Rationale for recommending A.** The doors are **few, known, and pre-persistence**; the address is a
**closure hash** (Git's exact precondition for "one-shot beats incremental"); and the `Canon` frozen-tag
model already gives Alternative C's benefit *for the additive doors* without its permanent tax. So the
batched settlement is the cheapest path to a clean single-scheme end-state — and, timed before first
persistence, costs a migration of **zero** values. Alternatives B and C are named here precisely so the
maintainer ratifies the batch on merit, not by default (house rule #4; VR-5 — no assent past its basis).

## 6. What this settles — D3 and D6 become decidable (the unblock)

The two CU gates that `depends_on` "the E20-1 content-address settlement" are **downstream of exactly
this discipline** (`docs/planning/deferred-design-CU-gates.md:50-94`; DN-90 §6):

- **D3 — CU-9 Dense dtype / quantization surface** (`blocked`, `depends_on: E20-1 (ADR-030)`, `:52`).
  Its blocker is not the *design* (ADR-030 already decided the descriptor shape) but the *unsettled
  rehash coordination* — "when does this identity change land, and does adding it trigger a second
  one-way door?" The discipline answers both: **D3's identity delta is D-Dense in the manifest (§2);
  it lands with the batch before first persistence; it triggers no second rehash.** D3 thus drops from
  `blocked` (waiting on an undefined event) to `needs-design` (a normal surface-design task under a
  known gate).
- **D6 — CU-7 growable ternary value form** (`blocked`, `depends_on: E20-1 (ADR-030)`, `:87`; DN-90 §6).
  Same shape: the arithmetic + `BigTernary` are built (§2/§3); the blocker is the growable
  `Repr::Ternary` payload's identity coupling. The discipline places it as **D-BigTernary in the
  manifest**, landing with the batch — so D6 becomes a decidable "surface the growable form + decide its
  canonical growable encoding" task, its rehash coupling resolved by the batch, not by a bespoke migration.
- **DN-90's growable tier** (§5/§6) is unblocked the same way: its held pieces (growable `Repr::Ternary`,
  growable/beyond-`u32` Dense/VSA `dim`, quantized-Dense growth) are D-BigTernary + D-Dim in the manifest;
  DN-90's decidable-now list (§6.1) is unchanged (those spend no identity change and stay landable
  independently).

Settling the discipline does **not** implement D3/D6 or spend the rehash — it converts them from *blocked
on an unscheduled, unshaped event* to *gated on a named milestone (first-persistence) with a known landing
mechanism*. That is the precise sense in which they become **decidable** (DN-90 §6).

## 7. User stories

- *As the **maintainer**, I want* the single-rehash policy turned into a concrete coordination discipline
  — a frozen manifest, a named trigger, a before-vs-settled split — *so that* I can ratify *how* the one
  rehash absorbs every door and *when* it fires, on merit, without spending it prematurely.
- *As a **kernel engineer**, I want* every accepted-but-unspent identity change tracked in one append-only
  manifest with its additive-vs-reshaping class, *so that* I never land a reshaping door past first
  persistence and trigger a costly second content-address flag-day (KC-3, NFR-5).
- *As a **value-model contributor (D3/D6)**, I want* my door's rehash coupling resolved by a known batch
  rather than an undefined event, *so that* the Dense dtype/quant surface and the growable ternary form
  become normal design tasks under a named gate instead of indefinitely `blocked`.
- *As an **auditor**, I want* the settlement to spend no rehash and move no decision status, *so that* the
  strategic-timing call stays explicitly the maintainer's and the record shows the batch was chosen over
  per-feature and self-describing identity on cited merit, not by drift.

## 8. Definition of Done (this note's gate)

- [x] The **one-way doors that must be batched** are enumerated with each door's *identity change* and
      *why it must be in the batch*, verify-first and cited (§2) — landed work (D-Float additive, D-BigTernary
      arithmetic) documented as **existing**, not re-proposed.
- [x] The **`Canon` mechanism** is grounded (§3, `content.rs` source-read): additive doors spend no
      rehash; reshaping doors need it; **pre-persistence ⇒ the batch can be a no-op migration**.
- [x] A **coordination discipline is recommended** (§4): the append-only manifest, a refined trigger
      predicate, the freeze-point + ordering, and the decided-before-vs-settled-by split.
- [x] **At least one alternative** is presented with tradeoffs (§5: per-feature rehash; self-describing
      versioned identity) plus a **rationale** for the recommendation, with **cited prior art** (Git,
      Nix, IPFS, Unison — `Declared`).
- [x] The note shows the settlement makes **D3 and D6 decidable** and unblocks DN-90's growable tier (§6).
- [x] **User stories** (§7) + this **Definition of Done** are present (house rule #6); the note is
      **`Declared`/Draft** throughout and cites its basis (house rule #4); it **spends no rehash and
      self-advances nothing** (house rule #3).
- [ ] **Maintainer ratifies the discipline** (the recommended freeze-and-land coordination + the trigger
      predicate) — or requests revisions — **and separately decides the strategic timing** (when/whether to
      schedule the first value-persistence feature that fires the trigger). This note **stays Draft** until
      then and **never spends the rehash** (house rule #3 — strategic-timing is the maintainer's). *(Open.)*
- [ ] **FLAG handoffs to the orchestrator** (this note does not apply them): the **manifest's home** (a new
      `docs/planning/content-address-rehash-manifest.md` vs. an RFC-0033 §7 appendix), the Doc-Index row, the
      CHANGELOG entry, and any minted tracking id are **orchestrator/maintainer-owned** close-out — DN-93 owns
      only its own file (§4a). *(Open.)*

> **Append-only (house rule #3).** This note **supersedes nothing** and moves no decision status. It
> **operationalizes** RFC-0033 §7's already-ratified single-rehash policy, **builds on** ADR-029/030/031,
> ADR-040 §3, ADR-038 §2.6, and DN-90 §5/§6, and **defers** the strategic timing (when the trigger fires)
> and the rehash spend entirely to the maintainer. It records the manifest and the trigger as *recommendations
> for ratification*, resolving neither the timing nor the still-open door shapes (D3/D6, FLAG-5, D-Signed —
> §4c). CHANGELOG / Doc-Index / issues.yaml / docs/api-index and the manifest's placement are owned by the
> integrating parent — this leaf flags, it does not edit them.

---

## Meta — changelog

- **2026-07-08 — Created (Draft) — authored on the `trx2` design lane (non-blocking, non-conflicting
  worktree; maintainer directive 2026-07-08 — leverage Tero for grounding).** Works up the **E20-1
  content-address rehash settlement** — the meta-blocker DN-90 §5/§6 flagged under the growable tier and
  the D3/D6 CU gates. **Enumerates the pending-identity-change manifest** (§2): the ADR-030 Dense-quant
  door, the ADR-031 VSA door, the ADR-040 scalar-float commitments (additive, "documented not spent"), a
  growable `Repr::Ternary` (ADR-029; D6), growable/beyond-`u32` Dense & VSA `dim` + quantized-Dense growth
  (D3; DN-90), the conditional Binary-`signed` door (RFC-0033 §4.1.4), and the FLAG-5 f64-payload NaN
  commitment — each with its identity delta and why it batches. **Grounds the `Canon` mechanism** (§3,
  `content.rs` source-read): frozen prefix tags make *additive* doors spend no rehash while *reshaping*
  doors need it, and because Mycelium is **pre-persistence** the batch can be a **no-op migration over
  zero values** if every door lands before the first value is persisted. **Recommends a freeze-and-land
  coordination discipline** (§4): an append-only manifest, a refined trigger predicate for "the first
  value-persistence feature," the freeze-point + ordering, and the decided-before-vs-settled-by-the-rehash
  split. **Presents alternatives with cited prior art** (§5, `Declared`): the batched single rehash
  (recommended — Git's SHA-1→SHA-256 closure-coupled flag-day), per-feature rehash (Nix CA-derivations
  coexistence), and self-describing versioned identity (IPFS CIDv1 / Unison append-only) — with the
  rationale that few, known, pre-persistence, closure-hashed doors make the batch cheapest. **Shows the
  settlement makes D3 (Dense dtype/quant) and D6 (growable ternary) decidable** and unblocks DN-90's
  growable tier (§6): they drop from `blocked`-on-an-undefined-event to gated-on-a-named-milestone.
  Includes user stories (§7) + a Definition of Done (§8). **Status Draft; `Declared` throughout — decides
  nothing, spends no rehash, never self-advances (house rule #3); the strategic timing is the maintainer's;
  every normative claim cites a ratified ADR/RFC/DN or is marked open (house rule #4).** CHANGELOG /
  Doc-Index / issues.yaml / docs/api-index and the manifest's placement owned by the integrating parent.
  (Append-only; VR-5; G2.)
