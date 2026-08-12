use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const EXPECTED_EXPORTS: [&str; 112] = [
    "Affine2V2",
    "REGISTERED_SPATIAL_LIMITS_V2",
    "SpatialAabbV2",
    "SpatialAffineComponentV2",
    "SpatialAnchorComponentV2",
    "SpatialAnchorTargetKindV2",
    "SpatialAnchorTargetV2",
    "SpatialAnchorV2",
    "SpatialAxisV2",
    "SpatialBrushContentV2",
    "SpatialBrushKeyV2",
    "SpatialBrushKindV2",
    "SpatialBrushV2",
    "SpatialClipKeyV2",
    "SpatialClipV2",
    "SpatialContainerErrorKindV2",
    "SpatialContainerV2",
    "SpatialCoverageKindV2",
    "SpatialCoverageV2",
    "SpatialDependencyErrorKindV2",
    "SpatialErrorLocationV2",
    "SpatialExtentV2",
    "SpatialFillRuleV2",
    "SpatialFreePlacementV2",
    "SpatialGeometryInputV2",
    "SpatialGradientStopV2",
    "SpatialHitV2",
    "SpatialImageDestinationRectV2",
    "SpatialImageKeyV2",
    "SpatialImageSourceRectV2",
    "SpatialImageV2",
    "SpatialInputErrorKindV2",
    "SpatialInputPolicyV2",
    "SpatialItemInputV2",
    "SpatialLayoutDimensionErrorKindV2",
    "SpatialLayoutPlacementV2",
    "SpatialLimitKindV2",
    "SpatialLimitsV2",
    "SpatialLocalTransformV2",
    "SpatialNodeFieldV2",
    "SpatialNodeKeyV2",
    "SpatialNodeV2",
    "SpatialOffsetV2",
    "SpatialPaintContentV2",
    "SpatialPaintKindV2",
    "SpatialPaintV2",
    "SpatialPathKeyV2",
    "SpatialPathV2",
    "SpatialPathVerbKindV2",
    "SpatialPathVerbV2",
    "SpatialPlacementKindV2",
    "SpatialPlacementV2",
    "SpatialPointV2",
    "SpatialResourceInputV2",
    "SpatialRgba8V2",
    "SpatialScalarV2",
    "SpatialSemanticGeometryV2",
    "SpatialShapeGeometryV2",
    "SpatialShapeKeyV2",
    "SpatialShapeKindV2",
    "SpatialShapeV2",
    "SpatialTopologyInputV2",
    "SpatialTransformErrorKindV2",
    "SpatialTransformScalarFieldV2",
    "SpatialTransformStageV2",
    "SpatialViewportV2",
    "SpatialArithmeticOperationV2",
    "round_ratio_v2",
    "SpatialInputV2",
    "SpatialColorChannelV2",
    "SpatialPathFieldV2",
    "SpatialPathVerbFieldV2",
    "SpatialShapeFieldV2",
    "SpatialPolygonPointFieldV2",
    "SpatialBrushFieldV2",
    "SpatialGradientStopFieldV2",
    "SpatialImageFieldV2",
    "SpatialClipFieldV2",
    "SpatialPaintFieldV2",
    "SpatialHitFieldV2",
    "SpatialSemanticFieldV2",
    "SpatialOutputTableV2",
    "SpatialOutputFieldV2",
    "SpatialKeyedContentTableV2",
    "SpatialPayloadTableV2",
    "SpatialContentReferenceV2",
    "SpatialOrderedItemTableV2",
    "SpatialPathGrammarErrorV2",
    "SpatialShapeErrorV2",
    "SpatialStrokeErrorV2",
    "SpatialGradientErrorV2",
    "SpatialImageErrorV2",
    "SpatialClipErrorV2",
    "SpatialContentErrorKindV2",
    "SpatialLayoutErrorKindV2",
    "SpatialOutputErrorKindV2",
    "SpatialResolveErrorKindV2",
    "SpatialResolveErrorV2",
    "SpatialOutputAabbV2",
    "SpatialPaintOutputReferenceV2",
    "SpatialGeometryOutputRecordV2",
    "SpatialClipOutputRecordV2",
    "SpatialPaintOutputRecordV2",
    "SpatialHitOutputRecordV2",
    "SpatialSemanticOutputRecordV2",
    "SpatialOutputV2",
    "SpatialOwnedInputV2",
    "PreparedSpatialV2",
    "prepare_spatial_v2",
    "SpatialResolvedSnapshotV2",
    "materialize_reference_spatial_v2",
    "resolve_spatial_v2",
];
const EXPECTED_STRUCTS: [&str; 47] = [
    "Affine2V2",
    "SpatialAabbV2",
    "SpatialAnchorV2",
    "SpatialBrushKeyV2",
    "SpatialBrushV2",
    "SpatialClipKeyV2",
    "SpatialClipV2",
    "SpatialContainerV2",
    "SpatialFreePlacementV2",
    "SpatialGeometryInputV2",
    "SpatialGradientStopV2",
    "SpatialHitV2",
    "SpatialImageDestinationRectV2",
    "SpatialImageKeyV2",
    "SpatialImageSourceRectV2",
    "SpatialImageV2",
    "SpatialItemInputV2",
    "SpatialLayoutPlacementV2",
    "SpatialLimitsV2",
    "SpatialLocalTransformV2",
    "SpatialNodeKeyV2",
    "SpatialNodeV2",
    "SpatialOffsetV2",
    "SpatialPaintV2",
    "SpatialPathKeyV2",
    "SpatialPathV2",
    "SpatialPointV2",
    "SpatialResourceInputV2",
    "SpatialRgba8V2",
    "SpatialScalarV2",
    "SpatialSemanticGeometryV2",
    "SpatialShapeKeyV2",
    "SpatialShapeV2",
    "SpatialTopologyInputV2",
    "SpatialViewportV2",
    "SpatialInputV2",
    "SpatialResolveErrorV2",
    "SpatialOutputAabbV2",
    "SpatialGeometryOutputRecordV2",
    "SpatialClipOutputRecordV2",
    "SpatialPaintOutputRecordV2",
    "SpatialHitOutputRecordV2",
    "SpatialSemanticOutputRecordV2",
    "SpatialOutputV2",
    "SpatialOwnedInputV2",
    "PreparedSpatialV2",
    "SpatialResolvedSnapshotV2",
];

#[test]
fn prototype_reexports_only_the_registered_first_slice() {
    let source = read(&source_dir().join("lib.rs"));
    let all_source = all_source();
    for forbidden in ["include!", "#[macro_export]"] {
        assert!(
            !all_source.contains(forbidden),
            "unexpected API form {forbidden}"
        );
    }
    let marker = "pub mod prototype {";
    assert!(source.contains("#[doc(hidden)]\npub mod prototype {"));
    let marker_offset = source.find(marker).expect("missing prototype module");
    let before = &source[..marker_offset];
    assert!(!before.lines().any(is_public_line));

    let prototype = &source[marker_offset + marker.len()..source.len() - 2];
    for forbidden in [" as ", "::*"] {
        assert!(
            !prototype.contains(forbidden),
            "unexpected API form {forbidden}"
        );
    }
    assert!(
        prototype
            .lines()
            .filter(|line| is_public_line(line))
            .all(|line| line.trim_start().starts_with("pub use crate::"))
    );

    let mut observed = BTreeSet::new();
    for item in prototype.split("pub use crate::").skip(1) {
        let names = if let Some(list_start) = item.find("::{") {
            let list_end = item.find("};").expect("terminated grouped reexport");
            &item[list_start + 3..list_end]
        } else {
            let item_end = item.find(';').expect("terminated singleton reexport");
            item[..item_end]
                .rsplit("::")
                .next()
                .expect("singleton name")
        };
        for name in names.split(',') {
            let name = name.trim();
            if !name.is_empty() {
                assert!(observed.insert(name), "duplicate reexport");
            }
        }
    }
    assert_eq!(observed, EXPECTED_EXPORTS.into_iter().collect());
}

#[test]
fn payload_and_indexed_enums_have_no_fieldless_all_array() {
    let sources = all_source();

    for type_name in [
        "SpatialAnchorTargetV2",
        "SpatialPlacementV2",
        "SpatialErrorLocationV2",
    ] {
        assert_no_associated_all(&sources, type_name);
    }
}

#[test]
fn every_first_slice_struct_keeps_its_fields_private() {
    let mut structs = BTreeSet::new();
    let source = all_source();
    let lines: Vec<_> = source.lines().collect();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index].trim();
        if !line.starts_with("pub struct ") {
            index += 1;
            continue;
        }
        let name = line["pub struct ".len()..]
            .split(['<', '(', '{'])
            .next()
            .expect("struct name")
            .trim();
        assert!(structs.insert(name.to_owned()), "duplicate struct {name}");
        if line.ends_with(';') {
            assert!(
                !line.contains("(pub ") && !line.contains("(pub("),
                "public tuple field"
            );
            index += 1;
            continue;
        }
        index += 1;
        while index < lines.len() && lines[index].trim() != "}" {
            assert!(
                !is_public_line(lines[index]),
                "public field: {}",
                lines[index]
            );
            index += 1;
        }
        index += 1;
    }
    assert_eq!(
        structs,
        EXPECTED_STRUCTS.into_iter().map(str::to_owned).collect()
    );
}

fn is_public_line(line: &str) -> bool {
    let line = line.trim_start();
    line.starts_with("pub ") || line.starts_with("pub(")
}

fn assert_no_associated_all(source: &str, type_name: &str) {
    let marker = format!("impl {type_name} {{");
    let mut remaining = source;
    while let Some(start) = remaining.find(&marker) {
        let implementation = &remaining[start..];
        let mut depth = 0_usize;
        let mut end = implementation.len();
        for (offset, character) in implementation.char_indices() {
            match character {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = offset + character.len_utf8();
                        break;
                    }
                }
                _ => {}
            }
        }
        assert!(
            !implementation[..end].contains("pub const ALL"),
            "unexpected {type_name}::ALL"
        );
        remaining = &implementation[end..];
    }
}

fn source_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn all_source() -> String {
    let mut paths = Vec::new();
    collect_rust_sources(&source_dir(), &mut paths);
    paths.sort();
    paths
        .into_iter()
        .map(|path| read(&path))
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

fn read(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}
