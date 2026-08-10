use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{DeviceId, ElementState, MouseButton, WindowEvent};
use winit::window::WindowId;

use super::super::super::shell::mapping::{
    NativeShellInputV1, map_scale_factor_changed_v1, map_window_event_v1,
};
use super::super::super::types::{NativePhysicalExtentV1, NativePhysicalPointV1};

const CURRENT_SIZE: PhysicalSize<u32> = PhysicalSize::new(800, 600);
const CURRENT_SCALE: f64 = 1.25;

#[test]
fn real_winit_resize_and_scale_inputs_sample_the_complementary_window_value() {
    let active = WindowId::from(7);

    assert_eq!(
        map_window_event_v1(
            active,
            active,
            &WindowEvent::Resized(PhysicalSize::new(640, 480)),
            CURRENT_SIZE,
            CURRENT_SCALE,
        ),
        Some(NativeShellInputV1::Surface {
            physical: NativePhysicalExtentV1::new(640, 480),
            scale: 1.25,
        })
    );
    assert_eq!(
        map_scale_factor_changed_v1(active, active, 2.0, CURRENT_SIZE),
        Some(NativeShellInputV1::Surface {
            physical: NativePhysicalExtentV1::new(800, 600),
            scale: 2.0,
        })
    );
}

#[test]
fn real_winit_cursor_left_press_redraw_and_close_map_to_candidate_free_inputs() {
    let active = WindowId::from(9);
    let point = NativePhysicalPointV1::new(4.5, 7.25);
    let cases = [
        (
            WindowEvent::CursorMoved {
                device_id: DeviceId::dummy(),
                position: PhysicalPosition::new(4.5, 7.25),
            },
            Some(NativeShellInputV1::CursorMoved(point)),
        ),
        (
            WindowEvent::MouseInput {
                device_id: DeviceId::dummy(),
                state: ElementState::Pressed,
                button: MouseButton::Left,
            },
            Some(NativeShellInputV1::PrimaryPressed),
        ),
        (
            WindowEvent::RedrawRequested,
            Some(NativeShellInputV1::RedrawRequested),
        ),
        (
            WindowEvent::CloseRequested,
            Some(NativeShellInputV1::CloseRequested),
        ),
    ];

    for (event, expected) in cases {
        assert_eq!(
            map_window_event_v1(active, active, &event, CURRENT_SIZE, CURRENT_SCALE),
            expected
        );
    }

    for event in [
        WindowEvent::MouseInput {
            device_id: DeviceId::dummy(),
            state: ElementState::Released,
            button: MouseButton::Left,
        },
        WindowEvent::MouseInput {
            device_id: DeviceId::dummy(),
            state: ElementState::Pressed,
            button: MouseButton::Right,
        },
        WindowEvent::Moved(PhysicalPosition::new(10, 20)),
    ] {
        assert_eq!(
            map_window_event_v1(active, active, &event, CURRENT_SIZE, CURRENT_SCALE),
            None
        );
    }
}

#[test]
fn every_real_winit_callback_from_a_foreign_window_is_ignored() {
    let active = WindowId::from(1);
    let foreign = WindowId::from(2);
    let events = [
        WindowEvent::Resized(PhysicalSize::new(720, 520)),
        WindowEvent::CursorMoved {
            device_id: DeviceId::dummy(),
            position: PhysicalPosition::new(1.0, 2.0),
        },
        WindowEvent::MouseInput {
            device_id: DeviceId::dummy(),
            state: ElementState::Pressed,
            button: MouseButton::Left,
        },
        WindowEvent::RedrawRequested,
        WindowEvent::CloseRequested,
    ];

    for event in events {
        assert_eq!(
            map_window_event_v1(active, foreign, &event, CURRENT_SIZE, CURRENT_SCALE),
            None
        );
    }
    assert_eq!(
        map_scale_factor_changed_v1(active, foreign, 2.0, CURRENT_SIZE),
        None
    );
}
