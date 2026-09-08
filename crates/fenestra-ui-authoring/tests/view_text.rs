use fenestra_ui_authoring::view::{
    Limits, compile_fen, compile_fen_with_limits, compile_ui, compile_ui_with_limits,
};

fn source(content: &str) -> String {
    format!("format 3; view hello {{ text title {{ content: {content}; }} }}")
}

#[test]
fn text_emits_public_facade_and_has_identical_frontends() {
    let source = "format 3; view hello { text title { content: \"Hello\\nworld\"; width: 240; height: 60; font_size: 20; line_height: 28; color: rgba8(235,241,246,255); input: accept; } }";
    let fen = compile_fen(source.as_bytes()).expect("authored text");
    let ui = compile_ui(source.parse().unwrap()).expect("macro text");
    assert_eq!(fen.rust_source(), ui.rust_source());
    let emitted = fen.rust_source();
    for fragment in [
        "Element :: text (\"title\" , \"Hello\\nworld\")",
        "TextStyle :: new ()",
        "font_size (20u32)",
        "line_height (28u32)",
        "color (:: fenestra_ui :: Color :: rgba8 (235u8 , 241u8 , 246u8 , 255u8))",
        "width (240i32)",
    ] {
        assert!(emitted.contains(fragment), "{fragment}: {emitted}");
    }
}

#[test]
fn strings_preserve_unicode_escapes_and_comment_markers() {
    for (literal, expected) in [
        (r#""""#, ""),
        (
            r#""Hello\nworld\t\"quote\"\\""#,
            "Hello\nworld\t\"quote\"\\",
        ),
        (r#""\u{4e16}\u{754c}\x21""#, "\u{4e16}\u{754c}!"),
        (
            "\"caf\u{e9} // /* not comments */\"",
            "caf\u{e9} // /* not comments */",
        ),
        ("\"line\r\nbreak\"", "line\nbreak"),
        ("\"line\\\r\n  continued\"", "linecontinued"),
        (
            r##"r#"raw "quotes" // /* \n"#"##,
            "raw \"quotes\" // /* \\n",
        ),
    ] {
        let source = source(literal);
        let fen = compile_fen(source.as_bytes()).expect(literal);
        let ui = compile_ui(source.parse().unwrap()).expect(literal);
        assert_eq!(fen.rust_source(), ui.rust_source(), "{literal}");
        let canonical = proc_macro2::Literal::string(expected).to_string();
        assert!(fen.rust_source().contains(&canonical), "{literal}");
    }
}

#[test]
fn text_properties_are_required_unique_typed_and_leaf_only() {
    for (body, token, message) in [
        ("text title {}", "}", "requires content"),
        ("text title { content: 12; }", "12", "string literal"),
        (
            "text title { content: \"a\"; content: \"b\"; }",
            "content: \"b\"",
            "duplicate",
        ),
        (
            "text title { content: \"a\"; font_size: 0; }",
            "0",
            "positive",
        ),
        (
            "text title { content: \"a\"; font_size: 513; }",
            "513",
            "512",
        ),
        (
            "text title { content: \"a\"; line_height: 2049; }",
            "2049",
            "2048",
        ),
        (
            "text title { content: \"a\"; line_height: 4294967296; }",
            "4294967296",
            "2048",
        ),
        (
            "text title { content: \"a\"; padding: 0; }",
            "padding",
            "container",
        ),
        ("text title { content: \"a\"; gap: 0; }", "gap", "container"),
        (
            "text title { content: \"a\"; rect child {} }",
            "rect child",
            "children",
        ),
        ("rect title { content: \"a\"; }", "content", "text element"),
        ("row title { font_size: 20; }", "font_size", "text element"),
        (
            "column title { color: rgba8(1,2,3,4); }",
            "color",
            "text element",
        ),
    ] {
        let source = format!("format 3; view hello {{ {body} }}");
        let fen = compile_fen(source.as_bytes()).expect_err(body);
        assert_eq!(
            fen.byte_range().unwrap().0,
            source.find(token).unwrap(),
            "{body}: {fen}"
        );
        assert!(fen.to_string().contains(message), "{body}: {fen}");
        let ui = compile_ui(source.parse().unwrap()).expect_err(body);
        assert_eq!(fen.to_string(), ui.to_string(), "{body}");
        assert!(ui.byte_range().is_none());
    }
}

#[test]
fn text_sizes_accept_inclusive_public_style_bounds() {
    for (font_size, line_height) in [(1, 1), (512, 2048)] {
        let source = format!(
            "format 3; view hello {{ text title {{ content: \"a\"; font_size: {font_size}; line_height: {line_height}; }} }}"
        );
        assert!(compile_fen(source.as_bytes()).is_ok());
        assert!(compile_ui(source.parse().unwrap()).is_ok());
    }
}

#[test]
fn string_values_cannot_stand_in_for_keywords() {
    for body in [
        "\"rect\" title {}",
        "rect title { \"width\": 1; }",
        "rect title { input: \"accept\"; }",
    ] {
        let source = format!("format 3; view hello {{ {body} }}");
        assert!(compile_fen(source.as_bytes()).is_err(), "{body}");
        assert!(compile_ui(source.parse().unwrap()).is_err(), "{body}");
    }
}

#[test]
fn token_limit_on_a_string_reports_the_full_literal() {
    let source = source(r#""\u{4e16} // /* text */""#);
    let limits = Limits::new(source.len(), 1, 2, 11, 4096);
    let error = compile_fen_with_limits(source.as_bytes(), limits).unwrap_err();
    assert_eq!(error.to_string(), "authoring limit exceeded: tokens");
    let start = source.find('"').unwrap();
    let end = source.rfind('"').unwrap() + 1;
    assert_eq!(error.byte_range(), Some((start, end)));
}

#[test]
fn malformed_strings_have_original_byte_ranges() {
    for literal in [
        r#""bad\q""#,
        r#""bad\u{110000}""#,
        r#""bad\x80""#,
        r#""bad\u{}""#,
        "\"unfinished",
    ] {
        let source = source(literal);
        let error = compile_fen(source.as_bytes()).expect_err(literal);
        assert_eq!(error.byte_range().unwrap().0, source.find(literal).unwrap());
        assert!(error.to_string().contains("string"), "{literal}: {error}");
    }
    let source = "// caf\u{e9}\r\nformat 3; view hello { text title { content: \"// /* text */\"; /* spacing */ font_size: 0; } }";
    let error = compile_fen(source.as_bytes()).expect_err("invalid font size");
    assert_eq!(
        error.byte_range(),
        Some((source.find('0').unwrap(), source.find('0').unwrap() + 1))
    );
}

#[test]
fn text_string_is_one_token_and_all_limits_remain_inclusive() {
    let source = source(r#""hello /* \u{4e16} */""#);
    let output = compile_fen(source.as_bytes()).unwrap();
    let generated = output.rust_source().len();
    let exact = Limits::new(source.len(), 1, 2, 15, generated);
    assert!(compile_fen_with_limits(source.as_bytes(), exact).is_ok());
    assert!(compile_ui_with_limits(source.parse().unwrap(), exact).is_ok());
    for limits in [
        Limits::new(source.len(), 1, 2, 14, generated),
        Limits::new(source.len(), 1, 1, 15, generated),
        Limits::new(source.len(), 0, 2, 15, generated),
        Limits::new(source.len(), 1, 2, 15, generated - 1),
    ] {
        assert!(compile_fen_with_limits(source.as_bytes(), limits).is_err());
        assert!(compile_ui_with_limits(source.parse().unwrap(), limits).is_err());
    }
    assert!(
        compile_fen_with_limits(
            source.as_bytes(),
            Limits::new(source.len() - 1, 1, 2, 15, generated)
        )
        .is_err()
    );
}
