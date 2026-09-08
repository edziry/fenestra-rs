use fenestra_ui::{Event, ImeEvent, InputEvent, Key, KeyState, KeyboardInput, Modifiers};

#[test]
fn owned_input_and_application_events_are_available_without_native() {
    let key = KeyboardInput {
        key: Key::Character("a".into()),
        state: KeyState::Pressed,
        modifiers: Modifiers::default(),
        repeat: false,
        text: Some("a".into()),
        is_synthetic: false,
    };
    let input = InputEvent::KeyboardInput(key.clone());
    assert_eq!(input.clone(), input);
    assert_eq!(Event::KeyboardInput(key.clone()), Event::KeyboardInput(key));
    let ime = ImeEvent::Preedit {
        text: "a".into(),
        cursor: Some((0, 1)),
    };
    assert_eq!(InputEvent::Ime(ime.clone()), InputEvent::Ime(ime.clone()));
    assert_eq!(Event::Ime(ime.clone()), Event::Ime(ime));
}

#[cfg(all(feature = "native", any(target_os = "linux", target_os = "windows")))]
#[test]
fn native_exports_preserve_the_canonical_owned_type_identity() {
    use fenestra_ui::native;

    let event: native::WindowEvent = InputEvent::PointerPressed;
    let key: native::Key = Key::Enter;
    let state: native::KeyState = KeyState::Released;
    let modifiers: native::Modifiers = Modifiers::default();
    let input: native::KeyboardInput = KeyboardInput {
        key,
        state,
        modifiers,
        repeat: false,
        text: None,
        is_synthetic: false,
    };
    let ime: native::ImeEvent = ImeEvent::Disabled;
    assert_eq!(event, InputEvent::PointerPressed);
    assert_eq!(input.state, KeyState::Released);
    assert_eq!(InputEvent::Ime(ime), InputEvent::Ime(ImeEvent::Disabled));
}
