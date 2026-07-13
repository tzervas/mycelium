# Design Note DN-135 — Result/Option Combinator-over-Closure Native Lowering (the match-inline desugar)

| Field | Value |
|---|---|
| **Note** | DN-135 |
| **Status** | **Accepted** (2026-07-12; was **Draft** 2026-07-12, ratified the same day). Ratified by the strict DN-review gate (9-criterion pass — grounding/VR-5/G2/append-only/native-solution/KC-3/adversarial/DoD/consistency), with every decision-critical witness verified against `dev @ 08d8fc21` and the receiver-gate/`prim_map` citations corrected (see §9). Renumbered from an initial DN-133 draft to resolve a collision with the Qualified-Associated-Fn-Call DN-133. **Still builds nothing** — the decision is ratified, but every mechanism below stays `Declared`/unbuilt until the FLAGGED build issue (M-1092) lands and is differential-witnessed (VR-5). Does not edit `crates/**`; `Doc-Index.md`/`CHANGELOG.md`/`issues.yaml` rows are **FLAGGED** for the integrating parent, not applied here. |
| **Decides** | *Proposes, for ratification:* (1) the **verified residual** — the Result/Option *combinator surface* (`map`/`map_err`/`and_then`/`or_else`/`fold`/`unwrap_or`) is **already a Native Equivalent** that exists and runs (`lib/std/result.myc`, `lib/std/option.myc`; M-649/M-685/687), and the generic method-desugar already rewrites `recv.m(a)` → `m(recv, a)` (`emit.rs:2179–2189`). The gap is **not** the combinator — it is the **closure argument** when its parameter is a **unit pattern `\|()\|`, a wildcard `\|_\|`, or otherwise not a single explicitly-typed identifier** (exactly what DN-118 Phase-1 leaves out — `emit.rs:2526–2549`). (2) The **native-solution class** (DN-111 §3.2): the combinator is a Native Equivalent; the closure-param residual is closed by an **Idiomatic Remapping** — a **combinator-directed match-inline** that inlines the combinator's *own* stdlib `match` body with the closure body substituted, **relocating the unit/wildcard from the (unsupported) `lambda`-parameter position to the (fully-supported) `match`-pattern position**. (3) The **gate**: the desugar fires **only** on a *confirmed* Result/Option receiver (`expr_env_type`, the exact `prim_map::receiver_gate_matches` discipline — consulted at `emit.rs:2120–2123`, defined at `prim_map.rs:228`); an unknown receiver or a non-Result/Option `.map` (e.g. an iterator's) **falls through** to the unchanged generic desugar or an honest gap — never a guess (VR-5). (4) The **closure-literal vs. function-value split**: a **closure-literal** argument is inlined; a **named-function value** argument (`.map(SomeFn)`) takes the existing `m(recv, SomeFn)` free-function call unchanged. (5) The **honesty boundary + open questions** (§7/§8). It **references DN-118** for the closure-emit pass it extends (does not duplicate it), **DN-126** for the two-mode invariance argument, and **DN-111** for the taxonomy. |
| **Feeds** | The `std-sys-host` 6/6 path — closes residual #1 in `OsEntropy::fill_bytes` (`lib.rs:27–30`); a large slice of the corpus DN-34 §8.22 "Other" gap class (combinator-over-closure appears across the stdlib port surface); DN-118 (the closure-emit pass this note's match-inline **complements** — it handles the param shapes DN-118 Phase-1 declines); DN-126 (loose/strict two-mode typing — this note's desugar is mode-invariant, a data point for that note's runnable-floor argument). |
| **Grounds on** | **DN-111 §3.2** (native-equivalence taxonomy — Native Equivalent combinator + Idiomatic Remapping for the param residual); **DN-106 GP2** (gap-closure default = the mechanically-lowering desugar over a new kernel form — the ratified basis for desugar-to-`match` over any new closure-param grammar); **KC-3** (zero kernel growth — `match` and the stdlib combinators exist; the pass adds no L0/`Ty`/eval node); **KISS/YAGNI** (no unit-type lever, no general type-inference pass, no new combinator surface); **DRY** (inlines the combinator's *own* `match` definition — no parallel semantics; `result.myc:23–46`, `option.myc:36–58`); **DN-118** (the closure-emit pass — single explicitly-typed-identifier param only, `emit.rs:2519–2592`; the unit/wildcard/untyped param is its named out-of-scope residual); **DN-126** (two-mode typing — the match-inline needs *no* param type, so it is identical in loose and strict mode, where the alternative lambda-param-typing path would diverge); **G2/never-silent** (an unconfirmed receiver, a non-inlinable argument, and a multi-param closure are all never-silent gaps/fallthroughs, never a fabricated emission); **VR-5** (the whole design is `Declared` until built + differential-witnessed against the real oracle; no `Proven` claim). |
| **Date** | July 12, 2026 |
| **Task** | Scope the Result-combinator-over-closure residual that (with DN-134) blocks `std-sys-host` 6/6, and that is a corpus-wide "Other" gap class — verify-first, native solution, ranked recommendation, emission spec, adversarial stress-test, DoD. Read-only except this DN + its FLAGGED rows. Parallel-cluster slot: **DN-135** (mit #1 — DN-133 held by the Qualified-Associated-Fn-Call DN, DN-134 held by the sibling struct-variant DN; DN-135 verified free — renumbered from an initial DN-133 draft to resolve the collision). |

> **Grounding + honesty (house rule #4 / VR-5 / G2).** This note **works a decision forward and
> recommends, ranked**; it does not take it (house rule #3). Its central finding — reported on the
> evidence, not shaped to a larger deliverable — is that **the combinator surface is not the gap.**
> `map`/`map_err`/`and_then`/`or_else`/`fold`/`unwrap_or` already exist as native `.myc` free functions
> whose *bodies are `match` expressions* (`result.myc:23–46`), and the generic method-desugar already
> produces the `m(recv, f)` call shape. The **only** residual is the **closure argument** when its
> parameter is `|()|`/`|_|`/untyped — the shapes DN-118 Phase-1 explicitly declined (single typed
> identifier only). The honest deliverable is therefore small and precise: **a combinator-directed
> match-inline** that moves the unmappable lambda-param into a match-pattern where `_` and unit destructure
> natively — not a new combinator library, not a unit-type lever, not a type-inference pass.

---

## §1 The problem, precisely

`crates/mycelium-std-sys-host/src/lib.rs:27–30` — the production `EntropySource` adapter — carries the
residual verbatim:

```rust
mycelium_std_sys::rand::fill_bytes(buf)
    .map(|()| EntropyEffect)
    .map_err(|_| RandErr::EntropyUnavailable)
```

Two Result combinators, each taking a closure whose parameter has **no** `lambda`-parameter surface in
Mycelium's grammar:

| # | Rust closure | Parameter shape | Why DN-118 Phase-1 declines it |
|---|---|---|---|
| a | `\|()\| EntropyEffect` | **unit pattern** (`Pat::Tuple` empty) — binds a `()` payload it ignores | `lambda`'s `param ::= Ident ':' type_ref` has no unit-destructure param; the unit type `()` itself has *no* Mycelium literal/type in this grammar fragment (`emit.rs:2210–2214` gaps `()`; M-826: tuple arity ≥ 2). |
| b | `\|_\| RandErr::EntropyUnavailable` | **wildcard pattern** (`Pat::Wild`), untyped | `lambda`'s param must be a single **explicitly-typed identifier** (`emit.rs:2526–2549`); `_` is not an identifier and carries no type, and this transpiler has **no type-inference pass** to recover it (DN-118, VR-5: absence, never a guess). |

The underlying **problem** each solves: *transform the success (or error) branch of a fallible value by a
pure function that **ignores** its argument (a constant function) — `map` to a constant, `map_err` to a
constant error.* This constant-function-over-a-fallible-value idiom is pervasive across the corpus port
surface (it is a large slice of the DN-34 §8.22 "Other" gap class), which is why it is worth a native
ruling rather than a per-site gap.

**What is NOT the gap (verify-first, mitigation #14).** Confirmed against `dev @ 08d8fc21`:

1. **The combinator surface exists and runs.** `lib/std/result.myc:23,29,39,45` define `map`, `and_then`,
   `map_err`, `or_else`; `:33` `fold`; `:18` `unwrap_or` — all as native free functions **whose bodies
   are `match`** (`map(r,f) = match r { Ok(x) => Ok(f(x)), Err(e) => Err(e) }`). `lib/std/option.myc:36–58`
   mirrors them for `Some(A) | None`. HOF combinators run three-way (M-685/687/688, `Empirical`).
2. **The call shape already lowers.** `emit.rs:2179–2189` desugars `recv.map(f)` → `map(recv, f)`
   (the grammar has no postfix method form; the stdlib free functions take the receiver first). So a
   combinator call over a *lowerable* closure already emits today (DN-118 Phase-1 handles the
   single-typed-identifier case, e.g. `.map(|x: Binary{8}| add(x, 1))`).
3. **`match`-on-a-call scrutinee and nested `match` are legal, used stdlib idioms.** `lib/std/cmp.myc:86`:
   `match eq(a,b) { 0b1 => Eq, _ => match lt(a,b) { … } }` — a `match` whose scrutinee is a call, with a
   nested `match` arm, both `myc check`-clean.

So the residual is **exactly** the closure-argument param shape — not the combinator, not the call form,
not `match`.

**The Mycelium-native answer (DN-111 / DN-110 taxonomy).** The combinator is a **Native Equivalent**
(`map(r,f)` *is* the native form). The closure-param residual is an **Idiomatic Remapping** (DN-111 §3.2 /
DN-110 "Solution"): the key observation is that **`|()|` and `|_|` have no native `lambda`-parameter
surface but a *perfect* native `match`-pattern surface** — `_` is the wildcard pattern (used throughout
`result.myc`) and a unit payload destructures to `_`. So inlining the combinator's own `match` body, with
the closure body substituted for `f(x)` and the closure param lowered as the arm's **binder pattern**,
*relocates* the unmappable construct into a position where Mycelium expresses it natively. Concretely:

```
fill_bytes(buf).map(|()| EntropyEffect).map_err(|_| RandErr::EntropyUnavailable)
```

lowers to (inlining `map`'s and `map_err`'s stdlib bodies, `Ok`/`Err` from `result.myc:10`):

```mycelium
match (match fill_bytes(buf) { Ok(_) => Ok(EntropyEffect), Err(e) => Err(e) }) {
  Ok(x)  => Ok(x),
  Err(_) => Err(RandErr.EntropyUnavailable)
}
```

— every construct here is already active grammar (`match`, `Ok`/`Err` ctors, wildcard patterns, nested
`match`), and this is *definitionally* what `result.myc`'s `map`/`map_err` mean (§1.1 above), so it is
maximally faithful (a beta-reduction of the stdlib combinator), zero kernel growth (KC-3).

---

## §2 Alternatives (real, ranked)

Three genuinely different lowerings for the closure-param residual (the combinator surface is fixed — it
already exists):

**Alt A — Combinator-directed match-inline (RECOMMENDED).** On a confirmed Result/Option receiver, inline
the combinator's own `match` body with the closure body substituted and the closure param lowered as the
arm binder (`_` for wildcard/unit, the identifier otherwise). No param type needed. Closure-literal args
only; function-value args keep `m(recv, f)`.

**Alt B — Combinator-directed lambda-param typing.** Keep the `m(recv, f)` call; recover the closure
param's type from the *combinator signature × receiver type* (`map`'s `f: A => B` ⇒ param type `A` = the
receiver's `Ok`-type), then emit `lambda(_p: A) => body` (a throwaway `_p` binder for `_`). This is a
*targeted* inference (not general), so it is honest — but it **cannot lower `|()|`**: `A = ()` and there is
no Mycelium unit type/`lambda(_p: Unit)` surface (would require a new unit-type language lever). So Alt B
is *partial by construction* — it closes `|_|` but not `|()|`, i.e. not the `std-sys-host` case.

**Alt C — Status quo (per-site gap).** Leave every `|()|`/`|_|` combinator argument an honest gap. Zero
work, zero risk, but closes nothing — `std-sys-host` stays ≤ 5/6 and the corpus "Other" class does not move.

### §2.1 Objective function (criteria table)

Weights reflect the mandate: a **native** solution first, then faithfulness, kernel cost, and the DN-126
two-mode invariant.

| Criterion (weight) | Alt A (match-inline) | Alt B (lambda-param typing) | Alt C (gap) |
|---|---|---|---|
| **Closes `std-sys-host` residual #1 — `\|()\|` *and* `\|_\|`** (×3) | **Yes — both** (3) | Partial — `\|_\|` only, **not `\|()\|`** (1) | No (0) |
| **Native / idiomatic (DN-111)** (×3) | **Native — inlines the stdlib combinator's own `match`** (3) | Native call + synthesized `lambda` (2) | n/a (0) |
| **Kernel cost (KC-3)** (×2) | **Zero — reuses `match` + ctors** (2) | Zero for `\|_\|`; **needs a unit-type lever** for `\|()\|` (0) | Zero (2) |
| **Faithfulness / VR-5** (×2) | **Beta-reduction of the stdlib def — exact** (2) | Faithful where it fires (2) | n/a (2) |
| **Two-mode invariance (DN-126)** (×2) | **Mode-invariant — no param type used** (2) | **Diverges** — needs the type strict mode has and loose mode may defer (0) | n/a (1) |
| **Corpus "Other" reach** (×1) | Broad — every combinator-over-closure-literal (1) | Broad but blocked on the `\|()\|`/unit slice (0.5) | None (0) |
| **Weighted total** | **28** | 12.5 | 8 |

**Recommendation (ranked): Alt A ≫ Alt B ≫ Alt C.** Alt A wins decisively: it is the *only* option that
closes the actual `std-sys-host` residual (`|()|` included), it is the most native (it *is* the stdlib
combinator's own body), it grows nothing, and — the load-bearing DN-126 point — it needs **no** parameter
type, so it lowers **identically in loose and strict mode**, whereas Alt B's synthesized `lambda(_p: A)`
depends on a type that the two modes treat differently. Alt B is retained **only** as the fallback shape
for a *function-value* argument (`.map(SomeFn)`), where there is no body to inline — but that already works
via the unchanged `m(recv, f)` call, so it needs no new code.

---

## §3 The emission spec (what M-1092 builds)

A **combinator-directed pass** consulted inside `visit_method_call` (`emit.rs:2113`) **before** the generic
desugar (mirroring the `prim_map` consultation order, `emit.rs:2120–2155`):

1. **Recognize** the method name ∈ the combinator set `{map, map_err, and_then, or_else, fold, unwrap_or}`
   (the `result.myc`/`option.myc` surface). Any other name → unchanged path.
2. **Gate on a confirmed receiver type** — `expr_env_type(&m.receiver)` ∈ `{Result, Option}` (the exact
   `prim_map::receiver_gate_matches` discipline). **Unknown or non-Result/Option** ⇒ **fall through** to the
   unchanged generic desugar (which may itself gap) — never guess `Ok`/`Err` (VR-5/G2). *(This is why a
   real port, whose `std.sys.rand.fill_bytes` signature is in-program, resolves cleanly where bare
   cross-crate vet profiling of the Rust file may not — see §5 honesty boundary.)*
3. **Split on the argument shape:**
   - **Closure literal** (`Expr::Closure`, single param, value-safe per DN-118's D5/D7 gate) ⇒ **inline**:
     emit the combinator's stdlib `match` body (per-combinator arm template, below), substituting the
     closure body for the `f(x)` position and lowering the closure param as the arm binder — **identifier
     `x`** → `Ok(x)`/`Err(x)`; **wildcard `_` or unit `()`** → `Ok(_)`/`Err(_)`.
   - **Function value** (a path/other expr) ⇒ unchanged `m(recv, f)` free-function call (Alt B's residual
     role — no new code).
   - **Multi-param / value-unsafe closure** ⇒ inherit DN-118's existing never-silent gap unchanged.
4. **Per-combinator arm templates** (from `result.myc`/`option.myc`, so the desugar cannot drift from the
   library semantics — DRY): `map` → `{ Ok(⟨p⟩) => Ok(⟨body⟩), Err(e) => Err(e) }`; `map_err` →
   `{ Ok(x) => Ok(x), Err(⟨p⟩) => Err(⟨body⟩) }`; `and_then` → `{ Ok(⟨p⟩) => ⟨body⟩, Err(e) => Err(e) }`;
   `or_else` → `{ Ok(x) => Ok(x), Err(⟨p⟩) => ⟨body⟩ }`; `unwrap_or(fallback)` →
   `{ Ok(x) => x, Err(_) => ⟨fallback⟩ }` (fallback is an ordinary expr, not a closure); `fold(on_ok,
   on_err)` → the two-arm eliminator. `Some`/`None` variants for `Option`.
5. **Chains nest** (`.map(..).map_err(..)`): the outer combinator's receiver is the inner combinator's
   emitted `match` expression — a nested `match` scrutinee, legal per `cmp.myc:86`.

**Guarantee tag:** `Declared` until built; **`Empirical`** once the emitted `.myc` is `myc check`-clean and
differential-witnessed (three-way, DN-26) on the `std-sys-host` body + a fixture table; **no `Proven`
claim** (VR-5).

---

## §4 Two-mode typing (DN-126) — the mode-invariance argument

DN-126 splits type-resolution into **loose** (the checker in a non-refusing posture over the unchanged
evaluator — a *type-level* refusal demotes to a DN-04 flag) and **strict** (compile-gating). Alt A's
match-inline **uses no parameter type at all** — the closure param becomes a `match` binder pattern, whose
type the checker infers from the scrutinee's constructor in *both* modes. So the desugar's *text* is
identical under loose and strict typing, and it never sits on the runnable-floor boundary DN-126 §3.3 keeps
hard (name/arity/parse), only on the type-level surface the two modes agree on for a bound pattern. Alt B,
by contrast, must *write down* the param type `A`, which strict mode requires and loose mode may leave to
the DN-04 residual channel — so Alt B's output would differ by mode. This is an independent argument for
Alt A beyond the `|()|` blocker, and a small confirming data point for DN-126's "reuse the checker, grow no
type theory" thesis.

---

## §5 Adversarial stress-test (VR-5 / house rule #4)

Run the recommendation through the sequences that would break it:

1. **Iterator `.map` false-fire.** `v.iter().map(|x| …)` must NOT get the Result desugar. **Held:** the
   receiver gate (§3.2) fires only on a *confirmed* `Result`/`Option` receiver; an iterator receiver falls
   through to the generic desugar unchanged. The gate is the same no-guess mechanism `prim_map` already
   uses — the receiver-type gate `receiver_gate_matches` (consulted at `emit.rs:2120–2123`, defined at
   `prim_map.rs:228`); and the `prim_map` module doc (`prim_map.rs:25–29`) is a sibling precedent for the
   same decline-don't-guess discipline — it *declines* to map `.checked_mul` for a **value-shape** mismatch
   (`Option[Binary{N}]` vs bare `Binary{N}`), never fabricating a body.
2. **Cross-crate receiver whose type is unknown.** In *bare vet profiling* of the Rust file,
   `mycelium_std_sys::rand::fill_bytes(buf)`'s `Result` return may not resolve in `expr_env_type` ⇒ the
   gate honestly **gaps** (never fabricates `Ok`/`Err`). **This is a bounded-faithfulness point, stated
   plainly (not hidden):** the desugar is clean on the **real 6/6 port path** (where `std.sys.rand.fill_bytes`
   is an in-program `.myc` signature the checker knows), and honestly-gated under standalone cross-crate
   vet profiling. It is *not* wrong in either case — it either emits the faithful `match` or gaps
   never-silently. Recovering the cross-crate return type under bare profiling is a *separate* env-typing
   improvement, not a correctness dependency of this note.
3. **Capture scope.** Inlining substitutes the closure body at the call site — the *same* lexical scope the
   closure occupied — so captured `let`/param names stay in scope (DN-118's model: captures are ordinary
   in-scope references). **Held:** no env record, no scope change; the binder pattern shadows correctly
   within its arm.
4. **`FnMut`/mutation.** DN-118 D5/D7 flags a closure that mutates a non-parameter capture. **Held and
   *strengthened*:** the DN-118 value-safety gate is applied *before* inlining unchanged; and a *single-use*
   inlined body has no "across calls" surface at all (there is no reified closure value to snapshot), so
   inlining is *strictly safer* than emitting a `lambda`, never less safe.
5. **Evaluation order.** `r.map(f)` evaluates `r` once; the emitted `match r { … }` also scrutinizes `r`
   once. Mycelium is value-semantic (ADR-003) with effects carried in types (`DeclaredTime*`), so there is
   no effect-ordering divergence. **Held.**
6. **Named-function argument.** `.map(parse_row)` — no body to inline. **Held:** falls to the unchanged
   `map(recv, parse_row)` free-function call (named-fn-as-value already runs, RFC-0024 §4A).
7. **Chain depth / readability.** A long `.map().map_err().and_then()` chain nests `match` scrutinees. This
   is legal (`cmp.myc:86`) but can read densely. **Verdict:** accept for v1 (it is exactly the semantics);
   an optional `let`-binding form (`let t0 = match … in match t0 { … }`) is an OQ (§6), not a blocker.
8. **Disconfirming-evidence check against the maintainer's framing.** The task framed residual #1 as "no
   `map` primitive on Result" and "an unscoped combinator-surface decision." **The evidence disconfirms the
   surface framing:** `map`/`map_err`/`and_then` *do* exist natively (`result.myc:23–46`) and run — so this
   is **not** an unscoped combinator-surface decision; it is a *scoped closure-param lowering*. Reporting
   that correction is the point (house rule #4: follow the evidence, not the phrasing). The combinator
   surface needs no new design; only the match-inline does.

**Verdict:** Alt A survives every sequence. Its one honest bound (stress #2) is a *never-silent gap under
bare profiling*, not an incorrectness, and it does not affect the real 6/6 port path.

---

## §6 Open questions (never-silent)

- **OQ-1 — `let`-bind vs. nest for chains.** Whether a ≥ 3-deep combinator chain should emit nested `match`
  scrutinees (v1) or a `let`-sequence for readability (DN-106 statement-sequencing territory). No
  correctness impact; deferred to a measured readability call.
- **OQ-2 — `ok_or`/`ok_or_else` (Option → Result) and `?`-adjacent combinators.** Not in `option.myc`'s
  current surface; if a port driver needs them they are a *combinator-surface* addition (a new `.myc`
  function + its arm template), distinct from this note's lowering. Named-trigger, not built here.
- **OQ-3 — cross-crate receiver typing under bare vet profiling** (stress #2). A future `expr_env_type`
  improvement to resolve known-signature cross-crate return types; independent of this note's correctness.

---

## §7 Definition of Done (the gate for M-1092; what "Accepted" then "Enacted" require)

**For this DN to move Draft → Accepted** (maintainer / DN-review gate): the §2 ranked recommendation and
§5 stress-test are accepted on the merits, or amended; the §1 finding (combinator surface already exists;
residual is the closure param) is confirmed against the codebase.

**For M-1092 to be Done (Enacted basis):**

- **Recognize + gate:** the combinator-set match in `visit_method_call`, gated on a confirmed
  `Result`/`Option` receiver via `expr_env_type`/`receiver_gate_matches`; a non-confirmed receiver falls
  through unchanged (a test that `v.iter().map(..)` is untouched).
- **Inline:** the per-combinator arm templates (§3.4) emit the stdlib `match` body with the closure body
  substituted and the param lowered as `_` (wildcard/unit) or the identifier binder; closure-literal only.
- **Fallthrough:** a function-value argument keeps `m(recv, f)`; a multi-param/value-unsafe closure keeps
  DN-118's gap.
- **Never-silent tests:** iterator-`.map` untouched; unknown receiver → gap (no fabricated `Ok`/`Err`);
  `|()|` and `|_|` both lower on a confirmed receiver; a `.map().map_err()` chain nests correctly.
- **Witness:** `myc check`-clean on the `std-sys-host` `OsEntropy::fill_bytes` body and a fixture table;
  **differential-witnessed** (three-way, DN-26) before any `Empirical` upgrade past `Declared` (VR-5).

---

## §8 Build-leaf decomposition + the `std-sys-host` 6/6 path

- **Lane:** `crates/mycelium-transpile/src/emit.rs` — **serial** (single-owner file; the combinator pass is
  a new block in `visit_method_call`). One leaf: **M-1092** (`depends_on: []`; no shared-file or cross-DN
  dependency — the combinator surface it targets already exists).
- **Where it sits in 6/6:** `std-sys-host`'s two impl bodies carry the two residuals beyond the
  5-capability punch-list. **DN-135/M-1092 closes residual #1** (the `.map(|()|…).map_err(|_|…)` chain in
  `OsEntropy::fill_bytes`, `lib.rs:27–30`); **DN-134/M-1093 closes residual #2** (the struct-variant
  construction in `OsClock::wall_now`, `lib.rs:63–65`). Both are `emit.rs` serial-lane leaves; landed with
  the existing 5-capability punch-list they take `std-sys-host` to **6/6**. M-1092 and M-1093 touch
  *different* methods of the same file (`visit_method_call` vs. `visit_struct`) and can land in either
  order (no shared hunk).

---

## §9 Changelog

- 2026-07-12 — DN-135 created (**Draft**, renumbered from an initial DN-133 draft to resolve a
  DN-133 collision with the Qualified-Associated-Fn-Call DN). Scopes the Result/Option combinator-over-closure residual
  (`std-sys-host` #1 + corpus "Other" class); recommends **Alt A, the combinator-directed match-inline**
  (native, mode-invariant, zero kernel growth) over lambda-param typing (partial — cannot lower `|()|`) and
  status-quo gap; emission spec + adversarial stress-test + DoD; build leaf **M-1092**. `Declared` — builds
  nothing; the DN-review gate/maintainer ratifies. FLAGs Doc-Index/CHANGELOG/issues rows for the
  integrating parent.
- 2026-07-12 — **Draft → Accepted** (strict DN-review gate, 9-criterion clean pass). Pre-ratification
  citation corrections (grounding/consistency, verified against `dev @ 08d8fc21`): the receiver-gate
  reference `prim_map.rs:2121–2123` (an impossible line — `prim_map.rs` is 234 lines) corrected to the
  actual mechanism — consulted at `emit.rs:2120–2123`, `receiver_gate_matches` defined at `prim_map.rs:228`
  (§Decides item 3, §5.1); and the `.checked_mul` decline at `prim_map.rs:25–29` recharacterized from a
  "receiver-type mismatch" to its actual **value-shape** mismatch reason (§5.1). All decision-critical
  witnesses (`emit.rs:2113/2179–2189/2210–2214/2526–2549`, `result.myc:23–46`, `option.myc:36–58`,
  `cmp.myc:86`, `std-sys-host lib.rs:27–30`) verified exact. The design is unchanged; the decision is
  ratified, mechanisms stay `Declared` until M-1092 lands + is differential-witnessed (VR-5).
