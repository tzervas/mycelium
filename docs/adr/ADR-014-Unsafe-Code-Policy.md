# ADR-014 — `unsafe` Rust: permitted-but-warned (explicit, justified, dev-warned, release-silenceable), not forbidden

| Field | Value |
|---|---|
| **ADR** | 014 |
| **Title** | Relax the workspace `unsafe_code` lint from `forbid` to a *permitted-but-warned* policy: unsafe is allowed when explicit and justified, warns in dev/test as a caution incentive, is non-blocking in the check gate, and is silenceable for production release |
| **Status** | **Accepted** (maintainer deliberation, 2026-06-15) |
| **Date** | 2026-06-15 |
| **Depends on** | ADR-007 (Rust kernel = trusted base); ADR-009 (hybrid execution — AOT/JIT need FFI); RFC-0004 §2 (native backend / direct-LLVM path); KC-3 (small auditable kernel); NFR-3 (auditability); **G2** (no black boxes / never silent) |
| **Amends** | the workspace lint policy (`Cargo.toml [workspace.lints.rust] unsafe_code`) and the lint check gate (`scripts/checks/lint.sh`) |

## Context

The workspace pinned `[workspace.lints.rust] unsafe_code = "forbid"` since M-091 — a strong stance
that kept the trusted base (ADR-007) and every crate provably free of `unsafe` (NFR-3 / KC-3). That
was the right default for the design phase. But the build path now needs `unsafe` in *bounded,
deliberate* places: **in-process JIT / FFI** (M-340; `dlopen`/`dlsym` to call a compiled artifact
without spawning a process) and any future libMLIR/native FFI (RFC-0004 §2; ADR-009). `forbid` is
absolute — it **cannot** be overridden by an `#[allow]` at the site — so it blocks even a single,
well-justified, reviewed `unsafe` block, forcing an all-or-nothing choice.

The maintainer's intent (2026-06-15): *permit* deliberate `unsafe`, but **incentivize caution,
explicitness, and intentionality** — surface a warning during dev/test compilation so an unsafe
block is never invisible, let it run (don't block intentional unsafe), and let a developer silence
the warning for production-release code once they own the decision. This keeps the never-silent
ethos (G2) pointed at the *right* thing: an unjustified or accidental unsafe is loud, a justified
one is explicit and auditable.

## Decision

Replace `forbid` with a **permitted-but-warned** policy, implemented with three verified mechanisms
(the rustc lint-level semantics were checked empirically before adopting them):

1. **Workspace default `unsafe_code = "warn"`** (was `"forbid"`). Plain `cargo build` / `cargo test`
   — the dev and test runs — emit a non-fatal **warning at every `unsafe` site** (the caution
   incentive) and **compile/run** (intentional unsafe is never blocked here).
2. **The check gate allows it: `scripts/checks/lint.sh` runs `cargo clippy … -- -D warnings -A
   unsafe_code`.** `-D warnings` keeps *every other* warning a hard error (the gate stays strict);
   `-A unsafe_code` exempts only the unsafe-usage lint, so **intentional unsafe passes CI/`just
   check`** rather than being escalated to an error. (Verified: `-D warnings` alone *implies*
   `-D unsafe_code` and a trailing `-W` does **not** override it, but `-A unsafe_code` does, while an
   unrelated warning such as an unused variable still errors.)
3. **Per-site discipline (the explicit, intentional, release-silenceable part):** every `unsafe`
   block carries a mandatory `// SAFETY:` justification, and to silence the dev warning **for
   production release only** it is annotated:

   ```rust
   // SAFETY: <why this is sound>
   #[cfg_attr(not(debug_assertions), allow(unsafe_code))]
   unsafe { /* … */ }
   ```

   This warns in debug/dev/test (the nudge stays) and is silent in `--release` (production). A site
   the developer fully owns may instead use an unconditional `#[allow(unsafe_code)]` (silent
   everywhere) — still explicit and grep-auditable.

The trusted-base ethos is preserved by *convention*, not by an absolute compiler bar: unsafe stays
**rare, bounded, justified, and reviewed** (the `/security-review` skill already expects "`unsafe`
blocks justified + bounded"). The `forbid`-grade guarantee is downgraded to a *warned + audited*
guarantee — honestly, per G2, this is a real reduction in the static guarantee, recorded here rather
than slipped in.

## Consequences

- **Enables** in-process JIT/FFI (M-340) and future native FFI without an all-or-nothing fight, using
  raw `extern "C"` `dlopen`/`dlsym` (no new dependency) as the first intentional, justified unsafe.
- **Costs** the absolute "zero unsafe" property. Mitigations: the dev/test warning keeps every unsafe
  visible; the mandatory `// SAFETY:` comment + `#[allow]` makes each site explicit and grep-able
  (`rg 'unsafe '` / `rg 'allow\(unsafe_code\)'`); the trusted kernel crates (`mycelium-core`,
  `-cert`, `-numerics`, `-vsa`) should **stay unsafe-free** and may re-pin `unsafe_code = "forbid"`
  at the *crate* level (a crate-level `forbid` still works) — recommended follow-on so the relaxation
  applies to the *perf/FFI* crates, not the trusted base.
- A lightweight follow-on check (a `just` / security-review grep asserting every `unsafe` has an
  adjacent `// SAFETY:`) would make the convention enforceable; recorded as future work, not gated
  here.
- Append-only: this ADR *amends* the lint policy; it does not rewrite the M-091 rationale. If the
  relaxation proves too loose, supersede with a tighter ADR (e.g. crate-scoped allow-lists).

## Grounding

ADR-007 (trusted base), ADR-009 + RFC-0004 §2 (AOT/JIT FFI need), KC-3 / NFR-3 (auditability), **G2**
(never silent — the reduction in guarantee is disclosed, and unsafe is warned, not hidden).
