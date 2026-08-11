use crate::model::SpatialScalarV2;

use super::error::{
    GeometryK1Error, GeometryK1ErrorKind, GeometryK1Field, GeometryK1Location, GeometryK1StrokeKind,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GeometryK1StrokeSource {
    Paint { index: u32 },
    Hit { index: u32 },
}

impl GeometryK1StrokeSource {
    const fn location(self) -> GeometryK1Location {
        match self {
            Self::Paint { index } => GeometryK1Location::Paint {
                index,
                field: GeometryK1Field::StrokeWidth,
            },
            Self::Hit { index } => GeometryK1Location::Hit {
                index,
                field: GeometryK1Field::StrokeWidth,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ValidatedStrokeK1 {
    width: SpatialScalarV2,
}

impl ValidatedStrokeK1 {
    pub(crate) const fn width(self) -> SpatialScalarV2 {
        self.width
    }
}

pub(crate) fn validate_stroke_k1(
    source: GeometryK1StrokeSource,
    width: SpatialScalarV2,
) -> Result<ValidatedStrokeK1, GeometryK1Error> {
    let location = source.location();
    if !width.is_in_domain() {
        return Err(GeometryK1Error::new(
            GeometryK1ErrorKind::ScalarOutOfDomain,
            location,
        ));
    }
    if width.raw() < 0 {
        return Err(GeometryK1Error::new(
            GeometryK1ErrorKind::InvalidStroke(GeometryK1StrokeKind::NegativeWidth),
            location,
        ));
    }
    if width.raw() == 0 {
        return Err(GeometryK1Error::new(
            GeometryK1ErrorKind::InvalidStroke(GeometryK1StrokeKind::ZeroWidth),
            location,
        ));
    }

    Ok(ValidatedStrokeK1 { width })
}
