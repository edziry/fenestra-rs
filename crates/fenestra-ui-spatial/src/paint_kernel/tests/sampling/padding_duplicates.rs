use super::*;

#[test]
fn padding_and_endpoint_duplicates_select_the_last_equal_stop() {
    let stops = [
        stop(0, opaque(255, 0, 0)),
        stop(0, opaque(0, 255, 0)),
        stop(u16::MAX, opaque(0, 0, 255)),
        stop(u16::MAX, opaque(255, 255, 0)),
    ];
    let proof = match prepare_gradient_p2(
        BRUSH_INDEX,
        STOP_START,
        stops.len() as u32,
        point(10, 20),
        point(14, 20),
        &stops,
        stops.len(),
    ) {
        Ok(proof) => proof,
        Err(_) => panic!("endpoint duplicate fixture must prepare"),
    };

    for query_x in [9, 10] {
        assert_color(
            sample_gradient_p3(&proof, point(query_x, 20)),
            [0, 255, 0, 255],
        );
    }
    for query_x in [14, 15] {
        assert_color(
            sample_gradient_p3(&proof, point(query_x, 20)),
            [255, 255, 0, 255],
        );
    }
}

#[test]
fn an_interior_duplicate_uses_first_equal_as_upper_and_last_equal_as_lower() {
    let stops = [
        stop(0, opaque(0, 0, 0)),
        stop(20_000, opaque(200, 0, 0)),
        stop(20_000, opaque(0, 200, 0)),
        stop(u16::MAX, opaque(0, 0, 200)),
    ];

    assert_color(sample_at_parameter(&stops, 19_999), [200, 0, 0, 255]);
    assert_color(sample_at_parameter(&stops, 20_000), [0, 200, 0, 255]);
}
