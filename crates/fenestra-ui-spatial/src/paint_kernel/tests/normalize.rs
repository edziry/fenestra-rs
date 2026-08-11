use super::*;

#[test]
fn transparent_and_opaque_straight_colors_normalize_exactly() {
    assert_color(normalize_straight_p1(color(255, 128, 1, 0)), [0, 0, 0, 0]);
    assert_color(
        normalize_straight_p1(color(1, 127, 254, 255)),
        [1, 127, 254, 255],
    );
}

#[test]
fn normalization_scales_each_color_channel_and_preserves_alpha() {
    assert_color(
        normalize_straight_p1(color(5, 11, 201, 128)),
        [3, 6, 101, 128],
    );
    assert_color(
        normalize_straight_p1(color(17, 89, 203, 137)),
        [9, 48, 109, 137],
    );
}

#[test]
fn normalization_rounds_on_both_sides_of_the_integer_threshold() {
    assert_color(normalize_straight_p1(color(1, 1, 1, 127)), [0, 0, 0, 127]);
    assert_color(normalize_straight_p1(color(1, 1, 1, 128)), [1, 1, 1, 128]);
}

#[test]
fn normalization_matches_the_registered_formula_over_every_byte_pair() {
    for alpha in u8::MIN..=u8::MAX {
        for channel in u8::MIN..=u8::MAX {
            let normalized =
                normalize_straight_p1(color(channel, u8::MAX - channel, channel, alpha));
            assert_color(
                normalized,
                [
                    reference_scale(channel, alpha),
                    reference_scale(u8::MAX - channel, alpha),
                    reference_scale(channel, alpha),
                    alpha,
                ],
            );
        }
    }
}
