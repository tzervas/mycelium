# `gen/myc-drafts/` — transpiler-drafted `.myc` starting points (M-1002/M-1003, kickoff `trx2` E-B)

> **The honesty contract.** Everything under this directory is **`Declared`** draft material — a
> heuristic `syn` → surface-text emission from `crates/mycelium-transpile`, unvalidated beyond
> `myc check`'s per-file parse/type-check pass (`Empirical` — measured by the real toolchain, never
> `Proven`; see `crates/mycelium-transpile/src/vet.rs`). These are **starting points, not ports**:
> - **Never imported by anything under `lib/`.** No `.myc` here is a nodule any program `use`s.
> - **Never gated by `/myc-dogfood`** or any other check that treats `lib/` as ground truth — a
>   `Declared`, mostly-gapped draft is expected here, not a defect to fix before merge.
> - **Graduate into `lib/` only when hand-vetted** during the M-993 semcore port (or the analogous
>   stdlib port work) — a human (or a dedicated port task) reviews, fixes the gaps, and re-lands the
>   result as a real `lib/` nodule. Nothing here is copied into `lib/` wholesale.
> - A **mostly-gapped draft is a successful, honest output**, not a failure to chase away with a
>   weaker check (DN-34 §8.8's go/no-go: the transpiler is a **gap-profiling instrument**, not a bulk
>   porter — real `checked_fraction` on this port surface runs ~0–8%). Do not read a low
>   `checked_fraction` here as "the tool is broken" — it is the tool doing its honest job.

## What is here

Two wave-1 target classes (E33-1 launch-scope record, `.claude/kickoffs/trx2.md`):

- **`semcore/`** — one subdirectory per file of the `mycelium-l1` semantic-core SCC (`checkty`,
  `elab`, `eval`, `mono`, `fuse` — the M-993 port target, ~15.4k Rust LOC). Per
  `lib/compiler/README.md`, these nine Rust modules (this wave covers five; `decision`/
  `usefulness`/`grade`/`affine` are out of wave-1 scope) are destined for **one Mycelium nodule**
  (`compiler.semcore`) because they form a single strongly-connected component needing nodule-wide
  mutual recursion — a fact recorded per-target in `manifest.json`/`MANIFEST.md` even though this
  transpiler emits one `.myc` file per Rust file (it does not itself merge SCC members into one
  nodule; that merge is part of the M-993 hand-port). Where a semcore draft's constructors would
  collide with a Mycelium reserved word or across the merged nodule's flat namespace, the manifest
  flags it against the `lib/compiler/README.md` **FLAG-ast-5**/**FLAG-parse-2** per-type
  constructor-prefixing convention the hand-port must apply (this transpiler does not auto-rename —
  a collision is gapped, never silently renamed; DN-34 §8.8 / `crates/mycelium-transpile/src/gap.rs`
  `Category::ReservedWord`).
- **`stdlib/`** — one subdirectory per unported stdlib crate (the 12 with no existing `.myc` twin):
  `std-conformance`, `std-content`, `std-dense`, `std-fs`, `std-io`, `std-numerics`, `std-rand`,
  `std-runtime`, `std-sys`, `std-sys-host`, `std-time`, `std-vsa`.

Each target subdirectory holds the `mycelium-transpile --vet` output for that target: per-Rust-file
`<stem>.myc` + `<stem>.gap.json`, and (batch/directory targets only) `summary.json` +
`union.gap.json`, plus every target's `vet.json` (the `myc check` verdict per emitted `.myc`).

## The manifest

`manifest.json` (machine-readable) + `MANIFEST.md` (human-readable) map every draft to its Rust
source (path + content hash, for change detection), transpile stats (non-test items, emitted items,
gap counts by category), and vet status (`checked_fraction`/`expressible_fraction`, per-class
`myc-check` exit counts). **Deterministic by construction:** stable target ordering (an explicit
list, not a directory glob), no wall-clock timestamps anywhere in the diffed artifacts — provenance
is instead the **source git commit SHA** (`generated_from_commit`) the drafts were regenerated
against, which only changes when the drafted Rust sources (or the transpiler) actually change.

## Regenerating

```sh
bash gen/myc-drafts/regenerate.sh
```

Builds `myc-check` + `mycelium-transpile` once (avoids nested-`cargo` build-lock contention across
targets — the same discipline as `scripts/checks/transpile-vet.sh`), runs `--vet` over the explicit
wave-1 target list into each target's subdirectory, then assembles `MANIFEST.md`/`manifest.json`
from the already-deterministic per-target JSON artifacts (`manifest_gen.py` — pure aggregation, no
re-derivation of transpile/gap/vet logic). Skips gracefully with a clear message if `cargo` or
`python3` is unavailable. Non-gating: this tree is not part of `just check`.

## Provenance

Wave-1 breadth is the maintainer-confirmed launch-scope record (E33-1 body, `.claude/kickoffs/
trx2.md`): the port surface only. The whole-corpus rip-through ladder beyond this surface is a
separate, later-minted task (M-1006) — not this tree's job to expand.
