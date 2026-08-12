use super::*;

#[test]
fn transparent_source_is_an_exact_no_op() {
    let destination = color(80, 40, 20, 100);

    assert_color(
        source_over_p1(color(0, 0, 0, 0), destination),
        [80, 40, 20, 100],
    );
    assert_color(
        source_over_p1(color(0, 0, 0, 0), color(255, 128, 1, 255)),
        [255, 128, 1, 255],
    );
}

#[test]
fn opaque_source_replaces_the_destination() {
    let source = color(200, 100, 50, 255);

    assert_color(
        source_over_p1(source, color(80, 40, 20, 100)),
        [200, 100, 50, 255],
    );
}

#[test]
fn source_over_composes_every_channel_with_source_alpha() {
    assert_color(
        source_over_p1(color(64, 32, 16, 128), color(80, 40, 20, 100)),
        [104, 52, 26, 178],
    );
    assert_color(
        source_over_p1(color(40, 80, 120, 160), color(20, 40, 60, 200)),
        [47, 95, 142, 235],
    );
}

#[test]
fn source_over_rounds_on_both_sides_of_the_integer_threshold() {
    let destination = color(1, 1, 1, 1);

    assert_color(
        source_over_p1(color(0, 0, 0, 128), destination),
        [0, 0, 0, 128],
    );
    assert_color(
        source_over_p1(color(0, 0, 0, 127), destination),
        [1, 1, 1, 128],
    );
}
