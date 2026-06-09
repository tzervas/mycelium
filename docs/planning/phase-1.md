# Phase 1 — Minimal Viable Core (working plan)

| Field | Value |
|---|---|
| **Status** | **Living draft** (initial cut, 2026-06-09) |
| **Owns** | the concrete, issue-coupled expansion of Foundation §6 "Phase 1 — Minimal viable core (kernel)" |
| **Source of truth above this doc** | `docs/Mycelium_Project_Foundation.md` §6 (roadmap), `docs/spec/SPECIFICATION.md` §10 (open build items), `tools/github/issues.yaml` (task ids), RFC-0001…0005 + ADR-007…011 + DN-01 (design corpus, all Accepted/Resolved) |
| **Mirrors** | the GitHub board: every task here carries its issue number from `tools/github/idmap.tsv` |
| **Companion docs** | `phase-0.md` (predecessor, gate closed except KC-2); `phase-2.md`/`phase-3.md` (forthcoming) |

> **Grounding discipline.** This is a planning artifact, not a normative one. It cites the corpus
> (`FR/NFR/VR/SC/KC`, `RFC-xxxx §`, `ADR-0xx`, `Tx.y`, `G#`) for every claim about *what* is
> required; it does not introduce new requirements. Where it records a *decision about sequencing or
> scope* it says so explicitly and routes anything normative back to an RFC/ADR. The honesty rule
> applies to the gate verdicts below: a guarantee tag or kill-criterion verdict stays at the
> strength actually *established* by a checked run (VR-5), never pre-written.

---

## 1. What Phase 1 is for

Phase 1 ("Minimal viable core") turns the ratified design corpus and the Phase-0 data contracts
into a **small, auditable, executable kernel** (Foundation §6; KC-3). Its deliverables (Foundation
§6 Phase 1):

1. A **typed Core IR** — `Value<Repr, Meta>`, first-class `Bit`/`Trit`, the guarantee lattice, the
   bound vocabulary, the node grammar — with the honesty invariants enforced by construction
   (**M-101**), the guarantee `meet`-composition (**M-102**), content-addressed identity
   (**M-103**), and (de)serialization to the JSON contracts (**M-104**). *[Batch A — done.]*
2. A **reference interpreter** (executable small-step semantics; the trusted base, ADR-009/NFR-7),
   with binary + balanced-ternary execution and ≥2 inspectable lowering stages (**M-110/111/112**).
3. A **certified binary↔ternary swap** emitting a `LosslessWithinRange` certificate (**M-120**) with
   a machine-checked round-trip proof (**M-121**).
4. **One or two VSA ops** via the optional submodule (ADR-008) carrying attached, tagged bounds —
   the `Proven` capacity bound (carrying P0.1/KC-1 forward) and cleanup memory
   (**M-130/131/132**).
5. A **minimal toolchain surface** (FR-S5): an LSP skeleton, an invariant linter, a formatter
   (**M-140/141/142**), and the first MLIR→LLVM ternary-dialect skeleton + interp↔AOT differential
   (**M-150/151**).

### Phase-1 exit gate (what "done" means)

Phase 1 closes when **all** of:

- **Core IR complete** — M-101…M-104 landed, `fmt`/`clippy -D warnings`/`test` green on MSRV 1.92;
  Core IR round-trips through the ratified schemas and validates in CI (M-104). *[Met, 2026-06-09.]*
- **Interpreter is the reference** — M-110 executes the Core IR small-step (closing SPEC §10.3),
  with a golden corpus; balanced-ternary arithmetic property-tested vs. an oracle (M-111); ≥2
  dumpable/diffable lowering stages (M-112, SC-4).
- **Certified swap** — M-120 binary↔ternary swap emits a `Bijective`/`LosslessWithinRange`
  certificate and is `Exact` within range / `Option`-typed (never silent) out of range; M-121
  supplies the machine-checked round-trip proof (VR-1, SC-3).
- **Honest VSA op(s)** — M-130 `VsaModel` + MAP-I; M-131's capacity bound tagged `Proven` cites the
  M-001 theorem with a **checked instantiation** and ≥1e4-trial validation (SC-2, VR-5); M-132
  cleanup memory.
- **Toolchain surface** — M-140 LSP emits the four semantic-feedback artifact kinds (SC-5 channel);
  M-141 linter enforces *no implicit swap* / *no untagged bound*; M-142 formatter is a projection
  (identity-preserving, §4.6/§4.8).
- **Dual-path equivalence** — M-150 MLIR→LLVM ternary skeleton + M-151 interp↔AOT differential show
  observable equivalence on the kernel corpus (NFR-7).

Maps to Foundation §6 Phase-1 success metrics: SC-1, SC-3 (in-scope swaps), SC-4, SC-2 (shipped VSA
op), NFR-7 equivalence, and the SC-5 LSP channel.

---

## 2. The Phase-1 task set (readiness at a glance)

All Phase-1 issues, with issue number (`idmap.tsv`), priority, dependency, and **build readiness**.

| Task | Issue | Pri | Depends on | Maps to | Readiness |
|---|---|---|---|---|---|
| **M-101** Core IR data structures | [#11](https://github.com/tzervas/mycelium/issues/11) | P0 | M-010 | SPEC §10.2 / RFC-0001 §4.1–§4.6 | **Done (2026-06-09)** — 17 tests; invariants by construction |
| **M-102** Guarantee lattice + `meet` | [#12](https://github.com/tzervas/mycelium/issues/12) | P0 | M-101 | RFC-0001 §3.4/§4.7 | **Done (2026-06-09)** — §6.1; laws exhaustive over 4×4(×4) |
| **M-103** Content-addressing | [#13](https://github.com/tzervas/mycelium/issues/13) | P1 | M-101 | RFC-0001 §4.6 / ADR-003 | **Done (2026-06-09)** — §6.2; BLAKE3 hash-of-AST + `Names` |
| **M-104** Core IR (de)serialization | [#14](https://github.com/tzervas/mycelium/issues/14) | P1 | M-101, M-010 | RFC-0001 §4.8 | **Done (2026-06-09)** — §6.3; round-trip + schema-pinned examples |
| **M-110** Reference interpreter | [#15](https://github.com/tzervas/mycelium/issues/15) | P0 | M-101 | SPEC §10.3 / RFC-0004 §2 / ADR-009 | **Ready** — Batch A complete; the next keystone (§4) |
| **M-111** Bit/Trit + balanced-ternary arithmetic | [#16](https://github.com/tzervas/mycelium/issues/16) | P0 | M-110 | RFC-0004 / `binary-ternary.md` §1 | **Done (2026-06-09)** — `core::ternary` codec + neg/add/sub/mul; exhaustive `i64`-oracle property tests; `trit.*` prims |
| **M-112** ≥2 inspectable lowering stages | [#17](https://github.com/tzervas/mycelium/issues/17) | P1 | M-110 | SPEC §10.5 / RFC-0004 §5 / SC-4 | **Done (2026-06-09)** — `core::lower`: `core` → ANF `substrate` + scheduled layouts; dumpable/diffable |
| **M-120** Binary↔ternary swap | [#18](https://github.com/tzervas/mycelium/issues/18) | P0 | M-101 | `binary-ternary.md` / RFC-0002 §4 | **Done (2026-06-09)** — `mycelium-cert` enc/dec + `Bijective` cert; exhaustive `dec(enc x)` over all 256 bytes; interp `SwapEngine` |
| **M-121** Machine-checked round-trip proof | [#19](https://github.com/tzervas/mycelium/issues/19) | P0 | M-120 | RFC-0002 §4 (P1/P2) / VR-1 | **Done (2026-06-09)** — `proofs/binary-ternary-roundtrip/`; Z3 `unsat` (injectivity, 8↔6) |
| **M-130** `VsaModel` trait + MAP-I | [#20](https://github.com/tzervas/mycelium/issues/20) | P0 | M-101 | RFC-0003 / ADR-008 | **Done (2026-06-09)** — `mycelium-vsa`: trait + MAP-I bind/unbind/permute (Exact) + bundle; dependency-gated |
| **M-131** `Proven` capacity bound + validation | [#21](https://github.com/tzervas/mycelium/issues/21) | P0 | M-130, M-001 | RFC-0003 §5 / SC-2 / KC-1 | **Done (2026-06-09)** — checked-instantiation `Proven` `CapacityBound`; ≥10⁴-trial validation ≤ δ |
| **M-132** Cleanup memory | [#22](https://github.com/tzervas/mycelium/issues/22) | P2 | M-130 | RFC-0003 / FR-S4 | **Done (2026-06-09)** — `CleanupMemory` nearest-neighbour item memory → `(label, confidence, margin)` |
| **M-140** LSP skeleton | [#23](https://github.com/tzervas/mycelium/issues/23) | P1 | M-110 | FR-S5 / SC-5 | **After interpreter** |
| **M-141** Invariant linter | [#24](https://github.com/tzervas/mycelium/issues/24) | P1 | M-101 | FR-S5 / RFC-0001 WF1/WF2 | **Done (2026-06-09)** — `mycelium-lsp::lint`: implicit-swap / unverified-bound / placeholder-policy / free-variable |
| **M-142** Formatter | [#25](https://github.com/tzervas/mycelium/issues/25) | P2 | M-104 | §4.8 SC-4 | **Ready** — projection, identity-preserving |
| **M-150** MLIR→LLVM ternary dialect skeleton | [#26](https://github.com/tzervas/mycelium/issues/26) | P1 | M-110 | RFC-0004 / ADR-007 | **After interpreter** |
| **M-151** interp↔AOT differential | [#27](https://github.com/tzervas/mycelium/issues/27) | P0 | M-110, M-150 | NFR-7 / ADR-009 | **After M-150** |

Legend — **Ready**: can start now from the corpus + landed deps. **Ready after X**: a hard
dependency is open. **Done**: landed, tests green, issue closed.

---

## 3. Batch structure (the parallelization plan)

Phase 1 is sequenced into four batches; tasks **within** a batch touch different modules/crates and
parallelize, while batches serialize on real dependencies. This mirrors the working-style mandate
(parallelize aggressively but safely).

- **Batch A — Core IR completion** (all in `mycelium-core`): **M-102, M-103, M-104**. *Done
  2026-06-09* — independent modules (`guarantee`, `content`, the `serde` layer), developed together
  atop M-101's `GuaranteeStrength::rank`, the `Node` grammar, and the ratified schemas.
- **Batch B — execution** (depends on A): **M-110** reference interpreter → then **M-111**
  (arithmetic) and **M-112** (lowering) in parallel. M-110 is the keystone; it also unblocks the
  long-deferred **M-002 (#3) KC-2 experiment**, which needs a fragment parser/type-checker to score
  type-check pass rate.
- **Batch C — swaps & VSA** (two parallel tracks, each depends only on A): **M-120 → M-121**
  (certified swap + proof) and **M-130 → M-131 → M-132** (VSA model, `Proven` bound, cleanup).
- **Batch D — toolchain & backend** (depends on B): **M-140/141/142** (LSP/linter/formatter; M-141
  and M-142 can start off A) and **M-150 → M-151** (MLIR skeleton + interp↔AOT differential).

---

## 4. Critical path & sequencing

```
 Batch A (mycelium-core, DONE 2026-06-09)
   M-101 ──► M-102 meet ─┐
        ├──► M-103 hash ─┤  (independent modules, developed in parallel)
        └──► M-104 serde ┘
                 │
   CRITICAL PATH ▼
   M-110 reference interpreter  ── keystone: trusted base (ADR-009/NFR-7) ──┐
        ├──► M-111 Bit/Trit + balanced-ternary arithmetic                   │
        ├──► M-112 ≥2 inspectable lowering stages (SC-4)                    │
        ├──► M-140 LSP skeleton ─► (SC-5 channel)                           │
        └──► M-150 MLIR→LLVM ternary skeleton ─► M-151 interp↔AOT diff (NFR-7)

   PARALLEL TRACKS (depend only on Batch A, not on M-110):
   M-120 bin↔tern swap (LosslessWithinRange) ─► M-121 round-trip proof (VR-1)
   M-130 VsaModel + MAP-I ─► M-131 Proven capacity bound (SC-2, cites M-001) ─► M-132 cleanup
   M-141 invariant linter   M-142 formatter   (start off Batch A)
```

**Why M-110 is the keystone.** The interpreter *is* the reference semantics (ADR-009, NFR-7):
M-111's arithmetic is validated against it, M-112's lowering stages must preserve its observable
behaviour, M-151's AOT path is differential-tested against it, and the M-140 LSP surfaces its
diagnostics. It also retires the last Phase-0 blocker (M-002/KC-2) by giving the experiment a
type-checker to measure. So M-110 is started first after Batch A; the swap and VSA tracks proceed in
parallel since they need only the Core IR.

---

## 5. Gate verdicts — honest status (KC-1…KC-4)

Per the honesty rule and VR-5, kill-criterion status is tracked at the strength actually
*established*. Re-run KC-1…KC-4 at the Phase-1 gate (Foundation Meta).

| Gate | Question | Current status | What moves it in Phase 1 |
|---|---|---|---|
| **KC-1** | Honest, usefully-tight bound for a core VSA op? | ✅ **confirmed (build)** 2026-06-09 (M-001 LH probe SAFE) | M-131 instantiates it as a `Proven` capacity bound in code with a *checked* side-condition + ≥1e4-trial validation (SC-2) — the in-kernel realization, not a re-upgrade. |
| **KC-2** | LLM code-gen/reasoning survives the Mycelium surface? | **open** — research did not settle it (RR-3) | Unblocked *operationally* by M-110 (a parser/type-checker to score against). M-002 (#3) then produces the SC-5b baseline + proceed/reweight/fall-back verdict. |
| **KC-3** | Kernel stays single-expert auditable? | **holding** — `mycelium-core` is small + by-construction-correct | Re-assess at the gate: interpreter + swap + one VSA op must not balloon the kernel; keep VSA behind the ADR-008 submodule boundary. |
| **KC-4** | Per-swap certificate-check overhead within budget? | **n/a yet** (Phase-2 concern; first swap lands M-120/M-121) | Out of Phase-1 scope to *budget*; M-121 establishes the proof exists. Carried to Phase 2. |

**KC-1 honesty note (carried).** The literature pass justified the `Proven` *tag*; M-001 confirmed
the *strategy* (axiomatize the cited theorem, Z3-discharge the arithmetic instantiation). M-131 must
likewise tag `Proven` only with the cited theorem **and** its side-conditions checked at the call
site (RFC-0003 §5; ADR-010/011); otherwise it falls back to `Empirical` (Frady-Sommer trials).

---

## 6. Per-task detail (completed Batch A)

### 6.1 M-102 — Guarantee lattice + meet-composition  ·  #12 · P0 · done 2026-06-09

- **Goal / acceptance (from issue).** `meet` (weakest-wins) with assoc/comm/idempotent laws
  property-tested; ops propagate by `meet`; all 4×4 pairs tested (RFC-0001 §3.4/§4.7).
- **Delivered.** `GuaranteeStrength::meet` (greatest-lower-bound = larger `rank`), `propagate`
  (`meet(inputs…, g_f)`) and `meet_all`, plus `TOP`/`ALL`. Laws verified by **exhaustion** over all
  4×4 pairs and 4×4×4 triples (commutativity, associativity, idempotence, identity `Exact`,
  `Declared`-absorbing) — complete for the finite lattice, stronger than sampling. 9 tests.
- **Honesty.** Composition can only *degrade* strength (VR-3/VR-5); no path upgrades a guarantee.

### 6.2 M-103 — Content-addressing  ·  #13 · P1 · done 2026-06-09

- **Goal / acceptance (from issue).** Hash-of-AST identity + names-as-metadata; identical defs
  collide, trivial renames don't (RFC-0001 §4.6; ADR-003).
- **Delivered.** `Node::content_hash`/`Value::content_hash` — **BLAKE3** over an injective,
  domain-separated, length-prefixed encoding of the *identity-bearing* content: α-normalized
  structure (bound vars → de Bruijn indices, binder names dropped), types-with-`Repr`, constant
  literals, operator names, swap target+policy. Dynamic `Meta` excluded. A separable `hash ↔ name`
  table (`Names`) embodies names-as-metadata. 10 tests cover collision, α-rename invariance,
  metadata-exclusion, and paradigm/precision/literal/operator sensitivity.

### 6.3 M-104 — Core IR (de)serialization  ·  #14 · P1 · done 2026-06-09

- **Goal / acceptance (from issue).** (De)serialize Core IR to/from the JSON schemas; round-trip
  property test; output validates against `value.schema.json` in CI (RFC-0001 §4.8).
- **Delivered.** `serde` impls emitting *exactly* the ratified contracts (`kind`/`class`/`layout`
  tags; `VSA`/`BF16`/`TL1`/`TL2` renames; `payload` as `{bits|trits|scalars|hypervector}` with
  MSB-first bit/trit strings per `binary-ternary.md`; `bound` by presence; flat `kind`+`basis`
  `Bound`). `Deserialize` for `Value`/`Meta` routes through the checked constructors, so M-I1…M-I4
  and payload↔repr mismatches are rejected on the wire — never silently accepted. Faithful
  round-trip is tested over all four paradigms × every guarantee/bound/basis/layout; serializer
  output is **pinned** to three new committed `value` examples that `scripts/checks/schema.sh`
  validates against `value.schema.json` in CI. 8 integration tests + the round-trip corpus.
- **Honesty.** The "never silent" rule (G2/SC-3) extends to deserialization: malformed wire data is
  an explicit error, not a coerced value.

---

## 7. Risks & open questions

| Id | Item | Disposition |
|---|---|---|
| **OQ-6** | Concrete `payload` wire encoding (unconstrained in `value.schema.json`). | **Resolved by M-104 (2026-06-09)** — an externally-tagged `bits`/`trits`/`scalars`/`hypervector` object; bits/trits as MSB-first strings over the glyphs `0`/`1` and `−`/`0`/`+` (aligns with `binary-ternary.md` §1). A future paradigm-specific payload schema may tighten this; it is forward-compatible. |
| **OQ-7** | Kernel hash algorithm for content-addressing. | **Resolved by M-103 (2026-06-09)** — **BLAKE3**, rendered `blake3:<64-hex>`; `ContentHash` stays algorithm-agnostic so migration is a value change, not a type change. |
| **OQ-8** | Should `serde`/`serde_json` be a kernel dependency or feature-gated? | **Decision (2026-06-09):** non-optional `serde` dep (round-trip is a core M-104 requirement); `serde_json` is a dev-dependency (tests). Revisit only if a no-`std`/no-`serde` consumer appears. |
| **RR-12** | Dual-path semantic divergence (interpreter vs. AOT). | The Phase-1 headline risk (Foundation §6). M-151's interp↔AOT differential is its circuit-breaker (NFR-7); the interpreter (M-110) is the sole reference. |
| **G8** | VSA/float kernel opacity blocks proofs. | Confine the unprovable to tagged `Empirical` bounds (VR-5); keep `Proven` only where M-001's checked-instantiation pattern applies (M-131). |
| **KC-3** | Integrative complexity → un-auditable kernel. | Hold the line: VSA behind the ADR-008 submodule; the interpreter is small-step and inspectable; no black boxes (G2). Re-run KC-3 at the gate (§5). |

---

## 8. How this doc stays honest

- **Append-only with status transitions**, mirroring the ADR/RFC discipline (Foundation Meta): this
  file moves `Living draft → ratified` only when the Phase-1 exit gate (§1) is met; task rows update
  in place as their issues progress, but gate verdicts (§5) never pre-record an upgrade.
- **Every task row carries its issue number** (`idmap.tsv` is the join key) so the board and this
  doc cannot silently diverge.
- **Progress is reported back to the issues**, not only here — each task's substantive output links
  its artifact from the GitHub issue and the issue is closed when its acceptance is met (or left
  open with an honest note if blocked).

---

## Meta — changelog & maintenance

- **2026-06-09 (initial draft):** first issue-coupled expansion of Foundation §6 Phase 1. Records
  the readiness table (§2), the batch/parallelization plan (§3), the critical path with M-110 as
  keystone (§4), honest KC-1…KC-4 status (§5), and per-task detail for the completed Batch A
  (M-102/M-103/M-104, §6). Open questions OQ-6/OQ-7/OQ-8 recorded as resolved by Batch A.
- Maintain append-only; supersede, don't rewrite. Re-run KC-1…KC-4 at the phase gate (Foundation
  Meta). Keep `Proven|Empirical|Declared` verdicts honest per VR-5.
