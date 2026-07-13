# Design Note DN-133 — Qualified / Associated-Function Call-Site Emission (`Type::method(...)`): the Soundness Gate and the Mangled-Name Reconciliation

| Field | Value |
|---|---|
| **Note** | DN-133 |
| **Status** | **Accepted** (2026-07-12; **Draft → Accepted** by the automated strict project-flavored DN-review gate — all 9 criteria clean on the citation-corrected note, `origin/dn133-citation-fix@24b0463a`; every `emit.rs`/`lib.gap.json` anchor independently re-verified against `origin/dev@08d8fc21` blob `86afd16d`; see §7). Ratifies *design only* — this note still **enacts nothing, ships no code**; the build (resolution-gated `Type__method` emission) lands under a separately-minted issue, `depends_on: [M-1084]` (§6 FLAG). Prior: **Draft** (2026-07-12), authored as **READ + a new DN only** by the design-reasoner (it did not self-ratify — house rule #3, append-only). This note is **soundness-sensitive** (the wave-1 "D4" mis-emit lesson, DN-34 §8.2); its core is a *never-mis-emit* discipline, not a permissive one. Every claim is **`Declared`** except code facts read against the tree (`Empirical` at `file:line`, verification base `origin/dev@08d8fc21`). |
| **Decides** | *Proposes, for ratification (does not self-ratify):* (1) the **verified problem** — `crates/mycelium-transpile/src/emit.rs`'s `visit_call` **unconditionally gaps every** `Type::method(...)` qualified/associated call (`emit.rs:2071-2094`, `Empirical`), even though the *declaration* side already mints a deterministic mangled name for no-`self` associated fns (`mangled_inherent_fn_name` → **`{Type}__{method}`**, `emit.rs:3490-3491`; applied `emit.rs:3849-3872`). DN-99 register **row 18** records the closure *direction* (`Type::m`→`type_m`, `use`+dotted App; `map.rs:85`, M-664/M-662, P2) but **not** the emission discipline. (2) The **name reconciliation (a real discrepancy this note settles)** — DN-99 row 18's `type_m` and the landed decl-side `Type__method` **disagree**; the call side MUST emit the *identical* string the decl side emits or it desyncs — so ratify **`Type__method`** (the landed decl-side form) as canonical and correct DN-99 row 18's shorthand. (3) The **ranked recommendation** — close the loop **only under a resolution gate**: emit `Type__method(args)` **iff** the callee type resolves (same-file inherent impl, or a batch sibling via the DN-113 / M-1084 symtab) **and** that nodule actually emitted the mangled decl; **otherwise gap** — never a bare last-segment (the D4 fabrication). (4) The **hard exclusions** — a primitive/std associated fn with no emitted decl (`i128::try_from`, `.from(...)`) **always gaps**; a cross-*module free-function* path (`a::b::c()`) routes through Import/symtab (M-1084), not this path. It does **not** edit `issues.yaml`, `CHANGELOG.md`, `Doc-Index.md`, grammar, or `crates/**` — §FLAGs lists the rows. |
| **Feeds** | **std-sys-host vertical slice** (`OsClock` impl — `mono_now`/`wall_now`/`logical_now` gap on qualified calls: `DeclaredTime::new`, `MonoInstant::from_nanos`, `mycelium_std_sys::time::mono_nanos`, `i128::try_from`, `WallInstant::from_nanos_since_epoch`, `LogicalInstant::from_tick`, `DeclaredTimeEntropy::new` — `gen/myc-drafts/stdlib/std-sys-host/lib.gap.json` gap @ lines 74-75, `Empirical`); **DN-34 §8.13/8.14 "D4"** (inherent-impl associated-fn name mangling — the *declaration* half this note completes on the *call* half; §8.2 `from(self)` fabrication precedent); **DN-99 register row 18** (`qualified-fn-call (T::m)`); **DN-113 / M-1060** (cross-phylum import resolution) and **M-1084** (cross-nodule import net-close — the symtab this note's resolution gate consumes); **DN-124** (phylum-mode measurement basis); **DN-125 / M-1081** (the `&mut self`/`&mut [u8]` half of the `fill_bytes` residual — a *different* blocker, noted for scope honesty). |
| **Grounds on** | **VR-5 / G2 (never-silent, no fabrication)** — a call target is emitted **only** when its referent is proven present, else an explicit gap; **the wave-1 D4 lesson** (call-site receiver-type resolution can silently mis-emit — the reason `visit_call` gaps today); **DN-34 §8.14** (the flat-namespace `M-664` desugar that makes `Type__method` collision-free *by construction*); **DRY** (one canonical mangled name shared by decl + call side — a desync is a soundness hole); **KISS/YAGNI** (mangle only the receiver-less associated-fn class the decl side already mangles); **house rule #4** (surface the finding: this unblocks `mono_now`/`logical_now`, only *partially* `wall_now`, nothing cross-crate without M-1084). |
| **Date** | July 12, 2026 |
| **Task** | Design-vs-build call for the UNTRACKED, SOUNDNESS-SENSITIVE qualified/associated-fn call-emission gap in the std-sys-host punch-list; scope the emission strategy + no-mis-emit discipline + the DN-99-row-18/emit.rs name reconciliation as a ratification-ready Draft DN. Read-only except this DN + FLAGGED rows. |
| **Definition of Done (for the maintainer to ratify Draft → Accepted)** | The maintainer confirms: (a) **`Type__method`** is canonical (call side emits the identical decl-side string; DN-99 row 18 is corrected, append-only); (b) the **resolution gate** is the soundness contract (emit only when callee type resolves AND the mangled decl is known-emitted; else gap — never bare-segment); (c) primitive/std associated fns (`try_from`/`from`) and cross-module *free* fns route to a gap / to M-1084 respectively; (d) a **T-A3-style emit↔check agreement test** (the `emit.rs:3529` T-A3 "emit iff check accepts" agreement anchor, applied at the `emit.rs:3724` MVP prelude-trait recognizer) is required before the build lands; (e) the build issue is minted (**FLAG**), `depends_on: [M-1084, DN-133 ratify]`. |

> **Posture (house rule #4 / VR-5 / G2).** This note **recommends, it does not ratify.** The
> soundness-sensitive core, reported plainly: the *easy* fix — emit the bare last segment of
> `Type::method` — is exactly the silent mis-emit the "D4" lesson (DN-34 §8.2) caught and the reason
> `visit_call` gaps unconditionally today. The *correct* fix emits a name **only when its referent is
> proven to exist**, and gaps otherwise — which makes this change **depend on M-1084** for the
> std-sys-host types (`DeclaredTime`, `MonoInstant`, … live in `mycelium-std-time`, a *different*
> crate). Without M-1084 landed, this note closes only *same-nodule* associated-fn calls — honestly a
> narrow slice — and it settles a latent **name desync** (DN-99 row 18 `type_m` vs the landed
> `Type__method`) that would otherwise become a live soundness hole the moment the call side emits.

---

## §1 The problem, precisely (verify-first — mitigation #14)

`emit.rs`'s `visit_call` (`emit.rs:2063-2110`, `Empirical`) has three arms: a bare single-segment call
(`f(x)`, emitted), a **qualified path call** `Type::method(...)` with `qself.is_none()`
(`emit.rs:2071-2094` — **unconditionally gapped**: "no established Mycelium surface form … emitting the
bare last-segment name would fabricate a call"), and a non-path target (gapped). This gap message is
verbatim what the std-sys-host `OsClock` impl records (`lib.gap.json` gap @ lines 74-75: three sub-issues,
one per method).

**The declaration side is already built.** For a **no-`self`-receiver** inherent-impl associated fn,
`emit_impl` renames the *declaration* to `mangled_inherent_fn_name(self_ty, method) = "{Type}__{method}"`
(`emit.rs:3490-3491`, `Empirical`) and records an EXPLAIN doc line at the call decl (`emit.rs:3857-3865`). Its
scope note (`emit.rs:3474-3489`) states — verify-first, `Empirical` — that this is safe *precisely
because* "`visit_call` already unconditionally gaps every qualified/associated-function call, so no
currently-emitted call site ever references a no-`self` method by its bare name." **This note lifts that
boundary — under a gate — and so must also update that scope note (§5.4).**

**The latent name desync (found in this review).** DN-99 register row 18 (`docs/notes/DN-99…md:81`,
`Empirical`) writes the closure as `Type::m` → **`type_m`** (lowercase, single token). The landed
decl-side emits **`Type__method`** (capitalized, double-underscore). These are **different strings**; a
call side that emitted `type_m` while the decl is `Type__method` would call a name that does not exist
(`myc check`: unknown function) — a fabrication. **Ratify `Type__method` as canonical; correct DN-99
row 18 (append-only).**

## §2 The construct is three sub-kinds (the single "qualified call" label hides them)

The `OsClock` body mixes three call shapes:

1. **Associated fn on a resolvable user type with a mangled decl** — `MonoInstant::from_nanos(...)`,
   `DeclaredTime::new(...)`, `WallInstant::from_nanos_since_epoch(...)`, `LogicalInstant::from_tick(...)`,
   `DeclaredTimeEntropy::new(...)`. These have a `Type__method` decl **iff** their impl was emitted — in
   std-sys-host they live in **`mycelium-std-time` (a different crate/phylum)**, so resolving them needs
   the **M-1084** batch-scoped symtab. This is the *only* sub-kind this note's mangling path emits.
2. **Associated fn on a primitive / std type with no emitted decl** — `i128::try_from(ns)`. `i128` maps
   to `Binary{128}`; there is no `Binary{128}__try_from`. Emitting one fabricates. **Always gap**
   (never-silent, VR-5). (A future `try_from`/`from` *conversion* surface is DN-91/DN-41 territory, not
   this note.)
3. **Cross-*module* free-function path** — `mycelium_std_sys::time::mono_nanos()`,
   `mycelium_std_sys::time::wall_nanos()`: free fns behind a module path, **not** `Type::method` — they
   route through the **Import/symtab free-fn resolver (M-1084)**, which emits the home-qualified `use
   <nodule>.<fn>;` + bare call. Not this note's path.

## §3 The real alternatives + the soundness gate

- **Alt A (recommended) — resolution-gated mangled emission.** In the `Type::method` arm: resolve the
  head type via (i) the enclosing `self_ty` / same-file inherent impls, then (ii) the M-1084 symtab. If
  resolved **and** the target is a known-emitted no-`self` associated fn, emit `{Type}__{method}(args)`
  (identical to the decl side — deterministic, EXPLAIN-traceable, `grep Type__method`). If the head is a
  primitive/builtin, or unresolved, or a `self`-method (reachable from a *bare* call site — mangling it
  desyncs), **gap with the precise reason** (unchanged behavior).
- **Alt B — bare last-segment emission.** **Rejected** — the wave-1 D4 hazard verbatim: `i16::from(self)`
  → `from(self)` fabricates; two types' same-named methods collapse (VR-5/G2).
- **Alt C — status quo (gap all).** Safe; blocks every `OsClock` method and the cross-nodule constructor
  idiom. The floor this note improves on.

## §4 Ranked recommendation + objective function

Objective: **maximize resolved-and-emitted calls with ZERO mis-emit** (the D4 invariant), a paired
emit/check agreement (T-A3), one canonical name (DRY), KISS.

| Criterion | Alt A — resolution-gated | Alt B — bare segment | Alt C — status quo |
|---|---|---|---|
| Zero mis-emit / no fabrication (VR-5/G2, D4) — **must-pass** | Pass (emit only when referent proven present) | **Fail** (the D4 hazard) | Pass |
| Closes real residual | `mono_now`/`logical_now` (with M-1084); partial `wall_now` | (unsound) | None |
| Canonical EXPLAIN-traceable name (decl ≡ call) | Yes (`Type__method`) | No | n/a |
| Emit/check agreement test (T-A3) | Required + feasible | n/a | n/a |
| **Verdict** | **Recommended** | Rejected | Rejected |

**Recommendation: Alt A**, sequenced **after M-1084**, shipped with the T-A3 agreement test and the
DN-99-row-18/decl-side name reconciliation.

## §5 Adversarial stress-test (VR-5 / house rule #4)

1. **Cross-crate types dominate the std-sys-host case** — without M-1084 resolving them (and std-time
   having emitted the mangled decls), Alt A gaps them, so **this note is a no-op for std-sys-host until
   M-1084 lands.** Same-nodule associated-fn calls close immediately; cross-nodule ones are M-1084-gated.
   Reported, not hidden.
2. **`wall_now` is only partially closed even with #1 + M-1084** — its body has a nested `match` and
   constructs a **struct-enum-variant** `TimeErr::ClockUnavailable { reason: "…" }` (a named-field
   variant *in expression position*), a construct beyond this note (cf. DN-132/M-1089's struct-variant
   *pattern* work — the construction side is its own decision). `mono_now`/`logical_now` are the ones
   this note (with M-1084) tips.
3. **`Ok(...)`/`Err(...)` are enum constructors, not associated fns** — handled by existing path/struct
   arms, not this note.
4. **The decl-side scope note (`emit.rs:3474-3489`) becomes stale on ratification** — it asserts safety
   *because* no call site references the bare name; Alt A changes that premise, so the note must be
   updated in the same change (append the call-side closure), never left contradicting behavior (G2).

## §6 FLAGs (append-only, dated — applied by the integrating parent, NOT here)

- **issues.yaml**: mint the build issue (next free `M-109x` — `grep 'id: M-109' tools/github/issues.yaml`,
  M-1091 currently highest), `depends_on: [M-1084, DN-133 ratify]`, `doc_refs: [corpus:DN-133,
  corpus:DN-34#8.14, corpus:DN-99, src:crates/mycelium-transpile/src/emit.rs:2071,
  src:crates/mycelium-transpile/src/emit.rs:3490]`.
- **DN-99 register row 18**: correct the closure name `type_m` → `Type__method` (append-only; cite this
  note as the reconciliation basis).
- **DN-34 §8.14**: note the D4 residual now has a *call-side* closure proposal (cross-ref, append-only).
- **CHANGELOG.md**: Draft-DN-133 line. **Doc-Index.md**: register DN-133.

## §7 Changelog

- **2026-07-12** — DN-133 created (**Draft**). Designs the resolution-gated `Type__method` mangled-call
  emission for `visit_call`'s qualified/associated-fn arm, ratifies `Type__method` as canonical over
  DN-99 row 18's `type_m` shorthand, and scopes the std-sys-host `OsClock` closure to same-nodule calls
  pending M-1084. Read against `origin/dev@08d8fc21` (`Empirical` cites); the proposed mechanism is
  `Declared` (unbuilt). Authored the READ + this DN only — no edit to `issues.yaml`, `CHANGELOG.md`, or
  `Doc-Index.md` (integration-owned; FLAGGED up). Append-only; status advances only by maintainer
  ratification (house rule #3).
- **2026-07-12** — **Citation correction** (Draft stays Draft; append-only, no design change; the
  DN-130/DN-132 mis-citation pattern, caught by the strict DN-review gate on criteria #1/#9). Every
  `emit.rs` `src:line` locator in the original draft was displaced ~800 lines from its actual target.
  Corrected, each re-verified against `origin/dev@08d8fc21` (blob `86afd16d`) before writing: `visit_call`
  `emit.rs:1263-1311` → **`emit.rs:2063-2110`** (the qualified-call gap arm `emit.rs:1271-1294` →
  **`emit.rs:2071-2094`**, confirmed `Expr::Path(p) if p.qself.is_none()`, closing brace at 2094);
  `mangled_inherent_fn_name` `emit.rs:2689-2691` → **`emit.rs:3490-3491`** (2689 in the original draft
  landed on a bare `}`, not the function); the applied site + EXPLAIN doc line `emit.rs:3029-3046` /
  `emit.rs:3031-3039` → **`emit.rs:3849-3872`** / **`emit.rs:3857-3865`**; the decl-side scope note
  `emit.rs:2673-2688` → **`emit.rs:3474-3489`**; the DoD(d)/§4 T-A3 "emit iff check accepts" agreement
  test cite `emit.rs:2726-2771` (that range is closure-param scanning, not the T-A3 agreement) →
  **`emit.rs:3529`** (the T-A3 agreement anchor) applied at **`emit.rs:3724`** (the MVP prelude-trait
  recognizer); the `OsClock` qualified-call gap cite `lib.gap.json` gap @ line 45 (that line is the
  `OsEntropy` derive-drop gap, a different entry) → **lines 74-75** (the `OsClock` impl gap object's
  `snippet`/`reason` fields, `crates/mycelium-std-sys-host/...lib.gap.json` — verified path
  `gen/myc-drafts/stdlib/std-sys-host/lib.gap.json`; the "3 sub-issue(s)" count was already correct, only
  the line was wrong). The §6 FLAG `doc_refs` were updated to the corrected `visit_call` arm
  (`emit.rs:2071`) and mangler (`emit.rs:3490`) lines. No design conclusion, alternative ranking, or
  recommendation changed — only the code-fact locators. Status stays **Draft**; the maintainer / the
  automated DN-review gate re-runs.
- **2026-07-12** — **Ratified Draft → Accepted** by the automated strict project-flavored DN-review gate
  (the maintainer's standing automation: strict 9-criterion review → auto-approve + ratify on a clean
  pass). All nine criteria pass on the citation-corrected note: (1) grounding — every `src:line` now
  resolves to its claimed construct; (2) VR-5 — `Empirical` cites honest, no tag upgraded; (3) G2 — the
  resolution gate emits only a proven-present referent, else an explicit gap (Alt B bare-segment rejected
  as the D4 hazard); (4) append-only — Draft → Accepted, never skipped to Enacted; (5) native-solution —
  the `Type__method` flat-namespace mangle (M-664); (6) KC-3 — deterministic, EXPLAIN-traceable, DRY;
  (7) adversarial — the `Type__method`-vs-`type_m` reconciliation holds (decl side emits `Type__method`
  at `emit.rs:3490-3491`), the resolution gate is leak-free for primitive-assoc-fns and cross-module
  free-fns, and the M-1084 dependency is correct (`mycelium_std_time` is out-of-batch per
  `lib.gap.json:34-35`); (8) DoD — stated; (9) consistency — every `emit.rs`/`lib.gap.json` anchor
  independently re-verified against `origin/dev@08d8fc21` (blob `86afd16d`), the ~800-line displacement
  bug eliminated with no new displacement. Basis: the design-reasoner strict-gate review, this date.
  Ratifies **design only** — no code shipped; the build lands under the §6-FLAGGED issue
  (`depends_on: [M-1084, DN-133 ratify]`), unblocking the L-EMIT-b qualified-call emit leaf. Append-only;
  no design content changed by this transition (house rule #3).

*Guarantee: design claims `Declared`; code `file:line` facts `Empirical`. No tag upgraded past its basis
(VR-5).*
