use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn runtime_spatial_cut_has_the_exact_workspace_dependency_boundary() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest = read(&manifest_dir.join("Cargo.toml"));
    assert_eq!(
        section(&manifest, "[dependencies]").trim(),
        concat!(
            "fenestra-ui-ir.workspace = true\n",
            "fenestra-ui-layout.workspace = true\n",
            "fenestra-ui-spatial.workspace = true"
        )
    );
    for forbidden_header in ["[dev-dependencies]", "[build-dependencies]", "[target."] {
        assert!(
            !manifest.contains(forbidden_header),
            "unexpected dependency section {forbidden_header}"
        );
    }

    let lock = read(&workspace_root(&manifest_dir).join("Cargo.lock"));
    let package = package_section(&lock, "fenestra-ui-runtime");
    assert!(package.contains("version = \"0.2.0\""));
    assert!(package.contains(concat!(
        "dependencies = [\n",
        " \"fenestra-ui-ir\",\n",
        " \"fenestra-ui-layout\",\n",
        " \"fenestra-ui-spatial\",\n",
        "]"
    )));
}

fn workspace_root(manifest_dir: &Path) -> PathBuf {
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("runtime crate should remain under crates/")
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
