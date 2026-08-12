use super::*;

#[test]
fn channel_interpolation_rounds_signed_half_ties_away_from_zero() {
    let stops = [
        stop(0, opaque(10, 20, 30)),
        stop(100, opaque(10, 20, 30)),
        stop(102, opaque(11, 19, 31)),
        stop(u16::MAX, opaque(11, 19, 31)),
    ];

    assert_color(sample_at_parameter(&stops, 101), [11, 19, 31, 255]);
}

#[test]
fn channel_interpolation_rounds_positive_and_negative_thirds_to_nearest() {
    let stops = [
        stop(0, opaque(10, 20, 30)),
        stop(100, opaque(10, 20, 30)),
        stop(103, opaque(11, 19, 33)),
        stop(u16::MAX, opaque(11, 19, 33)),
    ];

    assert_color(sample_at_parameter(&stops, 101), [10, 20, 31, 255]);
    assert_color(sample_at_parameter(&stops, 102), [11, 19, 32, 255]);
}

#[test]
fn alpha_interpolation_rounds_half_ties_in_both_directions() {
    let decreasing = [
        stop(0, color(0, 0, 0, 100)),
        stop(100, color(0, 0, 0, 100)),
        stop(102, color(0, 0, 0, 99)),
        stop(u16::MAX, color(0, 0, 0, 99)),
    ];
    let increasing = [
        stop(0, color(0, 0, 0, 99)),
        stop(200, color(0, 0, 0, 99)),
        stop(202, color(0, 0, 0, 100)),
        stop(u16::MAX, color(0, 0, 0, 100)),
    ];

    assert_color(sample_at_parameter(&decreasing, 101), [0, 0, 0, 99]);
    assert_color(sample_at_parameter(&increasing, 201), [0, 0, 0, 100]);
}

#[test]
fn sampling_interpolates_once_normalized_premultiplied_colors() {
    let constant = [
        stop(0, color(128, 64, 32, 128)),
        stop(u16::MAX, color(128, 64, 32, 128)),
    ];
    assert_color(sample_at_parameter(&constant, 27_000), [64, 32, 16, 128]);

    let blend = [
        stop(0, color(255, 0, 0, 64)),
        stop(2, color(0, 255, 0, 192)),
        stop(u16::MAX, color(0, 255, 0, 192)),
    ];
    assert_color(sample_at_parameter(&blend, 1), [32, 96, 0, 128]);
}
