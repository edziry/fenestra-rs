use std::fs;
use std::path::{Path, PathBuf};

const PACKAGE_NAME: &str = "fenestra-ui-exp-0014-windows-gpu";

#[test]
fn gpu_probe_is_unpublished_and_additive() {
    let root = workspace_root();
    let workspace = read(&root.join("Cargo.toml"));
    let manifest = read(&root.join("probes/exp-0014-windows-gpu/Cargo.toml"));

    assert!(workspace.contains("\"probes/exp-0014-windows-gpu\""));
    assert!(manifest.contains(&format!("name = \"{PACKAGE_NAME}\"")));
    assert!(manifest.contains("version.workspace = true"));
    assert!(manifest.contains("publish.workspace = true"));

    for directory in ["crates", "probes/exp-0001-native-spine"] {
        for manifest_path in manifests(&root.join(directory)) {
            assert!(
                !read(&manifest_path).contains(PACKAGE_NAME),
                "{} must not depend on the later GPU probe",
                manifest_path.display()
            );
        }
    }
}

fn manifests(directory: &Path) -> Vec<PathBuf> {
    fs::read_dir(directory)
        .expect("manifest directory should be readable")
        .map(|entry| entry.expect("manifest entry should be readable").path())
        .filter(|path| path.is_dir())
        .map(|path| path.join("Cargo.toml"))
        .filter(|path| path.is_file())
        .collect()
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("probe manifest should remain two levels below the workspace")
        .to_path_buf()
}

fn read(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}
