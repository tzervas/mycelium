# Rust Reference Completion and Acyclic Dependency Graph — Closeout Plan

| Field | Value |
|---|---|
| **Status** | **Advisory / Proposed for maintainer review** (2026-07-01). Planning posture, like the other `docs/planning/*.md` — this doc **decides nothing**; it proposes a sequence and flags every maintainer decision it depends on. `Declared` wherever it forecasts effort or ordering. |
| **Goal** | Get **everything implemented in Rust** (the reference implementation), with a clean **acyclic crate dependency graph** — no circular dependencies, in normal *and* dev deps — enforced structurally so it stays that way. |
| **Gates** | This milestone **precedes and gates E18-1 self-hosting** (the subsequent phase — out of scope here; see §11). |
| **Basis** | ADR-022 (1.0.0 full-language gate, T1–T9) · DN-56 (kernel-freeze conditions) · ADR-036 (public-release gate) · three Fable-5 research digs, 2026-07-01: dependency-graph survey (`cargo metadata`, `Empirical`), roadmap/architecture survey, open-work survey. |

**The crucial reframe.** The Rust reference is **largely complete**: elaboration is done (M-673),
generics/traits run, and there are **zero** `todo!`/`unimplemented!`/`TODO`/`FIXME` markers in
non-test source — every incompleteness is an explicit, never-silent `Residual`/refusal with a
tracker item (G2 held all the way down). So this is **not** a green-field implementation phase; it
is a **bounded closeout**: seven finite workstreams of remaining code, plus **dependency-graph
hardening** so the "no circular deps" mandate is a *structural invariant* rather than a survey
result. When this plan's exit criteria (§1) are met, the Rust reference is the stable trusted base
E18-1 rewrites *against* — which is exactly why it must land first.

---

## 1. Definition of "Rust reference complete" (exit criteria)

The milestone is **done** when all of the following hold. Each criterion names its basis; none may
be waved through — a criterion that is *deferred* must be deferred **explicitly**, with rationale
recorded (G2: never silently dropped).

1. **All workstream buckets (§2–§8) are landed or explicitly deferred.** Every remaining-work item
   in the open-work survey is either implemented, or carries a recorded maintainer deferral with a
   tracker item. No silent gaps.
2. **The dependency graph is acyclic and the invariant is structural** (§2). Normal deps: the
   8-stratum downward-only DAG holds. Dev deps: the three `mycelium-cert` dev-cycles are resolved.
   The layering rules (downward-only; `interp` never depends on any `std-*`) are enforced by a
   committed check that runs in `just check` — a future PR *cannot* introduce a cycle without
   turning the gate red.
3. **Kernel freeze is met per DN-56** (§8): reject-ledger complete; primitive set closed
   (ADR-033 `FieldSpec::Fn` FLAG-1 resolved); lowering surface closed (RFC-0037 migration plus the
   DN-54 extension-checker); KC-3 completeness review done. *(Currently: 0 of 4 conditions met —
   `Empirical`, roadmap survey.)*
4. **ADR-022 T8 (docs/stability) is closeable modulo maintainer-reserved acts.** T1–T7 are already
   done (`Empirical`, roadmap survey); T8 closes except for the acts this plan does not touch
   (M-703 kernel tag, M-738 release act — §11).
5. **Every surviving refusal is intentional.** Each remaining never-silent refusal in the codebase
   (e.g. AOT dense/VSA codegen, `myc run`) is either lifted by §4/§6 or re-affirmed as a
   ratified-permanent refusal with its tracker item — a refusal is a *decision*, not a leftover.
6. **The graduation note is written.** A short closeout note records the final state (strata map,
   check locations, deferred list) and hands off to E18-1 — the durable artifact for intent
   (mitigation #8 applied at milestone scale).

*Not* in the definition: T9 self-hosting (explicitly **not tag-gating** per ADR-022; it gates
*public release* per ADR-036 — the next milestone, §11), and the maintainer-reserved acts (§11).

---

## 2. Workstream A — Dependency-graph hardening (FIRST; the "no circular deps" mandate)

**Why first:** every other workstream lands *crate changes*. If the acyclicity invariant is
enforced before they start, all subsequent work lands under cycle-enforcement and the mandate holds
by construction — retrofitting the check after six workstreams of churn would mean re-auditing all
of it. A is small, foundational, and unblocks nothing-else-blocks-it parallelism.

**Current state (`Empirical`, cargo-metadata dig 2026-07-01; spot-verified against `Cargo.toml`s at
HEAD):** 52 workspace crates form a clean **acyclic 8-stratum DAG in normal deps**
(`mycelium-core` is the pure foundation: 41 dependents, 0 dependencies). Three defects/risks:

1. **Three dev-dependency cycles, all centered on `mycelium-cert`:**
   `select →[dev] cert → vsa → select` · `cert →[dev] proj → l1 → cert` ·
   `cert →[dev] spore → proj → l1 → cert`. Cargo tolerates dev-dep cycles, but they couple test
   builds upward, defeat clean stratification, and are exactly the shape that hardens into a real
   cycle later.
2. **Top-layer anomaly: `mycelium-mlir → mycelium-std-runtime`** (normal dep,
   `crates/mycelium-mlir/Cargo.toml`) — a Tier-2 backend depending on a Tier-4 stdlib crate. This
   is the same shape as the interp↔std-runtime cycle fixed in PR #864 by extracting
   `mycelium-sched`; left alone it invites the same failure.
3. **Latent `interp ↔ cert` cycle** if certified-mode execution ever lands inside `interp`
   (cert already depends on interp). The invariant to preserve: **strictly downward deps; `interp`
   must NEVER depend on any `std-*`** (interp is the trusted base — `sched` sits below it, stdlib
   above it).

### User stories

- *As a contributor landing a crate change*, I want the build to **fail loudly** if my edit
  introduces a cycle or an upward-layer dep, so that I cannot degrade the architecture by accident
  (G2 — never-silent at the dependency-graph level).
- *As the maintainer*, I want the 8-stratum layering written down and machine-checked, so that the
  architecture documented in DN-38/ADR-022 and the architecture actually built cannot drift apart.
- *As the E18-1 self-hosting effort*, I want a stratified, cycle-free crate graph, so that crates
  can be ported bottom-up in dependency order with no entanglement to unpick first.
- *As a `mycelium-cert` test author*, I want cert's tests to run against fixtures rather than
  importing `proj`/`spore`, so that the certification layer stays below the things it certifies.

### Tasks

| ID | Task | Notes |
|---|---|---|
| A1 | **Structural enforcement** — encode the downward-only 8-stratum invariant plus the `interp`-is-`std`-free rule as a committed check wired into `just check`: a `cargo-deny` ban-set and/or an `xtask`/script layering check over `cargo metadata` (must cover **dev-deps** too — cargo itself never rejects dev-dep cycles, so the check has to). | The choice of mechanism (cargo-deny vs xtask vs both) is an implementation call, not maintainer-gated; prefer whichever gives per-edge diagnostics. Never-silent: a violation prints the offending edge plus the stratum rule it breaks. |
| A2 | **Resolve the three `cert` dev-cycles** — move cert's data model / guarantee-tag types **down toward `core`** (so `select`/`l1` consumers need not pull `cert` itself), and convert cert's tests to **fixtures** instead of dev-importing `proj`/`spore`. | Follows the house test-layout rule (fixtures + parameterization, not bespoke imports). After A2, the dev-dep graph is as acyclic as the normal graph. |
| A3 | **Preempt the `mlir → std-runtime` anomaly** — extract the runtime-ABI surface `mlir` actually needs into a lower crate (mirroring the `mycelium-sched` extraction, PR #864), so the backend depends downward only. | The mlir crate's own comment says it routes through a std-runtime re-export — the extraction seam is already visible. |
| A4 | **Document the invariant** — the strata map, the two rules (downward-only; interp-std-free), where the check lives, and how to change a stratum assignment (deliberately, via review — not by editing the ban-list in the same PR that violates it). | Placement (a DN vs an architecture-doc section) is a small maintainer call — FLAG at PR time, default to a DN. |

### Definition of Done (Workstream A)

- `just check` fails on any introduced cycle (normal *or* dev) and on any upward-layer or
  `interp→std-*` edge, with a per-edge diagnostic naming the violated rule.
- Zero dev-dep cycles at HEAD (the three cert cycles resolved); `cargo metadata` re-survey confirms
  the 8-stratum DAG including dev edges (`Empirical`, re-run and recorded).
- `mycelium-mlir` no longer depends on `mycelium-std-runtime` (the ABI surface lives in a lower
  crate); interp's dep set contains no `std-*` crate.
- The invariant doc (A4) exists and is cross-referenced from the check's error message.
- Change-scoped tests green for every touched crate; the moved cert types keep their guarantee tags
  at unchanged strength (VR-5 — a code move never upgrades a tag).

---

## 3. Workstream B — Language-semantics remainder (`mycelium-l1`; small, partly decision-gated)

**What's left** (open-work survey; all in or around `mycelium-l1`, which makes this the **serial,
high-collision** workstream — see §10):

| Item | Covering ref | Ready vs gated |
|---|---|---|
| Tuple-type decision → un-gate multi-arg lambda / partial application (currently an explicit `Residual`) | RFC-0024 §4A.8 | **Design-gated — maintainer decides the tuple-type question**; the code change is small once decided. FLAG, do not guess. |
| Fn-typed record-field lowering (dynamic dispatch) | ADR-033 | **Design-gated** — depends on the `FieldSpec::Fn` soundness FLAG-1 resolution (also kernel-freeze condition 2, §8). |
| `consume` checked semantics (asserted-not-checked today) | DN-54 (derive-site model) | Ready once the DN-54 model is confirmed as the implementation target — confirm, then implement. |
| `Fuse` prelude + semilattice-law checker | FLAG F-A1 / F-A2 | Ready-to-implement modulo the two open FLAGs — resolve or explicitly scope them at kickoff. |
| `via` delegation trait-registry ordering | (surface spec) | Ready. |
| Bound `mono.rs` `free_vars`/`pattern_binders` recursion — a real recursion-safety bug | M-866 | **Ready — do early** (correctness/safety, not a feature). |
| Per-instantiation guarantee-tag context through mono | M-844 | Ready. |
| Guard clauses | M-833 | Design item — needs its design pass before code. |

### Definition of Done (Workstream B)

- M-866 fixed with a regression/property test bounding the recursion (a property test for every
  bound); M-844 landed with tags at supportable strength.
- Every design-gated row above either landed (decision made, cited) or explicitly deferred by the
  maintainer with the `Residual` refusal re-affirmed — no silent gaps, no guessed decisions.
- `mycelium-l1` change-scoped checks green; conformance corpus extended for each activated
  construct (the `REJECT_EXPECTED` pattern for what stays rejected).

---

## 4. Workstream C — Value-model completion + lifting the AOT refusals (E20-1 tail)

**What's left:** land ADR-030 (`QuantDesc` into `Repr`) and ADR-031 (VSA element-space,
block-sparse, complex carriers); **then lift the AOT dense/VSA codegen refusals** in
`crates/mycelium-mlir/src/dialect/native/{dense,vsa}.rs` (which today never-silently refuse these
forms — the refusals exist *because* the value-model decisions weren't landed; lifting them is the
point of the workstream). Plus M-758/M-759: `PackedTernary` limbed representation and
Karatsuba/Toom ternary multiplication (perf — gate on benchmarks per the value-model plan's YAGNI
note).

Companion doc: `docs/planning/value-model-implementation-plan.md` (E20-1 task breakdown,
M-754…M-784) — this workstream is its remaining tail, not a re-plan.

### Definition of Done (Workstream C)

- ADR-030/ADR-031 carriers implemented with per-op guarantee tags; content-address implications
  handled per the value-model plan's one-way-door sequencing (no second rehash).
- The dense/VSA AOT refusals are **lifted** where the model now supports the form, and each
  refusal that remains is re-affirmed with a tracker item (exit criterion 5).
- AOT output differentially validated against the interpreter for every lifted form (the AOT is
  beside-and-validated-against interp, never the source of meaning).
- M-758/M-759 landed **or** explicitly deferred behind a benchmark gate (YAGNI, recorded).

---

## 5. Workstream D — Runtime vocabulary R2 + concurrency maturity

**What's left:**

- **Activate the R2 runtime vocabulary** — `forage` / `backbone` / `xloc` / `mesh` / `cyst` /
  `graft` (currently lexed-reserved, inactive) per DN-63 — tracked as **M-828**.
- **M-869** — AOT/interp hypha/colony/async **parity** (the differential surface for concurrency).
- **M-868** — scheduler leapfrogging.
- **M-831** — substrate/hypha reclamation.
- **RT2 determinism `Empirical` → `Proven`** — the Kahn-determinism theorem with *checked*
  side-conditions (VR-5: `Proven` only with a theorem whose side-conditions are checked; until
  then the tag honestly stays `Empirical`).

**Decision dependency (FLAG):** RFC-0027/DN-32 **memory-model ratification** bears on the
concurrency-maturity items — confirm with the maintainer whether the RT2 proof and reclamation
work should wait on it or proceed against the current draft (do not guess).

### Definition of Done (Workstream D)

- R2 vocabulary active per DN-63, each construct with conformance cases (accept + reject).
- Hypha/colony/async differential parity green between interp and AOT (M-869).
- M-868/M-831 landed with property tests for their bounds.
- The RT2 determinism tag is `Proven` **only if** the Kahn theorem's side-conditions are
  machine-checked; otherwise it remains `Empirical` and the gap is a recorded, tracked deferral.

---

## 6. Workstream E — Toolchain / UX completeness

**What's left:**

- **Wire `myc run`** — the project→interpreter pipeline is currently an honest refusal in
  `mycelium-cli`; lift it (exit criterion 5).
- **LSP**: sub-expression diagnostic spans; hover type/guarantee inference (surfacing the
  per-op tags in the editor is the transparency rule made visible).
- **M-697** — editor packaging (this **Enacts RFC-0026** — note the append-only step: Accepted →
  Enacted only once fully landed, never skipped).
- **M-848** — `just setup-scoped` (the DN-65 §2.3 scoped-toolchain automation).
- **E22-1** — security scanner. **Design-gated: blocked on RFC-0035 ratification** (FLAG —
  maintainer decision; until ratified this stays out of the implementable set).
- **M-797** — the ~185-file inline-test extraction. Continues as the **lazy as-touched sweep**
  (the maintainer's chosen rollout) — this plan does *not* convert it into a big-bang refactor;
  it simply notes that workstreams B–F will retire a share of it as they touch files.

### Definition of Done (Workstream E)

- `myc run` executes a project through the interpreter end-to-end with never-silent errors;
  the CLI refusal is gone.
- LSP spans/hover landed with tests; hover shows the honest per-op tag (never an upgraded one).
- M-697 landed and RFC-0026 moved `Accepted → Enacted` (append-only, with the landing cited);
  M-848 landed and referenced from CONTRIBUTING/CLAUDE workspace-prep text.
- E22-1 either implemented post-ratification or explicitly held at "blocked on RFC-0035"
  (maintainer's call — FLAGged, not guessed).

---

## 7. Workstream F — Inject-mode security mechanism (RFC-0038; design ratified, mechanism unbuilt)

**What's left:** the whole mechanism chain — **M-836 → M-837 → M-838 → M-839 → M-842 → M-847 →
M-849**, plus **M-841** (naming) and **M-806** (disclosure gate). RFC-0038's *design* is ratified;
the *build* has not started. This is the one workstream that is a real (bounded) construction
effort rather than a closeout, and it is **design-first**: each mechanism piece lands against the
ratified design, with any deviation FLAGged back rather than improvised.

**Decision dependency (FLAG):** confirm the **build scope** with the maintainer before kickoff —
the RFC is ratified, but "everything in Rust" here should mean the scope the maintainer intends
for the reference implementation, not the maximal reading of the RFC.

### Definition of Done (Workstream F)

- The M-836…M-849 chain landed in dependency order, each with tests (including negative/`reject`
  cases — a security mechanism's refusals are its spec); M-841 naming applied; M-806 disclosure
  gate in place.
- `/security-review` pass on the completed mechanism (secrets, input handling, the usual classes).
- Every deviation from RFC-0038 as ratified is either zero or recorded as a FLAGged,
  maintainer-acknowledged delta (append-only: a real design change supersedes, it doesn't rewrite).

---

## 8. Workstream G — Kernel freeze (DN-56; the closing act)

**What's left — the four open DN-56 conditions** (`Empirical`, roadmap survey: 0/4 met today):

| # | Condition | Covering work | Gated on |
|---|---|---|---|
| 1 | Reject-ledger complete | Ledger sweep + closure | Ready (mechanical + review). |
| 2 | Primitive set closed | ADR-033 `FieldSpec::Fn` **FLAG-1** resolution | **Maintainer decision** (same flag as §3's fn-typed fields). |
| 3 | Lowering surface closed | **RFC-0037** grammar-migration + **DN-54** extension-checker | **Maintainer decision/ratification** on both. |
| 4 | KC-3 completeness review | The small-auditable-kernel review pass over L0 (10 nodes, frozen small) | Ready once 1–3 land (it reviews the *final* set). |

G runs **last** among the code workstreams by construction: freezing the kernel before B–F land
their kernel-adjacent pieces would either block them or force post-freeze exceptions. DN-39's
default-DENY kernel-promotion stays in force throughout — nothing enters L0 during B–F without its
own promotion case.

### Definition of Done (Workstream G)

- All four DN-56 conditions checked off with citations (the ledger, the two decisions, the review
  record); the freeze itself is declared by the **maintainer**, not by this plan — the workstream's
  job is to make the declaration *possible* and grounded.
- The L0 kernel is unchanged by B–F except through recorded DN-39 promotions (audit the diff).

---

## 8a. Stdlib surface-sufficiency (verified 2026-07-01) — which self-host blockers are Rust work in this plan

A dedicated verification (Fable-5, 2026-07-01) answered "is the Mycelium *surface* sufficient to write
all 26 stdlib crates in pure `.myc`?" **Answer: PARTIALLY-TRUE, mostly stale.** The grammar/checker
surface IS sufficient for ~19/26 crates (8 already run as `.myc`; the classic suspected gaps —
generics, traits, effects, HOF **including capturing closures** (M-704), FFI, sequences — have all
landed). The ~5–7 blocked crates are blocked by **below-grammar** items, and those items are
**Rust-reference work already inside this plan's workstreams** — this section maps them so the plan
is complete on the self-host-enabler axis. *(The `trx` transpiler gap-backlog OVER-states surface
gaps — it measures a mechanical mapper, not the surface; use this verification, not that backlog, for
surface-sufficiency. The stale `docs/spec/stdlib/self-hosting-readiness.md` §0 update, 2026-07-01,
records the same finding.)*

| Verified surface-enabler gap | Blocks | Home in this plan | Tracking |
|---|---|---|---|
| Float value form + ops (no float literal/type/prims) | `math`(f64), `numerics` | **Workstream C** (value-model; the AOT-refusal tail rides the same crates) | E20-1 / RFC-0033 (post-1.0 per ADR-035) |
| Binary `mul`/`div`/`shl`/`shr` prims + signed-op set | `math`/`numerics` integer half | **Workstream C** (value-ops) / **B** (surfacing to L1) | M-718 FLAG · ADR-028 · E20-1 |
| Surface `dense.*`/`vsa.*` op-prims to L1 | `dense`, `vsa` | **Workstream C** | **Untracked — MINT an issue** |
| RFC-0008 R2 runtime vocabulary activation | `runtime` full surface | **Workstream D** (M-828) | E12-1 / DN-63 |
| `Substrate`/`consume` execution (staged `Residual`) | `fs`, `io` | **Workstream B** (extends the `consume`-semantics row) | **Untracked as execution — MINT an issue** |
| Textual string literal (ergonomics, not expressiveness) | authoring `text`/`fmt`/`diag`/`error` | **Workstream B** (grammar/lexer) or a small standalone | **Untracked — MINT (ergonomic, low priority)** |
| `hash.*` prim surfacing for `content` (blake3 in core, unsurfaced) | `content` | **Workstream C** (prim surfacing) | **Untracked — MINT** |

**Actions folded into this plan:** (1) the tracked gaps are already covered by C/D/B — no new
workstream; (2) **four issues should be minted** (Dense/VSA-to-L1 prims, Substrate execution, string
literal, `hash.*` prim) so the tracker is complete — do this at kickoff, verifying free `M-xxx` slots
(mitigation #1); (3) the surface-sufficiency verdict is `PARTIALLY-TRUE`, and full self-hosting stays
post-1.0 (ADR-035) — so these enablers are **in scope for Rust-complete** (they're prim/value work),
while the actual `.myc` porting that consumes them is E18-1 (out of scope, §11). Distinguishing the
two is the point: this plan lands the *enablers* in Rust; E18-1 writes the `.myc` against them.

## 9. Guarantee-tag / proof debt (continuous, cross-cutting)

Runs alongside all workstreams, not as a phase: **M-512** (dense accumulation Higham bound),
**M-541** (libm math audit), **M-511** (float total-order), **M-827** (graded-soundness
machine-checked proof), **M-829** (bound-composition). Rules of engagement:

- A proof-debt item upgrades a tag (`Empirical → Proven`) **only** when its side-conditions are
  machine-checked (VR-5); partial progress is recorded without an upgrade.
- Proof items attach to whichever workstream touches their code (e.g. M-512 rides Workstream C);
  the ones nothing touches are scheduled opportunistically and are **not** exit-blocking unless the
  maintainer says so — but each must be explicitly dispositioned (landed / deferred-with-item) at
  closeout (exit criterion 1).

---

## 10. Dependency ordering + milestones

```
M1  Workstream A — acyclic graph enforced structurally          (serial, short, FIRST)
     │
M2  Workstream B — language-semantics remainder                 (serial thread on mycelium-l1)
     │        ╲  (B runs as the serial l1 thread; C/D/E/F fan out beside it on disjoint crates)
M3  Workstream C — value-model + AOT refusal lifting   ─┐
M4  Workstream D — runtime R2 + concurrency            ─┼─ parallel (disjoint crate ownership)
M5  Workstream E — toolchain/UX                        ─┤
M6  Workstream F — inject-mode security                ─┘
     │
M7  Workstream G — kernel freeze (DN-56, 4 conditions)          (last code milestone)
     │
M-final  "Rust reference complete" declared (§1 criteria)  →  hand off to E18-1 self-hosting
```

**Parallel vs serial** (`Declared` — a forecast, to be validated at kickoff):

- **A is strictly first** — short and serial; everything after lands under cycle-enforcement.
- **B is the serial thread**: `mycelium-l1` is the shared high-collision crate, so B runs as one
  sequential lane (serial-on-shared-files, per the Wave-N collision-profile rule) rather than a
  fan-out.
- **C, D, E, F are parallelizable** with each other and with B — disjoint crate ownership:
  C owns `mlir`/`dense`/`vsa`(+value-model crates), D owns `sched`/`interp`/`std-runtime`,
  E owns `cli`/`lsp`/tooling, F owns the `sec`/inject-mode surface. **Watch the D↔F seam**: if
  F's mechanism lands inside `interp`, D and F share a crate — sequence those specific PRs or
  raise ownership to the shared parent (the fractal ownership rule). Also note the **latent
  interp↔cert cycle** (§2): D's interp work must not pull `cert` downward — A1's check makes this
  a hard error rather than a review catch.
- **G is last**, and **§9 rides along** continuously.

**Maintainer decisions gating each milestone** (FLAG — ratify before or during; none are decided
by this plan):

| Decision | Gates | Ref |
|---|---|---|
| Tuple-type decision (un-gates multi-arg lambda / partial application) | M2 (B) | RFC-0024 §4A.8 |
| ADR-033 `FieldSpec::Fn` soundness FLAG-1 | M2 (B) + M7 condition 2 | ADR-033 |
| RFC-0037 grammar-migration + DN-54 extension-checker | M7 condition 3 (and B's `consume` model) | RFC-0037 · DN-54 |
| RFC-0038 mechanism build-scope confirmation | M6 (F) | RFC-0038 |
| RFC-0035 ratification | E22-1 inside M5 (E) | RFC-0035 |
| RFC-0027/DN-32 memory-model ratification | M4 (D) proof/reclamation sequencing | RFC-0027 · DN-32 |
| Whether E18-1 waits for full Rust-complete or overlaps | M-final → next phase | ADR-022 T9 · ADR-036 |

The last row is a **sequencing** decision about the *next* milestone, recorded here only so the
handoff is explicit; this plan's own scope ends at M-final either way.

---

## 11. Out of scope (noted, not planned)

- **E18-1 self-hosting** — the *subsequent* milestone. It is `needs-design`, **not tag-gating**
  for 1.0.0 (ADR-022), and gates the public release (ADR-036). This plan produces its
  precondition (a complete, stratified, cycle-free Rust reference) and hands off; it does not plan
  E18-1's execution. Its demand data exists in two forms: the `trx` transpiler gap-backlog (DN-34 §8
  — but that *over-states* surface gaps, being a mechanical-mapper measure), and the **authoritative**
  surface-sufficiency verification (§8a, 2026-07-01) which shows the surface is sufficient for ~19/26
  crates and names the real below-grammar enablers (which §8a lands *in Rust* as part of this plan).
- **Maintainer-reserved acts** — M-703 (kernel tag), M-655, M-381/M-646 (LLM runs), M-816, and
  the M-738 release act. Noted as exit-adjacent; not planned, not performed by agents.
- **Big-bang M-797 extraction** — stays the lazy as-touched sweep (§6), by standing maintainer
  choice.

---

## 12. Grounding / corpus references

- **ADR-022** — full-language 1.0.0 gate, tracks T1–T9 (T1–T7 done; T8 in progress; T9 non-gating).
- **DN-56** — kernel-freeze conditions (the four in §8). **DN-39** — default-DENY kernel promotion.
- **DN-38** — lowering law (L2/L3 defined entirely by elaboration to L0). **KC-3** — small
  auditable kernel.
- **ADR-036** — public-release gate (self-hosting). **ADR-030/ADR-031** — value-model carriers
  (§4). **ADR-033** — fn-typed record fields + FLAG-1 (§3, §8).
- **RFC-0024 §4A.8** (tuple/partial-application `Residual`) · **RFC-0026** (editor packaging) ·
  **RFC-0027/DN-32** (memory model) · **RFC-0035** (security scanner) · **RFC-0037** (grammar
  migration) · **RFC-0038** (inject mode) · **DN-54** (`consume`/extension-checker) · **DN-63**
  (R2 runtime vocabulary) · **DN-65** (scoped PRs + workspace prep) · **DN-20** (test tiers).
- **PR #864** — the `mycelium-sched` extraction precedent for §2 A3.
- **Fable-5 research digs (2026-07-01)** — dependency-graph survey (`cargo metadata`,
  `Empirical`; spot-verified at HEAD: cert's dev-deps on `proj`/`spore`, `select`'s dev-dep on
  `cert`, `l1 → cert`, `mlir → std-runtime`), roadmap/architecture survey, open-work survey.
  Issue IDs (M-xxx) cited per those surveys; verify each against `tools/github/issues.yaml` at
  workstream kickoff (they are leads with `Empirical` provenance, not re-verified here one by one).
- Companion plans: `docs/planning/value-model-implementation-plan.md` (E20-1 detail),
  `docs/planning/Blocked-Decisions-Ratification-Map.md` (the decision batching this plan's FLAG
  table should be reconciled with), `docs/planning/dogfooding-effort-and-usage-assessment.md` +
  `docs/planning/self-hosting-port-ledger.md` (the E18-1 side of the handoff).

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
