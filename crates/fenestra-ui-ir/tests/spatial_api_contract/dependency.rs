use super::source::{all_source, read, source_dir, workspace_root};

#[test]
fn symbolic_spatial_ir_keeps_the_exact_dependency_free_boundary() {
    let manifest = read(
        &source_dir()
            .parent()
            .expect("crate root")
            .join("Cargo.toml"),
    );
    for forbidden_header in [
        "[dependencies]",
        "[dev-dependencies]",
        "[build-dependencies]",
        "[target.",
    ] {
        assert!(
            !manifest.contains(forbidden_header),
            "unexpected dependency section {forbidden_header}"
        );
    }

    let lock = read(&workspace_root().join("Cargo.lock"));
    let package = package_section(&lock, "fenestra-ui-ir");
    assert!(!package.contains("dependencies = ["));
}

#[test]
fn symbolic_spatial_ir_imports_no_downstream_or_raw_spatial_identity() {
    let source = all_source();
    for forbidden in [
        "fenestra_ui_layout",
        "fenestra_ui_spatial",
        "fenestra_ui_runtime",
        "fenestra_ui_authoring",
        "fenestra_ui_testkit",
        "SpatialNodeKeyV2",
        "SpatialShapeKeyV2",
        "SpatialBrushKeyV2",
        "SpatialClipKeyV2",
        "SpatialImageKeyV2",
        "SpatialPathKeyV2",
        "RuntimeGeneration",
    ] {
        assert!(
            !source.contains(forbidden),
            "forbidden dependency or raw identity {forbidden}"
        );
    }

    let compact = source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    for forbidden in [
        "pubfnitem_ordinal(",
        "pubconstfnitem_ordinal(",
        "pubitem_ordinal:",
    ] {
        assert!(
            !compact.contains(forbidden),
            "forbidden public raw identity {forbidden}"
        );
    }
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
