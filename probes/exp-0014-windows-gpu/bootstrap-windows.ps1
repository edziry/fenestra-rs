[CmdletBinding()]
param(
    [string] $Checkout = (Join-Path $PSScriptRoot "fenestra-wu-0014")
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Assert-NativeExit {
    param([string] $Step)

    if ($LASTEXITCODE -ne 0) {
        throw "$Step exited with code $LASTEXITCODE"
    }
}

if ([Environment]::OSVersion.Platform -ne [PlatformID]::Win32NT) {
    throw "WU-0014 bootstrap requires Windows"
}
if (Test-Path -LiteralPath $Checkout) {
    throw "checkout path already exists"
}

$Bundles = @(
    Get-ChildItem -LiteralPath $PSScriptRoot -Filter "fenestra-wu-0014-*.bundle" -File
)
if ($Bundles.Count -ne 1) {
    throw "expected exactly one WU-0014 Git bundle beside this script"
}
$Bundle = $Bundles[0].FullName
$ExpectedCommitPrefix = [System.IO.Path]::GetFileNameWithoutExtension($Bundle).Split("-")[-1]
if ($ExpectedCommitPrefix -notmatch "^[0-9a-f]{7,40}$") {
    throw "bundle filename has no commit prefix"
}

git bundle verify $Bundle
Assert-NativeExit "git bundle verify"
git clone -b feat/windows-interactive-gpu-spine $Bundle $Checkout
Assert-NativeExit "git clone"

$ObservedCommit = (git -C $Checkout rev-parse HEAD).Trim()
Assert-NativeExit "git rev-parse HEAD"
if (-not $ObservedCommit.StartsWith($ExpectedCommitPrefix, [StringComparison]::OrdinalIgnoreCase)) {
    throw "cloned commit does not match bundle filename"
}

$Runner = Join-Path $Checkout "probes\exp-0014-windows-gpu\run-windows.ps1"
Push-Location $Checkout
try {
    & $Runner
    Assert-NativeExit "WU-0014 Windows runner"
} finally {
    Pop-Location
}
