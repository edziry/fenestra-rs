use std::ffi::OsString;
use std::path::Path;
use std::process::Command;

use fenestra_ui_exp_0014_windows_gpu::{ProbeCliErrorKindV1, parse_probe_cli_v1};

#[test]
fn release_cli_accepts_exactly_one_artifact_path() {
    let cli = parse_probe_cli_v1([
        OsString::from("fenestra-ui-exp-0014-windows-gpu"),
        OsString::from("evidence/windows-dx12.txt"),
    ])
    .expect("one artifact path");

    assert_eq!(cli.artifact_path(), Path::new("evidence/windows-dx12.txt"));
}

#[test]
fn missing_and_extra_arguments_are_closed_failures() {
    assert_eq!(
        parse_probe_cli_v1([OsString::from("probe")]).expect_err("missing path"),
        ProbeCliErrorKindV1::MissingArtifactPath
    );
    assert_eq!(
        parse_probe_cli_v1([
            OsString::from("probe"),
            OsString::from("one.txt"),
            OsString::from("two.txt"),
        ])
        .expect_err("extra path"),
        ProbeCliErrorKindV1::ExtraArgument
    );
}

#[test]
fn artifact_path_must_name_a_file() {
    assert_eq!(
        parse_probe_cli_v1([OsString::from("probe"), OsString::from("")]).expect_err("empty path"),
        ProbeCliErrorKindV1::InvalidArtifactPath
    );
    assert_eq!(
        parse_probe_cli_v1([OsString::from("probe"), OsString::from(".")])
            .expect_err("directory-shaped path"),
        ProbeCliErrorKindV1::InvalidArtifactPath
    );
}

#[test]
fn release_binary_reports_the_typed_probe_failure() {
    let output = Command::new(env!("CARGO_BIN_EXE_fenestra-ui-exp-0014-windows-gpu"))
        .arg("debug-build-must-not-write-evidence.txt")
        .output()
        .expect("run probe binary");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert_eq!(
        std::str::from_utf8(&output.stderr)
            .expect("ASCII diagnostic")
            .trim(),
        "interactive-probe-error=BuildProfile"
    );
}
