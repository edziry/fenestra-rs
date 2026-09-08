use super::*;

#[test]
fn focus_remains_visible_when_an_opaque_child_fills_the_control() {
    for child in [
        Element::rect("fill").style(Style::new().width(8).height(8).background(CHECKED)),
        Element::text("fill", "abc")
            .style(Style::new().width(8).height(8))
            .text_style(TextStyle::new().color(CHECKED)),
    ] {
        let mut app = Application::with_text_engine(
            View::new(
                "covered",
                Element::button("button", "Button")
                    .style(Style::new().width(8).height(8))
                    .child(child),
            ),
            Size::new(8, 8),
            Engine(Rc::default()),
        )
        .unwrap();
        let unfocused = app.raster().unwrap();
        let bounds = app.bounds("button").unwrap();
        let child_bounds = app.bounds("fill").unwrap();
        app.focus(Some("button")).unwrap();
        assert!(app.control_snapshot("button").unwrap().state().focused());
        let focused = app.raster().unwrap();
        assert_ne!(focused, unfocused);
        assert_eq!(&focused.bytes()[..4], &[0, 0, 0, 255]);
        assert_eq!(&focused.bytes()[(8 + 1) * 4..(8 + 2) * 4], &[255; 4]);
        assert_eq!(
            &focused.bytes()[(3 * 8 + 3) * 4..(3 * 8 + 4) * 4],
            &CHECKED.to_rgba8()
        );
        assert_eq!(app.bounds("button").unwrap(), bounds);
        assert_eq!(app.bounds("fill").unwrap(), child_bounds);
        assert_eq!(app.hit_test(0, 0), Some("button"));
    }
}

#[test]
fn nested_text_focus_uses_control_coordinates_and_preserves_later_sibling_paint_order() {
    const LATER: Color = Color::rgba8(10, 30, 200, 255);
    let state = Rc::new(EngineState::default());
    let root = Element::row("root")
        .style(Style::new().width(16).height(12).padding(1))
        .child(
            Element::column("small")
                .style(Style::new().width(5).height(10))
                .child(
                    Element::button("button", "Button")
                        .style(Style::new().width(10).height(10).padding(2))
                        .child(
                            Element::text("fill", "abc")
                                .style(Style::new().width(6).height(6))
                                .text_style(TextStyle::new().color(CHECKED)),
                        ),
                ),
        )
        .child(
            Element::rect("later").style(
                Style::new()
                    .width(6)
                    .height(10)
                    .background(LATER)
                    .input(true),
            ),
        );
    let mut app = Application::with_text_engine(
        View::new("nested", root),
        Size::new(16, 12),
        Engine(state.clone()),
    )
    .unwrap();
    let bounds = app.bounds("button").unwrap();
    let text_bounds = app.bounds("fill").unwrap();
    let calls = state.calls();
    app.focus(Some("button")).unwrap();
    let raster = app.raster().unwrap();
    let pixel = |x: usize, y: usize| &raster.bytes()[(y * 16 + x) * 4..(y * 16 + x + 1) * 4];
    assert_eq!(pixel(1, 1), &[0, 0, 0, 255]);
    assert_eq!(pixel(2, 2), &[255; 4]);
    assert_eq!(pixel(4, 4), &CHECKED.to_rgba8());
    assert_eq!(pixel(6, 1), &LATER.to_rgba8());
    assert_eq!(pixel(10, 1), &LATER.to_rgba8());
    assert_eq!(app.hit_test(10, 1), Some("later"));
    assert_eq!(app.bounds("button").unwrap(), bounds);
    assert_eq!(app.bounds("fill").unwrap(), text_bounds);
    assert_eq!(state.calls(), calls);
}

fn byte_limit(actual: usize, limit: usize) -> Error {
    Error::Text(TextError::LimitExceeded {
        resource: "text bytes",
        actual,
        limit,
    })
}

#[test]
fn construction_counts_all_semantic_labels_and_visible_text_before_engine_work() {
    let state = Rc::new(EngineState::default());
    let view = View::new(
        "budget",
        Element::column("root")
            .child(control(StateStyle::new()))
            .child(Element::button("other", "Other")),
    );
    let result = Application::with_limits_and_text_engine(
        view,
        Size::new(64, 64),
        Limits::default().text_limits(TextLimits::new(13, 1024, 128)),
        Engine(state.clone()),
    );
    assert_eq!(result.err(), Some(byte_limit(14, 13)));
    assert_eq!(state.calls(), (0, 0));
}

#[test]
fn semantic_label_and_visible_text_share_one_budget_and_owned_snapshots_keep_their_id() {
    let state = Rc::new(EngineState::default());
    let mut app = application(&state, StateStyle::new(), 9);
    let old = app.control_snapshot("choice").unwrap();
    let frame = app.raster().unwrap();
    let calls = state.calls();
    let generation = app.generation();
    assert_eq!(
        app.set_control_label("choice", "Options"),
        Err(byte_limit(10, 9))
    );
    assert_eq!(app.set_text("label", "abcd"), Err(byte_limit(10, 9)));
    assert_eq!(app.control_snapshot("choice").unwrap(), old);
    assert_eq!(app.text("label").unwrap(), "abc");
    assert_eq!(app.generation(), generation);
    assert_eq!(app.raster().unwrap(), frame);
    assert_eq!(state.calls(), calls);
    app.set_control_label("choice", "Switch").unwrap();
    let renamed = app.control_snapshot("choice").unwrap();
    assert_eq!(renamed.id(), old.id());
    assert_eq!(renamed.label(), "Switch");
    assert_eq!(old.label(), "Choice");
    assert_eq!(renamed.bounds(), old.bounds());
    assert_eq!(app.text("label").unwrap(), "abc");
    assert_eq!(app.raster().unwrap(), frame);
    assert_eq!(state.calls(), calls);
    assert_eq!(app.generation(), generation + 1);
    app.set_control_label("choice", "X").unwrap();
    app.set_text("label", "abcdefgh").unwrap();
    assert_eq!(app.text("label").unwrap(), "abcdefgh");
    assert_eq!(app.control_snapshot("choice").unwrap().id(), old.id());
    assert!(state.layouts.get() > calls.1);
}

#[test]
fn invalid_control_context_is_rejected_before_text_preparation() {
    let state = Rc::new(EngineState::default());
    for (element, invalid_node) in [
        (Element::rect("plain").disabled(false), "plain"),
        (Element::button("button", "Button").checked(false), "button"),
        (
            Element::rect("plain").state_style(StateStyle::new().hover_background(BASE)),
            "plain",
        ),
        (
            Element::button("outer", "Outer").child(Element::checkbox("inner", "Inner")),
            "inner",
        ),
        (
            Element::button("button", "Button")
                .child(Element::text("label", "abc").style(Style::new().input(true))),
            "label",
        ),
        (
            Element::button("button", "Button").child(
                Element::text("label", "abc").state_style(StateStyle::new().checked_color(CHECKED)),
            ),
            "label",
        ),
    ] {
        let result = Application::with_text_engine(
            View::new("invalid", element),
            Size::new(64, 64),
            Engine(state.clone()),
        );
        assert!(
            matches!(result.err(), Some(Error::InvalidElement { node, .. }) if node == invalid_node)
        );
    }
    assert_eq!(state.calls(), (0, 0));
}

#[test]
fn invalid_public_setters_preserve_control_text_and_frame() {
    let state = Rc::new(EngineState::default());
    let mut app = application(&state, StateStyle::new(), 128);
    let old = app.control_snapshot("choice").unwrap();
    let frame = app.raster().unwrap();
    let generation = app.generation();
    let calls = state.calls();
    for result in [
        app.set_disabled("label", true),
        app.set_checked("label", true),
        app.set_control_label("label", "Label"),
        app.set_control_label("choice", " \n\t"),
        app.focus(Some("label")),
        app.set_state_style("choice", StateStyle::new().disabled_color(DISABLED)),
        app.set_state_style("label", StateStyle::new().focus_color(BASE)),
        app.set_style("label", app.style("label").unwrap().input(true)),
    ] {
        assert!(matches!(result, Err(Error::InvalidElement { .. })));
    }
    assert_eq!(app.control_snapshot("choice").unwrap(), old);
    assert_eq!(app.focused_control(), None);
    assert_eq!(app.text("label").unwrap(), "abc");
    assert_eq!(app.text_style("label").unwrap().color_value(), BASE);
    assert_eq!(app.state_style("label").unwrap(), StateStyle::new());
    assert_eq!(app.generation(), generation);
    assert_eq!(app.raster().unwrap(), frame);
    assert_eq!(state.calls(), calls);
}
