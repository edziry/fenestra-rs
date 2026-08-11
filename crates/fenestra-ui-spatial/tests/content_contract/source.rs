use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn all_source() -> String {
    let mut paths = Vec::new();
    collect_rust_sources(&source_dir(), &mut paths);
    paths.sort();
    paths
        .into_iter()
        .map(|path| fs::read_to_string(path).expect("read spatial source"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn collect_rust_sources(directory: &Path, paths: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).expect("read spatial source directory") {
        let path = entry.expect("source entry").path();
        if path.is_dir() {
            collect_rust_sources(&path, paths);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            paths.push(path);
        }
    }
}

pub(super) fn source_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}
