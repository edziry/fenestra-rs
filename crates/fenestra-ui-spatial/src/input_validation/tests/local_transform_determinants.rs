use super::local_transform_support::{
    Placement, VIEWPORT, expect_transform, expect_valid, input, limits, node, root, transform,
    validate,
};
use crate::error::SpatialErrorLocationV2;
use crate::model::SpatialScalarV2;
use crate::numeric_error::SpatialTransformErrorKindV2;

#[test]
fn determinant_widens_beyond_i64_before_testing_zero() {
    let q = 1_i64 << 32;
    let local = transform([q, 0, 0, q, 0, 0, 0, 0]);
    assert_eq!(local.affine().determinant_raw(), 1_i128 << 64);
    let fixture = input(vec![root(), node(Placement::Layout, 1, 0, local)]);

    expect_valid(validate(&fixture, VIEWPORT, limits(1, 2, 2)));
}

#[test]
fn determinant_uses_exact_products_without_float_or_epsilon() {
    let maximum = SpatialScalarV2::MAX_RAW;
    let local = transform([maximum, maximum - 1, maximum - 1, maximum - 2, 0, 0, 0, 0]);
    assert_eq!(local.affine().determinant_raw(), -1);
    let fixture = input(vec![root(), node(Placement::Free, 1, 0, local)]);

    expect_valid(validate(&fixture, VIEWPORT, limits(0, 0, 0)));
}

#[test]
fn exact_zero_determinant_is_singular_at_the_owning_node() {
    let maximum = SpatialScalarV2::MAX_RAW;
    let local = transform([maximum, maximum - 1, maximum, maximum - 1, 0, 0, 0, 0]);
    assert_eq!(local.affine().determinant_raw(), 0);
    let fixture = input(vec![root(), node(Placement::Layout, 1, 0, local)]);

    expect_transform(
        validate(&fixture, VIEWPORT, limits(1, 2, 2)),
        SpatialTransformErrorKindV2::SingularTransform,
        SpatialErrorLocationV2::Node { index: 1 },
    );
}
