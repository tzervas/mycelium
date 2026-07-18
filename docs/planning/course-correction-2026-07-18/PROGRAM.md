# Course-correction program — 2026-07-18

| Field | Value |
|---|---|
| **Status** | Active execution program (maintainer-directed 2026-07-18, in-session): apply the alignment corrections identified by `../alignment-assessment-2026-07-18/ALIGNMENT-REPORT.md` against the committed steer pack (`../design-steer-2026-07-17/`), then fix the component repos to build, then transpile readiness → `*-myc` delivery → dual-side validation + apples-to-apples metrics → releases. **Human review comes post-fix**; no mid-way stop points, so every maintainer-grade choice is recorded in §2 and mirrored in the blockers file — proceed-on-recorded-default, never silent (G2). |
| **Honesty** | This ledger is `Declared` program state. Decision artifacts minted by this program land **Draft** (H1/H2 — no agent-side `Accepted`). Verification claims in the phase log carry their evidence class. |
| **Grounds** | `../design-steer-2026-07-17/PROGRAM-HANDOFF-DESIGN-STEER-2026-07-17.md` (the approved plan) · `../alignment-assessment-2026-07-18/` (findings F1–F19) · maintainer directive of 2026-07-18 (execute course correction; validate Rust + Mycelium; metrics; releases) |

## 1. Phase map

| Phase | Work | State |
|---|---|---|
| **A** | Monorepo alignment corrections: steer pack committed (A1) · record fixes (A3) · Phase-0 audit ledger (A2) · Phase-1 Draft captures (A4) · deviation captures + blockers (A5) · G-8 ledger entry (A6) | **complete 2026-07-18** (see phase log) |
| **B** | 46 component repos: workspace roots (train v0.464.0), git-pinned cross-repo deps in topo order, toolchain/supply-chain/CI/CROSS-REF/docs scaffolding, umbrella upgrade, per-repo green + identity suite | **complete 2026-07-18** (see phase log) |
| **C** | AX-core waves W-A..W-D in the monorepo; delta propagated to affected components via the §6.2-5 umbrella-PR protocol | pending |
| **D** | Transpile readiness: closable gap classes iterated, M-1006 remeasure, per-component DoD per steer Phase 4 | pending |
| **E** | `*-myc` delivery, leaf-first: graduated `lib/` nodules routed to twins where they exist; vetted drafts + honest gap ledgers where not; differential witnesses | pending |
| **F** | Validation both sides + metrics report (perf/stability/QA/UX/DX; VSA/GPU rows desktop-held) | pending |
| **G** | Releases: per component + `*-myc` + umbrella (tag v0.464.0, GitHub Release, tarball); monorepo promotion PRs staged, terminal squash held for review | pending |

## 2. Recorded defaults (maintainer-grade choices made to keep executing; ratify or reverse at post-fix review)

| # | Choice | Basis | Steer clause affected |
|---|---|---|---|
| D-1 | **Keep the executed 46-repo seam topology** (no re-extraction to PARTITION scope groups) | Maintainer directive wording: "correcting the alignment of the rust component repos … so they all properly work and compile" — the existing repos are the object | H§6.1 topology (deviation captured as Draft DN) |
| D-2 | **Clean-slice seeds stand; history deviation ratified as Draft** (filter-repo re-extraction offered post-review, not executed; monorepo retains all history) | Compile-ability is the stated goal; history re-extraction is orthogonal + layerable later | H§6.2-1 |
| D-3 | **AX-core W-A..W-C in scope**, sequenced: components fixed from archive first, AX-core lands in monorepo, delta propagated via umbrella-PR protocol | Steer gates transpile readiness on AX-core; §6.2-5 protocol gets exercised | H§5, H§6.2-5 |
| D-4 | **Transpile delivery is honesty-laddered**: graduated ports (differential-witnessed) where they exist; drafts + gap ledgers where not; coverage stated at measured strength | VR-5; M-991 verdict (instrument, not bulk porter); current all-7 `checked_fraction` 28.7% `Empirical` | H§7 per-component DoD |
| D-5 | **Version train v0.464.0** lockstep across components/`*-myc`/umbrella | Steer §6.2-2 "lockstep version train v0"; next minor over monorepo `0.463.1` | H§6.2-2 |
| D-6 | **Releases** = git tag + GitHub Release + `git archive` tarball per repo; **no crates.io publishing** (ADR-018 posture unchanged); monorepo terminal squash **held** for human review | Maintainer: "ensure releases are cut for each"; repo release policy is deliberately source-only | — |
| D-7 | **PR #1705 grows** with monorepo course-correction work (same designated branch); component-repo PRs agent-merged after review + green per the autonomous-PR workflow | Harness branch mandate; maintainer: "human review will come post fix" | — |
| D-8 | **`mycelium-lang` umbrella retained** (U1); the H§6.1 `mycelium`-as-front conversion **deferred** — it conflicts with the maintainer's standing requirement that `tzervas/mycelium` contain everything in the components | Assessment F8 tension, surfaced not silently resolved | H§6.1 front repo |

## 3. Blockers / ratification queue (EXPRESS-ORACLE pattern — maintainer answers collect at post-fix review)

| ID | Item | Default in force |
|---|---|---|
| CC-B1 | Which 2026-07-17 program governs long-term (steer pack vs PROGRAM-SELFHOST-DECOMPOSE) — supersession capture needed | Steer pack treated as the approved plan for this program (maintainer directive 2026-07-18); formal supersession doc left Draft |
| CC-B2 | Ratify D-1 topology + D-2 history deviations (Draft DN minted in A5) | In force |
| CC-B3 | Front-repo end-state (`mycelium` thin front per H§6.1 vs monorepo-keeps-everything) | Deferred (D-8) |
| CC-B4 | Phase-1 capture ratifications (Swap Ergonomics DN, DN-141, RFC-0013 amendment, RFC-0034 footnote, retention spec, W-1 capture) | All land Draft |
| CC-B5 | Version-train number v0.464.0 | In force |
| CC-B6 | G-8 `PolicySlot` uncapped logs (assessment F13): cap with the §1.4 retention store (Phase C), not ad hoc | Ledgered; code unchanged until C |
| CC-B7 | Monorepo integration→main terminal squash | Held for human review |

## 4. Phase log (append-only)

| When | Entry |
|---|---|
| 2026-07-18 | Program opened. A1 done: steer pack committed verbatim (`../design-steer-2026-07-17/` + PROVENANCE). |
| 2026-07-18 | **Phase C / W-A COMPLETE** (commits `2818d372..864e214f`): Diag `FirstFaultEnvelope` (RFC-0013 A1, additive — pinned-golden backward compat), `swap_check` emitter + **G-3 fix** at the mode.rs seam, `Meta.cert` handle (hash-excluded, boxed), capped mode-gated `CertStore` (Declared caps, EXPLAINed drops), CertMode print on check/run, `first_fault_line()` exit criterion proven. ~537 change-scoped tests green; api-index regenerated; CHANGELOG row applied; audit-ledger G-3 disposition cites the landing. Judgment calls recorded in the W-A CHANGELOG entry. Next: W-B (`policy: ambient` + catalog + legal-pair matrix + elaboration-hash goldens). |
| 2026-07-18 | **Phase B COMPLETE.** All **45 Rust component repos** land buildable at train v0.464.0 — 9 waves (W1–W9), each repo verified `cargo fmt --check` + `clippy --all-targets -D warnings` + `cargo test` **green standalone** at its merged rev (`Empirical`, logs in-session; per-repo CI v2 is the running witness), PR-merged to its `main`. Umbrella `mycelium-lang` upgraded: **lock v2** (45 Rust pins with rev **+ tree hash**; 46 `*-myc` seed pins carried `Declared`; no-self-pin design note) + a **real integration CI** (lock-format assertions + apex `cargo check` transitively resolving the whole pinned graph — steer §6.2-4); merged (`123cac8c`). Monorepo mirror lock updated with the umbrella tip. Container conversions beyond the seeds: l1, doc, std-conformance (compile-time fixture parity); dual-layout fixture resolution in transpile + fmt (flagged for Phase-C upstreaming); DN-80 sibling-source audits layout-gated with printed skips (canonical audit stays monorepo-side). F14 resolves as documented-by-README (cli-myc declares dual provenance). **Identity invariant status (honest):** source byte-parity vs archive `Exact` (assessment parity engine) + content-hash/conformance suites green in the component layout (`Empirical`); the dedicated cross-layout golden-suite comparison artifact remains queued (Phase F). Next: Phase C (AX-core W-A). |
| 2026-07-18 | **Phase B in flight (checkpoint).** Select+vsa-decode → runtime move executed (maintainer-approved; DN-143 §4 updated — vsa-decode rides along, zero dependents). Generator `scripts/decompose/fix_component_workspace.py` landed (plan/apply; 9-wave DAG). **W1–W4 + std-error landed: 31/46 repos buildable, verified fmt/clippy/test green standalone, PR-merged to their `main`s.** Working rules learned + encoded: (1) **frozen lockstep pins** — one rev per dep repo train-wide (mixed revs split cargo package identity → E0308; generator re-pins drifted git lines); (2) dep lines carrying `version =` keys normalized (stale train pin breaks resolution); (3) owned fixture slices travel with their repo (core: `docs/spec/schemas`; l1: `lib/compiler`, `lib/std`, `docs/spec/grammar`); (4) l1 converted to container layout so monorepo-relative fixture paths resolve with zero source divergence; (5) RFC-0034 clause-(c) conformance split per-repo with delegation notes (runtime checks cert; core/value check their own trusted-base crates); (6) one CI failure (clippy `single_element_loop`) fixed forward same-hour. W5 l1 full suite running; W6–W9 (bench/proj/transpile → doc/fmt/lsp/spore/std-testing → check/cli/lint/std-spore → std-conformance) pending. |
| 2026-07-18 | **Phase A complete.** A2: audit-grok ledger — G-1/2/4 clean (`Exact`/`Empirical`; RFC-0012 goldens 12/12, mode tests 7/7, io.myc tags oracle-matched), G-3 one P2 latent pre-archive finding (mode.rs Certified/`check:None` — forward-fix scheduled with W-A), G-5..G-11 rowed. A3: program record corrected append-only + CHANGELOG rows + lock accounting. A4: all six Phase-1 captures landed **Draft** — DN-142, DN-141, RFC-0013 §10 Amendment A1, RFC-0034 §7 footnote, `Language-Retention-Policy.md`, W-1 amendments + **M-1119** (E-W1). A5: DN-143 (four deviation captures incl. the `select`→runtime repo-cycle fix). A6: G-8 `PolicySlot` ledgered (CC-B6). Gates at close: markdown 672 clean · doc_refs OK · links OK. Two agent-flagged grounded corrections recorded in the docs themselves (RFC-0018 §4.5 status; RFC-0013 I1-vs-I3 citation). Next: Phase B. |
