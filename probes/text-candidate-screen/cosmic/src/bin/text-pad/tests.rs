use fenestra_ui::native::{
    ImeEvent, Key, KeyState, KeyboardInput, Modifiers, WindowContent, WindowEvent,
};
use fenestra_ui::{Selection, Size, TextBuffer};

use super::TextPad;

#[path = "tests/background.rs"]
mod background;

fn pad(text: &str, limit: usize) -> TextPad {
    TextPad::new(TextBuffer::new(text, limit).unwrap(), Size::new(720, 520)).unwrap()
}

fn key(pad: &mut TextPad, key: Key, text: Option<&str>, modifiers: Modifiers, repeat: bool) {
    pad.event(WindowEvent::KeyboardInput(KeyboardInput {
        key,
        state: KeyState::Pressed,
        modifiers,
        repeat,
        text: text.map(str::to_owned),
        is_synthetic: false,
    }))
    .unwrap();
}

#[test]
fn repeated_text_and_legacy_space_are_inserted_once_per_real_press() {
    let mut pad = pad("", 40);
    for repeat in [false, true, true] {
        key(
            &mut pad,
            Key::Space,
            Some(" "),
            Modifiers::default(),
            repeat,
        );
        pad.event(WindowEvent::SpacePressed).unwrap();
    }
    assert_eq!(pad.text.text(), "   ");
    for repeat in [false, true] {
        key(&mut pad, Key::Backspace, None, Modifiers::default(), repeat);
    }
    assert_eq!(pad.text.text(), " ");
    key(
        &mut pad,
        Key::Enter,
        Some("\r"),
        Modifiers::default(),
        false,
    );
    assert_eq!(pad.text.text(), " \n");
    for (state, synthetic) in [(KeyState::Released, false), (KeyState::Pressed, true)] {
        pad.event(WindowEvent::KeyboardInput(KeyboardInput {
            key: Key::Backspace,
            state,
            modifiers: Modifiers::default(),
            repeat: false,
            text: Some("ignored".into()),
            is_synthetic: synthetic,
        }))
        .unwrap();
    }
    assert_eq!(pad.text.text(), " \n");
}

#[test]
fn preedit_is_provisional_and_commit_inserts_once() {
    let mut pad = pad("ab", 20);
    pad.text.set_selection(Selection::new(0, 1)).unwrap();
    pad.event(WindowEvent::Ime(ImeEvent::Preedit {
        text: "e\u{301}".into(),
        cursor: Some((1, 2)),
    }))
    .unwrap();
    assert_eq!(pad.text.text(), "ab");
    assert_eq!(pad.display_text().text(), "e\u{301}b");
    key(&mut pad, Key::Backspace, None, Modifiers::default(), false);
    key(
        &mut pad,
        Key::Character("z".into()),
        Some("z"),
        Modifiers::default(),
        false,
    );
    assert_eq!(pad.text.text(), "ab");
    pad.event(WindowEvent::Ime(ImeEvent::Commit("e\u{301}".into())))
        .unwrap();
    assert_eq!(pad.text.text(), "e\u{301}b");
    assert!(pad.preedit.is_none());
    pad.event(WindowEvent::SpacePressed).unwrap();
    assert_eq!(pad.text.text(), "e\u{301}b");
}

#[test]
fn focus_loss_and_empty_updates_cancel_preedit_without_committing() {
    let mut pad = pad("abc", 20);
    pad.text.select_all();
    pad.event(WindowEvent::Ime(ImeEvent::Preedit {
        text: "x".into(),
        cursor: None,
    }))
    .unwrap();
    pad.event(WindowEvent::Focused(false)).unwrap();
    assert!(pad.preedit.is_none());
    pad.event(WindowEvent::Ime(ImeEvent::Commit("late".into())))
        .unwrap();
    key(&mut pad, Key::Delete, None, Modifiers::default(), false);
    assert_eq!(pad.text.text(), "abc");
    pad.event(WindowEvent::Focused(true)).unwrap();
    pad.event(WindowEvent::Ime(ImeEvent::Commit(String::new())))
        .unwrap();
    assert_eq!(pad.text.text(), "abc");
    assert_eq!(pad.text.selected_text(), "abc");
    pad.event(WindowEvent::Ime(ImeEvent::Preedit {
        text: "x".into(),
        cursor: None,
    }))
    .unwrap();
    pad.event(WindowEvent::Ime(ImeEvent::Preedit {
        text: String::new(),
        cursor: None,
    }))
    .unwrap();
    assert!(pad.preedit.is_none());
    assert_eq!(pad.text.text(), "abc");
}

#[test]
fn keyboard_selection_uses_logical_graphemes_and_supports_select_all() {
    let mut pad = pad("ae\u{301}z", 40);
    let shift = Modifiers {
        shift: true,
        ..Modifiers::default()
    };
    key(&mut pad, Key::ArrowLeft, None, shift, false);
    key(&mut pad, Key::ArrowLeft, None, shift, true);
    assert_eq!(pad.text.selected_text(), "e\u{301}z");
    key(&mut pad, Key::ArrowRight, None, Modifiers::default(), false);
    assert_eq!(pad.text.selection(), Selection::caret(5));
    key(&mut pad, Key::Home, None, shift, false);
    assert_eq!(pad.text.selected_text(), "ae\u{301}z");
    key(&mut pad, Key::End, None, Modifiers::default(), false);
    key(
        &mut pad,
        Key::Character("a".into()),
        Some("\u{1}"),
        Modifiers {
            control: true,
            ..Modifiers::default()
        },
        false,
    );
    assert_eq!(pad.text.selected_text(), "ae\u{301}z");
    key(
        &mut pad,
        Key::Character("x".into()),
        Some("x"),
        Modifiers::default(),
        false,
    );
    assert_eq!(pad.text.text(), "x");
}

#[test]
fn byte_budget_errors_are_visible_and_leave_committed_text_unchanged() {
    let mut pad = pad("abc", 3);
    key(
        &mut pad,
        Key::Character("x".into()),
        Some("x"),
        Modifiers::default(),
        true,
    );
    assert_eq!(pad.text.text(), "abc");
    assert!(pad.status_text().contains("limit"));
    pad.event(WindowEvent::Ime(ImeEvent::Preedit {
        text: "x".into(),
        cursor: None,
    }))
    .unwrap();
    assert!(pad.preedit.is_none());
    assert!(pad.status_text().contains("limit"));
    pad.event(WindowEvent::Ime(ImeEvent::Commit("x".into())))
        .unwrap();
    assert_eq!(pad.text.text(), "abc");
    assert!(pad.frame().is_ok());
}

#[test]
fn altgr_reported_text_is_accepted_without_triggering_control_shortcuts() {
    let mut pad = pad("", 8);
    key(
        &mut pad,
        Key::Character("a".into()),
        Some("@"),
        Modifiers {
            control: true,
            alt: true,
            ..Modifiers::default()
        },
        false,
    );
    assert_eq!(pad.text.text(), "@");
}

#[test]
fn pointer_focus_and_caret_placement_use_committed_editor_bounds() {
    let mut pad = pad("abc", 40);
    let bounds = pad.app.bounds("editor").unwrap();
    pad.event(WindowEvent::PointerMoved {
        x: bounds.x() as i32,
        y: bounds.y() as i32 + 4,
    })
    .unwrap();
    pad.event(WindowEvent::PointerPressed).unwrap();
    assert_eq!(pad.text.selection(), Selection::caret(0));
    pad.event(WindowEvent::PointerMoved { x: 0, y: 0 }).unwrap();
    pad.event(WindowEvent::PointerPressed).unwrap();
    key(
        &mut pad,
        Key::Character("x".into()),
        Some("x"),
        Modifiers::default(),
        false,
    );
    assert_eq!(pad.text.text(), "abc");
}

#[test]
fn focus_loss_discards_pointer_position_until_a_fresh_move() {
    let mut pad = pad("abc", 40);
    let bounds = pad.app.bounds("editor").unwrap();
    pad.event(WindowEvent::PointerMoved {
        x: bounds.x() as i32,
        y: bounds.y() as i32 + 4,
    })
    .unwrap();
    pad.event(WindowEvent::Focused(false)).unwrap();
    pad.event(WindowEvent::Focused(true)).unwrap();
    pad.event(WindowEvent::PointerPressed).unwrap();
    assert_eq!(pad.text.selection(), Selection::caret(3));
    assert!(!pad.editor_focused);
    pad.event(WindowEvent::PointerMoved {
        x: bounds.x() as i32,
        y: bounds.y() as i32 + 4,
    })
    .unwrap();
    pad.event(WindowEvent::PointerPressed).unwrap();
    assert_eq!(pad.text.selection(), Selection::caret(0));
    assert!(pad.editor_focused);
}

#[test]
fn resize_reflows_authored_panes_and_pixels_include_clipped_text_ink() {
    let mut pad = pad("visible ink\nsecond line", 100);
    let old_bounds = pad.app.bounds("editor").unwrap();
    let first = pad.frame().unwrap();
    let count = pad.render_count.get();
    pad.event(WindowEvent::PointerMoved { x: 50, y: 60 })
        .unwrap();
    assert_eq!(pad.frame().unwrap(), first);
    assert_eq!(pad.render_count.get(), count);
    pad.resize(520, 360).unwrap();
    let bounds = pad.app.bounds("editor").unwrap();
    assert!(bounds.width() < old_bounds.width());
    assert!(bounds.height() < old_bounds.height());
    let raster = pad.frame().unwrap();
    assert_eq!(raster.size(), Size::new(520, 360));
    let base = pad.app.raster().unwrap();
    let changed = raster
        .bytes()
        .chunks_exact(4)
        .zip(base.bytes().chunks_exact(4))
        .enumerate()
        .filter(|(i, (a, b))| {
            let (x, y) = ((*i % 520) as i64, (*i / 520) as i64);
            x >= bounds.x()
                && x < bounds.x() + i64::from(bounds.width())
                && y >= bounds.y()
                && y < bounds.y() + i64::from(bounds.height())
                && a != b
        })
        .count();
    assert!(changed > 50);
    assert!(raster.bytes().chunks_exact(4).all(|pixel| pixel[3] == 255));
    pad.resize(1, 1).unwrap();
    assert_eq!(pad.frame().unwrap().bytes().len(), 4);
    pad.resize(5000, 1).unwrap();
    assert_eq!(pad.frame().unwrap().bytes().len(), 20_000);
}

#[test]
fn rejected_preedit_still_blocks_editing_commands_until_composition_ends() {
    let mut pad = pad("abc", 3);
    pad.event(WindowEvent::Ime(ImeEvent::Preedit {
        text: "too long".into(),
        cursor: None,
    }))
    .unwrap();
    key(&mut pad, Key::Backspace, None, Modifiers::default(), false);
    assert_eq!(pad.text.text(), "abc");
    pad.event(WindowEvent::Ime(ImeEvent::Disabled)).unwrap();
    key(&mut pad, Key::Backspace, None, Modifiers::default(), false);
    assert_eq!(pad.text.text(), "ab");
}

#[test]
fn overflowing_text_selection_and_caret_leave_pane_padding_untouched() {
    let mut pad = pad(&"Wide selected text\n".repeat(24), 1000);
    pad.text.select_all();
    let frame = pad.frame().unwrap();
    let base = pad.app.raster().unwrap();
    let bounds = ["title", "hint", "editor", "status"].map(|name| pad.app.bounds(name).unwrap());
    for (index, (painted, original)) in frame
        .bytes()
        .chunks_exact(4)
        .zip(base.bytes().chunks_exact(4))
        .enumerate()
    {
        let x = (index % 720) as i64;
        let y = (index / 720) as i64;
        let inside = bounds.iter().any(|rect| {
            x >= rect.x()
                && x < rect.x() + i64::from(rect.width())
                && y >= rect.y()
                && y < rect.y() + i64::from(rect.height())
        });
        if !inside {
            assert_eq!(painted, original, "paint escaped text panes at {x},{y}");
        }
    }
    let generation = pad.app.generation();
    assert!(pad.resize(0, 0).is_err());
    assert_eq!(pad.app.size(), Size::new(720, 520));
    assert_eq!(pad.app.generation(), generation);
    assert_eq!(pad.frame().unwrap(), frame);
}
