# Mycelium GitHub PM sync — Windows/PowerShell entry point.
#
# The cross-platform twin of gh-sync-all.sh. The whole reconcile lives in ONE pure-Python engine,
# gh-issues-sync.py, so this wrapper needs only `python` and `gh` — no bash, no jq. It drives the
# engine with `--all`, the FULL maintenance suite:
#
#   preflight (auth/scope sanity) -> validate (manifests vs codebase) -> labels -> milestones
#   -> issues -> PRs -> project (Project v2 board, when the `project` scope is present).
#
# Every level is create-if-absent + update-to-match + -DryRun + never-silent + idempotent.
# See RECONCILE.md for the full contract.
#
# Requires: PowerShell 5.1+ (or 7+), python (3.x) on PATH, gh authenticated to the repo owner
#   (winget install GitHub.cli ; gh auth login).
#
# Usage:
#   pwsh tools/github/gh-sync-all.ps1                  # full suite (live)
#   pwsh tools/github/gh-sync-all.ps1 -DryRun          # preview the whole reconcile (no writes)
#   pwsh tools/github/gh-sync-all.ps1 -UpdateBodies    # also push issues.yaml bodies
#   pwsh tools/github/gh-sync-all.ps1 -Repo owner/name # override the repo

[CmdletBinding()]
param(
    [string]$Repo = $(if ($env:REPO) { $env:REPO } else { "tzervas/mycelium" }),
    [switch]$DryRun,
    [switch]$UpdateBodies,
    [switch]$NoPreflight
)

$ErrorActionPreference = "Stop"
$here = $PSScriptRoot

function Die([string]$Message, [int]$Code = 1) {
    [Console]::Error.WriteLine($Message)
    exit $Code
}

# Resolve a python interpreter (Windows usually has `python`; some setups expose `python3`/`py`).
$py = $null
foreach ($cand in @("python", "python3", "py")) {
    if (Get-Command $cand -ErrorAction SilentlyContinue) { $py = $cand; break }
}
if (-not $py) { Die "no python interpreter on PATH (install Python 3.x)" }

if (-not (Get-Command gh -ErrorAction SilentlyContinue)) {
    Die "gh (GitHub CLI) not found — install with: winget install GitHub.cli ; then gh auth login"
}

$syncArgs = @((Join-Path $here "gh-issues-sync.py"), "--all", "--repo", $Repo)
if ($DryRun) { $syncArgs += "--dry-run" }
if ($UpdateBodies) { $syncArgs += "--update-bodies" }
if ($NoPreflight) { $syncArgs += "--no-preflight" }

& $py @syncArgs
if ($LASTEXITCODE -ne 0) { Die "gh-issues-sync failed" $LASTEXITCODE }
