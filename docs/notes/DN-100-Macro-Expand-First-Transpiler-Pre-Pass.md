# Design Note DN-100 — Macro Expand-First Transpiler Pre-Pass (the toolchain decision for M-1032 / ENB-9)

| Field | Value |
|---|---|
| **Note** | DN-100 |
| **Status** | **Draft** (2026-07-10). The **toolchain DN** the M-1032 (ENB-9) issue requires before implementation. Authored as **READ + a new DN only** in the `crates/mycelium-transpile` lane; it **enacts nothing** and **moves no other doc's status** (house rule #3, append-only). It **decides how** the transpiler should expand Rust macros before translation — `cargo expand` vs a vendored expander vs a hybrid — recommends (ranked) but **does not self-ratify** (house rule #4); the maintainer ratifies. |
| **Decides** | *Proposes, for ratification:* (1) the **macro-expansion mechanism** for the transpiler's pre-pass (Alt A `cargo expand` · Alt B vendored `macro_rules` engine · Alt C hybrid/std-only · Alt D status-quo hand-expand); (2) the **honest ROI framing** — macro expansion raises `expressible_fraction`, with an *uncertain* `checked_fraction` effect (the dominant corpus macros expand to constructs the transpiler still gaps); (3) the **never-silent contract** for an unexpandable macro; (4) the **DoD** for M-1032. It does **not** edit `issues.yaml`, `CHANGELOG.md`, `Doc-Index.md`, `lib/compiler/**`, or `crates/mycelium-l1/**`. |
| **Lane / collision** | **`crates/mycelium-transpile/**` only** — `none` collision with the cloud semcore lane (M-1013). Confirmed by M-1032's own body: "lands OUTSIDE the semcore lane, no M-1013 coordination." |
| **Feeds** | M-1032 (ENB-9) implementation; DN-34 §8 (transpiler gap taxonomy — register row #51 macro-expand); DN-99 §4 Track B (transpiler closures). |
| **Date** | July 10, 2026 |
| **Task** | Decide the macro-expansion toolchain for the transpiler pre-pass, grounded in a whole-corpus macro profile. |

> **Grounding + honesty (transparency rule / VR-5 / G2 / house rule #4).** Every count below is
> `Empirical` — measured by running the transpiler over the whole `crates/` corpus (337 files, dev tip
> `8cd0a796`, 2026-07-10) and reading the `union.gap.json`. Every design not yet built/ratified is
> `Declared`. **No sycophancy:** the honest finding is that macro expansion is a **profiling/expressibility
> lever, not a guaranteed `checked_fraction` win** — the corpus's dominant macros expand into impls the
> transpiler *still* gaps (§3). No tag is upgraded past its basis.

---

## §1 Purpose and grounding

Register row #51 (DN-34 §8.3/§8.5) and DN-99 §8 (ENB-9) call for a **macro expand-first pre-pass**: run
a macro-expansion step over the Rust source *before* the transpiler's `syn`-item translation, so a
`matches!` / `format!` / a custom `macro_rules!` invocation becomes ordinary code the transpiler can map,
instead of an opaque `Category::MacroInvocation` gap. Today the transpiler records every macro invocation
as a never-silent gap (`transpile.rs`, `Category::MacroInvocation`) and the M-993/M-1006 port work
hand-expands each one (`matches!`→`match`, `format!`→byte encoders). The question this DN settles is
**which expansion mechanism** the pre-pass should use — a genuine toolchain decision with real
dependency/hygiene trade-offs.

**House-rule anchors:** the transparency lattice `Exact ⊐ Proven ⊐ Empirical ⊐ Declared` (rule #1);
never-silent gaps (rule #2, G2); append-only status (rule #3); grounded claims (rule #4); small auditable
kernel (rule #5, KC-3); YAGNI (don't build a general macro engine we don't need).

---

## §2 The whole-corpus macro profile (`Empirical`, dev tip `8cd0a796`)

Measured over all 337 `crates/**` files:

| Metric | Value |
|---|---:|
| `MacroInvocation` gaps (item-position macros) | **82** |
| distinct files carrying ≥1 macro gap | **18** |
| `MultiStmtBody` gaps citing a macro **inside a fn body** (body-level poison) | **13** |

**Macro-name breakdown of the 82 item-position invocations** (the ROI-shaping fact):

| Macro | Count | Kind | What it expands to |
|---|---:|---|---|
| `impl_narrow_int!` | 51 | project `macro_rules!` | `impl Narrow<…> for …` blocks |
| `impl_std_error!` | 25 | project `macro_rules!` | `impl std::error::Error for …` + `Display` |
| `thread_local!` | 2 | std `macro_rules!` | a `LocalKey` static |
| `impl_myc_ord!` / `impl_myc_partial_ord_float!` / `impl_narrow_f64_to_int!` / `impl_narrow_f32_to_int!` | 4 total | project `macro_rules!` | trait impls |

**The load-bearing observation (house rule #4):** the corpus's macro population is **≈93% custom
project `macro_rules!`** (`impl_narrow_int!` + `impl_std_error!` alone = 76 of 82), **not** the std
`format!`/`vec!`/`matches!` family that a lightweight "std-macro-only" expander could target. A
mechanism that only handled std macros would clear **≤6** of the 82.

---

## §3 The honest ROI: expressibility ≠ checkability

Macro expansion **necessarily raises `expressible_fraction`** (an expanded macro becomes visible items the
transpiler attempts). Its **`checked_fraction` effect is uncertain and likely small on this corpus**, for a
reason grounded in §2:

- `impl_narrow_int!` (51 invocations) expands to **`Narrow` impls**, which the transpiler **already gaps**
  by design — `Narrow::narrow` is fallible (`Result<To, NarrowError>`) with no `= expr` surface (DN-41 §3;
  `emit.rs` narrow arm). So expanding these 51 would move them from `MacroInvocation` gaps to `Impl`/
  `Conversion` gaps — **more honest categorization, not more checked-clean items.**
- `impl_std_error!` (25) expands to `Display`/`Error` impls (derive-attr / method-body territory that
  largely gaps).

This mirrors the **trx-increment finding** (DN-34 §8.21): a construct the transpiler *emits* is not
thereby *myc-check-clean*. So this DN frames M-1032 honestly as a **profiling + expressibility lever**
(it reveals the real post-expansion surface and de-opaques the `MacroInvocation` bucket) whose
`checked_fraction` payoff waits on the *downstream* closures (Narrow surface, Display encoders). The
M-1032 DoD's "before/after `expressible_fraction` on a macro-heavy target" is exactly the right metric —
**not** `checked_fraction`, which this pass should not be expected to move much (VR-5: don't claim a win
the mechanism can't deliver).

---

## §4 Alternatives

**Alt A — `cargo expand` (the real compiler's expansion).** Shell out to `cargo expand` (a `rustc
-Zunpretty=expanded` wrapper) per crate, then transpile the expanded output. **Pros:** correct for
**every** macro — custom `macro_rules!` *and* proc-macros *and* std — because it *is* the compiler's own
expansion (handles the 76 custom invocations that dominate §2). **Cons:** requires a **buildable crate + a
nightly toolchain** (a heavy, network/​build-order dependency — at odds with the change-scoped,
skip-gracefully check posture); expanded output is **desugared past the surface** (elaborated paths,
`#[allow]` noise, fully-qualified everything) so the transpiler's own path/type heuristics see a *harder*
input; hygiene-mangled identifiers may appear. **Never-silent:** a crate that fails to `cargo expand`
(build error, no nightly) is a recorded per-crate skip, never a silent empty result (G2).

**Alt B — vendored `macro_rules!` engine.** Vendor/implement a `macro_rules!` matcher-transcriber and
expand only *declarative* macros found in-tree. **Pros:** no toolchain/build dependency; expands the 76
dominant custom `macro_rules!`; stays in the crate's minimal-deps stance. **Cons:** **re-implements a
non-trivial slice of the compiler** (matcher fragment specifiers, repetition, hygiene) — a large,
bug-prone surface that **violates YAGNI/KISS** unless the corpus truly needs it; **cannot** expand
proc-macros or `derive`. Risk: partial/incorrect expansion is *worse* than an honest gap (a fabricated
expansion that mis-transpiles — VR-5/G2).

**Alt C — hybrid / std-macro-only shim.** A small hand-written expander for a fixed allowlist of std
macros (`matches!`, `format!`, `write!`, `assert!`, `vec!`) — the M-993 hand-expansions, mechanized.
**Pros:** tiny, dependency-free, exactly matches the constructs the port work already hand-expands; each
rule is auditable + testable. **Cons:** clears **≤6** of *this* corpus's 82 (the std tail), leaving the 76
custom `macro_rules!` gapped — **low ROI on the measured corpus**, though it removes the most repetitive
*hand* work in general port targets.

**Alt D — status quo (hand-expand at port time).** Keep every macro a never-silent `MacroInvocation` gap;
the porter hand-expands (DN-34 §8.3/§8.5). **Pros:** zero new machinery/deps; the porter's judgment
handles hygiene + intent. **Cons:** no automation of the 82; the opaque-bucket profiling stays coarse.

---

## §5 Recommendation (ranked; NOT ratified — house rule #4)

**Rank 1 — Alt A (`cargo expand`) as an *opt-in, off-by-default, never-gating* profiling mode**, *not* a
default pre-pass. Rationale: only Alt A actually handles the §2-dominant custom `macro_rules!`, and a
"gap-profiling instrument" (M-991) is exactly where the "de-opaque the `MacroInvocation` bucket, see the
real surface" value lands — run it deliberately on a target, read the before/after `expressible_fraction`,
feed the revealed downstream gaps back into the ladder. Gating it behind an explicit flag (and the
buildable-crate/nightly precondition, skipped-gracefully) keeps the toolchain dependency **out** of the
change-scoped `just check` path (parity with `transpile-vet`'s advisory posture).

**Rank 2 — Alt C (std-macro shim)** as an *independent, always-available* accelerator for the handful of
std macros, since it is tiny, dependency-free, and mechanizes real port-time hand-work — even though its
*measured-corpus* ROI is small. Ranks 1 and 2 are **complementary**, not exclusive.

**Not recommended:** Alt B (re-implementing `macro_rules!` — YAGNI/KISS/VR-5 risk) as a first step;
**Alt D** remains the honest fallback for anything the chosen mechanism cannot expand (never-silent gap).

**Adversarial stress-test of Rank 1.** *Does it actually raise the number the north star cares about?* No —
and the DN says so plainly (§3): on this corpus `checked_fraction` barely moves because the expanded
`impl_narrow_int!` bodies gap on `Narrow` fallibility. *Then why do it?* For **expressibility profiling**
(the M-1032 DoD's stated metric) and to **de-opaque** the 82-gap bucket, not for a checked win — and the
recommendation is explicitly *opt-in/advisory* so it never burdens the gate for an uncertain payoff. *Is
the toolchain dependency acceptable?* Only because it is opt-in + skip-graceful; a default pre-pass
requiring nightly would violate the check-parity/skip-gracefully posture, which is why Rank 1 is a mode,
not a default.

---

## §6 Never-silent contract (G2/VR-5)

Whichever mechanism lands: **an unexpandable macro is a recorded gap, never a fabricated expansion.** A
`cargo expand` failure (no nightly, build error) → a per-crate skip line (never a silent empty file); a
macro outside the chosen mechanism's scope → the existing `Category::MacroInvocation` gap, unchanged. The
expanded `.myc` stays **`Declared`** — expansion changes *what* is attempted, never the guarantee tag
(M-991; the emission is still unvalidated until a differential upgrades it).

---

## §7 Definition of Done (for M-1032, gated by this DN's ratification)

1. The expansion **mechanism** (Alt A/B/C/hybrid) is **ratified** by the maintainer (status stays Draft
   until then — the reasoner does not self-ratify, house rule #3/#4).
2. The pre-pass is implemented in **`crates/mycelium-transpile/**` only**, behind the ratified
   invocation surface (Rank 1: an opt-in flag; off by default; skip-graceful on a missing precondition).
3. **Before/after `expressible_fraction`** is measured on a macro-heavy target (M-1032 names `std-cmp`;
   §2 shows `std-runtime`/`std-time`/the `impl_narrow_int!` sites are richer) and recorded (`Empirical`).
   `checked_fraction` is reported too but **not** expected to move materially (§3) — the honest number,
   never inflated.
4. An **unexpandable macro** stays a **never-silent gap** (no fabricated expansion, VR-5/G2); the
   emitted `.myc` stays **`Declared`**.
5. Change-scoped `cargo fmt` / `clippy -D warnings` / `test -p mycelium-transpile` green; a data-driven
   test pins each expansion rule (or the `cargo expand` invocation's skip-graceful behavior).

---

## §8 Doc-Index + issues (FLAGGED up, not applied here)

`docs/Doc-Index.md`, `CHANGELOG.md`, and `tools/github/issues.yaml` are **integration-owned** (the
concurrent-PR pattern: leaves FLAG, the integrating parent applies once). **FLAG to the integrator:** add
a Design-Notes row for `DN-100 — Macro Expand-First Transpiler Pre-Pass (Draft)` to `Doc-Index.md`, a
`CHANGELOG.md` entry, and set M-1032's `doc_refs` to include `corpus:DN-100` (replacing the "needs a
toolchain Draft DN" note in its body with the DN reference on ratification).

---

## §9 Changelog

- **2026-07-10** — DN-100 created (**Draft**). The toolchain decision for M-1032 (ENB-9): profiled the
  whole-corpus macro population (82 item-position invocations across 18 files, ≈93% custom
  `macro_rules!`; 13 body-level macro poisons), framed the honest ROI (expressibility lever, uncertain
  `checked_fraction`), enumerated four mechanisms, and recommended (ranked, unratified) `cargo expand` as
  an opt-in advisory profiling mode plus a small std-macro shim. `Empirical` where measured against the
  tree (dev `8cd0a796`); `Declared` for the unbuilt mechanism. Authored READ + DN only — no edit to
  `issues.yaml`, `CHANGELOG.md`, `Doc-Index.md`, `lib/compiler/**`, or `crates/mycelium-l1/**`
  (integration/cloud-semcore-lane owned; FLAGGED up per §8). Append-only; status advances only by
  maintainer ratification (house rule #3).
