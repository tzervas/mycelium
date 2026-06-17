#!/data/data/com.termux/files/usr/bin/bash
# =============================================================================
# cc-termux-bootstrap.sh  (v2.1)
# Claude Code CLI on Android (Termux) — robust, upgrade-resilient setup.
#
# RUN IT LIKE THIS (do NOT 'source' it):
#       bash cc-termux-bootstrap.sh
#   or  ./cc-termux-bootstrap.sh
#
# v2.1 (captured in the Mycelium repo):
#   * LAUNCHER now defaults to 'claude' (was 'cc'). Naming the launcher 'cc'
#     OVERWROTE Termux's C-compiler symlink ($PREFIX/bin/cc -> clang), so every
#     native build (cargo build scripts, etc.) failed with
#       "Unknown command '.../symbols.o'. Try: cc help"
#     until clang was reinstalled. The launcher must NOT take a toolchain name.
#   * Added a guard that refuses LAUNCHER in {cc,c++,gcc,g++,clang,clang++,cpp,
#     ld,as,ar} so the footgun can't recur.
#   * The launcher's help text is now self-named (uses its own basename), so it
#     reads correctly whatever you call it.
#   Override the name with CC_LAUNCHER=... if you want something else (just not
#   a compiler/toolchain command). Use a shell `alias` for muscle memory.
#
# Idempotency & secrets:
#   * Safe to re-run: the container is reused if already enterable, the dev user
#     is created only if absent, packages/files are (re)asserted, and Claude is
#     installed only if missing. Re-running repairs drift; it never duplicates.
#   * NO secrets are stored in this script or the repo. Claude auth is interactive
#     at first run (browser OAuth or an API key you paste), kept in ~/.claude
#     inside the container. Nothing is committed.
#   * Optional dev-user password: default is passwordless sudo (it's a single-user
#     proot sandbox, not a privilege boundary). Set CC_SUDO_MODE=password to be
#     PROMPTED (no echo) for a password — it is piped over stdin into the
#     container, never placed in argv/env/disk/the repo.
#
# v2 fixes (vs v1):
#   * proot-distro v5 rewrite uses OCI images and MOVED the rootfs path
#     (containers/<name>/rootfs, not installed-rootfs/<name>). v1 checked the
#     old path and falsely reported "Rootfs install failed". v2 never depends
#     on the rootfs path at all.
#   * Install is verified by ENTERING the container (login probe), not by
#     stat-ing a file from outside the proot.
#   * Hook/config files are staged in Termux and BIND-MOUNTED into the
#     container, then copied into place from inside — path-independent.
#   * apt split into required (curl/ca-certificates) vs best-effort.
#
# Design recap: real glibc Ubuntu via proot-distro on INTERNAL storage so the
# official Claude installer + background auto-update work. TMPDIR is a real
# internal dir (the correct fix for hardcoded /tmp). SD card is bind-mounted
# for BULK, NON-EXECUTABLE data only (no exec/symlinks on exFAT, non-rooted).
# =============================================================================

set -euo pipefail

# ---- Tunables (override by exporting before running) -----------------------
DISTRO="${CC_DISTRO:-ubuntu}"        # OCI image / container name (v5: 'ubuntu' == ubuntu:latest)
DEV_USER="${CC_DEV_USER:-dev}"       # non-root user inside Ubuntu (avoids root-warning friction)
LAUNCHER="${CC_LAUNCHER:-claude}"    # the Termux command you'll type — NEVER a compiler name (see v2.1 note)
SD_GUEST="${CC_SD_GUEST:-/mnt/sdcard}"
WORK_GUEST="${CC_WORK_GUEST:-/home/$DEV_USER/work}"
TMP_GUEST="${CC_TMP_GUEST:-/home/$DEV_USER/.cache/cc-tmp}"
CC_PREFIX_GUEST="/opt/cc"
CC_SD_SRC="${CC_SD_SRC:-}"
CC_SUDO_MODE="${CC_SUDO_MODE:-nopasswd}"  # nopasswd (sandbox default) | password (prompts, never hardcoded)

PREFIX="${PREFIX:-/data/data/com.termux/files/usr}"
STAGING="$HOME/.cc-staging"          # Termux-side staging dir (bind-mounted into container)

# ---- Pretty logging --------------------------------------------------------
c_g=$'\033[1;32m'; c_y=$'\033[1;33m'; c_r=$'\033[1;31m'; c_b=$'\033[1;34m'; c_0=$'\033[0m'
log()  { printf '%s[*]%s %s\n' "$c_b" "$c_0" "$*"; }
ok()   { printf '%s[+]%s %s\n' "$c_g" "$c_0" "$*"; }
warn() { printf '%s[!]%s %s\n' "$c_y" "$c_0" "$*"; }
die()  { printf '%s[x]%s %s\n' "$c_r" "$c_0" "$*" >&2; exit 1; }

# Read a secret WITHOUT echo and WITHOUT it ever landing in argv/env/history.
# Usage: prompt_secret VAR "Prompt: "   (value is stored in $VAR)
prompt_secret() {
  local __var="$1" __prompt="$2" __val __confirm
  [ -t 0 ] || die "$__prompt needs an interactive terminal (no TTY); aborting rather than using a blank/hardcoded secret."
  printf '%s' "$__prompt" >&2; IFS= read -rs __val; printf '\n' >&2
  printf 'Confirm: ' >&2;       IFS= read -rs __confirm; printf '\n' >&2
  [ "$__val" = "$__confirm" ] || die "Entries did not match."
  [ -n "$__val" ] || die "Empty secret; aborting."
  printf -v "$__var" '%s' "$__val"
}

# Guard against being sourced (so a die() can't kill the user's shell).
if (return 0 2>/dev/null); then
  printf '%s[x]%s Do not source this script. Run:  bash %s\n' "$c_r" "$c_0" "${BASH_SOURCE[0]##*/}" >&2
  return 1
fi

# ---- 0. Sanity -------------------------------------------------------------
[ -d "$PREFIX" ] || die "Must run inside Termux ($PREFIX not found)."
command -v pkg >/dev/null 2>&1 || die "'pkg' not found — install Termux from F-Droid."
# Refuse a launcher name that would shadow the C/C++ toolchain. This is the v2.1
# fix: LAUNCHER='cc' previously clobbered $PREFIX/bin/cc (-> clang) and broke
# every native build until clang was reinstalled. Never name it a compiler.
case "$LAUNCHER" in
  cc|c++|gcc|g++|clang|clang++|cpp|ld|as|ar|cc1|cc1plus)
    die "CC_LAUNCHER='$LAUNCHER' would shadow a compiler/toolchain command on PATH and break native builds. Choose another (e.g. 'claude' or 'ccode'); use a shell alias for muscle memory." ;;
esac
log "Termux prefix: $PREFIX"
log "Target distro: $DISTRO   user: $DEV_USER   launcher: $LAUNCHER   sudo: $CC_SUDO_MODE"

# Optional: prompt for the dev-user password up-front (so a long provision doesn't
# block on input later). Held only in this process's memory; piped to the container
# over stdin after provisioning — never argv/env/disk/repo.
DEV_PW=""
case "$CC_SUDO_MODE" in
  nopasswd) ;;
  password) prompt_secret DEV_PW "Set a password for '$DEV_USER' (sudo will require it): " ;;
  *) die "CC_SUDO_MODE must be 'nopasswd' or 'password' (got '$CC_SUDO_MODE')." ;;
esac

# ---- 1. Termux packages ----------------------------------------------------
log "Updating Termux and installing base packages..."
yes | pkg update  >/dev/null 2>&1 || true
yes | pkg upgrade >/dev/null 2>&1 || true
pkg install -y proot-distro termux-api >/dev/null 2>&1 || pkg install -y proot-distro >/dev/null 2>&1
command -v proot-distro >/dev/null 2>&1 || die "proot-distro failed to install."
ok "proot-distro present: $(proot-distro --version 2>/dev/null | head -n1 || echo '(version unknown)')"

# ---- 2. Storage permission -------------------------------------------------
if [ ! -d "$HOME/storage/shared" ]; then
  warn "Storage not set up. Running termux-setup-storage — GRANT the Android dialog."
  termux-setup-storage || true
  log "Waiting up to 60s for storage permission..."
  for _ in $(seq 1 60); do [ -d "$HOME/storage/shared" ] && break; sleep 1; done
fi
[ -d "$HOME/storage/shared" ] && ok "Shared storage available." \
  || warn "Shared storage not granted — SD offload skipped (re-run later to add it)."

# ---- 3. Detect a writable SD source ---------------------------------------
SD_SRC=""
if [ -n "$CC_SD_SRC" ]; then SD_SRC="$CC_SD_SRC"
else SD_SRC="$(ls -d "$HOME"/storage/external-* 2>/dev/null | head -n1 || true)"; fi
if [ -n "$SD_SRC" ] && [ -d "$SD_SRC" ]; then
  mkdir -p "$SD_SRC/cc-data" 2>/dev/null || true
  if ( : > "$SD_SRC/cc-data/.w" ) 2>/dev/null; then
    rm -f "$SD_SRC/cc-data/.w"; SD_SRC="$SD_SRC/cc-data"
    ok "SD card writable: $SD_SRC  ->  $SD_GUEST (inside Ubuntu)"
  else warn "Found $SD_SRC but not writable. Skipping SD offload."; SD_SRC=""; fi
else
  warn "No external SD detected (~/storage/external-*). Skipping SD offload."
  warn "  Set CC_SD_SRC=/path and re-run if your card mounts elsewhere."
  SD_SRC=""
fi

# ---- Helper: is the container installed AND enterable? ---------------------
distro_ready() { proot-distro login "$DISTRO" -- true >/dev/null 2>&1; }

# ---- 4. Ensure Ubuntu container exists (idempotent, path-independent) ------
if distro_ready; then
  ok "Ubuntu container already installed and enterable — reusing."
else
  log "Installing Ubuntu (OCI image)..."
  proot-distro install "$DISTRO" || warn "install returned non-zero (may already exist) — verifying..."
  distro_ready || die "Ubuntu container is not enterable after install. Try: proot-distro login $DISTRO -- true"
  ok "Ubuntu container ready."
fi

# ---- 5. Stage config + hooks in Termux (NOT in the rootfs) ----------------
log "Staging config, hooks, and provisioner..."
rm -rf "$STAGING"; mkdir -p "$STAGING/hooks"

cat > "$STAGING/cc.conf" <<EOF_CONF
# guest-side config (sourced by everything inside the container)
CC_DEV_USER="$DEV_USER"
CC_SD_GUEST="$SD_GUEST"
CC_WORK_GUEST="$WORK_GUEST"
CC_TMP_GUEST="$TMP_GUEST"
CC_PREFIX="$CC_PREFIX_GUEST"
CC_SUDO_MODE="$CC_SUDO_MODE"
EOF_CONF

cat > "$STAGING/cc-env.sh" <<'EOF_ENV'
# Claude Code environment (Ubuntu/proot). Sourced by login shells.
. /opt/cc/cc.conf 2>/dev/null || true
export PATH="$HOME/.local/bin:$PATH"
export TMPDIR="${CC_TMP_GUEST:-$HOME/.cache/cc-tmp}"   # real, internal, exec-capable
mkdir -p "$TMPDIR" 2>/dev/null || true
EOF_ENV

cat > "$STAGING/hooks/validate.sh" <<'EOF_VALIDATE'
#!/usr/bin/env bash
set -uo pipefail
. /opt/cc/cc.conf 2>/dev/null || true
. /etc/profile.d/cc-env.sh 2>/dev/null || true
fail=0
chk(){ if eval "$2" >/dev/null 2>&1; then echo "  [ok] $1"; else echo "  [XX] $1"; fail=1; fi; }
echo "Claude Code environment check:"
chk "claude on PATH"       "command -v claude"
chk "TMPDIR set"           "[ -n \"\$TMPDIR\" ]"
chk "TMPDIR writable"      "touch \"\$TMPDIR/.w\" && rm -f \"\$TMPDIR/.w\""
chk "/tmp writable"        "touch /tmp/.w && rm -f /tmp/.w"
chk "workspace exists"     "[ -d \"\$CC_WORK_GUEST\" ] || mkdir -p \"\$CC_WORK_GUEST\""
if [ -d "$CC_SD_GUEST" ]; then chk "SD mount reachable" "[ -d \"\$CC_SD_GUEST\" ]"
else echo "  [--] SD not bound this session (ok if not using it)"; fi
command -v claude >/dev/null 2>&1 && echo "  Claude version: $(claude --version 2>/dev/null | head -n1)"
exit "$fail"
EOF_VALIDATE

cat > "$STAGING/hooks/post-install.sh" <<'EOF_INST'
#!/usr/bin/env bash
set -uo pipefail
. /opt/cc/cc.conf 2>/dev/null || true
. /etc/profile.d/cc-env.sh 2>/dev/null || true
mkdir -p "$TMPDIR" "$CC_WORK_GUEST" "$HOME/.local/bin"
if ! command -v claude >/dev/null 2>&1; then
  echo "[post-install] installing Claude Code via official installer..."
  curl -fsSL https://claude.ai/install.sh | bash
fi
hash -r 2>/dev/null || true
v="$(command -v claude >/dev/null 2>&1 && claude --version 2>/dev/null | head -n1 || echo none)"
echo "$v" > "$CC_PREFIX/state/version.good"
echo "[post-install] Claude version: $v"
EOF_INST

cat > "$STAGING/hooks/pre-upgrade.sh" <<'EOF_PRE'
#!/usr/bin/env bash
set -uo pipefail
. /opt/cc/cc.conf 2>/dev/null || true
. /etc/profile.d/cc-env.sh 2>/dev/null || true
ts="$(date +%Y%m%d-%H%M%S)"
cur="$(command -v claude >/dev/null 2>&1 && claude --version 2>/dev/null | head -n1 || echo none)"
echo "$cur" > "$CC_PREFIX/state/version.pre"
if [ -d "$HOME/.claude" ]; then
  tar czf "$CC_PREFIX/backups/claude-config-$ts.tgz" -C "$HOME" .claude 2>/dev/null \
    && echo "[pre-upgrade] backed up ~/.claude"
  ls -1t "$CC_PREFIX"/backups/claude-config-*.tgz 2>/dev/null | tail -n +6 | xargs -r rm -f
fi
echo "[pre-upgrade] version before: $cur"
EOF_PRE

cat > "$STAGING/hooks/post-upgrade.sh" <<'EOF_POST'
#!/usr/bin/env bash
set -uo pipefail
. /opt/cc/cc.conf 2>/dev/null || true
. /etc/profile.d/cc-env.sh 2>/dev/null || true
mkdir -p "$TMPDIR" "$CC_WORK_GUEST" "$CC_SD_GUEST" 2>/dev/null || true
if /opt/cc/hooks/validate.sh; then
  new="$(command -v claude >/dev/null 2>&1 && claude --version 2>/dev/null | head -n1 || echo none)"
  echo "$new" > "$CC_PREFIX/state/version.good"
  echo "[post-upgrade] validated. last-known-good = $new"
else
  echo "[post-upgrade] VALIDATION FAILED — reinstalling via official installer..."
  curl -fsSL https://claude.ai/install.sh | bash || true
  /opt/cc/hooks/validate.sh || echo "[post-upgrade] still failing — run '$LAUNCHER doctor'."
fi
EOF_POST

cat > "$STAGING/run.sh" <<'EOF_RUN'
#!/usr/bin/env bash
set -uo pipefail
. /opt/cc/cc.conf 2>/dev/null || true
. /etc/profile.d/cc-env.sh 2>/dev/null || true
mkdir -p "$TMPDIR" 2>/dev/null || true
good="$(cat "$CC_PREFIX/state/version.good" 2>/dev/null || echo none)"
cur="$(command -v claude >/dev/null 2>&1 && claude --version 2>/dev/null | head -n1 || echo none)"
if [ "$cur" != "$good" ]; then
  echo "[cc] version drift: '$good' -> '$cur' — re-asserting environment..."
  /opt/cc/hooks/pre-upgrade.sh  >/dev/null 2>&1 || true
  /opt/cc/hooks/post-upgrade.sh || true
fi
TARGET="${CC_TARGET:-$CC_WORK_GUEST}"
mkdir -p "$TARGET" 2>/dev/null || true
cd "$TARGET" || cd "$HOME"
exec claude
EOF_RUN

cat > "$STAGING/update.sh" <<'EOF_UPD'
#!/usr/bin/env bash
set -uo pipefail
. /opt/cc/cc.conf 2>/dev/null || true
. /etc/profile.d/cc-env.sh 2>/dev/null || true
/opt/cc/hooks/pre-upgrade.sh || true
echo "[cc] updating Claude Code..."
if command -v claude >/dev/null 2>&1 && claude update >/dev/null 2>&1; then
  echo "[cc] 'claude update' completed."
else
  echo "[cc] falling back to official installer..."
  curl -fsSL https://claude.ai/install.sh | bash || true
fi
hash -r 2>/dev/null || true
/opt/cc/hooks/post-upgrade.sh || true
EOF_UPD

# Provisioner: runs INSIDE the container (as root). Installs our files from the
# bind-mounted staging dir, sets up the user, apt deps, and installs Claude.
cat > "$STAGING/provision.sh" <<'EOF_PROV'
#!/usr/bin/env bash
set -euo pipefail
STG=/opt/cc-staging
mkdir -p /opt/cc/hooks /opt/cc/state /opt/cc/backups /etc/profile.d
cp "$STG/cc.conf"      /opt/cc/cc.conf
cp "$STG/run.sh"       /opt/cc/run.sh
cp "$STG/update.sh"    /opt/cc/update.sh
cp "$STG"/hooks/*.sh   /opt/cc/hooks/
cp "$STG/cc-env.sh"    /etc/profile.d/cc-env.sh
chmod +x /opt/cc/*.sh /opt/cc/hooks/*.sh
. /opt/cc/cc.conf

export DEBIAN_FRONTEND=noninteractive
echo "[provision] apt update..."
apt-get update -y -qq || { echo "[provision] apt update failed (DNS/network?)"; exit 1; }
echo "[provision] installing required packages (curl, ca-certificates)..."
apt-get install -y -qq --no-install-recommends ca-certificates curl \
  || { echo "[provision] required apt packages failed"; exit 1; }
update-ca-certificates >/dev/null 2>&1 || true
echo "[provision] installing best-effort packages..."
apt-get install -y -qq --no-install-recommends git sudo ripgrep less procps nano xz-utils || true

if ! id -u "$CC_DEV_USER" >/dev/null 2>&1; then
  echo "[provision] creating user '$CC_DEV_USER'..."
  useradd -m -s /bin/bash "$CC_DEV_USER"
fi
# Idempotent: a single overwrite. NOPASSWD by default (proot sandbox); password
# mode requires the password we set post-provision (never written here).
if [ "${CC_SUDO_MODE:-nopasswd}" = "password" ]; then
  echo "$CC_DEV_USER ALL=(ALL) ALL"          > /etc/sudoers.d/cc-$CC_DEV_USER 2>/dev/null || true
else
  echo "$CC_DEV_USER ALL=(ALL) NOPASSWD: ALL" > /etc/sudoers.d/cc-$CC_DEV_USER 2>/dev/null || true
fi
chmod 0440 /etc/sudoers.d/cc-$CC_DEV_USER 2>/dev/null || true

mkdir -p "$CC_SD_GUEST" "$CC_WORK_GUEST" "$CC_TMP_GUEST"
chown -R "$CC_DEV_USER:$CC_DEV_USER" "$CC_WORK_GUEST" "$CC_TMP_GUEST" "/home/$CC_DEV_USER" \
        /opt/cc/state /opt/cc/backups 2>/dev/null || true

echo "[provision] installing Claude Code as '$CC_DEV_USER'..."
su - "$CC_DEV_USER" -c 'bash -lc /opt/cc/hooks/post-install.sh'
echo "[provision] done."
EOF_PROV

ok "Staged in $STAGING."

# ---- 6. Provision inside the container ------------------------------------
log "Provisioning inside Ubuntu (this binds the staging dir, installs Claude)..."
BINDS=(--bind "$STAGING:/opt/cc-staging")
[ -n "$SD_SRC" ] && BINDS+=(--bind "$SD_SRC:$SD_GUEST")
proot-distro login "$DISTRO" "${BINDS[@]}" -- bash /opt/cc-staging/provision.sh \
  || die "Provisioning failed (network? re-run, or check Termux network settings)."
ok "Provisioning complete."

# Set the dev-user password (password mode only). The secret is piped over STDIN
# into the container and consumed by chpasswd — it never appears in argv, the
# environment, on disk, or in the repo. Idempotent: re-running just re-sets it.
if [ "$CC_SUDO_MODE" = "password" ] && [ -n "$DEV_PW" ]; then
  log "Setting password for '$DEV_USER' (over stdin; not logged)..."
  printf '%s\n' "$DEV_PW" | proot-distro login "$DISTRO" -- \
      env TARGET_USER="$DEV_USER" bash -c 'IFS= read -r pw; printf "%s:%s\n" "$TARGET_USER" "$pw" | chpasswd' \
    && ok "Password set for '$DEV_USER'." || warn "Could not set password — sudo will still prompt; set it later with: passwd $DEV_USER"
  unset DEV_PW
fi

# ---- 7. Install the Termux-side launcher ----------------------------------
log "Installing the '$LAUNCHER' launcher..."
cat > "$PREFIX/etc/cc.conf" <<EOF_TCONF
CC_DISTRO="$DISTRO"
CC_DEV_USER="$DEV_USER"
CC_SD_GUEST="$SD_GUEST"
CC_WORK_GUEST="$WORK_GUEST"
EOF_TCONF

cat > "$PREFIX/bin/$LAUNCHER" <<'EOF_LAUNCH'
#!/data/data/com.termux/files/usr/bin/bash
# Claude Code launcher (Termux side). Single entrypoint; re-asserts environment
# every launch so background auto-updates can't break the setup.
set -uo pipefail
self="$(basename "$0")"      # self-named help/errors, whatever you called it
PREFIX="${PREFIX:-/data/data/com.termux/files/usr}"
. "$PREFIX/etc/cc.conf" 2>/dev/null || true
DISTRO="${CC_DISTRO:-ubuntu}"; DEV_USER="${CC_DEV_USER:-dev}"
SD_GUEST="${CC_SD_GUEST:-/mnt/sdcard}"; WORK_GUEST="${CC_WORK_GUEST:-/home/$DEV_USER/work}"

sd_src="${CC_SD_SRC:-}"
if [ -z "$sd_src" ]; then
  base="$(ls -d "$HOME"/storage/external-* 2>/dev/null | head -n1 || true)"
  [ -n "$base" ] && [ -d "$base/cc-data" ] && sd_src="$base/cc-data"
fi
bind=(); [ -n "$sd_src" ] && [ -d "$sd_src" ] && bind=(--bind "$sd_src:$SD_GUEST")

have_wl=0
command -v termux-wake-lock >/dev/null 2>&1 && { termux-wake-lock 2>/dev/null && have_wl=1; }
cleanup(){ [ "$have_wl" = 1 ] && termux-wake-unlock 2>/dev/null || true; }
trap cleanup EXIT

enter(){ proot-distro login "$DISTRO" --user "$DEV_USER" "${bind[@]}" -- bash -lc "$1"; }

case "${1:-launch}" in
  launch|run|"") enter "CC_TARGET='$WORK_GUEST' /opt/cc/run.sh" ;;
  work) enter "CC_TARGET='${2:-$WORK_GUEST}' /opt/cc/run.sh" ;;
  sd)
    [ -n "$sd_src" ] || { echo "No SD card bound. See '$self doctor'."; exit 1; }
    echo "NOTE: SD workspace has no symlinks/exec bits — complex git repos may misbehave."
    enter "CC_TARGET='$SD_GUEST/workspace' /opt/cc/run.sh" ;;
  update)  enter "/opt/cc/update.sh" ;;
  doctor)  enter "/opt/cc/hooks/validate.sh" ;;
  shell)   proot-distro login "$DISTRO" --user "$DEV_USER" "${bind[@]}" ;;
  help|-h|--help)
    cat <<H
$self                 Launch Claude Code in default workspace ($WORK_GUEST)
$self work <path>     Launch in a path inside Ubuntu (internal storage)
$self sd              Launch with workspace on the SD card (bulk data only)
$self update          Controlled update (pre -> update -> post hooks)
$self doctor          Validate the environment
$self shell           Ubuntu shell (env applied)
SD bound at: $SD_GUEST   (source: ${sd_src:-<none>})
H
    ;;
  *) echo "Unknown command '$1'. Try: $self help"; exit 1 ;;
esac
EOF_LAUNCH
chmod +x "$PREFIX/bin/$LAUNCHER"
ok "Launcher installed: $LAUNCHER"

# ---- 8. Final validation ---------------------------------------------------
log "Running environment check..."
proot-distro login "$DISTRO" --user "$DEV_USER" -- bash -lc '/opt/cc/hooks/validate.sh' \
  || warn "Validation reported issues — run '$LAUNCHER doctor' for detail."

cat <<DONE

${c_g}=============================================================${c_0}
 Setup complete.

 First run (authenticate):   ${c_b}$LAUNCHER${c_0}
   -> browser OAuth (Pro/Max) or paste an API key.

 Commands:  ${c_b}$LAUNCHER doctor${c_0} | ${c_b}$LAUNCHER update${c_0} | ${c_b}$LAUNCHER shell${c_0} | ${c_b}$LAUNCHER help${c_0}

 Layout:
   rootfs + binary + TMPDIR : INTERNAL (required, exec-capable)
   SD card (bulk data only) : ${SD_SRC:-<none>}  ->  $SD_GUEST

 Long runs: ${c_b}termux-wake-lock${c_0} and/or run inside ${c_b}tmux${c_0}.
 Muscle memory: if you liked typing 'cc', add a shell alias (NOT a file):
   ${c_b}echo 'alias cc=$LAUNCHER' >> ~/.bashrc${c_0}   # alias only; never a \$PREFIX/bin/cc file
${c_g}=============================================================${c_0}
DONE
