# Kickoff `enb` — Phase-I H1: enabler-gap closure (the critical path to usability)

> **UID:** `enb` · **Basis:** **ADR-038** (Proposed) §2.2/§2.6 + the umbrella roadmap
> `docs/planning/road-to-1.0.0-and-mycelium-rewrite.md` **§3 (H1)** ·
> `docs/spec/stdlib/self-hosting-readiness.md` **§0** (the 2026-07-01 below-grammar gap list,
> `Exact`/`Empirical` per item) · RFC-0033 §4.1.2–§4.1.3 + its 2026-07-01 pull-forward note
> (M-766/M-767) · ADR-028 (signedness-as-operations) · DN-39 (kernel-promotion bar) · DN-63 (R2
> vocabulary) · DN-54 §10 (derive-site consume design-pass, M-824).
> **Planned by:** Fable (ADR-038 §2.7); **implemented by:** Sonnet/Haiku leaves per the PM table.
> **References the doc-maintenance contract** (`_doc-maintenance.md`) in its DoD.
> **Sequencing: after `acy` (H0)** — every task here lands under the cycle-enforcement gate.

## Goal (roadmap §3)

Close the **below-grammar functional-usability enablers** so a Mycelium program can exercise the
whole ratified surface + stdlib end-to-end: the serial prim lane **B → C → A(route-ii float) →
E(Substrate)**, the parallel tracks **D-lite (forage/backbone subset of M-828)**, **`myc run`**,
**textual string literal**, and **`hash.*` surfacing** — each never-silent, honestly tagged,
conformance-tested (accept + reject), with a property test per bound. This kickoff IS Phase I's
critical path: its DoD is the bulk of ADR-038's Phase-I usability gate.

## Scope

**In:** the H1 gap set exactly as the roadmap §3 fixes it (labels A–E + the three small items),
plus one lightweight **research-start** task for the D-heavy mesh/xloc/cyst track (spec the research,
NOT the build — it is a separate, parallel, long-lead lane).

**Out:** the full R2 maturity wave (M-828 remainder — H2); the V-wave remainder (ADR-030/031 doors,
M-758/M-759 — Phase II; only the RFC-0033-noted pull-forwards land here); any content-address
**rehash** (the float `Repr`'s identity impact is *coordinated*, and the rehash itself **defers to
the first value-persistence feature** — RFC-0033 §7 / M-780 discipline, ADR-038 §2.6); H2 closeout
lanes; mass `.myc` porting (gated by `grm`).

## Swarm method + model tiering (ADR-038 §2.7)

**Hybrid-tiered Sonnet swarm**: Sonnet orchestrator; Sonnet leaves for semantics-bearing tasks,
Haiku leaves for mechanical registry/conformance tasks (per the Model column). **Collision law:**
the prim lane (B → C → A → E) is **serial on `crates/mycelium-interp/src/prims.rs` + the
`mycelium-l1` frontend** (`lexer/parse/checkty/elab/mono` — the repo's one known serial lane;
kickoffs README §Parallelization) — run those leaves one-at-a-time, land + pull down before the
next. The parallel tracks (`myc run` in `mycelium-cli`, `hash.*`, mesh research doc) are disjoint
by directory and run concurrently. **The string-literal tasks touch the L1 lexer — they join the
serial lane**, they are "parallel" only in *dependency* terms. One isolated worktree per leaf
(mitigation #11); commit/push split (#12); scoped PRs to `dev` via `/pr-land`.

## Ordering

```
(early, parallel)  M-908/M-909 myc run · M-912 hash.* · M-913 mesh-research start
serial prim lane:  B (M-887→M-889, M-766, M-767) → C (M-890→M-894) → A (M-896→M-900, after M-895 ratifies)
                   → E (M-902→M-904, after M-901 confirms) ;  M-910/M-911 strings slot into the lane early
(parallel design)  M-895 float ADR + DN-39 dossier — drafted from kickoff, ratified while B/C land
(parallel track)   D-lite: M-905 scope split → M-906 forage → M-907 backbone-verify
(capstone)         M-914 integration demo + readiness §0 re-verify
```

## PM decomposition — bite-sized tasks

Proposed M-ids: **M-887…M-914 new** (next-free after M-876; re-verify at minting — mitigation #1);
**M-766/M-767 are the RFC-0033-named pull-forward slots** (unminted today — verified absent from
`issues.yaml`; mint under those names to keep the corpus cross-refs true, or FLAG if taken). None
minted by this doc.

### Gap B — binary integer arithmetic + signed-op set (serial lane, first)

| M-id (proposed) | Task | User story | Definition of Done | Model | depends_on |
|---|---|---|---|---|---|
| M-887 | `bin.mul` prim — two's-complement multiply, **never-silent overflow** (explicit `Option`/error, RFC-0033 §4.1.2) | As a stdlib author, I want integer multiply in-language, so that `math`'s integer half is expressible in `.myc` | Prim registered + typed; overflow → explicit error (no wrap-by-default); property test on the overflow bound; conformance accept + reject; three-way where forms close | Sonnet | acy (H0) |
| M-888 | `bin.div` + `bin.rem` prims — division with **explicit div-by-zero** error (unsigned semantics first; signed variants ride M-767) | As a stdlib author, I want integer division that fails loudly on zero, so that no program divides silently | Prims registered; div-by-zero → explicit error; property test (`div`/`rem` Euclidean identity on the domain); conformance accept + reject | Sonnet | M-887 (registry pattern) |
| M-889 | `bin.shl` + `bin.shr` prims — shifts with **explicit out-of-range shift-amount** handling | As a stdlib author, I want shifts with defined bounds, so that bit-level code is portable and never-silent | Prims registered; shift-amount ≥ width → explicit error (not UB/wrap); property test on the bound; conformance accept + reject | Haiku | M-887 |
| M-766 (RFC-0033 name) | Two's-complement **shared-op completion**: `neg` + overflow-detect ops; reconcile against the kpr-landed `add`/`sub` (verify the current registry FIRST — don't re-land what E19-1 shipped) | As a stdlib author, I want the full shared two's-complement set (§4.1.2), so that signed-magnitude workarounds are never needed | Verified inventory of landed vs missing ops recorded (`Empirical`); missing ops landed with overflow-detect; property tests; conformance | Sonnet | M-887 |
| M-767 (RFC-0033 name) | **Signedness-split op set**: signed `div`/`cmp`/`shift` + overflow-detect (§4.1.3; ADR-028 signedness-as-operations — signedness lives in the *op*, not the type) | As a stdlib author, I want signed comparisons and division as distinct ops, so that `numerics` can express signed algorithms honestly | Signed op set registered; sign-sensitive property tests (e.g. `sdiv` rounding, `scmp` total order on the domain); conformance accept + reject; ADR-028 cited in the op docs | Sonnet | M-766, M-888 |

### Gap C — dense/VSA op-prims surfaced to L1 (types/literals exist; registry entries don't)

Grounded in the kernel functions that already exist (`mycelium-dense`, `mycelium-vsa` — `Exact`):

| M-id (proposed) | Task | User story | Definition of Done | Model | depends_on |
|---|---|---|---|---|---|
| M-890 | Surface the **dense elementwise group**: `dense.add`/`dense.sub`/`dense.neg`/`dense.scale` (kernel: `add_values`/`sub_values`/`neg_value`/`scale_value`) — sets the tensor-valued-prim registry pattern | As a stdlib author, I want elementwise dense ops callable from `.myc`, so that the `dense` module is expressible | Prims registered with honest per-op tags carried from `op_guarantee`; shape-mismatch → explicit error; conformance accept + reject; three-way where forms close | Sonnet | Gap B done (serial lane) |
| M-891 | Surface `dense.dot` + `dense.similarity` — with the kernel's `op_guarantee`/`op_rel_eps` metadata flowing into the per-op tag (EXPLAIN-able) | As a certified-mode user, I want dot/similarity with their real error bounds attached, so that accuracy claims are inspectable, never asserted | Prims registered; tag = the kernel's guarantee (no upgrade — VR-5); EXPLAIN shows the bound; conformance | Haiku | M-890 |
| M-892 | Surface the **VSA bind group**: `vsa.bind`/`vsa.unbind`/`vsa.permute` (model-dispatched: MAP-I/FHRR/BSC per the kernel's `VsaModel`) | As a stdlib author, I want core VSA algebra in-language, so that the `vsa` module is expressible | Prims registered across the model set; model mismatch → explicit error; per-model conformance cases; tags honest per model | Sonnet | M-890 |
| M-893 | Surface `vsa.bundle` (superpose — certified path via `bundle_values_certified`) | As a certified-mode user, I want bundling with its capacity-honest tag, so that superposition claims carry their basis | Prim registered; certified path preserves the kernel's tag + bound; conformance; property test vs capacity bound where cheap | Haiku | M-892 |
| M-894 | Surface **VSA cleanup + reconstruction**: `vsa.cleanup` (cleanup memory) + `vsa.reconstruct` (`reconstruct_role`/`reconstruct_factors`), with `required_dim`/`proven_capacity_bound` surfaced | As a stdlib author, I want cleanup/reconstruction callable with their capacity bounds visible, so that recovery guarantees are never black-boxed | Prims registered; capacity-bound query surfaced; below-capacity property test; conformance accept + reject | Sonnet | M-892 |

### Gap A — scalar-float value form, route (ii) (ADR-038 §2.6; design-gated)

**Gate:** M-895's ADR must be maintainer-**Accepted** and the DN-39 review passed before
M-896…M-900 land (kernel entry is earned — default-DENY). Draft M-895 at kickoff so ratification
overlaps the B/C lane.

| M-id (proposed) | Task | User story | Definition of Done | Model | depends_on |
|---|---|---|---|---|---|
| M-895 | **Float ADR draft + DN-39 promotion dossier** — width set, NaN/rounding semantics, never-silent boundaries; content-address impact coordinated with the deferred ADR-030/031 one-way doors (single rehash, deferred to first value-persistence — RFC-0033 §7) | As the maintainer, I want the float design + kernel-entry case laid out for ratification, so that the trusted base grows only through the DN-39 bar | ADR drafted (Proposed — **maintainer ratifies; not self-ratified**); DN-39 four-clause dossier complete; rehash-coordination section explicit; FLAGged decision points listed | Sonnet | — (design; parallel with B/C) |
| M-896 | Scalar-float **`Repr` variant + value form** in `mycelium-core` (per the Accepted ADR); identity impact *documented, not spent* (no rehash) | As a language user, I want floats to be first-class values, so that `math`'s f64 half stops being inexpressible | Repr + value form landed per ADR; content-address note verifies **no rehash occurred**; kernel tests green; KC-3 delta reviewed | Sonnet | M-895 **Accepted + DN-39 passed** (maintainer gate) |
| M-897 | **Float literal**: lex + parse + typecheck in the L1 surface | As a stdlib author, I want to write `1.5` in `.myc`, so that float code is authorable, not just representable | Literal lexed/parsed/typed; malformed literals → explicit diagnostics; grammar (`mycelium.ebnf`) updated; conformance accept + reject | Sonnet | M-896 (serial lane) |
| M-898 | **Float arithmetic prims**: `flt.add`/`sub`/`mul`/`div`/`neg` — rounding per the ADR; tags honest (`Empirical` where libm-dependent — M-541 audit pending) | As a stdlib author, I want float arithmetic with honest accuracy tags, so that `numerics` claims trace to a basis | Prims registered; per-op tags at the supportable strength (no `Proven` without checked side-conditions — VR-5); NaN/inf behavior per ADR, never-silent; conformance | Sonnet | M-897 |
| M-899 | **Float comparison/total-order prims** (NaN handling per ADR; ties to M-511 — total-order proof debt becomes load-bearing, tag stays `Empirical` until proven) | As a stdlib author, I want float ordering with explicit NaN semantics, so that sorting floats is defined behavior | Cmp prims registered; NaN cases in conformance (accept + reject); M-511 cross-referenced, tag honest | Haiku | M-898 |
| M-900 | Float **three-way conformance closure** (L1-eval ≡ L0-interp ≡ AOT where forms close) + readiness §0 blocker-1 re-verified | As the Phase-I gate, I want the float gap checked off with evidence, so that the usability DoD row is `Empirical`, not `Declared` | Three-way suite green (or AOT refusal explicitly recorded as a decision); readiness §0 item 1 re-verified + recorded | Haiku | M-898, M-899 |

### Gap E — `Substrate`/`consume` execution (staged `Residual` → real execution)

| M-id (proposed) | Task | User story | Definition of Done | Model | depends_on |
|---|---|---|---|---|---|
| M-901 | **Confirm the execution model**: the DN-54 §10 derive-site `consume` design-pass (M-824) as the target — a short confirm-memo for the maintainer (confirm, THEN implement; readiness §0 item 5 notes no execution issue exists) | As the maintainer, I want the consume-execution model confirmed before code, so that the affine semantics aren't improvised mid-implementation | Memo with the model, alternatives, and a recommendation; maintainer sign-off recorded; FLAGs listed | Sonnet | — (design; parallel with A) |
| M-902 | **`Substrate` v0 value form** — the value-model plumbing (creation, passage, inspection; never-silent errors) | As a stdlib author, I want `Substrate` values to exist at runtime, so that `fs`/`io`'s handle model has something to hold | Value form lands; invalid states unrepresentable or explicit errors; kernel/interp tests green | Sonnet | M-901 (confirmed model) |
| M-903 | **Affine tracker** — `consume` checked semantics (use-once enforcement; double-consume/leak → explicit diagnostics) | As a language user, I want double-use of a consumed handle to be a compile/runtime error with a clear message, so that resource bugs are impossible to write silently | Tracker enforces use-once; property test (no path consumes twice undetected); reject-case conformance; diagnostics name the violation site | Sonnet | M-902 |
| M-904 | **Lift the staged `Residual`** — `Substrate`/`consume` elaborate to *executing* forms in the interpreter; conformance accept + reject | As a stdlib author, I want `fs`/`io`'s affine-handle model to actually run, so that those modules become portable | The elab `Residual` for this fragment is gone (or explicitly narrowed + recorded); end-to-end accept cases run; reject cases still refuse cleanly | Sonnet | M-903 |

### D-lite — R2 runtime-vocabulary usability subset (parallel track; M-828 subset)

| M-id (proposed) | Task | User story | Definition of Done | Model | depends_on |
|---|---|---|---|---|---|
| M-905 | **Scope-split M-828**: define the D-lite subset (`forage`/`backbone` per the roadmap; what exactly `std.runtime`'s *usable surface* needs per DN-63) vs the H2 remainder — **FLAG: maintainer signs the split** | As the maintainer, I want the usability subset explicitly split from the full R2 wave, so that H1 stays bounded and honest | Split memo (in-scope rows vs deferred rows, each grounded in DN-63); M-828 body updated with the split; maintainer sign-off | Sonnet | — |
| M-906 | Activate **`forage`** per the D-lite subset (lex → parse → elab → interp; DN-63 semantics) | As a language user, I want `forage` working end-to-end, so that runtime-vocabulary programs execute rather than refuse | Construct executes three-way where forms close; never-silent on the deferred parts (explicit residual + tracker); conformance accept + reject | Sonnet | M-905; serial lane slot (L1 frontend) |
| M-907 | **Verify `backbone`'s landed state** vs the D-lite subset (M-825 landed a backbone construct 2026-06-29 — verify against DN-63/the split, close residual gaps ONLY; don't re-land) | As the Phase-I gate, I want backbone's D-lite row checked with evidence, so that landed work isn't redone and gaps aren't assumed closed | Verified inventory (landed vs subset) recorded (`Empirical`); residual gaps closed or FLAGged; conformance current | Haiku | M-905 |

### Small items + research start (parallel; `myc run` is first-among-equals)

| M-id (proposed) | Task | User story | Definition of Done | Model | depends_on |
|---|---|---|---|---|---|
| M-908 | **`myc run` v0** — lift the honest CLI refusal (`mycelium-cli/src/lib.rs` `run()`): single-nodule project → interpreter, end-to-end | As a language user, I want `myc run` to execute my program, so that Mycelium is a language I can use, not a corpus I can read | `myc run` executes a single-nodule project through the interpreter; failures are explicit `Report`s; CLI tests green | Sonnet | — (parallel; `mycelium-cli` is disjoint) |
| M-909 | **`myc run` multi-nodule** — manifest-driven project loading, nodule linking, end-to-end demo | As a language user, I want multi-nodule projects to run, so that real programs (not toys) execute through the shipped toolchain | A real multi-nodule project runs via `myc run` (committed as a fixture); link errors explicit; CLI tests green | Sonnet | M-908 |
| M-910 | **Textual string literal — lex + grammar** (`"…"` with an explicit, minimal escape set; ergonomic, not expressive — only `0x…` `BytesLit` exists today) | As a stdlib author, I want to write string literals, so that authoring `text`/`fmt`/`diag`/`error` in `.myc` stops requiring hex bytes | Literal lexed/parsed; escape errors explicit; `mycelium.ebnf` updated; **serial-lane slot** (L1 lexer) | Sonnet | serial lane slot |
| M-911 | **String-literal lowering + conformance** — lower to the existing Bytes/text value form; accept + reject cases | As a stdlib author, I want the literal to mean the same bytes everywhere, so that string behavior is deterministic three-way | Lowering lands (no new L0 node — KC-3); three-way conformance; reject cases (bad escapes) refuse cleanly | Haiku | M-910 |
| M-912 | Surface **`hash.*`** — blake3 (already in `mycelium-core`, `id.rs`/`content.rs`) as prims | As a stdlib author, I want content hashing in-language, so that the `content` module is expressible | `hash.*` prim(s) registered; `Exact` tag justified by the kernel's own use; conformance vector tests (known digests) | Haiku | Gap B landed (registry pattern; else M-887's pattern) |
| M-913 | **Start the mesh research track** (D-heavy: `mesh`/`xloc`/`cyst`) — a research note: problem statement, prior art survey plan, open questions, DN skeleton. **Spec the research, NOT the build** — separate, parallel, long-lead lane | As the Phase-II runtime effort, I want the distributed-vocabulary research started now, so that its long lead time overlaps Phase I instead of following it | Research note committed (`research/` or DN — Draft, `Declared` posture); explicitly non-gating; scoped questions listed; no implementation | Sonnet | — |

### Capstone

| M-id (proposed) | Task | User story | Definition of Done | Model | depends_on |
|---|---|---|---|---|---|
| M-914 | **H1 integration demo + readiness re-verify** — a committed Mycelium program exercising integers (signed ops), floats, strings, dense/vsa ops, fs/io (Substrate), and the R2-lite surface via `myc run`; re-run the readiness §0 verification and record the result | As the maintainer, I want the H1 DoD demonstrated by one running program plus a re-verified blocker list, so that "usable" is checked (`Empirical`), not declared | Demo program committed + running via `myc run` (three-way-clean where forms close); readiness §0 blocker list re-verified empty or explicitly reduced, recorded append-only | Sonnet | all lanes above |

## Definition of Done (kickoff)

- Each gap A–E closed (or explicitly maintainer-deferred with rationale — G2); the three small
  items landed; M-914's demo runs; readiness §0 re-verified and recorded (`Empirical`).
- Every op never-silent, honestly tagged (VR-5 — no tag above its checked basis; float/libm items
  stay `Empirical` until M-511/M-541 close), property test per bound, conformance accept + reject.
- No content-address rehash spent (RFC-0033 §7 verified in M-896's DoD).
- Doc-maintenance per `_doc-maintenance.md`; issues minted at kickoff (slots re-verified), statuses
  current; grammar + api-index regenerated at integration tier.

## Prerequisites

1. **`acy` (H0) landed** — the enforcement gate this wave lands under.
2. **ADR-038 ratification** (Proposed → Accepted) — the strategy this wave executes; H1 work that
   is also justified under prior ratified decisions (e.g. the RFC-0033 pull-forward note) may start,
   but the *wave as a whole* is ADR-038's Phase-I critical path — confirm with the maintainer.
3. **Maintainer gates inside the wave** (FLAG, never guess): float ADR + DN-39 outcome (M-895 →
   M-896); the consume-model confirmation (M-901); the M-828 D-lite split sign-off (M-905).
