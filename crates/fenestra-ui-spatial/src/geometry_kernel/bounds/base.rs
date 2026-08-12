use crate::aabb::SpatialAabbV2;
use crate::model::{SpatialPointV2, SpatialScalarV2};
use crate::numeric::scalar_from_i128;
use crate::path::SpatialPathVerbV2;
use crate::vocabulary::SpatialAxisV2;

use super::super::{
    error::{GeometryK1Field, GeometryK1Location},
    path::ValidatedPathK1,
    shape::{ValidatedCircleK1, ValidatedPolygonK1, ValidatedRectK1},
};
use super::{error::GeometryK3Error, model::DerivedLocalBoundsK3};

pub(crate) fn derive_rect_bounds_k3(
    shape: u32,
    rect: ValidatedRectK1,
) -> Result<DerivedLocalBoundsK3, GeometryK3Error> {
    let max_x = add_extent(rect.origin().x(), rect.width())
        .ok_or_else(|| shape_error(SpatialAxisV2::X, shape, GeometryK1Field::RectWidth))?;
    let max_y = add_extent(rect.origin().y(), rect.height())
        .ok_or_else(|| shape_error(SpatialAxisV2::Y, shape, GeometryK1Field::RectHeight))?;
    let base = closed_aabb(rect.origin().x(), rect.origin().y(), max_x, max_y);
    let empty_fill_clip = rect.width().raw() == 0 || rect.height().raw() == 0;
    Ok(DerivedLocalBoundsK3::new(base, empty_fill_clip))
}

pub(crate) fn derive_circle_bounds_k3(
    shape: u32,
    circle: ValidatedCircleK1,
) -> Result<DerivedLocalBoundsK3, GeometryK3Error> {
    let radius = i128::from(circle.radius().raw());
    let center_x = i128::from(circle.center().x().raw());
    let min_x = scalar_from_i128(center_x - radius);
    let max_x = scalar_from_i128(center_x + radius);
    let (min_x, max_x) = match (min_x, max_x) {
        (Some(minimum), Some(maximum)) => (minimum, maximum),
        _ => {
            return Err(shape_error(
                SpatialAxisV2::X,
                shape,
                GeometryK1Field::CircleRadius,
            ));
        }
    };

    let center_y = i128::from(circle.center().y().raw());
    let min_y = scalar_from_i128(center_y - radius);
    let max_y = scalar_from_i128(center_y + radius);
    let (min_y, max_y) = match (min_y, max_y) {
        (Some(minimum), Some(maximum)) => (minimum, maximum),
        _ => {
            return Err(shape_error(
                SpatialAxisV2::Y,
                shape,
                GeometryK1Field::CircleRadius,
            ));
        }
    };

    Ok(DerivedLocalBoundsK3::new(
        closed_aabb(min_x, min_y, max_x, max_y),
        circle.radius().raw() == 0,
    ))
}

pub(crate) fn derive_polygon_bounds_k3(polygon: ValidatedPolygonK1<'_>) -> DerivedLocalBoundsK3 {
    let extrema = Extrema::from_points(polygon.points());
    DerivedLocalBoundsK3::new(extrema.into_aabb(), false)
}

pub(crate) fn derive_path_bounds_k3(path: ValidatedPathK1<'_>) -> DerivedLocalBoundsK3 {
    let mut extrema = None;
    for verb in path.verbs().iter().copied() {
        match verb {
            SpatialPathVerbV2::MoveTo { to } | SpatialPathVerbV2::LineTo { to } => {
                include_point(&mut extrema, to);
            }
            SpatialPathVerbV2::QuadraticTo { control, to } => {
                include_point(&mut extrema, control);
                include_point(&mut extrema, to);
            }
            SpatialPathVerbV2::CubicTo {
                control1,
                control2,
                to,
            } => {
                include_point(&mut extrema, control1);
                include_point(&mut extrema, control2);
                include_point(&mut extrema, to);
            }
            SpatialPathVerbV2::Close => {}
        }
    }
    let extrema = extrema.expect("K1 path proof guarantees point-bearing verbs");
    DerivedLocalBoundsK3::new(extrema.into_aabb(), false)
}

fn add_extent(origin: SpatialScalarV2, extent: SpatialScalarV2) -> Option<SpatialScalarV2> {
    scalar_from_i128(i128::from(origin.raw()) + i128::from(extent.raw()))
}

fn shape_error(axis: SpatialAxisV2, shape: u32, field: GeometryK1Field) -> GeometryK3Error {
    GeometryK3Error::new(
        axis,
        GeometryK1Location::Shape {
            index: shape,
            field,
        },
    )
}

fn include_point(extrema: &mut Option<Extrema>, point: SpatialPointV2) {
    match extrema {
        Some(extrema) => extrema.include(point),
        None => *extrema = Some(Extrema::new(point)),
    }
}

#[derive(Clone, Copy)]
struct Extrema {
    min_x: SpatialScalarV2,
    min_y: SpatialScalarV2,
    max_x: SpatialScalarV2,
    max_y: SpatialScalarV2,
}

impl Extrema {
    fn from_points(points: &[SpatialPointV2]) -> Self {
        let (first, rest) = points
            .split_first()
            .expect("K1 polygon proof guarantees at least three points");
        let mut extrema = Self::new(*first);
        for point in rest.iter().copied() {
            extrema.include(point);
        }
        extrema
    }

    const fn new(point: SpatialPointV2) -> Self {
        Self {
            min_x: point.x(),
            min_y: point.y(),
            max_x: point.x(),
            max_y: point.y(),
        }
    }

    fn include(&mut self, point: SpatialPointV2) {
        if point.x().raw() < self.min_x.raw() {
            self.min_x = point.x();
        }
        if point.y().raw() < self.min_y.raw() {
            self.min_y = point.y();
        }
        if point.x().raw() > self.max_x.raw() {
            self.max_x = point.x();
        }
        if point.y().raw() > self.max_y.raw() {
            self.max_y = point.y();
        }
    }

    fn into_aabb(self) -> SpatialAabbV2 {
        closed_aabb(self.min_x, self.min_y, self.max_x, self.max_y)
    }
}

fn closed_aabb(
    min_x: SpatialScalarV2,
    min_y: SpatialScalarV2,
    max_x: SpatialScalarV2,
    max_y: SpatialScalarV2,
) -> SpatialAabbV2 {
    match SpatialAabbV2::from_edges(min_x, min_y, max_x, max_y) {
        Some(bounds) => bounds,
        None => unreachable!("K3 supplies canonical ordered closed edges"),
    }
}
