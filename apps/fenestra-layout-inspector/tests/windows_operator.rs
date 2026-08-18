use std::fs;
use std::path::Path;

#[test]
fn windows_operator_is_bounded_ascii_and_uses_real_input() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("run-windows.ps1");
    let bytes = fs::read(path).expect("versioned Windows operator");
    let script = std::str::from_utf8(&bytes).expect("operator UTF-8");

    assert!(bytes.is_ascii());
    assert!(!bytes.contains(&b'\r'));
    assert_eq!(bytes.last(), Some(&b'\n'));
    assert!(bytes.len() <= 16_384);
    for required in [
        "Set-StrictMode -Version Latest",
        "1.97.1-x86_64-pc-windows-msvc",
        "git status --porcelain",
        "$Status.Count -ne 0",
        "cargo +$Toolchain fmt --all -- --check",
        "cargo +$Toolchain test -p $Package --all-targets --locked",
        "cargo +$Toolchain clippy -p $Package --all-targets --locked -- -D warnings",
        "cargo +$Toolchain doc -p $Package --no-deps --locked",
        "cargo +$Toolchain build --release -p $Package --bins --locked",
        "Join-Path $PSScriptRoot",
        "Set-Location -LiteralPath $Repo",
        "SessionState.Path.GetUnresolvedProviderPathFromPSPath($Artifact)",
        "SendInput",
        "ClickClient",
        "PressSpace",
        "SetWindowPos",
        "PostMessage",
        "fenestra-layout-inspector-native.exe",
        "fenestra-layout-inspector-verify.exe",
        "Get-FileHash -Algorithm SHA256 -LiteralPath $ArtifactPath",
        "ReadAllBytes($ArtifactPath).Length",
        "New-ScheduledTaskPrincipal -UserId $env:USERNAME -LogonType Interactive",
        "Get-ScheduledTask -TaskName $TaskName",
    ] {
        assert!(script.contains(required), "missing `{required}`");
    }
    assert!(!script.contains("Invoke-Expression"));
}
