const ENTRY_POINT: &str = include_str!("../src/lib.rs");

#[test]
fn ui_macro_delegates_once_with_both_registered_limit_profiles() {
    assert!(ENTRY_POINT.contains(
        "use fenestra_ui_authoring::prototype::{\n    REFERENCE_AUTHORING_LIMITS_V1, REFERENCE_AUTHORING_LIMITS_V2, expand_ui,\n};"
    ));
    assert_eq!(ENTRY_POINT.matches("expand_ui(").count(), 1);
    assert!(ENTRY_POINT.contains(
        "expand_ui(\n        input.into(),\n        REFERENCE_AUTHORING_LIMITS_V1,\n        REFERENCE_AUTHORING_LIMITS_V2,\n    )\n    .into()"
    ));
    assert!(!ENTRY_POINT.contains("expand_ui_v1"));
    assert!(!ENTRY_POINT.contains("expand_ui_v2"));
}
