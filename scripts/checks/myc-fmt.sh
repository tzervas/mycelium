#!/usr/bin/env bash
# Canonical-formatting gate (M-364; `mycfmt`). For each real project root, every tracked `.myc`
# must already be in canonical form — `mycfmt --check` writes nothing and exits 1 if any file
# would reformat (contract §5: 0 ok · 1 would-reformat · 2 parse · 3 header · 4 out-of-scope/pin ·
# 64 usage · 66 io). Skips gracefully when cargo is absent or there is no project to format.
# Scope excludes tests/fixtures/ (intentionally-bad must-fail inputs; locked decision #3).
source "${BASH_SOURCE%/*}/../lib.sh"
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
  if cargo run -q -p mycelium-fmt --bin mycfmt -- --check --config "$root/mycelium-proj.toml" "${files[@]}"; then
    ok "$root: ${#files[@]} nodule(s) canonically formatted"
  else
    fail "$root: nodule(s) not canonical (run \`cargo run -p mycelium-fmt --bin mycfmt -- --write <file>\`)"
    rc=1
  fi
done

exit $rc
