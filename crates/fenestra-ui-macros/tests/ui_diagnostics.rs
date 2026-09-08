#[test]
fn ui_macro_preserves_v1_diagnostics_and_dispatches_format_2() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/unsupported_token.rs");
    cases.compile_fail("tests/ui/unknown_component.rs");
    cases.compile_fail("tests/ui/nesting_depth.rs");
    cases.compile_fail("tests/ui/format_2_dispatch.rs");
}

#[test]
fn format_3_errors_point_to_the_offending_property_name_or_value() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/view_unknown_property.rs");
    cases.compile_fail("tests/ui/view_duplicate_name.rs");
    cases.compile_fail("tests/ui/view_bad_color.rs");
    cases.compile_fail("tests/ui/view_text_diagnostics.rs");
}
