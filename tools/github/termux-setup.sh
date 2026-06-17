#!/usr/bin/env bash
# Mycelium — Termux (Android) one-shot PM bootstrap: provision -> auth -> run the bootstrap.
#
# WHAT THIS IS
# ------------
# A single, ordered, re-runnable script to drive the whole GitHub project-management
# bootstrap from a phone (Termux), with nothing pre-configured. It:
#   1. installs the needed packages from the package manager (pkg/apt — no `curl | bash`);
#   2. sets your git identity (prompt or env);
#   3. generates a GPG signing key (or reuses one), wires commit signing, uploads the
#      PUBLIC key to GitHub;
#   4. authenticates `gh` (browser/device OAuth, or a token you supply) and wires the git
#      credential helper;
#   5. clones / locates the repo;
#   6. runs gh-bootstrap-local.sh (labels + milestones) then gh-issues-sync.py (issues +
#      milestones + idmap) — the full bootstrap, idempotent end to end.
#
# SECURITY POSTURE (house rule: never-silent, no black boxes, KISS)
# ----------------------------------------------------------------
#   * No secrets are embedded or written to the repo. Your GPG *private* key never leaves
#     the device; only the PUBLIC key is uploaded.
#   * The GPG key is passphrase-protected via pinentry by default (override with
#     --no-gpg-passphrase, which prints a warning).
#   * The GitHub token is held by `gh` in its own config (not echoed, not committed). The
#     git remote uses the credential helper, not a token-in-URL.
#   * Every step is idempotent and prints what it does; re-running is safe.
#
# USAGE
#   bash tools/github/termux-setup.sh                 # interactive (recommended)
#   GIT_USER_NAME="Tyler Zervas" GIT_USER_EMAIL=you@example.com \
#     bash tools/github/termux-setup.sh
#   GH_TOKEN=*** bash tools/github/termux-setup.sh    # non-interactive gh auth
#   bash tools/github/termux-setup.sh --dry-run-issues --skip-install
#
# FLAGS
#   --repo <owner/name>     target repo (default: tzervas/mycelium)
#   --repo-dir <path>       checkout location (default: $HOME/mycelium, or the cwd if it is
#                           already this repo)
#   --no-gpg-passphrase     generate the GPG key WITHOUT a passphrase (less secure; warned)
#   --skip-install          skip the package-install step
#   --skip-gpg              skip GPG key generation / signing setup
#   --skip-issues           run labels+milestones only (no issue create/sync)
#   --dry-run-issues        show which issues WOULD be created, create nothing
#   -h, --help              this help
set -euo pipefail

# ---- tiny logger (color only on a tty) -------------------------------------------------
if [[ -t 1 ]]; then C_G=$'\033[32m'; C_Y=$'\033[33m'; C_R=$'\033[31m'; C_D=$'\033[2m'; C_0=$'\033[0m'
else C_G=''; C_Y=''; C_R=''; C_D=''; C_0=''; fi
step() { printf '\n%s== %s ==%s\n' "$C_D" "$*" "$C_0"; }
ok()   { printf '  %sok%s   %s\n' "$C_G" "$C_0" "$*"; }
warn() { printf '  %swarn%s %s\n' "$C_Y" "$C_0" "$*"; }
die()  { printf '  %sERROR%s %s\n' "$C_R" "$C_0" "$*" >&2; exit 1; }
have() { command -v "$1" >/dev/null 2>&1; }

# ---- args ------------------------------------------------------------------------------
REPO="tzervas/mycelium"
REPO_DIR="${REPO_DIR:-$HOME/mycelium}"
GPG_PASSPHRASE=1
DO_INSTALL=1
DO_GPG=1
DO_ISSUES=1
DRY_ISSUES=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo) REPO="$2"; shift 2 ;;
    --repo-dir) REPO_DIR="$2"; shift 2 ;;
    --no-gpg-passphrase) GPG_PASSPHRASE=0; shift ;;
    --skip-install) DO_INSTALL=0; shift ;;
    --skip-gpg) DO_GPG=0; shift ;;
    --skip-issues) DO_ISSUES=0; shift ;;
    --dry-run-issues) DRY_ISSUES=1; shift ;;
    -h|--help) sed -n '2,40p' "$0"; exit 0 ;;
    *) die "unknown flag: $1 (try --help)" ;;
  esac
done

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# ---- package manager (pkg on Termux, else apt-get) -------------------------------------
pm_install() {
  local pkgs=("$@")
  if have pkg; then
    pkg install -y "${pkgs[@]}"
  elif have apt-get; then
    local sudo=""; [[ $EUID -ne 0 ]] && have sudo && sudo="sudo"
    $sudo apt-get update -y
    $sudo apt-get install -y "${pkgs[@]}"
  else
    die "no 'pkg' or 'apt-get' found — install git gh gnupg jq python manually, then --skip-install"
  fi
}

step "0. environment"
if [[ -n "${PREFIX:-}" && "${PREFIX}" == *com.termux* ]]; then ok "Termux detected ($PREFIX)"
else warn "not Termux — continuing (pkg/apt path still applies)"; fi

# ---- 1. packages -----------------------------------------------------------------------
step "1. packages"
# Idempotent: only install when something is actually missing (re-running is a clean no-op).
missing=()
for tool in git gh gpg jq python; do have "$tool" || missing+=("$tool"); done
if [[ $DO_INSTALL -eq 0 ]]; then
  warn "skipped (per --skip-install)"
elif [[ ${#missing[@]} -eq 0 ]]; then
  ok "git gh gpg jq python already installed — nothing to do (idempotent)"
else
  warn "installing missing: ${missing[*]}"
  if have pkg; then pkg update -y || warn "pkg update had warnings"; fi
  pm_install git gh gnupg jq python openssh   # package-manager installs are themselves idempotent
  ok "packages installed"
fi
# Resolve a python interpreter UP FRONT (python3 preferred; Termux often has bare `python`) so a
# missing one is an explicit prerequisite error here — not a confusing late failure inside the
# delegated python tools (git-signing-sync.py / gh-issues-sync.py), even under --skip-install.
PY=""
for cand in python3 python; do have "$cand" && { PY="$cand"; break; }; done
# pyyaml only when not already importable (pip is a no-op when satisfied).
if [[ -n "$PY" ]]; then
  if "$PY" -c "import yaml" >/dev/null 2>&1; then ok "pyyaml ready"
  elif "$PY" -m pip install --quiet --upgrade pip pyyaml 2>/dev/null; then ok "pyyaml installed"
  else warn "pyyaml install failed — issue sync may skip (install: pip install pyyaml)"; fi
fi
for tool in git gh gpg jq; do have "$tool" || die "$tool missing — drop --skip-install or install it"; done
[[ -n "$PY" ]] || die "python3/python missing — install it (or drop --skip-install)"

# ---- 2. git identity -------------------------------------------------------------------
step "2. git identity"
GIT_USER_NAME="${GIT_USER_NAME:-$(git config --global user.name || true)}"
GIT_USER_EMAIL="${GIT_USER_EMAIL:-$(git config --global user.email || true)}"
if [[ -z "$GIT_USER_NAME" ]]; then read -rp "  git user.name : " GIT_USER_NAME; fi
if [[ -z "$GIT_USER_EMAIL" ]]; then read -rp "  git user.email: " GIT_USER_EMAIL; fi
[[ -n "$GIT_USER_NAME" && -n "$GIT_USER_EMAIL" ]] || die "git identity required"
git config --global user.name "$GIT_USER_NAME"
git config --global user.email "$GIT_USER_EMAIL"
git config --global init.defaultBranch main
ok "identity: $GIT_USER_NAME <$GIT_USER_EMAIL>"

# ---- 3. GPG signing key ----------------------------------------------------------------
step "3. GPG signing key"
# Delegated to the portable, idempotent reconciler (git-signing-sync.py) — identical logic on
# Linux + Windows. It REUSES an existing key and generates one only when absent (this --setup is
# the explicit trigger; pass --new-key to rotate). Git config is set create-if-absent / on-drift.
if [[ $DO_GPG -eq 1 ]]; then
  GPG_TTY="$(tty || true)"; export GPG_TTY
  setup_args=(--setup --name "$GIT_USER_NAME" --email "$GIT_USER_EMAIL")
  [[ $GPG_PASSPHRASE -eq 0 ]] && setup_args+=(--no-passphrase)
  [[ -t 0 ]] || setup_args+=(--no-prompt)   # non-interactive: use the provided name/email, no prompts
  if "$PY" "$HERE/git-signing-sync.py" "${setup_args[@]}"; then
    ok "commit signing configured (idempotent; existing key reused)"
  else
    die "signing setup failed (run: python $HERE/git-signing-sync.py to diagnose)"
  fi
else
  warn "skipped (per --skip-gpg)"
fi

# ---- 4. GitHub auth --------------------------------------------------------------------
step "4. GitHub auth"
if gh auth status >/dev/null 2>&1; then
  ok "gh already authenticated"
elif [[ -n "${GH_TOKEN:-}" ]]; then
  printf '%s' "$GH_TOKEN" | gh auth login --hostname github.com --git-protocol https --with-token
  ok "authenticated from GH_TOKEN"
else
  printf '  opening GitHub OAuth (browser/device code). Approve scopes: repo, admin:gpg_key.\n'
  gh auth login --hostname github.com --git-protocol https --web --scopes "repo,read:org,admin:gpg_key"
  ok "authenticated"
fi
gh auth setup-git
ok "git credential helper wired to gh"

# ---- 5. upload the PUBLIC GPG key ------------------------------------------------------
# Step 3 wired user.signingkey; publish its PUBLIC half now that gh is authed (skip for SSH signing).
keyid="$(git config --global user.signingkey 2>/dev/null || true)"
gpgfmt="$(git config --global gpg.format 2>/dev/null || echo openpgp)"
if [[ $DO_GPG -eq 1 && -n "$keyid" && "$gpgfmt" != "ssh" ]]; then
  step "5. publish the public GPG key"
  if gpg --armor --export "$keyid" | gh gpg-key add - 2>/dev/null; then ok "public key uploaded to GitHub"
  else warn "gpg key not uploaded (already present, or scope admin:gpg_key not granted)"; fi
fi

# ---- 6. locate / clone the repo --------------------------------------------------------
step "6. repository"
if git -C "$PWD" rev-parse --show-toplevel >/dev/null 2>&1 \
   && git -C "$PWD" remote get-url origin 2>/dev/null | grep -q "$REPO"; then
  REPO_DIR="$(git -C "$PWD" rev-parse --show-toplevel)"
  ok "using current checkout: $REPO_DIR"
elif [[ -d "$REPO_DIR/.git" ]]; then
  ok "using existing checkout: $REPO_DIR"
else
  git clone "https://github.com/$REPO.git" "$REPO_DIR"
  ok "cloned into $REPO_DIR"
fi
cd "$REPO_DIR"

# ---- 7. run the bootstrap --------------------------------------------------------------
step "7. labels + milestones (gh-bootstrap-local.sh)"
MSMAP="$(mktemp -t mycelium-msmap.XXXXXX.tsv)"
export MSMAP REPO
bash "$HERE/gh-bootstrap-local.sh"
ok "labels + milestones in place"

if [[ $DO_ISSUES -eq 1 ]]; then
  step "8. issues + milestone assignment (gh-issues-sync.py)"
  sync_args=(--repo "$REPO")
  [[ $DRY_ISSUES -eq 1 ]] && sync_args+=(--dry-run)
  if "$PY" "$HERE/gh-issues-sync.py" "${sync_args[@]}"; then ok "issues synced"
  else warn "issue sync failed (check pyyaml + gh auth); labels/milestones are still done"; fi
else
  warn "issue sync skipped (per --skip-issues)"
fi

step "done"
printf '  Bootstrap complete.\n'
printf '  If %s/tools/github/idmap.tsv changed, review and commit it:\n' "$REPO_DIR"
printf '    git add tools/github/idmap.tsv && git commit -m "chore(github): record bootstrapped issue ids"\n'
printf '  (Commits are GPG-signed; you will be asked for your key passphrase.)\n'
