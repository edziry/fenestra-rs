use super::source::all_source;
use super::surface_support::struct_field_types;

#[test]
fn owned_input_stores_only_the_registered_fields_in_table_order() {
    let source = all_source();
    assert_eq!(
        struct_field_types(&source, "SpatialOwnedInputV2"),
        [
            "SpatialViewportV2",
            "Box<[SpatialNodeV2]>",
            "Box<[SpatialPointV2]>",
            "Box<[SpatialPathVerbV2]>",
            "Box<[SpatialPathV2]>",
            "Box<[SpatialShapeV2]>",
            "Box<[SpatialClipV2]>",
            "Box<[SpatialGradientStopV2]>",
            "Box<[SpatialBrushV2]>",
            "Box<[SpatialImageV2]>",
            "Box<[SpatialPaintV2]>",
            "Box<[SpatialHitV2]>",
            "Box<[SpatialSemanticGeometryV2]>",
        ]
    );
}
