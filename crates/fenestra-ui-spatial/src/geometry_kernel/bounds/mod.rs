use crate::aabb::SpatialAabbV2;

use super::{
    shape::ValidatedRectK1,
    stroke::{GeometryK1StrokeSource, ValidatedStrokeK1},
};

mod base;
mod error;
mod model;
mod stroke;

pub(crate) use base::derive_rect_bounds_k3;
#[cfg(test)]
pub(crate) use base::{derive_circle_bounds_k3, derive_path_bounds_k3, derive_polygon_bounds_k3};
pub(crate) use error::GeometryK3Error;
#[cfg(test)]
pub(crate) use error::GeometryK3ErrorKind;
pub(crate) use model::DerivedLocalBoundsK3;
pub(crate) use stroke::stroke_bounds_k3;

pub(crate) const fn fill_bounds_k3(derived: &DerivedLocalBoundsK3) -> SpatialAabbV2 {
    derived.fill_clip_bounds()
}

pub(crate) const fn clip_bounds_k3(derived: &DerivedLocalBoundsK3) -> SpatialAabbV2 {
    derived.fill_clip_bounds()
}

pub(crate) fn rect_stroke_bounds_k3(
    shape: u32,
    rect: ValidatedRectK1,
    source: GeometryK1StrokeSource,
    stroke: ValidatedStrokeK1,
) -> Result<SpatialAabbV2, GeometryK3Error> {
    let derived = derive_rect_bounds_k3(shape, rect)?;
    stroke_bounds_k3(&derived, source, stroke)
}
