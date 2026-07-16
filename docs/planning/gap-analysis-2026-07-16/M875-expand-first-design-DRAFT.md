# M-875 — Expand-first macro pre-pass design (Draft / needs-design)

| Field | Value |
|---|---|
| **Artifact** | M-875 design close-out (Draft DN style) |
| **Status** | **Draft / needs-design** — design artifact only; **not Accepted**, not Enacted, **no implement** in this note (house rule #3). Maintainer ratification required before any expand-first code lands. |
| **Path rationale** | Planning-path draft under `docs/planning/gap-analysis-2026-07-16/` rather than a new numbered DN: **DN-100 already Accepted** the mechanism decision (2026-07-10). Minting a second DN that re-decides the same question would either re-litigate an Accepted note or create a duplicate status surface. This draft **defers** mechanism ratification to DN-100 and fills the residual design surface M-875's DoD still needs (pipeline integration, env constraints, sequencing vs M-1084/M-1037, implement-issue DoD). |
| **Decides (proposes)** | For ratification of *this* close-out: (1) M-875's design question is **answered** by DN-100 Rank 1 + Rank 2 as scoped; (2) the integration point is a **pre-`syn` expand step** in `mycelium-transpile` (optional flag); (3) ranked path options including the `cargo expand` vs `rustc -Zunpretty=expanded` split; (4) sequencing vs Import/conversion levers; (5) DoD for the follow-on implement issue (**M-1032** / ENB-9). |
| **Does not** | Implement expand-first code; edit `crates/**`; change DN-100 status; claim a `checked_fraction` win; force nightly into MSRV/CI without a further ADR. |
| **Grounds** | DN-34 §8.3/§8.5; `crates/mycelium-transpile/fixtures/UNION-BACKLOG.md`; DN-100 (Accepted); M-875 / M-1032 bodies; ADR-041 (MSRV 1.96.1); one-shot handoff Epic B5 (2026-07-16). |
| **Date** | 2026-07-16 |
| **Base tip** | `origin/dev` @ `2ac85a84`+ |
| **Task** | M-875 — design/arch only (ONE-SHOT B5). |

> **Transparency (VR-5 / G2 / house rule #4).** Union counts below are **`Empirical`** (measured by the
> transpile batch over the 6-crate core-lib slice / DN-100 whole-corpus profile). Unbuilt mechanisms and
> post-expansion `checked_fraction` effects are **`Declared`/uncertain**. This note does **not** upgrade
> DN-100's honest ROI framing. Agree only on merit: expand-first is a **profiling and expressibility**
> lever first; claiming a checkability win without a measured before/after is a defect.

---

## §1 Problem

`syn` does not expand macros. The transpiler's exhaustive item dispatch therefore records every
`macro_rules!` definition and every item-position macro invocation as a never-silent gap
(`Category::MacroDef` / `Category::MacroInvocation` in `transpile.rs` ~644–655). Body- and
expression-position macros similarly poison multi-statement bodies or fall through to
`Category::MacroInvocation` (except the small DN-127 / M-1090 WU-3 `write!`/`format!` expression
shim in `emit/macros/`).

**Demand data (DN-34 §8.5 + UNION-BACKLOG, `Empirical`):**

| Signal | Value |
|---|---|
| Macros share of union gaps (6-crate slice) | **73 / ~22%** (68 `MacroInvocation` + 5 `MacroDef`) |
| std-cmp pilot (DN-34 §8.3) | **~55%** of residue is macro-generated (`impl_narrow_int!` etc.) |
| DN-100 whole-corpus item-position macro gaps | **82** across 18 files; **≈93% custom project `macro_rules!`** |

**User story (M-875):** as a self-hosting engineer, I want macro-generated impls to become ordinary
items the emitter already attempts, so the gap report shows the *true* post-expansion surface
instead of an opaque macro skeleton (~55% of std-cmp).

**What expand-first is:** run a Rust macro-expansion step over source **before** the transpiler's
`syn::parse_file` walk, so expanded impls/fns become ordinary `syn::Item`s the existing dispatch
can emit or gap by finer category. What it is **not:** a Mycelium-side macro system (that is
DN-110 native metaprogramming — complementary target facility, not this issue's scope).

---

## §2 Options (toolchain paths)

Three concrete expand paths (plus status quo). DN-100 grouped "cargo expand / rustc unpretty" as
Alt A; this close-out **splits** them so env constraints are explicit.

### Option 1 — `cargo expand` (wrapper)

Shell out to [`cargo-expand`](https://github.com/dtolnay/cargo-expand) per crate (or filtered
targets), then feed expanded Rust text into the existing transpile driver.

| | |
|---|---|
| **How** | `cargo expand` → `rustc -Zunpretty=expanded` under the hood for the package |
| **Pros** | Correct for **custom `macro_rules!` + proc-macros + std** (compiler's own expansion); handles the §2-dominant project macros DN-100 measured |
| **Cons** | Needs a **buildable crate graph** + **nightly** (or a nightly component for `-Z`); network/build-order weight; expanded AST is desugared (FQ paths, hygiene noise) — harder for heuristics |
| **Never-silent** | Expand failure / missing tool → per-crate skip + record, never silent empty emission (G2) |

### Option 2 — direct `rustc -Zunpretty=expanded`

Invoke `rustc` with `-Zunpretty=expanded` (and the crate's `--extern` / `cfg` / edition flags)
without the `cargo-expand` binary.

| | |
|---|---|
| **How** | Driver builds a `rustc` cmdline equivalent to what `cargo expand` would issue |
| **Pros** | Same expansion fidelity as Option 1; one fewer cargo subcommand dependency |
| **Cons** | **Still requires nightly `-Z`**; still needs correct crate metadata (deps, cfgs, features) — re-implements a slice of what cargo already knows; easy to get wrong flags → partial/wrong expansion (VR-5 risk if treated as truth) |
| **Never-silent** | Same skip contract as Option 1 |

**Relation:** Option 1 is a **packaging** of Option 2. Fidelity is the same class; the real choice is
"use cargo's metadata resolution" vs "hand-roll rustc flags." Prefer Option 1 unless cargo-expand
is unavailable and a measured rustc cmdline is already maintained elsewhere.

### Option 3 — vendored / in-process expander

| | |
|---|---|
| **3a `macro_rules` engine** | Vendor or implement a declarative-macro matcher/transcriber; expand only in-tree `macro_rules!` |
| **3b std-macro allowlist shim** | Hand rules for a fixed set (`matches!`, `assert!`, `vec!`, …) — DN-100 Alt C; partially prefigured by `emit/macros` for `write!`/`format!` |
| **Pros** | No nightly; no crate build; stays on MSRV 1.96.1 stable; tiny (3b) or offline (3a) |
| **Cons** | **3a** re-implements a non-trivial compiler slice (YAGNI/KISS; incorrect expansion worse than a gap — G2/VR-5). **3b** clears only the std tail (DN-100: ≤6 of 82 on the measured corpus) and cannot expand project `impl_narrow_int!` / `impl_std_error!` |
| **Never-silent** | Macro outside allowlist/engine → existing `MacroInvocation`/`MacroDef` gap |

### Option 4 — status quo (hand-expand at port time)

Keep macros as never-silent gaps; porters expand by hand (M-993 path). Zero new deps; no
automation of the 73/82 bucket.

---

## §3 Environment constraints

| Constraint | Binding | Implication for expand-first |
|---|---|---|
| **MSRV 1.96.1** (ADR-041) | Committed pin; no silent bump | Default `just check` / workspace build stays **stable 1.96.1**. Nightly is **not** MSRV. |
| **No silent nightly force** | House rule + ADR discipline | Requiring nightly as a **default** pre-pass or CI gate needs an **explicit ADR** (or remains opt-in + skip-graceful). This draft does **not** propose such an ADR. |
| **CI self-hosted / advisory** | Remote CI is `workflow_dispatch`, skip-graceful tools | Expand must **not** gate `just check` or advisory CI when nightly/`cargo-expand` is absent. |
| **Session FLAG (M-875 body)** | Early env lacked nightly + cargo-expand | Treat missing expander as **expected**, not a red build — record `ExpandSkipped{reason}` (`Empirical` when measured). |
| **KC-3 / minimal deps** | Transpiler already avoids `clap`; syn scoped to transpile crate | Prefer shell-out over new workspace deps; keep expand out of the kernel/self-host surface. |

**Bottom line:** any nightly-dependent path (Options 1–2) is **opt-in, off-by-default, never-gating**.
Stable-only shims (Option 3b) may be always-available. This matches DN-100's Accepted Rank 1 + Rank 2.

---

## §4 Integration point in `mycelium-transpile` (pre-syn)

Today's pipeline (simplified):

```text
read .rs text
  → syn::parse_file          # transpile_source_with_ctx
  → dispatch_item / emit     # exhaustive syn::Item walk
  → GapReport + .myc text
  → optional --vet (myc check)
```

**Proposed expand-first insertion (Declared — unbuilt):**

```text
read .rs / crate root
  → [optional] expand pre-pass   # NEW: only when --expand (name TBD)
       Option 1/2: cargo expand | rustc -Zunpretty=expanded  (crate- or file-scoped)
       Option 3b:  pure-text / token allowlist rewrite (no rustc)
  → syn::parse_file(expanded_or_raw)
  → existing dispatch / emit / gaps / --vet   # unchanged
```

| Concern | Design rule |
|---|---|
| **Where** | **Before** `syn::parse_file` in `transpile_source_with_ctx` (or a thin wrapper used by `transpile_file` / `transpile_batch`). Post-syn expand is wrong: `syn` already lost unexpanded macro tokens as opaque `Item::Macro` / `Expr::Macro`. |
| **CLI** | Opt-in flag (e.g. `--expand`), default off; composable with `--vet` / `--files` / dir mode. No `clap` required (hand-rolled args, same as `--vet`). |
| **Batch mode** | Prefer **crate-scoped** expand once per package, then map expanded modules back to output paths; file-at-a-time expand is a degraded mode when crate metadata is unavailable. |
| **Provenance** | Gap report / summary must record `expanded` (true/false) and `expand_backend` (`cargo-expand`, `rustc-unpretty`, `shim`, `none`, or `skipped`) — never silent about whether a run was expanded or raw (M-875 DoD). |
| **Guarantee tag** | Expanded emissions stay **`Declared`** until a differential upgrades them (M-991). Expansion changes *what is attempted*, not the tag. |
| **Collision surface** | Lands only in `crates/mycelium-transpile/**` (M-1032: outside semcore lane). |

**Partial landing already present (do not re-do as expand-first):** expression-position
`write!`/`format!` lowering (`emit/macros`, DN-127 / M-1090 WU-3) is an **in-emitter Alt C shim**,
not a pre-syn expand. Expand-first remains the path for item-position project macros that dominate
the union backlog.

---

## §5 Honesty: expanded bodies still hit type / surface gaps

DN-100 §3 carries forward unchanged (Accepted ROI framing):

1. Expand **necessarily** can raise **`expressible_fraction`** (opaque macro items become visible
   items the emitter attempts).
2. On this corpus, **`checked_fraction` is uncertain and likely small** after expand alone:
   - `impl_narrow_int!` → `Narrow` impls the emitter **already gaps** (fallible, no `= expr` surface).
   - `impl_std_error!` → `Display`/`Error` bodies that still need format/Display levers.
3. Vet poison classes (Import, reserved words, conversion fabrication) still dominate
   **checkability** (DN-34 §8.7–§8.8). Expand without those closed often **reclassifies** gaps
   (`MacroInvocation` → `Impl` / `Conversion` / `Other`) rather than clearing files — more honest
   profiling, not a free check win (VR-5: do not claim otherwise).

**Metric contract for implement work:** primary success metric = before/after
**`expressible_fraction`** (and category histogram) on a macro-heavy target. Report
`checked_fraction` but **do not** require it to move for DoD.

---

## §6 Sequencing vs M-1084 / M-1037 (and related)

| Lever | Role | Relation to expand-first |
|---|---|---|
| **M-1084** Import net-close | Symtab resolves cross-nodule `use` → **checked_fraction** | **Sequence before** expecting expand to help *check*: expanded files still emit `use`s; unresolved imports poison whole-file vet. Expand may run earlier as pure profiling. |
| **M-1037** Conversion identity | Map `.clone`/`.into`/… without fabricating prims | **Sequence before** or with expand for checkability of expanded method bodies that are conversion no-ops. |
| **M-874** type-coverage (historical) | Surface types | M-875 body: "post-expansion still hits type gaps — sequence type work first." Still true for *checked* ROI; expand alone is valid for *expressible* profiling. |
| **M-1086** derive residual | DeriveAttr class | Complementary; expand does not replace derive strategy (proc-macro derive still needs Option 1/2 or native DN-110 mapping). |
| **M-1032** ENB-9 implement | Build the pre-pass | **Blocked on design** (this note + DN-100). Depends_on includes M-875. |
| **DN-110** native meta | Mycelium `lower`/`derive` | Complementary target; not a re-decision of expand-first (M-875 note 2026-07-10). |

**Recommended schedule (one-shot Epic B, 2026-07-16):**

1. **B1 M-1084 → B2 M-1037** (serial transpile net-close for *check*).
2. **B5 this design** (parallel OK — docs only).
3. **Implement expand-first (M-1032)** after design Accepted: opt-in profiling mode first; measure
   expressible delta; only then consider whether Rank-2 shim expansion should grow.

Expand-first **may** be prototyped in parallel as a non-default flag without waiting for M-1084 if
the only claimed metric is expressibility — but marketing a checked win without B1/B2 is dishonest.

---

## §7 Never-silent contract (G2)

1. Unexpandable macro → existing category gap (or finer post-expand category), **never** a fabricated
   expansion body.
2. Missing nightly / cargo-expand / rustc `-Z` / build failure → **`ExpandSkipped`**, raw path
   continues, summary line states why — never a green empty success.
3. Summary always states expand mode used.
4. Emissions remain **`Declared`**.

---

## §8 Definition of Done

### §8.1 This design task (M-875) — satisfied by this artifact + DN-100 when ratified as close-out

- [x] Design decision recorded on expand-first approach + toolchain requirement (DN-100 Accepted
      Rank 1+2; this draft restates + adds pipeline/sequence/env).
- [x] Options compared: `cargo expand` vs `rustc -Zunpretty=expanded` vs vendored/shim vs status quo.
- [x] Env constraints: MSRV 1.96.1, no silent nightly force, CI skip-graceful.
- [x] Integration point: pre-syn, opt-in flag, provenance fields.
- [x] Honesty: expressible vs checked; sequence vs M-1084/M-1037.
- [x] Status remains **Draft / needs-design** until maintainer marks M-875 design Accepted (or
      flips M-875 to a state that points at DN-100 + this note as the design basis).
- [ ] **No code** in this task (enforced).

### §8.2 Future implement issue (M-1032 / ENB-9) — DoD for a later leaf

1. Mechanism matches ratified scope: **opt-in `--expand` (name TBD)**, off by default; backend
   **Option 1 (`cargo expand`)** preferred; Option 2 only if justified; **Option 3b shim** may land
   independently always-on; **no Option 3a** as first step; **no default-on nightly path** without ADR.
2. Pre-syn insertion only; existing raw path byte-stable when flag off.
3. Before/after **`expressible_fraction`** on a macro-heavy target (M-1032: std-cmp; DN-100 also
   names `impl_narrow_int!` sites) recorded `Empirical`; `checked_fraction` reported, not required
   to rise.
4. Unexpandable / skip path never-silent (§7); emissions `Declared`.
5. `cargo fmt` / `clippy -D warnings` / `test -p mycelium-transpile` green; data-driven tests for
   skip-graceful and (if shim) each allowlisted rule.
6. Does not gate `just check` when expander missing.

---

## §9 Risks

| Risk | Severity | Mitigation |
|---|---|---|
| Nightly forced into CI/MSRV by accident | High | Opt-in only; skip-graceful; no ADR ⇒ no default |
| Desugared expand confuses type/path heuristics | Med | Treat expand as profiling mode; keep raw path; refine maps only with measured need |
| Partial/wrong vendored expand fabricates code | High | Do not ship Option 3a first; never silent wrong expansion |
| Reclassification mistaken for progress | Med | Publish category histogram before/after; primary metric expressible, not "fewer Macro\* gaps" alone |
| Parallel implement races B1/B2 on `mycelium-transpile` | Med | Serial ownership on crate; design-only until orch schedules M-1032 |
| Duplicate design vs DN-100 drift | Med | This draft **defers** mechanism to DN-100; changes to mechanism require superseding DN-100, not editing this file into a second authority |

---

## §10 Recommendation (ranked)

| Rank | Choice | Role |
|---:|---|---|
| **1** | **Option 1 — `cargo expand`**, opt-in, off-by-default, never-gating profiling mode | Only path that expands the corpus-dominant **custom** `macro_rules!`; aligns with **DN-100 Accepted Rank 1** |
| **2** | **Option 3b — std-macro shim** (always-available, independent) | Mechanizes port-time hand work; low measured-corpus ROI but tiny/stable; aligns with **DN-100 Rank 2**; extends existing `write!`/`format!` shim carefully |
| **3** | **Option 2 — direct `rustc -Zunpretty=expanded`** | Fallback packaging of Rank 1 when cargo-expand is unavailable; same nightly constraint — not a way around env limits |
| **Not recommended (first step)** | **Option 3a vendored `macro_rules` engine** | YAGNI/KISS; incorrect expansion worse than a gap |
| **Fallback** | **Option 4 status quo** | Honest default when expand skipped |

**One-liner:** *Ship expand-first as an opt-in `cargo expand` pre-syn profiling mode (DN-100 Rank 1), plus a small stable std-macro shim (Rank 2); never default-on nightly; measure expressible_fraction, not a claimed checked win; implement under M-1032 after this design is Accepted.*

**M-875 status proposal (FLAG for integrator, not applied here):** after maintainer ratification of
this close-out, flip M-875 from `needs-design` → design-complete / unblock M-1032 (exact label per
tracker convention), with `doc_refs` including this path and `corpus:DN-100`. Do **not** mark
expand-first Enacted until M-1032 lands code.

---

## §11 Relationship map

```text
DN-34 §8.5 (demand) ──► M-875 (this design) ──► M-1032 implement (ENB-9)
                              │
                              ├── defers mechanism to DN-100 (Accepted)
                              ├── native target facility: DN-110 (complementary)
                              └── sequence check ROI after M-1084, M-1037
```

---

## §12 Changelog (append-only)

- **2026-07-16** — Draft created (ONE-SHOT B5 / M-875). Design-only; no implement. Planning-path
  draft (not a new DN number) because DN-100 already Accepted the mechanism. Covers problem,
  Options 1–4 (cargo expand / rustc unpretty / vendored+shim / status quo), env constraints
  (MSRV 1.96.1, no silent nightly), pre-syn integration, honesty vs type gaps, sequencing vs
  M-1084/M-1037, DoD for M-875 and M-1032, risks, ranked recommendation. Status: **Draft /
  needs-design**.
