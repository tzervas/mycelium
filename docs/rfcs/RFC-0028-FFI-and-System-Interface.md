# RFC-0028 — FFI and System Interface

| Field | Value |
|---|---|
| **RFC** | 0028 |
| **Status** | **Draft** (2026-06-23) |
| **Feeds** | E14-1 (FFI & system interface) |
| **Decides** | The capability-based Mycelium↔C/Rust FFI model; the `wild`/`@std-sys` host-execution floor that currently type-checks but does not execute (DN-14 row 9); syscall binding strategy for `std.io`/`std.fs`/`std.sys`; the ADR-014 unsafe-floor confinement for the FFI surface. |
| **Date** | June 23, 2026 |
| **Task** | E14-1 (M-720) |

> **Posture (honesty rule / VR-5).** This is a planning stub — scope, user stories, and open
> questions only. Nothing is decided normatively here. All guarantee claims remain `Declared`
> until a mechanized or empirical basis is recorded. Status is **Draft** until a binding
> decision is reached and the maintainer signs off.

---

## 1. Problem / Goal

The `wild`/`@std-sys` FFI gate landed in M-661 (DN-14 row 9): a `wild { … }` block type-checks
and is capability-gated (`@std-sys` context + `!{ffi}` effect annotation) but **execution is
staged** — `elab.rs` lowers `wild` to an explicit `Residual`, so no Mycelium program can
actually invoke a host operation. Every `std.io`, `std.fs`, `std.sys`, `std.rand`, and `std.time`
module that bottoms out in a syscall is therefore non-executable in-language until the FFI floor
is real.

For Mycelium to be fully usable as a language (the 1.0.0 north star), programs must be able to
read files, write output, generate entropy, and call real system APIs. This requires:

1. **A normative FFI model** — how Mycelium values cross the ABI boundary to C or Rust, what
   the guarantee tag on an FFI call is, and how the EXPLAIN obligation extends to foreign
   dispatch (G2 / ADR-006).
2. **Capability-based confinement** — the `@std-sys` gate is already in place as the surface
   gating mechanism; this RFC decides the *capability model* it represents: what capabilities
   are granted, how they are composed, and what audit trail an FFI call leaves.
3. **The `wild` execution host** — `elab.rs` currently emits `Residual` for `wild` blocks;
   this RFC decides how the elaborator resolves them: via a Rust FFI trampoline, a `libc` syscall
   layer, or a host-provided capability handle.
4. **ABI honesty** — FFI calls to C/Rust carry implicit ABI contracts (calling convention,
   alignment, lifetime of pointers) that are not in Mycelium's type system. Per ADR-014, the
   `wild` body is the trusted/opaque FFI escape; this RFC defines what "audited, not verified"
   means as a protocol and what the `// SAFETY:` audit requirement covers.
5. **Syscall binding surface** — `crates/mycelium-std-{io,fs,sys,rand,time}` are partial stubs
   today; this RFC specifies the binding strategy (direct `libc` calls, a `std-sys` trampoline
   layer, or a thin `wild` shim) and the guarantee matrix format for syscall-backed operations.

Relation to ADR-014: the unsafe policy (permitted-but-warned, `// SAFETY:` required) is already
decided. This RFC adds the Mycelium-language-level specification on top — the `wild` semantic
model that ADR-014's confinement applies to. Relation to DN-14 row 9: the gate is in place; this
RFC defines what is behind it.

---

## 2. User stories

- As a **language user**, I want to write a Mycelium program that reads a file and prints its
  contents, so that Mycelium is usable for real programs at the 1.0.0 gate — not just programs
  that run entirely in-memory.
- As a **library/phylum author** writing `std.io` or `std.rand`, I want a normative FFI model
  that specifies what guarantee tag to assign syscall-backed operations, so that the guarantee
  matrix (RFC-0016 §4.5) is grounded rather than hand-waved.
- As a **compiler engineer**, I want the `wild` execution path to be a concrete, testable
  elaboration target — not a `Residual` stub — so that the DN-14 row 9 gate is provably closed
  end-to-end.
- As a **tool author** building a sandboxing layer on top of Mycelium programs, I want the
  capability model to be an explicit surface (not ambient OS-level permissions), so that a
  program that does not declare the `io` capability cannot obtain it through a side channel.
- As an **AI co-author agent** generating Mycelium stdlib code, I want the FFI protocol
  (capability annotation + `wild` body + `// SAFETY:` comment + guarantee tag) to be a
  normative pattern I can verify against rather than a convention I must infer from the source.
- As a **maintainer**, I want the FFI surface to be confined enough that the `just safety-check`
  gate (ADR-014) can audit every `wild` site, so that the trusted-base boundary (ADR-007) is
  auditable even as the FFI surface grows.
- As an **operator** deploying Mycelium programs in a sandboxed environment, I want the
  capability model to be runtime-enforceable: a program without the `fs` capability must not be
  able to open files, even if it calls a `@std-sys` nodule directly.

---

## 3. Scope and decision space

**In scope:**

- The capability model for `@std-sys`-gated nodules: what a capability is, how it is granted,
  how it composes, and what audit trail it produces
- The `wild` execution host: how `elab.rs`'s `Residual` for `wild` blocks is resolved at
  runtime (trampoline, `libc` shim, or host handle)
- Mycelium↔C/Rust calling convention: value layout, pointer lifetime, FFI ABI honesty
- Guarantee tag assignment for FFI-backed operations (the expected tag is `Declared`; the
  reasoning must be explicit)
- The `// SAFETY:` audit protocol as a Mycelium-language-level requirement (beyond ADR-014's
  Rust-level requirement)
- Syscall binding strategy for `std.io`, `std.fs`, `std.sys`, `std.rand`, `std.time`
- `just safety-check` scope: what the gate checks about the FFI surface
- Interaction with ADR-014 (unsafe-floor confinement) and RFC-0016 §8-Q6 (`std-sys` phylum
  split)

**Out of scope:**

- The network/`xloc` FFI (R2 distribution constructs; deferred to the R2 RFC)
- WASM/WASI target (a separate future RFC; the model here is native syscalls)
- Kernel-level or bare-metal syscall paths (the v0 target is a hosted OS environment)
- JIT/AOT code patching via `dlopen`/`dlsym` (already in `crates/mycelium-mlir/src/jit.rs`;
  not the same as user-facing FFI)
- Full formal verification of FFI safety (the guarantee is `Declared`; mechanized proof is
  future work)

---

## 4. Definition of Done

- [ ] A normative FFI model is specified: capability model, `wild` execution host, ABI boundary
  protocol.
- [ ] Guarantee tags are assigned per FFI-backed operation category on the lattice (`Declared`
  for all syscall-backed ops in v0 — must be explicit, not defaulted).
- [ ] The `wild` elaboration path is specified: `elab.rs`'s `Residual` is resolved to a concrete
  target (not left staged); the three-way differential (L1-eval ≡ L0-interp ≡ AOT) extends to
  cover `wild`-backed operations.
- [ ] The capability model is specified well enough to implement M-720 (FFI surface) and M-721
  (`wild` host execution).
- [ ] The `// SAFETY:` audit protocol is normative: every `wild` site in a `@std-sys` nodule
  must carry a `// SAFETY:` comment specifying the ABI contract; `just safety-check` verifies
  this mechanically.
- [ ] `crates/mycelium-std-{io,fs,sys,rand,time}` binding strategy is decided (not necessarily
  implemented here, but the pattern is specified).
- [ ] Status advances from `Draft` → `Proposed` → `Accepted` per the append-only discipline;
  maintainer sign-off required for `Accepted`.

---

## 5. Open questions

- **Capability model depth:** Is the capability model a Mycelium-language-level type (a
  `Capability<io>` value the program must hold) or a build-time / linker-time attribute (the
  `@std-sys` annotation is the full capability check, nothing runtime)? The former is more
  EXPLAIN-able; the latter is simpler and is what DN-14 row 9 implements today.
- **`wild` execution host:** Should the `wild` body be dispatched via a Rust FFI trampoline
  baked into the interpreter, or via a capability handle passed into the `Scope` context? The
  trampoline is simpler; the handle is more composable and testable.
- **ABI boundary:** Mycelium values are immutable and content-addressed; a C function expects a
  mutable pointer or a by-value primitive. Should the RFC specify a canonical value-to-C
  encoding (analogous to the `wild` body being the trusted escape), or defer encoding to the
  `@std-sys` author?
- **Guarantee tag for `wild`:** Today the gate is `Declared`. Is that the right tag for a
  successfully-executed `wild` block (the body is trusted/audited, not verified), or should the
  tag be `Empirical` if the call is covered by an in-repo property test?
- **`just safety-check` scope:** Should `safety-check` check only `// SAFETY:` comment presence
  (ADR-014's current scope) or also check that every `wild` site is in a `@std-sys` nodule with
  the `!{ffi}` annotation (a Mycelium-level audit, not just a Rust-level one)?
- **Interaction with `xloc`:** When a value is translocated to another node (R2), its `wild`
  host operations do not travel with it. How does the capability model compose across node
  boundaries? (Deferred to the R2 RFC, but must be flagged here to avoid a retroactive
  incompatibility.)

---

## 6. Grounding / honesty

Claims in this stub are `Declared` (stated intent) or open questions. No guarantee is `Proven`
without a mechanized proof; none is `Empirical` without a property test in-repo.

Grounding basis: DN-14 row 9 (`wild`/FFI gate — conditionally present, execution staged;
`elab.rs` `Residual`); ADR-014 (`unsafe` policy — permitted-but-warned, `// SAFETY:` required);
RFC-0016 §8-Q6 (`std-sys` phylum split — the mechanism for confining OS-level surface);
RFC-0004 §2 (native backend / AOT need for FFI — ADR-009); G2 (no black boxes / never silent —
FFI calls must surface their ABI contracts); VR-5 (honesty rule / no tag upgrade without checked
basis — `Declared` for all v0 FFI-backed ops); KC-3 (small auditable kernel — FFI surface must
not require new Core-IR nodes); LR-9 (`wild` is the single permitted language-level FFI escape;
all else is refused, not ignored).

Existing crate references (partial/stub state today): `crates/mycelium-std-io/src/io.rs`,
`crates/mycelium-std-fs/src/fs.rs`, `crates/mycelium-std-sys/src/lib.rs`,
`crates/mycelium-mlir/src/jit.rs` (the only current `unsafe` site, confined per DN-21/M-682).

---

## Meta — changelog

- **2026-06-23 — Draft created.** Planning stub for the FFI and system interface model. Scope,
  user stories, open questions established. Status: Draft. Task: E14-1 (M-720). No normative
  decision made. (Append-only; VR-5.)
