use super::source::all_source;
use super::surface_support::struct_field_types;

#[test]
fn raw_content_records_store_only_the_registered_field_types() {
    let source = all_source();
    let expected = [
        ("SpatialBrushKeyV2", vec!["u32"]),
        ("SpatialImageKeyV2", vec!["u32"]),
        ("SpatialRgba8V2", vec!["u8", "u8", "u8", "u8"]),
        ("SpatialGradientStopV2", vec!["u16", "SpatialRgba8V2"]),
        (
            "SpatialBrushV2",
            vec!["SpatialBrushKeyV2", "SpatialBrushContentV2"],
        ),
        (
            "SpatialImageV2",
            vec!["SpatialImageKeyV2", "u32", "u32", "u32", "Box<[u8]>"],
        ),
        ("SpatialImageSourceRectV2", vec!["u32", "u32", "u32", "u32"]),
        (
            "SpatialImageDestinationRectV2",
            vec![
                "SpatialScalarV2",
                "SpatialScalarV2",
                "SpatialScalarV2",
                "SpatialScalarV2",
            ],
        ),
        (
            "SpatialPaintV2",
            vec!["SpatialNodeKeyV2", "u32", "SpatialPaintContentV2"],
        ),
        (
            "SpatialHitV2",
            vec![
                "SpatialNodeKeyV2",
                "u32",
                "SpatialCoverageV2",
                "Option<SpatialClipKeyV2>",
                "SpatialInputPolicyV2",
            ],
        ),
        (
            "SpatialSemanticGeometryV2",
            vec![
                "SpatialNodeKeyV2",
                "u32",
                "SpatialShapeKeyV2",
                "SpatialFillRuleV2",
                "Option<SpatialClipKeyV2>",
            ],
        ),
        (
            "SpatialResourceInputV2",
            vec![
                "&'a[SpatialGradientStopV2]",
                "&'a[SpatialBrushV2]",
                "&'a[SpatialImageV2]",
            ],
        ),
        (
            "SpatialItemInputV2",
            vec![
                "&'a[SpatialPaintV2]",
                "&'a[SpatialHitV2]",
                "&'a[SpatialSemanticGeometryV2]",
            ],
        ),
    ];

    for (name, fields) in expected {
        assert_eq!(struct_field_types(&source, name), fields, "{name} storage");
    }
}
