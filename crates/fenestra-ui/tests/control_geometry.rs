use fenestra_ui::{Application, Color, Element, InputEvent, Size, StateStyle, Style, View};

fn app() -> Application {
    let button = |name| {
        Element::button(name, name)
            .style(Style::new().width(40).height(20))
            .state_style(StateStyle::new().hover_background(Color::rgba8(200, 60, 40, 255)))
    };
    Application::new(
        View::new(
            "stationary_pointer",
            Element::row("root")
                .style(Style::new().width(100).height(20))
                .child(button("first"))
                .child(button("second")),
        ),
        Size::new(100, 20),
    )
    .unwrap()
}

#[test]
fn layout_changes_retarget_hover_and_pressed_paint_without_pointer_motion() {
    let mut app = app();
    app.dispatch_input(InputEvent::PointerMoved { x: 45, y: 5 })
        .unwrap();
    app.dispatch_input(InputEvent::PointerPressed).unwrap();
    assert!(app.control_snapshot("second").unwrap().state().pressed());
    let before = app.generation();
    app.set_size("first", 60, 20).unwrap();
    assert_eq!(app.generation(), before + 1);
    assert!(app.control_snapshot("first").unwrap().state().hovered());
    let second = app.control_snapshot("second").unwrap().state();
    assert!(!second.hovered());
    assert!(!second.pressed());
    assert_eq!(app.focused_control(), Some("second"));
    let pixels = app.raster().unwrap();
    assert_eq!(
        &pixels.bytes()[(5 * 100 + 45) * 4..(5 * 100 + 45) * 4 + 4],
        &[200, 60, 40, 255]
    );
    assert!(
        !app.dispatch_input(InputEvent::PointerReleased)
            .unwrap()
            .iter()
            .any(|event| matches!(event, fenestra_ui::Event::Activated { .. }))
    );
}

#[test]
fn viewport_clipping_clears_hover_without_losing_logical_keyboard_focus() {
    let mut app = app();
    app.dispatch_input(InputEvent::PointerMoved { x: 45, y: 5 })
        .unwrap();
    app.focus(Some("second")).unwrap();
    app.resize(Size::new(30, 20)).unwrap();
    assert_eq!(app.focused_control(), Some("second"));
    assert!(!app.control_snapshot("second").unwrap().state().hovered());
    assert!(app.focus(Some("first")).is_ok());
    assert!(app.focus(Some("second")).is_err());
}
