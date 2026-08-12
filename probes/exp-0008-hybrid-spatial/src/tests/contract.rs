use super::support::source;

const DEPENDENCIES: [&str; 4] = [
    "fenestra-ui-ir.workspace = true",
    "fenestra-ui-layout.workspace = true",
    "fenestra-ui-runtime.workspace = true",
    "fenestra-ui-spatial.workspace = true",
];

#[test]
fn package_is_an_additive_private_workspace_probe() {
    let workspace = source::workspace_manifest();
    assert_eq!(
        workspace
            .matches("\"probes/exp-0008-hybrid-spatial\"")
            .count(),
        1
    );
    assert_eq!(
        workspace
            .matches("\"probes/exp-0008-layout-conformance\"")
            .count(),
        1
    );

    let manifest = source::manifest();
    assert!(manifest.contains("name = \"fenestra-ui-exp-0008-hybrid-spatial\""));
    assert!(manifest.contains("version.workspace = true"));
    assert!(manifest.contains("publish.workspace = true"));
    assert!(!manifest.contains("[dev-dependencies]"));
    assert!(!manifest.contains("[build-dependencies]"));

    let dependency_body = manifest
        .split_once("[dependencies]\n")
        .expect("dependency section")
        .1
        .trim();
    assert_eq!(dependency_body.lines().collect::<Vec<_>>(), DEPENDENCIES);
}

#[test]
fn evidence_surface_never_leaves_the_probe() {
    let lib = source::rust_sources("src")
        .into_iter()
        .find(|(path, _)| path.ends_with("lib.rs"))
        .expect("library root")
        .1;
    assert!(!lib.contains("pub mod"));
    assert!(!lib.contains("pub use"));
    assert!(!lib.contains("prototype"));

    for (path, text) in source::rust_sources("src/baseline") {
        for line in text.lines().map(str::trim) {
            assert!(
                !line.starts_with("pub "),
                "{} exposes a probe-private item: {line}",
                path.display()
            );
        }
    }
}

#[test]
fn literal_oracle_imports_only_std_and_probe_private_literal_types() {
    let sources = source::rust_sources("src/baseline/literal");
    assert!(!sources.is_empty());
    for (path, text) in sources {
        for import in text
            .lines()
            .map(str::trim)
            .filter(|line| line.starts_with("use "))
        {
            let allowed = import.starts_with("use std::")
                || import.starts_with("use super::")
                || import.starts_with("use crate::baseline::literal_types::");
            assert!(
                allowed,
                "forbidden literal import in {}: {import}",
                path.display()
            );
            for forbidden in [
                "fenestra_",
                "reference",
                "runtime",
                "authoring",
                "macros",
                "testkit",
                "candidate",
                "exp_0007",
                "generated",
                "manual",
            ] {
                assert!(
                    !import.contains(forbidden),
                    "forbidden literal dependency {forbidden}"
                );
            }
        }
    }
}

#[test]
fn baseline_has_no_candidate_or_native_dependency_seam() {
    let manifest = source::manifest();
    for forbidden in [
        "euclid",
        "kurbo",
        "fixed",
        "lyon",
        "rstar",
        "tiny-skia",
        "raqote",
        "vello",
        "wgpu",
        "png",
        "image",
        "testkit",
        "authoring",
        "macros",
        "native-spine",
    ] {
        assert!(
            !manifest.contains(forbidden),
            "baseline dependency leaked: {forbidden}"
        );
    }
    assert!(!PathLikeSources::baseline_text().contains("lanes/"));
}

struct PathLikeSources;

impl PathLikeSources {
    fn baseline_text() -> String {
        source::rust_sources("src/baseline")
            .into_iter()
            .map(|(_, text)| text)
            .collect::<Vec<_>>()
            .join("\n")
    }
}
