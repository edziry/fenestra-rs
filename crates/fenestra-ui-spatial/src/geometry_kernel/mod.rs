// K1-K5 are staged before resolver consumers.
#![allow(dead_code)]

mod bounds;
mod coverage;
mod error;
mod flatten;
mod path;
mod round_stroke;
mod shape;
mod stroke;

pub(crate) use bounds::{
    DerivedLocalBoundsK3, GeometryK3Error, GeometryK3ErrorKind, derive_circle_bounds_k3,
    derive_path_bounds_k3, derive_polygon_bounds_k3, derive_rect_bounds_k3, fill_bounds_k3,
    stroke_bounds_k3,
};
#[cfg(test)]
pub(crate) use bounds::{clip_bounds_k3, rect_stroke_bounds_k3};
#[cfg(test)]
pub(crate) use coverage::{
    circle_fill_contains_k4, path_fill_contains_k4, polygon_fill_contains_k4, rect_fill_contains_k4,
};
pub(crate) use error::{
    GeometryK1Error, GeometryK1ErrorKind, GeometryK1Field, GeometryK1LimitKind, GeometryK1Location,
    GeometryK1PathGrammarKind, GeometryK1ShapeKind, GeometryK1StrokeKind,
};
pub(crate) use flatten::{
    FlattenedPathK2, GeometryK2Error, GeometryK2ErrorKind, GeometryK2LimitKind, flatten_path_k2,
};
pub(crate) use path::{ValidatedPathK1, validate_path_k1};
#[cfg(test)]
pub(crate) use round_stroke::{
    circle_round_stroke_contains_k5, path_round_stroke_contains_k5,
    polygon_round_stroke_contains_k5, rect_round_stroke_contains_k5,
};
pub(crate) use shape::{
    ValidatedCircleK1, ValidatedPolygonK1, ValidatedRectK1, validate_circle_k1,
    validate_polygon_k1, validate_rect_k1,
};
pub(crate) use stroke::{GeometryK1StrokeSource, ValidatedStrokeK1, validate_stroke_k1};

#[cfg(test)]
mod tests;
