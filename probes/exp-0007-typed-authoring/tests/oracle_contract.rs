#[path = "support/layout_board/mod.rs"]
mod support;

use fenestra_ui_authoring::prototype::{AnchorKindV1, AuthoringLimitKindV1};
use fenestra_ui_ir::prototype::{
    StyleValidationLimits, ValidationLimits, validate_construction, validate_schema, validate_style,
};

const IR_LIMITS: ValidationLimits = ValidationLimits::new(1, 5, 4, 1, 3, 12, 2, 3, 5);

#[test]
fn registered_limits_and_anchor_table_are_self_consistent() {
    let expected_limits = [8_192, 1_024, 32, 8, 1, 5, 4, 1, 3, 12, 2, 2, 34, 32_768];
    assert_eq!(AuthoringLimitKindV1::ALL.len(), expected_limits.len());
    for (kind, expected) in AuthoringLimitKindV1::ALL.into_iter().zip(expected_limits) {
        assert_eq!(support::REGISTERED_LIMITS.limit(kind), expected);
    }

    assert_eq!(support::EXPECTED_ANCHORS.len(), 34);
    assert_eq!(support::EXPECTED_LOGICAL_CATALOG, &[b'@'; 34]);
    assert_eq!(support::SOURCE.get(), 7);
    for expected in support::EXPECTED_ANCHORS {
        assert!(AnchorKindV1::ALL.contains(&expected.kind));
        assert_eq!(
            &support::FIXTURE[expected.start as usize..expected.end as usize],
            expected.label.as_bytes()
        );
    }
}

#[test]
fn hand_built_ir_oracle_is_accepted_by_current_validators() {
    let schema = validate_schema(support::expected_schema(), IR_LIMITS)
        .expect("the exact expected schema should validate");
    let construction = validate_construction(&schema, support::expected_construction(), IR_LIMITS)
        .expect("the exact expected construction should validate");
    validate_style(
        &construction,
        support::expected_style(),
        StyleValidationLimits::new(2),
    )
    .expect("the exact expected style should validate");
}
