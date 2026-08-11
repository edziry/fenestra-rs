use super::*;

use super::fixture::RawInputFixture;

const TABLE_LENGTHS: [usize; DIRECT_COUNT] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];

#[test]
fn aggregate_input_maps_tables_and_preserves_direct_priority() {
    let fixture = RawInputFixture::new(TABLE_LENGTHS);

    for expected_index in 0..DIRECT_COUNT {
        let mut maxima = TABLE_LENGTHS;
        for maximum in &mut maxima[expected_index..] {
            *maximum -= 1;
        }

        expect_limit(
            prepare_direct_counts(fixture.input(), limits_with_direct(maxima)),
            SpatialLimitKindV2::DIRECT_ALL[expected_index],
            TABLE_LENGTHS[expected_index] as u128,
            (TABLE_LENGTHS[expected_index] - 1) as u128,
        );
    }
}

#[test]
fn aggregate_count_pass_does_not_validate_malformed_records() {
    let fixture = RawInputFixture::new(TABLE_LENGTHS);

    expect_valid(prepare_direct_counts(
        fixture.input(),
        limits_with_direct(TABLE_LENGTHS),
    ));
}
