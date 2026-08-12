#[test]
fn ui_macro_preserves_v1_diagnostics_and_dispatches_format_2() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/unsupported_token.rs");
    cases.compile_fail("tests/ui/unknown_component.rs");
    cases.compile_fail("tests/ui/nesting_depth.rs");
    cases.compile_fail("tests/ui/format_2_dispatch.rs");
}
