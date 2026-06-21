# Design Note DN-16 — Stdlib Ratification Readiness Survey (Wave-5)

| Field | Value |
|---|---|
| **Note** | DN-16 |
| **Status** | **Resolved** (2026-06-19; M-374 survey + M-377 honesty cleanups — the actionable cross-cutting divergences are closed; per-spec *ratification* to Accepted remains the maintainer's append-only call, DN-07) |
| **Decides** | *Nothing normatively.* Per-spec readiness survey for the maintainer's ratification pass. Specifying "ratification-ready" here means the code matches the spec's stated contract; it does **not** move any spec to Accepted — that is the maintainer's append-only call. (DN-07 posture.) |
| **Feeds** | RFC-0016 (Core Library RFC — Accepted); each `docs/spec/stdlib/*.md` spec; `crates/mycelium-std-*/`; DN-07 (RFC-0016 ratification note); DN-14 (self-hosting gate). |
| **Date** | 2026-06-19 |
| **Task** | M-374 (Wave-5 stdlib ratification survey) |

> **Posture (honesty rule / VR-5).** This note is *advisory*. Verdicts are grounded in the actual
> spec files and crate source (file:section cited per finding). "ratification-ready" means the
> surveyor verified the specific items listed — it does **not** mean all possible divergences are
> checked. Items marked "unverified" were not independently checked beyond the spec or crate
> header. No spec is moved to Accepted here; these are inputs to the maintainer's ratification
> call. Append-only.

---

## 1. Survey Scope and Method

This note surveys every `docs/spec/stdlib/*.md` file (excluding `_TEMPLATE.md` and `README.md`)
against its landed crate in `crates/mycelium-std-*/`. For each spec, the surveyor:

1. Read the spec's status field, open-questions section (§7), and guarantee-matrix (§4).
2. Read the corresponding crate's `lib.rs` (and supporting files where present) for public API,
   guarantee matrix, and any FLAG comments.
3. Classified the spec against three criteria: **(a)** does a crate exist at all; **(b)** does the
   crate export a `GUARANTEE_MATRIX` asserted in tests; **(c)** are spec §7 open questions still
   open (i.e., blocking ratification per the spec's own phrasing) or resolved.

**Evidence basis:** spec files at `docs/spec/stdlib/<name>.md`; crate sources at
`crates/mycelium-std-<name>/src/`. All spec statuses read as of 2026-06-19.

---

## 2. Summary Table

| Spec | Crate | Spec Status | Guarantee Matrix in code? | Open Qs (unresolved) | Verdict | Key note |
|---|---|---|---|---|---|---|
| `cmp` | `mycelium-std-cmp` | Implemented, pending ratification | yes (`lib.rs`) | 3 (Q1 float total-order; Q2 f64→i32 seam; Q3 cross-Repr eq) | **ratification-ready with flags** | Trait names `MycEq`/`MycOrd`/`MycPartialOrd` diverge from spec sketch (`Eq`/`Ord`/`PartialOrd`) — acknowledged naming-gap (RFC-0016 §8-Q2, unresolved); GM present; all rows `Exact`; Q1–Q3 are design questions, not correctness blockers |
| `collections` | `mycelium-std-collections` | Implemented, pending ratification | yes (`guarantee_matrix.rs`) | 5 (Q1 structure set; Q2 order-tag; Q3 content-hash seam; Q4 seeded-hash effect; Q5 ergonomics) | **ratification-ready with flags** | GM present; Q1–Q5 are scope/design questions, not structural contract violations; Q2 `Declared`→`Empirical` upgrade depends on landed property test |
| `content` | `mycelium-std-content` | Implemented, pending ratification | yes (`guarantee_matrix.rs`) | 4 (Q1 digest API not fixed; Q2 `hash_of_value` scope; Q3 HMAC-auth seam; Q4 `phylum`-level seam) | **ratification-ready with flags** | Q1–Q4 are design calls, not contract violations; GM present |
| `core` | `mycelium-std-core` | Implemented, pending ratification | yes (`lib.rs`) | 2 (Q1 prelude membership; Q3 re-export completeness — Q2 noted resolved in spec) | **ratification-ready with flags** | 9-row GM present; re-export surface verified; Q1/Q3 are scoping calls |
| `dense` | `mycelium-std-dense` | Implemented, pending ratification | yes (`lib.rs`) | 5 (Q1 Higham bound + checker instantiation; Q2 batch norm; Q3 out-of-range domain; Q4 `wild`/FFI floor; Q5 `Approx<T>` carrier — noted resolved in numerics) | **divergent — provisional `Proven` tags** | Q1 is load-bearing: `Proven` rows for float `sum`/`dot` are **provisional** pending M-512 discharging the ADR-010 checked side-conditions (spec §7-Q1 explicit warning); surveyor could not independently verify the theorem instantiation (unverified); GM present |
| `diag` | `mycelium-std-diag` | Implemented, pending ratification | yes (`guarantee_matrix.rs`) | 1 unresolved (Q5 typed tags) + 4 noted with resolution context | **ratification-ready with flags** | Q1 (`diag`↔`fmt` boundary) has resolution context in changelog; Q2–Q4 deferred to `runtime`/M-502/surface-layer; Q5 is post-v0; GM present; all rows `Exact` per I1 structural proof |
| `error` | `mycelium-std-error` | Implemented, pending ratification | yes (`guarantee_matrix.rs`) | 3 unresolved (Q1 `recover`-bridge sig; Q2 error-value naming; Q4 self-hosting bar) + Q3 noted resolved | **ratification-ready with flags** | Q1 bridge signature is FLAGGED cross-module with `recover`; Q2 naming is DN-level (RFC-0016 §8-Q2); Q4 is M-502-gated; GM present |
| `fmt` | `mycelium-std-fmt` | Implemented, pending ratification | yes (`lib.rs`) | 2 unresolved (Q2 human round-trip scope; Q3 truncation marker) + Q1 resolved | **divergent — guarantee-tag framing mismatch** | `std.fmt` tags `from_json` `Exact` (lib.rs `GUARANTEE_MATRIX`, op `"from_json"`, tag `GuaranteeStrength::Exact`); `std.io` tags the same op `Empirical` (io `guarantee_matrix.rs`, op `"from_json"`, tag `GuaranteeTag::Empirical`). Both are noted as honest from their perspective (spec §7-Q1 resolution note), but the divergence is live in the code and unreconciled. Maintainer call required (VR-5). |
| `fs` | `mycelium-std-fs` | Implemented, pending ratification | yes (`guarantee_matrix.rs`) | 5 (Q1 `wild`/std-sys split; Q2 `io` seam; Q3 Path model; Q4 atomicity/symlinks; Q5 capability scoping) | **ratification-ready with flags** | All 5 Qs are design/scoping calls, not structural contract failures; GM present; C5 "no new `wild`" qualified in spec (inventoried in C5 section §5) |
| `io` | `mycelium-std-io` | Implemented, pending ratification | yes (`guarantee_matrix.rs`) | 2 unresolved (Q2 `Source`/`Sink` naming; Q3 budget-exceeded `Empirical` floor) + Q1/Q4/Q5 resolved | **ratification-ready with flags** | Q2 naming is §8-Q2; Q3 `BudgetExceeded` tag is a VR-5 design call; GM present |
| `iter` | `mycelium-std-iter` | Implemented, pending ratification | yes (`guarantee_matrix.rs`) | 4 unresolved (Q1 zip length-mismatch; Q2 `any`/`all` witness ergonomics; Q4 lazy surface scope; Q5 migration bar) + Q3 noted resolved | **ratification-ready with flags** | Q1–Q2/Q4–Q5 are design/scoping calls; GM present |
| `math` | `mycelium-std-math` | Implemented, pending ratification | yes (`lib.rs`) | 1 unresolved (Q1 `Approx<T>` carrier shape deferred — resolved in `numerics` but cross-module reconcile needed) + Q2–Q5 noted resolved | **ratification-ready with flags** | Q1 is residual cross-module reconcile with M-512/`numerics`; GM present; `Proven` rows for `sum`/`dot` depend on same ADR-010 checker instantiation as `dense` (shared dependency — not independently verified here) |
| `numerics` | `mycelium-std-numerics` | Implemented, pending ratification | yes (`matrix.rs`) | 3 unresolved (Q2 `ProvenThm` witness scope; Q4 call-site ergonomics; Q5 migration bar) + Q1/Q3 resolved | **ratification-ready with flags** | GM in separate `matrix.rs`; Q1 `Approx<T>` carrier resolved; Q2 scope of `ProvenThm` sealed witness is a design call; GM present |
| `rand` | `mycelium-std-rand` | Implemented, pending ratification | yes (`lib.rs`) | 3 unresolved (Q1 distribution scope; Q2 `wild`/FFI floor; Q5 seeded-read effect framing) + Q3/Q4 noted resolved | **ratification-ready with flags** | Q1 is M-501-gated scope; Q2 is §8-Q6 (`std-sys` split); GM present |
| `recover` | `mycelium-std-recover` | Implemented (Rust-first half), pending ratification | yes (`guarantee_matrix.rs`) | 0 unresolved (all 5 Qs have resolution context) | **ratification-ready** | All §7 questions noted resolved or deferred with direction in changelog; self-hosted half is M-502-gated (noted in status); GM present; I2/VR-5 honest-tag fix in Wave-4 changelog |
| `runtime` | *no crate* | **Draft (needs-design)** | no | 5 (all open: Q1 phylum placement; Q2 `wild` transport floor; Q3 structured-concurrency scope; Q4 `reclaim` cascade seam; Q5 migration bar) | **not-yet-implemented — no ratification possible** | RFC-0008 cross-phase-gated (Phase-7); binding set is RESERVED-not-active; no `mycelium-std-runtime` crate exists; spec is draft |
| `select` | `mycelium-std-select` | Implemented, pending ratification | yes (`lib.rs`) | 4 (Q1 `Explanation` record schema; Q2 override-priority ergonomics; Q3 `CostModel` scope; Q4 migration bar) | **ratification-ready with flags** | GM present; spec explicitly notes `Explanation` field names are owned by landed crate (not invented in spec — the right posture); Q1–Q4 are design/scoping calls |
| `self-hosting-readiness` | *no crate (assessment doc)* | **Draft (needs-design) — verdict: not yet established** | n/a | 2 (Q-a and Q-b: surface-language gates) | **not-yet-implemented — no ratification possible** | This is an assessment spec, not a code module; DN-14 (same wave) records the formal readiness verdict ("not yet established"); the assessment itself is up-to-date |
| `spore` | `mycelium-std-spore` | Implemented (library/manifest half), pending ratification | yes (`guarantee_matrix.rs`) | 2 unresolved (Q2 Phase-6 native-deploy half M-620; Q4 `vsa`↔`spore` reconstruction placement) + Q1/Q3/Q5 noted resolved | **ratification-ready with flags (scoped)** | Ratification can be scoped to library/manifest half; native-deploy (M-620) is Phase-6-gated and explicitly not in this wave; Q4 cross-module seam with `vsa` needs maintainer call; GM present |
| `swap` | `mycelium-std-swap` | Implemented, pending ratification | yes (`lib.rs`) | 4 unresolved (Q1 naming; Q2 call-site ergonomics; Q4 landed-crate surface — abstract only; Q5 migration bar) + Q3 noted resolved | **divergent — spec surface is abstract only** | Spec §3 describes the exported-op surface abstractly (Q4 explicit: "pin §3 to the real surfaces before this spec is ratified"); the crate exports `bin_to_tern`/`tern_to_bin`/`f32_to_bf16`/`dense_to_vsa`/`vsa_to_dense`/`check_swap`/`explain` at `lib.rs:pub fn …` lines — the §3 sketch does not name these exactly; a surface-reconcile pass is needed before ratification. GM present. |
| `ternary` | `mycelium-std-ternary` | Implemented, pending ratification | yes (`guarantee_matrix.rs`) | 1 unresolved (Q1 surface naming / fungal lexicon) + Q2–Q4 noted resolved | **ratification-ready with flags** | Q1 naming is §8-Q2; GM present; all GM invariants tested |
| `testing` | `mycelium-std-testing` | Implemented, pending ratification | yes (`guarantee_matrix.rs`) | 4 unresolved (Q1 scope; Q2 `FailRecord`↔`Diag` seam; Q4 `Proven`-witness mode; Q5 migration bar) + Q3 noted resolved | **ratification-ready with flags** | Q2 is a known fast-follow (diag changelog records it); Q1/Q4/Q5 are scope/design; GM present |
| `text` | `mycelium-std-text` | Implemented, pending ratification | yes (`guarantee_matrix.rs`) | 4 unresolved (Q1 `Lossy<T>` shape; Q2 grapheme-table versioning; Q3 parse↔numerics seam; Q4 UTF-8 `wild`/FFI floor) | **ratification-ready with flags** | Q1–Q4 are design/scoping calls; GM present; all rows `Exact`; Q3 cross-module seam with `cmp`/`math` needs maintainer sign-off |
| `time` | `mycelium-std-time` | Implemented, pending ratification | yes (`lib.rs`) | 5 unresolved (Q1 logical-clock API owned by M-521; Q2 timers deferred; Q3 `wild` floor for wall/mono; Q4 `Duration` representation; Q5 effect-surface syntax) | **ratification-ready with flags** | All 5 Qs deferred or design calls; Q1 ownership is explicit (not invented); GM present |
| `vsa` | `mycelium-std-vsa` | Implemented, pending ratification | yes (`lib.rs`, also `matrix.rs`) | 5 unresolved (Q1 scope; Q2 `similarity` primitive vs decision tag; Q3 `Proven` cell exact instantiation; Q4 resonator `Declared` coverage; Q5 ergonomics) | **ratification-ready with flags** | Q3 is the same ADR-010 / encoded-matrix dependency as `dense`/`math` — not independently verified here; GM present (both `lib.rs` const and `matrix.rs` per-model rows) |

---

## 3. Per-Spec Details

### 3.1 `cmp` (`mycelium-std-cmp`)

**Verdict: ratification-ready with flags**

- Spec: `docs/spec/stdlib/cmp.md` §3, §4, §7. Crate: `crates/mycelium-std-cmp/src/lib.rs`.
- Guarantee matrix present at `lib.rs:GUARANTEE_MATRIX` (9 rows, all `Exact`; spec §4 says "nine
  rows" — `lib.rs:assert_eq!(GUARANTEE_MATRIX.len(), 9)` checks this).
- Contract conformance C1–C6: verified in `lib.rs` doc-comments and test assertions (clamp
  inverted-bounds `Err`, narrow `OutOfRange`/`NotRepresentable`, widen total).
- **Naming divergence (FLAG):** Spec §3 sketch uses `Eq`/`Ord`/`PartialOrd`; crate exports
  `MycEq`/`MycOrd`/`MycPartialOrd` (`lib.rs:pub trait MycEq`, `MycOrd`, `MycPartialOrd`). The `Myc`
  prefix is a Rust naming-collision avoidance; it is not noted in the spec. This is the RFC-0016
  §8-Q2 naming question applied — FLAGGED for the maintainer to explicitly accept or rework.
- §7 open questions Q1 (float total order), Q2 (f64→i32 rounding seam), Q3 (cross-Repr `eq`):
  all unresolved; none are structural contract violations, all are scope/design calls.

### 3.2 `collections` (`mycelium-std-collections`)

**Verdict: ratification-ready with flags**

- Spec: `docs/spec/stdlib/collections.md` §3–§7. Crate: `crates/mycelium-std-collections/src/`.
- GM at `guarantee_matrix.rs`; `Seq`/`Map`/`Set` exported.
- Q1 (concrete structure set, ordered vs bucketed) is M-501-to-ratify; Q2 (`Declared`→`Empirical`
  upgrade) depends on landed property test; Q3 (`content` hash seam) defers to `content` §7-Q2;
  Q4 (seeded hash effect) is a declared-effect design call; Q5 (ergonomics) is §8-Q3.
- None of Q1–Q5 are structural contract failures.

### 3.3 `content` (`mycelium-std-content`)

**Verdict: ratification-ready with flags**

- Spec: `docs/spec/stdlib/content.md` §3–§7. Crate: `crates/mycelium-std-content/src/`.
- GM at `guarantee_matrix.rs`; `ContentRef`, `NameRegistry`, `content_ref.rs`, `name_registry.rs`
  present.
- Q1 (digest API abstraction) is the spec's explicit non-commitment posture; Q2 (`hash_of_value`
  scope) deferred to `std.content` itself; Q3 (HMAC-auth seam) is a Ring-boundary design call;
  Q4 (phylum-level seam) is §8-Q2.

### 3.4 `core` (`mycelium-std-core`)

**Verdict: ratification-ready with flags**

- Spec: `docs/spec/stdlib/core.md` §3–§7. Crate: `crates/mycelium-std-core/src/lib.rs`.
- 9-row GM at `lib.rs:GUARANTEE_MATRIX` (`assert_eq!(GUARANTEE_MATRIX.len(), 9)`); re-exports
  `mycelium_core` value model (`Value`, `Repr`, `Meta`, `GuaranteeStrength`, `CoreValue`, `Datum`).
- Q1 (prelude membership) is a scope call; Q2 noted resolved in spec (error-value naming); Q3
  (re-export completeness) is a maintainer audit item.

### 3.5 `dense` (`mycelium-std-dense`)

**Verdict: divergent — provisional `Proven` tags**

- Spec: `docs/spec/stdlib/dense.md` §4, §7-Q1. Crate: `crates/mycelium-std-dense/src/lib.rs`.
- GM at `lib.rs:GUARANTEE_MATRIX`.
- **Load-bearing divergence (FLAG — spec §7-Q1 explicit):** `dense` tags float `sum`/`dot`/
  `add`-family rows `Proven` *on the premise* that the ADR-010 numerics checker discharges the
  standard backward-error bound (Higham). The spec itself says: "Until M-512 fixes the checked
  instantiation, treat the `Proven` tag as **provisional**; any op the checker cannot discharge
  **downgrades to `Empirical`** (M-I3)." The surveyor could not independently verify whether
  M-512 has discharged the checked side-conditions (unverified — VR-5 applies; the spec's own
  warning stands). **Recommendation:** before ratifying `dense`, confirm with the `numerics`
  (M-512) maintainer that the checked bound is instantiated; downgrade to `Empirical` if not.
- Q2–Q5 are design calls.

### 3.6 `diag` (`mycelium-std-diag`)

**Verdict: ratification-ready with flags**

- Spec: `docs/spec/stdlib/diag.md` §4, §7. Crate: `crates/mycelium-std-diag/src/`.
- GM at `guarantee_matrix.rs` (14 rows, all `Exact`); `present` returns the diagnostic unchanged
  (I1 structural proof noted in status and changelog).
- Q1 (`diag`↔`fmt` rendering boundary) has resolution context in changelog (2026-06-18 entry
  mentions delegation direction); Q2 (trace/span carrier) deferred to surface-layer; Q3 (sink
  transport `wild`) deferred to `runtime`; Q4 (self-hosting bar) is M-502-gated; Q5 (typed tags)
  is explicitly post-v0.
- A fast-follow is noted in the changelog: reconcile `std.testing`'s `FailRecord` to delegate to
  `Diag` (§7-Q2 / testing Q2 seam) — this is an integration item, not a ratification blocker for
  `diag` itself.

### 3.7 `error` (`mycelium-std-error`)

**Verdict: ratification-ready with flags**

- Spec: `docs/spec/stdlib/error.md` §3–§7. Crate: `crates/mycelium-std-error/src/`.
- GM at `guarantee_matrix.rs`; `combinators.rs` present.
- Q1 (`recover`-bridge sig) is cross-module with M-520 — FLAGGED; Q2 (error-value naming) is
  §8-Q2 naming; Q3 noted resolved; Q4 (self-hosting bar) is M-502-gated.

### 3.8 `fmt` (`mycelium-std-fmt`)

**Verdict: divergent — guarantee-tag framing mismatch**

- Spec: `docs/spec/stdlib/fmt.md` §4, §7-Q1. Crate: `crates/mycelium-std-fmt/src/lib.rs`;
  `crates/mycelium-std-io/src/guarantee_matrix.rs`.
- **Live divergence (FLAG — VR-5):** `std.fmt` `GUARANTEE_MATRIX` (`lib.rs`, row `"from_json"`)
  records `tag: GuaranteeStrength::Exact`. `std.io` `guarantee_matrix.rs` (row `"from_json"`)
  records `GuaranteeTag::Empirical`. The `fmt` spec §7-Q1 resolution note explicitly acknowledges
  this ("both honest from their angle; unifying the framing is a finer reconciliation deferred to
  the maintainer"). The divergence is **in the committed code** and is honesty-relevant (VR-5 —
  the maintainer must resolve which tag framing is correct, or formally accept both as scoped views
  of the same op). This is not a structural contract failure (neither over-claims `Proven`), but it
  is a VR-5 residual that should be explicitly closed before ratification.
- Q2 (human round-trip scope) and Q3 (truncation marker form) remain open.

### 3.9 `fs` (`mycelium-std-fs`)

**Verdict: ratification-ready with flags**

- Spec: `docs/spec/stdlib/fs.md` §4–§7. Crate: `crates/mycelium-std-fs/src/`.
- GM at `guarantee_matrix.rs`; `path.rs`, `substrate.rs`, `metadata.rs`, `options.rs`, `error.rs`
  present.
- Q1–Q5 are design/scoping calls (the `std-sys` split, `io` seam, path model, atomicity, capability
  scoping). C5 "no `wild`" is explicitly narrowed in the spec's §5 section — the `wild` blocks are
  inventoried. Not structural violations.

### 3.10 `io` (`mycelium-std-io`)

**Verdict: ratification-ready with flags**

- Spec: `docs/spec/stdlib/io.md` §3–§7. Crate: `crates/mycelium-std-io/src/`.
- GM at `guarantee_matrix.rs`; `io.rs`, `serialize.rs` present.
- Q1 resolved (delegation wired); Q2 naming is §8-Q2; Q3 `BudgetExceeded` tag is a VR-5 design
  call (not over-claimed); Q4/Q5 noted resolved.
- `from_json` tagged `Empirical` in GM (`guarantee_matrix.rs:GuaranteeTag::Empirical`) — consistent
  with proptest basis (no theorem). This is the honest side of the `fmt`/`io` framing divergence.

### 3.11 `iter` (`mycelium-std-iter`)

**Verdict: ratification-ready with flags**

- Spec: `docs/spec/stdlib/iter.md` §3–§7. Crate: `crates/mycelium-std-iter/src/`.
- GM at `guarantee_matrix.rs`; `foldable.rs`, `lazy.rs`, `transducer.rs`, `zip_outcome.rs`
  present.
- Q1 (zip length-mismatch) is a design call; Q2 (any/all witness ergonomics) is §8-Q3; Q3 noted
  resolved; Q4 (lazy surface scope) and Q5 (migration bar) are scope/design calls.

### 3.12 `math` (`mycelium-std-math`)

**Verdict: ratification-ready with flags**

- Spec: `docs/spec/stdlib/math.md` §3–§7. Crate: `crates/mycelium-std-math/src/`.
- GM at `lib.rs:GUARANTEE_MATRIX`; `approx.rs`, `exact.rs`, `matrix.rs` present.
- Q1 (`Approx<T>` carrier shape) deferred to `numerics` M-512 — cross-module reconcile needed
  before ratification. Q2–Q5 noted resolved.
- **Shared `Proven`-tag dependency** (same as `dense`): `sum`/`dot` rows depend on ADR-010 checker
  discharge (unverified here — VR-5; treat as provisional pending M-512 confirmation).

### 3.13 `numerics` (`mycelium-std-numerics`)

**Verdict: ratification-ready with flags**

- Spec: `docs/spec/stdlib/numerics.md` §3–§7. Crate: `crates/mycelium-std-numerics/src/`.
- GM at `matrix.rs:GUARANTEE_MATRIX`; `lib.rs` present.
- Q1 (`Approx<T>` carrier) resolved — proposed as thin-view `Meta`-attached bound (not a kernel
  change); Q2 (`ProvenThm` sealed witness scope) is a design call; Q3 noted resolved; Q4/Q5 are
  ergonomics/scope.
- **This crate is the key dependency** for the `Proven` tags in `dense` and `math`; the M-512
  checked bound instantiation must be verified by the maintainer before those specs ratify their
  `Proven` rows.

### 3.14 `rand` (`mycelium-std-rand`)

**Verdict: ratification-ready with flags**

- Spec: `docs/spec/stdlib/rand.md` §3–§7. Crate: `crates/mycelium-std-rand/src/lib.rs`.
- GM at `lib.rs:GUARANTEE_MATRIX`.
- Q1 (distribution scope) is M-501-gated; Q2 (`wild`/FFI floor) is §8-Q6 (`std-sys`); Q3/Q4
  noted resolved; Q5 (seeded-read effect framing) is a §8-Q3 design call.

### 3.15 `recover` (`mycelium-std-recover`)

**Verdict: ratification-ready**

- Spec: `docs/spec/stdlib/recover.md` §3–§7. Crate: `crates/mycelium-std-recover/src/`.
- GM at `guarantee_matrix.rs`; `action.rs`, `effect.rs`, `handle.rs`, `outcome.rs`, `policy.rs`,
  `registry.rs`, `tests.rs` present.
- All §7 questions have resolution context or are M-502-gated (the self-hosted migration half).
  The Wave-4 changelog records the honest-tag fix (I2/VR-5 — recovered tag not laundered up).
- The self-hosted migration half is **not** in scope here (M-502-gated per spec status) — this
  ratification-readiness is for the Rust-first library half only.

### 3.16 `runtime` (no crate)

**Verdict: not-yet-implemented — no ratification possible**

- Spec: `docs/spec/stdlib/runtime.md`. No `mycelium-std-runtime` crate.
- Spec status is **Draft (needs-design)**; most surface is RESERVED-not-active (RFC-0008
  constructs, Phase-7 track). All 5 §7 questions are unresolved.
- No ratification possible until RFC-0008 constructs land (Phase-7) and a crate is implemented.

### 3.17 `select` (`mycelium-std-select`)

**Verdict: ratification-ready with flags**

- Spec: `docs/spec/stdlib/select.md` §3–§7. Crate: `crates/mycelium-std-select/src/lib.rs`.
- GM at `lib.rs:GUARANTEE_MATRIX` (5 rows); `Explanation` re-exported from `mycelium-select`.
- Spec correctly notes it does not fabricate the `Explanation` schema (Q1 posture is correct);
  Q2–Q4 are ergonomics/scope/design calls.

### 3.18 `self-hosting-readiness` (assessment doc, no crate)

**Verdict: not-yet-implemented — no ratification possible**

- Spec: `docs/spec/stdlib/self-hosting-readiness.md`. No crate; this is a readiness verdict doc.
- Verdict: "not yet established" (VR-5, per spec). DN-14 (same wave, M-502) records the formal
  assessment with per-feature verdicts (5 gate-fails out of 11 required features).
- No ratification possible; this is a readiness gate, not an implementation spec.

### 3.19 `spore` (`mycelium-std-spore`)

**Verdict: ratification-ready with flags (scoped to library/manifest half)**

- Spec: `docs/spec/stdlib/spore.md` §3–§7. Crate: `crates/mycelium-std-spore/src/`.
- GM at `guarantee_matrix.rs`; `recon_manifest.rs`, `spore_ops.rs` present.
- Q1 (Ring 1 vs Ring 2 placement) is a spec §4.2/§4.3 discrepancy flagged to the maintainer —
  needs one-line reconciliation in RFC-0016. Q2 (Phase-6 native-deploy, M-620) is explicitly
  Phase-6-gated; Q3 (schema fields) tracks M-368/M-359; Q4 (`vsa`↔`spore` reconstruction seam)
  needs cross-module maintainer call; Q5 (ergonomics) is §8-Q3.
- Ratification can be scoped to the library/manifest half (the landed deliverable); the
  deploy/germination half must wait for M-620.

### 3.20 `swap` (`mycelium-std-swap`)

**Verdict: divergent — spec surface is abstract only**

- Spec: `docs/spec/stdlib/swap.md` §3–§7. Crate: `crates/mycelium-std-swap/src/lib.rs`.
- GM at `lib.rs:GUARANTEE_MATRIX`.
- **Structural divergence (spec §7-Q4 explicit):** Spec §3 describes the exported-op surface
  abstractly ("illustrative — not a committed grammar"), and Q4 explicitly says: "pin §3 to the
  real surfaces before this spec is ratified." The crate exports: `bin_to_tern`, `tern_to_bin`,
  `f32_to_bf16`, `dense_to_vsa`, `vsa_to_dense`, `check_swap`, `explain`, `Swapped`,
  `SwapCertificate`, `SwapError`, `CheckError`, `ExplainRecord` (lib.rs `pub fn`/`pub struct`/
  `pub enum` entries). The spec §3 sketch does not enumerate these names explicitly. A §3
  surface-reconcile pass is needed before ratification — the spec should be pinned to the crate's
  actual export list. This is a **docs divergence** (spec underspecified relative to code), not a
  contract violation, but it means §3/§4 row labels cannot be matched to landed APIs without
  additional mapping work.
- Q1 (naming) and Q2 (call-site ergonomics) are §8-Q2/§8-Q3 design calls; Q3 noted resolved;
  Q5 (migration bar) is §8-Q5.

### 3.21 `ternary` (`mycelium-std-ternary`)

**Verdict: ratification-ready with flags**

- Spec: `docs/spec/stdlib/ternary.md` §3–§7. Crate: `crates/mycelium-std-ternary/src/`.
- GM at `guarantee_matrix.rs`; `arithmetic.rs`, `packing.rs`, `primitives.rs` present.
- Q1 (surface naming / fungal lexicon) is §8-Q2; Q2–Q4 noted resolved.

### 3.22 `testing` (`mycelium-std-testing`)

**Verdict: ratification-ready with flags**

- Spec: `docs/spec/stdlib/testing.md` §3–§7. Crate: `crates/mycelium-std-testing/src/`.
- GM at `guarantee_matrix.rs`; `verdict.rs` present.
- Q1 (scope) is M-501-gated; Q2 (`FailRecord`↔`Diag` seam) is a known fast-follow noted in
  `diag` changelog; Q3 noted resolved; Q4 (`Proven`-witness mode) is a post-v0 scope call; Q5
  (migration bar) is M-502-gated.

### 3.23 `text` (`mycelium-std-text`)

**Verdict: ratification-ready with flags**

- Spec: `docs/spec/stdlib/text.md` §3–§7. Crate: `crates/mycelium-std-text/src/`.
- GM at `guarantee_matrix.rs`; `ops.rs`, `types.rs`, `error.rs` present.
- Q1 (`Lossy<T>` shape) is a design call; Q2 (grapheme-table versioning) is a VR-5 reification
  call; Q3 (parse↔numerics seam) needs cross-module sign-off with `cmp`/`math`; Q4 (`wild`/FFI
  floor) is §8-Q6.

### 3.24 `time` (`mycelium-std-time`)

**Verdict: ratification-ready with flags**

- Spec: `docs/spec/stdlib/time.md` §3–§7. Crate: `crates/mycelium-std-time/src/lib.rs`.
- GM at `lib.rs:GUARANTEE_MATRIX`.
- Q1 (logical-clock API, M-521-owned) explicitly does not fabricate the API; Q2 (timers) deferred
  from v0; Q3 (`wild` floor, wall/mono) is §8-Q6; Q4 (`Duration` representation) is a value-model
  design call; Q5 (effect-declaration surface) is RFC-0014's to settle.

### 3.25 `vsa` (`mycelium-std-vsa`)

**Verdict: ratification-ready with flags**

- Spec: `docs/spec/stdlib/vsa.md` §3–§7. Crate: `crates/mycelium-std-vsa/src/`.
- GM at `lib.rs:GUARANTEE_MATRIX` (const) and `matrix.rs` (per-model rows); `encoding.rs`,
  `ops.rs`, `recon.rs` present.
- Q1 (scope) is M-501-gated; Q2 (`similarity` primitive vs decision tag) needs confirmation
  against `mycelium-vsa` contract; Q3 (`Proven` cells and exact theorem instantiation) shares the
  ADR-010 / encoded-matrix dependency with `dense`/`math` — **not independently verified here**
  (unverified — VR-5; treat as provisional); Q4 (resonator `Declared` coverage) defers to
  RFC-0009 §10.3; Q5 (ergonomics) is §8-Q3.

---

## 4. Cross-Cutting Divergences

### 4.1 `fmt`/`io` guarantee-tag framing mismatch for `from_json`

- **Files:** `crates/mycelium-std-fmt/src/lib.rs` (row `"from_json"`, `tag: GuaranteeStrength::Exact`);
  `crates/mycelium-std-io/src/guarantee_matrix.rs` (row `"from_json"`, `GuaranteeTag::Empirical`).
- Both are acknowledged in `fmt` spec §7-Q1 resolution and `fmt` lib.rs doc-comment. Neither
  over-claims `Proven`. However, the framing is unreconciled in committed code.
- **Advisory recommendation:** maintainer should explicitly choose one framing (or formally accept
  both as scope-distinct views) before ratifying `fmt`.

### 4.2 Provisional `Proven` tags (`dense`, `math`, `vsa`)

- **Files:** `crates/mycelium-std-dense/src/lib.rs`, `crates/mycelium-std-math/src/lib.rs`,
  `crates/mycelium-std-vsa/src/matrix.rs` and `crates/mycelium-std-numerics/src/matrix.rs`.
- `dense` §7-Q1 explicitly warns that `Proven` rows for float ops are provisional pending M-512
  checked instantiation. `vsa` Q3 similarly defers exact theorem instantiation to the encoded
  matrix. The surveyor could not independently verify these checked side-conditions.
- **Advisory recommendation:** before ratifying `dense`, `math`, or `vsa`'s `Proven` rows,
  confirm with `numerics` (M-512) maintainer that the ADR-010 checker has discharged the
  backward-error bound with all side-conditions checked. Any row the checker cannot discharge must
  downgrade to `Empirical` (VR-5; spec §7-Q1 explicit instruction).

### 4.3 Spec surface abstraction in `swap`

- **File:** `docs/spec/stdlib/swap.md` §3, §7-Q4.
- Spec §3 is deliberately illustrative ("not a committed grammar"); Q4 flags that §3 must be
  pinned to the real crate surface before ratification. The crate has a fully concrete API.
- **Advisory recommendation:** a §3 surface-reconcile pass against `crates/mycelium-std-swap/src/lib.rs`
  exported names is needed before the spec can be ratified.

### 4.4 `cmp` trait naming (`MycEq`/`MycOrd` vs `Eq`/`Ord`)

- **File:** `crates/mycelium-std-cmp/src/lib.rs:pub trait MycEq`, `MycOrd`, `MycPartialOrd`.
- Spec §3 sketch uses unqualified `Eq`/`Ord`/`PartialOrd`. The `Myc` prefix is a Rust
  namespace-collision avoidance; it is not explained in the spec and is not covered under §8-Q2.
- **Advisory recommendation:** spec §3 should note the `Myc` prefix explicitly (or accept it as
  the ratified naming) before ratification. Not a contract violation; a documentation gap.

### 4.5 `spore` Ring 1 vs Ring 2 placement discrepancy

- **File:** `docs/spec/stdlib/spore.md` §7-Q1; RFC-0016 §4.2 vs §4.3.
- The spec surfaces a discrepancy between RFC-0016 §4.2 (files `spore` under Ring 2) and §4.3
  (lists it under Tier A / Ring 1). This is a one-line reconciliation in RFC-0016, not a `spore`
  spec defect.
- **Advisory recommendation:** RFC-0016 §4.2 / §4.3 reconciliation should be done at the RFC
  ratification pass (orchestrator-owned).

---

## 5. Verdict Counts

| Verdict | Count | Specs |
|---|---|---|
| **ratification-ready** (no open blockers) | 1 | `recover` |
| **ratification-ready with flags** (open Qs are design/scope calls, not contract violations) | 17 | `cmp`, `collections`, `content`, `core`, `diag`, `error`, `fs`, `io`, `iter`, `math`, `numerics`, `rand`, `select`, `ternary`, `testing`, `text`, `time` |
| **ratification-ready with flags (scoped)** (ready for a defined subset) | 2 | `spore` (library half), `vsa` |
| **divergent** (a specific spec-to-code gap needing maintainer action before ratification) | 3 | `fmt` (tag framing), `dense` (provisional `Proven`), `swap` (abstract spec surface) |
| **not-yet-implemented** (no crate; ratification not possible) | 2 | `runtime`, `self-hosting-readiness` |

**Totals:** 25 specs surveyed. 20 ratification-ready (including scoped). 3 divergent. 2
not-yet-implemented.

---

## 6. Recommendations (for maintainer decision)

> **This section is advisory only. No spec is moved to Accepted here. All the following are
> inputs to the maintainer's append-only ratification call (DN-07 posture).**

**Pre-ratification actions the surveyor recommends the maintainer take:**

1. **`fmt`/`io` tag reconciliation** — Resolve the `from_json` guarantee-tag framing divergence
   (`fmt` `Exact` vs `io` `Empirical`). Both are honest; a formal decision on which framing wins
   (or that both are accepted as scope-distinct) closes the VR-5 residual. Required before
   ratifying `fmt`.

2. **`dense`/`math`/`vsa` `Proven`-row sign-off** — Confirm with the `numerics` (M-512)
   maintainer that the ADR-010 backward-error bound is checked and the side-conditions are
   instantiated. Any unverifiable row downgrades to `Empirical` (spec §7-Q1 instruction; VR-5).
   Required before ratifying the `Proven` rows in these three specs.

3. **`swap` §3 surface-reconcile** — Pin spec §3 to the exact exported names in
   `crates/mycelium-std-swap/src/lib.rs`. This is an editorial pass, not a contract redesign.
   Required before ratifying `swap` (spec §7-Q4 explicit gate).

4. **`cmp` naming note** — Add a note to `cmp` spec §3 explaining the `Myc` prefix (Rust
   namespace-collision avoidance for `Eq`/`Ord`/`PartialOrd`) or rename — whichever is the
   ratified choice. Low-effort documentation gap.

5. **`spore` Ring 1 vs Ring 2 reconciliation** — One-line fix in RFC-0016 §4.2/§4.3;
   orchestrator-owned.

6. **`runtime` and `self-hosting-readiness`** — No action required at this wave; these are
   correctly classified as not-yet-implemented. `runtime` awaits Phase-7 RFC-0008 constructs.
   `self-hosting-readiness` tracks the M-502 gate (gate-fails documented in DN-14).

**Sequence suggestion (advisory):**

- Straightforward batch (no research needed): `recover`, `ternary`, `diag`, `error`, `select`,
  `content`, `core`, `testing`, `io`, `rand`, `iter`, `text`, `fs`, `collections`.
- After cross-module sign-off: `cmp` (naming), `time`, `spore` (scoped), `vsa` (scoped).
- After tag reconciliation: `fmt`.
- After `Proven`-row confirmation: `dense`, `math`, `vsa` (full).
- After spec surface-reconcile: `swap`.
- Deferred (requires Phase-7 / M-502): `runtime`, `self-hosting-readiness`.

---

## Fresh post-implementation honesty re-audit (2026-06-21, swarm)

A three-batch **Sonnet swarm** (Opus-orchestrated) re-audited all 25 `mycelium-std-*`
crates *against the landed code* (guarantee-tag honesty, `cargo test -p <crate>`,
`#![forbid(unsafe_code)]`, spec↔code drift). This is a **verification** pass — it does
**not** ratify (status flips to Accepted remain the maintainer's call, RFC-0016 / DN-07).
Empirical (per-crate test runs + code reads); source is ground truth.

**Result: 24 / 25 RATIFICATION-READY; 1 NEEDS-WORK; no honesty-tag violations found.**
Every `Proven` tag traces to a checked basis (the only `Proven` rows — `dense` elementwise
float ops — are the ADR-010 per-element IEEE bound with the finiteness side-condition
guarded; `vsa`/`swap` `Proven` cells mirror the cited RFC-0003 / cert kernel matrices).
Every approximation is `Empirical`/`Declared`; every fallible op is an explicit
`Result`/`Option`; no silent NaN/sentinel escape in any crate.

| batch | crates | verdict |
|---|---|---|
| 1 | cmp, collections, content, core, dense, diag, error, fmt, fs | 9/9 ready |
| 2 | io, iter, math, numerics, rand, recover, runtime, select | 8/8 ready |
| 3 | spore, swap, sys, ternary, testing, text, time, vsa | 7 ready, **sys NEEDS-WORK** |

**Actionable items (for the maintainer's ratification pass):**
- **`mycelium-std-sys` — no `docs/spec/stdlib/sys.md`.** Code + tags honest (`[Declared]`
  wrappers; fallibility explicit), but it lacks the per-crate spec every other crate has.
  The one ratification blocker in the batch — write the spec (or fold sys under another).
- **`FLAG-RAND-IMPL` is RESOLVED** [Empirical]: `mycelium-std-rand` now uses xoshiro256++
  (Blackman & Vigna 2021) with splitmix64 seeding — the old non-crypto DefaultHasher+
  SystemTime stand-in is gone; statistical claims honestly `Declared`/`Empirical`. Any
  corpus text still describing the stand-in is stale.
- **Minor spec-text drifts (not honesty issues, document at ratification):** `swap`
  `tern_to_bin` returns `Err(OutOfRange)` (stricter) where §4 says Option `None`; `vsa`
  reuses `Err(BelowCleanupThreshold)` for margin-shortfall where §3 listed a distinct
  `Ambiguous` (documented FLAG); `text` guarantee-matrix `guarantee` field is `&str` vs the
  `GuaranteeStrength` enum used elsewhere (stylistic).
- **Test depth note:** `mycelium-std-runtime` (21 tests) is the thinnest; tags are honest
  but the empirical base is light (no concurrent-load tests) — a V&V follow-up, not a
  blocker.

This re-audit supersedes nothing; it adds a current readiness signal on top of the M-377
honesty cleanups (all of which it confirms held through implementation).

## Changelog

- **2026-06-21 — Fresh post-implementation honesty re-audit (swarm).** Three Sonnet agents
  re-verified all 25 stdlib crates against landed code: 24/25 ratification-ready, 1
  NEEDS-WORK (`std-sys` missing its spec), **no honesty-tag violations**. FLAG-RAND-IMPL
  confirmed RESOLVED (xoshiro256++). Minor spec-text drifts noted (swap `Err` vs `None`,
  vsa `Ambiguous`, text field type). Verification only — no status flips (maintainer's
  call). Append-only.
- **2026-06-19 — Draft (M-374).** Produces per-spec ratification-readiness survey for all 25
  `docs/spec/stdlib/*.md` entries. Classifies 1 ratification-ready, 17 ratification-ready-with-flags,
  2 ratification-ready-with-flags (scoped), 3 divergent, and 2 not-yet-implemented. Identifies four
  cross-cutting divergences: `fmt`/`io` guarantee-tag mismatch (`from_json` `Exact` vs `Empirical`),
  provisional `Proven` tags in `dense`/`math`/`vsa` pending M-512 ADR-010 checker discharge, `swap`
  spec surface abstract (§7-Q4 gate), and `cmp` `MycEq`/`MycOrd` naming gap. Advisory; no spec
  status changes. Append-only.
- **2026-06-19 — Resolved (M-377; maintainer-ratified honesty cleanups).** Grounded each cross-cutting
  divergence in code and closed the actionable ones:
  - **`dense`/`math`/`vsa` `Proven` rows — verified.** `math` already tags all approx ops `Declared`
    (no `Proven` claim) — honest, no change. `vsa` `Proven` cells *mirror* the RFC-0003 §4 kernel matrix
    (cited, not restated; divergence caught in tests) — honest by construction, no change. `dense` (Q1):
    **finalized** elementwise float `add`/`sub`/`scale`/`hadamard` as `Proven` (the ADR-010 per-element
    IEEE backward-error bound; finiteness side-condition guarded by `DenseError::NonFinite`; M-512
    delivered) and **aligned the §4 table's accumulation rows (`sum`/`dot`) down to `Empirical`** to match
    the landed crate (the `nγ_n` bound is a distinct unchecked theorem — VR-5-safe downgrade).
  - **`fmt`/`io` `from_json` framing — resolved scope-distinct (both tags kept).** `fmt`-`Exact` = decode
    determinism; `io`-`Empirical` = round-trip fidelity (proptest, no theorem). Different properties of the
    same call; neither over-claims. Cross-referenced in both crate guarantee matrices and both specs.
  - **`swap` §3 — pinned** to the landed `mycelium-std-swap` surface (`check_swap`, the re-exported
    `mycelium_cert::check` (M-210), no `build`, richer `CheckError`); §7-Q4 gate resolved.
  - **Remaining (minor, deferred):** the `cmp` `MycEq`/`MycOrd`/`MycPartialOrd` naming-vs-spec doc gap
    (not an honesty issue); and full per-spec *ratification* to Accepted (the maintainer's call). Append-only.
