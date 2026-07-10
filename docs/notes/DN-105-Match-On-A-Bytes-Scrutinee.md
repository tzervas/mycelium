# Design Note DN-105 — `match` on a `Bytes` Scrutinee (the ENB-12 string-literal-pattern enabler)

| Field | Value |
|---|---|
| **Note** | DN-105 |
| **Status** | **Accepted** (2026-07-11, maintainer ratification — the maintainer approved this note as drafted, part of a batch ratifying DN-101–DN-109; see the dated "Ratification / Maintainer decision" note below). Originally **Draft** (2026-07-10). Authored alongside the **first landable increment** of M-1035 (ENB-12). It records the design of allowing a `match` whose scrutinee is a `Bytes` value, with **byte-string-literal** arms and a **required default arm** — the surface, the equality/exhaustiveness semantics, the never-silent non-exhaustive refusal, and the DN-26 Rust↔`.myc` dual. It **enacts nothing** and **moves no other doc's status** (house rule #3, append-only). Tags are `Empirical` where read against the code / witnessed by a running differential, `Declared` for any design not yet ratified (VR-5). |
| **Decides** | *Proposes, for ratification:* (1) **lift the checker's match-scrutinee-type gate** to admit `Ty::Bytes` alongside `Data`/`Binary`/`Ternary` (`crates/mycelium-l1/src/checkty.rs::check_match`) — the single categorical block; (2) the **pattern form is the existing byte-string literal**, in both its surface spellings — the `0x…` hex form (`Literal::Bytes`, RFC-0032 D4 / M-750) **and** the `"…"` text form (`Literal::Str`, M-910/M-911) — both already type as `Bytes` and already normalize to a matrix `Pat::Lit`; **no new pattern/AST node** (KC-3); (3) **equality is byte-content equality** — a literal arm matches iff the scrutinee's `Repr::Bytes`/`Payload::Bytes` byte-vector equals the literal's (the evaluator's existing `try_match` `Pattern::Lit` `repr==repr && payload==payload` rule, unchanged); (4) **`Bytes` is an OPEN domain** — a literal column never completes it, so a **wildcard/default arm is REQUIRED**; a non-exhaustive `Bytes` match is a never-silent `W7` refusal (witness `_`), exactly as the existing usefulness/decision machinery already treats every non-`Data` scrutinee; (5) the **genuine fork resolves to literal-only equality patterns** (§3), **not** structural byte patterns (prefix/slice/cons destructuring) — the latter is a large separate design, deferred; (6) the **redundancy key stays per-surface-form** (`by:` for hex, `s:` for text) for this increment (§4), a conservative under-report of cross-form redundancy that is documented as a limitation, never a silent miscompile. It does **not** edit `issues.yaml`, `CHANGELOG.md`, or `Doc-Index.md` (the integrating session owns those). |
| **Feeds** | DN-99 §A3 / register row **#72** (string-literal match pattern) — reclassified from `tr-only` to **language-enabler** (mitigation #14); ENB-12 / M-1035; the trx transpiler pin `string_literal_pattern_gaps_with_l1_enabler_reason` (DN-34 §8.21, FLAG-L1-match-Bytes), which flips from *gapped* to *emitted* once this lands; M-750 / M-910/M-911 (the `Bytes` repr + the `0x…` and `"…"` literal forms this reuses, Enacted); DN-26 (SCC self-hosting, the Rust↔`.myc` dual). |
| **Grounds on** | KC-3 (small kernel — no new L0 node, no new pattern node, no new checking pass; the enabler is a **one-clause relaxation** of an existing type gate, and every downstream consumer — normalize/usefulness/decision/eval/elab — already handles an open-domain literal column generically), DRY (reuse the `Binary`/`Ternary` open-domain machinery verbatim — `Bytes` was already `signature() → None`), G2 (never-silent — a non-exhaustive `Bytes` match and an ill-typed literal arm are explicit refusals, and a first-match-wins runtime is deterministic), VR-5 (no tag upgraded past its basis — the semantics are `Declared` until ratified, earned `Empirical` by the §7 witnesses), KISS/YAGNI (literal-only equality over a structural byte-pattern grammar). |
| **Date** | July 10, 2026 |
| **Task** | M-1035 (ENB-12) — L1 `match` on a `Bytes` scrutinee. |

> **Grounding + honesty (house rule #4 / VR-5 / G2).** This note records a design and a running
> increment; it does **not** take a decision (house rule #3 — the maintainer ratifies). Empirical claims
> are witnessed by the differential/conformance witnesses named in §7. The equality/exhaustiveness
> semantics, the fork resolution, and the redundancy-key choice are `Declared` until ratified. **No
> sycophancy:** §3 confronts the genuine fork (literal-only vs structural byte patterns) on its merits,
> and §4/§6 state the residuals (the cross-surface-form redundancy under-report; the `.myc` interpreter's
> `Bytes`-value carrier; the `0x…`-hex-in-`.myc` eval deferral) plainly rather than claiming a finished
> "byte-pattern system".

---

## §1 Purpose

Close DN-99 register row **#72** (string-literal match pattern). A stdlib porter translating a Rust
`match s { "foo" => …, "bar" => …, _ => … }` string-dispatch has **no faithful target** in Mycelium v0:
the surface is grammatically valid (`pattern ::= literal ::= StrLit`), the literal already types as
`Bytes` (M-910/M-911), and the evaluator already knows how to compare two `Bytes` values — but the L1
checker **categorically rejects the whole match**, because `check_match`'s scrutinee-type gate admits
only `Data`/`Binary`/`Ternary`:

```
check-error: match scrutinee must be a data, Binary, or Ternary type, got Bytes
```

(verified against the real `myc check` oracle, DN-34 §8.21). The trx transpiler consequently *gaps* every
string-`match` never-silently (VR-5/G2) rather than emit check-failing `.myc`. This was reclassified
(mitigation #14) from #72's original `tr-only` guess to a **language-enabler** gap: the block is one
clause in the checker, and lifting it unblocks #72 **and** every string-dispatch port target.

The striking finding on investigation (§7): **everything below the gate already works.** The pattern
normalizer already types `0x…`/`"…"` literal patterns as `Bytes` and lowers them to a matrix `Pat::Lit`
(`checkty.rs::normalize_pattern`, `literal_key`); the usefulness analysis already treats `Bytes` as an
**open** domain (`usefulness.rs::signature() → None` for every non-`Data` type — "a literal column always
needs a default"); the decision-tree compiler already marks a non-`Data` column `complete = false`
(always needs a default); and the evaluator's `try_match` `Pattern::Lit` arm already compares
`Repr::Bytes`/`Payload::Bytes` by content. The gate is the **only** categorical block. This note is
therefore a **one-clause enabler**, not a new pattern subsystem (KC-3).

## §2 The surface + semantics

**Surface.** No grammar change. A `Bytes` scrutinee is matched with byte-string literal arms plus a
required default:

```mycelium
fn tag(s: Bytes) => Binary{8} =
  match s {
    "get"  => 0b0000_0001,
    "post" => 0b0000_0010,
    0x2a   => 0b0000_0011,   // the hex spelling is equally legal (both type as Bytes)
    _      => 0b0000_0000    // REQUIRED — Bytes is an open domain (see below)
  };
```

Both literal spellings are admitted because both already lower to the **same** value form
(`Repr::Bytes` + `Payload::Bytes`, KC-3 — M-750/M-910/M-911): `"foo"` is its UTF-8 bytes, `0x666f6f`
is those same three bytes.

**Equality.** A literal arm matches iff the scrutinee's byte-vector **equals** the literal's, byte for
byte — the evaluator's existing `try_match` rule (`lv.repr() == v.repr() && lv.payload() == v.payload()`).
Because `"foo"` and `0x666f6f` denote the **same** `Bytes` value, they match the **same** scrutinees:
value identity, not surface identity. Arms are tried top-to-bottom; the **first** matching arm wins
(deterministic — as for every other scrutinee kind).

**Exhaustiveness (the never-silent core, G2/VR-5).** `Bytes` is an **open** value domain — there is no
finite constructor signature to enumerate (unlike a `Data` type's ctor set; and unlike `Binary{N}`/
`Ternary{N}` only in that even their bounded domains are never enumerated either). So a set of literal
arms is **never** exhaustive: the coverage analysis (`usefulness::useful` with `signature() → None`)
always finds a `_` witness unless a wildcard/binder arm is present. A `Bytes` match **without** a default
arm is therefore a never-silent `W7` refusal:

```
non-exhaustive match on Bytes: missing _ (W7 — coverage is checked, never assumed)
```

This is not new machinery — it is the **identical** treatment `Binary`/`Ternary` scrutinees already get;
lifting the gate simply lets `Bytes` reach it. A default arm is thus **required by construction**, which
is exactly the "wildcard/default arm required since byte-strings are open" invariant the enabler asks for.

## §3 The genuine fork — literal-only vs structural byte patterns

There are two real designs for "matching on a `Bytes` value", and they are **not** the same size:

- **(A) Literal-only equality patterns** *(proposed).* A byte-string literal arm is a whole-value
  equality test; destructuring is out of scope. This is what DN-99 #72 ("string-literal match pattern")
  and every string-dispatch port target actually need (`match verb { "get" => …, "post" => … }`). It is
  a **one-clause** checker relaxation over machinery that already exists — no new pattern node, no new
  L0 node, no new coverage rule. Exhaustiveness is trivially "a default is required".

- **(B) Structural byte patterns.** Prefix/slice/cons destructuring with binders and guards
  (`match s { [0x2f, rest @ ..] => …, [] => … }` / `s"prefix" ++ rest`), i.e. treating `Bytes` as a
  cons/slice-able sequence in pattern position. This is a **large, separate** design: it needs new
  pattern nodes, a slice/rest binder, a length-and-prefix coverage lattice (partial exhaustiveness over
  an open sequence domain), and an evaluator that binds sub-slices. It also overlaps the deferred
  `Seq`-pattern surface (`normalize_pattern` already refuses `[…]` patterns as "not a v0 surface").

**Resolution: (A).** KISS/YAGNI — (A) delivers the whole of #72 and the string-port unblock with a
one-clause change and zero new kernel surface; (B) is a design in its own right with no current port
driver, and would be an append-only extension **on top of** (A) (a literal arm is the degenerate
whole-value case of a structural pattern), so choosing (A) now forecloses nothing. (B) is recorded as a
residual (§6), not adopted.

## §4 The redundancy-key sub-fork (honest limitation)

The coverage matrix de-duplicates literal patterns by a **canonical key** (`checkty::literal_key`). The
two `Bytes` surface spellings currently key **distinctly** — a hex literal as `by:<hex>`, a text literal
as `s:<content>` (the M-910/M-911 choice, made when a `Bytes` scrutinee could not yet be matched, so the
keys never met). With the gate lifted they **can** meet, exposing a sub-fork:

- **Canonicalize** both to one content key (so `0x666f6f` and `"foo"` collide, and the second is flagged
  **redundant**); or
- **Keep** per-surface-form keys (so `0x666f6f` and `"foo"` are treated as **distinct** literal columns).

**Resolution for this increment: keep per-surface-form keys.** Rationale: it holds exact Rust↔`.myc`
`literal_key` parity (the `.myc` mirror already emits `by:`/`s:`), and — critically — it is **never a
silent miscompile**. The only observable effect of the conservative choice is a **missed redundancy
lint**: a program that writes both `0x666f6f => a` and `"foo" => b` compiles, and at runtime the *first*
arm deterministically wins (the second is dead but un-flagged). This **under-reports** redundancy (a
conservative, honest gap — it never *falsely accepts* a bad program and never picks a wrong arm), and it
**never** weakens exhaustiveness (a default is still required regardless of keys). Canonicalizing the key
to byte content is a clean follow-up (it would also normalize hex letter-case) — recorded as a residual
(§6). Presented here rather than silently chosen (house rule #4).

## §5 The DN-26 dual (Rust frontend + `.myc` mirror)

- **Rust frontend (`crates/mycelium-l1`).** (1) `check_match` admits `Ty::Bytes`; (2) the elaborator's
  L0 literal bridge (`elab.rs::lit_key_to_value`) decodes the `by:`/`s:` keys back to a `Repr::Bytes`
  value (so a `Bytes` match lowers to the same `Alt::Lit` decision tree every other literal match does);
  (3) the witness renderer (`usefulness.rs::render`) renders a `by:`/`s:` key back to surface syntax for
  diagnostics. Everything else (normalize, usefulness, decision, `try_match`, mono, ambient, totality,
  fmt) is already generic over an open-domain literal column and needs **no** change.
- **`.myc` mirror (`lib/compiler/semcore.myc`).** The pattern-typing + coverage leaves
  (`lit_ty_of`/`literal_key`/`normalize_pattern`/`signature`/`useful`/exhaustiveness) already handle a
  `Bytes` scrutinee at parity (they were written generically over open types — `signature → None`). The
  self-hosted interpreter value `LVal` previously **collapsed** every repr value to one opaque
  `LOpaque` (FLAG-semcore-35, lossless for the arms then ported), which is why its `try_match` `PLit`
  arm was an honest blanket refusal. This increment lifts the **`Bytes`** case out of that collapse — a
  new `LReprBytes(Bytes)` carrier — so the `.myc` `lval_try_match` can compare a `Bytes` value against a
  **text** (`Str`) literal pattern by byte content, giving a **real** eval-match differential for the
  #72 target. The `0x…`-hex literal *value-synthesis* stays deferred in `.myc` (FLAG-semcore-25 — hex→byte
  synthesis needs a primitive absent from the wild-free surface); a `0x…` pattern arm's `.myc` eval
  propagates that deferral as an explicit `Err`, never a silent mismatch (G2). Other reprs (`Binary`/
  `Ternary`/`Float`) stay `LOpaque` and their `PLit` eval-match stays honestly deferred (FLAG-semcore-35).

## §6 Residuals (stated plainly, not hidden)

1. **Structural byte patterns (fork B, §3)** — prefix/slice/cons destructuring is not adopted; only
   whole-value literal equality. An append-only future extension with no current driver.
2. **Cross-surface-form redundancy under-report (§4)** — a hex and a text literal denoting the same bytes
   are not flagged mutually redundant. Conservative (never a wrong arm at runtime); the fix is to
   canonicalize `literal_key` to byte content (also normalizes hex case). Follow-up, not this increment.
3. **`.myc` `0x…`-hex eval-match** — deferred with the pre-existing FLAG-semcore-25 (no wild-free
   hex→byte synthesis primitive); the `.myc` eval-match parity is delivered for the **text** (`"…"`)
   form, which is exactly the #72 string-literal target. The `0x…` form is fully covered on the Rust leg.
4. **`.myc` non-`Bytes` repr eval-match** — `Binary`/`Ternary`/`Float` literal patterns stay
   `LOpaque`-collapsed and their `.myc` `try_match` stays deferred (FLAG-semcore-35); untouched here.
5. **Native-LLVM textual-IR `Bytes`-match codegen** — the full **three-way** differential (L1-eval ≡
   elaborate→L0-interp ≡ **trampoline-AOT**, `mycelium_mlir::run`) **does** hold for a `Bytes` match:
   the interpreter and the trampoline AOT both compare an `Alt::Lit` `Repr::Bytes`/`Payload::Bytes`
   value by content generically. Only the **separate** native-LLVM textual-IR backend
   (`llvm.rs::emit_llvm_ir`, a bit-subset direct-LLVM fallback) is `Binary{8}`-specialized (`as_binary8`
   scrutinee + `lit_binary8_packed` arm → an `i64` switch), so it cannot lower a `Bytes` scrutinee — a
   never-silent `AotError` refusal (pinned by `bytes_match_native_llvm_refuses_explicitly`), never a
   silent miscompile (G2). A byte-vector-compare LLVM lowering is a separate perf-path piece with no
   current driver — recorded here, tracked for a follow-up.

## §7 Definition of Done + witnesses

**Definition of Done (from M-1035).** The L1 checker accepts a `match` on a `Bytes` scrutinee with
byte-string-literal arms and a required default; a differential vs the Rust semantics plus a conformance
accept + reject; never-silent on a non-exhaustive/ill-typed match (explicit error, not a panic); a Draft
DN records the rule before the L1 change; the trx #72 pin flips in lockstep (the integrating session owns
the `issues.yaml`/trx-pin edits).

**Witnesses (this increment).**

- **Rust three-way** (`crates/mycelium-l1/tests/enablement.rs`, `assert_three_way` — L1-eval ≡
  elaborate→L0-interp ≡ trampoline-AOT): a `Bytes` match that **hits a literal arm** (`"…"` and `0x…`
  spellings, including the cross-spelling `0x666f6f` ≡ `"foo"` case with a `Bytes`-valued arm body), and
  one that **falls through to the default** — identical value on all three paths. The native-LLVM
  backend's explicit refusal is separately pinned (`bytes_match_native_llvm_refuses_explicitly`, §6.5).
- **Rust reject** (same file, `*_reject`): a **non-exhaustive** `Bytes` match (literal arms, no default)
  → explicit `W7` non-exhaustive refusal; an **ill-typed** literal arm (a non-`Bytes` literal against a
  `Bytes` scrutinee) → explicit refusal. Never a panic (G2).
- **`.myc` eval-match differential** (`crates/mycelium-l1/src/tests/compiler_stage5_evalmatch.rs`): the
  self-hosted `lval_try_match` picks the **matched** text-literal arm and **falls through** to the
  default on a `Bytes` (`LReprBytes`) value, cross-checked against the real Rust `Evaluator::try_match`
  oracle; the `0x…`-hex arm's honest deferral is a standalone probe (explicit `Err`, no oracle-parity
  claim).

Honesty tags: the enabler is `Empirical` (witnessed by the running differentials above); the design
choices (§3 fork, §4 key) are `Declared` until the maintainer ratifies (VR-5).

---

## Ratification / Maintainer decision (2026-07-11)

> **Ratified as drafted** — part of the maintainer's batch approval "approving and ratifying the rest
> of that set from 101–109."

**Recorded decision (append-only — this note's original §2–§6 text above is unchanged; this section
adds the ratification, per house rule #3):** DN-105 is ratified **as drafted**, with no amendment.
Confirmed on this basis: the checker's match-scrutinee-type gate lifts to admit `Ty::Bytes` (§1); the
pattern form is the existing byte-string literal in both spellings, no new pattern/AST node (§2);
byte-content equality, with `Bytes` an **open domain** requiring a wildcard/default arm — a
non-exhaustive `Bytes` match stays a never-silent `W7` refusal (§2); the **§3 fork resolves to (A)
literal-only equality patterns**, not structural byte patterns (deferred, §6.1); the **§4 redundancy-key
sub-fork resolves to keeping per-surface-form keys** (`by:`/`s:`), the conservative, never-silently-wrong
choice (§4, §6.2). **DN-105 moves Draft → Accepted** on this basis; §6's residuals (structural byte
patterns, the cross-form redundancy under-report, the `.myc` `0x…`-hex eval deferral, the native-LLVM
`Bytes`-match refusal) remain open follow-up scope, not blocking this ratification, and are already
tracked (M-1035 / the DN's own §6 notes).

## Changelog

- **2026-07-11** — **Ratified (maintainer, house rule #3).** Status **Draft → Accepted**, as drafted
  (no amendment) — part of the batch ratification of DN-101–DN-109. Append-only — the original design
  record above is unchanged; this is an added ratification note.
- **2026-07-10** — DN-105 created (**Draft**): `match` on a `Bytes` scrutinee — the ENB-12
  string-literal-pattern enabler. Records the one-clause `check_match` gate lift, the byte-content
  equality + required-default (open-domain) exhaustiveness semantics, the literal-only-vs-structural fork
  (resolved to literal-only), the per-surface-form redundancy-key limitation, and the DN-26 Rust↔`.myc`
  dual. Authored alongside the first landable increment of M-1035; enacts nothing, moves no other doc's
  status (append-only).
