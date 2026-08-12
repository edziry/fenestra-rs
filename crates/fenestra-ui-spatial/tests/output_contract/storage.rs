use super::source::all_source;
use super::surface_support::struct_field_types;

#[test]
fn raw_output_records_store_only_the_registered_field_types() {
    let source = all_source();
    let expected = [
        (
            "SpatialOutputAabbV2",
            vec![
                "bool",
                "SpatialScalarV2",
                "SpatialScalarV2",
                "SpatialScalarV2",
                "SpatialScalarV2",
            ],
        ),
        (
            "SpatialGeometryOutputRecordV2",
            vec![
                "SpatialNodeKeyV2",
                "SpatialScalarV2",
                "SpatialScalarV2",
                "SpatialScalarV2",
                "SpatialScalarV2",
                "Affine2V2",
                "i128",
                "SpatialOutputAabbV2",
            ],
        ),
        (
            "SpatialClipOutputRecordV2",
            vec![
                "SpatialClipKeyV2",
                "Affine2V2",
                "i128",
                "SpatialOutputAabbV2",
                "SpatialNodeKeyV2",
                "Option<SpatialClipKeyV2>",
                "SpatialShapeKeyV2",
            ],
        ),
        (
            "SpatialPaintOutputRecordV2",
            item_fields("SpatialPaintOutputReferenceV2"),
        ),
        ("SpatialHitOutputRecordV2", item_fields("SpatialShapeKeyV2")),
        (
            "SpatialSemanticOutputRecordV2",
            item_fields("SpatialShapeKeyV2"),
        ),
        (
            "SpatialOutputV2",
            vec![
                "&'a[SpatialGeometryOutputRecordV2]",
                "&'a[SpatialClipOutputRecordV2]",
                "&'a[SpatialPaintOutputRecordV2]",
                "&'a[SpatialHitOutputRecordV2]",
                "&'a[SpatialSemanticOutputRecordV2]",
            ],
        ),
    ];

    for (name, fields) in expected {
        assert_eq!(struct_field_types(&source, name), fields, "{name} storage");
    }
}

fn item_fields(reference: &'static str) -> Vec<&'static str> {
    vec![
        "u32",
        "Affine2V2",
        "i128",
        "SpatialOutputAabbV2",
        "SpatialNodeKeyV2",
        reference,
        "Option<SpatialClipKeyV2>",
        "u32",
        "u32",
    ]
}
