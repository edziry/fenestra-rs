use fenestra_ui_authoring::view::{
    Limits, compile_fen, compile_fen_with_limits, compile_ui, compile_ui_with_limits,
};

fn source(properties: &str) -> String {
    format!("format 3; view responsive {{ rect panel {{ {properties} }} }}")
}

#[test]
fn responsive_dimensions_emit_public_modes_and_bounds_in_both_frontends() {
    let source = "format 3; view responsive { column root { width: fill; height: auto; min_width: 120; max_width: 600; min_height: 0; max_height: 2147483647; text title { content: \"Hello\\nworld\"; width: fill(2); height: auto; } } }";
    let fen = compile_fen(source.as_bytes()).expect("responsive dimensions");
    let ui = compile_ui(source.parse().unwrap()).expect("macro dimensions");
    assert_eq!(fen.rust_source(), ui.rust_source());
    for expected in [
        "width_mode (:: fenestra_ui :: Dimension :: Fill (1u32))",
        "width_mode (:: fenestra_ui :: Dimension :: Fill (2u32))",
        "height_mode (:: fenestra_ui :: Dimension :: Auto)",
        "min_width (120i32)",
        "max_width (600i32)",
        "min_height (0i32)",
        "max_height (2147483647i32)",
    ] {
        assert!(fen.rust_source().contains(expected), "{expected}");
    }
}

#[test]
fn bare_fill_and_weight_one_have_the_same_canonical_expression() {
    let bare = source("width: fill; height: fill;");
    let explicit = source("width: fill(1); height: fill(1);");
    let bare = compile_fen(bare.as_bytes()).unwrap();
    let explicit = compile_fen(explicit.as_bytes()).unwrap();
    assert_eq!(bare.rust_source(), explicit.rust_source());
    let reversed = source("max_width: 600; min_width: 120; height: auto; width: fill(65535);");
    let ordered = source("width: fill(65535); height: auto; min_width: 120; max_width: 600;");
    assert_eq!(
        compile_fen(reversed.as_bytes()).unwrap().rust_source(),
        compile_ui(ordered.parse().unwrap()).unwrap().rust_source(),
    );
}

#[test]
fn fixed_dimensions_keep_the_previous_exact_canonical_expression() {
    let source = "format 3; view fixed { row root { width: 240; height: 120; padding: 12; gap: 8; background: rgba8(1,2,3,4); input: accept; rect item { width: 80; height: 40; } } }";
    let expected = concat!(
        ":: fenestra_ui :: View :: new (\"fixed\" ,:: fenestra_ui :: Element :: row (\"root\")",
        " . style (:: fenestra_ui :: Style :: new () . width (240i32) . height (120i32)",
        " . padding (12i32) . gap (8i32) . background (:: fenestra_ui :: Color :: rgba8",
        " (1u8 , 2u8 , 3u8 , 4u8)) . input (true)) . child (:: fenestra_ui :: Element :: rect",
        " (\"item\") . style (:: fenestra_ui :: Style :: new () . width (80i32) . height (40i32))))\n",
    );
    assert_eq!(
        compile_fen(source.as_bytes()).unwrap().rust_source(),
        expected
    );
    assert_eq!(
        compile_ui(source.parse().unwrap()).unwrap().rust_source(),
        expected
    );
}

#[test]
fn invalid_weights_and_bounds_report_the_original_offending_token() {
    for (properties, offending, message) in [
        ("width: fill(0);", "0", "1 through 65535"),
        ("height: fill(65536);", "65536", "1 through 65535"),
        ("width: fill(-1);", "-", "1 through 65535"),
        ("height: fill();", ")", "1 through 65535"),
        ("width: fill(auto);", "auto", "1 through 65535"),
        ("width: fill(2,3);", ",", "expected ')'"),
        ("height: auto(2);", "(", "expected ';'"),
        ("min_width: -1;", "-", "nonnegative"),
        ("max_height: 2147483648;", "2147483648", "2147483647"),
        ("min_width: auto;", "auto", "nonnegative"),
        ("max_width: fill;", "fill", "nonnegative"),
        (
            "width: auto; width: fill;",
            "width: fill",
            "duplicate property",
        ),
        (
            "min_width: 1; min_width: 2;",
            "min_width: 2",
            "duplicate property",
        ),
        (
            "min_width: 200; max_width: 100;",
            "max_width",
            "minimum width exceeds maximum width",
        ),
        (
            "max_width: 100; min_width: 200;",
            "min_width",
            "minimum width exceeds maximum width",
        ),
        (
            "max_height: 4; min_height: 5;",
            "min_height",
            "minimum height exceeds maximum height",
        ),
        (
            "min_height: 5; max_height: 4;",
            "max_height",
            "minimum height exceeds maximum height",
        ),
    ] {
        let source = source(properties);
        let fen = compile_fen(source.as_bytes()).expect_err(properties);
        assert_eq!(
            fen.byte_range().unwrap().0,
            source.find(offending).unwrap(),
            "{properties}: {fen}"
        );
        assert!(fen.to_string().contains(message), "{properties}: {fen}");
        let ui = compile_ui(source.parse().unwrap()).expect_err(properties);
        assert_eq!(fen.to_string(), ui.to_string(), "{properties}");
        assert!(ui.byte_range().is_none());
    }
}

#[test]
fn equal_bounds_and_fixed_sizes_outside_bounds_are_preserved_for_the_facade() {
    for properties in [
        "width: 80; min_width: 120; max_width: 120;",
        "height: 80; min_height: 0; max_height: 0;",
        "width: 80; max_width: 20;",
        "min_height: 2147483647; max_height: 2147483647;",
    ] {
        let source = source(properties);
        let fen = compile_fen(source.as_bytes()).expect(properties);
        let ui = compile_ui(source.parse().unwrap()).expect(properties);
        assert_eq!(fen.rust_source(), ui.rust_source());
    }
}

#[test]
fn comments_and_unicode_text_preserve_weight_error_offsets() {
    let source = "// caf\u{e9}\r\nformat 3; view demo { text title { content: \"// text \\u{4e16}\"; width: fill(/* weight */ 65536); } }";
    let error = compile_fen(source.as_bytes()).unwrap_err();
    let start = source.find("65536").unwrap();
    assert_eq!(error.byte_range(), Some((start, start + 5)));
    assert!(error.to_string().contains("1 through 65535"));
    let valid = source.replace("65536", "2");
    assert_eq!(
        compile_fen(valid.as_bytes()).unwrap().rust_source(),
        compile_ui(valid.parse().unwrap()).unwrap().rust_source(),
    );
}

#[test]
fn responsive_dimensions_obey_inclusive_frontend_and_generated_limits() {
    let source = source("width: fill(2); height: auto;");
    let output = compile_fen(source.as_bytes()).unwrap();
    let generated = output.rust_source().len();
    let exact = Limits::new(source.len(), 1, 3, 22, generated);
    assert!(compile_fen_with_limits(source.as_bytes(), exact).is_ok());
    assert!(compile_ui_with_limits(source.parse().unwrap(), exact).is_ok());
    for limits in [
        Limits::new(source.len(), 1, 2, 22, generated),
        Limits::new(source.len(), 1, 3, 21, generated),
        Limits::new(source.len(), 1, 3, 22, generated - 1),
    ] {
        assert!(compile_fen_with_limits(source.as_bytes(), limits).is_err());
        assert!(compile_ui_with_limits(source.parse().unwrap(), limits).is_err());
    }
}
