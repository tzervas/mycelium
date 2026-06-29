# Design Note DN-64 — Mycelium Language Design: Synthesis Exploration Note

| Field | Value |
|---|---|
| **Note** | DN-64 |
| **Status** | **Draft** (2026-06-29 — initial research-synthesis capture; advisory; enacts nothing; all claims tagged per VR-5; open questions explicitly unflagged, not silently resolved) |
| **Decides** | *Nothing normatively.* Synthesizes five corpus-sweep facets (surface ergonomics, unique types, application capabilities, hot-inject security model, and idioms) into one advisory exploration note for maintainer review and later ratification. No design decision here supersedes an existing ADR/RFC; every proposal is `Declared` unless cited to a checked basis. |
| **Feeds** | RFC-0006 (S1–S6 surface invariants); RFC-0018 (graded typing); RFC-0020 (L2 surface); RFC-0025 (operator grammar); RFC-0034 / ADR-032 (tunable certification); RFC-0037 / DN-31 / DN-57 (delimiter and bracket grammar); RFC-0002 / RFC-0005 (swap certificates and policies); RFC-0003 / RFC-0009 (VSA algebra and resonator); RFC-0008 (runtime vocabulary); RFC-0014 (effects); ADR-003 (content-addressed identity); ADR-006 (no black boxes); ADR-013 / ADR-016 / ADR-017 (spore, ABI, hot-inject); DN-44 (codebase security posture); DN-63 (R2 distribution vocabulary) |
| **Date** | June 29, 2026 |

> **Posture (transparency rule / VR-5 / G2).** This is an advisory synthesis — a `Declared`
> aggregation over five corpus sweeps. Every claim is tagged at its highest supportable strength;
> nothing here is upgraded past its swept basis. Proposals are `Declared` design directions, not
> decisions. Open questions stay open — "I don't know" is the correct answer where no checked
> basis exists. Append-only: this note may gain dated sections as open questions resolve; it must
> not be rewritten. The parent orchestration task that generated this note is the authoritative
> commissioning record.

---

## §1 Mycelium-unique types and constructs

The following named constructs either have no equivalent in any mainstream general-purpose language,
or are extensions that let the programmer express something that cannot be expressed elsewhere.
Each is mapped to its nearest traditional paradigm and explained as an extension.

### §1.1 Paradigm-keyed Repr as a first-class type component

**What it is.** Every Mycelium value's type carries a closed `Repr` discriminant:
`Binary{N}`, `Ternary{N}`, `Dense{dim, dtype[, quant]}`, `Vsa{model, dim, elem, sparsity}`.
These four kinds are co-equal in the kernel type system (`crates/mycelium-core/src/repr.rs:57`,
`RFC-0001 §4.1`). Two values of different Repr are **unrelated types** — no subtype relation,
no implicit coercion.

**Traditional paradigm.** Closest to F# units-of-measure (a phantom type index on numerics).

**Extension.** F# units are erased at compile time and carry no guarantee strength (FR-M5 / G2).
Mycelium's Repr is **content-identity-bearing**: the paradigm is part of the BLAKE3 identity
hash (ADR-003, RFC-0001 §4.6), so `Binary{8}` and `Ternary{6}` are **structurally distinct
definitions**, not aliases. This forces every cross-paradigm interaction to go through an
explicit, inspectable `swap` node — the foundational anchor of the transparency story.
(`Proven` — RFC-0001 §4.1/§4.6, ADR-028/029/030/031.)

The survey records the four-way co-equal union as unprecedented in general-purpose language
design (Foundation §1, G1).

### §1.2 `swap` — the certified, never-silent representation change

**What it is.** `swap` is the only kernel node that changes a value's `Repr` (WF1,
RFC-0001 §4.5). Every `swap` carries a mandatory `PolicyRef` (WF2 — not an `Option`;
enforced by construction, `crates/mycelium-core/src/node.rs:37`). It produces a
`SwapCertificate`: either `Bijective` (binary ↔ ternary within range, proof reusable by
content hash) or `Bounded` (lossy, carrying `Bound{kind, basis}` where basis is universal
per ADR-011 and strength is derived, never asserted). Out-of-range conversion is an explicit
`SwapError::OutOfRange`, never a silent truncation.

**Traditional paradigm.** Closest to a C/C++ cast or Rust's `From`/`Into` trait.

**Extension.** C casts are unchecked; Rust's `From`/`Into` are type-safe but produce no
per-instance certificate; MLIR lowering passes produce no per-value inspectable certificate
at the program surface. The `SwapCertificate` is the difference — it makes the approximation
quality of **each individual value conversion** auditable, and the `PolicyRef` records which
selection policy drove the decision, so `EXPLAIN` can answer "why this representation, at what
cost" for every swap at every call site. (`Proven` — RFC-0002 §2–§5, RFC-0001 WF1/WF2.)

### §1.3 Guarantee lattice and graded type index

**What it is.** Every value carries `meta.guarantee: GuaranteeStrength` on the four-point
lattice `Exact ⊐ Proven ⊐ Empirical ⊐ Declared`. Composition law: `meet` — the weakest
always wins (`RFC-0001 §4.7`). `Meta::new` enforces consistency by construction (M-I1 through
M-I4, `crates/mycelium-core/src/meta.rs`). RFC-0018 lifts this into a graded static typing
judgment `Γ ⊢ e : τ @ g`, with `swap` as the sole endorsed upgrade point.

**Traditional paradigm.** Closest to security type systems that track information-flow taint
(FlowCaml, Jif) or effect systems (Koka, Eff) that track computational effects.

**Extension.** Security type systems track **confidentiality or integrity of information**;
effect systems track **computational effects** (side effects, exceptions, state). Mycelium's
lattice tracks **approximation trustworthiness** — how well-established is the numerical
accuracy of this value? — as a monotone-downward type-level property that propagates through
computation. RFC-0018 §1 flags this specific combination (static grades plus machine-checkable
runtime certificates as the endorsement proof) as having no found precedent (T3.2). The
mechanism is `Proven`-implemented (RFC-0018 Enacted stage 1a, M-663); the full noninterference
proof remains `Declared` (RFC-0018 §11 — not machine-checked; see §6 open question OQ-A).

### §1.4 Provenance DAG as an operational gate

**What it is.** Every derived value records its provenance as an acyclic DAG in `Meta`:
`Provenance::Root | Derived{op: ContentHash, inputs: Vec<ContentHash>}`. The `policy_used`
field records which `SelectionPolicy` drove a `swap`. The full DAG can be walked to reconstruct
how any value was produced (the `EXPLAIN` surface, Foundation §5.8). (`Proven` —
RFC-0001 §4.3/§4.8, `crates/mycelium-core/src/meta.rs:88`.)

**Traditional paradigm.** Closest to data-lineage metadata in data-engineering tools (dbt,
Apache Atlas, Arrow).

**Extension.** In data engineering, provenance is **advisory documentation**. In Mycelium,
the provenance DAG is a **precondition for applying certain operations**: `vsa_to_dense` checks
that the source was produced by `swap.dense_vsa.enc.v1` before attaching the delta
(`crates/mycelium-cert/src/dense_vsa.rs:223`); a differently-produced VSA value yields
`NotDenseVsaEncoding`. No mainstream language makes value provenance a **runtime gate on
operation applicability**.

### §1.5 VSA (Vector Symbolic Architecture) as a first-class value kind

**What it is.** Five VSA models (MAP-I, MAP-B, BSC, HRR, FHRR) share one algebra
(`bind`/`unbind`/`bundle`/`permute`/`similarity`) but have **different per-operation guarantee
tags** per model, recorded in a normative matrix (`RFC0003_MATRIX`,
`crates/mycelium-vsa/src/matrix.rs`). MAP-I `bundle` is `Proven` iff the side-condition
`dim ≥ requiredDim(m, δ)` is checked (Clarkson-Ubaru-Yang 2023, proofs/lh-bundle/). HRR
`unbind` is `Empirical` (not self-inverse). Resonator-network factorization (RFC-0009) carries
at most `Empirical`, with a validated regime envelope and explicit `OutsideEmpiricalProfile`
refusal. (`Proven` for the matrix and Liquid Haskell proof; `Declared` for the application
layer — RFC-0003, RFC-0009, ADR-031.)

**Traditional paradigm.** Closest to research VSA libraries (torchhd, openhdmap) embedded in
Python or Rust.

**Extension.** Existing libraries provide the algebra but no typed guarantee lattice, no
per-operation tags backed by non-asymptotic theorems at the type level, and no explicit
operational-regime gate. Mycelium ties `τ @ Empirical` vs `τ @ Proven` to the specific
model-and-operation combination, and the runtime checks the regime profile at factorization
time — making out-of-regime calls an explicit error rather than a silently-degraded answer.

### §1.6 `substrate` — affine external resource

**What it is.** A `substrate` is an affine external resource (file handle, socket, capability)
consumed exactly once by construction (LR-8, Glossary §2.18). The fungal metaphor teaches the
linearity: a substrate is consumed as the fungus grows. In safe Mycelium, a memory leak is not
expressible (LR-9); a phylum/nodule without `wild` blocks is certified leak-free by
construction.

**Traditional paradigm.** Closest to Rust ownership (linear types, Clean) or session-type
systems.

**Extension.** Affine/linear types are not novel in themselves. Mycelium's distinction is the
**combination** with the certification lattice (a substrate handle carries `Meta` with its
guarantee tag), the three-layer hybrid memory model (DN-32/RFC-0027: affine as primary, RC only
for explicit sharing, region-based for scopes), and the fungal vocabulary that makes the
ownership metaphor structurally legible. The static-analysis claim (leak-freedom at the
phylum/nodule level, MEM-4/DN-33) is `Declared` — designed but not fully enacted.

### §1.7 Bounded effects with declared budgets

**What it is.** A function's effect set is visible in its signature `-> ret !{ eff1, eff2 }`
(enacted, M-660). Any potentially-unbounded effect carries an explicit static budget
(`retry(<=3)`, `alloc(<=64KiB)`); exceeding the budget is `EvalError::EffectBudget`, never OOM
or a hang. Recovery is additive (I1: a handler acts on an error and produces a new explicit
outcome; it never makes the original refusal vanish unobserved). A substituted fallback carries
at most `Declared` (I2 — VR-5 applied to recovery). The budget enforcement reuses the
interpreter's existing fuel/depth ledger (mycelium-interp::budget, M-353). (`Empirical` for
enacted v1; the static literal budget-bound syntax (RFC-0014 §10.1 D1) is `Proposed`.)

**Traditional paradigm.** Closest to algebraic effect systems (Koka, OCaml 5, Effekt).

**Extension.** Existing effect systems track and handle effects but have no typed static budget
bounds on effect **cardinality** (a `retry(<=3)` annotation that is statically checked and emits
a graceful error on overrun is not expressible in Koka or OCaml 5). The budget generalizes the
same fuel-clock already used for totality (RFC-0007 Fix/FixGroup), keeping the trusted kernel
small (KC-3).

### §1.8 `matured` scope with totality gate and path-independent lattice

**What it is.** A scope (nodule/phylum/program) declared `// @matured: true` is promoted from
interpreted to compiled-and-frozen. Totality is required for **all** reachable definitions
(RFC-0007 §4.5, quantified over the scope). `thaw fn f` is the escape hatch (one definition
kept interpreted inside an otherwise matured scope, with guaranteed identical observable —
NFR-7). (`Declared` — RFC-0017 §4.3/§5, Glossary §2.10, DN-08.)

**Traditional paradigm.** Closest to MetaOCaml staging or RPython translation-time annotations.

**Extension.** MetaOCaml has staging; JVM has JIT. Mycelium's `matured` is distinctive in that
(a) it is **scope-level**, not per-function; (b) it requires totality of **all** reachable
definitions (guaranteeing AOT-eligibility by construction); and (c) it is tied to the guarantee
lattice — `thaw` de-matures without weakening the guarantee tag (NFR-7 path-independence). The
guarantee tag is not an artifact of execution mode.

### §1.9 Tunable certification: `fast` and `certified` modes

**What it is.** RFC-0034 (Enacted via ADR-032) makes the per-swap certificate machinery, per-
value guarantee tags, and content hashing a tunable policy. `fast` is the default (transparent,
inspectable, non-certified); `certified` is opt-in per global/phylum/nodule scope. The mode is
never silent (G2): every result is mode-tagged and EXPLAIN-able; cross-mode composition is
explicit. In `fast` mode the system provides transparent, debuggable auditability; in `certified`
mode it provides checked, certificate-backed auditability. (`Proven` — ADR-032, RFC-0034, DN-29.)

**Traditional paradigm.** Closest to Rust's `unsafe` (opt-in escape) or dependent types (all-or-
nothing verification).

**Extension.** `unsafe` gates memory operations; Mycelium gates **representational approximation
claims**. Unlike dependent types (all-or-nothing), tunable certification degrades gracefully to
`Declared` (never to silence) when the machinery is off — fast mode gives a systematic, flagged
downgrade, not a hidden overclaim (KC-4 generalized from a one-time kill-switch into a knob).

### §1.10 Unified SelectionPolicy across three selection sites

**What it is.** RFC-0005's `SelectionPolicy` is a single reified, content-addressed, EXPLAIN-able
mechanism serving three historically-separate systems: swap-target selection (RFC-0002), packing
schedule (RFC-0004), and task placement/foraging (RFC-0008 RT3). The `policy_used` field in every
`Meta` records which policy drove the decision. (`Declared` — RFC-0005 §2, RFC-0008 RT3, ADR-006;
the three-site unification is Accepted direction, not yet fully enacted in code.)

**Traditional paradigm.** Closest to Legion's mapper-separation principle (T4.3) or SQL EXPLAIN
for query plans.

**Extension.** In conventional languages, representation choice (compiler), memory layout (linker),
and task placement (scheduler) are three separate, opaque systems with no common inspection surface.
Mycelium's RFC-0005 mechanism serves all three sites, so `EXPLAIN` gives a **uniform answer** about
what was chosen and why, from the value level to the placement level.

---

## §2 Optimal surface sugar and ergonomics

### §2.1 Ratified commitments binding all sugar candidates

The following are `Proven` constraints from the corpus; no candidate sugar may violate them:

- **S1 (never-silent swap).** No sugar or trait resolution may insert a `swap` node implicitly.
  Any path that would require an implicit representation change must fail with an explicit
  `MissingConversion` error (RFC-0006 §4.1, RFC-0020 §4.1, RFC-0012 §4.4).
- **S2 (honest tags surface).** Desugaring may not strengthen a guarantee tag. A `Declared`
  annotation propagates into elaborated L1 unmodified (RFC-0006 §4.1, RFC-0020 §4.1).
- **S3 (content-addressed identity over elaborated L1).** Sugar names and syntactic variants do
  not affect identity. Sugar is purely a surface convenience (RFC-0006 §4.1, ADR-003).
- **S4 (inspectable elaboration).** Every sugar term must elaborate through a deterministic,
  dumpable pipeline. Hidden choices are forbidden (RFC-0006 §4.1, ADR-006).
- **S5 (explicit partiality).** Sugar cannot hide partial operations. Any sugar that reduces a
  match or conditional must either be total by construction or be rejected (RFC-0006 §4.1,
  RFC-0020 §4.4).

Additionally: the bracket allocation is fixed (RFC-0037 D1–D2); the precedence table is fixed
(RFC-0025 §4.1, Enacted 2026-06-28); layout-independence (RFC-0037 D3) requires that sugar
parse identically whether written dense or formatted; the `lambda` keyword is reserved and
active (RFC-0037 D5, Enacted 2026-06-27); the semicolon terminator is mandatory (DN-57 §5,
M-818 Enacted 2026-06-29).

### §2.2 Recommended sugar

**Pattern sugar via Maranget compilation.** Nested patterns, or-patterns, and guard clauses
compile to flat `Match(Match(…))` trees via usefulness-checked decision-tree compilation
(RFC-0020 §4.4). This is the correct template for expression-level sugar: no new kernel node
(KC-3); dumpable; exhaustiveness and redundancy checked. Pattern sugar is `Proven` within this
framework.

**Derived forms (if-else chains, let-chains, for-loops).** These are ratified L2 elaborations
with specified desugaring to L1 nodes; each is inspectable; guarantee tag is the meet of
constituent tags (RFC-0020 §4.5, RFC-0007 §4.8). They set the precedent for further syntactic
convenience.

**Short paradigm-type keywords** (`bin{N}`, `tern{N}`, `emb{D,S}`, `hvec{E,D,Sp}`).
Proposed syntactic aliases for `Binary{N}`, `Ternary{N}`, `Dense{D,S}`, `VSA{E,D,Sp}`
(RFC-0037 D2-b, DN-31 §2). Elaboration identity with their long forms is `Declared` pending
full implementation. If adopted, they should bind only to type literals, not to trait
methods/associated types (open question — see §6).

**Word-canonical operators.** Symbolic infix sugar is frontend-only desugaring to word
functions (`add`, `mul`, `lt`, etc.); words remain valid everywhere symbols are
(RFC-0025 §4.2, Enacted). This is the extensibility pattern: new operators are defined by
adding their word function, not by extending the grammar.

**Guarantee-index annotation** (`@ strength`). A value-position annotation specifying the
lattice tag, binding tighter than all infix operators (RFC-0018 §4, Enacted M-663). Sugar that
affects guarantee strength must be transparent about it via `@ annotation`.

**Record-literal shorthand** (`{x, y}` elaborating to `{x: x, y: y}`). Syntactically
convenient; no identity change; no `swap`. Requires careful shadowing rules (the bound `x` must
deterministically refer to the record field — see §6). `Declared` candidate.

**Pipe operator** (`|>` for sequential composition). Desugars to nested function application;
tests that no transparency is lost, staged pipeline reads cleanly, and identifier shadowing
is handled. `Declared` candidate; must parse identically dense and formatted (RFC-0037 D3).

### §2.3 Anti-patterns — sugar that is forbidden

**Any implicit representation coercion.** Operator overloading that implicitly converts
arguments, trait-bound resolution that inserts a representation change, or any form of
"automatic alignment" of paradigms is forbidden by S1. The `MissingConversion` error is the
correct outcome.

**Guarantee-strength upgrade without a checked swap.** Sugar that strengthens a `Declared`
or `Empirical` tag to `Proven` or `Exact` — even syntactically, by omitting an annotation —
violates S2. The `@` annotation is the only surface mechanism for expressing a grade, and
`swap` is the only kernel mechanism for endorsing an upgrade.

**Hidden elaboration.** Any sugar whose desugaring cannot be dumped via the stage-dump channel
(M-140) violates S4 (inspectable elaboration / ADR-006). No "magic" synthesis or invisible
inference is permitted.

**Juxtaposition application or indexing syntax.** Introducing `f x` (juxtaposition) or `seq[i]`
(indexing) would violate the type-position vs value-position disambiguation invariant
(RFC-0037 D3) and the no-indexing principle. Array element access is `get(seq, i)` — a
function call, not syntax.

**Ternary operator** (`cond ? a : b`). The `?` token is not in the bracket allocation
(RFC-0037 D1); the `:` is already the ascription token; the construct conflicts with the
comma-and-semicolon grammar. Anti-pattern: the canonical form is `match cond { True => a;
False => b; }`.

**Postfix `await` or ad-hoc effect-handling suffixes.** RFC-0014 §3 defines divergence-only
effect tracking for v1; multi-effect row polymorphism is `Proposed` (RFC-0014 §10 r2). No
postfix effect syntax is committed; any proposal must fit within the `!{…}` declared-budget
surface.

**Varargs or variadic arguments.** Conflicts with the value-semantics model (all args explicit)
and the multi-arg/tuple prerequisite (RFC-0024 §4A.8). Multi-argument functions use tuple types.

---

## §3 Unique application capabilities

The following are concrete small-app sketches for programs Mycelium makes natural. All are
`Declared` (design-phase illustrations; the underlying machinery components are implemented but
complete runnable Mycelium programs are not yet executable per the design-phase status note in
`docs/examples/binary64-https-downloader.myc`).

### §3.1 Provenance-audited numeric pipeline with static grade propagation

A sensor-to-actuator pipeline where every intermediate value carries a statically-enforced
guarantee grade that degrades by meet at each composition step. Raw `Binary{64}` readings are
checked-added (grade `Exact` within range), passed through a lossy swap to a dense embedding
(grade `Empirical`, tagged with ε), and a downstream function demanding `Exact` input is a
**static type error** if called with the `Empirical` result — caught at compile time, not at
runtime (RFC-0018 §4.3 G-App rule).

**Downstream effect on authoring.** Every function signature must carry a grade annotation,
making the trust chain structurally visible in the source. A pipeline that mixes `Proven` bounds
(MAP-I bundle, Clarkson-Ubaru-Yang 2023) with `Declared` assertions (an FFI invariant) cannot
silently return a result tagged `Proven` — the meet rule forces the result to `Declared`. No
other language-as-substrate makes this the structural authoring discipline.

### §3.2 Certified approximation with EXPLAIN-able swap history

A quantization pipeline where fp32 weights are swapped to packed ternary (BitNet-class I2_S
packing). The swap emits a `SwapCertificate::Bounded` carrying ε (rounding error, ADR-010
`ErrorBound` kernel) tagged `Empirical` (Frady-Sommer basis) and a `PolicyRef` referencing the
cost-based selection policy that chose I2_S over TL1. `EXPLAIN` on any weight tensor shows
which policy chose this representation, at what cost, and why. The policy itself is a
content-addressed artifact, diffable across versions.

**Downstream effect on reviewing.** A quantization review diff shows exactly which representation
was chosen, under which policy, with what stated bound. Regulatory or safety audits can demand the
certificate corpus rather than rerunning experiments. No existing quantization framework (PyTorch,
TensorFlow, ONNX) provides a per-tensor, policy-traced, inspectable approximation certificate.
(RFC-0002 §2–§3, RFC-0005 §2, ADR-006, RFC-0034 §3.)

### §3.3 VSA-native holographic knowledge base with regime-validated factorization

A holographic knowledge-base nodule that stores structured facts as MAP-I hypervectors
(bind roles to fillers, bundle multiple facts into a superposition), retrieves by unbinding
with known roles (`Proven` capacity bound), and — where factor roles are unknown — runs a
resonator-network factorization with a validated empirical success profile
(`F≤3`, `∏k≤4096`, `d≥4096`, `δ=0.02`). The factorization (a) attaches a per-run trace
(iteration count, per-slot similarity trajectory, stop reason) that `EXPLAIN` can render;
(b) refuses out-of-profile queries with explicit `OutsideEmpiricalProfile` error.

**Downstream effect on program behavior.** A knowledge-base call with `F=4` is an explicit
error rather than a silently-degraded answer. The program cannot accidentally use factorization
outside its validated regime — the regime check is structural, not a convention. No existing VSA
toolkit provides this. (RFC-0009 §5–§6, RFC-0003 §4 matrix.)

### §3.4 Ternary-native classifier with inspectable packing metadata

A classifier network where weights are stored as `Ternary{2}` (three states: -1 suppress,
0 neutral, plus1 excite), arithmetic is balanced-ternary, and the substrate packs five trits per
byte (I2_S metadata). A runtime `EXPLAIN` on any weight tensor shows: paradigm=`Ternary{2}`,
packing=I2_S, guarantee=`Exact` (within-range bijection applied at load time), swap-certificate
hash for the original Binary→Ternary conversion. Out-of-range conversion is an explicit
`Option`/error, never a silent truncation.

**Downstream effect on trust.** A ternary network's authoring provenance is auditable per-weight.
The forward-compat contract (ADR-005, NFR-5) means value-semantics for `{-1,0,+1}` is preserved
if ternary hardware ever arrives. No existing language or framework provides ternary as a first-
class paradigm with certificate-backed, inspectable packing metadata.
(RFC-0002 §4, ADR-005, FR-C3, DN-01.)

### §3.5 Budget-bounded plugin sandbox with typed effect signatures

A host program running untrusted plugin code typed with `!{ io, alloc(<=1MiB), cascade(<=2) }`.
The runner enforces: no undeclared IO (coverage checker, M-660); no more than 1 MiB allocated
(budget ledger, M-353); no handler cascade deeper than 2 levels (`EffectBudgetExhausted` on
overrun). A plugin performing a `retry` effect undeclared in its signature is an explicit
`UndeclaredEffect`, not a runtime surprise. `EXPLAIN` on the run answers what effects were
performed, against which budgets, with which remaining headroom.

**Downstream effect on security review.** A plugin interface review reads the effect signatures
rather than running a profiler. The never-silent overrun means a budget violation is always an
explicit, matchable error value the host can act on. Koka and OCaml 5 have effects but no typed
static budget cardinality bounds. (RFC-0014 §4.5, RFC-0007 §4.5, M-353/M-660.)

### §3.6 Security-typed network client where insecure states are unrepresentable

The HTTPS-downloader example (`docs/examples/binary64-https-downloader.myc`) demonstrates a
program where: `TlsPolicy` has no `Disabled` variant (disabling verification is a type error);
`Url` can only be constructed by `parse_https_url` (non-HTTPS yields explicit `Err`);
`Budget` has mandatory finite fields (unbounded transfer is unrepresentable); secrets come
through `!{io}`, not source.

**Downstream effect on authoring discipline.** The guarantee-tag discipline applies uniformly:
a security contract (`Declared` TLS delegation) and an approximation bound (`Exact` length
comparison) use the **same infrastructure**. A reviewer auditing the download function reads
the guarantee tags alongside the type signatures and sees exactly where the trust chain was
delegated. (RFC-0018 §4.3, RFC-0014 §3.1, RFC-0034 §3 invariant 4, FR-M3.)

---

## §4 The hot-inject security model

### §4.1 Current state — correctness, not a security gate

Hot-inject (ADR-017) is currently designed as a **correctness and atomicity** mechanism. The
dispatch table registers a `ContentHash → entry` pair; immutability plus content-addressing
dissolve the atomicity hazard (a change is a new hash under a new entry, never an in-place
mutation). **No signing requirement or unsigned-code refusal is present in the current corpus.**
(`Exact` — ADR-017 §decision 2–5, `crates/mycelium-mlir/src/inject.rs`.)

This is the gap: the current mechanism is sound for atomicity but does not yet constitute a
trust boundary that refuses unsigned code in production. A content hash is collision-resistant
identity but **not authentication**: an attacker who can write a new node to the Image can
register an arbitrary definition under its own valid hash (ADR-003; `inject.rs:153-168`).

The concrete attack vector: a malicious `.so` artifact loaded via `dlopen` (the JIT substrate,
`crates/mycelium-mlir/src/jit.rs`, M-340; ADR-014/DN-44 §2 — eight unsafe blocks confined
to `jit.rs`). Content-addressing prevents a silently-stale entry but does **not** prevent
injecting a newly-minted malicious definition. The signing gate must verify that the artifact
was produced by a **trusted preparation phase**, not just that its hash is internally
consistent.

### §4.2 Corpus groundwork for a security extension

Several existing corpus elements are load-bearing for the proposed extension:

- **Spore signatures are already named** (ADR-013 §2 component 4: "artifact metadata —
  provenance, guarantee/bound certificates, signatures") but their design is explicitly deferred
  (RFC-0008 §R8-Q5).
- **The compile/deploy phase split** (RFC-0034 §8, ADR-016/017 footnotes): spore identity hash
  and ABI dispatch keys live at the compile/deploy phase and remain available even with runtime
  certification off. A signing requirement analogously belongs to the **preparation phase**,
  checked at inject time, independent of the swap-cert certification mode.
- **Never-silent pattern** (G2) already applies to dispatch misses: `Image::call()` returns
  `InjectError::DispatchMiss` for an unknown hash — never a silent guess
  (`inject.rs:185-193`). Unsigned-code refusal is the security instantiation of the same
  principle.
- **NativeArtifact** (DN-18, M-620): a preparation-phase artifact with content-hash-derived
  identity, VR-4 attestation, and `Empirical` faithfulness tag. This is the closest existing
  precedent for a "preparation-phase certificate" attached to a compiled artifact.
- **DN-44 §1 thesis**: "the only security vulnerabilities that can exist are ones a developer
  introduces into their own `.myc` programs." Hot-inject of arbitrary unsigned code in
  production would be a direct counter to this thesis.

(`Exact` for the corpus groundwork; `Declared` for all proposed extensions below.)

### §4.3 Proposed design direction — two orthogonal axes

The fast/certified certification duality (RFC-0034) governs swap-certificate emission and
checking. It does **not** govern hot-inject security; these are **orthogonal axes**
(RFC-0034 §4 knob matrix, RFC-0034 §8). Conflating the two would be a design error. A developer
can run `fast` (no swap certs) while still requiring signed inject in production; a `certified`
runtime can run in loose inject mode during development. The security gate needs its own knob.

**Proposed inject-mode names** (`Declared` — candidates for maintainer ratification):

- **`loose`** (local-dev): unsigned injection permitted; every injected call is G2-tagged to
  make the unsigned status never silent. This maps to the "local-dev fast iteration" use case
  from the maintainer intent.
- **`sealed`** (production): injection requires a valid `InjectCert`; unsigned code yields an
  explicit `InjectError::UnsignedCode(ContentHash)` refusal — never-silent, carrying the exact
  hash that was rejected.

**Proposed `InjectCert`** (`Declared`): a preparation-phase artifact bound to a specific
`ContentHash`, produced by a trusted `myc-prepare` step. Proposed components: `content_hash:
ContentHash` (the dispatch key, ADR-003 — the signature is over the dispatch key itself, so
no secondary identity can drift); `signer: SignerId` (signing authority public-key fingerprint);
`signature: Bytes` (over `content_hash || vr4_attestation_digest`); `vr4_attestation:
CrossBackendGate` (the DN-18/M-630 no-opaque-lowering attestation, so the sealed gate also
asserts no opaque transform happened — fusing the security gate with the transparency/
auditability mechanism per ADR-006); `issued_at: Timestamp` (replay-attack surface — open
question OQ-I2).

**`myc-prepare`** (`Declared`): a toolchain step that (a) compiles the definition via the
native backend; (b) produces a `NativeArtifact` (DN-18); (c) signs the `ContentHash` plus VR-4
attestation with the project's signing key; (d) emits an `InjectCert`. This is the **preparation
phase** — the authorization phase, separate from the build phase, analogous to `cargo publish`'s
release-signing step.

**`TrustRoot`** on the `Image` (`Declared`): the set of trusted `SignerId`s. In a production
colony the `TrustRoot` is set at image genesis (RFC-0008 §4 germinate) and is immutable
thereafter — an attempt to change it at runtime is an explicit error, never a silent downgrade.
An empty `TrustRoot` is loose mode.

**`inject_mode` tag on dispatch decisions** (`Declared`): extending the existing `EXPLAIN`-able
`Resolution` enum (`Compiled | Interpreted | Miss`) with an inject-mode dimension (e.g.
`Resolution::Compiled { inject_mode: InjectMode }`) so every dispatch decision is inspectable
for both its execution path and its security posture (ADR-006 / G2).

### §4.4 Mapping onto the fast/certified duality

| Axis | `fast` (cert) | `certified` (cert) | `loose` (inject) | `sealed` (inject) |
|---|---|---|---|---|
| Swap certificates | omitted | full | independent | independent |
| Value provenance tags | omitted | full | independent | independent |
| Hot-inject gate | independent | independent | unsigned permitted | InjectCert required |
| Mode tag on results | never silent (G2) | never silent (G2) | G2-tagged on injected calls | G2-tagged on all calls |

The two axes compose freely. No coupling by design. (`Proven` — RFC-0034 §4/§8.)

### §4.5 Threat model and never-silent refusal path

**What the gate closes.** A `sealed` image refuses injection of any compiled `.so` artifact
whose `ContentHash` is not accompanied by a valid `InjectCert` from a trusted `SignerId`. This
closes the attack vector identified in §4.1: a malicious artifact minted by an attacker cannot
be injected without a valid signature from the project's preparation-phase key.

**What the gate does not close** (honest disclosure, G2 / DN-44 §1.1). The gate does not
address Byzantine or adversarial injection from a peer colony in a mesh (RFC-0008 §R8-Q4,
explicitly deferred — see §6 OQ-I7). It does not address a compromised signing key. It does
not address the interpreter fallback path (an interpreted definition executes in the trusted
reference interpreter — ADR-007 — with no `dlopen` and no native code; the signing requirement
is arguably only load-bearing for the compiled path — see §6 OQ-I5).

**Never-silent refusal path.** Analogous to the existing `InjectError::DispatchMiss` pattern:
`InjectError::UnsignedCode(ContentHash)` is the proposed variant — explicit, inspectable,
carrying the exact hash that was rejected. The pattern is already established in the inject path
(ADR-017 decision 5); this is a structural extension consistent with the never-silent
discipline.

### §4.6 Key differentiators from other runtimes

- **Content-addressing dissolves versioning and staleness for free.** An edited definition
  is a new hash; the signed `InjectCert` for the old hash is automatically invalidated without
  any revocation machinery. The content hash **is** the version and the revocation. Structurally
  different from Java's signed JAR (version numbers plus explicit revocation lists).
- **The signature is over the dispatch key itself.** The `InjectCert`'s `content_hash` is
  exactly the dispatch key (ADR-016/017). No secondary identity can drift from the dispatch key.
- **The security gate also asserts lowering auditability.** The VR-4 no-opaque-lowering
  attestation (DN-18/M-630) is carried inside the `InjectCert`. The cert is not just "this came
  from an authorized party" but "this came from an authorized party and the lowering is auditable
  (no black-box pass)" — fusing security and transparency per ADR-006.
- **The gate is independent of the swap-cert mode.** A `fast` runtime can enforce `sealed`
  injection; a `certified` runtime can operate in `loose` mode. The two axes are independently
  configurable (RFC-0034 §8).

---

## §5 Conventions and idioms

These are canonical conventions derived from the transparency/never-silent/provenance ethos,
with their DX and downstream effects.

### §5.1 Never-silent swap at every call site

Every representation change is lexically visible: `swap(x, to: Ternary{6}, policy: rt)`. Both
target and policy are always explicitly named. No silent inference may insert a `swap`.
(`Exact` — RFC-0006 §4.1 S1, RFC-0001 §4.5, RFC-0002 §4, G2, DN-57 §3.)

**DX effect.** The cost and safety of paradigm crossing is auditable at every call site. A
code-review pass can grep for `swap` and see the complete set of representation changes in a
program. A downstream tool can collect all swap certificates from the program's provenance DAG.

### §5.2 Content-addressed identity — names as separable metadata

Definition identity is the α-normalized structure hash (ADR-003) after elaboration. A rename
that does not change elaborated L0 does not change identity (S3). Names are bindings to hashes,
not the hashes themselves. (`Exact` — RFC-0006 §4.1 S3, ADR-003, RFC-0001 §4.6.)

**DX effect.** Safe refactoring, diffing, and the EXPLAIN/policy_used audit story work uniformly
across layers. A refactor that changes the representation of a data structure produces a
genuinely distinct content address, making cross-paradigm compatibility explicit.

### §5.3 Inspectable elaboration — every sugar step is dumpable

Every L2→L1→L0 step must be dumpable and diffable via the stage-dump channel (M-140). No
surface form may hide its desugaring. The query "what did this elaborate to?" always has an
answer. (`Exact` — RFC-0006 §4.1 S4, ADR-006.)

**DX effect.** Developers and tools can audit what code actually does after lowering. This
enables trust in transformations: if a macro or derived form changes behavior, the stage dump
shows it. AI co-authors receive the same signal (FR-S5).

### §5.4 Explicit partiality — Option/Result visibility at every failure site

Out-of-range, illegal pair, unsatisfied constraint: always an explicit `Option`/`Result`/
diagnostic. No surface construct may erase a kernel refusal (S5, G2). (`Exact` — RFC-0006 §4.1
S5, RFC-0013 §4.1 I1.)

**DX effect.** Fallibility is always visible. There are no hidden failure modes behind silent
defaults. A reviewer auditing an API can see every error path structurally.

### §5.5 Per-operation guarantee tags — never aggregate

Each swap, each approximation, each lossy operation carries its own `Exact`/`Proven`/`Empirical`/
`Declared` tag. A function is **not** tagged once; its operations are. Claim composition uses the
meet rule: the weakest always wins. (`Exact` — RFC-0001, CLAUDE.md house-rule 1, RFC-0013 §4.6.)

**DX effect.** Reflects the real cost/safety breakdown of a computation. A reviewer can ask
"which swaps in this function are lossy?" by reading the operation-level tags. Aggregation loss
is prevented by design.

### §5.6 Additive diagnostics — presentation never suppresses error

A policy or diagnostic configuration may describe a refusal more richly (message, tags, level,
route) but can never swallow, soften, or stand in for the explicit `Option`/error that G2
requires. The error still propagates unchanged. (`Exact` — RFC-0013 §4.1, RFC-0013 §4.5 X3,
RFC-0006 §4.1 S5.)

**DX effect.** Diagnostic machinery cannot become a silent black box. Rich error presentation
and transparent error propagation are compatible; suppression is not.

### §5.7 Three-test naming gate — named terms must earn their name

Every candidate term passes T-map (fidelity: does the name accurately map to the behavior?),
T-illuminate (teaching value: does the name teach the semantics?), and T-learn (dual
readability: can both humans and LLMs read it correctly?). No decorative metaphors.
(`Exact` — DN-02 §1, CLAUDE.md Lexicon section.)

**DX effect.** Controls terminology inflation. The unified fungal lexicon (phylum, nodule,
spore, hypha, colony, swap, wild, matured, substrate, fuse, reclaim, tier) is consistent across
IR, RFCs, and surface — one vocabulary, never language-specific aliases. Reduces cognitive load
and LLM-leverage fragmentation.

### §5.8 One name per term — no canonical/alias pairs

Each concept has exactly one name. No per-audience spelling variants. (`Exact` — DN-03 §3.)

**DX effect.** Prevents readability and LLM-leverage fragmentation. Both human readers and
language models encounter one consistent identifier, reducing synthesis error rate.

### §5.9 Reserved-not-active keywords — parse errors, not silent identifiers

`phylum`, `colony`, `mesh`, `graft`, `cyst`, `xloc`, `forage`, `backbone` lex as keywords
(open no construct). Using one as a function name is a parse error with a diagnostic
(`.claude/memory/lang-lexicon-syntax.md §8`, DN-06, `crates/mycelium-l1/src/lib.rs` test
`phylum_and_colony_are_reserved_not_active`). (`Exact`.)

**DX effect.** Future constructs can claim reserved words without breaking old code (parse errors
force migration, never silent shadowing). Teaches the language's direction upfront.

### §5.10 Mandatory `;` terminator — streamable, unambiguous boundary

Every component ends with `;`, including blocks that close with `}`. Missing `;` is a never-
silent `ParseError`. The delimiter triad is: `:` (ascribe) · `,` (separate siblings) · `;`
(terminate component). (`Exact` — DN-57 §3, M-818 Enacted 2026-06-29.)

**DX effect.** Enables streamable, fully whitespace-free parsing and unambiguous recovery.
The boundary is a token, not a newline or absence of tokens. Simplifies the elaborator and the
formatter.

### §5.11 Exhaustive match checking with redundancy — no silent logic bugs

Pattern compilation uses the Maranget usefulness algorithm for both exhaustiveness and
redundancy checking. Neither a missing case nor a dead arm is silently accepted.
(`Proven` — `crates/mycelium-l1/src/usefulness.rs`, RFC-0020 §4.4, RFC-0007 §3.)

**DX effect.** Partiality is visible: a missing case is a compile-time error, not a runtime
surprise. Dead arms are caught, preventing silently unreachable code from masking logic errors.

### §5.12 Immutable value semantics — no aliasing or data-race model

Values are immutable; no aliased mutable state crosses hypha boundaries (RT1). No memory
aliasing issues, use-after-free, or data races in the value model. (`Exact` — RFC-0006 §4.2
LR-9, RFC-0001, RFC-0008 RT1.)

**DX effect.** Simplifies reasoning and removes whole classes of bugs. Referential transparency
is the default. Safe value movement without lifetimes.

### §5.13 Universal-until-elaboration literals — no cross-Repr defaults

A literal (e.g., `0b1011`, `42`, `[1.5, -2.25]`) is unresolved until elaboration assigns
exactly one `Repr`. No defaulting across representation families. Ambiguous literals are explicit
errors, not silent coercions. (`Empirical` — RFC-0006 §8 Q6, `.claude/memory/lang-lexicon-
syntax.md §4.3`.)

**DX effect.** Stricter than Rust's `i32` default. Prevents silent precision loss or paradigm
confusion. Every value has a definite, inspectable representation from the elaboration step
onward.

---

## §6 Open questions for maintainer ratification

These questions are **not** resolved by this note. Each is tagged with the facet it belongs to
and a strength tag reflecting the confidence that the question is correctly formulated.

**OQ-A (types — graded type soundness).** The full noninterference proof for the graded typing
judgment `Γ ⊢ e : τ @ g` (RFC-0018 stage-1a) remains `Declared` — not machine-checked.
RFC-0018 §11 names mechanization as the basis for a future `Proven` upgrade. Until mechanized,
the full soundness of the graded type system stays `Declared`. Is there a near-term feasible
path to a machine-checked proof (e.g., Lean formalization, Liquid Haskell)? (`Declared`.)

**OQ-B (types — SelectionPolicy ergonomics at three sites).** Whether one policy language is
expressive and ergonomic for swap-target selection, packing schedule, and task placement
simultaneously — without becoming unwieldy — is an open empirical question. RFC-0005 is
Accepted direction; the three-site unification is not yet fully enacted; forage and backbone are
R2 reserved-not-active (DN-63 §2). (`Declared`.)

**OQ-C (types — chained approximation composition).** The `ApproximateSource` error path
(RFC-0002, `SwapError::ApproximateSource`) refuses compositions of a non-`Exact` source into a
swap. The E2-1 composition rule for combining an input bound with the swap's ε is explicitly
open (`crates/mycelium-cert` memory §Gotchas). This limits chained approximation pipelines to
exact-source-only. Is a tractable composition rule for bounds definable? (`Declared`.)

**OQ-D (types — three-layer memory model soundness).** Whether the three-layer hybrid memory
model (DN-32: affine Layer 1 / RC Layer 2 / region Layer 3) achieves "stupid easy" ergonomics
with maximum throughput and memory safety simultaneously is `Declared` in DN-32 §Posture
("unbuilt and Declared"). RFC-0027 OQ-1, OQ-4 are still open. (`Declared`.)

**OQ-E (types — substrate/hypha interaction on reclamation).** If a hypha holds a substrate,
what happens when the hypha is reclaimed (reclaim policy) — is the substrate consumed, dormanted,
or escalated? The interaction of LR-8 affine resources with RT7 structured lifetimes and RFC-0014
bounded-effects recovery is not fully specified in the corpus. Deferred to DN-59 and a future R2
RFC for `graft`. (`Declared`.)

**OQ-F (types — VSA multi-hop compositional Proven bounds).** The resonator factorization is at
most `Empirical` (RFC-0009 §5; RFC0003_MATRIX HRR/FHRR rows). Whether there is a tractable
subset of compositional VSA programs admitting `Proven` capacity bounds is an open research
question. (`Declared`.)

**OQ-G (sugar — guard clauses and guarantee propagation).** Should `when condition` guard clauses
on patterns be ratified? If so, does the guarantee tag of the guard weaken the arm's tag — if the
guard is `Declared`, does the arm become `Declared`? RFC-0020 §4.1 S2 does not yet specify guards.
(`Declared`.)

**OQ-H (sugar — record-literal shorthand shadowing rules).** `{x, y}` elaborating to `{x: x, y: y}`
requires deterministic shadowing: which `x` and `y` are in scope? If both a local binding and
a record field share the name, the disambiguation rule must be explicit and never silent. Is the
readability win worth the shadowing rules? (`Declared`.)

**OQ-I (sugar — short paradigm keyword scope).** Should `bin{N}`, `tern{N}`, `emb{D,S}`,
`hvec{E,D,Sp}` bind only to type literals, or also to trait methods/associated types? (RFC-0037
D2-b, DN-31 §2.) The rationale for limiting scope to type literals: binding to methods would
extend the surface of the short-form to method resolution, complicating the "one canonical form"
discipline (DN-03 §3). (`Declared`.)

**OQ-J (sugar — annotation burden in practice).** What is the practical annotation burden of
graded types in a medium-sized application program? RFC-0018 §5 notes that stage 1a (monomorphic
grades) requires explicit grade annotations on every function signature. Stage 1b (grade
polymorphism, inference over the 4-chain) is deferred. Until 1b lands, is the annotation burden a
usability barrier? The T3.6 rigorous ablation (DN-09 §4 open follow-up — Mycelium surface
fragment vs Python-embedded DSL, with and without semantic feedback) was not run. (`Declared`.)

**OQ-K (hot-inject — signing authority and key management).** Who signs an `InjectCert`? A
project-level key in `mycelium-proj.toml`? A phylum-level key? A colony-bootstrap key injected
at germination? The key management story is entirely undesigned. This is likely the hardest part
of the inject-security design and requires its own RFC and research pass (RFC-0008 §R8-Q4 for the
adversarial-mesh analogue). (`Declared`.)

**OQ-L (hot-inject — replay and expiry).** Should an `InjectCert` have a monotonic counter or
expiry to prevent replay attacks (injecting an old cert for a definition later superseded)?
Content-addressing provides some protection but not against injecting a known-vulnerable version.
The corpus has no existing revocation or cert-expiry machinery. (`Declared`.)

**OQ-M (hot-inject — scoping hierarchy for inject-mode).** RFC-0034 §6 resolves the cert mode
at global/phylum/nodule scope. Should the inject-mode knob reuse the same `@certification`
scoping, or live at the image/colony/runtime level? The right scope for a security gate (runtime
policy, not per-nodule authoring choice) is unclear. (`Declared`.)

**OQ-N (hot-inject — relation to spore signing in ADR-013).** ADR-013 names "signatures" as
component 4 of the spore DAG but defers the story. Should the `InjectCert` be the spore's
signature component, or a separate artifact? If they are the same, `myc-prepare` produces a
signed spore that is both the deployable unit and the inject gate — the cleanest design.
(`Declared`.)

**OQ-O (hot-inject — interpreter fallback and sealed mode).** In the current `Image`, a hash
with no compiled entry falls back to the interpreter. In `sealed` mode, should interpreted
definitions also require a cert? The threat model differs: an interpreted definition executes in
the trusted reference interpreter (ADR-007, memory-safe, no `unsafe`); a compiled `dlopen`
artifact executes arbitrary native code. The signing requirement is arguably only load-bearing
for the compiled path. (`Declared`.)

**OQ-P (hot-inject — naming alignment with fungal lexicon).** The candidate mode names `loose`
and `sealed` are functional but not on-brand with the fungal lexicon. Alternatives grounded in
the lexicon: `spore-sealed` (the image only accepts spore-signed inject); `inoculated`
(biological term for introducing a verified organism). This is a naming question for the
maintainer to ratify per the three-test gate (DN-02 §1). (`Declared`.)

**OQ-Q (hot-inject — cross-colony injection in a mesh).** If a colony receives a hot-inject from
a peer colony in a mesh, what is the trust model? Does the receiving colony verify the
`InjectCert` against its own `TrustRoot`, or does it inherit the sending colony's trust? This is
the distributed extension of OQ-K and is explicitly undesigned in the corpus (RFC-0008 §R8-Q4).
(`Declared`.)

**OQ-R (conventions — composite operation guarantee aggregation).** What is the exact scope of
the per-operation guarantee tag in composite operations (a function that calls multiple
primitives)? The corpus assumes fine-grained per-swap tagging but composite aggregation — and
whether there is a function-level summary beyond the meet of its operations — is not fully
specified. (`Declared`.)

**OQ-S (conventions — guarantee propagation through generic instantiation).** If a generic
function is monomorphized at two different guarantee levels, does each instantiation carry its
own tag context? (RFC-0019 §4.4 monomorphization; RFC-0018 §4 graded typing.) (`Declared`.)

**OQ-T (conventions — three-test gate at proposal time).** The three-test gate (T-map/T-illuminate/
T-learn) is applied to ratified terms (DN-02 §7). What is the process for candidate terms during
the design phase — is the gate applied at proposal time or only at ratification? And does the
"one name per term" rule (DN-03 §3) apply to library-defined names (can a phylum export both
`add` and `plus` as aliases)? (`Declared`.)

---

## §7 Maintainer dispositions (2026-06-29)

Append-only resolution of the §6 open questions. The maintainer ruled on 19 of the 20 OQs
(**OQ-H is left explicitly open** — not silently resolved, G2). Each disposition is recorded at
the strength the maintainer set it to; **none is upgraded past its basis** (VR-5). A disposition of
**R&D-commission** means the question is *answered as "research it"* — the decision is to investigate,
not a design commitment; the resulting findings stay `Declared` until a checked basis exists. A
disposition of **Direct** means a design direction is now chosen (still `Declared` until enacted via
its feeding ADR/RFC). Nothing in this section is `Enacted`; this note remains **Draft**. The
maintainer's standing rule binds the whole table: *"unless it can be mechanically proven, it must drop
to `Declared`"* (OQ-G generalized — VR-5).

Tracking ids (`M-827`…`M-845`) are minted here and registered in `tools/github/issues.yaml`; the
deeper artifacts they commission are: the **VSA compositional-bounds GPU experiment** (`experiments/
mycelium_experiments/vsa_bounds/`, OQ-F), **RFC-0038 — the inject-mode security axis** (the hot-inject
cluster OQ-K…OQ-Q), and two R&D records (`research/26-…`, `research/27-…`).

| OQ | Facet | Maintainer decision (faithful) | Disposition | Feeds |
|---|---|---|---|---|
| **A** | graded type soundness | **Yes — R&D and enact** the machine-checked noninterference proof for `Γ ⊢ e : τ @ g`. | Direct + R&D-commission (M-827) | RFC-0018 §11; `research/26` |
| **B** | SelectionPolicy / R2 vocab | **`forage` and `backbone` must be made *active*.** Selection-policy + swap machinery must support **mechanized capture & setting** to improve ergonomics **while retaining transparency, provenance, explainability**. | Direct + R&D-commission (M-828) | RFC-0005, RFC-0008 RT3, DN-63; `research/27` |
| **C** | chained approximation | **Define a tractable composition rule for bounds** (the E2-1 input-bound ⊕ swap-ε rule). | R&D-commission (M-829) | RFC-0002 `SwapError::ApproximateSource`; `research/26` |
| **D** | three-layer memory model | **Build to ensure & guarantee the "stupid easy" ergonomics** (affine / RC / region). | Direct + R&D-commission (M-830) | DN-32, RFC-0027; `research/26` |
| **E** | substrate/hypha reclaim | **Investigate, R&D and plan** the reclaim interaction (LR-8 affine × RT7 lifetime × RFC-0014 recovery). | R&D-commission (M-831) | DN-59, future `graft` R2 RFC; `research/26` |
| **F** | VSA multi-hop `Proven` | **R&D on GPU** (maintainer's desktop, tonight). Build runnable experiments to map the tractable `Proven`-bound subset and feed insight. | R&D-commission + **build experiment** (M-832) | `experiments/mycelium_experiments/vsa_bounds/`; RFC-0009 |
| **G** | guard clauses | **Yes, ratify guards.** A guard's tag propagates to the arm: **unless mechanically proven, the arm drops to `Declared`** (VR-5). | Direct (M-833) | RFC-0020 §4.1 S2; `research/27` |
| **H** | record-literal shadowing | *(no decision — left open, G2.)* | **Open** | — |
| **I** | short-keyword scope | **Split trait methods and associated types** for clarity / **one-canonical-form** discipline; ergonomics is key. Short keywords (`bin{N}` …) **bind to type literals only**, not to methods/associated types. | Direct (M-834) | RFC-0037 D2-b, DN-31, DN-03 §3; `research/27` |
| **J** | annotation burden | For the stringent doctrine, provide **wrappers/decorators and/or tooling/ergonomic implementations** that ease use **without degrading** the guarantees. | Direct + R&D-commission (M-835) | RFC-0018 §5; `research/27` |
| **K** | inject signing authority | **Research.** Instinct: the signing scope **depends on what the dev is building** (script / nodule / library / application / other) — **scoped to their project, graded and dev-configurable by scope of work.** | Direct + R&D-commission (M-836) | RFC-0038 §K; RFC-0008 §R8-Q4 |
| **L** | InjectCert replay/expiry | **R&D.** | R&D-commission (M-837) | RFC-0038 §L |
| **M** | inject-mode scoping | **R&D.** | R&D-commission (M-838) | RFC-0038 §M |
| **N** | relation to spore signing | **The same** — the `InjectCert` **is** the spore's signature component (ADR-013 §2 comp. 4). `myc-prepare` produces a **signed spore** that is both the deployable unit **and** the inject gate (the note's "cleanest design"). | Direct (M-839) | RFC-0038 §N; ADR-013 |
| **O** | interpreter fallback in sealed | **In `inoculated` mode, yes** — interpreted definitions **also require a cert**. `inoculated` is the **secured, strictly-enforced production tier**. | Direct (M-840) | RFC-0038 §O |
| **P** | mode naming | **`inoculated` replaces `sealed`** (production tier). `loose` retained (local-dev). All forward references read `sealed` → **`inoculated`**; §4 above is preserved as the commissioning draft (append-only). | Direct (M-841) | RFC-0038 (naming); DN-02 §1 three-test gate |
| **Q** | cross-colony inject (mesh) | The receiving colony **verifies the `InjectCert` is valid, trusted, and not expired or superseded by the trusted signer** (verify against its **own** trust, never inherit the sender's). | Direct + R&D-commission (M-842) | RFC-0038 §Q; RFC-0008 §R8-Q4 |
| **R** | composite-op aggregation | **R&D.** | R&D-commission (M-843) | RFC-0001; `research/27` |
| **S** | grade through monomorphization | **Yes** — each monomorphized instantiation **carries its own guarantee-tag context.** | Direct (M-844) | RFC-0019 §4.4, RFC-0018 §4; `research/26` |
| **T** | proposal-time naming gate | **Apply the same three-test gate** (T-map / T-illuminate / T-learn) at **proposal time**, not only at ratification. | Direct (M-845) | DN-02 §7, DN-03 §3; `research/27` |

**Naming note (OQ-P, ratified).** The production inject mode is **`inoculated`** — the biological term
for introducing a *verified* organism into a substrate, on-brand with the fungal lexicon and earning
its three-test gate (T-map: a sealed/verified admission; T-illuminate: teaches "only verified code is
admitted"; T-learn: human- and LLM-legible). `loose` (local-dev, unsigned permitted, G2-tagged) is
retained. Wherever §4 / OQ-P above wrote `sealed`, read **`inoculated`**; RFC-0038 carries the
ratified naming forward.

**Hot-inject cluster (OQ-K…OQ-Q) → RFC-0038.** The firm directions (N: InjectCert = spore signature;
O: interpreted path also gated in `inoculated`; P: `inoculated`/`loose`; Q: verify valid/trusted/
unexpired/unsuperseded against own trust; K: project-scoped, graded, dev-configurable signing) are
captured as the ratified core of **RFC-0038 (Draft/Proposed)**; the still-`R&D` sub-parts (K key-
management detail, L replay/expiry mechanism, M inject-mode scoping) are named there as explicit open
items, not silently closed.

---

## Changelog

| Date | Change |
|---|---|
| 2026-06-29 | Initial draft — five-facet synthesis exploration note commissioned and drafted. |
| 2026-06-29 | §7 — maintainer dispositions on 19/20 OQs (OQ-H left open); `sealed`→`inoculated` ratified; hot-inject cluster routed to RFC-0038; M-827…M-845 minted. Append-only; note stays Draft. |
