use std::fs;
use std::process::Command;

mod support;

use support::valid_pass_artifact;

#[test]
fn verifier_cli_accepts_only_a_verified_pass_artifact() {
    let directory = std::env::temp_dir();
    let pass_path = directory.join(format!("fenestra-wu-0014-pass-{}.txt", std::process::id()));
    let invalid_path =
        directory.join(format!("fenestra-wu-0014-invalid-{}.txt", std::process::id()));
    fs::write(&pass_path, valid_pass_artifact()).expect("write pass fixture");
    fs::write(&invalid_path, b"not an artifact\n").expect("write invalid fixture");

    let pass = Command::new(env!("CARGO_BIN_EXE_fenestra-wu-0014-verify"))
        .arg(&pass_path)
        .output()
        .expect("run pass verifier");
    let invalid = Command::new(env!("CARGO_BIN_EXE_fenestra-wu-0014-verify"))
        .arg(&invalid_path)
        .output()
        .expect("run invalid verifier");

    fs::remove_file(pass_path).expect("remove pass fixture");
    fs::remove_file(invalid_path).expect("remove invalid fixture");
    assert!(pass.status.success());
    assert_eq!(pass.stdout, b"pass|records=16|bytes=1092|generation=2\n");
    assert!(pass.stderr.is_empty());
    assert!(!invalid.status.success());
    assert!(invalid.stdout.is_empty());
    assert_eq!(invalid.stderr, b"artifact verification failed\n");
}
