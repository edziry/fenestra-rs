use fenestra_ui::native::Modifiers;

use super::*;

fn editor(app: &Application) -> Editor {
    Editor {
        buffer: TextBuffer::new(app.text("content").unwrap(), MAX_EDIT_BYTES).unwrap(),
        composing: false,
    }
}

fn press(key: Key, text: Option<&str>) -> Event {
    Event::KeyboardInput(KeyboardInput {
        key,
        state: KeyState::Pressed,
        modifiers: Modifiers::default(),
        repeat: false,
        text: text.map(str::to_owned),
        is_synthetic: false,
    })
}

#[test]
fn preedit_is_provisional_and_commits_do_not_duplicate_keyboard_text() {
    let mut app = crate::application().unwrap();
    let mut editor = editor(&app);
    let original = app.text("content").unwrap().to_owned();
    editor
        .event(
            &mut app,
            Event::Ime(ImeEvent::Preedit {
                text: "e\u{301}".into(),
                cursor: Some((0, 3)),
            }),
        )
        .unwrap();
    editor
        .event(&mut app, press(Key::Character("e".into()), Some("e")))
        .unwrap();
    assert_eq!(app.text("content").unwrap(), original);
    editor
        .event(&mut app, Event::Ime(ImeEvent::Commit("e\u{301}".into())))
        .unwrap();
    assert_eq!(app.text("content").unwrap(), format!("{original}e\u{301}"));
    editor.event(&mut app, Event::SpacePressed).unwrap();
    assert_eq!(editor.buffer.text(), app.text("content").unwrap());
    assert!(!app.text("content").unwrap().ends_with(' '));
}

#[test]
fn backspace_removes_one_grapheme_and_rejected_glyphs_preserve_the_buffer() {
    let mut app = crate::application().unwrap();
    let mut editor = editor(&app);
    let original = app.text("content").unwrap().to_owned();
    editor
        .event(
            &mut app,
            press(Key::Character("e\u{301}".into()), Some("e\u{301}")),
        )
        .unwrap();
    editor.event(&mut app, press(Key::Backspace, None)).unwrap();
    assert_eq!(app.text("content").unwrap(), original);
    editor
        .event(&mut app, Event::Ime(ImeEvent::Commit("\u{1f680}".into())))
        .unwrap();
    assert_eq!(app.text("content").unwrap(), original);
    assert_eq!(editor.buffer.text(), original);
    assert!(app.text("status").unwrap().contains("rejected"));
}
