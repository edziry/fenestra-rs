use super::*;
use crate::{Color, Element, Style, View};
use crate::{ImeEvent, Key, KeyState, KeyboardInput, Modifiers};

#[test]
fn owned_keyboard_focus_and_ime_events_reach_the_application_handler() {
    let app = Application::new(View::new("test", Element::row("root")), Size::new(80, 80)).unwrap();
    let mut received = Vec::new();
    let mut content = ApplicationWindow {
        app,
        pointer: None,
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
        pointer: None,
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
        pointer: None,
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
