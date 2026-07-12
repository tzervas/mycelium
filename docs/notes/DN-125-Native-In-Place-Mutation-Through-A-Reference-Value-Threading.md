# Design Note DN-125 — Mycelium's Native Answer to the Problem Rust `&mut self` / `&mut T` Solves (In-Place Mutation Through a Reference): Value-Threading over Unique-In-Place Reuse

| Field | Value |
|---|---|
| **Note** | DN-125 |
| **Status** | **Accepted** (2026-07-12, ratified under explicit maintainer delegation — the maintainer authorized the orchestrator to "ratify based on objective reasoning and the project's needs/intents, keep to core principles, report results"; mirrors the DN-115/117/118/122/124 precedent). Was **Draft** (2026-07-12, same day). **Accepted, not Enacted** (house rule #3) — it **builds nothing** yet; the value-threading lowering (§5) stays `Declared`/unbuilt until the FLAGGED build issue (§10, minted **M-1081**) lands and is differential-witnessed. This note still does not edit `crates/**`; the integration close-out applies `Doc-Index.md`/`CHANGELOG.md`/`issues.yaml` per §10 (recorded there, append-only). |
| **Ratification basis (recorded verbatim, 2026-07-12)** | The native **mechanism** is **ANSWERED-BY-DESIGN**: value-threading (`fn f(&mut self)` → `f(self) -> Self`, call-site rebind `x.f()` → `x = f(x)`) is zero-copy via the already-ratified **DN-33** static uniqueness analysis, **DN-35 §5** `rc==1` in-place reuse, and **DN-120** identity coherence — so the genuinely **new** decision content this DN adds is the transpiler lowering (§5) plus the two adversarial narrowings (§6), not a re-litigation of the settled runtime design (§2's mechanism/application split, mitigation #14). **Rank-1 sound**: Alt A (value-threading) dominates the §4 objective function — value-semantics preserved (ADR-003), **KC-3** zero kernel/L0/runtime growth, never-silent on the aliased case. **Adversarially HELD** (§6) with two never-silent FLAG boundaries: (i) unprovable-uniqueness/aliasing routes to a borrowck precondition or a DN-33 uniqueness-proof on the emitted Mycelium, never a possibly-divergent rebind (§6.1); (ii) interior-`&mut`-return methods (`get_mut`/`iter_mut`/`IndexMut`) route to Approximation/Interop-Bridge, never a fabricated value-threaded form (§6.2). **Correct-with-a-copy when built; zero-copy as DN-33's static analysis and DN-35 §5's reuse-write land** (§5.3 — the honest landed-vs-`Declared` boundary is preserved, not upgraded past its basis). Carries forward to Python with a strictly greater aliasing-analysis burden, same mechanism (§8). Ratified on the merits under maintainer delegation — the orchestrator reasoned the recommendation through and decided to ratify; this note's own reasoning (§1–§8) is not re-litigated, only executed and recorded (VR-5: assent is a claim too, tagged here at the basis actually checked). |
| **Decides** | *Recommends, for ratification:* Mycelium's **native solution** to the problem Rust `&mut self`/`&mut T` solves — *in-place mutation of a value through a reference* — is **value-threading**: take the receiver/argument **by value**, return the mutated value, and rewrite the call site to **rebind** (`x = f(x, …)`). Under the hood, DN-33 static uniqueness + DN-35 §5 `rc==1` in-place reuse reclaim the storage so the value-threaded form is zero-copy; identity coherence is closed by DN-120. **Verdict on the crux question (§2):** the native *mechanism* is **answered-by-design** (value semantics + uniqueness + `rc==1` reuse are already ratified); the native *transpiler application* — mechanically lowering a `&mut self` method + rewriting its call sites — is **genuinely open** (no DN scopes it; DN-118 explicitly excludes the method receiver; the transpiler today flat-**gaps** `&mut self`/`&mut T` as "no correspondence"). This DN scopes that application and draws the honest landed-vs-`Declared` boundary. |
| **Feeds** | DN-34 §8.22 (the dominant `Impl`-class gap this addresses); the `mycelium-transpile` `&mut` lowering (new work, M-id FLAGGED §10); DN-118 (the closure-capture `FnMut` lane — the *sibling* problem this note draws the boundary with); DN-119 (removes `&mut` from the "deliberate-exclusion, do-NOT-add-grammar" set on the corrected understanding that the *problem* has a native answer even though the *`&mut` surface* stays excluded). |
| **Grounds on** | ADR-003 (content-addressed value identity, value semantics, no reference types); RFC-0001 §4.6 (value identity = content hash); DN-32 §2.2 (three-layer memory; Layer-2 `rc==1` reuse); DN-33 (Layer-1 static uniqueness analysis — permits in-place mutation of a *provably-unique* value); DN-35 §5 (the reuse-vs-content-address side-condition: reuse at `rc==1`, weak intern table, evict-or-copy); DN-120 (the identity-coherence residual: SOLVED-BY-DESIGN, `rc==1` detection landed `Exact`, reuse-write `Declared`); DN-109 §3 D7 (the `&mut`-aliasing VR-5 trap); DN-110/DN-111 (native-translation taxonomy: Native Equivalent / Idiomatic Remapping / Approximation / Interop Bridge); DN-118 §5 (the closure-capture `FnMut`/`&mut` boundary — the sibling, not this, problem); DN-119 (the deliberate-exclusion set); M-919 (affine/linearity tracker). House rules #1 (transparency), #2 (never-silent), #3 (append-only), #4 (grounded, no sycophancy), #5 (KISS/YAGNI/KC-3), VR-5, G2. |
| **Date** | July 12, 2026 |
| **Task** | Scope Mycelium's native value-semantics solution to the problem `&mut self`/`&mut T` solves — the dominant `Impl`-class transpiler gap (DN-34 §8.22) — as a design-reasoner recommendation the maintainer ratifies. Author only; do not build, do not self-ratify. |

> **Grounding + honesty (house rule #4 / VR-5 / G2).** Read against the tree at **dev tip `b36ebdbe`**
> (2026-07-12) — *not* the `fa53dc46` the task named, which is not present on this checkout; the honest
> citation is the SHA actually read (VR-5). Two corrections made explicitly rather than parroted (no
> sycophancy): (1) the task's premise that this may be "already answered-by-design and merely unbuilt in
> the transpiler" is **half right** — the runtime *mechanism* is answered-by-design, but the transpiler
> *lowering* is genuinely open, and the two must not be conflated (§2). (2) The task frames the target as
> "the dominant `Impl`-class gap"; verify-first, **most** of that gap is `&self`/`self` (read-only or
> by-value), which the transpiler *already* handles by erasing `&T` to value `T`
> (`crates/mycelium-transpile/src/map.rs:341-342`) — a landed **Native Equivalent**. Only the genuinely
> **`&mut self`/`&mut T`** subset needs the new mechanism this DN designs; the DN says so rather than
> overclaiming the whole class as open (§4).

---

## §1 The problem, stated without reference to Rust's mechanism (DN-110 §9 step 2)

Rust's `&mut self` / `&mut T` is a **mechanism**. The **problem** it solves is:

> **In-place mutation of a value through a reference:** a callee is handed a path to a caller's binding,
> mutates the value in place, and the caller observes the change after the call returns — *without
> copying the whole value* and *without the callee taking ownership*.

Three surface shapes carry this problem (the ones the transpiler and DN-34 §8.22 meet):

- **S1 — `fn f(&mut self, …)` method:** mutate the receiver in place (builder methods, `Iterator::next`,
  `Extend::extend`, `Vec::push`-style, `Hash::hash(&mut Hasher)` on the hasher side).
- **S2 — `fn g(x: &mut T)` parameter:** mutate an argument in place (`fn swap(a: &mut T, b: &mut T)`,
  `fn fill(buf: &mut [u8])`).
- **S3 — a `&mut` local threaded through a call:** `let r = &mut x; h(r);` — a mutable borrow passed down
  a call chain and mutated at the bottom.

Per DN-111, the classification is of a **(construct, context) pair, not a construct** — `&mut` lands in
different taxonomy cells depending on **aliasing**, a fact `syn` cannot see (DN-111's sharpest finding).
That single observation structures the entire recommendation below.

## §2 The crux verdict — answered-by-design *mechanism*, genuinely-open *application* (mitigation #14)

The task asks: is the `&mut self` → value-semantics mapping **already answered-by-design** and merely
unbuilt in the transpiler, or **genuinely open**? Grounded in code, the honest answer separates two
questions the framing tends to fuse:

**Q-mechanism — "does Mycelium have a native way to solve in-place-mutation-through-a-reference?"
→ ANSWERED-BY-DESIGN.** Mycelium's value-semantic answer is **value-threading**: a function that would
mutate `x` in place instead *consumes* `x` and *returns* the new value; the caller rebinds. This is a
pure-functional shape, and Mycelium's ratified memory chain makes it **zero-copy**:

- **DN-33** (Layer-1 static uniqueness analysis) proves a binding *unique* at a use site, "permitting
  in-place mutation of a unique value, identity reassigned on write."
- **DN-35 §5** gives the `rc==1` reuse rule: reuse a cell's storage in place iff `rc==1 ∧ same-shape ∧
  dead-after` (and the cell is not pinned in the content-address index, else evict-or-copy). At `rc==1`
  no live alias holds the old identity, so the reuse is **unobservable** (Clojure-transient discipline at
  cell granularity).
- **DN-120** already closed the identity-coherence residual ADR-003 disclosed: this is
  **SOLVED-BY-DESIGN**; the `rc==1` *detection gate* is landed and `Exact`
  (`crates/mycelium-std-runtime/src/rc.rs`, `RcProbe::UniqueOwner`), the reuse-*write* is `Declared`.

So the runtime substrate that makes value-threading as cheap as `&mut` **is designed and partly built**.
Nothing new needs inventing at the language/runtime tier. This is the "answered-by-design" half the task
anticipated — correctly.

**Q-application — "does the transpiler lower a `&mut self` method to that value-threaded form?"
→ GENUINELY OPEN.** Verified against the tree:

| Site | Current behavior at `dev@b36ebdbe` | Consequence |
|---|---|---|
| `emit::map_signature`, `FnArg::Receiver` (`crates/mycelium-transpile/src/emit.rs:559-566`) | `&mut self` → **hard gap** `GapReason(Category::Other, "&mut self conflicts with Mycelium's value semantics (ADR-003) — no correspondence")` | the whole method is dropped, un-emitted |
| `map_type::visit_reference` (`crates/mycelium-transpile/src/map.rs:341-358`) | `&T` → **erased to value `T`** (landed Native Equivalent); `&mut T` → **explicit gap** `Category::Other`, "in-place mutation through a borrow has no value-semantic correspondence" | `&mut T` params/returns drop |
| DN-118 §5 (the FnMut/`&mut` gate, `crates/mycelium-transpile/src/emit.rs`) | scans **closure bodies** for capture mutation; a method-call **receiver at all** is over-flagged because `&self`/`&mut self` dispatch "is unknowable from `syn` syntax alone" | covers **closure captures only**, explicitly *not* the method-receiver value-threading this DN scopes |

The transpiler's own comment — *"no correspondence"* — is the exact stance this DN **corrects**: there
*is* a native correspondence (value-threading), the transpiler simply does not emit it yet. No DN scopes
that lowering; DN-118 explicitly excludes it (it handles the *closure-capture* sibling problem, §5.2).
**Verdict: the mechanism is answered-by-design; the transpiler application is open, and this DN is that
scoping.** Conflating the two would either (a) declare the whole thing "solved" and never build the
lowering, or (b) re-litigate the settled DN-33/DN-35 runtime design — both wrong (mitigation #14).

## §3 The alternatives (real options, each with its mechanism)

**Alt A — Value-threading (take-by-value, return-the-value, rebind at the call site).** `fn f(&mut self,
a) [-> R]` lowers to a Mycelium method `f(self: Self, a) -> Self` (or `-> (Self, R)` when it also
returns), taking `self` by value and returning the mutated value; the call site `x.f(a)` rewrites to
`x = f(x, a)` (or `(x, r) = f(x, a)`). DN-33 uniqueness + DN-35 §5 `rc==1` reuse make it zero-copy under
the hood. **No kernel, L0, or runtime change** — it is emission + call-site rewrite over the *existing*
value-semantics substrate (KC-3/DRY). The FIP/`@unique` surface DN-33 Q6 deferred is *inferred*, not
required.

**Alt B — A first-class mutable-reference / place type in the kernel (mirror `&mut`).** Add a `&mut`-like
borrow or L-value place type. **Rejected.** It reintroduces exactly what Mycelium deliberately excludes:
aliased-mutation borrow-checking — "the *hardest* part of Rust-style borrow checking (policing aliased
mutation) is simply **absent**" (DN-33 §3, `research/03 §T3.5`) — and breaks value identity/immutability
(ADR-003, LR-8, DN-32). It is on DN-119's deliberate-exclusion set precisely to protect the
value-semantics guarantees. Maximum kernel surface for zero net capability over Alt A.

**Alt C — Interior-mutability cell (an RC-cell / `swap`-based mutable slot).** Model `&mut T` as a runtime
mutable cell (`RefCell`/atomic-slot analogue). **Rejected for the general case** — a mutable cell has no
stable content hash (breaks ADR-003 identity), reintroduces shared mutable state, and forces the
never-silent swap machinery onto what is normally a pure local mutation (YAGNI/over-engineering).
**Retained only** as the narrow **Interop-Bridge** fallback for genuinely-aliased state that cannot be
value-threaded (§6, adversarial case 1) — never the default.

**Alt D — Uniqueness-typed in-place mutation surface (`substrate`/`@unique` + a source-level mutate
syntax).** Expose M-919 affine + a `@unique`/`fip` annotation so a provably-unique binding is mutated in
place with mutation syntax. **Folded into Alt A** — the *semantics* are still value-threading (a unique
value consumed and reborn); the annotation only documents uniqueness. DN-33 Q6 already decided this
surface **invisible/inferred for R1** (no user-facing annotation — KISS). As a transpiler target it
reduces to Alt A.

## §4 Recommendation (ranked) with the objective function

**Objective function** (the criteria a native answer to this problem must satisfy, in priority order):
value-semantics preservation (ADR-003/LR-8) > faithfulness to source semantics on the non-aliased case
(VR-5) > never-silent on the hard/aliased case (G2/VR-5) > KISS/KC-3 (no kernel growth) > performance
(zero-copy) > source-language generality (§8).

| Criterion (weight) | **Alt A — Value-threading** | Alt B — `&mut` kernel type | Alt C — mutable cell |
|---|---|---|---|
| Value-semantics preserved (ADR-003) | **Yes** — new value, old consumed | **No** — reintroduces places | **No** — no stable content hash |
| Faithful, non-aliased (VR-5) | **Exact** (a rebind = the mutation) | Exact (but at the cost above) | Exact (but at the cost above) |
| Never-silent on aliased case (G2) | **Yes** — FLAG when uniqueness unprovable (§6) | N/A (allows aliasing) | masks aliasing as "fine" |
| KISS / no kernel growth (KC-3) | **Yes** — emit + rewrite only | **No** — max kernel surface | **No** — new runtime cell + swap |
| Zero-copy performance | **Yes** via DN-33+DN-35 §5 `rc==1` reuse (once reuse-write lands; correct-but-copying until) | Yes (native mutation) | allocation + indirection |
| Carries to Python & other source langs (§8) | **Yes** — mechanism is source-general | No (Rust-specific borrow) | Partial (models Python aliasing but loses value semantics) |

**Rank 1 — Alt A (value-threading), for the non-aliased, value-returning shape.** This is the dominant
`Impl`-class case and Mycelium's native answer. Classified per **DN-111**:

- **Read-only `&self`/`&T` methods** (the *bulk* of DN-34 §8.22 — `Display::fmt`, `PartialOrd::cmp`,
  most trait reads): **Native Equivalent**, and **already landed** (`map.rs:341-342` erases `&T` to `T`).
  The borrow was never load-bearing under value semantics. *This DN does not need to touch them* — it
  clarifies they are already solved, correcting an overbroad reading of "the `Impl`-class gap."
- **Mutating `&mut self`/`&mut T`, provably-unique/non-aliased:** **Idiomatic Remapping** (DN-111 — the
  surface shape changes: a `()`-returning in-place mutation becomes a value-returning rebind), and
  **`Exact`/`Proven`-eligible** because on the non-aliased case the rebind is *semantically identical* to
  the mutation. This is the new transpiler work (§5).
- **`Drop::drop(&mut self)`** is a **special case** routed to the reclamation lane, not value-threaded:
  it does not mutate for the caller's benefit; it releases resources at end-of-life — DN-35/DN-120's
  `rc==1` reclamation is its native answer. The DN calls this out so the build does not mis-map it.

**Rank 2 — Alt C (mutable cell) as a *narrow Interop-Bridge fallback only*** for the genuinely-aliased
residual (§6.1) that cannot be value-threaded — never the default, always FLAGGED never-silent, never a
`Native`-semantics claim (DN-111 Interop-Bridge honesty ceiling).

**Rejected — Alt B** (kernel `&mut`): loses the guarantee the whole language exists to provide.

## §5 The transpiler lowering (concrete, grounded in DN-33 / DN-35 §5)

### 5.1 Signature + body

Rust:

```rust
impl Counter {
    fn incr(&mut self, by: u64) { self.n += by; }
}
```

Mycelium (value-threaded):

```text
// &mut self  ->  by-value self, returns the new self
incr(self: Counter, by: U64) -> Counter = { self with n = self.n + by }
```

- `emit::map_signature`'s `FnArg::Receiver` arm (`emit.rs:559`) changes from *"`&mut self` → hard gap"*
  to *"`&mut self` → `self: Self` param **and** widen the return type to `Self`"* (or `(Self, R)` when the
  original returns `R`).
- `&mut T` params (`map.rs:344`, S2) change from *gap* to *`T` param + thread `T` into the return*.
- The body's in-place field writes (`self.n += by`) become **functional-update** expressions
  (`self with n = …`) — the DN-123 records/named-fields sugar over positional `Data` is the target form.

### 5.2 Call-site rewrite (the load-bearing half)

| Source shape | Rewrite | Notes |
|---|---|---|
| `x.f(a)`, `f: &mut self -> ()` | `x = f(x, a)` | plain rebind |
| `let r = x.f(a)`, `f: &mut self -> R` | `(x, r) = f(x, a)` | tuple-return; `Self × R` |
| `g(&mut y)` (S2) | `y = g(y)` | arg rebind |
| `g(&mut y)` returning `R` | `(y, r) = g(y)` | tuple-return |
| `h(&mut x); …; k(&mut x)` (S3 chain) | thread `x` through each: `x = h(x); …; x = k(x)` | sequential rebind |

**Why it is zero-cost (the DN-33/DN-35 §5 tie-in):** at each rebind `x = f(x, …)`, `x`'s old binding is
dead immediately after the call. DN-33 uniqueness analysis proves `x` unique at that point ⇒ DN-35 §5
fires `rc==1` in-place reuse ⇒ `f`'s returned `Counter` reuses `x`'s freed allocation ⇒ **no copy**, the
same store `self.n += by` would have done. The `EXPLAIN` ladder (DN-36 §7) surfaces the choice:
`in-place (statically unique)` / `in-place (rc==1 runtime)` / `copying (shared)` — never silent (G2).

### 5.3 The honest landed-vs-`Declared` boundary (VR-5)

| Piece | Status | Basis |
|---|---|---|
| `&T`/`&self` erased to value `T` (read-only Native Equivalent) | **Landed, `Empirical`** | `map.rs:341-342`; `emit.rs:567-573` |
| `rc==1` **detection** gate (uniqueness at runtime) | **Landed, `Exact`** | `crates/mycelium-std-runtime/src/rc.rs` `RcProbe::UniqueOwner` |
| The value-threading **transpiler lowering** (§5.1–5.2) | **Not built — `Declared`** | *this DN's proposed work; the M-id FLAGGED §10.* Today `&mut self`/`&mut T` flat-gap (`emit.rs:559`, `map.rs:344`) |
| DN-33 **static** uniqueness analysis (compile-time `rc` elision, MEM-4) | **Not built — `Declared`** | DN-33 Accepted design; the static half that makes value-threading zero-copy *without* a runtime probe |
| The `rc==1` **reuse-WRITE** (the in-place byte write) | **Not built — `Declared`** | DN-35 §5 / DN-120 §3 / E12 Increment 3 / task #6, desktop-held |

**Net:** value-threading is **correct today with a copy** (it needs only the emission + call-site rewrite
over the landed value-semantics substrate) and becomes **zero-copy** as DN-33's static analysis and
DN-35 §5's reuse-write land. The correctness of the mapping does **not** depend on the reuse-write — that
is a performance optimization the mapping *enables*, not a precondition. This DN must not claim the
lowering is built (it is not) nor that it is zero-copy today (it is a copy until the reuse-write lands).

## §6 Adversarial stress-test (house rule #4 / VR-5 — attack the recommendation)

**6.1 Aliasing — two `&mut` to overlapping state / a receiver that is not provably unique.**
In valid Rust, two live `&mut` to the same state are *impossible* (borrowck forbids them) — so if we
transpile **borrowck-valid** Rust, S1/S2 receivers are non-aliased by the source's own invariant.
**But the transpiler works from `syn`, which carries no borrowck facts** (DN-118/DN-109 D7 — "the core
VR-5 trap"). Two honest backstops, either sufficient, both cheap:
(a) **Precondition:** transpile only borrowck-checked Rust (a documented input contract) — then
non-aliasing is inherited from the source.
(b) **Verify on the Mycelium side:** run **DN-33 uniqueness analysis** on the *emitted* Mycelium at the
rebind site; if it **cannot prove** `x` unique (an escaping alias, a stored reference), **FLAG
never-silent** (`Category::Other`, message + suggested idiom) rather than emit a possibly-divergent
rebind. **Verdict: NARROWS.** The recommendation is conditioned: value-threading fires only where
uniqueness holds (by source precondition or Mycelium-side proof); otherwise it FLAGS (→ Alt C
Interop-Bridge or a human port). This is DN-109 D7 / VR-5 surfaced, not glossed.

**6.2 `&mut self` returning a reference *into* self** (`get_mut`, `iter_mut`, `IndexMut`).
Value-threading returns a *value*, not an interior mutable borrow — there is no value-semantic form for
"hand back a writable path into my interior." **Verdict: NARROWS — genuine hole for Alt A.** These are
**out of scope for value-threading** and must be **FLAGGED** (Approximation/Interop-Bridge, never-silent)
or remapped to a whole-value functional-update idiom (`self with field = new` returned by value). The DN
draws the boundary at `&mut self` methods whose return is a **value or `()`**; interior-`&mut`-returning
methods are a separate, flagged residual.

**6.3 Re-entrancy** — `f(&mut self)` calls something that re-enters and touches `self`.
Under value-threading `self` is **moved into** `f`; during the call there is **no aliased path** back to
the caller's binding (it is consumed). Re-entrant aliasing through the same binding is **impossible by
construction**. **Verdict: HOLDS — a strength.** Value-threading is *more* robust here than `&mut`
(which must dynamically forbid re-entrant aliasing).

**6.4 Observable identity change (ADR-003).** Does `x = f(x)` change `x`'s content-address identity
mid-computation, observed by a holder of the old identity? DN-35 §5 / DN-120: reuse fires only at
`rc==1`, so **nothing** holds the old identity; and at the source level value-threading *creates a new
value* — the old `x` is dead. Identity is "a property of *live* values, and reuse only touches *dead*
ones" (DN-35 §5). If the value is interned in the content-address table (effective `rc≥2` via the weak
map), reuse does **not** fire — it copies (DN-35 §5 evict-or-copy). **Verdict: HOLDS** (closed by
DN-120).

**Adversarial result: HELD for the non-aliased, value-returning shape (the dominant `Impl`-class
target); NARROWED to exclude (i) statically-unprovable-unique / aliased receivers and (ii)
`&mut self`-returning-an-interior-reference — both FLAGGED never-silent (G2), routed to Alt C
Interop-Bridge or a human port.** No unpatchable hole *inside the recommended scope*; the two narrowings
are the honest boundary, not a defeat.

## §7 The DN-119 exclusion-set reconciliation (append-only clarification)

DN-119 lists `&mut` in the **deliberate-exclusion set** that "must NOT get L3 grammar without regressing
value-semantics guarantees." This DN does **not** contradict that: the **`&mut` *surface* stays
excluded** (no `&mut` grammar, no places, no aliased mutation — Alt B rejected). What this DN adds is
that the **problem** `&mut` solves has a **native answer** (value-threading) that needs **no** `&mut`
grammar — consistent with DN-119's own Rank-1 reframe ("L3 expresses every construct's native *answer*").
The exclusion is of the mechanism; the problem is still natively solved. (A forward pointer at DN-119's
exclusion-set row is FLAGGED for the integrator, §10 — append-only, no normative edit.)

## §8 Carries-forward to Python and other source languages (source-generality)

The problem is **source-language-general**, and so is the native answer — but the *analysis burden*
differs by source:

- **Python** has no `&mut`, yet meets the *same* problem pervasively: methods mutating `self`
  (`self.x = …`, `list.append`, `dict.__setitem__`) and functions mutating arguments (in-place list/dict
  mutation, mutable default args). Python's semantics are **reference semantics** — mutation is visible
  through **all** aliases — which is **harder** than Rust, because Python *permits* the aliased mutation
  Rust forbids. The value-threading mapping carries forward **for the non-aliased case**; but Python's
  aliasing is **unchecked**, so the §6.1 "FLAG when uniqueness unprovable" path **fires more often** for
  Python than for (borrowck-valid) Rust. The native *mechanism* is the same; the *safety obligation* is
  strictly greater and must not be understated (VR-5). A dedicated Python DN (DN-119 already flags Python
  as needing its own note) would own that aliasing-analysis burden; this DN only records that the
  mechanism is shared.
- **Other value-semantics or copy-on-write source languages** (Swift `inout`, Clojure transients) map
  even more directly — `inout` *is* value-threading with compiler sugar; Clojure transients *are* the
  `rc==1` discipline. These are Native-Equivalent, not merely Remapping.

## §9 Definition of Done — what maintainer ratification (Draft→Accepted) requires

**Done at authoring (this note):**
1. Problem stated mechanism-free + the three shapes S1–S3 — **done** (§1).
2. Crux verdict grounded in code (mechanism answered-by-design; application open) — **done** (§2).
3. Alternatives enumerated with mechanisms + ranked against an explicit objective function — **done**
   (§3–§4).
4. Concrete transpiler lowering (signature + call-site rewrite) grounded in DN-33/DN-35 §5, with the
   honest landed-vs-`Declared` boundary — **done** (§5).
5. Adversarial stress-test with an explicit held/narrowed/hole verdict — **done** (§6).
6. DN-119 exclusion-set reconciliation + carries-to-Python note — **done** (§7–§8).

**For the maintainer to ratify (this note does not self-ratify — house rule #3):**
7. Confirm the verdict (§2): the runtime mechanism is answered-by-design (DN-33/DN-35 §5/DN-120), the
   transpiler application is open, and this DN is the correct scoping (not a re-litigation of DN-33/35).
8. Confirm **Rank 1 (Alt A, value-threading)** as Mycelium's native answer, with the §4 classification
   split (read-only `&self` = landed Native Equivalent; mutating `&mut self`/`&mut T` non-aliased =
   Idiomatic Remapping = the new work; `Drop` = reclamation lane; interior-`&mut`-return = flagged
   residual).
9. Confirm the §6 narrowings as the ratified boundary: value-threading fires only where uniqueness holds
   (source precondition or DN-33 Mycelium-side proof), FLAGS otherwise (never-silent), and does **not**
   cover interior-`&mut`-returning methods.
10. Confirm the honest tag boundary (§5.3): the lowering is `Declared`/unbuilt, correct-with-a-copy when
    built, zero-copy only as DN-33 static analysis + DN-35 §5 reuse-write land (E12 Increment 3).
11. **Mint the build issue** (M-id, FLAGGED §10) for the transpiler value-threading lowering, with the
    §6 uniqueness-gate + never-silent FLAG as hard DoD conditions and a differential-witness requirement
    (an emitted value-threaded method + call site that `myc check`-cleans and matches the Rust behavior
    on a fixture) before any `Empirical` upgrade.

Status stays **Draft** until 7–11 are ratified.

## §10 FLAGGED items (integration-owned — not applied here)

- **`Doc-Index.md`:** add a Design-Notes row — `DN-125 — Mycelium's Native Answer to &mut self/&mut T
  (In-Place Mutation Through a Reference): Value-Threading over Unique-In-Place Reuse (Draft)`.
- **`CHANGELOG.md`:** add a dated Design-phase entry for DN-125 (Draft).
- **`issues.yaml`:** **mint one build issue** (next free `M-xxxx`) — *"Transpiler `&mut self`/`&mut T`
  value-threading lowering (DN-125): by-value receiver/param + call-site rebind, uniqueness-gated,
  never-silent FLAG on unprovable-uniqueness and interior-`&mut`-return, differential-witnessed."*
  `doc_refs: corpus:DN-125`, `src:crates/mycelium-transpile/src/emit.rs:559`,
  `src:crates/mycelium-transpile/src/map.rs:344`. Note its dependency on DN-33 (static uniqueness) for
  the zero-copy half and DN-35 §5 / E12 Inc 3 for the reuse-write; correct-with-copy is buildable ahead
  of both.
- **DN-119** exclusion-set row: append-only forward pointer to DN-125 (§7) — *"`&mut` surface stays
  excluded; the problem it solves has a native answer, DN-125."* No normative text changed.
- **DN-118** §5: append-only forward pointer — the method-receiver value-threading DN-118 excluded is
  scoped by DN-125 (sibling lanes: DN-118 = closure captures, DN-125 = method/param receivers).

**Applied at the 2026-07-12 ratification close-out (append-only note, original FLAGs above left
as-authored):** `Doc-Index.md` DN-125 row added at status **Accepted**; `CHANGELOG.md` carries the
ratification entry; **M-1081** minted for the transpiler value-threading lowering build
(`depends_on: [M-1079]` — the DN-124 phylum-mode vet harness, needed so the lowering's
`checked_fraction` payoff is measurable across cross-nodule call sites, mirroring the M-1080
precedent); `doc_refs: corpus:DN-125, src:crates/mycelium-transpile/src/emit.rs:559,
src:crates/mycelium-transpile/src/map.rs:344`. DN-119/DN-118 forward-pointer rows are FLAGGED for a
follow-up append-only edit (not applied in this close-out — no normative text of either note changes;
tracked so the reconciliation isn't lost).

**Follow-up applied (2026-07-12, at the DN-126–DN-132 ratification close-out):** the DN-119/DN-118
forward-pointer rows FLAGGED above are now applied — an append-only row added to DN-119's exclusion-set
table (the `&mut` row) and an append-only paragraph added to DN-118 §5, both pointing at this note
(DN-125) as the ratified native answer. No normative text of either note changes.

## §11 Grounding

- **ADR-003** (`docs/Mycelium_Project_Foundation.md:365-370`) — content-addressed identity; value
  semantics; no reference types. Grounds §1, §2, §6.4.
- **`docs/notes/DN-33-Layer1-Static-Uniqueness-Analysis.md`** (read 2026-07-12) — static uniqueness
  permitting in-place mutation of a unique value; §3 "no write barriers / no mutation-alias tracking";
  Q6 FIP surface deferred/inferred. Grounds §2, §3 (Alt A/D), §5.2, §6.1.
- **`docs/notes/DN-35-Env-Machine-Reclamation.md` §5** (read 2026-07-12, full) — the reuse-vs-content-
  address side-condition (reuse at `rc==1`, weak intern table, evict-or-copy). Grounds §2, §5.2, §6.4.
- **`docs/notes/DN-32-Three-Layer-Hybrid-Memory-Architecture.md` §2.2** — Layer-2 `rc==1` reuse; LR-8
  immutability. Grounds §2, §3.
- **`docs/notes/DN-120-Content-Addressed-Identity-vs-Temporary-Copy-Mutation-Verdict.md`** (read
  2026-07-12, full) — identity-coherence SOLVED-BY-DESIGN; the landed (`rc==1` detection, `Exact`) vs
  `Declared` (reuse-write) boundary. Grounds §2, §5.3, §6.4.
- **`docs/notes/DN-118-Closure-To-Value-Semantics-Transpiler-Enabler-And-Native-Conformance-Contract.md`
  §5** (read 2026-07-12) — the closure-capture `FnMut`/`&mut` gate; the method-call-**receiver** is
  over-flagged because `&self`/`&mut self` dispatch is unknowable from `syn` — the explicit exclusion this
  DN's method-receiver lane complements. Grounds §2, §6.1, §10.
- **`docs/notes/DN-109-Idiom-Optimal-Transpilation-And-Structural-Remapping.md` §3 D7** — the
  `&mut`-aliasing VR-5 trap. Grounds §6.1.
- **`docs/notes/DN-111-Canonical-Rust-To-Mycelium-Native-Translation-Taxonomy.md`** — Native Equivalent /
  Idiomatic Remapping / Approximation / Interop Bridge; classification is of a (construct, context) pair;
  the honesty ceilings. Grounds §1, §4, §6.
- **`docs/notes/DN-119-L3-Comprehensive-Surface-Expressibility-Scoping.md`** — `&mut` in the
  deliberate-exclusion set; the "native answer" reframe; Python out of scope needing its own DN.
  Grounds §7, §8.
- **`crates/mycelium-transpile/src/emit.rs:559-573`** (read 2026-07-12) — `&mut self` hard gap "no
  correspondence"; `&self` → value `self`. The ground-truth for the "open application" verdict. Grounds
  §2, §5.1, §5.3.
- **`crates/mycelium-transpile/src/map.rs:326-358`** (read 2026-07-12) — `&T` erased to value `T`
  (landed); `&mut T` explicit gap. Grounds §2, §4, §5.1, §5.3.
- **`crates/mycelium-std-runtime/src/rc.rs`** (read 2026-07-12, full) — `RcProbe::UniqueOwner` `rc==1`
  detection, `Exact`; reuse-write `Declared` (DN-32 §6a). Grounds §2, §5.3.
- **DN-34 §8.22** — the `Impl`-class gap this addresses (69 sub-issues / 36 pure gaps). Grounds §4's
  read-only-vs-mutating split note (the precise `&self`/`&mut self` census within the class is a
  measurement left to the build, §9.11 — not claimed here). Grounds §1, §4.
- **M-919** — affine/linearity tracker (the uniqueness backstop of §6.1). Grounds §3 (Alt D), §6.1.
- **House rules:** #1 (transparency/per-op tags), #2 (never-silent), #3 (append-only, no self-ratify),
  #4 (grounded, no sycophancy — the two epigraph corrections), #5 (KISS/YAGNI/KC-3 — Alt A no kernel
  growth), VR-5 (no tag upgraded past basis), G2 (FLAG the hard case, never silently emit).

---

## Meta — changelog

- **2026-07-12 — Created (Draft).** Design-reasoner scoping + recommendation for Mycelium's native answer
  to the problem Rust `&mut self`/`&mut T` solves (in-place mutation through a reference — the dominant
  DN-34 §8.22 `Impl`-class gap). Crux verdict, grounded against `dev@b36ebdbe`: the runtime **mechanism**
  is **answered-by-design** (value-threading over DN-33 uniqueness + DN-35 §5 `rc==1` reuse; identity
  coherence closed by DN-120; `rc==1` detection landed `Exact` in `mycelium-std-runtime/src/rc.rs`), but
  the transpiler **application** is **genuinely open** — the transpiler today flat-**gaps** `&mut self`
  (`emit.rs:559`) and `&mut T` (`map.rs:344`) as "no correspondence," and DN-118 explicitly excludes the
  method receiver (it covers closure captures). Ranked three alternatives against an explicit objective
  function; recommends **Rank 1 — value-threading** (by-value receiver/param + call-site rebind), with
  the DN-111 split (read-only `&self` = landed Native Equivalent; mutating `&mut self`/`&mut T`
  non-aliased = Idiomatic Remapping = the new work; `Drop` = reclamation lane); rejects a kernel `&mut`
  type (Alt B) and an interior-mutability cell as default (Alt C, retained only as a narrow Interop-Bridge
  fallback). Adversarial stress-test: **HELD** for the non-aliased value-returning shape, **NARROWED** to
  FLAG (i) unprovable-unique/aliased receivers (the DN-109 D7 / VR-5 trap — gated by borrowck precondition
  or DN-33 Mycelium-side proof) and (ii) interior-`&mut`-returning methods (a genuine value-threading
  hole); re-entrancy and identity-change **HELD** (value-threading is strictly more robust). Honest tag
  boundary: the lowering is `Declared`/unbuilt, correct-with-a-copy when built, zero-copy only as DN-33
  static analysis + DN-35 §5 reuse-write land (E12 Inc 3, desktop-held). Reconciled with DN-119's
  exclusion set (the `&mut` *surface* stays excluded; the *problem* is natively answered) and noted the
  carry-forward to Python (same mechanism, strictly greater aliasing-analysis burden — Python's reference
  semantics permit the aliased mutation Rust forbids). Authored the DN only — no edit to `issues.yaml`,
  `CHANGELOG.md`, `Doc-Index.md`, or any `crates/**` (integration-owned; FLAGGED §10). Append-only; status
  advances only by maintainer ratification (house rule #3).
- 2026-07-12 — **Accepted.** Ratified by the maintainer's explicit delegation to the orchestrator
  ("ratify based on objective reasoning and the project's needs/intents, keep to core principles,
  report results"; mirrors the DN-115/117/118/122/124 precedent). Ratifies **Rank 1 — Alt A
  (value-threading)** as Mycelium's native answer to the problem `&mut self`/`&mut T` solves, with the
  §4 classification split (read-only `&self` = landed Native Equivalent; mutating `&mut self`/`&mut T`
  non-aliased = Idiomatic Remapping = the new transpiler work; `Drop` = reclamation lane; interior-`&mut`
  -return = flagged residual) and the §6 adversarial narrowings as the ratified boundary (value-threading
  fires only where uniqueness holds — source precondition or DN-33 Mycelium-side proof — and FLAGS
  never-silently otherwise; does not cover interior-`&mut`-returning methods). Basis recorded verbatim in
  the header table's "Ratification basis" row: the mechanism is answered-by-design (DN-33/DN-35 §5/
  DN-120, zero kernel growth, KC-3), Rank-1 sound against the §4 objective function, adversarially HELD
  with the two never-silent FLAG boundaries, correct-with-a-copy today and zero-copy as DN-33/DN-35
  land, and carries to Python with a strictly greater aliasing burden. **Accepted, not Enacted** (house
  rule #3) — the lowering (§5) is unbuilt; every tag stays `Declared` until implemented and
  differential-witnessed. Minted **M-1081** (transpiler `&mut self`/`&mut T` value-threading lowering)
  this close-out, `depends_on: [M-1079]` (DN-124 phylum-mode vet harness, for measurable
  `checked_fraction` credit on the cross-nodule call sites the lowering rewrites).
