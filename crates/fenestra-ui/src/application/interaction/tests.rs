use super::*;
use crate::{ImeEvent, Key, KeyState, KeyboardInput, Modifiers};

mod cancellation;

fn targets() -> [Target; 4] {
    [
        Target {
            index: 2,
            role: ControlRole::Button,
            enabled: true,
            focusable: true,
        },
        Target {
            index: 4,
            role: ControlRole::Button,
            enabled: false,
            focusable: true,
        },
        Target {
            index: 6,
            role: ControlRole::Button,
            enabled: true,
            focusable: false,
        },
        Target {
            index: 8,
            role: ControlRole::Checkbox,
            enabled: true,
            focusable: true,
        },
    ]
}

fn keyboard(key: Key, state: KeyState) -> KeyboardInput {
    KeyboardInput {
        key,
        state,
        modifiers: Modifiers::default(),
        repeat: false,
        text: None,
        is_synthetic: false,
    }
}

fn key(state: &mut Interaction, key: Key, pressed: bool) -> Option<usize> {
    let input = keyboard(
        key,
        if pressed {
            KeyState::Pressed
        } else {
            KeyState::Released
        },
    );
    send(state, InputEvent::KeyboardInput(input), None)
}

fn send(state: &mut Interaction, input: InputEvent, hit: Option<usize>) -> Option<usize> {
    let result = state.reduce(&input, &targets(), hit);
    *state = result.next;
    result.activate
}

fn focused(index: usize) -> Interaction {
    let mut state = Interaction::default();
    state.set_focus(Some(index));
    state
}

#[test]
fn enter_activates_button_on_fresh_down_until_release() {
    let mut state = focused(2);
    assert_eq!(key(&mut state, Key::Enter, true), Some(2));
    assert!(state.state(2, false, None).pressed());
    assert_eq!(key(&mut state, Key::Enter, true), None);
    assert_eq!(key(&mut state, Key::Enter, false), None);
    assert!(!state.state(2, false, None).pressed());
    assert_eq!(key(&mut state, Key::Enter, true), Some(2));
}

#[test]
fn checkbox_ignores_enter_and_both_roles_activate_space_on_up() {
    let mut checkbox = focused(8);
    assert_eq!(key(&mut checkbox, Key::Enter, true), None);
    assert!(!checkbox.state(8, false, Some(false)).pressed());
    for index in [2, 8] {
        for space in [Key::Space, Key::Character(" ".into())] {
            let mut state = focused(index);
            assert_eq!(key(&mut state, space.clone(), true), None);
            assert!(state.state(index, false, None).pressed());
            assert_eq!(key(&mut state, space.clone(), true), None);
            assert_eq!(key(&mut state, space, false), Some(index));
            assert!(!state.state(index, false, None).pressed());
            assert_eq!(key(&mut state, Key::Space, false), None);
        }
    }
}

#[test]
fn tab_cycles_authored_order_skipping_disabled_and_offscreen_targets() {
    let mut state = Interaction::default();
    key(&mut state, Key::Tab, true);
    assert_eq!(state.focus, Some(2));
    key(&mut state, Key::Tab, true);
    assert_eq!(state.focus, Some(2));
    key(&mut state, Key::Tab, false);
    key(&mut state, Key::Tab, true);
    assert_eq!(state.focus, Some(8));
    key(&mut state, Key::Tab, false);
    key(&mut state, Key::Tab, true);
    assert_eq!(state.focus, Some(2));
    key(&mut state, Key::Tab, false);
    let mut reverse = keyboard(Key::Tab, KeyState::Pressed);
    reverse.modifiers.shift = true;
    send(&mut state, InputEvent::KeyboardInput(reverse.clone()), None);
    assert_eq!(state.focus, Some(8));
    let initial =
        Interaction::default().reduce(&InputEvent::KeyboardInput(reverse), &targets(), None);
    assert_eq!(initial.next.focus, Some(8));
}

#[test]
fn pointer_release_requires_original_target_and_move_back_restores_pressed() {
    let original = Interaction::default();
    let mut state = original
        .reduce(
            &InputEvent::PointerMoved { x: 5, y: 6 },
            &targets(),
            Some(2),
        )
        .next;
    assert_eq!(original, Interaction::default());
    assert_eq!(state.pointer, Some((5, 6)));
    assert!(state.state(2, false, None).hovered());
    assert_eq!(send(&mut state, InputEvent::PointerPressed, Some(2)), None);
    assert_eq!(state.focus, Some(2));
    assert!(state.state(2, false, None).pressed());
    send(
        &mut state,
        InputEvent::PointerMoved { x: 50, y: 60 },
        Some(8),
    );
    assert!(!state.state(2, false, None).pressed());
    assert_eq!(state.pointer_arm, Some(2));
    send(&mut state, InputEvent::PointerMoved { x: 5, y: 6 }, Some(2));
    assert!(state.state(2, false, None).pressed());
    assert_eq!(
        send(&mut state, InputEvent::PointerReleased, Some(2)),
        Some(2)
    );
    assert_eq!(send(&mut state, InputEvent::PointerReleased, Some(2)), None);
    send(&mut state, InputEvent::PointerPressed, Some(2));
    assert_eq!(send(&mut state, InputEvent::PointerReleased, Some(8)), None);
}

#[test]
fn disabled_hit_blocks_without_blur_and_background_clears_focus() {
    let mut state = focused(2);
    send(&mut state, InputEvent::PointerPressed, Some(4));
    assert_eq!(state.focus, Some(2));
    assert_eq!(state.pointer_arm, None);
    assert_eq!(send(&mut state, InputEvent::PointerReleased, Some(4)), None);
    send(&mut state, InputEvent::PointerPressed, None);
    assert_eq!(state.focus, None);
}

#[test]
fn repeated_pointer_down_cannot_transfer_or_start_an_existing_hold() {
    for first in [None, Some(2)] {
        let mut state = Interaction::default();
        send(&mut state, InputEvent::PointerPressed, first);
        send(&mut state, InputEvent::PointerPressed, Some(8));
        assert_eq!(send(&mut state, InputEvent::PointerReleased, Some(8)), None);
        send(&mut state, InputEvent::PointerPressed, Some(8));
        assert_eq!(
            send(&mut state, InputEvent::PointerReleased, Some(8)),
            Some(8)
        );
    }
}

#[test]
fn pointer_leave_cancels_only_pointer_and_keeps_keyboard_activation() {
    let mut state = focused(2);
    send(&mut state, InputEvent::PointerMoved { x: 5, y: 6 }, Some(2));
    send(&mut state, InputEvent::PointerPressed, Some(2));
    key(&mut state, Key::Space, true);
    send(&mut state, InputEvent::PointerLeft, None);
    assert_eq!(
        (state.pointer, state.hover, state.pointer_arm),
        (None, None, None)
    );
    assert_eq!(state.focus, Some(2));
    assert_eq!(key(&mut state, Key::Space, false), Some(2));
    assert_eq!(send(&mut state, InputEvent::PointerReleased, Some(2)), None);
    send(&mut state, InputEvent::PointerPressed, Some(2));
    assert_eq!(
        send(&mut state, InputEvent::PointerReleased, Some(2)),
        Some(2)
    );
}

#[test]
fn disabled_state_keeps_checkbox_value_and_masks_all_interaction_flags() {
    let mut state = focused(8);
    send(&mut state, InputEvent::PointerMoved { x: 5, y: 6 }, Some(8));
    key(&mut state, Key::Space, true);
    let active = state.state(8, false, Some(true));
    assert!(active.focused() && active.hovered() && active.pressed());
    let disabled = state.state(8, true, Some(true));
    assert!(disabled.disabled());
    assert_eq!(disabled.checked(), Some(true));
    assert!(!disabled.focused() && !disabled.hovered() && !disabled.pressed());
}
