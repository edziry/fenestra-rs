use winit::event::{ElementState, Ime, MouseButton, WindowEvent as PlatformEvent};
use winit::keyboard::{Key as PlatformKey, KeyCode, ModifiersState, NamedKey, PhysicalKey};

use super::{ImeEvent, Key, KeyState, KeyboardInput, Modifiers, WindowEvent};

#[cfg(test)]
mod tests;

pub(super) fn requests_redraw(event: &PlatformEvent) -> bool {
    match event {
        PlatformEvent::CursorMoved { .. }
        | PlatformEvent::KeyboardInput { .. }
        | PlatformEvent::ModifiersChanged(_)
        | PlatformEvent::Focused(_)
        | PlatformEvent::Ime(_)
        | PlatformEvent::MouseInput {
            state: ElementState::Pressed,
            button: MouseButton::Left,
            ..
        } => true,
        PlatformEvent::Resized(size) => size.width > 0 && size.height > 0,
        _ => false,
    }
}

pub(super) struct InputState {
    modifiers: Modifiers,
    composing: bool,
    space_pressed: bool,
    focused: bool,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            modifiers: Modifiers::default(),
            composing: false,
            space_pressed: false,
            focused: true,
        }
    }
}

impl InputState {
    pub(super) fn application_events(
        &mut self,
        event: &PlatformEvent,
    ) -> Result<Vec<WindowEvent>, ()> {
        let event = match event {
            PlatformEvent::CursorMoved { position, .. } => WindowEvent::PointerMoved {
                x: pixel_coordinate(position.x)?,
                y: pixel_coordinate(position.y)?,
            },
            PlatformEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => WindowEvent::PointerPressed,
            PlatformEvent::KeyboardInput {
                event,
                is_synthetic,
                ..
            } => {
                return Ok(self.keyboard_event(
                    &event.logical_key,
                    event.text.as_deref(),
                    event.physical_key,
                    event.state,
                    event.repeat,
                    *is_synthetic,
                ));
            }
            PlatformEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifier_snapshot(modifiers.state());
                WindowEvent::ModifiersChanged(self.modifiers)
            }
            PlatformEvent::Focused(focused) => {
                self.focused = *focused;
                if !focused {
                    self.composing = false;
                    self.space_pressed = false;
                    let previous = std::mem::take(&mut self.modifiers);
                    if previous != Modifiers::default() {
                        return Ok(vec![
                            WindowEvent::ModifiersChanged(self.modifiers),
                            WindowEvent::Focused(false),
                        ]);
                    }
                }
                WindowEvent::Focused(*focused)
            }
            PlatformEvent::Ime(ime) => WindowEvent::Ime(self.ime_event(ime)),
            PlatformEvent::CloseRequested => WindowEvent::CloseRequested,
            _ => return Ok(Vec::new()),
        };
        Ok(vec![event])
    }

    fn keyboard_event(
        &mut self,
        key: &PlatformKey,
        text: Option<&str>,
        physical_key: PhysicalKey,
        state: ElementState,
        repeat: bool,
        is_synthetic: bool,
    ) -> Vec<WindowEvent> {
        let pressed = state == ElementState::Pressed;
        let accepts_text = pressed && !is_synthetic && self.focused && !self.composing;
        let mut events = vec![WindowEvent::KeyboardInput(KeyboardInput {
            key: logical_key(key),
            state: if pressed {
                KeyState::Pressed
            } else {
                KeyState::Released
            },
            modifiers: self.modifiers,
            repeat,
            text: text.filter(|_| accepts_text).map(str::to_owned),
            is_synthetic,
        })];
        if physical_key == PhysicalKey::Code(KeyCode::Space) {
            if accepts_text && !repeat && !self.space_pressed {
                events.push(WindowEvent::SpacePressed);
            }
            self.space_pressed = pressed;
        }
        events
    }

    fn ime_event(&mut self, ime: &Ime) -> ImeEvent {
        match ime {
            Ime::Enabled => {
                self.composing = false;
                ImeEvent::Enabled
            }
            Ime::Disabled => {
                self.composing = false;
                ImeEvent::Disabled
            }
            Ime::Preedit(text, cursor) => {
                self.composing = !text.is_empty();
                ImeEvent::Preedit {
                    text: text.clone(),
                    cursor: *cursor,
                }
            }
            Ime::Commit(text) => {
                self.composing = false;
                ImeEvent::Commit(text.clone())
            }
        }
    }
}

fn modifier_snapshot(state: ModifiersState) -> Modifiers {
    Modifiers {
        shift: state.shift_key(),
        control: state.control_key(),
        alt: state.alt_key(),
        super_key: state.super_key(),
    }
}

fn logical_key(key: &PlatformKey) -> Key {
    match key {
        PlatformKey::Character(text) => Key::Character(text.to_string()),
        PlatformKey::Dead(value) => Key::Dead(*value),
        PlatformKey::Named(NamedKey::Space) => Key::Space,
        PlatformKey::Named(NamedKey::Enter) => Key::Enter,
        PlatformKey::Named(NamedKey::Tab) => Key::Tab,
        PlatformKey::Named(NamedKey::Backspace) => Key::Backspace,
        PlatformKey::Named(NamedKey::Delete) => Key::Delete,
        PlatformKey::Named(NamedKey::Escape) => Key::Escape,
        PlatformKey::Named(NamedKey::ArrowLeft) => Key::ArrowLeft,
        PlatformKey::Named(NamedKey::ArrowRight) => Key::ArrowRight,
        PlatformKey::Named(NamedKey::ArrowUp) => Key::ArrowUp,
        PlatformKey::Named(NamedKey::ArrowDown) => Key::ArrowDown,
        PlatformKey::Named(NamedKey::Home) => Key::Home,
        PlatformKey::Named(NamedKey::End) => Key::End,
        PlatformKey::Named(NamedKey::PageUp) => Key::PageUp,
        PlatformKey::Named(NamedKey::PageDown) => Key::PageDown,
        PlatformKey::Named(NamedKey::Insert) => Key::Insert,
        PlatformKey::Named(NamedKey::Shift) => Key::Shift,
        PlatformKey::Named(NamedKey::Control) => Key::Control,
        PlatformKey::Named(NamedKey::Alt) => Key::Alt,
        PlatformKey::Named(NamedKey::Super) => Key::Super,
        _ => Key::Unidentified,
    }
}

fn pixel_coordinate(value: f64) -> Result<i32, ()> {
    if !value.is_finite() || value < f64::from(i32::MIN) || value > f64::from(i32::MAX) {
        return Err(());
    }
    Ok(value as i32)
}
