param(
    [Parameter(Mandatory = $true)]
    [string]$Artifact,
    [switch]$Interactive,
    [string]$Runner,
    [string]$Verifier
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

Add-Type @"
using System;
using System.Runtime.InteropServices;

public static class FenestraWin32 {
    private delegate bool EnumWindowsProc(IntPtr hwnd, IntPtr lParam);
    [DllImport("user32.dll")] private static extern bool EnumWindows(EnumWindowsProc callback, IntPtr lParam);
    [DllImport("user32.dll")] private static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint processId);
    [DllImport("user32.dll")] private static extern bool IsWindowVisible(IntPtr hwnd);
    [DllImport("user32.dll")] private static extern bool SetForegroundWindow(IntPtr hwnd);
    [DllImport("user32.dll")] private static extern bool ShowWindow(IntPtr hwnd, int command);
    [DllImport("user32.dll")] private static extern bool GetClientRect(IntPtr hwnd, out RECT rect);
    [DllImport("user32.dll")] private static extern bool ClientToScreen(IntPtr hwnd, ref POINT point);
    [DllImport("user32.dll")] private static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] private static extern uint SendInput(uint count, INPUT[] inputs, int size);
    [DllImport("user32.dll")] private static extern bool SetWindowPos(IntPtr hwnd, IntPtr insertAfter, int x, int y, int width, int height, uint flags);
    [DllImport("user32.dll")] private static extern bool PostMessage(IntPtr hwnd, uint message, IntPtr wParam, IntPtr lParam);

    [StructLayout(LayoutKind.Sequential)] private struct POINT { public int X; public int Y; }
    [StructLayout(LayoutKind.Sequential)] private struct RECT { public int Left; public int Top; public int Right; public int Bottom; }
    [StructLayout(LayoutKind.Sequential)] private struct INPUT { public uint Type; public MOUSEKEYBDATA Data; }
    [StructLayout(LayoutKind.Explicit)] private struct MOUSEKEYBDATA {
        [FieldOffset(0)] public MOUSEINPUT Mouse;
        [FieldOffset(0)] public KEYBDINPUT Keyboard;
    }
    [StructLayout(LayoutKind.Sequential)] private struct MOUSEINPUT {
        public int Dx; public int Dy; public uint MouseData; public uint Flags; public uint Time; public IntPtr Extra;
    }
    [StructLayout(LayoutKind.Sequential)] private struct KEYBDINPUT {
        public ushort Vk; public ushort Scan; public uint Flags; public uint Time; public IntPtr Extra;
    }

    public static IntPtr FindWindow(uint targetProcessId) {
        IntPtr result = IntPtr.Zero;
        EnumWindows((hwnd, ignored) => {
            uint processId;
            GetWindowThreadProcessId(hwnd, out processId);
            if (processId == targetProcessId && IsWindowVisible(hwnd)) {
                result = hwnd;
                return false;
            }
            return true;
        }, IntPtr.Zero);
        return result;
    }

    public static bool ClickClient(IntPtr hwnd, int x, int y) {
        POINT point = new POINT { X = x, Y = y };
        if (!ClientToScreen(hwnd, ref point) || !SetForegroundWindow(hwnd) || !SetCursorPos(point.X, point.Y)) return false;
        return Mouse(0x0002) && Mouse(0x0004);
    }

    public static bool PressSpace() {
        return Keyboard(0x0039, 0x0008) && Keyboard(0x0039, 0x0008 | 0x0002);
    }

    public static bool Resize(IntPtr hwnd, int width, int height) {
        return SetWindowPos(hwnd, IntPtr.Zero, 0, 0, width, height, 0x0002 | 0x0004 | 0x0010);
    }

    public static bool Close(IntPtr hwnd) {
        return PostMessage(hwnd, 0x0010, IntPtr.Zero, IntPtr.Zero);
    }

    private static bool Mouse(uint flags) {
        INPUT[] inputs = { new INPUT { Type = 0, Data = new MOUSEKEYBDATA { Mouse = new MOUSEINPUT { Flags = flags } } } };
        return SendInput(1, inputs, Marshal.SizeOf(typeof(INPUT))) == 1;
    }

    private static bool Keyboard(ushort scan, uint flags) {
        INPUT[] inputs = { new INPUT { Type = 1, Data = new MOUSEKEYBDATA { Keyboard = new KEYBDINPUT { Scan = scan, Flags = flags } } } };
        return SendInput(1, inputs, Marshal.SizeOf(typeof(INPUT))) == 1;
    }
}
"@

function Wait-Window([int]$ProcessId) {
    for ($attempt = 0; $attempt -lt 80; $attempt++) {
        $handle = [FenestraWin32]::FindWindow([uint32]$ProcessId)
        if ($handle -ne [IntPtr]::Zero) { return $handle }
        Start-Sleep -Milliseconds 100
    }
    throw "native window was not found"
}

if ($Interactive) {
    if ([string]::IsNullOrWhiteSpace($Runner) -or [string]::IsNullOrWhiteSpace($Artifact)) { throw "interactive paths are required" }
    $process = Start-Process -FilePath $Runner -ArgumentList "--artifact=$Artifact" -PassThru
    try {
        $window = Wait-Window $process.Id
        Start-Sleep -Milliseconds 750
        if (-not [FenestraWin32]::ClickClient($window, 4, 3)) { throw "native pointer input failed" }
        Start-Sleep -Milliseconds 400
        if (-not [FenestraWin32]::PressSpace()) { throw "native keyboard input failed" }
        Start-Sleep -Milliseconds 500
        if (-not [FenestraWin32]::Resize($window, 704, 460)) { throw "native resize failed" }
        Start-Sleep -Milliseconds 700
        if (-not [FenestraWin32]::Close($window)) { throw "native close failed" }
        $process.WaitForExit()
        if ($process.ExitCode -ne 0) { throw "native runner failed with exit code $($process.ExitCode)" }
    } finally {
        if (-not $process.HasExited) { Stop-Process -Id $process.Id -Force }
    }
    exit 0
}

$ArtifactPath = [System.IO.Path]::GetFullPath(
    $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($Artifact)
)
$Repo = (Get-Location).Path
$Package = "fenestra-layout-inspector"
$Toolchain = "1.97.1-x86_64-pc-windows-msvc"
$Runner = Join-Path $Repo "target\release\fenestra-layout-inspector-native.exe"
$Verifier = Join-Path $Repo "target\release\fenestra-layout-inspector-verify.exe"
if (Test-Path -LiteralPath $ArtifactPath) { throw "artifact path already exists" }
if ((git status --porcelain) -ne "") { throw "worktree is not clean" }

cargo +$Toolchain fmt --all -- --check
cargo +$Toolchain test -p $Package --all-targets --locked -- --test-threads=1
cargo +$Toolchain clippy -p $Package --all-targets --locked -- -D warnings
cargo +$Toolchain doc -p $Package --no-deps --locked
cargo +$Toolchain build --release -p $Package --bins --locked
if (-not (Test-Path -LiteralPath $Runner) -or -not (Test-Path -LiteralPath $Verifier)) { throw "release binaries are missing" }

$TaskName = "Fenestra-WU0015-Layout-Inspector"
$Arguments = "-NoProfile -File `"$PSCommandPath`" -Interactive -Artifact `"$ArtifactPath`" -Runner `"$Runner`" -Verifier `"$Verifier`""
$Action = New-ScheduledTaskAction -Execute "powershell.exe" -Argument $Arguments
$Principal = New-ScheduledTaskPrincipal -UserId "$env:USERDOMAIN\$env:USERNAME" -LogonType Interactive -RunLevel Limited
Register-ScheduledTask -TaskName $TaskName -Action $Action -Principal $Principal -Force | Out-Null
try {
    $started = Get-Date
    Start-ScheduledTask -TaskName $TaskName
    for ($attempt = 0; $attempt -lt 120; $attempt++) {
        $info = Get-ScheduledTaskInfo -TaskName $TaskName
        if ($info.LastRunTime -ge $started -and $info.State -eq "Ready") { break }
        Start-Sleep -Milliseconds 250
    }
    $info = Get-ScheduledTaskInfo -TaskName $TaskName
    if ($info.LastTaskResult -ne 0) { throw "interactive task failed with code $($info.LastTaskResult)" }
} finally {
    Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false -ErrorAction SilentlyContinue
}

if (-not (Test-Path -LiteralPath $ArtifactPath)) { throw "native artifact was not written" }
& $Verifier $ArtifactPath
if ($LASTEXITCODE -ne 0) { throw "native artifact verification failed" }
$hash = Get-FileHash -Algorithm SHA256 -LiteralPath $ArtifactPath
$bytes = [System.IO.File]::ReadAllBytes($ArtifactPath).Length
"pass|artifact=$ArtifactPath|bytes=$bytes|sha256=$($hash.Hash.ToLowerInvariant())"
