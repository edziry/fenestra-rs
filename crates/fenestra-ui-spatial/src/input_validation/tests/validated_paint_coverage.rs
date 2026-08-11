use super::validated_paint_support::{
    expect_clip, expect_content, expect_reference, expect_stroke, expect_valid, fill, fixture,
    limits, paint_location, stroke, validate,
};
use crate::content_diagnostic::{SpatialContentReferenceV2, SpatialStrokeErrorV2};
use crate::content_error::SpatialContentErrorKindV2;
use crate::coverage::SpatialFillRuleV2;
use crate::item_field::SpatialPaintFieldV2;
use crate::model::SpatialScalarV2;

fn prefix() -> crate::paint::SpatialPaintV2 {
    fill(1, 0, 0, 0, None, SpatialFillRuleV2::NonZero)
}

#[test]
fn missing_and_wrong_owner_shapes_are_invalid_references() {
    for (owner, shape) in [(2, 4), (2, u32::MAX), (2, 0), (3, 0)] {
        let fixture = fixture(vec![
            prefix(),
            fill(owner, 0, shape, 0, None, SpatialFillRuleV2::EvenOdd),
        ]);
        expect_reference(
            validate(&fixture, limits(1)),
            SpatialContentReferenceV2::Shape,
            1,
            SpatialPaintFieldV2::Shape,
        );
    }
}

#[test]
fn every_out_of_domain_stroke_width_maps_to_the_paint_field() {
    for raw in [SpatialScalarV2::MIN_RAW - 1, SpatialScalarV2::MAX_RAW + 1] {
        let fixture = fixture(vec![prefix(), stroke(2, 0, 1, raw, 0, None)]);
        expect_content(
            validate(&fixture, limits(1)),
            SpatialContentErrorKindV2::ScalarOutOfDomain,
            paint_location(1, SpatialPaintFieldV2::StrokeWidth),
        );
    }
}

#[test]
fn accepted_domain_boundaries_then_apply_negative_and_zero_stroke_rules() {
    for raw in [SpatialScalarV2::MIN_RAW, -1] {
        let fixture = fixture(vec![prefix(), stroke(2, 0, 1, raw, 0, None)]);
        expect_stroke(
            validate(&fixture, limits(1)),
            SpatialStrokeErrorV2::NegativeWidth,
            1,
        );
    }

    let input = fixture(vec![prefix(), stroke(2, 0, 1, 0, 0, None)]);
    expect_stroke(
        validate(&input, limits(1)),
        SpatialStrokeErrorV2::ZeroWidth,
        1,
    );

    for raw in [1, SpatialScalarV2::MAX_RAW] {
        let input = fixture(vec![prefix(), stroke(2, 0, 1, raw, 0, None)]);
        expect_valid(validate(&input, limits(1)));
    }
}

#[test]
fn shape_then_stroke_then_brush_then_clip_is_the_exact_coverage_order() {
    let cases = [
        (
            0,
            SpatialScalarV2::MAX_RAW + 1,
            2,
            Some(9),
            SpatialPaintFieldV2::Shape,
        ),
        (
            1,
            SpatialScalarV2::MAX_RAW + 1,
            2,
            Some(9),
            SpatialPaintFieldV2::StrokeWidth,
        ),
        (1, 1, 2, Some(9), SpatialPaintFieldV2::Brush),
    ];
    for (shape, width, brush, clip, field) in cases {
        let fixture = fixture(vec![prefix(), stroke(2, 0, shape, width, brush, clip)]);
        let result = validate(&fixture, limits(1));
        match field {
            SpatialPaintFieldV2::Shape => {
                expect_reference(result, SpatialContentReferenceV2::Shape, 1, field)
            }
            SpatialPaintFieldV2::StrokeWidth => expect_content(
                result,
                SpatialContentErrorKindV2::ScalarOutOfDomain,
                paint_location(1, field),
            ),
            SpatialPaintFieldV2::Brush => {
                expect_reference(result, SpatialContentReferenceV2::Brush, 1, field)
            }
            _ => unreachable!("the cases cover the failing coverage stages"),
        }
    }

    let fixture = fixture(vec![prefix(), stroke(2, 0, 1, 1, 0, Some(9))]);
    expect_reference(
        validate(&fixture, limits(1)),
        SpatialContentReferenceV2::Clip,
        1,
        SpatialPaintFieldV2::Clip,
    );
}

#[test]
fn fill_coverage_also_validates_its_brush_reference() {
    for brush in [2, u32::MAX] {
        let fixture = fixture(vec![
            prefix(),
            fill(2, 0, 1, brush, Some(99), SpatialFillRuleV2::EvenOdd),
        ]);
        expect_reference(
            validate(&fixture, limits(1)),
            SpatialContentReferenceV2::Brush,
            1,
            SpatialPaintFieldV2::Brush,
        );
    }
}

#[test]
fn terminal_clips_accept_none_same_owner_and_transitive_ancestors() {
    for clip in [None, Some(2), Some(1), Some(0)] {
        let fixture = fixture(vec![fill(3, 0, 2, 0, clip, SpatialFillRuleV2::EvenOdd)]);
        expect_valid(validate(&fixture, limits(1)));
    }
}

#[test]
fn missing_unrelated_and_descendant_terminal_clips_are_distinct() {
    for clip in [4, u32::MAX] {
        let fixture = fixture(vec![fill(
            3,
            0,
            2,
            0,
            Some(clip),
            SpatialFillRuleV2::NonZero,
        )]);
        expect_reference(
            validate(&fixture, limits(1)),
            SpatialContentReferenceV2::Clip,
            0,
            SpatialPaintFieldV2::Clip,
        );
    }

    let unrelated = fixture(vec![fill(3, 0, 2, 0, Some(3), SpatialFillRuleV2::NonZero)]);
    expect_clip(validate(&unrelated, limits(1)), 0);

    let descendant = fixture(vec![fill(1, 0, 0, 0, Some(1), SpatialFillRuleV2::NonZero)]);
    expect_clip(validate(&descendant, limits(1)), 0);

    let lower_but_unrelated = fixture(vec![fill(4, 0, 3, 0, Some(0), SpatialFillRuleV2::NonZero)]);
    expect_clip(validate(&lower_but_unrelated, limits(1)), 0);
}
