use std::process::Command;

use fenestra_controls_app::{DemoState, application, checksum, exercise};

#[test]
fn headless_cli_exports_the_public_raster_and_validates_its_options() {
    let path = std::env::temp_dir().join(format!("fenestra-controls-{}.ppm", std::process::id()));
    let output = Command::new(env!("CARGO_BIN_EXE_fenestra-controls-app"))
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
    exercise(&mut app, &mut DemoState::default()).unwrap();
    let raster = app.raster().unwrap();
    let mut expected = b"P6\n420 560\n255\n".to_vec();
    for pixel in raster.bytes().chunks_exact(4) {
        expected.extend_from_slice(&pixel[..3]);
    }
    assert_eq!(bytes, expected);
    let stdout = String::from_utf8_lossy(&output.stdout);
    for checkpoint in [
        "initial viewport=640x520",
        "changed viewport=640x520",
        "applied viewport=640x520",
        "reset viewport=420x560",
    ] {
        assert!(stdout.contains(checkpoint));
    }
    assert!(stdout.contains(&format!("checksum={:016x}", checksum(raster.bytes()))));
    for arguments in [
        vec!["--headless", "--native"],
        vec!["--native-smoke", "--headless"],
        vec!["--ppm"],
    ] {
        assert!(
            !Command::new(env!("CARGO_BIN_EXE_fenestra-controls-app"))
                .args(arguments)
                .output()
                .unwrap()
                .status
                .success()
        );
    }
}
