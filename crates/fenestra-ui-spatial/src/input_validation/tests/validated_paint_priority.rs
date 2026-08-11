use super::validated_clip_support::{expect_non_dense, root_clip};
use super::validated_image_support::{expect_image as expect_prior_image, image_location};
use super::validated_paint_support::{
    destination, expect_image, expect_reference, fill, fixture, image_paint, limits, source,
    stroke, valid_destination, valid_source, validate,
};
use crate::content_diagnostic::{SpatialContentReferenceV2, SpatialImageErrorV2};
use crate::coverage::SpatialFillRuleV2;
use crate::error::SpatialErrorLocationV2;
use crate::geometry_field::SpatialImageFieldV2;
use crate::item_field::{SpatialClipFieldV2, SpatialPaintFieldV2};

fn late_clip(owner: u32, ordinal: u32) -> crate::paint::SpatialPaintV2 {
    fill(
        owner,
        ordinal,
        owner - 1,
        0,
        Some(99),
        SpatialFillRuleV2::NonZero,
    )
}

#[test]
fn a_late_failure_in_record_zero_beats_every_header_stage_in_record_one() {
    let cases = [
        vec![
            late_clip(1, 0),
            fill(0, 99, 0, 0, None, SpatialFillRuleV2::NonZero),
        ],
        vec![
            late_clip(2, 0),
            fill(1, 0, 0, 0, None, SpatialFillRuleV2::NonZero),
        ],
        vec![
            late_clip(1, 0),
            fill(1, 2, 0, 0, None, SpatialFillRuleV2::NonZero),
        ],
        vec![
            late_clip(1, 0),
            fill(1, 1, 0, 0, None, SpatialFillRuleV2::NonZero),
        ],
    ];
    for (case, paints) in cases.into_iter().enumerate() {
        let fixture = fixture(paints);
        let maximum = if case == 3 { 1 } else { usize::MAX };
        expect_reference(
            validate(&fixture, limits(maximum)),
            SpatialContentReferenceV2::Clip,
            0,
            SpatialPaintFieldV2::Clip,
        );
    }
}

#[test]
fn a_late_failure_in_record_zero_beats_later_stroke_brush_and_p5_stages() {
    let later_records = [
        stroke(1, 1, 0, crate::model::SpatialScalarV2::MAX_RAW + 1, 0, None),
        fill(1, 1, 0, 99, None, SpatialFillRuleV2::EvenOdd),
        image_paint(1, 1, 0, source(0, 0, 0, 0), valid_destination(), None),
    ];
    for later in later_records {
        let fixture = fixture(vec![late_clip(1, 0), later]);
        expect_reference(
            validate(&fixture, limits(2)),
            SpatialContentReferenceV2::Clip,
            0,
            SpatialPaintFieldV2::Clip,
        );
    }
}

#[test]
fn coverage_and_image_records_are_not_batched_by_variant() {
    let coverage_first = fixture(vec![
        late_clip(1, 0),
        image_paint(1, 1, 99, valid_source(), valid_destination(), None),
    ]);
    expect_reference(
        validate(&coverage_first, limits(2)),
        SpatialContentReferenceV2::Clip,
        0,
        SpatialPaintFieldV2::Clip,
    );

    let image_first = fixture(vec![
        image_paint(1, 0, 0, valid_source(), destination(0, 0, 0, 1), None),
        fill(1, 1, 99, 0, None, SpatialFillRuleV2::NonZero),
    ]);
    expect_image(
        validate(&image_first, limits(2)),
        SpatialImageErrorV2::EmptyDestination,
        0,
        SpatialPaintFieldV2::DestinationWidth,
    );
}

#[test]
fn a_late_p5_failure_beats_the_next_records_owner_reference() {
    let fixture = fixture(vec![
        image_paint(1, 0, 0, valid_source(), destination(0, 0, 0, 1), None),
        fill(0, 99, 0, 0, None, SpatialFillRuleV2::NonZero),
    ]);
    expect_image(
        validate(&fixture, limits(usize::MAX)),
        SpatialImageErrorV2::EmptyDestination,
        0,
        SpatialPaintFieldV2::DestinationWidth,
    );
}

#[test]
fn an_image_terminal_clip_beats_a_later_coverage_shape() {
    let fixture = fixture(vec![
        image_paint(1, 0, 0, valid_source(), valid_destination(), Some(99)),
        fill(1, 1, 99, 0, None, SpatialFillRuleV2::NonZero),
    ]);
    expect_reference(
        validate(&fixture, limits(2)),
        SpatialContentReferenceV2::Clip,
        0,
        SpatialPaintFieldV2::Clip,
    );
}

#[test]
fn an_earlier_coverage_shape_beats_later_image_p5_validation() {
    let fixture = fixture(vec![
        fill(1, 0, 99, 0, None, SpatialFillRuleV2::NonZero),
        image_paint(1, 1, 0, source(0, 0, 0, 0), destination(0, 0, 0, 0), None),
    ]);
    expect_reference(
        validate(&fixture, limits(2)),
        SpatialContentReferenceV2::Shape,
        0,
        SpatialPaintFieldV2::Shape,
    );
}

#[test]
fn terminal_clip_checks_precede_deferred_image_far_edges() {
    let maximum = crate::model::SpatialScalarV2::MAX_RAW;
    let fixture = fixture(vec![image_paint(
        1,
        0,
        0,
        valid_source(),
        destination(maximum, maximum, 1, 1),
        Some(99),
    )]);
    expect_reference(
        validate(&fixture, limits(1)),
        SpatialContentReferenceV2::Clip,
        0,
        SpatialPaintFieldV2::Clip,
    );
}

#[test]
fn every_later_paint_failure_precedes_an_earlier_deferred_far_edge() {
    let maximum = crate::model::SpatialScalarV2::MAX_RAW;
    for later in [
        image_paint(1, 1, 99, valid_source(), valid_destination(), None),
        fill(1, 1, 99, 0, None, SpatialFillRuleV2::NonZero),
    ] {
        let fixture = fixture(vec![
            image_paint(
                1,
                0,
                0,
                valid_source(),
                destination(maximum, maximum, 1, 1),
                None,
            ),
            later,
        ]);
        let reference = match later.content() {
            crate::paint::SpatialPaintContentV2::ImagePaint { .. } => {
                SpatialContentReferenceV2::Image
            }
            crate::paint::SpatialPaintContentV2::CoveragePaint { .. } => {
                SpatialContentReferenceV2::Shape
            }
        };
        let field = match reference {
            SpatialContentReferenceV2::Image => SpatialPaintFieldV2::Image,
            SpatialContentReferenceV2::Shape => SpatialPaintFieldV2::Shape,
            _ => unreachable!("the cases contain image and shape failures"),
        };
        expect_reference(validate(&fixture, limits(2)), reference, 1, field);
    }
}

#[test]
fn validated_images_and_clips_precede_every_paint_check() {
    let invalid_image = fixture(vec![fill(
        0,
        99,
        99,
        99,
        Some(99),
        SpatialFillRuleV2::NonZero,
    )])
    .with_images(vec![super::validated_image_support::image(
        0,
        0,
        0,
        0,
        Vec::new(),
    )]);
    expect_prior_image(
        prepare_validated_paint_items!(
            &invalid_image,
            super::local_transform_support::VIEWPORT,
            limits(0)
        ),
        SpatialImageErrorV2::ZeroExtent,
        image_location(0, SpatialImageFieldV2::Width),
    );

    let invalid_clip = fixture(vec![fill(
        0,
        99,
        99,
        99,
        Some(99),
        SpatialFillRuleV2::NonZero,
    )])
    .with_clips(vec![root_clip(1, 1, 0)]);
    expect_non_dense(
        prepare_validated_paint_items!(
            &invalid_clip,
            super::local_transform_support::VIEWPORT,
            limits(0)
        ),
        SpatialErrorLocationV2::Clip {
            index: 0,
            field: SpatialClipFieldV2::Key,
        },
    );
}
