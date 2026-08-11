use crate::model::{SpatialPointV2, SpatialScalarV2};

use super::error::{
    GeometryK1Error, GeometryK1ErrorKind, GeometryK1Field, GeometryK1LimitKind, GeometryK1Location,
    GeometryK1ShapeKind,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ValidatedRectK1 {
    origin: SpatialPointV2,
    width: SpatialScalarV2,
    height: SpatialScalarV2,
}

impl ValidatedRectK1 {
    pub(crate) const fn origin(self) -> SpatialPointV2 {
        self.origin
    }

    pub(crate) const fn width(self) -> SpatialScalarV2 {
        self.width
    }

    pub(crate) const fn height(self) -> SpatialScalarV2 {
        self.height
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ValidatedCircleK1 {
    center: SpatialPointV2,
    radius: SpatialScalarV2,
}

impl ValidatedCircleK1 {
    pub(crate) const fn center(self) -> SpatialPointV2 {
        self.center
    }

    pub(crate) const fn radius(self) -> SpatialScalarV2 {
        self.radius
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ValidatedPolygonK1<'a> {
    points: &'a [SpatialPointV2],
}

impl<'a> ValidatedPolygonK1<'a> {
    pub(crate) const fn points(self) -> &'a [SpatialPointV2] {
        self.points
    }
}

pub(crate) fn validate_rect_k1(
    shape: u32,
    origin: SpatialPointV2,
    width: SpatialScalarV2,
    height: SpatialScalarV2,
) -> Result<ValidatedRectK1, GeometryK1Error> {
    validate_shape_scalar(shape, origin.x(), GeometryK1Field::RectX)?;
    validate_shape_scalar(shape, origin.y(), GeometryK1Field::RectY)?;
    validate_shape_scalar(shape, width, GeometryK1Field::RectWidth)?;
    validate_shape_scalar(shape, height, GeometryK1Field::RectHeight)?;

    if width.raw() < 0 {
        return Err(shape_error(
            shape,
            GeometryK1Field::RectWidth,
            GeometryK1ShapeKind::NegativeExtent,
        ));
    }
    if height.raw() < 0 {
        return Err(shape_error(
            shape,
            GeometryK1Field::RectHeight,
            GeometryK1ShapeKind::NegativeExtent,
        ));
    }

    Ok(ValidatedRectK1 {
        origin,
        width,
        height,
    })
}

pub(crate) fn validate_circle_k1(
    shape: u32,
    center: SpatialPointV2,
    radius: SpatialScalarV2,
) -> Result<ValidatedCircleK1, GeometryK1Error> {
    validate_shape_scalar(shape, center.x(), GeometryK1Field::CircleCenterX)?;
    validate_shape_scalar(shape, center.y(), GeometryK1Field::CircleCenterY)?;
    validate_shape_scalar(shape, radius, GeometryK1Field::CircleRadius)?;

    if radius.raw() < 0 {
        return Err(shape_error(
            shape,
            GeometryK1Field::CircleRadius,
            GeometryK1ShapeKind::NegativeRadius,
        ));
    }

    Ok(ValidatedCircleK1 { center, radius })
}

pub(crate) fn validate_polygon_k1<'a>(
    shape: u32,
    points: &'a [SpatialPointV2],
    maximum_points: usize,
) -> Result<ValidatedPolygonK1<'a>, GeometryK1Error> {
    for (ordinal, point) in points.iter().copied().enumerate() {
        validate_polygon_scalar(shape, ordinal, point.x(), GeometryK1Field::X)?;
        validate_polygon_scalar(shape, ordinal, point.y(), GeometryK1Field::Y)?;
    }

    if points.len() > maximum_points {
        return Err(GeometryK1Error::limit(
            GeometryK1LimitKind::PolygonPointsPerShape,
            shape_location(shape, GeometryK1Field::PolygonPointLength),
            points.len() as u128,
            maximum_points as u128,
        ));
    }
    if points.len() < 3 {
        return Err(shape_error(
            shape,
            GeometryK1Field::PolygonPointLength,
            GeometryK1ShapeKind::PolygonTooShort,
        ));
    }

    if points.last() == points.first() {
        return Err(polygon_error(
            shape,
            points.len() - 1,
            GeometryK1ShapeKind::PolygonRepeatedFirst,
        ));
    }
    for (ordinal, pair) in points.windows(2).enumerate() {
        if pair[0] == pair[1] {
            return Err(polygon_error(
                shape,
                ordinal + 1,
                GeometryK1ShapeKind::PolygonAdjacentEqual,
            ));
        }
    }

    Ok(ValidatedPolygonK1 { points })
}

fn validate_shape_scalar(
    shape: u32,
    scalar: SpatialScalarV2,
    field: GeometryK1Field,
) -> Result<(), GeometryK1Error> {
    if scalar.is_in_domain() {
        Ok(())
    } else {
        Err(GeometryK1Error::new(
            GeometryK1ErrorKind::ScalarOutOfDomain,
            shape_location(shape, field),
        ))
    }
}

fn validate_polygon_scalar(
    shape: u32,
    ordinal: usize,
    scalar: SpatialScalarV2,
    field: GeometryK1Field,
) -> Result<(), GeometryK1Error> {
    if scalar.is_in_domain() {
        Ok(())
    } else {
        Err(GeometryK1Error::new(
            GeometryK1ErrorKind::ScalarOutOfDomain,
            polygon_location(shape, ordinal, field),
        ))
    }
}

fn shape_error(shape: u32, field: GeometryK1Field, kind: GeometryK1ShapeKind) -> GeometryK1Error {
    GeometryK1Error::new(
        GeometryK1ErrorKind::InvalidShape(kind),
        shape_location(shape, field),
    )
}

fn polygon_error(shape: u32, ordinal: usize, kind: GeometryK1ShapeKind) -> GeometryK1Error {
    GeometryK1Error::new(
        GeometryK1ErrorKind::InvalidShape(kind),
        polygon_location(shape, ordinal, GeometryK1Field::X),
    )
}

const fn shape_location(shape: u32, field: GeometryK1Field) -> GeometryK1Location {
    GeometryK1Location::Shape {
        index: shape,
        field,
    }
}

fn polygon_location(shape: u32, ordinal: usize, field: GeometryK1Field) -> GeometryK1Location {
    GeometryK1Location::PolygonPoint {
        shape,
        point: ordinal as u32,
        field,
    }
}
