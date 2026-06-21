# stdlib.md — Mycelium Standard Library (`mycelium-std-*`)

> **Orientation aid (Declared/Empirical) — not normative.** Source + the normative corpus are
> ground truth. Cite: `docs/spec/stdlib/README.md`, RFC-0016, DN-07, DN-16.

---

## What it is

The Rust-first standard library of Mycelium: 25 `mycelium-std-*` crates organised into one
`std` phylum. **Not a kernel change** (KC-3): every crate lives *above* `mycelium-core` and the
capability crates; it adds no new trusted code and confines all `wild`/FFI to the single
`mycelium-std-sys` phylum (LR-9). The load-bearing deliverable of Phase 5 (tasks M-510…M-534,
M-541). The honesty contract (C1–C6 from RFC-0016 §4.1) applies to **every exported op**
without exception.

---

## Where it lives

| Path | Role |
|---|---|
| `crates/mycelium-std-*/` | 25 crates (one per module) |
| `docs/spec/stdlib/README.md` | Living index + Ring/Tier table + cross-module reconciliation |
| `docs/spec/stdlib/<module>.md` | Per-module design spec + guarantee matrix |
| `docs/spec/stdlib/_TEMPLATE.md` | Single conformance template every spec follows |
| `docs/rfcs/RFC-0016-Core-Library-and-Standard-Library.md` | Keystone: the §4.1 per-op contract (C1–C6), §4.2 ring layering, §4.3/4.4 taxonomy |
| `docs/notes/DN-07-Stdlib-Ratification.md` | 2026-06-20 ratification of 23 Rust-first specs |
| `docs/notes/DN-16-Runtime-Sys-Ratification.md` | 2026-06-21 ratification of `runtime` + `sys` (completing 25/25) |

---

## Ring / Tier organisation (RFC-0016 §4.2–§4.4)

| Ring | Tier | Role | Crates |
|---|---|---|---|
| **Ring 0** | — | Kernel-adjacent re-exports; no new trusted code | `std-core` (M-515) |
| **Ring 1** | **Tier A** (differentiators) | Ergonomic surfaces over capability crates; certificate/EXPLAIN consumers | `std-swap`, `std-numerics`, `std-vsa`, `std-dense`, `std-select`, `std-content`, `std-diag`, `std-recover`, `std-ternary`, `std-spore`, `std-runtime` |
| **Ring 2** | **Tier B** (common/expected) | General library written over Ring 0/1 | `std-collections`, `std-text`, `std-math`, `std-iter`, `std-error`, `std-io`, `std-fs`, `std-time`, `std-rand`, `std-cmp`, `std-fmt`, `std-testing` |
| **`std-sys`** | (floor) | Single audited `wild`/FFI/OS-syscall phylum (LR-9) | `std-sys` (M-541) |

**25 crates total** (23 Rust-first + `runtime` + `sys`). One more spec exists:
`self-hosting-readiness.md` (M-502) — a *gate doc*, not a crate; stays `Draft (needs-design)`.

---

## Crate map by module

| Module / Crate | Task | Ring/Tier | Key capability |
|---|---|---|---|
| `std-core` (prelude) | M-515 | Ring 0 | Re-exports value model, `GuaranteeStrength`, `Bound`/`BoundBasis` from `mycelium-core`; thin query surface (`repr_of`, `meta_of`, `guarantee_of`) |
| `std-swap` | M-516 | R1/A | `swap()` → `(Value, SwapCertificate)` or `SwapError`; `check_swap`; `explain()` → `ExplainRecord`; delegates to `mycelium-cert` |
| `std-numerics` | M-512 | R1/A | `Approx<T>` thin view (value + `Meta`-attached `{Bound, strength}`); ε/δ meet-composition; `refuse_without_a_rule` posture (M-204); carrier for `math`/`dense` deferreds |
| `std-vsa` | M-513 | R1/A | VSA/HDC bind/bundle/unbind/cleanup; per-(model,op) guarantee matrix mirroring `mycelium-vsa`; `ResonatorTrace` (EXPLAIN) |
| `std-dense` | M-518 | R1/A | Dense embedding ops; ε through `std-numerics` |
| `std-select` | M-519 | R1/A | RFC-0005 selection policies + EXPLAIN; delegates to `mycelium-select` |
| `std-content` | M-523 | R1/A | Content-addressed identity hashing (ADR-003); not hash-for-maps |
| `std-diag` | M-510 | R1/A | RFC-0013 structured error records + traces; `diag` is never presentation-gated |
| `std-recover` | M-520 | R1/A | `Outcome`/`RecoverOutcome`/`PolicyRef` surface (RFC-0014); owns recovery bridge |
| `std-ternary` | M-517 | R1/A | Balanced-ternary surface; FR-M2; RFC-0004 §5 |
| `std-spore` | M-522 | R1/A | Deployable content-addressed spore + reconstruction manifest (ADR-013) |
| `std-runtime` | M-521 | R1/A | `Colony`/`Scope`/`Task`/`Network` — v0 R1 surface (ADR-020); RFC-0008 reserved vocab not yet active |
| `std-sys` | M-541 | (floor) | Audited `wild` floor: `math`, `rand`, `fs`, `time` modules; `Declared` tag throughout |
| `std-collections` | M-511 | R2/B | Value-semantic; no silent reorder |
| `std-text` | M-524 | R2/B | `parse` → `Result`; lossy encoding explicit |
| `std-math` | M-525 | R2/B | Approx ops carry tag; transcendentals via `std-sys`; no invented ε |
| `std-iter` | M-526 | R2/B | `total`/terminating where kernel guarantees it |
| `std-error` | M-527 | R2/B | `Option`/`Result` combinators; `RecoverOutcome` bridge |
| `std-io` | M-514 | R2/B | One canonical JSON projection; `substrate` single-consumption (LR-8) |
| `std-fs` | M-528 | R2/B | Every path/permission failure explicit; wires through `std-sys` |
| `std-time` | M-529 | R2/B | Monotonic vs wall-clock as typed distinction; declared nondeterminism (RT3) |
| `std-rand` | M-531 | R2/B | Nondeterminism reified/named; platform entropy via `std-sys` |
| `std-cmp` | M-532 | R2/B | Lossy convert fallible; lossless `BF16→F32` widening lives here (not `std-swap`) |
| `std-fmt` | M-533 | R2/B | Dual human/machine projection (G11); `to_json`/`from_json` delegate to `std-io` |
| `std-testing` | M-534 | R2/B | A skipped check is reported; differential harness; reuses interp↔AOT agreement bar |

---

## Key types and operations

**Contract (RFC-0016 §4.1 C1–C6, non-negotiable per op):**

- **C1 Never-silent (G2):** every fallible op → `Option`/`Result`; no sentinel, no clamp.
- **C2 Honest per-op tag (VR-5):** tag on `Exact ⊐ Proven ⊐ Empirical ⊐ Declared` in `Meta`.
  `Proven` only with a checked theorem; downgrade to stay honest; never upgrade without a basis.
- **C3 No black boxes / EXPLAIN (SC-3/G11):** selecting/converting/approximating ops expose
  *why* via reified artifact (swap certificate, EXPLAIN record, diagnostic).
- **C4 Value-semantic (ADR-003/RFC-0001):** pure functions + content-addressed identity.
- **C5 Above the kernel (KC-3):** no new trusted code; `wild`/FFI confined to `std-sys`.
- **C6 Declared bounded effects (RFC-0014):** IO/time/randomness declared on signature.

**Guarantee matrix (RFC-0016 §4.5):** every module ships one. Rows = exported ops; columns =
`{guarantee_tag, fallibility (explicit error set), declared effects, EXPLAIN-able?}`. Encoded
as data (e.g. `GUARANTEE_MATRIX` in `mycelium-std-swap/src/lib.rs:17`, `mycelium-std-vsa/src/lib.rs:20`),
asserted in tests — **never prose-only** (VR-5).

**`std-core` re-exports (crates/mycelium-std-core/src/lib.rs:50+):**
`Bound`, `BoundBasis`, `BoundKind`, `NormKind` from `mycelium_core::bound`; value model types;
`GuaranteeStrength` lattice; `repr_of`, `meta_of`, `guarantee_of` (total, pure, `Exact`-tagged).

**`std-swap` (crates/mycelium-std-swap/src/lib.rs):**
`swap(value, target_repr, policy) -> Result<(Value, SwapCertificate), SwapError>` — never silent;
`check_swap` (M-210 checker); `explain(cert) -> ExplainRecord` (G11).

**`std-runtime` (crates/mycelium-std-runtime/src/lib.rs):**
v0 R1: `colony`, `network`, `task` modules; `Colony`/`Scope` structured concurrency;
`Network`/`Sender`/`Receiver`/`TrySend`/`TryRecv` channels; `SweepOrder`/`Deadlock`.
RFC-0008 §4.5 vocab (`hypha`/`fuse`/`xloc`/`cyst`/`graft`/`forage`/`backbone`/`mesh`/
`tier`/`reclaim`) is **reserved but not activated** in v0. `#![forbid(unsafe_code)]`.

**`std-sys` (crates/mycelium-std-sys/src/lib.rs):**
`math`, `rand`, `fs`, `time` modules; all functions `Declared`; `#![forbid(unsafe_code)]`
(uses Rust's own `f64`/`std::*` wrappers as `wild`-free v0 placeholders — wiring to actual
FFI is the M-541 follow-on).

---

## Status: 25/25 specs Accepted

| Wave | Date | What |
|---|---|---|
| First design wave | 2026-06-17 | 11 specs `Draft — landed` (Tier-A `core`/`swap`/`ternary`/`dense`/`select`/`content` + Tier-B `iter`/`math`/`error`/`cmp`/`fmt`) |
| Second design wave | 2026-06-17 | 13 more specs `Draft — landed` (completing all 23 Rust-first modules) |
| DN-07 ratification | 2026-06-20 | **23 Rust-first specs → `Accepted`** (per-spec guarantee matrices asserted in tests; VR-5 satisfied; `runtime` + `self-hosting-readiness` stayed `Draft`) |
| DN-16 ratification | 2026-06-21 | **`runtime` (v0 R1 surface; ADR-020)** + **`sys` (M-541)** → `Accepted`; 25/25 complete |
| `self-hosting-readiness` | — | Stays **`Draft (needs-design)`** — the M-502 gate doc (not a crate); verdict *not yet established* |

---

## Key invariants (honesty)

1. **No module upgrades a tag without a checked basis.** `std-sys` is entirely `Declared`;
   `std-numerics` only lifts to `Proven` where Higham's side-conditions are checked. Downgrade is
   always the safe direction (VR-5).
2. **Every failure is structured (I1).** A program may fail; every failure is an RFC-0013 record
   with trace + debug info (owned by `std-diag`), recovered or re-propagated, never swallowed.
3. **`wild`/FFI audit surface is bounded.** Only `std-sys`; all other crates are `#![forbid(unsafe_code)]`.
4. **Representation change is never silent.** `std-swap` always returns a `SwapCertificate` or
   an explicit `SwapError`; no sentinel, no clamp.
5. **`runtime` activates construct-by-construct at the Phase-7 gate** (ADR-020/RFC-0008 §8-Q4).
   Reserved vocabulary is not part of the v0 public API.
6. **Self-hosting (M-502) is NOT established.** The Mycelium-lang migration half waits on a
   concrete L3 authoring surface (KC-2-gated); `self-hosting-readiness.md` capability #3 flipped
   `not-yet → ready` after DN-09, but the verdict is still `Draft (needs-design)` (post-1.0).

---

## Read more

- `docs/spec/stdlib/README.md` — living index, Ring/Tier table, full cross-module reconciliation
- `docs/rfcs/RFC-0016-Core-Library-and-Standard-Library.md` — §4.1 the contract, §4.2 ring layering
- `docs/spec/stdlib/runtime.md` — `runtime` spec (Phase-7-gated constructs, ADR-020)
- `docs/spec/stdlib/sys.md` — `sys` spec (FFI floor, M-541)
- `docs/adr/ADR-020-Runtime-Colony-Phylum-Placement.md` — `runtime` phylum placement decision
- Per-module specs: `docs/spec/stdlib/<module>.md` (e.g. `swap.md`, `vsa.md`, `numerics.md`)

---

## Gotchas

- **`self-hosting-readiness.md` is not a crate** — it is the M-502 gate doc, and its status
  is `Draft (needs-design)`. Do not count it among the 25 Accepted crate specs.
- **`std-sys` carries `#![forbid(unsafe_code)]`** — v0 uses Rust's own wrappers as placeholders;
  the actual FFI wiring is a follow-on task. Do not assume the `wild` audit is complete.
- **The `BF16→F32` lossless widening lives in `std-cmp`**, not `std-swap`; `std-swap` owns
  only certified/lossy representation changes (resolved at ratification, DN-07).
- **`fmt.to_json`/`from_json` delegate to `std-io`** (one canonical JSON projection; the tag
  framing residual — `std-io` `Empirical` vs `std-fmt` `Exact` for `from_json` — is a noted
  open residual, not a silent decision).
- **The `runtime` RFC-0008 reserved vocabulary** (`hypha`, `fuse`, `xloc`, …) must NOT be
  added to the public API without a Phase-7 gate decision.
- **`just check`** runs `cargo clippy -D warnings` + `cargo test`; never commit a red check.
