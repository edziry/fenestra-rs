use super::*;

#[test]
fn canonical_affine_constructors_use_y_down_coordinates() {
    assert_eq!(Affine2V2::identity(), integer_affine([1, 0, 0, 1, 0, 0]));
    assert_eq!(
        Affine2V2::translation(integer(2), integer(3)),
        integer_affine([1, 0, 0, 1, 2, 3])
    );
    assert_eq!(
        Affine2V2::scale(integer(2), integer(3)),
        integer_affine([2, 0, 0, 3, 0, 0])
    );
    assert_eq!(
        Affine2V2::quarter_turn_clockwise(),
        integer_affine([0, 1, -1, 0, 0, 0])
    );
    assert_eq!(
        Affine2V2::quarter_turn_clockwise()
            .checked_apply_point(SpatialPointV2::new(integer(2), integer(3))),
        Ok(SpatialPointV2::new(integer(-3), integer(2)))
    );
}

#[test]
fn compose_uses_the_literal_right_first_formula() {
    let left = integer_affine([2, 3, 5, 7, 11, 13]);
    let right = integer_affine([17, 19, 23, 29, 31, 37]);

    assert_eq!(
        left.checked_compose(right),
        Ok(integer_affine([129, 184, 191, 272, 258, 365]))
    );

    let translation = Affine2V2::translation(integer(5), integer(7));
    let scale = Affine2V2::scale(integer(2), integer(3));
    assert_ne!(
        translation.checked_compose(scale),
        scale.checked_compose(translation)
    );
}

#[test]
fn affine_components_sum_before_their_single_rounding_step() {
    let left = affine([1, 0, 1, 0, 0, 0]);
    let right = affine([SCALE / 2, SCALE / 2, 0, 0, 0, 0]);
    let composed = left.checked_compose(right).expect("sum is canonical");
    assert_eq!(composed.a(), scalar(1));

    let applied = left
        .checked_apply_point(point(SCALE / 2, SCALE / 2))
        .expect("sum is canonical");
    assert_eq!(applied.x(), scalar(1));

    let extreme = i64::MAX;
    let cancelled = affine([extreme, 0, extreme, 0, 0, 0])
        .checked_compose(affine([extreme, -extreme, 0, 0, 0, 0]))
        .expect("exact widened products cancel");
    assert_eq!(cancelled.a(), scalar(0));
}

#[test]
fn point_application_uses_the_literal_complete_component_formula() {
    let transform = integer_affine([2, 3, 5, 7, 11, 13]);
    assert_eq!(
        transform.checked_apply_point(SpatialPointV2::new(integer(17), integer(19))),
        Ok(SpatialPointV2::new(integer(140), integer(197)))
    );

    let extreme = i64::MAX;
    let cancelled = affine([extreme, 0, extreme, 0, 0, 0])
        .checked_apply_point(point(extreme, -extreme))
        .expect("exact widened products cancel");
    assert_eq!(cancelled.x(), scalar(0));
}

#[test]
fn fixed_composition_is_observably_non_associative() {
    let first = Affine2V2::scale(scalar(1), scalar(1));
    let second = Affine2V2::scale(scalar(32_767), scalar(32_767));
    let third = Affine2V2::scale(scalar(98_304), scalar(98_304));

    let left_grouped = first
        .checked_compose(second)
        .and_then(|value| value.checked_compose(third))
        .expect("left grouping is canonical");
    let right_grouped = second
        .checked_compose(third)
        .and_then(|value| first.checked_compose(value))
        .expect("right grouping is canonical");

    assert_eq!(left_grouped.a(), scalar(0));
    assert_eq!(right_grouped.a(), scalar(1));
}

#[test]
fn compose_reports_components_in_contract_order() {
    let maximum = SpatialScalarV2::MAX_RAW;
    let over_one = SCALE + 1;
    let cases = [
        (
            affine([maximum, 0, 0, 0, 0, 0]),
            affine([over_one, 0, 0, 0, 0, 0]),
            SpatialAffineComponentV2::A,
        ),
        (
            affine([0, maximum, 0, 0, 0, 0]),
            affine([over_one, 0, 0, 0, 0, 0]),
            SpatialAffineComponentV2::B,
        ),
        (
            affine([maximum, 0, 0, 0, 0, 0]),
            affine([0, 0, over_one, 0, 0, 0]),
            SpatialAffineComponentV2::C,
        ),
        (
            affine([0, maximum, 0, 0, 0, 0]),
            affine([0, 0, over_one, 0, 0, 0]),
            SpatialAffineComponentV2::D,
        ),
        (
            affine([maximum, 0, 0, 0, 0, 0]),
            affine([0, 0, 0, 0, over_one, 0]),
            SpatialAffineComponentV2::Tx,
        ),
        (
            affine([0, maximum, 0, 0, 0, 0]),
            affine([0, 0, 0, 0, over_one, 0]),
            SpatialAffineComponentV2::Ty,
        ),
    ];

    for (left, right, expected) in cases {
        assert_eq!(left.checked_compose(right), Err(expected));
    }

    let both = affine([maximum, maximum, 0, 0, 0, 0]);
    assert_eq!(
        both.checked_compose(affine([over_one, 0, 0, 0, 0, 0])),
        Err(SpatialAffineComponentV2::A)
    );

    assert_eq!(
        affine([i64::MAX, 0, i64::MAX, 0, i64::MAX, 0]).checked_compose(affine([
            0,
            0,
            0,
            0,
            i64::MAX,
            i64::MAX
        ])),
        Err(SpatialAffineComponentV2::Tx)
    );
    assert_eq!(
        affine([i64::MIN, 0, i64::MIN, 0, 0, 0]).checked_compose(affine([
            i64::MIN,
            i64::MIN,
            0,
            0,
            0,
            0
        ])),
        Err(SpatialAffineComponentV2::A)
    );
}

#[test]
fn point_application_reports_x_before_y() {
    let maximum = SpatialScalarV2::MAX_RAW;
    let over_one = SCALE + 1;
    let input = SpatialPointV2::new(scalar(over_one), scalar(0));

    assert_eq!(
        affine([maximum, maximum, 0, 0, 0, 0]).checked_apply_point(input),
        Err(SpatialAxisV2::X)
    );
    assert_eq!(
        affine([0, maximum, 0, 0, 0, 0]).checked_apply_point(input),
        Err(SpatialAxisV2::Y)
    );
    assert_eq!(
        affine([i64::MAX, 0, i64::MAX, 0, i64::MAX, 0,])
            .checked_apply_point(point(i64::MAX, i64::MAX)),
        Err(SpatialAxisV2::X)
    );
    assert_eq!(
        affine([i64::MIN, i64::MIN, i64::MIN, i64::MIN, 0, 0])
            .checked_apply_point(point(i64::MIN, i64::MIN)),
        Err(SpatialAxisV2::X)
    );
    assert_eq!(
        affine([0, i64::MIN, 0, i64::MIN, 0, 0]).checked_apply_point(point(i64::MIN, i64::MIN)),
        Err(SpatialAxisV2::Y)
    );
}

#[test]
fn determinant_is_exact_and_has_no_epsilon() {
    assert_eq!(Affine2V2::identity().determinant_raw(), 4_294_967_296);
    assert_eq!(
        Affine2V2::scale(integer(-1), integer(1)).determinant_raw(),
        -4_294_967_296
    );
    assert_eq!(affine([1, 0, 0, 1, 0, 0]).determinant_raw(), 1);
    assert_eq!(affine([1, 2, 2, 4, 0, 0]).determinant_raw(), 0);
    assert_eq!(
        affine([i64::MAX, i64::MAX, i64::MIN, i64::MAX, 0, 0]).determinant_raw(),
        170_141_183_460_469_231_704_017_187_605_319_778_305
    );
}

#[test]
fn rounded_composition_may_become_singular() {
    let tiny = Affine2V2::scale(scalar(1), scalar(1));
    assert_eq!(tiny.determinant_raw(), 1);
    let composed = tiny.checked_compose(tiny).expect("zero remains canonical");
    assert_eq!(composed.determinant_raw(), 0);
}

#[test]
fn inverse_point_uses_the_forward_matrix_and_negative_determinants() {
    let transform = Affine2V2::translation(integer(7), integer(-11))
        .checked_compose(Affine2V2::scale(integer(-2), integer(3)))
        .expect("transform is canonical");
    let local = SpatialPointV2::new(integer(4), integer(5));
    let scene = transform
        .checked_apply_point(local)
        .expect("point is canonical");
    assert_eq!(transform.inverse_point(scene), Some(local));

    let double_x = Affine2V2::scale(integer(2), integer(1));
    assert_eq!(
        double_x.inverse_point(point(1, 0)).map(SpatialPointV2::x),
        Some(scalar(1))
    );
    assert_eq!(
        double_x.inverse_point(point(-1, 0)).map(SpatialPointV2::x),
        Some(scalar(-1))
    );

    let determinant_one = affine([1, 0, 0, 1, 0, 0]);
    assert_eq!(
        determinant_one.inverse_point(point(1, 1)),
        Some(point(SCALE, SCALE))
    );
}

#[test]
fn inverse_rejects_exact_rational_results_before_rounding() {
    let maximum = SpatialScalarV2::MAX_RAW;
    let transform = affine([3 * SCALE, 0, -2 * SCALE, SCALE, -maximum, -maximum]);

    assert_eq!(transform.inverse_point(point(1, 0)), None);
    assert_eq!(
        transform.inverse_point(point(-1, 0)),
        Some(point(maximum, maximum))
    );

    let vertical = affine([SCALE, -2 * SCALE, 0, 3 * SCALE, -maximum, -maximum]);
    assert_eq!(vertical.inverse_point(point(0, 1)), None);
    assert_eq!(
        vertical.inverse_point(point(0, -1)),
        Some(point(maximum, maximum))
    );
}

#[test]
fn inverse_returns_none_for_every_noncoverage_boundary() {
    let maximum = SpatialScalarV2::MAX_RAW;
    assert_eq!(
        Affine2V2::scale(scalar(SCALE / 2), integer(1)).inverse_point(point(maximum, 0)),
        None
    );
    assert_eq!(affine([1, 2, 2, 4, 0, 0]).inverse_point(point(0, 0)), None);
    assert_eq!(
        affine([i64::MAX, 0, 0, SCALE, 0, 0]).inverse_point(point(0, 0)),
        None
    );
    assert_eq!(
        Affine2V2::identity().inverse_point(point(i64::MAX, 0)),
        None
    );
}
