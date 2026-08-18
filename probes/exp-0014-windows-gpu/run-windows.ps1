[CmdletBinding()]
param(
    [string] $Artifact = ".\probes\exp-0014-windows-gpu\evidence\windows-dx12-v1.txt"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$Toolchain = "1.97.1-x86_64-pc-windows-msvc"
$Package = "fenestra-ui-exp-0014-windows-gpu"

function Assert-NativeExit {
    param([string] $Step)

    if ($LASTEXITCODE -ne 0) {
        throw "$Step exited with code $LASTEXITCODE"
    }
}

if ([Environment]::OSVersion.Platform -ne [PlatformID]::Win32NT) {
    throw "WU-0014 requires Windows"
}
if (
    [Runtime.InteropServices.RuntimeInformation]::OSArchitecture -ne
    [Runtime.InteropServices.Architecture]::X64
) {
    throw "WU-0014 requires Windows x86-64"
}
$ArtifactPath = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($Artifact)
if (Test-Path -LiteralPath $ArtifactPath) {
    throw "artifact path already exists"
}

rustup toolchain install $Toolchain --profile minimal --component rustfmt,clippy
Assert-NativeExit "rustup toolchain install"

$SourceCommit = (git rev-parse HEAD).Trim()
Assert-NativeExit "git rev-parse HEAD"
$InitialStatus = @(git status --porcelain)
Assert-NativeExit "git status --porcelain"
if ($InitialStatus.Count -ne 0) {
    throw "source tree must be clean before WU-0014"
}
Write-Output "source-commit=$SourceCommit"

cargo +$Toolchain fmt --all -- --check
Assert-NativeExit "cargo fmt"
cargo +$Toolchain test -p $Package --all-targets --locked -- --test-threads=1
Assert-NativeExit "cargo test"
cargo +$Toolchain clippy -p $Package --all-targets --locked -- -D warnings
Assert-NativeExit "cargo clippy"

$HadRustdocFlags = Test-Path Env:RUSTDOCFLAGS
$PreviousRustdocFlags = $env:RUSTDOCFLAGS
try {
    $env:RUSTDOCFLAGS = "-D warnings -D missing-docs"
    cargo +$Toolchain doc -p $Package --no-deps --locked
    Assert-NativeExit "cargo doc"
} finally {
    if ($HadRustdocFlags) {
        $env:RUSTDOCFLAGS = $PreviousRustdocFlags
    } else {
        Remove-Item Env:RUSTDOCFLAGS -ErrorAction SilentlyContinue
    }
}

cargo +$Toolchain build --release -p $Package --bins --locked
Assert-NativeExit "cargo build"

$Runner = ".\target\release\fenestra-ui-exp-0014-windows-gpu.exe"
$Verifier = ".\target\release\fenestra-wu-0014-verify.exe"
& $Runner $ArtifactPath
$ProbeExit = $LASTEXITCODE
if (-not (Test-Path -LiteralPath $ArtifactPath)) {
    throw "probe exited with code $ProbeExit without an artifact"
}

& $Verifier $ArtifactPath
Assert-NativeExit "artifact verifier"
if ($ProbeExit -ne 0) {
    throw "probe exited with code $ProbeExit"
}

$ArtifactHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $ArtifactPath).Hash.ToLowerInvariant()
$ArtifactBytes = [System.IO.File]::ReadAllBytes($ArtifactPath).Length
Write-Output "artifact-sha256=$ArtifactHash"
Write-Output "artifact-bytes=$ArtifactBytes"

rustc +$Toolchain -Vv
Assert-NativeExit "rustc -Vv"
cargo +$Toolchain -V
Assert-NativeExit "cargo -V"
$Os = Get-CimInstance Win32_OperatingSystem
Write-Output "windows-version=$($Os.Version)"
Write-Output "windows-build=$($Os.BuildNumber)"
Write-Output "windows-architecture=$($Os.OSArchitecture)"
git status --short
Assert-NativeExit "git status --short"
