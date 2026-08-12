use super::prepared_brush_support::{
    expect_scalar, fixture, gradient_values, limits, point, stop, valid_stops, validate,
};
use crate::geometry_field::SpatialBrushFieldV2;
use crate::model::SpatialScalarV2;

#[test]
fn every_gradient_scalar_rejects_both_domain_sides_in_field_order() {
    for outside in [SpatialScalarV2::MIN_RAW - 1, SpatialScalarV2::MAX_RAW + 1] {
        let cases = [
            (
                point(outside, outside),
                point(outside, outside),
                SpatialBrushFieldV2::GradientStartX,
            ),
            (
                point(0, outside),
                point(outside, outside),
                SpatialBrushFieldV2::GradientStartY,
            ),
            (
                point(0, 1),
                point(outside, outside),
                SpatialBrushFieldV2::GradientEndX,
            ),
            (
                point(0, 1),
                point(2, outside),
                SpatialBrushFieldV2::GradientEndY,
            ),
        ];

        for (start, end, field) in cases {
            let fixture = fixture(vec![gradient_values(0, 0, 2, start, end)], valid_stops());
            expect_scalar(validate(&fixture, limits(2)), 0, field);
        }
    }
}

#[test]
fn scalar_validation_completes_before_coincidence_and_stop_semantics() {
    let outside = SpatialScalarV2::MAX_RAW + 1;
    let fixture = fixture(
        vec![gradient_values(
            0,
            0,
            2,
            point(outside, outside),
            point(outside, outside),
        )],
        vec![stop(1), stop(u16::MAX - 1)],
    );

    expect_scalar(
        validate(&fixture, limits(2)),
        0,
        SpatialBrushFieldV2::GradientStartX,
    );
}
