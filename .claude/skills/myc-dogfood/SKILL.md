---
name: myc-dogfood
description: >-
  Run the real native Mycelium toolchain over the self-hosted `lib/compiler/*.myc` frontend as a
  second, independent witness alongside the Rust `cargo test` differential — dual old-Rust + new-`myc`
  parity as the L1 frontend is ported to Mycelium (M-989 / boot10 / DN-26). Light checks only:
  `myc check` is the core parity check; `mycfmt`/`myc-lint` run advisory; heavy VSA/GPU-bound work is
  held out of the gate and belongs on a local/teleport machine. Non-gating by default (the port is
  in-progress until M-741) — a failure prints, never silently, but does not turn `just check` red.
when_to_use: >-
  Run after touching any `lib/compiler/*.myc` (a port stage, a new self-hosted nodule), and whenever
  you want to confirm the native toolchain still accepts the self-hosted frontend — not just the Rust
  differential harnesses. It also runs automatically inside `just check` (non-gating). Use `--strict`
  to enforce the core `myc check` deliberately (e.g. as the port stabilizes toward M-741).
allowed-tools: Bash(scripts/checks/myc-dogfood.sh:*), Bash(just myc-dogfood:*), Bash(cargo run:*), Bash(git ls-files:*)
---

# myc-dogfood

As the Rust L1 frontend is ported to Mycelium (`lib/compiler/*.myc`, per **DN-26** / **M-740**), each
`.myc` file is checked by the Rust **differential harnesses** (`crates/mycelium-l1/tests/compiler_stage*.rs`).
This gate adds the **second, independent witness — the actual `myc` toolchain** — so the port is vetted
by **both the old (Rust) and the new (native `myc`) tooling for parity**, not only from inside `cargo test`.
It is the operational form of **M-989**.

`lib/compiler/` is **not** a `mycelium-proj.toml` project root, so the `just myc-check` / `just myc-fmt`
project gates skip it; this gate walks the tracked `lib/compiler/*.myc` directly.

## Use it

```sh
just myc-dogfood            # non-gating: myc check (core) + mycfmt/myc-lint (advisory) over lib/compiler
just myc-dogfood --strict   # make a core `myc check` failure exit non-zero (deliberate enforcement)
bash scripts/checks/myc-dogfood.sh          # the same, invoked directly (runs inside `just check`)
MYC_DOGFOOD_STRICT=1 bash scripts/checks/myc-dogfood.sh   # strict via env
```

- **What it runs:** `myc check` (oracle mode, one file per invocation) is the **core** parity check
  (parse + L1 type-check); `mycfmt --check` (canonical form) and `myc-lint` (error-severity only) run
  **advisory** — they report but never fail the gate (`mycfmt` refuses a couple files on the **M-690**
  nested-match-arm-comment limitation, which must not gate).
- **What it deliberately skips:** `myc-sec` — it has no per-file interface (dir-only `--project`) and
  `lib/compiler` has no `wild { … }` blocks, so its audit is a no-op; repo-root `myc-sec` covers the tree.
- **Light by design — heavy work routes to local:** none of these tools has a VSA/GPU/expensive mode;
  the *heavy* dogfood work (running the self-hosted evaluator over whole programs, `cargo-mutants`,
  fuzzing) is held **out** of this gate and belongs on a GPU-equipped local machine (session teleport),
  mirroring how `just check-full` / `just mutants` / `just fuzz` sit outside `just check`.
- **Non-gating (G2, never silent):** default is advisory — a `myc check` failure prints a `FAIL` line
  but the script `exit 0`s, so it can't turn `just check` red on the in-progress self-hosted files (the
  Rust differential gates own correctness). `--strict` / `MYC_DOGFOOD_STRICT=1` makes a core failure
  exit non-zero (reason sub-code 3 = check error). Graceful `skip` when `cargo` is absent.
- **Graduation:** flip the default to gating once the port is canonical (**M-741**).

## Where it fits

- Runs automatically inside **`just check`** (in `scripts/checks/all.sh`, component id 30) as a
  non-gating witness — so every gate run reports native-toolchain parity over the self-hosted frontend.
- Run it **after any `lib/compiler/*.myc` change**, alongside the change-scoped `cargo test -p
  mycelium-l1 --test compiler_stage<N>` differential — the two together are the dual-tooling parity.
- Pairs with the **DN-26** bootstrap plan: this per-file native check is the interpreted-first,
  light-weight precursor to the **Stage-6 / M-742** `just bootstrap` gate; it graduates to project-level
  (cross-nodule) checking once **M-982** (cross-nodule execution) lifts.
