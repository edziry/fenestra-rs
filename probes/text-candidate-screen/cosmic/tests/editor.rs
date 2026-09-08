use fenestra_text_screen_cosmic::editor;
use fenestra_ui::{Selection, TextBuffer};

#[test]
fn caret_and_selection_use_absolute_offsets_across_hard_lines() {
    let mut buffer = TextBuffer::new("one\ntwo\nthree", 100).unwrap();
    buffer.set_selection(Selection::new(4, 7)).unwrap();
    let geometry = editor::geometry(&buffer, 320, 20.0, 28.0);
    assert_eq!(geometry.caret.y, 28.0);
    assert!(geometry.caret.x > 20.0);
    assert!(!geometry.highlights.is_empty());
    assert!(geometry.highlights.iter().all(|rect| rect.y == 28.0));
}

#[test]
fn empty_text_and_trailing_newline_have_visible_caret_geometry() {
    for (text, expected_y) in [("", 0.0), ("a\n", 28.0), ("a\r\n", 28.0)] {
        let buffer = TextBuffer::new(text, 100).unwrap();
        let geometry = editor::geometry(&buffer, 320, 20.0, 28.0);
        assert_eq!((geometry.caret.x, geometry.caret.y), (0.0, expected_y));
        assert!(geometry.highlights.is_empty());
    }
}

#[test]
fn pointer_hits_always_return_a_valid_extended_grapheme_boundary() {
    let buffer = TextBuffer::new("a\u{301} \u{1f469}\u{200d}\u{1f4bb} end\nnext", 100).unwrap();
    for x in [-10.0, 0.0, 5.0, 12.0, 30.0, 90.0, 500.0] {
        for y in [-10.0, 5.0, 30.0, 100.0] {
            let hit = editor::hit(&buffer, 320, 20.0, 28.0, x, y);
            let mut checked = buffer.clone();
            checked.set_selection(Selection::caret(hit)).unwrap();
        }
    }
}

#[test]
fn crlf_blank_lines_and_trailing_lines_preserve_absolute_hit_offsets() {
    let mut buffer = TextBuffer::new("a\r\n\r\nb\n", 100).unwrap();
    for (offset, expected_y) in [(0, 0.0), (3, 28.0), (5, 56.0), (7, 84.0)] {
        buffer.set_selection(Selection::caret(offset)).unwrap();
        let geometry = editor::geometry(&buffer, 320, 20.0, 28.0);
        assert_eq!((geometry.caret.x, geometry.caret.y), (0.0, expected_y));
        assert_eq!(
            editor::hit(&buffer, 320, 20.0, 28.0, 0.0, expected_y + 5.0),
            offset
        );
    }
}

#[test]
fn legacy_lfcr_interior_grapheme_boundary_maps_to_following_line() {
    let mut buffer = TextBuffer::new("a\n\rb", 100).unwrap();
    // LF and CR are separate graphemes, while the candidate treats LFCR as
    // one line ending. Both legal caret offsets map to the following line.
    for offset in [2, 3] {
        buffer.set_selection(Selection::caret(offset)).unwrap();
        let geometry = editor::geometry(&buffer, 320, 20.0, 28.0);
        assert_eq!((geometry.caret.x, geometry.caret.y), (0.0, 28.0));
    }
    assert_eq!(buffer.text(), "a\n\rb");
}

#[test]
fn rtl_ligature_partial_selection_highlights_the_correct_visual_half() {
    // DejaVu Sans shapes Arabic lam + alef as one RTL ligature. The first
    // logical grapheme occupies the right half, and the second the left.
    let mut buffer = TextBuffer::new("\u{644}\u{627}", 100).unwrap();
    buffer.set_selection(Selection::caret(0)).unwrap();
    let right = editor::geometry(&buffer, 320, 20.0, 28.0).caret.x;
    buffer.set_selection(Selection::caret(2)).unwrap();
    let middle = editor::geometry(&buffer, 320, 20.0, 28.0).caret.x;
    buffer.set_selection(Selection::caret(4)).unwrap();
    let left = editor::geometry(&buffer, 320, 20.0, 28.0).caret.x;
    assert!(left < middle && middle < right);
    for (anchor, focus, expected_left, expected_right) in [
        (0, 2, middle, right),
        (2, 0, middle, right),
        (2, 4, left, middle),
    ] {
        buffer.set_selection(Selection::new(anchor, focus)).unwrap();
        let geometry = editor::geometry(&buffer, 320, 20.0, 28.0);
        assert_eq!(geometry.highlights.len(), 1);
        let rect = geometry.highlights[0];
        assert!((rect.x - expected_left).abs() < 0.01, "{rect:?}");
        assert!(
            (rect.x + rect.width - expected_right).abs() < 0.01,
            "{rect:?}"
        );
    }
}

#[test]
fn rtl_ligature_hits_choose_the_nearest_logical_grapheme_boundary() {
    let mut buffer = TextBuffer::new("\u{644}\u{627}", 100).unwrap();
    buffer.set_selection(Selection::caret(0)).unwrap();
    let right = editor::geometry(&buffer, 320, 20.0, 28.0).caret.x;
    buffer.set_selection(Selection::caret(4)).unwrap();
    let left = editor::geometry(&buffer, 320, 20.0, 28.0).caret.x;
    let advance = right - left;
    for (fraction, expected) in [(0.1, 4), (0.4, 2), (0.6, 2), (0.9, 0)] {
        let x = left + advance * fraction;
        assert_eq!(editor::hit(&buffer, 320, 20.0, 28.0, x, 5.0), expected);
    }
}

#[test]
fn selected_paragraphs_exclude_unselected_rows_and_keep_reverse_focus() {
    let mut buffer =
        TextBuffer::new("before\r\nabc \u{5d0}\u{5d1}\u{5d2} xyz\r\nafter", 100).unwrap();
    let start = "before\r\n".len();
    let end = "before\r\nabc \u{5d0}\u{5d1}\u{5d2} xyz".len();
    buffer.set_selection(Selection::new(end, start)).unwrap();
    let geometry = editor::geometry(&buffer, 320, 20.0, 28.0);
    assert_eq!((geometry.caret.x, geometry.caret.y), (0.0, 28.0));
    assert!(!geometry.highlights.is_empty());
    assert!(geometry.highlights.iter().all(|rect| rect.y == 28.0));
}

#[test]
fn mixed_bidi_selection_keeps_visually_unselected_glyphs_between_spans() {
    let mut buffer = TextBuffer::new("abc \u{5d0}\u{5d1}\u{5d2} xyz", 100).unwrap();
    buffer.set_selection(Selection::new(1, 6)).unwrap();
    let geometry = editor::geometry(&buffer, 320, 20.0, 28.0);
    assert_eq!(geometry.highlights.len(), 2);
    let left = geometry.highlights[0];
    let right = geometry.highlights[1];
    assert!(left.x + left.width < right.x);
    assert_eq!((left.y, right.y), (0.0, 0.0));
}

#[test]
fn a_hit_at_the_start_of_a_soft_wrapped_line_keeps_its_visual_row() {
    let mut buffer = TextBuffer::new("abcdefghij", 100).unwrap();
    let offset = editor::hit(&buffer, 40, 20.0, 28.0, 0.0, 33.0);
    assert!(offset > 0 && offset < buffer.text().len());
    buffer.set_selection(Selection::caret(offset)).unwrap();
    let geometry = editor::geometry(&buffer, 40, 20.0, 28.0);
    assert_eq!((geometry.caret.x, geometry.caret.y), (0.0, 28.0));
}
