use super::*;

mod path;
mod polygon;
mod primitives;
mod projection;

const STROKE_SOURCE: GeometryK1StrokeSource = GeometryK1StrokeSource::Paint { index: 13 };

fn validated_stroke(width: i64) -> ValidatedStrokeK1 {
    expect_valid(validate_stroke_k1(STROKE_SOURCE, scalar(width)))
}

fn expanded_bounds(derived: &DerivedLocalBoundsK3, width: i64) -> SpatialAabbV2 {
    match stroke_bounds_k3(derived, STROKE_SOURCE, validated_stroke(width)) {
        Ok(bounds) => bounds,
        Err(error) => panic!("expected K3 stroke bounds success, got {error:?}"),
    }
}

fn rect_stroke_contains(
    origin: SpatialPointV2,
    width: i64,
    height: i64,
    stroke_width: i64,
    query: SpatialPointV2,
) -> bool {
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
    let bounds = expanded_bounds(&derived, stroke_width);
    let rect = expect_valid(validate_rect_k1(
        SHAPE_INDEX,
        origin,
        scalar(width),
        scalar(height),
    ));
    let stroke = validated_stroke(stroke_width);
    rect_round_stroke_contains_k5(rect, bounds, stroke, query)
}

fn circle_stroke_contains(
    center: SpatialPointV2,
    radius: i64,
    stroke_width: i64,
    query: SpatialPointV2,
) -> bool {
    let bounds_circle = expect_valid(validate_circle_k1(SHAPE_INDEX, center, scalar(radius)));
    let derived = match derive_circle_bounds_k3(SHAPE_INDEX, bounds_circle) {
        Ok(derived) => derived,
        Err(error) => panic!("expected circle K3 success, got {error:?}"),
    };
    let bounds = expanded_bounds(&derived, stroke_width);
    let circle = expect_valid(validate_circle_k1(SHAPE_INDEX, center, scalar(radius)));
    let stroke = validated_stroke(stroke_width);
    circle_round_stroke_contains_k5(circle, bounds, stroke, query)
}

fn polygon_stroke_contains(
    points: &[SpatialPointV2],
    stroke_width: i64,
    query: SpatialPointV2,
) -> bool {
    let bounds_polygon = expect_valid(validate_polygon_k1(
        SHAPE_INDEX,
        points,
        POLYGON_POINT_MAXIMUM,
    ));
    let derived = derive_polygon_bounds_k3(bounds_polygon);
    let bounds = expanded_bounds(&derived, stroke_width);
    let polygon = expect_valid(validate_polygon_k1(
        SHAPE_INDEX,
        points,
        POLYGON_POINT_MAXIMUM,
    ));
    let stroke = validated_stroke(stroke_width);
    polygon_round_stroke_contains_k5(polygon, bounds, stroke, query)
}

fn flattened_path_with_stroke_bounds(
    verbs: &[SpatialPathVerbV2],
    stroke_width: i64,
) -> (FlattenedPathK2, SpatialAabbV2, ValidatedStrokeK1) {
    let bounds_path = expect_valid(validate_path_k1(PATH_INDEX, verbs, 0, PATH_SUBPATH_MAXIMUM));
    let derived = derive_path_bounds_k3(bounds_path);
    let flatten_path = expect_valid(validate_path_k1(PATH_INDEX, verbs, 0, PATH_SUBPATH_MAXIMUM));
    let flattened = match flatten_path_k2(PATH_INDEX, flatten_path, 0, usize::MAX, usize::MAX) {
        Ok(flattened) => flattened,
        Err(error) => panic!("expected K2 flattening success, got {error:?}"),
    };
    let bounds = expanded_bounds(&derived, stroke_width);
    let stroke = validated_stroke(stroke_width);
    (flattened, bounds, stroke)
}

fn path_stroke_contains(
    verbs: &[SpatialPathVerbV2],
    stroke_width: i64,
    query: SpatialPointV2,
) -> bool {
    let (path, bounds, stroke) = flattened_path_with_stroke_bounds(verbs, stroke_width);
    path_round_stroke_contains_k5(&path, bounds, stroke, query)
}
