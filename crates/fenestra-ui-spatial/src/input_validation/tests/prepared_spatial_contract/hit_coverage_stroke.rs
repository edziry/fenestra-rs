use super::hit_support::*;

#[test]
fn k5_dispatches_all_shape_kinds_and_retains_degenerate_segments() {
    use super::super::flattened_path_support::{line_to, move_to, path};
    use super::super::validated_shape_support::{circle_values, path_shape, polygon, rect_values};

    let source = owned_fixture(
        root_and_owners(1, 40, 20),
        vec![
            rect_values(0, 1, 0, 10 * S, 4 * S, 4 * S),
            circle_values(1, 1, 10 * S, 12 * S, 2 * S),
            polygon(2, 1, 0, 3),
            path_shape(3, 1, 0),
        ],
        vec![
            point(20 * S, 10 * S),
            point(28 * S, 10 * S),
            point(20 * S, 18 * S),
        ],
        vec![path(0, 0, 3)],
        vec![
            move_to(30 * S, 10 * S),
            line_to(34 * S, 10 * S),
            line_to(34 * S, 14 * S),
        ],
        Vec::new(),
        vec![
            accepting_stroke(1, 0, 0, 2 * S, None),
            accepting_stroke(1, 1, 1, 4 * S, None),
            accepting_stroke(1, 2, 2, 2 * S, None),
            accepting_stroke(1, 3, 3, 2 * S, None),
        ],
    );
    let snapshot = reference_snapshot(source);

    for (query, key) in [
        (point(2 * S, 10 * S), 0),
        (point(10 * S, 12 * S), 1),
        (point(24 * S, 10 * S), 2),
        (point(34 * S, 14 * S), 3),
    ] {
        assert_hit(snapshot.hit_test(query), key, 1, key, query);
    }
    assert_hit(
        snapshot.hit_test(point(8 * S, 12 * S)),
        1,
        1,
        1,
        point(8 * S, 12 * S),
    );
    assert_hit(
        snapshot.hit_test(point(20 * S, 14 * S)),
        2,
        1,
        2,
        point(20 * S, 14 * S),
    );
    assert_eq!(snapshot.hit_test(point(32 * S, 12 * S)), None);
}

#[test]
fn k5_rejects_interior_and_expanded_aabb_false_positives() {
    use super::super::flattened_path_support::{line_to, move_to, path};
    use super::super::validated_shape_support::{circle_values, path_shape, rect_values};

    let source = owned_fixture(
        root_and_owners(1, 40, 20),
        vec![
            rect_values(0, 1, 0, 10 * S, 6 * S, 6 * S),
            circle_values(1, 1, 12 * S, 13 * S, 2 * S),
            path_shape(2, 1, 0),
        ],
        Vec::new(),
        vec![path(0, 0, 2)],
        vec![move_to(30 * S, 10 * S), line_to(30 * S, 10 * S)],
        Vec::new(),
        vec![
            accepting_stroke(1, 0, 0, 2 * S, None),
            accepting_stroke(1, 1, 1, 2 * S, None),
            accepting_stroke(1, 2, 2, 2 * S, None),
        ],
    );
    let snapshot = reference_snapshot(source);

    for query in [
        point(-S, 9 * S),
        point(3 * S, 13 * S),
        point(12 * S, 13 * S),
        point(31 * S, 11 * S),
    ] {
        assert_eq!(snapshot.hit_test(query), None);
    }
}

#[test]
fn k5_keeps_zero_length_path_and_zero_extent_rect_as_radius_disks() {
    use super::super::flattened_path_support::{line_to, move_to, path};
    use super::super::validated_shape_support::{path_shape, rect_values};

    let source = owned_fixture(
        root_and_owners(1, 20, 10),
        vec![rect_values(0, 1, 2 * S, 2 * S, 0, 0), path_shape(1, 1, 0)],
        Vec::new(),
        vec![path(0, 0, 2)],
        vec![move_to(10 * S, 2 * S), line_to(10 * S, 2 * S)],
        Vec::new(),
        vec![
            accepting_stroke(1, 0, 0, 2 * S, None),
            accepting_stroke(1, 1, 1, 2 * S, None),
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
    assert_hit(
        snapshot.hit_test(point(10 * S, 2 * S)),
        1,
        1,
        1,
        point(10 * S, 2 * S),
    );
}
