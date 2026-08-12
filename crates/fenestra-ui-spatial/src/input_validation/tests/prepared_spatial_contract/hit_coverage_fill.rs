use super::hit_support::*;
use crate::coverage::SpatialFillRuleV2;

#[test]
fn k4_dispatches_all_shape_kinds_with_exact_fill_rules() {
    use super::super::flattened_path_support::{line_to, move_to, path};
    use super::super::validated_shape_support::{circle_values, path_shape, polygon, rect_values};

    let source = owned_fixture(
        root_and_owners(1, 40, 10),
        vec![
            rect_values(0, 1, 0, 0, 4 * S, 4 * S),
            circle_values(1, 1, 10 * S, 2 * S, 2 * S),
            polygon(2, 1, 0, 3),
            polygon(3, 1, 3, 3),
            path_shape(4, 1, 1),
        ],
        vec![
            point(15 * S, 0),
            point(16 * S, 0),
            point(15 * S, S),
            point(20 * S, 0),
            point(24 * S, 0),
            point(20 * S, 4 * S),
        ],
        vec![path(0, 0, 2), path(1, 2, 3)],
        vec![
            move_to(15 * S, 5 * S),
            line_to(16 * S, 5 * S),
            move_to(30 * S, 0),
            line_to(34 * S, 0),
            line_to(30 * S, 4 * S),
        ],
        Vec::new(),
        vec![
            accepting_fill(1, 0, 0, None, SpatialFillRuleV2::EvenOdd),
            accepting_fill(1, 1, 1, None, SpatialFillRuleV2::NonZero),
            accepting_fill(1, 2, 3, None, SpatialFillRuleV2::EvenOdd),
            accepting_fill(1, 3, 4, None, SpatialFillRuleV2::NonZero),
        ],
    );
    let snapshot = reference_snapshot(source);

    for (query, key) in [
        (point(S, S), 0),
        (point(12 * S, 2 * S), 1),
        (point(21 * S, S), 2),
        (point(31 * S, S), 3),
    ] {
        assert_hit(snapshot.hit_test(query), key, 1, key, query);
    }
}

#[test]
fn k4_preserves_nonzero_vs_even_odd_for_repeated_path_winding() {
    use super::super::flattened_path_support::{line_to, move_to, path};
    use super::super::validated_shape_support::path_shape;
    use crate::path::SpatialPathVerbV2;

    let winding = vec![
        move_to(0, 0),
        line_to(4 * S, 0),
        line_to(4 * S, 4 * S),
        line_to(0, 4 * S),
        SpatialPathVerbV2::Close,
    ];
    let mut verbs = vec![move_to(8 * S, 8 * S), line_to(9 * S, 8 * S)];
    verbs.extend(winding.iter().copied());
    verbs.extend(winding);
    let source = owned_fixture(
        root_and_owners(1, 10, 10),
        vec![path_shape(0, 1, 1)],
        Vec::new(),
        vec![path(0, 0, 2), path(1, 2, 10)],
        verbs,
        Vec::new(),
        vec![
            accepting_fill(1, 0, 0, None, SpatialFillRuleV2::NonZero),
            accepting_fill(1, 1, 0, None, SpatialFillRuleV2::EvenOdd),
        ],
    );
    let snapshot = reference_snapshot(source);
    assert_hit(
        snapshot.hit_test(point(2 * S, 2 * S)),
        0,
        1,
        0,
        point(2 * S, 2 * S),
    );
}

#[test]
fn k4_rejects_aabb_false_positives_and_half_open_rect_far_edges() {
    use super::super::flattened_path_support::{line_to, move_to, path};
    use super::super::validated_shape_support::{circle_values, path_shape, polygon, rect_values};

    let source = owned_fixture(
        root_and_owners(1, 40, 10),
        vec![
            rect_values(0, 1, 0, 0, 4 * S, 4 * S),
            circle_values(1, 1, 10 * S, 2 * S, 2 * S),
            polygon(2, 1, 0, 3),
            polygon(3, 1, 3, 3),
            path_shape(4, 1, 1),
        ],
        vec![
            point(15 * S, 0),
            point(16 * S, 0),
            point(15 * S, S),
            point(20 * S, 0),
            point(24 * S, 0),
            point(20 * S, 4 * S),
        ],
        vec![path(0, 0, 2), path(1, 2, 3)],
        vec![
            move_to(15 * S, 5 * S),
            line_to(16 * S, 5 * S),
            move_to(30 * S, 0),
            line_to(34 * S, 0),
            line_to(30 * S, 4 * S),
        ],
        Vec::new(),
        vec![
            accepting_fill(1, 0, 0, None, SpatialFillRuleV2::NonZero),
            accepting_fill(1, 1, 1, None, SpatialFillRuleV2::NonZero),
            accepting_fill(1, 2, 3, None, SpatialFillRuleV2::NonZero),
            accepting_fill(1, 3, 4, None, SpatialFillRuleV2::NonZero),
        ],
    );
    let snapshot = reference_snapshot(source);

    for query in [
        point(4 * S, S),
        point(12 * S, 4 * S),
        point(23 * S, 3 * S),
        point(33 * S, 3 * S),
    ] {
        assert_eq!(snapshot.hit_test(query), None);
    }
}
