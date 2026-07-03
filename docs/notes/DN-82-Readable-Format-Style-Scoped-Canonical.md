# Design Note DN-82 — `mycfmt` Readable Format Style: a scoped, human-facing canonical

| Field | Value |
|---|---|
| **Note** | DN-82 |
| **Status** | **Accepted** (2026-07-02) — *Rust-first implemented and green, pending maintainer ratification to Enacted.* The readable multi-line style (`format_source_readable` / `mycfmt --readable`) and the scoped `myc-fmt` gate enforcement for `lib/std` are landed in `crates/mycelium-fmt` (M-974) with the full `mycelium-fmt` + `mycelium-l1` `std_*` + `mycelium-std-conformance` suites green (behaviour-neutrality proven — see §5). Minted under the maintainer's explicit delegation of the canonical-form choice (§1); ratification `Accepted → Enacted` is the maintainer's (VR-5: the decision is implemented, not yet ratified). |
| **Feeds** | DN-57 (the `;`-terminator + `--flatten`/`--stream` machine/stream form — this note is its human-facing counterpart); RFC-0001 §4.6/§4.8 + ADR-003 (formatting is an identity-preserving projection); the `mycfmt` formatter contract (`docs/spec/Mycfmt-Formatter-Contract.md`). |
| **Date** | July 2, 2026 |
| **Decides** | The **human-readable multi-line layout** is a *scoped* canonical: it is the form the `myc-fmt` gate enforces for the **human-authored stdlib** (`lib/std/*.myc`), while the compact form stays the default canonical for `examples/*` and any generic `.myc` root, and `--flatten`/`--stream` stay the explicit machine/stream forms. |

> **Posture (transparency / VR-5 / G2).** The readable style is **presentation-only and functionally
> inert**: its output re-parses to the *same* surface AST as the compact form (C1) and is a fixed
> point (C2). Machines ingest the flattened stream / full file either way, so the layout choice never
> changes any parse/elaborate/eval behaviour. The `READABLE_WIDTH = 88` threshold is `Declared` — a
> readability heuristic, not a proven bound. The behaviour-neutrality of the `lib/std` reformat is
> `Empirical` (the `std_*` differential/three-way suites, §5).

## 1. The decision

Mycelium already canonicalizes `.myc` source toward a **flattened** machine/stream form: DN-57's
mandatory `;` terminator (M-818), `mycfmt --flatten` (M-819), and `myc --stream` (M-820) make the
whole nodule a single, whitespace-independent component stream. That is right for *machines*. But a
*human* editing `lib/std/*.myc` reads long inlined segments — deeply-nested `match`, 6-argument
calls, 8-variant sum types on one 200-column line — poorly.

M-974 adds the **inverse posture**: a **readable multi-line style** that breaks long argument /
value-parameter / sum-type-variant / match-arm segments across lines (line breaks after commas and
`|`), indents nested structure, and keeps short constructs inline (the `READABLE_WIDTH` heuristic —
a construct whose compact single-line render fits at its indent column stays inline; otherwise it
breaks).

**Canonical-form question (delegated to the implementer by the maintainer).** Which form does the
`myc-fmt` gate enforce for `.myc` *source*? Two clean readings:

- **(A) Global flip.** Make the readable form the default canonical `mycfmt --check` enforces for
  *all* `.myc` source; `--flatten` stays the explicit machine form.
- **(B) Scoped canonical.** Enforce the readable form only for the **human-authored stdlib**
  (`lib/std/*.myc`); leave the compact form as the default for `examples/*` and tiny fixtures.

## 2. Decision: (B) scope the readable canonical to `lib/std`

Reasons, ranked:

1. **Ownership + churn.** A global flip forces `examples/*.myc` (four `repr-tour` nodules + two
   `hello-phylum`) to reformat too. Those are not part of the M-974 change surface, and reformatting
   them would either widen the blast radius or leave the `myc-fmt` gate **red** (the examples would
   report "would reformat"). Scoping to `lib/std` — the files this note is *about* — keeps the gate
   green with the minimum touched set.
2. **No race with the grammar-conformance wave.** A concurrent `grm/frz` wave is still *adding* small
   single-construct fixtures under `docs/spec/grammar/conformance/`. Those fixtures are **not** under
   a project root (no `mycelium-proj.toml`), so the `myc-fmt` gate never checks them — and the
   `mycelium-fmt` conformance test is **behaviour-based** (C1 round-trip + C2 idempotence, not a byte
   golden), which the readable style satisfies exactly as the compact style does. Scoping therefore
   guarantees **zero interaction** with the fixtures those agents are landing; a global flip would
   have to reason about every fixture's layout.
3. **The distinction is real, not incidental.** `lib/std` is *human-authored, human-maintained*
   surface; `examples/*` and fixtures are demonstrative/adversarial. Enforcing readability exactly
   where a human reads-and-edits, and compactness elsewhere, matches the actual audiences. `--flatten`
   / `--stream` remain the machine forms for both.

The cost of (B) is one extra concept — a per-root *style* — carried by the `myc-fmt` gate. That is a
one-line `case` in `scripts/checks/myc-fmt.sh`; the CLI and library carry the style as an explicit
parameter (`Style::{Compact, Readable}`), never an implicit mode.

## 3. Mechanism

- **Library.** `mycelium_fmt::format_source_readable(src, pin)` (public) renders the readable style;
  it shares every scope/identity/header guard with `format_source` via `format_source_styled(src,
  pin, Style)`. `format_source` is exactly `format_source_styled(.., Style::Compact)` — byte-for-byte
  unchanged (regression-tested).
- **CLI.** `mycfmt --readable` selects the readable style. It is mutually exclusive with `--flatten`
  (opposite layout postures — an explicit usage error, never a silent precedence, G2). It preserves
  comments + the structured header exactly like the default form.
- **Gate.** `scripts/checks/myc-fmt.sh` passes `--readable` for the `lib/std` root only; every other
  root uses the compact default. The style flag is the *sole* per-root difference; both are
  `--check` (writes nothing, exit 1 on drift).

## 4. Readability heuristic (what wraps)

A construct wraps iff its compact single-line render, placed at its current indent column, would
exceed `READABLE_WIDTH` (88), or a child already broke. When it wraps:

- **Function application** `f(a, b, …)` — one argument per line, break after each comma.
- **Value-parameter list** `fn name(p1, p2, …)` — one parameter per line (the sum-type `[T]` /
  width `{N}` lists and the return/effects stay inline).
- **Sum type** `type T = C1 | C2 | …` — one constructor per line, break after each `|`.
- **`match`** — one arm per line; an arm with a long/nested body puts the `=>` on its own line and
  the body on the next (indented) line, so deeply-nested matches stay shallow rather than marching
  rightward.
- **`let … in …`**, **`if … then … else …`**, **tuple** and **list literals** — split across lines.

Short constructs stay inline. Because every decision is driven by the AST (never the input text's
existing layout), the render is **idempotent by construction** (C2).

## 5. Definition of Done — met

- **The readable style exists** in `mycelium-fmt` with `--readable` on the CLI; `--flatten` /
  `--stream` intact. ✔
- **Round-trip (C1) + idempotence (C2).** The existing runtime identity guard refuses any non-round-
  tripping format (exit 4, never a garbled rewrite); no `lib/std` file is refused. Corpus unit tests
  (`readable_round_trips_and_is_idempotent`, `readable_wraps_long_and_keeps_short_inline`,
  `styled_compact_equals_default_format`) are green, and all 17 `lib/std` nodules re-format to a fixed
  point. ✔
- **`lib/std/*.myc` reads readably** — the 14 nodules with long inlined segments are reformatted;
  `math`/`option`/`result` already fit and are unchanged. ✔
- **`myc-fmt` gate green + consistent** — `lib/std` enforced `--readable`, examples enforced compact,
  all three roots `ok`. ✔
- **Behaviour-neutral (the proof).** The reformat changed *no* functional behaviour: the
  `mycelium-l1` `std_*` harnesses and `mycelium-std-conformance` differential/three-way tests
  (`std_core/diag/error/recover/select/spore/swap/ternary/testing`, plus
  `cmp/collections/fmt/iter/math/text/…`) `include_str!` these exact nodules and run L0-interp ≡
  L1-eval ≡ AOT — **all green, unchanged**, which is the behaviour-neutrality evidence (`Empirical`).
  ✔

## 6. Open / out of scope (FLAGs — VR-5)

- **FLAG-82-1 (grm/frz interaction — none).** M-974 touches **no** `docs/spec/grammar/conformance/`
  fixture. A pre-existing, *unrelated* failure —
  `mycelium-std-conformance::reject_ledger::parse_level_reject_corpus_matches_the_ledger` — is caused
  by a new reject fixture (`31-old-le-ge-glyph-retired.myc`) added by the concurrent `grm/frz` wave
  with no DN-80 §3 ledger row yet. That is owned by the ledger/grammar wave, not this note.
- **FLAG-82-2 (width is `Declared`).** `READABLE_WIDTH = 88` is a chosen threshold, not a proven
  ergonomic optimum. Changing it is a presentation tweak (still C1/C2-safe), not a semantics change.
- **Out of scope.** Whole-phylum readable formatting, trailing-comment reflow inside wrapped
  segments, and a configurable width (via `[toolchain]`) are future increments — captured, not
  claimed.

## 7. M-976 — Shape-Dispatched Readable (flat-spine · rustfmt-block · honest-tree)

> **Status addendum (2026-07-02).** DN-82 stays **Accepted**; M-976 is an append-only *refinement* of
> the readable renderer (still whitespace-only, still C1/C2-guarded, still `Empirical` behaviour-
> neutral), not a new decision. It supersedes nothing — it makes the §4 heuristic shape-aware so the
> maintainer-flagged "deepening Cons pyramid" and its mirror "vertical closer wall" both disappear.

### 7.1 The rule-set (R0–R6)

One width threshold (`READABLE_WIDTH = 88`, the shipped `Declared` const) is the SOLE inline-vs-break
trigger (**R0**); four AST-shape rules decide HOW a node that must break lays out, dispatched in a
single deterministic order (**R6**: same-head chain, else App, else match/if, else let, else per-kind):

- **R1 — flat spine for right-nested same-head chains.** A chain `H(a, H(b, … H(z, T)))` (Cons /
  GLCons / TCons / bool_and / cat, depth ≥ 2) renders every link at ONE fixed indent (never +2 per
  link), the terminal `T` beginning the final line. A semantically FLAT N-element list is no longer
  drawn as an N-deep rightward-drifting tree; appending an item touches one line.
- **R2 — wide-flat call → rustfmt block.** A breaking non-chain call is one argument per line at
  indent + 2 with the `)` alone on its own line — reproducing the confirmed-good `Row(...)` anchor
  byte-for-byte.
- **R3 — genuine-tree indent.** `match` / `if` get one indent per REAL nesting level; a nested-match
  body RIDES its arm line (`pat => match … {`), halving a linear guard-ladder's depth versus the old
  +4/level.
- **R4 — binding layout.** A `let` bound blocks at the let's OWN indent (structural, rename-stable —
  never aligned to the `= callee(` column), and the body always starts on its own line. **R4c**: an
  arm whose body is a top-level `let … in` always breaks open, surfacing the twice-flagged buried
  binding.
- **R5 — trailing-closer collapse (global invariant).** No closing delimiter is ever part of a
  vertical one-per-line stack: a spine coalesces ALL its closers into ONE horizontal run on the
  terminal line, and a wide-flat / tree break emits at most one `)`-line per real nesting level. So a
  lone closer line's count equals tree DEPTH (small), never a flat list's ITEM count — the anti-wall
  guarantee.

The 10 canonical `lib/std` samples (`matrix`, `only_query_rows_explainable`, `row_guarantee_of`,
`testing::matrix`, `pack_tl1`, `tl1_group`, `explain_deployed`, `rng_next`, `inspect`/`inspect_err`,
`guarantee_matrix`) are the acceptance oracle: the compact style reproduces each `after` byte-for-byte
(the `row_guarantee_of` wide-flat anchor and the `tl1_group` let-chain are asserted UNCHANGED). See
`crates/mycelium-fmt/src/tests.rs::shape_dispatched_samples_reproduce_after_byte_for_byte`.

### 7.2 The house-style knob (required)

A `LayoutCfg { width, spine_inner }` is exposed (`format_source_readable_cfg`; `mycfmt --readable
--expand-spine`). `spine_inner` picks the same-head-chain inner-argument layout:

- **`InlineWhenFits` (default, compact).** A fitting inner call stays inline (the shipped `lib/std`
  canonical); an overflowing one blocks per R2.
- **`AlwaysExpand` (expanded house style).** The spine STILL stays flat (each link at one indent, no
  pyramid), but every inner nested call is broken onto its own lines. Both kill the pyramid; they
  differ only in inner-call density, and BOTH are behaviour-neutral (C1/C2).

**Width retune — 88 → 100 (`READABLE_WIDTH`, M-976 maintainer decision).** The earlier `88` was
Black's Python default — an arbitrary import that misleadingly ties a value-semantics systems language
to a Python formatter. The default is now **`100`**, `rustfmt`'s `max_width` default — *the formatter
the Mycelium Rust kernel already uses* — so the single R0 threshold is grounded in the project's own
toolchain, not borrowed. It stays `Declared` (a readability heuristic, not a proven bound) and is
overridable per call via `LayoutCfg::width`. The `lib/std` corpus was re-rendered at 100 (more inline,
net a further −249 lines) with every behaviour crate still green; the 10 spec samples (§7.1) remain the
oracle **at their defining width 88** (the acceptance test pins `width: 88` explicitly, validating the
R0–R6 rules against the exact fixtures independent of the retuned default).

### 7.3 Seq ≠ Vec — why the flat spine is the honest fix, not a `[…]` rewrite

The each-item-fully-closed-comma-`;` ideal (`[row_a(), row_b(), …];`) is **NOT reachable by whitespace
alone**, and the tempting source rewrite of the `matrix()` / `guarantee_matrix()` Cons chains to `[…]`
Seq literals is **behaviour-CHANGING** — grounded end-to-end against the pipeline: the `[…]` literal
(grammar `mycelium.ebnf`) is the RFC-0032 Seq literal — it types to `Ty::Seq(elem, len)`, elaborates
to `Repr::Seq`, and `eval.rs` explicitly REFUSES any element that is a data-constructor value ("a v0
Seq is built from repr elements only"). Every `matrix()` returns the user type `Vec[A] = Nil |
Cons(A, Vec[A])` whose rows are data-constructor values, so a rewrite fails on two independent grounds:
(1) the literal's type `Seq{Row, N}` ≠ `Vec[Row]` (signature + every `Cons`/`Nil`-pattern consumer
changes); (2) the rows are refused at elab/eval. The **flat spine (R1) is the correct, honest fix**;
the coalesced closer run (R5) on a spine terminal line is the irreducible residual of Cons tokens with
no list literal, flagged (not hidden) to the representation track.

### 7.4 Future-RFC FLAGs (out of scope — recommended, NOT enacted here)

- **FLAG-976-1 (list-literal RFC).** A separate append-only RFC for a Cons/Nil-**desugaring** `Vec`
  list literal (a `[a, b, c]` form that lowers to `Cons(a, Cons(b, …, Nil))` for a `Vec[A]`-shaped
  type), OR a `Seq{T,N} → Vec[T]` bridge admitting data-constructor elements. This is the ONLY path to
  the each-item-closed ideal that REMOVES the residual closer run — the Readable renderer already lays
  a list literal one element per line, so the day such a literal desugars to Cons, a mechanical source
  rewrite plus the existing formatter compose to exactly the ideal with zero further formatter change.
  It changes typing + the lowering surface, so it REQUIRES its own decision (three-property gate:
  flatten ≡ lower ≡ semantics unchanged).
- **FLAG-976-2 (variadic `all_of([…])` / `concat([…])`).** Same family, lower priority: a variadic
  conjunction / concatenation desugaring to the SAME right-fold (`bool_and` / `cat`) so the
  `only_query_rows_explainable` and `explain_deployed` pyramids can be written flat. Until it lands,
  R1's flat spine + R5's coalesced closer is the honest fallback.

### 7.5 Definition of Done — met (M-976)

- **R0–R6 + the house-style knob implemented** in `crates/mycelium-fmt` (`render_expr_readable` split
  into a fit-check + `render_expr_broken`; `same_head_chain` / `render_spine` helpers; `LayoutCfg` /
  `SpineInner`; `--expand-spine` CLI). ✔
- **Acceptance oracle green.** All 10 canonical samples reproduce their `after` byte-for-byte (the
  oracle pins width 88, its defining width); an `AlwaysExpand` case and structural shape invariants
  (flat links, single coalesced closer run, shallow tree, lone R2 closer) pass. ✔
- **`lib/std` re-rendered at the retuned default width 100** (`mycfmt --write --readable`; net −249
  lines versus the prior corpus — more inline at 100); the `myc-fmt` gate stays green under the same
  `--readable` scoping. ✔
- **Behaviour-neutral (`Empirical`, NOT `Proven`).** The C1 identity guard refused any non-round-
  tripping write, and the `mycelium-l1` `std_*` + `mycelium-std-conformance` + every touched
  `mycelium-std-*` eval crate are green after the reformat (L0-interp ≡ L1-eval ≡ AOT). This is trials
  evidence (C1/C2 + the differential suites), not a theorem — VR-5: stated at its supportable strength. ✔
