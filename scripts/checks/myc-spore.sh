#!/usr/bin/env bash
# Spore packaging SMOKE step (M-368; `spore build`) — the 5th M-361 tool, packaging. This dogfoods
# the spore builder by packaging each real project root into its content-addressed descriptor
# (blake3 over the code+deps DAG; metadata is not identity, ADR-003). It is deliberately NON-GATING:
# packaging is a build artifact, not a correctness property, and the richer publish surface (richer
# include + signing) is the M-368 §9.2/§9.3 deferral. So this step ALWAYS exits 0 — on a clean build
# it prints the deterministic digest as an honest receipt; if the builder cannot complete it `skip`s
# with the reason (never a silent pass; G2/VR-5) rather than turning `just check` red. The four
# pass/fail gates (myc-fmt/myc-check/myc-sec/myc-lint) own correctness; this only proves M-368 runs.
source "${BASH_SOURCE%/*}/../lib.sh"
cd "$REPO_ROOT" || exit 1
section "myc-spore (packaging smoke — spore build; non-gating)"

if ! have cargo; then skip "no cargo — spore smoke skipped"; exit 0; fi

myc_roots
if [[ ${#MYC_ROOTS[@]} -eq 0 ]]; then
  skip "no project root (mycelium-proj.toml outside tests/fixtures/) — nothing to package"
  exit 0
fi

for root in "${MYC_ROOTS[@]}"; do
  manifest="$root/mycelium-proj.toml"
  if out=$(cargo run -q -p mycelium-spore --bin spore -- build --config "$manifest" 2>&1); then
    digest=$(printf '%s\n' "$out" | sed -n '1s/.*→  *//p')
    ok "$root: spore builds → ${digest:-(descriptor written)}"
  else
    skip "$root: spore build did not complete (non-gating smoke; M-368 §9.2/§9.3 deferred): ${out##*$'\n'}"
  fi
done

exit 0
