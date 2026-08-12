use std::collections::BTreeSet;
use std::fs;

use super::source::{all_source, source_dir};
use super::surface_support::{
    assert_const_and_must_use, assert_struct_fields_private, names, public_constants,
    public_methods,
};

const EXPECTED_EXPORTS: [&str; 109] = [
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
    "SpatialSemanticOutputRecordV2",
    "SpatialOutputV2",
    "SpatialOwnedInputV2",
    "PreparedSpatialV2",
    "prepare_spatial_v2",
];

#[test]
fn raw_output_slice_has_exact_explicit_prototype_exports() {
    let source = fs::read_to_string(source_dir().join("lib.rs")).expect("read lib.rs");
    let marker = "pub mod prototype {";
    let start = source.find(marker).expect("prototype module") + marker.len();
    let end = source.rfind('}').expect("prototype end");
    let prototype = &source[start..end];

    let mut observed = BTreeSet::new();
    for item in prototype.split("pub use crate::").skip(1) {
        let exported = if let Some(list_start) = item.find("::{") {
            let list_end = item.find("};").expect("terminated grouped reexport");
            &item[list_start + 3..list_end]
        } else {
            let item_end = item.find(';').expect("terminated singleton reexport");
            item[..item_end]
                .rsplit("::")
                .next()
                .expect("singleton name")
        };
        for name in exported
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
        {
            assert!(observed.insert(name), "duplicate reexport {name}");
        }
    }
    assert_eq!(observed, EXPECTED_EXPORTS.into_iter().collect());
}

#[test]
fn raw_output_struct_fields_are_private_and_surfaces_are_exact() {
    let source = all_source();
    for type_name in [
        "SpatialOutputAabbV2",
        "SpatialGeometryOutputRecordV2",
        "SpatialClipOutputRecordV2",
        "SpatialPaintOutputRecordV2",
        "SpatialHitOutputRecordV2",
        "SpatialSemanticOutputRecordV2",
        "SpatialOutputV2",
    ] {
        assert_struct_fields_private(&source, type_name);
        assert!(public_constants(&source, type_name).is_empty());
    }

    assert_surface(
        &source,
        "SpatialOutputAabbV2",
        &["new", "is_empty", "min_x", "min_y", "max_x", "max_y"],
    );
    assert_surface(
        &source,
        "SpatialGeometryOutputRecordV2",
        &[
            "new",
            "key",
            "base_x",
            "base_y",
            "base_width",
            "base_height",
            "world_from_local",
            "world_determinant",
            "world_aabb",
        ],
    );
    assert_surface(
        &source,
        "SpatialClipOutputRecordV2",
        &[
            "new",
            "key",
            "world_from_local",
            "world_determinant",
            "primitive_world_aabb",
            "owner",
            "parent",
            "shape",
        ],
    );
    assert_surface(
        &source,
        "SpatialPaintOutputRecordV2",
        &[
            "new",
            "key",
            "world_from_local",
            "world_determinant",
            "world_aabb",
            "owner",
            "reference",
            "clip",
            "stack_ordinal",
            "item_ordinal",
        ],
    );
    for type_name in ["SpatialHitOutputRecordV2", "SpatialSemanticOutputRecordV2"] {
        assert_surface(
            &source,
            type_name,
            &[
                "new",
                "key",
                "world_from_local",
                "world_determinant",
                "world_aabb",
                "owner",
                "shape",
                "clip",
                "stack_ordinal",
                "item_ordinal",
            ],
        );
    }
    assert_surface(
        &source,
        "SpatialOutputV2",
        &["new", "geometry", "clips", "paints", "hits", "semantics"],
    );
}

#[test]
fn paint_reference_enum_has_only_its_two_payload_variants() {
    let source = all_source();
    assert_eq!(
        source
            .matches("pub enum SpatialPaintOutputReferenceV2")
            .count(),
        1
    );
    assert!(public_methods(&source, "SpatialPaintOutputReferenceV2").is_empty());
    assert!(public_constants(&source, "SpatialPaintOutputReferenceV2").is_empty());
}

fn assert_surface(source: &str, type_name: &str, methods: &[&str]) {
    assert_eq!(public_methods(source, type_name), names(methods));
    assert_const_and_must_use(source, type_name, methods, methods);
}
