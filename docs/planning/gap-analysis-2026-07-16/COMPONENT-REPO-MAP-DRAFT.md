# D1 — Component-repo map (DRAFT)

| Field | Value |
|---|---|
| **Program** | [`PROGRAM-SELFHOST-DECOMPOSE-2026-07-17.md`](./PROGRAM-SELFHOST-DECOMPOSE-2026-07-17.md) Phase **D** step **D1** |
| **Status** | **Draft map** — operational plan for this program; **not** a claim that DN-88 §3 is met |
| **Honesty** | Entire map is **`Declared`** until extract + CI green (Phase D DoD). Source inventory is tip-bound to monorepo `dev` at authoring |
| **Owner** | `tzervas/*` GitHub (org/user of monorepo `tzervas/mycelium`) |
| **Ground** | DN-88 · DN-27 · DN-34 · ADR-022 §7/§10 · ADR-036 · ADR-038 · ADR-042 · ADR-045 · RFC-0016 rings · RFC-0031 |
| **License (all first-party)** | **MIT only** (ADR-022 §7; CONTRIBUTING) |
| **Does not** | Create repos · rewrite `main` · delete monorepo content · enact DN-88 |

---

## 0. Honesty vs DN-88 (never-silent tension)

| Claim | Tag | Basis |
|---|---|---|
| DN-88 gates **full** spore/phylum decomposition on **production-ready dogfood** (interp 100% checkout **then** AOT `.myc` per unit) | `Declared` (DN-88 §3) | DN-88 Proposed; ADR-038 §2.8 terminal `1.0.0` reading |
| This program runs an **earlier** path: archive Rust monorepo tip → **component Rust repos** → transpile to paired `*-myc` → umbrella re-export | `Declared` (user directive) | PROGRAM-SELFHOST-DECOMPOSE end-state |
| Managerial phylum **re-export** mechanism does **not** exist in-language today (`phylum` reserved-not-active; `[surface].exports` own-nodules only) | `Checked` (DN-88 §4.2) | DN-88 FLAG → follow-on RFC still open |
| Map is **operational for Phase D/T/R of this program**, not ratification that DN-88’s dogfood bar is closed | `Declared` | This document |

**Implication:** Phase D creates **Rust component repos** (and later T fills `*-myc` siblings). DN-88’s GHCR-spore + managerial-re-export end-state remains the **later** horizon; do not market D as “DN-88 complete.”

**Precedent (already extracted):** `crates/mycelium-tero` → `tzervas/tero-rs` (see `tools/tero-rs/PROVENANCE.md`). Pattern: extract → pin → monorepo consumes published artifact.

---

## 1. Phase D Definition of Done

| # | Criterion |
|---|---|
| D-DoD-1 | Every row in §3 has a created `tzervas/<repo>` (empty or seed README + LICENSE MIT + minimal CI) |
| D-DoD-2 | Minimal CI green enough to **accept** a first code push (fmt/clippy/test skeleton or `workflow_dispatch` parity note) |
| D-DoD-3 | Monorepo still holds **full** tree + **archive pointer** (Phase A tag/branch + SHA recorded in program file) |
| D-DoD-4 | This map (or a successor) lists **source paths**, **outbound deps**, **paired `*-myc` name** per component |
| D-DoD-5 | No monorepo content deleted; extract is copy/filter into new repos, not force-rewrite of trunks |
| D-DoD-6 | Umbrella strategy chosen (§5) at least as `Declared` with pin mechanism sketched |

**Phase order remains G → A → D → T → R.** This draft is ready so D can run autonomously once G gates clear and A is verified.

---

## 2. Naming conventions (proposed)

| Kind | Pattern | Example |
|---|---|---|
| Rust component repo | `mycelium-<unit>` | `mycelium-std-io`, `mycelium-core` |
| Phase T Mycelium sibling | `<same>-myc` | `mycelium-std-io-myc` |
| Managerial re-export (later) | `mycelium-<group>` presenting members | `mycelium-std` (re-exports std-* spores) |
| Umbrella presentation | **FLAG** — `mycelium-lang` *or* keep monorepo name | §5 |
| Already out | `tero-rs` (not `mycelium-tero`) | PROVENANCE pin |

**Paired-repo rule (T3):** every extractable component that will receive `.myc` gets a **`*-myc` repo** named by appending `-myc` to the Rust component repo name. Kernel crates that stay Rust-only longer still reserve the name; emptiness is OK until T.

---

## 3. Component list (by group)

### 3.0 Legend

| Column | Meaning |
|---|---|
| **Repo** | Suggested `tzervas/<name>` |
| **`*-myc`** | Phase T sibling |
| **Sources** | Monorepo paths to extract (primary) |
| **Outbound** | Other **components** this unit depends on (not external crates) |
| **CI** | Minimal gate sketch |

Dependency edges are from workspace `Cargo.toml` path deps at tip (`Empirical` inventory). Cycles/near-cycles in the monorepo are **FLAGGED** under kernel (§3.1 notes).

---

### 3.1 Kernel / trusted base & language core

**Grouping choice (recommended): seam multi-crate repos, not 1:1 crate→repo for every kernel crate.**

Justification: `mycelium-l1` / `mycelium-interp` / `mycelium-mlir` form a tightly coupled DAG (l1 → cert, dense, interp, mlir, numerics, select, stack, workstack, core). One-repo-per-crate would force a long path-dep / publish-order chain for every tiny change. DN-88 §5 “cohesive independently-versionable unit” + acyclic layering favors **ring seams** here. **FLAG-K1:** maintainer may still prefer 1:1 for max extract autonomy — table lists both **recommended multi-crate repo** and **member crates**.

| Repo (recommended) | Member crates | Sources | Outbound deps (other components) | `*-myc` | CI (minimal) |
|---|---|---|---|---|---|
| **`mycelium-core`** | `mycelium-core`, `mycelium-stack`, `mycelium-workstack` | `crates/mycelium-core/`, `crates/mycelium-stack/`, `crates/mycelium-workstack/` | *(none internal)* | `mycelium-core-myc` | `cargo test -p mycelium-core -p mycelium-stack -p mycelium-workstack`; MSRV 1.96.1; MIT LICENSE |
| **`mycelium-value`** | `mycelium-dense`, `mycelium-numerics`, `mycelium-vsa`, `mycelium-vsa-decode`, `mycelium-select` | `crates/mycelium-dense/`, `…-numerics/`, `…-vsa/`, `…-vsa-decode/`, `…-select/` | `mycelium-core` (+ select→interp **FLAG-K2** — see notes) | `mycelium-value-myc` | scoped `cargo test` on members; forbid-unsafe crates keep `#![forbid(unsafe_code)]` |
| **`mycelium-runtime`** | `mycelium-sched`, `mycelium-rt-abi`, `mycelium-interp`, `mycelium-cert`, `mycelium-diag` | matching `crates/` dirs | `mycelium-core`, `mycelium-value` | `mycelium-runtime-myc` | interp + cert unit tests; no GPU/VSA-full in default CI |
| **`mycelium-l1`** | `mycelium-l1` | `crates/mycelium-l1/` | `mycelium-core`, `mycelium-value`, `mycelium-runtime`, **`mycelium-codegen` (mlir)** | `mycelium-l1-myc` | lexer/parser/check unit + regression; change-scoped |
| **`mycelium-codegen`** | `mycelium-mir-passes`, `mycelium-mlir` | `crates/mycelium-mir-passes/`, `crates/mycelium-mlir/` | `mycelium-core`, `mycelium-value`, `mycelium-runtime` | `mycelium-codegen-myc` | mir-passes always; mlir JIT **optional job** (host may lack LLVM) |

**Notes / FLAGs**

| ID | Issue |
|---|---|
| **FLAG-K1** | Multi-crate vs 1:1 kernel repos — default multi-crate; override = one GitHub repo per row-member crate |
| **FLAG-K2** | `mycelium-select` depends on `mycelium-interp` while l1/mlir also depend on select — layering is not a pure DAG at crate level. Keep `select` in **value** or **runtime**? **Recommend runtime** if extract order demands interp-before-select; table currently co-locates select with value (semantic affinity). Resolve at D2 extract |
| **FLAG-K3** | `mycelium-l1` ↔ `mycelium-proj` (proj depends on l1; check/fmt depend on both). Keep proj in tooling (§3.3) with git dep on l1 |

**1:1 alternative (if FLAG-K1 → 1:1):** each of the 18 kernel/runtime crates becomes `tzervas/mycelium-<crate-suffix>` with the same outbound edges as the inventory below.

<details>
<summary>Per-crate outbound inventory (kernel-ish)</summary>

| Crate | Internal deps |
|---|---|
| `mycelium-core` | — |
| `mycelium-stack` | — |
| `mycelium-workstack` | stack |
| `mycelium-dense` | core |
| `mycelium-numerics` | core |
| `mycelium-vsa` | core |
| `mycelium-vsa-decode` | core, select, vsa |
| `mycelium-sched` | core |
| `mycelium-select` | core, interp |
| `mycelium-rt-abi` | core, interp, sched |
| `mycelium-interp` | core, dense, numerics, sched, vsa, workstack |
| `mycelium-cert` | core, interp, numerics, vsa |
| `mycelium-diag` | core |
| `mycelium-mir-passes` | core, workstack |
| `mycelium-mlir` | cert, core, dense, interp, mir-passes, numerics, rt-abi, sched, select, vsa, workstack |
| `mycelium-l1` | cert, core, dense, interp, mlir, numerics, select, stack, workstack |

</details>

---

### 3.2 Std phyla (Rust crates + `lib/std` nodules)

**Grouping choice (recommended): one repo per std phylum / crate** — matches DN-88 §4 exemplar (`std-io`, `std-fs`, `std-vsa`, `std-numerics`, …) and RFC-0016 module taxonomy. Ring-level fat repos are the **fallback** if N≈25 repos is too operationally heavy (**FLAG-S1**).

**`.myc` co-location:** today’s `lib/std/*.myc` lives in one monorepo phylum (`lib/std/mycelium-proj.toml`). On extract:

- Rust component repo holds `crates/mycelium-std-<x>/` (reference).
- Phase T `*-myc` holds the graduated `.myc` nodule(s) + its own `mycelium-proj.toml`.
- Until T, optional seed: copy matching `lib/std/<nodule>.myc` into `*-myc` as **Declared** draft (do not claim production-ready).

| Repo | Sources (Rust + optional .myc seed) | Outbound (components) | `*-myc` | Ring (RFC-0016) | CI |
|---|---|---|---|---|---|
| `mycelium-std-core` | `crates/mycelium-std-core/`; seed `lib/std/core.myc`, `option.myc`, `result.myc` | `mycelium-core` | `mycelium-std-core-myc` | R0 | `cargo test -p mycelium-std-core` |
| `mycelium-std-error` | `crates/mycelium-std-error/`; `error.myc` | core, std-core, std-recover | `mycelium-std-error-myc` | R0 | scoped test |
| `mycelium-std-ternary` | `…-std-ternary/`; `ternary.myc` | core | `mycelium-std-ternary-myc` | R1 | scoped test |
| `mycelium-std-content` | `…-std-content/`; `content.myc` | core | `mycelium-std-content-myc` | R1 | scoped test |
| `mycelium-std-dense` | `…-std-dense/` | core, value/dense | `mycelium-std-dense-myc` | R1 | scoped test |
| `mycelium-std-select` | `…-std-select/`; `select.myc` | core, select | `mycelium-std-select-myc` | R1 | scoped test |
| `mycelium-std-vsa` | `…-std-vsa/` | core, vsa | `mycelium-std-vsa-myc` | R1 | scoped; VSA-heavy optional |
| `mycelium-std-swap` | `…-std-swap/`; `swap.myc` | core, cert, numerics | `mycelium-std-swap-myc` | R1 | scoped test |
| `mycelium-std-collections` | `…-std-collections/`; `collections.myc` | core, std-core | `mycelium-std-collections-myc` | R2 | scoped test |
| `mycelium-std-cmp` | `…-std-cmp/`; `cmp.myc` | core, std-core | `mycelium-std-cmp-myc` | R2 | scoped test |
| `mycelium-std-iter` | `…-std-iter/`; `iter.myc` | core, std-core | `mycelium-std-iter-myc` | R2 | scoped test |
| `mycelium-std-math` | `…-std-math/`; `math.myc` | core, numerics, std-core, std-numerics | `mycelium-std-math-myc` | R2 | scoped test |
| `mycelium-std-numerics` | `…-std-numerics/`; `numerics.myc` | core, numerics, std-core | `mycelium-std-numerics-myc` | R2 | scoped test |
| `mycelium-std-text` | `…-std-text/`; `text.myc` | core, std-core | `mycelium-std-text-myc` | R2 | scoped test |
| `mycelium-std-fmt` | `…-std-fmt/`; `fmt.myc` | core, std-core, std-io | `mycelium-std-fmt-myc` | R2 | scoped test |
| `mycelium-std-diag` | `…-std-diag/`; `diag.myc` | diag | `mycelium-std-diag-myc` | R2 | scoped test |
| `mycelium-std-recover` | `…-std-recover/`; `recover.myc` | core, diag, interp, std-core | `mycelium-std-recover-myc` | R2 | scoped test |
| `mycelium-std-spore` | `…-std-spore/`; `spore.myc` | core, proj, spore, std-content, std-core, std-numerics, std-vsa, vsa | `mycelium-std-spore-myc` | R2 | scoped test |
| `mycelium-std-runtime` | `…-std-runtime/` | core, rt-abi, sched, select, std-core | `mycelium-std-runtime-myc` | R2 | scoped test |
| `mycelium-std-testing` | `…-std-testing/`; `testing.myc` | core, diag, proj, std-core | `mycelium-std-testing-myc` | R2 | scoped test |
| `mycelium-std-io` | `…-std-io/`; `io.myc` | core, std-core | `mycelium-std-io-myc` | R2 | scoped test |
| `mycelium-std-fs` | `…-std-fs/` | core, std-core | `mycelium-std-fs-myc` | R2 | scoped test |
| `mycelium-std-time` | `…-std-time/`; `time.myc` | core, std-core | `mycelium-std-time-myc` | R2 | scoped test |
| `mycelium-std-rand` | `…-std-rand/` | core, std-core | `mycelium-std-rand-myc` | R2 | scoped test |
| `mycelium-std-sys` | `…-std-sys/` | — | `mycelium-std-sys-myc` | host seam | scoped test |
| `mycelium-std-sys-host` | `…-std-sys-host/` | std-rand, std-sys, std-time | `mycelium-std-sys-host-myc` | host seam | scoped test |
| `mycelium-std-conformance` | `…-std-conformance/` | cert, core, interp, l1, mlir, many std-* | `mycelium-std-conformance-myc` | meta | larger job OK; not a publish surface |

**Managerial std (later, DN-88 pattern — not Phase D blocker):**

| Repo | Role | Outbound |
|---|---|---|
| `mycelium-std` | Managerial re-export phylum (when mechanism exists) | all `mycelium-std-*-myc` (or Rust) by pin |
| *(until mechanism)* | Umbrella or monorepo `lib/std` continues to present unified `exports` list | — |

**FLAG-S1 — one-vs-many std:**

| Option | Pros | Cons |
|---|---|---|
| **A. One repo per phylum (recommended)** | DN-88 exemplar; minimal consumer surface; independent versioning | ~27 GitHub repos; more CI surfaces |
| **B. Ring-level** (`mycelium-std-r0`, `-r1`, `-r2`, `-host`) | Fewer repos; matches RFC-0016 rings | Fat versioning; PRs collide across modules |
| **C. Single `mycelium-std`** | Simplest ops | Defeats decomposition goal |

**Default for autonomous D:** **Option A**; maintainer can collapse to B before D3 create-batch without redoing T naming (rename map in one PR).

---

### 3.3 Tooling / project / CLI

| Repo | Sources | Outbound | `*-myc` | CI |
|---|---|---|---|---|
| `mycelium-cli-common` | `crates/mycelium-cli-common/` | — | `mycelium-cli-common-myc` | unit tests |
| `mycelium-proj` | `crates/mycelium-proj/` | core, l1 | `mycelium-proj-myc` | manifest + unit |
| `mycelium-spore` | `crates/mycelium-spore/` | core, proj | `mycelium-spore-myc` | unit |
| `mycelium-build` | `crates/mycelium-build/` | core | `mycelium-build-myc` | unit |
| `mycelium-check` | `crates/mycelium-check/` | cli-common, l1, lsp, proj | `mycelium-check-myc` | unit + smoke `myc check` fixture |
| `mycelium-fmt` | `crates/mycelium-fmt/` | cli-common, l1, proj, workstack | `mycelium-fmt-myc` | unit |
| `mycelium-lint` | `crates/mycelium-lint/` | cli-common, doc, l1, lsp | `mycelium-lint-myc` | unit |
| `mycelium-doc` | `crates/mycelium-doc/` | core, l1, proj, workstack | `mycelium-doc-myc` | unit |
| `mycelium-lsp` | `crates/mycelium-lsp/` | cert, core, interp, l1, proj, select, workstack | `mycelium-lsp-myc` | unit |
| `mycelium-cli` | `crates/mycelium-cli/` | cli-common, interp, l1, proj, spore | `mycelium-cli-myc` | bin smoke |
| `mycelium-sec` | `crates/mycelium-sec/` | cli-common | `mycelium-sec-myc` | unit |
| `mycelium-transpile` | `crates/mycelium-transpile/` | l1, workstack | `mycelium-transpile-myc` | unit + gap-profiler smoke |
| `mycelium-bench` | `crates/mycelium-bench/` | cert, core, interp, l1, mlir, std-runtime | `mycelium-bench-myc` | optional/nightly |

**Cluster alternative (FLAG-T1):** single `mycelium-toolchain` repo holding check/fmt/lint/lsp/cli/cli-common — fewer repos, heavier CI. Default remains **one tool per repo** for leaf autonomy.

---

### 3.4 Compiler `.myc` surface (`lib/compiler`)

Not workspace Rust crates today; self-host front-end nodules.

| Repo | Sources | Outbound | `*-myc` note | CI |
|---|---|---|---|---|
| `mycelium-compiler-myc` | `lib/compiler/**` | eventual l1/runtime pins | **Primary is already Mycelium** — no separate Rust twin required; optional `mycelium-compiler` empty reserved | `myc check` when toolchain available; else document skip |

---

### 3.5 Docs / research / index (optional separate)

| Repo | Sources | When | Notes |
|---|---|---|---|
| `mycelium-docs` (optional) | `docs/` (minus generated churn if desired) | After D kernel/std stable | **FLAG-D1:** keep docs in monorepo umbrella until R; split only if public multi-repo needs standalone docs site |
| `mycelium-research` (optional) | `research/` | low priority | Evidence corpus; can stay monorepo |
| *(stay monorepo)* | `docs/api-index/`, `docs/tero-index/`, `tools/github/`, `scripts/`, `justfile` | — | Orchestrator surfaces until R redesign |

**Already extracted / out of map as first-party language components:**

| External | Provenance |
|---|---|
| `tzervas/tero-rs` | was `mycelium-tero`; pin in `tools/tero-rs/PROVENANCE.md` |
| `packages/tero-mcp-lite/` | portable package; may remain monorepo or own package repo |

---

### 3.6 Workspace leftovers

| Path | Disposition |
|---|---|
| `xtask/` | Keep monorepo or fold into `mycelium-build` / umbrella scripts |
| `gen/myc-drafts/` | Monorepo or transpile tooling repo until graduation |
| `proofs/` | Monorepo or `mycelium-proofs` optional |
| `experiments/` | Stay monorepo / desktop-held |
| `.github/workflows/` | Template → copy per component; monorepo keeps archive/CI advisory |

---

## 4. Extract method (high level — D2)

Prefer **history-preserving** where practical; never rewrite monorepo trunks.

### 4.1 Recommended pipeline

```text
1. Phase A verified: archive/main-pre-component-transpile-YYYY-MM-DD exists + SHA logged
2. For each component in extract order (§4.3):
   a. git fetch origin
   b. git filter-repo (or git subtree split) on monorepo clone:
        --path crates/<crate>/ [--path lib/std/<nodule>.myc ...]
        --path-rename crates/<crate>:.   (layout as repo root or keep crates/ prefix — pick one per repo, document)
   c. Push to new empty tzervas/<repo> (main default branch)
   d. Add LICENSE (MIT), README (provenance: monorepo SHA + filter paths), minimal CI
   e. Record component tip SHA in umbrella pin file (§5)
3. Monorepo: do NOT delete paths; add ARCHIVE.md / program pointer only
4. Optional later: monorepo path deps → git deps for dogfood of multi-repo builds (post-D)
```

### 4.2 Tool choice

| Method | When | Notes |
|---|---|---|
| **`git filter-repo`** (preferred) | Multi-path component, clean history subset | Supports multiple `--path`; rewrite once per component from archive tag |
| **`git subtree split`** | Single directory crate | Simpler; one path only |
| **Clean slice (no history)** | Only if history rewrite cost >> value | Record “clean start at archive SHA” never-silently; still MIT + provenance |

**Do not** use force-push on monorepo. New component repos may receive force only while **empty/private pre-publish** if recreate needed — after first consumer pin, treat like protected history.

### 4.3 Extract order (acyclic-ish)

```text
mycelium-core
  → mycelium-value (resolve FLAG-K2)
  → mycelium-runtime
  → mycelium-codegen
  → mycelium-l1
  → mycelium-cli-common, mycelium-proj, mycelium-spore, mycelium-build
  → mycelium-std-core → other std-* (R0 → R1 → R2 → host → conformance)
  → tooling (check, fmt, lsp, cli, transpile, …)
  → compiler-myc / docs (optional)
```

### 4.4 Seed contents (every new repo)

| File | Required |
|---|---|
| `LICENSE` | MIT text |
| `README.md` | Purpose, monorepo origin SHA, extract paths, paired `*-myc` name, honesty tags |
| `.github/workflows/ci.yml` | `workflow_dispatch` + push; `cargo fmt` / `clippy -D warnings` / `test` for members; skip-graceful if no Rust yet (`*-myc` empty) |
| `rust-toolchain.toml` | Pin **1.96.1** when Rust present (ADR-041) |
| `PROVENANCE.md` | Optional; follow `tools/tero-rs/PROVENANCE.md` style when monorepo consumes published artifact |

---

## 5. Umbrella re-export / pin strategy

### 5.1 Options

| Option | Umbrella repo | Role of current `tzervas/mycelium` | Pin mechanism |
|---|---|---|---|
| **U1 (recommended draft)** | New **`mycelium-lang`** | Becomes **archive + historical monorepo** (Phase A tip immortal); optional thin README pointing to `mycelium-lang` | `mycelium-lang` holds `components.lock` (or `Cargo.toml`/`mycelium-proj.toml` git deps) pinning each component + `*-myc` **git SHA** (and later spore content-hash) |
| **U2** | Keep **`mycelium`** as umbrella | Same repo gains pin file; tree gradually thins after T/R | Same pin file inside monorepo; risk: confuses archive vs presentation |
| **U3** | `mycelium-std` + `mycelium-toolchain` managerial only | No single language umbrella | Consumers compose manually — weaker “self-hosted presentation” |

**FLAG-U1:** choose U1 vs U2 before R1. This map **proposes U1** so archive (A) and presentation (R) stay distinct.

### 5.2 Pin file sketch (`components.lock` — Declared schema)

```toml
# mycelium-lang / monorepo — Declared pin schema (not implemented)
[component.mycelium-core]
git = "https://github.com/tzervas/mycelium-core"
rev = "<sha>"
paired_myc = "https://github.com/tzervas/mycelium-core-myc"
paired_myc_rev = "<sha-or-empty>"

[component.mycelium-std-io]
git = "https://github.com/tzervas/mycelium-std-io"
rev = "<sha>"
paired_myc = "https://github.com/tzervas/mycelium-std-io-myc"
paired_myc_rev = "<sha-or-empty>"
```

R4 CI: umbrella job checks out pins and runs `myc check` / smoke on composed surface; fail if pin missing.

### 5.3 Spore / GHCR (later, DN-88)

When spore publish is live: pins may migrate from git SHA → **content-addressed spore id** (ADR-013/037). Git SHA pins are the Phase R **bridge**.

---

## 6. Minimal CI template (per component)

```yaml
# Sketch only — adapt per repo
name: ci
on:
  push:
  pull_request:
  workflow_dispatch:
jobs:
  rust:
    if: hashFiles('Cargo.toml') != ''
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: 1.96.1
          components: rustfmt, clippy
      - run: cargo fmt --check
      - run: cargo clippy --all-targets -- -D warnings
      - run: cargo test
  myc:
    if: hashFiles('**/*.myc') != ''
    runs-on: ubuntu-latest
    continue-on-error: true   # until toolchain pin solid
    steps:
      - uses: actions/checkout@v4
      - run: echo "myc check — wire when umbrella/toolchain pin available"
```

Monorepo remote CI stays **manual-dispatch advisory** (existing policy). Component repos may use push PR CI — **FLAG-C1** cost control (private vs public, required checks).

---

## 7. Repo creation checklist (D3 — autonomous when gated)

For each §3 row (Rust then optional empty `*-myc`):

1. `gh repo create tzervas/<name> --private --license mit` (or org policy)
2. Push filter-repo result **or** seed commit (LICENSE + README + CI)
3. Protect default branch lightly (PR preferred; matches monorepo discipline when public)
4. Record URL + tip SHA in program status log + umbrella pin draft
5. Do **not** remove monorepo paths

**Batch size:** create in extract order (§4.3); stop if auth/org quota fails — never-silent FLAG.

---

## 8. Count summary (recommended map)

| Group | Rust component repos | `*-myc` siblings | Notes |
|---|---:|---:|---|
| Kernel (multi-crate) | 5 | 5 | core, value, runtime, l1, codegen |
| Std phyla | 27 | 27 | includes conformance + sys-host |
| Tooling | 13 | 13 | incl. bench |
| Compiler surface | 0–1 | 1 | `mycelium-compiler-myc` |
| Umbrella | 1 (`mycelium-lang`) | — | pins only |
| Optional docs/research | 0–2 | — | FLAG-D1 |
| **Approx total new** | **~46–50** | **~46** | plus empty `*-myc` seeds |

If FLAG-K1 → 1:1 kernel: +~13 repos. If FLAG-S1 → ring std: std Rust repos drop to ~4.

---

## 9. Open questions (FLAG → L0 / maintainer)

| ID | Question | Options | Draft default |
|---|---|---|---|
| **FLAG-K1** | Kernel: multi-crate seam repos vs 1:1 crate repos? | multi / 1:1 | **multi** (§3.1) |
| **FLAG-K2** | Where does `mycelium-select` live (value vs runtime)? | value / runtime | **value** pending D2 DAG check |
| **FLAG-S1** | Std: one-repo-per-phylum vs ring-level vs single? | A / B / C | **A** (DN-88 exemplar) |
| **FLAG-T1** | Tooling: one-repo-per-tool vs `mycelium-toolchain` cluster? | one / cluster | **one** |
| **FLAG-U1** | Umbrella: new `mycelium-lang` vs keep `mycelium` name? | U1 / U2 / U3 | **U1 `mycelium-lang`** |
| **FLAG-D1** | Split `docs/` / `research/` now? | yes / monorepo-until-R | **monorepo until R** |
| **FLAG-C1** | Component CI on every push vs manual like monorepo? | push / manual | **push for tiny crates; manual for mlir/VSA** |
| **FLAG-N1** | Exact `*-myc` spelling: `mycelium-std-io-myc` vs `std-io-myc`? | full prefix / short | **full `mycelium-…-myc`** (T3 clarity) |
| **FLAG-H1** | History: filter-repo full path history vs clean slice? | history / clean | **filter-repo history** |
| **FLAG-X1** | Public now vs private until Phase I usability (ADR-038)? | private / public | **private** until maintainer flip |

None of the above block **authoring** this map; they block **unsupervised mass `gh repo create`** only where marked.

---

## 10. Cross-refs

| Doc | Use |
|---|---|
| PROGRAM-SELFHOST-DECOMPOSE | Phase order G→A→D→T→R; autonomy notes |
| DN-88 | Topology, dogfood gate, managerial re-export, mapping **criteria** |
| DN-27 | Post-1.0.0 decomposition capture (Draft; refined by DN-88) |
| DN-34 / M-1006 | Transpile as gap profiler → Phase T honesty |
| ADR-022 §7/§10 | MIT-only; long-term component + re-export vision |
| ADR-036 / ADR-038 | Dogfood + public/usability gates (do not conflate with DN-88 §3) |
| ADR-042 | Mycelium-first expansion; zero-foreign horizon at DN-88 gate |
| ADR-045 | Early gap-closure window (Phase G context) |
| RFC-0016 | Std ring/module taxonomy |
| RFC-0031 | Self-hosted std composition; spore packaging D7 |
| `tools/tero-rs/PROVENANCE.md` | Extract-and-pin precedent |

---

## 11. Changelog (append-only)

| When | Note |
|---|---|
| 2026-07-17 | **D1 draft opened** — component map from workspace members + `lib/std`/`lib/compiler`; honesty vs DN-88 §3 stated; extract + umbrella + DoD recorded. No repos created. |
