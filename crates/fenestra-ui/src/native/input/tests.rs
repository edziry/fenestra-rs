use winit::event::{ElementState, Ime, WindowEvent as PlatformEvent};
use winit::keyboard::{Key as PlatformKey, KeyCode, ModifiersState, NamedKey, PhysicalKey};

use super::{InputState, logical_key, requests_redraw};
use crate::native::{ImeEvent, Key, KeyState, KeyboardInput, Modifiers, WindowEvent};

fn press(input: &mut InputState, text: &str) -> Vec<WindowEvent> {
    input.keyboard_event(
        &PlatformKey::Character(text.into()),
        Some(text),
        PhysicalKey::Code(KeyCode::KeyA),
        ElementState::Pressed,
        false,
        false,
    )
}

fn keyboard(events: &[WindowEvent]) -> &KeyboardInput {
    let WindowEvent::KeyboardInput(input) = &events[0] else {
        panic!("expected keyboard input")
    };
    input
}

#[test]
fn logical_keys_keep_text_and_use_owned_navigation_names() {
    for (platform, expected) in [
        (NamedKey::Space, Key::Space),
        (NamedKey::Enter, Key::Enter),
        (NamedKey::Tab, Key::Tab),
        (NamedKey::Backspace, Key::Backspace),
        (NamedKey::Delete, Key::Delete),
        (NamedKey::Escape, Key::Escape),
        (NamedKey::ArrowLeft, Key::ArrowLeft),
        (NamedKey::ArrowRight, Key::ArrowRight),
        (NamedKey::ArrowUp, Key::ArrowUp),
        (NamedKey::ArrowDown, Key::ArrowDown),
        (NamedKey::Home, Key::Home),
        (NamedKey::End, Key::End),
        (NamedKey::PageUp, Key::PageUp),
        (NamedKey::PageDown, Key::PageDown),
        (NamedKey::Insert, Key::Insert),
        (NamedKey::Shift, Key::Shift),
        (NamedKey::Control, Key::Control),
        (NamedKey::Alt, Key::Alt),
        (NamedKey::Super, Key::Super),
        (NamedKey::AudioVolumeUp, Key::Unidentified),
    ] {
        assert_eq!(logical_key(&PlatformKey::Named(platform)), expected);
    }
    assert_eq!(
        logical_key(&PlatformKey::Character("\u{e9}".into())),
        Key::Character("\u{e9}".into())
    );
    assert_eq!(
        logical_key(&PlatformKey::Dead(Some('^'))),
        Key::Dead(Some('^'))
    );
}

#[test]
fn key_text_is_distinct_from_logical_key_and_preserves_altgr_input() {
    let mut state = InputState::default();
    let modifiers = ModifiersState::CONTROL | ModifiersState::ALT;
    assert_eq!(
        state.application_events(&PlatformEvent::ModifiersChanged(modifiers.into())),
        Ok(vec![WindowEvent::ModifiersChanged(Modifiers {
            control: true,
            alt: true,
            ..Modifiers::default()
        })])
    );
    let events = state.keyboard_event(
        &PlatformKey::Character("e".into()),
        Some("\u{b4}e"),
        PhysicalKey::Code(KeyCode::KeyE),
        ElementState::Pressed,
        false,
        false,
    );
    assert_eq!(keyboard(&events).key, Key::Character("e".into()));
    assert_eq!(keyboard(&events).text.as_deref(), Some("\u{b4}e"));
    assert!(keyboard(&events).modifiers.control);
    assert!(keyboard(&events).modifiers.alt);
}

#[test]
fn repeats_releases_and_synthetic_keys_keep_metadata_without_spurious_text() {
    let mut state = InputState::default();
    for (key_state, repeat, synthetic, expected_text) in [
        (ElementState::Pressed, false, false, Some("a")),
        (ElementState::Pressed, true, false, Some("a")),
        (ElementState::Released, false, false, None),
        (ElementState::Pressed, false, true, None),
    ] {
        let events = state.keyboard_event(
            &PlatformKey::Character("a".into()),
            Some("a"),
            PhysicalKey::Code(KeyCode::KeyA),
            key_state,
            repeat,
            synthetic,
        );
        let input = keyboard(&events);
        assert_eq!(input.text.as_deref(), expected_text);
        assert_eq!(input.repeat, repeat);
        assert_eq!(input.is_synthetic, synthetic);
        assert_eq!(
            input.state,
            if key_state == ElementState::Pressed {
                KeyState::Pressed
            } else {
                KeyState::Released
            }
        );
    }
}

#[test]
fn legacy_space_occurs_once_per_real_nonrepeating_press() {
    let mut state = InputState::default();
    for (key_state, repeat, synthetic, count) in [
        (ElementState::Pressed, false, false, 1),
        (ElementState::Pressed, false, false, 0),
        (ElementState::Pressed, true, false, 0),
        (ElementState::Released, false, false, 0),
        (ElementState::Pressed, false, true, 0),
        (ElementState::Released, false, true, 0),
        (ElementState::Pressed, false, false, 1),
    ] {
        let events = state.keyboard_event(
            &PlatformKey::Named(NamedKey::Space),
            Some(" "),
            PhysicalKey::Code(KeyCode::Space),
            key_state,
            repeat,
            synthetic,
        );
        assert_eq!(
            events
                .iter()
                .filter(|event| **event == WindowEvent::SpacePressed)
                .count(),
            count
        );
        assert_eq!(events.len(), 1 + count);
    }
}

#[test]
fn ime_preedit_and_commit_have_one_text_delivery_path() {
    let mut state = InputState::default();
    assert_eq!(
        state.application_events(&PlatformEvent::Ime(Ime::Enabled)),
        Ok(vec![WindowEvent::Ime(ImeEvent::Enabled)])
    );
    assert_eq!(keyboard(&press(&mut state, "a")).text.as_deref(), Some("a"));
    assert_eq!(
        state.application_events(&PlatformEvent::Ime(Ime::Preedit(
            "\u{e9}".into(),
            Some((0, 2))
        ))),
        Ok(vec![WindowEvent::Ime(ImeEvent::Preedit {
            text: "\u{e9}".into(),
            cursor: Some((0, 2))
        })])
    );
    assert_eq!(keyboard(&press(&mut state, "e")).text, None);
    assert_eq!(
        state.application_events(&PlatformEvent::Ime(Ime::Preedit(String::new(), None))),
        Ok(vec![WindowEvent::Ime(ImeEvent::Preedit {
            text: String::new(),
            cursor: None
        })])
    );
    assert_eq!(
        state.application_events(&PlatformEvent::Ime(Ime::Commit("\u{e9}".into()))),
        Ok(vec![WindowEvent::Ime(ImeEvent::Commit("\u{e9}".into()))])
    );
    assert_eq!(keyboard(&press(&mut state, "a")).text.as_deref(), Some("a"));
    state
        .application_events(&PlatformEvent::Ime(Ime::Preedit("x".into(), None)))
        .unwrap();
    assert_eq!(
        state.application_events(&PlatformEvent::Ime(Ime::Disabled)),
        Ok(vec![WindowEvent::Ime(ImeEvent::Disabled)])
    );
    assert_eq!(keyboard(&press(&mut state, "a")).text.as_deref(), Some("a"));
}

#[test]
fn focus_loss_clears_modifiers_composition_and_held_space() {
    let mut state = InputState::default();
    state
        .application_events(&PlatformEvent::ModifiersChanged(
            ModifiersState::all().into(),
        ))
        .unwrap();
    assert_eq!(
        keyboard(&press(&mut state, "a")).modifiers,
        Modifiers {
            shift: true,
            control: true,
            alt: true,
            super_key: true
        }
    );
    state
        .application_events(&PlatformEvent::Ime(Ime::Preedit("x".into(), None)))
        .unwrap();
    state.keyboard_event(
        &PlatformKey::Named(NamedKey::Space),
        Some(" "),
        PhysicalKey::Code(KeyCode::Space),
        ElementState::Pressed,
        false,
        false,
    );
    assert_eq!(
        state.application_events(&PlatformEvent::Focused(false)),
        Ok(vec![
            WindowEvent::ModifiersChanged(Modifiers::default()),
            WindowEvent::Focused(false)
        ])
    );
    assert_eq!(keyboard(&press(&mut state, "a")).text, None);
    assert_eq!(
        state.application_events(&PlatformEvent::Focused(true)),
        Ok(vec![WindowEvent::Focused(true)])
    );
    let events = press(&mut state, "a");
    assert_eq!(keyboard(&events).modifiers, Modifiers::default());
    assert_eq!(keyboard(&events).text.as_deref(), Some("a"));
    let space = state.keyboard_event(
        &PlatformKey::Named(NamedKey::Space),
        Some(" "),
        PhysicalKey::Code(KeyCode::Space),
        ElementState::Pressed,
        false,
        false,
    );
    assert_eq!(space.last(), Some(&WindowEvent::SpacePressed));
}

#[test]
fn focus_modifiers_and_ime_changes_request_a_frame() {
    for event in [
        PlatformEvent::Focused(false),
        PlatformEvent::ModifiersChanged(ModifiersState::SHIFT.into()),
        PlatformEvent::Ime(Ime::Enabled),
        PlatformEvent::Ime(Ime::Preedit("a".into(), None)),
        PlatformEvent::Ime(Ime::Commit("a".into())),
        PlatformEvent::Ime(Ime::Disabled),
    ] {
        assert!(requests_redraw(&event));
    }
}
