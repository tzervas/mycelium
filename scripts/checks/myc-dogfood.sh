#!/usr/bin/env bash
# myc-dogfood — run the REAL native toolchain over the self-hosted `lib/compiler/*.myc` frontend
# (M-989). As the L1 frontend is ported to Mycelium, its `.myc` files are checked by the Rust
# `cargo test` differential harnesses (crates/mycelium-l1/tests/compiler_stage*.rs); this gate adds
# the SECOND, independent witness — the actual `myc` toolchain — so the port is vetted by both the
# old (Rust) and new (native `myc`) tooling for parity, not just embedded in cargo tests.
#
# Scope is LIGHT + on purpose. `lib/compiler/` is not a `mycelium-proj.toml` project root, so the
# `just myc-check`/`myc-fmt` project gates skip it; this walks the tracked `lib/compiler/*.myc`
# directly. Tools:
#   * `myc-check` (oracle mode, per file) — the CORE parity check (parse + L1 type-check).
#   * `mycfmt --check` — canonical-form, ADVISORY (mycfmt refuses a couple files on the M-690
#     nested-match-arm-comment limitation; that must not fail this gate).
#   * `myc-lint`      — ADVISORY (warnings are exit 0; only an error-severity finding is non-zero).
#   * `myc-sec` is intentionally NOT run here: it has no per-file interface (dir-only `--project`)
#     and `lib/compiler` has no `wild { … }` blocks, so its audit is a no-op — repo-root `myc-sec`
#     already covers the tree.
#
# HEAVY, VSA/GPU-bound dogfood work is OUT of this gate by design — running the self-hosted evaluator
# over whole programs, `cargo-mutants`, and fuzzing belong on a GPU-equipped local machine (session
# teleport), mirroring how `just check-full`/`just mutants`/`just fuzz` are held out of `just check`.
#
# NON-GATING by default: the self-hosted frontend is in-progress (M-740, canonical only at M-741), so
# a `myc check` failure here PRINTS (never silent, G2) but does NOT turn `just check` red — the Rust
# differential gates own correctness. Pass `--strict` (or MYC_DOGFOOD_STRICT=1) to make a core
# `myc check` failure exit non-zero (for deliberate enforcement once the port stabilizes).
#
# Exit: 0 on success / advisory / graceful skip; non-zero ONLY under --strict when the core
# `myc check` fails (reason sub-code 3 = check error, matching myc-check's own contract). Skips
# gracefully when cargo is absent.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib.sh"
cd "$REPO_ROOT" || exit 1
section "myc-dogfood (native toolchain over self-hosted lib/compiler — M-989; non-gating)"

if ! have cargo; then skip "no cargo — myc-dogfood gate skipped"; exit 0; fi

strict=0
[[ "${MYC_DOGFOOD_STRICT:-0}" == 1 || "${1:-}" == "--strict" ]] && strict=1

tracked 'lib/compiler/*.myc'
if [[ ${#TRACKED[@]} -eq 0 ]]; then
  skip "no tracked lib/compiler/*.myc — nothing to dogfood"
  exit 0
fi

rc=0

# --- CORE: native `myc check` (oracle mode, one file per invocation) --------------------------
check_fail=0
for f in "${TRACKED[@]}"; do
  if cargo run -q -p mycelium-check --bin myc-check -- "$f" >/dev/null 2>&1; then
    ok "myc check   $f"
  else
    fail "myc check   $f  (native parse/type-check failed)"
    check_fail=1
  fi
done
if (( check_fail == 1 && strict == 1 )); then rc=3; fi

# --- ADVISORY: mycfmt --check (canonical form; mycfmt batches, exit 1 = would-reformat) -------
if cargo run -q -p mycelium-fmt --bin mycfmt -- --check "${TRACKED[@]}" >/dev/null 2>&1; then
  ok "mycfmt      all ${#TRACKED[@]} nodule(s) canonical"
else
  skip "mycfmt      non-canonical or refused (advisory; run \`cargo run -p mycelium-fmt --bin mycfmt -- --check lib/compiler/*.myc\` — M-690)"
fi

# --- ADVISORY: myc-lint (batches; warnings exit 0, only an error-severity finding is non-zero) -
if cargo run -q -p mycelium-lint --bin myc-lint -- "${TRACKED[@]}" >/dev/null 2>&1; then
  ok "myc-lint    no error-severity findings"
else
  skip "myc-lint    error-severity finding(s) (advisory; run \`cargo run -p mycelium-lint --bin myc-lint -- lib/compiler/*.myc\`)"
fi

if (( rc != 0 )); then
  fail "myc-dogfood: core \`myc check\` failed under --strict"
else
  ok "myc-dogfood: native toolchain parity over ${#TRACKED[@]} self-hosted nodule(s) (Rust differential is the other witness)"
fi
exit "$rc"
