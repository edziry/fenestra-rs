# WU-0014 Windows DX12 operator protocol

Status: ready for registered execution
Required host: Windows x86-64 with a physical GPU
Required Rust host: `x86_64-pc-windows-msvc`

## Purpose

This protocol builds and runs the disposable WU-0014 probe. A passing run
proves one exact interactive DX12 path. It does not establish broad Windows or
GPU support.

Run every command from the repository root in a regular PowerShell session.
Do not set `WGPU_BACKEND` or other backend-selection environment variables.
The executable enables DX12 directly and rejects a different effective
backend or a CPU or virtual adapter.

## Prerequisites

- Windows 11 x86-64 with current GPU drivers;
- Visual Studio 2022 C++ x64 build tools and a Windows SDK; and
- Rustup with network access for the pinned Rust toolchain.

Install the exact toolchain without changing another default toolchain:

```powershell
rustup toolchain install 1.97.1-x86_64-pc-windows-msvc --profile minimal --component rustfmt,clippy
```

## Source and pure gates

Record the source commit and require a clean source tree before execution:

```powershell
git rev-parse HEAD
git status --short
```

The second command must print nothing. Then run the serial pure gates:

```powershell
cargo +1.97.1-x86_64-pc-windows-msvc fmt --all -- --check
cargo +1.97.1-x86_64-pc-windows-msvc test -p fenestra-ui-exp-0014-windows-gpu --all-targets --locked -- --test-threads=1
cargo +1.97.1-x86_64-pc-windows-msvc clippy -p fenestra-ui-exp-0014-windows-gpu --all-targets --locked -- -D warnings
$env:RUSTDOCFLAGS = "-D warnings -D missing-docs"
cargo +1.97.1-x86_64-pc-windows-msvc doc -p fenestra-ui-exp-0014-windows-gpu --no-deps --locked
Remove-Item Env:RUSTDOCFLAGS
cargo +1.97.1-x86_64-pc-windows-msvc build --release -p fenestra-ui-exp-0014-windows-gpu --bins --locked
```

## Interactive sequence

The artifact path must not already exist. The runner refuses to overwrite it.

```powershell
$Artifact = ".\probes\exp-0014-windows-gpu\evidence\windows-dx12-v1.txt"
if (Test-Path $Artifact) { throw "artifact path already exists" }
& ".\target\release\fenestra-ui-exp-0014-windows-gpu.exe" $Artifact
```

Follow the window title one step at a time:

1. Wait for `move the pointer`, then move the pointer inside the window.
2. Wait for `press the primary button`, then press the left mouse button once.
3. Wait for `resize the window`, then drag one window edge to a distinct size.
4. Wait for `minimize the window`, then minimize it using Windows.
5. Restore the same window from the taskbar.
6. Wait for `close the window`, then close it normally.

Do not close the window early. An early close writes a valid `stop` artifact
and returns a failing process exit. An unsupported environment writes an
`adapt` artifact and also returns a failing process exit.

## Independent verification

Run the standalone verifier against the bytes written by the runner:

```powershell
& ".\target\release\fenestra-wu-0014-verify.exe" $Artifact
if ($LASTEXITCODE -ne 0) { throw "WU-0014 evidence did not pass" }
Get-FileHash -Algorithm SHA256 $Artifact
[System.IO.File]::ReadAllBytes($Artifact).Length
```

The verifier prints only a `pass` summary containing record count, byte count,
and final runtime generation. Preserve the artifact bytes exactly for review.

Record these non-private environment facts with the artifact:

```powershell
rustc +1.97.1-x86_64-pc-windows-msvc -Vv
cargo +1.97.1-x86_64-pc-windows-msvc -V
Get-CimInstance Win32_OperatingSystem | Select-Object Version,BuildNumber,OSArchitecture
git status --short
```

The expected source-tree change after a pass is only the new evidence file.
