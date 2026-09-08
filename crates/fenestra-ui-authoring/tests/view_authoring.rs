use fenestra_ui_authoring::view::{Limits, compile_fen, compile_fen_with_limits, compile_ui};

const SOURCE: &str = "format 3; view hello { column root { width: 240; height: 160; padding: 12; gap: 8; background: rgba8(24,32,48,255); input: accept; row cards { rect card { width: 50; height: 40; } } } }";

#[test]
fn nested_view_emits_the_public_facade_with_identical_frontends() {
    let fen = compile_fen(SOURCE.as_bytes()).expect("valid format-3 view");
    let ui =
        compile_ui(SOURCE.parse().expect("Rust token spelling")).expect("equivalent macro view");
    assert_eq!(fen.rust_source(), ui.rust_source());
    assert_eq!(fen.tokens().to_string(), ui.tokens().to_string());
    assert!(fen.rust_source().contains(":: fenestra_ui :: View :: new"));
    assert!(
        fen.rust_source()
            .contains(":: fenestra_ui :: Element :: column")
    );
    assert!(
        fen.rust_source()
            .contains(":: fenestra_ui :: Element :: row")
    );
    assert!(
        fen.rust_source()
            .contains(":: fenestra_ui :: Element :: rect")
    );
    assert!(!fen.rust_source().contains("prototype"));
    assert!(!fen.rust_source().contains("fenestra_ui_authoring"));
}

#[test]
fn failures_report_the_authored_token_and_an_actionable_message() {
    for (source, token, message) in [
        (
            "format 3; view hello { text label {} }",
            "text",
            "unknown element",
        ),
        (
            "format 3; view hello { rect root { widht: 12; } }",
            "widht",
            "unknown property",
        ),
        (
            "format 3; view hello { rect root { width: -1; } }",
            "-",
            "nonnegative",
        ),
        (
            "format 3; view hello { rect root { width: 2147483648; } }",
            "2147483648",
            "2147483647",
        ),
        (
            "format 3; view hello { rect root { background: rgba8(256,0,0,255); } }",
            "256",
            "255",
        ),
        (
            "format 3; view hello { rect root { width: 1; width: 2; } }",
            "width: 2",
            "duplicate property",
        ),
        (
            "format 3; view hello { column root { rect item {} rect item {} } }",
            "item {} }",
            "duplicate element name",
        ),
        (
            "format 3; view hello { rect root { padding: 1; } }",
            "padding",
            "container",
        ),
        (
            "format 3; view hello { rect root { rect child {} } }",
            "rect child",
            "children",
        ),
        (
            "format 3; view hello { rect root { input: true; } }",
            "true",
            "accept or ignore",
        ),
    ] {
        let failure = compile_fen(source.as_bytes()).expect_err(source);
        let (start, end) = failure.byte_range().expect("file range");
        let expected = source.find(token).expect("bad token present");
        assert_eq!(start, expected, "{source}: {failure}");
        assert!(end > start, "nonempty offending token: {source}");
        assert!(failure.to_string().contains(message), "{source}: {failure}");
    }
}

#[test]
fn limits_are_inclusive_and_apply_before_utf8_validation() {
    let source = b"format 3; view hello { rect root {} }";
    let compiled = compile_fen(source).expect("fixture");
    let exact = Limits::new(source.len(), 1, 2, 11, compiled.rust_source().len());
    assert!(compile_fen_with_limits(source, exact).is_ok());
    for limits in [
        Limits::new(source.len() - 1, 1, 2, 11, compiled.rust_source().len()),
        Limits::new(source.len(), 0, 2, 11, compiled.rust_source().len()),
        Limits::new(source.len(), 1, 1, 11, compiled.rust_source().len()),
        Limits::new(source.len(), 1, 2, 10, compiled.rust_source().len()),
        Limits::new(source.len(), 1, 2, 11, compiled.rust_source().len() - 1),
    ] {
        assert!(compile_fen_with_limits(source, limits).is_err());
    }
    let error =
        compile_fen_with_limits(&[0xff], Limits::new(0, 1, 1, 1, 1)).expect_err("byte limit first");
    assert!(error.to_string().contains("source bytes"));
}

#[test]
fn comments_preserve_frontend_parity_and_physical_error_ranges() {
    let plain = "format 3; view hello { rect root { width: 12; } }";
    let comments = "// a view\nformat /* outer /* nested */ comment */ 3;\nview hello { rect root { width: /* pixels */ 12; } }";
    let expected = compile_fen(plain.as_bytes()).expect("plain view");
    let commented = compile_fen(comments.as_bytes()).expect("commented view");
    let ui = compile_ui(comments.parse().expect("Rust comments")).expect("commented macro");
    assert_eq!(expected.rust_source(), commented.rust_source());
    assert_eq!(expected.rust_source(), ui.rust_source());
    let invalid = "// a view\nformat 3; view hello { rect root { width: /* size */ -1; } }";
    let error = compile_fen(invalid.as_bytes()).expect_err("negative dimension");
    assert_eq!(error.byte_range().unwrap().0, invalid.find('-').unwrap());
    let open = "format 3; /* unfinished";
    let error = compile_fen(open.as_bytes()).expect_err("unterminated comment");
    assert_eq!(error.byte_range(), Some((10, 12)));
    assert!(error.to_string().contains("unterminated block comment"));
}

#[test]
fn ui_compilation_has_no_file_source_byte_budget() {
    use fenestra_ui_authoring::view::compile_ui_with_limits;
    let source = "format 3; view hello { rect root {} }";
    let limits = Limits::new(0, 1, 2, 11, 1024);
    assert!(compile_ui_with_limits(source.parse().unwrap(), limits).is_ok());
}

#[test]
fn unicode_comments_and_crlf_keep_original_byte_offsets() {
    let source = "// caf\u{e9}\r\nformat 3; /*\r\n\u{4e16}\u{754c} /* nested */\r\n*/ view hello { rect root { width: -1; } }";
    let error = compile_fen(source.as_bytes()).expect_err("negative value");
    let start = source.find('-').expect("negative token");
    assert_eq!(error.byte_range(), Some((start, start + 1)));
    let valid = source.replace("-1", "12");
    assert_eq!(
        compile_fen(valid.as_bytes()).unwrap().rust_source(),
        compile_ui(valid.parse().unwrap()).unwrap().rust_source(),
    );
}

#[test]
fn syntax_errors_cover_eof_bad_headers_and_trailing_input() {
    for source in [
        "",
        "format",
        "format 3",
        "format 3;",
        "format 3; view hello { rect root {",
    ] {
        let error = compile_fen(source.as_bytes()).expect_err("incomplete view");
        assert_eq!(error.byte_range(), Some((source.len(), source.len())));
        assert!(error.to_string().contains("incomplete"));
    }
    for source in [
        "format 1; view hello { rect root {} }",
        "format 2; view hello { rect root {} }",
        "format 03; view hello { rect root {} }",
        "format 3; view hello { rect root {} } view second { rect second {} }",
        "format 3; view hello { rect root {} rect extra {} }",
        "format 3; view hello { rect root { width: 1 height: 2; } }",
        "format 3; view hello { rect root { width 1; } }",
        "format 3; view hello { rect root { width: 1.5; } }",
        "format 3; view hello { rect root { width: 1px; } }",
        "format 3; view hello { rect root { width: 0x10; } }",
        "format 3; view hello { rect r#root {} }",
        "format 3; view hello { rect \u{e9} {} }",
        "format 3; view hello { rect root { background: rgba8(1,2,3); } }",
        "format 3; view hello { rect root { background: rgba8(1,2,3,4,5); } }",
        "format 3; view hello { rect root { background: red; } }",
    ] {
        assert!(compile_fen(source.as_bytes()).is_err(), "{source}");
    }
}

#[test]
fn invalid_utf8_is_located_without_decoding_or_echoing_source() {
    let source = b"format 3; // \xff\nview hello { rect root {} }";
    let failure = compile_fen(source).expect_err("UTF-8 is required even in comments");
    let start = source.iter().position(|&byte| byte == 0xff).unwrap();
    assert_eq!(failure.byte_range(), Some((start, start + 1)));
    assert_eq!(failure.to_string(), "source is not valid UTF-8");
}

#[test]
fn names_are_unique_across_nested_branches_and_are_not_rust_identifiers() {
    let source = "format 3; view type { column self { row left { rect item {} } row right { rect item {} } } }";
    let failure = compile_fen(source.as_bytes()).expect_err("duplicate across branches");
    assert_eq!(
        failure.byte_range().unwrap().0,
        source.rfind("item").unwrap()
    );
    let valid = source.replacen("rect item", "rect async", 1);
    assert!(compile_fen(valid.as_bytes()).is_ok());
    assert!(compile_ui(valid.parse().unwrap()).is_ok());
}

#[test]
fn documentation_comments_are_rejected_by_both_frontends() {
    for comment in ["/// docs\n", "//! docs\n", "/** docs */", "/*! docs */"] {
        let source = format!("{comment}format 3; view hello {{ rect root {{}} }}");
        assert!(compile_fen(source.as_bytes()).is_err(), "{comment}");
        assert!(compile_ui(source.parse().unwrap()).is_err(), "{comment}");
    }
    for comment in ["//// ordinary\n", "/**/", "/*** ordinary */"] {
        let source = format!("{comment}format 3; view hello {{ rect root {{}} }}");
        assert!(compile_fen(source.as_bytes()).is_ok(), "{comment}");
        assert!(compile_ui(source.parse().unwrap()).is_ok(), "{comment}");
    }
}

#[test]
fn macro_dispatch_emits_the_same_public_view_tokens() {
    use fenestra_ui_authoring::prototype::{
        REFERENCE_AUTHORING_LIMITS_V1, REFERENCE_AUTHORING_LIMITS_V2, expand_ui,
    };
    let tokens = SOURCE.parse().unwrap();
    let expanded = expand_ui(
        tokens,
        REFERENCE_AUTHORING_LIMITS_V1,
        REFERENCE_AUTHORING_LIMITS_V2,
    );
    assert_eq!(
        expanded.to_string(),
        compile_fen(SOURCE.as_bytes()).unwrap().tokens().to_string()
    );
}

#[test]
fn delimiter_limits_apply_to_color_arguments_and_nested_elements() {
    use fenestra_ui_authoring::view::compile_ui_with_limits;
    let source = "format 3; view hello { row root { rect child { background: rgba8(0,0,0,0); } } }";
    for depth in [0, 1, 2, 3] {
        let limits = Limits::new(4096, 2, depth, 1024, 4096);
        assert!(compile_fen_with_limits(source.as_bytes(), limits).is_err());
        assert!(compile_ui_with_limits(source.parse().unwrap(), limits).is_err());
    }
    let limits = Limits::new(4096, 2, 4, 1024, 4096);
    assert!(compile_fen_with_limits(source.as_bytes(), limits).is_ok());
    assert!(compile_ui_with_limits(source.parse().unwrap(), limits).is_ok());
}

#[test]
fn header_detection_ignores_invalid_bodies_and_matches_exact_format_tokens() {
    use fenestra_ui_authoring::view::has_format_header;
    for source in [
        b"format 3;".as_slice(),
        b"// leading\r\n/* outer /* inner */ */ format /**/ 3 /**/ ;",
        b"format 3; /* unterminated body",
        b"format 3; \xff",
        b"format 3; view hello { rect root { width: -1; } }",
    ] {
        assert!(has_format_header(source), "{source:?}");
    }
    for source in [
        "format 2;",
        "format 30;",
        "format 03;",
        "format 3",
        "format 3.0;",
        "format3;",
        "format 3u32;",
        "format 3 /* unterminated header",
        "/// docs\nformat 3;",
        "/* unterminated leading format 3;",
        "r#format 3;",
    ] {
        assert!(!has_format_header(source.as_bytes()), "{source}");
    }
}
