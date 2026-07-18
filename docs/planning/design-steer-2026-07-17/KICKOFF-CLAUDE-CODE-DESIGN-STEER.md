# Kickoff — Claude Code: design-steer execution (2026-07-17)

You are the implementing agent for the Mycelium project (`tzervas/mycelium`, branch `dev`). The maintainer has completed deliberation on the four-doc design pack (`docs/planning/gap-analysis-2026-07-16/DESIGN-01..04`). Your job is to execute the resulting program **exactly as captured**, honoring every stop point.

## Read order (before any change)

1. `PROGRAM-HANDOFF-DESIGN-STEER-2026-07-17.md` — binding steers, corrective W-1, phases 0–4. This is your source of truth for *what* and *in what order*.
2. `CITATIONS-DESIGN-STEER-2026-07-17.md` — evidence pack (`[C#]`) + repo grounds index. Verify **◆** entries before load-bearing use.
3. Repo-root `maint-guide.md` + `.claude/kickoffs/_doc-maintenance.md` — house process (L0→L1→L2, PM close-out, doc discipline).
4. The four DESIGN docs + grounds cited per task (RFC-0002/0005/0012/0013/0018/0033/0034; ADR-003/013/032; DN-16; `docs/spec/stdlib/swap.md`).

## Hard rules (violations are P0 incidents)

- **Honor the gates:** N1–N9 + H1–H4 as tabulated in the handoff §0. In particular: never-silent (G2); no auto-`swap` (S1); no grade upgrade without basis (VR-5); first-fault localization (N9); free-ID check before minting any DN/ADR number; never mark anything `Accepted` without a maintainer ratification date; never rewrite git history — fixes are forward commits; respect the do-not-lift list.
- **Steer conformance:** `policy: ambient` is the ratified spelling — `default`/`auto`/`_` must not appear in code, docs, or tests. Width canon is `Binary{64}` (fallback `Binary{32}`) per handoff §2 — do not introduce new `Binary{8}` outside embedded profiles/test vectors. No new unbounded accumulators on cert/diag/explain paths (caps land with the store, handoff §1.4).
- **One diagnostics system:** every new diagnostic goes through `mycelium-diag::Diag` (extended per the RFC-0013 amendment). No parallel schemas.
- **Escalate, don't guess:** ambiguity or a conflict between grounds → write an entry in the `EXPRESS-ORACLE-BLOCKERS` pattern (`docs/planning/gap-analysis-2026-07-16/EXPRESS-ORACLE-BLOCKERS-*.md`) and stop that thread. A blocked thread is honest; an invented resolution is not.

## Execution phases & stop points

**Phase 0 — Grok-era audit (start here).** Run the G-1…G-11 checklist (handoff §3) over the current `dev`. Output `docs/planning/audit-grok-2026-07/AUDIT-LEDGER.md` (one row per finding: id, severity, site, gate violated, evidence, fix commit or escalation). Mechanical fixes (G-10) may land as forward commits during the audit; **gate-violating findings (G-1..G-9, G-11) get ledger entries + proposed fixes, not unilateral rewrites of semantics.** → **STOP: maintainer reviews the ledger.**

**Phase 1 — Ratifiable capture.** Mint/amend per handoff §4: Swap Ergonomics DN (free-ID check first), DN-141 rewrite (+S1/S2 spike stubs), RFC-0013 envelope amendment, RFC-0034 §7 clarifying footnote, `LanguageRetentionPolicy` spec (defaults marked `Declared`), W-1 capture + E-W1 work item. Append-only; changelog + `Doc-Index.md` rows for each. X15 scaffolding may proceed on branches but does not merge pre-ratification. → **STOP: maintainer ratifies.**

**Phase 2 — Implementation waves.** W-A first (X15 emitters at the named seams in `mycelium-cert/src/mode.rs`; `Meta.cert` handle; mode-gated capped cert store; CertMode print; three-axis labels; lean first-fault one-liner = exit criterion). Then W-B (X1 `policy: ambient` + catalog + legal-pair matrix), W-C (X2–X5), W-D (spikes S1/S2 as design notes only; sizing pass upgrading retention defaults `Declared`→`Empirical`; E-W1 + W-1 sweep), W-E (AX-iso). **ONESHOT residual stays HOLD until AX-core DoD.** Every wave: tests + goldens (elaboration-hash conformance) green before merge.

**Phase 3 — Repository decomposition** (only after Phase-0 ledger resolved, Phase-1 ratified, AX-core DoD, CI green). Execute handoff §6 verbatim: component repos seeded from `PARTITION.md`; per-repo docs + `CROSS-REF.md` (Mycelium-internal deps only); `mycelium-docs` full-record repo; `mycelium` front re-export/orchestration repo with the version train. **Identity invariant is a hard gate:** golden content-hash/spore suite must be byte-identical across the split (ADR-003); any delta = stop + escalate. → **STOP: maintainer reviews topology before repos go live.**

**Phase 4 — Transpile readiness,** one component at a time, leaf-first, per-component DoD (handoff §7). One-shot claim remains **HOLD**.

## Working style

- Small, reviewable commits; conventional messages per `.cz.toml`; keep `CHANGELOG.md` and `Doc-Index.md` current with every doc change.
- Prefer extending landed mechanisms over inventing (the whole steer converges on three reused mechanisms: ambient scoping, content-hash handles, the `Diag` record — keep it that way).
- When citing rationale in code comments or docs, cite repo grounds directly and external evidence as `[C#]` against the citations doc.
- Report progress per phase with: what landed, ledger/blocker deltas, next gate.

Begin with Phase 0.
