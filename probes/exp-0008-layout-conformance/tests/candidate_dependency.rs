#![forbid(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};

const TAFFY_DEPENDENCY: &str = "taffy = { version = \"=0.13.0\", default-features = false, features = [\"std\", \"taffy_tree\", \"flexbox\"] }";
const RUNTIME_DEV_DEPENDENCIES: &str = "\
fenestra-ui-ir.workspace = true
fenestra-ui-runtime.workspace = true
fenestra-ui-testkit.workspace = true";

#[test]
fn probe_manifest_has_the_exact_private_taffy_pin_and_features() {
    let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let manifest = read(&manifest_path);

    assert_eq!(
        manifest
            .lines()
            .filter(|line| *line == TAFFY_DEPENDENCY)
            .count(),
        1,
        "probe must have one exact Taffy dependency declaration"
    );
}

#[test]
fn taffy_dependency_does_not_leak_to_other_workspace_manifests() {
    let root = workspace_root();
    let probe_manifest = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("Cargo.toml")
        .canonicalize()
        .expect("probe manifest path must resolve");
    let mut manifests = Vec::new();
    collect_manifests(&root, &mut manifests);

    for path in manifests {
        if path == probe_manifest {
            continue;
        }
        let manifest = read(&path);
        assert!(
            !manifest
                .lines()
                .any(|line| line.trim_start().starts_with("taffy ")),
            "Taffy dependency leaked into {}",
            path.display()
        );
    }
}

#[test]
fn runtime_acceptance_dependencies_are_exactly_dev_only() {
    let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let manifest = read(&manifest_path);
    let normal = manifest_section(&manifest, "[dependencies]");
    let development = manifest_section(&manifest, "[dev-dependencies]");

    for dependency in [
        "fenestra-ui-ir.workspace",
        "fenestra-ui-runtime.workspace",
        "fenestra-ui-testkit.workspace",
    ] {
        assert!(
            !normal.contains(dependency),
            "runtime acceptance dependency leaked into normal dependencies"
        );
    }
    assert_eq!(development.trim(), RUNTIME_DEV_DEPENDENCIES);
}

#[test]
fn lockfile_has_the_exact_candidate_and_transitive_sections() {
    let lock = read(&workspace_root().join("Cargo.lock"));

    assert_package(&lock, "arrayvec", "0.7.8", None);
    assert_package(
        &lock,
        "slotmap",
        "1.1.1",
        Some("dependencies = [\n \"version_check\",\n]"),
    );
    assert_package(
        &lock,
        "taffy",
        "0.13.0",
        Some("dependencies = [\n \"arrayvec\",\n \"serde\",\n \"slotmap\",\n]"),
    );
    assert_package(&lock, "version_check", "0.9.5", None);
    assert_package(
        &lock,
        "fenestra-ui-exp-0008-layout-conformance",
        "0.1.1",
        Some(
            "dependencies = [\n \"fenestra-ui-ir\",\n \"fenestra-ui-layout\",\n \"fenestra-ui-runtime\",\n \"fenestra-ui-testkit\",\n \"taffy\",\n]",
        ),
    );
}

fn manifest_section<'a>(manifest: &'a str, heading: &str) -> &'a str {
    manifest
        .split_once(heading)
        .unwrap_or_else(|| panic!("missing {heading}"))
        .1
        .split("\n[")
        .next()
        .expect("manifest section must have a body")
}

fn assert_package(lock: &str, name: &str, version: &str, dependencies: Option<&str>) {
    let sections = lock
        .split("[[package]]")
        .filter(|section| {
            section
                .lines()
                .any(|line| line == format!("name = \"{name}\""))
        })
        .collect::<Vec<_>>();
    assert_eq!(sections.len(), 1, "wrong number of {name} lock sections");
    let section = sections[0];
    assert!(
        section
            .lines()
            .any(|line| line == format!("version = \"{version}\"")),
        "wrong locked {name} version"
    );
    if let Some(dependencies) = dependencies {
        assert!(
            section.contains(dependencies),
            "wrong locked {name} dependencies"
        );
    }
}

fn collect_manifests(directory: &Path, manifests: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).expect("workspace directory must be readable") {
        let entry = entry.expect("workspace entry must be readable");
        let path = entry.path();
        if path.is_dir() {
            let name = entry.file_name();
            if name != "target" && name != ".git" {
                collect_manifests(&path, manifests);
            }
        } else if entry.file_name() == "Cargo.toml" {
            manifests.push(path);
        }
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root must resolve")
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}
