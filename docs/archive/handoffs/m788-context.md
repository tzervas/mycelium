# M-788 — mode-gated swap-cert emission + checking (working notes + bound-reconciliation analysis)

Status: implemented (Rust-first), **pending maintainer ratification of the bound/basis decision below**.
Branch: `claude/m788-mode-gated-swap-cert`. Depends on M-786 (CertMode + tag) and M-787 (`gate_guarantee`).

## THE DESIGN DECISION TO RATIFY — bound/basis reconciliation under the `Fast` floor

**Question (deferred from M-787):** when `Fast` floors a would-be `Proven`/`Empirical` result
(which carries a *computed* bound) to `Declared`, what happens to that bound, given the `Meta`
invariants?

The actual invariants enforced by `Meta::new` → `check_guarantee_bound` (crates/mycelium-core/src/meta.rs):
- **M-I1** `Exact ⟺ bound == None`
- **M-I2** `Proven ⟹ basis == ProvenThm`
- **M-I3** `Empirical ⟹ basis == EmpiricalFit`
- **M-I4** `Declared ⟹ basis == UserDeclared`

So a `Declared` tag carrying a `ProvenThm`/`EmpiricalFit` basis is **rejected** by the constructor.
A naive "floor the tag, keep the bound as-is" would therefore violate M-I4 (and be dishonest: it
would still *claim* a proven/empirical basis the `Fast` run never established).

**DECISION TAKEN (candidate (a) from the kickoff): keep the computed bound *value*, relabel its
basis to `UserDeclared`.**

Rationale (VR-5 / transparency rule):
- The ε/δ *value* was genuinely computed by the operation, so dropping it (candidate (b)) would
  *lose* honest information — the value is real, it just wasn't *certified* in `Fast`.
- But the **basis** `ProvenThm`/`EmpiricalFit` was **not earned**: `Fast` runs no cert machinery, no
  theorem side-conditions are checked, no trials are run. Carrying that basis would be an
  upgrade-past-evidence (the exact VR-5 violation the lattice exists to prevent).
- `UserDeclared` is precisely "asserted, not (yet) verified — tooling must surface a declared,
  unverified marker" (see `BoundBasis::UserDeclared` doc). That is the honest reading of a
  `Fast`-computed bound: *computed, asserted-not-verified in this mode*.
- `UserDeclared` is the **only** basis M-I4 admits for a `Declared` tag, so this is also the unique
  reconciliation that keeps the result `Meta`-constructible. The bound payload is untouched, so a
  well-formed input stays well-formed.

Implemented as `CertMode::gate_result(intended_guarantee, intended_bound) -> (GuaranteeStrength,
Option<Bound>)` in crates/mycelium-core/src/cert_mode.rs:
- `Fast + Exact` → `(Exact, None)` (M-I1; a stray bound on an Exact intent is dropped, never a
  silent carry).
- `Fast + Proven|Empirical|Declared` → `(Declared, bound.map(relabel→UserDeclared))` (M-I4).
- `Balanced|Certified` → `(intended, bound)` unchanged — machinery runs, basis is earned.

**Tested contract:** `gate_result`'s output is *always* a pair `Meta::new` accepts — swept
exhaustively over `CertMode::ALL × {Exact, Proven, Empirical, Declared}` (the same M-I1…M-I4
checker the constructor uses). See `crates/mycelium-core/src/tests/cert_mode.rs`.

### If the maintainer prefers a different resolution
- Candidate (b) "drop the computed bound, synthesize a fresh `UserDeclared` bound" loses the
  computed value — rejected as less honest (it discards real information). If wanted instead, change
  `relabel_user_declared` to a synthesizer.
- A future option: carry the *original* computed basis somewhere inspectable (e.g. an EXPLAIN
  side-channel / provenance note) so "this UserDeclared bound was computed by a would-be-Proven op
  in Fast" is recoverable. Out of M-788 scope; flag if desired.

## Mode-gated emission/checking (the M-788 deliverable proper)

`crates/mycelium-cert/src/mode.rs` (new module):
- `gate_swap(src, value, cert, mode) -> Result<GatedSwap, SwapError>` applies the per-mode policy to
  a raw `(value, certificate)` swap result:
  - **Fast**: reconciles the value's `Meta` via `gate_result`; emits **no** certificate; runs **no**
    check (`GatedSwap { certificate: None, check: None }`).
  - **Balanced**: emits the certificate; does **not** check it; tags propagate unchanged.
  - **Certified**: emits **and** checks through the existing M-210 `check()` (unchanged); a
    non-validating verdict is carried on `GatedSwap.check` (never silent).
- `GatedSwap { value, certificate: Option, check: Option<CheckVerdict> }` makes the mode's effect
  inspectable (no black box).
- `ModeGatedSwapEngine { mode }` (default `Fast`) implements `SwapEngine`; `swap()` returns the
  gated value, and in `Certified` a `NotValidated` verdict becomes an explicit `EvalError::Swap`
  (never a silent accept of an unvalidated swap). `swap_gated()` exposes the full `GatedSwap`.
- The relation + `claimed` certificate for the `Certified` check are *derived from the emitted
  certificate* (Bijective → `Bijection`/`{0,0,Exact}`; Bounded → `BoundedSimilarity`/lifted ε|δ at
  basis-strength), so the check validates exactly what was emitted — never a tighter claim (VR-5).

**Axis-B (fallibility) is NOT gated** (RFC-0034 §4): the raw swap runs first, so out-of-range /
illegal-pair / refusal stays an explicit error in *every* mode. Tested.

## Tests
- `crates/mycelium-core/src/tests/cert_mode.rs` (white-box; new + extracted M-786/787 tests):
  `gate_result` reconciliation, M-I round-trip exhaustive, guarantee-component agrees with
  `gate_guarantee`, fast-never-Empirical/Proven negative.
- `crates/mycelium-cert/tests/mode.rs` (mode-parametric integration, RFC-0034 §13): bijective +
  bounded classes across `CertMode::ALL`; cross-mode negatives (fast never emits/checks, never
  Empirical/Proven); Axis-B errors in every mode; engine default = Fast; Certified returns value on
  validation.

## Test-layout note (as-touched, M-797)
Touching `cert_mode.rs` + `lib.rs` triggered the as-touched extraction rule. Created
`crates/mycelium-core/src/tests/{mod.rs,cert_mode.rs,lib.rs}` and moved the inline `cert_mode`
tests + the `lib.rs` `WfError` test there (`#[cfg(test)] mod tests;` in lib.rs). The rest of
mycelium-core's inline tests are left for the M-797 lazy sweep (not touched here).

## FLAGs
- **FLAG-1 (ratify):** the bound/basis decision above (keep value, relabel basis → `UserDeclared`).
- **FLAG-2 (mode source not wired):** `SwapEngine::swap(src, target, policy)` has no `CertMode`
  parameter, and the interpreter holds a `Box<dyn SwapEngine>` with no mode in scope. The active
  mode source is the `@certification` scope = **M-790** (not yet landed). So `ModeGatedSwapEngine`
  carries an **explicit** mode (default `Fast`) rather than resolving it from runtime context. The
  full mode-aware swap path is implemented and tested at the engine/`gate_swap` layer; wiring the
  resolved mode *through the interpreter* is correctly M-790's job, not fabricated here.
- **FLAG-3 (spec status):** RFC-0034 is the design; this is "implemented (Rust-first), pending
  ratification" — not silently moved to Accepted/Enacted.
