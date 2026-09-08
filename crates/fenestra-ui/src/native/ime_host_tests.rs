use std::cell::RefCell;

use accesskit::{Action, ActionRequest, NodeId, TreeId};
use accesskit_winit::WindowEvent as AccessibilityEvent;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent as PlatformEvent;

use super::shell::NativeApplication;
use super::{ImeContext, NativeError, WindowContent, WindowEvent, WindowOptions};
use crate::{
    AccessibilityActionRequest, AccessibilityTree, Application, Element, Error, Raster, Size,
    Style, View,
};

struct Content {
    app: Application,
    calls: RefCell<Vec<&'static str>>,
    reject: Option<&'static str>,
}

impl Content {
    fn new() -> Self {
        Self {
            app: Application::new(
                View::new(
                    "test",
                    Element::button("editor", "Editor").style(Style::new().width(20).height(20)),
                ),
                Size::new(40, 40),
            )
            .unwrap(),
            calls: RefCell::default(),
            reject: None,
        }
    }

    fn called(&self, name: &'static str) -> Result<(), Error> {
        self.calls.borrow_mut().push(name);
        if self.reject == Some(name) {
            Err(Error::CapacityOverflow)
        } else {
            Ok(())
        }
    }
}

impl WindowContent for Content {
    type Error = Error;

    fn resize(&mut self, width: u32, height: u32) -> Result<(), Error> {
        self.called("resize")?;
        self.app.resize(Size::new(width, height))
    }

    fn event(&mut self, _event: WindowEvent) -> Result<(), Error> {
        self.called("event")
    }

    fn frame(&self) -> Result<Raster, Error> {
        self.called("frame")?;
        self.app.raster()
    }

    fn presented(&mut self) -> Result<(), Error> {
        self.called("presented")
    }

    fn ime_context(&self) -> Result<Option<ImeContext>, Error> {
        self.called("ime")?;
        Ok(Some(ImeContext::active(1, 2, 2, 1, 18).unwrap()))
    }

    fn accessibility(&self) -> Result<Option<AccessibilityTree>, Error> {
        self.app.accessibility_tree().map(Some)
    }

    fn accessibility_action(&mut self, request: AccessibilityActionRequest) -> Result<(), Error> {
        self.called("action")?;
        self.app.dispatch_accessibility_action(request).map(|_| ())
    }
}

#[test]
fn desired_context_is_read_after_successful_resize_input_and_minimization() {
    let mut content = Content::new();
    let mut host = NativeApplication::new(&mut content, WindowOptions::new("Test"));
    host.process_event(PlatformEvent::Resized(PhysicalSize::new(60, 40)))
        .unwrap();
    host.process_event(PlatformEvent::Focused(false)).unwrap();
    host.process_event(PlatformEvent::Resized(PhysicalSize::new(0, 0)))
        .unwrap();
    assert_eq!(
        *host.content.calls.borrow(),
        ["resize", "ime", "event", "ime", "ime"]
    );
}

#[test]
fn failed_lifecycle_callbacks_do_not_read_a_new_context() {
    for (failure, event) in [
        ("resize", PlatformEvent::Resized(PhysicalSize::new(60, 40))),
        ("event", PlatformEvent::Focused(false)),
    ] {
        let mut content = Content::new();
        content.reject = Some(failure);
        let mut host = NativeApplication::new(&mut content, WindowOptions::new("Test"));
        assert!(matches!(
            host.process_event(event),
            Err(NativeError::Application(Error::CapacityOverflow))
        ));
        assert_eq!(*host.content.calls.borrow(), [failure]);
    }
}

#[test]
fn context_failure_preserves_the_original_callback_error() {
    let mut content = Content::new();
    content.reject = Some("ime");
    let mut host = NativeApplication::new(&mut content, WindowOptions::new("Test"));
    assert!(matches!(
        host.process_event(PlatformEvent::Focused(true)),
        Err(NativeError::Application(Error::CapacityOverflow))
    ));
    assert_eq!(*host.content.calls.borrow(), ["event", "ime"]);
}

#[test]
fn context_is_read_only_after_a_successful_post_presentation_callback() {
    for reject in [None, Some("presented")] {
        let mut content = Content::new();
        content.reject = reject;
        let mut host = NativeApplication::new(&mut content, WindowOptions::new("Test"));
        let result = host.presentation_completed();
        assert!(host.presented);
        if reject.is_some() {
            assert!(matches!(
                result,
                Err(NativeError::Application(Error::CapacityOverflow))
            ));
            assert_eq!(*host.content.calls.borrow(), ["presented"]);
        } else {
            result.unwrap();
            assert_eq!(*host.content.calls.borrow(), ["presented", "ime"]);
        }
    }
}

fn action(action: Action) -> AccessibilityEvent {
    AccessibilityEvent::ActionRequested(ActionRequest {
        action,
        target_tree: TreeId::ROOT,
        target_node: NodeId(1),
        data: None,
    })
}

#[test]
fn only_accepted_accessibility_actions_refresh_the_context() {
    let mut content = Content::new();
    let mut host = NativeApplication::new(&mut content, WindowOptions::new("Test"));
    assert!(!host.accessibility_event(action(Action::Blur)).unwrap());
    assert!(host.content.calls.borrow().is_empty());
    assert!(host.accessibility_event(action(Action::Focus)).unwrap());
    assert_eq!(*host.content.calls.borrow(), ["action", "ime"]);
    host.content.calls.borrow_mut().clear();
    host.content.reject = Some("action");
    assert!(host.accessibility_event(action(Action::Click)).is_err());
    assert_eq!(*host.content.calls.borrow(), ["action"]);
}
