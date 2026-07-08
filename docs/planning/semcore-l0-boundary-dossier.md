# semcore L0 boundary — decision dossier (M-1012 / M-1013)

| Field | Value |
|---|---|
| **Status** | **Non-normative `Declared` decision-support.** This is analysis + a recommendation for the maintainer to decide; it is **not** a ratified decision and moves no issue status. It records no `Accepted`/`Enacted` transition (house rule #3). The recorded decision, when made, appends to **DN-26** (per M-1012's Definition of Done). |
| **Question** | How to represent the kernel L0 `Value`/`Repr`/`FieldSpec` types in the self-hosted `lib/compiler/semcore.myc` elaborator — the `[FLAG]` that gates M-1012 (`needs-design`) and, through it, the elab core and eval legs of M-1013 (`needs-design`). |
| **Feeds** | M-1012 (increment 7; the boundary pick + pure lowering helpers), M-1013 (increments 8..14, the heavy sequential core), DN-26 §7.1/§7.2/§9, DN-14 row 9. |
| **Grounded against** | `crates/mycelium-l1/src/{elab,checkty,mono,eval,fuse}.rs` (measured spans, this session), `crates/mycelium-core/src/{value,repr,data,node}.rs` (kernel type defs), `lib/compiler/semcore.myc` (2541-line partial port on `dev`, FLAG-semcore-1..20). |

> **Transparency (VR-5).** Every LOC figure below carries its source: numbers tagged **measured**
> come from `wc -l` / grepped function spans in this session over the `origin/dev` tree; numbers
> tagged **Declared** are the estimates copied from M-1013's body. Every feasibility/effort judgment
> is **`Declared`** unless it cites a landed precedent, in which case it is marked **`Empirical`**
> (the mirror pattern already runs, live-oracle-differentialed, for increments 1–6). The A-vs-B
> recommendation is **`Declared`** with an `Empirical` supporting precedent; its strongest
> counter-argument is stated in §5, not buried. Where the corpus did not let me settle a mechanism
> (the precise `wild` return-type story for structured non-`Value` kernel objects), I say so plainly
> rather than guess (G2).

---

## 1. The question, and why it must be decided before the elab core and eval

M-1012 ports `elab.rs`'s **pure lowering helpers** — `lit_value`, `type_repr`, `field_spec`,
`ty_to_repr`, `ty_to_field_ty_ref`, `scalar_kind`, `sparsity_class`, `policy_name_ref`. Seven of the
eight **construct kernel L0 types**: `mycelium_core::{Value, Repr, FieldSpec, FieldTyRef, PolicyRef}`
(`crates/mycelium-l1/src/elab.rs:102,244,344,923,982,1023`). This is the **first** wave to touch the
frontend/kernel seam DN-26 §7.2 names — every prior increment (1–6, M-1007..M-1011) ported only
`Ty`-algebra and `Bytes`-string transformers that never leave the frontend's own vocabulary.

The decision cannot be deferred past increment 7 because the elaborator's **output** is a closed L0
`Node` (`crates/mycelium-core/src/node.rs:46`), and `Node` **embeds** these kernel types directly:
`Const(Value)`, `Swap { target: Repr, policy: PolicyRef, .. }`, `Construct { ctor: CtorRef, .. }`,
and the data registry that `build_registry` produces is `FieldSpec`-valued. So:

1. **The elab core (M-1013 increment 10) diffs against this model.** `elaborate` produces the L0
   `Node`; the Stage-5 gate (DN-26 §7.3 row 5) is the **L0-output differential** Rust-host ≡ self-host
   over the conformance corpus. Whatever shape the self-hosted elaborator gives `Value`/`Repr`/
   `FieldSpec` **is** the thing the differential compares. Choosing it after the elab core is written
   means rewriting the core.
2. **The eval leg (increment 12) reads and builds the same L0.** `eval.rs`'s `L1Value` and its
   `to_core` path materialize kernel `Value`s; `Swap` targets are `Repr`. The evaluator and the
   elaborator must agree on the L0 model bit-for-bit or the NFR-7 differential is meaningless — the
   Rust source already shares one bridge (`elab::lit_value`/`type_repr`) between both paths precisely
   so they "cannot drift on the basics" (`elab.rs:10-12`).

So M-1012 is correctly `needs-design`: it fixes the L0 `Node` model that increments 10 and 12 both
build against. Deciding it once, here, is cheaper than re-litigating it per wave.

---

## 2. Option A — the in-language mirror model

Declare the kernel L0 vocabulary as plain Mycelium ADTs in `semcore.myc`, mirroring
`mycelium_core` field-for-field, and have the pure helpers construct **those**. The differential
compares the mirror's canonical rendering against the Rust oracle's.

### 2.1 What the mirror types look like

Concretely (prefix scheme per FLAG-semcore-2, disjoint from the existing `Ty`/`Wd`/`Mp`/`Hd`
families):

```
type ScalarK = SkF16 | SkBf16 | SkF32 | SkF64;                 // mycelium_core::ScalarKind
type SparsityC = ScDense | ScSparse(Binary{32});              // SparsityClass (max_active passthrough)
type Repr =
    RBinary(Binary{32})            // Repr::Binary { width }
  | RTernary(Binary{32})           // Repr::Ternary { trits }
  | RDense(Binary{32}, ScalarK)    // Repr::Dense { dim, dtype }
  | RVsa(Bytes, Binary{32}, SparsityC)
  | RSeq(Repr, Binary{32})         // Repr::Seq { elem, len }
  | RFloat                         // Repr::Float { width: F64 } — nullary; FloatWidth is F64-only (ADR-040 FLAG-1)
  | RBytes;
type Payload =
    PBits(Vec[Binary{1}]) | PTrits(Vec[Trit]) | PScalars(Vec[Float])
  | PHyper(Vec[Float]) | PFloat(Float) | PSeq(Vec[Value]) | PBytes(Vec[Binary{8}]);
type Value = V(Repr, Payload, Meta);                          // the (repr, payload, meta) triple
type FieldTyRef = FtRepr(Repr) | FtData(Bytes) | FtFn(FnSig);
type FieldSpec  = FsRepr(Repr) | FsData(Bytes) | FsFn(Binary{32}, FnSig);
```

This is **exactly the move FLAG-semcore-1/-2 already made** for the frontend's checked-type
vocabulary — `semcore.myc:390-421` declares `type Ty = TyBinary(Width) | …` and `type DataInfo =
DI(…)` as verbatim in-language mirrors of `checkty.rs`'s `Ty`/`DataInfo`. Option A extends that same
mirroring **one layer down**, from the frontend's `Ty` to the kernel's `Value`/`Repr`. The helpers
then read as direct transcriptions of the Rust: `type_repr`'s `BaseType::Binary(WidthRef::Lit(n)) =>
Repr::Binary { width: n }` becomes `TBinLit(n) => RBinary(n)`, and so on.

### 2.2 How the differential stays honest

M-1012's Definition of Done asks for a "live-oracle differential **where a value can be materialised
for comparison**" (`Empirical`). The mirror is materialisable two honest ways, both consistent with
the FLAG-semcore-10 live-oracle discipline (compare the `.myc` verdict against the **real Rust
oracle fn's own computed result**, never a hand-typed literal):

1. **Canonical-serialization equality.** `mycelium_core::Value` already has a total serde wire form
   (`value.schema.json`; `value.rs:100-157` externally-tagged `PayloadWire`), and the kernel is
   content-addressed (`operation_hash`, `content.rs:585`; `Node::content_hash`). The harness renders
   both the Rust `Repr`/`Value`/`FieldSpec` and the mirror value to the **same** canonical byte form
   (JSON or content-hash) and asserts equality. This is the cleanest witness kind — the same
   `bytes_eq` posture increment 4's mangling differential already uses (`semcore.myc` M-1009 notes).
2. **Harness-side marshalling.** A small Rust `fn` in the in-crate test module maps the mirror ADT
   to the real kernel type and compares with derived `==`. More code than (1), but it makes the
   structural correspondence explicit and catches a mis-encoded tag directly.

Either way the differential is a **true** Rust ≡ self comparison: the oracle produces a real kernel
value, the port produces a mirror value, and a canonical projection of each is asserted equal.

### 2.3 Maintenance cost (stated plainly)

The mirror is a **hand-maintained copy**. When `mycelium_core::Repr` grows a variant, the mirror
must grow in lockstep or the self-host silently lags. This is **not hypothetical**: `Repr` has grown
repeatedly — `Seq` and `Bytes` (RFC-0032 D3/D4, M-749/M-750) and `Float` (ADR-040, M-896) were all
added after v0, and each would have forced a mirror edit. That is a real, recurring tax for the life
of the self-hosted frontend, and it is the honest cost of Option A.

Three things bound it, none of which make it zero:

1. **It is the pattern already in force.** FLAG-semcore-1 already accepts and documents exactly this
   drift for the AST/`Ty` vocabulary ("if a later increment needs a dropped piece, re-copy it from
   `ast.myc`"). Option A adds no *new kind* of cost — it widens an accepted one.
2. **Drift is never silent (G2).** A program that uses a new kernel variant produces a divergent L0,
   so the L0-output differential **fails** the moment the mirror lags. The tax is a red test, not a
   wrong answer.
3. **The kernel types are append-only, frozen-tag.** `ScalarKind::tag`/`FloatWidth::tag` are frozen
   ("existing codes are frozen so a definition's identity never shifts"; `repr.rs:50-85`), so the
   mirror's encoding is a stable, monotone target — additions only, never a re-numbering. (The flip
   side: the mirror must reproduce the **frozen tag values** to keep content-addressing identical, a
   real constraint the encoder has to get right and the differential has to check.)

### 2.4 Composition with the FLAG-semcore-4/-11 conventions

`build_registry` (increment 10) turns the checked `Env` into a `DataRegistry` of `FieldSpec`s. In
`semcore.myc` the type registry is already the FLAG-semcore-4 ordered `Vec[Pair[Bytes, _]]` assoc
list (`types_lookup`, `semcore.myc:429`); a `FieldSpec`-valued registry slots into the same
convention with no new machinery. The mirror `Value`/`Repr` are plain value-semantics ADTs — they
thread through the existing FLAG-semcore-6 value-threaded style (no interior mutability) exactly as
`Ty` does today. Option A is the low-friction fit with what has already landed.

---

## 3. Option B — the `@std-sys` + `wild` FFI seam

Reach the **real** kernel constructors through the `@std-sys` + `wild` seam DN-14 row 9 confirms now
*executes* (RFC-0028; M-720/M-721). Instead of a mirror `type Repr`, `semcore.myc` would call a host
op — conceptually `wild { core_repr_binary(width) }` — that invokes `mycelium_core`'s constructor and
returns a real `Repr`. No mirror to maintain; one source of truth for the L0 types.

### 3.1 What the FFI surface would be

DN-14 row 9 (`DN-14-Self-Hosting-Gate.md:87`) fixes the mechanism precisely: a `wild` body in an
`@std-sys` nodule with `!{ffi}` declared lowers via `elab.rs`'s `elab_wild` to `Op { prim:
"wild:<name>" }` — the reserved host-dispatch namespace, **no new Core-IR node** — and the prim
registry dispatches it at eval/interp/AOT time. So Option B's surface is a set of `wild:`-namespaced
host prims: `wild:core_repr_binary`, `wild:core_value_new`, `wild:core_field_spec_repr`, etc., the
self-hosted elaborator's driver nodule being `@std-sys` and each construction site declaring
`!{ffi}`.

### 3.2 What "executes" buys

1. **No mirror to maintain.** The kernel types are used directly; a new `Repr` variant needs a new
   `wild:` prim but no parallel ADT to keep in sync. The §2.3 drift tax is paid once (add a prim),
   not forever (add a variant to a hand-copy).
2. **Real kernel types, no second source of truth.** The elaborator produces genuine
   `mycelium_core::Value`/`Repr` — the exact objects the kernel evaluates — so there is no
   mirror-vs-real correspondence to prove.

### 3.3 What it costs (stated plainly — this is the load-bearing part)

1. **The `wild`/prim seam is `Value → Value` by construction; `Repr`/`FieldSpec` are not `Value`s.**
   Row 9's mechanism dispatches `Op { prim, args }` over L0 **`Node`/`Value`** operands and results —
   the prim registry is the paradigm-op surface (`crates/mycelium-core/src/prim.rs`), whose currency
   is `Value`. But `Repr`, `FieldSpec`, `FieldTyRef`, `PolicyRef` are **descriptors/components of** a
   `Value`/`Node`, not `Value`s themselves. A `wild:` prim cannot naturally *return a bare `Repr`* as
   an in-language value, because the in-language value world at that seam is L0 `Value`s. To carry a
   `Repr` across the seam you must either (a) encode it **as** some `Value` — which is a mirror
   encoding again, just an opaque one living in the `Value` domain — or (b) smuggle it as an opaque
   host handle (a `Substrate`-typed resource). I could **not** find in the corpus a `wild` return-type
   story for structured non-`Value` kernel objects; DN-14 row 9's worked example is a **deterministic
   scalar** host op, not a descriptor factory. **This is an unresolved mechanism, flagged (G2), not a
   solved one** — and it is the crux of why Option B is not the clean win it first looks like.
2. **Opaque handles defeat the point of self-hosting.** If `Repr`/`Value` cross the seam as opaque
   `Substrate` handles, the elaborator's **own output** — the L0 program it produces — becomes a bag
   of un-inspectable host handles. That collides head-on with house rule #2 (no black boxes: the L0
   must be reified, inspectable, `EXPLAIN`-able) and with the **entire value proposition of the
   self-hosted frontend**: that the frontend is auditable *in the language it compiles* (DN-26 §1).
   A frontend whose output you can only read by asking the Rust kernel is not a self-hosted frontend
   in the sense the port is chasing.
3. **The differential weakens toward circularity.** The Stage-5 gate proves *self-host ≡ Rust-host*
   over the L0 output. If the self-host's L0 is real-kernel handles minted by the same
   `mycelium_core` constructors the Rust host uses, the comparison is partly comparing the kernel to
   itself — the divergence the differential is meant to catch (a mis-lowering in the ported
   elaborator) is exactly the part that gets hidden behind the shared constructor. Option A's mirror,
   compared by serialization, keeps the two productions genuinely independent.
4. **`wild` bodies are trusted/opaque, audited-not-verified.** Row 9 is explicit: the `wild` body is
   "NOT recursively type-checked (audited, not verified — VR-5/ADR-014)". Routing every L0
   construction through `wild` moves the elaborator's most identity-critical step (what `Value` did
   we build?) into the un-verified trust zone, and expands the `just safety-check` audit surface
   (`@std-sys` + `!{ffi}` + `// SAFETY:` per site) across the whole lowering.
5. **It couples the self-hosted elaborator to the kernel ABI.** Every constructor signature becomes a
   `wild:` prim contract; a kernel refactor is now an FFI-contract change, not a local edit.

---

## 4. The arithmetic-free subset that lands under **either** choice

Two helpers are pure enum→enum maps with **no** kernel-value construction and **no** arithmetic, so
they are boundary-independent and can land first regardless of the A/B pick:

1. **`scalar_kind`** (`elab.rs:1043`) — `Scalar → ScalarKind`, four arms, total. Its tag-rendering
   twin `scalar_tag` (`mono.rs::scalar_tag`) **already landed** in increment 4 as `fn scalar_tag(s:
   Scalar) => Bytes` (`semcore.myc:2063`), so the mapping's shape is already present in the nodule.
2. **`sparsity_class`** (`elab.rs:1053`) — `Sparsity → SparsityClass`; the `Sparse(k)` arm is a bare
   `max_active` passthrough (a `Binary{32}` copy, no arithmetic). Its tag twin `vsa_sp_tag` also
   **already landed** in increment 4 (`semcore.myc:2067`).

Under Option A these produce the small mirror `ScalarK`/`SparsityC` enums (§2.1); under Option B they
are still trivial (no `Value` is constructed, so no FFI is needed either way). **Land these two
first** — they de-risk increment 7 without prejudging the boundary.

**One near-neighbor that is *not* in the free set — flag it:** `policy_name_ref` (`elab.rs:344`)
produces a `PolicyRef` via `operation_hash(&format!("policy-name.v0:{…}"))` — a **content hash**, not
an enum map. It needs a hash primitive: either an in-language re-implementation of the
domain-separated `operation_hash` (BLAKE3-family; must match the kernel byte-for-byte or every
`PolicyRef` diverges) or a `wild:` call to the kernel hash. That is a **mini instance of the same A/B
question** (re-implement vs FFI), narrower and hash-specific. It should be decided alongside the main
boundary, not silently folded into "pure helpers".

---

## 5. Recommendation

**Adopt Option A — the in-language mirror model — with the `wild` seam reserved for the *execution /
materialization* boundary only (handing a finished L0 program to the kernel to run), never for
per-descriptor construction.** Decide `policy_name_ref`'s hash the same way: re-implement the
domain-separated hash in-language if it is cheap to match byte-for-byte, else a single `wild:` hash
prim — but keep the `PolicyRef` a mirror value everywhere else.

**Confidence: `Declared`, with an `Empirical` supporting precedent.** It is a design judgment, not a
proved result — but it is not ungrounded: the mirror pattern is **already running** for the frontend
vocabulary (increments 1–6, live-oracle-differentialed and green on `dev`), so "a mirror composes
with this nodule and stays honestly diffable" is `Empirical` for the layer above, and the argument
extends it one layer down.

Grounds:

1. **It is the established, landed pattern (DRY/KISS/KC-3).** `semcore.myc` already mirrors
   `checkty.rs`'s `Ty`/`DataInfo` as in-language ADTs (FLAG-semcore-1/-2). Mirroring
   `mycelium_core`'s `Value`/`Repr` is the identical move; Option A introduces no new mechanism.
2. **It keeps the elaborator's output inspectable in-language (house rule #2).** The produced L0 is a
   plain Mycelium value tree, `EXPLAIN`-able and auditable in the language it compiles — the whole
   reason to self-host. Option B's handle path forfeits this.
3. **It keeps the Stage-5 differential a genuine independent comparison.** Serialization/content-hash
   equality between an independently-built mirror and the Rust oracle is a real Rust ≡ self witness;
   Option B risks comparing the kernel to itself (§3.3.3).
4. **The seam it needs is the one that actually exists.** DN-14 row 9's `wild` is `Value`-currency
   and its worked example is a scalar op; using it for a **single** "materialize this finished L0"
   crossing is squarely within what "executes" delivers, whereas using it as a pervasive descriptor
   factory relies on an **unconfirmed** return-type story for structured non-`Value` objects (§3.3.1).

**Strongest argument against my pick (stated, not softened):** the **drift/duplication tax** (§2.3).
Option A hand-maintains a copy of a kernel type-set that has demonstrably grown (`Seq`/`Bytes`/`Float`
all post-v0), so every future kernel-type addition is a standing obligation to edit the mirror or
watch the self-host silently lag — precisely the duplication KC-3/DRY exists to discourage. Option B
pays that cost once by linking the real types. My rebuttal is that the tax is **bounded**
(append-only, frozen-tag, rare), **never-silent** (the differential fails on drift), and **already
the accepted posture** for the frontend vocabulary — but it is a real, recurring cost, and if the
kernel L0 vocabulary were expected to churn heavily, the balance would tip toward B. It is not
expected to (the L0 calculus is deliberately small and frozen, KC-3), which is why I still land on A.

**A note on a hybrid the maintainer may prefer:** mirror the descriptor types in-language (A), and
add exactly **one** `wild:` prim at the very end that takes the serialized mirror `Node` (a `Bytes`
wire form) and asks the kernel to deserialize it into a real `mycelium_core::Node` for
execution/AOT. This is fully consistent with DN-26 §7.2 ("the ported `.myc` frontend calls back into
`mycelium-core`/`interp` … through the `@std-sys` + `wild` seam") — the callback is **at the seam**
(hand the L0 over once), not smeared across every descriptor construction. I regard this hybrid as
the concrete form of the recommendation, not a third option.

---

## 6. M-1013 sub-wave decomposition (increments 8..14)

### 6.1 Measured Rust line counts vs the issue's `Declared` estimates

Spans are `origin/dev` this session (grepped top-level item boundaries; the impl blocks dominate).
"Core (measured)" excludes the pure leaves already landed in increments 1–7.

| Inc | Unit | Dominant Rust span (measured) | Core LOC (measured) | Issue `Declared` | Divergence |
|---|---|---|---|---|---|
| 8 | checkty registration + resolution + `Env` | `register_*`/`resolve_imports`/`Env` named-fns ≈ 559; `check_nodule_with` driver 345, effect-coverage 161, impl-method checks 169, object desugaring 197, model structs ≈ 230 | **~0.56k named / ~1.9k compilable unit** | **~1.5k** | **Under-labeled by composition** — the `register_*` functions are only ~560 lines; the bulk of "registration" is the `check_nodule_with` driver + desugaring. |
| 9 | checkty bidirectional checker (`Cx`) | `impl Cx<'_>` **3035** (`checkty.rs:3599→6633`) + `struct Cx` 52 + `check_fn_body` 49 + `check_bounds` 52 + `cons_list_ctors` 34 | **~3.22k** | **~3.5k** | Declared slightly **high** (safe over-estimate). |
| 10 | elab core (AST→L0) | `impl Elab<'_>` **1132** (`elab.rs:1082→2214`) + `elaborate*` 258 + `recursive_sccs` 110 + `build_registry` 51 + prelude/calls 100 | **~1.7–1.9k** | **~2k** | Match (Declared slightly high). |
| 11 | mono core (mono + defunctionalization) | `struct Mono`+`impl Mono` region **~2124** (`mono.rs:483→2607`) + entry fns 304 | **~2.43k** | **~2.5k** | **Match.** |
| 12 | eval (L1 evaluator) | `struct/impl Evaluator` **1539** (`eval.rs:556→2095`) + `L1Value` 164 + `Ctrl`/`Regs`/`Frame` 167 + errors/opts 200 | **~2.07k** | **~2k** | **Match.** |
| 13 | fuse (`check_fuse_laws`) | whole file **292** (`fuse.rs`) | **292** | **~292** | **Exact.** |
| 14 | whole-program L0 differential + mutants | Rust-side harness (`crates/mycelium-l1/tests/`) + `cargo-mutants` on the SCC | n/a (test infra) | n/a | — |

**Aggregate divergence, stated honestly.** The increment labels sum to ≈ **11.8k Declared**; the
measured remaining across the five files (`checkty` 7356 + `elab` 2294 + `mono` 3219 + `eval` 2263 +
`fuse` 292 = **15,424 total**, minus ≈ **1.6k** already-landed pure leaves) is ≈ **13.8k**. The ~2k
gap is almost entirely in **increment 8**: the "~1.5k registration" label captures the `register_*`
family but not the `check_nodule_with` driver, object/lower-decl desugaring, and effect-coverage
machinery that must come with it for the nodule to compile. Everything else (9–13) is within ~10% and
generally slightly generous — safe. This matches DN-26 §7.1's own "~16.7k Rust lines" for the full
nine-file SCC once the four already-ported leaf files (`usefulness`+`decision`+`grade`+`affine` ≈
1.3k) are added back.

### 6.2 Dependency order, and which waves are pure vs pull in the deferred `infer` SCC

```
8 registration ─┐
                ├─→ 9 checker (Cx) ─┬─→ 10 elab core ─┬─→ 11 mono core
resolve_ty(M-1008)                  │  (needs M-1012)  │
                                    └───────┬──────────┘
                                            └─→ 12 eval ─→ 13 fuse ─→ 14 differential
```

- **Pure / closest-to-leaf: increment 8 (registration).** Building the `Env`/registries from the
  checked AST is mechanical (assoc-list construction, FLAG-semcore-4). It references the checker for
  signature-resolution (`check_sig_resolves`), so it is not *fully* leaf, but it is the most
  self-contained heavy wave and the natural first landing.
- **Pull in the deferred `infer` SCC (NOT leaf): increments 9, 10, 11, 12.** Increment 9 **is** the
  `infer` core — FLAG-semcore-20 deferred `infer_type` precisely because it "constructs a full
  [checker context]" and is "a thin wrapper over the deferred core", so M-1011's `infer_type` residual
  lands **with** increment 9, not before. `checkty ↔ elab` is a true SCC cycle (DN-26 §7.1), so 9 and
  10 are mutually recursive and land as a tight cluster; 11 needs 10 (plus the already-landed M-1009
  mangling + M-1010 free-vars); 12 needs 10 and the §5 boundary. These four are the entangled bulk that
  can only be witnessed by the **whole-program** L0 differential (increment 14), never as isolated
  leaves.
- **Also deferred and only unblocked here:** FLAG-semcore-17's `mangle_ty_in_ty` and `item_key`
  (`mono.rs:3095,2854`) land with increment 11 (they need mono's `Item` work-item type); the
  `infer_type` residual (FLAG-semcore-20) lands with increment 9.

### 6.3 Proposed PR-sized split (DN-65 ≈1–2k soft rule) — several increments must sub-split

The soft ≈1–2k-LOC-delta rule means the biggest increments **cannot** land as one PR. Proposed
fan (each its own `/pr-review`, sequential where they share `semcore.myc` regions):

1. **Increment 7 (M-1012), split by the boundary decision:**
   1a. `scalar_kind` + `sparsity_class` (the §4 free set) — small, lands first, boundary-independent.
   1b. the L0 mirror types + `lit_value`/`type_repr`/`field_spec`/`ty_to_repr`/`ty_to_field_ty_ref`
   under the chosen boundary — ~1 PR (the helpers are small; the mirror ADTs are declarations).
   1c. `policy_name_ref` + its hash decision — small, can ride with 1b.
2. **Increment 8** → **1–2 PRs.** Tight register-family (~0.8–1k) as one; if `check_nodule_with` +
   object/lower desugaring come along (~+1k), split them into a second.
3. **Increment 9 (~3.2k) → MUST sub-split into ~2–3 PRs.** Natural seams inside `impl Cx`: (a)
   synthesis (`infer`/one-sided `check` per `Expr` family + `lit_ty_of`/`infer_type` residual), (b)
   checking-against-expected + `check_bounds`/`check_fn_body`, (c) effect-coverage + totality
   integration + the swap/`forage`/prim-call arms.
4. **Increment 10 (~1.8k) → 1–2 PRs.** Seam: the recursion/SCC/`FixGroup` machinery (`recursive_sccs`
   Tarjan + `elab_prelude` + `build_registry`) vs the `Expr`→`Node` lowering body (`impl Elab`), the
   latter being the leg that consumes the §5 boundary.
5. **Increment 11 (~2.4k) → 2 PRs.** Seam: the monomorphization walk (generic specialization +
   trait-method resolution) vs the defunctionalization/closure-conversion engine
   (`ClosureSum`/`Ctor`/`Specialization`).
6. **Increment 12 (~2.0k) → 1–2 PRs, feasibility-gated (see §7).** Seam: the `L1Value` model +
   `Frame`/`Ctrl`/`Regs` vs the eval loop + `to_core`.
7. **Increment 13 (292) → 1 small PR**, but gated on 12 (it *runs* the ported evaluator).
8. **Increment 14 → 1 PR** (Rust-side differential harness + `cargo-mutants` witness; the DN-26 §7.3
   row-5 gate proper).

Estimate: **~11–13 scoped PRs** for M-1013. The ones that **must** sub-split when worked (over the
soft rule): **9** (definitely, ~3.2k), **11** (~2.4k), and **8/10/12** (borderline — split if the
driver/loop legs pull them over ~2k). 13 and 14 are single PRs.

---

## 7. Open questions / FLAGs for the maintainer

1. **[HEADLINE] The A/B boundary pick (M-1012).** Decide the L0 `Value`/`Repr`/`FieldSpec`
   representation and **append it to DN-26** (M-1012 DoD). This dossier recommends **A (mirror) with
   `wild` reserved for the single materialization crossing**; the strongest case for B is the
   drift tax (§2.3/§5). Nothing downstream (increments 10, 12) should be written until this lands.
2. **The `wild` return-type story for structured non-`Value` kernel objects is unresolved (G2).** I
   could not confirm from the corpus how a `wild:` prim would return a bare `Repr`/`FieldSpec` (the
   seam is `Value`-currency; §3.3.1). If the maintainer leans toward B, this mechanism needs a
   concrete answer first — it may be the deciding constraint.
3. **`policy_name_ref`'s hash (§4).** Re-implement the domain-separated `operation_hash` in-language
   (must match the kernel byte-for-byte) or a single `wild:` hash prim? A narrow instance of the same
   A/B question; decide it with #1.
4. **The eval feasibility path (M-1013's own `needs-design` reason).** DN-26 §9 flag-2 resolved
   "validate on interpreted `myc` first, then AOT," but the `semcore.myc` header and M-1013 both warn
   that a self-hosted checker/elaborator/**evaluator** run *inside* the L1 evaluator "almost certainly
   cannot complete at today's cost model" (M-986 TCO gap + M-987 ~n³ eval cost). This is **independent
   of** the §5 boundary pick but gates increments 12 and 14. It firms up only as 8–11 land; flag that
   increment 12 may need the AOT leg rather than the interpreted one, per §9 flag-2.
5. **Stale issue statuses observed (not changed here — FLAG up, per the constraint).**
   `M-1007` is `status:in-progress` but its own Definition of Done records "DONE (2026-07-07)" and
   increment 2 landed (PR #1231) — looks like a missed close-out.
   `M-1011` is `status:todo` but its literal/pattern-typing port landed (PR #1238, commit `a36d9998`)
   with only `infer_type` deferred (FLAG-semcore-20) — it is **partially** done, not `todo`; its
   residual is exactly increment 9's `infer` core. Suggest reconciling both against the codebase
   (mitigation #14) before working M-1013.
6. **Increment-8 sizing (§6.1).** The "~1.5k registration" label under-counts the compilable unit
   (~1.9k+ once the `check_nodule_with` driver + desugaring come along). Plan increment 8 as up to two
   PRs, not one, to stay under the DN-65 soft rule.
