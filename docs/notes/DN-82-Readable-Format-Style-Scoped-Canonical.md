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
