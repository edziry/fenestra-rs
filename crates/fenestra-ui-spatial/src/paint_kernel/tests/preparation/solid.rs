use super::*;

#[test]
fn solid_preparation_is_infallible_and_normalizes_exactly_once() {
    assert_color(prepare_solid_p2(color(17, 89, 203, 137)), [9, 48, 109, 137]);
}
