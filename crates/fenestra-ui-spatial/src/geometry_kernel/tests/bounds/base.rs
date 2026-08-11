use super::*;

#[test]
fn rect_and_circle_base_bounds_use_analytic_closed_extrema() {
    let rect = rect(point(-10, 20), 30, 40);
    let derived = expect_derived(derive_rect_bounds_k3(SHAPE_INDEX, rect));
    assert_eq!(derived.base_bounds(), aabb(-10, 20, 20, 60));

    let circle = circle(point(10, -20), 7);
    let derived = expect_derived(derive_circle_bounds_k3(SHAPE_INDEX, circle));
    assert_eq!(derived.base_bounds(), aabb(3, -27, 17, -13));
}

#[test]
fn polygon_base_bounds_include_every_authored_point() {
    let points = [point(-5, 2), point(7, -9), point(3, 11), point(0, 0)];
    let derived = derive_polygon_bounds_k3(polygon(&points));

    assert_eq!(derived.base_bounds(), aabb(-5, -9, 7, 11));
}

#[test]
fn path_base_bounds_use_raw_moves_endpoints_and_controls_not_flattened_points() {
    let verbs = [
        move_to(0, 0),
        SpatialPathVerbV2::QuadraticTo {
            control: point(-900, 700),
            to: point(10, 10),
        },
        SpatialPathVerbV2::CubicTo {
            control1: point(800, -600),
            control2: point(-500, 900),
            to: point(20, -30),
        },
        move_to(1_000, -1_000),
        line_to(4, 5),
    ];
    let derived = derive_path_bounds_k3(path(&verbs));

    assert_eq!(derived.base_bounds(), aabb(-900, -1_000, 1_000, 900));
}

#[test]
fn quadratic_control_and_endpoint_each_determine_raw_path_extrema() {
    let verbs = [
        move_to(0, 0),
        SpatialPathVerbV2::QuadraticTo {
            control: point(-10, 20),
            to: point(30, -40),
        },
    ];
    let derived = derive_path_bounds_k3(path(&verbs));

    assert_eq!(derived.base_bounds(), aabb(-10, -40, 30, 20));
}

#[test]
fn cubic_controls_and_endpoint_each_determine_raw_path_extrema() {
    let verbs = [
        move_to(0, 0),
        SpatialPathVerbV2::CubicTo {
            control1: point(-50, 60),
            control2: point(70, -80),
            to: point(90, 100),
        },
    ];
    let derived = derive_path_bounds_k3(path(&verbs));

    assert_eq!(derived.base_bounds(), aabb(-50, -80, 90, 100));
}

#[test]
fn close_adds_no_new_base_extrema() {
    let verbs = [move_to(-4, 8), line_to(12, -3), SpatialPathVerbV2::Close];
    let derived = derive_path_bounds_k3(path(&verbs));

    assert_eq!(derived.base_bounds(), aabb(-4, -3, 12, 8));
}
