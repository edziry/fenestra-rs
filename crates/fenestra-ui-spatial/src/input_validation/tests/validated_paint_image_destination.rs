use super::validated_paint_support::{
    destination, expect_content, expect_image, expect_valid, fill, fixture, image_paint, limits,
    paint_location, valid_source, validate,
};
use crate::content_diagnostic::SpatialImageErrorV2;
use crate::content_error::SpatialContentErrorKindV2;
use crate::coverage::SpatialFillRuleV2;
use crate::item_field::SpatialPaintFieldV2;
use crate::model::SpatialScalarV2;
use crate::vocabulary::SpatialExtentV2;

fn prefix() -> crate::paint::SpatialPaintV2 {
    fill(1, 0, 0, 0, None, SpatialFillRuleV2::NonZero)
}

fn target(
    destination: crate::image::SpatialImageDestinationRectV2,
) -> crate::paint::SpatialPaintV2 {
    image_paint(2, 0, 1, valid_source(), destination, Some(99))
}

#[test]
fn every_destination_scalar_maps_both_domain_sides_to_its_exact_field() {
    for field in [
        SpatialPaintFieldV2::DestinationX,
        SpatialPaintFieldV2::DestinationY,
        SpatialPaintFieldV2::DestinationWidth,
        SpatialPaintFieldV2::DestinationHeight,
    ] {
        for raw in [SpatialScalarV2::MIN_RAW - 1, SpatialScalarV2::MAX_RAW + 1] {
            let values = match field {
                SpatialPaintFieldV2::DestinationX => [raw, 0, 1, 1],
                SpatialPaintFieldV2::DestinationY => [0, raw, 1, 1],
                SpatialPaintFieldV2::DestinationWidth => [0, 0, raw, 1],
                SpatialPaintFieldV2::DestinationHeight => [0, 0, 1, raw],
                _ => unreachable!("the table contains only destination scalar fields"),
            };
            let fixture = fixture(vec![
                prefix(),
                target(destination(values[0], values[1], values[2], values[3])),
            ]);
            expect_content(
                validate(&fixture, limits(1)),
                SpatialContentErrorKindV2::ScalarOutOfDomain,
                paint_location(1, field),
            );
        }
    }
}

#[test]
fn destination_scalar_order_is_x_y_width_then_height() {
    let low = SpatialScalarV2::MIN_RAW - 1;
    for (destination, field) in [
        (
            destination(low, low, low, low),
            SpatialPaintFieldV2::DestinationX,
        ),
        (
            destination(0, low, low, low),
            SpatialPaintFieldV2::DestinationY,
        ),
        (
            destination(0, 0, low, low),
            SpatialPaintFieldV2::DestinationWidth,
        ),
        (
            destination(0, 0, 1, low),
            SpatialPaintFieldV2::DestinationHeight,
        ),
    ] {
        let fixture = fixture(vec![prefix(), target(destination)]);
        expect_content(
            validate(&fixture, limits(1)),
            SpatialContentErrorKindV2::ScalarOutOfDomain,
            paint_location(1, field),
        );
    }
}

#[test]
fn negative_destination_width_precedes_height_after_all_scalars() {
    let input = fixture(vec![prefix(), target(destination(0, 0, -1, -1))]);
    expect_image(
        validate(&input, limits(1)),
        SpatialImageErrorV2::NegativeDestinationExtent(SpatialExtentV2::Width),
        1,
        SpatialPaintFieldV2::DestinationWidth,
    );

    let input = fixture(vec![prefix(), target(destination(0, 0, 1, -1))]);
    expect_image(
        validate(&input, limits(1)),
        SpatialImageErrorV2::NegativeDestinationExtent(SpatialExtentV2::Height),
        1,
        SpatialPaintFieldV2::DestinationHeight,
    );
}

#[test]
fn all_destination_scalars_precede_negative_extent_checks() {
    let input = fixture(vec![
        prefix(),
        target(destination(0, 0, -1, SpatialScalarV2::MAX_RAW + 1)),
    ]);
    expect_content(
        validate(&input, limits(1)),
        SpatialContentErrorKindV2::ScalarOutOfDomain,
        paint_location(1, SpatialPaintFieldV2::DestinationHeight),
    );
}

#[test]
fn empty_destination_width_precedes_height_after_negative_checks() {
    let input = fixture(vec![prefix(), target(destination(0, 0, 0, 0))]);
    expect_image(
        validate(&input, limits(1)),
        SpatialImageErrorV2::EmptyDestination,
        1,
        SpatialPaintFieldV2::DestinationWidth,
    );

    let input = fixture(vec![prefix(), target(destination(0, 0, 1, 0))]);
    expect_image(
        validate(&input, limits(1)),
        SpatialImageErrorV2::EmptyDestination,
        1,
        SpatialPaintFieldV2::DestinationHeight,
    );
}

#[test]
fn all_negative_checks_precede_empty_destination_checks() {
    let input = fixture(vec![prefix(), target(destination(0, 0, 0, -1))]);
    expect_image(
        validate(&input, limits(1)),
        SpatialImageErrorV2::NegativeDestinationExtent(SpatialExtentV2::Height),
        1,
        SpatialPaintFieldV2::DestinationHeight,
    );
}

#[test]
fn inclusive_scalar_boundaries_with_positive_extents_are_accepted_preclip() {
    for destination in [
        destination(SpatialScalarV2::MIN_RAW, SpatialScalarV2::MAX_RAW, 1, 1),
        destination(0, 0, SpatialScalarV2::MAX_RAW, SpatialScalarV2::MAX_RAW),
    ] {
        let fixture = fixture(vec![
            prefix(),
            image_paint(2, 0, 1, valid_source(), destination, None),
        ]);
        expect_valid(validate(&fixture, limits(1)));
    }
}
