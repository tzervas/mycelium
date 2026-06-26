# Corpus-Alignment Audit — RECORD

**Date:** 2026-06-25
**Scope:** Exhaustive read-only alignment audit of the entire Mycelium design corpus — **92 documents** (all DNs, ADRs, RFCs).
**Posture:** Read-only. This audit reports drift between the corpus and the landed tree; it **enacts nothing** and applies no corrections (recommendations only, §6).
**Honesty lattice:** `Exact ⊐ Proven ⊐ Empirical ⊐ Declared`. Guarantee strength is never upgraded past a checked basis (VR-5). All findings preserve the documents' own tags.
**Ground-truth rule:** Where a document and the code/lexicon disagree, **the code and lexicon are treated as ground truth** and the document is recorded as drifted. Per house-rule 3 (append-only), drift is reconciled by an append-only note/supersession — *never* by rewriting decision history.

---

## 1. Executive Summary

**Documents audited:** 92 (per-doc fan-out, one auditor per doc).

**Findings by severity:**

| Severity | Count |
|---|---|
| Critical | 0 |
| High | 8 |
| Medium | 7 |
| Low | 37 |
| **Total** | **52** |

**Documents by status-verdict:**

| Verdict | Count |
|---|---|
| honest | 76 |
| internally-inconsistent | 6 |
| stale | 6 |
| over-claims | 3 |
| under-claims | 0 |
| **Total** | **92** |

**Headline issues:**

- **ADR-018 over-claims a false load-bearing fact** (two High findings): asserts every crate is pinned `0.0.0` with per-manifest `publish = false`, but three crates (`mycelium-std-math`, `-sys`, `-sys-host`) are at `0.1.0` and carry no `publish` key. Distribution is still blocked at the workspace level (`release-plz.toml`), so the policy holds, but the Enacted premise is untrue and suggests a partial undocumented release-cut.
- **ADR-020 jumped to Enacted without recording the Accepted→Enacted append-only transitions** (one High): the sole changelog entry still reads "Proposed … awaiting ratification" while the header says Enacted (house-rule 3 violation). The underlying `mycelium-std-runtime` crate genuinely landed, so the defect is procedural, not fabricated.
- **DN-23 "current state" is factually false against landed code** (two High): claims there are no symbolic infix operators, but the lexer tokenizes `+ - * / %`; RFC-0025 (framed as "forthcoming") exists and is implemented Rust-first. No changelog footer reconciles the drift.
- **DN-16 and DN-17 are stale planning surveys** (two High each): DN-16 contradicts itself on `runtime` ("no crate" vs "ratification-ready") and names a now-closed `sys.md` blocker; DN-17's P1 premises ("no `[workspace.dependencies]`", "xtask path-deps lack version") are already done in the tree.
- **DN-19 spine is superseded** (one High): treats ADR-021 as "Proposed, needs ratification", but ADR-021 reached Accepted then was Superseded by ADR-022; DN-19 never mentions ADR-022.
- **DN-06 contradicts itself** on whether the M-358 `colony→nodule` migration executed (Low, but internally-inconsistent): Status/§2/final-changelog say "not yet executed", §4 table + code say executed.
- **Six "Accepted" ADRs (029/030/031) carry stale in-body `## Status: Proposed` blocks** contradicting their ratified headers — a hygiene cluster from the 2026-06-24 ADR wave.
- **RFC-0017 "Enacted" over-claims §4.1**: the scope-maturation gate + thaw are built, but the normative top-down `@matured` header/manifest inheritance and the §4.4 maturation record are explicitly deferred in code (`resolve.rs`).

No VR-5 guarantee-tag upgrade violations were found anywhere in the corpus. Every drift is a status/fact/locator staleness issue, not a strength over-claim.

---

## 2. Ranked Drift Table

### 2.1 Critical findings

*None.*

### 2.2 High findings (all)

| Doc | Sev | Summary | doc_loc | Evidence |
|---|---|---|---|---|
| ADR-018 | High | FALSE FACT: asserts every crate pinned at `0.0.0`, but three crates are `0.1.0`. Load-bearing premise of the Enacted claim. | Context §1; changelog 2026-06-23 Enacted | `crates/mycelium-std-math|-sys|-sys-host/Cargo.toml` version=`0.1.0`; all others `0.0.0`. |
| ADR-018 | High | FALSE FACT (VR-5 grounding): claims `publish = false` in every crate Cargo.toml; three crates lack the key. Workspace guard in `release-plz.toml` still blocks publish. | Context §2; Grounding §; changelog Enacted | The same 3 crates have no `publish` key (47/50 do); `release-plz.toml` carries the workspace-level guard. |
| ADR-020 | High | Status header says Enacted, but the only changelog entry is "Proposed … Awaiting maintainer ratification". No Accepted/Enacted append-only entries — house-rule 3 violation. | Status (line 7) vs Meta-changelog (line 231) | Sole entry `2026-06-20 — Proposed`; grep finds no dated Accepted/Enacted entry; underlying `mycelium-std-runtime` crate genuinely landed (procedural defect, not fabricated). |
| DN-16 | High | §3.16 declares `runtime` has "no crate / not-yet-implemented", but `mycelium-std-runtime` exists and the doc's own re-audit lists it ratification-ready — self-contradiction. | §2 table (runtime) + §3.16 vs re-audit (~line 492) | `crates/mycelium-std-runtime/src/lib.rs` exists. |
| DN-16 | High | Names `mycelium-std-sys` "NEEDS-WORK / the one ratification blocker" for lacking `docs/spec/stdlib/sys.md`; that spec now exists. | Re-audit "Actionable items" | `docs/spec/stdlib/sys.md` exists; the stated single blocker is closed; 24/25 tally stale. |
| DN-17 | High | §2.1 P1 premise false: "No `[workspace.dependencies]` exists in root Cargo.toml." One now exists; the proposed dedup appears already done. | §2.1 "Workspace external-dependency duplication — P1" | `Cargo.toml:67 [workspace.dependencies]`; e.g. `mycelium-spore` uses `blake3.workspace = true`. |
| DN-17 | High | §2.2/§4 P1 "xtask path-deps lack a version field" stale: the 5 xtask path-deps already carry `version = "0.0.0"`. | §2.2; §4 P1 row | `xtask/Cargo.toml:16-22` each path-dep has `version = "0.0.0"`. |
| DN-19 | High | Entire spine (ADR-021 "Proposed, needs ratification") is superseded: ADR-021 reached Accepted then was Superseded by ADR-022; DN-19 never mentions ADR-022. | §1; GAP-6; §6 changelog | ADR-021 Status = Superseded by ADR-022 (2026-06-23); `ADR-022` file exists; `grep -c ADR-022 DN-19` = 0. |
| DN-23 | High | Core "current state" false: §1 asserts no symbolic infix operators, but the lexer tokenizes `+ - * / % &`. | §1 (l.20-24) | `lexer.rs:181-193` `+→Plus`, `*→Star`, `/→Slash`, `%→Percent`; `lex_dash` returns `Minus` (RFC-0025/M-705); `token.rs:201-219` defines all. |
| DN-23 | High | §4 frames a `-` math operator and RFC-0025 as "forthcoming"; both already exist and are implemented Rust-first. | §3-§4; Decides l.9 | `RFC-0025-Operator-Syntax.md` exists (Proposed/implemented Rust-first, M-705); `lexer.rs:262` already cites RFC-0025/M-705. |

### 2.3 Medium findings (grouped by theme)

**Stale-status / supersession-not-recorded:**

| Doc | Summary | doc_loc | Evidence |
|---|---|---|---|
| DN-19 | GAP-6 "ADR-021 currently Proposed, not Accepted" — false as of the doc's own 2026-06-24 addendum. | GAP-6 | ADR-021 reached Accepted 2026-06-21 then Superseded 2026-06-23. |
| DN-21 | Header Status "Draft" but the changelog declares the epic M-678 Enacted/landed — header stale vs body. | Header (l.6) vs changelog (l.153-173) | Changelog: "M-679…M-683 landed (epic M-678 enacted)". |
| DN-23 | No changelog/Meta footer at all; the post-RFC-0025 staleness was never reconciled append-only. | whole doc | grep for changelog/Meta returns nothing; sibling DN-24 moved to Resolved when its RFC landed. |
| DN-24 | "Current state" stale: the recommended stack (semantic-tokens provider, tmLanguage, tree-sitter) was BUILT, not just planned. | §3 (l.54-56); §5 (l.86-87) | `mycelium-lsp/src/semantic.rs` is a full provider (M-730, RFC-0026); `tools/grammar/` has tmLanguage.json + tree-sitter + generate.py (M-731). |
| DN-26 | Cites ADR-021 as "Accepted"; it is now Superseded by ADR-022 (same date). Stale cross-ref. | §1 (l.25-26); §6 (l.147) | ADR-021 line 6 Superseded by ADR-022; DN-26 dated 2026-06-23. |

**Scale / completeness drift:**

| Doc | Summary | doc_loc | Evidence |
|---|---|---|---|
| DN-16 | Survey scope "25 specs" but the tree now has 26 stdlib specs and 26 `mycelium-std-*` crates; `mycelium-std-sys-host` unsurveyed. | §1; §5; §2 table | `ls docs/spec/stdlib/*.md` = 26; `ls crates | grep mycelium-std-` = 26. |
| DN-17 | §2.3/§4 P2 "MLIR test corpus helpers — factor into tests/common/mod.rs" stale: that file already exists. | §2.3; §4 P2 row | `mycelium-mlir/tests/common/mod.rs` exists. |

**Internal-status contradiction (ADR wave):**

| Doc | Summary | doc_loc | Evidence |
|---|---|---|---|
| ADR-031 | Header "Accepted" but §Status and §Posture still read "Proposed"; references unminted milestone IDs M-775…M-780. | Header (l.6) vs §Status (l.33) / §Posture (l.11-13) | Changelog records 2026-06-24 Accepted; in-body blocks not updated; M-760–M-784 not minted in issues.yaml. |

**Over-claim of full design:**

| Doc | Summary | doc_loc | Evidence |
|---|---|---|---|
| RFC-0017 | §4.1 normative top-down `@matured` header/manifest inheritance NOT implemented, yet doc is Enacted. Gate + thaw built; inheritance deferred. | Status; §4.1; manifest `[project].matured` | `resolve.rs:136-141` states both inheritance tiers deferred (`@matured` resolves local-only); `checkty.rs` gate takes a caller-supplied bool; header only lexed as a comment. |

**Framing-completeness gap:**

| Doc | Summary | doc_loc | Evidence |
|---|---|---|---|
| RFC-0035 | §1 claims "lacks a Mycelium-native security toolkit" and omits the existing `mycelium-sec` (`myc-sec`, M-367) crate entirely; framing understates the native surface it would build on. Not a false status (RFC implements nothing, Proposed). | §1 (l.35-41); Coupled-with (l.11) | `mycelium-sec/src/lib.rs:1-15` (v0 wild-block ADR-014 audit + secrets/supply-chain orchestration). |

### 2.4 Low findings (grouped by theme)

**Successor-status lag (a doc correctly captures its successor passing a gate, but parenthetical maturity label trails):**
- **DN-08** — names RFC-0017 "(Accepted)"; RFC-0017 is now Enacted. DN-08's own resolution is correct; only the label lags.
- **DN-11** — §5.3 (dated 2026-06-21 snapshot) says RFC-0018/0019 "not yet Enacted"; both later Enacted. Append-only snapshot, not a backward move.
- **DN-25** — §7 "stdlib is Rust save lib/std/result.myc" accurate at 2026-06-23; `option.myc`+`cmp.myc` landed 2026-06-24.
- **ADR-014** — frames crate-level `forbid(unsafe_code)` re-pin as "recommended follow-on"; already implemented on all four trusted-kernel crates. "Accepted" (not Enacted), so no status over-claim.
- **ADR-023** — index prose `docs/adr/README.md:14` still calls it "Draft"; the README's own table row (:42) and the ADR header say Accepted. Defect is in the cross-ref index, not this ADR.

**Status-field self-contradiction (2026-06-24 ADR wave hygiene — header Accepted, in-body `## Status` still Proposed):**
- **ADR-029** — header Accepted, body §Status (l.31-33) "Proposed (recommended)". Implementation (`BigTernary`, `checked_to_width`, 3^41 witness) verifies against code.
- **ADR-030** — header Accepted, body §Status (l.30) "Proposed (recommended)". Dense still float-only with no quant descriptor in code — Accepted-but-unbuilt is honest.
- **ADR-031** — (see Medium) header Accepted, §Status/§Posture "Proposed".

**Internal not-yet-executed vs executed (migration already landed in code):**
- **DN-06** — Status/§2/final-changelog say M-358 "staged, not yet executed"; §4 table + changelog say "executed 2026-06-16"; code confirms executed (`token.rs:24` Nodule supersedes static colony, `nodule.rs:75` parse_nodule_header).
- **DN-14** — row 9 `wild`/FFI: §3 table + changelog say "present (executes three-ways)"; a superseded Resolved block still says "execution stays staged → Residual". Code confirms wild executes (`elab.rs` emits `Op{prim:"wild:<name>"}`). Internal-doc only.

**Locator / count imprecision (substance correct, pointer stale):**
- **DN-10** — changelog says prim table migrated "in `mycelium-core::data`"; actually a new `mycelium-core::prim` module (`prim.rs`). Body §3.2/§3.5 hedged correctly.
- **DN-17** — §1 scale figures stale (~43 crates / 23-crate stdlib); tree has 50 crates / 26 std crates.
- **DN-21** — §2 says 6 unsafe blocks; changelog reconciles to 8 post-M-682. §2 prose not annotated historical.
- **DN-22** — §4 says all.sh assigns component ids "1-24"; script comment says "1-31" (5 bits = 0-31). Packing formula otherwise exact.
- **ADR-018** — Context "44 mycelium-* crates"; actual 50. Numeric only.
- **ADR-020** — Colony alias cited at `runtime.rs line 106`; actually line 116. std.runtime facade nodule not yet a Mycelium-language nodule (design phase; honest deferral).
- **RFC-0016** — changelog "all 25 mycelium-std-* crates landed"; 26 dirs exist (extra is `mycelium-std-sys-host`, post-dates enactment). Stale-by-one.

**Stale "blocked / textual-skeleton" sub-claims (real path landed under off-by-default feature, recorded append-only elsewhere):**
- **DN-15** — §2/§4.4 prose calls `dialect.rs` a "permanent block until M-348"; M-601 real arith/func→LLVM path now exists under `mlir-dialect` (OFF by default). Reconciled in §9; doc stays Draft.
- **RFC-0004** — §11.3 says `dialect.rs` is "a textual skeleton only; real lowering requires libMLIR (M-348, blocked)"; M-601 landed real native lowering (`dialect/native.rs`, 895 lines, feature-gated). Under-claims (safe direction); qualifier stays technically true.

**Forward-reference / dating-order quirks (mutually consistent, not contradictory):**
- **ADR-012** — §8 short-form names re-checked against DN-03; set matches, no drift; recorded informational.
- **ADR-013** — "Depends on RFC-0008 §4.4" but RFC-0008 Accepted six days later; RFC-0008 §4.4 defers to ADR-013. Mutually consistent dating quirk.

**Proposed-ADR design-tag-without-checked-basis (honest because Proposed, no impl lands):**
- **ADR-027** — `get`'s Exact tag + blessed `lift_option` adapter are design claims for an unbuilt op; no `lift_option`/`get`/`in_bounds` symbol in crates. Consistent with Proposed posture.

**Positive confirmations (claims verified against code — recorded as Low informational, no defect):**
- **DN-16** §3.1/§3.8/§3.20 (cmp/swap exports, fmt-vs-io from_json divergence, rand xoshiro256++) all accurate.
- **DN-18** — NativeArtifact API, spore-not-depending-on-mlir, ADR-013 Accepted, faithfulness=Empirical all verified; "impl-pending" honest.
- **DN-20** — justfile tiers, PROPTEST_CASES=8/256, ci=check, numerics FLAG fix all verified.
- **DN-28** — M-732 spore_id-vs-artifact seam verified verbatim against `registry.rs`.
- **DN-29** — frozen rev.6 "TDD pending" mildly stale vs now-Enacted RFC-0034; correct append-only Superseded behavior. All §4/§5 machinery refs exist.
- **DN-30** — all security-tooling refs resolve; honestly claims no impl.
- **ADR-014/015/016/017/019** — mechanism + prototype claims verified (inject.rs dispatch, setup-mlir.sh, unsafe-per-use.sh).
- **ADR-024/025/028** — E19-1 pending honestly (Repr::Seq/Bytes absent); BigTernary verified; sign-free binary honest.
- **RFC-0009/0010/0011/0012/0013/0019/0021/0029/0032** — implementation claims spot-verified against code (matrix.rs, decode_select.rs, mono.rs, project.rs llm_canonical totality, prims.rs cmp.eq/lt).

---

## 3. Per-Status Roster (all 92 documents, each exactly once)

### Design Notes (DN)

| Doc | Declared status | Verdict |
|---|---|---|
| DN-01 | Resolved | honest |
| DN-02 | Resolved (2026-06-10) | honest |
| DN-03 | Resolved (2026-06-10) | honest |
| DN-04 | Resolved (2026-06-21) | honest |
| DN-05 | Resolved (2026-06-21) | honest |
| DN-06 | Resolved | internally-inconsistent |
| DN-07 | Resolved | honest |
| DN-08 | Resolved | honest |
| DN-09 | Resolved | honest |
| DN-10 | Resolved | honest |
| DN-11 | Resolved / Draft-as-capture | honest |
| DN-12 | Resolved | honest |
| DN-13 | Resolved | honest |
| DN-14 | Resolved | honest |
| DN-15 | Draft | honest |
| DN-16 | Resolved | stale |
| DN-17 | Draft | stale |
| DN-18 | Draft (design-complete, impl-pending) | honest |
| DN-19 | Draft | stale |
| DN-20 | Accepted | honest |
| DN-21 | Draft | internally-inconsistent |
| DN-22 | Draft | honest |
| DN-23 | Draft | stale |
| DN-24 | Resolved | stale |
| DN-25 | Draft | honest |
| DN-26 | Draft | honest |
| DN-27 | Draft | honest |
| DN-28 | Draft | honest |
| DN-29 | Superseded (by RFC-0034 + ADR-032) | honest |
| DN-30 | Draft | honest |
| DN-31 | Draft (advisory) | honest |
| DN-32 | Accepted (2026-06-25) | honest |
| DN-33 | Accepted (2026-06-25) | honest |
| DN-34 | Draft (advisory) | honest |

### Architecture Decision Records (ADR)

| Doc | Declared status | Verdict |
|---|---|---|
| ADR-010 | Accepted | honest |
| ADR-011 | Accepted | honest |
| ADR-012 | Accepted | honest |
| ADR-013 | Accepted | honest |
| ADR-014 | Accepted | honest |
| ADR-015 | Enacted (2026-06-23) / Accepted | honest |
| ADR-016 | Accepted | honest |
| ADR-017 | Accepted | honest |
| ADR-018 | Enacted (2026-06-23) / Accepted | over-claims |
| ADR-019 | Enacted (2026-06-23) / Accepted | honest |
| ADR-020 | Enacted (2026-06-20) | internally-inconsistent |
| ADR-021 | Superseded by ADR-022 | honest |
| ADR-022 | Accepted (2026-06-23) | honest |
| ADR-023 | Accepted (2026-06-23) | honest |
| ADR-024 | Accepted (2026-06-23) | honest |
| ADR-025 | Proposed (2026-06-24) | honest |
| ADR-026 | Proposed | honest |
| ADR-027 | Proposed | honest |
| ADR-028 | Proposed | honest |
| ADR-029 | Accepted (impl Rust-first) | internally-inconsistent |
| ADR-030 | Accepted (decision locked, impl pending) | internally-inconsistent |
| ADR-031 | Accepted | internally-inconsistent |
| ADR-032 | Enacted | honest |

### Requests for Comment (RFC)

| Doc | Declared status | Verdict |
|---|---|---|
| RFC-0001 | Accepted (r5) | honest |
| RFC-0002 | Accepted | honest |
| RFC-0003 | Accepted (r4) | honest |
| RFC-0004 | Accepted (r1-r5 additive) | honest |
| RFC-0005 | Accepted | honest |
| RFC-0006 | Accepted (r5) | honest |
| RFC-0007 | Accepted (r4 + §11/§12) | honest |
| RFC-0008 | Accepted (staged) | honest |
| RFC-0009 | Enacted | honest |
| RFC-0010 | Enacted | honest |
| RFC-0011 | Enacted (r3) | honest |
| RFC-0012 | Enacted | honest |
| RFC-0013 | Enacted | honest |
| RFC-0014 | Enacted | honest |
| RFC-0015 | Enacted | honest |
| RFC-0016 | Enacted (Rust-first scope) | honest |
| RFC-0017 | Enacted | over-claims |
| RFC-0018 | Enacted (stage 1a) | honest |
| RFC-0019 | Enacted (stage-1 surface) / Accepted | honest |
| RFC-0020 | Accepted (scoped) | honest |
| RFC-0021 | Enacted / Accepted (framework) | honest |
| RFC-0022 | Accepted (Enacted gated on mycelium-web) | honest |
| RFC-0023 | Accepted (Enacted gated on mycelium-adk) | honest |
| RFC-0024 | Proposed / implemented Rust-first | honest |
| RFC-0025 | Proposed / implemented Rust-first | honest |
| RFC-0026 | Accepted | honest |
| RFC-0027 | Accepted (2026-06-25) | honest |
| RFC-0028 | Accepted | honest |
| RFC-0029 | Accepted | honest |
| RFC-0030 | Draft | honest |
| RFC-0031 | Accepted | honest |
| RFC-0032 | Accepted | honest |
| RFC-0033 | Proposed | honest |
| RFC-0034 | Enacted — with code (Rust-first) | honest |
| RFC-0035 | Proposed | honest |

**Roster total: 34 DN + 23 ADR + 35 RFC = 92 documents.** Every audited document appears exactly once.

---

## 4. Consolidated Open-Question Ledger (by theme)

`GATES` = whether the question gates near-term work (Y/N). Most open questions are deliberately deferred (YAGNI / future-RFC / measurement-driven) and do **not** gate.

### Theme A — VSA / numerics verification probes
| Origin | Question | Gates? |
|---|---|---|
| RFC-0001 / RFC-0003 / ADR-010 | The single confirming Liquid-Haskell `bundle` capacity-refinement probe (KC-1 make-or-break). | **N** — confirmatory; gates only upgrading capacity-bound tags from axiomatized-citation basis. |
| RFC-0002 | Mechanically checking lossy-swap bound derivations end-to-end. | N — accepted path axiomatizes cited theorems. |
| RFC-0003 / RFC-0009 | Resonator factorization convergence; ≥100-vector measured corpus; non-default cleanup variants. | N — probabilistic-only by design; decode params landed. |
| DN-32 / RFC-0027 | Performance figures `Declared`; no in-repo Mycelium benchmark; RC-cascade drop-latency SLO. | N — correctness unaffected; benchmark required before any perf-tag upgrade (VR-5). |

### Theme B — Native MLIR backend (libMLIR-gated)
| Origin | Question | Gates? |
|---|---|---|
| DN-05 / DN-15 / RFC-0004 / ADR-016/017/019 | Native managed-stack error shape; real ternary→arith→LLVM correctness (M-601) + three-way differential (M-602); cross-process hot-inject; non-tail/FixGroup trampoline. | **N for AOT env-machine path** (landed); gates only the native MLIR path (libMLIR off-by-default in this env). Provisioning unblocked on Linux (ADR-019). |

### Theme C — Core 1.0.0 / full-language gate
| Origin | Question | Gates? |
|---|---|---|
| ADR-022 / ADR-024 / DN-25 | T1 core-1.0.0 tag act (M-703) — depends_on E19-1 (M-749/750 todo). | **Y** — gates the core 1.0.0 tag (not T2-T9). |
| ADR-024 / ADR-025/026/027 | E19-1 Repr::Seq / Repr::Bytes value-model invariant + adapters; length-in-type ratification. | **Y** — gates the E19-1 Repr additions; M-749/750 todo. |
| DN-19 | Gate A2/A3/A4 (Medium ledger, durability mutants/fuzz, cargo-deny wiring), A5 KC-4 threshold. | **Y** — now live under ADR-022; A3 durability is the largest gap. |
| DN-25 | T4 stdlib-in-Mycelium (M-714-719); T9 self-hosting capstone. | **Y** — long poles for lang 1.0.0. |

### Theme D — Self-hosting & surface-grammar
| Origin | Question | Gates? |
|---|---|---|
| DN-26 | Port order, module-system design, toolchain build system, Rust-dep surface. | N near-term (port not begun); gates Stage 4/5. |
| RFC-0030 | Draft→Proposed gate (M-707 + M-745 angle-bracket operators); EBNF ownership; conformance scale. | **Y** for the Draft→Proposed move (the doc's own DoD); angle-bracket ops absent from grammar. |
| RFC-0024 / RFC-0025 | Multi-arg arrows, closures, partial application; angle-bracket comparison/shift operators (collide with type-args). | N — v0 single-arg / arithmetic sugar landed; deferred to v2 / M-745. |
| DN-31 / DN-23 | `[]` type-args vs list-literal disambiguation; word vs symbol comparison operators. | N near-term (grammar still `<>`); gates a future supersession wave. |

### Theme E — Stdlib ratification & runtime placement
| Origin | Question | Gates? |
|---|---|---|
| DN-07 / DN-16 / RFC-0016 | Per-spec ratification to Accepted; ergonomics-vs-contract per-ring pass (M-540). | N — crates implemented; gates formal status flips only. |
| DN-16 / RFC-0016 / ADR-020 | Q4 runtime/colony Phase-7 placement; construct-by-construct activation. | N near-term — gates Phase-7 only. |
| RFC-0017 | Manifest `[project].matured` spelling; thaw beyond fn; maturation-record attribution. | **Partial** — gates the §4.1 inheritance work `resolve.rs` deferred. |

### Theme F — Future phyla / capabilities (post-core, advisory)
| Origin | Question | Gates? |
|---|---|---|
| RFC-0022 / RFC-0023 | mycelium-web / mycelium-adk crate builds (greenfield); racing/delegation; LLM-leverage Declared. | N — gate Enacted, not the Accepted design; LLM-leverage no-verdict by VR-5. |
| RFC-0028 / RFC-0027 | Capability sandboxing; xloc/WASM; cross-hypha RC Option A vs B (resolved A for R1 by DN-33). | N for v0; gates R2/future. |
| DN-27 / DN-28 | Repo decomposition trigger; registry index hosting + content-store + reconstruction model. | N — post-1.0.0, deferred to future ADR/RFC. |
| DN-34 / RFC-0035 | Rust→Mycelium transpiler home/fidelity; security-toolkit vuln classes, SARIF/CWE schemas, disclosure governance. | N near-term; gate the future phase / RFC-0035 Proposed→Accepted (WE-1/WE-2 worked examples). |
| DN-22 / DN-30 | Diagnostic-code byte layout, registry home; v0 vuln classes. | N — advisory design capture. |

### Theme G — LLM-leverage empirical gate (the genuinely-open one)
| Origin | Question | Gates? |
|---|---|---|
| DN-09 / RFC-0021 / RFC-0023 | arm-3 (grammar-constrained decoding) + arm-5 (embedded-DSL) live runs; familiar-skin idiom (S-expr vs Rust/Haskell). KC-2 verdict "proceed" unaffected. | N — non-blocking research follow-up (M-381); needs local GBNF/llama.cpp model. The genuinely-open empirical gate of the wave. |

**Gating tally:** of the open-question clusters above, the genuinely near-term-gating items are concentrated in **Theme C (core/full-language 1.0.0 gate: M-703, E19-1 Repr::Seq/Bytes, Gate A2/A3/A4, T4/T9)**, **Theme D (RFC-0030 Draft→Proposed, M-707/M-745)**, and **Theme E (RFC-0017 §4.1 inheritance, partial)** — roughly **9 gating questions**. All other open questions are deferred non-blockers honestly named, not silent.

---

## 5. Recommended Corrections (append-only-safe; RECOMMEND-ONLY — not applied)

All recommendations are **append-only** (a Meta/changelog entry or a status-field reconciliation that moves status only forward, or an erratum note). None rewrite decision history. Per house-rule 3, where a decision must change, **supersede**; where a fact drifted, **append an erratum**.

### ADR-018 (over-claims — highest priority)
- Append a changelog erratum: three crates (`mycelium-std-math`, `-sys`, `-sys-host`) are at `0.1.0` and lack a per-manifest `publish` key; clarify that distribution remains blocked by the workspace-level `release-plz.toml` guard, and state the corrected per-crate fact.
- **Maintainer reconciliation flagged:** the `0.1.0` triplet suggests a partial, undocumented release-cut outside the §4 capability gate — surface for an explicit decision (re-pin to `0.0.0` + add `publish = false`, or record the partial cut as a decision).
- Update the `44 → 50` crate count via erratum.

### ADR-020 (internally-inconsistent)
- Add the missing **append-only Accepted and Enacted changelog entries** (with dates + M-521 landed_basis) so the record steps Proposed → Accepted → Enacted per house-rule 3; the implementation genuinely landed, so this is a record-completion, not a status change.
- Correct the `runtime.rs line 106 → 116` cross-ref.

### DN-23 (stale, no footer)
- Add a Meta/changelog footer recording supersession-by-RFC-0025, and move status to **Resolved** (direction ratified into RFC-0025, implemented Rust-first). Annotate §1/§4 "current state" prose as historical (symbolic operators now built).

### DN-16 (stale)
- Append a re-audit erratum reconciling the `runtime` row (crate exists, ratification-ready) and closing the `sys.md` blocker; widen scope to 26 specs / 26 crates and add `mycelium-std-sys-host`.

### DN-17 (stale)
- Append a "superseded by tree" note marking P1 (workspace deps, xtask version) and P2 (MLIR test common/) as **already done**; update §1 scale figures (50 crates / 26 std).

### DN-19 (stale spine)
- Append a note re-framing the spine under **ADR-022** (ADR-021 Accepted → Superseded by ADR-022); correct GAP-6 to record ADR-021 reached Accepted then was superseded.

### ADR-029 / ADR-030 / ADR-031 (internally-inconsistent — ADR-wave hygiene)
- Reconcile the in-body `## Status` / §Posture blocks to **Accepted** to match the ratified headers and changelogs (the headers are authoritative). For ADR-031, either mint M-760…M-784 in `issues.yaml` or replace the over-specific forward IDs with the existing `E20-1` epic reference.

### RFC-0017 (over-claims)
- Append a status erratum scoping "Enacted" to **§4.2 (scope-maturation gate + thaw)**, and explicitly mark **§4.1 inheritance (`[project].matured`, top-down) and §4.4 maturation record as deferred** (matching `resolve.rs:136-141`). Consider a status note "Enacted (gate); §4.1 inheritance pending M-xxx".

### DN-06 / DN-14 / DN-21 (internally-inconsistent — stale "not-yet" lines)
- DN-06: annotate the Status/§2/final-changelog "not yet executed" lines as superseded by the §4 table + code (M-358 executed 2026-06-16).
- DN-14: annotate the superseded "execution stays staged → Residual" block as historical; §3 table + code (wild executes) are authoritative.
- DN-21: move header **Draft → Enacted** (or note "Enacted per changelog"); annotate §2's 6-block count as pre-M-682 (now 8).

### Low-priority locator/count errata (batch)
- DN-10 (`::data` → `::prim`), DN-22 (`1-24` → `1-31`), DN-24 (Decides "forthcoming" → landed; "current state" built), DN-25 (§7 .myc count snapshot), DN-26 (ADR-021 Accepted → Superseded-by-ADR-022), RFC-0004 §11.3 + DN-15 §2/§4.4 (dialect.rs M-601 path landed under off-by-default feature), RFC-0016 (25 → 26 crate count), ADR-023 (README.md:14 Draft → Accepted), ADR-013 (dating-order note), RFC-0035 (mention existing `mycelium-sec` in §1).

---

*End of RECORD. Read-only audit; no corrections applied. All guarantee tags preserved at their documented strength (VR-5). Code/lexicon treated as ground truth over docs throughout.*
