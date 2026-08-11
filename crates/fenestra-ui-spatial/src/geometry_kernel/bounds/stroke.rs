use crate::aabb::SpatialAabbV2;
use crate::model::SpatialScalarV2;
use crate::numeric::scalar_from_i128;
use crate::vocabulary::SpatialAxisV2;

use super::super::{
    error::{GeometryK1Field, GeometryK1Location},
    stroke::{GeometryK1StrokeSource, ValidatedStrokeK1},
};
use super::{error::GeometryK3Error, model::DerivedLocalBoundsK3};

pub(crate) fn stroke_bounds_k3(
    derived: &DerivedLocalBoundsK3,
    source: GeometryK1StrokeSource,
    stroke: ValidatedStrokeK1,
) -> Result<SpatialAabbV2, GeometryK3Error> {
    let base = derived.base_bounds();
    let expansion = (i128::from(stroke.width().raw()) + 1) / 2;

    let min_x = scalar_from_i128(i128::from(base.min_x().raw()) - expansion);
    let max_x = scalar_from_i128(i128::from(base.max_x().raw()) + expansion);
    let (min_x, max_x) = axis_edges(min_x, max_x, SpatialAxisV2::X, source)?;

    let min_y = scalar_from_i128(i128::from(base.min_y().raw()) - expansion);
    let max_y = scalar_from_i128(i128::from(base.max_y().raw()) + expansion);
    let (min_y, max_y) = axis_edges(min_y, max_y, SpatialAxisV2::Y, source)?;

    match SpatialAabbV2::from_edges(min_x, min_y, max_x, max_y) {
        Some(bounds) => Ok(bounds),
        None => unreachable!("successful K3 expansion keeps ordered closed edges"),
    }
}

fn axis_edges(
    minimum: Option<SpatialScalarV2>,
    maximum: Option<SpatialScalarV2>,
    axis: SpatialAxisV2,
    source: GeometryK1StrokeSource,
) -> Result<(SpatialScalarV2, SpatialScalarV2), GeometryK3Error> {
    match (minimum, maximum) {
        (Some(minimum), Some(maximum)) => Ok((minimum, maximum)),
        _ => Err(GeometryK3Error::new(axis, stroke_location(source))),
    }
}

const fn stroke_location(source: GeometryK1StrokeSource) -> GeometryK1Location {
    match source {
        GeometryK1StrokeSource::Paint { index } => GeometryK1Location::Paint {
            index,
            field: GeometryK1Field::StrokeWidth,
        },
        GeometryK1StrokeSource::Hit { index } => GeometryK1Location::Hit {
            index,
            field: GeometryK1Field::StrokeWidth,
        },
    }
}
