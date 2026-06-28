# RFC-0028 — FFI and System Interface

| Field | Value |
|---|---|
| **RFC** | 0028 |
| **Status** | **Accepted** (maintainer sign-off, 2026-06-23; §4.4 host-encoding **signed off 2026-06-28** — in-session ratification). DN-40 A1 (CRITICAL), A2 (HIGH), A3 (HIGH) fixes were **COMMISSIONED for implementation** (must land before E14-1) and are now **CLOSED — landed 2026-06-26 (`4456bd3`; A3 `e7e705f`/`3f55eaa`), verified green 2026-06-28 (`hrd`); see the §4.4.4 closure note**. |
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
(§4.2); the execution host (§4.3); the host-encoding validation bridge (§4.4, **pending maintainer
sign-off — appended 2026-06-28, G11**); ABI honesty + the `// SAFETY:` protocol (§4.5); the
syscall binding strategy (§4.6); the guarantee-tag policy (§4.7); the `just safety-check` scope
(§4.8).

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
contact is a `wild` block in a `@std-sys` nodule (§4.8), and `EXPLAIN` over an FFI-backed value
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
  single audited `mycelium-std-sys` phylum (LR-9; §4.6) and are wired into a registry by a host
  registration bridge that must implement the encoding discipline specified in §4.4.

*Chosen over a Rust trampoline baked into the interpreter* because the registry-as-handle is
composable (the host chooses which ops to grant), testable (a deterministic mock host op can be
injected into the differential), and keeps the interpreter free of a hard-coded syscall set.

### 4.4 Host-encoding validation bridge (M-722 / E14-1) — **signed off 2026-06-28 (in-session); DN-40 A1/A2/A3 COMMISSIONED**

> **Status note (append-only; 2026-06-28).** This subsection adds a normative host-encoding
> validation spec for the `wild`/FFI boundary, closing the DN-40 M2 finding and the G11 ratification
> gap. **SIGNED OFF by maintainer 2026-06-28 (in-session).** DN-40 A1/A2/A3 fixes are COMMISSIONED
> for implementation (must land before E14-1). The parent RFC remains **Accepted**. No existing
> decision is modified or superseded — this is an append-only addition to §4 per the
> Blocked-Decisions Ratification Map group G11 ("must-fix before E14-1"). Each point below is
> `Declared` design direction (the architecture) citing `Proven` exhibited gaps (A1/A2/A3 from
> DN-40 §3) that must close before E14-1 ships (DN-40 §8 OQ-1, Ratification Map G11 ⚠ note).
> Guarantee tags are held at the finding's own honest basis throughout (VR-5). `FLAG-ENCODING-SPEC`
> tracks the pending sign-off in `mycelium-std-sys` host-bridge code.
>
> **CLOSURE (2026-06-28, `hrd`):** the COMMISSIONED A1/A2/A3 input-validation fixes are now
> **landed + re-verified green** (see the §4.4.4 closure note — they were already on `dev` from the
> 2026-06-26 DN-40 hardening release). The host-encoding *bridge* spec below (§4.4.1–4.4.3, M-722)
> is the separate, still-`Declared` deliverable; only the A1/A2/A3 gaps are closed by this note.

The original §4.4 ABI boundary clause ("encoding deferred to the `@std-sys` author; the host
`PrimFn` owns the `Value`↔native conversion") was the right initial answer for a staged design.
But DN-40 identifies this deferral as the **hardest, least-specified boundary in the stack** (DN-40
§Feeds / M2 finding), and the Ratification Map (G11) flags it as a **CRITICAL/HIGH must-fix before
E14-1**. This subsection specifies the **three normative obligations** the host-encoding bridge must
satisfy before any production `wild:` op is wired into the registry.

#### 4.4.1 Parse-into-typed — untrusted host bytes enter as a typed Mycelium value

*(`Declared` design mandate; basis: DN-40 §2 P1/P6, §4 architecture, §7 principle — the "parse,
don't validate" maxim; the underlying refinement-type reasoning is `Proven` in type theory.)*

Every native→`Value` decode path in the host bridge **must** be a total, named, never-silent
function of the form:

```rust
fn decode_<op>(native_output: &[u8]) -> Result<Value, EvalError>
```

- **Never trusted raw.** Untrusted host bytes are **never** cast, transmuted, or string-coerced
  into a `Value` directly; they always pass through a recognizer that constructs the typed `Value`
  via `Value::new` / an existing smart constructor (e.g. `ContentHash::parse` for identity-bearing
  results). The `Value`/`Payload`/`ContentHash` `Deserialize` implementations already enforce this
  (`deny_unknown_fields` + re-run `Value::new` — DN-40 §4) and are the reference pattern.
- **The constructor is the sole gate.** Illegal states must be unrepresentable: use newtypes with
  private fields and smart constructors (DN-40 §2 P6) so downstream code cannot hold a `Value`
  that did not pass the recognizer. No `From<RawBytes>` or `unsafe { transmute }` back-doors.
- **Closed-grammar allowlist.** Each decode function accepts exactly the bytes that belong to its
  grammar (the intended language) and returns an explicit `EvalError::BadHostBytes { op, reason }`
  naming the offending input on any failure — never a silent default or a fallback fill (G2/P5).
- **Re-enter the recognizer on deserialization.** Any path that reconstitutes a typed value from
  persisted or transferred bytes (e.g. a host op that caches its result) re-runs the decode
  function — the immutability guarantee depends on true re-validation, not a trusting blob-load
  (DN-40 §5 limit 2).

**Honest caveat:** the architecture is `Declared` (a mandated design direction, not yet
implemented). The `Value::new` / `ContentHash::parse` re-entry discipline is `Empirical` — it is
present in the current code (DN-40 §4) but the **production `wild:` host bridge itself does not
exist yet** (DN-40 M2 finding: only `wild:echo` test fixtures exist as of this writing); the
guarantee tags are therefore `Declared` until a validation corpus runs against a real bridge.

#### 4.4.2 Injective-encode — `Value`→host encoding is injective and escaped

*(`Declared` design mandate; basis: DN-40 §2 P2, §4 canonicalization discipline; reference:
Bernstein netstrings, spec-level `Declared`; the quasi-encoding/injection bug class is `Empirical`
from prior art.)*

Every `Value`→native encode path in the host bridge **must** be injective: two distinct `Value`s
must encode to distinct byte strings, and the encoding must be decodable to the original `Value`
(i.e. `decode(encode(v)) == v` for every `v` in the bridge's domain, and `decode(x)` fails unless
`encode(decode(x)) == x` byte-for-byte).

- **Length-prefix or escape every variable-length field.** No delimiter can be embedded in a
  field; the v1 spore `push_field` discipline (`<len>:<bytes>\n`) and the kernel `Canon` hasher
  (DN-40 §4) are the canonical reference encoders. Any new host-bridge pre-image copies this
  pattern, with a property test asserting round-trip canonicality.
- **Per-field injectivity is necessary but not sufficient.** Injectivity of the encoding faithfully
  commits whatever it is handed — if the input is unvalidated (§4.4.1), a garbage `Value` is
  injectively encoded into garbage bytes. The two obligations (parse-into-typed + injective-encode)
  are independent and both required (DN-40 §4 critical caveat; the A3 finding is the live proof:
  `ContentHash` encodes injectively but the unvalidated dep-hash `String` was its input).
- **The encoding is injective, not an oracle.** The bridge encodes into the host's ABI
  representation (e.g. a C string, a byte buffer, a length-width integer) — it is not expected to
  produce a fully canonicalized universal serialization. The requirement is that the particular
  encoding function has no injection vulnerabilities (no delimiter confusion, no truncation, no
  silent reinterpretation of control bytes). Shell injection, path traversal, and SQL injection are
  the textbook failure modes; the `Value`→native bridge must be verified clean against each
  applicable class.
- **A `// SAFETY:` (or `// ENCODING:`) comment at each encode site** names the ABI contract and
  asserts injectivity, analogous to the Rust `// SAFETY:` requirement at `unsafe` sites (§4.5).

**Honest caveat:** injective-encode is `Declared` for the bridge (unbuilt). The reference encoders
(`push_field`, `Canon`) are model-grade (`Empirical` — present in the codebase) but a new bridge
must be verified independently; no host-bridge encode path is `Empirical` or `Proven` before
adversarial property tests run.

#### 4.4.3 Bounded — length and resource bounds enforced, never-silent on overflow

*(`Declared` design mandate; basis: DN-40 §2 P3, §3 M1 finding: `read_to_end()` + bare `&Path`
pass-through at `mycelium-std-sys/src/io.rs:35-39` and `src/fs.rs:18-50`, severity MEDIUM `Proven`;
the A1/A2 parser-DoS pattern, severity CRITICAL/HIGH `Proven`; G2 never-silent.*)*

Every input-driven allocation or buffer read in the host bridge **must** be capped **before** the
allocation or copy:

- **Read cap before any buffer alloc/copy.** A `wild:read` op reads at most `CAP` bytes (an
  explicit, documented constant per op) into a pre-sized buffer; if the host returns more, the
  bridge returns `EvalError::TooLarge { cap, op }` — never a silent truncation and never an
  unbounded `Vec` growth. The M1 finding (`read_to_end` → unbounded stdin `Vec`) is the cautionary
  anti-pattern (DN-40 §3 M1; MEDIUM `Proven`).
- **Length-check before pointer arithmetic.** A buffer/length/pointer-shaped return value from a
  host call is bounds-checked against an explicit cap before any indexing, copy, or slice
  construction — no `unsafe` pointer arithmetic on untrusted lengths.
- **Path confinement for filesystem ops.** `wild:fs_*` ops pass `&Path` through an explicit root-
  confinement step: canonicalize the path, verify the prefix is in the allowed roots, and reject
  traversal (`./../`) and symlink-escape with a named `EvalError` before the call reaches
  `std::fs` (DN-40 §3 M1 fix direction). No bare `&Path` pass-through is ever the boundary.
- **Never-silent on overflow (G2).** Any overflow, out-of-range, or resource-limit breach is an
  explicit `EvalError` naming the operation and the limit — never a silent wrap, truncation, or
  default fill. This is the parser's own contract (parse.rs:14-20 G2) applied to the host bridge.
- **Resource caps are documented constants**, not magic numbers: named as `BRIDGE_READ_CAP`,
  `BRIDGE_PATH_DEPTH`, etc., with a `// RESOURCE-BOUND:` comment stating the chosen ceiling and
  the reasoning (analogous to `MAX_EXPR_DEPTH` at parse.rs:20).

**Honest caveat:** the bounds discipline is `Declared` (mandated, unimplemented). The M1 finding
(unbounded `read_to_end`, bare path pass-through at named lines in `mycelium-std-sys`) is `Proven`
— the code exists and has no cap. The fix direction above is `Declared`; the chosen cap values are
`Declared` until measured and justified.

#### 4.4.4 How §4.4.1–4.4.3 close the DN-40 A1/A2/A3 input-validation gaps

> **FLAG (CRITICAL/HIGH must-fix before E14-1):** DN-40 A1/A2/A3 are active gaps in the current
> codebase. A1 (type-subgrammar parser DoS, CRITICAL `Proven`) and A2 (pattern-subgrammar parser
> DoS, HIGH `Proven`-by-structure) must be fixed before any FFI work ships, per DN-40 §8 OQ-1 and
> the Ratification Map G11 ⚠ annotation. A3 (dep-hash parse-don't-validate, HIGH `Proven`) must
> close before the encoding bridge is built (the A3 lesson is that injective encoding does not
> substitute for input validation). These are not this RFC's fixes — they live in
> `mycelium-l1/src/parse.rs` and `mycelium-proj/src/manifest.rs` — but E14-1/M-722 is blocked on
> their landing per DN-40 §8 OQ-1.

> **CLOSURE (2026-06-28, `hrd`): A1/A2/A3 LANDED — the FLAG above is now historical.** Verification
> this session found all three fixes already on `dev`, landed *before* this §4.4 sign-off, so the
> must-fix-before-E14-1 sequencing was in fact met (E14-1 and M-722 are `status:done` and landed
> *after* them):
> - **A1 + A2** — the shared recursion-depth guard (`enter_depth`/`leave_depth`, `MAX_EXPR_DEPTH =
>   256`, never-silent `ParseError`; charged at `parse_type_ref` and `parse_pattern` in `parse.rs`),
>   landed `4456bd3` (2026-06-26 DN-40 hardening release). Crash-refused regressions
>   `mycelium-l1/tests/check.rs::deeply_nested_{type_arrow,type_args,ctor_pattern}_is_refused_not_a_crash`
>   re-run green this session. **Tag: the bound is `Proven`-by-construction (every recursive entry
>   charges a finite budget); the `256` limit *value* stays `Declared` — no upgrade without a
>   measured basis (VR-5).**
> - **A3** — `Dependency.hash: Option<ContentHash>` parsed at the manifest boundary (malformed pin →
>   explicit `ManifestError`), landed `e7e705f`/`3f55eaa`; `mycelium-proj` manifest tests re-run
>   green. The §4.4.1 parse-into-typed lesson the FLAG demanded is enforced in code, not merely
>   documented.
>
> Recorded in `CHANGELOG.md` §Security (2026-06-26: DN-40 input-validation hardening — full gap
> ledger). Append-only (house rule #3): the FLAG and the per-gap analysis below are preserved as the
> as-signed-off record; this note supersedes only their *open/active* framing — the host-encoding
> *bridge* spec (§4.4.1–4.4.3, M-722) remains the separate, still-`Declared` deliverable.

The relationship between §4.4 and the three DN-40 priority gaps:

- **A1 (parse.rs type-subgrammar overflow, CRITICAL `Proven`).** The host bridge (§4.4.3) mandates
  bounded recursion / capped input at the `wild`/FFI boundary. But the L1 parser's own never-crash
  contract (parse.rs:14-20) must hold *before* a Mycelium source file can reach a `wild` block —
  and A1 breaks that contract today. The §4.4.3 resource-bound discipline at the bridge is a
  separate, independent gate; it does not substitute for the A1 parser fix.
- **A2 (parse.rs pattern-subgrammar overflow, HIGH `Proven`-by-structure).** Same dependency as
  A1: A2 is the same overflow class on `parse_pattern`, must land with A1, and is a precondition
  for any FFI-backed op to be reached safely by a user-supplied source file.
- **A3 (dep-hash parse-don't-validate, HIGH `Proven`).** The §4.4.1 parse-into-typed mandate
  directly closes the class A3 belongs to — *encoding does not substitute for input validation*.
  The A3 fix (route `Dependency.hash` through `ContentHash::parse` at the manifest boundary,
  `Dependency.hash: ContentHash` not `Option<String>`) must land **before** the host bridge is
  built, so the lesson is not merely documented but enforced. Once A3 is fixed, the bridge's
  parse-into-typed discipline (§4.4.1) extends the same pattern to the foreign boundary.

The §4.4 spec closes **the M2 finding** (DN-40 §3 M2, MEDIUM `Declared`): the "deferred
encoding" clause is replaced by these three normative obligations. The A1/A2/A3 fixes are tracked
separately and are gated blockers for E14-1 per DN-40 §8 OQ-1. **This RFC does not re-open or
re-decide A1/A2/A3 — it notes the sequencing constraint and delegates to their own issues/PRs.**

#### 4.4.5 Guarantee tag posture at the validation boundary

*(`Declared` tag basis; per-finding tags held at DN-40's own honest basis — no upgrade without a
checked basis; VR-5.)*

- **The host-encoding bridge is `Declared`** in v0: the bridge is unbuilt; the three obligations
  above are mandated design directions, not verified properties.
- **The bridge earns `Empirical`** when a validation corpus — adversarial property tests covering
  (1) parse-into-typed round-trip, (2) injective-encode round-trip with delimiter-injection
  adversaries, and (3) bounded-read/path-confinement limits — runs against the real implementation
  and passes. Each property test, when green, constitutes a recorded `Empirical` basis for its
  respective obligation. The corpus passes being the gate mirrors the in-repo three-way differential
  as the basis for `Empirical` on deterministic `wild:` ops (§4.7, formerly §4.6).
- **The bridge is never `Proven`** without a mechanized theorem over the implementation (deferred,
  per §4.7/§4.8's guarantee-tag policy; future work).
- **FFI residual insecurity is `Declared` and disclosed per DN-44 §1.1.** The `wild`/FFI surface
  is an intentional escape hatch — it cannot be made fully transparent at the language level.
  Wherever a host op cannot be fully confined (e.g. a future network call, a platform ABI nuance),
  the disclosure discipline from DN-44 §1.1 applies: the gap is documented co-located with the
  surface, with a justification and practical guidance for the `.myc` program author — never silent
  (G2).

### 4.5 ABI honesty and the `// SAFETY:` protocol (M-720/M-724)

- The `wild` body's argument-to-host encoding is **governed by the three obligations of §4.4**
  (parse-into-typed / injective-encode / bounded — pending maintainer sign-off); the host `PrimFn`
  implements those obligations and is responsible for its ABI contract. v0 does **not** impose a
  canonical value-to-C encoding — the syscall surface (§4.6) is native Rust `std`/`libc`-level,
  not arbitrary C structs.
- **Never-silent ABI claims.** A host `PrimFn` that cannot honour its ascribed result type returns
  an explicit `EvalError` (or the syscall's `Result::Err`), never a silently wrong-typed `Value`
  (G2). The `wild` block's *result* type is the ascribed Mycelium type (M-661); a host op that
  produces something else is a runtime refusal.
- **The `// SAFETY:` protocol.** Every `wild` site in a `@std-sys` `.myc` nodule must carry a
  `// SAFETY:` comment stating the ABI/host contract it relies on, and every Rust `unsafe` block in
  the FFI host layer must carry the ADR-014 `// SAFETY:` justification (`scripts/checks/safety.sh`,
  M-681). `just safety-check` verifies both (§4.8).

### 4.6 Syscall binding strategy (M-722/M-723)

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

> **Clarification (2026-06-25, append-only).** The v0 `std.rand` source is `/dev/urandom` **via
> `std::fs`** only; `getrandom(2)` is **deferred** (`FLAG-GETRANDOM` in `mycelium-std-sys/src/rand.rs`)
> to avoid a workspace dependency and preserve `#![forbid(unsafe_code)]`. The `Declared` tag and
> never-silent behavior are unchanged; only the named mechanism is narrower than the §4.6 table's
> "`getrandom(2)`" implies. Status remains **Accepted** (no decision change).

### 4.7 Guarantee-tag policy (VR-5)

- **`Declared`** is the v0 tag for every syscall-backed operation (the host body is audited, not
  verified — no theorem, no measured bound).
- A specific `wild`-backed operation that is covered by the in-repo three-way differential (a
  **deterministic** host op whose L1-eval ≡ L0-interp ≡ AOT agreement is asserted) earns
  **`Empirical`** for that operation only — the differential is the recorded basis. Non-deterministic
  syscalls (entropy, clock) cannot be covered by an equality differential and stay `Declared`.
- For the host-encoding bridge specifically: the bridge earns `Empirical` per §4.4.5 once a
  validation corpus passes; until then the bridge tag is `Declared`.
- No FFI claim is `Proven`. Promotion requires a checked basis recorded at the site (VR-5).

### 4.8 `just safety-check` scope (M-724)

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
  (§4.2), execution host (§4.3), ABI protocol (§4.5).
- [x] Guarantee tags assigned per FFI-backed operation category (§4.7): `Declared` baseline,
  `Empirical` only for a differentially-covered deterministic op.
- [x] The `wild` elaboration path is specified: lower to `Op { prim: "wild:…" }` (no new node);
  the three-way differential extends to a `wild`-backed operation.
- [x] The capability model is specified well enough to implement M-720/M-721 (§4.1–4.3).
- [x] The `// SAFETY:` audit protocol is normative (§4.5/§4.8).
- [x] The `std.{io,fs,sys,rand,time}` binding strategy is decided (§4.6).
- [x] Status advances `Draft → Accepted` (this revision), maintainer sign-off recorded (§Meta).
- [x] **Host-encoding validation bridge spec (§4.4) — signed off by maintainer 2026-06-28 (in-session).**
  Three normative obligations specified: parse-into-typed (§4.4.1), injective-encode (§4.4.2),
  bounded (§4.4.3). DN-40 A1/A2/A3 closure sequencing noted (§4.4.4). Guarantee tag posture stated
  (§4.4.5). This item is open until the maintainer ratifies §4.4 — at which point the DoD row is
  checked and a §Meta changelog entry is appended.

---

## 6. Open questions — resolved

The five Draft-stage open questions are resolved as follows (maintainer sign-off, 2026-06-23):

- **Capability model depth** → build-time `@std-sys` gate; no runtime `Capability<io>` in v0 (§4.1).
- **`wild` execution host** → capability handle = the prim registry, shared across the three paths
  (§4.3).
- **ABI boundary** → encoding initially deferred to the `@std-sys` author; the host `PrimFn` owns
  it (§4.5). The encoding discipline is **now further specified** in §4.4 (parse-into-typed /
  injective-encode / bounded — pending maintainer sign-off; G11 must-fix before E14-1).
- **Guarantee tag for `wild`** → `Declared`, except `Empirical` for a differentially-covered
  deterministic op (§4.7); bridge earns `Empirical` after validation corpus passes (§4.4.5).
- **`just safety-check` scope** → full Mycelium-level audit (SAFETY comment + `@std-sys` + `!{ffi}`)
  *and* the existing Rust-level adjacency check (§4.8).

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

For §4.4 (host-encoding validation bridge, 2026-06-28): DN-40 (input-validation architecture —
`Accepted` 2026-06-26; the M2 finding's fix mandate and A1/A2/A3 sequencing constraint; §4 "only-
intended-inputs" architecture; §7 generalizable principle; §8 OQ-1); DN-44 §1.1 (residual-insecurity
disclosure discipline — the honesty corollary; `Proposed`); Blocked-Decisions Ratification Map §G11
(axis: "host-encoding validation bridge, A1/A2/A3 input-validation gaps"; must-fix-before-E14-1 ⚠);
DN-39 §5 (spore `content_address` v0→v1 injective encoding, the canonical reference pattern —
`FIXED` PR #617); G2 (never-silent — `EvalError::BadHostBytes`, `EvalError::TooLarge`, path-
confinement errors); VR-5 (bridge is `Declared` until validation corpus passes; residual insecurity
`Declared` + disclosed per DN-44 §1.1); KC-3 (no new bridge abstraction beyond what §4.3 already
specifies — the bridge is the `PrimFn` registration host, §4.3 plus the three obligations of §4.4);
E14-1 / M-722 as the consuming epic/task.

Implementation references: `crates/mycelium-l1/src/elab.rs` (the `wild → Op` lowering, M-720);
`crates/mycelium-l1/src/eval.rs` + `crates/mycelium-interp/src/{lib,prims}.rs` (the host dispatch,
M-721); `crates/mycelium-std-sys/src/` (the syscall floor, M-722/M-723);
`crates/mycelium-mlir/src/jit.rs` (the unsafe-confinement reference, DN-21/M-682);
`scripts/checks/safety.sh` (the audit gate, M-724).

---

## Meta — changelog

- **2026-06-28 — DN-40 A1/A2/A3 CLOSED (drift closure; `hrd`).** The A1/A2/A3 fixes COMMISSIONED in the entry below were verified already-landed on `dev` from the 2026-06-26 DN-40 hardening release (A1/A2 shared parser depth-guard `MAX_EXPR_DEPTH = 256` / never-silent `ParseError`, `4456bd3`; A3 typed `Dependency.hash: Option<ContentHash>`, `e7e705f`/`3f55eaa`). Crash-refused L1 depth tests + `mycelium-proj` manifest tests re-run green this session. Reconciled the status row, §4.4 status-note, and §4.4.4 (closure note) — append-only; the commissioning entries are preserved. The host-encoding *bridge* (§4.4.1–4.4.3, M-722) stays the separate, still-`Declared` deliverable. Bound `Proven`-by-construction; `256` value `Declared` (VR-5). (Append-only; house rule #3; VR-5.)
- **2026-06-28 — §4.4 signed off; DN-40 A1/A2/A3 COMMISSIONED (in-session ratification).** §4.4 host-encoding validation bridge spec is accepted; DN-40 A1 (CRITICAL: type-subgrammar parser DoS), A2 (HIGH: pattern-subgrammar parser DoS), A3 (HIGH: dep-hash parse-don't-validate gap) are COMMISSIONED for implementation — must land before E14-1. (Append-only; house rule #3; VR-5.)
- **2026-06-28 — §4.4 host-encoding validation bridge added (append-only; pending maintainer
  sign-off; G11 must-fix before E14-1).** Adds RFC-0028 §4.4 specifying the three normative
  obligations for the `wild`/FFI boundary: (1) §4.4.1 parse-into-typed — untrusted host bytes
  enter via a total named `decode_<op>` function producing a typed `Value`, never trusted raw;
  (2) §4.4.2 injective-encode — every `Value`→native encode path is injective/length-prefixed,
  with a `// ENCODING:` justification comment at the site; (3) §4.4.3 bounded — every
  input-driven alloc/read is capped before the alloc, path-confined, never-silent on overflow
  (`EvalError::TooLarge`/`EvalError::BadHostBytes`). §4.4.4 traces the closure of the DN-40 M2
  finding and the **CRITICAL/HIGH sequencing constraint** (A1 parser DoS, A2 pattern DoS, A3
  dep-hash parse-don't-validate must land **before** E14-1). §4.4.5 states the honest guarantee
  tag posture: bridge is `Declared` until a validation corpus passes → `Empirical`; FFI residual
  insecurity `Declared` + disclosed per DN-44 §1.1; never `Proven` without a mechanized theorem.
  The existing §4.4–4.7 renumbered to §4.5–4.8 (scope table, DoD, §6, §8 updated accordingly).
  The parent RFC remains **Accepted**; §4.4 is **pending maintainer sign-off** (flagged in scope
  table, DoD, and §4.4 opening note). No existing decision modified or superseded. Grounded in:
  DN-40 (Accepted), DN-44 §1.1 (Proposed), Ratification Map §G11, DN-39 §5, G2, VR-5, KC-3.
  (Append-only; house rule #3.)
- **2026-06-25 — §4.6 clarification (append-only; no status move; was §4.5 prior to
  2026-06-28 renumbering).** Per an alignment audit, noted that v0 `std.rand` entropy is
  `/dev/urandom` via `std::fs` only; `getrandom(2)` is deferred (`FLAG-GETRANDOM`,
  `mycelium-std-sys/src/rand.rs`) to avoid a workspace dep and keep `#![forbid(unsafe_code)]`.
  The `Declared` tag and never-silent behavior are unchanged; only the named mechanism is narrower
  than the §4.6 table implies. Status remains **Accepted**. (Append-only; VR-5; G2.)
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
