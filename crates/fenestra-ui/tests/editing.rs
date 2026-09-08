use fenestra_ui::{EditingError, Selection, TextBuffer};

const ACCENT: &str = "e\u{301}";
const FAMILY: &str = "\u{1f468}\u{200d}\u{1f469}\u{200d}\u{1f467}";

#[test]
fn buffer_owns_text_and_starts_with_a_caret_at_the_end() {
    let source = String::from("hello");
    let buffer = TextBuffer::new(source.clone(), 16).unwrap();
    drop(source);
    assert_eq!(buffer.text(), "hello");
    assert_eq!(buffer.max_bytes(), 16);
    assert_eq!(buffer.selection(), Selection::caret(5));
    assert_eq!(buffer.selected_text(), "");
}

#[test]
fn selection_preserves_direction_and_exposes_an_ordered_range() {
    let selection = Selection::new(8, 2);
    assert_eq!(selection.anchor(), 8);
    assert_eq!(selection.focus(), 2);
    assert_eq!(selection.range(), 2..8);
    assert!(!selection.is_caret());
    assert!(Selection::caret(2).is_caret());
}

#[test]
fn construction_enforces_an_inclusive_utf8_byte_limit() {
    assert_eq!(TextBuffer::new(ACCENT, 3).unwrap().text(), ACCENT);
    assert_eq!(
        TextBuffer::new(ACCENT, 2),
        Err(EditingError::LimitExceeded {
            limit: 2,
            actual: 3,
        })
    );
    assert!(TextBuffer::new("", 0).is_ok());
}

#[test]
fn construction_accepts_borrowed_sources_without_an_owned_conversion() {
    struct BorrowedText<'a>(&'a str);

    impl AsRef<str> for BorrowedText<'_> {
        fn as_ref(&self) -> &str {
            self.0
        }
    }

    assert_eq!(
        TextBuffer::new(BorrowedText(ACCENT), 2),
        Err(EditingError::LimitExceeded {
            limit: 2,
            actual: 3,
        })
    );
    let buffer = TextBuffer::new(BorrowedText(ACCENT), 3).unwrap();
    assert_eq!(buffer.text(), ACCENT);
}

#[test]
fn empty_insertions_preserve_carets_at_nonempty_grapheme_boundaries() {
    let text = format!("{ACCENT}{FAMILY}\r\nx");
    let mut buffer = TextBuffer::new(&text, text.len()).unwrap();
    for offset in [0, 3, 3 + FAMILY.len(), 5 + FAMILY.len(), text.len()] {
        buffer.set_selection(Selection::caret(offset)).unwrap();
        let initial = buffer.clone();
        buffer.replace_selection("").unwrap();
        assert_eq!(buffer, initial);
    }
}

#[test]
fn directed_crlf_selections_replace_the_same_complete_cluster() {
    for selection in [Selection::new(1, 3), Selection::new(3, 1)] {
        let mut buffer = TextBuffer::new("a\r\nb", 4).unwrap();
        buffer.set_selection(selection).unwrap();
        assert_eq!(buffer.selected_text(), "\r\n");
        buffer.replace_selection("\n").unwrap();
        assert_eq!(buffer.text(), "a\nb");
        assert_eq!(buffer.selection(), Selection::caret(2));
    }
}

#[test]
fn invalid_selection_rejects_utf8_and_grapheme_interiors_atomically() {
    let mut buffer = TextBuffer::new(format!("{ACCENT}{FAMILY}\r\nZ"), 64).unwrap();
    buffer.select_all();
    let initial = buffer.clone();
    for offset in [1, 2, 4, 7, 10, 22, usize::MAX] {
        assert!(buffer.set_selection(Selection::new(0, offset)).is_err());
        assert_eq!(buffer, initial);
        assert!(buffer.set_selection(Selection::new(offset, 0)).is_err());
        assert_eq!(buffer, initial);
    }
    let crlf_interior = ACCENT.len() + FAMILY.len() + 1;
    assert!(
        buffer
            .set_selection(Selection::caret(crlf_interior))
            .is_err()
    );
    assert_eq!(buffer, initial);
    let end = buffer.text().len();
    buffer.set_selection(Selection::new(end, 0)).unwrap();
    assert_eq!(buffer.selected_text(), initial.text());
}

#[test]
fn replacement_uses_the_ordered_selection_and_collapses_it() {
    let mut buffer = TextBuffer::new("first second", 32).unwrap();
    buffer.set_selection(Selection::new(12, 6)).unwrap();
    assert_eq!(buffer.selected_text(), "second");
    buffer.replace_selection(ACCENT).unwrap();
    assert_eq!(buffer.text(), "first e\u{301}");
    assert_eq!(buffer.selection(), Selection::caret(9));
}

#[test]
fn oversized_replacement_preserves_text_and_selection() {
    let mut buffer = TextBuffer::new("abcd", 5).unwrap();
    buffer.set_selection(Selection::new(3, 1)).unwrap();
    let initial = buffer.clone();
    assert_eq!(
        buffer.replace_selection("long"),
        Err(EditingError::LimitExceeded {
            limit: 5,
            actual: 6,
        })
    );
    assert_eq!(buffer, initial);
    buffer.replace_selection(ACCENT).unwrap();
    assert_eq!(buffer.text(), "ae\u{301}d");
    assert_eq!(buffer.text().len(), 5);
}

#[test]
fn logical_movement_skips_combining_emoji_and_crlf_clusters() {
    let text = format!("a{ACCENT}{FAMILY}\r\nz");
    let mut buffer = TextBuffer::new(text, 64).unwrap();
    let boundaries = [
        0,
        1,
        4,
        4 + FAMILY.len(),
        6 + FAMILY.len(),
        7 + FAMILY.len(),
    ];
    for offset in boundaries.into_iter().rev().skip(1) {
        buffer.move_left(false);
        assert_eq!(buffer.selection(), Selection::caret(offset));
    }
    buffer.move_left(false);
    assert_eq!(buffer.selection(), Selection::caret(0));
    for offset in boundaries.into_iter().skip(1) {
        buffer.move_right(false);
        assert_eq!(buffer.selection(), Selection::caret(offset));
    }
    buffer.move_right(false);
    assert_eq!(buffer.selection(), Selection::caret(buffer.text().len()));
}

#[test]
fn movement_collapses_a_selection_or_extends_its_focus() {
    let mut buffer = TextBuffer::new("abcd", 8).unwrap();
    for selection in [Selection::new(1, 3), Selection::new(3, 1)] {
        buffer.set_selection(selection).unwrap();
        buffer.move_left(false);
        assert_eq!(buffer.selection(), Selection::caret(1));
        buffer.set_selection(selection).unwrap();
        buffer.move_right(false);
        assert_eq!(buffer.selection(), Selection::caret(3));
    }
    buffer.set_selection(Selection::caret(2)).unwrap();
    buffer.move_left(true);
    assert_eq!(buffer.selection(), Selection::new(2, 1));
    buffer.move_right(true);
    assert_eq!(buffer.selection(), Selection::caret(2));
    buffer.move_right(true);
    assert_eq!(buffer.selection(), Selection::new(2, 3));
    buffer.move_to_start(true);
    assert_eq!(buffer.selection(), Selection::new(2, 0));
    buffer.move_to_end(true);
    assert_eq!(buffer.selection(), Selection::new(2, 4));
    buffer.move_to_start(false);
    assert_eq!(buffer.selection(), Selection::caret(0));
    buffer.move_to_end(false);
    assert_eq!(buffer.selection(), Selection::caret(4));
}

#[test]
fn backspace_removes_whole_graphemes() {
    let mut buffer = TextBuffer::new(format!("a{ACCENT}{FAMILY}\r\n"), 64).unwrap();
    for expected in [
        format!("a{ACCENT}{FAMILY}"),
        format!("a{ACCENT}"),
        "a".into(),
        "".into(),
    ] {
        buffer.backspace();
        assert_eq!(buffer.text(), expected);
        assert_eq!(buffer.selection(), Selection::caret(expected.len()));
    }
    buffer.backspace();
    assert_eq!(buffer.text(), "");
}

#[test]
fn forward_delete_removes_whole_graphemes() {
    let mut buffer = TextBuffer::new(format!("{ACCENT}{FAMILY}\r\nz"), 64).unwrap();
    buffer.move_to_start(false);
    for expected in [
        format!("{FAMILY}\r\nz"),
        "\r\nz".into(),
        "z".into(),
        "".into(),
    ] {
        buffer.delete_forward();
        assert_eq!(buffer.text(), expected);
        assert_eq!(buffer.selection(), Selection::caret(0));
    }
    buffer.delete_forward();
    assert_eq!(buffer.text(), "");
}

#[test]
fn both_delete_directions_replace_nonempty_selections() {
    for backwards in [true, false] {
        let mut buffer = TextBuffer::new("abcd", 8).unwrap();
        buffer.set_selection(Selection::new(3, 1)).unwrap();
        if backwards {
            buffer.backspace();
        } else {
            buffer.delete_forward();
        }
        assert_eq!(buffer.text(), "ad");
        assert_eq!(buffer.selection(), Selection::caret(1));
    }
}

#[test]
fn inserted_text_can_merge_with_neighboring_graphemes() {
    let mut buffer = TextBuffer::new("\u{301}x", 32).unwrap();
    buffer.move_to_start(false);
    buffer.replace_selection("e").unwrap();
    assert_eq!(buffer.text(), "e\u{301}x");
    assert_eq!(buffer.selection(), Selection::caret(3));

    let mut buffer = TextBuffer::new("\u{1f468}\u{1f469}", 32).unwrap();
    buffer.set_selection(Selection::caret(4)).unwrap();
    buffer.replace_selection("\u{200d}").unwrap();
    assert_eq!(buffer.text(), "\u{1f468}\u{200d}\u{1f469}");
    assert_eq!(buffer.selection(), Selection::caret(buffer.text().len()));
    buffer.backspace();
    assert_eq!(buffer.text(), "");
}

#[test]
fn deletion_revalidates_boundaries_when_neighbors_merge() {
    let mut buffer = TextBuffer::new("\ra\n", 8).unwrap();
    buffer.set_selection(Selection::new(1, 2)).unwrap();
    buffer.replace_selection("").unwrap();
    assert_eq!(buffer.text(), "\r\n");
    assert_eq!(buffer.selection(), Selection::caret(2));
    buffer.backspace();
    assert_eq!(buffer.text(), "");

    let mut buffer = TextBuffer::new("\ra\n", 8).unwrap();
    buffer.set_selection(Selection::caret(1)).unwrap();
    buffer.delete_forward();
    assert_eq!(buffer.selection(), Selection::caret(2));
    assert_eq!(buffer.text(), "\r\n");
}

#[test]
fn regional_indicator_pairing_is_recomputed_after_insertion() {
    let mut buffer = TextBuffer::new("\u{1f1fa}\u{1f1f8}\u{1f1e8}", 32).unwrap();
    buffer.move_to_start(false);
    buffer.replace_selection("\u{1f1e6}").unwrap();
    assert_eq!(buffer.selection(), Selection::caret(8));
    buffer.move_right(false);
    assert_eq!(buffer.selection(), Selection::caret(16));
    buffer.move_left(false);
    assert_eq!(buffer.selection(), Selection::caret(8));
}

#[test]
fn empty_buffers_and_zero_limits_are_stable() {
    let mut buffer = TextBuffer::new("", 0).unwrap();
    let initial = buffer.clone();
    buffer.select_all();
    buffer.move_left(false);
    buffer.move_right(true);
    buffer.move_to_start(true);
    buffer.move_to_end(false);
    buffer.backspace();
    buffer.delete_forward();
    buffer.replace_selection("").unwrap();
    assert_eq!(buffer, initial);
    assert!(buffer.set_selection(Selection::caret(1)).is_err());
    assert!(buffer.replace_selection("a").is_err());
    assert_eq!(buffer, initial);
}
