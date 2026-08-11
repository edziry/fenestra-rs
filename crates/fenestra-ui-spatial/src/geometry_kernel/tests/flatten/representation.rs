use super::*;

#[test]
fn lines_and_explicit_close_form_owned_subpath_descriptors() {
    let verbs = [
        move_to(1, 2),
        line_to(3, 4),
        SpatialPathVerbV2::Close,
        move_to(10, 20),
        line_to(30, 40),
    ];
    let flattened = expect_flattened(flatten(&verbs, 7, 8, 16));

    assert_points(
        &flattened,
        &[
            point(1, 2),
            point(3, 4),
            point(1, 2),
            point(10, 20),
            point(30, 40),
        ],
    );
    assert_eq!(flattened.subpaths().len(), 2);
    assert_subpath(&flattened, 0, 0, 3, true);
    assert_subpath(&flattened, 1, 3, 2, false);
    assert_eq!(flattened.segment_count(), 3);
}

#[test]
fn zero_length_line_and_close_segments_are_both_retained() {
    let same = point(4, 5);
    let verbs = [move_to(4, 5), line_to(4, 5), SpatialPathVerbV2::Close];
    let flattened = expect_flattened(flatten(&verbs, 0, 2, 2));

    assert_points(&flattened, &[same, same, same]);
    assert_subpath(&flattened, 0, 0, 3, true);
    assert_eq!(flattened.segment_count(), 2);
}

#[test]
fn open_subpaths_gain_no_stored_implicit_fill_closure() {
    let verbs = [move_to(0, 0), line_to(5, 0)];
    let flattened = expect_flattened(flatten(&verbs, 0, 1, 1));

    assert_points(&flattened, &[point(0, 0), point(5, 0)]);
    assert_subpath(&flattened, 0, 0, 2, false);
    assert_eq!(flattened.segment_count(), 1);
}

#[test]
fn flattened_representation_outlives_raw_verbs_and_has_no_bound_input() {
    let flattened = {
        let verbs = [move_to(0, 0), line_to(2, 3)];
        expect_flattened(flatten(&verbs, 0, 1, 1))
    };

    assert_points(&flattened, &[point(0, 0), point(2, 3)]);
    assert_eq!(flattened.segment_count(), 1);
}
