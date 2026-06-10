# Mycelium — Language & Substrate Specification (skeleton)

| Field | Value |
|---|---|
| **Status** | **ratified-skeleton** (M-011, #6) — §1–§9 reconciled to the Accepted corpus; every §10 item linked to a live issue |
| **Date** | 2026-06-09 |
| **Normative sources** | RFC-0001 (r2), RFC-0002, RFC-0003, RFC-0004, RFC-0005; ADR-001…011; DN-01; Foundation (r3) |
| **Machine-readable contracts** | `docs/spec/schemas/*.schema.json` (ratified, M-010) |
| **Supersedes** | nothing (first consolidation) |

## 0. How to read this document

This SPECIFICATION is an **index and consolidation layer**, not a re-derivation. The **RFCs remain
the normative source**; each section here states the rule in one or two sentences and points at the
authoritative RFC section and the machine-readable JSON-Schema contract. Where this document and an
RFC ever disagree, **the RFC wins** and the divergence is a bug to be fixed (per the append-only
discipline — supersede, don't rewrite). The goal (DRY) is a single navigable map a reader or an AI
agent can use to find the precise contract without re-reading the whole corpus.

**Status discipline.** `consolidating-draft → ratified-skeleton → ratified`. This is the
`ratified-skeleton`: §1–§9 confirmed against their RFCs and §10 fully issue-linked. It becomes
`ratified` (full) when the §10 build items close and their concrete grammars/semantics (Core IR
EBNF, interpreter small-step rules, lowering stages) land and are folded in.

**The non-negotiables (Foundation; CONTRIBUTING).** Honesty rule (per-op guarantee tags on the
lattice `Exact ⊐ Proven ⊐ Empirical ⊐ Declared`); no black boxes (every swap/selection/approximation
is reified and `EXPLAIN`-able, never silent); append-only decisions; ground every claim; small
auditable kernel (KC-3).

---

## 1. Scope & paradigms

Mycelium is a unified value-semantics substrate over four **closed paradigm kinds** —
`Binary`, `Ternary`, `Dense`, `VSA` — with certified, never-silent representation swaps and honest,
per-operation guarantees. The four kinds are closed in the kernel (a fifth needs an RFC + ADR);
the *parameter* registries (`ScalarKind`, `ModelId`, `PackScheme`, `NormKind`) are open.

- **Normative:** RFC-0001 §1, §3.1, §4.1; Foundation §2.
- **Contract:** [`repr.schema.json`](schemas/repr.schema.json).

## 2. Value model

A value is `{ repr, payload, meta }`: a representation descriptor, a representation-specific
payload, and runtime metadata. The static type is `Value<R: Repr>`; `meta` is runtime data. The
wire form `[Repr] ‖ [Meta] ‖ [payload]` is faithfully round-trippable
(`deserialize(serialize(v)) ≡ v`, including `Meta`).

- **Normative:** RFC-0001 §3.1, §4.2, §4.8.
- **Contracts:** [`value.schema.json`](schemas/value.schema.json), [`repr.schema.json`](schemas/repr.schema.json).

## 3. Metadata, the guarantee lattice & bounds

`Meta` carries `provenance`, `guarantee`, an optional `bound`, measured `sparsity`, a `physical`
layout record, optional `reconstruction` info, and `policy_used`. **Honesty is typed and monotone:**
`guarantee ∈ {Exact, Proven, Empirical, Declared}` (lattice `Exact ⊐ Proven ⊐ Empirical ⊐ Declared`),
and every operation's result takes the **meet** of its inputs' strengths and its own intrinsic
strength — disclosure can only degrade. Every `Bound` records the **basis** by which it was obtained
(`ProvenThm | EmpiricalFit | UserDeclared`); the strength tag is *derived from the basis*, never
asserted (ADR-011).

**Schema invariants (normative; RFC-0001 §4.3).**

- **M-I1.** `guarantee == Exact ⟺ bound == None`.
- **M-I2.** `Proven ⟹ bound.basis == ProvenThm` (side-conditions checked).
- **M-I3.** `Empirical ⟹ bound.basis == EmpiricalFit`.
- **M-I4.** `Declared ⟹ bound.basis == UserDeclared`, and tooling MUST surface a "declared,
  unverified" marker.
- **M-I5.** `physical` is a *lossless* re-encoding of `payload`; it changes neither type nor guarantee.

Bound composition uses ADR-010's two kernels (ε via affine arithmetic; δ via union-bound/apRHL)
meeting at one shared `{ε, δ, strength}` certificate; `strength` composes by meet.

- **Normative:** RFC-0001 §3.4, §4.3, §4.7; ADR-010; ADR-011.
- **Contracts:** [`meta.schema.json`](schemas/meta.schema.json), [`guarantee.schema.json`](schemas/guarantee.schema.json), [`bound.schema.json`](schemas/bound.schema.json), [`physical-layout.schema.json`](schemas/physical-layout.schema.json).

## 4. Core IR & typing discipline

The Core IR is the typed, content-addressed, metadata-bearing single source of truth. Node subset:
`Const | Var | Let | Op{prim,args} | Swap{src,target,policy}`. Typing is `Γ ⊢ e : Value<R>` with
**no coercion/subsumption rule between paradigms**: an `Op` is well-typed only if each argument's
`Repr` matches the prim's declared operand paradigm exactly. `Swap` is the **only** node that
changes a value's `Repr`.

**Well-formedness invariants (the mechanical "no black boxes"; RFC-0001 §4.5).** WF1 every `Repr`
change is a `Swap`; WF2 every `Swap` carries a `PolicyRef`; WF3 every non-`Exact` result carries a
consistent `bound`; WF4 every node is content-addressable; WF5 lowering preserves `Meta` semantics.

- **Normative:** RFC-0001 §4.4, §4.5.
- **Core node grammar:** committed in `mycelium-core::node` (M-101); RFC-0001 §4.5.
- **Open:** the full term language (abstraction, recursion, modules) — see §10.2.

## 5. Content-addressing & identity

Definition identity is the hash of normalized structure ‖ types-with-`Repr` ‖ static contract
(Unison-style). Human names, source spans, formatting, and *all dynamic value metadata* are **not**
hashed; names are a separate `hash ↔ name` map, so renaming/reformatting preserves identity while a
paradigm change does not. Provenance references are content hashes forming an acyclic derivation DAG.

- **Normative:** RFC-0001 §4.6; ADR-003.
- **Contract:** [`provenance.schema.json`](schemas/provenance.schema.json).

## 6. Swaps, certificates & the split regime

A `Swap` yields a target-paradigm value **and** a `SwapCertificate` — **never silent** (SC-3).
Two forms: `Bijective` (references a once-per-swap-kind bijectivity lemma + concrete params,
cacheable by content hash) and `Bounded` (carries a `Bound` + basis + the `PolicyRef`). The
binary↔ternary swap is the genuinely bijective class: `LosslessWithinRange` — `enc : Bin_n → Tern_m`
total, `dec : Tern_m → Option Bin_n`, `Exact` within range, out-of-range an explicit `Option`/error.
A pair with no statable bound is a **type error**, not a `Declared` gamble. The certificate format
and its checker are shared with the interpreter-vs-compiled equivalence check (§8).

- **Normative:** RFC-0002 (all); ADR-002. Precise binary↔ternary `enc`/`dec`: [`swaps/binary-ternary.md`](swaps/binary-ternary.md) (M-012, ratified).
- **Contract:** [`swap-certificate.schema.json`](schemas/swap-certificate.schema.json).
- **Open:** the binary↔ternary *implementation* + round-trip proof (§10.4); the shared certificate checker (§10.8).

## 7. VSA submodule

VSA is **in the core semantics** (the `VSA` `Repr` kind, the `Hypervector` slot, its metadata, the
swap machinery, the `ModelId` registry) but its **operational algebra** is a dependency-gated
submodule (ADR-008, KC-3): a kernel built without it type-checks programs that *mention*
hypervectors but offers no operations. Each model implements a composition-style `VsaModel` trait
supplying `bind/unbind/bundle/permute/similarity/cleanup` with **honest per-model × per-operation
guarantee tags** (RFC-0003 §4): MAP/BSC `bind`/`unbind` and all `permute` are `Exact`/algebraic;
`bundle` is `Proven` for MAP-I/sparse/on-expectation-BSC and `Empirical` for HRR/FHRR; **HRR/FHRR
`unbind` is the residual `Empirical` weak link.** Declared sparsity class is a static refinement;
capacity bounds are axiomatized-theorem + checked-instantiation (the M-001 confirming probe).
Reconstruction distinguishes indexed retrieval from true compositional reconstruction.

- **Normative:** RFC-0003 (all); ADR-008; ADR-010.
- **Contract:** [`reconstruction-manifest.schema.json`](schemas/reconstruction-manifest.schema.json).
- **Open:** `VsaModel` + MAP-I + tagged bounds + cleanup (§10.6); manifest impl (§10.10).

## 8. Execution model

Pipeline: `Surface → Core IR → Substrate IR → Backend`, every arrow dumpable and diffable (SC-4),
metadata persisting across all of them (WF5). The **Rust interpreter is the reference semantics and
trusted base**; the **MLIR→LLVM** path (a `ternary` dialect first) is the AOT performance path. A
definition is a **"stable component"** — eligible for AOT — only if it is content-addressed, has a
ratified spec, and has passed its verification gate; every AOT/JIT lowering is validated against the
interpreter (NFR-7, ADR-009). **Lossless physical packing is schedule-staged**, not in the type:
chosen at a lowering stage from the fixed bitnet.cpp set and *recorded* in `Meta.physical` (DN-01).

- **Normative:** RFC-0004 (all); ADR-007, ADR-009; DN-01.
- **Open:** inspectable lowering ≥2 stages (§10.5); MLIR→LLVM skeleton + interp↔AOT differential (§10.5);
  the shared certificate checker (§10.8); packing selector (§10.8 / Phase 2).

## 9. Selection-policy language

Automatic representation/packing selection is a **total, non-learned, content-addressed
decision-table policy** over inspectable `Meta` to a finite candidate set with an explicit cost
function — deliberately not Turing-complete. Every automatic selection emits a **mandatory EXPLAIN**
record `{inputs considered, per-candidate cost, chosen option, override hook}`. One mechanism serves
two sites: swap-target selection (§6) and packing-schedule selection (§8). Mycelium avoids the DB
cardinality-estimation opacity because its statistics are *exact metadata*, not sampled estimates.

- **Normative:** RFC-0005 (all); ADR-006.
- **Contract:** [`policy.schema.json`](schemas/policy.schema.json).
- **Open:** the concrete predicate grammar + EXPLAIN impl (§10.9).

---

## 10. Open build items (each linked to a live issue)

> Every TODO points at a tracked issue (`tools/github/idmap.tsv`). No floating TODOs. When an item
> lands, fold its concrete grammar/semantics into §1–§9 and flip the relevant text from
> "Open" to normative; that is the path to full `ratified` status.

| § | Open item | Issue(s) | Phase |
|---|---|---|---|
| 10.1 | Minimal surface-syntax fragment — ✅ [`experiments/surface-fragment/`](../../experiments/surface-fragment/README.md) (M-020, throwaway/experiment-only) | M-020 ([#4](https://github.com/tzervas/mycelium/issues/4)) | 0 |
| 10.2 | Core IR node grammar — ✅ committed (EBNF in `mycelium-core::node`, RFC-0001 §4.5; M-101); full term language (abstraction/recursion/modules) now under deliberation → [RFC-0006](../rfcs/RFC-0006-Surface-Language-and-Term-Layering.md) (Draft; concrete syntax KC-2-gated) | M-101 ([#11](https://github.com/tzervas/mycelium/issues/11)) | 1 |
| 10.3 | Reference-interpreter small-step operational semantics — ✅ committed (the `e ⟶ e'` rules E-Let-Bind/Step, E-Op-Arg/Apply, E-Swap-Arg/Apply documented in `mycelium-interp` crate docs; call-by-value substitution semantics + honest metadata threading; golden corpus, M-110). Arithmetic `δ` is M-111; certified swap `σ` is M-120 | M-110 ([#15](https://github.com/tzervas/mycelium/issues/15)) | 1 |
| 10.4 | Binary↔ternary swap impl + machine-checked round-trip — ✅ committed: `enc`/`dec` + `Bijective` certificate in `mycelium-cert` (M-120; exhaustive `dec(enc x)` over all bytes), and the Z3-discharged injectivity proof in [`proofs/binary-ternary-roundtrip/`](../../proofs/binary-ternary-roundtrip/README.md) (M-121). Encoding spec ✅ [`swaps/binary-ternary.md`](swaps/binary-ternary.md) (M-012) | M-120 ([#18](https://github.com/tzervas/mycelium/issues/18)), M-121 ([#19](https://github.com/tzervas/mycelium/issues/19)) | 1 |
| 10.5 | Inspectable lowering — ✅ ≥2 dumpable/diffable stages (`mycelium-core::lower`: `core` → A-normal-form `substrate` with scheduled `PhysicalLayout`; SC-4/WF5, M-112); ✅ textual ternary-dialect emitter + runnable AOT artifact over the lowered IR (`mycelium-mlir`, M-150; native libMLIR/LLVM codegen deferred); ✅ interp↔AOT differential on the kernel corpus (NFR-7, M-151) | M-112 ([#17](https://github.com/tzervas/mycelium/issues/17)), M-150 ([#26](https://github.com/tzervas/mycelium/issues/26)), M-151 ([#27](https://github.com/tzervas/mycelium/issues/27)) | 1 |
| 10.6 | `VsaModel` trait + MAP-I + tagged `bundle` bound + cleanup | M-130 ([#20](https://github.com/tzervas/mycelium/issues/20)), M-131 ([#21](https://github.com/tzervas/mycelium/issues/21)), M-132 ([#22](https://github.com/tzervas/mycelium/issues/22)) | 1 |
| 10.7 | Verified-numerics checker (two kernels + shared certificate) | E2-4 ([#31](https://github.com/tzervas/mycelium/issues/31)) | 2 |
| 10.8 | Full swap + the single shared certificate checker | E2-3 ([#30](https://github.com/tzervas/mycelium/issues/30)) | 2 |
| 10.9 | Selection-policy predicate grammar + EXPLAIN impl | E2-6 ([#33](https://github.com/tzervas/mycelium/issues/33)) | 2 |
| 10.10 | Reconstruction-manifest implementation | E2-5 ([#32](https://github.com/tzervas/mycelium/issues/32)) | 2 |
| 10.11 | Toolchain surface — ✅ committed in `mycelium-lsp`: invariant linter (M-141), α-normalizing canonical formatter (M-142), and the LSP feedback facade (M-140) exposing the four artifact kinds (diagnostics, swap certificates, bound/guarantee annotations, lowering-stage dumps) over one surface | M-140 ([#23](https://github.com/tzervas/mycelium/issues/23)), M-141 ([#24](https://github.com/tzervas/mycelium/issues/24)), M-142 ([#25](https://github.com/tzervas/mycelium/issues/25)) | 1 |
| 10.12 | Confirming Liquid-Haskell `bundle` capacity probe (ratifies KC-1) | M-001 ([#2](https://github.com/tzervas/mycelium/issues/2)) | 0 |
| 10.13 | KC-2 LLM-leverage experiment (verdict) | M-002 ([#3](https://github.com/tzervas/mycelium/issues/3)) | 0 |

**Note (naming).** Issue E2-5 (#32) refers to the reconstruction schema as `recon-info.schema.json`;
the ratified file is [`reconstruction-manifest.schema.json`](schemas/reconstruction-manifest.schema.json)
(M-010). The ratified name is authoritative; the issue text can be reconciled when E2-5 is decomposed.

---

## Meta — changelog & maintenance

- **2026-06-09 (ratified-skeleton):** initial consolidation (M-011, #6). §1–§9 reconciled to
  RFC-0001 (r2) / RFC-0002…0005 / ADR-010/011 / DN-01 and pointed at the ratified
  `docs/spec/schemas/` contracts (M-010); §10 enumerates the open build items, each linked to a live
  issue. Status `consolidating-draft → ratified-skeleton`.
- **2026-06-09 (§10.3 lands):** M-110 (#15) — the reference-interpreter small-step semantics are
  committed (`mycelium-interp`): the `e ⟶ e'` rules, call-by-value substitution, honest guarantee/
  provenance threading, and a golden corpus. §10.3 flipped to ✅. (Balanced-ternary arithmetic is
  M-111; the certified binary↔ternary swap is M-120.)
- **2026-06-09 (§10.4 lands):** M-120 (#18) + M-121 (#19) — the binary↔ternary `enc`/`dec` with a
  `Bijective`/`LosslessWithinRange` certificate (`mycelium-cert`, exhaustive round-trip), and its
  Z3-discharged injectivity proof (`proofs/binary-ternary-roundtrip/`, `unsat`). §10.4 flipped to ✅.
  Balanced-ternary arithmetic (`mycelium-core::ternary`, oracle-tested) landed alongside (M-111).
- Maintain append-only with status transitions, mirroring the RFC/ADR discipline. The RFCs stay
  normative; this index is folded forward, never used to override them.
