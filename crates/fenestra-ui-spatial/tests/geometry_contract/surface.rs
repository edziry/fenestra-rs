use std::collections::BTreeSet;
use std::fs;

use crate::*;

use super::source::{all_source, source_dir};

const EXPECTED_EXPORTS: [&str; 114] = [
    "Affine2V2",
    "REGISTERED_SPATIAL_LIMITS_V2",
    "SpatialAabbV2",
    "SpatialAffineComponentV2",
    "SpatialAnchorComponentV2",
    "SpatialAnchorTargetKindV2",
    "SpatialAnchorTargetV2",
    "SpatialAnchorV2",
    "SpatialArithmeticOperationV2",
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
    "SpatialHitResultV2",
    "SpatialSemanticOutputRecordV2",
    "SpatialOutputV2",
    "SpatialOwnedInputV2",
    "PreparedSpatialV2",
    "prepare_spatial_v2",
    "SpatialResolvedSnapshotV2",
    "materialize_reference_spatial_v2",
    "resolve_spatial_v2",
    "validate_spatial_output_v2",
];

const NEW_STRUCTS: [&str; 7] = [
    "SpatialClipKeyV2",
    "SpatialClipV2",
    "SpatialGeometryInputV2",
    "SpatialPathKeyV2",
    "SpatialPathV2",
    "SpatialShapeKeyV2",
    "SpatialShapeV2",
];

#[test]
fn geometry_slice_reexports_exactly_the_registered_surface() {
    let source = fs::read_to_string(source_dir().join("lib.rs")).expect("read lib.rs");
    let marker = "pub mod prototype {";
    let start = source.find(marker).expect("prototype module") + marker.len();
    let end = source.rfind('}').expect("prototype end");
    let prototype = &source[start..end];

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
        for name in names.split(',').map(str::trim) {
            if !name.is_empty() {
                assert!(observed.insert(name), "duplicate reexport {name}");
            }
        }
    }
    assert_eq!(observed, EXPECTED_EXPORTS.into_iter().collect());
}

#[test]
fn geometry_methods_constants_and_payload_surfaces_are_exact() {
    let source = all_source();
    for key in ["SpatialPathKeyV2", "SpatialShapeKeyV2", "SpatialClipKeyV2"] {
        assert_eq!(public_methods(&source, key), names(&["new", "get"]));
        assert!(public_constants(&source, key).is_empty());
        assert_const_and_must_use(&source, key, &["new", "get"]);
    }
    assert_surface(
        &source,
        "SpatialPathV2",
        &["new", "key", "verb_start", "verb_length"],
    );
    assert_surface(
        &source,
        "SpatialShapeV2",
        &["new", "key", "owner", "geometry"],
    );
    assert_surface(
        &source,
        "SpatialClipV2",
        &["new", "key", "owner", "parent", "shape", "fill_rule"],
    );
    assert_surface(
        &source,
        "SpatialGeometryInputV2",
        &[
            "new",
            "polygon_points",
            "path_verbs",
            "paths",
            "shapes",
            "clips",
        ],
    );

    for payload in [
        "SpatialPathVerbV2",
        "SpatialShapeGeometryV2",
        "SpatialCoverageV2",
    ] {
        assert!(public_methods(&source, payload).is_empty());
        assert!(public_constants(&source, payload).is_empty());
        assert_no_associated_all(&source, payload);
    }
    for vocabulary in [
        "SpatialPathVerbKindV2",
        "SpatialShapeKindV2",
        "SpatialFillRuleV2",
        "SpatialCoverageKindV2",
    ] {
        assert!(public_methods(&source, vocabulary).is_empty());
        assert_eq!(public_constants(&source, vocabulary), names(&["ALL"]));
    }
}

#[test]
fn geometry_all_arrays_have_the_exact_registered_types() {
    let _: [SpatialPathVerbKindV2; 5] = SpatialPathVerbKindV2::ALL;
    let _: [SpatialShapeKindV2; 4] = SpatialShapeKindV2::ALL;
    let _: [SpatialFillRuleV2; 2] = SpatialFillRuleV2::ALL;
    let _: [SpatialCoverageKindV2; 2] = SpatialCoverageKindV2::ALL;
}

#[test]
fn every_new_geometry_struct_keeps_its_fields_private() {
    let source = all_source();
    for name in NEW_STRUCTS {
        assert_struct_fields_private(&source, name);
    }
}

fn assert_surface(source: &str, type_name: &str, methods: &[&str]) {
    assert_eq!(public_methods(source, type_name), names(methods));
    assert!(public_constants(source, type_name).is_empty());
    assert_const_and_must_use(source, type_name, methods);
}

fn public_methods(source: &str, type_name: &str) -> BTreeSet<String> {
    let mut methods = BTreeSet::new();
    for implementation in implementation_blocks(source, type_name) {
        for line in implementation.lines().map(str::trim) {
            if line.starts_with("pub ") && line.contains("fn ") {
                let suffix = line.split_once("fn ").expect("public method").1;
                let name = suffix.split(['(', '<']).next().expect("method name").trim();
                assert!(methods.insert(name.to_owned()), "duplicate method {name}");
            }
        }
    }
    methods
}

fn public_constants(source: &str, type_name: &str) -> BTreeSet<String> {
    let mut constants = BTreeSet::new();
    for implementation in implementation_blocks(source, type_name) {
        for line in implementation.lines().map(str::trim) {
            let Some(suffix) = line.strip_prefix("pub const ") else {
                continue;
            };
            if suffix.starts_with("fn ") {
                continue;
            }
            let name = suffix
                .split([':', '='])
                .next()
                .expect("constant name")
                .trim();
            assert!(
                constants.insert(name.to_owned()),
                "duplicate constant {name}"
            );
        }
    }
    constants
}

fn assert_const_and_must_use(source: &str, type_name: &str, names: &[&str]) {
    let implementations = implementation_blocks(source, type_name);
    for name in names {
        let marker = format!("pub const fn {name}");
        let mut found = false;
        for implementation in &implementations {
            if let Some(offset) = implementation.find(&marker) {
                assert!(has_must_use(implementation, offset), "{type_name}::{name}");
                found = true;
            }
        }
        assert!(found, "missing const method {type_name}::{name}");
    }
}

fn has_must_use(source: &str, item_offset: usize) -> bool {
    for line in source[..item_offset].lines().rev() {
        let line = line.trim();
        if line == "#[must_use]" || line.starts_with("#[must_use = ") {
            return true;
        }
        if line.is_empty() || line.starts_with("///") || line.starts_with("#[") {
            continue;
        }
        break;
    }
    false
}

fn assert_no_associated_all(source: &str, type_name: &str) {
    for implementation in implementation_blocks(source, type_name) {
        assert!(!implementation.contains("pub const ALL"));
        assert!(!implementation.contains("pub static ALL"));
    }
}

fn assert_struct_fields_private(source: &str, type_name: &str) {
    let marker = format!("pub struct {type_name}");
    let start = source
        .find(&marker)
        .unwrap_or_else(|| panic!("missing {type_name}"));
    let declaration = &source[start..];
    let brace = declaration.find('{');
    let tuple = declaration.find('(');
    match (brace, tuple) {
        (Some(brace), None) => assert_braced_fields_private(declaration, brace),
        (Some(brace), Some(tuple)) if brace < tuple => {
            assert_braced_fields_private(declaration, brace);
        }
        (_, Some(tuple)) => {
            let end = declaration[tuple..].find(';').expect("tuple struct end") + tuple;
            let fields = &declaration[tuple + 1..end];
            assert!(!fields.contains("pub ") && !fields.contains("pub("));
        }
        _ => panic!("unsupported struct {type_name}"),
    }
}

fn assert_braced_fields_private(declaration: &str, brace: usize) {
    let end = declaration[brace..].find('}').expect("struct end") + brace;
    assert!(
        !declaration[brace + 1..end]
            .lines()
            .any(|line| line.trim_start().starts_with("pub"))
    );
}

fn implementation_blocks<'a>(source: &'a str, type_name: &str) -> Vec<&'a str> {
    let mut remaining = source;
    let mut blocks = Vec::new();
    loop {
        let mut line_offset = 0_usize;
        let mut start = None;
        for line in remaining.split_inclusive('\n') {
            let trimmed = line.trim_start();
            if trimmed.starts_with("impl")
                && trimmed.contains(type_name)
                && !trimmed.contains(" for ")
            {
                start = Some(line_offset + line.len() - trimmed.len());
                break;
            }
            line_offset += line.len();
        }
        let Some(start) = start else {
            break;
        };
        let implementation = &remaining[start..];
        let end = balanced_block_end(implementation);
        blocks.push(&implementation[..end]);
        remaining = &implementation[end..];
    }
    blocks
}

fn balanced_block_end(source: &str) -> usize {
    let mut depth = 0_usize;
    for (offset, character) in source.char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return offset + 1;
                }
            }
            _ => {}
        }
    }
    panic!("unterminated impl")
}

fn names(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}
