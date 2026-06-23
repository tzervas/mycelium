# RFC-0028 — FFI and System Interface

| Field | Value |
|---|---|
| **RFC** | 0028 |
| **Status** | **Accepted** (maintainer sign-off, 2026-06-23) |
| **Feeds** | E14-1 (FFI & system interface) |
| **Decides** | The capability-based Mycelium↔C/Rust FFI model; the `wild`/`@std-sys` host-execution floor that previously type-checked but did not execute (DN-14 row 9); the syscall binding strategy for `std.io`/`std.fs`/`std.sys`/`std.rand`/`std.time`; the ADR-014 unsafe-floor confinement for the FFI surface. |
| **Date** | June 23, 2026 |
| **Task** | E14-1 (M-720 … M-724) |

> **Posture (honesty rule / VR-5).** Every guarantee tag this RFC assigns to an FFI-backed
> operation is **`Declared`** in v0 — an audited, not verified, claim — *except* a `wild` path
> that is covered by the in-repo three-way differential (which earns **`Empirical`** for that
> specific deterministic operation). No FFI claim is `Proven`; none is upgraded without a checked
> basis. The capability model decided here is the **build-time `@std-sys` gate** (§4.1); the
> runtime-enforced sandboxing variant is explicitly deferred (§7) and flagged so a future RFC can
> add it without a retroactive incompatibility.

---

## 1. Problem / Goal

The `wild`/`@std-sys` FFI gate landed in M-661 (DN-14 row 9): a `wild { … }` block type-checks
and is capability-gated (`@std-sys` context + `!{ffi}` effect annotation) but **execution was
staged** — `elab.rs` lowered `wild` to an explicit `Residual`, so no Mycelium program could
actually invoke a host operation. Every `std.io`, `std.fs`, `std.sys`, `std.rand`, and `std.time`
module that bottoms out in a syscall was therefore non-executable in-language until the FFI floor
became real.

For Mycelium to be fully usable as a language (the 1.0.0 north star), programs must be able to
read files, write output, generate entropy, and call real system APIs. This RFC decides the five
pieces that make that real:

1. **A normative FFI model** — how a `wild` block names a host operation, how its arguments cross
   the boundary, and how the EXPLAIN obligation extends to foreign dispatch (G2 / ADR-006).
2. **Capability-based confinement** — what the `@std-sys` gate represents as a capability model.
3. **The `wild` execution host** — how `elab.rs`'s `Residual` for `wild` blocks is resolved at
   runtime.
4. **ABI honesty** — what "audited, not verified" means as a protocol and what the `// SAFETY:`
   audit requirement covers.
5. **Syscall binding surface** — the binding strategy for `std.{io,fs,sys,rand,time}` and the
   guarantee-matrix format for syscall-backed operations.

Relation to ADR-014: the unsafe policy (permitted-but-warned, `// SAFETY:` required) is already
decided; this RFC adds the Mycelium-language-level specification on top. Relation to DN-14 row 9:
the gate was in place; this RFC defines, and M-720…M-724 implement, what is behind it.

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
  end-to-end across the three-way differential.
- As a **tool author** building a sandboxing layer on top of Mycelium programs, I want the
  capability surface to be explicit (the `@std-sys` gate) and the runtime-enforcement extension
  point named, so I can build a runtime capability check on a stable seam.
- As an **AI co-author agent** generating Mycelium stdlib code, I want the FFI protocol
  (capability annotation + `wild` body shape + `// SAFETY:` comment + guarantee tag) to be a
  normative pattern I can verify against rather than a convention I must infer from the source.
- As a **maintainer**, I want the FFI surface confined enough that `just safety-check` can audit
  every `wild` site, so that the trusted-base boundary (ADR-007) is auditable even as the FFI
  surface grows.

---

## 3. Scope

**In scope (decided here):** the capability model (§4.1); the `wild` body grammar + elaboration
(§4.2); the execution host (§4.3); ABI honesty + the `// SAFETY:` protocol (§4.4); the syscall
binding strategy (§4.5); the guarantee-tag policy (§4.6); the `just safety-check` scope (§4.7).

**Out of scope (deferred):**

- The network/`xloc` FFI (R2 distribution constructs; deferred to the R2 RFC) — see §7.
- WASM/WASI target (a separate future RFC; the v0 model is native syscalls).
- Kernel-level or bare-metal syscall paths (the v0 target is a hosted OS environment).
- JIT/AOT code patching via `dlopen`/`dlsym` (already in `crates/mycelium-mlir/src/jit.rs`;
  not the same as user-facing FFI).
- **Runtime-enforced** capability sandboxing (a `Capability<io>` value threaded through programs)
  — explicitly deferred (§7); v0 uses the build-time `@std-sys` gate.
- Full formal verification of FFI safety (the guarantee is `Declared`; mechanized proof is future
  work).

---

## 4. Decision

### 4.1 Capability model — the build-time `@std-sys` gate

**The capability is the `@std-sys` nodule attribute, checked at compile time. There is no runtime
`Capability<io>` value in v0.** A `wild` block is admissible **iff** it is lexically inside a
`@std-sys` nodule whose enclosing `fn` declares the `!{ffi}` effect (the M-661 gate, unchanged); a
`wild` block anywhere else remains a **hard `CheckError`** (G2 — never silent, not a lint). This
is the simplest model that satisfies the confinement user story, adds **no new language-level type
and no new Core-IR node** (KC-3), and matches what DN-14 row 9 already implements.

*Rationale (KC-3 / KISS / YAGNI).* The richer alternative — a first-class `Capability<io>` value
that a program holds and threads to each `wild` call, enabling runtime-enforced sandboxing — is
more EXPLAIN-able but adds new language types, new kernel surface, and a much larger v0 build for a
property (runtime sandboxing) that no shipped consumer needs yet. It is **deferred, not rejected**
(§7): the build-time gate is forward-compatible with it (a future RFC can require the capability
value *in addition to* the `@std-sys` gate without invalidating any program this RFC admits).

*Audit trail.* Because the gate is lexical, the audit trail is static and grep-able: every host
contact is a `wild` block in a `@std-sys` nodule (§4.7), and `EXPLAIN` over an FFI-backed value
reports the `wild:`-namespaced operation it came from (§4.3).

### 4.2 The `wild` body grammar and elaboration (M-720)

The body of a `wild { … }` block is the **trusted/opaque FFI escape** (M-661: not recursively
type-checked — audited, not verified). v0 fixes its **shape** so the elaborator can resolve it to
a concrete host dispatch without type-checking it:

```ebnf
WildBody ::= HostName "(" [ Expr { "," Expr } ] ")"   (* a host-call form *)
           | HostName                                  (* a bare host op, no arguments *)
HostName ::= Ident                                     (* single, undotted *)
```

- The `HostName` is the **host operation key**; each argument `Expr` is an ordinary in-scope
  Mycelium expression (a variable reference or value), elaborated through the normal path.
- A body that is **not** a single host-call form (e.g. a `let`, a nested block, a dotted name) is
  an **explicit elaboration refusal** (`ElabError::Residual`) — never a silent or fabricated
  artifact (G2). This keeps the v0 FFI surface narrow and auditable; richer host-body forms are a
  later, append-only extension.

**Elaboration target (no new Core-IR node — KC-3).** A `wild` block lowers to the existing
[`Node::Op`](../../crates/mycelium-core/src/node.rs) primitive-application node, under a reserved
**`wild:` prim namespace**:

```text
wild { name(a₁, …, aₙ) }   ⟶   Op { prim: "wild:name", args: [⟦a₁⟧, …, ⟦aₙ⟧] }
```

The `wild:` prefix is reserved: no built-in (paradigm) primitive may use it, so a `wild:`-prefixed
`Op` is unambiguously a host call. This reuses the kernel's single primitive-application node — the
FFI surface introduces **no new node** and so no new well-formedness obligation on the kernel.

### 4.3 The execution host — the capability handle is the prim registry (M-721)

`elab.rs`'s `Residual` is resolved by dispatching a `wild:`-namespaced `Op` through the
interpreter's **primitive registry** (`mycelium-interp::PrimRegistry`), which all three evaluation
paths (the L1 fuel-guarded evaluator, the L0 reference interpreter, and the AOT env-machine)
already thread. The registry **is** the capability handle (the "host dispatch table"):

- A host operation is registered under its `wild:<name>` key. The **default** registry
  (`PrimRegistry::with_builtins()`) registers **no** `wild:` op, so a program that uses `wild`
  but is run on a host that did not grant the operation gets an **explicit, never-silent**
  `EvalError::UnknownPrim` (whose message, for a `wild:` key, states that the host capability was
  not granted — G2). The capability is thereby *opt-in at the host*, composable, and testable.
- Because the registry is shared across all three paths, a host op registered once is resolved
  identically by L1-eval, L0-interp, and AOT — so the **three-way differential extends to
  `wild`-backed operations** with no change to the AOT signatures (the seam is the existing
  `prims` parameter).
- A host op is a `PrimFn` — `fn(prim: &str, args: &[&Value]) -> Result<Value, EvalError>` — i.e.
  it converts in-scope Mycelium `Value`s to/from its native effect. Real syscalls live in the
  single audited `mycelium-std-sys` phylum (LR-9; §4.5) and are wired into a registry by a host
  registration bridge.

*Chosen over a Rust trampoline baked into the interpreter* because the registry-as-handle is
composable (the host chooses which ops to grant), testable (a deterministic mock host op can be
injected into the differential), and keeps the interpreter free of a hard-coded syscall set.

### 4.4 ABI honesty and the `// SAFETY:` protocol (M-720/M-724)

- The `wild` body's argument-to-host encoding is **deferred to the `@std-sys` author** (the body
  is the trusted escape): the host `PrimFn` owns the `Value`↔native conversion and is responsible
  for its ABI contract. v0 does **not** impose a canonical value-to-C encoding — the syscall
  surface (§4.5) is native Rust `std`/`libc`-level, not arbitrary C structs.
- **Never-silent ABI claims.** A host `PrimFn` that cannot honour its ascribed result type returns
  an explicit `EvalError` (or the syscall's `Result::Err`), never a silently wrong-typed `Value`
  (G2). The `wild` block's *result* type is the ascribed Mycelium type (M-661); a host op that
  produces something else is a runtime refusal.
- **The `// SAFETY:` protocol.** Every `wild` site in a `@std-sys` `.myc` nodule must carry a
  `// SAFETY:` comment stating the ABI/host contract it relies on, and every Rust `unsafe` block in
  the FFI host layer must carry the ADR-014 `// SAFETY:` justification (`scripts/checks/safety.sh`,
  M-681). `just safety-check` verifies both (§4.7).

### 4.5 Syscall binding strategy (M-722/M-723)

All host/OS contact is confined to the **single audited `mycelium-std-sys` phylum** (LR-9 /
RFC-0016 §8-Q6) — the pure `std-*` crates stay `wild`-free and may keep `#![forbid(unsafe_code)]`.
The v0 binding strategy is **safe-Rust `std`/`libc` wrappers** inside `std-sys`, surfaced as host
`PrimFn`s:

| Module | v0 operations | Source | Tag |
|---|---|---|---|
| `std.io` | `read`/`write` over stdin/stdout streams | `std::io` | `Declared` |
| `std.fs` | `open`/`close`/`read`/`write`/`stat`/`remove` | `std::fs` | `Declared` |
| `std.sys` | process `exit`; env-var `get` | `std::process`, `std::env` | `Declared` |
| `std.rand` | OS entropy read (`/dev/urandom` / `getrandom(2)`) | `mycelium-std-sys::rand` | `Declared` |
| `std.time` | monotonic + wall-clock read | `mycelium-std-sys::time` | `Declared`* |

\* a *structural* monotonicity invariant on a monotonic clock may be tagged `Exact` where it is a
checked structural property; the *value* read remains `Declared` (VR-5). Every syscall failure is
an explicit `Result::Err`/`Option::None` — never a silent discard (G2).

### 4.6 Guarantee-tag policy (VR-5)

- **`Declared`** is the v0 tag for every syscall-backed operation (the host body is audited, not
  verified — no theorem, no measured bound).
- A specific `wild`-backed operation that is covered by the in-repo three-way differential (a
  **deterministic** host op whose L1-eval ≡ L0-interp ≡ AOT agreement is asserted) earns
  **`Empirical`** for that operation only — the differential is the recorded basis. Non-deterministic
  syscalls (entropy, clock) cannot be covered by an equality differential and stay `Declared`.
- No FFI claim is `Proven`. Promotion requires a checked basis recorded at the site (VR-5).

### 4.7 `just safety-check` scope (M-724)

`safety-check` performs **two** audits — a Rust-level one (existing) and a Mycelium-level one (new):

1. **Rust `// SAFETY:` adjacency** (existing, M-681, `scripts/checks/safety.sh`): every Rust
   `unsafe` site under `crates/` carries an adjacent `// SAFETY:` justification.
2. **Mycelium `wild`-site audit** (new): every `wild` block in a `.myc` file must (a) be in a
   nodule whose header carries `@std-sys`, (b) be inside a `fn` that declares `!{ffi}`, and
   (c) carry a `// SAFETY:` comment. A `wild` site failing any of these **fails the gate** (a
   gate, not a lint — G2). This is a regex heuristic (`Empirical`/`Declared`; the checker/source is
   ground truth) and runs in pure shell so it never skips on a missing tool.

---

## 5. Definition of Done

- [x] A normative FFI model is specified: capability model (§4.1), `wild` body + elaboration
  (§4.2), execution host (§4.3), ABI protocol (§4.4).
- [x] Guarantee tags assigned per FFI-backed operation category (§4.6): `Declared` baseline,
  `Empirical` only for a differentially-covered deterministic op.
- [x] The `wild` elaboration path is specified: lower to `Op { prim: "wild:…" }` (no new node);
  the three-way differential extends to a `wild`-backed operation.
- [x] The capability model is specified well enough to implement M-720/M-721 (§4.1–4.3).
- [x] The `// SAFETY:` audit protocol is normative (§4.4/§4.7).
- [x] The `std.{io,fs,sys,rand,time}` binding strategy is decided (§4.5).
- [x] Status advances `Draft → Accepted` (this revision), maintainer sign-off recorded (§Meta).

---

## 6. Open questions — resolved

The five Draft-stage open questions are resolved as follows (maintainer sign-off, 2026-06-23):

- **Capability model depth** → build-time `@std-sys` gate; no runtime `Capability<io>` in v0 (§4.1).
- **`wild` execution host** → capability handle = the prim registry, shared across the three paths
  (§4.3).
- **ABI boundary** → encoding deferred to the `@std-sys` author; the host `PrimFn` owns it (§4.4).
- **Guarantee tag for `wild`** → `Declared`, except `Empirical` for a differentially-covered
  deterministic op (§4.6).
- **`just safety-check` scope** → full Mycelium-level audit (SAFETY comment + `@std-sys` + `!{ffi}`)
  *and* the existing Rust-level adjacency check (§4.7).

## 7. Deferred — runtime capability + `xloc` (flagged to avoid retroactive incompatibility)

- **Runtime-enforced capability.** A future RFC may add a `Capability<io>`-style value that a
  program must hold to call a `wild` op, enabling runtime sandboxing (the operator user story). The
  build-time gate decided here is forward-compatible: it admits a *subset* of what a runtime check
  would, so adding the runtime requirement later only *narrows* the admissible set — no program
  this RFC admits becomes ill-formed retroactively, and no new program becomes silently admissible.
- **`xloc` composition (R2).** When a value is translocated to another node, its `wild` host
  operations do not travel with it; the capability model's composition across node boundaries is
  deferred to the R2 RFC. Flagged here so the R2 design treats host capability as node-local.

---

## 8. Grounding / honesty

Grounding basis: DN-14 row 9 (`wild`/FFI gate — was *conditionally present, execution staged*;
this RFC + M-720…M-724 move it to *executes*); ADR-014 (`unsafe` policy — permitted-but-warned,
`// SAFETY:` required); RFC-0016 §8-Q6 (`std-sys` phylum split — the confinement mechanism);
RFC-0004 §2 / ADR-009 (native backend / AOT need for FFI); G2 (no black boxes / never silent — FFI
calls surface their host op and refuse explicitly); VR-5 (no tag upgrade without a checked basis —
`Declared` for v0 FFI, `Empirical` only where the differential covers it); KC-3 (small auditable
kernel — the FFI surface adds **no** new Core-IR node, reusing `Node::Op`); LR-9 (`wild` is the
single permitted language-level FFI escape; all else is refused, not ignored).

Implementation references: `crates/mycelium-l1/src/elab.rs` (the `wild → Op` lowering, M-720);
`crates/mycelium-l1/src/eval.rs` + `crates/mycelium-interp/src/{lib,prims}.rs` (the host dispatch,
M-721); `crates/mycelium-std-sys/src/` (the syscall floor, M-722/M-723);
`crates/mycelium-mlir/src/jit.rs` (the unsafe-confinement reference, DN-21/M-682);
`scripts/checks/safety.sh` (the audit gate, M-724).

---

## Meta — changelog

- **2026-06-23 — Accepted.** Resolved all five Draft open questions (maintainer sign-off): build-time
  `@std-sys` capability gate (no runtime `Capability` value in v0); the prim registry as the
  capability handle / execution host; `wild` lowers to `Op { prim: "wild:…" }` (no new Core-IR node,
  KC-3); `Declared` guarantee baseline with `Empirical` only for a differentially-covered
  deterministic op (VR-5); full Mycelium-level `just safety-check` audit. Runtime-enforced capability
  and `xloc` composition deferred (§7), flagged forward-compatible. Implemented by M-720…M-724.
  (Append-only; supersedes nothing — fills in the Draft stub's deferred decision.)
- **2026-06-23 — Draft created.** Planning stub for the FFI and system interface model. Scope,
  user stories, open questions established. Status: Draft. Task: E14-1 (M-720). No normative
  decision made. (Append-only; VR-5.)
