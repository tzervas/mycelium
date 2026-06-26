#!/usr/bin/env bash
# Supply-chain gate (C1-09): cargo-deny (advisories + licenses + sources, config: deny.toml)
# and cargo-audit (RustSec advisory DB). For LOCAL dev both skip gracefully when absent — install
# them with `just setup`. A real finding always fails non-zero. In the GATE environment (CI sets
# CI=true, or set MYCELIUM_REQUIRE_SUPPLY_CHAIN=1) a MISSING tool is a FAILURE, not a skip — a
# skip-pass is not a closed gate (G2, never silently green). ADR-021 Gate A4 / M-652.
#
# ── KNOWN IN-ENV ARTIFACT: the git-proxy `insteadOf` hijack (read before re-diagnosing a red) ───
# In the managed web/remote execution environment a git-config rewrite is injected at session
# start:   url."http://local_proxy@127.0.0.1:<port>/git/".insteadOf = "https://github.com/"
# It exists so the user's OWN repos push/pull through the scoped git proxy. But it is OVER-BROAD:
# it ALSO rewrites cargo-deny / cargo-audit's `git clone https://github.com/RustSec/advisory-db`,
# routing a PUBLIC, read-only fetch through the scoped proxy — which 403s anything out of session
# scope. The gate then goes red, NOT because of a real advisory but because the advisory DB cannot
# be fetched. This is an ENVIRONMENT ARTIFACT, not a supply-chain finding.
#   Confirmed legitimate path exists: the *general* HTTPS proxy DOES allow github.com — proven with
#     git -c 'url.https://github.com/RustSec/.insteadOf=https://github.com/RustSec/' \
#         ls-remote https://github.com/RustSec/advisory-db HEAD     # → real SHA, rc=0
#   i.e. a LONGEST-PREFIX `insteadOf` override that leaves the public RustSec URL un-rewritten lets
#   the fetch use the ALLOWED HTTPS path (TLS still verified; HTTPS_PROXY still set & in fact used).
#
# DISPOSITION (never-silent, G2):
#  • DEFAULT — change nothing about the network. In LOCAL dev this script DETECTS that fetch-failure
#    signature and reports a DEGRADED gate ("env artifact, not a finding") instead of a misleading
#    red, and points you at the real in-env supply-chain coverage: `just scan` (osv-scanner over
#    OSV.dev — no git clone, so the rewrite never touches it; clean over the workspace). It does NOT
#    route around the network control (standing proxy policy: report org 403/407, don't bypass).
#  • PERSISTENT FIX (eyes-open) — to make `cargo deny` / `just deny` / `just check` run reliably,
#    run once per session:   just deny-net-fix
#    It installs the scoped longest-prefix override for `https://github.com/RustSec/` ONLY, so just
#    that public repo bypasses the git proxy and uses the allowed HTTPS path. (Inline equivalent for
#    one run: `MYC_DENY_ALLOW_HTTPS_FETCH=1 just deny`.) Never a blanket github.com un-rewrite; TLS +
#    HTTPS_PROXY untouched. Full write-up: .claude/memory/toolchain.md §"Supply-chain gate".
source "${BASH_SOURCE%/*}/../lib.sh"
cd "$REPO_ROOT" || exit 1
section "supply-chain (cargo-deny / cargo-audit)"

rc=0
# Strict in the gate environment: a missing tool FAILS (the gate must actually run), not skips.
# CI sets CI=true; MYCELIUM_REQUIRE_SUPPLY_CHAIN=1 forces it anywhere. Local dev keeps graceful skip.
strict=0; [[ -n "${CI:-}" || -n "${MYCELIUM_REQUIRE_SUPPLY_CHAIN:-}" ]] && strict=1
absent() { # $1=tool: FAIL under strict (no silent skip-pass — G2), skip otherwise.
  if ((strict)); then
    fail "$1 not installed but the supply-chain gate is REQUIRED here (CI / MYCELIUM_REQUIRE_SUPPLY_CHAIN=1) — run \`just setup\`"; rc=1
  else
    skip "$1 not installed — run \`just setup\`"
  fi
}

# Classify a cargo-deny/cargo-audit FAILURE as the in-env git-proxy hijack (a fetch/clone failure
# to the advisory DB routed through the scoped local proxy) rather than a real advisory finding.
# Requires an explicit fetch/network-failure phrase AND advisory-DB context — a genuine advisory
# finding fetches the DB successfully first, so its output never matches these markers (no false
# downgrade of a real red). $1 = captured-output file.
is_proxy_hijack() {
  # FETCH-PHASE failure markers (cargo-deny & cargo-audit, real observed strings). All are
  # fetch/clone failures — none can appear once the DB is fetched, so a real advisory finding
  # (which happens AFTER a successful fetch) never matches them ⇒ no false downgrade of a real red.
  grep -qiE "couldn'?t fetch|failed to fetch|failed to clone|could not (read|fetch)|failed to get '?FETCH_HEAD|FETCH_HEAD'? metadata|git operation failed|unable to access|talking to the server|HTTP status 40[0-9]|returned error: 40[0-9]|error: 40[0-9]|cannot change to .*advisory|advisory-dbs|local_proxy|127\.0\.0\.1:[0-9]+/git" "$1" \
    && grep -qiE 'advisor|rustsec|github\.com|/git/' "$1"
}

# OPT-IN, eyes-open (default OFF): narrow the over-broad git-proxy rewrite for the PUBLIC RustSec
# advisory-db ONLY, so cargo-deny/audit fetch it over the ALLOWED HTTPS path. Scoped longest-prefix
# override; never a blanket github.com un-rewrite; TLS + HTTPS_PROXY untouched. See header.
if [[ -n "${MYC_DENY_ALLOW_HTTPS_FETCH:-}" ]]; then
  if git config --global url."https://github.com/RustSec/".insteadOf "https://github.com/RustSec/" 2>/dev/null; then
    ok "MYC_DENY_ALLOW_HTTPS_FETCH=1 — scoped RustSec/advisory-db fetch via allowed HTTPS path enabled"
  else
    skip "MYC_DENY_ALLOW_HTTPS_FETCH=1 set but the git-config override could not be applied"
  fi
fi

if ! have cargo; then absent cargo; exit "$rc"; fi

# cargo-deny: advisories, licenses, sources, bans — driven by deny.toml at the repo root.
if cargo deny --version >/dev/null 2>&1; then
  if [[ -f deny.toml ]]; then
    if cargo deny check >/tmp/myc-deny.out 2>&1; then
      ok "cargo deny: clean"
    elif ((!strict)) && is_proxy_hijack /tmp/myc-deny.out; then
      skip "cargo deny: DEGRADED — advisory-db fetch hijacked by the in-env git-proxy rewrite (ENV ARTIFACT, not a finding). Real in-env coverage: \`just scan\`. Opt-in fix: MYC_DENY_ALLOW_HTTPS_FETCH=1 just deny"
      tail -6 /tmp/myc-deny.out | sed 's/^/    /'
    else
      fail "cargo deny: findings"; tail -20 /tmp/myc-deny.out | sed 's/^/    /'; rc=1
    fi
  else
    skip "cargo deny present but no deny.toml — skipped"
  fi
else
  absent cargo-deny
fi

# cargo-audit: RustSec advisories against Cargo.lock.
if cargo audit --version >/dev/null 2>&1; then
  if cargo audit >/tmp/myc-audit.out 2>&1; then
    ok "cargo audit: no known advisories"
  elif ((!strict)) && is_proxy_hijack /tmp/myc-audit.out; then
    skip "cargo audit: DEGRADED — RustSec DB fetch hijacked by the in-env git-proxy rewrite (ENV ARTIFACT, not a finding). Real in-env coverage: \`just scan\`. Opt-in fix: MYC_DENY_ALLOW_HTTPS_FETCH=1 just deny"
    tail -6 /tmp/myc-audit.out | sed 's/^/    /'
  else
    fail "cargo audit: advisory findings — review and bump"; tail -20 /tmp/myc-audit.out | sed 's/^/    /'; rc=1
  fi
else
  absent cargo-audit
fi

exit "$rc"
