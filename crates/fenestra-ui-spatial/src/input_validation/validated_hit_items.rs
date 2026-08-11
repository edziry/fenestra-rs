//! Ordered hit-item validation and item-local geometry dispatch.

use super::make_resolve_error;
use super::ordered_items::OrderedItemCursor;
use super::stroke_k1_mapping::map_stroke_k1_error;
use super::validated_paint_items::{PaintLocalBoundsInput, ValidatedPaintItemsProof};
use super::validated_shapes::ShapeLocalBoundsInput;
use crate::aggregate_input::SpatialInputV2;
use crate::content_diagnostic::{
    SpatialClipErrorV2, SpatialContentReferenceV2, SpatialOrderedItemTableV2,
};
use crate::content_error::SpatialContentErrorKindV2;
use crate::content_item::SpatialInputPolicyV2;
use crate::coverage::{SpatialCoverageV2, SpatialFillRuleV2};
use crate::error::SpatialErrorLocationV2;
use crate::geometry_kernel::{GeometryK1StrokeSource, ValidatedStrokeK1, validate_stroke_k1};
use crate::item_field::SpatialHitFieldV2;
use crate::limits::{SpatialLimitKindV2, SpatialLimitsV2};
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};

#[cfg(test)]
mod facts;

struct ValidatedHitItem {
    owner: u32,
    item_ordinal: u32,
    coverage: ValidatedHitCoverage,
    clip: Option<u32>,
    input_policy: SpatialInputPolicyV2,
}

enum ValidatedHitCoverage {
    Fill {
        shape: u32,
        rule: SpatialFillRuleV2,
    },
    RoundStroke {
        shape: u32,
        stroke: ValidatedStrokeK1,
    },
}

pub(super) enum HitLocalBoundsInput {
    Fill {
        shape: u32,
    },
    RoundStroke {
        shape: u32,
        stroke: ValidatedStrokeK1,
    },
}

pub(super) struct ValidatedHitItemsProof<'a> {
    paints: ValidatedPaintItemsProof<'a>,
    hits: Vec<ValidatedHitItem>,
}

impl<'a> ValidatedHitItemsProof<'a> {
    pub(super) fn input(&self) -> SpatialInputV2<'a> {
        self.paints.input()
    }

    pub(super) fn limits(&self) -> SpatialLimitsV2 {
        self.paints.limits()
    }

    pub(super) fn dependency_islands(
        &self,
    ) -> impl Iterator<Item = super::islands::preflight::DependencyIslandInput<'_>> + '_ {
        self.paints.dependency_islands()
    }

    pub(super) fn take_prepared_island(
        &mut self,
        index: u32,
    ) -> fenestra_ui_layout::prototype::PreparedLayoutInputV1 {
        self.paints.take_prepared_island(index)
    }

    pub(super) fn validated_paths(&self) -> &[crate::geometry_kernel::ValidatedPathK1<'a>] {
        self.paints.validated_paths()
    }

    pub(super) fn shape_local_bounds_inputs(
        &self,
    ) -> impl Iterator<Item = ShapeLocalBoundsInput<'a>> + '_ {
        self.paints.shape_local_bounds_inputs()
    }

    pub(super) fn paint_local_bounds_inputs(
        &self,
    ) -> impl Iterator<Item = PaintLocalBoundsInput<'a>> + '_ {
        self.paints.paint_local_bounds_inputs()
    }

    pub(super) fn hit_local_bounds_inputs(&self) -> impl Iterator<Item = HitLocalBoundsInput> + '_ {
        self.hits.iter().map(|hit| match &hit.coverage {
            ValidatedHitCoverage::Fill { shape, .. } => HitLocalBoundsInput::Fill { shape: *shape },
            ValidatedHitCoverage::RoundStroke { shape, stroke } => {
                HitLocalBoundsInput::RoundStroke {
                    shape: *shape,
                    stroke: *stroke,
                }
            }
        })
    }

    pub(super) fn clip_owner_is_same_or_ancestor_of(&self, clip: u32, owner: u32) -> Option<bool> {
        self.paints.clip_owner_is_same_or_ancestor_of(clip, owner)
    }
}

pub(super) fn prepare_validated_hit_items<'a>(
    paints: ValidatedPaintItemsProof<'a>,
) -> Result<ValidatedHitItemsProof<'a>, SpatialResolveErrorV2> {
    let input = paints.input();
    let hits = input.items().hit_items();
    let node_count = input.topology().nodes().len() as u128;
    let limits = paints.limits();
    let mut cursor = OrderedItemCursor::new();
    let mut validated = Vec::with_capacity(hits.len());

    for (index, hit) in hits.iter().copied().enumerate() {
        let ordinal = trusted_hit_ordinal(index);
        let owner = hit.owner().get();
        let candidate = cursor.validate(
            SpatialOrderedItemTableV2::Hit,
            ordinal,
            owner,
            hit.item_ordinal(),
            node_count,
        )?;
        validate_hit_item_limit(ordinal, candidate.owner_count(), limits)?;

        let coverage = validate_coverage(&paints, ordinal, owner, hit.coverage())?;
        let clip =
            validate_terminal_clip(&paints, ordinal, owner, hit.clip().map(|key| key.get()))?;
        validated.push(ValidatedHitItem {
            owner: candidate.owner(),
            item_ordinal: candidate.item_ordinal(),
            coverage,
            clip,
            input_policy: hit.input_policy(),
        });
        cursor.commit(candidate);
    }

    Ok(ValidatedHitItemsProof {
        paints,
        hits: validated,
    })
}

pub(super) fn validate_hit_item_limit(
    hit: u32,
    observed: usize,
    limits: SpatialLimitsV2,
) -> Result<(), SpatialResolveErrorV2> {
    let observed = observed as u128;
    let maximum = limits.limit(SpatialLimitKindV2::HitItemsPerNode) as u128;
    if observed > maximum {
        return Err(SpatialResolveErrorV2::limit_exceeded(
            SpatialLimitKindV2::HitItemsPerNode,
            hit_location(hit, SpatialHitFieldV2::ItemOrdinal),
            observed,
            maximum,
        ));
    }
    Ok(())
}

fn validate_coverage(
    paints: &ValidatedPaintItemsProof<'_>,
    hit: u32,
    owner: u32,
    coverage: SpatialCoverageV2,
) -> Result<ValidatedHitCoverage, SpatialResolveErrorV2> {
    let shapes = paints.input().geometry().shapes();
    let validate_shape = |shape: u32| {
        if u128::from(shape) >= shapes.len() as u128
            || shapes[shape as usize].owner().get() != owner
        {
            Err(invalid_reference(
                SpatialContentReferenceV2::Shape,
                hit,
                SpatialHitFieldV2::Shape,
            ))
        } else {
            Ok(())
        }
    };

    match coverage {
        SpatialCoverageV2::Fill { shape, rule } => {
            let shape = shape.get();
            validate_shape(shape)?;
            Ok(ValidatedHitCoverage::Fill { shape, rule })
        }
        SpatialCoverageV2::RoundStroke { shape, width } => {
            let shape = shape.get();
            validate_shape(shape)?;
            let stroke = validate_stroke_k1(GeometryK1StrokeSource::Hit { index: hit }, width)
                .map_err(map_stroke_k1_error)?;
            Ok(ValidatedHitCoverage::RoundStroke { shape, stroke })
        }
    }
}

fn validate_terminal_clip(
    paints: &ValidatedPaintItemsProof<'_>,
    hit: u32,
    owner: u32,
    clip: Option<u32>,
) -> Result<Option<u32>, SpatialResolveErrorV2> {
    let Some(clip) = clip else {
        return Ok(None);
    };
    match paints.clip_owner_is_same_or_ancestor_of(clip, owner) {
        None => Err(invalid_reference(
            SpatialContentReferenceV2::Clip,
            hit,
            SpatialHitFieldV2::Clip,
        )),
        Some(false) => Err(content_error(
            SpatialContentErrorKindV2::InvalidClip(SpatialClipErrorV2::ItemOwnerNotDescendant),
            hit_location(hit, SpatialHitFieldV2::Clip),
        )),
        Some(true) => Ok(Some(clip)),
    }
}

fn trusted_hit_ordinal(index: usize) -> u32 {
    u32::try_from(index).expect("phase one validated the hit-item row capacity")
}

const fn hit_location(index: u32, field: SpatialHitFieldV2) -> SpatialErrorLocationV2 {
    SpatialErrorLocationV2::Hit { index, field }
}

fn invalid_reference(
    reference: SpatialContentReferenceV2,
    hit: u32,
    field: SpatialHitFieldV2,
) -> SpatialResolveErrorV2 {
    content_error(
        SpatialContentErrorKindV2::InvalidReference(reference),
        hit_location(hit, field),
    )
}

fn content_error(
    kind: SpatialContentErrorKindV2,
    location: SpatialErrorLocationV2,
) -> SpatialResolveErrorV2 {
    make_resolve_error(SpatialResolveErrorKindV2::Content(kind), location)
}
