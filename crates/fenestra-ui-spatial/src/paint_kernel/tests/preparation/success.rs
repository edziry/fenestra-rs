use super::*;

#[test]
fn gradient_preparation_retains_duplicate_order_and_normalizes_every_stop() {
    let starts = [0, 0, 32_768, 32_768, 65_535, 65_535];
    let colors = [
        color(255, 128, 1, 0),
        color(254, 1, 255, 128),
        color(5, 11, 201, 128),
        color(17, 89, 203, 137),
        color(1, 127, 254, 255),
        color(64, 32, 7, 128),
    ];
    let stops: Vec<_> = starts
        .into_iter()
        .zip(colors)
        .map(|(offset, color)| stop(offset, color))
        .collect();
    let snapshot = stops.clone();
    let start = point(SpatialScalarV2::MIN_RAW, SpatialScalarV2::MIN_RAW);
    let end = point(SpatialScalarV2::MAX_RAW, SpatialScalarV2::MAX_RAW);

    let proof = match prepare_gradient_p2(
        BRUSH_INDEX,
        STOP_START,
        stops.len() as u32,
        start,
        end,
        &stops,
        8,
    ) {
        Ok(proof) => proof,
        Err(_) => panic!("expected valid gradient preparation"),
    };

    assert_eq!(proof.start(), start);
    assert_eq!(proof.end(), end);
    assert_eq!(proof.stop_count(), 6);
    let expected = [
        [0, 0, 0, 0],
        [127, 1, 128, 128],
        [3, 6, 101, 128],
        [9, 48, 109, 137],
        [1, 127, 254, 255],
        [32, 16, 4, 128],
    ];
    for (index, expected) in expected.into_iter().enumerate() {
        assert_eq!(proof.stop(index).offset(), starts[index]);
        assert_color(proof.stop(index).color(), expected);
    }
    assert_eq!(stops, snapshot);
}

#[test]
fn one_equal_axis_does_not_make_a_gradient_coincident() {
    let stops = valid_stops();
    for (start, end) in [(point(0, 0), point(1, 0)), (point(0, 0), point(0, 1))] {
        assert!(
            prepare_gradient_p2(
                BRUSH_INDEX,
                STOP_START,
                stops.len() as u32,
                start,
                end,
                &stops,
                8,
            )
            .is_ok()
        );
    }
}
