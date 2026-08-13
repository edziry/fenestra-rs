use std::fs;
use std::path::Path;

#[test]
fn windows_operator_script_is_bounded_pinned_and_fail_closed() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("run-windows.ps1");
    let bytes = fs::read(path).expect("versioned Windows operator script");
    let script = std::str::from_utf8(&bytes).expect("operator script UTF-8");

    assert!(bytes.is_ascii());
    assert!(!bytes.contains(&b'\r'));
    assert_eq!(bytes.last(), Some(&b'\n'));
    assert!(bytes.len() <= 8_192);
    for required in [
        "Set-StrictMode -Version Latest",
        "1.97.1-x86_64-pc-windows-msvc",
        "git status --porcelain",
        "cargo +$Toolchain fmt --all -- --check",
        "cargo +$Toolchain test -p $Package --all-targets --locked -- --test-threads=1",
        "cargo +$Toolchain clippy -p $Package --all-targets --locked -- -D warnings",
        "cargo +$Toolchain doc -p $Package --no-deps --locked",
        "cargo +$Toolchain build --release -p $Package --bins --locked",
        "Test-Path -LiteralPath $Artifact",
        "fenestra-ui-exp-0014-windows-gpu.exe",
        "fenestra-wu-0014-verify.exe",
        "Get-FileHash -Algorithm SHA256",
        "ReadAllBytes($Artifact).Length",
    ] {
        assert!(script.contains(required), "missing `{required}`");
    }
    assert!(!script.contains("WGPU_BACKEND="));
    assert!(!script.contains("--features"));
}
