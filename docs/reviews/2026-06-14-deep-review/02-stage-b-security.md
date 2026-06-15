# Stage B — Security Audit

Status: Advisory — report only. Part of the 2026-06 deep review (see `00-summary.md`).
Baseline: HEAD `e2d627e`. Conducted against the working tree (not a diff), applying the repo's
own `.claude/skills/security-review` checklist in exhaustive mode. Two scoped passes: B1
(supply chain / CI / shell) and B2 (code-level).

**Threat model.** Mycelium is a **local research substrate / developer tool**, not a
network-facing service: no listener, no auth surface, no secrets handling inside the crates. The
one genuinely-untrusted input surface is `myc-check`, the M-002 oracle designed to run on
**LLM-generated Mycelium source**. Severities below are calibrated to that model — the residual
risks are availability/integrity (a crashable oracle, a non-injective audit anchor), not RCE.

---

## B1 — supply chain / CI / shell safety

**Verdict.** This surface is in good shape and clearly built by someone who internalized the
repo's own security checklist. The single CI workflow is correctly `workflow_dispatch`-only with
`permissions: contents: read`, honoring the manual-CI policy; secrets are fail-open/advisory and
never echoed; shell scripts uniformly use `set -euo pipefail` (via `lib.sh`), quote expansions,
and pass shellcheck clean (16/16); the Rust workspace forbids `unsafe` and `cargo audit` reports
zero advisories across 32 deps; `Cargo.lock` and `uv.lock` are committed with hashes. The
residual risks are all *mutable-reference* supply-chain exposures (actions pinned to tags not
SHAs; unpinned `npx`/tool fetches) plus process gaps (no dependabot, no `cargo deny`). Nothing
rises to Critical/High given the manual-dispatch + advisory context.

- **[Medium] B1-01** `.github/workflows/checks.yml:26,30,36,39,42,48,57,81` — every `uses:` is
  pinned to a **mutable major tag, not a full commit SHA**: `actions/checkout@v4`,
  `actions/setup-python@v5`, `actions/setup-node@v4`, `extractions/setup-just@v2`,
  `astral-sh/setup-uv@v6`, `haskell-actions/setup@v2`, `actions/cache@v4`,
  `codecov/codecov-action@v5`. → A tag can be force-moved to a malicious commit (the classic
  Actions supply-chain vector; the repo's own SKILL.md asks for pinned refs). → Pin each to a
  full 40-char SHA with a trailing `# vX` comment. Held at Medium because the workflow is
  manual-dispatch + advisory with `contents: read` only, so blast radius is small.
- **[Medium] B1-02** `scripts/checks/markdown.sh:11` (and `scripts/install-tools.sh:25`) — `npx
  --yes markdownlint-cli2 …` fetches and executes the **latest** package from npm with no
  version pin or integrity hash, on every `just check`/CI run. → A compromised/typo'd package
  runs with the user's privileges. → Pin the version (`markdownlint-cli2@<x.y.z>`). Medium since
  it's a well-known package over HTTPS, not `curl|bash`.
- **[Low] B1-03** `.github/workflows/checks.yml:54` — `sudo apt-get install -y z3` installs z3
  unpinned. → Non-reproducible proof-step input; low risk (distro-signed). → Optionally pin, or
  accept as advisory-CI-only.
- **[Low] B1-04** No `.github/dependabot.yml`/`renovate.json` (confirmed absent). → With
  tag-pinned actions (B1-01) and no update/alert bot, drift and disclosed-CVE actions go
  unnoticed. Note the *non*-tension with the manual-CI policy: dependabot raises PRs, it does
  not auto-run the advisory workflow, so adding it would not violate the no-auto-CI stance. →
  Consider a config scoped to `github-actions` + `cargo` + `pip`/`uv`.
- **[Low] B1-05** `tools/github/gh-bootstrap-local.sh:35,68` — the milestone map writes to a
  predictable world-readable temp path `MSMAP=/tmp/mycelium-msmap.tsv` via `: > "$MSMAP"`. →
  Mild TOCTOU/symlink-clobber on a shared host; content is only public milestone titles. → Use
  `mktemp`. Script is otherwise solid (preflight, quoted vars).
- **[Nit] B1-06** `.gitleaks.toml:14-19` — allowlists whole paths `research/` and `README.md`.
  → A real key pasted under `research/` would be silently un-scanned (the file's own comment
  flags this). → Tighten to specific files when feasible; currently a documented trade-off.
- **[Info] B1-07** `.github/workflows/checks.yml:78-87` — `CODECOV_TOKEN` is referenced only as
  `env:` to the codecov action (never echoed in `run:`), and the step is `continue-on-error:
  true` + `fail_ci_if_error: false` → fail-open, no leakage. Correct posture; recorded as a
  clean positive.

**Confirmations (no finding):** trigger is `workflow_dispatch` only — policy honored;
`permissions: contents: read` — least privilege; no untrusted input (`github.event.*`,
PR titles) is interpolated into any `run:` block — no script-injection vector; embedded Python
is safe (`structured.sh:30` uses `yaml.safe_load_all`; `lint_links.py` uses list-form
`subprocess.run`, reads only); no `eval`, no `curl … | bash` anywhere; `uv run --frozen` is used
in CI (`checks.yml:77`) and `test.sh:17` (pre-commit runs no tests, so "frozen in pre-commit" is
N/A, not a gap).

**Tool availability / gaps (→ Stage D):** Ran — `cargo audit` 0.22.2 → **clean, 0 advisories
over 32 deps** (`Cargo.lock` has registry + sha256 checksums); `shellcheck` → clean 16/16;
`scripts/checks/secrets.sh` → ran the narrow fallback (gitleaks absent); manual broad credential
grep → no matches. Absent (GAP) — **`gitleaks`** (secrets.sh degrades to a minimal git-grep
fallback, so full secret scanning is never exercised locally or, per C1-09, in CI); **`cargo
deny`** (license/ban/source-allowlist + duplicate-dep policy unenforced). Not installed here per
instructions.

**Clean areas:** `Cargo.toml` workspace (`unsafe_code = "forbid"`, resolver 2, no
git/path/wildcard deps, MSRV pinned via `rust-toolchain.toml`); committed `Cargo.lock` +
`experiments/uv.lock` (hash-locked, `.venv` untracked); `experiments/pyproject.toml` (empty
runtime deps); `.pre-commit-config.yaml` (pre-commit-hooks pinned `rev: v5.0.0`, local hooks
delegate to repo scripts); no `.claude/settings.json` or auto-executing hook/skill script in
the tree.

---

## B2 — code-level security

**Verdict.** The Rust kernel is disciplined (zero `unsafe`, BLAKE3 content addressing with
domain separation, serde wire structs that re-validate invariants on decode, explicit-error
parsing with no `unwrap`/panic on input), so memory-safety and most injection classes are out of
scope. The exploitable issues are **availability/integrity, not RCE**: an unbounded
recursive-descent parser that one crafted input crashes (DoS of the oracle), a content-hash
collision where non-finite `f64` literals canonicalize to JSON `null` (two opposite policies
share one PolicyRef), and a documented-but-not-enforced `exec()` of baseline code that is safe
only by operational convention. The parser DoS and the PolicyRef collision both undermine the
oracle/audit-trail trustworthiness that is the whole point of M-002.

- **[High] B2-01** `crates/mycelium-l1/src/parse.rs` (recursive-descent stack,
  `parse_expr`→`parse_let`/`parse_if`/`parse_match`→`parse_expr`, ~lines 352-476) +
  `bin/myc-check.rs:102` — **no recursion-depth guard.** A crafted nested input overflows the
  stack and aborts (SIGABRT). Independently confirmed: debug-build crash between depth 2000-3000
  (`exit=134`); ~20k reliably aborts. The harness `check()` has a 60s timeout, but an instant
  abort bypasses it — mapped to `ToolUnavailable` and *raised*, halting the run rather than
  scoring a `parse-error`. `check_colony` and `elaborate` are likewise unbounded over the same
  AST. → One adversarial/degenerate LLM-generated program crashes the oracle. → Add an explicit
  depth counter (or `stacker`-style growth) in parser/checker/elaborator returning a clean
  error. (Corroborates A4-02 — A4 measured ~5k in a release build with a larger frame budget;
  debug crashes earlier.)
- **[Medium] B2-02** `crates/mycelium-select/src/lib.rs:117,140-143,387-391` — **PolicyRef hash
  collision on non-finite predicate literals.** `policy_ref()` hashes `serde_json::to_string`;
  serde_json serializes `NaN`/`+Inf`/`-Inf` all to `null` (verified). So
  `ErrorEpsAtMost(NaN/Inf/-Inf)` yield identical PolicyRefs yet evaluate to opposite results.
  `new` validates `cost.storage_weight` but not predicate literals. → Integrity / audit trail:
  the EXPLAIN record's "which policy decided this swap?" becomes non-injective. → Reject
  non-finite `f64` in predicate literals at construction/deserialization. (Same defect as A5-01,
  from the integrity angle.)
- **[Medium] B2-03** `crates/mycelium-core/src/bound.rs:99-108` (`Bound::well_formed`) —
  `EmpiricalFit{trials}` is unvalidated and `Error{eps}` accepts `+Inf`. → Tampered-manifest
  threat: a crafted JSON `Meta`/`Bound` with `basis: EmpiricalFit{trials: 0}` deserializes as a
  well-formed **Empirical** guarantee with zero supporting trials — an evidence-free
  honesty-lattice claim that a stronger guarantee than the evidence supports. `Meta::deserialize`
  re-runs M-I1…M-I4 but those check only guarantee↔basis coupling, never `trials >= 1` or `eps`
  finiteness. → Require `trials >= 1` for `EmpiricalFit` and `eps.is_finite()` for `Error`.
  (Same defect as A6-02 / A1-02, as a tamper vector.)
- **[Low] B2-04** `experiments/mycelium_experiments/kc2/checkers.py:108-158`
  (`BaselineChecker.exec`) — `exec(source, namespace)` of baseline-DSL source with an unguarded
  namespace: no `__builtins__` key, so CPython auto-injects the **full** builtins (`open`,
  `__import__`, `eval`, …); no restricted-builtins sandbox, path allowlist, or AST denylist. The
  "fixtures/reference-solutions only; untrusted output runs in a disposable container" invariant
  lives only in the class docstring. → Arbitrary-code-execution **if** the M-002 run ever feeds
  model output here outside a sandbox; in the intended use (curated fixtures + reference
  solutions, sandbox for the real run) it is not attacker-reachable, hence Low. → Gate `exec`
  behind an explicit `allow_untrusted=False` that refuses unless a sandbox marker is set, or
  strip `__builtins__`; at minimum make the requirement a runtime assertion. (Refines A6-10 from
  the security angle — the missing-`__builtins__` full-injection is the salient detail and the
  requirement is not structurally sufficient.)
- **[Low] B2-05** workspace `Cargo.toml` — no `[profile.release]`, so `overflow-checks = false`
  (cargo default) in release. No exploitable overflow exists today (lexer int parse is checked
  `i64`; widths via `u32::try_from`; hash length-prefixes `as u64` on `Vec` lengths). → Latent:
  future numeric/packing code would wrap silently in release, at odds with the never-silent
  ethos. → Consider `overflow-checks = true` for the trusted kernel crates' release profile, or
  a `checked_*`/`saturating_*` discipline.

**Confirmations.** `unsafe` count: **0** (the two `rg unsafe` hits are doc comments for L1's
`wild` keyword); `unsafe_code = "forbid"` set once at `Cargo.toml:27` (`[workspace.lints.rust]`)
and inherited via `[lints] workspace = true`, **no per-crate `allow` override** — forbid is
effective. **Hash:** BLAKE3 (`content.rs`) via a `Canon` encoder that is **domain-separated**
(one tag byte per syntactic form) and **length-prefixed**, injective, **no truncation** (full
32-byte digest; `id.rs` validates shape on deserialize); kernel `Node`/`Value` hashing has no
B2-02-style collision because f64 goes via `to_bits` (so `NaN`/`±0.0`/`Inf` are distinct) — the
collision is specific to `mycelium-select`'s JSON path. **Serde decode is validating**
(`Meta`/`ReconInfo`/`SelectionPolicy`/`ContentHash` re-run constructor invariants), **but no
struct uses `#[serde(deny_unknown_fields)]`** — unknown fields are silently dropped on all wire
types (posture note, combines with B2-03). **Not exploitable:** parser/lexer have no
`unwrap`/`expect`/panic/slice-index on input (`parse.rs:382` `(self.i+1).min(len-1)` is
bounds-safe because the lexer appends EOF); integer literals are checked-`i64`; no command
injection (`checkers.py` uses list-form `subprocess.run`, never `shell=True`).

**`MYC_CHECK` trust note (informational, not a finding).** `checkers.py:60-66` reads
`$MYC_CHECK`, requires `Path(env).is_file()`, and execs it — no validation beyond existence, no
allowlist, trust model undocumented. In the local-dev model, anyone who can set `MYC_CHECK`
already controls the environment and `PATH` (and could poison the `cargo build` tree anyway), so
it grants no privilege escalation. → Worth a one-line docstring note that `MYC_CHECK` is
operator-trusted. The fallback discovery (`target/debug/myc-check`, then `cargo build`) uses
fixed args and `cwd=root` with no shell.

**Clean areas:** Rust memory safety (zero unsafe, forbid effective); content-hash core integrity
(BLAKE3, domain-separated, no truncation); subprocess invocation (list-form, no shell);
lexer/parser panic-freedom and integer-overflow handling; serde invariant re-validation. No
secrets, no network surface, no `curl|bash`, no command injection.
