use winit::event::{ElementState, MouseButton, WindowEvent as PlatformEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

use super::WindowEvent;

pub(super) fn requests_redraw(event: &PlatformEvent) -> bool {
    match event {
        PlatformEvent::CursorMoved { .. }
        | PlatformEvent::MouseInput {
            state: ElementState::Pressed,
            button: MouseButton::Left,
            ..
        } => true,
        PlatformEvent::KeyboardInput { event, .. } => {
            keyboard_event(event.physical_key, event.state, event.repeat).is_some()
        }
        PlatformEvent::Resized(size) => size.width > 0 && size.height > 0,
        _ => false,
    }
}

pub(super) fn application_event(event: &PlatformEvent) -> Result<Option<WindowEvent>, ()> {
    match event {
        PlatformEvent::CursorMoved { position, .. } => Ok(Some(WindowEvent::PointerMoved {
            x: pixel_coordinate(position.x)?,
            y: pixel_coordinate(position.y)?,
        })),
        PlatformEvent::MouseInput {
            state: ElementState::Pressed,
            button: MouseButton::Left,
            ..
        } => Ok(Some(WindowEvent::PointerPressed)),
        PlatformEvent::KeyboardInput { event, .. } => Ok(keyboard_event(
            event.physical_key,
            event.state,
            event.repeat,
        )),
        PlatformEvent::CloseRequested => Ok(Some(WindowEvent::CloseRequested)),
        _ => Ok(None),
    }
}

pub(super) fn keyboard_event(
    key: PhysicalKey,
    state: ElementState,
    repeat: bool,
) -> Option<WindowEvent> {
    (key == PhysicalKey::Code(KeyCode::Space) && state == ElementState::Pressed && !repeat)
        .then_some(WindowEvent::SpacePressed)
}

fn pixel_coordinate(value: f64) -> Result<i32, ()> {
    if !value.is_finite() || value < f64::from(i32::MIN) || value > f64::from(i32::MAX) {
        return Err(());
    }
    Ok(value as i32)
}
