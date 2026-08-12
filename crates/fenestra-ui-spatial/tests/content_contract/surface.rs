use std::collections::BTreeSet;
use std::fs;

use crate::*;

use super::source::{all_source, source_dir};
use super::surface_support::{
    assert_const_and_must_use, assert_struct_fields_private, implementation_blocks, names,
    public_constants, public_methods,
};

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

const NEW_STRUCTS: [&str; 13] = [
    "SpatialBrushKeyV2",
    "SpatialImageKeyV2",
    "SpatialRgba8V2",
    "SpatialGradientStopV2",
    "SpatialBrushV2",
    "SpatialImageV2",
    "SpatialImageSourceRectV2",
    "SpatialImageDestinationRectV2",
    "SpatialPaintV2",
    "SpatialHitV2",
    "SpatialSemanticGeometryV2",
    "SpatialResourceInputV2",
    "SpatialItemInputV2",
];

#[test]
fn content_slice_reexports_exactly_the_registered_surface() {
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
fn content_methods_and_constants_are_exact() {
    let source = all_source();
    assert_surface(&source, "SpatialBrushKeyV2", &["new", "get"]);
    assert_surface(&source, "SpatialImageKeyV2", &["new", "get"]);
    assert_surface(&source, "SpatialRgba8V2", &["new", "r", "g", "b", "a"]);
    assert_surface(
        &source,
        "SpatialGradientStopV2",
        &["new", "offset", "color"],
    );
    assert_surface(&source, "SpatialBrushV2", &["new", "key", "content"]);
    assert_surface(
        &source,
        "SpatialImageSourceRectV2",
        &["new", "x", "y", "width", "height"],
    );
    assert_surface(
        &source,
        "SpatialImageDestinationRectV2",
        &["new", "x", "y", "width", "height"],
    );
    assert_surface(
        &source,
        "SpatialPaintV2",
        &["new", "owner", "item_ordinal", "content"],
    );
    assert_surface(
        &source,
        "SpatialHitV2",
        &[
            "new",
            "owner",
            "item_ordinal",
            "coverage",
            "clip",
            "input_policy",
        ],
    );
    assert_surface(
        &source,
        "SpatialSemanticGeometryV2",
        &["new", "owner", "item_ordinal", "shape", "fill_rule", "clip"],
    );
    assert_surface(
        &source,
        "SpatialResourceInputV2",
        &["new", "gradient_stops", "brushes", "images"],
    );
    assert_surface(
        &source,
        "SpatialItemInputV2",
        &["new", "paint_items", "hit_items", "semantic_items"],
    );

    let image_methods = ["new", "key", "width", "height", "stride", "bytes"];
    assert_eq!(
        public_methods(&source, "SpatialImageV2"),
        names(&image_methods)
    );
    assert!(public_constants(&source, "SpatialImageV2").is_empty());
    assert_const_and_must_use(
        &source,
        "SpatialImageV2",
        &["key", "width", "height", "stride"],
        &image_methods,
    );

    for payload in ["SpatialBrushContentV2", "SpatialPaintContentV2"] {
        assert!(public_methods(&source, payload).is_empty());
        assert!(public_constants(&source, payload).is_empty());
        assert_no_associated_all(&source, payload);
    }
    for vocabulary in [
        "SpatialBrushKindV2",
        "SpatialPaintKindV2",
        "SpatialInputPolicyV2",
    ] {
        assert!(public_methods(&source, vocabulary).is_empty());
        assert_eq!(public_constants(&source, vocabulary), names(&["ALL"]));
    }
}

#[test]
fn content_all_arrays_have_the_exact_registered_types() {
    let _: [SpatialBrushKindV2; 2] = SpatialBrushKindV2::ALL;
    let _: [SpatialPaintKindV2; 2] = SpatialPaintKindV2::ALL;
    let _: [SpatialInputPolicyV2; 2] = SpatialInputPolicyV2::ALL;
}

#[test]
fn every_new_content_struct_keeps_its_fields_private() {
    let source = all_source();
    for name in NEW_STRUCTS {
        assert_struct_fields_private(&source, name);
    }
}

fn assert_surface(source: &str, type_name: &str, methods: &[&str]) {
    assert_eq!(public_methods(source, type_name), names(methods));
    assert!(public_constants(source, type_name).is_empty());
    assert_const_and_must_use(source, type_name, methods, methods);
}

fn assert_no_associated_all(source: &str, type_name: &str) {
    for implementation in implementation_blocks(source, type_name) {
        assert!(!implementation.contains("pub const ALL"));
        assert!(!implementation.contains("pub static ALL"));
    }
}
