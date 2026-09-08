use fenestra_ui_authoring::view::{
    Limits, compile_fen, compile_fen_with_limits, compile_ui, compile_ui_with_limits,
};

fn source(body: &str) -> String {
    format!("format 3; view controls {{ {body} }}")
}

fn assert_rejected(body: &str, offending: &str, message: &str) {
    let source = source(body);
    let fen = compile_fen(source.as_bytes()).expect_err(body);
    assert_eq!(
        fen.byte_range().unwrap().0,
        source.find(offending).unwrap(),
        "{body}: {fen}",
    );
    assert!(fen.to_string().contains(message), "{body}: {fen}");
    let ui = compile_ui(source.parse().unwrap()).expect_err(body);
    assert_eq!(fen.to_string(), ui.to_string(), "{body}");
    assert!(ui.byte_range().is_none());
}

#[test]
fn controls_emit_public_constructors_and_identical_frontend_expressions() {
    let source = source(
        "column root { button save { label: \"Save\\nchanges\"; disabled: false; width: fill; height: auto; padding: 12; gap: 4; text caption { content: \"Save\"; } } checkbox enabled { label: \"Enable feature\"; checked: true; disabled: true; height: auto; rect indicator {} } }",
    );
    let fen = compile_fen(source.as_bytes()).expect("authored controls");
    let ui = compile_ui(source.parse().unwrap()).expect("macro controls");
    assert_eq!(fen.rust_source(), ui.rust_source());
    for fragment in [
        "Element :: button (\"save\" , \"Save\\nchanges\") . disabled (false)",
        "Element :: checkbox (\"enabled\" , \"Enable feature\") . disabled (true) . checked (true)",
        "width_mode (:: fenestra_ui :: Dimension :: Fill (1u32))",
        "padding (12i32) . gap (4i32)",
        "Element :: text (\"caption\" , \"Save\")",
        "Element :: rect (\"indicator\")",
    ] {
        assert!(fen.rust_source().contains(fragment), "{fragment}: {fen:?}");
    }
    assert!(!fen.rust_source().contains("prototype"));
    assert!(!fen.rust_source().contains("fenestra_ui_authoring"));
}

#[test]
fn optional_booleans_emit_only_when_authored_and_properties_have_canonical_order() {
    let absent = source("checkbox check { label: \"Check\"; }");
    let absent = compile_fen(absent.as_bytes()).unwrap();
    assert!(!absent.rust_source().contains("disabled"));
    assert!(!absent.rust_source().contains("checked"));
    assert!(!absent.rust_source().contains("StateStyle"));
    let first =
        source("checkbox check { label: \"Check\"; disabled: false; checked: false; width: 80; }");
    let reversed =
        source("checkbox check { width: 80; checked: false; disabled: false; label: \"Check\"; }");
    assert_eq!(
        compile_fen(first.as_bytes()).unwrap().rust_source(),
        compile_ui(reversed.parse().unwrap()).unwrap().rust_source(),
    );
}

#[test]
fn state_styles_apply_to_controls_and_descendants_without_implicit_nodes() {
    let source = source(
        "checkbox check { label: \"Check\"; hover_background: rgba8(1,2,3,4); pressed_background: rgba8(5,6,7,8); checked_background: rgba8(9,10,11,12); disabled_background: rgba8(13,14,15,16); focus_color: rgba8(25,26,27,28); row content { text caption { content: \"Visible\"; checked_color: rgba8(29,30,31,32); disabled_color: rgba8(33,34,35,36); } } }",
    );
    let fen = compile_fen(source.as_bytes()).unwrap();
    let ui = compile_ui(source.parse().unwrap()).unwrap();
    assert_eq!(fen.rust_source(), ui.rust_source());
    assert_eq!(fen.rust_source().matches("Element ::").count(), 3);
    assert_eq!(fen.rust_source().matches("StateStyle :: new ()").count(), 2);
    for property in [
        "hover_background",
        "pressed_background",
        "checked_background",
        "disabled_background",
        "checked_color",
        "disabled_color",
        "focus_color",
    ] {
        assert!(
            fen.rust_source()
                .contains(&format!("{property} (:: fenestra_ui :: Color :: rgba8"))
        );
    }
    assert!(
        fen.rust_source()
            .contains(". state_style (:: fenestra_ui :: StateStyle")
    );
}

#[test]
fn labels_and_boolean_properties_are_required_unique_and_typed() {
    for (body, token, message) in [
        ("button save {}", "}", "requires label"),
        ("checkbox check {}", "}", "requires label"),
        ("button save { label: \"\"; }", "\"\"", "nonempty"),
        ("button save { label: \" \\n\\t\"; }", "\" ", "nonempty"),
        ("button save { label: 1; }", "1", "string literal"),
        (
            "button save { label: \"a\"; label: \"b\"; }",
            "label: \"b\"",
            "duplicate",
        ),
        (
            "button save { label: \"a\"; disabled: 1; }",
            "1",
            "true or false",
        ),
        (
            "checkbox check { label: \"a\"; checked: accept; }",
            "accept",
            "true or false",
        ),
        (
            "checkbox check { label: \"a\"; checked: \"true\"; }",
            "\"true\"",
            "true or false",
        ),
        (
            "button save { label: \"a\"; disabled: true; disabled: false; }",
            "disabled: false",
            "duplicate",
        ),
        (
            "checkbox check { label: \"a\"; checked: true; checked: false; }",
            "checked: false",
            "duplicate",
        ),
        (
            "button save { label: \"a\"; checked: false; }",
            "checked",
            "checkbox",
        ),
        ("rect item { label: \"a\"; }", "label", "control"),
        ("column item { disabled: false; }", "disabled", "control"),
        (
            "button save { label: \"a\"; content: \"visible\"; }",
            "content",
            "text element",
        ),
    ] {
        assert_rejected(body, token, message);
    }
}

#[test]
fn nested_controls_and_derived_input_are_rejected_in_either_property_order() {
    for (body, token, message) in [
        (
            "button save { label: \"a\"; input: ignore; }",
            "input",
            "derived",
        ),
        (
            "checkbox check { input: accept; label: \"a\"; }",
            "input",
            "derived",
        ),
        (
            "button outer { label: \"a\"; checkbox inner { label: \"b\"; } }",
            "checkbox",
            "nested controls",
        ),
        (
            "checkbox outer { row wrapper { button inner { label: \"b\"; } } label: \"a\"; }",
            "button",
            "nested controls",
        ),
        (
            "button outer { disabled: true; label: \"a\"; button inner { disabled: true; label: \"b\"; } }",
            "button inner",
            "nested controls",
        ),
        (
            "button outer { label: \"a\"; rect child { input: accept; } }",
            "input",
            "inside a control",
        ),
        (
            "checkbox outer { column wrapper { rect child { input: accept; } } label: \"a\"; }",
            "input",
            "inside a control",
        ),
    ] {
        assert_rejected(body, token, message);
    }
    for body in [
        "button outer { label: \"a\"; row wrapper { input: ignore; text caption { content: \"A\"; input: ignore; } } }",
        "column root { button first { label: \"a\"; } checkbox second { label: \"b\"; } rect legacy { input: accept; } }",
        "column legacy { input: accept; rect child { input: accept; } button control { label: \"a\"; } }",
    ] {
        let source = source(body);
        assert_eq!(
            compile_fen(source.as_bytes()).expect(body).rust_source(),
            compile_ui(source.parse().unwrap())
                .expect(body)
                .rust_source(),
        );
    }
}

#[test]
fn state_style_scopes_and_values_report_the_authored_property() {
    for property in [
        "hover_background",
        "pressed_background",
        "checked_background",
        "disabled_background",
        "checked_color",
        "disabled_color",
        "focus_color",
    ] {
        assert_rejected(
            &format!("rect outside {{ {property}: rgba8(1,2,3,4); }}"),
            property,
            "control",
        );
    }
    for (body, token, message) in [
        (
            "button save { label: \"a\"; checked_color: rgba8(1,2,3,4); }",
            "checked_color",
            "checkbox",
        ),
        (
            "button save { text caption { content: \"a\"; checked_background: rgba8(1,2,3,4); } label: \"a\"; }",
            "checked_background",
            "checkbox",
        ),
        (
            "checkbox check { label: \"a\"; rect child { focus_color: rgba8(1,2,3,4); } }",
            "focus_color",
            "control element",
        ),
        (
            "checkbox check { label: \"a\"; checked_color: rgba8(1,2,3,4); }",
            "checked_color",
            "text element",
        ),
        (
            "button save { label: \"a\"; rect child { disabled_color: rgba8(1,2,3,4); } }",
            "disabled_color",
            "text element",
        ),
        (
            "button save { label: \"a\"; hover_background: rgba8(256,2,3,4); }",
            "256",
            "255",
        ),
        (
            "button save { label: \"a\"; text caption { content: \"a\"; disabled_color: rgba8(1,2,3,4); disabled_color: rgba8(5,6,7,8); } }",
            "disabled_color: rgba8(5",
            "duplicate",
        ),
        (
            "column root { button save { label: \"a\"; } hover_background: rgba8(1,2,3,4); }",
            "hover_background",
            "control",
        ),
    ] {
        assert_rejected(body, token, message);
    }
    let before = source(
        "checkbox check { label: \"a\"; checked_background: rgba8(1,2,3,4); row wrapper { text caption { content: \"a\"; checked_color: rgba8(5,6,7,8); } } }",
    );
    let after = source(
        "checkbox check { row wrapper { text caption { checked_color: rgba8(5,6,7,8); content: \"a\"; } } checked_background: rgba8(1,2,3,4); label: \"a\"; }",
    );
    assert_eq!(
        compile_fen(before.as_bytes()).unwrap().rust_source(),
        compile_ui(after.parse().unwrap()).unwrap().rust_source(),
    );
}

#[test]
fn label_strings_preserve_unicode_raw_literals_comments_and_error_offsets() {
    let commented = "// caf\u{e9}\r\nformat 3; view controls { button save { label: r#\"Save \\\" // /* \\n\"#; text caption { content: \"Save \\u{4e16}\"; } /* gap */ disabled: false; } }";
    let fen = compile_fen(commented.as_bytes()).unwrap();
    assert_eq!(
        fen.rust_source(),
        compile_ui(commented.parse().unwrap())
            .unwrap()
            .rust_source()
    );
    let malformed = commented.replace("disabled: false", "disabled: 4");
    let failure = compile_fen(malformed.as_bytes()).unwrap_err();
    let start = malformed.rfind('4').unwrap();
    assert_eq!(failure.byte_range(), Some((start, start + 1)));
    for literal in [r#""bad\q""#, r#""bad\u{110000}""#] {
        let source = source(&format!("button save {{ label: {literal}; }}"));
        let failure = compile_fen(source.as_bytes()).unwrap_err();
        assert_eq!(
            failure.byte_range().unwrap().0,
            source.find(literal).unwrap()
        );
        assert!(failure.to_string().contains("string"));
    }
}

#[test]
fn control_strings_and_state_styles_obey_inclusive_resource_limits() {
    let source = source(
        "checkbox check { label: \"ok\"; checked: true; disabled: false; focus_color: rgba8(1,2,3,4); }",
    );
    let output = compile_fen(source.as_bytes()).unwrap();
    let generated = output.rust_source().len();
    let exact = Limits::new(source.len(), 1, 3, 36, generated);
    assert!(compile_fen_with_limits(source.as_bytes(), exact).is_ok());
    assert!(compile_ui_with_limits(source.parse().unwrap(), exact).is_ok());
    for limits in [
        Limits::new(source.len(), 0, 3, 36, generated),
        Limits::new(source.len(), 1, 2, 36, generated),
        Limits::new(source.len(), 1, 3, 35, generated),
        Limits::new(source.len(), 1, 3, 36, generated - 1),
    ] {
        assert!(compile_fen_with_limits(source.as_bytes(), limits).is_err());
        assert!(compile_ui_with_limits(source.parse().unwrap(), limits).is_err());
    }
    assert!(
        compile_fen_with_limits(
            source.as_bytes(),
            Limits::new(source.len() - 1, 1, 3, 36, generated)
        )
        .is_err()
    );
}
