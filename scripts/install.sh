#!/usr/bin/env bash
# Mycelium — one-command dev-environment installer. Idempotent, parameterized, never-silent.
#
#   bash scripts/install.sh                 # default components (rust python checks hooks)
#   bash scripts/install.sh --all           # everything EXCEPT mlir (mlir stays opt-in)
#   bash scripts/install.sh --rust --hooks  # only the named components
#   bash scripts/install.sh --mlir          # add the OFF-by-default libMLIR toolchain
#   bash scripts/install.sh --help          # full usage
#   MYC_INSTALL_COMPONENTS="rust hooks" bash scripts/install.sh   # via env (flags override)
#
# What it provisions (each component probes first and no-ops when already present):
#   rust    — the Rust toolchain pinned to the repo MSRV (rustup, if present). NEVER bumps the pin.
#   python  — `uv` + a CPython for the project (3.13/3.14 target; skip-gracefully on older).
#   checks  — `just`, the lint/check tools, gitleaks, supply-chain gates (reuses install-tools.sh).
#   hooks   — the pre-commit git hooks (so `just check`-equivalent runs on commit).
#   mlir    — OPT-IN ONLY (libMLIR for the off-by-default `mlir-dialect` feature; ADR-019). Never
#             part of `--all` or the defaults, because an off-by-default feature must not
#             apt-install/sudo-prompt by default.
#
# Honesty / house rules (CLAUDE.md):
#   - never-silent (G2): a failed REQUIRED step is reported and aborts with a clear message; an
#     OPTIONAL/best-effort step prints a skip line and continues (advisory, never blocks the rest).
#   - no black box: every step echoes what it is doing and why; selections are inspectable.
#   - SECURITY: NO `curl | bash`, no piping a remote download to a shell, no unpinned fetch. Only
#     rustup / uv / pre-commit / the distro package manager, with explicit names. All expansions
#     quoted. (Read by /security-review.)
#   - Don't silently bump committed version pins (MSRV, Python) — that's an ADR, not a build detail.
#
# Idempotent: safe to re-run. Every component is gap-fill only — a re-run on a fully-provisioned
# machine prints "present" for everything and changes nothing. Exit status is 0 on success or a
# clean skip; non-zero only when a REQUIRED, requested component genuinely fails.
set -euo pipefail

# ── House shell helpers (have/section/ok/skip/fail). lib.sh lives beside this script. ─────────────
LIB="${BASH_SOURCE%/*}/lib.sh"
if [[ -f "$LIB" ]]; then
  # shellcheck source=scripts/lib.sh
  source "$LIB"
else
  echo "fatal: scripts/lib.sh not found beside install.sh — run from a checkout" >&2
  exit 1
fi
cd "$REPO_ROOT" || exit 1
SCRIPT_DIR="$REPO_ROOT/scripts"

# Make uv's tool-bin and cargo's bin visible for this run (a fresh shell may not have re-sourced the
# profile yet). Mirrors install-tools.sh; harmless when already on PATH.
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$PATH"

# ── MSRV: single source of truth is the committed pin (rust-toolchain.toml, else workspace
# Cargo.toml `rust-version`). NEVER hard-code a version here — read it, so this script can never
# silently disagree with the pin (CLAUDE.md: don't bump committed pins). Empty ⇒ unknown (we then
# let rustup pick the repo's pinned toolchain itself rather than guessing).
detect_msrv() {
  local v=""
  if [[ -f "$REPO_ROOT/rust-toolchain.toml" ]]; then
    v="$(grep -oE 'channel[[:space:]]*=[[:space:]]*"[^"]+"' "$REPO_ROOT/rust-toolchain.toml" 2>/dev/null \
         | head -n1 | grep -oE '"[^"]+"' | tr -d '"' || true)"
  fi
  if [[ -z "$v" && -f "$REPO_ROOT/Cargo.toml" ]]; then
    v="$(grep -oE 'rust-version[[:space:]]*=[[:space:]]*"[^"]+"' "$REPO_ROOT/Cargo.toml" 2>/dev/null \
         | head -n1 | grep -oE '"[^"]+"' | tr -d '"' || true)"
  fi
  printf '%s' "$v"
}

# ── Component selection ───────────────────────────────────────────────────────────────────────────
# Defaults (no flags, no env): the everyday contributor set. `mlir` is intentionally excluded.
# `--all` enables the same set (every NON-opt-in component) directly in enable(); mlir stays opt-in.
DEFAULT_COMPONENTS="rust python checks hooks"

WANT_RUST=0 WANT_PYTHON=0 WANT_CHECKS=0 WANT_HOOKS=0 WANT_MLIR=0
EXPLICIT=0   # did the user name any component (flag or env)? then don't apply defaults.

enable() {
  case "$1" in
    rust)   WANT_RUST=1 ;;
    python) WANT_PYTHON=1 ;;
    checks) WANT_CHECKS=1 ;;
    hooks)  WANT_HOOKS=1 ;;
    mlir)   WANT_MLIR=1 ;;
    all)    WANT_RUST=1; WANT_PYTHON=1; WANT_CHECKS=1; WANT_HOOKS=1 ;;  # NOT mlir
    *)      fail "unknown component: '$1' (valid: rust python checks hooks mlir all)"; exit 2 ;;
  esac
}

usage() {
  cat <<'EOF'
Mycelium one-command dev-environment installer — idempotent, parameterized, never-silent.

USAGE
  bash scripts/install.sh [COMPONENT FLAGS]
  MYC_INSTALL_COMPONENTS="rust hooks" bash scripts/install.sh   # env form (flags override)

COMPONENTS (probe-before-act; each no-ops when already present)
  --rust      Rust toolchain pinned to the repo MSRV via rustup (never bumps the pin).
  --python    `uv` + a project CPython (3.13/3.14 target; skips gracefully on older).
  --checks    `just`, lint/check tools, gitleaks, supply-chain gates (runs scripts/install-tools.sh).
  --hooks     Install the pre-commit git hooks.
  --mlir      OPT-IN: libMLIR for the off-by-default `mlir-dialect` feature (ADR-019; may use apt/sudo).
  --all       Everything EXCEPT --mlir (mlir stays opt-in — an off-by-default feature must not
              apt-install by default).

OTHER
  -h, --help  Show this help and exit.

DEFAULT (no flags, no env)
  Installs: rust python checks hooks   (i.e. everything except the opt-in mlir).

NOTES
  - Idempotent: safe to re-run; a re-run on a provisioned machine prints "present" and changes nothing.
  - Optional tools that can't be installed → a clear skip line, not a hard failure (advisory).
  - No `curl | bash`: uses only rustup / uv / pre-commit / the distro package manager.
  - libMLIR can also be provisioned later with: just setup-mlir
EOF
}

# Env first (so explicit flags can override it). Whitespace/comma separated component words.
if [[ -n "${MYC_INSTALL_COMPONENTS:-}" ]]; then
  EXPLICIT=1
  # shellcheck disable=SC2086  # intentional word-splitting of the component list
  for c in ${MYC_INSTALL_COMPONENTS//,/ }; do enable "$c"; done
fi

# Flags override / extend.
while [[ $# -gt 0 ]]; do
  case "$1" in
    --rust)   EXPLICIT=1; enable rust ;;
    --python) EXPLICIT=1; enable python ;;
    --checks) EXPLICIT=1; enable checks ;;
    --hooks)  EXPLICIT=1; enable hooks ;;
    --mlir)   EXPLICIT=1; enable mlir ;;
    --all)    EXPLICIT=1; enable all ;;
    -h|--help) usage; exit 0 ;;
    --) shift; break ;;
    -*) fail "unknown flag: '$1' (try --help)"; exit 2 ;;
    *)  fail "unexpected argument: '$1' (try --help)"; exit 2 ;;
  esac
  shift
done

# No component named anywhere ⇒ apply the sensible defaults.
if [[ "$EXPLICIT" -eq 0 ]]; then
  # shellcheck disable=SC2086  # intentional word-splitting of the default list
  for c in $DEFAULT_COMPONENTS; do enable "$c"; done
fi

# Track REQUIRED-component failures so we can exit non-zero at the end with a clear summary,
# while still attempting every requested component (don't abort the whole run on the first miss).
FAILURES=()

# ── Component: Rust toolchain (pinned to MSRV; never bumps the pin) ────────────────────────────────
install_rust() {
  section "rust toolchain (MSRV-pinned)"
  local msrv; msrv="$(detect_msrv)"
  if [[ -n "$msrv" ]]; then ok "repo MSRV pin: $msrv"; else skip "MSRV pin not found in repo (rustup will use its default channel)"; fi

  if ! have rustup; then
    # cargo present without rustup ⇒ a system Rust; we won't touch it (and can't pin via rustup).
    if have cargo; then
      skip "rustup absent but cargo present ($(cargo --version 2>/dev/null || echo 'cargo')) — using existing Rust; cannot manage the MSRV pin without rustup"
    else
      skip "rustup not installed — install it from https://rustup.rs (no curl|bash here), then re-run \`bash scripts/install.sh --rust\`"
    fi
    return 0
  fi

  # rustup present: idempotently ensure the pinned toolchain. With a committed rust-toolchain.toml,
  # rustup auto-installs/selects it on first cargo use; `rustup show` triggers that resolution.
  # We still explicitly install the pinned channel so a fresh machine has it before first build.
  if [[ -n "$msrv" ]]; then
    if rustup toolchain list 2>/dev/null | grep -q -- "$msrv"; then
      ok "rustup: toolchain $msrv present"
    elif rustup toolchain install "$msrv" --profile minimal --component clippy,rustfmt >/dev/null 2>&1; then
      ok "rustup: installed $msrv (minimal + clippy + rustfmt)"
    else
      skip "rustup: could not install $msrv (offline, or channel unavailable) — re-run when online; \`just check\` will report a missing toolchain"
    fi
  else
    # No explicit pin we could parse — let rustup ensure *a* stable toolchain rather than guess a version.
    if rustup show active-toolchain >/dev/null 2>&1; then ok "rustup: an active toolchain is present"
    elif rustup toolchain install stable --profile minimal --component clippy,rustfmt >/dev/null 2>&1; then ok "rustup: installed stable (minimal + clippy + rustfmt)"
    else skip "rustup: could not install a toolchain (offline?)"; fi
  fi

  # Ensure the components `just check` needs (clippy/rustfmt) on whatever toolchain is selected.
  for comp in clippy rustfmt; do
    if rustup component list --installed 2>/dev/null | grep -q "^$comp"; then ok "rustup: component $comp present"
    elif rustup component add "$comp" >/dev/null 2>&1; then ok "rustup: added component $comp"
    else skip "rustup: could not add component $comp (\`just fmt\`/\`just lint\` may skip)"; fi
  done
}

# ── Component: Python via uv (project 3.13/3.14; skip-gracefully on older) ─────────────────────────
install_python() {
  section "python (uv + project CPython)"

  # 1) uv itself — install path is the official standalone installer's pip/uv-managed binary. We do
  #    NOT pipe the remote installer to a shell. If uv is absent we try pip (--user), else skip.
  if have uv; then
    ok "uv present ($(uv --version 2>/dev/null || echo uv))"
  elif have pipx; then
    if pipx install uv >/dev/null 2>&1; then ok "pipx: installed uv"; else skip "pipx: uv install failed"; fi
  elif have python3 && python3 -m pip --version >/dev/null 2>&1; then
    if python3 -m pip install --user --quiet uv >/dev/null 2>&1; then ok "pip --user: installed uv"
    else skip "pip: uv install failed — install uv from https://docs.astral.sh/uv (no curl|bash here)"; fi
  else
    skip "no uv/pipx/pip — install uv from https://docs.astral.sh/uv, then re-run \`bash scripts/install.sh --python\`"
  fi

  # 2) project Python. uv can fetch a managed CPython (3.13/3.14 target). If uv is present we let it
  #    provision/pin via uv; otherwise we only PROBE the system python and report honestly.
  if have uv; then
    # Prefer 3.14, then 3.13 (the project's target band). uv is idempotent: a present version is reused.
    local got=""
    for ver in 3.14 3.13; do
      if uv python find "$ver" >/dev/null 2>&1; then got="$ver"; ok "uv: CPython $ver available"; break; fi
    done
    if [[ -z "$got" ]]; then
      for ver in 3.14 3.13; do
        if uv python install "$ver" >/dev/null 2>&1; then got="$ver"; ok "uv: installed CPython $ver"; break; fi
      done
    fi
    if [[ -z "$got" ]]; then
      # Fall back to whatever python3 exists; warn if below the band (skip-gracefully on 3.11).
      if have python3; then
        local sysver; sysver="$(python3 -c 'import sys;print("%d.%d"%sys.version_info[:2])' 2>/dev/null || echo '?')"
        skip "uv could not provision 3.13/3.14 (offline?) — system python3 is $sysver; the project targets 3.13/3.14 (older skips gracefully)"
      else
        skip "uv could not provision a CPython and no system python3 found"
      fi
    fi
  elif have python3; then
    local sysver; sysver="$(python3 -c 'import sys;print("%d.%d"%sys.version_info[:2])' 2>/dev/null || echo '?')"
    ok "system python3 present ($sysver)"
    case "$sysver" in
      3.13|3.14) : ;;
      *) skip "python $sysver is outside the project target 3.13/3.14 — install uv to provision a matching CPython (older skips gracefully)";;
    esac
  else
    skip "no python available — install uv (provisions CPython) or a system python3"
  fi
}

# ── Component: check tools (delegates to the existing, battle-tested install-tools.sh) ────────────
install_checks() {
  section "check tools (just / pre-commit / lint / gitleaks / supply-chain)"
  local tools="$SCRIPT_DIR/install-tools.sh"
  if [[ ! -f "$tools" ]]; then
    fail "scripts/install-tools.sh not found — cannot install the check tools"
    FAILURES+=("checks"); return 0
  fi
  # install-tools.sh is itself idempotent and skip-graceful; reuse it rather than re-implement.
  if bash "$tools"; then
    ok "check tools step complete (see lines above for per-tool present/skip)"
  else
    # install-tools.sh is best-effort and exits 0 on graceful skips; a non-zero here is a real error.
    fail "install-tools.sh reported an error"
    FAILURES+=("checks")
  fi
}

# ── Component: pre-commit git hooks ───────────────────────────────────────────────────────────────
install_hooks() {
  section "pre-commit git hooks"
  if ! have pre-commit; then
    skip "pre-commit not installed yet — run the \`checks\` component first (it installs pre-commit), then \`bash scripts/install.sh --hooks\`"
    return 0
  fi
  if [[ ! -f "$REPO_ROOT/.pre-commit-config.yaml" ]]; then
    skip ".pre-commit-config.yaml not found — nothing to install"
    return 0
  fi
  # `pre-commit install` is idempotent (rewrites the hook shim). --install-hooks pre-fetches the hook
  # environments (incl. the gitleaks binary the gitleaks repo manages) so the first commit is fast.
  if pre-commit install --install-hooks >/dev/null 2>&1; then
    ok "pre-commit hooks installed (incl. gitleaks; hook envs pre-fetched)"
  else
    # Hook env pre-fetch can fail offline; the hook shim itself may still be installed. Don't hard-fail.
    if pre-commit install >/dev/null 2>&1; then
      skip "installed the hook shim, but could not pre-fetch hook environments (offline?) — they fetch on first run"
    else
      fail "pre-commit install failed"; FAILURES+=("hooks")
    fi
  fi
}

# ── Component: libMLIR (OPT-IN; delegates to setup-mlir.sh which is itself idempotent/skip-graceful)
install_mlir() {
  section "libMLIR (opt-in — off-by-default \`mlir-dialect\` feature; ADR-019)"
  local mlir="$SCRIPT_DIR/setup-mlir.sh"
  if [[ ! -f "$mlir" ]]; then
    skip "scripts/setup-mlir.sh not found — skipping the opt-in MLIR provisioning"
    return 0
  fi
  # setup-mlir.sh derives the LLVM major from the installed toolchain and exits 0 on any skip.
  if bash "$mlir"; then
    ok "mlir step complete (see lines above)"
  else
    # setup-mlir.sh is advisory (exit 0 on skips); a non-zero is unexpected but mlir is opt-in, so
    # surface it without failing the whole run.
    skip "setup-mlir.sh returned non-zero — MLIR is opt-in, continuing (build/test are unaffected: ADR-019)"
  fi
}

# ── Run requested components in a stable order (toolchain → language → tools → hooks → opt-in) ─────
echo
section "Mycelium install — components: $( \
  out=""; \
  [[ $WANT_RUST   -eq 1 ]] && out+="rust "; \
  [[ $WANT_PYTHON -eq 1 ]] && out+="python "; \
  [[ $WANT_CHECKS -eq 1 ]] && out+="checks "; \
  [[ $WANT_HOOKS  -eq 1 ]] && out+="hooks "; \
  [[ $WANT_MLIR   -eq 1 ]] && out+="mlir "; \
  printf '%s' "${out:-<none>}")"

[[ $WANT_RUST   -eq 1 ]] && install_rust
[[ $WANT_PYTHON -eq 1 ]] && install_python
[[ $WANT_CHECKS -eq 1 ]] && install_checks
[[ $WANT_HOOKS  -eq 1 ]] && install_hooks
[[ $WANT_MLIR   -eq 1 ]] && install_mlir

echo
if [[ ${#FAILURES[@]} -gt 0 ]]; then
  fail "install finished with errors in: ${FAILURES[*]}"
  echo "  Re-run the failed component(s) after resolving the cause; other components above are done."
  exit 1
fi
ok "install done — next: \`just check\` (run \`bash scripts/install.sh --mlir\` or \`just setup-mlir\` for the opt-in MLIR feature)"
