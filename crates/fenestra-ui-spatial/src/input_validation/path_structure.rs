//! Dense path keys and trusted path-verb ranges.

use std::ops::Range;

use super::make_resolve_error;
use super::transforms::LocalTransformProof;
use crate::content_diagnostic::{SpatialKeyedContentTableV2, SpatialPayloadTableV2};
use crate::content_error::SpatialContentErrorKindV2;
use crate::error::SpatialErrorLocationV2;
use crate::geometry_field::SpatialPathFieldV2;
use crate::limits::SpatialLimitsV2;
use crate::path::SpatialPathVerbV2;
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};

pub(super) struct PathStructureProof<'a> {
    transforms: LocalTransformProof<'a>,
    ranges: Vec<Range<usize>>,
}

impl<'a> PathStructureProof<'a> {
    pub(super) fn input(&self) -> crate::aggregate_input::SpatialInputV2<'a> {
        self.transforms.input()
    }

    pub(super) fn limits(&self) -> SpatialLimitsV2 {
        self.transforms.limits()
    }

    pub(super) fn dependency_islands(
        &self,
    ) -> impl Iterator<Item = super::islands::preflight::DependencyIslandInput<'_>> + '_ {
        self.transforms.dependency_islands()
    }

    pub(super) fn take_prepared_island(
        &mut self,
        index: u32,
    ) -> fenestra_ui_layout::prototype::PreparedLayoutInputV1 {
        self.transforms.take_prepared_island(index)
    }

    pub(super) fn path_verbs(&self, index: usize) -> &'a [SpatialPathVerbV2] {
        let range = self
            .ranges
            .get(index)
            .expect("phase seven supplied a trusted path ordinal")
            .clone();
        &self.transforms.input().geometry().path_verbs()[range]
    }
}

pub(super) fn prepare_path_structure(
    transforms: LocalTransformProof<'_>,
) -> Result<PathStructureProof<'_>, SpatialResolveErrorV2> {
    let geometry = transforms.input().geometry();
    let paths = geometry.paths();
    let verb_count = geometry.path_verbs().len() as u128;

    for (index, path) in paths.iter().copied().enumerate() {
        let ordinal = trusted_path_ordinal(index);
        if path.key().get() != ordinal {
            return Err(path_error(
                SpatialContentErrorKindV2::NonDenseKey(SpatialKeyedContentTableV2::Path),
                SpatialErrorLocationV2::Path {
                    index: ordinal,
                    field: SpatialPathFieldV2::Key,
                },
            ));
        }
    }

    let mut cursor = 0_u128;
    let mut ranges = Vec::with_capacity(paths.len());
    for (index, path) in paths.iter().copied().enumerate() {
        let ordinal = trusted_path_ordinal(index);
        let start = path.verb_start() as u128;
        if start != cursor {
            return Err(invalid_range(SpatialErrorLocationV2::Path {
                index: ordinal,
                field: SpatialPathFieldV2::VerbStart,
            }));
        }

        let end = start + path.verb_length() as u128;
        if end > verb_count {
            return Err(invalid_range(SpatialErrorLocationV2::Path {
                index: ordinal,
                field: SpatialPathFieldV2::VerbLength,
            }));
        }

        ranges.push(
            usize::try_from(start).expect("a trusted path start fits the payload table")
                ..usize::try_from(end).expect("a trusted path end fits the payload table"),
        );
        cursor = end;
    }

    if cursor != verb_count {
        return Err(invalid_range(SpatialErrorLocationV2::Input));
    }

    Ok(PathStructureProof { transforms, ranges })
}

pub(super) fn trusted_path_ordinal(index: usize) -> u32 {
    u32::try_from(index).expect("phase one validated the path row capacity")
}

fn invalid_range(location: SpatialErrorLocationV2) -> SpatialResolveErrorV2 {
    path_error(
        SpatialContentErrorKindV2::InvalidRange(SpatialPayloadTableV2::PathVerb),
        location,
    )
}

fn path_error(
    kind: SpatialContentErrorKindV2,
    location: SpatialErrorLocationV2,
) -> SpatialResolveErrorV2 {
    make_resolve_error(SpatialResolveErrorKindV2::Content(kind), location)
}

#[cfg(test)]
impl PathStructureProof<'_> {
    pub(super) fn path_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.ranges
            .iter()
            .enumerate()
            .map(|(index, range)| {
                (
                    trusted_path_ordinal(index),
                    range.start as u128,
                    range.end as u128,
                )
            })
            .collect()
    }

    pub(super) fn prepared_island_facts(&self) -> Vec<(u32, Vec<u32>)> {
        self.transforms.prepared_island_facts()
    }
}
