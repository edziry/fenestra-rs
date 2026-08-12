use crate::*;

type InputConstructor<'a> = fn(
    SpatialTopologyInputV2<'a>,
    SpatialGeometryInputV2<'a>,
    SpatialResourceInputV2<'a>,
    SpatialItemInputV2<'a>,
) -> SpatialInputV2<'a>;

#[test]
fn aggregate_input_and_error_function_signatures_are_exact() {
    assert_input_signatures(&());

    let _: fn(SpatialResolveErrorV2) -> SpatialResolveErrorKindV2 = SpatialResolveErrorV2::kind;
    let _: fn(SpatialResolveErrorV2) -> SpatialErrorLocationV2 = SpatialResolveErrorV2::location;
    let _: fn(SpatialResolveErrorV2) -> Option<u128> = SpatialResolveErrorV2::observed;
    let _: fn(SpatialResolveErrorV2) -> Option<u128> = SpatialResolveErrorV2::maximum;
}

fn assert_input_signatures<'a>(_: &'a ()) {
    let _: InputConstructor<'a> = SpatialInputV2::new;
    let _: fn(SpatialInputV2<'a>) -> SpatialTopologyInputV2<'a> = SpatialInputV2::topology;
    let _: fn(SpatialInputV2<'a>) -> SpatialGeometryInputV2<'a> = SpatialInputV2::geometry;
    let _: fn(SpatialInputV2<'a>) -> SpatialResourceInputV2<'a> = SpatialInputV2::resources;
    let _: fn(SpatialInputV2<'a>) -> SpatialItemInputV2<'a> = SpatialInputV2::items;
}

#[test]
fn every_new_all_array_has_its_exact_registered_type() {
    let _: [SpatialColorChannelV2; 4] = SpatialColorChannelV2::ALL;
    let _: [SpatialPathFieldV2; 3] = SpatialPathFieldV2::ALL;
    let _: [SpatialPathVerbFieldV2; 9] = SpatialPathVerbFieldV2::ALL;
    let _: [SpatialShapeFieldV2; 13] = SpatialShapeFieldV2::ALL;
    let _: [SpatialPolygonPointFieldV2; 2] = SpatialPolygonPointFieldV2::ALL;
    let _: [SpatialBrushFieldV2; 12] = SpatialBrushFieldV2::ALL;
    let _: [SpatialGradientStopFieldV2; 5] = SpatialGradientStopFieldV2::ALL;
    let _: [SpatialImageFieldV2; 6] = SpatialImageFieldV2::ALL;
    let _: [SpatialClipFieldV2; 5] = SpatialClipFieldV2::ALL;
    let _: [SpatialPaintFieldV2; 19] = SpatialPaintFieldV2::ALL;
    let _: [SpatialHitFieldV2; 8] = SpatialHitFieldV2::ALL;
    let _: [SpatialSemanticFieldV2; 5] = SpatialSemanticFieldV2::ALL;
    let _: [SpatialOutputTableV2; 5] = SpatialOutputTableV2::ALL;
    let _: [SpatialOutputFieldV2; 25] = SpatialOutputFieldV2::ALL;

    let _: [SpatialKeyedContentTableV2; 5] = SpatialKeyedContentTableV2::ALL;
    let _: [SpatialPayloadTableV2; 3] = SpatialPayloadTableV2::ALL;
    let _: [SpatialContentReferenceV2; 6] = SpatialContentReferenceV2::ALL;
    let _: [SpatialOrderedItemTableV2; 3] = SpatialOrderedItemTableV2::ALL;
    let _: [SpatialPathGrammarErrorV2; 6] = SpatialPathGrammarErrorV2::ALL;
    let _: [SpatialShapeErrorV2; 5] = SpatialShapeErrorV2::ALL;
    let _: [SpatialStrokeErrorV2; 2] = SpatialStrokeErrorV2::ALL;
    let _: [SpatialGradientErrorV2; 5] = SpatialGradientErrorV2::ALL;
    let _: [SpatialImageErrorV2; 9] = SpatialImageErrorV2::ALL;
    let _: [SpatialClipErrorV2; 4] = SpatialClipErrorV2::ALL;
    let _: [SpatialContentErrorKindV2; 52] = SpatialContentErrorKindV2::ALL;
    let _: [SpatialLayoutErrorKindV2; 22] = SpatialLayoutErrorKindV2::ALL;
    let _: [SpatialOutputErrorKindV2; 10] = SpatialOutputErrorKindV2::ALL;
    let _: [SpatialResolveErrorKindV2; 192] = SpatialResolveErrorKindV2::ALL;
}
