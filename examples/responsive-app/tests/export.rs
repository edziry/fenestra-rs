use std::process::Command;

use fenestra_responsive_app::{application, checksum, exercise};

#[test]
fn headless_cli_exports_the_final_responsive_frame_and_validates_its_options() {
    let path = std::env::temp_dir().join(format!("fenestra-responsive-{}.ppm", std::process::id()));
    let output = Command::new(env!("CARGO_BIN_EXE_fenestra-responsive-app"))
        .arg("--headless")
        .arg("--ppm")
        .arg(&path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let bytes = std::fs::read(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    let mut app = application().unwrap();
    exercise(&mut app).unwrap();
    let raster = app.raster().unwrap();
    let mut expected = b"P6\n960 520\n255\n".to_vec();
    for pixel in raster.bytes().chunks_exact(4) {
        expected.extend_from_slice(&pixel[..3]);
    }
    assert_eq!(bytes, expected);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("initial viewport=820x520"));
    assert!(stdout.contains("narrow viewport=520x520"));
    assert!(stdout.contains("updated viewport=960x520"));
    assert!(stdout.contains(&format!("checksum={:016x}", checksum(raster.bytes()))));
    for arguments in [
        vec!["--headless", "--native"],
        vec!["--native-smoke", "--headless"],
        vec!["--headless", "--headless"],
        vec!["--ppm"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_fenestra-responsive-app"))
            .args(arguments)
            .output()
            .unwrap();
        assert!(!output.status.success());
    }
}
