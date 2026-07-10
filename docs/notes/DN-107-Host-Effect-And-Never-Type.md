# Design Note DN-107 — Host-Effect Real-Syscall Registry + the Never-Type (`-> !`) Divergence Model (the ENB-7 close)

| Field | Value |
|---|---|
| **Note** | DN-107 |
| **Status** | **Draft** (2026-07-10). A **design-reasoner** note working the M-1030 (ENB-7) decision **forward to a ranked recommendation, not a ratification**. It records the decision frame, the real alternatives for both sub-gaps, an evaluated ranked recommendation with a minimal v0 scope, and an adversarial stress-test; it **enacts nothing** and **moves no other doc's status** (house rule #3, append-only). Tags: `Empirical` for claims read against the tree at `5130badc`; `Declared` for every proposed-but-unratified design and every unproven soundness claim (VR-5 — nothing here is `Proven`). |
| **Decides** | *Proposes, for ratification (does NOT decide — house rule #3):* (1) the two ENB-7 sub-gaps are **separable**, coupled at exactly one point (`exit` is both a host op and a divergence); (2) **host-effects (#79)** land by **granting real `wild:read/write/get_env/exit` prims into the `PrimRegistry`** over the **already-ratified** RFC-0028 `wild`-block surface + the RFC-0014 `ffi` effect (no new representation) plus a **fixture/sandbox differential** — this sub-gap is design-closed, largely registry wiring; (3) **the never-type (#88)** is modelled as **divergence-as-effect with NO bottom `Ty`** (Rank 1): a new `diverges` effect on the RFC-0014 lattice, `wild:exit` carrying `!{ffi, diverges}`, a **narrow, never-silent checker admissibility rule** ("an *unconditionally-diverging* expression is well-typed at any expected type"), and an **honest totality-checker interaction** (`!diverges` = non-total, flagged, never silent); (4) a **nominal `Never` + `absurd` eliminator** is the Rank-2 **append-only upgrade path** held in reserve; a **true bottom subtype `⊥ <: T`** (Rank 3) is **rejected for v0** (it forces subtyping into a structural-equality checker); (5) a **FORK for the maintainer** — whether general-position `?` (DN-102 §6 FLAG-try-1) actually *depends* on M-1030 at all, since a CPS lift appears to need no bottom type (§6). It does **not** edit `issues.yaml`, `CHANGELOG.md`, or `Doc-Index.md` (the integrating session owns those). |
| **Feeds** | DN-99 §A6 (#79) + §A7 (#88) / §8 ENB-7 / register rows #79 (host-effect wild execution, `partial`) + #88 (never-type divergence `-> !`, `open`); M-1030; DN-102 §2/§5/§6 (the `?` desugar — the sole named downstream consumer, and the source of the "depends on the never-type" framing this note tests); RFC-0014 (declarative error recovery + **bounded effects** — the effect lattice this extends); RFC-0028 (`wild` blocks — the host-effect surface this reuses); DN-26 (SCC self-hosting, the Rust↔`.myc` dual); DN-34 §8 (surface-gap census). |
| **Grounds on** | KC-3 (small kernel — Rank 1 adds **no** `Ty` variant and **no** subtyping; it extends an existing effect lattice and adds one narrow admissibility rule), DRY (reuse RFC-0014 effects + RFC-0028 `wild` + the `wild:` prim naming — the host-effect representation already exists), G2 (never-silent — an out-of-domain host op, a non-total `diverges` function, a `?` outside its supported position each print the fix), VR-5 (no tag upgraded past its basis — every soundness claim below is `Declared`, earned `Empirical` only by the §8 witnesses), KISS/YAGNI (divergence-as-effect over a bottom type; the `let`-RHS `?` subset over a full CPS lift). |
| **Date** | July 10, 2026 |
| **Task** | M-1030 (ENB-7) — host-effect real-syscall registry + fixture differential, incl. the never-type `-> !`. |

> **Grounding + honesty (house rule #4 / VR-5 / G2 / no sycophancy).** This is a *design-evaluation*
> note: it enumerates, evaluates, and **recommends-ranked**, and it **argues against its own
> recommendation** (§7). It does **not** ratify (house rule #3 — the maintainer decides). Two honesty
> commitments bind it: **(a)** every soundness/expressiveness claim about an unbuilt design is `Declared`
> (there is no running increment here, unlike DN-102 — nothing is `Empirical` yet except the tree reads
> in §2); **(b)** it does **not** rubber-stamp the direction DN-99 §A7 and the M-1030 issue already
> sketch. §6 surfaces disconfirming evidence *against the task's own framing* — that general-`?` may **not**
> depend on the never-type at all — because rule #4 binds assent to merit, not to the speaker. Where the
> evidence does not let me confidently pick, §6/§7 name the **fork** and leave it to the maintainer.

---

## §1 Purpose + user stories

Close ENB-7 (DN-99 §8), which bundles two DN-99 register rows into one issue (M-1030):

- **#79 — host-effect wild execution (`partial`).** The `wild { name(args) }` surface exists
  (RFC-0028) and lowers to a `wild:name` prim, but the `PrimRegistry` does not yet grant the **real**
  `read`/`write`/`get_env`/`exit` syscalls, so a std-sys port cannot run against the OS.
- **#88 — never-type divergence `-> !` (`open`).** There is **no bottom/never type** in the kernel
  (verified §2), so a function that never returns a value (`exit`, `panic`, a non-terminating host op)
  cannot be typed, and any construct that wants a "this branch diverges" escape hatch has none.

**User stories.**

- *As a **std-sys author**, I want real host syscalls exposed as never-silent `wild:` prims (and `exit`
  modelled as divergence), so that `std.sys` ports run against the OS with an honest fixture witness* —
  the M-1030 issue's own story.
- *As a **semcore/stdlib porter**, I want a diverging expression (`exit(1)`, an `unreachable`/`panic`
  shim) to be well-typed in a value position (a `match` arm, an `if`/`else` branch), so that I can port
  a Rust function whose one arm bails out without hand-restructuring it into a total shape.*
- *As a **language maintainer**, I want divergence tracked **visibly** (never silent — G2) and **without
  bloating the kernel** (KC-3), so that "this function may not return" is an inspectable, `EXPLAIN`-able
  property rather than an invisible checker special-case.*

**Definition of Done — see §8.** The DoD names exactly what a maintainer ratification requires.

## §2 The decision frame — separable, coupled at one point (verified against the tree)

The first question a reasoner must answer is *what M-1030 actually requires* and *whether #79 and #88
are one decision or two.* Read against the tree at `5130badc` (`Empirical`):

1. **The host-effect representation already exists** — this is not a design decision, it is registry
   wiring. RFC-0028 `wild` blocks parse and lower (`elab.rs:1523`: `format!("wild:{name}")` →
   `Node::Op`); the effect system (RFC-0014) already treats a `wild` block as **performing the `ffi`
   effect** (`checkty.rs:3265` — `Expr::Wild(_) => performed.insert(("ffi", EffectSource::Wild))`), and
   an undeclared effect is a never-silent `CheckError` (`checkty.rs:3281`). So **#79 is `partial`, not
   `open`**: the surface + effect + prim-naming are ratified; the residual is (a) *granting the real
   syscall prims* into the `PrimRegistry` and (b) a *fixture/sandbox differential* (an equality
   differential is defeated by real, non-deterministic syscalls).
2. **The never-type genuinely does not exist** — this *is* an open design decision. `checkty.rs:78`
   `enum Ty { Binary, Ternary, Dense, Vsa, Data, Substrate, Seq, Bytes, Float, Var, Fn }` has **no**
   `Bottom`/`Never`/`Diverge` variant; `ast.rs:571` `enum BaseType { … }` likewise. The checker unifies
   by **structural equality** (`A ~ C`), **not subtyping** — there is no `<:` relation anywhere in
   `checkty.rs` to hang a bottom on. This confirms DN-99 #88 and the M-1030 body ("verified `ast.rs` has
   no bottom `BaseType`").
3. **The coupling is exactly one point: `exit`.** `exit` is *both* a host op (#79 — a `wild:` prim) and
   a divergence (#88 — it never returns). Every other host op (`read`/`write`/`get_env`) returns a value
   and needs no divergence; divergence proper (`panic`, `loop {}`, a non-terminating op) needs no host
   syscall. So the two sub-gaps are **separable decisions that meet only at `exit`** — and the elegant
   move DN-99 §A7 already spotted is to let that one meeting-point ride the *same* mechanism: model
   `exit` as a divergent host-effect prim, so **divergence is just another effect on the lattice the
   host-effects already use.** That is the seed of the Rank-1 recommendation (§5), but §3/§7 test it
   rather than assume it.

**Verdict:** treat M-1030 as **one host-effect implementation task (#79, design-closed) + one genuine
design decision (#88, the never-type)**, joined at `exit`. The rest of this note is mostly about #88.

## §3 The real alternatives — the never-type (#88)

| # | Alternative | Mechanism | Kernel cost | Verdict |
|---|---|---|---|---|
| **N-A** | **True bottom subtype `⊥ <: T`** (Rust's `!`) | Add `Ty::Never`; make it a subtype of every type so a diverging expression unifies with any expected type. | **Large.** Introduces **subtyping** into a checker that is today *pure structural equality* — every `unify`/`ty_eq` site must now consider `⊥ <: T`, and coercion/exhaustiveness/`Fn`-variance all inherit it. A pervasive KC-3 violation. | **Reject for v0.** Only defensible if the language later adopts subtyping wholesale — a separate, much larger decision. |
| **N-B** | **Distinguished nominal `Never`** (an uninhabited `Data` type) + an `absurd : Never -> T` eliminator | `type Never` = (zero constructors); a diverging fn returns `Never`; the desugar inserts `absurd` (ex-falso) where a `Never` must flow into a `T` position; the empty match `match e {}` is well-typed when `e : Never`. | **Moderate.** No subtyping — reuses `Ty::Data`. But needs: an *uninhabited-type* notion, the empty-match being exhaustive/well-typed, and the checker/desugar to **insert `absurd`** at each use site. | **Rank 2 — the reserve/upgrade path.** Append-only over Rank 1: if a first-class bottom is ever *witnessed* (a `Never`-typed field, higher-order divergence passing a diverging fn where `A -> B` is expected), escalate to N-B. Not needed for the v0 gap (YAGNI). |
| **N-C** | **Divergence-as-effect, NO bottom `Ty`** (DN-99 §A7) | Add a `diverges` effect to the RFC-0014 lattice; `wild:exit` (and any non-returning op) carries `!{ffi, diverges}`; a **narrow checker rule**: an *unconditionally-diverging* expression is admissible at **any** expected type; the totality checker treats `!diverges` as honestly **non-total** (flagged, never silent). | **Smallest.** **No** `Ty` variant, **no** subtyping. Extends an existing effect lattice (DRY) and adds **one** admissibility rule at the expected-type check. | **Rank 1 (recommended).** Smallest blast radius; reuses the mechanism #79 already uses; honest divergence tracking (`EXPLAIN`-able effect). Its soundness surface is the subject of §7. |
| **N-D** | **Defer entirely** (keep `-> !` a never-silent refusal) | No change; `exit`/`panic` stay unmodelled; the `->!` position stays refused. | **Zero.** | **Reject** as the *general* answer — it leaves `exit` and every std-sys IO-then-exit shape unportable. **But** honestly viable *for the never-type specifically* **if** the maintainer resolves the §6 fork toward "general-`?` does not need it and no port yet needs `exit`-in-value-position." |

## §4 The real alternatives — host-effects (#79)

| # | Alternative | Mechanism | Verdict |
|---|---|---|---|
| **H-i** | **Effect row — extend RFC-0014's `!{…}`** (the existing mechanism) | Grant real `wild:read/write/get_env/exit` prims in the `PrimRegistry`; each carries the `ffi` effect (`exit` additionally `diverges`, §3 N-C); add a fixture/sandbox differential for the non-deterministic ops. | **Recommended.** The representation exists (RFC-0028 + RFC-0014); this is registry wiring + a test harness, **not** a new mechanism. DRY, KC-3. |
| **H-ii** | **Capability parameter** (object-capability) | Thread an explicit capability/handle argument through every IO function. | **Reject for v0.** A large new mechanism; over-reaches the gap (YAGNI). More principled for ADR-032 *certified* mode — record as a *future* direction, not this close. |
| **H-iii** | **Marker trait** (`IO`/`Host` bound) | Bound effectful functions with a marker trait. | **Reject.** Redundant with RFC-0014 effects — **two** effect systems (DRY violation). |
| **H-iv** | **Defer** | Keep `wild` partial. | **Reject.** #79 is *partial*, not open; deferring leaves std-sys ports unable to touch the OS for no design saving. |

## §5 Recommendation — RANKED (a Draft for the maintainer, not a ratification)

**Objective function.** Rank by, in order: **(1)** unblocks the gap (host `exit`/IO portable; honest
divergence modelling); **(2)** KC-3 smallest kernel / smallest blast radius; **(3)** DRY (reuse over
invent); **(4)** never-silent + honest tags (G2/VR-5); **(5)** append-only upgrade path preserved.

| Criterion (weight) | N-A bottom `⊥` | **N-C effect (rec.)** | N-B nominal `Never` | N-D defer |
|---|---|---|---|---|
| Unblocks the gap | ✅ | ✅ | ✅ | ❌ (never-type) |
| KC-3 / blast radius | ❌ subtyping everywhere | ✅ one rule, no `Ty` | ◐ new type + `absurd` insertion | ✅ (nothing) |
| DRY (reuse) | ❌ new relation | ✅ extends RFC-0014 | ◐ reuses `Data` | ✅ |
| Never-silent / honest | ◐ | ✅ `diverges` is `EXPLAIN`-able | ✅ | ✅ |
| Upgrade path preserved | — | ✅ → N-B if witnessed | ✅ | ✅ |

**Rank 1 (recommended) — divergence-as-effect (N-C) + host-effects via H-i.** The v0 scope, minimal
enough to land (KISS/YAGNI):

1. **Host-effects (H-i, #79).** Grant real `wild:read`, `wild:write`, `wild:get_env`, `wild:exit` prims
   in the `PrimRegistry` (from `mycelium-std-sys`); keep the `@std-sys` context gate (`Cx::check_wild`)
   and the `ffi` coverage check unchanged. Add a **fixture/sandbox differential** (golden-trace, not
   equality) for the non-deterministic ops. Out-of-domain host op ⇒ never-silent error (G2).
2. **Divergence effect (N-C, #88).** Add one effect, `diverges` (name TBR — `div`/`noreturn` are
   candidates; §7), to the RFC-0014 lattice. `wild:exit` carries `!{ffi, diverges}`.
3. **The admissibility rule.** In the expected-type check, an **unconditionally-diverging** expression —
   a call to a `!diverges` function, or `wild:exit` directly — is well-typed at **any** expected type.
   This is the *entire* type-level footprint: no `Ty` variant, no subtyping, no unification change.
4. **Totality interaction (honest, never silent).** The totality checker treats a `!diverges` function
   as **non-total** — it does not satisfy "produces a value for all inputs," and that is **recorded and
   flagged**, never silently accepted as total. `diverges` is the *visible* marker of a function that
   may not return.
5. **Honesty tags.** The whole scheme is `Declared` until §8's witnesses run; the *soundness* of the
   admissibility rule rests on `wild:exit` **never returning**, which is `Declared` (no theorem) and
   earns `Empirical` from a fixture — **never `Proven`** (VR-5).

**Deferred out of v0 (YAGNI):** a first-class `Never` *type* (N-B — held in reserve); divergence at any
*conditional* site beyond the recognised syntactic forms; `panic`-with-unwinding (v0 `exit` aborts;
unwinding is a separate runtime-tier decision); the general-`?` CPS lift (DN-102 §6 FLAG-try-1 — see the
§6 fork).

**Rank 2 — nominal `Never` + `absurd` (N-B).** The append-only escape hatch. Adopt only when a port
*witnesses* a genuine first-class-bottom need (a `Never`-typed field, higher-order divergence). N-C → N-B
is a clean additive upgrade (the `diverges` effect stays; a `Never` *type* is added alongside), so
choosing N-C now forecloses nothing.

**Rank 3 — true bottom subtype (N-A).** Rejected for v0; only revisit if subtyping is adopted
language-wide (a far larger, separate decision).

## §6 The FORK — does general-`?` actually depend on the never-type? (disconfirming evidence, surfaced)

The task's framing (and DN-102's) is that this note "unblocks the general-position `?` try-operator CPS
lift, M-1025's deferred residual, [which] depends on the never-type existing." **Rule #4/VR-5 obliges me
to test that claim rather than assent to it — and on analysis I cannot confirm it; I find the opposite
more likely.** This is a genuine fork; I flag it for the maintainer.

DN-102 §2 shows the *local* desugar of a bare `e?` is ill-typed because `Err(err) => Err(err)` (type
`Result[_,E]`) will not unify with `Ok(x) => x` (type `A`) — in Rust the `Err` arm **diverges** and `!`
unifies with `A`. DN-102 resolves the **`let`-RHS** case by putting the continuation *inside* the binding
arm (monadic bind), so both arms are `typeof(body) = Result[B,E]` — **no never-type**. It then defers the
**general** position (`g(f()?)`) to a "CPS lift," and (via #88) frames that lift as needing the
never-type.

**But the CPS lift, worked through, appears not to need a bottom type either.** `g(f()?)` in a
`Result[B,E]`-returning function lifts to:

```
match f() { Ok(x) => Ok(g(x)), Err(e) => Err(e) }        // both arms : Result[B, E]
```

The continuation (`g(·)` wrapped back to `Result`) is threaded into the `Ok` arm; the `Err` arm is
`Result[B,E]`; the two **unify by ordinary structural equality — no `⊥`, no divergence.** The never-type
was Rust's device for a *local, in-place, early-returning* desugar; **CPS is the alternative that
*replaces* the never-type, not a transformation that *depends* on it.** So my reading:

- **General-`?` is unblocked by *implementing the CPS lift*** (a checker/desugar transformation over
  arbitrary expression shapes — real work, but bottom-type-free), **not** by adding a never-type.
- **The never-type's real consumers are divergence proper** — `exit`/`panic`/non-returning host ops in a
  value position (#88) — **not `?`.**

I am **confident (`Empirical`, from the DN-102 desugar + the §2 tree reads)** that the *simple* CPS lift
above needs no bottom type. I am **not certain (`Declared`)** that *no* corner of a fully-general CPS
lift over every expression shape needs one (e.g. threading a continuation through a construct that itself
can diverge). **So the fork for the maintainer:**

- **(6-a)** If general-`?` is judged **independent** of the never-type (my lean), then M-1030 should be
  scoped to *divergence + host-effects only*, and general-`?` becomes its own follow-up (a CPS-lift
  wave) that does **not** wait on this note. DN-102 §6 FLAG-try-1's "gated on the never-type" line should
  then be **relaxed** (append-only correction, not a rewrite).
- **(6-b)** If the maintainer *wants* general-`?` lowered by synthesising a *diverging propagation*
  (reusing N-C's admissibility rule instead of a full CPS lift), then it **does** couple to this note —
  a legitimate, possibly-simpler design, but a different one than DN-102's CPS framing.

I cannot confidently pick between "implement CPS (no dependency)" and "lower `?` via divergence (couples
here)" without the CPS-lift design in hand — **maintainer's call.** Either way, the **never-type is worth
having for `exit`/`panic`** regardless of how `?` is ultimately generalised, so Rank 1 stands.

## §7 Adversarial stress-test (argue against the recommendation — VR-5/§rule 4)

Run Rank 1 through the sequences that break it. Surviving concerns fold into §8-open/§10-FLAGs.

1. **The "hidden bottom" objection (the strongest skeptic point).** *"N-C spells a bottom type as an
   effect + a special admissibility rule. A bottom in `Ty` is one inspectable place; a rule scattered in
   the checker is **less** auditable, not more — you've hidden the bottom, not avoided it."* **This is
   legitimate and I concede the framing:** N-C *is* morally a bottom type. My rebuttal is about **blast
   radius, not morality**: a `Ty::Never` with `⊥ <: T` (N-A) forces **subtyping** into every `unify`
   site — pervasive; N-C confines the whole thing to **one** admissibility check + one effect + one
   totality rule, introducing **no subtyping**. So N-C is smaller *in the checker* even though it is
   "morally bottom." N-B (nominal `Never` + `absurd`) is the middle path that *does* make the bottom a
   single inspectable `Ty` **without** subtyping — which is exactly why it is Rank 2, the upgrade path if
   the "auditability of a named type" wins over "one fewer type." **Recorded as OPEN-1.**
2. **Soundness rests on an unproven premise.** The admissibility rule is sound **only if**
   `wild:exit` *provably never returns a value*. It calls `process::exit`/`libc::exit`, so this is
   **`Declared`** (no theorem) and at best **`Empirical`** (a fixture that observes non-return) — **never
   `Proven`.** If any op tagged `diverges` *could* return, a value of the wrong type flows and the
   checker is unsound. So the `diverges` tag is **load-bearing for soundness** and must be grant-time
   controlled (only genuinely-non-returning prims may carry it), never user-assertable in v0. **OPEN-2.**
3. **"Unconditionally-diverging" is a real analysis, and its edges are the danger.** The rule admits an
   *unconditionally*-diverging expression at any type. A *conditionally*-diverging one
   (`if c then exit() else 3`) must **not** get the free pass at an arbitrary type — the whole `if` is
   `Binary{…}` and only the `exit()` sub-term diverges. So the checker needs a precise
   **diverges-detection judgement** (which syntactic forms are unconditional divergences: a direct
   `wild:exit`, a call to a `!diverges` fn in tail position, a `match` all of whose arms diverge). Get
   this scope wrong and it is either unsound (admits too much) or useless (admits too little). This is
   the genuine implementation risk of N-C. **OPEN-3.**
4. **Value-semantics / affine interaction — checked, not a hole.** A bottom value never exists at
   runtime, so no linearity/affine (LR-8 `Substrate`) obligation can be violated by it — you cannot
   consume a value that is never produced. This is *vacuously* fine under N-A/N-B/N-C alike. **Not a
   concern**, but stated so the reader need not wonder.
5. **Exhaustiveness interaction.** N-C touches exhaustiveness only at the empty match (`match e {}`),
   which v0 **does not add** (that is an N-B feature). Under N-C, `exit()` sits in an *arm value*, not a
   scrutinee, so Maranget usefulness (the existing W7 checker) is untouched. N-B *would* need `match e {}`
   to be exhaustive when `e : Never` — a reason N-B is heavier, and correctly Rank 2.
6. **Over-reach check.** N-C is the *minimal* option — it cannot over-reach. The live risk is the
   opposite: N-C **under**-reaches if a first-class bottom is genuinely needed later — which is precisely
   why N-B is pre-identified as the append-only upgrade (no rework: the `diverges` effect survives).
7. **Non-determinism defeats the equality differential (#79).** Real `read`/`write`/`get_env` return
   host-dependent values, so the DN-102-style *equality* differential (`.myc` ≡ Rust oracle) cannot
   witness them. Hence the **fixture/sandbox golden-trace** differential in the v0 scope — a *different*
   witness shape, honestly `Empirical`-per-fixture, never a claimed equality it cannot have. **OPEN-4.**

## §8 Definition of Done (this note's gate — what "Accepted" requires of the maintainer)

A maintainer ratifying DN-107 → Accepted confirms:

1. **The frame (§2):** #79 is `partial` (representation exists; residual = real prims + fixture
   differential) and #88 is genuinely `open` (no bottom `Ty`), coupled only at `exit` — as read against
   the tree.
2. **The host-effect decision (§4 H-i):** real `wild:` prims ride RFC-0028 + RFC-0014, with a
   fixture/sandbox differential; capability-param/marker-trait are rejected for v0 (recorded, not
   silently dropped).
3. **The never-type decision (§5):** Rank 1 = divergence-as-effect (N-C, **no bottom `Ty`**); N-B is the
   named append-only upgrade path; N-A is rejected for v0; and the honest totality interaction (`!diverges`
   = non-total, flagged) is accepted.
4. **The §6 fork is decided:** either general-`?` is **independent** of the never-type (relax DN-102 §6
   FLAG-try-1, append-only) **or** `?` is to be lowered via divergence (couples here) — the maintainer
   picks; this note does not.
5. **The open questions (§7 OPEN-1…4) are accepted as the risk register** for the implementing wave,
   with the soundness tag pinned `Declared`/`Empirical` (never `Proven`).
6. **Then, for the *implementing* increment (a separate landing, not this note):** one real syscall
   end-to-end + a fixture/sandbox witness; `exit`/divergence modelled with the totality interaction; a
   never-silent out-of-domain host-op refusal; a conformance **reject** witness for a `diverges` tag on a
   returning op and for a conditionally-diverging expression at the wrong type (OPEN-2/OPEN-3). The
   agreement claims upgrade to `Empirical` only when those witnesses run (VR-5).

Status stays **Draft** until the maintainer ratifies — the reasoner does not self-ratify (house rule #3/#4).

## §9 Grounding

- **KC-3 / small kernel:** Rank 1 adds **no** `Ty` variant and **no** subtyping relation — one effect on
  an existing lattice + one admissibility rule + one totality rule. N-A (subtyping) and N-B (new type +
  `absurd`) are the heavier options, correctly ranked below.
- **DRY:** host-effects reuse RFC-0028 `wild` + the RFC-0014 `ffi` effect + the `wild:` prim naming
  (`elab.rs:1523`) already in the tree; divergence reuses the **same** effect lattice — one mechanism,
  not three (rejecting the marker-trait H-iii on exactly this ground).
- **G2 (never-silent):** an out-of-domain host op, a `!diverges` function's non-totality, a `diverges`
  tag on a returning op, and a `?` outside its supported position are each an explicit refusal/flag with
  the fix — never a silent coercion or a silent "may not return."
- **VR-5 (no upgraded tag):** every design claim here is `Declared`; the §2 tree reads are `Empirical`;
  the admissibility rule's soundness is `Declared`/`Empirical`-by-fixture, **never `Proven`** — there is
  no discharged theorem that `wild:exit` never returns.
- **DN-99 §A6/§A7 + §8 ENB-7 / rows #79 + #88:** this note is the Draft DN those rows call for; it
  **evaluates** §A7's sketch (divergence-as-effect, no bottom type) rather than merely restating it, and
  adopts it as Rank 1 on the merits while recording the skeptic's counter (§7 OPEN-1).
- **DN-102 §2/§6:** the sole named downstream; §6 here **tests** its "depends on the never-type" framing
  and surfaces the disconfirming CPS analysis rather than assenting to it (rule #4, no sycophancy).

## §10 Residual / FLAGs (never claimed as done — VR-5/G2)

- **FLAG-ne-1 — the §6 general-`?` fork is unresolved.** I lean "general-`?` is independent of the
  never-type (CPS needs no bottom)," but cannot rule out a corner of a fully-general CPS lift needing
  one. Maintainer decides (§6-a vs §6-b). If §6-a, DN-102 §6 FLAG-try-1 is relaxed append-only.
- **FLAG-ne-2 — the effect name is unbid.** `diverges` vs `div` vs `noreturn` — a surface/lexicon choice
  (DN-02/03 lexicon), not settled here; it must be reserved so it can never be a silent identifier (G2).
- **FLAG-ne-3 — `panic` vs `exit` semantics.** v0 models `exit` (process abort). `panic`-with-stack-
  unwinding is a *runtime-tier* decision (`hypha`/unwinding — ratified-not-lexed), out of this close;
  v0 `panic` (if any) is `exit`-shaped abort, flagged as such.
- **FLAG-ne-4 — `diverges` is grant-time-only in v0.** Only genuinely-non-returning prims may carry the
  tag; a *user-assertable* `!diverges` (a way for user code to claim divergence) is deferred — it would
  reopen OPEN-2's soundness surface without a checked basis. Revisit only with a witness + a discharge.
- **FLAG-ne-5 — the `.myc` self-hosted mirror (DN-26).** The Rust frontend is the trusted base and lands
  the effect + admissibility rule + real prims first; the `lib/compiler/*.myc` mirror follows the port's
  general cadence, not this note's increment.
- **FLAG-ne-6 — capability-based IO is a *future* direction, not a rejection of the idea.** H-ii is
  rejected only *for v0's gap-close* (YAGNI); an object-capability IO model may be the right ADR-032
  *certified*-mode design later. Recorded so it is not silently foreclosed.

---

## Changelog

- **2026-07-10** — DN-107 created as **Draft** (M-1030 / ENB-7). A design-reasoner note working the
  host-effect (#79) + never-type (#88) decision forward to a **ranked recommendation, not a
  ratification**. Frames the two sub-gaps as separable-but-coupled-at-`exit` (§2, verified against the
  tree at `5130badc`); enumerates the real alternatives for each (§3/§4); recommends **divergence-as-
  effect with no bottom `Ty`** (N-C, Rank 1) + **real `wild:` prims over the existing RFC-0028/RFC-0014
  representation** (H-i), with a nominal `Never` (N-B) held as the append-only upgrade path; and
  **surfaces disconfirming evidence against the task's own framing** — that general-`?` appears **not**
  to depend on the never-type (a CPS lift needs no bottom), left as a maintainer fork (§6). Enacts
  nothing, moves no other doc's status (append-only, house rule #3); every design claim `Declared`,
  soundness never `Proven` (VR-5).
