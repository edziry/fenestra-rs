use super::*;
use crate::{Color, Element, Style, View};
use crate::{ImeEvent, Key, KeyState, KeyboardInput, Modifiers};

#[test]
fn release_hits_current_geometry_and_preserves_legacy_click_on_press() {
    let mut received = Vec::new();
    let mut content = ApplicationWindow {
        app: pointer_application(),
        handler: |_: &mut Application, event| {
            received.push(event);
            Ok(())
        },
    };
    content.event(WindowEvent::PointerReleased).unwrap();
    content
        .event(WindowEvent::PointerMoved { x: 25, y: 4 })
        .unwrap();
    content.event(WindowEvent::PointerPressed).unwrap();
    content.app.set_size("card", 40, 20).unwrap();
    content.event(WindowEvent::PointerReleased).unwrap();
    content.event(WindowEvent::PointerPressed).unwrap();
    content.app.set_size("card", 20, 20).unwrap();
    content.event(WindowEvent::PointerReleased).unwrap();
    assert_eq!(
        received,
        [
            Event::PointerReleased { target: None },
            Event::PointerMoved { x: 25, y: 4 },
            Event::Click { target: None },
            Event::PointerReleased {
                target: Some("card".into())
            },
            Event::Click {
                target: Some("card".into())
            },
            Event::PointerReleased { target: None },
        ]
    );
}

#[test]
fn leaving_or_losing_focus_clears_pointer_before_release_and_next_press() {
    for interruption in [WindowEvent::PointerLeft, WindowEvent::Focused(false)] {
        let mut received = Vec::new();
        let mut content = ApplicationWindow {
            app: pointer_application(),
            handler: |_: &mut Application, event| {
                received.push(event);
                Ok(())
            },
        };
        content
            .event(WindowEvent::PointerMoved { x: 4, y: 4 })
            .unwrap();
        content.event(WindowEvent::PointerPressed).unwrap();
        content.event(interruption.clone()).unwrap();
        content.event(WindowEvent::PointerReleased).unwrap();
        content.event(WindowEvent::PointerPressed).unwrap();
        assert_eq!(
            received[1],
            Event::Click {
                target: Some("card".into())
            }
        );
        assert_eq!(
            received[2],
            match interruption {
                WindowEvent::PointerLeft => Event::PointerLeft,
                _ => Event::Focused(false),
            }
        );
        assert_eq!(received[3], Event::PointerReleased { target: None });
        assert_eq!(received[4], Event::Click { target: None });
    }
}

fn pointer_application() -> Application {
    Application::new(
        View::new(
            "test",
            Element::row("root")
                .child(Element::rect("card").style(Style::new().width(20).height(20).input(true))),
        ),
        Size::new(80, 80),
    )
    .unwrap()
}

#[test]
fn owned_keyboard_focus_and_ime_events_reach_the_application_handler() {
    let app = Application::new(View::new("test", Element::row("root")), Size::new(80, 80)).unwrap();
    let mut received = Vec::new();
    let mut content = ApplicationWindow {
        app,
        handler: |_: &mut Application, event| {
            received.push(event);
            Ok(())
        },
    };
    let key = KeyboardInput {
        key: Key::Character("a".into()),
        state: KeyState::Pressed,
        modifiers: Modifiers {
            shift: true,
            ..Modifiers::default()
        },
        repeat: true,
        text: Some("A".into()),
        is_synthetic: false,
    };
    let preedit = ImeEvent::Preedit {
        text: "\u{e9}".into(),
        cursor: Some((0, 2)),
    };
    for event in [
        WindowEvent::KeyboardInput(key.clone()),
        WindowEvent::ModifiersChanged(key.modifiers),
        WindowEvent::Ime(ImeEvent::Enabled),
        WindowEvent::Ime(preedit.clone()),
        WindowEvent::Ime(ImeEvent::Commit("\u{e9}".into())),
        WindowEvent::Ime(ImeEvent::Disabled),
        WindowEvent::Focused(false),
        WindowEvent::SpacePressed,
    ] {
        content.event(event).unwrap();
    }
    assert_eq!(
        received,
        [
            Event::KeyboardInput(key.clone()),
            Event::ModifiersChanged(key.modifiers),
            Event::Ime(ImeEvent::Enabled),
            Event::Ime(preedit),
            Event::Ime(ImeEvent::Commit("\u{e9}".into())),
            Event::Ime(ImeEvent::Disabled),
            Event::Focused(false),
            Event::SpacePressed,
        ]
    );
}

#[test]
fn focus_loss_discards_the_cached_click_target() {
    let view = View::new(
        "test",
        Element::row("root")
            .child(Element::rect("card").style(Style::new().width(20).height(20).input(true))),
    );
    let mut targets = Vec::new();
    let mut content = ApplicationWindow {
        app: Application::new(view, Size::new(80, 80)).unwrap(),
        handler: |_: &mut Application, event| {
            if let Event::Click { target } = event {
                targets.push(target);
            }
            Ok(())
        },
    };
    for event in [
        WindowEvent::PointerMoved { x: 2, y: 2 },
        WindowEvent::PointerPressed,
        WindowEvent::Focused(false),
        WindowEvent::Focused(true),
        WindowEvent::PointerPressed,
    ] {
        content.event(event).unwrap();
    }
    assert_eq!(targets, [Some("card".into()), None]);
}

#[test]
fn native_clicks_resolve_against_current_layout_and_mutate_named_content() {
    let view = View::new(
        "test",
        Element::row("root")
            .child(Element::rect("card").style(Style::new().width(20).height(20).input(true))),
    );
    let app = Application::new(view, Size::new(80, 80)).unwrap();
    let mut content = ApplicationWindow {
        app,
        handler: |app: &mut Application, event| {
            if let Event::Click { target: Some(name) } = event {
                app.set_background(&name, Color::rgba8(255, 0, 0, 255))?;
            }
            Ok(())
        },
    };
    content.event(WindowEvent::PointerPressed).unwrap();
    assert_eq!(content.app.generation(), 0);
    content
        .event(WindowEvent::PointerMoved { x: 30, y: 4 })
        .unwrap();
    content.app.set_size("card", 40, 20).unwrap();
    content.event(WindowEvent::PointerPressed).unwrap();
    let raster = content.frame().unwrap();
    let start = (4 * 80 + 30) * 4;
    assert_eq!(&raster.bytes()[start..start + 4], &[255, 0, 0, 255]);
}
