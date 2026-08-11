use fenestra_ui_spatial::prototype::{SpatialNodeKeyV2, SpatialPointV2};

use crate::*;

// The five-table function type is the boundary this test intentionally fixes.
#[allow(clippy::type_complexity)]
type GeometryInputConstructor<'a> = fn(
    &'a [SpatialPointV2],
    &'a [SpatialPathVerbV2],
    &'a [SpatialPathV2],
    &'a [SpatialShapeV2],
    &'a [SpatialClipV2],
) -> SpatialGeometryInputV2<'a>;

#[test]
fn geometry_function_signatures_are_exact() {
    let _: fn(u32) -> SpatialPathKeyV2 = SpatialPathKeyV2::new;
    let _: fn(SpatialPathKeyV2) -> u32 = SpatialPathKeyV2::get;
    let _: fn(u32) -> SpatialShapeKeyV2 = SpatialShapeKeyV2::new;
    let _: fn(SpatialShapeKeyV2) -> u32 = SpatialShapeKeyV2::get;
    let _: fn(u32) -> SpatialClipKeyV2 = SpatialClipKeyV2::new;
    let _: fn(SpatialClipKeyV2) -> u32 = SpatialClipKeyV2::get;

    let _: fn(SpatialPathKeyV2, u32, u32) -> SpatialPathV2 = SpatialPathV2::new;
    let _: fn(SpatialPathV2) -> SpatialPathKeyV2 = SpatialPathV2::key;
    let _: fn(SpatialPathV2) -> u32 = SpatialPathV2::verb_start;
    let _: fn(SpatialPathV2) -> u32 = SpatialPathV2::verb_length;
    let _: fn(SpatialShapeKeyV2, SpatialNodeKeyV2, SpatialShapeGeometryV2) -> SpatialShapeV2 =
        SpatialShapeV2::new;
    let _: fn(SpatialShapeV2) -> SpatialShapeKeyV2 = SpatialShapeV2::key;
    let _: fn(SpatialShapeV2) -> SpatialNodeKeyV2 = SpatialShapeV2::owner;
    let _: fn(SpatialShapeV2) -> SpatialShapeGeometryV2 = SpatialShapeV2::geometry;

    let _: fn(
        SpatialClipKeyV2,
        SpatialNodeKeyV2,
        Option<SpatialClipKeyV2>,
        SpatialShapeKeyV2,
        SpatialFillRuleV2,
    ) -> SpatialClipV2 = SpatialClipV2::new;
    let _: fn(SpatialClipV2) -> SpatialClipKeyV2 = SpatialClipV2::key;
    let _: fn(SpatialClipV2) -> SpatialNodeKeyV2 = SpatialClipV2::owner;
    let _: fn(SpatialClipV2) -> Option<SpatialClipKeyV2> = SpatialClipV2::parent;
    let _: fn(SpatialClipV2) -> SpatialShapeKeyV2 = SpatialClipV2::shape;
    let _: fn(SpatialClipV2) -> SpatialFillRuleV2 = SpatialClipV2::fill_rule;

    assert_input_signatures(&());
}

fn assert_input_signatures<'a>(_: &'a ()) {
    let _: GeometryInputConstructor<'a> = SpatialGeometryInputV2::new;
    let _: fn(SpatialGeometryInputV2<'a>) -> &'a [SpatialPointV2] =
        SpatialGeometryInputV2::polygon_points;
    let _: fn(SpatialGeometryInputV2<'a>) -> &'a [SpatialPathVerbV2] =
        SpatialGeometryInputV2::path_verbs;
    let _: fn(SpatialGeometryInputV2<'a>) -> &'a [SpatialPathV2] = SpatialGeometryInputV2::paths;
    let _: fn(SpatialGeometryInputV2<'a>) -> &'a [SpatialShapeV2] = SpatialGeometryInputV2::shapes;
    let _: fn(SpatialGeometryInputV2<'a>) -> &'a [SpatialClipV2] = SpatialGeometryInputV2::clips;
}
