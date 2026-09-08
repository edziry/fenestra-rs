use super::*;
use fenestra_ui::{
    AccessibilityAction, AccessibilityActionRequest, AccessibilityId, AccessibilityRole,
};

#[test]
fn tree_exposes_standalone_text_without_duplicating_composed_control_content() {
    let engine = Rc::new(EngineState::default());
    let view = View::new(
        "labels",
        Element::column("content")
            .style(Style::new().width(32).height(32))
            .child(Element::text("title", "Preferences").style(Style::new().width(32).height(4)))
            .child(control(StateStyle::new()))
            .child(Element::text("readout", "Saved").style(Style::new().width(32).height(4))),
    );
    let mut app = Application::with_text_engine(view, Size::new(32, 32), Engine(engine)).unwrap();
    let tree = app.accessibility_tree().unwrap();
    assert_eq!(
        tree.nodes()
            .iter()
            .map(|node| node.name())
            .collect::<Vec<_>>(),
        ["", "content", "title", "choice", "readout"]
    );
    let old = tree.nodes().last().unwrap();
    assert_eq!(old.role(), AccessibilityRole::Label);
    assert_eq!(old.label(), "Saved");
    assert_eq!(old.bounds(), app.bounds("readout").unwrap());
    assert!(!old.focusable());
    assert!(old.control_state().is_none());
    app.set_text("readout", "Updated").unwrap();
    let new = app.accessibility_tree().unwrap();
    assert_eq!(old.label(), "Saved");
    assert_eq!(new.nodes().last().unwrap().id(), old.id());
    assert_eq!(new.nodes().last().unwrap().label(), "Updated");
    assert_eq!(new.generation(), tree.generation() + 1);
}

#[test]
fn rejected_accessibility_activation_preserves_tree_and_pending_input_until_retry() {
    let engine = Rc::new(EngineState::default());
    let mut app = application(&engine, StateStyle::new().checked_color(CHECKED), 128);
    app.focus(Some("choice")).unwrap();
    app.dispatch_input(key(KeyState::Pressed)).unwrap();
    let before = (app.accessibility_tree().unwrap(), app.raster().unwrap());
    let request = AccessibilityActionRequest {
        target: AccessibilityId::new(u64::from(
            app.control_snapshot("choice").unwrap().id().get(),
        )),
        action: AccessibilityAction::Activate,
    };
    engine.reject.set(Some(CHECKED));
    for _ in 0..2 {
        assert_eq!(
            app.dispatch_accessibility_action(request),
            Err(rejected_color())
        );
        assert_eq!(
            (app.accessibility_tree().unwrap(), app.raster().unwrap()),
            before
        );
    }
    engine.reject.set(None);
    assert_eq!(
        app.dispatch_accessibility_action(request).unwrap(),
        [Event::CheckedChanged {
            target: "choice".into(),
            checked: true
        }]
    );
    let after = app.control_snapshot("choice").unwrap();
    assert_eq!(after.state().checked(), Some(true));
    assert!(!after.state().pressed());
    assert!(
        !app.dispatch_input(key(KeyState::Released))
            .unwrap()
            .iter()
            .any(|event| matches!(event, Event::CheckedChanged { .. }))
    );
}
