# ADR-019 — libMLIR is a version-matched, feature-gated build dependency; provisioning is a durable decision

| Field | Value |
|---|---|
| **ADR** | 019 |
| **Title** | Adopt libMLIR (the `mlir-opt`/`mlir-translate` toolchain, version-matched to the installed LLVM major) as the OPTIONAL build dependency of the `mlir-dialect` Cargo feature of `mycelium-mlir` — provisioned durably by `scripts/setup-mlir.sh`, OFF by default, with an honest skip path; the default build and `cargo test` stay green without it |
| **Status** | **Accepted** (drafted 2026-06-20; ratified 2026-06-20; grounds RFC-0004 §2 / ADR-009) |
| **Date** | 2026-06-20 |
| **Depends on** | RFC-0004 §2 (MLIR→LLVM backbone) / §6 (inspectability) / §9.3 (host-target-only); ADR-009 (hybrid execution / no-opaque-lowering, all backends); DN-15 (native-path decomposition — the libMLIR-gated half); M-348 (the provisioning block this resolves); M-601 (real ternary→arith/vector→LLVM lowering) / M-602 (three-way differential) / M-603 (this provisioning recipe + ADR) |
| **Resolves** | M-348 provisioning block (libMLIR-absent premise) on Linux |

## Context

RFC-0004 §2 ratified **`MLIR → LLVM`** as Mycelium's AOT (perf-path) backbone; ADR-009 is the
hybrid-execution decision that binds **no-opaque-lowering to *all* backends** (every stage
dumpable/inspectable — RFC-0004 §6). The native dialect path (the `ternary` → `arith`/`vector` →
LLVM lowering) requires a live libMLIR binding; M-348 ("Provision libMLIR to unblock the native
MLIR→LLVM path") long carried the **"libMLIR absent" premise**, and DN-15 §2 / §4.4 recorded that
half as honestly **blocked** until that toolchain was provisioned (VR-5: no verdict upgraded without
the toolchain present).

That premise no longer holds on **Linux**. Verified in this repo's container (the ground truth this
ADR records):

- `apt-get install -y --no-install-recommends libmlir-18-dev mlir-18-tools` installs candidate
  **`1:18.1.3-1ubuntu1`** — **version-matched to the installed LLVM 18.1.3** (`llc --version` reports
  `Ubuntu LLVM version 18.1.3`) — providing `/usr/bin/mlir-opt-18`, `/usr/bin/mlir-translate-18`, and
  `libMLIR.so.18.1`.
- The pipeline
  `mlir-opt-18 --convert-func-to-llvm --convert-arith-to-llvm --reconcile-unrealized-casts <in.mlir> | mlir-translate-18 --mlir-to-llvmir`
  emits **valid LLVM IR** end-to-end.

So on Linux, libMLIR is **provisionable now**, version-matched to the LLVM major already installed.
The M-348 premise is empirically false there (VR-5 cuts both ways: downgrade to stay honest, but
**upgrade** when a checked basis appears — here the basis is the verified install + working pipeline).

## Decision

**1. libMLIR is the OPTIONAL build dependency of the `mlir-dialect` Cargo feature of `mycelium-mlir`
— NOT of the default build.** The `mlir-opt`/`mlir-translate` toolchain (version-matched to the
installed LLVM major) is what the real ternary-dialect lowering (M-601) links/shells out to. It is
declared as a dependency of that **feature**, which is **OFF by default**.

**2. The default build and default `cargo test` stay green WITHOUT libMLIR.** With the feature off,
`mycelium-mlir` neither requires nor probes libMLIR; with the feature on, the dialect path **probes**
for the tools and **skips gracefully** when absent — mirroring the existing `llc`/`clang`
`ToolchainMissing` idiom in `crates/mycelium-mlir/src/llvm.rs` (an explicit, never-silent skip; G2).
The **interpreter remains the trusted base** (NFR-7); the dialect path is a perf/inspectability path,
never the trusted base.

**3. Provisioning is a DECISION, made durable by `scripts/setup-mlir.sh` wired into `just setup`.**
The script **derives the LLVM major from the installed `llc`** (then `clang`) and installs the
**version-matched** `libmlir-$MAJOR-dev` + `mlir-$MAJOR-tools` via the distro package manager only
(no `curl | bash`, no unpinned remote fetch). It is idempotent (no-ops when the tools are present)
and skips gracefully (clear message + `exit 0`) when LLVM, `apt-get`, or the packages are absent.
This is **never a silent toolchain bump** — the major is read from what is already installed, honoring
CLAUDE.md's "Don't silently bump committed version pins (MSRV, Python) — that's a decision (ADR), not
a build detail."

**4. The committed MSRV / LLVM-major pins are UNCHANGED.** This ADR adds an *optional*,
*version-matched* native toolchain gated behind a feature with an honest skip path — it does not move
any pin and does not make libMLIR a precondition of the trusted base.

## Consequences

- The real `ternary` → `arith`/`vector` → LLVM dialect lowering (**M-601**) becomes **provisionable
  and testable on Linux**; the three-way differential (interp ≡ AOT ≡ native dialect, **M-602**) can
  run there.
- **CI/dev on a box without libMLIR is unaffected**: the `mlir-dialect` feature is off, its tests
  skip, the default build/test stay green (the `llc`/`clang` skip idiom, generalized).
- **Honest cost:** a contributor who wants the `mlir-dialect` feature must run `just setup` (or
  `bash scripts/setup-mlir.sh`) **once**. The feature is advisory, never required for the trusted base
  (the interpreter — NFR-7). On non-Linux (Windows/macOS) the script prints a **named-package**
  message instead of auto-installing (§ Scope below).
- DN-15's §5 "Increment 4 — real ternary MLIR dialect" row moves from *blocked on M-348 / not
  established* toward *provisionable + in-progress under M-601* (recorded in DN-15 §9; the table text
  itself is append-only and unchanged).

## Alternatives considered

- **(a) Vendor / build libMLIR from source.** Rejected: heavy, slow, and platform-fragile; it
  re-introduces the C++ build cost that RFC-0004 §2 deliberately confines to the AOT path. The distro
  package is **version-matched and immediate** — strictly better for a provisioning recipe.
- **(b) Make libMLIR a hard *default* dependency.** Rejected: it breaks the "default build green
  without the toolchain" house idiom and the trusted-base-is-the-interpreter principle (NFR-7,
  ADR-009). The dialect path is a perf/inspectability path, not the trusted base — gating it behind an
  off-by-default feature is the honest shape.
- **(c) Stay blocked / textual-skeleton-only.** Rejected: the M-348 premise is **empirically false on
  Linux now** (verified install + working pipeline). VR-5 requires the upgrade once a checked basis
  appears; staying blocked would be dishonest in the other direction. (The textual skeleton in
  `dialect.rs`, DN-15 §2, remains valid as the inspectable stand-in where libMLIR is absent.)

## Scope / honesty (VR-5)

This ADR decides the **toolchain dependency + the durability of provisioning**. It does **not** itself
claim the dialect lowering is complete — that is **M-601**, tagged at its own honestly-supportable
strength (no guarantee upgraded here) — nor does it claim any **speedup**: **M-602** is the measured
differential, with **no pre-written performance target** (VR-5 — never upgrade without a checked
basis; G2 — never silent). **Cross-target codegen for non-host triples stays out of scope** (RFC-0004
§9.3: host-target only until the native backend lands). It records that libMLIR is now provisionable
**on Linux specifically**; other platforms (Windows/macOS) receive a **named-package message** from
`scripts/setup-mlir.sh`, not an automated install — an honest skip, never a silent half-step.

## Meta — changelog

- **2026-06-20 — Proposed.** Drafts the resolution of the M-348 "libMLIR absent" premise on Linux:
  the verified `apt-get install libmlir-18-dev mlir-18-tools` (candidate `1:18.1.3-1ubuntu1`,
  version-matched to the installed LLVM 18.1.3) provides `mlir-opt-18`/`mlir-translate-18` and the
  `--convert-*-to-llvm | mlir-translate --mlir-to-llvmir` pipeline emits valid LLVM IR (VR-5: upgrade
  on a checked basis). Decides libMLIR as the **optional**, version-matched build dependency of the
  off-by-default `mlir-dialect` feature of `mycelium-mlir` — default build/test stay green without it
  (the `llc`/`clang` skip idiom, G2) — made durable by `scripts/setup-mlir.sh` (LLVM major derived
  from the installed `llc`, never hard-coded; no `curl | bash`) wired into `just setup`. MSRV/LLVM
  pins unchanged; the interpreter stays the trusted base (NFR-7). Awaiting maintainer ratification
  (Proposed → Accepted). Append-only.
- **2026-06-20 — Accepted.** Same-day maintainer ratification (Proposed → Accepted; matching ADR-017's
  same-day pattern — steps through Accepted, does **not** skip to Enacted). No change to the decision
  or its scope: the toolchain dependency is optional + feature-gated + version-matched, provisioning is
  durable via `scripts/setup-mlir.sh` + `just setup`, and the honesty scope (this ADR decides the
  dependency, not the completeness of M-601 nor any M-602 speedup) stands. This **unblocks M-601** (the
  real ternary→arith/vector→LLVM lowering becomes provisionable + testable on Linux) and **M-602** (the
  three-way differential); CI/dev without libMLIR is unaffected (feature off, tests skip). Cross-target
  codegen stays deferred (RFC-0004 §9.3) — honest scope, VR-5. Append-only.
