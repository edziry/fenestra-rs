use std::path::PathBuf;
use std::process::Command;

use fenestra_text_app::{application, update_headless};

struct ExportPath(PathBuf);

impl ExportPath {
    fn new() -> Self {
        Self(std::env::temp_dir().join(format!("fenestra-text-app-{}.ppm", std::process::id())))
    }
}

impl Drop for ExportPath {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

#[test]
fn ppm_export_contains_the_exact_final_headless_rgb_channels() {
    let path = ExportPath::new();
    let output = Command::new(env!("CARGO_BIN_EXE_fenestra-text-app"))
        .arg("--ppm")
        .arg(&path.0)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("checksum=dddfe50366b70bcf"));
    let mut app = application().unwrap();
    update_headless(&mut app).unwrap();
    let raster = app.raster().unwrap();
    let mut expected = b"P6\n640 360\n255\n".to_vec();
    for pixel in raster.bytes().chunks_exact(4) {
        expected.extend_from_slice(&pixel[..3]);
    }
    assert_eq!(std::fs::read(&path.0).unwrap(), expected);
}

#[test]
fn export_arguments_reject_missing_paths_duplicates_and_conflicting_modes() {
    for arguments in [
        vec!["--ppm"],
        vec!["--ppm", "one.ppm", "--ppm", "two.ppm"],
        vec!["--native", "--native-smoke"],
        vec!["--unknown"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_fenestra-text-app"))
            .args(arguments)
            .output()
            .unwrap();
        assert!(!output.status.success());
    }
    let help = Command::new(env!("CARGO_BIN_EXE_fenestra-text-app"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("--ppm PATH"));
}
