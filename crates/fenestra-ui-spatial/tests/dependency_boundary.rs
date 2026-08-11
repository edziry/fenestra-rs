use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn manifest_and_lock_keep_the_spatial_boundary_candidate_neutral() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest = read(&manifest_dir.join("Cargo.toml"));
    let dependencies = section(&manifest, "[dependencies]");
    assert_eq!(dependencies.trim(), "fenestra-ui-layout.workspace = true");

    for forbidden in [
        "fenestra-ui-ir",
        "fenestra-ui-runtime",
        "fenestra-ui-testkit",
        "fenestra-ui-authoring",
        "taffy",
    ] {
        assert!(
            !manifest.contains(forbidden),
            "leaked dependency {forbidden}"
        );
    }

    let lock = read(&workspace_root(&manifest_dir).join("Cargo.lock"));
    let package = package_section(&lock, "fenestra-ui-spatial");
    assert!(package.contains("version = \"0.2.0\""));
    assert!(package.contains("dependencies = [\n \"fenestra-ui-layout\",\n]"));
}

fn workspace_root(manifest_dir: &Path) -> PathBuf {
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("spatial crate should remain under crates/")
        .to_path_buf()
}

fn read(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn section<'a>(source: &'a str, header: &str) -> &'a str {
    let start = source
        .find(header)
        .unwrap_or_else(|| panic!("missing section {header}"));
    let body = &source[start + header.len()..];
    let end = body.find("\n[").unwrap_or(body.len());
    &body[..end]
}

fn package_section<'a>(lock: &'a str, name: &str) -> &'a str {
    let marker = format!("[[package]]\nname = \"{name}\"");
    let start = lock
        .find(&marker)
        .unwrap_or_else(|| panic!("missing lock package {name}"));
    let body = &lock[start..];
    let end = body[marker.len()..]
        .find("\n[[package]]")
        .map_or(body.len(), |offset| marker.len() + offset);
    &body[..end]
}
