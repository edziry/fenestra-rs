use fenestra_ui::{
    Application, Color, ControlRole, Element, Event, InputEvent, Key, KeyState, KeyboardInput,
    Modifiers, Size, StateStyle, Style, View,
};

fn app() -> Application {
    Application::new(
        View::new(
            "controls",
            Element::column("root")
                .style(Style::new().width(120).height(120).gap(4))
                .child(
                    Element::button("save", "Save changes")
                        .style(Style::new().width(40).height(20)),
                )
                .child(
                    Element::checkbox("autosave", "Automatic saving")
                        .style(Style::new().width(40).height(20))
                        .child(
                            Element::rect("mark")
                                .style(Style::new().width(8).height(8))
                                .state_style(
                                    StateStyle::new()
                                        .checked_background(Color::rgba8(20, 200, 80, 255)),
                                ),
                        ),
                )
                .child(
                    Element::button("unavailable", "Unavailable")
                        .disabled(true)
                        .style(Style::new().width(40).height(20)),
                ),
        ),
        Size::new(120, 120),
    )
    .unwrap()
}

fn key(key: Key, state: KeyState) -> InputEvent {
    InputEvent::KeyboardInput(KeyboardInput {
        key,
        state,
        modifiers: Modifiers::default(),
        repeat: false,
        text: None,
        is_synthetic: false,
    })
}

#[test]
fn keyboard_navigation_and_activation_use_semantic_controls_without_native_features() {
    let mut app = app();
    app.dispatch_input(key(Key::Tab, KeyState::Pressed))
        .unwrap();
    assert_eq!(app.focused_control(), Some("save"));
    assert!(
        app.dispatch_input(key(Key::Enter, KeyState::Pressed))
            .unwrap()
            .contains(&Event::Activated {
                target: "save".into()
            })
    );
    app.dispatch_input(key(Key::Enter, KeyState::Released))
        .unwrap();
    app.dispatch_input(key(Key::Tab, KeyState::Released))
        .unwrap();
    app.dispatch_input(key(Key::Tab, KeyState::Pressed))
        .unwrap();
    assert_eq!(app.focused_control(), Some("autosave"));
    assert!(
        app.dispatch_input(key(Key::Space, KeyState::Pressed))
            .unwrap()
            .iter()
            .all(|event| !matches!(event, Event::CheckedChanged { .. }))
    );
    assert_eq!(
        app.control_snapshot("autosave").unwrap().state().checked(),
        Some(false)
    );
    let events = app
        .dispatch_input(key(Key::Space, KeyState::Released))
        .unwrap();
    assert!(events.contains(&Event::CheckedChanged {
        target: "autosave".into(),
        checked: true
    }));
    assert_eq!(
        app.control_snapshot("autosave").unwrap().state().checked(),
        Some(true)
    );
    let frame = app.raster().unwrap();
    assert_eq!(
        &frame.bytes()[(27 * 120 + 3) * 4..(27 * 120 + 3) * 4 + 4],
        &[20, 200, 80, 255]
    );
    assert_eq!(
        &frame.bytes()[(24 * 120 + 3) * 4..(24 * 120 + 3) * 4 + 4],
        &[0, 0, 0, 255]
    );
}

#[test]
fn pointer_activation_requires_matching_release_and_cancellation_discards_the_arm() {
    let mut app = app();
    app.dispatch_input(InputEvent::PointerMoved { x: 4, y: 4 })
        .unwrap();
    let press = app.dispatch_input(InputEvent::PointerPressed).unwrap();
    assert!(press.contains(&Event::Click {
        target: Some("save".into())
    }));
    assert!(
        !press
            .iter()
            .any(|event| matches!(event, Event::Activated { .. }))
    );
    app.dispatch_input(InputEvent::PointerMoved { x: 60, y: 4 })
        .unwrap();
    assert!(
        !app.dispatch_input(InputEvent::PointerReleased)
            .unwrap()
            .iter()
            .any(|event| matches!(event, Event::Activated { .. }))
    );
    app.dispatch_input(InputEvent::PointerMoved { x: 4, y: 4 })
        .unwrap();
    app.dispatch_input(InputEvent::PointerPressed).unwrap();
    app.dispatch_input(InputEvent::PointerLeft).unwrap();
    app.dispatch_input(InputEvent::PointerMoved { x: 4, y: 4 })
        .unwrap();
    assert!(
        !app.dispatch_input(InputEvent::PointerReleased)
            .unwrap()
            .iter()
            .any(|event| matches!(event, Event::Activated { .. }))
    );
    app.dispatch_input(InputEvent::PointerPressed).unwrap();
    assert!(
        app.dispatch_input(InputEvent::PointerReleased)
            .unwrap()
            .contains(&Event::Activated {
                target: "save".into()
            })
    );
}

#[test]
fn disabling_a_focused_control_clears_focus_and_preserves_owned_semantic_snapshots() {
    let mut app = app();
    app.focus(Some("save")).unwrap();
    let old = app.control_snapshot("save").unwrap();
    assert_eq!(old.role(), ControlRole::Button);
    assert_eq!(old.label(), "Save changes");
    assert!(old.state().focused());
    app.set_disabled("save", true).unwrap();
    assert_eq!(app.focused_control(), None);
    let new = app.control_snapshot("save").unwrap();
    assert_eq!(old.id(), new.id());
    assert!(!old.state().disabled());
    assert!(new.state().disabled());
    let before = app.raster().unwrap();
    assert!(app.focus(Some("save")).is_err());
    assert_eq!(app.raster().unwrap(), before);
}
