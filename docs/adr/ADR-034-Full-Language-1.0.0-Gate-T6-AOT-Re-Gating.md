# ADR-034 — Full-Language 1.0.0 Gate (Track T6) Re-Gating: Native AOT Becomes a Hard Release Row

| Field | Value |
|---|---|
| **ADR** | 034 |
| **Status** | **Accepted** (2026-06-30 — maintainer-ratified). Amends **ADR-022 track T6** (native AOT maturity): **reverses ADR-022 §8 Q4** (which un-gated T6 to `1.1`) and re-adds epic **E15-1** to the `lang 1.0.0` Definition of Done as a **hard gate row**, with scope **expanded to full native-codegen coverage of the whole language**. This is a **scoped amendment** — ADR-022's dual-version model, tracks T1–T5 + T7–T9, and the ADR-024 T1 amendment all **remain in force**. `Accepted → Enacted` with ADR-022 at the `lang 1.0.0` tag. |
| **Decides** | That the **full-language 1.0.0 gate (ADR-022 §5)** additionally requires **E15-1** — the native AOT path — to reach **full-language native-codegen coverage**, verified, **before** the `lang 1.0.0` tag (M-738). "Full coverage" = the native path lowers the whole Core IR fragment the interpreter accepts (closures, non-tail/mutual recursion, `trit.mul`, `Swap`, Dense, VSA) plus JIT for dynamic VSA/HDC workloads, each carried by a checked three-way differential (interp ≡ AOT ≡ JIT) — not only the bit/trit + bounded-data subset shipped at waveN2. |
| **Amends** | **ADR-022 §3 / §5 track T6 / §8 Q4** — re-gates T6 from `1.1` into the `lang 1.0.0` Definition of Done and expands E15-1's scope. ADR-022 is updated only with append-only "re-gated by ADR-034" pointers (its §8 Q4 resolution text and §5 T6 cell are **not** rewritten — a forward pointer is added). `M-738` (the `lang 1.0.0` release act) now `depends_on E15-1`. |
| **Grounds** | Maintainer decision (2026-06-30 — "fully implemented and fully featured in the Rust 1.0.0 version; AOT a hard gate row; full native coverage; extend parallelism"); RFC-0029 §7 (the AOT maturity + JIT design, Accepted); RFC-0004 §2/§11 (the MLIR→LLVM backbone + the staged data/closure/recursion increments); DN-15 (the direct-LLVM-advanceable vs libMLIR-gated decomposition); DN-25 (the full-language 1.0.0 program map); ADR-022 (the gate amended); ADR-024 (the precedent amending pattern); ADR-009 (the JIT deferral this scope lifts for dynamic VSA); KC-2/KC-3, G2/VR-5 (small auditable kernel — the AOT path stays outside the trusted base; honest tags; never-silent). |
| **Date** | 2026-06-30 |

> **Posture (honesty rule / VR-5).** This ADR records *criteria*, maintainer-ratified; it asserts no
> release and declares nothing done. The native AOT is **implemented and landed on `main`** (the
> waveN2 wave — `crates/mycelium-mlir`: direct-LLVM + feature-gated MLIR-dialect + JIT + BitNet +
> EXPLAIN-able passes + a mutant-witnessed three-way differential), but at a **subset** of the
> language; "full coverage" is **not yet met** and is **not** claimed here. E15-1's per-issue DoDs
> (M-725…M-729 + the new E25-1 increments M-850…M-863) carry the evidence bar — each lands with a
> checked three-way differential before it counts. The interpreter **remains the trusted base and the
> reference** throughout (ADR-007/NFR-7); the native path is the performance layer, validated against
> it, never the source of meaning. RFC-0029 moves `Accepted → Enacted` only when the path is complete
> and stable (house rule #3) — not by this ADR.

---

## 1. Why this amendment exists

ADR-022 §8 Q4 (RESOLVED 2026-06-23) **un-gated** T6 (native AOT maturity — E15-1) to `1.1`, on the
reasoning that 1.0.0 ships on the interpreter trusted base plus the existing direct-LLVM kernel subset,
so optimized native codegen is *performance, not correctness* and need not gate the release.

The maintainer has since decided (2026-06-30) that the Rust 1.0.0 release should be **fully implemented
and fully featured**, with the native AOT **completed, performant, and a hard part of 1.0.0** — and
that the native path should cover the **whole language** (not only the bit/trit + bounded-data subset
shipped at waveN2), delivered over multiple scoped PRs "through the lowers." Because ADR-022's Status
makes **changing a resolved gate question a supersede-only act** (house rule #3 — append-only), that
reversal is enacted **here**, in a focused amending ADR, rather than by rewriting ADR-022 §8 Q4's
resolution text in place. This mirrors how **ADR-024** amended T1 for E19-1.

The maintainer's accompanying scope is recorded so the gate is honest about what "full" means: native
codegen for closures, non-tail and mutual recursion, `trit.mul`, the `Swap` node, Dense and VSA reprs,
and JIT for dynamic VSA/HDC workloads (lifting ADR-009's deferral for that path) — each behind a
checked three-way differential, with the interpreter the reference.

## 2. The amendment — T6 re-gated, E15-1 scope expanded

ADR-022 §5 track T6 previously read "→ `1.1` (un-gated; QoL/perf, not a 1.0.0 blocker — §8 Q4)". **This
ADR replaces that status by adding T6 back to the `lang 1.0.0` gate:**

> **T6 (re-gated into `lang 1.0.0`).** Before the `lang 1.0.0` tag, epic **E15-1** (native AOT) reaches
> **full-language native-codegen coverage**, verified: the native path (direct-LLVM first; the
> MLIR-dialect path where libMLIR is provisioned, ADR-019) lowers the whole Core IR fragment the
> interpreter accepts — **closures** (any repr/width, currying), **non-tail + mutual recursion**
> (`Fix`/`FixGroup`, heap-trampoline, stack-robust per DN-05), **`trit.mul`**, the **`Swap`** node
> (certificate-preserving), **Dense** and **VSA** reprs, and **JIT for dynamic VSA/HDC** workloads —
> each with a checked **three-way differential (interp ≡ AOT ≡ JIT)** through the shared M-210 checker,
> **mutant-witnessed** (RFC-0029 §7.5). Every still-unsupported fragment, if any remains, is an
> **explicit never-silent refusal** routed to the interpreter (G2), never a silent miscompile. The
> existing waveN2 deliverables (M-725…M-729) are subsumed and closed under this expanded DoD.

The umbrella for the expansion is the new epic **E25-1** (native AOT full-language coverage,
parallelism, and 1.0.0 gating; `depends_on E15-1`), with issues **M-850…M-863** (the lowering
increments, the perf/parallelism work, and the ratification act). `M-738` (the `lang 1.0.0` release
act, track T8) now `depends_on E15-1`.

## 3. Consequence

- The `lang 1.0.0` tag now **waits on native AOT full coverage** (M-738 `depends_on E15-1`), alongside
  the existing long poles (T4 stdlib-in-Mycelium / E13-1, T9 self-hosting / E18-1). The tag is coupled
  to AOT readiness — the maintainer's deliberate choice (full-featured 1.0.0 over an earlier tag).
- The trusted base **does not grow**: the AOT path lives entirely in `crates/mycelium-mlir`
  (KC-3-outside-kernel); the interpreter stays the reference. New native lowering is honestly tagged
  (`Empirical` once differential-checked; `Declared` until the mutant-witness fires) and never-silent.
- ADR-022 §3 + §5 T6 + §8 Q4 carry append-only "re-gated by ADR-034" pointers; their resolution text is
  **not** rewritten (the supersede-to-change-criteria rule is honored, per ADR-024's precedent).
- Two scope pieces need their **own normative design before implementation** and are flagged here
  (never assumed): **native Dense + VSA codegen** (out of RFC-0029's current scope, §3) → a new RFC
  (RFC-0039 proposed), and the **ADR-009 JIT-deferral lift for dynamic VSA** → recorded in that RFC's
  execution section or a focused ADR-009 amendment (maintainer's call at the relevant PR).

## 4. Rationale & alternative considered

**Chosen:** native AOT as a hard `lang 1.0.0` gate with full-language coverage — the 1.0.0 release is
genuinely "fully implemented and fully featured," with a performant native path for the whole language,
not only a subset, at the headline tag.

**Alternative (not taken):** keep Q4's deferral — ship 1.0.0 on the interpreter + the waveN2 native
subset and roll full AOT coverage to `1.1` as QoL/perf. This tags sooner and decouples the release from
AOT readiness, but the 1.0.0 language would be feature-complete only on the interpreter, with native
codegen partial — weaker on the maintainer's "fully featured in the Rust 1.0.0 version" criterion. The
maintainer weighed the coupling cost against that goal and chose full native coverage at 1.0.0.

## 5. Definition of Done

- [x] E15-1 reaches full-language native coverage: the increments (E25-1 / M-850 recursion · M-851
  closures · M-852 `Swap` · M-853 Dense · M-854 VSA · M-855 dynamic-VSA JIT · M-856 dialect catch-up ·
  M-857 `trit.mul` dialect · M-858 unified mutant-witnessed three-way) each land with a checked
  three-way differential and honest tags; never-silent (G2) at every remaining boundary. **Met
  2026-07-01 (M-863 ratification act):** every E15-1 (M-725…M-729) and E25-1 (M-850…M-862) child is
  `done`, including the M-856b Dense/VSA-dialect split-out and M-860/M-862 parallelism increments
  that were still open at this ADR's own ratification date.
- [x] RFC-0029 reaches **Enacted** (the path complete + stable; the M-729 mutant-witness fires —
  `Empirical`, §7.5); DN-15 → **Resolved**; DN-25's T6 row reflects the re-gating. **Met 2026-07-01
  (M-863):** RFC-0029 → Enacted, DN-15 → Resolved, DN-25 §2/§3 refreshed — all same act.
- [x] ADR-022 carries the append-only "T6 re-gated by ADR-034" pointers (§3 + §5 T6 + §8 Q4 + changelog);
  its §8 Q4 resolution text is otherwise unchanged. Already landed (2026-06-30, same wave as this
  ADR's own Accepted) — confirmed present, not re-added.
- [x] `M-738` `depends_on E15-1` (tracker); the `lang 1.0.0` tag is cut only once E15-1 is met (with the
  other §5 tracks). Already wired (`tools/github/issues.yaml`); confirmed present.
- [x] This ADR reaches **Accepted** (maintainer-ratified) and is indexed (`docs/adr/README.md`).
- **Enacted** with ADR-022 at the `lang 1.0.0` tag (append-only; M-738). **NOT flipped by the M-863
  ratification act (2026-07-01) — FLAG.** This ADR's own Status field and this final DoD bullet both
  couple Enactment to the `lang 1.0.0` tag act (M-738), not to E15-1/E25-1 coverage landing alone;
  every checkbox above is now met, but the tag itself has not been cut (M-738 is still
  `status:blocked` — stdlib-in-Mycelium/E13-1 and self-hosting/E18-1 remain open per M-738's own
  `landed_basis`). Per VR-5, this stays **Accepted** until M-738 actually cuts the tag; do not guess
  the transition ahead of that checked basis.

## 6. Grounding / honesty

- Maintainer decision (2026-06-30) — the reversal of Q4 + the full-coverage scope this ADR enacts into
  the gate.
- RFC-0029 §7 — the native AOT maturity + JIT design (Accepted); §7.5 the mutant-witnessed differential
  bar that keeps the durability claim `Empirical`, not `Declared`.
- RFC-0004 §2/§11 — the MLIR→LLVM backbone + the staged data/closure/recursion increments the coverage
  work realizes; DN-15 — the honest direct-LLVM-advanceable vs libMLIR-gated split.
- ADR-022 §3/§5/§8 Q4 — the gate amended (scoped; the rest stands); ADR-024 — the precedent amending
  pattern this follows.
- ADR-007/NFR-7, KC-2/KC-3, G2, VR-5 — the interpreter stays the reference; the AOT path stays outside
  the trusted base; additions are never-silent + honestly tagged; nothing is "met" ahead of a checked
  differential, and no spec moves to Enacted here.

---

### Changelog

| Date | Status | Note |
|---|---|---|
| 2026-07-01 | **Accepted** (unchanged — §5 DoD progress note) | **M-863 ratification act.** Every §5 DoD checkbox but the terminal one is now met: E15-1 + E25-1 close with all children `done` (M-856b, M-860, M-862 — the last three open at this ADR's Accepted date — land this wave); RFC-0029 → **Enacted**; DN-15 → **Resolved**; DN-25's T6 row refreshed. **Status stays Accepted, NOT Enacted** — this ADR's own Status field and §5's final bullet both couple `Accepted → Enacted` to the `lang 1.0.0` tag act (M-738), which has not run (M-738 remains `status:blocked` on E13-1/E18-1, per its own `landed_basis`). FLAGGED, not guessed past this checked basis (house rule #3/VR-5). Task: E25-1/M-863. |
| 2026-06-30 | **Accepted** | Maintainer-ratified scoped amendment of ADR-022 track T6: **reverses §8 Q4** (which un-gated native AOT to `1.1`) and re-adds epic **E15-1** to the `lang 1.0.0` Definition of Done as a **hard gate row**, with scope expanded to **full-language native-codegen coverage** (closures · non-tail/mutual recursion · `trit.mul` · `Swap` · Dense · VSA · JIT for dynamic VSA/HDC). Umbrella epic **E25-1** (issues M-850…M-863). `M-738` now `depends_on E15-1`. ADR-022 §3/§5 T6/§8 Q4 carry append-only "re-gated by ADR-034" pointers (their resolution text is not rewritten). The interpreter stays the trusted-base reference; the AOT path stays outside the kernel (KC-3); honest tags + never-silent throughout (G2/VR-5). Enacts the maintainer's 2026-06-30 decision append-only (mirrors ADR-024). |
