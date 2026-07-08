# Design Note DN-91 — The CU-3 Float↔Int Conversion Surface: Reified Lossy-Conversion Swaps + Checked Signed Prims

| Field | Value |
|---|---|
| **Note** | DN-91 |
| **Status** | **Draft** (2026-07-08; **works up the decision for the maintainer to ratify** — recommends an option with rationale + an alternative; decides nothing normatively and moves no decision status. Advisory, `Declared` throughout — house rule #3, never jumps to `Accepted`). |
| **Decides** | *Proposes, for ratification:* (D5) the **reified lossy-conversion swap surface** that ADR-040 §2.4/§5 rules is a certificate-carrying **swap, NOT a prim** — the two **paradigm-crossing** lossy modes Rust `as` needs: a **rounding** swap `Binary{N} → Float{F64}` (for `\|n\| > 2^53`, RNE, carrying a relative-error bound) and a **saturating** swap `Float{F64} → Binary{N}` (NaN/±inf/out-of-range → a reified clamp, fractional part truncated-toward-zero), each reusing RFC-0002's `SwapCertificate` machinery and inheriting M-788 mode-gating; (D4) the **checked signed** int↔float conversion **prims** (`bin.to_flt_s` / `flt.to_bin_s`, exact-or-refuse, two's-complement), distinct from the lossy swap, resolving `FLAG-cu3-signed-conv`; and the **verify-first correction** that the third listed mode — **wrapping-truncate** (int→narrower-int) — is **already designed** as DN-51's explicit `truncate` (same-paradigm, no certificate), not a new swap. |
| **Grounds** | **ADR-040 §2.4** (never-silent conversion boundaries — the lossy-round direction is "a reified, `EXPLAIN`-able conversion carrying its bound"), **§2.5** (minimal kernel surface), **§2.6** (conversion tags `Empirical`), **§3** (content-address — adding swap arms spends no rehash; a lossy swap produces a distinct honestly-addressed value, it does not fork the source's identity) · **RFC-0002 §2/§3/§5** (the `SwapCertificate`, the `Bounded` regime, the legal-pair table; the `strength`-derived-from-basis rule) · **RFC-0033 §4.1.1** (Binary is a sign-free bitvector; signedness is a property of *operations*) · **DN-41** (`width_cast` — the width-witness ABI the CU-3 prims reuse) · **DN-51** (Accepted — the explicit `truncate` op that *is* the wrapping-truncate mode) · **DN-34 §8.17/§8.18** (the CU-3 prims landed; the A1 `Expr::Cast` FLAGs `PENDING-DESIGN(CU-3-fidelity)` + `FLAG-cast-narrow-fidelity`) · **M-211** (`dense_f32_to_bf16` — the first Bounded lossy swap, the shape to mirror) · **M-788 / RFC-0034 §4** (mode-gated cert emission/checking) · source: `crates/mycelium-core/src/prim.rs:571-605`, `crates/mycelium-interp/src/prims.rs:2368-2439`, `crates/mycelium-cert/src/{lib.rs:54-81,dense.rs:81-141,mode.rs:120-150}`, `crates/mycelium-core/src/bound.rs:18-111`, `crates/mycelium-std-math/src/exact.rs:24-149`, `docs/spec/stdlib/math.md:37,81`. |
| **Task** | tracking id **not minted** (mitigation #1 — the orchestrator/maintainer verifies a free `E*`/`M-xxx` slot before minting). This note is a design artifact on a non-conflicting leaf worktree; it **enacts no code**. Sibling coordination: the `trx2-a2-wrapping-surface` leaf owns the CU-5 **wrapping** surface — this note **defers wrapping-truncate to that lane + DN-51** (see §3.2), it does not design it. |
| **Related** | ADR-028 (Binary sign-free ⇒ unsigned magnitude is the default reading; signed is a distinct op) · DN-72 (the `_u`/`_s` signedness naming convention) · RFC-0016 §4.5 (the `std.swap` guarantee matrix) · `docs/spec/stdlib/swap.md` (the swap surface home) |

> **Posture (transparency rule / VR-5 / G2).** This note is **Draft / advisory** — a design direction
> for the maintainer to ratify. It **does not** move any decision to `Accepted`/`Enacted` on its own
> authority (house rule #3), and it **upgrades no guarantee past its basis**. Every normative claim
> cites a ratified ADR/RFC/DN or is marked an open question (house rule #4). The current-state survey
> (§2) is `Empirical` — grounded in cited source at `file:line`, where the source is ground truth, not
> this note. The proposed design is `Declared` (a proposal). Where this note's evidence cuts against the
> task's framing — the "three lossy modes" — it **says so plainly** (§3.2: wrapping-truncate is not a
> new swap; it is DN-51's `truncate`), because being corrected beats being wrongly affirmed.

---

## §1 Purpose & the gap (closes design gates D4 + D5)

Lane A1 (`trx2`, DN-34 §8.18) landed the transpiler's `Expr::Cast` arm but had to **gap every
float-crossing cast** with `PENDING-DESIGN(CU-3-fidelity)` and every narrow cast with
`FLAG-cast-narrow-fidelity`. The reason is a genuine, deliberate semantic mismatch (house rule #2):
**Rust `as` is lossy/saturating/rounding/wrapping by design, while Mycelium's CU-3 conversion prims are
checked/refusing by design.** The landed prims (DN-34 §8.17, source `prims.rs:2368-2439`) are:

- `bin.to_flt : Binary{N} → Float` — **checked-exact**: refuses (`EvalError::Overflow`) when the
  unsigned magnitude exceeds `2^53` (binary64's exact-integer bound). It does **not** round.
- `flt.to_bin : (Float, Binary{M}) → Binary{M}` — **checked-exact**, width-witness shape (à la DN-41
  `bit.width_cast`): refuses NaN/±inf/negative/fractional/out-of-target-width. It does **not** saturate
  or truncate.

So a faithful transpilation of `n as f64` (which *rounds* for `|n| > 2^53`), `f as i32` (which
*saturates* NaN/±inf/out-of-range since Rust 1.45), or `x as u16` (which *wraps* — keeps the low bits)
**cannot be emitted** against the checked prims: emitting a refusing prim where Rust produces a total,
lossy value would be an *unfaithful* emission (VR-5). The prims are correct; what is missing is the
**opt-in, never-silent lossy surface** that *matches Rust's total semantics while reifying the loss*.

This note designs that surface. It closes two gates:

- **D5 (highest-leverage) — the reified lossy-conversion swap.** ADR-040 §2.4/§5 already **rules** the
  lossy direction is "a reified, `EXPLAIN`-able conversion carrying its bound — never a silent lossy
  cast" and that it is a **swap, NOT a prim**. This note designs that swap's surface, its
  `SwapCertificate` shape, and its honesty tags (§4).
- **D4 — checked signed int↔float conversions.** The CU-3 prims read Binary as an **unsigned**
  magnitude (ADR-028; `FLAG-cu3-signed-conv` flags the signed variant as undecided). This note designs
  the checked **signed** conversion prims — exact-or-refuse, distinct from the lossy swap (§5).

## §2 Verify-first survey — what already exists (`Empirical`, cited)

Per the maintainer directive and mitigation #14, the honest current state is recorded *before*
proposing, so this note reuses machinery rather than re-inventing it.

| Asset | Where | Shape / status |
|---|---|---|
| CU-3 checked prims | `prims.rs:2368-2439`, `prim.rs:604-605` | `bin.to_flt` (checked-exact, refuse `\|n\|>2^53`), `flt.to_bin` (checked-exact, width-witness, refuse NaN/inf/neg/fractional). **Unsigned-magnitude** (ADR-028). Tag `Empirical` (ADR-040 §2.6). |
| `SwapCertificate` | `mycelium-cert/src/lib.rs:54-81` | `#[serde(tag="kind")]` enum: `Bijective{…, lemma_ref, params}` and **`Bounded{src, target, policy_used, bound: Bound}`** — the variant a lossy conversion reuses. |
| `Bound` / `BoundKind` / `BoundBasis` | `mycelium-core/src/bound.rs:18-111` | `Bound{kind, basis}`; `BoundKind::Error{eps:f64, norm:NormKind}` (+ `Probability`/`Crosstalk`/`Capacity`); `BoundBasis::{ProvenThm, EmpiricalFit, UserDeclared}` with `.strength()` → `Proven`/`Empirical`/`Declared`. |
| M-211 bounded swap (the precedent) | `mycelium-cert/src/dense.rs:81-141` | `dense_f32_to_bf16(src, policy) -> Result<(Value, SwapCertificate), SwapError>`: guards source, rounds, emits value + `Bounded{bound: Error{eps=BF16_REL_EPS, norm:Rel}, basis: ProvenThm}`. **The exact function shape to mirror.** |
| Mode-gating (M-788) | `mycelium-cert/src/mode.rs:120-150`, `cert_mode.rs:117-148` | `gate_swap(src, value, cert, mode)` → drops the cert in `fast`, emits-not-checks in `balanced`, emit+check in `certified`. **Axis-B (fallibility) is never gated** — the raw swap runs and refuses in every mode. |
| Reified rounding kernel | `mycelium-std-math/src/exact.rs:24-149` | `RoundMode{Floor,Ceil,TruncTowardZero,HalfAwayFromZero,HalfToEven}`; `checked_round_to_i64` **refuses** overflow (never saturates) — the never-silent rounding vocabulary a saturating swap composes with. |
| DN-51 `truncate` (Accepted) | `docs/notes/DN-51-*.md` §Decides(3), §57-95 | Explicit **`truncate`** — "unconditionally drops the high `N−M` bits, total but lossy, only ever via the named op." **This is the wrapping-truncate mode** (§3.2). |
| SoC boundary | `docs/spec/stdlib/math.md:37`, RFC-0016 §4.5 | A **representation** change (paradigm-crossing) is `std.swap`; `math`'s `round` explicitly **refuses** saturation (`math.md:81,175`). |

**No** float↔int *swap* exists yet; **no** saturating/clamping conversion exists anywhere
(`std-swap/src/lib.rs:8` states swaps are "never a clamp, a re-round, or a sentinel"). That absence —
the deliberate never-silent stance — is exactly what makes a *reified, opt-in* lossy swap the correct
addition rather than a relaxation of the checked prims.

### §2.1 Prior art (external, `Declared` context — not a Mycelium guarantee)

Every mainstream toolchain treats lossy float↔int conversion as an *explicit, defined* operation — the
design converges on exactly the shapes §4 proposes. This is cited context, not a checked Mycelium
claim (`Declared`):

- **Rust `as`.** Since **1.45**, `f as iN/uN` is **total and saturating**: NaN → `0`, values above the
  target max → `MAX`, below the min → `MIN`, the fractional part truncated toward zero; `n as f64`
  rounds to nearest (RNE). This is precisely §4.1 (rounding) + §4.2 (saturating). Before 1.45 the
  out-of-range case was undefined behavior — the very "silent, unspecified lossy cast" Mycelium's
  never-silent rule (house rule #2) rejects.
- **LLVM.** The plain `fptosi`/`fptoui` return **poison** on unrepresentable inputs; LLVM added
  **`llvm.fptosi.sat` / `llvm.fptoui.sat`** intrinsics that *saturate to MIN/MAX and map NaN → 0* —
  the exact `SatCase` set (`NanToZero`/`SaturateHigh`/`SaturateLow`/`InRange`) §4.2 reifies. `fptrunc`
  is the width-narrowing float→float round (out of this note's scope: F64-only, ADR-040).
- **WebAssembly.** The trapping `iNN.trunc_fMM_s/u` instructions were joined by the non-trapping
  **`trunc_sat_*`** family with the *same* saturate-to-MIN/MAX, NaN→0 semantics — Rust's saturating
  `as` lowers to these on the wasm backend.
- **C11.** `(int)f` when the value is out of range is **undefined behavior** (§6.3.1.4) — the
  cautionary anti-pattern: a silent, unspecified, platform-dependent result. Mycelium's checked prim
  (refuse) + reified saturating swap (record) is the never-silent alternative to C's UB.

The takeaway (VR-5): the *behavior* is universal and well-specified elsewhere; Mycelium's contribution
is not new semantics but making the loss **reified and honesty-tagged** (a certificate carrying the ε
or the clamp case), never a silent or poison-valued result.

## §3 The taxonomy — three loss shapes, two families

The task frames "three lossy modes." Verify-first, they do **not** all share a family, and being
precise about this is the note's core contribution:

### §3.1 Two of the three cross paradigms → the certificate-swap family (D5, this note's novelty)

A `Repr::Binary{N}` ↔ `Repr::Float{F64}` conversion changes the **paradigm** of the value — it is a
**representation swap** in the RFC-0002/ADR-040 §5 sense, and per `math.md:37` its home is `std.swap`.
Two of the modes are here:

- **Rounding** (`Binary → Float`, `\|n\| > 2^53`): a value close to `n` with a **small, statable
  relative-error bound**. This is a *genuine* `Bounded` swap (a metric ε).
- **Saturating** (`Float → Binary`, NaN/±inf/out-of-range): the value is **discretely replaced** by a
  clamp (`0`, `MAX`, or `MIN`) — the "error" can be arbitrarily large (NaN→0), so it is **not** an
  ε-bounded swap. Its certificate records **which clamp fired** and the truncation mode, not an ε.

That these two have *different bound flavors* (a metric bound vs a discrete replacement record) is the
key honest design constraint §4 addresses.

### §3.2 The third does not cross paradigms → it is DN-51's `truncate`, not a new swap (verify-first correction)

Wrapping-truncate (`x as u16`, keep low `M` bits) is a **same-paradigm** `Binary{N} → Binary{M}`
narrowing. It changes width, not paradigm — so it is the **width-cast family** (DN-41/DN-51), **not** a
`SwapCertificate`-bearing representation swap. And it is **already designed**: DN-51 (Accepted
2026-06-27) adds the explicit **`truncate`** op — "unconditionally drops the high `N−M` bits, total but
lossy, only ever via the named op." The honest bound is an **exact congruence** (`result ≡ n (mod
2^M)`), not an ε and not a clamp — so forcing it into the certificate machinery would be a category
error.

**Ruling (G2/VR-5, mitigation #14):** wrapping-truncate is **not** in scope for the D5 swap surface.
`x as u16` (narrow) transpiles to DN-51 `truncate` (or, under a `wrapping { … }` block, the CU-5
wrapping mode owned by the `trx2-a2` sibling leaf). This note **flags the seam** and defers, rather
than duplicating a landed design. The A1 gap `FLAG-cast-narrow-fidelity` is thereby closed by *wiring
`truncate`*, not by a new swap — see §6.

## §4 D5 — the reified lossy-conversion swap surface (recommended: Option A)

Two swaps, in `std.swap`, each a `(src, policy) -> Result<Swapped, SwapError>` function mirroring
`dense_f32_to_bf16` (`dense.rs:81-141`), each dispatched through `CertifiedSwapEngine`/`raw_swap` and
therefore inheriting M-788 mode-gating (`mode.rs:120-150`) for free.

### §4.1 The rounding swap — `swap.round.bin_flt` (Binary{N} → Float{F64})

The lossy companion to `bin.to_flt`. When the unsigned magnitude `n ≤ 2^53` the result is exact
(the checked prim's domain); when `n > 2^53` the result is `n` rounded to nearest, ties-to-even (RNE,
ADR-040 §2.3) — the host `u64→f64`/`i64→f64` conversion.

- **Certificate:** reuses `SwapCertificate::Bounded{src: Binary{N}, target: Float{F64}, policy_used,
  bound}` unchanged, with `bound = Bound{ kind: BoundKind::Error{ eps: 2^-53, norm: NormKind::Rel },
  basis: … }`. The relative bound `u = 2^-53` is the binary64 unit roundoff — the exact statement
  "`|fl(n) − n| / |n| ≤ u`" (standard correctly-rounded-RNE rounding theory; the same relative-error
  shape M-211 uses for bf16, `dense.rs:106-114`).
- **Honesty tag (VR-5):** `Empirical` at introduction. Rationale, stated honestly: the ε *value* is
  `ProvenThm`-citable (rounding theory), but the *claim that the host delivers correctly-rounded RNE*
  is host-conformance — exactly the `Empirical` posture ADR-040 §2.6 pins on the whole `flt.*` group
  and on `bin.to_flt` itself. So `basis: EmpiricalFit{trials, method}` at introduction, with a
  documented path to `ProvenThm` once host-conformance is discharged (never upgraded before then).

### §4.2 The saturating swap — `swap.sat.flt_bin` (Float{F64} → Binary{N})

The lossy companion to `flt.to_bin`, faithful to Rust 1.45+ `f as iN/uN`: **total** (never refuses),
saturating. The fractional part is truncated **toward zero** (`RoundMode::TruncTowardZero`, reused from
`exact.rs:24-40`); then the integer is clamped to the target range. The four clamp cases:

| Input | Result | Recorded case |
|---|---|---|
| finite, in range | truncated integer | `InRange` (bound: `\|err\| < 1`, from truncation) |
| `NaN` | `0` | `NanToZero` |
| `+inf` or `> MAX` | target `MAX` | `SaturateHigh` |
| `−inf` or `< MIN` | target `MIN` | `SaturateLow` |

The out-of-range cases **replace** the value; there is no small ε (NaN→0 is unbounded). So the
saturating swap does **not** reuse `BoundKind::Error` — that would abuse a metric type as a
replacement record (a house-rule-#4 honesty smell). Instead:

- **Recommended:** a **new `SwapCertificate` variant** `Saturated{ src: Float{F64}, target: Binary{N},
  policy_used, case: SatCase, rounding: RoundMode }` — a *discrete transform record*, EXPLAIN-able,
  sitting alongside `Bijective`/`Bounded` as append-only enum growth (the same pattern the enum was
  built for). `SatCase ∈ {InRange, NanToZero, SaturateHigh, SaturateLow}`.
- **Honesty tag:** `InRange` → `Empirical` (the truncation is bounded, host-conformance posture);
  every clamp case → `Declared` (the value was *replaced* per a declared saturation policy — an
  asserted, always-flagged transform, never a checked equality). `flt_to_bin_sat` carries a signed or
  unsigned target (§5's two's-complement reading selects the `MAX`/`MIN` bounds).

### §4.3 Cross-cutting: mode-gating, EXPLAIN, and content-address

- **Mode-gating comes for free (M-788).** Both swaps dispatch through the same engine, so
  `gate_swap` applies unchanged: in `fast` the certificate is dropped but **the swap still runs and the
  value still carries its honest tag** (`cert_mode.rs:117-148` floors the grade to `Declared`, keeping
  the computed ε value, relabelling its basis `UserDeclared`); in `balanced` the cert is emitted, not
  checked; in `certified` it is emitted and checked. Crucially, **Axis-B is never gated** — the swap is
  *reified and visible in every mode* (G2). Never-silent holds regardless of mode.
- **EXPLAIN-ability (no black boxes).** Each swap is a reified `Node::Swap{src, target, policy}`
  (`node.rs:67-75`); its certificate (`Bounded` ε or `Saturated` case) is the `EXPLAIN` payload. A
  future `EXPLAIN` reports "round `Binary{64} → Float{F64}` (rel-err ≤ 2⁻⁵³)" or "saturate `Float{F64}
  → Binary{32}` (case: NanToZero; fractional part truncated toward zero)" directly from the node +
  cert — the loss is inspectable, never hidden.
- **Content-address: no identity fork (ADR-040 §3).** A lossy swap produces a **new, distinct value**
  (the rounded/saturated result) with its own honest address = `blake3(canon(target Repr) ‖
  canon(payload))`; the source value's address is untouched — a swap *derives*, it does not *rewrite*.
  `Meta.policy_used` records the derivation. Because the `Float`/`Binary` variant tags are already
  frozen (ADR-040 §3), adding these swap arms produces existing `Repr` shapes and **spends no rehash**.
  NaN is canonicalized at value construction (ADR-040 §2.3), so `swap.sat`'s `NaN` input maps
  deterministically to `0` on every host.

## §5 D4 — the checked signed int↔float conversion prims (exact-or-refuse)

The CU-3 prims read Binary as an **unsigned** magnitude (ADR-028). Rust's `i32 as f64` /
`f64 as i32` are **signed**. Because Binary is a sign-free bitvector and *signedness is a property of
operations* (RFC-0033 §4.1.1, ADR-028), a signed conversion is a **distinct named op** that reads the
bits as **two's-complement** — exactly how `bin.div_s`/`bin.shr_s` split from their unsigned kin
(`prim.rs:326-328`). These stay **prims** (small, checked, exact-or-refuse — KC-3), *not* swaps:

- **`bin.to_flt_s : Binary{N} → Float`** — reads `Binary{N}` as a signed two's-complement integer;
  exact when `\|n\| ≤ 2^53`, else explicit `EvalError::Overflow`. (The lossy signed-rounding direction
  is the §4.1 swap with a signed source reading — not this prim.)
- **`flt.to_bin_s : (Float, Binary{M}) → Binary{M}`** — width-witness shape (DN-41); exact when the
  `Float` is finite, integer-valued, and in the signed `M`-bit range `[−2^{M−1}, 2^{M−1})`; else
  explicit refusal (NaN/±inf → `PrimType`; out-of-range/fractional → `PrimType`/`Overflow`). Never
  saturates (that is §4.2's opt-in swap).

**Tags:** `Empirical` on both (ADR-040 §2.6 host-conformance posture, matching the existing unsigned
prims). This **resolves `FLAG-cu3-signed-conv`**.

**Naming (open question, DN-72).** The `_s` suffix follows DN-72's signedness convention. Whether the
existing `bin.to_flt`/`flt.to_bin` should gain an explicit `_u` suffix for symmetry (they are the
unsigned default per ADR-028) is a **naming question deferred to the maintainer** — this note
recommends keeping the bare name as the unsigned default and adding `_s`, avoiding a rename churn, but
flags the alternative.

## §6 The transpiler flip (DN-34 §8.18, once ratified + implemented)

Once this surface is ratified and landed, A1's gapped `Expr::Cast` arms **flip from gap → emit**:

| Rust cast | Today (DN-34 §8.18) | After ratify + impl |
|---|---|---|
| `n as f64` (int→float, may lose precision) | `PENDING-DESIGN(CU-3-fidelity)` | emit `swap.round.bin_flt` (signed source ⇒ the §4.1 swap over the §5 signed reading); the exact checked prim (`bin.to_flt`/`_s`) is the tighter choice where the transpiler can prove `\|n\| ≤ 2^53` |
| `f as i32` (float→signed int) | `PENDING-DESIGN(CU-3-fidelity)` | emit `swap.sat.flt_bin` (signed target `Binary{32}`) — faithful to Rust 1.45+ saturation |
| `x as u16` (int→narrower int, wrap) | `FLAG-cast-narrow-fidelity` | emit **DN-51 `truncate`** (§3.2 — *not* a new swap); or the CU-5 wrapping mode under `wrapping { … }` |
| `Binary{N} as Binary{M}`, `M ≥ N` (unsigned widen) | already emits `width_cast` | unchanged (DN-41, `Exact`) |

The float-crossing arms move from `Declared` gap-reasons to `Declared` emissions that a differential
can then upgrade to `Empirical` (the M-1006 ladder path). No arm is upgraded past its basis.

## §7 Guarantee posture (lattice `Exact ⊐ Proven ⊐ Empirical ⊐ Declared`; VR-5)

- **`Exact`** — the D5 rounding swap's *in-range* case (`n ≤ 2^53`) and the D4 checked signed prims'
  in-domain results: each equals the reference value exactly. (Grounded: the value is representable, so
  the conversion is a total, lossless, decidable map.)
- **`Empirical`** — the rounding swap's out-of-range result (`n > 2^53`, ε = 2⁻⁵³ Rel, host-conformance
  per ADR-040 §2.6); the saturating swap's `InRange` truncation; the D4 prims' host-conformance. Each
  established by property/boundary trial, not proof.
- **`Declared`** — every **clamp** case of the saturating swap (`NanToZero`/`SaturateHigh`/
  `SaturateLow`): the value was *replaced* per a declared policy; asserted-and-flagged, never a checked
  equality. Also the whole surface in `fast` mode (the M-788 floor).
- **Never upgraded.** No `Proven` is claimed. The ε value is `ProvenThm`-citable but the *result grade*
  stays `Empirical` until host-conformance is discharged (VR-5).

## §8 User stories

- As a **transpiler author**, I want a faithful, never-silent emission for `n as f64` / `f as i32`, so
  that the A1 `PENDING-DESIGN(CU-3-fidelity)` gap flips to a real emission instead of a refusing prim
  that misrepresents Rust's total semantics.
- As a **Mycelium programmer**, I want lossy float↔int conversion to be an **explicit, opt-in** value
  op that reifies its loss (a rounding bound or a named clamp case), so that I can match hardware/Rust
  semantics when I need them **without** a silent lossy cast — and reach for the checked prim (D4/D5's
  exact-or-refuse) when I want the loss to be an error instead.
- As a **certified-mode user**, I want every lossy conversion's accuracy claim tagged at its honest
  strength (rounding `Empirical` with ε; saturation `Declared` with its clamp case) and checkable via
  the swap certificate, so that a `certified` run validates the bound and a `fast` run still shows the
  swap and its floored tag.
- As a **kernel auditor**, I want the signed conversion to be a small, checked prim distinct from the
  lossy swap (exact-or-refuse, two's-complement per ADR-028), so that the trusted base stays minimal
  (KC-3) and the lossy behavior lives entirely in the reified, certificate-carrying swap layer.
- As the **maintainer**, I want the swap-certificate machinery *reused* (not re-invented) and the
  content-address impact settled (no identity fork, no rehash spent), so that this surface lands as
  append-only growth over RFC-0002/ADR-040, not a new mechanism.

## §9 Alternatives considered

**Option B — fold both lossy modes under `SwapCertificate::Bounded` by extending `BoundKind`.** Instead
of a new `Saturated` cert variant (§4.2), add `BoundKind::Saturated{case}` (and let wrapping-truncate
be `BoundKind::Truncated`), so all lossy conversions share the one `Bounded` variant.
*Tradeoff:* smaller enum surface (one type grows, not two) and a single dispatch arm — but it
**conflates a metric bound with a discrete replacement record** inside `Bound`, whose whole meaning is
"an ε/δ metric" (`bound.rs:66-100`). A `Bound` that is sometimes a real ε and sometimes a "the value
was replaced" flag is the honesty smell house rule #4 warns against; downstream code that reads
`BoundKind::Error.eps` would have to special-case the non-metric variants. **Rejected** in favor of A
(a distinct `Saturated` cert variant keeps `Bound` meaning exactly "metric bound"), but B is the
smaller diff if the maintainer prefers to freeze the `SwapCertificate` variant set.

**Option C — no saturating swap; refuse out-of-range, require an explicit compose.** Keep only the
checked prims + the rounding swap; make the transpiler emit an explicit `is_finite`/range-guard +
clamp composition for `f as i32`.
*Tradeoff:* zero new certificate machinery and maximal never-silent purity — but it **cannot faithfully
transpile Rust's total saturating `as`** (a total cast becomes a partial, multi-op expansion), and it
pushes the clamp policy into transpiler-generated glue where it is *not* reified in a certificate
(harder to EXPLAIN/audit than a first-class `Saturated` swap). **Rejected:** it trades the D5 fidelity
goal for machinery savings, and the reification (the whole point of ADR-040 §2.4's "reified,
EXPLAIN-able conversion") is weaker.

**Option D — make the lossy conversions prims after all.** *Rejected by ADR-040 §2.4/§5 directly* — the
lossy direction "is a swap with an explicit cert," not a prim. Recorded only to show the ruling was
consulted, not re-litigated (house rule #3).

## §10 Definition of Done

- [ ] **Maintainer ratifies** the recommended Option A (or selects B/C) — the `Draft → Accepted` move
      (house rule #3). Until then this stays **Draft** and enacts nothing.
- [ ] The **taxonomy** (§3) is accepted: D5 covers the two paradigm-crossing modes (round, saturate);
      wrapping-truncate is DN-51 `truncate`/CU-5, **not** a new swap (the verify-first correction).
- [ ] The **D5 swap shapes** are ratified: `swap.round.bin_flt` reusing `SwapCertificate::Bounded`
      (`Error{eps=2⁻⁵³, norm:Rel}`); `swap.sat.flt_bin` via the `Saturated{case, rounding}` cert
      variant (or Option B's `BoundKind` extension) — with the `SatCase` set fixed.
- [ ] The **D4 signed prims** (`bin.to_flt_s`/`flt.to_bin_s`, exact-or-refuse, two's-complement) are
      ratified, resolving `FLAG-cu3-signed-conv`; the `_u`/`_s` naming question (§5) is decided.
- [ ] **Per-op tags at honest basis** (§7): in-range `Exact`; rounding/truncation `Empirical` (ε or
      `<1`); clamp cases `Declared`; `fast`-mode floor to `Declared` — no upgrade past basis (VR-5).
- [ ] **Never-silent + EXPLAIN + content-address** confirmed (§4.3): swap reified in every mode
      (Axis-B ungated), certificate is the EXPLAIN payload, no identity fork, no rehash spent.
- [ ] **Transpiler-flip plan** (§6) is agreed so A1's `PENDING-DESIGN(CU-3-fidelity)` and
      `FLAG-cast-narrow-fidelity` arms have a wiring target once the surface lands.
- [ ] (On ratification) a follow-on task is minted (mitigation #1: verify a free `M-xxx`/`E*` slot) to
      implement the swaps + prims Rust-first with three-way (L1/L0/AOT) differentials mirroring the
      CU-3 tests, moving to "implemented (Rust-first), pending ratification" — never silently
      `Accepted`/`Enacted`.

> **Append-only (house rule #3).** This note **supersedes nothing** and moves no decision status from
> itself. It *extends* ADR-040 (§2.4's lossy-conversion ruling → a concrete swap surface), *reuses*
> RFC-0002's `SwapCertificate`, *resolves* `FLAG-cu3-signed-conv`, and *defers* wrapping-truncate to
> DN-51 (Accepted) + the CU-5 sibling lane. ADR-040 stays Enacted, RFC-0002/RFC-0033/DN-41/DN-51 stay
> Accepted — this note advances none of them. CHANGELOG / Doc-Index / issues.yaml / docs/api-index are
> owned by the integrating parent (FLAG up, not edited here).

---

## Meta — changelog

- **2026-07-08 — Created (Draft, advisory) — authored (`trx2` design wave, leaf).** Works up the CU-3
  float↔int conversion surface for maintainer ratification, closing design gates **D4 + D5**. Records
  the **verify-first survey** (§2 — the landed CU-3 checked prims, the `SwapCertificate`/`Bound`/M-788
  machinery, the M-211 bounded-swap precedent, DN-51 `truncate`, the `std.swap` SoC boundary — every
  row a `file:line` citation, `Empirical`). Proposes (§3-§6): the **taxonomy** (two paradigm-crossing
  lossy modes are certificate swaps; wrapping-truncate is DN-51 `truncate`, a **verify-first
  correction** of the task's "three modes" framing — house rule #4, mitigation #14); **D5** — two
  reified lossy swaps in `std.swap`: `swap.round.bin_flt` (reuses `SwapCertificate::Bounded` with a
  `2⁻⁵³` relative bound, `Empirical`) and `swap.sat.flt_bin` (a **new `Saturated{case, rounding}` cert
  variant**, clamp cases `Declared`, in-range truncation `Empirical`), both inheriting M-788
  mode-gating and never-silent in every mode (Axis-B ungated); **D4** — checked **signed** conversion
  **prims** `bin.to_flt_s`/`flt.to_bin_s` (exact-or-refuse, two's-complement per ADR-028/RFC-0033
  §4.1.1), resolving `FLAG-cu3-signed-conv`; the **transpiler flip** table (§6) that turns A1's
  `PENDING-DESIGN(CU-3-fidelity)` + `FLAG-cast-narrow-fidelity` gaps into emissions. **Per-op tags at
  honest basis** (§7, VR-5 — in-range `Exact`, rounding/truncation `Empirical`, clamp `Declared`, no
  `Proven`). **Content-address** settled (§4.3 — a lossy swap derives a distinct honestly-addressed
  value, forks no identity, spends no rehash; ADR-040 §3). Recommends **Option A** with **Options
  B/C/D** as evaluated alternatives (§9). DoD = the `Draft → Accepted` ratification gate + a minted
  follow-on task (§10). **Enacts no code; decides nothing normatively — Draft, `Declared` throughout.**
  CHANGELOG / Doc-Index / issues.yaml / docs/api-index owned by the integrating parent. (Append-only;
  VR-5; G2.)
