# Mycelium GitHub PM sync — Windows/PowerShell entry point.
#
# The cross-platform twin of gh-sync-all.sh. Because gh-bootstrap-local.sh (labels +
# milestones) is bash + jq, Windows users run the whole reconcile through the pure-Python
# engine instead: gh-issues-sync.py now does labels + milestones + issues (create AND
# intelligent update) on its own, so this wrapper needs only `python` and `gh` — no bash,
# no jq.
#
# It runs, in order:
#   0. manifest-check.py   — preflight: every label/milestone issues.yaml references must be
#                            defined in the manifests (explicit, never silent).
#   1. gh-issues-sync.py --all — labels (create-or-update) + milestones (create-absent) +
#                            issues (create-absent + reconcile existing to issues.yaml).
#
# Idempotent: re-run any time. Nothing is duplicated; an in-sync issue is left untouched.
#
# Requires: PowerShell 5.1+ (or 7+), python (3.x) on PATH, gh authenticated to the repo owner
#   (winget install GitHub.cli ; gh auth login).
#
# Usage:
#   pwsh tools/github/gh-sync-all.ps1                       # full reconcile (live)
#   pwsh tools/github/gh-sync-all.ps1 -DryRun               # preview, no repo writes
#   pwsh tools/github/gh-sync-all.ps1 -Repo owner/name      # override the repo
#   pwsh tools/github/gh-sync-all.ps1 -UpdateBodies         # also push issues.yaml bodies
#
# (On Linux/macOS the bash gh-sync-all.sh remains the native path; both reconcile the same
# manifests through the same Python issue logic.)

[CmdletBinding()]
param(
    [string]$Repo = $(if ($env:REPO) { $env:REPO } else { "tzervas/mycelium" }),
    [switch]$DryRun,
    [switch]$UpdateBodies
)

# Stop on unexpected cmdlet errors. Controlled failures below use Die (stderr + exit) rather
# than Write-Error, because under 'Stop' Write-Error is terminating and would skip the exit.
$ErrorActionPreference = "Stop"

# $PSScriptRoot is the script's own directory (valid in all scripts since PowerShell 3.0).
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

Write-Host "============================================================"
Write-Host ">> Mycelium PM sync (PowerShell) — repo: $Repo  (dry-run: $DryRun)"
Write-Host "============================================================"

Write-Host ""
Write-Host ">> [0/1] preflight: manifest consistency"
& $py (Join-Path $here "manifest-check.py")
if ($LASTEXITCODE -ne 0) { Die "manifest-check failed" $LASTEXITCODE }

Write-Host ""
Write-Host ">> [1/1] labels + milestones + issues (gh-issues-sync.py --all)"
$syncArgs = @((Join-Path $here "gh-issues-sync.py"), "--all", "--repo", $Repo)
if ($DryRun) { $syncArgs += "--dry-run" }
if ($UpdateBodies) { $syncArgs += "--update-bodies" }
& $py @syncArgs
if ($LASTEXITCODE -ne 0) { Die "gh-issues-sync failed" $LASTEXITCODE }

Write-Host ""
Write-Host ">> sync complete — repo reconciled with issues.yaml / labels.json / milestones.json."
