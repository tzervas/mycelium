# Road to 1.0.0 and the Mycelium Rewrite — the Function-First Umbrella Roadmap

| Field | Value |
|---|---|
| **Status** | **Advisory / Proposed for maintainer review** (revised 2026-07-01). Planning posture, like the other `docs/planning/*.md` — this doc **decides nothing**; it sequences the ratified + proposed decisions and flags every maintainer decision it depends on. `Declared` wherever it forecasts effort or ordering. **This is the revised form of the Rust-reference-completion + acyclic-deps closeout plan** (kickoff UID `rcp`; see the meta-changelog — the prior plan's content is absorbed, not lost). |
| **Governing decision** | **ADR-038** (Proposed — *Pragmatic Dogfooding: the Function-First Release Strategy*): North Star *"Rust where appropriate, Mycelium everywhere else"*; **Phase I → reach functional usability, then go public at a `0.x`** (the public release is gated on usability and is **version-independent** — ADR-038 §2.8; **not** at `1.0.0`); **Phase II → post-public progressive Mycelium rewrite**, the public semver climbing `0.x → 1.0.0` in the open, where **`1.0.0` ≡ "fully rewritten into Mycelium (where appropriate) and 100% operational."** Within Phase I the functional-completeness gate stays ADR-022 (as amended by ADR-024/034/035); ADR-036 §2.1–§2.3 govern the dogfooding/validation model throughout. **For now the version stays `0.0.0`; the semver scheme is deferred until publish-time** (ADR-038 §2.8, FLAG-V1/V2). |
| **Goal** | **Phase I:** a **fully functional, usable** language — a Mycelium *program* can exercise the whole ratified surface + stdlib end-to-end (`myc run`) — on a clean, structurally-enforced **acyclic** crate graph; then **go public at whatever `0.x` version fits** (DN-27 decomposition + per-repo GHCR). **Phase II:** the progressive rewrite of the remaining corpus in Mycelium, in the open, advancing the public semver toward **`1.0.0` (= fully rewritten + 100% operational)**, with compiler self-hosting a conditional, evidence-gated aspiration. |
| **Basis** | ADR-038 (Proposed) · ADR-022 (tracks T1–T9 as amended) · ADR-035/ADR-036 (gates) · RFC-0031 §5 (composition + boundary; D1 permanence lifted, 2026-07-01 note) · RFC-0033 §7 (dogfood-gate rehash discipline) · `docs/spec/stdlib/self-hosting-readiness.md` §0 (the 2026-07-01 surface-sufficiency verification) · DN-34 §8 / M-873 (transpiler PoC data) · the three Fable-5 research digs, 2026-07-01 (dep-graph / roadmap / open-work surveys, `Empirical`) |

**The reframe (2026-07-01).** The prior form of this plan ("Rust reference complete, acyclic deps,
*then* self-hosting") sequenced the phases by implementation language. ADR-038 re-sequences them by
**function**: Phase I lands whatever makes the language *usable* — in Rust where appropriate, in
Mycelium where the surface is ready — and the public release follows usability, not Rust-replacement.
Two prior findings make this cheap rather than aspirational: the Rust reference is **largely complete**
(zero `todo!`/`unimplemented!` in non-test source; every gap an explicit never-silent `Residual` with a
tracker item — `Empirical`, open-work survey), and the language **surface is already sufficient** for
the structural majority of the stdlib (readiness §0: ~19/26 crates expressible, 8 nodules executing
three-way) — the real blockers are a short list of **below-grammar enablers** (§3/H1). So Phase I is a
**bounded closeout plus a bounded enabler wave**, not a green-field phase.

---

## 1. Phase map and exit criteria

```text
PHASE I  (private · 0.0.0 · publish=false)          PHASE II  (PUBLIC · progressive · semver 0.x → 1.0.0)
──────────────────────────────────────────          ────────────────────────────────────────────────
H0  foundations: acyclic-deps enforcement,           mass .myc porting (rest of corpus)
    workspace hygiene                                transpiler-accelerated (polished)
H1  functional-usability enablers                    Rust replaced module-by-module, IN THE OPEN
    (B→C→A→E→D-lite, myc run, strings, hash)         (ADR-036 §2.3, unchanged); each replacement
H2  Rust-reference closeout remainder                 advances the public semver
    (l1 semantics · value/AOT tail · runtime ·       V-wave remainder (one rehash, at first
     toolchain · inject-mode · kernel freeze)          value-persistence)
H2a grammar-stability gate (before mass porting)     compiler self-hosting: ONLY IF stability/perf-
opportunistic .myc ports (ready-now crates)           proven, post-transpiler-polish (ADR-038 §2.3)
    ↓
functional-usability gate (ADR-022 completeness + ADR-038 DoD)  →  maintainer usability check
    →  PUBLIC FLIP at a 0.x  (DN-27 decomposition · per-repo GHCR · public version decided here)
    →  … progressive rewrite in public …  →  semver 1.0.0 = fully rewritten + 100% operational
```

**Note (§2.8 / FLAG-V1):** "functional-usability gate" is ADR-022's full-language completeness
milestone (historically labelled `lang 1.0.0` — a release *event*, ADR-023). Under ADR-038 §2.8 it
is the **public-release trigger at a `0.x`**, and the numeric `1.0.0` is **reserved for the completed
rewrite** — the label reconciliation is a maintainer decision (FLAG-V1). Read "the ADR-022 tag" as
the functional-completeness event, not the semver `1.0.0`.

**Phase I is done** when ADR-038's Phase-I Definition of Done is checked by the maintainer: the H1
enabler set closed, `myc run` end-to-end, the ADR-022 functional-completeness gate closed on its own
criteria, and the maintainer ratifies "fully functional + usable" — then the **public flip executes at
a `0.x`** (the public version is decided at that point; the semver scheme was deferred to here per
ADR-038 §2.8). **Until then:** repo private, every crate `0.0.0`, `publish = false` workspace-wide (H0
closes the current 3-of-52 gap — `Empirical`, 2026-07-01). **Phase II's terminal is the semver `1.0.0`**
(= fully rewritten into Mycelium where appropriate + 100% operational, subject to FLAG-V2 on the
compiler), reached progressively in public; past `1.0.0` the North Star continues as maintenance.

### User stories (the roadmap's own)

- *As a language user*, I want `myc run` to execute a real multi-nodule program that touches
  integers, floats, strings, collections, files, and hyphae, so that Mycelium is a language I can
  *use*, not a corpus I can read.
- *As the maintainer*, I want the public flip gated on a checkable usability DoD rather than on
  rewrite completion, so that release timing follows user value (ADR-038) and the append-only
  decision trail stays honest.
- *As a contributor*, I want the crate graph acyclic by construction and the grammar frozen before
  mass porting, so that neither the architecture nor my ported nodules churn under me.
- *As the Phase-II port effort*, I want Phase I to land the below-grammar enablers and a polished
  transpiler, so that porting is mechanical-plus-review, not blocked-on-missing-prims.

---

## 2. H0 — Foundations: acyclic-deps hardening + workspace hygiene (FIRST)

Everything later lands under these invariants; retrofitting them after H1/H2 churn would mean
re-auditing all of it. Absorbed from the prior plan's Workstream A, unchanged in substance.

**Current state (`Empirical`, cargo-metadata dig 2026-07-01; spot-verified at HEAD):** 52 workspace
crates form a clean **acyclic 8-stratum DAG in normal deps** (`mycelium-core`: 41 dependents, 0
dependencies). Three defects/risks: (1) **three dev-dependency cycles** centered on `mycelium-cert`
(`select →[dev] cert → vsa → select` · `cert →[dev] proj → l1 → cert` · `cert →[dev] spore → proj →
l1 → cert`); (2) the **`mycelium-mlir → mycelium-std-runtime`** upward-layer anomaly (the same shape
PR #864 fixed by extracting `mycelium-sched`); (3) the **latent `interp ↔ cert` cycle** if
certified-mode execution ever lands inside `interp`. The invariant to preserve: **strictly downward
deps; `interp` NEVER depends on any `std-*`**.

| ID | Task | Notes |
|---|---|---|
| H0-1 | **Structural enforcement** — encode the downward-only 8-stratum invariant plus the `interp`-is-`std`-free rule as a committed check wired into `just check` (a cargo-deny ban-set and/or an xtask over `cargo metadata`; MUST cover **dev-deps** — cargo never rejects dev-dep cycles). Never-silent: a violation prints the offending edge and the rule it breaks. | Mechanism choice is an implementation call; prefer per-edge diagnostics. |
| H0-2 | **Resolve the three `cert` dev-cycles** — move cert's data-model/guarantee-tag types down toward `core`; convert cert's tests to fixtures instead of dev-importing `proj`/`spore`. | House test-layout rule (fixtures + parameterization). Tags keep their strength across the move (VR-5). |
| H0-3 | **Preempt `mlir → std-runtime`** — extract the runtime-ABI surface `mlir` needs into a lower crate (the `mycelium-sched` extraction precedent, PR #864). | The extraction seam is already visible in mlir's own re-export comment. |
| H0-4 | **Document the invariant** — strata map, the two rules, where the check lives, how a stratum assignment changes (by review, not by editing the ban-list in the violating PR). | Placement: default a DN; FLAG at PR time. |
| H0-5 | **Workspace publication hygiene** — `publish = false` across all 52 crates (workspace-level; 3/52 today), versions stay `0.0.0` until the Phase-I flip (ADR-038 §2.2). | Mechanical; lands with H0-1's PR wave. |
| H0-6 | **M-866** — fix the `mono.rs` `free_vars`/`pattern_binders` unbounded recursion (a real recursion-safety bug; property test bounding the recursion). | Correctness-first: do early, independent of H1/H2 sequencing. |

**Definition of Done (H0):** `just check` fails on any introduced cycle (normal *or* dev) and any
upward-layer or `interp→std-*` edge, with per-edge diagnostics; zero dev-dep cycles at HEAD
(re-survey recorded, `Empirical`); `mlir` no longer depends on `std-runtime`; the invariant doc
exists and is cited by the check's error message; `publish = false` workspace-wide; M-866 fixed with
a bound property test; change-scoped tests green throughout.

---

## 3. H1 — The functional-usability enablers (Phase I's critical path)

The readiness-§0 verification (2026-07-01) established that the remaining self-host/usability
blockers are **below-grammar** — kernel-prim surfacing, value-model implementation, or staged
execution — not surface gaps. H1 closes them. The gap labels A–E fix the shorthand used across the
kickoffs; the ordering **B → C → A → E → D-lite** is `Declared` (cheapest-first, with the
design-gated items latest so their ADR/reviews run in parallel):

| # | Gap | What lands | Blocks (stdlib) | Tracking |
|---|---|---|---|---|
| **B** | **Binary integer arithmetic + signed-op set** | `mul`/`div`/`shl`/`shr` prims; two's-complement shared ops plus the signedness-split `div`/`cmp`/`shift`/overflow-detect set (RFC-0033 §4.1.2–§4.1.3) — **M-766/M-767 pulled forward from the V-wave** (RFC-0033 changelog note, 2026-07-01) | `math`/`numerics` integer half | M-718 FLAG · ADR-028 · M-766/M-767 |
| **C** | **Dense/VSA op-prims surfaced to L1** | `dense.*`/`vsa.*` ops into the prim registry (types/literals already exist) | `dense`, `vsa` | mint at kickoff (readiness §0 item 3) |
| **A** | **Scalar-float value form — route (ii)** | A first-class scalar-float `Repr` with float literal/type/prims, via a **dedicated float ADR** plus a **DN-39 promotion review** (default-DENY — kernel entry is earned); content-address impact coordinated with the deferred ADR-030/031 one-way doors so the **rehash happens once**, deferred to the first value-persistence feature (RFC-0033 §7 / M-780 discipline — ADR-038 §2.6) | `math` (f64 half), `numerics` | future float ADR (design starts at H1 kickoff) |
| **E** | **`Substrate`/`consume` execution** | The staged `Residual` becomes real execution (v0 value form; the DN-54 derive-site `consume` model confirmed as the target — confirm, then implement) | `fs`, `io` (affine-handle model) | mint at kickoff (readiness §0 item 5) |
| **D-lite** | **R2 runtime vocabulary — usability subset** | The subset of `forage`/`backbone`/`xloc`/`mesh`/`cyst`/`graft` (DN-63) that `std.runtime`'s *usable surface* needs — activated per DN-63; the full R2 maturity wave stays H2/Phase II | `runtime` surface | M-828 (subset; scope the split at kickoff — FLAG) |

Plus three small items, threaded early (not letter-ranked):

- **`myc run`** — lift the honest CLI refusal; project→interpreter end-to-end. *The usability
  backbone: every other H1 item is validated through it.* First among equals.
- **Textual string literal** — ergonomic, not expressive (only `0x…` `BytesLit` today); cheap;
  unblocks *authoring* `text`/`fmt`/`diag`/`error` in `.myc`. Mint at kickoff (readiness §0).
- **`hash.*` prim surfacing** — blake3 lives in `mycelium-core`, unsurfaced; unblocks `content`.
  Mint at kickoff (readiness §0).

Every H1 item ships the house way: never-silent boundaries (`Option`/`Result`), honest per-op tags,
a property test per bound, conformance accept + reject cases. **Issue minting for the untracked
items happens at execution kickoff** (verify free `M-xxx` slots — mitigation #1), not in this doc.

**Definition of Done (H1):** each gap A–E closed (or explicitly maintainer-deferred with rationale —
G2); the three small items landed; a Mycelium program exercising integers (signed ops), floats,
strings, dense/vsa ops, fs/io, and the R2-lite runtime surface runs via `myc run`, three-way-clean
where forms close (L1-eval ≡ L0-interp ≡ AOT); readiness §0's blocker list re-verified empty or
explicitly reduced (`Empirical`, re-run and recorded).

---

## 4. H2 — Rust-reference closeout remainder (the ADR-022 tag tracks)

The prior plan's Workstreams B–G, minus what H0/H1 absorbed. These close the `lang 1.0.0` tag gate.
Parallelizable by disjoint crate ownership exactly as before (`mycelium-l1` stays the serial,
high-collision lane).

| Lane | Remainder | Gated on (FLAG — maintainer decisions) |
|---|---|---|
| **Language semantics** (`mycelium-l1`, serial) | Tuple decision → un-gate multi-arg lambda / partial application (RFC-0024 §4A.8); Fn-typed record-field lowering (ADR-033); `consume` checked semantics (rides H1-E); `Fuse` prelude + semilattice-law checker (F-A1/F-A2); `via` delegation trait-registry ordering; M-844 per-instantiation guarantee-tag context through mono; M-833 guard clauses (design-first) | Tuple decision · ADR-033 FLAG-1 (both also H2a items) |
| **Value model / AOT tail** | Any AOT dense/VSA codegen refusal still standing after H1-C is lifted where the model supports the form, or re-affirmed as a ratified refusal with a tracker item (a refusal is a decision, not a leftover); AOT output differentially validated per lifted form (the AOT is beside-and-validated-against interp, never the source of meaning). The deferred V-wave (ADR-030/031 doors, M-758/M-759 perf) stays **Phase II** except the H1 pull-forwards | RFC-0033 deferral stands (only B/A pulled forward) |
| **Runtime / concurrency maturity** | Full R2 vocabulary beyond D-lite (M-828 remainder); M-869 AOT/interp hypha/colony/async parity; M-868 scheduler leapfrogging; M-831 substrate/hypha reclamation; RT2 determinism `Empirical → Proven` **only** with machine-checked side-conditions (VR-5 — otherwise it honestly stays `Empirical`, tracked) | RFC-0027/DN-32 memory-model ratification (sequencing FLAG — confirm, don't guess) |
| **Toolchain / UX** | LSP sub-expression diagnostic spans + hover type/guarantee inference (the transparency rule made visible — hover shows the honest tag, never an upgraded one); M-697 editor packaging (Enacts RFC-0026 — step through Accepted properly); M-848 `just setup-scoped` (DN-65 §2.3); E22-1 security scanner | E22-1: RFC-0035 ratification |
| **Inject-mode security** (RFC-0038) | The full mechanism chain M-836 → M-837 → M-838 → M-839 → M-842 → M-847 → M-849, plus M-841 naming and M-806 disclosure gate; design-first against the ratified RFC (deviations FLAGged, never improvised); `/security-review` pass on completion | Build-scope confirmation |
| **Kernel freeze** (DN-56 — the closing act) | The four conditions: reject-ledger complete; primitive set closed (ADR-033 FLAG-1); lowering surface closed (RFC-0037 migration + DN-54 extension-checker); KC-3 completeness review. Runs **last** among the code lanes; DN-39 default-DENY holds throughout (0/4 conditions met today, `Empirical`) | FLAG-1 · RFC-0037/DN-54 (H2a) |

**H1 interaction:** H1's prim additions (B/C/A/`hash.*`) all *precede* the kernel freeze — each
enters under DN-39 review where it touches the kernel, and the freeze closes after them by
construction.

**Proof debt** (cross-cutting, unchanged from the prior plan): M-512 (dense-accumulation Higham
bound), M-541 (libm audit), M-511 (float total-order — note it becomes load-bearing with H1-A),
M-827 (graded soundness), M-829 (bound composition) ride whichever lane touches their code; a tag
upgrades `Empirical → Proven` only with machine-checked side-conditions; each item is explicitly
dispositioned (landed / deferred-with-item) at Phase-I closeout.

**Definition of Done (H2):** every lane's rows landed or explicitly maintainer-deferred with the
`Residual` re-affirmed; every surviving refusal intentional and tracked; DN-56's four conditions
checkable (the freeze itself is the **maintainer's** declaration, not this plan's); ADR-022 T8
closeable modulo the maintainer-reserved acts (M-703, M-738 — §7).

---

## 5. H2a — The grammar-stability gate (before mass porting)

**Rule: no mass `.myc` porting against a moving grammar.** Each ported nodule is a consumer of the
surface; grammar churn after a mass port re-touches every ported file. Before the port wave scales
past the opportunistic set (§6), the following close — this is the gate between "porting
opportunistically" and "porting the corpus":

1. **RFC-0037 follow-ons** — D2-b short repr keywords; RFC-0025 operator wiring (the enacted
   migration's named non-blockers become blockers *for mass porting specifically*).
2. **DN-54 completion (M-812-cont)** — RHS elaboration to L0, the §4.1 IL-grammar RHS type-check,
   the §6 KC-3 kernel-growth guard, §4.2 cross-rule acyclicity, the §7 verification harness. The
   lowering surface a mass port writes against must be the *final* one.
3. **Tuple-type decision** (RFC-0024 §4A.8) — un-gates multi-arg lambda / partial application;
   pervasive enough to reshape ported signatures. Maintainer decision; FLAG, never guess.
4. **ADR-033 FLAG-1** (Fn-typed record-field soundness) — the last open primitive-set question
   (also DN-56 condition 2).

**Definition of Done (H2a):** all four closed (decisions cited); `mycelium.ebnf`, editor grammars,
and the api-index regenerated and stable through a maintainer-set no-normative-change window
(`Declared` — the window length is the maintainer's call).

---

## 6. Opportunistic Phase-I Mycelium ports (ready-now, not gating)

Per ADR-038 §2.2, `.myc` porting proceeds **where the surface is ready** throughout Phase I —
welcome, honest, and **never the release gate**:

- **Landed:** `option`/`result` authored directly in Mycelium (no Rust source — DN-34 §8.6); 8
  `lib/std/*.myc` nodules executing three-way (readiness §0, `Empirical`).
- **Portable now — roughly 9 further pure/structural crates** (`Empirical` estimate: the ~19/26
  expressible set minus the landed 8; the exact list is confirmed crate-by-crate at kickoff):
  the pure/structural crates needing no H1 enabler — candidates from the readiness verification's
  expressible set (e.g. `error`, `recover`, structural `diag`, `convert`, structural `testing`;
  confirm each against the D5 bar before claiming it).
- **Discipline per port:** the RFC-0031 D5 per-op stability bar unchanged (differential vs the Rust
  oracle, honest tags, frozen signature); **pre-port polish** — clean the ambiguous Rust first
  (ADR-038 §2.5); transpiler-assisted where its coverage genuinely helps, hand-ported where not
  (accelerant, not gate); D6 oracle retention unchanged.
- The port *of the corpus at scale* waits for **H2a**; the opportunistic ports are small enough to
  absorb grammar follow-ons if those land after (`Declared` risk, accepted).

---

## 7. Phase II — post-public, progressive (clearly separated)

Everything below happens **after** the Phase-I flip, **in the open**, at the progressive cadence
ADR-038 §2.3 sets. None of it gated the public release (that already happened at a `0.x`); instead
this phase is the axis the **public semver climbs `0.x → 1.0.0`** (ADR-038 §2.8), where **`1.0.0` =
fully rewritten into Mycelium (where appropriate) + 100% operational** (subject to FLAG-V2 on whether
that requires compiler self-hosting). The concrete semver scheme is decided at the flip, not here.

- **Mass `.myc` porting** of the remaining corpus (post-H2a), transpiler-accelerated under the
  polished-transpiler + pre-port-polish + manifest-where-ROI-positive doctrine (ADR-038 §2.5); each
  replacement per ADR-036 §2.3 (differential-validated, replaced only on maintainer satisfaction) —
  unchanged.
- **The V-wave remainder** (RFC-0033 V1–V5: the ADR-030/031 one-way doors, swap/guarantee
  reconciliation, M-758/M-759 perf): lands with the **single rehash** (the M-780 pattern) at the
  first value-persistence feature, coordinated with H1-A's float `Repr` so the identity set changes
  **once** (RFC-0033 §7 honored — see its 2026-07-01 changelog note).
- **Repository decomposition + per-repo publication:** DN-27's mechanics ADR (topology, history,
  re-export form — DN-27 §5's open questions) is authored **at the Phase-I→II boundary**; spores
  publish per-repo via the ADR-037 GHCR/OCI backend extended per-repo. (Prior shorthand that called
  this "the decomposition ADR-038" is stale: **ADR-038 is the *strategy* ADR**; the decomposition
  *mechanics* ADR takes the next free number when authored.)
- **Compiler self-hosting — conditional aspiration** (ADR-038 §2.3; RFC-0031 §5 D1 note): only if
  demonstrably better on stability and/or performance, only after the transpiler is 100% polished;
  the bootstrap protocol is ratified then (D3's no-circularity staging holds until that act).
- **Maintainer-reserved acts** stay reserved: M-703 (kernel tag), M-738 (release act), M-655,
  M-381/M-646 (LLM runs), M-816 — exit-adjacent, never performed by agents.
- **M-797** inline-test extraction stays the lazy as-touched sweep (standing maintainer choice).

---

## 8. Ordering, and the maintainer decisions this plan waits on

```text
H0 (serial, short, FIRST)
 └─→ H1 enablers: B → C → A(route-ii) → E → D-lite   (+ myc run · strings · hash.*, early)
      │            (A's float ADR + DN-39 review drafted in parallel from H1 kickoff)
      ├─→ H2 lanes fan out (disjoint crates; l1 serial; kernel freeze LAST)
      │     └─→ lang 1.0.0 tag gate closes (ADR-022 as amended)
      ├─→ opportunistic .myc ports (continuous, non-gating)
      └─→ H2a grammar-stability gate ──→ (Phase II) mass porting
                 ↓
      ADR-038 Phase-I DoD checked by maintainer → PUBLIC FLIP → Phase II
```

| Decision (FLAG — none decided by this plan) | Gates | Ref |
|---|---|---|
| **ADR-038 ratification** (the strategy itself) | everything beyond the prior plan's baseline | ADR-038 |
| Float ADR + DN-39 promotion-review outcome | H1-A | ADR-038 §2.6 · DN-39 |
| D-lite scope split of M-828 | H1 D-lite vs H2 remainder | DN-63 |
| Tuple-type decision (RFC-0024 §4A.8) | H2 l1 lane + H2a | RFC-0024 |
| ADR-033 `FieldSpec::Fn` FLAG-1 | H2 l1 lane + H2a + DN-56 cond. 2 | ADR-033 |
| RFC-0037 follow-ons + DN-54 completion sign-off | H2a | RFC-0037 · DN-54 |
| RFC-0038 build-scope confirmation | H2 inject-mode lane | RFC-0038 |
| RFC-0035 ratification | E22-1 (H2 toolchain lane) | RFC-0035 |
| RFC-0027/DN-32 memory-model ratification | H2 runtime-lane sequencing | RFC-0027 · DN-32 |
| Kernel-freeze declaration (DN-56, after its 4 conditions) | Phase-I closeout | DN-56 |
| "Fully functional + usable" ratification (the flip) | Phase I → Phase II | ADR-038 §5 |
| **`lang 1.0.0` label reconciliation (FLAG-V1)** — ADR-022's functional-completeness event vs the semver `1.0.0` reserved for the completed rewrite | the flip's version + naming | ADR-038 §2.8 · ADR-022/023 |
| **Whether `1.0.0` requires compiler self-hosting (FLAG-V2)** | Phase-II terminal definition | ADR-038 §2.8 · §2.3 |
| **Public semver scheme** (how rewrite progress maps to `0.x`; deferred until publish-time) | the flip | ADR-038 §2.8 |

Execution follows ADR-038 §2.7: **Fable-class models are reserved solely for planning + complex
design** — this plan and its decompositions are prepared by that tier; **implementation and lighter
work land on Opus/Sonnet/Haiku scoped to task complexity**, as bite-sized, PM-prepped (user stories +
DoD) tasks; issue minting happens at each kickoff, not here (mitigation #1).

## 9. Grounding / corpus references

- **ADR-038** (Proposed) — the governing strategy; **ADR-022** (with ADR-024/034/035) — the
  unchanged tag gate; **ADR-036** — the dogfooding/validation model (§2.1–§2.3) and the §2.4 gate
  as refined; **ADR-037** — the GHCR/OCI registry Phase II extends per-repo; **DN-27** — flip
  mechanics (its binding mechanics ADR is authored at Phase-II kickoff).
- **RFC-0031 §5** — composition boundary/mechanics (D1 scope note, 2026-07-01); **RFC-0033**
  (with §7 and the 2026-07-01 pull-forward note); **ADR-028/030/031** · **DN-39** · **DN-42** ·
  **DN-54** · **DN-56** · **DN-63** · **RFC-0024 §4A.8** · **RFC-0026/0027/0035/0037/0038** ·
  **ADR-033** · **DN-20/DN-65** — as cited in the lanes above.
- **`docs/spec/stdlib/self-hosting-readiness.md` §0** (2026-07-01) — the surface-sufficiency
  verification H1 is built from; **DN-34 §8 / M-873** — transpiler coverage (`Empirical`, ≈12.4%
  union); **PR #864** — the extraction precedent for H0-3.
- **Fable-5 research digs (2026-07-01)** — dependency-graph survey (`Empirical`, spot-verified at
  HEAD), roadmap/architecture survey, open-work survey. M-xxx ids are `Empirical` leads —
  re-verify each against `tools/github/issues.yaml` at its kickoff.
- Companion plans: `value-model-implementation-plan.md` (E20-1 detail),
  `Blocked-Decisions-Ratification-Map.md` (decision batching — reconcile §8's FLAG table with it),
  `dogfooding-effort-and-usage-assessment.md` and `self-hosting-port-ledger.md` (the port side).

---

## Meta — changelog

- **2026-07-01 — created** (advisory, for maintainer review; authored per the maintainer's
  "everything in Rust, no circular deps, before self-hosting" directive). Grounded in the three
  Fable-5 research digs of the same date; forecasts are `Declared`, survey facts `Empirical`.
  Companion kickoff: `.claude/kickoffs/rcp.md`. Append-only.
- **2026-07-01 — §8a added** (stdlib surface-sufficiency verification): maps the ~5 below-grammar
  self-host-enabler gaps to Workstreams B/C/D, lists 4 issues to mint, and corrects the §11 E18-1
  demand-data note (the `trx` transpiler backlog over-states surface gaps; §8a is authoritative).
  Companion: the append-only §0 currency update to `docs/spec/stdlib/self-hosting-readiness.md`
  (whose 2026-06-17 "not-yet" verdict was stale). Append-only.
- **2026-07-01 — REVISED into the function-first umbrella roadmap** (maintainer-directed, same
  session as ADR-038; renamed from `rust-reference-completion-and-acyclic-deps.md` — a pointer stub
  remains at the old path). The reframe: phases sequenced by **function** (usability) rather than
  by implementation language, per ADR-038 (Proposed). Content mapping from the prior form:
  Workstream A → **H0** (plus publish-hygiene H0-5 and M-866 as H0-6); the §8a gap table → **H1**
  (labels A–E fixed; order B→C→A(route-ii)→E→D-lite; plus `myc run`/string-literal/`hash.*`);
  Workstreams B–G remainder → the **H2** lanes; the grammar-stability items split out as the
  **H2a** mass-porting gate; opportunistic `.myc` ports (§6) made explicit as
  Phase-I-welcome/non-gating; **Phase II** (post-public progressive rewrite, V-wave remainder with
  its single rehash, DN-27 decomposition ADR and per-repo GHCR, conditional compiler self-hosting)
  separated cleanly (§7). Nothing from the prior plan was dropped: every workstream row is absorbed
  into a horizon/lane or explicitly listed as Phase II / reserved. Forecasts remain `Declared`;
  survey facts `Empirical`. Append-only meta-log; the prior entries above are preserved verbatim.
- **2026-07-01 — versioning-axis + execution-doctrine refinement** (maintainer, same session; folds
  ADR-038's same-day pre-ratification refinement into this roadmap). **Public release decoupled from
  the version number:** the flip happens at functional usability, at **whatever `0.x` fits — well
  before `1.0.0`** (ADR-038 §2.8); the public semver **tracks the Mycelium rewrite**, climbing
  `0.x → 1.0.0` **in the open**, with **`1.0.0` = fully rewritten + 100% operational** as Phase II's
  terminal (was "no terminal gate"). **For now the version stays `0.0.0`; the semver scheme is
  deferred to publish-time.** Updated: the phase-map ASCII (public flip at a `0.x`, semver climb in
  Phase II), §1 exit criteria, the Governing/Goal cells, §7, and §8's FLAG table (added FLAG-V1 label
  reconciliation, FLAG-V2 compiler-in-`1.0.0`, and the deferred semver-scheme rows). **Execution
  doctrine (§8):** Fable reserved solely for planning/complex-design; implementation on
  Opus/Sonnet/Haiku scoped to complexity. Append-only; prior entries preserved verbatim.
