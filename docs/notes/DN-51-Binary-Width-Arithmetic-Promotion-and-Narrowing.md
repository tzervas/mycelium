# Design Note DN-51 — `Binary` Width-Arithmetic: Accuracy-First Promotion, Hybrid Overflow, Explicit Narrowing

| Field | Value |
|---|---|
| **Note** | DN-51 |
| **Status** | **Accepted** (2026-06-27; **ratified by the maintainer in-session**) — sets the cross-width arithmetic policy for `Binary{N}`: **auto-widen to the wider operand** (accuracy-first), a **hybrid overflow** rule (promotion by default, growth-to-fit on explicit opt-in), and **two explicit narrowing forms** (checked-narrow + `truncate`). **Enacts no code** and moves no other decision's status (house rule #3); it **extends** DN-41 (the width-cast prim) and **supersedes** the conservative mixed-width *refusal* that DN-42 v1 shipped, as a follow-on implementation. Per the swarm-integration rule the implementing tasks move to *"implemented (Rust-first), pending ratification"* as they land — never silently to `Accepted`. |
| **Feeds** | The width-generic arithmetic surface — `lib/std/math.myc` (`badd`/`bsub`/…), which today **refuses** mixed widths (M-718/M-753/DN-42 §4); and `lib/std/cmp.myc` / collection key compares, which already need the DN-41 widen to compare differing widths. Closes the "what happens on mixed widths?" question DN-42 left as a conservative refusal. |
| **Date** | June 27, 2026 |
| **Decides** | *Ratified design:* (1) **operand promotion** — a binary op on operands of differing width auto-widens the narrower to the wider (DN-41 zero-extend, `Exact`/lossless), result width = the wider operand; (2) **hybrid overflow** — default is promotion-only (overflow past the result width stays a never-silent `Overflow` refusal), and **growth-to-fit** is an *explicit opt-in* form (`widening_add` → N+1 bits, `widening_mul` → N+M bits, …) that cannot overflow; (3) **explicit narrowing, two named forms** — DN-41's **checked-narrow** (refuse if the value does not fit; the safe default) and a new explicit **`truncate`** (intentional high-bit drop, total-but-lossy); narrowing is **never** automatic; (4) **per-instance guarantee tags** — each monomorphized instance carries the tag of its actual specialized op (widen/growth/identity/in-range-narrow `Exact`; `truncate` its own honest lossy tag); the width-generic *source* fn is a template and carries no executable grade. |
| **Task** | (follow-on) supersede the mixed-width refusal with operand-promotion; add the `widening_*` growth-to-fit forms; add the explicit `truncate` op; record the per-instance grading model. To be minted as `M-xxx` (E11-1/E13-1) — see §6. |

> **Posture (transparency rule / VR-5 / G2).** This note records a **design decision the maintainer
> ratified in conversation**; it **enacts no code**. Every guarantee claim below is tagged at its
> *established* strength: the widen/growth/in-range cases are `Exact` (a total, lossless map on the
> unsigned magnitude — `Binary` is sign-free, **ADR-028** (*Proposed* at this note's authoring, later
> **Accepted** 2026-07-01 via RFC-0033's ratification act — already adopted in practice by DN-41
> either way); `truncate` is honestly **lossy** (its own tag, never `Exact`); the never-silent
> refusals are `Declared`/never-silent. Nothing is upgraded to `Proven`. The policy **softens** two
> prior positions (see §3) — this is recorded explicitly, not silently.

## 1. Problem

DN-42 v1 (width-generics, M-753) made a single width param `N` flow across a function's operands, so
a width-generic op requires its operands to share a width; a **mixed-width** call
(`badd(Binary{8}, Binary{16})`) is an explicit never-silent **refusal** (DN-42 §4). That refusal was
the *conservative* never-silent placeholder — "don't guess" — not a decision that mixed widths are
meaningless. It leaves two real questions open:

- **Cross-width operands.** When operands genuinely differ in width, refusing forces the programmer to
  hand-insert a `width_cast` (DN-41) at every site. The accuracy-safe answer — widen the narrower to
  the wider — is *always lossless* (`Binary` is sign-free, ADR-028 → widening is zero-extension), so the
  refusal trades ergonomics for nothing the type system couldn't do safely itself.
- **Overflow.** Even at a single width, `badd` refuses on carry-out (never wraps — G2). Sometimes the
  programmer wants the carry *kept* (a wider, carry-safe result) rather than refused.

## 2. Decision

**(D1) Operand promotion — auto-widen to the wider, accuracy-first.** A binary arithmetic/logic op on
operands of differing width widens the narrower operand to the wider via the DN-41 zero-extension widen
(`Exact`, lossless), and the result takes the wider width. `badd(Binary{8}, Binary{16}) → Binary{16}`.
This **replaces** the DN-42 mixed-width refusal for arithmetic. The inserted widen is **reified and
`EXPLAIN`-able** (an inspectable node, "zero-extend `Binary{8} → Binary{16}`"), so it is *automatic but
not silent* — see §3.

**(D2) Hybrid overflow — promote by default, grow on opt-in.**
- *Default (promotion-only):* the result width is the wider operand width; a result that overflows that
  width is the **existing never-silent `Overflow` refusal** (e.g. `Binary{8}+Binary{8}` whose sum > 255
  still refuses). The never-silent overflow floor is unchanged.
- *Opt-in (growth-to-fit):* explicit `widening_*` forms grow the result so the op **cannot** overflow —
  `widening_add`/`widening_sub` → N+1 bits, `widening_mul` → N+M bits. Carry is preserved, not refused.
  Truncating the grown result back to a smaller width is the explicit narrow of (D3).

**(D3) Narrowing is explicit, with two named forms (never automatic).** To put a value into a *smaller*
width, the programmer names one of:
- **checked-narrow** (DN-41 narrow) — keeps the low `M` bits **iff** the dropped high bits are all zero;
  otherwise an explicit `Overflow` refusal. The *safe default* narrow.
- **`truncate`** (new) — unconditionally drops the high `N − M` bits. **Total but lossy**, and **only
  ever via this named op** — so "never a *silent* truncation" (DN-41) still holds; truncation is a
  deliberate, visible choice.

**(D4) Per-instance guarantee tags** (resolves the Decision-1 fork). The grade is evaluated on each
monomorphized instance, exactly as for a hand-written concrete fn: widen / growth-to-fit / identity /
in-range checked-narrow are `Exact`; `truncate` carries its own honest lossy tag; an out-of-range
checked-narrow is the never-silent refusal. The width-generic *source* fn is a template and carries no
executable grade — only its instances do. The genericity never *upgrades* a tag (DN-42 §4).

## 3. Honest reconciliation — what this softens, and why it stays never-silent

This policy **refines two ratified positions**; recorded here, not buried:

1. **DN-41 / convert doctrine leaned "conversions are explicit."** DN-41's widen is an *explicit* prim
   (`width_cast(idx8, len32)`); this note makes the widen **automatic inside arithmetic**. That is a
   deliberate softening of "all conversions explicit." It stays within never-silent (G2) because the
   widen is **lossless** *and* **reified/`EXPLAIN`-able** *and* the result width is **visible in the
   type** — automatic ≠ silent for a lossless, recorded op. Never-silent bites on *loss* and on
   *approximation/selection*; a transparent lossless widen is neither. (If a future audit wants the
   widen visible at the call site too, an `EXPLAIN`/lint surfaces it — it is never *hidden*.)

2. **DN-41 made narrowing *checked-only* ("never a silent truncation").** This note **adds** an explicit
   `truncate`. It does **not** contradict DN-41: checked-narrow stays the safe default, and truncation
   happens **only** when the programmer names `truncate` — so truncation is never *silent*. It is an
   *extension* (a second, deliberately-lossy narrow), not a reversal.

Neither softening touches the **overflow** floor: the default still refuses on carry (G2); growth-to-fit
is opt-in.

## 4. Guarantee matrix (per-op, per-instance)

| Operation | Guarantee | Never-silent contract |
|---|---|---|
| operand promotion (widen narrower → wider) | **`Exact`** (lossless zero-extend, ADR-028) | reified/`EXPLAIN`-able; result width visible in type |
| arithmetic, promotion-only, in-range | **`Exact`** | overflow past result width → explicit `Overflow` refusal |
| `widening_add`/`sub`/`mul` (growth-to-fit) | **`Exact`** (carry-safe; cannot overflow) | n/a (no overflow by construction) |
| checked-narrow (DN-41), in range | **`Exact`** | out-of-range → explicit `Overflow` refusal |
| `truncate` (explicit high-bit drop) | **lossy** (its own honest tag — never `Exact`) | only ever via the named op (never automatic) |
| three-way agreement (L1 ≡ L0 ≡ AOT) | **`Empirical`** (trials) | — |

## 5. Grounding

- **DN-41** (Accepted) — the `bit.width_cast` widen/checked-narrow prim this builds on; the widen
  semantics (zero-extension) and the checked-narrow are reused verbatim.
- **DN-42** (Accepted) — width-generics (M-753); §4's mixed-width *refusal* is the conservative
  placeholder this note replaces (for arithmetic) with operand promotion.
- **ADR-028** (*Proposed at this note's authoring, later **Accepted** 2026-07-01 via RFC-0033's
  ratification act*) — `Binary` is sign-free, which is what makes widening a pure zero-extension
  (lossless, `Exact`). Honest caveat, as recorded at authoring time: ADR-028 was then in the
  RFC-0033/ADR-025…031 value-model cluster and Proposed, not yet Accepted — but DN-41 (Accepted)
  already committed the zero-extension widen, so the lossless-widen basis was effectively settled via
  DN-41 even ahead of ADR-028's ratification (which has since landed).
- **`std.cmp`/`convert` spec** — lossless widening total, lossy narrowing explicit/fallible: this note
  is consistent with that doctrine (and extends it with the explicit `truncate`).
- **RFC-0032 §5 D2** — the binary-arithmetic kernel enablers (`bit.add`/`bit.sub`) the `widening_*`
  forms and promotion sit above.
- House rules: **G2** (never-silent), **VR-5** (honest tags, no upgrade), **KISS/YAGNI** (per-instance
  grading needs no new machinery), **KC-3** (no new kernel node beyond DN-41's `width_cast` + the
  existing arith prims).

## 6. Definition of Done (the follow-on implementation)

1. **Operand promotion** — the checker/elaborator, on a binary op with differing operand widths, inserts
   a DN-41 widen on the narrower operand (reified node) and types the result at the wider width; the
   DN-42 mixed-width *refusal* is removed *for arithmetic* (a cross-**paradigm** mix — binary vs ternary —
   still refuses; that is a swap question, RFC-0034/ADR-032, out of scope here).
2. **Growth-to-fit forms** — `widening_add`/`widening_sub` (N+1) and `widening_mul` (N+M) surfaced over
   the RFC-0032 D2 prims; `Exact`, carry-safe; three-way differential.
3. **`truncate` op** — explicit high-bit drop over `width_cast`'s machinery; honest lossy tag;
   never-silent (only via the named op); three-way differential + a refusal/identity test matrix.
4. **Per-instance grading** — confirm the RFC-0018 grader runs on the monomorphized instances (not the
   pre-mono template); record the per-instance model as an append-only note on DN-42 §4 + RFC-0018.
5. Update `lib/std/math.myc` (drop the mixed-width FLAG; add the `widening_*` + `truncate` surface) and
   its tests; `just check` green.

Tasks to mint (collision-checked against `issues.yaml`): one E11-1/E13-1 leaf per (1)–(4), gated behind
DN-41 (landed) and DN-42 (landed).

## 7. Open sub-questions (FLAGGED — for the maintainer at implementation time)

- **(Q1) Surface spelling.** `widening_add` / `truncate` are working names. Alternatives in the DN-31
  grammar-wave idiom, or a result-type-ascription trigger (`badd(a, b) : Binary{16}` ⇒ checked-narrow;
  a `: trunc Binary{8}` ⇒ truncate)? Defer the exact spelling to the grammar pass; the *semantics* are
  fixed here.
- **(Q2) Ternary parity.** Does the same promotion/growth/explicit-narrow model apply to `Ternary{M}`
  balanced-ternary arithmetic? Balanced ternary is sign-symmetric (no sign-extension asymmetry), so
  widening is also lossless — likely yes, but the carry/range semantics differ (M-111). FLAGGED; decide
  with the ternary arithmetic increment, not assumed here.
- **(Q3) Promotion across logic ops.** `band`/`bor`/`bxor` on differing widths — promote-then-op (the
  narrower's high bits become 0, so `band` masks, `bor`/`bxor` pass through), or refuse? Default per
  (D1) is promote; confirm that is the wanted semantics for bitwise (it is the natural zero-extension
  reading) at implementation time.

## Changelog

| Date | Status | Note |
|---|---|---|
| 2026-06-27 | **Accepted** *(§7 sub-questions concurred)* | The maintainer concurred with the §7 leans: **Q3** (logic ops `band`/`bor`/`bxor` on differing widths) → **promote (zero-extend) then op** — the natural reading; **Q1** (surface spelling of `widening_*`/`truncate`) → deferred to the grammar pass, **semantics fixed here**; **Q2** (ternary parity) → the **same promotion / hybrid-overflow / explicit-narrow model applies in DIRECTION to `Ternary{M}`**, with the carry/range specifics settled at the ternary-arithmetic increment (balanced ternary differs — M-111; not assumed closed here). Append-only. |
| 2026-06-27 | **Accepted** | Maintainer ratified in-session: cross-width binary arithmetic auto-widens to the wider operand (accuracy-first, DN-41 zero-extend, `Exact`); **hybrid overflow** (promotion-only default keeps the never-silent `Overflow` refusal; growth-to-fit `widening_*` is an explicit opt-in); narrowing is explicit with two named forms (checked-narrow + new `truncate`); per-instance guarantee tags (resolves the Decision-1 fork). Extends DN-41, supersedes the DN-42 §4 mixed-width *refusal* for arithmetic (follow-on impl). Enacts no code; softens two prior positions explicitly (§3); upgrades no guarantee (VR-5). |
