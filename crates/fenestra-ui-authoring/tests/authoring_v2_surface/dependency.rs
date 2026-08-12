use std::path::{Path, PathBuf};

use super::source::read;

#[test]
fn format_2_adds_no_authoring_dependency_or_build_boundary() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest = read(&manifest_dir.join("Cargo.toml"));
    assert_eq!(
        section(&manifest, "[dependencies]").trim(),
        concat!(
            "fenestra-ui-ir.workspace = true\n",
            "proc-macro2 = \"=1.0.107\"\n",
            "quote = \"=1.0.47\""
        )
    );
    for forbidden in ["[dev-dependencies]", "[build-dependencies]", "[target."] {
        assert!(!manifest.contains(forbidden), "unexpected {forbidden}");
    }

    let lock = read(&workspace_root(&manifest_dir).join("Cargo.lock"));
    let package = package_section(&lock, "fenestra-ui-authoring");
    assert!(package.contains("version = \"0.2.0\""));
    assert!(package.contains(concat!(
        "dependencies = [\n",
        " \"fenestra-ui-ir\",\n",
        " \"proc-macro2\",\n",
        " \"quote\",\n",
        "]"
    )));
}

fn workspace_root(manifest_dir: &Path) -> PathBuf {
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("authoring crate should remain under crates/")
        .to_path_buf()
}

fn section<'a>(source: &'a str, header: &str) -> &'a str {
    let start = source.find(header).expect("manifest section");
    let body = &source[start + header.len()..];
    let end = body.find("\n[").unwrap_or(body.len());
    &body[..end]
}

fn package_section<'a>(source: &'a str, name: &str) -> &'a str {
    let marker = format!("[[package]]\nname = \"{name}\"");
    let start = source.find(&marker).expect("lock package");
    let body = &source[start..];
    let end = body[marker.len()..]
        .find("\n[[package]]")
        .map_or(body.len(), |offset| marker.len() + offset);
    &body[..end]
}
