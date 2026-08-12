use crate::aabb::SpatialAabbV2;
use crate::coverage::SpatialFillRuleV2;
use crate::model::SpatialPointV2;

use super::flatten::FlattenedPathK2;
use super::shape::{ValidatedCircleK1, ValidatedPolygonK1, ValidatedRectK1};

mod segments;

use segments::FillAccumulator;
pub(crate) use segments::bounds_contains;

pub(crate) fn rect_fill_contains_k4(
    rect: ValidatedRectK1,
    bounds: SpatialAabbV2,
    query: SpatialPointV2,
) -> bool {
    if !bounds_contains(bounds, query) {
        return false;
    }

    let query_x = i128::from(query.x().raw());
    let query_y = i128::from(query.y().raw());
    let min_x = i128::from(rect.origin().x().raw());
    let min_y = i128::from(rect.origin().y().raw());
    let max_x = min_x + i128::from(rect.width().raw());
    let max_y = min_y + i128::from(rect.height().raw());
    query_x >= min_x && query_x < max_x && query_y >= min_y && query_y < max_y
}

pub(crate) fn circle_fill_contains_k4(
    circle: ValidatedCircleK1,
    bounds: SpatialAabbV2,
    query: SpatialPointV2,
) -> bool {
    if !bounds_contains(bounds, query) || circle.radius().raw() == 0 {
        return false;
    }

    let dx = i128::from(query.x().raw()) - i128::from(circle.center().x().raw());
    let dy = i128::from(query.y().raw()) - i128::from(circle.center().y().raw());
    let radius = i128::from(circle.radius().raw());
    dx * dx + dy * dy <= radius * radius
}

pub(crate) fn polygon_fill_contains_k4(
    polygon: ValidatedPolygonK1<'_>,
    bounds: SpatialAabbV2,
    rule: SpatialFillRuleV2,
    query: SpatialPointV2,
) -> bool {
    if !bounds_contains(bounds, query) {
        return false;
    }

    let points = polygon.points();
    let mut coverage = FillAccumulator::new(query);
    for pair in points.windows(2) {
        if coverage.add_segment(pair[0], pair[1]) {
            return true;
        }
    }
    if coverage.add_segment(
        *points.last().expect("K1 polygon proof is nonempty"),
        points[0],
    ) {
        return true;
    }
    coverage.matches_rule(rule)
}

pub(crate) fn path_fill_contains_k4(
    path: &FlattenedPathK2,
    bounds: SpatialAabbV2,
    rule: SpatialFillRuleV2,
    query: SpatialPointV2,
) -> bool {
    if !bounds_contains(bounds, query) {
        return false;
    }

    let mut coverage = FillAccumulator::new(query);
    for subpath in path.subpaths().iter().copied() {
        let start = subpath.point_start();
        let end = start + subpath.point_length();
        let points = &path.points()[start..end];
        for pair in points.windows(2) {
            if coverage.add_segment(pair[0], pair[1]) {
                return true;
            }
        }
        if !subpath.is_explicitly_closed()
            && coverage.add_segment(
                *points.last().expect("K2 subpath proof is nonempty"),
                points[0],
            )
        {
            return true;
        }
    }
    coverage.matches_rule(rule)
}
