use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn manifest() -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .expect("probe manifest")
}

pub(crate) fn workspace_manifest() -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.toml"))
        .expect("workspace manifest")
}

pub(crate) fn rust_sources(relative: &str) -> Vec<(PathBuf, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    let mut paths = Vec::new();
    collect(&root, &mut paths);
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            let source = fs::read_to_string(&path).expect("Rust source");
            (path, source)
        })
        .collect()
}

fn collect(path: &Path, output: &mut Vec<PathBuf>) {
    let metadata = fs::symlink_metadata(path)
        .unwrap_or_else(|_| panic!("missing source path {}", path.display()));
    assert!(
        !metadata.file_type().is_symlink(),
        "source scan rejects symlink {}",
        path.display()
    );
    if metadata.is_file() {
        if path.extension().and_then(|value| value.to_str()) == Some("rs") {
            output.push(path.to_path_buf());
        }
        return;
    }
    assert!(
        metadata.is_dir(),
        "source path must be a file or directory {}",
        path.display()
    );
    let entries =
        fs::read_dir(path).unwrap_or_else(|_| panic!("missing source root {}", path.display()));
    for entry in entries {
        collect(&entry.expect("directory entry").path(), output);
    }
}
