use super::hit_support::*;
use crate::coverage::SpatialFillRuleV2;
use crate::model::SpatialViewportV2;

#[test]
fn empty_hit_table_and_all_ignored_rows_return_none() {
    let empty = reference_snapshot(std::sync::Arc::new(
        empty_fixture().into_owned(SpatialViewportV2::new(7, 9)),
    ));
    assert_eq!(empty.hit_test(point(0, 0)), None);

    let source = owned_fixture(
        root_and_owners(1, 20, 20),
        vec![super::super::validated_shape_support::rect_values(
            0,
            1,
            0,
            0,
            10 * S,
            10 * S,
        )],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![ignored_fill(1, 0, 0)],
    );
    assert_eq!(reference_snapshot(source).hit_test(point(S, S)), None);

    let empty_bounds = owned_fixture(
        root_and_owners(1, 1, 1),
        vec![super::super::validated_shape_support::rect_values(
            0, 1, 0, 0, 0, S,
        )],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![accepting_fill(1, 0, 0, None, SpatialFillRuleV2::NonZero)],
    );
    assert_eq!(reference_snapshot(empty_bounds).hit_test(point(0, 0)), None);
}

#[test]
fn reverse_record_order_skips_later_misses_and_ignored_rows() {
    let rect = |key, owner, x| {
        super::super::validated_shape_support::rect_values(key, owner, x, 0, 4 * S, 4 * S)
    };
    let source = owned_fixture(
        root_and_owners(3, 40, 20),
        vec![rect(0, 1, 0), rect(1, 2, 0), rect(2, 3, 20 * S)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![
            accepting_fill(1, 0, 0, None, SpatialFillRuleV2::NonZero),
            accepting_fill(2, 0, 1, None, SpatialFillRuleV2::EvenOdd),
            ignored_fill(2, 1, 1),
            accepting_fill(3, 0, 2, None, SpatialFillRuleV2::NonZero),
        ],
    );
    let snapshot = reference_snapshot(source);
    assert_hit(snapshot.hit_test(point(S, 2 * S)), 1, 2, 0, point(S, 2 * S));
}

#[test]
fn later_round_stroke_wins_over_an_earlier_fill() {
    let source = owned_fixture(
        root_and_owners(1, 20, 20),
        vec![
            super::super::validated_shape_support::rect_values(0, 1, 0, 0, 10 * S, 10 * S),
            super::super::validated_shape_support::circle_values(1, 1, 5 * S, 5 * S, 3 * S),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![
            accepting_fill(1, 0, 0, None, SpatialFillRuleV2::NonZero),
            accepting_stroke(1, 1, 1, 2 * S, None),
        ],
    );
    let snapshot = reference_snapshot(source);
    assert_hit(
        snapshot.hit_test(point(8 * S, 5 * S)),
        1,
        1,
        1,
        point(8 * S, 5 * S),
    );
}

#[test]
fn later_fill_wins_over_an_earlier_round_stroke() {
    let source = owned_fixture(
        root_and_owners(1, 20, 20),
        vec![
            super::super::validated_shape_support::circle_values(0, 1, 5 * S, 5 * S, 3 * S),
            super::super::validated_shape_support::rect_values(1, 1, 0, 0, 10 * S, 10 * S),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![
            accepting_stroke(1, 0, 0, 2 * S, None),
            accepting_fill(1, 1, 1, None, SpatialFillRuleV2::NonZero),
        ],
    );
    let snapshot = reference_snapshot(source);
    assert_hit(
        snapshot.hit_test(point(8 * S, 5 * S)),
        1,
        1,
        1,
        point(8 * S, 5 * S),
    );
}

#[test]
fn viewport_does_not_clip_overflowing_hit_geometry() {
    let source = owned_fixture(
        root_and_owners(1, 1, 1),
        vec![super::super::validated_shape_support::rect_values(
            0,
            1,
            30 * S,
            0,
            5 * S,
            5 * S,
        )],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![accepting_fill(1, 0, 0, None, SpatialFillRuleV2::NonZero)],
    );
    let snapshot = reference_snapshot(source);
    assert_hit(
        snapshot.hit_test(point(32 * S, 2 * S)),
        0,
        1,
        0,
        point(32 * S, 2 * S),
    );
}
