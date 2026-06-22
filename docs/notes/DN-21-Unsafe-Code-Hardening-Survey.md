# Design Note DN-21 — Unsafe-Code Hardening Survey

| Field | Value |
|---|---|
| **Note** | DN-21 |
| **Status** | **Draft** (2026-06-22; planning capture) |
| **Feeds** | ADR-014 (Unsafe-Code Policy, **Accepted**) — enacts its named follow-ons; LR-9 (`wild` is the only *in-language* unsafe escape, denied by default); KC-3 (small auditable kernel); house rule #5 (SOLID · DRY · KISS · YAGNI) |
| **Date** | June 22, 2026 |
| **Decides** | *Planning capture, advisory (DN-17 posture) — **not** a ratified decision.* Records a grounded, read-only audit of **every** `unsafe` block in the Rust workspace, verifies each carries an adequate `// SAFETY:` justification per ADR-014, and gives a priority-ordered, risk-tagged hardening plan (the tracked epic **M-678**). **No code is changed by this note** — nothing is refactored here. |
| **Task** | M-678 — unsafe-code hardening (design-first) |

> **Posture (honesty rule / VR-5 / ADR-014).** Every finding below is grounded in source
> (`file:line`, verified by reading — `Exact` unless tagged). The survey is conservative: it
> recommends hardening that is **behaviour-preserving** and **conforms to ADR-014** (which keeps the
> workspace at `unsafe_code = "warn"`, not `forbid`, and requires a per-site `// SAFETY:`), and it
> names the **irreducible** unsafe floor honestly rather than pretending it away. Two standing
> constraints carry over from DN-17: **(1)** any refactor must be a behaviour-preserving no-op,
> verified by the existing `mycelium-mlir` JIT differential (`tests/*differential*`) + the workspace
> suite green; **(2)** standard Rust conventions now (a small in-house newtype over a new dependency
> where it keeps the kernel auditable — KC-3). Crate/API claims about third-party crates are
> `Empirical` (docs verified), not asserted.

---

## 1. Why this note exists

ADR-014 (Accepted, 2026-06-15) set the workspace unsafe policy — `unsafe_code = "warn"`, a mandatory
per-site `// SAFETY:` comment, `clippy … -D warnings -A unsafe_code` as the lint gate — and recorded
two **follow-ons as future work**: re-pinning the trusted-base crates to `forbid`, and a lightweight
check asserting every `unsafe` has an adjacent `// SAFETY:`. Neither has been enacted. This note
audits the current reality against the policy, confirms the justifications, and turns those
follow-ons (plus a structural lifetime hardening) into a tracked, priority-ordered plan. It is the
unsafe-code twin of DN-17's DRY survey: a read-only map + a fearless-refactor plan, nothing changed.

## 2. The unsafe inventory (exhaustive)

A whole-`crates/` audit (real `unsafe { }` / `unsafe fn` / `unsafe impl`, excluding `forbid`/`deny`
lines, doc-comments, and string literals) finds **exactly 6 `unsafe` blocks, all in
`crates/mycelium-mlir`**, and all are **dynamic-linking FFI** (there is *no* Rust-side SIMD-intrinsic
unsafe in the workspace — the SIMD lives inside JIT-compiled LLVM IR, which Rust never touches):

| # | Location | What it does | Kind |
|---|---|---|---|
| 1 | `crates/mycelium-mlir/src/jit.rs:168` | `dlopen` the JIT'd `.so` (`RTLD_NOW`) | FFI |
| 2 | `crates/mycelium-mlir/src/jit.rs:181` | `dlsym` a symbol | FFI |
| 3 | `crates/mycelium-mlir/src/jit.rs:150` | `dlclose` in `Drop for Lib` | FFI |
| 4 | `crates/mycelium-mlir/src/jit.rs:118` | `transmute` `*mut c_void` → `extern "C" fn` + call the JIT'd kernel | FFI + fn-ptr |
| 5 | `crates/mycelium-mlir/src/bitnet.rs:349` | `transmute` + call the packed-ternary dot kernel | FFI + fn-ptr |
| 6 | `crates/mycelium-mlir/src/specialize.rs:128` | `transmute` + call the weight-specialized dot kernel | FFI + fn-ptr |

Everything else is clean (read-verified):
- **29 crates** carry `#![forbid(unsafe_code)]` (the interpreter trusted base, all 23 `mycelium-std-*`,
  `mycelium-l1`, `mycelium-stack`, …).
- The `mycelium-sec` / `mycelium-lsp` textual "unsafe" hits are **doc-comments / string literals** about
  the `wild` escape hatch — not code.
- `mycelium-stack`'s deep worker stack (`with_deep_stack`, M-674) uses **pure-safe `std::thread`**
  (`Builder::stack_size(256 MiB)` + scoped threads) — no `unsafe`, no dependency.
- `mycelium-mlir` declares the `dlopen`/`dlsym`/`dlclose` FFI with a bare `extern "C" {}` and
  `std::os::raw` — **no `libc` dependency** (an intentional ADR-014 choice). `libc` appears only as a
  *transitive* floor (via `blake3`→`cpufeatures` and `rustix`), never called from Mycelium source;
  that tier is governed by `cargo-deny`/`cargo-audit`.

**The architecture holds:** forbid-by-default everywhere, unsafe confined to the AOT/JIT perf-path,
the interpreter trusted base unsafe-free (ADR-014's intent, verified).

## 3. Justification adequacy (ADR-014)

All 6 blocks carry both a `// SAFETY:` comment **and** `#[cfg_attr(not(debug_assertions), allow(unsafe_code))]`.
Adequacy (read-verified):

| Block | `// SAFETY:` adequacy | Gap |
|---|---|---|
| `jit.rs:118` transmute+call | **Strongest in the crate** — names ABI match, buffer bounds, library liveness | mentions neither the ptr-size-equivalence of the transmute nor the overflow-path no-write (both low-risk) |
| `bitnet.rs:349`, `specialize.rs:128` | Adequate — ABI match (the exact IR signature), in-bounds (the pre-call length check), liveness (`_lib`) | the "symbol is exactly the one we compiled" side-condition (unique tmp-dir) is implicit |
| `jit.rs:168` `dlopen` | Adequate — NUL-termination, `RTLD_NOW` eager resolve | `RTLD_NOW = 2` is **hard-coded** (a platform assumption — no `libc` constant); global-constructor safety unstated |
| `jit.rs:150` `dlclose` | **Incomplete** — covers single-close, but **omits the dangling-pointer obligation** (no derived symbol ptr may outlive the `Lib`) | see §4 |
| `jit.rs:181` `dlsym` | **Thin** — "checked non-null" is true only by the `open_lib` construction chain, not stated | handle-liveness argument implicit |

No block is *wrong*; three are *thin* and warrant a one-line strengthening (M-679).

## 4. The structural finding — the `*mut c_void` co-location risk

`JitArtifact::call` keeps the library handle and the derived fn-pointer in the **same stack frame**, so
the borrow checker enforces "symbol does not outlive library" structurally. But `BitnetDotKernel` /
`SpecializedDotKernel` store `fptr: *mut c_void` as a **raw field alongside `_lib: Lib`** — the
"`fptr` must not outlive `_lib`" invariant is held only by **field co-location convention**, not a
compiler-checked lifetime. It is **not a current bug** (the struct keeps both fields together and
`call` takes `&self`), but a future refactor that extracted `fptr` from the struct would produce a
**silent dangling pointer** with no diagnostic. This is the single highest-value hardening target, and
the reason the §6 plan adopts a lifetime-binding newtype.

## 5. Policy-coverage findings

- **F-1 — trusted-base crates lack `#![forbid(unsafe_code)]`.** ADR-014 *recommends* re-pinning the
  trusted base (`mycelium-core`, `-cert`, `-numerics`, `-vsa`, `-interp`, and `-select`) to `forbid`;
  they have zero unsafe today but rely on the workspace `"warn"`. Not yet enacted → a future accidental
  `unsafe` would only warn, not fail. (→ M-680)
- **F-2 — 11 zero-unsafe `mycelium-mlir` submodules** (`aot`, `llvm`, `pack`, `dialect`, `runtime`,
  `simd`, `inject`, `budget`, `channel`, `deploy`, `vr4`) could carry a per-file `#![forbid(unsafe_code)]`,
  confining unsafe to exactly `jit`/`bitnet`/`specialize` while the crate stays `"warn"` for those three.
  (→ M-680)
- **F-3 — no machine-enforced `// SAFETY:` adjacency check.** ADR-014 explicitly defers a grep asserting
  every `unsafe` has an adjacent `// SAFETY:`. The 6 blocks comply today, but only peer review + the
  `"warn"` lint guard it. (→ M-681)
- **F-4 — the `myc-sec` `audit_wild` scope boundary is implicit.** `audit_wild` scans **`.myc`** files for
  the in-language `wild { }` surface; it does **not** cover the Rust `unsafe { }` in `mycelium-mlir`
  (those are governed by `clippy -A unsafe_code` + the `// SAFETY:` convention). This two-population model
  (`.myc` ⟂ `.rs`) is architecturally correct but under-documented. (→ M-682)
- **F-5 (informational) — `libc` transitive floor.** Reaches the tree via `blake3`/`rustix`, below the
  Mycelium source boundary; governed by `cargo-deny`/`cargo-audit`. No action.

## 6. The hardening plan (priority-ordered, risk-tagged)

| P | Action | Risk | New dep? | Issue |
|---|---|---|---|---|
| **1** | **Quick-win SAFETY hardening** — strengthen the 3 thin comments (state the `dlclose` dangling-ptr obligation §4, the `dlsym` liveness chain, the `RTLD_NOW` platform assumption) + add `debug_assert!(!ptr.is_null())` at the 4 FFI/transmute sites | none (docs + dev-asserts) | no | **M-679** |
| **1** | **Forbid-pin** the trusted base (F-1) + the 11 zero-unsafe `mycelium-mlir` submodules (F-2) — convert "no unsafe by convention" into compiler-enforced | low | no | **M-680** |
| **2** | **`just safety-check`** — a `// SAFETY:`-adjacency gate (F-3); closes the ADR-014-named follow-on; skip-graceful shell/python, no dep | low | no | **M-681** |
| **2** | **In-house `Sym<'lib, T>` newtype** (maintainer-chosen over `libloading`) — lifetime-bind `dlsym` results to the owning `Lib`, replacing the raw-field `fptr` in `BitnetDotKernel`/`SpecializedDotKernel` and closing the §4 co-location risk; zero new dependency (KC-3) | moderate (self-referential ownership — design carefully) | no | **M-682** |
| **3** | **Document** the `audit_wild` (`.myc`) vs `clippy -A unsafe_code` (`.rs`) two-population split in `docs/spec/Security-Checks-Contract.md` §4 (F-4) | none (docs) | no | **M-683** |

All five are **behaviour-preserving** and **zero new dependency** (the in-house `Sym<'lib,T>` is the
deliberate KC-3 choice over `libloading`). None touch the JIT hot path's performance.

## 7. The irreducible-unsafe floor (leave alone, honestly)

These cannot be removed by **any** safe-Rust wrapper (including `libloading`) and must stay `unsafe`,
honestly justified rather than pretended away:

1. **Calling a JIT-compiled function pointer.** The compiler cannot verify externally-emitted native
   code's ABI/behaviour. Irreducible for any in-process JIT — mitigated by the existing differential
   tests + the single-source-of-truth IR emitter (`emit_*_ir`), not by a type.
2. **`dlopen` / `Library::new` constructor safety** — a shared library's global constructors are a
   semantic obligation, not a type. (`Empirical`: the JIT IR emits no `@llvm.global_ctors`.)
3. **The ABI type claim in the `transmute` / `Library::get::<T>`** — the fn signature is the caller's
   assertion, definitionally outside the type system.

And a non-goal: **do not** rewrite the kernels in Rust `std::arch`/`core::simd` — that needs nightly
(or a SIMD-wrapper dependency), loses the IR-inspectability guarantee (FR-C3 / RFC-0004 §6), and
reintroduces the CPU target-feature gating that delegating to `clang` currently sidesteps. The JIT
emit-IR→compile→`dlopen`→call architecture is the right design; this note hardens its seams, not its shape.

## Meta — changelog

- **2026-06-22 — DN-21 created (planning capture; advisory, DN-17 posture).** Records a grounded,
  read-only audit of all 6 workspace `unsafe` blocks (every one dynamic-linking FFI in
  `crates/mycelium-mlir`), confirms each carries an adequate ADR-014 `// SAFETY:` justification (3 thin,
  none wrong), identifies the `BitnetDotKernel` co-location dangling-ptr risk as the top hardening
  target, and scopes a 5-issue behaviour-preserving, zero-new-dependency hardening epic (**M-678** →
  M-679…M-683) enacting ADR-014's named follow-ons + an in-house `Sym<'lib,T>` lifetime-binding newtype
  (maintainer-chosen over `libloading`). No code changed. Grounded in ADR-014 (Accepted), LR-9, KC-3.
  Append-only.
- **2026-06-22 — M-679…M-683 landed (epic M-678 enacted; behaviour-preserving, zero new dependency).**
  M-679: strengthened the 3 thin `// SAFETY:` comments (§3) + `debug_assert!(!ptr.is_null())` at the
  FFI/transmute sites. M-680: re-pinned `#![forbid(unsafe_code)]` on the trusted base (§5 F-1) + the 11
  zero-unsafe `mycelium-mlir` submodules (§5 F-2). M-681: `just safety-check` (`scripts/checks/safety.sh`)
  — the `// SAFETY:`-adjacency gate (§5 F-3), wired into `just check`. M-683: documented the `audit_wild`
  (`.myc`) ⟂ Rust `unsafe` (`.rs`) two-population split (§5 F-4 → `Security-Checks-Contract.md` §4.1).
  M-682: the in-house `Sym<'lib, T>` lifetime-binding newtype + a `bind`-once `BoundBitnetDot`/
  `BoundSpecializedDot` handle closed the §4 co-location dangling-ptr risk **structurally** (compiler-
  checked lifetime; no raw `*mut c_void` field survives; no per-call `dlsym` in the E1 hot loop).
  **Inventory + confinement (honesty):** all `unsafe` now lives in `jit.rs` and nowhere else —
  `bitnet`/`specialize` are themselves `#![forbid(unsafe_code)]`, resolving their symbols through safe,
  fixed-type accessors. The fn-pointer `transmute` is a single private `unsafe fn get` (so the ABI claim
  is never made by safe code — a soundness fix from the PR review), wrapped by three audited safe
  accessors (`jit_kernel`/`bitnet_dot`/`spec_dot`) that each assert the correct `extern "C"` signature
  against the IR this crate emits. The workspace `unsafe`-block count is **8**, every one in `jit.rs`:
  the 3 dynamic-linker FFI (`dlopen`/`dlsym`/`dlclose`), the private `unsafe fn get` + its `transmute_copy`,
  and the 3 ABI-asserting accessors. (This is up from §2's **6** because each ABI claim is now made
  explicitly and per-symbol-class rather than inline-and-implicit at the call site — the surface is
  larger in count but smaller in *kind*: confined to one file, no generic safe resolver, no safe-code UB.)
  The §7 irreducible floor (calling the JIT'd fn-ptr; the ABI claim) is unchanged — now expressed in one
  place, lifetime-bound, not pretended away. Append-only.
