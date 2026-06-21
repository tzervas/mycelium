#!/usr/bin/env bash
# cargo-fuzz smoke for the Tier-2 durability gate (`just check-full`, DN-20). Runs ONE fuzz target
# for a short bounded time (default 60s) as a crash smoke — not an exhaustive campaign. Wraps the
# `just fuzz` recipe so the heavy tier exercises the fuzz path without the caller needing nightly
# wiring inline.
#
# Skip-graceful (house rule / G2-aware): cargo-fuzz needs the NIGHTLY toolchain + the cargo-fuzz
# subcommand + a libFuzzer-capable target. When any is absent we print a clear skip and exit 0 — a
# missing fuzz smoke is reduced durability coverage on this run, surfaced honestly, not a silent
# pass and not a hard build failure (the full mutants gate + the proptest suites still ran). On a
# release machine with the toolchain present, the smoke runs and a crash fails the run.
source "${BASH_SOURCE%/*}/../lib.sh"
cd "$REPO_ROOT" || exit 1
section "cargo-fuzz smoke (durability; Tier-2)"

SECS="${MYC_FUZZ_SECS:-60}"

if ! have cargo; then skip "cargo not found — fuzz smoke skipped"; exit 0; fi
if [[ ! -d fuzz/fuzz_targets ]]; then skip "no fuzz/fuzz_targets dir — nothing to smoke"; exit 0; fi
if ! have rustup; then skip "rustup not found — cannot select the nightly cargo-fuzz needs"; exit 0; fi
if ! rustup run nightly cargo --version >/dev/null 2>&1; then
  skip "nightly toolchain absent (\`rustup toolchain install nightly\`) — fuzz smoke skipped"; exit 0
fi
if ! cargo +nightly fuzz --version >/dev/null 2>&1; then
  skip "cargo-fuzz not installed (\`cargo install --locked cargo-fuzz\`) — fuzz smoke skipped"; exit 0
fi

# Pick the first target deterministically (sorted) so the smoke is reproducible.
target="$(cargo +nightly fuzz list 2>/dev/null | sort | head -n1)"
if [[ -z "$target" ]]; then skip "cargo-fuzz reports no targets — nothing to smoke"; exit 0; fi

ok "smoking target '$target' for ${SECS}s"
if cargo +nightly fuzz run "$target" -- -max_total_time="$SECS"; then
  ok "fuzz smoke clean (0 crashes) — $target"
  exit 0
else
  fail "fuzz smoke found a crash in '$target' — see the libFuzzer artifact above"
  exit 1
fi
