use super::validated_paint_support::{
    expect_clip, expect_image, expect_reference, expect_valid, fill, fixture, image_paint, limits,
    source, valid_destination, valid_source, validate,
};
use crate::content_diagnostic::{SpatialContentReferenceV2, SpatialImageErrorV2};
use crate::coverage::SpatialFillRuleV2;
use crate::item_field::SpatialPaintFieldV2;

fn prefix() -> crate::paint::SpatialPaintV2 {
    fill(1, 0, 0, 0, None, SpatialFillRuleV2::NonZero)
}

fn target(source: crate::image::SpatialImageSourceRectV2) -> crate::paint::SpatialPaintV2 {
    image_paint(
        2,
        0,
        1,
        source,
        super::validated_paint_support::destination(
            crate::model::SpatialScalarV2::MAX_RAW + 1,
            crate::model::SpatialScalarV2::MIN_RAW - 1,
            -1,
            0,
        ),
        Some(99),
    )
}

#[test]
fn missing_images_fail_before_source_or_destination_validation() {
    for image in [2, u32::MAX] {
        let fixture = fixture(vec![
            image_paint(1, 0, 0, valid_source(), valid_destination(), None),
            image_paint(
                2,
                0,
                image,
                source(0, 0, 0, 0),
                super::validated_paint_support::destination(i64::MAX, 0, -1, 0),
                Some(u32::MAX),
            ),
        ]);
        expect_reference(
            validate(&fixture, limits(1)),
            SpatialContentReferenceV2::Image,
            1,
            SpatialPaintFieldV2::Image,
        );
    }
}

#[test]
fn a_later_dense_image_key_is_a_valid_reference() {
    let fixture = fixture(vec![image_paint(
        1,
        0,
        1,
        valid_source(),
        valid_destination(),
        None,
    )]);
    expect_valid(validate(&fixture, limits(1)));
}

#[test]
fn image_paint_rejects_an_existing_clip_on_an_unrelated_owner() {
    let fixture = fixture(vec![image_paint(
        4,
        0,
        0,
        valid_source(),
        valid_destination(),
        Some(0),
    )]);
    expect_clip(validate(&fixture, limits(1)), 0);
}

#[test]
fn empty_source_width_precedes_empty_source_height() {
    let input = fixture(vec![prefix(), target(source(0, 0, 0, 0))]);
    expect_image(
        validate(&input, limits(1)),
        SpatialImageErrorV2::EmptySource,
        1,
        SpatialPaintFieldV2::SourceWidth,
    );

    let input = fixture(vec![prefix(), target(source(0, 0, 1, 0))]);
    expect_image(
        validate(&input, limits(1)),
        SpatialImageErrorV2::EmptySource,
        1,
        SpatialPaintFieldV2::SourceHeight,
    );
}

#[test]
fn source_axes_use_x_near_x_far_y_near_y_far_order() {
    let cases = [
        (source(2, 2, 1, 1), SpatialPaintFieldV2::SourceX),
        (source(1, 2, u32::MAX, 1), SpatialPaintFieldV2::SourceWidth),
        (source(0, 2, 1, 1), SpatialPaintFieldV2::SourceY),
        (source(0, 1, 1, u32::MAX), SpatialPaintFieldV2::SourceHeight),
    ];
    for (source, field) in cases {
        let fixture = fixture(vec![prefix(), target(source)]);
        expect_image(
            validate(&fixture, limits(1)),
            SpatialImageErrorV2::SourceOutOfBounds,
            1,
            field,
        );
    }
}

#[test]
fn source_x_failures_precede_simultaneous_source_y_failures() {
    for (source, field) in [
        (source(2, 2, 1, 1), SpatialPaintFieldV2::SourceX),
        (source(1, 2, u32::MAX, 1), SpatialPaintFieldV2::SourceWidth),
    ] {
        let fixture = fixture(vec![prefix(), target(source)]);
        expect_image(
            validate(&fixture, limits(1)),
            SpatialImageErrorV2::SourceOutOfBounds,
            1,
            field,
        );
    }
}
