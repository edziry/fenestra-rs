use winit::dpi::PhysicalSize;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::window::WindowId;

use super::super::types::{NativePhysicalExtentV1, NativePhysicalPointV1};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum NativeShellInputV1 {
    Surface {
        physical: NativePhysicalExtentV1,
        scale: f64,
    },
    CursorMoved(NativePhysicalPointV1),
    PrimaryPressed,
    RedrawRequested,
    CloseRequested,
}

pub(crate) fn map_window_event_v1(
    active: WindowId,
    observed: WindowId,
    event: &WindowEvent,
    current_size: PhysicalSize<u32>,
    current_scale: f64,
) -> Option<NativeShellInputV1> {
    if observed != active {
        return None;
    }
    match event {
        WindowEvent::Resized(size) => Some(surface(*size, current_scale)),
        WindowEvent::CursorMoved { position, .. } => Some(NativeShellInputV1::CursorMoved(
            NativePhysicalPointV1::new(position.x, position.y),
        )),
        WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: MouseButton::Left,
            ..
        } => Some(NativeShellInputV1::PrimaryPressed),
        WindowEvent::RedrawRequested => Some(NativeShellInputV1::RedrawRequested),
        WindowEvent::CloseRequested => Some(NativeShellInputV1::CloseRequested),
        WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
            Some(surface(current_size, *scale_factor))
        }
        _ => None,
    }
}

#[cfg(test)]
pub(crate) fn map_scale_factor_changed_v1(
    active: WindowId,
    observed: WindowId,
    scale: f64,
    current_size: PhysicalSize<u32>,
) -> Option<NativeShellInputV1> {
    (active == observed).then(|| surface(current_size, scale))
}

const fn surface(size: PhysicalSize<u32>, scale: f64) -> NativeShellInputV1 {
    NativeShellInputV1::Surface {
        physical: NativePhysicalExtentV1::new(size.width, size.height),
        scale,
    }
}
