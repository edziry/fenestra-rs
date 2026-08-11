use super::*;

#[test]
fn ratio_rounds_nearest_with_ties_away_from_zero() {
    let cases = [
        (1, 2, 1),
        (-1, 2, -1),
        (1, 3, 0),
        (-1, 3, 0),
        (2, 3, 1),
        (-2, 3, -1),
        (3, 2, 2),
        (-3, 2, -2),
    ];

    for (numerator, denominator, expected) in cases {
        assert_eq!(
            round_ratio_v2(numerator, denominator),
            Some(expected),
            "{numerator}/{denominator}"
        );
    }
}

#[test]
fn ratio_covers_the_complete_signed_numerator_domain() {
    assert_eq!(round_ratio_v2(i128::MIN, 1), Some(i128::MIN));
    assert_eq!(round_ratio_v2(i128::MAX, 1), Some(i128::MAX));
    assert_eq!(round_ratio_v2(i128::MIN, 2), Some(i128::MIN / 2));
    assert_eq!(round_ratio_v2(i128::MAX - 1, i128::MAX), Some(1));
    assert_eq!(round_ratio_v2(i128::MIN, i128::MAX), Some(-1));
}

#[test]
fn ratio_rejects_nonpositive_denominators() {
    assert_eq!(round_ratio_v2(1, 0), None);
    assert_eq!(round_ratio_v2(1, -1), None);
    assert_eq!(round_ratio_v2(i128::MIN, -1), None);
}
