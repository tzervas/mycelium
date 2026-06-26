# Design Note DN-44 — Codebase Security Posture (the implementation-hardening reference)

| Field | Value |
|---|---|
| **Note** | DN-44 |
| **Status** | **Proposed** (2026-06-26) — a **consolidating reference** for the security posture of Mycelium's *implementation* (the Rust kernel + reference interpreter + toolchain), tying together the decisions already made (ADR-014, RFC-0034, RFC-0035, the Security-Checks-Contract, DN-21) into one grounded map + an explicit statement of the **thesis** and its **trade-offs**. It **makes no new normative decision** — it cites the Enacted/Accepted basis and *proposes* a hardening **ratchet** (advisory→blocking) for the maintainer to ratify. The advisory scan additions it describes (`cargo-machete`, the kernel-hardening clippy pass) land **Rust-first** with this note; the spec status stays **Proposed** until ratified — never silently `Accepted`/`Enacted`. |
| **Feeds** | The **M-678 hardening epic** (DN-21 is its unsafe-inventory capture; this note is its posture map). Complements **RFC-0035** (the *shippable, Mycelium-native* security toolkit for `.myc` **programs**) — DN-44 covers the *implementation* side; RFC-0035 covers the *developer's-program* side. Informs `/security-review`. |
| **Date** | June 26, 2026 |
| **Decides** | *Proposes, for ratification:* (1) the **posture thesis** below as the project's stated security goal; (2) the **advisory `just scan` additions** — `cargo-machete` (unused-dep / supply-chain-surface reduction) and a **kernel-hardening clippy pass** (panic-path visibility over the trusted base), both **advisory / never-silent**, never blocking (they would fight the *intentional* panic-on-overflow design and the `-D warnings` gate if forced); (3) the **ratchet roadmap** (§6) — the conditions under which each advisory check is promoted to blocking, each its own future decision. |
| **Task** | M-678 (hardening epic — posture capture). The advisory scan wiring lands with this note (Rust-first). |

> **Posture (transparency rule / VR-5 / G2).** This note is **Empirical/Declared** and **aggregating** —
> source + the cited Enacted/Accepted decisions are ground truth; it neither re-decides nor upgrades any
> guarantee past its basis. Every claim about the *current* posture carries a `file:line` or doc citation;
> every *future* tightening is marked **proposed** and gated on its own decision (append-only — house rule
> #3). Where the posture is aspirational rather than enforced today, this note says so plainly.

---

## §1 The thesis (the goal this posture serves)

**Mycelium itself — the Rust kernel, the reference interpreter/compiler, and the toolchain — is
inherently hardened, so that the *only* security vulnerabilities that can exist are ones a developer
introduces into their *own* `.myc` programs. Nothing inherent to the implementation should be a
vulnerability.** (CLAUDE.md §what-this-repo-is; the maintainer's stated intent.)

This is a *goal* (`Declared`), approached by construction — not a proven-complete claim. What makes it
**true today**, with evidence, is §2; the trade-offs accepted to get there are §4; the gap between
today's enforcement and the goal is the ratchet in §6. The complementary half — catching the vulns a
developer *does* write into their programs — is **RFC-0035** (the Mycelium-native security toolkit),
out of scope here.

## §2 The hardening floor — what holds today (grounded)

| Property | Mechanism (enforced) | Citation |
|---|---|---|
| **Trusted base is unsafe-free, by the compiler** | `#![forbid(unsafe_code)]` at crate root of `mycelium-core` / `-cert` / `-numerics` / `-vsa` | `crates/mycelium-core/src/lib.rs:11`, `-cert/src/lib.rs:20`, `-numerics/src/lib.rs:23`, `-vsa/src/lib.rs:20` |
| **Interpreter + all stdlib are unsafe-free** | 29 crates total carry per-crate `forbid` (the reference interpreter + all 23 `mycelium-std-*`) | DN-21 §2 |
| **The entire `unsafe` surface is 8 blocks, all confined + justified** | every `unsafe` is in `mycelium-mlir` (JIT `dlopen`/`dlsym`/`transmute`-call for the AOT path), each with an adjacent `// SAFETY:` | DN-21 §2; `crates/mycelium-mlir/src/jit.rs`, `bitnet.rs`, `specialize.rs` |
| **Never-silent failability** | out-of-range / partial ops return `Option`/`Result`; arithmetic **panics on overflow, never silently wraps** (`overflow-checks = true`) | `Cargo.toml:88`; RFC-0001; lint.sh §comment |
| **Memory-safe in *every* certification mode** | `fast` and `certified` are both memory-safe; `unsafe` reachable only via an explicit, per-use, source-visible escape | RFC-0034 §9 (Enacted) |
| **Supply chain is pinned + permissive-only** | crates.io as the only registry; MIT + permissive license allow-list; RustSec (cargo-deny/audit) + OSV.dev (osv-scanner) | `deny.toml:6-48`; `osv-scanner.toml` |

The trusted base being `forbid(unsafe)` is the load-bearing fact: memory-unsafety in the kernel is not
"avoided by review", it is **rejected by the compiler** — so the only unsafe the project ships is the 8
audited FFI/JIT blocks in one non-trusted-base crate, where the unavoidable native-perf path lives.

## §3 The gate map — enforcement (never silently green, G2)

`just check` (the blocking, local↔CI-parity gate) runs all of these; each is **never-silent** (a missing
tool is reported as reduced coverage in dev and **hard-fails** in CI — Security-Checks-Contract §2):

| Gate | Script | Enforces | On absence |
|---|---|---|---|
| Supply-chain | `deny.sh` | cargo-deny (advisories/licenses/sources/bans via `deny.toml`) + cargo-audit | skip local / **fail** CI |
| Rust `unsafe` justification | `safety.sh` | every Rust `unsafe` carries an adjacent `// SAFETY:` (≤12 lines); every shippable `.myc` `wild {}` is `@std-sys` + `!{ffi}` + justified | **hard-fail** (pure git-grep) |
| Per-use unsafe escape | `unsafe-per-use.sh` | (A) trusted-kernel crates retain `#![forbid(unsafe_code)]`; (B) every non-kernel `unsafe` has a per-use `#[allow(unsafe_code)]` — **no** crate-global allow | **hard-fail** (pure git-grep) |
| Secrets | `secrets.sh` | gitleaks (+ high-confidence fallback: private keys, AWS/GitHub/Slack tokens) | fallback + report |
| Lint | `lint.sh` | `clippy --all-targets --all-features -- -D warnings -A unsafe_code` + `ruff` | skip if absent |
| Mycelium security | `myc-sec.sh` | `wild`-block inventory + justification-presence over real `.myc` roots | skip if no roots |

Advisory, opt-in, **in-env / no CI runners** (`just scan`, **not** part of `just check`):

| Scanner | Tool | Surfaces | Verdict |
|---|---|---|---|
| Supply-chain (OSV) | `osv-scanner` | vulns via OSV.dev (no git clone — survives the env git-proxy hijack; see §5) | **fail** on finding |
| Unsafe usage | `cargo-geiger` | `unsafe` across the dep tree | advisory |
| Feature combos | `cargo-hack` | feature-powerset compile of `mycelium-mlir` (`mlir-dialect` / `bitnet-accel`) | **fail** on broken combo |
| Unused deps | `cargo-machete` | dependency-surface reduction (candidates — false-positive-prone) | advisory |
| Kernel panic-paths | clippy (`unwrap`/`expect`/`indexing`) over the trusted base | panic-path sites for review (panic-freedom ratchet) | advisory |

## §4 The trade-offs, made explicit (the balance)

The posture is a set of *deliberate* trade-offs, not an absolute. Naming them is the point (the maintainer's
"target specific things, make the balance legible" instinct):

1. **`unsafe` is permitted-but-warned, not globally forbidden** (ADR-014, sharpened by RFC-0034 §9). The
   trade: a *zero-unsafe* workspace would forbid the JIT/FFI the AOT performance path needs. Resolution —
   forbid `unsafe` in the **trusted base + interpreter + all stdlib** (29 crates, compiler-enforced) and
   **confine** the unavoidable `unsafe` to one non-trusted crate (`mycelium-mlir`), per-use-escaped,
   `// SAFETY:`-justified, and grep-auditable. Performance is bought without putting the trusted base at risk.
2. **Kernel arithmetic *panics* on overflow — it does not silently wrap, and it is not unwrap-free**
   (`overflow-checks = true`, `Cargo.toml:88`). Correctness + never-silent are ranked **above**
   panic-freedom: a wrong-but-quiet result is worse than a loud abort. Consequence: panic-freedom lints
   (`unwrap_used`/`expect_used`/`indexing_slicing`) are **advisory**, not blocking — a blanket `deny` would
   fight this design and, under `-D warnings`, break the build. (Today: **47 logic-path sites** across the
   trusted base — 22 `core`, 21 `vsa`, 1 `cert`, 0 `numerics` — most invariant-guaranteed; §6 is the path
   to ratchet these down.)
3. **`bans` are `warn`, not `deny`** for duplicate versions and version-less path deps (`deny.toml:40-42`).
   The trade: a pre-1.0 dependency tree carries transitive duplicates outside our control, and the
   publishable `mycelium-std-*` crates legitimately use version-less intra-workspace path deps. `warn`
   surfaces the smell (G2) without failing on an unavoidable pre-1.0 reality; the documented plan is to
   tighten to `deny` once the tree settles.
4. **`fast` default · `certified` on request** (RFC-0034). The trade: full per-op auditability has a cost.
   Resolution — memory-safety + never-silent failability hold in **every** mode (including `fast`); only the
   *certification* machinery (provenance, swap-certs) is opt-in. Usability is the default; auditability is a
   tunable, never a silent omission.
5. **In-env supply-chain leans on OSV.dev, not RustSec-via-git** — see §5; a pragmatic trade forced by the
   execution environment, not a weakening.

## §5 The environment caveat — the supply-chain gate's git-proxy artifact

In the managed web/remote execution environment, the `deny` gate's RustSec path is **hijacked** by a
session-injected git rewrite (`insteadOf` routes cargo-deny/audit's public `advisory-db` fetch through a
scoped git proxy that 403s out-of-scope repos) — a **false red, not a finding**. The disposition (full
write-up: `.claude/memory/toolchain.md` §"Supply-chain gate"; `scripts/checks/deny.sh` header):
- **OSV.dev (`osv-scanner`) is the working in-env supply-chain gate** — it queries over plain HTTPS with no
  git clone, so the rewrite never touches it. Currently **clean** over the workspace.
- **`cargo deny` is made reliable on demand** by `just deny-net-fix` (a scoped, eyes-open override that lets
  *only* the public RustSec repo use the allowed HTTPS path — TLS and HTTPS_PROXY untouched, org 403s still
  reported, never a blanket bypass).
- The `deny` gate now **degrades with an explicit "env artifact" message** instead of a misleading red, and
  still **hard-fails in CI** (where no proxy hijack exists). Never silently green.

## §6 The hardening ratchet (proposed — each promotion is its own decision)

The posture *tightens over time*; it never loosens (house rule #3). The following are **proposed**, not
enacted — each needs its own ratification, and none is silently assumed done:

1. **Kernel panic-freedom: advisory → blocking.** Triage the 47 trusted-base panic-path sites (§4.2);
   for each, either replace with a never-silent `Result` or annotate with a justified
   `#[allow(clippy::unwrap_used)] // INVARIANT: …`. When the trusted base is clean under the hardening
   lints, promote the `just scan` advisory pass to a blocking gate (an ADR — it formalizes "the kernel
   does not panic except on the documented overflow contract").
2. **Dependency-surface cleanup.** Resolve the `cargo-machete` candidates (`mycelium-std-io`→`serde`,
   `mycelium-std-math`→`mycelium-numerics`, `mycelium-std-testing`→`mycelium-std-core`, +`mycelium-core`)
   — each verified per-crate (machete has false positives, e.g. derive-only `serde`), removed or ignored
   with a recorded reason. Shrinks the supply-chain surface.
3. **`bans` warn → deny** once the pre-1.0 dependency tree settles (§4.3) — a `deny.toml` change + decision.
4. **RFC-0035 toolkit** lands the developer-program-side scanning (the other half of the thesis).

## §7 Definition of Done (this note)

- **DN-44 written** (this file), Status **Proposed**, honestly framed as an *aggregating reference* + a
  *proposed ratchet* (no new normative decision), every current-posture claim cited.
- **Advisory scan additions landed** (Rust-first): `cargo-machete` and the kernel-hardening clippy pass wired
  into `scripts/checks/scan.sh` (advisory / never-silent), with `setup-scan` installing `cargo-machete`.
- **Registered** in `docs/Doc-Index.md`; **CHANGELOG** entry appended (append-only).
- **`just scan` green** (advisory; the new checks report, never block); `just check` unaffected.
- The ratchet items (§6) are recorded as **proposed**, each gated on its own future decision — nothing in
  this note moves a decision to `Accepted`/`Enacted` on its own authority.

---

## Changelog
- 2026-06-26 — **Proposed** (M-678 / DN-44). Consolidates the implementation-hardening posture (trusted-base
  `forbid(unsafe)`, the 8-block confined unsafe surface, the gate map, the explicit trade-offs, the env
  supply-chain caveat) into one grounded reference; lands the advisory `cargo-machete` + kernel-hardening
  clippy additions to `just scan` (Rust-first); proposes the hardening ratchet (§6). Spec **Proposed**,
  pending maintainer ratification.
