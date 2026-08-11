//! Dense shape keys and trusted structural references and ranges.

use std::ops::Range;

use super::make_resolve_error;
use super::validated_paths::ValidatedPathsProof;
use crate::content_diagnostic::{
    SpatialContentReferenceV2, SpatialKeyedContentTableV2, SpatialPayloadTableV2,
};
use crate::content_error::SpatialContentErrorKindV2;
use crate::error::SpatialErrorLocationV2;
use crate::geometry_field::SpatialShapeFieldV2;
use crate::limits::SpatialLimitsV2;
use crate::model::SpatialPointV2;
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};
use crate::shape::SpatialShapeGeometryV2;

struct PolygonRange {
    shape: u32,
    points: Range<usize>,
}

pub(super) struct ShapeStructureProof<'a> {
    paths: ValidatedPathsProof<'a>,
    polygon_ranges: Vec<PolygonRange>,
}

impl<'a> ShapeStructureProof<'a> {
    pub(super) fn input(&self) -> crate::aggregate_input::SpatialInputV2<'a> {
        self.paths.input()
    }

    pub(super) fn limits(&self) -> SpatialLimitsV2 {
        self.paths.limits()
    }

    pub(super) fn dependency_islands(
        &self,
    ) -> impl Iterator<Item = super::islands::preflight::DependencyIslandInput<'_>> + '_ {
        self.paths.dependency_islands()
    }

    pub(super) fn take_prepared_island(
        &mut self,
        index: u32,
    ) -> fenestra_ui_layout::prototype::PreparedLayoutInputV1 {
        self.paths.take_prepared_island(index)
    }

    pub(super) fn validated_paths(&self) -> &[crate::geometry_kernel::ValidatedPathK1<'a>] {
        self.paths.validated_paths()
    }

    pub(super) fn polygon_points(&self, polygon: usize, shape: u32) -> &'a [SpatialPointV2] {
        let range = self
            .polygon_ranges
            .get(polygon)
            .expect("phase seven supplied a trusted polygon ordinal");
        assert_eq!(
            range.shape, shape,
            "trusted polygon ranges remain aligned with shape order"
        );
        &self.input().geometry().polygon_points()[range.points.clone()]
    }
}

pub(super) fn prepare_shape_structure(
    paths: ValidatedPathsProof<'_>,
) -> Result<ShapeStructureProof<'_>, SpatialResolveErrorV2> {
    let input = paths.input();
    let topology = input.topology();
    let geometry = input.geometry();
    let shapes = geometry.shapes();

    for (index, shape) in shapes.iter().copied().enumerate() {
        let ordinal = trusted_shape_ordinal(index);
        if shape.key().get() != ordinal {
            return Err(content_error(
                SpatialContentErrorKindV2::NonDenseKey(SpatialKeyedContentTableV2::Shape),
                shape_location(ordinal, SpatialShapeFieldV2::Key),
            ));
        }
    }

    let node_count = topology.nodes().len() as u128;
    let path_count = geometry.paths().len() as u128;
    let point_count = geometry.polygon_points().len() as u128;
    let mut cursor = 0_u128;
    let mut polygon_ranges = Vec::new();

    for (index, shape) in shapes.iter().copied().enumerate() {
        let ordinal = trusted_shape_ordinal(index);
        let owner = u128::from(shape.owner().get());
        if owner == 0 || owner >= node_count {
            return Err(content_error(
                SpatialContentErrorKindV2::InvalidReference(SpatialContentReferenceV2::Owner),
                shape_location(ordinal, SpatialShapeFieldV2::Owner),
            ));
        }

        match shape.geometry() {
            SpatialShapeGeometryV2::Rect { .. } | SpatialShapeGeometryV2::Circle { .. } => {}
            SpatialShapeGeometryV2::Polygon {
                point_start,
                point_length,
            } => {
                let end = validate_polygon_range(
                    ordinal,
                    cursor,
                    point_start,
                    point_length,
                    point_count,
                )?;
                polygon_ranges.push(PolygonRange {
                    shape: ordinal,
                    points: trusted_point_index(cursor)..trusted_point_index(end),
                });
                cursor = end;
            }
            SpatialShapeGeometryV2::Path { path } => {
                if u128::from(path.get()) >= path_count {
                    return Err(content_error(
                        SpatialContentErrorKindV2::InvalidReference(
                            SpatialContentReferenceV2::Path,
                        ),
                        shape_location(ordinal, SpatialShapeFieldV2::Path),
                    ));
                }
            }
        }
    }

    if cursor != point_count {
        return Err(invalid_polygon_range(SpatialErrorLocationV2::Input));
    }

    Ok(ShapeStructureProof {
        paths,
        polygon_ranges,
    })
}

pub(super) fn validate_polygon_range(
    shape: u32,
    cursor: u128,
    point_start: u32,
    point_length: u32,
    point_count: u128,
) -> Result<u128, SpatialResolveErrorV2> {
    let start = u128::from(point_start);
    if start != cursor {
        return Err(invalid_polygon_range(shape_location(
            shape,
            SpatialShapeFieldV2::PolygonPointStart,
        )));
    }

    let end = start + u128::from(point_length);
    if end > point_count {
        return Err(invalid_polygon_range(shape_location(
            shape,
            SpatialShapeFieldV2::PolygonPointLength,
        )));
    }

    Ok(end)
}

fn trusted_shape_ordinal(index: usize) -> u32 {
    u32::try_from(index).expect("phase one validated the shape row capacity")
}

fn trusted_point_index(index: u128) -> usize {
    usize::try_from(index).expect("a trusted polygon range fits the payload table")
}

const fn shape_location(index: u32, field: SpatialShapeFieldV2) -> SpatialErrorLocationV2 {
    SpatialErrorLocationV2::Shape { index, field }
}

fn invalid_polygon_range(location: SpatialErrorLocationV2) -> SpatialResolveErrorV2 {
    content_error(
        SpatialContentErrorKindV2::InvalidRange(SpatialPayloadTableV2::PolygonPoint),
        location,
    )
}

fn content_error(
    kind: SpatialContentErrorKindV2,
    location: SpatialErrorLocationV2,
) -> SpatialResolveErrorV2 {
    make_resolve_error(SpatialResolveErrorKindV2::Content(kind), location)
}

#[cfg(test)]
impl ShapeStructureProof<'_> {
    pub(super) fn polygon_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.polygon_ranges
            .iter()
            .map(|range| {
                (
                    range.shape,
                    range.points.start as u128,
                    range.points.end as u128,
                )
            })
            .collect()
    }

    pub(super) fn validated_path_facts(&self) -> Vec<(u32, usize, usize)> {
        self.paths.validated_path_facts()
    }

    pub(super) fn subpath_total(&self) -> usize {
        self.paths.subpath_total()
    }

    pub(super) fn path_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.paths.path_range_facts()
    }

    pub(super) fn prepared_island_facts(&self) -> Vec<(u32, Vec<u32>)> {
        self.paths.prepared_island_facts()
    }
}
