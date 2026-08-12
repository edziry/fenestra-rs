use super::validated_hit_support::{
    expect_clip, expect_content, expect_reference, expect_stroke, expect_valid, fill, fixture,
    hit_location, limits, stroke, validate,
};
use crate::content_diagnostic::{SpatialContentReferenceV2, SpatialStrokeErrorV2};
use crate::content_error::SpatialContentErrorKindV2;
use crate::content_item::SpatialInputPolicyV2;
use crate::coverage::SpatialFillRuleV2;
use crate::item_field::SpatialHitFieldV2;
use crate::model::SpatialScalarV2;

fn prefix() -> crate::content_item::SpatialHitV2 {
    fill(
        1,
        0,
        0,
        None,
        SpatialFillRuleV2::NonZero,
        SpatialInputPolicyV2::Accept,
    )
}

#[test]
fn missing_and_wrong_owner_shapes_are_invalid_references_even_when_ignored() {
    for (owner, shape) in [(2, 4), (2, u32::MAX), (2, 0), (3, 0)] {
        let fixture = fixture(vec![
            prefix(),
            fill(
                owner,
                0,
                shape,
                Some(u32::MAX),
                SpatialFillRuleV2::EvenOdd,
                SpatialInputPolicyV2::Ignore,
            ),
        ]);
        expect_reference(
            validate(&fixture, limits(1)),
            SpatialContentReferenceV2::Shape,
            1,
            SpatialHitFieldV2::Shape,
        );
    }
}

#[test]
fn every_out_of_domain_stroke_width_maps_to_the_later_hit_record() {
    for raw in [SpatialScalarV2::MIN_RAW - 1, SpatialScalarV2::MAX_RAW + 1] {
        let fixture = fixture(vec![
            prefix(),
            stroke(2, 0, 1, raw, Some(u32::MAX), SpatialInputPolicyV2::Ignore),
        ]);
        expect_content(
            validate(&fixture, limits(1)),
            SpatialContentErrorKindV2::ScalarOutOfDomain,
            hit_location(1, SpatialHitFieldV2::StrokeWidth),
        );
    }
}

#[test]
fn domain_boundaries_then_apply_negative_and_zero_stroke_rules() {
    for raw in [SpatialScalarV2::MIN_RAW, -1] {
        let fixture = fixture(vec![
            prefix(),
            stroke(2, 0, 1, raw, None, SpatialInputPolicyV2::Ignore),
        ]);
        expect_stroke(
            validate(&fixture, limits(1)),
            SpatialStrokeErrorV2::NegativeWidth,
            1,
        );
    }

    let input = fixture(vec![
        prefix(),
        stroke(2, 0, 1, 0, Some(99), SpatialInputPolicyV2::Ignore),
    ]);
    expect_stroke(
        validate(&input, limits(1)),
        SpatialStrokeErrorV2::ZeroWidth,
        1,
    );

    for raw in [1, SpatialScalarV2::MAX_RAW] {
        let input = fixture(vec![
            prefix(),
            stroke(2, 0, 1, raw, None, SpatialInputPolicyV2::Accept),
        ]);
        expect_valid(validate(&input, limits(1)));
    }
}

#[test]
fn shape_then_stroke_then_terminal_clip_is_the_exact_hit_order() {
    let wrong_shape = fixture(vec![
        prefix(),
        stroke(
            2,
            0,
            0,
            SpatialScalarV2::MAX_RAW + 1,
            Some(99),
            SpatialInputPolicyV2::Ignore,
        ),
    ]);
    expect_reference(
        validate(&wrong_shape, limits(1)),
        SpatialContentReferenceV2::Shape,
        1,
        SpatialHitFieldV2::Shape,
    );

    let bad_stroke = fixture(vec![
        prefix(),
        stroke(
            2,
            0,
            1,
            SpatialScalarV2::MAX_RAW + 1,
            Some(99),
            SpatialInputPolicyV2::Ignore,
        ),
    ]);
    expect_content(
        validate(&bad_stroke, limits(1)),
        SpatialContentErrorKindV2::ScalarOutOfDomain,
        hit_location(1, SpatialHitFieldV2::StrokeWidth),
    );

    let bad_clip = fixture(vec![
        prefix(),
        stroke(2, 0, 1, 1, Some(99), SpatialInputPolicyV2::Ignore),
    ]);
    expect_reference(
        validate(&bad_clip, limits(1)),
        SpatialContentReferenceV2::Clip,
        1,
        SpatialHitFieldV2::Clip,
    );
}

#[test]
fn terminal_clips_accept_none_same_owner_and_transitive_ancestors() {
    for (clip, policy) in [
        (None, SpatialInputPolicyV2::Accept),
        (Some(2), SpatialInputPolicyV2::Ignore),
        (Some(1), SpatialInputPolicyV2::Accept),
        (Some(0), SpatialInputPolicyV2::Ignore),
    ] {
        let fixture = fixture(vec![fill(
            3,
            0,
            2,
            clip,
            SpatialFillRuleV2::EvenOdd,
            policy,
        )]);
        expect_valid(validate(&fixture, limits(1)));
    }
}

#[test]
fn missing_unrelated_descendant_and_lower_unrelated_clips_are_distinct() {
    for clip in [4, u32::MAX] {
        let fixture = fixture(vec![fill(
            3,
            0,
            2,
            Some(clip),
            SpatialFillRuleV2::NonZero,
            SpatialInputPolicyV2::Ignore,
        )]);
        expect_reference(
            validate(&fixture, limits(1)),
            SpatialContentReferenceV2::Clip,
            0,
            SpatialHitFieldV2::Clip,
        );
    }

    for (owner, shape, clip) in [(3, 2, 3), (1, 0, 1), (4, 3, 0)] {
        let fixture = fixture(vec![fill(
            owner,
            0,
            shape,
            Some(clip),
            SpatialFillRuleV2::NonZero,
            SpatialInputPolicyV2::Ignore,
        )]);
        expect_clip(validate(&fixture, limits(1)), 0);
    }
}

#[test]
fn round_stroke_also_checks_clip_ancestry_before_retaining_ignore_policy() {
    let fixture = fixture(vec![stroke(
        4,
        0,
        3,
        1,
        Some(0),
        SpatialInputPolicyV2::Ignore,
    )]);
    expect_clip(validate(&fixture, limits(1)), 0);
}
