use std::cell::Cell;
use std::error::Error;
use std::fmt;

use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{DeviceId, ElementState, Ime, MouseButton, WindowEvent as PlatformEvent};
use winit::keyboard::ModifiersState;

use super::input::requests_redraw;
use super::presentation::copy_pixels;
use super::shell::NativeApplication;
use super::{ImeEvent, Modifiers, NativeError, WindowContent, WindowEvent, WindowOptions};
use crate::{Raster, Size};

#[derive(Debug, Eq, PartialEq)]
struct Rejected;

impl fmt::Display for Rejected {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("content rejected")
    }
}

impl Error for Rejected {}

struct Content {
    size: Size,
    resized: Vec<Size>,
    events: Vec<WindowEvent>,
    frame_calls: Cell<usize>,
    presents: usize,
    reject: bool,
}

impl Default for Content {
    fn default() -> Self {
        Self {
            size: Size::new(192, 128),
            resized: Vec::new(),
            events: Vec::new(),
            frame_calls: Cell::new(0),
            presents: 0,
            reject: false,
        }
    }
}

impl WindowContent for Content {
    type Error = Rejected;

    fn resize(&mut self, width: u32, height: u32) -> Result<(), Self::Error> {
        if self.reject {
            return Err(Rejected);
        }
        self.size = Size::new(width, height);
        self.resized.push(self.size);
        Ok(())
    }

    fn event(&mut self, event: WindowEvent) -> Result<(), Self::Error> {
        if self.reject {
            return Err(Rejected);
        }
        self.events.push(event);
        Ok(())
    }

    fn frame(&self) -> Result<Raster, Self::Error> {
        self.frame_calls.set(self.frame_calls.get() + 1);
        let bytes = vec![0; self.size.width() as usize * self.size.height() as usize * 4];
        Ok(Raster::new(self.size, bytes).expect("valid test raster"))
    }

    fn presented(&mut self) -> Result<(), Self::Error> {
        self.presents += 1;
        Ok(())
    }
}

#[test]
fn ime_is_opt_in_without_changing_other_window_options() {
    let options = WindowOptions::new("Example");
    assert!(!options.ime_allowed);
    let enabled = options.clone().ime_allowed(true);
    assert!(enabled.ime_allowed);
    assert_eq!(enabled.title, options.title);
    assert_eq!(enabled.size, options.size);
    assert_eq!(enabled.smoke, options.smoke);
    assert_eq!(enabled.ime_allowed(false), options);
}

#[test]
fn composition_and_focus_resets_reach_content_and_schedule_frames() {
    let mut content = Content::default();
    let mut application = NativeApplication::new(&mut content, WindowOptions::new("Example"));
    for event in [
        PlatformEvent::ModifiersChanged(ModifiersState::SHIFT.into()),
        PlatformEvent::Ime(Ime::Enabled),
        PlatformEvent::Ime(Ime::Preedit("a".into(), Some((1, 1)))),
        PlatformEvent::Focused(false),
        PlatformEvent::Ime(Ime::Disabled),
    ] {
        assert!(application.process_event(event).unwrap().redraw);
    }
    assert_eq!(
        application.content.events,
        [
            WindowEvent::ModifiersChanged(Modifiers {
                shift: true,
                ..Modifiers::default()
            }),
            WindowEvent::Ime(ImeEvent::Enabled),
            WindowEvent::Ime(ImeEvent::Preedit {
                text: "a".into(),
                cursor: Some((1, 1))
            }),
            WindowEvent::ModifiersChanged(Modifiers::default()),
            WindowEvent::Focused(false),
            WindowEvent::Ime(ImeEvent::Disabled),
        ]
    );
}

#[test]
fn presenting_a_frame_does_not_schedule_another_frame() {
    assert!(!requests_redraw(&PlatformEvent::RedrawRequested));
}

#[test]
fn unrelated_window_events_do_not_schedule_frames() {
    for event in [
        PlatformEvent::Occluded(false),
        PlatformEvent::CloseRequested,
    ] {
        assert!(!requests_redraw(&event), "unexpected redraw for {event:?}");
    }
}

#[test]
fn pointer_interactions_schedule_frames_and_reach_the_application() {
    let mut content = Content::default();
    let mut application = NativeApplication::new(&mut content, WindowOptions::new("Example"));
    let moved = PlatformEvent::CursorMoved {
        device_id: DeviceId::dummy(),
        position: PhysicalPosition::new(4.9, 3.1),
    };
    assert!(requests_redraw(&moved));
    assert!(
        application
            .process_event(moved)
            .expect("move accepted")
            .redraw
    );
    for (state, button, redraw) in [
        (ElementState::Pressed, MouseButton::Left, true),
        (ElementState::Released, MouseButton::Left, true),
        (ElementState::Pressed, MouseButton::Right, false),
        (ElementState::Released, MouseButton::Right, false),
    ] {
        let event = PlatformEvent::MouseInput {
            device_id: DeviceId::dummy(),
            state,
            button,
        };
        assert_eq!(requests_redraw(&event), redraw);
        assert_eq!(
            application
                .process_event(event)
                .expect("input accepted")
                .redraw,
            redraw
        );
    }
    let left = PlatformEvent::CursorLeft {
        device_id: DeviceId::dummy(),
    };
    assert!(requests_redraw(&left));
    assert!(application.process_event(left).unwrap().redraw);
    assert_eq!(
        application.content.events,
        [
            WindowEvent::PointerMoved { x: 4, y: 3 },
            WindowEvent::PointerPressed,
            WindowEvent::PointerReleased,
            WindowEvent::PointerLeft,
        ]
    );
}

#[test]
fn minimized_window_skips_presentation_and_resize_until_restored() {
    let mut content = Content::default();
    let mut application = NativeApplication::new(&mut content, WindowOptions::new("Example"));
    application.resize_content(192, 128).expect("initial size");
    for (width, height) in [(0, 128), (192, 0), (0, 0)] {
        let outcome = application
            .process_event(PlatformEvent::Resized(PhysicalSize::new(width, height)))
            .expect("minimize");
        assert!(!outcome.redraw);
        application
            .process_event(PlatformEvent::RedrawRequested)
            .expect("zero size does not access the presenter");
        assert!(!application.presented);
    }
    assert_eq!(application.content.resized, [Size::new(192, 128)]);
    assert_eq!(application.content.frame_calls.get(), 0);
    let outcome = application
        .process_event(PlatformEvent::Resized(PhysicalSize::new(224, 160)))
        .expect("restore");
    assert!(outcome.redraw);
    assert!(application.drawable);
    assert_eq!(application.content.size, Size::new(224, 160));
}

#[test]
fn closure_reaches_the_retained_application_without_requesting_a_frame() {
    let mut content = Content::default();
    {
        let mut application = NativeApplication::new(&mut content, WindowOptions::new("Example"));
        let outcome = application
            .process_event(PlatformEvent::CloseRequested)
            .expect("close");
        assert!(outcome.close);
        assert!(!outcome.redraw);
    }
    assert_eq!(content.events, [WindowEvent::CloseRequested]);
    assert_eq!(content.presents, 0);
}

#[test]
fn rejected_callback_preserves_the_application_error() {
    let mut content = Content {
        reject: true,
        ..Content::default()
    };
    let mut application = NativeApplication::new(&mut content, WindowOptions::new("Example"));
    let error = application
        .process_event(PlatformEvent::CloseRequested)
        .expect_err("callback rejects");
    assert!(matches!(error, NativeError::Application(Rejected)));
    assert_eq!(
        error.source().expect("original error").to_string(),
        "content rejected"
    );
    assert!(application.content.events.is_empty());
}

#[test]
fn failed_resize_does_not_publish_a_new_drawable_size() {
    let mut content = Content::default();
    let mut application = NativeApplication::new(&mut content, WindowOptions::new("Example"));
    application.resize_content(192, 128).expect("initial size");
    application.content.reject = true;
    assert!(matches!(
        application.resize_content(224, 160),
        Err(NativeError::Application(Rejected))
    ));
    assert_eq!(application.size, Some(Size::new(192, 128)));
    assert_eq!(application.content.size, Size::new(192, 128));
}

#[test]
fn pointer_coordinates_use_the_containing_pixel_on_both_sides_of_zero() {
    let mut content = Content::default();
    let mut application = NativeApplication::new(&mut content, WindowOptions::new("Example"));
    for (x, y) in [(-0.5, -1.1), (0.5, 1.9), (-1.0, 0.0)] {
        application
            .process_event(PlatformEvent::CursorMoved {
                device_id: DeviceId::dummy(),
                position: PhysicalPosition::new(x, y),
            })
            .unwrap();
    }
    assert_eq!(
        application.content.events,
        [
            WindowEvent::PointerMoved { x: -1, y: -2 },
            WindowEvent::PointerMoved { x: 0, y: 1 },
            WindowEvent::PointerMoved { x: -1, y: 0 },
        ]
    );
}

#[test]
fn invalid_pointer_coordinates_never_invoke_application_callbacks() {
    let mut content = Content::default();
    let mut application = NativeApplication::new(&mut content, WindowOptions::new("Example"));
    for value in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::from(i32::MIN) - 0.5,
        f64::from(i32::MAX) + 1.0,
    ] {
        let event = PlatformEvent::CursorMoved {
            device_id: DeviceId::dummy(),
            position: PhysicalPosition::new(value, 0.0),
        };
        assert!(matches!(
            application.process_event(event),
            Err(NativeError::Presenter)
        ));
    }
    assert!(application.content.events.is_empty());
}

#[test]
fn raster_pixels_keep_rgb_channel_order_and_do_not_partially_copy_short_buffers() {
    let raster = Raster::new(Size::new(2, 1), vec![255, 192, 32, 255, 48, 128, 192, 255])
        .expect("valid pixels");
    let mut pixels = [0; 2];
    copy_pixels(&raster, &mut pixels).expect("matching buffer");
    assert_eq!(pixels, [0xffc020, 0x3080c0]);
    let mut short = [99];
    assert!(copy_pixels(&raster, &mut short).is_err());
    assert_eq!(short, [99]);
}
