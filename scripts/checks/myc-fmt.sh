#!/usr/bin/env bash
# Canonical-formatting gate (M-364; `mycfmt`). For each real project root, every tracked `.myc`
# must already be in canonical form — `mycfmt --check` writes nothing and exits 1 if any file
# would reformat (contract §5: 0 ok · 1 would-reformat · 2 parse · 3 header · 4 out-of-scope/pin ·
# 64 usage · 66 io). Skips gracefully when cargo is absent or there is no project to format.
# Scope excludes tests/fixtures/ (intentionally-bad must-fail inputs; locked decision #3).
#
# Canonical form per root (M-974 / DN-82): the human-authored stdlib `lib/std` is enforced in the
# **readable** multi-line style (`--readable`) — long argument/variant/arm segments wrapped for a
# human reader; every other root (the `examples/*`) stays in the default compact canonical. Both are
# identity-preserving projections (same surface AST — C1/C2); the choice is presentation-only and
# scoped to keep churn off the examples + off the grammar conformance fixtures (DN-82 §Decision).
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../lib.sh"
cd "$REPO_ROOT" || exit 1
section "myc-fmt (canonical formatting — mycfmt)"

if ! have cargo; then skip "no cargo — myc-fmt gate skipped"; exit 0; fi

myc_roots
if [[ ${#MYC_ROOTS[@]} -eq 0 ]]; then
  skip "no project root (mycelium-proj.toml outside tests/fixtures/) — nothing to format"
  exit 0
fi

rc=0
for root in "${MYC_ROOTS[@]}"; do
  mapfile -d '' -t files < <(git ls-files -z -- "$root/*.myc")
  if [[ ${#files[@]} -eq 0 ]]; then
    skip "$root: no .myc nodules"
    continue
  fi
  # M-974/DN-82: the human-authored stdlib enforces the readable multi-line canonical; other roots
  # (examples) keep the compact default. The style flag is the ONLY per-root difference.
  style_flag=()
  style_hint=""
  case "$root" in
    */lib/std|lib/std) style_flag=(--readable); style_hint=" --readable" ;;
  esac
  if cargo run -q -p mycelium-fmt --bin mycfmt -- --check "${style_flag[@]}" --config "$root/mycelium-proj.toml" "${files[@]}"; then
    ok "$root: ${#files[@]} nodule(s) canonically formatted"
  else
    fail "$root: nodule(s) not canonical (run \`cargo run -p mycelium-fmt --bin mycfmt -- --write${style_hint} --config $root/mycelium-proj.toml <file>\`)"
    rc=1
  fi
done

exit $rc
