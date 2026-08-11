use super::*;

#[test]
fn zero_and_full_opacity_are_exact_endpoints() {
    let premultiplied = color(64, 32, 7, 128);

    assert_color(apply_opacity_p1(premultiplied, 0), [0, 0, 0, 0]);
    assert_color(apply_opacity_p1(premultiplied, 255), [64, 32, 7, 128]);
}

#[test]
fn opacity_scales_every_premultiplied_channel_exactly_once() {
    assert_color(
        apply_opacity_p1(color(64, 32, 7, 128), 128),
        [32, 16, 4, 64],
    );
    assert_color(
        apply_opacity_p1(color(17, 89, 203, 211), 73),
        [5, 25, 58, 60],
    );
}

#[test]
fn opacity_rounds_on_both_sides_of_the_integer_threshold() {
    let premultiplied = color(1, 1, 1, 1);

    assert_color(apply_opacity_p1(premultiplied, 127), [0, 0, 0, 0]);
    assert_color(apply_opacity_p1(premultiplied, 128), [1, 1, 1, 1]);
}

#[test]
fn opacity_matches_the_registered_formula_over_every_byte_pair() {
    for factor in u8::MIN..=u8::MAX {
        for channel in u8::MIN..=u8::MAX {
            let premultiplied = color(channel, channel, channel, u8::MAX);
            assert_color(
                apply_opacity_p1(premultiplied, factor),
                [
                    reference_scale(channel, factor),
                    reference_scale(channel, factor),
                    reference_scale(channel, factor),
                    factor,
                ],
            );
        }
    }
}
