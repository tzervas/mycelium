# Design Note DN-127 — Mycelium's Native Answer to the Problem Rust `Display` / `write!` / `format!` Solves (Value-to-Text): Pure Rendering to `Bytes` over a Mutating `Formatter` Sink

| Field | Value |
|---|---|
| **Note** | DN-127 |
| **Status** | **Draft** (2026-07-12). A design note in the design-reasoner pattern (enumerate → evaluate → recommend-ranked → adversarially test); it **recommends, it does not ratify** (house rule #3 — Draft→Accepted is the maintainer's, never the reasoner's). **Builds nothing** — every mechanism/tag here is `Declared`/unbuilt until landed and differential-witnessed. Does not edit `crates/**` or any integration-tier shared file. |
| **Decides (proposes, for ratification)** | Mycelium's **native solution** to the problem Rust `impl Display { fn fmt(&self, f: &mut Formatter) }` + `write!(f, …)` / `format!(…)` solves — *turning a typed value into human-readable text, interpolating sub-values* — is a **pure value-returning render**: `render: T → Bytes`, with `write!`/`format!` interpolation lowering to `bytes_concat` of rendered fragments. The `&mut Formatter` receiver is **not** a native concept: it is a mutation-through-a-sink, remapped by value-threading (DN-125 — **Accepted, but on a parallel not-yet-merged worktree branch as of this note's base**; cross-referenced, not duplicated; see the §2 caveat). The primitive gap — int→decimal-`Bytes` — is resolved **in std, from existing kernel prims** (`div_u`/`rem_u`/`width_cast`/`bytes_concat` + recursion), **not** by growing the kernel (KC-3). A `Show`-shaped **prelude trait** provides generic dispatch for the interpolation of arbitrary types (seeded like `Fuse`/`Ord3`; the seeding mechanism is DN-129's, shared). |
| **Native-solution class (DN-110/DN-111 taxonomy)** | **Idiomatic Remapping** for the `Display`/`write!` shape (the mutating `Formatter` sink → a pure `Bytes`-returning render; a different mechanism that solves the same PROBLEM) **+ Native Equivalent** for the int→decimal primitive (derivable exactly, on-grid, from the landed prim set — no approximation, no bridge). |
| **Feeds** | DN-34 §8.22 (the 30/30 `&mut Formatter` + `write!`/`format!` gap this addresses — the single largest *pure* bucket in the transpile corpus); DN-128 (std-derive lowering — `derive Debug` / `derive Show` reuse this render surface); DN-129 (the prelude-seed mechanism this note's `Show` trait rides); `lib/std/fmt.myc` (the first-order rendering nodule this generalizes); the `mycelium-transpile` `write!`/`format!` lowering (new work, M-id FLAGGED §8). |
| **Grounds on** | RFC-0032 D4 (`bytes.*` never-silent byte-string access); RFC-0033 §4.1.2 (`bin.div`/`bin.rem` = surface `div_u`/`rem_u`, M-888); DN-43/M-799 (`bytes_concat` surface-callable, FLAG-text-3 CLOSED); DN-41/M-798 (`width_cast` — checked `Binary` re-width, reused to narrow a remainder into `digit_byte`'s `Binary{8}` domain); M-715 (recursion executes three-way in `.myc`); DN-125 (value-threading — the `&mut Formatter` *parameter* lane; **Accepted on a parallel worktree branch, not yet merged to `dev`/`main`** — see §2 caveat); DN-54 (the `derive`→L0 facility DN-128 builds the render impls on); KC-3 (small kernel — no new prim for ergonomics); G2 (never-silent — an unrenderable value is an explicit residual, never fabricated text); VR-5 (no tag upgraded past its basis); KISS/YAGNI (the simplest surface that solves the PROBLEM wins). |
| **Verified-against** | `@dev fa53dc46` (this note's base) / cited sites re-checked at `b36ebdbe` / the H1/M1 strict-review fixes re-verified at `@a0f4b90c` (this leaf branch's tip). |
| **Date** | July 12, 2026 |
| **Task** | Design-first; build FLAGGED §8 (recommend minting; std `to_dec`/render + a `Show` prelude trait + the transpiler `write!`/`format!` rule). |

> **Posture (transparency rule / VR-5 / G2).** This is a **Draft design note**. Every claim is tagged at
> its established strength: facts re-derived from the tree carry `Empirical` + a `file:line`; the proposed
> mechanism and its leverage are `Declared`; prior-art parallels are `Empirical`. No claim is upgraded past
> its basis, and the note **argues against its own recommendation** (§7) per house rule #4.

---

## §1 The PROBLEM (not Rust's mechanism)

Rust's `Display`/`write!`/`format!` machinery solves one problem: **render a typed value into human-readable
text, splicing in the rendered forms of its sub-values.** Rust's *mechanism* for it is incidental and
non-native to Mycelium:

- a trait method `fn fmt(&self, f: &mut Formatter) -> fmt::Result` that **mutates a shared output sink**
  through a `&mut` borrow, and
- a `write!(f, "…{x}…", x)` macro that **appends** interpolated fragments to that sink.

Both mechanisms are alien to Mycelium's value semantics (ADR-003): there is no shared mutable sink, and
`&mut` is not a native reference form (DN-125). The design task is to solve the **problem**, not to emulate
the **mechanism** — the DN-110 §9 / DN-111 decision procedure applied.

**Why this is the highest-value formatting gap (grounded).** DN-34 §8.22 (the method-body Impl-lever
breakdown, `Empirical @b36ebdbe`) measured the `&mut Formatter` `Display`/`Debug::fmt` bucket at **30
sub-issues, all 30 "pure"** (fixing them alone would unblock the whole gap) — the single largest clean bucket
in the whole transpile corpus (`DN-34-…:1099`, `:1123`). But §8.22's sub-finding is the load-bearing one:
sampling all **30/30**, every body is `match self { V => write!(f, "…", …), … }` or a bare `write!(f, "…")`,
so the real blocker co-located underneath is the **`write!`/`format!` macro invocation**, which the transpiler
gaps independently — fixing only the signature yields **zero** movement (`DN-34-…:1126`–`1138`). Of the 30,
only **4/30** are pure string literals the existing `bytes.concat`-only surface could serve; **26/30**
interpolate non-`Bytes` values (`{n}`/`{limit}`/`{e}`), for which "there is **no** int→string / generic-Display
kernel prim (`grep`-confirmed empty in `mycelium-core/src/prim.rs`)" (`DN-34-…:1136`).

I **re-verified** the prim-surface claim directly (mitigation #14): `crates/mycelium-core/src/prim.rs`
(824 lines, `@fa53dc46`) declares exactly the `core`/`bit`/`trit`/`cmp`/`bin`/`flt`/`dense`/`vsa`/`bytes`/
`hash`/`fuse_join` groups — **no `str.*`, no `fmt.*`, no int→string, no `Show`/`Display` dispatch prim**, and
`PrimParadigm` has no first-class string paradigm. The §8.22 grep stands.

---

## §2 The native shape — a pure `render: T → Bytes`

Mycelium's value-semantic answer: **a value renders to a `Bytes`**, and interpolation is **concatenation of
rendered fragments**. There is no sink, no `&mut`, no ordering hazard.

- A `Display`/`Debug` impl `fn fmt(&self, f) { write!(f, "a{x}b{y}") }` is the native function
  `render(self) → Bytes = bytes_concat(bytes_concat(bytes_concat("a", render(x)), "b"), render(y))`, where the
  literals are `Bytes` constants and each `{…}` is a recursive `render` of the sub-value.
- **This is already the de-facto native idiom in the corpus** — `lib/std/content.myc:213`
  (`malformed_digest_display(e) → Bytes`, "the Rust `Display` impl, via `bytes_concat`") and
  `lib/std/fmt.myc` (`hex_digit`/`nibble_hi`/`to_hex` render a `Binary{8}` to ASCII via `lt`/`add_u`
  arithmetic + match trees, **no prim added**) already do exactly this, first-order and by hand. This note
  **generalizes the existing idiom into a surface + a transpiler lowering**, rather than inventing a new one
  (KISS — the corpus already voted with `std.fmt`).

**The `&mut Formatter` *parameter* is DN-125's lane, not this note's.** The receiver-threading (`fn fmt(&self,
f: &mut Formatter)` → a by-value form) is exactly the `&mut T` value-threading DN-125 already scopes; this
note takes that as given and instead settles the render **target** (`Bytes`) and the `write!`/`format!`
**body** lowering. The two compose: DN-125 removes the `&mut Formatter` from the signature, DN-127 replaces
the `write!`-into-`f` body with a `bytes_concat` render. Neither duplicates the other (checked against DN-125
§5/§6.2).

**Cross-worktree provenance caveat (M1, checked against the tree, not asserted from the citation alone).**
DN-125's own file states **Status: Accepted (2026-07-12, ratified under explicit maintainer delegation)** —
that is a real, checked status, not fabricated — but it lives only on the sibling worktree branch
`claude/leaf/gc2-integ-ratify-dn125-dn123` (`docs/notes/DN-125-Native-In-Place-Mutation-Through-A-Reference-Value-Threading.md`)
as of this fix's base (`@a0f4b90c`, this leaf branch's tip): `git merge-base --is-ancestor <that-branch> HEAD` is **false**, and
DN-125 is **absent** from `docs/notes/` on both `origin/dev` and `origin/main` at the same point. So from
*this* branch's vantage, DN-125 is **in-flight, parallel-worktree provenance** — Accepted where it was
authored, not yet a landed fact this note's own ancestry can see. **Merge-order note:** DN-127's
`&mut Formatter`-parameter handling (the receiver-threading half of §6/§8) is only actionable once DN-125's
branch lands on `dev` (bringing its M-1081 build issue with it); until then this citation should be read as
"a parallel Accepted DN this note takes a dependency on," not as an already-landed decision on `dev`/`main`.
This does not change DN-127's own mechanism (§2/§5/§6), only the honesty of the cross-reference.

---

## §3 The primitive question — is a new kernel prim needed? (the crux)

§8.22 frames the 26/30 interpolating bodies as blocked by a missing "int→string / generic-Display kernel
prim." **I challenge that premise (VR-5 / house rule #4 — do not inherit an unchecked framing).** A kernel
prim is *sufficient* but the evidence says it is **not necessary**:

**Int→decimal-`Bytes` is derivable in `.myc` today from the landed prim set** (`Empirical`, all four
building blocks re-verified `@fa53dc46` (this note's original base); the corrected sketch below
(H1 fix) additionally re-checked against `@a0f4b90c`, this fix's own base):

1. `div_u` / `rem_u` are **surface-callable** and map to `bin.div` / `bin.rem`
   (`crates/mycelium-l1/src/checkty.rs:10241`–`10242`; prims exist, RFC-0033 §4.1.2 / M-888).
2. `bytes_concat` is **surface-callable** — FLAG-text-3 **CLOSED** by DN-43/M-799
   (`lib/std/text.myc:99`–`102`; the `Exact` kernel prim `bytes.concat` wrapped as a total surface fn) —
   and it **requires `Bytes` on both operands** (runtime-enforced `as_bytes_payload`,
   `crates/mycelium-interp/src/prims.rs:1723`–`1735`; statically enforced too,
   `crates/mycelium-l1/src/checkty.rs:8676`–`8695`). There is **no landed `Binary→Bytes` construction
   prim** (`bytes.*` is exactly `len`/`get`/`slice`/`concat`/`eq` + `hash.blake3`,
   `crates/mycelium-core/src/prim.rs:400`–`410`) — so every digit fed to `bytes_concat` must already be
   a `Bytes` value, never a `Binary{N}` (this is the correction §H1 below applies).
3. **Recursion executes three-way** in `.myc` (L1-eval ≡ L0-interp ≡ AOT) — `lib/std/iter.myc`'s
   `map`/`filter`/`foldl` are recursive HOFs, the re-pass gap CLOSED by M-715 (`iter.myc:11`–`13`).
4. `width_cast` is **surface-callable** — a checked `Binary{N}→Binary{M}` re-width (DN-41/M-798;
   `crates/mycelium-l1/src/checkty.rs:8697`–`8730`), already used exactly this way (narrowing a
   provably-in-range value) by `lib/std/text.myc:165` (`byte_at`) and `:174` (`widen8`).

So the classic itoa recurrence is expressible with **no kernel growth** — corrected below to actually
type-check (the version originally drafted here called `digit_byte(n)` on the `Binary{64}` scrutinee `n`
and fed its result straight to `bytes_concat`; per point 2 above `bytes_concat` demands `Bytes`, so a
`digit_byte` that returns `Binary{8}` (e.g. `add_u(n, 0x30)`) does not type-check, and the `n: Binary{64}`
argument doesn't fit a `Binary{8}`-typed `digit_byte` parameter either — **two** distinct type errors in
the original sketch, both fixed here):

```
// std.fmt (proposed) — Native Equivalent, Exact over the in-range domain, no new prim.
// digit_byte: a total, finite literal-match over the ten decimal-digit values, returning the ASCII
// digit byte directly as a one-byte `Bytes` literal (never a `Binary{8}` needing a further
// Binary→Bytes conversion — there is none, see point 2 above). Bare-decimal patterns `0`..`9` resolve
// against the `Binary{8}` scrutinee type (RFC-0012 §4.3 ambient-int resolution,
// `crates/mycelium-l1/src/checkty.rs:8231`, `:8318`–`8330`) — the same total-over-a-finite-domain
// shape `hex_digit` uses (`lib/std/fmt.myc:39`–`43`), generalized from a `lt`-bridged Binary nibble
// table to a direct literal-match Bytes digit table. `_ => "?"` is the never-silent fallback (G2),
// unreachable for an in-range digit — the same shape as `hex_digit`'s `_ => 0b0011_1111` ('?').
fn digit_byte(d: Binary{8}) => Bytes =
  match d {
    0 => "0", 1 => "1", 2 => "2", 3 => "3", 4 => "4",
    5 => "5", 6 => "6", 7 => "7", 8 => "8", 9 => "9",
    _ => "?"
  };

// to_dec: most-significant-digit-first decimal render of a Binary{64} value. `n`/`div_u(n,ten)`/
// `rem_u(n,ten)` all stay Binary{64} (div_u/rem_u are same-width ops); each single-digit value is
// narrowed to `digit_byte`'s Binary{8} domain via the checked `width_cast` (point 4 above) —
// always in-range here (`n < ten` in the base case, `rem_u(n,ten) < ten` in the recursive case, and
// `ten = 10 < 256`), so the narrow never actually refuses, though that in-range-ness is a Declared/
// never-silent runtime contract (width_cast's own contract, DN-41), not machine-proven.
fn to_dec(n: Binary{64}) => Bytes =
  match lt(n, ten) {                                      // n < 10 → single digit
    0b1 => digit_byte(width_cast(n, 0b0000_0000)),
    _   => bytes_concat(to_dec(div_u(n, ten)), digit_byte(width_cast(rem_u(n, ten), 0b0000_0000)))
  };
```

**Both operands to `bytes_concat` are now `Bytes`** (the recursive `to_dec(div_u(n, ten))` call returns
`Bytes` by the function's own signature; `digit_byte(…)` returns `Bytes` by construction) — the sketch
type-checks. This is the **same shape** `hex_digit`/`to_hex` already use for base-16 (a total match over
a finite domain, no kernel prim), extended to base-10 with a recursive most-significant-digit-first
accumulation plus the `Bytes`-returning digit table point 2 above forces. The `spore.myc:66` FLAG-spore-6
("no div/rem prims resolve end-to-end, so int→decimal text is not expressible") and `math.myc:37` ("the
width-generic self-hosted division not surfaced") describe an **absent std function**, which their authors
read as needing a prim — but the fixed-width `div_u`/`rem_u` + `bytes_concat` + `width_cast` + recursion
already close it for a concrete width (`Binary{64}`). The honest residual is **width-genericity** (a
`to_dec` per width, or a width-polymorphic std fn once RFC-0033's width-generic division surfaces),
**not** a kernel prim.

**KC-3 verdict, re-confirmed against the corrected sketch:** `width_cast` is not a *new* prim — it is
already landed (DN-41/M-798) and already surface-callable, reused here exactly as `lib/std/text.myc`
already reuses it (point 4 above); its addition to this derivation does not reopen the "no new kernel
prim" conclusion, it only makes the existing-prim inventory this note draws from complete and honest.
The "no new kernel prim" conclusion **still holds**.

**Verdict on the crux (the design-reasoner's sharpest correction to the inherited premise):** the movers are a
**std-library build + a transpiler lowering rule**, *not* a kernel prim. KC-3 stays intact.

---

## §4 Ranked alternatives + objective function

Objective (criteria table; each option scored against the same function — the maintainer ratifies on this):

| Criterion (weight) | Alt A: new kernel prim `bin.to_dec`/`fmt.render` | Alt B: **std `to_dec` + `Show` prelude trait + transpiler `write!`/`format!` rule** | Alt C: transpiler-only `write!`→`bytes_concat` (no `Show`, literals + already-`Bytes` only) |
|---|---|---|---|
| **KC-3 (small kernel)** — highest | ✗ grows the kernel for ergonomics (ADR-045 unfrozen ≠ license) | ✓ **zero kernel growth** — derived from landed prims | ✓ zero kernel growth |
| **Solves the 26/30 interpolating bucket** | ✓ | ✓ (`to_dec` + `Show::render` dispatch) | ✗ only the 4/30 pure-literal bucket |
| **Value semantics (ADR-003)** | partial (a prim sink is fine, but no generic dispatch) | ✓ pure render, generic via static-dispatch trait (DN-55, monomorphizes away) | ✓ |
| **Never-silent (G2)** | ✓ | ✓ an unrenderable type = explicit gap / missing `Show` impl | ✓ |
| **Genericity (arbitrary `{x}`)** | ✗ a prim can't dispatch on user types | ✓ `Show` is the dispatch surface | ✗ |
| **Verification cost** | low (one prim + differential) | medium (std fn + trait seed + transpiler rule + emit↔check agreement test) | low |
| **Reversibility / YAGNI** | hard to walk back a kernel prim | ✓ std + transpiler rule freely revisable | ✓ but under-solves |

**Recommendation (ranked): Alt B ≻ Alt C ≻ Alt A.** Alt B dominates: it closes the full 26/30 interpolating
bucket **and** the 4/30 literal bucket with **zero kernel growth**, generic dispatch, and full value semantics.
Alt C is the honest *fallback increment* (ship the literal/already-`Bytes` `write!`→`bytes_concat` lowering
first — it moves the 4/30 immediately with the least risk), but it under-solves. Alt A is rejected on KC-3
unless §3's derivation is later disproven for a width Mycelium must support (the honest escape hatch: **if** a
width-generic `to_dec` proves inexpressible without a shift/width-change prim, a *single narrow* `bin.to_dec`
prim is the minimal fallback — but the evidence says it is not needed, so YAGNI forbids building it now).

---

## §5 The `Show` prelude trait (generic dispatch)

For `{x}` where `x` is an arbitrary type, interpolation needs a dispatch surface. The native form is a
**single-parameter, param-only prelude trait** — exactly the DN-122 §13 shape that checks clean today:

```
// prelude (seeded conditionally like Fuse/Ord3; see DN-129 for the seeding mechanism)
trait Show[T] { fn render(x: T) => Bytes; }
```

- **Seeding** mirrors `Fuse` (`checkty.rs:1358`–`1369`) and `Ord3` (`checkty.rs:1372`–`1381`, M-1080): present
  in the linked env **iff** some nodule declares `impl Show[…] for …`. Zero `use`/manifest emission (DN-122
  OQ-6 "prelude-seed" resolution). **This note does not re-decide the seeding mechanism — it is DN-129's, and
  `Show` rides it.**
- **`Show` is single-param, param-only** (`Show[T] { fn render(x: T) => Bytes }`) — the DN-122 §13.1 admitted
  class, so it needs **no new checker work** (the foreign-sig guard does not fire; the orphan rule admits
  `impl Show for LocalType`). This is why `Show` is cheap where a two-type trait would not be.
- **`render` (not `display`/`fmt`)** — `display` is not a keyword (safe), but `Show`/`render` pass the DN-02
  T-map/T-illuminate/T-learn gate better (a value is *shown*/*rendered*, not "formatted into a sink"). Final
  naming is **DN-02-gated** (flagged, not fixed here).
- **`Debug` vs `Display`.** Rust splits user-facing (`Display`) from structural (`Debug`). The native MVP is
  **one `Show`** (user-facing render); a structural/`Debug`-shaped render is DN-128's derive concern (a derived
  `render` that emits `Ctor(field, …)`), not a second kernel/prelude concept. Keep the prelude to one trait
  (KISS); DN-128 layers the structural derive on top.

---

## §6 The kernel / std / transpiler split

| Layer | What it owns here | Grounding |
|---|---|---|
| **Kernel** | **Nothing new** (KC-3 held). Reuses `bin.div`/`bin.rem` (M-888), `bytes.concat` (RFC-0032 D4), `add_u`. | §3; `prim.rs @fa53dc46` |
| **Std (`lib/std/fmt.myc`)** | `to_dec` (int→decimal `Bytes`, per width), `digit_byte`, sign handling; the `Show[T]` prelude trait decl + `impl Show for {Binary{N}, Bytes, Bool, …}`; generalizes the existing `hex_digit`/`to_hex`. | `fmt.myc`; §3 recurrence |
| **Prelude (`checkty.rs`)** | Seed `Show` conditionally, **by DN-129's mechanism** (mirror `Fuse`/`Ord3` at `checkty.rs:1358`/`1372`). | §5; DN-129 |
| **Transpiler (`emit.rs`)** | A `write!`/`format!` **lowering rule**: parse the format string, emit `bytes_concat` of literal-`Bytes` and `render(argᵢ)` fragments; the `Display`/`Debug::fmt` body → a `render(self) → Bytes` fn (with DN-125 removing the `&mut Formatter` param). Emit↔check agreement test (the emitter only ships a `render` call where a `Show` impl resolves). Otherwise **emit the honest gap** (no fabricated text, G2). | §2; DN-34 §8.22; DN-125 |

**Disjoint work-units** (for the FLAGGED build, `Declared` sizes):
- **WU-1 (std):** `to_dec` + `digit_byte` + sign, and `impl Show` for the primitive reprs, in `lib/std/fmt.myc`
  — the DN-125-independent piece, moves the numeric-interpolation coverage.
- **WU-2 (prelude):** seed `Show` (trivial once DN-129's seed helper exists).
- **WU-3 (transpiler):** the `write!`/`format!` format-string parser + `bytes_concat`-fragment emitter +
  the `Display`/`Debug::fmt` body rule; land **Alt C first** (literals/already-`Bytes`), then the `Show`
  dispatch. Property tests: (T-1) a pure-literal `write!` emits + `myc check`s clean (the 4/30); (T-2) a
  `{n}` numeric interpolation emits `to_dec(n)` and checks clean; (T-3) emit↔check agreement — no `render`
  call shipped for a type with no `Show` impl (honest gap instead).

---

## §7 Adversarial stress-test (VR-5 / house rule #4 — attack the recommendation)

1. **"Recursion in `.myc` might not terminate / might not myc-check for `to_dec`."** *Attacked:* the itoa
   recurrence decreases (`div_u(n,10) < n` for `n ≥ 10`), and `iter.myc`'s recursive HOFs already `myc
   check` + execute three-way (M-715). *Result: HELD for a concrete width.* **Honest residual:** width-generic
   `to_dec` (one fn per width until RFC-0033's width-generic division surfaces) — flagged, not glossed. This
   does **not** reopen Alt A: per-width `to_dec` is still zero-kernel.
2. **"Negative / signed integers."** *Attacked:* `to_dec` above is unsigned (`div_u`/`rem_u`). *Result:
   NARROWED.* Signed render = sign test (`cmp.lt_s` vs 0) + `bytes_concat("-", to_dec(neg_s(n)))`, all landed
   prims — no new mechanism, but it **must be written**, not assumed. Added to WU-1.
3. **"Float rendering (`{f}` for a `Float`)."** *Attacked:* ADR-040 floats have no decimal-render prim, and
   correct float→shortest-decimal (Ryū/Grisu) is genuinely hard and **not** derivable trivially. *Result:
   HONEST GAP.* The MVP's `Show` covers `Binary{N}`/`Bytes`/`Bool`/user-`Data`; **float `Show` is an explicit
   residual** (a reified swap with its bound per ADR-040 §2.4, or a later dedicated fn), refused never-silently,
   not fabricated. This is the one place a future prim/std-fn is a real open question — flagged as OQ-1.
4. **"Genericity: `write!("{x}")` for a type with no `Show` impl."** *Result: HELD by G2.* The emit↔check
   agreement test (T-3) forces an honest gap, never a fabricated render — matching DN-122 §13's emit↔check
   discipline exactly.
5. **"Does this actually move `checked_fraction`, or is it another zero-yield lever (the §8.14 D3/D4
   precedent)?"** *Attacked — the §8.22 verdict was that fixing the signature alone yields zero.* *Result:
   HELD as a genuine mover, with a caveat.* Unlike the signature-only lever, Alt B closes the **actual
   blocker** (the `write!`/`format!` body) for the 4/30 literal bucket immediately (Alt C) and the 26/30
   numeric bucket once `to_dec`+`Show` land — but the **measurement is the deliverable**: the leverage figure
   stays `Declared` until a re-run of the M-1006 ladder / DN-124 phylum-mode vet witnesses it. I do **not**
   claim a percentage.
6. **"Is `Show` really cheaper than just special-casing `Display` in the transpiler (Alt C forever)?"**
   *Result:* Alt C alone under-solves (only literals), and a transpiler that hard-codes per-type render logic
   duplicates what a `Show` impl expresses once — Alt B is DRY. But Alt C **first** is correct sequencing
   (ship the safe increment, measure, then add dispatch).

---

## §8 Definition of Done (what ratification requires of the maintainer) + FLAGs

**Ratifying this note = accepting the mechanism** (pure `render: T → Bytes`; `write!`/`format!` →
`bytes_concat` fragments; int→decimal in std from landed prims, **no kernel prim**; a `Show` single-param
prelude trait dispatched for interpolation) **and the honest boundaries** (float render = OQ-1 residual;
width-generic `to_dec` = residual; DN-125 owns the `&mut Formatter` param). It does **not** enact code.

**DoD for `Enacted`** (the gate, house rule #6): (1) `lib/std/fmt.myc` `to_dec` (unsigned+signed, per
supported width) + `impl Show` for the primitive reprs, `myc check`-clean and three-way differential-witnessed;
(2) `Show` seeded in the prelude (DN-129 mechanism) with a T-B1-style visibility test; (3) the transpiler
`write!`/`format!` lowering (Alt C literals first, then `Show` dispatch) with the T-1/T-2/T-3 property tests
green and emit↔check agreement proven; (4) an M-1006 ladder / DN-124 vet re-measure that **quantifies** the
`checked_fraction` movement on the 30-body bucket (the leverage figure is `Declared` until this runs).

**FLAGs (append-only rows the integrating parent applies — I do not edit these files):**
- **CHANGELOG.md** — add a Draft-DN row for DN-127 (this authoring).
- **docs/Doc-Index.md** — register DN-127 (Draft).
- **tools/github/issues.yaml** — mint build issues (READ-ONLY here; recommend the IDs, parent assigns; note
  M-1081 is claimed by DN-125's parallel worktree, so start at M-1082):
  - **`M-⟨new-a⟩` — std `to_dec` + `Show` prelude trait + `impl Show` for primitive reprs** (`lib/std/fmt.myc`,
    `mycelium-l1` seed). Depends on DN-129 (seed helper).
  - **`M-⟨new-b⟩` — transpiler `write!`/`format!` lowering** (`crates/mycelium-transpile/src/emit.rs`) — Alt C
    literals first, then `Show` dispatch; emit↔check agreement test. Depends on M-⟨new-a⟩ + DN-125's M-1081.
  - **OQ-1 (float render)** — track as a residual issue (reified swap per ADR-040 §2.4, or a dedicated
    shortest-decimal fn), not part of the MVP.

---

## Meta — changelog

- **2026-07-12 — Created (Draft, design-reasoner pattern).** Scopes Mycelium's native answer to the
  `Display`/`write!`/`format!` value-to-text problem (DN-34 §8.22's 30/30 `&mut Formatter` bucket): a pure
  `render: T → Bytes` (Idiomatic Remapping of the mutating sink), `write!`/`format!` → `bytes_concat`
  fragments, int→decimal **in std from landed prims** (`div_u`/`rem_u`/`bytes_concat`/recursion — **no kernel
  prim**, the sharpest correction to §8.22's inherited "need a prim" premise, KC-3 held), and a `Show`
  single-param prelude trait for generic dispatch (rides DN-129's seed mechanism). Ranked Alt B ≻ Alt C ≻ Alt A;
  adversarially held with float-render (OQ-1) and width-generic-`to_dec` as honest residuals. Cross-references
  DN-125 for the `&mut Formatter` *parameter* (not duplicated). All mechanism/leverage `Declared`; the
  building-block facts (`div_u`/`rem_u`/`bytes_concat`/recursion/`prim.rs`) `Empirical` + `file:line`.
  **Recommends, does not ratify** (house rule #3). Enacts nothing. (Append-only; VR-5; G2.)
- **2026-07-12 — Strict-review fixes (H1, M1), @a0f4b90c.** **H1 (load-bearing):** the §3 `digit_byte`/
  `to_dec` sketch did not type-check — the original `digit_byte(n)` returned a `Binary{8}` (`add_u(n, 0x30)`)
  fed straight to `bytes_concat`, which requires `Bytes` on both operands
  (`crates/mycelium-interp/src/prims.rs:1723`–`1735`; `crates/mycelium-l1/src/checkty.rs:8676`–`8695`) and
  there is no landed `Binary→Bytes` construction prim (`crates/mycelium-core/src/prim.rs:400`–`410`); `n`'s
  `Binary{64}` width also didn't fit a `Binary{8}`-typed `digit_byte` parameter. **Fixed:** `digit_byte`
  now returns `Bytes` literals directly via a finite literal-match (mirroring `hex_digit`'s total-over-a-
  finite-domain shape, `lib/std/fmt.myc:39`–`43`, generalized via the ambient-int pattern-resolution
  mechanism, `checkty.rs:8231`, `:8318`–`8330`), and `to_dec` narrows each digit through the already-landed
  `width_cast` (DN-41/M-798, `checkty.rs:8697`–`8730`) before calling it. The corrected sketch type-checks
  (`Bytes` in, `Bytes` out; both `bytes_concat` operands `Bytes`); the **KC-3 "no kernel prim" conclusion
  still holds** — `width_cast` is an existing, already-surface-callable prim, not a new one. **M1:** the
  DN-125 cross-reference cited "DN-125 (Accepted 2026-07-12)" as if landed; DN-125's Accepted status is real
  but checked *on its own sibling worktree branch* (`claude/leaf/gc2-integ-ratify-dn125-dn123`), not yet
  merged into this branch's ancestry, `origin/dev`, or `origin/main` — corrected to an explicit cross-
  worktree/in-flight-parallel-DN caveat with the merge-order dependency stated (§2). Both fixes are
  append-only edits to this still-Draft note; no status change. (VR-5; G2.)
