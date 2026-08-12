//! Lifetime-free clip and item-plan extraction.

use super::super::model::{
    PreparedClipPlan, PreparedCoverage, PreparedHitPlan, PreparedPaintContent, PreparedPaintPlan,
    PreparedSemanticPlan,
};
use crate::aabb::SpatialAabbV2;
use crate::image::{SpatialImageDestinationRectV2, SpatialImageSourceRectV2};
use crate::input_validation::local_bounds::PaintLocalBounds;
use crate::input_validation::validated_clips::ValidatedClipPlan;
use crate::input_validation::validated_hit_items::{ValidatedHitCoverage, ValidatedHitItem};
use crate::input_validation::validated_paint_items::{
    ValidatedPaintContent, ValidatedPaintCoverage, ValidatedPaintItem,
};
use crate::input_validation::validated_semantic_items::ValidatedSemanticItem;
use crate::paint_kernel::ValidatedImageP4;

type ImagePaintParts<'a> = (
    u32,
    ValidatedImageP4<'a>,
    SpatialImageSourceRectV2,
    SpatialImageDestinationRectV2,
    u8,
);

pub(super) fn extract_clips(clips: Vec<ValidatedClipPlan>) -> Box<[PreparedClipPlan]> {
    clips
        .into_iter()
        .map(|clip| {
            let (owner, parent, shape, fill_rule, depth) = clip.into_parts();
            PreparedClipPlan {
                owner,
                parent,
                shape,
                fill_rule,
                depth,
            }
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

pub(super) fn extract_paints(
    paints: Vec<ValidatedPaintItem<'_>>,
    bounds: Vec<PaintLocalBounds<'_>>,
) -> Box<[PreparedPaintPlan]> {
    assert_eq!(
        paints.len(),
        bounds.len(),
        "paint bounds remain row aligned"
    );
    paints
        .into_iter()
        .zip(bounds)
        .enumerate()
        .map(|(index, (paint, bounds))| {
            let (owner, item_ordinal, content) = paint.into_parts();
            let (content, local_bounds) = match (content, bounds) {
                (
                    ValidatedPaintContent::Coverage {
                        coverage,
                        brush,
                        opacity,
                        clip,
                    },
                    PaintLocalBounds::Coverage(local_bounds),
                ) => (
                    PreparedPaintContent::Coverage {
                        coverage: paint_coverage(coverage),
                        brush,
                        opacity,
                        clip,
                    },
                    local_bounds,
                ),
                (
                    ValidatedPaintContent::Image {
                        image,
                        preclip,
                        clip,
                    },
                    PaintLocalBounds::Image(finalized),
                ) => {
                    let expected = preclip.into_parts();
                    let (finalized, local_bounds) = finalized.into_parts();
                    let actual = finalized.into_parts();
                    assert_image_tokens(index, image, &expected, &actual);
                    (
                        PreparedPaintContent::Image {
                            image,
                            source: actual.2,
                            destination: actual.3,
                            opacity: actual.4,
                            clip,
                        },
                        local_bounds,
                    )
                }
                _ => panic!("paint content and finalized local bounds remain variant aligned"),
            };
            PreparedPaintPlan {
                owner,
                item_ordinal,
                content,
                local_bounds,
            }
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

pub(super) fn extract_hits(
    hits: Vec<ValidatedHitItem>,
    bounds: Vec<SpatialAabbV2>,
) -> Box<[PreparedHitPlan]> {
    assert_eq!(hits.len(), bounds.len(), "hit bounds remain row aligned");
    hits.into_iter()
        .zip(bounds)
        .map(|(hit, local_bounds)| {
            let (owner, item_ordinal, coverage, clip, input_policy) = hit.into_parts();
            PreparedHitPlan {
                owner,
                item_ordinal,
                coverage: hit_coverage(coverage),
                input_policy,
                clip,
                local_bounds,
            }
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

pub(super) fn extract_semantics(
    semantics: Vec<ValidatedSemanticItem>,
) -> Box<[PreparedSemanticPlan]> {
    semantics
        .into_iter()
        .map(|semantic| {
            let (owner, item_ordinal, shape, fill_rule, clip) = semantic.into_parts();
            PreparedSemanticPlan {
                owner,
                item_ordinal,
                shape,
                fill_rule,
                clip,
            }
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

fn paint_coverage(coverage: ValidatedPaintCoverage) -> PreparedCoverage {
    match coverage {
        ValidatedPaintCoverage::Fill { shape, rule } => PreparedCoverage::Fill { shape, rule },
        ValidatedPaintCoverage::RoundStroke { shape, stroke } => {
            PreparedCoverage::RoundStroke { shape, stroke }
        }
    }
}

fn hit_coverage(coverage: ValidatedHitCoverage) -> PreparedCoverage {
    match coverage {
        ValidatedHitCoverage::Fill { shape, rule } => PreparedCoverage::Fill { shape, rule },
        ValidatedHitCoverage::RoundStroke { shape, stroke } => {
            PreparedCoverage::RoundStroke { shape, stroke }
        }
    }
}

fn assert_image_tokens(
    index: usize,
    image: u32,
    expected: &ImagePaintParts<'_>,
    actual: &ImagePaintParts<'_>,
) {
    let expected_image = expected.1.into_parts();
    let actual_image = actual.1.into_parts();
    let ordinal = u32::try_from(index).expect("validated paint ordinal fits u32");
    assert_eq!(expected.0, ordinal, "preclip paint ordinal remains aligned");
    assert_eq!(actual.0, ordinal, "finalized paint ordinal remains aligned");
    assert_eq!(
        expected_image, actual_image,
        "P5 retains the validated image token"
    );
    assert_eq!(
        actual_image.0, image,
        "P5 image key remains aligned with the paint"
    );
    assert_eq!(expected.2, actual.2, "P5 source remains aligned");
    assert_eq!(expected.3, actual.3, "P5 destination remains aligned");
    assert_eq!(expected.4, actual.4, "P5 opacity remains aligned");
}
