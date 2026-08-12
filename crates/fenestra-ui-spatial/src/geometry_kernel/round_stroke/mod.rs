use crate::aabb::SpatialAabbV2;
use crate::model::{SpatialPointV2, SpatialScalarV2};

use super::coverage::bounds_contains;
use super::flatten::FlattenedPathK2;
use super::shape::{ValidatedCircleK1, ValidatedPolygonK1, ValidatedRectK1};
use super::stroke::ValidatedStrokeK1;

mod segment;

use segment::segment_round_stroke_contains;

pub(crate) fn rect_round_stroke_contains_k5(
    rect: ValidatedRectK1,
    bounds: SpatialAabbV2,
    stroke: ValidatedStrokeK1,
    query: SpatialPointV2,
) -> bool {
    if !bounds_contains(bounds, query) {
        return false;
    }

    let top_left = rect.origin();
    let top_right = SpatialPointV2::new(
        add_scalar(rect.origin().x(), rect.width()),
        rect.origin().y(),
    );
    let bottom_right =
        SpatialPointV2::new(top_right.x(), add_scalar(rect.origin().y(), rect.height()));
    let bottom_left = SpatialPointV2::new(top_left.x(), bottom_right.y());
    let width = i128::from(stroke.width().raw());

    for (start, end) in [
        (top_left, top_right),
        (top_right, bottom_right),
        (bottom_right, bottom_left),
        (bottom_left, top_left),
    ] {
        if segment_round_stroke_contains(start, end, width, query) {
            return true;
        }
    }
    false
}

pub(crate) fn circle_round_stroke_contains_k5(
    circle: ValidatedCircleK1,
    bounds: SpatialAabbV2,
    stroke: ValidatedStrokeK1,
    query: SpatialPointV2,
) -> bool {
    if !bounds_contains(bounds, query) {
        return false;
    }

    let dx = i128::from(query.x().raw()) - i128::from(circle.center().x().raw());
    let dy = i128::from(query.y().raw()) - i128::from(circle.center().y().raw());
    let four_distance_squared = 4 * (dx * dx + dy * dy);
    let diameter = 2 * i128::from(circle.radius().raw());
    let width = i128::from(stroke.width().raw());
    let inner = (diameter - width).max(0);
    let outer = diameter + width;
    inner * inner <= four_distance_squared && four_distance_squared <= outer * outer
}

pub(crate) fn polygon_round_stroke_contains_k5(
    polygon: ValidatedPolygonK1<'_>,
    bounds: SpatialAabbV2,
    stroke: ValidatedStrokeK1,
    query: SpatialPointV2,
) -> bool {
    if !bounds_contains(bounds, query) {
        return false;
    }

    let width = i128::from(stroke.width().raw());
    let points = polygon.points();
    for pair in points.windows(2) {
        if segment_round_stroke_contains(pair[0], pair[1], width, query) {
            return true;
        }
    }
    segment_round_stroke_contains(
        *points.last().expect("K1 polygon proof is nonempty"),
        points[0],
        width,
        query,
    )
}

pub(crate) fn path_round_stroke_contains_k5(
    path: &FlattenedPathK2,
    bounds: SpatialAabbV2,
    stroke: ValidatedStrokeK1,
    query: SpatialPointV2,
) -> bool {
    if !bounds_contains(bounds, query) {
        return false;
    }

    let width = i128::from(stroke.width().raw());
    for subpath in path.subpaths().iter().copied() {
        let start = subpath.point_start();
        let end = start + subpath.point_length();
        for pair in path.points()[start..end].windows(2) {
            if segment_round_stroke_contains(pair[0], pair[1], width, query) {
                return true;
            }
        }
    }
    false
}

fn add_scalar(left: SpatialScalarV2, right: SpatialScalarV2) -> SpatialScalarV2 {
    let raw = i128::from(left.raw()) + i128::from(right.raw());
    SpatialScalarV2::new(raw as i64)
}
