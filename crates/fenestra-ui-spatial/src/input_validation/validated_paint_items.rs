//! Ordered paint-item validation and item-local kernel dispatch.

use super::make_resolve_error;
use super::ordered_items::OrderedItemCursor;
use super::paint_p5_mapping::map_paint_p5_error;
use super::stroke_k1_mapping::map_stroke_k1_error;
use super::validated_clips::ValidatedClipsProof;
use crate::aggregate_input::SpatialInputV2;
use crate::content_diagnostic::{SpatialClipErrorV2, SpatialContentReferenceV2};
use crate::content_error::SpatialContentErrorKindV2;
use crate::coverage::{SpatialCoverageV2, SpatialFillRuleV2};
use crate::error::SpatialErrorLocationV2;
use crate::geometry_kernel::{GeometryK1StrokeSource, ValidatedStrokeK1, validate_stroke_k1};
use crate::item_field::SpatialPaintFieldV2;
use crate::limits::{SpatialLimitKindV2, SpatialLimitsV2};
use crate::paint::SpatialPaintContentV2;
use crate::paint_kernel::{PreclipImagePaintP5, prepare_image_paint_p5};
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};

#[cfg(test)]
mod facts;

struct ValidatedPaintItem<'a> {
    owner: u32,
    item_ordinal: u32,
    content: ValidatedPaintContent<'a>,
}

enum ValidatedPaintContent<'a> {
    Coverage {
        coverage: ValidatedPaintCoverage,
        brush: u32,
        opacity: u8,
        clip: Option<u32>,
    },
    Image {
        image: u32,
        preclip: PreclipImagePaintP5<'a>,
        clip: Option<u32>,
    },
}

enum ValidatedPaintCoverage {
    Fill {
        shape: u32,
        rule: SpatialFillRuleV2,
    },
    RoundStroke {
        shape: u32,
        stroke: ValidatedStrokeK1,
    },
}

pub(super) struct ValidatedPaintItemsProof<'a> {
    clips: ValidatedClipsProof<'a>,
    paints: Vec<ValidatedPaintItem<'a>>,
}

impl<'a> ValidatedPaintItemsProof<'a> {
    pub(super) fn input(&self) -> SpatialInputV2<'a> {
        self.clips.input()
    }

    pub(super) fn limits(&self) -> SpatialLimitsV2 {
        self.clips.limits()
    }

    pub(super) fn clip_owner_is_same_or_ancestor_of(&self, clip: u32, owner: u32) -> Option<bool> {
        self.clips.clip_owner_is_same_or_ancestor_of(clip, owner)
    }
}

pub(super) fn prepare_validated_paint_items<'a>(
    clips: ValidatedClipsProof<'a>,
) -> Result<ValidatedPaintItemsProof<'a>, SpatialResolveErrorV2> {
    let input = clips.input();
    let paints = input.items().paint_items();
    let node_count = input.topology().nodes().len() as u128;
    let limits = clips.limits();
    let mut cursor = OrderedItemCursor::new();
    let mut validated = Vec::with_capacity(paints.len());

    for (index, paint) in paints.iter().copied().enumerate() {
        let ordinal = trusted_paint_ordinal(index);
        let owner = paint.owner().get();
        let candidate = cursor.validate(
            crate::content_diagnostic::SpatialOrderedItemTableV2::Paint,
            ordinal,
            owner,
            paint.item_ordinal(),
            node_count,
        )?;
        validate_paint_item_limit(ordinal, candidate.owner_count(), limits)?;

        let content = match paint.content() {
            SpatialPaintContentV2::CoveragePaint {
                coverage,
                brush,
                opacity,
                clip,
            } => {
                let coverage = validate_coverage(&clips, ordinal, owner, coverage)?;
                let brush = brush.get();
                if u128::from(brush) >= input.resources().brushes().len() as u128 {
                    return Err(invalid_reference(
                        SpatialContentReferenceV2::Brush,
                        ordinal,
                        SpatialPaintFieldV2::Brush,
                    ));
                }
                let clip =
                    validate_terminal_clip(&clips, ordinal, owner, clip.map(|key| key.get()))?;
                ValidatedPaintContent::Coverage {
                    coverage,
                    brush,
                    opacity,
                    clip,
                }
            }
            SpatialPaintContentV2::ImagePaint {
                image,
                source,
                destination,
                opacity,
                clip,
            } => {
                let image = image.get();
                let Some(validated_image) = clips.validated_image(image) else {
                    return Err(invalid_reference(
                        SpatialContentReferenceV2::Image,
                        ordinal,
                        SpatialPaintFieldV2::Image,
                    ));
                };
                let preclip =
                    prepare_image_paint_p5(ordinal, &validated_image, source, destination, opacity)
                        .map_err(map_paint_p5_error)?;
                let clip =
                    validate_terminal_clip(&clips, ordinal, owner, clip.map(|key| key.get()))?;
                ValidatedPaintContent::Image {
                    image,
                    preclip,
                    clip,
                }
            }
        };

        validated.push(ValidatedPaintItem {
            owner,
            item_ordinal: candidate.item_ordinal(),
            content,
        });
        cursor.commit(candidate);
    }

    Ok(ValidatedPaintItemsProof {
        clips,
        paints: validated,
    })
}

pub(super) fn validate_paint_item_limit(
    paint: u32,
    observed: usize,
    limits: SpatialLimitsV2,
) -> Result<(), SpatialResolveErrorV2> {
    let observed = observed as u128;
    let maximum = limits.limit(SpatialLimitKindV2::PaintItemsPerNode) as u128;
    if observed > maximum {
        return Err(SpatialResolveErrorV2::limit_exceeded(
            SpatialLimitKindV2::PaintItemsPerNode,
            paint_location(paint, SpatialPaintFieldV2::ItemOrdinal),
            observed,
            maximum,
        ));
    }
    Ok(())
}

fn validate_coverage(
    clips: &ValidatedClipsProof<'_>,
    paint: u32,
    owner: u32,
    coverage: SpatialCoverageV2,
) -> Result<ValidatedPaintCoverage, SpatialResolveErrorV2> {
    let input = clips.input();
    let shapes = input.geometry().shapes();
    let validate_shape = |shape: u32| {
        if u128::from(shape) >= shapes.len() as u128
            || shapes[shape as usize].owner().get() != owner
        {
            Err(invalid_reference(
                SpatialContentReferenceV2::Shape,
                paint,
                SpatialPaintFieldV2::Shape,
            ))
        } else {
            Ok(())
        }
    };
    match coverage {
        SpatialCoverageV2::Fill { shape, rule } => {
            let shape = shape.get();
            validate_shape(shape)?;
            Ok(ValidatedPaintCoverage::Fill { shape, rule })
        }
        SpatialCoverageV2::RoundStroke { shape, width } => {
            let shape = shape.get();
            validate_shape(shape)?;
            let stroke = validate_stroke_k1(GeometryK1StrokeSource::Paint { index: paint }, width)
                .map_err(map_stroke_k1_error)?;
            Ok(ValidatedPaintCoverage::RoundStroke { shape, stroke })
        }
    }
}

fn validate_terminal_clip(
    clips: &ValidatedClipsProof<'_>,
    paint: u32,
    owner: u32,
    clip: Option<u32>,
) -> Result<Option<u32>, SpatialResolveErrorV2> {
    let Some(clip) = clip else {
        return Ok(None);
    };
    match clips.clip_owner_is_same_or_ancestor_of(clip, owner) {
        None => Err(invalid_reference(
            SpatialContentReferenceV2::Clip,
            paint,
            SpatialPaintFieldV2::Clip,
        )),
        Some(false) => Err(content_error(
            SpatialContentErrorKindV2::InvalidClip(SpatialClipErrorV2::ItemOwnerNotDescendant),
            paint_location(paint, SpatialPaintFieldV2::Clip),
        )),
        Some(true) => Ok(Some(clip)),
    }
}

fn trusted_paint_ordinal(index: usize) -> u32 {
    u32::try_from(index).expect("phase one validated the paint-item row capacity")
}

const fn paint_location(index: u32, field: SpatialPaintFieldV2) -> SpatialErrorLocationV2 {
    SpatialErrorLocationV2::Paint { index, field }
}

fn invalid_reference(
    reference: SpatialContentReferenceV2,
    paint: u32,
    field: SpatialPaintFieldV2,
) -> SpatialResolveErrorV2 {
    content_error(
        SpatialContentErrorKindV2::InvalidReference(reference),
        paint_location(paint, field),
    )
}

fn content_error(
    kind: SpatialContentErrorKindV2,
    location: SpatialErrorLocationV2,
) -> SpatialResolveErrorV2 {
    make_resolve_error(SpatialResolveErrorKindV2::Content(kind), location)
}
