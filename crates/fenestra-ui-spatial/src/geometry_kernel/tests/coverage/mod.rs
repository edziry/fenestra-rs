use super::*;

use crate::coverage::SpatialFillRuleV2;

mod path;
mod polygon;
mod primitives;

fn rect_contains(origin: SpatialPointV2, width: i64, height: i64, query: SpatialPointV2) -> bool {
    let bounds_rect = expect_valid(validate_rect_k1(
        SHAPE_INDEX,
        origin,
        scalar(width),
        scalar(height),
    ));
    let derived = match derive_rect_bounds_k3(SHAPE_INDEX, bounds_rect) {
        Ok(derived) => derived,
        Err(error) => panic!("expected rect K3 success, got {error:?}"),
    };
    let rect = expect_valid(validate_rect_k1(
        SHAPE_INDEX,
        origin,
        scalar(width),
        scalar(height),
    ));
    rect_fill_contains_k4(rect, fill_bounds_k3(&derived), query)
}

fn circle_contains(center: SpatialPointV2, radius: i64, query: SpatialPointV2) -> bool {
    let bounds_circle = expect_valid(validate_circle_k1(SHAPE_INDEX, center, scalar(radius)));
    let derived = match derive_circle_bounds_k3(SHAPE_INDEX, bounds_circle) {
        Ok(derived) => derived,
        Err(error) => panic!("expected circle K3 success, got {error:?}"),
    };
    let circle = expect_valid(validate_circle_k1(SHAPE_INDEX, center, scalar(radius)));
    circle_fill_contains_k4(circle, fill_bounds_k3(&derived), query)
}

fn polygon_contains(
    points: &[SpatialPointV2],
    rule: SpatialFillRuleV2,
    query: SpatialPointV2,
) -> bool {
    let bounds_polygon = expect_valid(validate_polygon_k1(
        SHAPE_INDEX,
        points,
        POLYGON_POINT_MAXIMUM,
    ));
    let derived = derive_polygon_bounds_k3(bounds_polygon);
    let polygon = expect_valid(validate_polygon_k1(
        SHAPE_INDEX,
        points,
        POLYGON_POINT_MAXIMUM,
    ));
    polygon_fill_contains_k4(polygon, fill_bounds_k3(&derived), rule, query)
}

fn flattened_path_with_bounds(verbs: &[SpatialPathVerbV2]) -> (FlattenedPathK2, SpatialAabbV2) {
    let bounds_path = expect_valid(validate_path_k1(PATH_INDEX, verbs, 0, PATH_SUBPATH_MAXIMUM));
    let derived = derive_path_bounds_k3(bounds_path);
    let flatten_path = expect_valid(validate_path_k1(PATH_INDEX, verbs, 0, PATH_SUBPATH_MAXIMUM));
    let flattened = match flatten_path_k2(PATH_INDEX, flatten_path, 0, usize::MAX, usize::MAX) {
        Ok(flattened) => flattened,
        Err(error) => panic!("expected K2 flattening success, got {error:?}"),
    };
    (flattened, fill_bounds_k3(&derived))
}

fn path_contains(
    verbs: &[SpatialPathVerbV2],
    rule: SpatialFillRuleV2,
    query: SpatialPointV2,
) -> bool {
    let (flattened, bounds) = flattened_path_with_bounds(verbs);
    path_fill_contains_k4(&flattened, bounds, rule, query)
}
