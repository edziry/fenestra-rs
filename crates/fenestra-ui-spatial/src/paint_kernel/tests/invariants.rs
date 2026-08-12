use super::*;

fn assert_premultiplied(value: SpatialRgba8V2) {
    assert!(value.r() <= value.a());
    assert!(value.g() <= value.a());
    assert!(value.b() <= value.a());
}

#[test]
fn every_normalized_and_opacity_scaled_sample_remains_premultiplied() {
    for alpha in u8::MIN..=u8::MAX {
        for channel in u8::MIN..=u8::MAX {
            let normalized =
                normalize_straight_p1(color(channel, u8::MAX - channel, channel / 2, alpha));
            assert_premultiplied(normalized);

            for opacity in [0, 1, 127, 128, 254, 255] {
                assert_premultiplied(apply_opacity_p1(normalized, opacity));
            }
        }
    }
}

#[test]
fn source_over_retains_premultiplication_across_every_alpha_pair() {
    for source_alpha in u8::MIN..=u8::MAX {
        let source = color(
            source_alpha,
            source_alpha / 2,
            source_alpha.saturating_sub(1),
            source_alpha,
        );
        for destination_alpha in u8::MIN..=u8::MAX {
            let destination = color(
                destination_alpha / 3,
                destination_alpha,
                destination_alpha / 2,
                destination_alpha,
            );
            assert_premultiplied(source_over_p1(source, destination));
        }
    }
}
