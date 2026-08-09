use std::fs;
use std::path::{Path, PathBuf};

const FIRST_PRE_ALPHA_VERSION: &str = "0.1.0";
const MEMBER_MANIFESTS: [&str; 5] = [
    "crates/fenestra-ui/Cargo.toml",
    "crates/fenestra-ui-ir/Cargo.toml",
    "crates/fenestra-ui-runtime/Cargo.toml",
    "crates/fenestra-ui-testkit/Cargo.toml",
    "probes/exp-0001-spine/Cargo.toml",
];

#[test]
fn workspace_uses_one_explicit_pre_alpha_semver_line() {
    assert_eq!(env!("CARGO_PKG_VERSION"), FIRST_PRE_ALPHA_VERSION);

    let root = workspace_root();
    let workspace_manifest = read(&root.join("Cargo.toml"));
    assert!(workspace_manifest.contains("version = \"0.1.0\""));
    assert_eq!(
        workspace_manifest.matches("version = \"=0.1.0\"").count(),
        4
    );

    for relative in MEMBER_MANIFESTS {
        let manifest = read(&root.join(relative));
        assert!(manifest.contains("version.workspace = true"), "{relative}");
        assert!(manifest.contains("publish.workspace = true"), "{relative}");
    }
}

#[test]
fn package_version_has_major_minor_and_patch_components() {
    let components = env!("CARGO_PKG_VERSION").split('.').collect::<Vec<_>>();
    assert_eq!(components.len(), 3);
    assert!(
        components
            .iter()
            .all(|component| component.parse::<u64>().is_ok())
    );
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("facade manifest should remain two levels below the workspace")
        .to_path_buf()
}

fn read(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}
