use super::*;

#[test]
fn key_cancellation_preserves_holds_until_up() {
    for cancellation in [
        InputEvent::KeyboardInput(keyboard(Key::Escape, KeyState::Pressed)),
        InputEvent::ModifiersChanged(Modifiers {
            control: true,
            ..Modifiers::default()
        }),
        InputEvent::Ime(ImeEvent::Preedit {
            text: "a".into(),
            cursor: None,
        }),
    ] {
        let mut state = focused(2);
        key(&mut state, Key::Space, true);
        assert_eq!(key(&mut state, Key::Enter, true), Some(2));
        send(&mut state, cancellation, None);
        assert!(!state.state(2, false, None).pressed());
        send(&mut state, InputEvent::Ime(ImeEvent::Disabled), None);
        assert_eq!(key(&mut state, Key::Enter, true), None);
        assert_eq!(key(&mut state, Key::Space, true), None);
        assert_eq!(key(&mut state, Key::Space, false), None);
        key(&mut state, Key::Enter, false);
        assert_eq!(key(&mut state, Key::Enter, true), Some(2));
    }
}

#[test]
fn focus_change_and_down_without_focus_cannot_transfer_key_holds() {
    for initial in [None, Some(2)] {
        let mut state = Interaction::default();
        state.set_focus(initial);
        key(&mut state, Key::Space, true);
        key(&mut state, Key::Enter, true);
        state.set_focus(Some(8));
        assert_eq!(key(&mut state, Key::Space, true), None);
        assert_eq!(key(&mut state, Key::Space, false), None);
        key(&mut state, Key::Space, true);
        assert_eq!(key(&mut state, Key::Space, false), Some(8));
        state.set_focus(Some(2));
        assert_eq!(key(&mut state, Key::Enter, true), None);
        key(&mut state, Key::Enter, false);
        assert_eq!(key(&mut state, Key::Enter, true), Some(2));
    }
}

#[test]
fn repeat_and_synthetic_presses_do_not_arm_or_activate_but_synthetic_up_clears() {
    for (repeat, synthetic) in [(true, false), (false, true)] {
        for logical in [Key::Enter, Key::Space, Key::Tab] {
            let mut state = focused(2);
            let old = state.clone();
            let mut input = keyboard(logical.clone(), KeyState::Pressed);
            input.repeat = repeat;
            input.is_synthetic = synthetic;
            assert_eq!(
                send(&mut state, InputEvent::KeyboardInput(input), None),
                None
            );
            assert_eq!(state, old);
            key(&mut state, logical.clone(), true);
            let mut release = keyboard(logical.clone(), KeyState::Released);
            release.is_synthetic = true;
            assert_eq!(
                send(&mut state, InputEvent::KeyboardInput(release), None),
                None
            );
            assert!(!state.tab_down);
            assert!(!state.state(2, false, None).pressed());
            if logical == Key::Enter {
                assert_eq!(key(&mut state, logical, true), Some(2));
            }
        }
    }
}

#[test]
fn control_alt_and_super_shortcuts_cancel_arms_but_shift_is_allowed() {
    for modifiers in [
        Modifiers {
            control: true,
            ..Modifiers::default()
        },
        Modifiers {
            alt: true,
            ..Modifiers::default()
        },
        Modifiers {
            super_key: true,
            ..Modifiers::default()
        },
    ] {
        let mut state = focused(2);
        key(&mut state, Key::Space, true);
        let mut input = keyboard(Key::Enter, KeyState::Pressed);
        input.modifiers = modifiers;
        assert_eq!(
            send(&mut state, InputEvent::KeyboardInput(input), None),
            None
        );
        assert_eq!(key(&mut state, Key::Space, false), None);
        assert_eq!(key(&mut state, Key::Enter, true), None);
    }
    let mut state = focused(2);
    let mut input = keyboard(Key::Enter, KeyState::Pressed);
    input.modifiers.shift = true;
    assert_eq!(
        send(&mut state, InputEvent::KeyboardInput(input), None),
        Some(2)
    );
}

#[test]
fn focus_loss_hides_retained_focus_resets_holds_and_ignores_late_input() {
    let mut state = focused(2);
    send(&mut state, InputEvent::PointerMoved { x: 5, y: 6 }, Some(2));
    send(&mut state, InputEvent::PointerPressed, Some(2));
    key(&mut state, Key::Space, true);
    key(&mut state, Key::Enter, true);
    send(&mut state, InputEvent::Focused(false), None);
    assert_eq!(state.focus, Some(2));
    assert_eq!(
        (state.pointer, state.hover, state.pointer_arm),
        (None, None, None)
    );
    let hidden = state.state(2, false, None);
    assert!(!hidden.focused() && !hidden.hovered() && !hidden.pressed());
    assert_eq!(key(&mut state, Key::Enter, true), None);
    assert_eq!(send(&mut state, InputEvent::PointerReleased, Some(2)), None);
    send(
        &mut state,
        InputEvent::Ime(ImeEvent::Preedit {
            text: "late".into(),
            cursor: None,
        }),
        None,
    );
    assert!(!state.composing);
    send(&mut state, InputEvent::Focused(true), None);
    assert!(state.state(2, false, None).focused());
    assert_eq!(key(&mut state, Key::Enter, true), Some(2));
    assert_eq!(key(&mut state, Key::Space, false), None);
}

#[test]
fn composition_suppresses_activation_and_legacy_space_never_activates() {
    let mut state = focused(2);
    send(
        &mut state,
        InputEvent::Ime(ImeEvent::Preedit {
            text: "a".into(),
            cursor: None,
        }),
        None,
    );
    assert!(state.composing);
    assert_eq!(key(&mut state, Key::Enter, true), None);
    assert_eq!(key(&mut state, Key::Space, true), None);
    assert_eq!(key(&mut state, Key::Space, false), None);
    send(&mut state, InputEvent::PointerPressed, Some(2));
    assert_eq!(send(&mut state, InputEvent::PointerReleased, Some(2)), None);
    send(
        &mut state,
        InputEvent::Ime(ImeEvent::Commit("a".into())),
        None,
    );
    assert!(!state.composing);
    assert_eq!(send(&mut state, InputEvent::SpacePressed, None), None);
    assert_eq!(key(&mut state, Key::Enter, true), None);
    key(&mut state, Key::Enter, false);
    assert_eq!(key(&mut state, Key::Enter, true), Some(2));
}

#[test]
fn disabling_clears_focus_and_arms_but_offscreen_only_blocks_activation() {
    let mut state = focused(2);
    key(&mut state, Key::Space, true);
    let mut changed = targets();
    changed[0].focusable = false;
    state.reconcile_disabled(&changed);
    assert_eq!(state.focus, Some(2));
    let release = InputEvent::KeyboardInput(keyboard(Key::Space, KeyState::Released));
    assert_eq!(state.reduce(&release, &changed, None).activate, None);
    changed[0].enabled = false;
    state.reconcile_disabled(&changed);
    assert_eq!(state.focus, None);
    assert_eq!(state.space_arm, None);
    state.set_focus(Some(2));
    assert_eq!(key(&mut state, Key::Space, true), None);
    assert_eq!(key(&mut state, Key::Space, false), None);
    state.reconcile_disabled(&[]);
    assert_eq!(state.focus, None);
}
