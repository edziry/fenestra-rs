use super::{ObservationLimitCrossingsV1, first_observation_limit_v1};
use crate::error::HarnessLimitKind;

#[test]
fn observation_limit_selection_follows_the_closed_harness_order() {
    let cases = [
        (
            ObservationLimitCrossingsV1::new(true, true, true, true, true),
            Some(HarnessLimitKind::LiveMemberships),
        ),
        (
            ObservationLimitCrossingsV1::new(false, true, true, true, true),
            Some(HarnessLimitKind::PathDepth),
        ),
        (
            ObservationLimitCrossingsV1::new(false, false, true, true, true),
            Some(HarnessLimitKind::NormalizedNodes),
        ),
        (
            ObservationLimitCrossingsV1::new(false, false, false, true, true),
            Some(HarnessLimitKind::NormalizedFragments),
        ),
        (
            ObservationLimitCrossingsV1::new(false, false, false, false, true),
            Some(HarnessLimitKind::NormalizedProperties),
        ),
        (
            ObservationLimitCrossingsV1::new(false, false, false, false, false),
            None,
        ),
    ];

    for (crossings, expected) in cases {
        assert_eq!(first_observation_limit_v1(crossings), expected);
    }
}
