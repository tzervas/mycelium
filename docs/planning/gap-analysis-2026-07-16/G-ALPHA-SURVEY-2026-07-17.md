# G-α survey — transpile first-poison residual (Empirical)

| Field | Value |
|---|---|
| **Status** | Active (L1 epic survey) |
| **L1 model assigned** | `grok-composer-2.5-fast` |
| **L1 model actual** | session-assigned composer path (record if runtime differs) |
| **Base tip** | `origin/dev` @ `67090f4aba69d15766681e55bd26c0fed6a996e0` |
| **Honesty** | Fractions **`Empirical`**; residual ranking **`Declared`** unless tied to a cited diagnostic |
| **Oracle** | real `myc-check` via `MYC_CHECK_CMD` / `mycelium-transpile --vet` |
| **Scratch** | `/tmp/g-alpha-survey.GZyccI` (ephemeral; measure leaf re-runs into `experiments/results/`) |

## Commands (reproducible)

```bash
git fetch origin dev   # tip >= 67090f4a
cargo build -q -p mycelium-check --bin myc-check
cargo build -q -p mycelium-transpile --bin mycelium-transpile
export MYC_CHECK_CMD="$PWD/target/debug/myc-check"
T="$PWD/target/debug/mycelium-transpile"
# default-5 + expansion — same set as M1006-remeasure-post-C3C4
$T --vet crates/mycelium-l1/src/eval.rs  OUT/eval
$T --vet crates/mycelium-l1/src/fuse.rs   OUT/fuse
$T --vet crates/mycelium-std-time/src     OUT/time
$T --vet crates/mycelium-std-rand/src     OUT/rand
$T --vet crates/mycelium-std-cmp/src      OUT/cmp
$T --vet crates/mycelium-std-fs/src       OUT/fs
$T --vet crates/mycelium-std-io/src       OUT/io
```

## Per-target results (`Empirical`, tip `67090f4a`)

### Default M-1001 five-target set

| Target | non-test | checked | `checked_fraction` | emitted | `expressible_fraction` | File class | First oracle poison |
|--------|---------:|--------:|-------------------:|--------:|-----------------------:|------------|---------------------|
| `eval.rs` | 42 | 9 | **21.4%** | 9 | **21.4%** | **Clean** | *(none)* |
| `fuse.rs` | 12 | 0 | **0.0%** | 0 | **0.0%** | Clean | *(no emission — gap profile)* |
| `std-time` | 37 | 17 | **45.9%** | 17 | **45.9%** | **Clean** | *(none)* |
| `std-rand` | 34 | 6 | **17.6%** | 6 | **17.6%** | **Clean** | *(none)* |
| `std-cmp` | 111 | 14 | **12.6%** | 14 | **12.6%** | **Clean** | *(none)* |

**Union default-5:** **46 / 236** → **`checked_fraction` 19.5%** (flat vs post-C3C4 / post-A5 floor).

### Expansion (std-fs / std-io)

| Target | non-test | checked | `checked_fraction` | emitted | `expressible_fraction` | File class | First / representative oracle poisons |
|--------|---------:|--------:|-------------------:|--------:|-----------------------:|------------|----------------------------------------|
| `std-fs` | 47 | 28 | **59.6%** | 28 | **59.6%** | **Clean×7** | *(none — Import residual closed on tip)* |
| `std-io` | 59 | 14 | **23.7%** | 25 | **42.4%** | CheckError×2, Clean×3 | see first-poison table |

**Union all-7:** **88 / 342** checked → **25.7%**; **100 / 342** emitted → **29.2%**.

Δ vs post-C3C4 (`167f0bf2`): fs **38.3%→59.6%** (Import phase-2 / Source-Sink landed); io **20.3%→23.7%** checked, expressible **33.9%→42.4%** (Source/Sink type emit #1675 + Import L2-B).

#### First-poison list (every CheckError file — oracle, single-file)

| Crate | File | First diagnostic (truncated) |
|-------|------|------------------------------|
| std-io | `io.rs` → `io.myc` | `check error in \`read_all\`: unknown type \`Result\`` |
| std-io | `lib.rs` → `lib.myc` | `` `use std.io.io.read_all`: no such name `std.io.io.read_all` in the phylum `` |

**Cleared vs post-C3C4:** std-fs `lib`/`substrate` Import `std.fs.error.*` (now Clean×7, checked=expressible=59.6%). Source/Sink **no longer** first phylum poison (types emitted; L2-C #1675).

### Phylum dual-report (`Empirical`)

| Target | `checked_fraction_phylum` | oracle `checked_fraction` | Δ_basis | phylum ok |
|--------|--------------------------:|--------------------------:|--------:|-----------|
| std-time / rand / cmp | = oracle | = oracle | +0.0pp | true |
| **std-fs** | **59.6%** | **59.6%** | **+0.0pp** | **true** (all 7 Clean) |
| **std-io** | **23.7%** | **23.7%** | **+0.0pp** | **false** |

**std-io phylum residual:** both `std.io.io` and `std.io` fail with **`unknown type Result`** on `read_all` (phylum `ok: false`). Import form is **not** the first phylum diagnostic once Source/Sink closed — Result is.

**std-io single-file `lib` residual:** still Import **non-type** (`use std.io.io.read_all`) under phylum-of-one. Likely secondary after Result unblocks `io` export of `read_all` under multi-nodule phylum; single-file may remain.

## Ranked residual classes (`Declared` ranking on Empirical diagnostics)

| Rank | Class | Evidence | Closable without design gate? | Suggested L2 |
|-----:|-------|----------|-------------------------------|--------------|
| **1** | **`Result` ambient / type surface** | `unknown type Result` on `read_all` — oracle + phylum. Emission already maps `Result[Vec[Binary{8}], IoError]`; checker has no conditional Result prelude (only Bool/Unit unconditional, Vec conditional). Tests already inject `type Result[A,E]=Ok(A)\|Err(E);` for live oracle. | **Yes** | **L2-A Result** |
| **2** | **Import non-type (fn/value use)** | `use std.io.io.read_all` refused single-file; L2-B co-include only types (`type_defs`), residual FLAG in `symtab.rs`/`dispatch_use`. | **Yes** (fn co-include or honest gap) | **L2-B Import non-type** |
| 3 | MacroInvocation / MacroDef | cmp heat 57+5; non-emission wall | **No** — M-875 needs-design until Accepted | design-gated |
| 4 | DeriveSatisfied / DeriveAttr heat | emission heat, not file-poison on tip | later | prefer after Rank 1–2 |
| 5 | NamedFieldDrop / Impl / MultiStmtBody | emission heat | later | G-β |
| 6 | fuse zero-emission | 0/12 profile | profile-only | not G-α |

### Top closable residual classes for G-α L2 leaves

1. **Result ambient co-emit** (primary file-poison; closes io `io.myc` + likely phylum `lib` Result path).
2. **Import non-type** (fn use co-include or never-silent residual after Result).
3. **Measure** (Empirical table post-merge / post-open-PR).

**Serial note:** both implement leaves may touch `emit.rs` / `transpile.rs`. Prefer **serial** if both need `emit.rs`; **parallel** if L2-A owns emit ambient + tests and L2-B owns only `transpile.rs`/`symtab.rs` fn co-include without emit assembly.

## Decomposition table (L2 fan-out)

| Leaf | Branch | Owns (write) | Does not own | Base | Depends |
|------|--------|--------------|--------------|------|---------|
| **L2-A Result** | `claude/leaf/G-alpha-result-ambient` | `crates/mycelium-transpile/src/emit.rs` (and/or file-assembly path), `src/tests/*` for Result ambient; optional `map.rs` only if needed | shared CHANGELOG/issues/Doc-Index; `mycelium-l1` unless FLAG | `origin/dev` | none |
| **L2-B Import non-type** | `claude/leaf/G-alpha-import-non-type` | `transpile.rs` `dispatch_use` non-type path, `symtab.rs` fn surface extract, tests | emit ambient Result (L2-A); shared files | `origin/dev` (+ L2-A if serial) | prefer after L2-A if phylum re-export depends on Clean `io` |
| **L2-M Measure** | `claude/leaf/G-alpha-measure` | `docs/planning/gap-analysis-2026-07-16/M1006-remeasure-post-G-alpha-*.md`, `experiments/results/m1006-remeasure-post-g-alpha/` | product code | tip after L2 PRs | after L2-A/B open or merge |

## L2 inject (common)

```
You are L2. model: grok-composer-2.5-fast (if runtime cannot offer, record actual — VR-5/G2).
isolation: worktree. Branch from origin/dev (or pushed orch tip).
Own only brief paths. cargo fmt / clippy -D warnings / test -p mycelium-transpile green.
PR base=dev, do NOT merge. Report PR# SHA FLAGs before/after metrics if possible.
No fabrication of prims (VR-5). EXPLAIN every co-include. Never silent gap.
FLAG shared files up (CHANGELOG, issues.yaml, Doc-Index, api-index).
```

## Next residual rank (post G-α expected)

After Result + Import non-type: Macro (M-875 design), emission-heat classes, fuller M-1006 ladder (G-β/G-γ). **No one-shot claim** on this evidence.
