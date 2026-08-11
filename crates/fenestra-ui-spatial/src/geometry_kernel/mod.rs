// K1 is staged before the private K2-K5 and resolver consumers.
#![allow(dead_code)]

mod error;
mod flatten;
mod path;
mod shape;
mod stroke;

#[cfg(test)]
pub(crate) use error::{
    GeometryK1Error, GeometryK1ErrorKind, GeometryK1Field, GeometryK1LimitKind, GeometryK1Location,
    GeometryK1PathGrammarKind, GeometryK1ShapeKind, GeometryK1StrokeKind,
};
#[cfg(test)]
pub(crate) use flatten::{
    FlattenedPathK2, GeometryK2Error, GeometryK2ErrorKind, GeometryK2LimitKind, flatten_path_k2,
};
#[cfg(test)]
pub(crate) use path::{ValidatedPathK1, validate_path_k1};
#[cfg(test)]
pub(crate) use shape::{
    ValidatedCircleK1, ValidatedPolygonK1, ValidatedRectK1, validate_circle_k1,
    validate_polygon_k1, validate_rect_k1,
};
#[cfg(test)]
pub(crate) use stroke::{GeometryK1StrokeSource, ValidatedStrokeK1, validate_stroke_k1};

#[cfg(test)]
mod tests;
