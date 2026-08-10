use std::fs;
use std::path::{Path, PathBuf};

const LINUX_TARGET: &str = r#"[target.'cfg(target_os = "linux")'.dependencies]
winit = { version = "=0.30.13", default-features = false, features = [
    "rwh_06",
    "wayland",
    "wayland-dlopen",
] }
softbuffer = { version = "=0.4.8", default-features = false, features = [
    "wayland",
    "wayland-dlopen",
] }
"#;

const WINDOWS_TARGET: &str = r#"[target.'cfg(target_os = "windows")'.dependencies]
winit = { version = "=0.30.13", default-features = false, features = [
    "rwh_06",
] }
softbuffer = { version = "=0.4.8", default-features = false }
"#;

#[test]
fn native_candidates_are_exact_target_scoped_and_replaceable() {
    let root = workspace_root();
    let manifest = read(&root.join("probes/exp-0001-native-spine/Cargo.toml"));
    assert!(manifest.contains(LINUX_TARGET));
    assert!(manifest.contains(WINDOWS_TARGET));
    assert_eq!(manifest.matches("winit = ").count(), 2);
    assert_eq!(manifest.matches("softbuffer = ").count(), 2);
    assert!(!manifest.contains("x11"));
    assert!(!manifest.contains("default-features = true"));

    let workspace = read(&root.join("Cargo.toml"));
    assert!(!workspace.contains("winit = "));
    assert!(!workspace.contains("softbuffer = "));
    for relative in [
        "crates/fenestra-ui/Cargo.toml",
        "crates/fenestra-ui-ir/Cargo.toml",
        "crates/fenestra-ui-runtime/Cargo.toml",
        "crates/fenestra-ui-testkit/Cargo.toml",
        "probes/exp-0001-spine/Cargo.toml",
    ] {
        let other = read(&root.join(relative));
        assert!(!other.contains("winit = "), "{relative}");
        assert!(!other.contains("softbuffer = "), "{relative}");
    }
}

#[test]
fn lockfile_records_the_exact_native_candidates() {
    let lock = read(&workspace_root().join("Cargo.lock"));
    assert!(package_block(&lock, "winit", "0.30.13").is_some());
    assert!(package_block(&lock, "softbuffer", "0.4.8").is_some());
}

fn package_block<'a>(lock: &'a str, name: &str, version: &str) -> Option<&'a str> {
    lock.split("[[package]]").find(|block| {
        block.contains(&format!("name = \"{name}\""))
            && block.contains(&format!("version = \"{version}\""))
            && block.contains("source = \"registry+")
            && block.contains("checksum = ")
    })
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("native probe should remain below the workspace root")
        .to_path_buf()
}

fn read(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}
