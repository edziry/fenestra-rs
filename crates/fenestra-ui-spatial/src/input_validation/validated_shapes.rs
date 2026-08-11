//! Record-major Geometry K1 shape validation.

use super::shape_k1_mapping::map_shape_k1_error;
use super::shape_structure::ShapeStructureProof;
use crate::geometry_kernel::{
    ValidatedCircleK1, ValidatedPolygonK1, ValidatedRectK1, validate_circle_k1,
    validate_polygon_k1, validate_rect_k1,
};
use crate::geometry_key::SpatialPathKeyV2;
use crate::resolve_error::SpatialResolveErrorV2;
use crate::shape::SpatialShapeGeometryV2;

#[cfg(test)]
use crate::model::SpatialPointV2;
#[cfg(test)]
use crate::shape::SpatialShapeKindV2;

struct ValidatedShape<'a> {
    owner: u32,
    geometry: ValidatedShapeGeometry<'a>,
}

enum ValidatedShapeGeometry<'a> {
    Rect(ValidatedRectK1),
    Circle(ValidatedCircleK1),
    Polygon(ValidatedPolygonK1<'a>),
    Path(SpatialPathKeyV2),
}

pub(super) struct ValidatedShapesProof<'a> {
    structure: ShapeStructureProof<'a>,
    shapes: Vec<ValidatedShape<'a>>,
}

impl<'a> ValidatedShapesProof<'a> {
    pub(super) fn input(&self) -> crate::aggregate_input::SpatialInputV2<'a> {
        self.structure.input()
    }

    pub(super) fn limits(&self) -> crate::limits::SpatialLimitsV2 {
        self.structure.limits()
    }
}

pub(super) fn prepare_validated_shapes<'a>(
    structure: ShapeStructureProof<'a>,
) -> Result<ValidatedShapesProof<'a>, SpatialResolveErrorV2> {
    let shapes = structure.input().geometry().shapes();
    let maximum_points = structure
        .limits()
        .limit(crate::limits::SpatialLimitKindV2::PolygonPointsPerShape);
    let mut polygon = 0_usize;
    let mut validated = Vec::with_capacity(shapes.len());

    for (index, shape) in shapes.iter().copied().enumerate() {
        let ordinal = u32::try_from(index).expect("phase one validated the shape row capacity");
        let geometry = match shape.geometry() {
            SpatialShapeGeometryV2::Rect {
                origin,
                width,
                height,
            } => ValidatedShapeGeometry::Rect(
                validate_rect_k1(ordinal, origin, width, height).map_err(map_shape_k1_error)?,
            ),
            SpatialShapeGeometryV2::Circle { center, radius } => ValidatedShapeGeometry::Circle(
                validate_circle_k1(ordinal, center, radius).map_err(map_shape_k1_error)?,
            ),
            SpatialShapeGeometryV2::Polygon { .. } => {
                let points = structure.polygon_points(polygon, ordinal);
                polygon += 1;
                ValidatedShapeGeometry::Polygon(
                    validate_polygon_k1(ordinal, points, maximum_points)
                        .map_err(map_shape_k1_error)?,
                )
            }
            SpatialShapeGeometryV2::Path { path } => ValidatedShapeGeometry::Path(path),
        };
        validated.push(ValidatedShape {
            owner: shape.owner().get(),
            geometry,
        });
    }

    Ok(ValidatedShapesProof {
        structure,
        shapes: validated,
    })
}

#[cfg(test)]
impl<'a> ValidatedShapesProof<'a> {
    pub(super) fn validated_shape_facts(&self) -> Vec<(u32, SpatialShapeKindV2, usize)> {
        self.shapes
            .iter()
            .enumerate()
            .map(|(index, shape)| {
                let (kind, point_count) = match shape.geometry {
                    ValidatedShapeGeometry::Rect(_) => (SpatialShapeKindV2::Rect, 0),
                    ValidatedShapeGeometry::Circle(_) => (SpatialShapeKindV2::Circle, 0),
                    ValidatedShapeGeometry::Polygon(proof) => {
                        (SpatialShapeKindV2::Polygon, proof.points().len())
                    }
                    ValidatedShapeGeometry::Path(_) => (SpatialShapeKindV2::Path, 0),
                };
                (
                    u32::try_from(index).expect("phase one validated the shape row capacity"),
                    kind,
                    point_count,
                )
            })
            .collect()
    }

    pub(super) fn validated_polygon_points(&self, shape: u32) -> &'a [SpatialPointV2] {
        match self
            .shapes
            .get(shape as usize)
            .expect("validated shape facts use a trusted shape ordinal")
            .geometry
        {
            ValidatedShapeGeometry::Polygon(proof) => proof.points(),
            _ => panic!("validated polygon facts require a polygon shape"),
        }
    }

    pub(super) fn polygon_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.structure.polygon_range_facts()
    }

    pub(super) fn validated_path_facts(&self) -> Vec<(u32, usize, usize)> {
        self.structure.validated_path_facts()
    }

    pub(super) fn subpath_total(&self) -> usize {
        self.structure.subpath_total()
    }

    pub(super) fn path_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.structure.path_range_facts()
    }

    pub(super) fn prepared_island_facts(&self) -> Vec<(u32, Vec<u32>)> {
        self.structure.prepared_island_facts()
    }
}
