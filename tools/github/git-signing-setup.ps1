# Mycelium commit-signing setup — Windows/PowerShell entry point (thin wrapper over the engine).
#
# No args   -> read-only SANITY CHECK (is signing installed + configured?).
# -Setup    -> configure signing (prompts name/email/comment; reuses a key, generates only if
#              absent or when -NewKey forces a rotation). Idempotent + nondestructive + never-silent.
#
#   pwsh tools/github/git-signing-setup.ps1                  # sanity check
#   pwsh tools/github/git-signing-setup.ps1 -Setup          # configure
#   pwsh tools/github/git-signing-setup.ps1 -Setup -NewKey -Upload
[CmdletBinding()]
param(
    [switch]$Setup,
    [switch]$NewKey,
    [switch]$Upload,
    [switch]$DryRun,
    [switch]$NoPassphrase,
    [string]$Name,
    [string]$Email,
    [string]$Comment
)

$ErrorActionPreference = "Stop"
$here = $PSScriptRoot

$py = $null
foreach ($cand in @("python", "python3", "py")) {
    if (Get-Command $cand -ErrorAction SilentlyContinue) { $py = $cand; break }
}
if (-not $py) { [Console]::Error.WriteLine("no python interpreter on PATH (install Python 3.x)"); exit 1 }

$a = @((Join-Path $here "git-signing-sync.py"))
if ($Setup) { $a += "--setup" }
if ($NewKey) { $a += "--new-key" }
if ($Upload) { $a += "--upload" }
if ($DryRun) { $a += "--dry-run" }
if ($NoPassphrase) { $a += "--no-passphrase" }
if ($Name) { $a += @("--name", $Name) }
if ($Email) { $a += @("--email", $Email) }
if ($PSBoundParameters.ContainsKey("Comment")) { $a += @("--comment", $Comment) }

& $py @a
exit $LASTEXITCODE
