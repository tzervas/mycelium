# Design Note DN-66 — Stdlib Stable-API Freeze & Rust-Crate Retirement Status (M-719 broader closure)

| Field | Value |
|---|---|
| **Note** | DN-66 |
| **Status** | **Accepted** (2026-07-01 — maintainer-ratified via **ADR-035**, which cites this note's §2 freeze + §3 D6 assessment as its grounding evidence and narrows ADR-022 track T4's `lang 1.0.0` Definition of Done accordingly. Was **Draft** (2026-07-01, authored by the M-719 leaf, W1-719 session).) |
| **Decides** | *Nothing normatively.* Freezes the **current** stable public-API baseline for all 26 `mycelium-std-*` crates as a dated snapshot, and assesses — with grounded evidence — whether any crate has cleared the RFC-0031 §5 **D6** Rust-crate-retirement trigger. It does **not** rename, remove, or move any spec's status (append-only, house rule #3); it does not apply any `#[deprecated]` attribute (see §3 for why). |
| **Feeds** | M-719 (the issue this closes the "broader closure" clause of, in part); RFC-0031 §5 D5/D6; ADR-023 (stability/deprecation mechanism); DN-16 (the prior per-spec ratification-readiness survey, which this note updates the crate-inventory count for); DN-07 (ratification posture). |
| **Depends on** | RFC-0031 (Accepted, 2026-06-23) — D5 stability bar, D6 retirement mechanism; ADR-023 (Accepted, 2026-06-23) — the `#[deprecated(note = "…")]` mechanism and release-based removal policy; RFC-0016 (Enacted) — the per-crate spec corpus. |
| **Date** | 2026-07-01 |
| **Task** | M-719 (E13-1), leaf branch `claude/leaf/W1-719-stdlib-freeze` |

> **Posture (transparency rule / VR-5).** This note is **advisory and grounded** — every claim
> below cites the actual crate source, spec file, or RFC/ADR section checked as of this commit.
> "Frozen" here means *the surface documented below is the baseline a future change must not
> silently break*, not that any crate reaches SemVer 1.0.0 (all 26 crates stay `0.0.0` per
> ADR-018's pre-1.0 policy — freezing the *documented contract*, not the version number, is what
> M-719's Definition of Done actually asks for). No spec status changes here; no crate is marked
> `#[deprecated]` here — §3 grounds exactly why, with evidence, rather than asserting it.

---

## 1. Scope and method

Surveyed every `crates/mycelium-std-*` crate (26 total — `cmp`, `collections`, `content`, `core`,
`dense`, `diag`, `error`, `fmt`, `fs`, `io`, `iter`, `math`, `numerics`, `rand`, `recover`,
`runtime`, `select`, `spore`, `swap`, `sys`, `sys-host`, `ternary`, `testing`, `text`, `time`,
`vsa`) against: (a) its `docs/spec/stdlib/<name>.md` status field, (b) its guarantee-matrix
location in source, (c) its public API surface (`pub fn`/`pub struct`/`pub trait`/`pub enum`
count, via `grep` over `src/*.rs`), (d) whether a self-hosted `lib/std/<name>.myc` nodule exists
and, if so, whether its exported ops are the *same* API as the Rust crate's, and (e) whether any
other workspace crate depends on it (`grep` over every `Cargo.toml`).

## 2. §1 — The stable-API freeze (M-719 DoD: "a stable documented public API")

**Freeze declaration.** As of this commit, for every crate below whose spec is **Accepted**, the
public API is the surface named in that spec **plus** the crate's committed
`GUARANTEE_MATRIX`/`MATRIX` table (source file cited). This is now the **documented baseline** —
a future change to a frozen crate's public signature, guarantee tag, or exported-op set is a
**breaking change** and must go through: an amendment to the crate's `docs/spec/stdlib/*.md`,
a `CHANGELOG.md` entry, and — per ADR-023 §3.3 — a `#[deprecated(note = "…")]` cycle of at least
one `1.x` minor release before removal (once the project reaches 1.0.0; pre-1.0 the same
mechanism applies as the honest, non-silent path per G2, even though SemVer itself doesn't yet
force it). Nothing here upgrades any guarantee tag; the freeze is of *shape*, not *strength*.

| Crate | Spec status (`docs/spec/stdlib/<name>.md`) | Guarantee matrix | Pub API items | Self-hosted `.myc` port? |
|---|---|---|---|---|
| `mycelium-std-cmp` | Accepted (2026-06-20) | `src/lib.rs:GUARANTEE_MATRIX` | 16 | Partial, disjoint surface — §3.1 |
| `mycelium-std-collections` | Accepted (2026-06-20) | `src/lib.rs:GUARANTEE_MATRIX` | 41 | Partial, disjoint surface — §3.1 |
| `mycelium-std-content` | Accepted (2026-06-20) | `src/lib.rs:GUARANTEE_MATRIX` | 25 | none |
| `mycelium-std-core` | Accepted (2026-06-20) | `src/lib.rs:GUARANTEE_MATRIX` | 9 | none (Tier-0 `option.myc`/`result.myc` are separate nodules with no matching Rust crate) |
| `mycelium-std-dense` | Accepted (2026-06-20) | `src/lib.rs:GUARANTEE_MATRIX` | 25 | none |
| `mycelium-std-diag` | Accepted (2026-06-20) | `src/guarantee_matrix.rs` | 3 | none |
| `mycelium-std-error` | Accepted (2026-06-20) | `src/lib.rs:GUARANTEE_MATRIX` | 25 | none |
| `mycelium-std-fmt` | Accepted (2026-06-20) | `src/lib.rs:GUARANTEE_MATRIX` | 16 | Partial, **explicitly disjoint** surface — §3.1 |
| `mycelium-std-fs` | Accepted (2026-06-20) | `src/lib.rs:GUARANTEE_MATRIX` | 64 | none |
| `mycelium-std-io` | Accepted (2026-06-20) | `src/guarantee_matrix.rs` | 32 | none |
| `mycelium-std-iter` | Accepted (2026-06-20) | `src/lib.rs:GUARANTEE_MATRIX` | 47 | Partial, disjoint surface — §3.1 |
| `mycelium-std-math` | Accepted (2026-06-20) | `src/lib.rs:GUARANTEE_MATRIX` | 38 | Partial, **fully disjoint domain** — §3.1 |
| `mycelium-std-numerics` | Accepted (2026-06-20) | `src/matrix.rs:GUARANTEE_MATRIX` | 24 | none |
| `mycelium-std-rand` | Accepted (2026-06-20) | `src/lib.rs:GUARANTEE_MATRIX` | 25 | none |
| `mycelium-std-recover` | Accepted, Rust-first half (2026-06-20) | `src/guarantee_matrix.rs` | 37 | none (self-hosted half is M-502-gated, not started) |
| `mycelium-std-runtime` | Accepted, v0 R1 (2026-06-21) | `src/lib.rs` (`guarantee_matrix::MATRIX`) | 123 | none — **load-bearing**, see §4.c |
| `mycelium-std-select` | Accepted (2026-06-20) | `src/lib.rs:GUARANTEE_MATRIX` | 8 | none |
| `mycelium-std-spore` | Accepted, library/manifest half (2026-06-20) | `src/guarantee_matrix.rs` | 37 | none |
| `mycelium-std-swap` | Accepted (2026-06-20) | `src/lib.rs:GUARANTEE_MATRIX` | 13 | none |
| `mycelium-std-sys` | Accepted (2026-06-21) | `src/guarantee_matrix.rs` | 39 | none |
| `mycelium-std-sys-host` | **no spec file** — see §4.a | inline (`lib.rs` doc-comments; no matrix table) | 2 | none |
| `mycelium-std-ternary` | Accepted (2026-06-20) | `src/guarantee_matrix.rs` | 38 | none |
| `mycelium-std-testing` | Accepted (2026-06-20) | `src/guarantee_matrix.rs` | 45 | none |
| `mycelium-std-text` | Accepted (2026-06-20) | `src/lib.rs:GUARANTEE_MATRIX` | 38 | Partial, disjoint surface — §3.1 |
| `mycelium-std-time` | Accepted (2026-06-20) | `src/lib.rs:GUARANTEE_MATRIX` | 39 | none |
| `mycelium-std-vsa` | Accepted (2026-06-20) | `src/lib.rs:GUARANTEE_MATRIX` | 14 | none |

25/26 specs Accepted; `recover`/`spore` Accepted for their landed (Rust-first/library) half only;
`sys-host` is the sole exception — no spec file at all (§4.a). This table **supersedes DN-16's stale "25 specs" scope note**
(DN-16 §Changelog 2026-06-25 already widened it to 26; this note reproduces the current count with
fresh per-crate evidence rather than re-asserting DN-16's now-dated per-spec verdicts).

## 3. §2 — D6 retirement-trigger assessment (grounded, not asserted)

RFC-0031 §5 **D6**: *"Once a `.myc` port **clears D5**, the Rust crate's public API is marked
`#[deprecated(...)]`."* **D5** requires, **per exported op** of the module: a differential test,
an honest guarantee tag, and a frozen signature — and explicitly: *"a single uncovered op leaves
the module not self-hosted."* This is a **whole-module** bar applied op-by-op, not a
partial-credit one.

### 3.1 Finding: no crate's `.myc` port covers its own exported surface — the ports are disjoint prototypes, not replacements

Six crates have *some* `.myc` nodule bearing the same module name (`cmp`, `math`, `collections`,
`text`, `iter`, `fmt` — the RFC-0031 2026-06-27 changelog's "Tier-0/Tier-1 self-hosted surface" entry).
Checked directly against each Rust crate's actual exported items, **none is a superset or
equivalent of the Rust crate's surface** — each `.myc` nodule implements a narrower, structurally
different prototype:

- **`math` — fully disjoint domains.** `crates/mycelium-std-math/src/{exact,approx}.rs` exports
  `floor`/`ceil`/`trunc`/`round`/`abs`/`gcd`/`lcm`/`sqrt`/`sin`/`cos`/… over `f64`/`i64` (38 items).
  `lib/std/math.myc` exports `badd`/`bsub`/`band`/`bor`/`bxor`/`bnot` over `Binary{N}` and
  `tadd`/`tsub`/`tmul`/`tneg` over `Ternary{M}` — width-indexed bitwise/modular arithmetic on the
  kernel's fixed-width types. **Zero op-name or domain overlap.** The `.myc` nodule is not a port
  of the Rust crate at all; it is a different, lower-level arithmetic surface that happens to
  share the module name `math`.
- **`fmt` — explicitly disclaimed in the source.** `lib/std/fmt.myc`'s own header comment states:
  *"crates/mycelium-std-fmt exists but exposes a different Ring-2 surface (no hex_digit/to_hex) —
  it is not the structural oracle."* The corresponding differential test
  (`crates/mycelium-l1/tests/std_fmt.rs`) repeats the same disclaimer verbatim. This is the
  clearest first-party confirmation of the pattern this note documents.
- **`cmp` — different value model.** `mycelium-std-cmp` is trait-generic (`MycOrd`/`MycEq` over
  any `T`); `lib/std/cmp.myc`'s `cmp{N}`/`le{N}`/`ge{N}`/`max{N}`/`min{N}` are monomorphized over
  the concrete kernel type `Binary{N}` only. `myc_min`/`myc_max`/`myc_clamp` (Rust) and
  `max{N}`/`min{N}` (`.myc`) are conceptually parallel but not the same signature or dispatch
  mechanism; 11 of the Rust crate's 16 items (`Widen`/`Narrow`/`Bf16Bits`/`MatrixRow`/`clamp`/…)
  have no `.myc` counterpart at all.
- **`collections` — narrower, differently-keyed.** `mycelium-std-collections::Map::get(&K) ->
  Option<&V>` is a persistent, arbitrary-`K`/`V` structure (`map.rs`/`seq.rs`/`set.rs`, 37 items);
  `lib/std/collections.myc`'s `map_get{N}(m: Map[Binary{N}, V], k: Binary{N})` is keyed
  specifically by the kernel width type `Binary{N}`. Two ops (`map_get`, `set_contains`) exist
  in `.myc`; the other 39 Rust items (`Seq`, ordered `Vec`-backed sequence ops, `union`/
  `intersection`/`difference` on `Set`, …) do not. The M-719 issue body itself records that **6**
  constructor/builder ops in `collections.myc` currently **refuse** (RFC-0007 §11.3 inference
  gap) rather than pass a differential — so even the `.myc` nodule's *own* surface is not D5-clean.
- **`iter`/`text`** show the same pattern at smaller scale: `iter.myc`'s 6 first-order
  combinators (`map`/`filter`/`foldl`/`any`/`all`/`find`, plain `bool`/`Option` returns) vs.
  `mycelium-std-iter`'s 47 items including witness-carrying `any_with_witness`/`all_with_witness`,
  `zip`/`zip_exact`/`transduce`/`lazy_take`; `text.myc`'s UTF-8-decode helpers vs.
  `mycelium-std-text`'s 38-item `Bytes`/`Lossy<T>` surface.

**Conclusion (grounded, not asserted):** the D6 trigger — *"a `.myc` port clears D5"* — has **not
fired for any of the 26 crates**. The 6 crates with a same-named `.myc` nodule have narrower,
structurally distinct prototype surfaces (RFC-0031's own D4 table calls these "Tier 0/1
prototypes"; its 2026-06-27 changelog entry says outright: *"these remain **prototypes** under
the D5 stability bar... the full migration (D6 retirement / frozen stable API) remains open"*).
Applying `#[deprecated]` to any Rust item today, pointing at a `.myc` op as its replacement, would
be a **false or misleading claim** for at least `math`/`fmt` (disjoint domains) and an unverified
one for `cmp`/`collections`/`iter`/`text` (narrower, not equivalent) — a VR-5 violation this leaf
declines to introduce. **No crate is marked `#[deprecated]` by this note.**

## 4. What this note does and does not close

**Closed by this note:**
- The §2 freeze table — a single, dated, grounded cross-reference of spec status + guarantee-matrix
  location + public surface size for all 26 crates (previously scattered across DN-16 and 26
  separate spec files).
- The §3 D6 assessment — a grounded (not asserted) answer to "has any crate cleared retirement,"
  with per-crate evidence, closing the ambiguity in M-719's body about whether the Tier-0/1
  prototypes count.
- **Stability doc-comment annotations** added to all 26 crates' `src/lib.rs` (or, for
  `mycelium-std-sys-host`, its `README.md`, since its crate doc is
  `#![doc = include_str!("../README.md")]`) — additive, non-breaking rustdoc pointing back to this
  note and stating the crate's frozen-baseline status. No `#[deprecated]` attributes; no signature
  changes; `cargo build`/`clippy`/`test` unaffected (doc comments only).

**Explicitly deferred (FLAGged, not silently dropped — G2):**

- **(a) `mycelium-std-sys-host` has no `docs/spec/stdlib/sys-host.md`.** DN-16 already flagged this
  gap for `sys` (since closed); `sys-host` (the OS-wiring adapter crate, RFC-0028 §4.5) still has
  none. Writing it is a small, scoped follow-up — not done here (out of this leaf's declared
  deliverable, and a spec is a design artifact the maintainer should review, not something a leaf
  should originate unilaterally for a crate it doesn't own the design of).
- **(b) A real per-op audit, crate by crate, is the actual precondition for any `#[deprecated]`
  annotation.** §3.1's finding is that the *existing* 5 same-named `.myc` prototypes don't qualify
  — it is not a finding that no op ever will. As the D4-sequenced migration (RFC-0031 §5 D4) lands
  real ports of `cmp`/`math`/`collections`/`text`/`iter`'s *full* surfaces (and eventually the
  other 21 crates), each module needs a dedicated per-op comparison against the D5 checklist before
  any deprecation notice is honest. Recommend this as its own tracked task per module (or a single
  M-719-successor issue iterating the D4 order) rather than folding it into this freeze note.
- **(c) `mycelium-std-runtime` is load-bearing, not reference-only, and must not be retired.**
  `crates/mycelium-mlir/src/{runtime.rs,rc_plan.rs,tests/rc_plan_tests.rs}` depend on it directly
  (`Cargo.toml:mycelium-std-runtime = { path = "../mycelium-std-runtime" }`) for real RC-plan /
  refcounting runtime behavior — it is not merely a differential oracle like the other 25. Any
  future retirement conversation for this crate needs cross-crate coordination with
  `mycelium-mlir` (explicitly outside this leaf's scope — do not touch) and is an
  orchestrator/maintainer-level decision, not a leaf one.
- **(d) Final crate removal remains a post-Enactment maintainer decision (RFC-0031 D6 explicit).**
  Even once a crate's `.myc` port genuinely clears D5, D6 only sanctions `#[deprecated]` marking,
  not deletion — deletion waits for RFC-0031 to reach `Enacted` and an explicit maintainer act.
  Nothing here anticipates that.
- **(e) Shared-file registration** (`docs/Doc-Index.md` entry for DN-66, `tools/github/issues.yaml`
  M-719 body update, `CHANGELOG.md` entry) is **orchestrator-owned** per this leaf's scope
  boundary and is not touched by this branch — the integrating parent should add
  `corpus:DN-66` to M-719's `doc_refs` and record the closure/deferral split in the changelog.

## 5. Definition of Done (for this note's own scope)

- [x] Every `mycelium-std-*` crate's current spec status + guarantee-matrix location + public
      surface size is tabulated with a grounded source citation (§2).
- [x] The D6 retirement trigger is assessed per crate with concrete evidence, not asserted (§3).
- [x] A stability annotation is added to every crate pointing back to this note (§4, applied
      in-tree; see the leaf's report for the exact diff).
- [x] No `#[deprecated]` is applied where the evidence does not support it (VR-5) — the FLAG list
      (§4.a–e) states precisely what remains and why it is out of this leaf's safe scope.
- [x] Maintainer ratification of this note's status (`Draft` → `Accepted`) — **met 2026-07-01** via
      **ADR-035**, which cites this note's §2/§3 as its grounding evidence.
- [x] `docs/Doc-Index.md` / `issues.yaml` / `CHANGELOG.md` registration — **met 2026-07-01**, applied
      alongside ADR-035's ratification (this note's own §4.e deferral is closed by that act).

## 6. Grounding / honesty

- RFC-0031 (Accepted, 2026-06-23), §5 D5/D6 and its 2026-06-27 changelog entry (the "remain
  prototypes" admission this note independently re-verifies against source).
- ADR-023 (Accepted, 2026-06-23), §3.3 — the `#[deprecated(note = "…")]` mechanism this note
  declines to apply prematurely, and why.
- DN-16 (Resolved, per-spec ratification survey) — the prior per-crate status table this note
  refreshes with a current, narrower-purpose (freeze + D6) survey.
- Direct source citations throughout §3.1: `crates/mycelium-std-math/src/{exact,approx}.rs`,
  `lib/std/{math,fmt,cmp,collections,iter,text}.myc`, `crates/mycelium-l1/tests/std_fmt.rs`,
  `crates/mycelium-std-collections/src/{map,seq,set}.rs`, `crates/mycelium-std-iter/src/lib.rs`,
  `crates/mycelium-mlir/src/{runtime.rs,rc_plan.rs}` and its `Cargo.toml`.
- M-719 issue body (`tools/github/issues.yaml`) — the "STILL OPEN... this is a maintainer/
  orchestrator decision, not closed by this leaf" framing this note takes literally and honors.

## 6. Currency note — 2026-07-02 (post-M-883/M-884, kickoff `acy`)

*Append-only currency record (house rule #3 — §4.c's original text stands unchanged; this records a
changed factual basis, it does not rewrite the decision).*

§4.c grounded "`mycelium-std-runtime` is load-bearing, not reference-only, and must not be retired" in
`mycelium-mlir` depending on it **directly** (`crates/mycelium-mlir/src/{runtime.rs,rc_plan.rs,tests/rc_plan_tests.rs}`)
for RC-plan and refcounting runtime behavior. That basis is now **void**: kickoff `acy` (M-883/M-884)
extracted the exact surface mlir consumed — `reclamation` and `supervision` — into a new lower-stratum
crate **`mycelium-rt-abi`** (tier `core`), which `mycelium-mlir` now depends on instead.
`mycelium-std-runtime` **re-exports** those modules at their original paths (no API break), and the
`mycelium-mlir → mycelium-std-runtime` edge is **removed** (`cargo metadata` confirms; it was the
upward-tier anomaly the acyclic-deps gate — DN-68 — forbids).

Consequences for §4.c:

- The load-bearing reclamation and supervision surface now lives in `mycelium-rt-abi`.
- `mycelium-std-runtime` remains a real, non-oracle crate (`network`/`rc`/`region`/`scope_region`/`colony`
  modules; still consumed directly by `mycelium-bench`), so §4.c's *conclusion* ("not reference-only")
  still holds — but **no longer via `mycelium-mlir`**.
- A future `mycelium-std-runtime` retirement conversation **no longer requires `mycelium-mlir`
  coordination** for reclamation and supervision. It remains an RFC-0031 §5 D6 maintainer-level decision;
  this note neither triggers nor forecloses it.

Basis: `cargo xtask deps` (0 violations post-extraction); `docs/notes/DN-68-Acyclic-Deps-Invariant.md`; PRs #935/#936.

## Changelog

- **2026-07-02 — Currency note appended (acy integration-closeout, no status change).** §6 records
  that the M-883/M-884 `mycelium-rt-abi` extraction voided §4.c's "`mlir` depends on
  `mycelium-std-runtime` directly" basis; §4.c's original text is unchanged (append-only, house rule
  #3). See PRs #935/#936, DN-68.
- **2026-07-01 — Accepted (maintainer ratification via ADR-035).** This note's §2 freeze +
  §3 D6 assessment is adopted as the grounding evidence for **ADR-035**, which narrows ADR-022
  track T4's `lang 1.0.0` Definition of Done to the stable-API freeze + the core-lib self-host
  slice (M-714…M-718), deferring full D6 Rust-crate retirement to the post-1.0 long-term arc
  (ADR-022 §10). Status moves forward `Draft → Accepted` (house rule #3); the §2/§3 findings and
  the §4 FLAG list are **unchanged** by this transition — the ratification adopts them, it does
  not revise them. `docs/Doc-Index.md`/`issues.yaml`/`CHANGELOG.md` registration (§4.e) is closed
  by the same act. See ADR-035.
- **2026-07-01 — Draft (M-719 leaf, W1-719 session).** Initial freeze + D6 assessment. Tabulates
  all 26 `mycelium-std-*` crates' spec status, guarantee-matrix location, and public-surface size;
  grounds the finding that no crate's `.myc` port clears the D5/D6 whole-module bar (six crates
  have same-named but structurally disjoint `.myc` prototypes — `math` most starkly, `fmt`
  explicitly self-disclaimed in its own source comment); declines to apply `#[deprecated]` on that
  evidence; adds non-breaking stability doc-comments across all 26 crates; FLAGs the five items
  (§4.a–e) that remain for the maintainer/orchestrator. Advisory; no spec status changed.
