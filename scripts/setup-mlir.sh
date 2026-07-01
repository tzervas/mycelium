#!/usr/bin/env bash
# Provision libMLIR (the `mlir-opt`/`mlir-translate` toolchain) version-matched to the installed
# LLVM, so the OPTIONAL `mlir-dialect` Cargo feature of `mycelium-mlir` (the real ternary →
# arith/vector → LLVM dialect lowering, M-601) can build and be tested. ADR-019 records this as a
# decision; this script makes it durable + idempotent. Wired into `just setup-mlir` — deliberately
# OPT-IN, kept OUT of the default `just setup` (an off-by-default feature must not apt-install/
# sudo-prompt by default; see the justfile's `setup-mlir` recipe comment).
#
# Honesty / house rules (CLAUDE.md):
#  - The LLVM major is DERIVED from the installed `llc` (then `clang`), never hard-coded — so the
#    MLIR packages are version-matched to the LLVM already present. NEVER a silent version bump
#    ("Don't silently bump committed version pins").
#  - No black box: every step echoes what it is doing (a provisioning script must be inspectable).
#  - Skip-gracefully: when a tool/package manager isn't present, print a clear message and `exit 0`
#    (advisory, never blocks) — mirroring scripts/install-tools.sh and the `llc`/`clang`
#    ToolchainMissing idiom in crates/mycelium-mlir/src/llvm.rs.
#  - SECURITY: NO `curl | bash`, no piping a download to a shell, no unpinned remote fetch. Only the
#    distro package manager — nala when present and functional, else apt-get (its universal
#    bootstrap/fallback) — with explicit, version-matched package names. All variable expansions
#    are quoted. (Read by /security-review.)
#
# The default Mycelium build and `cargo test` stay green WITHOUT libMLIR — the `mlir-dialect`
# feature is OFF by default and probes for the tools, skipping when absent (ADR-019). This script
# only matters for a contributor who wants that feature.
set -euo pipefail

# ── Reuse the house shell helpers when available (have/section/ok/skip), else define minimal
# fallbacks so the script is self-contained and runnable from anywhere. lib.sh lives beside this
# script under scripts/. CWD-independent resolution: `dirname` of a bare filename (e.g. when run as
# `bash setup-mlir.sh` from inside scripts/) yields `.`, and `cd ... && pwd` makes it absolute.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LIB="$SCRIPT_DIR/lib.sh"
if [ -f "$LIB" ]; then
  # shellcheck source=scripts/lib.sh
  source "$LIB"
else
  have()    { command -v "$1" >/dev/null 2>&1; }
  section() { printf -- '── %s ──\n' "$*"; }
  ok()      { printf -- '  ok    %s\n' "$*"; }
  skip()    { printf -- '  skip  %s\n' "$*"; }
fi

section "provision libMLIR (version-matched to the installed LLVM)"

# ── Step 1: detect the installed LLVM major version. Source of truth is the installed `llc`
# (then `clang` as a fallback) — the MLIR packages must match THIS major, never a hard-coded one.
detect_llvm_major() {
  local tool="$1" line major
  # The `--version` banner carries a line like "Ubuntu LLVM version 18.1.3" or "LLVM version 18.1.3"
  # (clang prints "clang version 18.1.3"). Grab the first dotted version and take its major integer.
  line="$("$tool" --version 2>/dev/null | grep -iE 'version[[:space:]]+[0-9]+' | head -n1 || true)"
  major="$(printf '%s\n' "$line" | grep -oE '[0-9]+(\.[0-9]+)+' | head -n1 | cut -d. -f1 || true)"
  printf '%s' "$major"
}

MAJOR=""
if have llc; then
  MAJOR="$(detect_llvm_major llc)"
  [ -n "$MAJOR" ] && ok "detected LLVM major $MAJOR from \`llc --version\`"
fi
if [ -z "$MAJOR" ] && have clang; then
  MAJOR="$(detect_llvm_major clang)"
  [ -n "$MAJOR" ] && ok "detected LLVM major $MAJOR from \`clang --version\`"
fi

# Fallback: no UNVERSIONED llc/clang on PATH (e.g. only version-suffixed `llc-NN`/`clang-NN` are
# installed — a distro/container without the Debian `llvm`/`clang` meta-packages, such as a bare
# apt.llvm.org install). Probe the highest version-suffixed binary directly so detection still
# succeeds; this only determines MAJOR for the libMLIR package match below — it does NOT make
# `llc`/`clang` resolve unversioned (that floor is `install-tools.sh`'s LLVM unversioned-binary step).
if [ -z "$MAJOR" ]; then
  for t in llc clang; do
    versioned="$(compgen -c "${t}-" 2>/dev/null | grep -E "^${t}-[0-9]+$" | sort -t- -k2 -n -r | head -n1 || true)"
    if [ -n "$versioned" ]; then
      MAJOR="${versioned##*-}"
      ok "detected LLVM major $MAJOR from version-suffixed \`$versioned\` (no unversioned $t on PATH)"
      break
    fi
  done
fi

if [ -z "$MAJOR" ]; then
  skip "no LLVM toolchain detected (neither \`llc\`/\`clang\` nor a version-suffixed \`llc-NN\`/\`clang-NN\` present, or version unparsable)"
  echo "  no LLVM toolchain detected; install LLVM first (e.g. \`bash scripts/install-tools.sh\`), then re-run \`bash scripts/setup-mlir.sh\`."
  exit 0
fi

# ── Step 2: idempotence — if the version-matched MLIR tools are already present, there is nothing
# to do. (In this repo's container they are: /usr/bin/mlir-opt-18 + /usr/bin/mlir-translate-18.)
if have "mlir-opt-$MAJOR" && have "mlir-translate-$MAJOR"; then
  ok "MLIR tools already present (mlir-opt-$MAJOR) — nothing to do."
  echo "  resolved: $(command -v "mlir-opt-$MAJOR")"
  exit 0
fi

# ── Step 3: install the version-matched packages via the distro package manager.
# On Debian/Ubuntu the packages are `libmlir-$MAJOR-dev` (provides libMLIR.so.$MAJOR) and
# `mlir-$MAJOR-tools` (provides mlir-opt-$MAJOR / mlir-translate-$MAJOR).
PKG_DEV="libmlir-$MAJOR-dev"
PKG_TOOLS="mlir-$MAJOR-tools"

if ! have apt-get; then
  skip "apt-get not found — cannot auto-install"
  echo "  apt-get not found; on this platform install $PKG_DEV + $PKG_TOOLS via your package manager,"
  echo "  then re-run \`bash scripts/setup-mlir.sh\`. (Packages must match the installed LLVM major $MAJOR.)"
  exit 0
fi

# Use sudo only when not already root (the container runs as root; CI may not).
SUDO=""
if [ "$(id -u)" -ne 0 ]; then
  if have sudo; then
    SUDO="sudo"
  else
    skip "not root and \`sudo\` absent — cannot run apt-get"
    echo "  run as root (or install sudo), then: apt-get install -y --no-install-recommends $PKG_DEV $PKG_TOOLS"
    exit 0
  fi
fi

# Prefer nala when it is present AND functional (some containers ship a nala whose apt_pkg/
# python3-apt binding is broken — ABI/path mismatch — so probe `nala --version`, not mere
# presence; mirrors install-tools.sh's nala_ok probe). apt-get is the universal fallback driver,
# both as the bootstrap for nala and when nala can't run.
PM="apt-get"
if have nala && nala --version >/dev/null 2>&1; then
  PM="nala"
fi

echo "  installing version-matched MLIR toolchain via $PM: $PKG_DEV $PKG_TOOLS"
echo "  + ${SUDO:+$SUDO }$PM install -y --no-install-recommends \"$PKG_DEV\" \"$PKG_TOOLS\""
if ${SUDO:+$SUDO} "$PM" install -y --no-install-recommends "$PKG_DEV" "$PKG_TOOLS"; then
  ok "$PM install succeeded"
elif [ "$PM" != "apt-get" ] \
     && ${SUDO:+$SUDO} apt-get install -y --no-install-recommends "$PKG_DEV" "$PKG_TOOLS"; then
  ok "apt-get install succeeded (fallback — the $PM driver failed)"
else
  skip "$PM could not install $PKG_DEV / $PKG_TOOLS (unavailable for LLVM major $MAJOR on this distro, or apt index stale)"
  echo "  ensure your apt sources provide LLVM $MAJOR (e.g. apt.llvm.org), then re-run; advisory, not blocking."
  echo "  manual fallback: ${SUDO:+$SUDO }apt-get install -y --no-install-recommends $PKG_DEV $PKG_TOOLS"
  exit 0
fi

# ── Step 4: re-probe and confirm the resolved path (or report the packages were unavailable).
if have "mlir-opt-$MAJOR"; then
  ok "MLIR toolchain provisioned: $(command -v "mlir-opt-$MAJOR")"
else
  skip "packages installed but mlir-opt-$MAJOR still not on PATH — check $PKG_TOOLS contents for this LLVM major"
fi

echo
ok "setup-mlir done — the \`mlir-dialect\` feature (M-601) can now build; default build/test are unaffected (ADR-019)"
exit 0
