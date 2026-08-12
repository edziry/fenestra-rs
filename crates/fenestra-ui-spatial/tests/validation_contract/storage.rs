use super::source::all_source;
use super::surface_support::{assert_struct_fields_private, struct_field_types};

#[test]
fn aggregate_input_and_resolver_error_store_only_registered_fields() {
    let source = all_source();

    assert_eq!(
        struct_field_types(&source, "SpatialInputV2"),
        [
            "SpatialTopologyInputV2<'a>",
            "SpatialGeometryInputV2<'a>",
            "SpatialResourceInputV2<'a>",
            "SpatialItemInputV2<'a>",
        ]
    );
    assert_eq!(
        struct_field_types(&source, "SpatialResolveErrorV2"),
        [
            "SpatialResolveErrorKindV2",
            "SpatialErrorLocationV2",
            "Option<u128>",
            "Option<u128>",
        ]
    );

    assert_struct_fields_private(&source, "SpatialInputV2");
    assert_struct_fields_private(&source, "SpatialResolveErrorV2");
}
