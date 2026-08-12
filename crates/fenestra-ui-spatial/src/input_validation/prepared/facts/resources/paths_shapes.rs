use super::super::super::PreparedSpatialV2;
use super::super::super::model::PreparedShapeGeometry;
use super::ordinal;
use crate::aabb::SpatialAabbV2;
use crate::shape::SpatialShapeKindV2;

type FlattenedPathFact = (u32, usize, Vec<(i64, i64)>, Vec<(usize, usize, bool)>);
type ShapePlanFact = (
    u32,
    u32,
    SpatialShapeKindV2,
    Option<u32>,
    usize,
    SpatialAabbV2,
    SpatialAabbV2,
);

impl PreparedSpatialV2 {
    pub(in crate::input_validation) fn path_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.state
            .paths
            .iter()
            .enumerate()
            .map(|(index, path)| {
                (
                    ordinal(index),
                    path.verb_range.start as u128,
                    path.verb_range.end as u128,
                )
            })
            .collect()
    }

    pub(in crate::input_validation) fn validated_path_facts(&self) -> Vec<(u32, usize, usize)> {
        self.state
            .paths
            .iter()
            .enumerate()
            .map(|(index, path)| (ordinal(index), path.verb_count, path.subpath_count))
            .collect()
    }

    pub(in crate::input_validation) fn flattened_path_facts(&self) -> Vec<FlattenedPathFact> {
        self.state
            .paths
            .iter()
            .enumerate()
            .map(|(index, path)| {
                (
                    ordinal(index),
                    path.flattened.segment_count(),
                    path.flattened
                        .points()
                        .iter()
                        .map(|point| (point.x().raw(), point.y().raw()))
                        .collect(),
                    path.flattened
                        .subpaths()
                        .iter()
                        .copied()
                        .map(|subpath| {
                            (
                                subpath.point_start(),
                                subpath.point_length(),
                                subpath.is_explicitly_closed(),
                            )
                        })
                        .collect(),
                )
            })
            .collect()
    }

    pub(in crate::input_validation) fn polygon_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.state
            .shapes
            .iter()
            .enumerate()
            .filter_map(|(index, shape)| match &shape.geometry {
                PreparedShapeGeometry::Polygon { point_range } => Some((
                    ordinal(index),
                    point_range.start as u128,
                    point_range.end as u128,
                )),
                _ => None,
            })
            .collect()
    }

    pub(in crate::input_validation) fn shape_plan_facts(&self) -> Vec<ShapePlanFact> {
        self.state
            .shapes
            .iter()
            .enumerate()
            .map(|(index, shape)| {
                let (kind, path, point_count) = match &shape.geometry {
                    PreparedShapeGeometry::Rect { .. } => (SpatialShapeKindV2::Rect, None, 0),
                    PreparedShapeGeometry::Circle { .. } => (SpatialShapeKindV2::Circle, None, 0),
                    PreparedShapeGeometry::Polygon { point_range } => (
                        SpatialShapeKindV2::Polygon,
                        None,
                        point_range.end - point_range.start,
                    ),
                    PreparedShapeGeometry::Path { path } => {
                        (SpatialShapeKindV2::Path, Some(*path), 0)
                    }
                };
                (
                    ordinal(index),
                    shape.owner,
                    kind,
                    path,
                    point_count,
                    shape.base_bounds,
                    shape.fill_clip_bounds,
                )
            })
            .collect()
    }
}
