//! Record-major Geometry K2 path flattening.

use super::path_k2_mapping::map_path_k2_error;
use super::path_structure::trusted_path_ordinal;
use super::validated_hit_items::HitLocalBoundsInput;
use super::validated_paint_items::PaintLocalBoundsInput;
use super::validated_semantic_items::ValidatedSemanticItemsProof;
use super::validated_shapes::ShapeLocalBoundsInput;
use crate::aggregate_input::SpatialInputV2;
use crate::geometry_kernel::{FlattenedPathK2, flatten_path_k2};
use crate::limits::{SpatialLimitKindV2, SpatialLimitsV2};
use crate::resolve_error::SpatialResolveErrorV2;

#[cfg(test)]
mod facts;

pub(super) struct FlattenedPathsProof<'a> {
    semantics: ValidatedSemanticItemsProof<'a>,
    paths: Vec<FlattenedPathK2>,
    accepted_segments: usize,
}

impl<'a> FlattenedPathsProof<'a> {
    pub(super) fn input(&self) -> SpatialInputV2<'a> {
        self.semantics.input()
    }

    pub(super) fn limits(&self) -> SpatialLimitsV2 {
        self.semantics.limits()
    }

    pub(super) fn dependency_islands(
        &self,
    ) -> impl Iterator<Item = super::islands::preflight::DependencyIslandInput<'_>> + '_ {
        self.semantics.dependency_islands()
    }

    pub(super) fn take_prepared_island(
        &mut self,
        index: u32,
    ) -> fenestra_ui_layout::prototype::PreparedLayoutInputV1 {
        self.semantics.take_prepared_island(index)
    }

    pub(super) fn validated_paths(&self) -> &[crate::geometry_kernel::ValidatedPathK1<'a>] {
        self.semantics.validated_paths()
    }

    pub(super) fn shape_local_bounds_inputs(
        &self,
    ) -> impl Iterator<Item = ShapeLocalBoundsInput<'a>> + '_ {
        self.semantics.shape_local_bounds_inputs()
    }

    pub(super) fn paint_local_bounds_inputs(
        &self,
    ) -> impl Iterator<Item = PaintLocalBoundsInput<'a>> + '_ {
        self.semantics.paint_local_bounds_inputs()
    }

    pub(super) fn hit_local_bounds_inputs(&self) -> impl Iterator<Item = HitLocalBoundsInput> + '_ {
        self.semantics.hit_local_bounds_inputs()
    }
}

pub(super) fn prepare_flattened_paths<'a>(
    semantics: ValidatedSemanticItemsProof<'a>,
) -> Result<FlattenedPathsProof<'a>, SpatialResolveErrorV2> {
    let limits = semantics.limits();
    let maximum_per_path = limits.limit(SpatialLimitKindV2::FlattenedSegmentsPerPath);
    let maximum_total = limits.limit(SpatialLimitKindV2::FlattenedSegmentsTotal);
    let mut accepted_segments = 0_usize;
    let mut flattened_paths = Vec::with_capacity(semantics.validated_paths().len());

    for (index, path) in semantics.validated_paths().iter().copied().enumerate() {
        let flattened = flatten_path_k2(
            trusted_path_ordinal(index),
            path,
            accepted_segments,
            maximum_per_path,
            maximum_total,
        )
        .map_err(map_path_k2_error)?;
        accepted_segments = accepted_segments
            .checked_add(flattened.segment_count())
            .expect("successful K2 paths keep the accepted total within caller limits");
        flattened_paths.push(flattened);
    }

    Ok(FlattenedPathsProof {
        semantics,
        paths: flattened_paths,
        accepted_segments,
    })
}
