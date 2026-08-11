use crate::model::{SpatialPointV2, SpatialScalarV2};
use crate::path::SpatialPathVerbV2;

use super::error::{
    GeometryK1Error, GeometryK1ErrorKind, GeometryK1Field, GeometryK1LimitKind, GeometryK1Location,
    GeometryK1PathGrammarKind,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ValidatedPathK1<'a> {
    verbs: &'a [SpatialPathVerbV2],
    subpath_count: usize,
}

impl<'a> ValidatedPathK1<'a> {
    pub(crate) const fn verbs(self) -> &'a [SpatialPathVerbV2] {
        self.verbs
    }

    pub(crate) const fn subpath_count(self) -> usize {
        self.subpath_count
    }
}

pub(crate) fn validate_path_k1(
    path: u32,
    verbs: &[SpatialPathVerbV2],
    accepted_subpaths: usize,
    maximum_subpaths: usize,
) -> Result<ValidatedPathK1<'_>, GeometryK1Error> {
    validate_path_scalars(path, verbs)?;
    validate_path_grammar(path, verbs)?;

    let mut total = accepted_subpaths as u128;
    let maximum_subpaths = maximum_subpaths as u128;
    let mut subpath_count = 0;
    for (ordinal, verb) in verbs.iter().enumerate() {
        if matches!(verb, SpatialPathVerbV2::MoveTo { .. }) {
            total += 1;
            subpath_count += 1;
            if total > maximum_subpaths {
                return Err(GeometryK1Error::limit(
                    GeometryK1LimitKind::PathSubpathsTotal,
                    path_verb_location(path, ordinal, GeometryK1Field::Kind),
                    total,
                    maximum_subpaths,
                ));
            }
        }
    }

    Ok(ValidatedPathK1 {
        verbs,
        subpath_count,
    })
}

fn validate_path_scalars(path: u32, verbs: &[SpatialPathVerbV2]) -> Result<(), GeometryK1Error> {
    for (ordinal, verb) in verbs.iter().copied().enumerate() {
        match verb {
            SpatialPathVerbV2::MoveTo { to } | SpatialPathVerbV2::LineTo { to } => {
                validate_point(
                    path,
                    ordinal,
                    to,
                    GeometryK1Field::ToX,
                    GeometryK1Field::ToY,
                )?;
            }
            SpatialPathVerbV2::QuadraticTo { control, to } => {
                validate_point(
                    path,
                    ordinal,
                    control,
                    GeometryK1Field::ControlX,
                    GeometryK1Field::ControlY,
                )?;
                validate_point(
                    path,
                    ordinal,
                    to,
                    GeometryK1Field::ToX,
                    GeometryK1Field::ToY,
                )?;
            }
            SpatialPathVerbV2::CubicTo {
                control1,
                control2,
                to,
            } => {
                validate_point(
                    path,
                    ordinal,
                    control1,
                    GeometryK1Field::Control1X,
                    GeometryK1Field::Control1Y,
                )?;
                validate_point(
                    path,
                    ordinal,
                    control2,
                    GeometryK1Field::Control2X,
                    GeometryK1Field::Control2Y,
                )?;
                validate_point(
                    path,
                    ordinal,
                    to,
                    GeometryK1Field::ToX,
                    GeometryK1Field::ToY,
                )?;
            }
            SpatialPathVerbV2::Close => {}
        }
    }
    Ok(())
}

fn validate_point(
    path: u32,
    ordinal: usize,
    point: SpatialPointV2,
    x_field: GeometryK1Field,
    y_field: GeometryK1Field,
) -> Result<(), GeometryK1Error> {
    validate_scalar(path, ordinal, point.x(), x_field)?;
    validate_scalar(path, ordinal, point.y(), y_field)
}

fn validate_scalar(
    path: u32,
    ordinal: usize,
    scalar: SpatialScalarV2,
    field: GeometryK1Field,
) -> Result<(), GeometryK1Error> {
    if scalar.is_in_domain() {
        Ok(())
    } else {
        Err(GeometryK1Error::new(
            GeometryK1ErrorKind::ScalarOutOfDomain,
            path_verb_location(path, ordinal, field),
        ))
    }
}

fn validate_path_grammar(path: u32, verbs: &[SpatialPathVerbV2]) -> Result<(), GeometryK1Error> {
    let Some(first) = verbs.first() else {
        return Err(grammar_error(
            GeometryK1PathGrammarKind::Empty,
            GeometryK1Location::Path {
                index: path,
                field: GeometryK1Field::VerbLength,
            },
        ));
    };
    if !matches!(first, SpatialPathVerbV2::MoveTo { .. }) {
        return Err(grammar_at_verb(
            path,
            0,
            GeometryK1PathGrammarKind::FirstNotMove,
        ));
    }

    let mut active = true;
    let mut has_segment = false;
    let mut last_move = 0;
    for (ordinal, verb) in verbs.iter().enumerate().skip(1) {
        match verb {
            SpatialPathVerbV2::MoveTo { .. } => {
                if active && !has_segment {
                    return Err(grammar_at_verb(
                        path,
                        ordinal,
                        GeometryK1PathGrammarKind::EmptySubpath,
                    ));
                }
                active = true;
                has_segment = false;
                last_move = ordinal;
            }
            SpatialPathVerbV2::LineTo { .. }
            | SpatialPathVerbV2::QuadraticTo { .. }
            | SpatialPathVerbV2::CubicTo { .. } => {
                if !active {
                    return Err(grammar_at_verb(
                        path,
                        ordinal,
                        GeometryK1PathGrammarKind::DrawingWithoutSubpath,
                    ));
                }
                has_segment = true;
            }
            SpatialPathVerbV2::Close => {
                if !active || !has_segment {
                    return Err(grammar_at_verb(
                        path,
                        ordinal,
                        GeometryK1PathGrammarKind::CloseWithoutSegment,
                    ));
                }
                active = false;
                has_segment = false;
            }
        }
    }

    if active && !has_segment {
        Err(grammar_at_verb(
            path,
            last_move,
            GeometryK1PathGrammarKind::TrailingMove,
        ))
    } else {
        Ok(())
    }
}

fn grammar_at_verb(path: u32, ordinal: usize, kind: GeometryK1PathGrammarKind) -> GeometryK1Error {
    grammar_error(
        kind,
        path_verb_location(path, ordinal, GeometryK1Field::Kind),
    )
}

fn grammar_error(kind: GeometryK1PathGrammarKind, location: GeometryK1Location) -> GeometryK1Error {
    GeometryK1Error::new(GeometryK1ErrorKind::InvalidPathGrammar(kind), location)
}

fn path_verb_location(path: u32, ordinal: usize, field: GeometryK1Field) -> GeometryK1Location {
    GeometryK1Location::PathVerb {
        path,
        verb: ordinal as u32,
        field,
    }
}
