#[test]
fn ui_macro_reports_closed_spanned_diagnostics() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/unsupported_token.rs");
    cases.compile_fail("tests/ui/unknown_component.rs");
    cases.compile_fail("tests/ui/nesting_depth.rs");
}
