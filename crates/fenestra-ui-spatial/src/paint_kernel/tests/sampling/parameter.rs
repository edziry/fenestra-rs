use super::*;

#[test]
fn projection_uses_the_registered_scale_and_nearest_ties_away_rounding() {
    let cases = [
        (point(10, -20), point(17, -20), point(11, -20), 9_362),
        (point(10, -20), point(10, -16), point(10, -19), 16_384),
        (point(10, -20), point(16, -20), point(11, -20), 10_923),
        (point(10, -20), point(13, -20), point(12, -20), 43_690),
        (point(10, -20), point(13, -16), point(15, -21), 28_835),
    ];

    for (start, end, query, expected) in cases {
        assert_impulse_parameter(start, end, query, expected);
    }
}

#[test]
fn canonical_scalar_edges_preserve_the_exact_side_of_a_half_parameter() {
    let maximum = SpatialScalarV2::MAX_RAW;
    let stops = [
        stop(0, opaque(255, 0, 0)),
        stop(32_768, opaque(255, 0, 0)),
        stop(32_768, opaque(0, 0, 255)),
        stop(u16::MAX, opaque(0, 0, 255)),
    ];
    let proof = match prepare_gradient_p2(
        BRUSH_INDEX,
        STOP_START,
        stops.len() as u32,
        point(maximum, 0),
        point(-maximum, 1),
        &stops,
        stops.len(),
    ) {
        Ok(proof) => proof,
        Err(_) => panic!("canonical edge fixture must prepare"),
    };

    assert_color(sample_gradient_p3(&proof, point(0, 0)), [255, 0, 0, 255]);
    assert_color(sample_gradient_p3(&proof, point(0, 1)), [0, 0, 255, 255]);
}

fn assert_impulse_parameter(
    start: SpatialPointV2,
    end: SpatialPointV2,
    query: SpatialPointV2,
    expected: u16,
) {
    let stops = [
        stop(0, opaque(0, 0, 0)),
        stop(expected - 1, opaque(0, 0, 0)),
        stop(expected, opaque(255, 255, 255)),
        stop(expected + 1, opaque(0, 0, 0)),
        stop(u16::MAX, opaque(0, 0, 0)),
    ];
    assert_color(sample_gradient(&stops, start, end, query), [255; 4]);
}
