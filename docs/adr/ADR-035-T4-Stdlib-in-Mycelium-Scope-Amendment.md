# ADR-035 — Full-Language 1.0.0 Gate (Track T4) Scope Amendment: Stdlib-in-Mycelium Narrowed to the Stable-API Freeze + Core-Lib Self-Host Slice

| Field | Value |
|---|---|
| **ADR** | 035 |
| **Status** | **Accepted** (2026-07-01 — maintainer-ratified). Amends **ADR-022 track T4** (stdlib written in Mycelium): **narrows** T4's `lang 1.0.0` Definition of Done to the **documented stable-API freeze (DN-66)** plus the **core-lib self-host slice** that DN-14/E13-1 already largely cover (M-714…M-718 + M-719's freeze half) — **not** full `#[deprecated]` retirement of the `mycelium-std-*` reference crates (RFC-0031 §5 D6). Full stdlib self-hosting (per-op D5 `.myc` ports of all 26 crates + D6 retirement) is **deferred to the post-1.0 long-term arc** (ADR-022 §10) — the same mechanism ADR-022 §8 Q1 already used to narrow T9, and the same amending pattern **ADR-024** used for T1. This is a **scoped amendment**, not a wholesale supersession — ADR-022's other tracks, the dual-version model, and both prior amendments (ADR-024 T1, ADR-034 T6) all **remain in force**. `Accepted → Enacted` with ADR-022 T4 at the `lang 1.0.0` tag. |
| **Decides** | That the **full-language 1.0.0 gate (ADR-022 T4)** is met once (a) the stdlib's public API surface is frozen and documented (DN-66 §2 — a dated, grounded baseline across all 26 `mycelium-std-*` crates) and (b) the core-lib self-host slice lands differential-tested, honestly tagged, and never-silent (M-714…M-718, plus M-719's stable-API-freeze half) — **not** once every `mycelium-std-*` Rust crate clears the RFC-0031 §5 D6 retirement trigger. Full per-op D5 ports of all 26 crates, and the D6 `#[deprecated]` marking that follows, remain real, tracked work — but they are a **post-1.0** continuation of the zero-Rust long-term arc (ADR-022 §10), not a 1.0.0 blocker. |
| **Amends** | **ADR-022 §5 track T4** — narrows T4's Definition of Done to the two criteria above. The row's original "done when" text is **preserved, not rewritten** (house rule #3 — a criteria change is a supersede/amend act, never an in-place edit); ADR-022 carries only an append-only "T4 scope amended by ADR-035" pointer (§5 T4 row + §8 Q1 note + changelog). `M-719`'s D6-retirement half is re-scoped **post-1.0** (a new backlog issue, minted below, `status:todo`); its stable-API-freeze half satisfies the narrowed T4 bar and closes M-719 for 1.0.0 purposes. |
| **Grounds** | **DN-66** (Draft → **Accepted** by this ADR's ratification) §3 — the grounded, source-cited finding that **zero of 26** `mycelium-std-*` crates clear the RFC-0031 §5 D5/D6 bar today: the six same-named `.myc` nodules (`cmp`/`math`/`collections`/`text`/`iter`/`fmt`) are **structurally disjoint prototypes**, not ports (e.g. `math.myc`'s width-indexed bitwise ops share zero op-name or domain overlap with `mycelium-std-math`'s `f64`/`i64` surface; `fmt.myc`'s own header comment self-discloses it is "not the structural oracle"); DN-66 §4.c — `mycelium-std-runtime` is **load-bearing**, not reference-only (`crates/mycelium-mlir/src/{runtime.rs,rc_plan.rs}` depend on it directly via `Cargo.toml`), so any retirement conversation needs cross-crate coordination, not a unilateral leaf decision; RFC-0031 §5 D5/D6 (the per-op stability bar + retirement mechanism — **unchanged** by this ADR, only its 1.0.0-gating status moves); ADR-023 (the `#[deprecated]`/stability-freeze mechanism this ADR leans on for what "stable" means at 1.0.0); **ADR-024** + **ADR-034** (the precedent amending pattern: narrow a track's Definition of Done via a focused, scoped ADR rather than rewriting ADR-022's Accepted normative text in place); ADR-022 §8 Q1 (already narrows T9's full-toolchain self-host to the long-term arc — the direct precedent this ADR extends to T4's D6 half) and §10 (the long-term zero-Rust arc this ADR defers full retirement into); KC-3 (small auditable kernel — the trusted base is untouched by this decision either way), G2/VR-5 (never-silent, honestly-tagged; no crate is deprecated on unverified grounds). |
| **Date** | 2026-07-01 |

> **Posture (transparency rule / VR-5).** This ADR records *criteria*, maintainer-ratified; it asserts
> no release. It **narrows** T4's Definition of Done — it does not declare T4 met by fiat, nor move any
> spec to `Enacted`. The narrowed bar is met on a checked basis: DN-66 §2's freeze table + M-714…M-718's
> existing differential tests are the evidence (cited, not asserted). The full D6 retirement work is
> **not** cancelled or diminished — it remains a real, tracked, honestly-scoped continuation of the
> project's zero-Rust vision (ADR-022 §10); this ADR only changes **when** it must land relative to the
> `lang 1.0.0` tag. Nothing here upgrades a guarantee tag, and no `mycelium-std-*` crate is marked
> `#[deprecated]` by this act — DN-66 §3 already grounded why that would be premature.

---

## 1. Why this amendment exists

ADR-022 §5 track T4 requires: *"the stdlib + core libs written in `.myc` (RFC-0031), differential-tested,
stable APIs; Rust `std-*` beyond the bare core superseded by `.myc`."* Read literally and combined with
RFC-0031 §5 D6 ("once a `.myc` port clears D5, the Rust crate's public API is marked `#[deprecated]`"),
this makes T4's 1.0.0 bar **full retirement** of all 26 `mycelium-std-*` reference crates — every module's
`.myc` port must independently clear the per-op D5 stability bar (≥1 three-way differential per exported
op, honest tags, a frozen signature) before its Rust crate can be deprecated.

DN-66 (this session, M-719's "broader closure" survey) checked that literally, crate by crate, against
source, and found the D6 trigger has **not fired for any of the 26 crates**: the six crates with a
same-named `.myc` nodule (`cmp`, `math`, `collections`, `text`, `iter`, `fmt`) have narrower, structurally
different prototype surfaces — `math.myc`'s width-indexed bitwise/modular arithmetic over `Binary{N}`/
`Ternary{M}` shares **zero op-name or domain overlap** with `mycelium-std-math`'s `f64`/`i64` numeric
surface; `fmt.myc`'s own source comment states outright it "is not the structural oracle" for
`mycelium-std-fmt`. The other 20 crates have no `.myc` nodule at all. `mycelium-std-runtime` — 123 public
items, the largest crate — is additionally **load-bearing**: `crates/mycelium-mlir` depends on it directly
for RC-plan/refcounting behavior, so it cannot be retired without cross-crate coordination regardless of
D5 status.

Full D6 retirement at these 26 crates' current state is therefore a **large, multi-wave engineering
effort** (a per-op audit and a genuine `.myc` port for each of the 20 crates with no prototype at all, plus
completing the six disjoint prototypes into true supersets) — disproportionate to what the 1.0.0
correctness bar actually needs. The maintainer has decided (2026-07-01) to **narrow** T4's 1.0.0 Definition
of Done rather than hold the `lang 1.0.0` tag hostage to that full effort, mirroring the precedent already
set for T9 (ADR-022 §8 Q1 — the full toolchain self-host trails to the long-term arc; only the core-lib
slice gates 1.0.0) and for T1/T6 (ADR-024, ADR-034 — focused amending ADRs, not in-place rewrites, per
house rule #3's supersede-to-change-criteria discipline).

## 2. The amendment — T4's Definition of Done, narrowed

ADR-022 track T4 (`lang 1.0.0`) previously read: *"the stdlib + core libs written in `.myc` (RFC-0031),
differential-tested, stable APIs; Rust `std-*` beyond the bare core superseded by `.myc`."* **This ADR
narrows that bar to two criteria, both already substantially satisfied:**

> **T4 (narrowed by ADR-035).** Before the `lang 1.0.0` tag, track T4 requires: **(a)** a documented,
> stable public-API baseline for the stdlib — the DN-66 stable-API freeze, covering all 26
> `mycelium-std-*` crates' public surface + guarantee-matrix location, dated and grounded (not asserted);
> and **(b)** the core-lib self-host slice — the structural/polymorphic core (`std.core`/`std.iter`/
> `std.cmp` etc., M-714…M-718) landed `.myc`, differential-tested against the Rust reference, with honest
> per-op guarantee tags and never-silent boundaries (G2). **Full RFC-0031 §5 D6 retirement** — a per-op
> D5 audit and genuine `.myc` port of every one of the 26 `mycelium-std-*` crates' full exported surface,
> followed by `#[deprecated]` marking — is **deferred to the post-1.0 long-term arc** (ADR-022 §10). The
> D5/D6 mechanism itself is **unchanged**; only its 1.0.0-gating status moves.

The T4 "done when" row text in ADR-022 §5 is **preserved, not rewritten** (append-only pointer added
instead — see §3 below).

## 3. Consequence

- The `lang 1.0.0` tag's T4 row is now satisfiable **without** a full stdlib rewrite: DN-66's freeze (§2)
  plus M-714…M-718 (already `done`) plus M-719's freeze half (already landed) **meet the narrowed bar**.
  T4 moves from "⏳ open" to met-on-the-narrowed-criteria (§5 below).
- **M-719** (E13-1's conformance/stability closure task) is re-scoped: its stable-API-freeze half is now
  T4's whole 1.0.0 requirement and is `done`; its D6-retirement half becomes a **separate, post-1.0
  backlog item** (a new issue, minted alongside this ADR, `status:todo`, explicitly out of the `lang
  1.0.0` critical path).
- **E13-1** (the epic T4 names) is satisfiable under the narrowed bar: all named children (M-714…M-719)
  are `done` for their (now-narrowed) 1.0.0 scope. The epic's full-retirement ambition continues
  post-1.0 as the backlog item above.
- **E18-1** (T9's self-hosting capstone) shares the "core-lib slice" language with T4 (ADR-022 §8 Q1);
  this ADR clarifies that the shared slice's bar is the same M-714…M-718 differential-tested core, now
  satisfied — E18-1's *own* remaining children (M-739…M-742, the compiler/toolchain self-host) are
  **unaffected** and remain open; they are the long-term-arc capstone work ADR-022 §8 Q1 already scoped
  past 1.0.0.
- ADR-022 §5 T4 row + §8 Q1 carry append-only "narrowed by ADR-035" pointers; the row's/Q1's original
  text is **not** rewritten (the supersede-to-change-criteria rule is honored, per ADR-024/ADR-034's
  precedent).
- The trusted base is **unaffected** either way — this amendment changes 1.0.0-gating scope, not any
  kernel surface, `Repr`, or prim (KC-3 untouched).
- RFC-0031's D5/D6 mechanism is **unchanged**; only the changelog records that its D6 half is no longer
  on the `lang 1.0.0` critical path.

## 4. Rationale & alternative considered

**Chosen:** narrow T4 to the stable-API freeze + core-lib self-host slice — the same reasonable-not-
maximal calculus ADR-022 §8 Q1 already applied to T9 ("properly usable without hand-written L0/L1," not
"zero Rust anywhere"). The stable-API freeze (DN-66, leaning on ADR-023's stability/deprecation mechanism)
already delivers the guarantee 1.0.0 users actually need — a documented, frozen contract they can build
against — without requiring every crate's implementation language to have changed. This keeps the
project's honest zero-Rust vision (ADR-022 §10) fully intact as a **named, tracked, post-1.0** continuation
rather than either abandoning it or holding the tag hostage to it.

**Alternative (not taken):** hold T4 to the literal full-D6-retirement bar and let it continue blocking
`lang 1.0.0` until every one of the 26 crates has a genuine `.myc` port. This is the more maximal reading
of ADR-022's original T4 text, and it *would* deliver a fully self-hosted stdlib at the tag — but DN-66's
grounded survey shows this is realistically a large multi-wave effort (20 crates with no `.myc` nodule at
all, plus completing 6 disjoint prototypes into true supersets, plus `mycelium-std-runtime`'s cross-crate
retirement coordination) with no correctness benefit over the frozen-API alternative: the stability
guarantee 1.0.0 actually promises (ADR-023) is already met by the freeze. The maintainer weighed the
proportionality of that effort against the 1.0.0 correctness bar and chose the narrower criterion,
consistent with how T1 (ADR-024) and T6 (ADR-034) were each amended by scoped ADRs rather than by
stretching or shrinking ADR-022's original text in place.

## 5. Definition of Done

- [x] The DN-66 stable-API freeze is complete and grounded: all 26 `mycelium-std-*` crates' public API +
  guarantee-matrix location tabulated with source citations (DN-66 §2); no crate is `#[deprecated]` on
  unverified grounds (DN-66 §3, VR-5). **Met** — DN-66 landed 2026-07-01 (W1-719 leaf), this ADR ratifies
  its status `Draft → Accepted`.
- [x] The core-lib self-host slice (M-714…M-718) lands `.myc`, differential-tested against the Rust
  reference, honest tags, never-silent (G2). **Met** — all four `done` (RFC-0031 landed 2026-06-23;
  M-715…M-718 landed progressively; conformance widened by the 2026-06-27/2026-06-30 M-719 sessions).
- [x] `M-719` re-scoped: stable-API-freeze half closes T4's 1.0.0 requirement (`status: done` on the
  narrowed bar); the D6-retirement half is spun out to a new post-1.0 backlog issue (`status:todo`,
  minted alongside this ADR).
- [x] ADR-022 carries the append-only "T4 scope amended by ADR-035" pointer (§5 T4 row + §8 Q1 note +
  changelog); its T4 §5 "done when" text and §8 Q1 resolution text are otherwise unchanged.
- [x] RFC-0031 carries an append-only note: the D6 retirement work is no longer on the `lang 1.0.0`
  critical path (deferred post-1.0 per this ADR); §5 D5/D6's normative text is otherwise unchanged.
- [x] E13-1 and E18-1 issue bodies carry an append-only note recording this amendment's effect on their
  respective Definitions of Done.
- [x] This ADR reaches **Accepted** (maintainer-ratified) and is indexed (`docs/Doc-Index.md`,
  `docs/adr/README.md`).
- **Enacted** with ADR-022 T4 at the `lang 1.0.0` tag (append-only; M-738) — not by this ADR itself.

## 6. Grounding / honesty

- DN-66 §2/§3/§4.c — the grounded freeze table + the per-crate D6-trigger assessment + the
  `mycelium-std-runtime` load-bearing finding, all cited to source rather than asserted.
- RFC-0031 §5 D5/D6 — the stability bar + retirement mechanism this ADR defers the *gating*, not the
  *mechanism*, of.
- ADR-023 — the stability/API-compatibility + `#[deprecated]` mechanism the narrowed T4 bar leans on.
- ADR-022 §5 (track T4, amended by this ADR) + §8 Q1 (the T9 precedent this ADR extends the same
  reasonable-not-maximal logic to) + §10 (the long-term zero-Rust arc full D6 retirement continues in).
- ADR-024, ADR-034 — the precedent amending pattern (focused, scoped ADR; append-only pointers; no
  in-place rewrite of ADR-022's Accepted text).
- KC-3, G2, VR-5 — the trusted base is unaffected; no crate is deprecated without a checked D5 basis; no
  status is upgraded past what DN-66's evidence supports.

---

### Changelog

| Date | Status | Note |
|---|---|---|
| 2026-07-01 | **Accepted** | Maintainer-ratified scoped amendment of ADR-022 track T4: narrows T4's `lang 1.0.0` Definition of Done to the **documented stable-API freeze (DN-66)** + the **core-lib self-host slice** (M-714…M-718 + M-719's freeze half) — full RFC-0031 §5 D6 Rust-crate retirement for all 26 `mycelium-std-*` crates is **deferred to the post-1.0 long-term arc** (ADR-022 §10), mirroring how ADR-022 §8 Q1 already narrowed T9 and how ADR-024 narrowed T1. Grounded in DN-66's per-crate finding that zero crates clear the D6 trigger today (six same-named `.myc` nodules are structurally disjoint prototypes, not ports; `mycelium-std-runtime` is load-bearing). DN-66 itself moves `Draft → Accepted` by this ratifying act. `M-719` re-scoped (stable-API-freeze half closes for 1.0.0; D6-retirement half spun out to a new post-1.0 backlog issue, `status:todo`). ADR-022 §5 T4 row + §8 Q1 carry append-only "narrowed by ADR-035" pointers (their normative text is not rewritten); RFC-0031 §5 D6 carries an append-only note that its retirement work is off the `lang 1.0.0` critical path (the D5/D6 mechanism itself is unchanged). E13-1 and E18-1 issue bodies note the effect on their own Definitions of Done. |
