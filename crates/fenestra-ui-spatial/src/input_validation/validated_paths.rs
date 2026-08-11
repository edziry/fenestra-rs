//! Per-path limits and Geometry K1 validation.

use super::make_resolve_error;
use super::path_structure::{PathStructureProof, trusted_path_ordinal};
use crate::content_diagnostic::SpatialPathGrammarErrorV2;
use crate::content_error::SpatialContentErrorKindV2;
use crate::error::SpatialErrorLocationV2;
use crate::geometry_field::{SpatialPathFieldV2, SpatialPathVerbFieldV2};
use crate::geometry_kernel::{
    GeometryK1Error, GeometryK1ErrorKind, GeometryK1Field, GeometryK1LimitKind, GeometryK1Location,
    GeometryK1PathGrammarKind, ValidatedPathK1, validate_path_k1,
};
use crate::limits::SpatialLimitKindV2;
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};

pub(super) struct ValidatedPathsProof<'a> {
    structure: PathStructureProof<'a>,
    paths: Vec<ValidatedPathK1<'a>>,
    subpath_total: usize,
}

impl<'a> ValidatedPathsProof<'a> {
    pub(super) fn input(&self) -> crate::aggregate_input::SpatialInputV2<'a> {
        self.structure.input()
    }

    pub(super) fn limits(&self) -> crate::limits::SpatialLimitsV2 {
        self.structure.limits()
    }
}

pub(super) fn prepare_validated_paths(
    structure: PathStructureProof<'_>,
) -> Result<ValidatedPathsProof<'_>, SpatialResolveErrorV2> {
    let paths = structure.input().geometry().paths();
    let limits = structure.limits();
    let per_path_maximum = limits.limit(SpatialLimitKindV2::PathVerbsPerPath) as u128;

    for (index, path) in paths.iter().copied().enumerate() {
        let observed = path.verb_length() as u128;
        if observed > per_path_maximum {
            return Err(SpatialResolveErrorV2::limit_exceeded(
                SpatialLimitKindV2::PathVerbsPerPath,
                SpatialErrorLocationV2::Path {
                    index: trusted_path_ordinal(index),
                    field: SpatialPathFieldV2::VerbLength,
                },
                observed,
                per_path_maximum,
            ));
        }
    }

    let maximum_subpaths = limits.limit(SpatialLimitKindV2::PathSubpathsTotal);
    let mut subpath_total = 0_usize;
    let mut validated_paths = Vec::with_capacity(paths.len());
    for index in 0..paths.len() {
        let path = validate_path_k1(
            trusted_path_ordinal(index),
            structure.path_verbs(index),
            subpath_total,
            maximum_subpaths,
        )
        .map_err(map_path_k1_error)?;
        subpath_total = subpath_total
            .checked_add(path.subpath_count())
            .expect("validated path subpaths fit the complete verb table");
        validated_paths.push(path);
    }

    Ok(ValidatedPathsProof {
        structure,
        paths: validated_paths,
        subpath_total,
    })
}

pub(super) fn map_path_k1_error(error: GeometryK1Error) -> SpatialResolveErrorV2 {
    match error.kind() {
        GeometryK1ErrorKind::ScalarOutOfDomain => map_scalar_error(error.location()),
        GeometryK1ErrorKind::InvalidPathGrammar(kind) => map_grammar_error(kind, error.location()),
        GeometryK1ErrorKind::LimitExceeded(GeometryK1LimitKind::PathSubpathsTotal) => {
            map_subpath_limit(error)
        }
        GeometryK1ErrorKind::InvalidShape(_)
        | GeometryK1ErrorKind::InvalidStroke(_)
        | GeometryK1ErrorKind::LimitExceeded(GeometryK1LimitKind::PolygonPointsPerShape) => {
            unreachable!("path K1 cannot return a shape or stroke failure")
        }
    }
}

fn map_scalar_error(location: GeometryK1Location) -> SpatialResolveErrorV2 {
    let GeometryK1Location::PathVerb { path, verb, field } = location else {
        unreachable!("path scalar failures use PathVerb locations")
    };
    content_error(
        SpatialContentErrorKindV2::ScalarOutOfDomain,
        SpatialErrorLocationV2::PathVerb {
            path,
            verb,
            field: path_scalar_field(field),
        },
    )
}

fn map_grammar_error(
    kind: GeometryK1PathGrammarKind,
    location: GeometryK1Location,
) -> SpatialResolveErrorV2 {
    let location = match location {
        GeometryK1Location::Path {
            index,
            field: GeometryK1Field::VerbLength,
        } => SpatialErrorLocationV2::Path {
            index,
            field: SpatialPathFieldV2::VerbLength,
        },
        GeometryK1Location::PathVerb {
            path,
            verb,
            field: GeometryK1Field::Kind,
        } => SpatialErrorLocationV2::PathVerb {
            path,
            verb,
            field: SpatialPathVerbFieldV2::Kind,
        },
        _ => unreachable!("path grammar failures use their registered locations"),
    };
    content_error(
        SpatialContentErrorKindV2::InvalidPathGrammar(path_grammar_kind(kind)),
        location,
    )
}

fn map_subpath_limit(error: GeometryK1Error) -> SpatialResolveErrorV2 {
    let GeometryK1Location::PathVerb {
        path,
        verb,
        field: GeometryK1Field::Kind,
    } = error.location()
    else {
        unreachable!("path subpath limits use the crossing MoveTo location")
    };
    SpatialResolveErrorV2::limit_exceeded(
        SpatialLimitKindV2::PathSubpathsTotal,
        SpatialErrorLocationV2::PathVerb {
            path,
            verb,
            field: SpatialPathVerbFieldV2::Kind,
        },
        error
            .observed()
            .expect("K1 subpath limit errors carry observed evidence"),
        error
            .maximum()
            .expect("K1 subpath limit errors carry maximum evidence"),
    )
}

fn path_scalar_field(field: GeometryK1Field) -> SpatialPathVerbFieldV2 {
    match field {
        GeometryK1Field::ToX => SpatialPathVerbFieldV2::ToX,
        GeometryK1Field::ToY => SpatialPathVerbFieldV2::ToY,
        GeometryK1Field::ControlX => SpatialPathVerbFieldV2::ControlX,
        GeometryK1Field::ControlY => SpatialPathVerbFieldV2::ControlY,
        GeometryK1Field::Control1X => SpatialPathVerbFieldV2::Control1X,
        GeometryK1Field::Control1Y => SpatialPathVerbFieldV2::Control1Y,
        GeometryK1Field::Control2X => SpatialPathVerbFieldV2::Control2X,
        GeometryK1Field::Control2Y => SpatialPathVerbFieldV2::Control2Y,
        _ => unreachable!("path scalar failures use applicable path scalar fields"),
    }
}

fn path_grammar_kind(kind: GeometryK1PathGrammarKind) -> SpatialPathGrammarErrorV2 {
    match kind {
        GeometryK1PathGrammarKind::Empty => SpatialPathGrammarErrorV2::Empty,
        GeometryK1PathGrammarKind::FirstNotMove => SpatialPathGrammarErrorV2::FirstNotMove,
        GeometryK1PathGrammarKind::EmptySubpath => SpatialPathGrammarErrorV2::EmptySubpath,
        GeometryK1PathGrammarKind::DrawingWithoutSubpath => {
            SpatialPathGrammarErrorV2::DrawingWithoutSubpath
        }
        GeometryK1PathGrammarKind::CloseWithoutSegment => {
            SpatialPathGrammarErrorV2::CloseWithoutSegment
        }
        GeometryK1PathGrammarKind::TrailingMove => SpatialPathGrammarErrorV2::TrailingMove,
    }
}

fn content_error(
    kind: SpatialContentErrorKindV2,
    location: SpatialErrorLocationV2,
) -> SpatialResolveErrorV2 {
    make_resolve_error(SpatialResolveErrorKindV2::Content(kind), location)
}

#[cfg(test)]
impl ValidatedPathsProof<'_> {
    pub(super) fn validated_path_facts(&self) -> Vec<(u32, usize, usize)> {
        self.paths
            .iter()
            .enumerate()
            .map(|(index, path)| {
                (
                    trusted_path_ordinal(index),
                    path.verbs().len(),
                    path.subpath_count(),
                )
            })
            .collect()
    }

    pub(super) fn subpath_total(&self) -> usize {
        self.subpath_total
    }

    pub(super) fn path_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.structure.path_range_facts()
    }

    pub(super) fn prepared_island_facts(&self) -> Vec<(u32, Vec<u32>)> {
        self.structure.prepared_island_facts()
    }
}
