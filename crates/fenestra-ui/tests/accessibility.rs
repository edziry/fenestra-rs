use fenestra_ui::{
    AccessibilityAction, AccessibilityActionRequest, AccessibilityId, AccessibilityRole,
    Application, Element, Event, InputEvent, Key, KeyState, KeyboardInput, Modifiers, Size, Style,
    View,
};

fn app() -> Application {
    Application::new(
        View::new(
            "accessible",
            Element::column("content")
                .style(Style::new().width(80).height(80).gap(4))
                .child(
                    Element::button("save", "Save changes")
                        .style(Style::new().width(40).height(20))
                        .child(Element::rect("decoration")),
                )
                .child(
                    Element::checkbox("automatic", "Save automatically")
                        .style(Style::new().width(40).height(20)),
                )
                .child(
                    Element::button("disabled", "Unavailable")
                        .disabled(true)
                        .style(Style::new().width(40).height(20)),
                ),
        ),
        Size::new(80, 80),
    )
    .unwrap()
}

fn request(app: &mut Application, name: &str, action: AccessibilityAction) -> Vec<Event> {
    let target = AccessibilityId::new(u64::from(app.control_snapshot(name).unwrap().id().get()));
    app.dispatch_accessibility_action(AccessibilityActionRequest { target, action })
        .unwrap()
}

#[test]
fn owned_tree_preserves_order_identity_and_control_labels_without_internal_decoration() {
    let mut app = app();
    let tree = app.accessibility_tree().unwrap();
    assert_eq!(tree.generation(), 0);
    assert_eq!(tree.viewport(), Size::new(80, 80));
    assert_eq!(tree.focus(), AccessibilityId::new(0));
    assert_eq!(
        tree.nodes()
            .iter()
            .map(|node| node.name())
            .collect::<Vec<_>>(),
        ["", "content", "save", "automatic", "disabled"]
    );
    let save = tree
        .nodes()
        .iter()
        .find(|node| node.name() == "save")
        .unwrap();
    assert_eq!(save.role(), AccessibilityRole::Button);
    assert_eq!(save.label(), "Save changes");
    assert_eq!(save.bounds(), app.bounds("save").unwrap());
    assert_eq!(
        save.id().get(),
        u64::from(app.control_snapshot("save").unwrap().id().get())
    );
    assert!(save.children().is_empty());
    assert!(save.focusable());
    assert_eq!(tree.nodes()[0].role(), AccessibilityRole::Window);
    assert_eq!(tree.nodes()[0].children(), &[AccessibilityId::new(1)]);
    app.set_control_label("save", "Publish changes").unwrap();
    assert_eq!(save.label(), "Save changes");
    assert_eq!(
        app.accessibility_tree().unwrap().nodes()[2].label(),
        "Publish changes"
    );
}

#[test]
fn semantic_actions_publish_checked_and_focus_events_without_forging_key_or_pointer_input() {
    let mut app = app();
    assert_eq!(
        request(&mut app, "automatic", AccessibilityAction::Focus),
        [Event::FocusChanged {
            target: Some("automatic".into())
        }]
    );
    assert_eq!(
        request(&mut app, "automatic", AccessibilityAction::Activate),
        [Event::CheckedChanged {
            target: "automatic".into(),
            checked: true
        }]
    );
    assert!(
        app.control_snapshot("automatic")
            .unwrap()
            .state()
            .checked()
            .unwrap()
    );
    assert_eq!(
        request(&mut app, "save", AccessibilityAction::Activate),
        [Event::Activated {
            target: "save".into()
        }]
    );
    assert_eq!(app.focused_control(), Some("automatic"));
    app.dispatch_input(InputEvent::Focused(false)).unwrap();
    let tree = app.accessibility_tree().unwrap();
    assert!(!tree.window_focused());
    assert_eq!(
        tree.focus().get(),
        u64::from(app.control_snapshot("automatic").unwrap().id().get())
    );
    assert_eq!(
        request(&mut app, "automatic", AccessibilityAction::Activate),
        [Event::CheckedChanged {
            target: "automatic".into(),
            checked: false
        }]
    );
}

#[test]
fn stale_disabled_noncontrol_and_offscreen_requests_are_inert() {
    let mut app = app();
    for target in [0, 1, 3, u64::MAX] {
        for action in [AccessibilityAction::Focus, AccessibilityAction::Activate] {
            let before = (
                app.generation(),
                app.raster().unwrap(),
                app.accessibility_tree().unwrap(),
            );
            assert!(
                app.dispatch_accessibility_action(AccessibilityActionRequest {
                    target: AccessibilityId::new(target),
                    action
                })
                .unwrap()
                .is_empty()
            );
            assert_eq!(
                (
                    app.generation(),
                    app.raster().unwrap(),
                    app.accessibility_tree().unwrap()
                ),
                before
            );
        }
    }
    assert!(request(&mut app, "disabled", AccessibilityAction::Activate).is_empty());
    app.resize(Size::new(80, 20)).unwrap();
    assert!(request(&mut app, "automatic", AccessibilityAction::Focus).is_empty());
    assert!(request(&mut app, "automatic", AccessibilityAction::Activate).is_empty());
}

#[test]
fn semantic_activation_cancels_a_pending_space_release() {
    let mut app = app();
    app.focus(Some("automatic")).unwrap();
    let key = |state| {
        InputEvent::KeyboardInput(KeyboardInput {
            key: Key::Space,
            state,
            modifiers: Modifiers::default(),
            repeat: false,
            text: None,
            is_synthetic: false,
        })
    };
    app.dispatch_input(key(KeyState::Pressed)).unwrap();
    assert_eq!(
        request(&mut app, "automatic", AccessibilityAction::Activate),
        [Event::CheckedChanged {
            target: "automatic".into(),
            checked: true
        }]
    );
    assert!(!app.control_snapshot("automatic").unwrap().state().pressed());
    assert!(
        !app.dispatch_input(key(KeyState::Released))
            .unwrap()
            .iter()
            .any(|event| matches!(event, Event::CheckedChanged { .. }))
    );
}
