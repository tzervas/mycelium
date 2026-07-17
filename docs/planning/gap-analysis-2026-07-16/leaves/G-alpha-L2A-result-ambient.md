# L2-A brief — Result ambient co-emit (G-α Rank 1)

## Identity

- **Leaf:** G-alpha-L2A-result-ambient
- **Branch:** `claude/leaf/G-alpha-result-ambient`
- **Model assigned:** `grok-composer-2.5-fast` (record actual if unavailable)
- **Base:** `origin/dev` @ `67090f4a` (or later tip)
- **PR base:** `dev` — **do not merge**

## Problem (Empirical)

On tip, `mycelium-std-io` first poison:

```
check-error: check error in `read_all`: unknown type `Result`
```

Emitted surface already has:

```text
pub fn read_all(src: Source) => Result[Vec[Binary{8}], IoError] = …
```

Checker seeds Bool/Unit unconditionally and Vec conditionally (`CONDITIONAL_PRELUDE_TYPE_NAMES`); **Result is not ambient**. Live-oracle tests already inject:

```text
type Result[A, E] = Ok(A) | Err(E);
```

(same shape as `lib/std/result.myc`). Source/Sink types are already emitted (L2-C #1675).

## Goal

Co-emit ambient `Result` (and `Option` if the emission mentions it) into nodules that use those type heads, so `io.myc` becomes myc-check-clean without fabricating combinators.

## Ownership (write)

- Prefer: `crates/mycelium-transpile/src/emit.rs` (file assembly / ambient inject after `nodule …;`)
- and/or the path that finalizes per-file `.myc` text in `transpile.rs` / `batch.rs`
- Tests: `crates/mycelium-transpile/src/tests/` (move any touched inline tests out)
- **Do not** edit `CHANGELOG.md`, `issues.yaml`, `Doc-Index`, `api-index` — FLAG up
- Prefer **not** changing `mycelium-l1` (checker prelude) unless transpile-only path is blocked — if you need checker seed, FLAG and stop rather than expand scope silently

## Acceptance

1. Synthetic or fixture emission containing `Result[…]` includes EXPLAIN + `type Result[A, E] = Ok(A) | Err(E);` once per file.
2. `cargo fmt` + `clippy -D warnings` + `cargo test -p mycelium-transpile` green.
3. Before/after on `crates/mycelium-std-io/src` if practical: `io.myc` should leave `unknown type Result`; report metrics.
4. No fabricated prims; no silent upgrade of guarantee tags.
5. Open PR → `dev`, **do not merge**. Report **PR# + SHA + FLAGs**.

## Out of scope

- Import non-type / `use …read_all` (L2-B)
- M-875 macro expand
- Shared-file close-out
- Claiming one-shot readiness
