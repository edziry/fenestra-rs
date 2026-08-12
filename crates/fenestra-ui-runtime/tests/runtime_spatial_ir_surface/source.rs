use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const SOURCE_ROOT_ENV: &str = "FENESTRA_RUNTIME_SPATIAL_IR_SOURCE_ROOT";

pub(super) fn all_source() -> String {
    let mut paths = Vec::new();
    collect_rust_sources(&source_dir(), &mut paths);
    paths.sort();
    paths
        .into_iter()
        .map(|path| read(&path))
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn read(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

pub(super) fn source_dir() -> PathBuf {
    env::var_os(SOURCE_ROOT_ENV).map_or_else(
        || PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src"),
        PathBuf::from,
    )
}

fn collect_rust_sources(directory: &Path, paths: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).expect("read runtime source directory") {
        let path = entry.expect("source entry").path();
        if path.is_dir() {
            collect_rust_sources(&path, paths);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            paths.push(path);
        }
    }
}
