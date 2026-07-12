# Design Note DN-124 — Vet-Harness Phylum Visibility: Partial Per-Nodule Verdicts, Phylum-Mode Vet Wiring, and the Measurement-Basis Honesty Question

| Field | Value |
|---|---|
| **Note** | DN-124 |
| **Status** | **Accepted** (2026-07-12, ratified under explicit maintainer delegation — "ratify based on objective reasoning and the project's needs/intents, keep to core principles, report results"). Was **Draft** (2026-07-12, same day). Ratifies **P-A** (partial per-nodule verdicts via import-closure sub-phylum re-check, reusing `check_phylum` unchanged) and **M-A** (dual-report both measurement bases one transition cycle, then re-baseline with `Δ_basis` attributed; annotate — never rewrite — the historical oracle §8 series). **Accepted, not Enacted** (house rule #3) — it **builds nothing** yet; every mechanism/verdict tag stays `Declared` until Units 1–3 (§5.3) land and are differential-witnessed. Tags are `Empirical` where read against the tree (verification base `dev`/worktree `b36ebdbe`; the finding's leaf `claude/leaf/gc2-imp1-cross-nodule-symtab` @ `4f346da3` on `dev@12f0632b` — the four cited harness files are byte-identical across both bases, checked), `Declared` for any proposed-but-unbuilt mechanism (VR-5). |
| **Ratification basis (recorded verbatim, 2026-07-12)** | **Phylum-mode is the demonstrably-correct basis** — oracle-mode false-fails valid cross-nodule uses a real build accepts, proven in-tree by `phylum_cross_nodule_reference_resolves` (`crates/mycelium-check/src/lib.rs:530-560`). **P-A is KC-3** (zero kernel growth — reuses `check_phylum` unchanged; the driver only composes it over sub-phyla). **The import-closure invariant adversarially prevents false-clean** (§2.1, §6 — a nodule is `Clean` only if its whole import closure is clean; VR-5, never fabricated). **M-A is VR-5 plus append-only applied to the metric itself** — the basis-change jump is disclosed, recovered-vs-earned is explicit, and the historical §8 series is annotated, never rewritten (house rule #3). Ratified on objective/core-principle grounds under explicit maintainer delegation, not by the design-reasoner self-ratifying. |
| **Decides** | *Proposes, for ratification (does not self-ratify):* (1) the **verified problem statement** — the transpiler `--vet` loop and `gen/myc-drafts/regenerate.sh` check each emitted `.myc` in **oracle (single-file) mode**, which has **zero phylum-import visibility**, so a correctly-resolved cross-nodule `use checkty.Width;` validates CLEAN under `myc check --phylum` but FAILS under oracle mode (`checkty.Width` unresolved) — making the whole cross-nodule/cross-phylum lever family (Import, DN-122 external-trait impls, cross-phylum records) **un-creditable even when the transpiler emits correctly**. (2) The **ranked recommendation**: **P-A** — add **sound partial per-nodule verdicts** to `PhylumReport` via **driver-level import-closure sub-phylum checking that re-uses the kernel `check_phylum` unchanged** (zero kernel growth, KC-3), then **switch the vet path to `myc check --phylum <dir>`** consuming those verdicts. Partial verdicts are a **precondition, not polish**: an all-or-nothing phylum-mode switch would **regress mixed phyla** (§3.3). (3) The **measurement-basis decision (the crux, §4)**: phylum-mode is the **more correct basis** (it matches the real phylum build; oracle mode checks a counterfactual phylum-of-one no build ever performs) — but the `checked_fraction` number will **JUMP** on the switch by recovering falsely-failed items, which is a **basis correction, NOT lever progress**. Recommend **M-A** — **dual-report both bases for a transition cycle, then re-baseline with the one-time `Δ_basis` explicitly attributed**, never retroactively rewriting the historical oracle §8 series (append-only). (4) The **rejected alternatives + soundness invariant**: a first-class kernel per-nodule-verdict rewrite (**P-B**) and a hard un-annotated switch (**M-B**) are rejected; the **import-closure soundness invariant** (a nodule is credited `Clean` **only if its whole import closure checks clean**) is the load-bearing guard against the false-clean failure mode. |
| **Feeds** | The gap-close-2 lever program (`.claude/kickoffs/README.md` gap-close-2 wave); the **Import** gap-class lever (DN-34 §8.19/§8.20, the leaf `symtab.rs`); **DN-122** (external-trait impls across the home boundary — un-creditable today for the same reason); **DN-113 / M-1060** (cross-phylum import resolution, `check_phylum_with_deps`, `checkty.rs:1935` — the cross-phylum dep-vetting interaction, §7 OQ-2); **DN-112 / M-1036** (nodule-qualified home identity — the home-qualified `use <nodule>.<Item>;` emission the leaf's `symtab.rs` produces); **DN-101** (cross-nodule runtime link) and **DN-20** (the three test tiers — base-crate touches desktop-held for the full sweep); **M-1000/M-1001** (the transpile→`myc check` vet loop this note extends); **M-1006** (the phylum-check mode and the ladder this feeds); the DN-99 register (the ENB/Import rows whose `checked_fraction` this un-blinds). |
| **Grounds on** | **KC-3** (small auditable kernel — the recommended P-A re-uses `check_phylum` unchanged; **zero L0/kernel growth**); **KISS/YAGNI** (driver-level closure re-check over a new kernel per-nodule verdict path); **DRY** (the kernel `check_phylum` stays the single source of truth for what "checks clean" IS; the driver only *composes* it over sub-phyla); **G2 / never-silent** (partial verdicts are strictly more informative and never fabricate a verdict — a blocked nodule is reported `Blocked`, never guessed `Clean`; the report carries BOTH per-nodule verdicts AND the whole-phylum verdict so the two are never conflated); **VR-5** (the `checked_fraction` basis change is disclosed, the recovered-vs-earned split is explicit, and the emitted `.myc` stays `Declared` until a differential upgrades it — the vet *verdict* stays `Empirical`); **house rule #3 append-only** (the historical oracle-mode §8 series is annotated, never rewritten); **house rule #4** (the honest finding — a naïve phylum-mode switch *regresses* mixed phyla — is surfaced, not glossed). |
| **Date** | July 12, 2026 |
| **Task** | Scope the vet-harness phylum-visibility fix into a ratification-ready Draft DN — verify-first problem statement, partial-verdict + vet-wiring + measurement-basis design, ranked recommendation with an objective table, adversarial stress-test, and a Definition of Done. Read-only except this DN plus its FLAGGED (not applied) Doc-Index / CHANGELOG / issues.yaml rows. |

> **Grounding + honesty (house rule #4 / VR-5 / G2).** This note **works a decision forward and
> recommends, ranked**; it does **not** take the decision (the maintainer ratifies). Its central,
> potentially-unwelcome finding — reported on the evidence, not softened to manufacture a clean
> deliverable — is that **switching the vet path to phylum-mode is only safe if partial per-nodule
> verdicts land first.** A phylum-mode switch over the *existing* all-or-nothing `PhylumReport` would
> **regress** every phylum that has even one gapped nodule (today oracle mode credits that phylum's
> clean files individually; all-or-nothing phylum-mode would credit **zero**). So the two changes are
> **one coherent unit**, sequenced partial-verdicts-first — and the `checked_fraction` jump on the
> switch is a **basis correction, disclosed as such**, never presented as lever progress.

---

## §1 The problem, precisely (verify-first — mitigation #14)

The gap-close-2 wave-2 leaf (`claude/leaf/gc2-imp1-cross-nodule-symtab` @ `4f346da3`) built a correct
transpiler-side **batch-scoped cross-nodule symbol table** — the Import gap-class lever. It emits a
resolved cross-nodule reference as a **home-qualified** `use <nodule_path>.<Item>;` (never a bare
`<Item>` — the same discipline `checkty.rs`'s DN-113 `qualify_cross_phylum` uses; source: the leaf's
`crates/mycelium-transpile/src/symtab.rs` module doc). The leaf then discovered that the **measurement
harness cannot credit this correct emission**, and a transient regen showed `checked_fraction`
**7.68% → 7.03%** — a pure measurement artifact, not a transpiler regression (`Declared` — the specific
percentages are the leaf's report; the wip commit `6d2ecb09` records a `6.8% → 7.7%` swing in the same
region; the *direction* of the artifact is confirmed below by construction).

The root cause is a **mode mismatch between what the transpiler emits and what the vet harness checks**:

### §1.1 Oracle mode checks one nodule with no sibling context (`Empirical`)

`myc-check` has three modes (`crates/mycelium-check/src/bin/myc-check.rs:6-10`): **oracle** (single
file), **project**, and **`--phylum <dir>`** (whole-phylum cross-nodule check, M-1006). Oracle mode is
the single-file path (`myc-check.rs:100-102` → `run_oracle`), reached when no `--phylum`/`--project`
flag is passed. It parses and type-checks **one nodule** via `check_nodule` and returns one exit code.

The transpiler vet loop uses **exactly this oracle mode**. `crates/mycelium-transpile/src/vet.rs:7-8`
states it runs "the **real** `myc check` oracle … the same per-file oracle mode
`scripts/checks/myc-dogfood.sh` uses" over each emitted `.myc`, and `vet.rs:26-27` is explicit:
"`myc check` (oracle mode) is a **per-file** verdict: it parses + type-checks a whole nodule and
returns one exit code." The `MycChecker::vet_file` implementation appends **one** `.myc` path per
invocation (`vet.rs:362-413`, esp. `:382` `cmd.arg(myc_file)`) — never a directory, never `--phylum`.

Consequence, by construction: a nodule that emits `use checkty.Width;` (where `Width` is
`pub enum Width` at `crates/mycelium-l1/src/checkty.rs:59`) is checked **alone**, with no `checkty`
sibling in scope, so `checkty.Width` cannot resolve — oracle mode refuses it (`no such name … in the
phylum`, the kernel's honest refusal — `checkty.rs:1493`, `:2091`). The transpiler emitted the
**correct** reference; the harness is checking it in a context (a phylum-of-one) that **no real build
ever constructs**.

### §1.2 The phylum-mode kernel resolver already exists — the vet path just doesn't use it (`Empirical`)

`mycelium-check` already has a whole-phylum mode. `check_phylum_sources` /`check_phylum_dir`
(`crates/mycelium-check/src/lib.rs:303`, `:391`) assemble the files into **one `Phylum`** and run the
kernel's cross-nodule resolver `mycelium_l1::check_phylum` (`checkty.rs:1912`). The module note at
`lib.rs:212-217` states the exact gap: the per-file modes "run `check_nodule` per file, so a
cross-nodule `use a.Foo;` **cannot** resolve (each file is a phylum-of-one)." The test
`phylum_cross_nodule_reference_resolves` (`lib.rs:530-560`) **witnesses both sides**: `b.myc`'s
`use a.*;` resolves as a phylum but FAILS the same check in isolation. This is the leaf's finding
reproduced in-tree: the transpiler's emission is correct under `--phylum`, wrong under oracle.

So the fix is **not** new checking logic — the kernel resolver is built and tested. It is **wiring**:
route the vet path through the phylum resolver. But that wiring exposes a second defect (§1.3).

### §1.3 `PhylumReport` verdicts are all-or-nothing (`Empirical`)

`PhylumReport` (`lib.rs:267-278`) populates its per-nodule `nodules: Vec<NoduleVerdict>` **only when
the whole phylum checks clean**. `NoduleVerdict`'s own doc says "emitted **only** when the whole phylum
checked clean (never fabricated)" (`lib.rs:258-259`, field `:276-277`). The implementation confirms:
on `check_phylum` success every nodule gets a `Clean` verdict (`lib.rs:354-370`); on **any** failure
the report carries a **single** `PhylumError` and `nodules: Vec::new()` (`lib.rs:371-382`). This is
faithful and honest today (the kernel `check_phylum` returns `Result<PhylumEnv, CheckError>` —
one env or one error, `checkty.rs:1912` — so the driver has no per-nodule outcomes to report), but it
means: **a phylum with one gapped nodule credits zero nodules.**

### §1.4 semcore is transpiled + vetted as five isolated nodules (`Empirical`)

`gen/myc-drafts/regenerate.sh:58-63` lists the five mutually-referencing semcore files
(`checkty`/`elab`/`eval`/`mono`/`fuse`) as **five separate `TARGETS` rows**, each its **own**
`<input>` (one `.rs`) and **own** `<outdir>`. The loop at `:79-106` runs
`"$TRANSPILE" --vet "$input" "$outdir"` **once per row** (`:98`) — so each of the five is transpiled
and vetted **independently**, never assembled into the one `l1` phylum they actually form. The
per-file `--vet` then checks each emission in oracle mode (§1.1). Every cross-nodule
`use elab.…;`/`use checkty.…;` between the five is therefore un-creditable.

### §1.5 The net effect (the finding, stated)

The three defects compose: the transpiler emits correct home-qualified cross-nodule references
(§1, leaf), but they are checked file-isolated (§1.1, §1.4) against an all-or-nothing report
(§1.3). So the **entire cross-nodule / cross-phylum lever family** — Import (the leaf's ~12.8%),
**DN-122** external-trait impls (~12.4%), cross-phylum records — is **un-creditable by the current
measurement even when the transpiler is correct.** The `checked_fraction` metric systematically
**under**-counts exactly the lever the wave is building. This is the highest-leverage unblock in the
program because it gates the *measurement* of three large levers at once.

---

## §2 Design — sound partial per-nodule verdicts (P-A, recommended) (`Declared`)

**Goal.** Give `PhylumReport` per-nodule verdicts on a phylum with *some* failing nodules, so the vet
loop can credit the clean nodules **without fabricating any verdict** (G2) and **without ever crediting
a nodule whose cleanliness depended on a failed sibling** (the §6 soundness obligation).

### §2.1 The soundness invariant (load-bearing)

> **Import-closure invariant.** A nodule *N* may be reported `Clean` **iff** (a) *N* itself checks
> clean **and** (b) every nodule in *N*'s transitive import closure checks clean. A nodule whose
> closure contains a failed nodule is reported `Blocked` (naming the failed dependency), **never
> `Clean`.**

This is what makes partial verdicts sound rather than a false-clean hazard (§6, Attack 1b/2). If a
sibling *M* that *N* imports fails, then either *N*'s use of *M*'s items itself fails to check
(so *N* is not locally clean) **or** *N* does not truly depend on the failed part — and the closure
rule conservatively blocks *N* regardless, which can only **under**-credit, never over-credit.
`checked_fraction` is already a documented **lower bound** (`vet.rs:42-44`), so conservative
under-credit is in-contract; false-clean is not.

### §2.2 The mechanism — driver-level closure sub-phylum re-check (reuse `check_phylum`, zero kernel change)

The driver already parses every source and can build the import DAG from the `use` heads (DN-113's
`build_phyla_graph` / DN-101 cross-nodule link give the edge relation; the leaf's `symtab.rs` already
computes sibling resolution). For each nodule *N*:

1. Assemble the **sub-phylum** `{N} ∪ closure(N)` (N plus its transitive in-batch import closure).
2. Run the **existing** `mycelium_l1::check_phylum` on that sub-phylum.
   - Clean ⇒ *N* is `Clean` (its whole closure checked — the invariant holds by construction).
   - Fails ⇒ *N* is `CheckError` (local) or `Blocked` (a closure member failed) — distinguished by
     whether the failure's site is inside *N* vs a dependency; when the distinction is not cleanly
     recoverable, report the weaker `CheckError` (VR-5: never claim a finer class than the evidence
     supports — the same discipline `lib.rs:12-14` already applies to flat `CheckError`s).

This **re-uses the kernel resolver unchanged** (KC-3, DRY — `check_phylum` stays the sole definition of
"checks clean"). It is `O(nodules)` check passes over overlapping closures; memoization of clean
sub-phylum results makes it near-linear (an OQ, §7 OQ-3 — irrelevant for semcore's five files, a perf
note for large stdlib phyla).

### §2.3 The report shape (never conflate per-item with whole-phylum)

`PhylumReport` gains, additively:

- `nodules: Vec<NoduleVerdict>` — now populated **always**, each `Clean | CheckError | Blocked{on}`
  (the `class` field widens from the current `"Clean"`-only, `lib.rs:263-264`).
- `ok: bool` and `error: Option<PhylumError>` — **retained unchanged**: the **whole-phylum** verdict.

Both are reported. The vet loop credits per-nodule-`Clean` items; the `ok` bit remains the stricter
"does the whole phylum check as one" signal. A reader can **never** mistake "k nodules clean" for
"the phylum builds" — both numbers are present (G2). This dual-signal shape is also what keeps the
coherence-conflict-outside-closure case honest (§6, Attack 1c).

### §2.4 Why partial verdicts are a *precondition*, not polish (the house-rule-#4 finding)

If the vet path switched to phylum-mode over the **current** all-or-nothing report, a phylum with one
gapped nodule would credit **zero** — strictly **worse** than today's oracle mode, which credits that
phylum's clean files individually. So the naïve "just switch to `--phylum`" instruction, taken alone,
is a **regression** for every mixed phylum (most stdlib crates). Partial verdicts (§2.1–§2.3) are what
make the switch a monotone improvement. The two ship as **one unit**, partial-first.

---

## §3 Design — vet-path wiring (`Declared`)

### §3.1 Transpiler `--vet` gains a directory/phylum mode

Today `--vet <input> <outdir>` vets each emitted file via oracle single-file (`vet.rs` `MycChecker`,
§1.1). Add a **directory/phylum vet mode** that, after emitting a batch into one dir, invokes
`myc check --phylum <dir> --json` (the stable one-line JSON contract already exists —
`myc-check.rs:139-164`, `run_phylum` `:122`) and consumes the **partial verdicts** of §2, crediting
each `Clean` nodule's emitted items to `checked_clean_items` (the existing file-gated bridge,
`vet.rs:150-156`, generalizes from "file clean" to "nodule Clean"). The denominator
(`non_test_items`, `vet.rs:169-171`) is **unchanged** — so `checked_fraction_phylum` and
`checked_fraction_oracle` share a denominator and are directly comparable (§4).

### §3.2 `regenerate.sh` — semcore as one phylum

Replace semcore's five isolated `TARGETS` rows (`regenerate.sh:58-63`) with **one `semcore` phylum
target** that transpiles all five `.rs` into **one** output dir (via the leaf's `transpile_batch` —
`crates/mycelium-transpile/src/batch.rs` — which already emits a batch with the cross-nodule symtab
installed) and vets that dir with `--phylum`. The stdlib crate targets (`regenerate.sh:64-76`) are
**already per-crate `src/` directories** — each is one phylum — so they switch to phylum-mode vetting
with no target-list change (they gain partial verdicts). The per-target output-subdir discipline
(`regenerate.sh:57` — "never a shared flat dir") is **preserved**: one dir per phylum, not one flat
dir for all.

### §3.3 Cross-phylum dependencies (the DN-113 seam)

A phylum that `use`s **another** phylum's items (`use dep::a.b.Item` — DN-113/M-1060) needs
`check_phylum_with_deps` (`checkty.rs:1935`) with the real dependency phyla, not bare `check_phylum`.
Without deps, such a use fails **unresolved** — which is a **false-FAIL** (safe, conservative), never a
false-clean. So the MVP may ship bare-`check_phylum` phylum-vet (under-crediting cross-phylum deps) and
add dep-wiring as a fast-follow (§7 OQ-2). Never-silent: an unresolved cross-phylum use is still a
recorded gap, not a silent drop.

---

## §4 The measurement-basis honesty question (the crux — VR-5) (`Declared` recommendation over `Empirical` framing)

Switching oracle → phylum mode **changes what `checked_fraction` MEANS**. Items that oracle mode
**falsely fails** (valid cross-nodule uses) become clean. The number will **JUMP** on the switch
commit. The honest handling of that jump is the heart of this note.

### §4.1 Is phylum-mode the more-correct basis? Yes (grounded)

- **Oracle mode checks a counterfactual.** A phylum-of-one is a context **no real build ever
  constructs**: semcore's five files *are* the one `l1`-frontend phylum; the stdlib crates *are* each
  one phylum. Oracle mode's per-file verdict answers "would this nodule check if it were the entire
  universe" — a question whose answer is irrelevant to whether the port is correct.
- **Phylum-mode matches the real target semantics.** `check_phylum` **is** the kernel's real
  cross-nodule resolver (`checkty.rs:1912`; `lib.rs:212-217` states the per-file mode's phylum-of-one
  limitation explicitly). A phylum-mode verdict answers "would this nodule check **as part of the
  phylum it actually belongs to**" — the question the metric should ask.

So phylum-mode is **strictly more faithful**. `checked_fraction_oracle` was an under-count of true
transpiler correctness on exactly the cross-nodule surface; `checked_fraction_phylum` corrects it.

### §4.2 But the jump is a basis correction, NOT lever progress (the honesty obligation)

The recovered items were **already emitted correctly** before the switch — the transpiler did not
improve; the *ruler* did. Presenting the jump as lever progress would be a VR-5 violation (upgrading a
measurement-artifact delta to a claimed gain). It must be disclosed as a **basis change**.

### §4.3 Recommendation — M-A: dual-report a transition cycle, then re-baseline with attribution

1. **Transition cycle (≥1 regen).** Report **both** `checked_fraction_oracle` (the historical
   per-file basis) **and** `checked_fraction_phylum` (the new basis) side by side in `vet.json` /
   `summary.json` / the manifest, over the **same denominator** (§3.1). The §8 series stays comparable
   across the switch and the delta is **visible and named**.
2. **Attribute the one-time delta.** Record, at the switch commit,
   `Δ_basis = checked_fraction_phylum − checked_fraction_oracle` **labeled a basis correction**
   (recovered falsely-failed items), explicitly **separated** from any real lever/transpiler gain
   landing in the same cycle. If a lever also improved in that cycle, its gain is measured
   `phylum`-to-`phylum` (new basis to new basis), never folded into `Δ_basis`.
3. **Re-baseline.** After the transition cycle, `checked_fraction_phylum` becomes the **canonical**
   basis. All subsequent deltas are real gains against the new basis.
4. **Never rewrite history (house rule #3).** The historical oracle-mode §8 numbers are **annotated**
   with the switch commit and `Δ_basis`, **not** retroactively recomputed. Append-only: the record
   shows the ruler changed, when, and by how much.

This is VR-5 applied to the metric itself: the basis change is never-silent, the recovered-vs-earned
split is explicit, and no artifact delta is ever upgraded to a claimed lever gain.

---

## §5 Ranked recommendation with the objective function

### §5.1 Partial-verdict mechanism (the enabling base crate)

| Criterion (weight) | **P-A** driver-closure re-check (reuse `check_phylum`) | P-B kernel per-nodule verdict rewrite | P-C keep all-or-nothing, only switch vet mode |
|---|---|---|---|
| Soundness / no false-clean (must-pass) | **Pass** (closure invariant §2.1) | Pass (if built correctly) | Pass (never partial) |
| Kernel growth (KC-3, high) | **none** — reuses `check_phylum` | high — new kernel verdict path + phases | none |
| Informativeness vs today (high) | **strictly more** (credits independent-clean nodules) | strictly more | **regresses mixed phyla** (§2.4) |
| KISS/YAGNI (high) | **best** — composition over new mechanism | worst | simplest but wrong (regresses) |
| Enables the vet switch safely (must) | **yes** | yes | **no** — makes the switch a regression |
| Perf on large phyla (low) | `O(n)` passes, memoizable (§7 OQ-3) | best (single pass) | best |
| **Verdict** | **RECOMMENDED** | rejected (KC-3 — no kernel change earns its cost yet) | rejected (regresses; violates the point) |

### §5.2 Measurement-basis reporting

| Criterion (weight) | **M-A** dual-report → re-baseline w/ attribution | M-B hard switch, annotate the jump | M-C keep oracle canonical, phylum advisory |
|---|---|---|---|
| VR-5 honesty (must-pass) | **Pass** — recovered-vs-earned explicit | partial — jump visible but comparability weaker | Pass but self-defeating |
| §8-series comparability (high) | **best** — both bases one cycle | weaker — one-point discontinuity | best (no change) |
| Credits the real levers (must) | **yes** after re-baseline | yes | **no** — under-counts forever |
| Append-only (house rule #3, must) | **Pass** — annotate, never rewrite | Pass | Pass |
| Complexity (low) | dual-report one cycle | simplest | simplest but defeats the fix |
| **Verdict** | **RECOMMENDED** | acceptable fallback | rejected (defeats the unblock) |

### §5.3 Build plan, leverage-ranked

1. **Unit 1 — `mycelium-check` partial verdicts (base crate; highest leverage; the enabler).**
   `PhylumReport` per-nodule `Clean|CheckError|Blocked` via §2.2 closure re-check. **Property tests:**
   (i) *soundness* — every `Clean` nodule's whole import closure is clean; (ii) *never-false-clean* — a
   nodule importing a failed sibling is never `Clean` (a `Blocked{on: sibling}` fixture); (iii)
   *monotonicity* — a wholly-clean phylum yields all-`Clean` (partial ⊒ today's all-or-nothing); (iv)
   *never-fabricate* — a `Blocked` nodule is never counted in `checked_clean_items`.
2. **Unit 2 — `mycelium-transpile` phylum/directory vet mode (§3.1).** Consumes the partial verdicts;
   dual-reports both fractions over one denominator. **Property tests:** `checked_fraction_phylum ≥
   checked_fraction_oracle` on any batch (recovers, never loses); denominator invariance;
   `Δ_basis` arithmetic; a tool-unavailable phylum-check is `ToolUnavailable`, never clean (the
   existing `vet.rs:75-98` discipline extended to phylum mode).
3. **Unit 3 — `regenerate.sh` + any CLI batch glue (§3.2).** semcore transpiled+vetted as one phylum;
   stdlib crates gain partial verdicts. **Test:** semcore's five-file phylum vets with the cross-nodule
   `use checkty.…` references resolving (the `lib.rs:530-560` witness, at the harness level).

**Crate/tier note (DN-20).** `mycelium-check` (and `mycelium-l1` if any kernel surface is touched — the
recommendation is that it is **not**) are **base crates**: a touch pulls in every reverse-dependent's
tests, so the **full sweep is desktop-held** (`just check-canary` per-promotion; `just check-full` on
the desktop). All three units are **change-scoped-testable in cloud** (`cargo test -p mycelium-check`,
`-p mycelium-transpile`; `bash gen/myc-drafts/regenerate.sh` skips gracefully without cargo).

---

## §6 Adversarial stress-test (VR-5 / house rule #4)

**Attack 1 — false-clean (the dangerous direction: crediting something a real build rejects).**

- **1a — a use resolves in an assembled phylum that a real build separates.** If the vet dir is a
  *bag of unrelated files* (not one real phylum), `check_phylum` might resolve a cross-nodule use that
  the real module boundary forbids. **Narrowed:** the phylum-vet target **must mirror the real phylum
  boundary** — semcore = the `l1` frontend's files = one phylum; each stdlib crate `src/` = one phylum.
  The design (§3.2) never assembles a mixed bag; the per-phylum output-subdir discipline enforces it.
  **Flag:** do not phylum-vet across a phylum boundary (§7 OQ-1). **Held, with the boundary constraint
  made explicit.**
- **1b — a partial verdict credits N whose cleanliness depended on a failed sibling M.** **Held** by
  the import-closure invariant (§2.1): `Clean` requires N's whole closure clean, so a failed M in N's
  closure forces `Blocked`, never `Clean`.
- **1c — a coherence/orphan conflict between nodules *outside* each other's closures.** N and P both
  `impl Tr for T` (overlapping), but neither imports the other; in the full phylum coherence fails, yet
  each one's closure sub-phylum (§2.2) passes. Crediting each as `Clean` is a **per-item
  transpiler-quality** claim (N's emission is expressible and checks in its closure) — but it is **not**
  a "the full phylum builds" claim. **Narrowed, never conflated:** the report carries **both** the
  per-nodule verdicts **and** the whole-phylum `ok` bit (§2.3), which is `false` here (coherence fails
  phylum-wide). A reader sees `k nodules Clean` **and** `phylum ok: false` — no false "it builds"
  signal. The residual (exact per-item semantics of a phylum-global coherence conflict) is a documented
  OQ (§7 OQ-4), never silent.

**Attack 2 — unsound partial: N reported Clean because a needed sibling was *absent* from the
sub-phylum (a name that should conflict didn't).** **Held:** the closure sub-phylum (§2.2) includes
N's **full** import closure, so a missing dependency can only cause a **false-FAIL** (unresolved →
conservative under-credit), never a false-clean. The whole-phylum `ok` bit is the additional backstop.

**Attack 3 — the switch silently inflates the headline number.** **Held** by §4: `Δ_basis` is
dual-reported and labeled a basis correction, not lever progress; the historical series is annotated,
not rewritten.

**Verdict: HELD, with two explicit narrowings** — (i) phylum-vet **only** over files forming one real
phylum boundary (§3.2 enforces; OQ-1 pins the general rule); (ii) a per-nodule `Clean` is a per-item
transpiler-quality claim **reported alongside, never conflated with,** the whole-phylum verdict (§2.3).
The core false-clean (1b) and unsound-partial (2) attacks are **defeated by the import-closure
invariant**; the boundary (1a) and coherence (1c) attacks are **contained by the boundary constraint
and the dual-signal report**, both documented and never-silent. **No hole requiring a redesign was
found;** the design was *narrowed* (boundary constraint added, dual-signal report made load-bearing),
not weakened.

---

## §7 Open questions (honest residuals)

- **OQ-1 — the phylum-boundary rule.** What formally defines "one real phylum" for a vet dir (a
  `mycelium-proj.toml`? the batch's crate root?)? The MVP uses the target-list convention (semcore =
  the five l1 files; each stdlib crate `src/` = one phylum). Codifying it prevents the Attack-1a bag.
- **OQ-2 — cross-phylum dep vetting.** Wiring `check_phylum_with_deps` (`checkty.rs:1935`, DN-113/
  M-1060) so a phylum that `use`s another phylum's items is vetted with real deps (vs the MVP's
  conservative false-FAIL). Interacts with DN-122 (foreign-trait import).
- **OQ-3 — memoization for large phyla.** The `O(n)` closure re-checks (§2.2) over overlapping
  sub-phyla — cache clean sub-phylum results (irrelevant for semcore; a perf note for big stdlib
  phyla).
- **OQ-4 — per-item semantics of a phylum-global coherence conflict** (Attack 1c) — whether to credit
  each conflicting nodule's per-item quality or block both. The dual-signal report is honest either
  way; the crediting *policy* is the open choice.
- **OQ-5 — item-granularity closure.** A more precise credit than nodule-granularity (§2.1): N credited
  if the specific *items* it imports check clean, even if a sibling's *unrelated* item gapped. More
  precise, needs item-level provenance the kernel doesn't expose today — deferred (YAGNI).

---

## §8 Definition of Done (the ratification gate — house rule #6)

**For the maintainer to move this note Draft → Accepted**, ratify:

1. **The recommendation** — P-A (partial verdicts via driver-level closure re-check, reusing
   `check_phylum` unchanged) + the phylum-mode vet switch as **one partial-first unit** (§2, §3).
2. **The soundness invariant** — the import-closure rule (§2.1) as the binding contract for any partial
   verdict (this is the false-clean guard; ratifying it is ratifying the design's safety).
3. **The measurement-basis decision** — M-A (dual-report a transition cycle → re-baseline with
   `Δ_basis` attributed; annotate, never rewrite the historical §8 series) (§4).
4. **The boundary constraint** — phylum-vet only over files forming one real phylum (§6 Attack 1a /
   OQ-1).
5. **The kernel-untouched intent** — confirm the driver-level approach is preferred over a kernel
   per-nodule-verdict rewrite (KC-3), or direct otherwise.

**For the subsequent implementation (the FLAGGED M-id, §9) to be "done":** Units 1–3 (§5.3) land with
their property tests green (change-scoped in cloud, full sweep desktop-held per DN-20); semcore vets as
one phylum with cross-nodule uses resolving; `vet.json`/manifest dual-report both fractions for the
transition cycle with `Δ_basis` labeled; the emitted `.myc` stays `Declared` and the vet verdict
`Empirical` (never upgraded); and the DN moves to **Enacted** only after it has stepped through
Accepted (house rule #3).

---

## §9 FLAGs (append-only rows for the integrating parent — NOT applied here)

This note edits **only itself**. The following are FLAGGED for the parent to apply (dated,
append-only); I did **not** touch `Doc-Index.md`, `CHANGELOG.md`, or `issues.yaml`.

- **`docs/Doc-Index.md` §Design Notes** — add a DN-124 row after the DN-123 row (`:154`), status
  **Draft** (2026-07-12), summary: vet-harness phylum-visibility fix — partial per-nodule verdicts +
  phylum-mode vet wiring + the measurement-basis (dual-report → re-baseline) decision; `Empirical`
  against `b36ebdbe`/leaf `4f346da3`, `Declared` for the proposed mechanism.
- **`CHANGELOG.md` `[Unreleased]`** — an append-only entry: "DN-124 created (**Draft**): vet-harness
  phylum-visibility — sound partial per-nodule verdicts (import-closure invariant), `myc check
  --phylum` vet wiring, and the `checked_fraction` basis-change honesty decision (dual-report →
  re-baseline). Recommends, ratifies nothing (house rule #3)."
- **`tools/github/issues.yaml`** — **mint one M-id** (next free slot — grep before assigning,
  mitigation #1) for the harness build: *"Vet-harness phylum visibility: partial `PhylumReport`
  verdicts + phylum-mode `--vet` wiring + dual-report measurement basis (DN-124)."* `depends_on`: the
  Import-lever leaf's issue and DN-113/M-1060 (cross-phylum dep vetting, OQ-2). `doc_refs`:
  `corpus:DN-124`, `src:crates/mycelium-check/src/lib.rs:267`, `src:crates/mycelium-transpile/src/vet.rs:26`,
  `src:gen/myc-drafts/regenerate.sh:58`. Status **todo** (design-ratification-gated on this DN).

**Applied at the 2026-07-12 ratification close-out (append-only note, original FLAGs above left
as-authored):** `Doc-Index.md` DN-124 row added at status **Accepted**; `CHANGELOG.md` carries the
ratification entry; **M-1079** minted for the harness build (`depends_on: [M-1060]`; the Import-lever
leaf's own M-id does not yet exist — flagged, not guessed, see the M-1079 issue body).

---

## §10 User stories

- *As a **port-lever author**, I want a correctly-emitted cross-nodule `use checkty.Width;` to be
  **credited** by `checked_fraction`, so that* the Import / external-trait / cross-phylum levers I build
  are measured on the semantics of the real phylum build, not a counterfactual phylum-of-one.
- *As a **maintainer reading the §8 series**, I want the one-time jump when the vet basis switches to be
  **labeled a basis correction with its `Δ_basis`**, so that* I can trust that every subsequent delta is
  a real transpiler/lever gain and never a ruler change dressed as progress (VR-5).
- *As a **vet-harness consumer**, I want a phylum with one gapped nodule to still credit its clean
  nodules **without ever crediting a nodule whose cleanliness depended on the gapped one**, so that* the
  metric is both more informative and never false-clean (the import-closure invariant, G2).
- *As a **kernel maintainer**, I want this fix to **reuse `check_phylum` unchanged**, so that* the small
  auditable kernel (KC-3) grows by zero and the driver composes the trusted resolver rather than
  duplicating it (DRY).

---

*DN-124 — Accepted (2026-07-12). Ratifies P-A + M-A; builds nothing yet — implementation (the
FLAGGED M-id, §9) is a separate, unlanded step (house rule #3: Accepted is not Enacted).*

## Changelog

- 2026-07-12 — DN-124 created (**Draft**): vet-harness phylum-visibility fix. Verify-first problem
  statement (oracle-mode phylum-blindness — `vet.rs:26-27`, `regenerate.sh:58-98`; `PhylumReport`
  all-or-nothing — `lib.rs:258-278,371-382`), the ranked recommendation (P-A partial verdicts via
  import-closure sub-phylum re-check reusing `check_phylum` + phylum-mode vet wiring, one partial-first
  unit), the measurement-basis decision (M-A dual-report → re-baseline with `Δ_basis` attributed,
  append-only), adversarial stress-test (held with two narrowings — real-phylum-boundary constraint +
  dual-signal report), DoD + user stories. `Empirical` where read against `b36ebdbe` / leaf
  `4f346da3`; `Declared` for the proposed mechanism until built + differential-witnessed.
- 2026-07-12 — **Accepted.** Ratified by the maintainer's explicit delegation to the orchestrator
  ("ratify based on objective reasoning and the project's needs/intents, keep to core principles,
  report results"). Ratifies **P-A** (partial per-nodule verdicts via import-closure sub-phylum
  re-check, reusing `check_phylum` unchanged — KC-3, zero kernel growth) and **M-A** (dual-report both
  measurement bases one transition cycle, then re-baseline with `Δ_basis` attributed; annotate, never
  rewrite, the historical oracle §8 series). Basis recorded verbatim in the header table's
  "Ratification basis" row: phylum-mode is the demonstrably-correct basis (proven in-tree by
  `phylum_cross_nodule_reference_resolves`); P-A is KC-3; the import-closure invariant adversarially
  prevents false-clean (VR-5); M-A is VR-5 plus append-only applied to the metric. **Accepted, not
  Enacted** (house rule #3) — Units 1–3 (§5.3) are unbuilt; every verdict/mechanism tag stays
  `Declared` until implemented and differential-witnessed. Minted **M-1079** (vet-harness phylum
  visibility build) this close-out, `depends_on: [M-1060]` (the Import-lever leaf work is in-flight
  under the gap-close-2 wave with no minted M-id yet — flagged, not guessed).
