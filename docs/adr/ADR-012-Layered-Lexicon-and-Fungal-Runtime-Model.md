# ADR-012: Layered Lexicon and Fungal Runtime Model

| Field | Value |
|-------|-------|
| **ADR** | 012 |
| **Title** | Layered Lexicon and Fungal Runtime Model |
| **Status** | Accepted |
| **Date** | 2026-06-10 |
| **Depends on** | DN-02 (Fungal Lexicon), RFC-0001 (Core IR), RFC-0003 (VSA), RFC-0004 (Execution Model) |

## 1. Context

The Mycelium language requires both a clean, learnable surface syntax and a powerful runtime model capable of expressing adaptive, resilient, distributed computation across heterogeneous substrates (including future underground AI infrastructure).

DN-02 established a conservative, hybrid approach to surface keywords using a strict three-test gate. However, the deeper fungal execution model research (hyphal growth, anastomosis, sclerotia, mycorrhizal symbiosis, foraging, etc.) offers significant value for the runtime and architectural layers.

A purely conservative approach would under-utilize the biological metaphor. A purely aggressive theming approach would harm learnability.

## 2. Decision

Adopt a **three-layer lexicon model**:

- **L1 (Surface)**: Conservative theming using the DN-02 ratified terms + `consume`, `embody`, `grow`.
- **L2 (Runtime & Architecture)**: More aggressive use of fungal concepts with short, mnemonic abbreviations (`hyph`, `anas`, `xloc`, `sclrt`, `myco`, `forage`, `rhizo`, `cmn`, `dimorph`, `reclaim`).
- **L3 (Formal/IR)**: Technical terms from RFC-0001 with light fungal influence where helpful.

Additionally, integrate the comparative analysis showing Mycelium’s position relative to Rust, Unison, MLIR, and others, highlighting its novel contributions around certified representation swapping and monotonic honesty.

## 3. Rationale

- Learnability is a first-class non-functional requirement.
- The fungal metaphor is most powerful at the runtime and systems architecture level.
- Short mnemonic forms in L2 allow expressive power without punishing daily development.
- This structure supports both human developers and future AI agents working with the language.

## 4. Consequences

**Positive**
- Clear separation of concerns.
- Strong mnemonics without cognitive overload.
- Positions Mycelium well for resilient, adaptive, multi-paradigm distributed systems.

**Negative / Risks**
- Requires good documentation and tooling to help developers navigate layers.
- Some L2 terms may still feel unfamiliar initially.

## 5. Implementation

- Add `docs/notes/Lexicon-Reference.md`
- Add `docs/notes/Example-Programs-Reference.md`
- This ADR itself
- Update `docs/Doc-Index.md` and relevant RFCs as needed.

## 6. Status

**Accepted** — 2026-06-20. All §7 flags resolved by downstream decisions; see §8.

---

## 7. Architect Review Notes (2026-06-10)

A deep verification of this ADR, `Lexicon-Reference.md` (v0.4), and `Example-Programs-Reference.md`
(v0.2) against the ratified corpus (DN-02, RFC-0001/0003/0004/0006/0007, ADR-010/011, Foundation).
Findings are grounded and flagged ⚑ where a maintainer decision is needed. Some were *applied* now
(terminology de-confliction, example fixes); the rest are recommendations.

### 7.1 Resolved now — layer-label collision (applied)

The lexicon's "L1/L2/L3" **collided** with RFC-0006 §3's language **layers L0–L3** (L0 Core IR →
L1 kernel calculus [RFC-0007] → L2 surface → L3 projection) — and *inverted* them (lexicon
"L1 = Surface" vs RFC-0006 "L1 = kernel calculus, internal"). Two different "L1"s is exactly the
kind of confusion the project forbids. **Applied:** the lexicon *tiers* are renamed
**Surface / Runtime / Formal** (no numeric collision) in `Lexicon-Reference.md`; this ADR's §2
should be read with that renaming. The vocabulary *tiers* are orthogonal to the language *layers*.

### 7.2 Control-flow contradiction with the functional core ⚑

The Lexicon's "Conventional Terms" list includes `loop, while, for, break, continue, return`, and
Example #17 uses `loop`. This **contradicts** RFC-0007 §6 and DN-02 §6: the L1 kernel calculus is
**functional** — recursion is via `fn` + `Fix`, with the structural totality posture (LR-4); there
are no imperative loops, and unbounded `loop`/`while` would undermine the totality/divergence story
(Q4/Q7). **Recommendation:** either (a) drop imperative iteration from the surface (recursion only,
consistent with the current design), or (b) introduce *bounded* iteration as Surface-tier sugar
that elaborates to recursion — which requires an RFC-0007 amendment. Until decided, treat
`loop/while/for/break/continue/return` as **not reserved**.

### 7.3 The Runtime tier is aspirational and ungrounded ⚑ (most important)

The Runtime vocabulary (`hyph`, `anas`, `xloc`, `sclrt`, `myco`, `forage`, `rhizo`, `cmn`,
`dimorph`, `reclaim`) presupposes a **concurrency + distribution execution model** — lightweight
tasks, channel fusion, translocation, a gossip mesh, checkpoint/migration. **No such model exists
in the ratified corpus.** RFC-0004 defines the execution model as a single trusted interpreter +
MLIR→LLVM AOT over value semantics; there is no concurrency, distribution, or actor/mesh anywhere,
and the T0–T3 research grounds representations/numerics/VSA/selection, **not** distributed systems.
Per the project's grounding rule (every normative claim cites its basis or is marked open), this
vocabulary is currently **ungrounded**.

**Recommendation (the honest path, matching how the rest of the corpus was built):**
1. Treat the Runtime tier as a **reserved, aspirational vocabulary** — documented, name-stable, but
   **not active syntax** and **not part of the execution model** yet.
2. Open a **Runtime/Concurrency RFC** (proposed: RFC-0008) that defines the execution model
   (concurrency unit `hyph`, its scheduling, what `anas`/`xloc`/`cmn` *mean* operationally, the
   memory/termination implications) — and reconciles it with RFC-0004, LR-4 (totality/`matured`),
   and LR-9 (a `spawn`/mesh world reopens leak/lifetime questions the value-semantics model had
   closed).
3. Commission a **research pass (Pass-4)** on distributed/actor runtimes (Koka/OCaml effects for
   concurrency, BEAM/actor supervision, CRDTs for `anas` state-merge, libp2p/gossip for `cmn`,
   checkpoint/restore for `sclrt`) to ground the model the way T0–T3 grounded the substrate.
Until (2)/(3) land, the Examples that use Runtime primitives are **illustrations of intent**, not
runnable language — marked as such in the Examples grounding notes.

### 7.4 `spore` scope drift ⚑

DN-02 / RFC-0003 §6 fix `spore` = **reconstruction manifest** (the recipe to regrow a *value*).
The Lexicon broadens it to "an immutable, verifiable artifact of *code + state + metadata* that can
*germinate into execution*" (a deployable unit, à la WASM module / container), and Example #12
treats the manifest as one *field* of a spore. These are different scopes. **Recommendation:**
generalize deliberately — `spore` becomes the content-addressed packaging unit, with the RFC-0003
reconstruction manifest as one component — and record it as an RFC-0003 revision so the narrow and
broad meanings don't silently diverge. (Do **not** leave both meanings unreconciled.)

### 7.5 Surface-tier additions vs the DN-02 three-test gate ⚑

`consume`, `embody`, `grow` are new Surface terms not in the Resolved DN-02 set. Gate evaluation:
- **`consume`** (take an affine `substrate` once) — passes (T-map: a fungus consumes substrate
  exactly once = affinity; teaches single-use). **Adopt.**
- **`grow`** (derive-like generation, `grow Debug for T`) — acceptable (T-map: the system grows new
  capability); mild genericness. **Adopt with note.**
- **`embody`** (inherent methods, ≈ Rust `impl`) — **weakest**: `impl` is highly machine-/human-
  familiar (T-learn favors keeping it), and "embody" teaches little about the behavior
  (T-illuminate weak). **Flag:** prefer keeping `impl`, or pick a stronger themed term.
Because DN-02 is **Resolved (append-only)**, these should be recorded via a DN-02 *amendment note*
(or a DN-03), not asserted only in a reference doc.

### 7.6 Short-form quality (Runtime tier) ⚑

Per the Lexicon's own rule (mnemonic, collision-free, 3–5 chars), several short forms are weak:
- **`sclrt`** — unpronounceable vowel-strip; its stated mnemonic ("scale + shelter") is an unrelated
  backronym. Prefer **`dorm`** (dormant→resumable) or **`chkpt`**, or keep full `sclerotize`.
- **`cmn`** — opaque (reads as an abbreviation of "common", not of a *mesh*). Prefer **`mesh`**.
- **`anas`** — opaque to newcomers. Prefer **`weave`** or **`fuse`** (teaches "connect/fuse").
- **`myco`** — collides with the language family name (Mycelium/"Myco"). Prefer **`graft`** (host
  binding) or **`symb`**.
- **`reclaim`** — clear, but **clarify scope**: it reclaims *runtime units* (stale `hyph`), **not
  memory** — LR-9 makes memory reclamation automatic with no manual free, so a memory-`reclaim`
  would contradict it.
The architecture makes this cheap: because identity is content-addressed (ADR-003) and there is one
canonical formatter (M-142, S3), a long form and a sanctioned short form are the *same* token (same
hash) projected differently — beginners can read `anastomose`, experts type `weave`, with no
identity cost. Recommend formalizing **one canonical long form + at most one sanctioned short
alias** per term.

### 7.7 Examples vs RFC-0001 semantics (fixes applied; notes added)

- Fixed two bracket typos (`Dense{… dtype: F16>>` → `}`; same in Example #11).
- **Ternary→Binary is partial** (RFC-0002 §4): Example #2 shows it as unconditionally `Exact` —
  added a note that `dec` is the partial direction (out-of-range is an explicit `Option`/error).
- **Example #11 bound kind:** a VSA→Dense lossy swap's bound is an `ErrorBound`/`ProbabilityBound`
  (ADR-010), not a `CapacityBound` (which describes VSA superposition capacity, not swap error).
- **Records / named type fields** (Example #1's `type Vector { data: … }`, `Dense{dim:…, dtype:…}`)
  are **not yet in the grammar** (`mycelium.ebnf` v0 has sum types with positional constructor
  fields). Either add record types + named fields to the grammar (a reasonable improvement) or
  rewrite the examples in the current sum-type form. Flagged in the Examples notes.
- **`Value<Repr>` vs repr-as-type:** RFC-0001/RFC-0006 make the representation *the* type
  (paradigm-in-the-type); the `mycelium-l1` grammar uses `Binary{8}` directly, so `Value<Binary{8}>`
  is redundant. Pick one surface convention.

### 7.8 No contradictions found with

ADR-010/011 (bounds), the guarantee lattice direction (`Exact ⊐ Proven ⊐ Empirical ⊐ Declared` —
the Lexicon's Formal tier states it correctly), content-addressing (ADR-003), and the `matured` ↔
stable-component mapping (RFC-0004 §4) — all consistent.

### 7.9 Recommended next steps

1. **RFC-0008 (Runtime/Concurrency execution model)** + **Research Pass-4** — ground the Runtime
   tier before any of it becomes active syntax (§7.3).
2. **DN-02 amendment** (or DN-03) recording `consume`/`grow`/(and a decision on `embody`) through
   the gate, plus the canonical-long-form + one-short-alias rule and the refined short forms (§7.5–§7.6).
3. **RFC-0007 amendment** resolving control flow (recursion-only vs bounded-iteration sugar; §7.2).
4. **RFC-0003 revision** reconciling the `spore` scope (§7.4).
5. **Grammar evolution** for record types/named fields if the examples' style is adopted (§7.7).

---

## 8. Resolution record (2026-06-20)

All §7 open flags were resolved by subsequent decisions before this ADR was accepted.
Status moved from **Proposed → Accepted** 2026-06-20.

| §7 flag | Resolution |
|---------|-----------|
| §7.1 L1/L2/L3 label collision | Applied at review time — tiers renamed Surface/Runtime/Formal in `Lexicon-Reference.md` |
| §7.2 Control-flow contradiction (`loop`/`while`/`for`) | RFC-0007 r2 — kernel stays functional; bounded iteration reserved as future sugar; `for` reserved, imperative loop keywords excluded |
| §7.3 Runtime tier ungrounded | RFC-0008 (Accepted 2026-06-16) + Research Pass-4 (`research/04-runtime-concurrency-RECORD.md`) — RT1–RT7 ground the Runtime vocabulary operationally |
| §7.4 `spore` scope drift | ADR-013 (Accepted 2026-06-10) — `spore` is the deployable unit; RFC-0003 reconstruction manifest becomes one digest-referenced component |
| §7.5 `consume`/`grow`/`embody` gate | DN-03 (Resolved 2026-06-10) — `consume`/`grow` adopted; `embody` declined (keep `impl`) |
| §7.6 Short-form quality | DN-03 §3 — one name per term (flat); ADR-012 §7.6's canonical+alias scheme rejected; single Runtime names ratified (`hypha`/`fuse`/`xloc`/`cyst`/`graft`/`forage`/`backbone`/`mesh`/`tier`/`reclaim`) |
| §7.7 Examples vs RFC-0001 semantics | Fixes applied at review time; grounding notes added to `Example-Programs-Reference.md` |

---

*End of ADR-012*
