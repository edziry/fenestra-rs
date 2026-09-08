use std::cell::Cell;

use accesskit::{Action, ActionRequest, NodeId, TreeId};
use accesskit_winit::WindowEvent as AccessibilityEvent;

use super::shell::NativeApplication;
use super::{NativeError, WindowContent, WindowEvent, WindowOptions};
use crate::{
    AccessibilityActionRequest, AccessibilityTree, Application, Element, Error, InputEvent, Raster,
    Size, Style, View,
};

struct Content {
    app: Application,
    reads: Cell<usize>,
    actions: Vec<AccessibilityActionRequest>,
    reject: bool,
    reject_action: bool,
    close: bool,
}

impl Content {
    fn new() -> Self {
        Self {
            app: Application::new(
                View::new(
                    "controls",
                    Element::column("root")
                        .style(Style::new().width(80).height(60))
                        .child(
                            Element::button("save", "Save")
                                .style(Style::new().width(40).height(20)),
                        )
                        .child(
                            Element::checkbox("auto", "Automatic")
                                .style(Style::new().width(40).height(20)),
                        ),
                ),
                Size::new(80, 60),
            )
            .unwrap(),
            reads: Cell::new(0),
            actions: Vec::new(),
            reject: false,
            reject_action: false,
            close: false,
        }
    }
}

impl WindowContent for Content {
    type Error = Error;
    fn resize(&mut self, width: u32, height: u32) -> Result<(), Error> {
        self.app.resize(Size::new(width, height))
    }
    fn event(&mut self, event: WindowEvent) -> Result<(), Error> {
        self.app.dispatch_input(event).map(|_| ())
    }
    fn frame(&self) -> Result<Raster, Error> {
        self.app.raster()
    }
    fn accessibility(&self) -> Result<Option<AccessibilityTree>, Error> {
        self.reads.set(self.reads.get() + 1);
        if self.reject {
            return Err(Error::CapacityOverflow);
        }
        self.app.accessibility_tree().map(Some)
    }
    fn accessibility_action(&mut self, request: AccessibilityActionRequest) -> Result<(), Error> {
        if self.reject_action {
            return Err(Error::CapacityOverflow);
        }
        self.actions.push(request);
        self.app.dispatch_accessibility_action(request).map(|_| ())
    }
    fn should_close(&self) -> bool {
        self.close
    }
}

fn request(action: Action, target: u64) -> AccessibilityEvent {
    AccessibilityEvent::ActionRequested(ActionRequest {
        action,
        target_tree: TreeId::ROOT,
        target_node: NodeId(target),
        data: None,
    })
}

#[test]
fn unsupported_platform_actions_do_not_query_or_invoke_application_callbacks() {
    let mut content = Content::new();
    content.reject = true;
    let mut host = NativeApplication::new(&mut content, WindowOptions::new("Test"));
    assert!(!host.accessibility_event(request(Action::Blur, 2)).unwrap());
    assert_eq!(host.content.reads.get(), 0);
    assert!(host.content.actions.is_empty());
}

#[test]
fn native_actions_use_current_eligibility_without_forging_device_input() {
    let mut content = Content::new();
    content
        .app
        .dispatch_input(InputEvent::Focused(false))
        .unwrap();
    let mut host = NativeApplication::new(&mut content, WindowOptions::new("Test"));
    assert!(host.accessibility_event(request(Action::Focus, 2)).unwrap());
    assert_eq!(host.content.app.focused_control(), Some("save"));
    assert!(host.accessibility_event(request(Action::Click, 3)).unwrap());
    assert_eq!(
        host.content
            .app
            .control_snapshot("auto")
            .unwrap()
            .state()
            .checked(),
        Some(true)
    );
    assert_eq!(host.content.app.focused_control(), Some("save"));
    host.content.app.set_disabled("auto", true).unwrap();
    let before = host.content.app.accessibility_tree().unwrap();
    for target in [0, 1, 3, 100, u64::MAX] {
        assert!(
            !host
                .accessibility_event(request(Action::Click, target))
                .unwrap()
        );
    }
    assert_eq!(host.content.actions.len(), 2);
    assert_eq!(host.content.app.accessibility_tree().unwrap(), before);
    host.content.app.resize(Size::new(80, 10)).unwrap();
    host.content.app.set_disabled("auto", false).unwrap();
    assert!(!host.accessibility_event(request(Action::Focus, 3)).unwrap());
    assert_eq!(host.content.actions.len(), 2);
}

#[test]
fn semantic_snapshot_failures_preserve_original_error_and_do_not_invoke_actions() {
    let mut content = Content::new();
    let before = content.app.accessibility_tree().unwrap();
    content.reject = true;
    let mut host = NativeApplication::new(&mut content, WindowOptions::new("Test"));
    assert!(matches!(
        host.accessibility_event(request(Action::Click, 2)),
        Err(NativeError::Application(Error::CapacityOverflow))
    ));
    assert!(host.content.actions.is_empty());
    assert_eq!(host.content.app.accessibility_tree().unwrap(), before);
}

#[test]
fn application_requested_closure_does_not_forge_a_close_or_input_event() {
    let mut content = Content::new();
    let before = content.app.accessibility_tree().unwrap();
    let mut host = NativeApplication::new(&mut content, WindowOptions::new("Test"));
    assert!(!host.should_exit());
    host.presented = true;
    assert!(!host.should_exit());
    host.content.close = true;
    assert!(host.should_exit());
    assert_eq!(host.content.app.accessibility_tree().unwrap(), before);
    assert!(host.content.actions.is_empty());
    assert_eq!(host.content.reads.get(), 0);
}

#[test]
fn rejected_semantic_action_preserves_content_and_original_application_error() {
    let mut content = Content::new();
    content.reject_action = true;
    let before = content.app.accessibility_tree().unwrap();
    let mut host = NativeApplication::new(&mut content, WindowOptions::new("Test"));
    assert!(matches!(
        host.accessibility_event(request(Action::Click, 3)),
        Err(NativeError::Application(Error::CapacityOverflow))
    ));
    assert_eq!(host.content.app.accessibility_tree().unwrap(), before);
    assert!(host.content.actions.is_empty());
}
