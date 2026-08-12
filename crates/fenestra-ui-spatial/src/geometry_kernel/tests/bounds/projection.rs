use super::*;

fn assert_fill_and_clip(derived: &DerivedLocalBoundsK3, expected: SpatialAabbV2) {
    assert_eq!(fill_bounds_k3(derived), expected);
    assert_eq!(clip_bounds_k3(derived), expected);
}

#[test]
fn rect_degenerate_axes_keep_closed_base_bounds_but_empty_fill_and_clip() {
    let cases = [
        (0, 5, aabb(2, 3, 2, 8)),
        (5, 0, aabb(2, 3, 7, 3)),
        (0, 0, aabb(2, 3, 2, 3)),
    ];

    for (width, height, expected_base) in cases {
        let derived = expect_derived(derive_rect_bounds_k3(
            SHAPE_INDEX,
            rect(point(2, 3), width, height),
        ));
        assert_eq!(derived.base_bounds(), expected_base);
        assert_fill_and_clip(&derived, SpatialAabbV2::empty());
    }
}

#[test]
fn zero_radius_circle_keeps_a_closed_point_but_empty_fill_and_clip() {
    let derived = expect_derived(derive_circle_bounds_k3(
        SHAPE_INDEX,
        circle(point(4, -5), 0),
    ));

    assert_eq!(derived.base_bounds(), aabb(4, -5, 4, -5));
    assert_fill_and_clip(&derived, SpatialAabbV2::empty());
}

#[test]
fn nondegenerate_rect_and_circle_project_their_base_bounds() {
    let rect = expect_derived(derive_rect_bounds_k3(SHAPE_INDEX, rect(point(1, 2), 3, 4)));
    assert_fill_and_clip(&rect, aabb(1, 2, 4, 6));

    let circle = expect_derived(derive_circle_bounds_k3(SHAPE_INDEX, circle(point(8, 9), 2)));
    assert_fill_and_clip(&circle, aabb(6, 7, 10, 11));
}

#[test]
fn collinear_polygon_and_zero_length_path_bounds_remain_nonempty() {
    let points = [point(0, 0), point(2, 2), point(4, 4)];
    let polygon = derive_polygon_bounds_k3(polygon(&points));
    assert!(!polygon.base_bounds().is_empty());
    assert_fill_and_clip(&polygon, aabb(0, 0, 4, 4));

    let verbs = [move_to(5, 6), line_to(5, 6)];
    let path = derive_path_bounds_k3(path(&verbs));
    assert!(!path.base_bounds().is_empty());
    assert_fill_and_clip(&path, aabb(5, 6, 5, 6));
}
