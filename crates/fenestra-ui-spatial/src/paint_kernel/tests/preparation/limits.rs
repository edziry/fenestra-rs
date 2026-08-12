use super::*;

use crate::limits::{REGISTERED_SPATIAL_LIMITS_V2, SpatialLimitKindV2};

fn ordered_stops(count: usize) -> Vec<SpatialGradientStopV2> {
    (0..count)
        .map(|index| {
            let offset = (index * usize::from(u16::MAX) / (count - 1)) as u16;
            stop(offset, color(255, 255, 255, 255))
        })
        .collect()
}

#[test]
fn registered_per_brush_limit_accepts_32_and_rejects_33() {
    let maximum = REGISTERED_SPATIAL_LIMITS_V2.limit(SpatialLimitKindV2::GradientStopsPerBrush);
    assert_eq!(maximum, 32);

    let accepted = ordered_stops(maximum);
    let proof = match prepare_gradient_p2(
        BRUSH_INDEX,
        STOP_START,
        maximum as u32,
        point(0, 0),
        point(1, 1),
        &accepted,
        maximum,
    ) {
        Ok(proof) => proof,
        Err(_) => panic!("expected registered P2 limit equality to pass"),
    };
    assert_eq!(proof.stop_count(), maximum);

    let rejected = ordered_stops(maximum + 1);
    expect_p2_limit(
        prepare_gradient_p2(
            BRUSH_INDEX,
            STOP_START,
            (maximum + 1) as u32,
            point(0, 0),
            point(1, 1),
            &rejected,
            maximum,
        ),
        brush_location(PaintP2Field::GradientStopLength),
        maximum + 1,
        maximum,
    );
}

#[test]
fn caller_supplied_maximum_is_not_replaced_by_the_registered_profile() {
    let accepted = ordered_stops(3);
    assert!(
        prepare_gradient_p2(
            BRUSH_INDEX,
            STOP_START,
            3,
            point(0, 0),
            point(1, 1),
            &accepted,
            3,
        )
        .is_ok()
    );

    let rejected = ordered_stops(4);
    expect_p2_limit(
        prepare_gradient_p2(
            BRUSH_INDEX,
            STOP_START,
            4,
            point(0, 0),
            point(1, 1),
            &rejected,
            3,
        ),
        brush_location(PaintP2Field::GradientStopLength),
        4,
        3,
    );

    let two = valid_stops();
    expect_p2_limit(
        prepare_gradient_p2(
            BRUSH_INDEX,
            STOP_START,
            2,
            point(0, 0),
            point(1, 1),
            &two,
            1,
        ),
        brush_location(PaintP2Field::GradientStopLength),
        2,
        1,
    );
}

#[test]
fn per_brush_limit_precedes_scalar_and_gradient_semantics() {
    let stops = vec![stop(1, color(255, 255, 255, 255)); 33];
    let invalid = SpatialScalarV2::MAX_RAW + 1;

    expect_p2_limit(
        prepare_gradient_p2(
            BRUSH_INDEX,
            STOP_START,
            33,
            point(invalid, invalid),
            point(invalid, invalid),
            &stops,
            32,
        ),
        brush_location(PaintP2Field::GradientStopLength),
        33,
        32,
    );
}
